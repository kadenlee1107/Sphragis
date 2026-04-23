#![allow(dead_code)]
// Bat_OS — VirtIO Network Driver (multi-NIC since 3c-multinic)
//
// Sends and receives raw Ethernet frames via virtio-net. Historically
// this module served a single device (nic 0). Followup #3c introduces
// a second interface so Bat_OS can sit between a "caves" segment and
// the host slirp — the policy enforcement point at packet level.
//
// Both NICs share the same virtio MMIO discovery + init sequence but
// keep independent descriptor rings, TX/RX buffers, and MAC addresses.
// Backward-compat: the zero-arg helpers (`mac()`, `send()`, `recv()`,
// `is_ready()`) default to nic 0 so legacy callers in net/ip.rs,
// net/arp.rs, main.rs etc. do not need to change.
//
// Reference: VirtIO Spec v1.2, Section 5.1

use super::mmio::{self, VirtioMmio};
use super::virtqueue::Virtqueue;
use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, AtomicBool, AtomicU32, Ordering};

const MTU: usize = 1514; // Max Ethernet frame size
const RX_BUFFERS: usize = 16;
/// We support up to two NICs today: nic 0 = host (slirp / control
/// plane), nic 1 = caves (per-cave packet pipeline). The probe loop
/// above this limit would silently drop later devices.
pub const MAX_NICS: usize = 2;

// VirtIO net header (prepended to every packet)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtioNetHeader {
    pub flags: u8,
    pub gso_type: u8,
    pub hdr_len: u16,
    pub gso_size: u16,
    pub csum_start: u16,
    pub csum_offset: u16,
    // v1 legacy doesn't have num_buffers
}

impl VirtioNetHeader {
    pub fn empty() -> Self {
        Self {
            flags: 0, gso_type: 0, hdr_len: 0,
            gso_size: 0, csum_start: 0, csum_offset: 0,
        }
    }
}

const NET_HDR_SIZE: usize = core::mem::size_of::<VirtioNetHeader>();

/// One virtio-net NIC's worth of mutable state. The array is a fixed
/// size so we avoid heap dependency during early boot.
struct Nic {
    base: AtomicUsize,
    tx_queue: AtomicUsize,
    rx_queue: AtomicUsize,
    rx_buf_base: AtomicUsize,
    tx_buf: AtomicUsize,
    mac: [u8; 6],
    ready: AtomicBool,
}

const NIC_NEW: Nic = Nic {
    base: AtomicUsize::new(0),
    tx_queue: AtomicUsize::new(0),
    rx_queue: AtomicUsize::new(0),
    rx_buf_base: AtomicUsize::new(0),
    tx_buf: AtomicUsize::new(0),
    mac: [0; 6],
    ready: AtomicBool::new(false),
};

static mut NICS: [Nic; MAX_NICS] = [NIC_NEW; MAX_NICS];
static NIC_COUNT: AtomicU32 = AtomicU32::new(0);

fn nic(id: usize) -> &'static Nic {
    unsafe {
        // SAFETY: NICS has MAX_NICS entries and id is clamped below.
        // All inner fields are Atomics / POD so concurrent reads are
        // safe under the single-threaded kernel invariant.
        &*core::ptr::addr_of!(NICS[id.min(MAX_NICS - 1)])
    }
}

fn nic_mut(id: usize) -> &'static mut Nic {
    unsafe {
        &mut *core::ptr::addr_of_mut!(NICS[id.min(MAX_NICS - 1)])
    }
}

/// Initialize every discovered virtio-net device (up to MAX_NICS).
/// Returns Some(()) iff at least one NIC came up. Further NICs are
/// best-effort — if nic 1's setup fails we log and keep going on nic 0.
pub fn init() -> Option<()> {
    // QEMU's `-machine virt` assigns virtio-mmio slots starting at the
    // HIGHEST numbered slot for the FIRST `-device` declared, then
    // working downward for each subsequent device. The generic MMIO
    // probe scans bottom-up and returns devices in ASCENDING slot
    // order — that reverses the QEMU declaration order. Walk the
    // returned list in reverse so nic 0 corresponds to the first
    // `-netdev` in the command line (same mental model Linux + BSDs
    // use: `ix0`/`enp0s1`/eth0 = first declared).
    let devices = mmio::probe(mmio::DEVICE_NET);
    let populated: [Option<usize>; 8] = {
        let mut out = [None; 8];
        let mut n = 0;
        for slot in devices.iter() { if slot.is_some() { out[n] = *slot; n += 1; } }
        // Reverse the first `n` entries so the highest MMIO slot is first.
        for i in 0..n/2 {
            let tmp = out[i]; out[i] = out[n - 1 - i]; out[n - 1 - i] = tmp;
        }
        out
    };

    let mut brought_up = 0u32;
    for (idx, slot) in populated.iter().enumerate() {
        if idx >= MAX_NICS { break; }
        let Some(base) = *slot else { continue; };
        if init_nic(idx, base).is_ok() {
            brought_up += 1;
        } else {
            uart::puts("  [net] nic ");
            let d = [b'0' + idx as u8];
            uart::puts(unsafe { core::str::from_utf8_unchecked(&d) });
            uart::puts(" init failed\n");
        }
    }

    NIC_COUNT.store(brought_up, Ordering::Relaxed);
    if brought_up == 0 { None } else { Some(()) }
}

fn init_nic(id: usize, base: usize) -> Result<(), &'static str> {
    uart::puts("  [net] nic ");
    let d = [b'0' + id as u8];
    uart::puts(unsafe { core::str::from_utf8_unchecked(&d) });
    uart::puts(": virtio-net @ ");
    // print the MMIO base in hex (last 4 nybbles are enough to disambiguate)
    let bytes = (base >> 8) as u16;
    let hex = b"0123456789abcdef";
    let mut buf = [0u8; 4];
    buf[0] = hex[(bytes >> 12) as usize & 0xF];
    buf[1] = hex[(bytes >> 8)  as usize & 0xF];
    buf[2] = hex[(bytes >> 4)  as usize & 0xF];
    buf[3] = hex[ bytes        as usize & 0xF];
    uart::puts(unsafe { core::str::from_utf8_unchecked(&buf) });
    uart::puts("00\n");

    let device = VirtioMmio::new(base);
    device.init_device()?;

    // Read MAC address from device config (offset 0x100 for legacy)
    // Use 32-bit reads to avoid HVF byte-access issues
    let mut mac = [0u8; 6];
    unsafe {
        let mac_lo = core::ptr::read_volatile((base + 0x100) as *const u32);
        let mac_hi = core::ptr::read_volatile((base + 0x104) as *const u32);
        mac[0] = (mac_lo >> 0)  as u8;
        mac[1] = (mac_lo >> 8)  as u8;
        mac[2] = (mac_lo >> 16) as u8;
        mac[3] = (mac_lo >> 24) as u8;
        mac[4] = (mac_hi >> 0)  as u8;
        mac[5] = (mac_hi >> 8)  as u8;
    }

    uart::puts("  [net]   MAC: ");
    for i in 0..6 {
        let b = mac[i];
        let hex = b"0123456789abcdef";
        uart::putc(hex[(b >> 4) as usize]);
        uart::putc(hex[(b & 0xf) as usize]);
        if i < 5 { uart::putc(b':'); }
    }
    uart::puts("\n");

    // Allocate RX queue
    let rx_queue_mem = frame::alloc_frame().ok_or("rx queue alloc")?;
    let rx_queue_ptr = rx_queue_mem as *mut Virtqueue;
    let rx_queue = Virtqueue::new().ok_or("rx queue construct")?;
    unsafe { core::ptr::write(rx_queue_ptr, rx_queue); }

    // Allocate TX queue
    let tx_queue_mem = frame::alloc_frame().ok_or("tx queue alloc")?;
    let tx_queue_ptr = tx_queue_mem as *mut Virtqueue;
    let tx_queue = Virtqueue::new().ok_or("tx queue construct")?;
    unsafe { core::ptr::write(tx_queue_ptr, tx_queue); }

    let rx_q = unsafe { &mut *rx_queue_ptr };
    let tx_q = unsafe { &mut *tx_queue_ptr };

    device.setup_queue(0, rx_q); // Queue 0 = RX
    device.setup_queue(1, tx_q); // Queue 1 = TX
    device.driver_ok();

    // Allocate RX buffer pool and post one receive buffer.
    let rx_buf_base = frame::alloc_frame().ok_or("rx buf alloc")?;
    for _ in 1..((RX_BUFFERS * (MTU + NET_HDR_SIZE) + 4095) / 4096) {
        frame::alloc_frame().ok_or("rx extra frame")?;
    }
    rx_q.add_writable(rx_buf_base as *mut u8, (MTU + NET_HDR_SIZE) as u32);
    device.notify(0);

    // Allocate static TX buffer
    let tx_buf = frame::alloc_frame().ok_or("tx buf alloc")?;

    // Commit into the NIC state. The tx_queue/rx_queue addrs point
    // to the on-heap Virtqueue structs allocated above; we stash the
    // raw pointer as usize. Subsequent send/recv reconstructs the
    // &mut Virtqueue via get_tx_queue / get_rx_queue.
    let n = nic_mut(id);
    n.base.store(base, Ordering::Relaxed);
    n.rx_queue.store(rx_queue_mem, Ordering::Relaxed);
    n.tx_queue.store(tx_queue_mem, Ordering::Relaxed);
    n.rx_buf_base.store(rx_buf_base, Ordering::Relaxed);
    n.tx_buf.store(tx_buf, Ordering::Relaxed);
    n.mac = mac;
    n.ready.store(true, Ordering::Relaxed);

    Ok(())
}

fn get_rx_queue(id: usize) -> &'static mut Virtqueue {
    unsafe { &mut *(nic(id).rx_queue.load(Ordering::Relaxed) as *mut Virtqueue) }
}

fn get_tx_queue(id: usize) -> &'static mut Virtqueue {
    unsafe { &mut *(nic(id).tx_queue.load(Ordering::Relaxed) as *mut Virtqueue) }
}

/// Number of NICs brought up (0, 1, or 2).
pub fn count() -> u32 {
    NIC_COUNT.load(Ordering::Relaxed)
}

pub fn mac_n(id: usize) -> [u8; 6] {
    nic(id).mac
}

pub fn mac() -> [u8; 6] {
    mac_n(0)
}

pub fn is_ready_n(id: usize) -> bool {
    nic(id).ready.load(Ordering::Relaxed)
}

pub fn is_ready() -> bool {
    is_ready_n(0)
}

/// Send a raw Ethernet frame on NIC `id`.
pub fn send_n(id: usize, frame: &[u8]) -> Result<(), &'static str> {
    if !is_ready_n(id) {
        return Err("network not ready");
    }

    let tx_q = get_tx_queue(id);
    let base = nic(id).base.load(Ordering::Relaxed);
    let device = VirtioMmio::new(base);

    let tx_buf = nic(id).tx_buf.load(Ordering::Relaxed);
    if tx_buf == 0 {
        return Err("TX buffer not allocated");
    }

    let total_len = NET_HDR_SIZE + frame.len();
    if total_len > 4096 {
        return Err("frame too large");
    }

    // Write virtio-net header (zeros) + frame data using safe writes
    for i in 0..(NET_HDR_SIZE / 4) {
        super::virtqueue::safe_write32(tx_buf + i * 4, 0);
    }
    // Remaining header bytes
    for i in (NET_HDR_SIZE / 4 * 4)..NET_HDR_SIZE {
        unsafe {
            core::arch::asm!("strb wzr, [{a}]", a = in(reg) tx_buf + i);
        }
    }

    // Copy frame data
    for i in 0..frame.len() {
        let val = frame[i];
        unsafe {
            core::arch::asm!("strb {v:w}, [{a}]", a = in(reg) tx_buf + NET_HDR_SIZE + i, v = in(reg) val as u32);
        }
    }

    tx_q.add_readable(tx_buf as *const u8, total_len as u32);
    device.notify(1);

    let mut attempts = 0u32;
    while tx_q.poll_used().is_none() {
        attempts += 1;
        if attempts > 5_000_000 {
            return Err("TX timeout");
        }
        core::hint::spin_loop();
    }

    Ok(())
}

/// Send on default NIC (nic 0). Kept for backward compatibility.
pub fn send(frame: &[u8]) -> Result<(), &'static str> {
    send_n(0, frame)
}

/// Poll for a received Ethernet frame on NIC `id`.
/// Returns the frame data (without virtio header) and its length.
pub fn recv_n(id: usize, buf: &mut [u8]) -> Option<usize> {
    if !is_ready_n(id) {
        return None;
    }

    let rx_q = get_rx_queue(id);

    if let Some((_id_used, len)) = rx_q.poll_used() {
        let rx_buf = nic(id).rx_buf_base.load(Ordering::Relaxed);
        // V8-ROOT-3 / V8-ARITH: clamp attacker-reported `len` to the size we
        // actually posted. A hostile virtio backend could report a value
        // larger than `MTU + NET_HDR_SIZE`; without this clamp we would read
        // past the posted region into adjacent kernel memory when copying.
        let posted_cap = MTU + NET_HDR_SIZE;
        let total_len = (len as usize).min(posted_cap);

        if total_len <= NET_HDR_SIZE {
            rx_q.add_writable(rx_buf as *mut u8, posted_cap as u32);
            let device = VirtioMmio::new(nic(id).base.load(Ordering::Relaxed));
            device.notify(0);
            return None;
        }

        let frame_len = total_len - NET_HDR_SIZE;
        let copy_len = frame_len.min(buf.len()).min(MTU);

        // Copy from the single RX buffer (byte by byte, HVF-safe)
        for i in 0..copy_len {
            let val: u32;
            unsafe {
                core::arch::asm!("ldrb {v:w}, [{a}]", a = in(reg) rx_buf + NET_HDR_SIZE + i, v = out(reg) val);
            }
            buf[i] = val as u8;
        }

        // V11-state-sweep: zero the RX buffer before re-posting. The
        // next inbound packet overwrites only the first `len` bytes, so
        // an attacker who short-packets can leak residue from a previous
        // longer packet's tail below byte `len`. Cheap (1526 bytes).
        unsafe {
            let p = rx_buf as *mut u8;
            for i in 0..posted_cap {
                core::ptr::write_volatile(p.add(i), 0);
            }
        }
        // Re-post the buffer immediately
        rx_q.add_writable(rx_buf as *mut u8, posted_cap as u32);
        let device = VirtioMmio::new(nic(id).base.load(Ordering::Relaxed));
        device.notify(0);

        Some(copy_len)
    } else {
        None
    }
}

/// Receive on default NIC (nic 0). Kept for backward compatibility.
pub fn recv(buf: &mut [u8]) -> Option<usize> {
    recv_n(0, buf)
}
