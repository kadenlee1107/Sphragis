// Bat_OS — VirtIO Network Driver
// Sends and receives raw Ethernet frames via virtio-net.
// Uses two queues: RX (receive) and TX (transmit).
// Reference: VirtIO Spec v1.2, Section 5.1

use super::mmio::{self, VirtioMmio};
use super::virtqueue::Virtqueue;
use crate::kernel::mm::frame;
use crate::drivers::uart;
use core::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

const MTU: usize = 1514; // Max Ethernet frame size
const RX_BUFFERS: usize = 16;

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

static NET_BASE: AtomicUsize = AtomicUsize::new(0);
static TX_QUEUE_ADDR: AtomicUsize = AtomicUsize::new(0);
static RX_QUEUE_ADDR: AtomicUsize = AtomicUsize::new(0);
static NET_READY: AtomicBool = AtomicBool::new(false);

// RX buffer pool
static RX_BUF_BASE: AtomicUsize = AtomicUsize::new(0);

// MAC address
static mut MAC_ADDR: [u8; 6] = [0; 6];

pub fn init() -> Option<()> {
    let devices = mmio::probe(mmio::DEVICE_NET);
    let base = devices[0]?;

    uart::puts("  [net] Found virtio-net, initializing...\n");

    let device = VirtioMmio::new(base);
    device.init_device().ok()?;

    // Read MAC address from device config (offset 0x100 for legacy)
    // Use 32-bit reads to avoid HVF byte-access issues
    unsafe {
        let mac_lo = core::ptr::read_volatile((base + 0x100) as *const u32);
        let mac_hi = core::ptr::read_volatile((base + 0x104) as *const u32);
        MAC_ADDR[0] = (mac_lo >> 0) as u8;
        MAC_ADDR[1] = (mac_lo >> 8) as u8;
        MAC_ADDR[2] = (mac_lo >> 16) as u8;
        MAC_ADDR[3] = (mac_lo >> 24) as u8;
        MAC_ADDR[4] = (mac_hi >> 0) as u8;
        MAC_ADDR[5] = (mac_hi >> 8) as u8;
    }

    uart::puts("  [net] MAC: ");
    unsafe {
        for i in 0..6 {
            let b = MAC_ADDR[i];
            let hex = b"0123456789abcdef";
            uart::putc(hex[(b >> 4) as usize]);
            uart::putc(hex[(b & 0xf) as usize]);
            if i < 5 { uart::putc(b':'); }
        }
    }
    uart::puts("\n");

    // Allocate RX queue
    let rx_queue_mem = frame::alloc_frame()?;
    let rx_queue_ptr = rx_queue_mem as *mut Virtqueue;
    let rx_queue = Virtqueue::new()?;
    unsafe { core::ptr::write(rx_queue_ptr, rx_queue); }

    // Allocate TX queue
    let tx_queue_mem = frame::alloc_frame()?;
    let tx_queue_ptr = tx_queue_mem as *mut Virtqueue;
    let tx_queue = Virtqueue::new()?;
    unsafe { core::ptr::write(tx_queue_ptr, tx_queue); }

    RX_QUEUE_ADDR.store(rx_queue_mem, Ordering::Relaxed);
    TX_QUEUE_ADDR.store(tx_queue_mem, Ordering::Relaxed);
    NET_BASE.store(base, Ordering::Relaxed);

    let rx_q = get_rx_queue();
    let tx_q = get_tx_queue();

    device.setup_queue(0, rx_q); // Queue 0 = RX
    device.setup_queue(1, tx_q); // Queue 1 = TX
    device.driver_ok();

    // Allocate RX buffer pool and post receive buffers
    let rx_buf_base = frame::alloc_frame()?;
    for _ in 1..((RX_BUFFERS * (MTU + NET_HDR_SIZE) + 4095) / 4096) {
        frame::alloc_frame()?;
    }
    RX_BUF_BASE.store(rx_buf_base, Ordering::Relaxed);

    // Post ONE receive buffer (simple — we process packets one at a time)
    let rx_q = get_rx_queue();
    rx_q.add_writable(rx_buf_base as *mut u8, (MTU + NET_HDR_SIZE) as u32);
    let device = VirtioMmio::new(base);
    device.notify(0);

    // Allocate static TX buffer
    let tx_buf = frame::alloc_frame()?;
    TX_BUF_ADDR.store(tx_buf, Ordering::Relaxed);

    NET_READY.store(true, Ordering::Relaxed);

    uart::puts("  [net] Network driver ready\n");
    Some(())
}

fn get_rx_queue() -> &'static mut Virtqueue {
    unsafe { &mut *(RX_QUEUE_ADDR.load(Ordering::Relaxed) as *mut Virtqueue) }
}

fn get_tx_queue() -> &'static mut Virtqueue {
    unsafe { &mut *(TX_QUEUE_ADDR.load(Ordering::Relaxed) as *mut Virtqueue) }
}

pub fn mac() -> [u8; 6] {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(MAC_ADDR)) }
}

pub fn is_ready() -> bool {
    NET_READY.load(Ordering::Relaxed)
}

// Static TX buffer — avoids per-packet allocation issues on HVF
static TX_BUF_ADDR: AtomicUsize = AtomicUsize::new(0);

/// Send a raw Ethernet frame.
pub fn send(frame: &[u8]) -> Result<(), &'static str> {
    if !is_ready() {
        return Err("network not ready");
    }

    let tx_q = get_tx_queue();
    let base = NET_BASE.load(Ordering::Relaxed);
    let device = VirtioMmio::new(base);

    let tx_buf = TX_BUF_ADDR.load(Ordering::Relaxed);
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

/// Poll for a received Ethernet frame.
/// Returns the frame data (without virtio header) and its length.
pub fn recv(buf: &mut [u8]) -> Option<usize> {
    if !is_ready() {
        return None;
    }

    let rx_q = get_rx_queue();

    if let Some((_id, len)) = rx_q.poll_used() {
        let rx_buf = RX_BUF_BASE.load(Ordering::Relaxed);
        let total_len = len as usize;

        if total_len <= NET_HDR_SIZE {
            rx_q.add_writable(rx_buf as *mut u8, (MTU + NET_HDR_SIZE) as u32);
            let device = VirtioMmio::new(NET_BASE.load(Ordering::Relaxed));
            device.notify(0);
            return None;
        }

        let frame_len = total_len - NET_HDR_SIZE;
        let copy_len = frame_len.min(buf.len());

        // Copy from the single RX buffer (byte by byte, HVF-safe)
        for i in 0..copy_len {
            let val: u32;
            unsafe {
                core::arch::asm!("ldrb {v:w}, [{a}]", a = in(reg) rx_buf + NET_HDR_SIZE + i, v = out(reg) val);
            }
            buf[i] = val as u8;
        }

        // Re-post the buffer immediately
        rx_q.add_writable(rx_buf as *mut u8, (MTU + NET_HDR_SIZE) as u32);
        let device = VirtioMmio::new(NET_BASE.load(Ordering::Relaxed));
        device.notify(0);

        Some(copy_len)
    } else {
        None
    }
}
