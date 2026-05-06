#![allow(dead_code)]
// Bat_OS — Chromium → virtio-gpu blit bridge.
//
// Kernel side of the Chromium display handoff. A patched
// SoftwareOutputDeviceHeadless in content_shell mmap()s /batos/fb0, writes
// BGRA pixels into it, and bumps the shared `seq` counter. This kthread
// polls that counter at ~60 Hz and, when it changes, copies the damage rect
// into the virtio-gpu scanout framebuffer and asks the GPU to flush.
//
// Contract: ports/chromium_port/PHASE5_DISPLAY.md §4.
//
// Layout of the shared region (kept in sync with
// src/batcave/linux/vfs.rs::create_chromium_fb and the Chromium patch in
// ports/chromium_port/patches/005-ozone-batos-shm.patch):
//
//   offset 0   u32  magic          = 'BFB1' (0x42464231)
//   offset 4   u32  version        = 1
//   offset 8   u32  width
//   offset 12  u32  height
//   offset 16  u32  stride         (bytes per row)
//   offset 20  u32  format         (1 = BGRA8888 premul)
//   offset 24  u32  seq            (producer-incremented)
//   offset 28  u32  last_seen_seq  (kernel ack — observability only)
//   offset 32  u32  damage_x
//   offset 36  u32  damage_y
//   offset 40  u32  damage_w
//   offset 44  u32  damage_h
//   offset 48  u64  pts_ns
//   offset 56  u32[8] reserved
//   offset 128 u8[w*stride] pixel bytes

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use crate::batcave::linux::vfs;
use crate::drivers::uart;
use crate::drivers::virtio::gpu;
use crate::kernel::process;
use crate::kernel::scheduler;

pub const MAGIC: u32 = 0x4246_4231; // 'BFB1'
pub const HDR_SIZE: usize = 128;

/// Statistics reported by the blit kthread.
#[derive(Clone, Copy, Default)]
pub struct BlitStats {
    pub frames_blitted: u64,
    pub frames_dropped: u64, // seq jumped by >1 between polls (we lagged)
    pub last_seq: u32,
    pub ticks: u64,          // total poll iterations (rough FPS estimator)
    pub running: bool,
}

// ── Globals ─────────────────────────────────────────────────────────────
static FRAMES_BLITTED: AtomicU64 = AtomicU64::new(0);
static FRAMES_DROPPED: AtomicU64 = AtomicU64::new(0);
static LAST_SEQ: AtomicU32 = AtomicU32::new(0);
static TICKS: AtomicU64 = AtomicU64::new(0);
static RUNNING: AtomicBool = AtomicBool::new(false);
static STOP_FLAG: AtomicBool = AtomicBool::new(false);

/// Snapshot the current stats.
pub fn stats() -> BlitStats {
    BlitStats {
        frames_blitted: FRAMES_BLITTED.load(Ordering::Relaxed),
        frames_dropped: FRAMES_DROPPED.load(Ordering::Relaxed),
        last_seq: LAST_SEQ.load(Ordering::Relaxed),
        ticks: TICKS.load(Ordering::Relaxed),
        running: RUNNING.load(Ordering::Relaxed),
    }
}

/// Ask the kthread to stop (cooperative; it will exit at its next poll).
pub fn stop() {
    STOP_FLAG.store(true, Ordering::Release);
}

/// V8-ROOT-2: stop the chromium blit on cave switch and clear sequence
/// counters so the next cave's framebuffer doesn't observe stale frame
/// numbers (which would let it leak frame-rate / activity timing of the
/// previous cave).
pub fn reset_for_cave_switch() {
    STOP_FLAG.store(true, Ordering::Release);
    RUNNING.store(false, Ordering::Release);
    LAST_SEQ.store(0, Ordering::Release);
    FRAMES_BLITTED.store(0, Ordering::Release);
    FRAMES_DROPPED.store(0, Ordering::Release);
    TICKS.store(0, Ordering::Release);
    // V11-FRESH-EYES: the /batos/fb0 region is a physical shared frame
    // wired into every cave's VFS. Without zeroing it, cave A can write
    // secrets to the framebuffer, destroy, and cave B can mmap /batos/fb0
    // to read them back — a 5 MiB residual-plaintext cross-cave leak.
    let (base, size) = vfs::chromium_fb_region();
    if base != 0 && size > 0 {
        unsafe {
            let p = base as *mut u8;
            for i in 0..size {
                core::ptr::write_volatile(p.add(i), 0);
            }
        }
    }
}

/// Spawn the 60-Hz blit kthread. Returns true if newly started.
///
/// Safe to call multiple times — subsequent calls are no-ops once the
/// thread is running. The /batos/fb0 region must already exist (created
/// by VFS init); if absent, we log and refuse to start.
pub fn start() -> bool {
    if RUNNING.load(Ordering::Acquire) {
        return false;
    }

    let (base, size) = vfs::chromium_fb_region();
    if base == 0 || size < HDR_SIZE {
        uart::puts("[chromium-blit] /batos/fb0 not present — not starting\n");
        return false;
    }

    // Validate header magic (defensive — VFS init should have stamped it).
    let magic = unsafe { core::ptr::read_volatile(base as *const u32) };
    if magic != MAGIC {
        uart::puts("[chromium-blit] bad magic, not starting\n");
        return false;
    }

    STOP_FLAG.store(false, Ordering::Release);
    RUNNING.store(true, Ordering::Release);

    // Priority 10 = above idle (255), below interactive shell tasks.
    match process::create_kernel_task("chromium-blit", kthread_entry, 10) {
        Some(_tid) => {
            let w = unsafe { core::ptr::read_volatile((base + 8) as *const u32) };
            let h = unsafe { core::ptr::read_volatile((base + 12) as *const u32) };
            uart::puts("[chromium-blit] started ");
            crate::kernel::mm::print_num(w as usize);
            uart::puts("x");
            crate::kernel::mm::print_num(h as usize);
            uart::puts(" BGRA\n");
            true
        }
        None => {
            RUNNING.store(false, Ordering::Release);
            uart::puts("[chromium-blit] failed to create task slot\n");
            false
        }
    }
}

// ── Kernel thread body ──────────────────────────────────────────────────
//
// The scheduler expects a `fn() -> !` entry. We spin forever polling the
// shared seq counter. Between polls we yield; with the current priority-
// preemptive scheduler that's the closest thing to "sleep until next tick"
// available today. Each yield returns on the next timer interrupt (~100 Hz
// depending on scheduler config), so our effective poll rate is bounded by
// the tick rate — good enough for v1 per PHASE5_DISPLAY.md §5.2(a).
fn kthread_entry() -> ! {
    let (base, _size) = vfs::chromium_fb_region();
    if base == 0 {
        // Shouldn't happen — start() validated it. Spin-park on the idle
        // path so the task slot is not wasted ping-ponging.
        loop { scheduler::yield_now(); }
    }

    loop {
        TICKS.fetch_add(1, Ordering::Relaxed);

        if STOP_FLAG.load(Ordering::Acquire) {
            RUNNING.store(false, Ordering::Release);
            uart::puts("[chromium-blit] stopped\n");
            // Park — we can't cleanly exit a kthread without a task::exit()
            // API. Future work: add process::exit_current() and call it here.
            loop { scheduler::yield_now(); }
        }

        tick(base);

        // Yield — next run ≈ next timer tick. We don't have timerfd or a
        // hardware vsync to wait on, so cooperative yielding is the v1
        // strategy. If we later get busy_wait_ns(16_666_667), slot it in
        // here to clamp to exactly 60 Hz.
        scheduler::yield_now();
    }
}

/// One poll of the shared region. Returns true if a frame was blitted.
pub fn tick(base: usize) -> bool {
    // Load the current seq with acquire semantics — pairs with Chromium's
    // release fetch_add on the same address. Guarantees every pixel store
    // happens-before our read of the damage rect.
    let seq_ptr = (base + 24) as *const AtomicU32;
    let cur = unsafe { (*seq_ptr).load(Ordering::Acquire) };
    let last = LAST_SEQ.load(Ordering::Relaxed);
    // STUMP #161 iter 28c: print one tick + seq value every 100 polls
    // so we can confirm the kthread is alive AND see what seq it reads.
    // Without this we can't tell "kthread isn't running" from "kthread
    // running but seq stays 0".
    {
        static POLLS: AtomicU32 = AtomicU32::new(0);
        let p = POLLS.fetch_add(1, Ordering::Relaxed);
        if p % 100 == 0 && p < 5000 {
            uart::puts("[chromium-blit] poll #");
            crate::kernel::mm::print_num(p as usize);
            uart::puts(" cur_seq=");
            crate::kernel::mm::print_num(cur as usize);
            uart::puts(" last=");
            crate::kernel::mm::print_num(last as usize);
            uart::puts("\n");
        }
    }
    if cur == last {
        return false;
    }
    {
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        if n < 5 {
            uart::puts("[chromium-blit] tick — seq ");
            crate::kernel::mm::print_num(last as usize);
            uart::puts(" → ");
            crate::kernel::mm::print_num(cur as usize);
            uart::puts("\n");
        }
    }

    // If we missed frames (cur > last + 1), count them as dropped. The
    // reader only ever catches up to the latest frame; intermediates are
    // intentionally skipped per the single-producer/single-consumer design.
    let delta = cur.wrapping_sub(last);
    if delta > 1 {
        FRAMES_DROPPED.fetch_add((delta - 1) as u64, Ordering::Relaxed);
    }

    let width  = unsafe { core::ptr::read_volatile((base + 8)  as *const u32) };
    let height = unsafe { core::ptr::read_volatile((base + 12) as *const u32) };
    let stride = unsafe { core::ptr::read_volatile((base + 16) as *const u32) };

    // On first sighting (last == 0), force a full-screen damage — Risk #7.
    let (mut dx, mut dy, mut dw, mut dh) = if last == 0 {
        (0u32, 0u32, width, height)
    } else {
        let x = unsafe { core::ptr::read_volatile((base + 32) as *const u32) };
        let y = unsafe { core::ptr::read_volatile((base + 36) as *const u32) };
        let w = unsafe { core::ptr::read_volatile((base + 40) as *const u32) };
        let h = unsafe { core::ptr::read_volatile((base + 44) as *const u32) };
        (x, y, w, h)
    };

    // Clip to the smaller of (shared width/height, virtio-gpu width/height).
    let fb_w = gpu::width();
    let fb_h = gpu::height();
    let clip_w = width.min(fb_w);
    let clip_h = height.min(fb_h);
    if dx >= clip_w || dy >= clip_h {
        LAST_SEQ.store(cur, Ordering::Relaxed);
        return false;
    }
    if dx + dw > clip_w { dw = clip_w - dx; }
    if dy + dh > clip_h { dh = clip_h - dy; }
    if dw == 0 || dh == 0 {
        LAST_SEQ.store(cur, Ordering::Relaxed);
        return false;
    }

    // Straight row-wise copy. BGRA matches on both sides (see §5.3 + Appendix A).
    let src_stride_u32 = (stride / 4) as usize;
    let dst_stride_u32 = fb_w as usize;
    let src_pixels = (base + HDR_SIZE) as *const u32;
    let dst_pixels = gpu::framebuffer();

    for row in 0..dh as usize {
        let sy = dy as usize + row;
        let dy_ = dy as usize + row;
        let src_row = unsafe { src_pixels.add(sy * src_stride_u32 + dx as usize) };
        let dst_row = unsafe { dst_pixels.add(dy_ * dst_stride_u32 + dx as usize) };
        // Volatile word copy — the region is shared with a separate trust
        // domain (EL0 Chromium); the compiler must not cache pixels.
        for col in 0..dw as usize {
            unsafe {
                let p = core::ptr::read_volatile(src_row.add(col));
                core::ptr::write_volatile(dst_row.add(col), p);
            }
        }
    }

    // Tell the GPU to send the damaged rect to the host.
    gpu::flush(dx, dy, dw, dh);

    // Record progress: both in our atomic (for stats) and in the
    // user-visible `last_seen_seq` field (offset 28) so debuggers can
    // observe kernel liveness without peeking into kernel memory.
    LAST_SEQ.store(cur, Ordering::Relaxed);
    unsafe {
        core::ptr::write_volatile((base + 28) as *mut u32, cur);
    }
    FRAMES_BLITTED.fetch_add(1, Ordering::Relaxed);
    true
}
