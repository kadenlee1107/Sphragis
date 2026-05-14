#![allow(dead_code)]
// Sphragis — VirtIO GPU Driver
// Creates a 2D framebuffer via virtio-gpu protocol.
// Reference: VirtIO Spec v1.2, Section 5.7

use super::mmio::{self, VirtioMmio};
use super::virtqueue::Virtqueue;
use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, Ordering};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 1024;
const BPP: u32 = 4;

// Software-render override: when SOFT_FB is non-zero, gpu::framebuffer()
// returns it and gpu::width/height return SOFT_W/H. Lets the shell's
// `render` command paint into a private buffer (no virtio-gpu device
// needed) and dump the result over the UART.
pub(crate) static SOFT_FB: AtomicUsize = AtomicUsize::new(0);
pub(crate) static SOFT_W:  core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
pub(crate) static SOFT_H:  core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

// GPU command types
const CMD_RESOURCE_CREATE_2D: u32 = 0x0101;
const CMD_SET_SCANOUT: u32 = 0x0103;
const CMD_RESOURCE_FLUSH: u32 = 0x0104;
const CMD_TRANSFER_TO_HOST_2D: u32 = 0x0105;
const CMD_RESOURCE_ATTACH_BACKING: u32 = 0x0106;

const RESP_OK: u32 = 0x1100;
const FORMAT_B8G8R8A8: u32 = 1;
const RES_ID: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct CtrlHdr {
    cmd: u32,
    flags: u32,
    fence_id: u64,
    ctx_id: u32,
    _pad: u32,
}

fn hdr(cmd: u32) -> CtrlHdr {
    CtrlHdr { cmd, flags: 0, fence_id: 0, ctx_id: 0, _pad: 0 }
}

// State
static FB_ADDR: AtomicUsize = AtomicUsize::new(0);
static GPU_BASE_ADDR: AtomicUsize = AtomicUsize::new(0);

// Virtqueue stored as raw memory to avoid static mut
static QUEUE_STORAGE: AtomicUsize = AtomicUsize::new(0);

fn get_queue() -> &'static mut Virtqueue {
    unsafe { &mut *(QUEUE_STORAGE.load(Ordering::Relaxed) as *mut Virtqueue) }
}

fn get_device() -> VirtioMmio {
    VirtioMmio::new(GPU_BASE_ADDR.load(Ordering::Relaxed))
}

fn gpu_cmd<T: Copy>(cmd: &T) -> u32 {
    // bail if the GPU never initialized. Without this
    // guard, callers like `console::putc` → `gpu::flush` → `gpu_cmd`
    // dereference a null Virtqueue pointer and the kernel does
    // millions of safe_read16/safe_write16 ops on garbage memory.
    // On HVF this surfaces as a DATA ABORT in Virtqueue::poll_used
    // shortly after the cave exits (the timer-driven
    // chromium-blit kthread isn't the cause; it's the headless
    // shell printing chromium's "launching ..." message via
    // console::puts which calls gpu::flush per character).
    if QUEUE_STORAGE.load(Ordering::Acquire) == 0 {
        return 0;
    }
    let queue = get_queue();
    let device = get_device();

    let mut resp = CtrlHdr { cmd: 0, flags: 0, fence_id: 0, ctx_id: 0, _pad: 0 };

    queue.add_chain(
        cmd as *const T as *const u8,
        core::mem::size_of::<T>() as u32,
        &mut resp as *mut CtrlHdr as *mut u8,
        core::mem::size_of::<CtrlHdr>() as u32,
    );

    device.notify(0);

    // Poll for completion
    let mut attempts = 0u32;
    while queue.poll_used().is_none() {
        attempts += 1;
        if attempts > 5_000_000 {
            uart::puts("  [gpu] timeout\n");
            return 0;
        }
        core::hint::spin_loop();
    }

    resp.cmd
}

pub fn init() -> Option<()> {
    let devices = mmio::probe(mmio::DEVICE_GPU);
    let base = devices[0]?;

    uart::puts("  [gpu] Found at slot, initializing...\n");

    let device = VirtioMmio::new(base);
    device.init_device().ok()?;

    // Allocate Virtqueue on the heap (a frame)
    let queue_mem = frame::alloc_frame()?;
    let queue_ptr = queue_mem as *mut Virtqueue;

    let queue = Virtqueue::new()?;
    unsafe {
        core::ptr::write(queue_ptr, queue);
    }

    QUEUE_STORAGE.store(queue_mem, Ordering::Relaxed);
    GPU_BASE_ADDR.store(base, Ordering::Relaxed);

    let vq = get_queue();
    device.setup_queue(0, vq);
    device.driver_ok();

    // Allocate framebuffer
    let fb_size = (SCREEN_WIDTH * SCREEN_HEIGHT * BPP) as usize;
    let fb_pages = (fb_size + frame::PAGE_SIZE - 1) / frame::PAGE_SIZE;
    let fb_base = frame::alloc_frame()?;
    for _ in 1..fb_pages {
        frame::alloc_frame()?;
    }
    FB_ADDR.store(fb_base, Ordering::Relaxed);

    uart::puts("  [gpu] Framebuffer: ");
    crate::kernel::mm::print_num(fb_size / 1024);
    uart::puts(" KB\n");

    // 1. Create 2D resource
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Create2d { h: CtrlHdr, res_id: u32, format: u32, w: u32, h2: u32 }
    let cmd = Create2d { h: hdr(CMD_RESOURCE_CREATE_2D), res_id: RES_ID, format: FORMAT_B8G8R8A8, w: SCREEN_WIDTH, h2: SCREEN_HEIGHT };
    if gpu_cmd(&cmd) != RESP_OK { uart::puts("  [gpu] create failed\n"); return None; }

    // 2. Attach backing
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct AttachBacking { h: CtrlHdr, res_id: u32, nr: u32, addr: u64, length: u32, _pad: u32 }
    let cmd = AttachBacking { h: hdr(CMD_RESOURCE_ATTACH_BACKING), res_id: RES_ID, nr: 1, addr: fb_base as u64, length: fb_size as u32, _pad: 0 };
    if gpu_cmd(&cmd) != RESP_OK { uart::puts("  [gpu] attach failed\n"); return None; }

    // 3. Set scanout
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SetScanout { h: CtrlHdr, x: u32, y: u32, w: u32, h2: u32, scanout: u32, res_id: u32 }
    let cmd = SetScanout { h: hdr(CMD_SET_SCANOUT), x: 0, y: 0, w: SCREEN_WIDTH, h2: SCREEN_HEIGHT, scanout: 0, res_id: RES_ID };
    if gpu_cmd(&cmd) != RESP_OK { uart::puts("  [gpu] scanout failed\n"); return None; }

    uart::puts("  [gpu] Display: ");
    crate::kernel::mm::print_num(SCREEN_WIDTH as usize);
    uart::puts("x");
    crate::kernel::mm::print_num(SCREEN_HEIGHT as usize);
    uart::puts("\n");

    Some(())
}

pub fn framebuffer() -> *mut u32 {
    let soft = SOFT_FB.load(Ordering::Relaxed);
    if soft != 0 { return soft as *mut u32; }
    FB_ADDR.load(Ordering::Relaxed) as *mut u32
}

pub fn width() -> u32 {
    let w = SOFT_W.load(Ordering::Relaxed);
    if w != 0 { return w; }
    SCREEN_WIDTH
}
pub fn height() -> u32 {
    let h = SOFT_H.load(Ordering::Relaxed);
    if h != 0 { return h; }
    SCREEN_HEIGHT
}

pub fn flush(x: u32, y: u32, w: u32, h: u32) {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Transfer { hdr: CtrlHdr, x: u32, y: u32, w: u32, h: u32, offset: u64, res_id: u32, _pad: u32 }
    let cmd = Transfer { hdr: hdr(CMD_TRANSFER_TO_HOST_2D), x, y, w, h, offset: 0, res_id: RES_ID, _pad: 0 };
    gpu_cmd(&cmd);

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Flush { hdr: CtrlHdr, x: u32, y: u32, w: u32, h: u32, res_id: u32, _pad: u32 }
    let cmd = Flush { hdr: hdr(CMD_RESOURCE_FLUSH), x, y, w, h, res_id: RES_ID, _pad: 0 };
    gpu_cmd(&cmd);
}

pub fn fill_screen(color: u32) {
    let fb = framebuffer();
    if fb.is_null() { return; }
    let w = width(); let h = height();
    let total = (w * h) as usize;
    for i in 0..total {
        unsafe { core::ptr::write_volatile(fb.add(i), color); }
    }
    if SOFT_FB.load(Ordering::Relaxed) == 0 { flush(0, 0, w, h); }
}

pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let fb = framebuffer();
    if fb.is_null() { return; }
    let sw = width(); let sh = height();
    for row in y..(y + h).min(sh) {
        for col in x..(x + w).min(sw) {
            unsafe {
                core::ptr::write_volatile(fb.add((row * sw + col) as usize), color);
            }
        }
    }
}

pub fn set_pixel(x: u32, y: u32, color: u32) {
    let fb = framebuffer();
    if fb.is_null() { return; }
    let sw = width(); let sh = height();
    if x < sw && y < sh {
        unsafe {
            core::ptr::write_volatile(fb.add((y * sw + x) as usize), color);
        }
    }
}
