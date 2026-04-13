// Bat_OS — TCP Layer (Minimal)
// Basic TCP: connect, send, receive, close.
// Not a full TCP stack — enough for HTTP requests.

use super::ip::{self, IpPacket};
use core::sync::atomic::{AtomicU32, AtomicU8, AtomicBool, Ordering};

const TCP_HDR_SIZE: usize = 20;
const TCP_FIN: u8 = 0x01;
const TCP_SYN: u8 = 0x02;
const TCP_RST: u8 = 0x04;
const TCP_PSH: u8 = 0x08;
const TCP_ACK: u8 = 0x10;

const STATE_CLOSED: u8 = 0;
const STATE_SYN_SENT: u8 = 1;
const STATE_ESTABLISHED: u8 = 2;
const STATE_FIN_WAIT: u8 = 3;

static CONN_STATE: AtomicU8 = AtomicU8::new(STATE_CLOSED);
static LOCAL_PORT: AtomicU32 = AtomicU32::new(49152);
static REMOTE_IP: AtomicU32 = AtomicU32::new(0);
static REMOTE_PORT: AtomicU32 = AtomicU32::new(0);
static SEQ_NUM: AtomicU32 = AtomicU32::new(1000);
static ACK_NUM: AtomicU32 = AtomicU32::new(0);
static DATA_READY: AtomicBool = AtomicBool::new(false);

// Receive buffer
const RX_BUF_SIZE: usize = 8192;
static mut RX_BUF: [u8; RX_BUF_SIZE] = [0; RX_BUF_SIZE];
static RX_LEN: AtomicU32 = AtomicU32::new(0);

pub fn handle_incoming(pkt: &IpPacket) {
    if pkt.payload.len() < TCP_HDR_SIZE { return; }

    let src_port = u16::from_be_bytes([pkt.payload[0], pkt.payload[1]]);
    let dst_port = u16::from_be_bytes([pkt.payload[2], pkt.payload[3]]);
    let flags = pkt.payload[13];

    // Debug: log incoming TCP
    if CONN_STATE.load(Ordering::Relaxed) == STATE_SYN_SENT {
        crate::drivers::uart::puts("[tcp] rx flags=0x");
        crate::drivers::uart::putc(b"0123456789abcdef"[((flags >> 4) & 0xF) as usize]);
        crate::drivers::uart::putc(b"0123456789abcdef"[(flags & 0xF) as usize]);
        crate::drivers::uart::puts(" dst=");
        crate::kernel::mm::print_num(dst_port as usize);
        crate::drivers::uart::puts("\n");
    }
    let seq = u32::from_be_bytes([pkt.payload[4], pkt.payload[5], pkt.payload[6], pkt.payload[7]]);
    let ack = u32::from_be_bytes([pkt.payload[8], pkt.payload[9], pkt.payload[10], pkt.payload[11]]);
    let data_offset = ((pkt.payload[12] >> 4) as usize) * 4;
    let flags = pkt.payload[13];

    let local_port = LOCAL_PORT.load(Ordering::Relaxed) as u16;
    if dst_port != local_port { return; }

    let state = CONN_STATE.load(Ordering::Relaxed);

    match state {
        STATE_SYN_SENT => {
            if flags & TCP_SYN != 0 && flags & TCP_ACK != 0 {
                // SYN-ACK received — complete handshake
                ACK_NUM.store(seq.wrapping_add(1), Ordering::Relaxed);
                SEQ_NUM.store(ack, Ordering::Relaxed);
                CONN_STATE.store(STATE_ESTABLISHED, Ordering::Release);

                // Send ACK
                send_tcp(TCP_ACK, &[]);
            }
        }
        STATE_ESTABLISHED => {
            let payload_len = pkt.payload.len() - data_offset;

            if payload_len > 0 {
                // Data received
                unsafe {
                    let rx_len = RX_LEN.load(Ordering::Relaxed) as usize;
                    let copy = payload_len.min(RX_BUF_SIZE - rx_len);
                    RX_BUF[rx_len..rx_len + copy].copy_from_slice(&pkt.payload[data_offset..data_offset + copy]);
                    RX_LEN.store((rx_len + copy) as u32, Ordering::Relaxed);
                }
                ACK_NUM.store(seq.wrapping_add(payload_len as u32), Ordering::Relaxed);
                DATA_READY.store(true, Ordering::Release);
                send_tcp(TCP_ACK, &[]);
            }

            if flags & TCP_FIN != 0 {
                ACK_NUM.store(ACK_NUM.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                send_tcp(TCP_ACK | TCP_FIN, &[]);
                CONN_STATE.store(STATE_CLOSED, Ordering::Relaxed);
            }
        }
        _ => {}
    }
}

fn send_tcp(flags: u8, payload: &[u8]) {
    let local_port = LOCAL_PORT.load(Ordering::Relaxed) as u16;
    let remote_port = REMOTE_PORT.load(Ordering::Relaxed) as u16;
    let remote_ip = REMOTE_IP.load(Ordering::Relaxed);
    let seq = SEQ_NUM.load(Ordering::Relaxed);
    let ack = ACK_NUM.load(Ordering::Relaxed);

    let total = TCP_HDR_SIZE + payload.len();
    let mut tcp = [0u8; 1400];

    tcp[0..2].copy_from_slice(&local_port.to_be_bytes());
    tcp[2..4].copy_from_slice(&remote_port.to_be_bytes());
    tcp[4..8].copy_from_slice(&seq.to_be_bytes());
    tcp[8..12].copy_from_slice(&ack.to_be_bytes());
    tcp[12] = 0x50; // Data offset: 5 words (20 bytes)
    tcp[13] = flags;
    tcp[14..16].copy_from_slice(&8192u16.to_be_bytes()); // Window size

    if !payload.is_empty() {
        tcp[TCP_HDR_SIZE..TCP_HDR_SIZE + payload.len()].copy_from_slice(payload);
    }

    // TCP checksum (with pseudo-header)
    let src_ip = ip::our_ip();
    let mut pseudo = [0u8; 12];
    pseudo[0..4].copy_from_slice(&src_ip.to_be_bytes());
    pseudo[4..8].copy_from_slice(&remote_ip.to_be_bytes());
    pseudo[8] = 0;
    pseudo[9] = 6; // TCP protocol
    pseudo[10..12].copy_from_slice(&(total as u16).to_be_bytes());

    let cksum = tcp_checksum(&pseudo, &tcp[..total]);
    tcp[16..18].copy_from_slice(&cksum.to_be_bytes());

    if flags & TCP_SYN != 0 || !payload.is_empty() {
        { let v = SEQ_NUM.load(Ordering::Relaxed); SEQ_NUM.store(v + if payload.is_empty() { 1 } else { payload.len() as u32 }, Ordering::Relaxed); }
    }

    let _ = ip::send(remote_ip, 6, &tcp[..total]);
}

fn tcp_checksum(pseudo: &[u8], tcp: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    // Add pseudo-header
    let mut i = 0;
    while i + 1 < pseudo.len() {
        sum += u16::from_be_bytes([pseudo[i], pseudo[i+1]]) as u32;
        i += 2;
    }
    // Add TCP segment
    i = 0;
    while i + 1 < tcp.len() {
        sum += u16::from_be_bytes([tcp[i], tcp[i+1]]) as u32;
        i += 2;
    }
    if i < tcp.len() {
        sum += (tcp[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// Connect to a remote TCP server.
pub fn connect(dst_ip: u32, dst_port: u16) -> Result<(), &'static str> {
    let port = LOCAL_PORT.load(Ordering::Relaxed); LOCAL_PORT.store(port + 1, Ordering::Relaxed); let port = port as u16;
    REMOTE_IP.store(dst_ip, Ordering::Relaxed);
    REMOTE_PORT.store(dst_port as u32, Ordering::Relaxed);
    LOCAL_PORT.store(port as u32, Ordering::Relaxed);
    ACK_NUM.store(0, Ordering::Relaxed);
    RX_LEN.store(0, Ordering::Relaxed);
    DATA_READY.store(false, Ordering::Relaxed);
    CONN_STATE.store(STATE_SYN_SENT, Ordering::Relaxed);

    // Send SYN
    crate::drivers::uart::puts("[tcp] sending SYN to ");
    crate::kernel::mm::print_num(((dst_ip >> 24) & 0xFF) as usize);
    crate::drivers::uart::putc(b'.');
    crate::kernel::mm::print_num(((dst_ip >> 16) & 0xFF) as usize);
    crate::drivers::uart::putc(b'.');
    crate::kernel::mm::print_num(((dst_ip >> 8) & 0xFF) as usize);
    crate::drivers::uart::putc(b'.');
    crate::kernel::mm::print_num((dst_ip & 0xFF) as usize);
    crate::drivers::uart::puts("\n");
    send_tcp(TCP_SYN, &[]);

    // Wait for SYN-ACK with real time-based timeout (30 seconds)
    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let timeout_ticks = freq * 30; // 30 seconds

    let mut poll_count: u64 = 0;
    let mut last_print: u64 = start;
    loop {
        super::poll_once();
        poll_count += 1;
        if CONN_STATE.load(Ordering::Acquire) == STATE_ESTABLISHED {
            return Ok(());
        }
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }

        // Print status every 5 seconds
        if now - last_print > freq * 5 {
            crate::drivers::uart::puts("[tcp] waiting... ");
            crate::kernel::mm::print_num(((now - start) / freq) as usize);
            crate::drivers::uart::puts("s polls=");
            crate::kernel::mm::print_num(poll_count as usize);
            crate::drivers::uart::puts("\n");
            last_print = now;
        }

        if now - start > timeout_ticks {
            break;
        }
        core::hint::spin_loop();
    }

    crate::drivers::uart::puts("[tcp] TIMEOUT after ");
    crate::kernel::mm::print_num(poll_count as usize);
    crate::drivers::uart::puts(" polls\n");
    CONN_STATE.store(STATE_CLOSED, Ordering::Relaxed);
    Err("connection timed out")
}

/// Send data on established connection.
pub fn send_data(data: &[u8]) -> Result<(), &'static str> {
    if CONN_STATE.load(Ordering::Relaxed) != STATE_ESTABLISHED {
        return Err("not connected");
    }
    send_tcp(TCP_PSH | TCP_ACK, data);
    Ok(())
}

/// Receive data (blocks until data available or 10s timeout).
pub fn recv_data(buf: &mut [u8]) -> Result<usize, &'static str> {
    let start: u64;
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
    }
    let timeout = freq * 10; // 10 seconds
    loop {
        super::poll_once();
        if DATA_READY.load(Ordering::Acquire) {
            DATA_READY.store(false, Ordering::Relaxed);
            unsafe {
                let len = RX_LEN.load(Ordering::Relaxed) as usize;
                let copy = len.min(buf.len());
                buf[..copy].copy_from_slice(&RX_BUF[..copy]);
                RX_LEN.store(0, Ordering::Relaxed);
                return Ok(copy);
            }
        }
        let now: u64;
        unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
        if now - start > timeout { break; }
        core::hint::spin_loop();
    }
    Err("receive timeout")
}

/// Close the connection.
pub fn close() {
    if CONN_STATE.load(Ordering::Relaxed) == STATE_ESTABLISHED {
        send_tcp(TCP_FIN | TCP_ACK, &[]);
        CONN_STATE.store(STATE_FIN_WAIT, Ordering::Relaxed);
    }
    // Wait briefly for FIN-ACK
    for _ in 0..500_000 {
        super::poll_once();
        core::hint::spin_loop();
    }
    CONN_STATE.store(STATE_CLOSED, Ordering::Relaxed);
}
