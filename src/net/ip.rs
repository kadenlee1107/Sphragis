// Sphragis — IPv4 Layer
// Handles IP packet construction and parsing.
// QEMU user-mode networking: gateway 10.0.2.2, our IP 10.0.2.15

use crate::drivers::virtio::net as netdev;
use super::{ethernet, arp};
use core::sync::atomic::{AtomicU32, AtomicU16, Ordering};

const IP_HDR_SIZE: usize = 20;
const PROTO_ICMP: u8 = 1;
const PROTO_TCP: u8 = 6;
const PROTO_UDP: u8 = 17;

// QEMU user-mode networking defaults
static OUR_IP: AtomicU32 = AtomicU32::new(0x0A00020F);    // 10.0.2.15
static GATEWAY: AtomicU32 = AtomicU32::new(0x0A000202);    // 10.0.2.2
static SUBNET_MASK: AtomicU32 = AtomicU32::new(0xFFFFFF00); // 255.255.255.0
static IP_ID: AtomicU16 = AtomicU16::new(1);

pub fn our_ip() -> u32 {
    OUR_IP.load(Ordering::Relaxed)
}

pub fn gateway() -> u32 {
    GATEWAY.load(Ordering::Relaxed)
}

pub struct IpPacket<'a> {
    pub src: u32,
    pub dst: u32,
    pub protocol: u8,
    pub payload: &'a [u8],
    pub ttl: u8,
}

/// Walk an IPv4 header's options field looking for a CIPSO label
/// (gov-grade §3.2 SECMARK slice). Returns the level byte from the
/// first CIPSO option found, or `None` if the packet carries no
/// label (or the DOI doesn't match Sphragis's). Receivers can feed
/// this into Bell-LaPadula / Biba checks against the destination
/// cave's labels before delivering the payload.
pub fn parse_cipso_sensitivity(data: &[u8]) -> Option<u8> {
    if data.len() < IP_HDR_SIZE { return None; }
    let ihl = (data[0] & 0x0F) as usize * 4;
    if ihl <= IP_HDR_SIZE || ihl > data.len() { return None; }
    let mut i = IP_HDR_SIZE;
    while i + 2 <= ihl {
        let opt_type = data[i];
        // End-of-options-list (0x00) and NOP (0x01) are single-byte.
        if opt_type == 0x00 { break; }
        if opt_type == 0x01 { i += 1; continue; }
        let opt_len = data[i + 1] as usize;
        if opt_len < 2 || i + opt_len > ihl { return None; }
        if opt_type == CIPSO_OPT_TYPE && opt_len >= 10 {
            // DOI 4 bytes BE, tag-type at +6, taglen at +7, sens at +9.
            let doi = u32::from_be_bytes(
                [data[i + 2], data[i + 3], data[i + 4], data[i + 5]]
            );
            if doi == CIPSO_DOI_SPHRAGIS
                && data[i + 6] == 0x01
                && data[i + 7] >= 4
            {
                return Some(data[i + 9]);
            }
        }
        i += opt_len;
    }
    None
}

impl<'a> IpPacket<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < IP_HDR_SIZE { return None; }

        let version = data[0] >> 4;
        if version != 4 { return None; }

        let ihl = (data[0] & 0x0F) as usize * 4;
        let total_len = u16::from_be_bytes([data[2], data[3]]) as usize;
        let protocol = data[9];
        let src = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
        let dst = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);

        // ATTACK-NET-004/005/006: validate IHL against the buffer and total_len
        // *before* indexing. A crafted packet with ihl < 20 feeds a garbage
        // header, and ihl > total_len causes a slice panic (remote kernel DoS).
        if ihl < IP_HDR_SIZE { return None; }
        if total_len < ihl { return None; }
        if total_len > data.len() { return None; }

        // AUDIT-NET-F1 (2026-05-15): reject IPv4 fragments. The prior
        // parser dispatched fragments (MF=1 or frag_offset!=0) to the
        // upper-layer protocol handlers (tcp::handle_incoming,
        // udp::handle, icmp::handle) as if they were complete datagrams.
        // The transport-layer parsers then interpreted the fragment
        // payload bytes as a forged transport header. Conntrack matches
        // only on (proto, src_ip, src_port, dst_port) so a TCP first-
        // fragment crafted to match an established outbound flow would
        // inject arbitrary seq/ack/flag bits — off-path RST attack +
        // RFC 5961 challenge-ACK state leak. Tail fragments (frag_offset
        // != 0) carrying bytes that look like a transport header had the
        // same shape.
        //
        // Sphragis does not perform reassembly. The right policy for a
        // security-first OS is "reject fragments outright" — RFC 8200/
        // 8900 §4.5 also strongly discourages fragmentation in modern
        // deployments. Bytes 6-7 layout: flags(3 bits) | frag_offset(13).
        let flags_and_frag = u16::from_be_bytes([data[6], data[7]]);
        let mf      = (flags_and_frag & 0x2000) != 0;  // More-Fragments bit
        let frag_off = flags_and_frag & 0x1FFF;
        if mf || frag_off != 0 { return None; }

        // ATTACK-NET-008: verify the IPv4 header checksum. Over a valid header
        // the one's-complement sum is 0xFFFF (!=0 after the final complement).
        if checksum(&data[..ihl]) != 0 { return None; }

        Some(Self {
            src, dst, protocol,
            payload: &data[ihl..total_len],
            ttl: data[8],
        })
    }
}

/// CIPSO Domain of Interpretation we use to brand outbound packets
/// (gov-grade §3.2 SECMARK slice). IANA reserves the DOI value
/// space; we pick a Sphragis-internal one (0x42_42_4F_53 = "BBOS")
/// rather than a real IANA-registered DOI because no router on
/// today's path actually inspects it — the field is purely for
/// internal info-flow accounting between Sphragis instances.
pub const CIPSO_DOI_SPHRAGIS: u32 = 0x42_42_4F_53;
/// CIPSO IP option type byte (RFC 2828 / Trusted-Solaris CIPSO).
const CIPSO_OPT_TYPE:   u8 = 0x86;
/// IP option NOP (used for 4-byte alignment padding).
const IP_OPT_NOP:       u8 = 0x01;
/// Total bytes of IP option (CIPSO + NOP padding) we emit. 10
/// bytes CIPSO + 2 NOPs = 12 bytes, padding IHL to 8 words (32B
/// total header).
const SECMARK_OPT_LEN:  usize = 12;

/// Send an IP packet.
///
/// Gov-grade §3.2 SECMARK slice: when the active cave's MLS
/// sensitivity label is non-Unclassified, we emit a CIPSO IP
/// option carrying the level byte into the IP header. The
/// receiving peer (or a downstream Sphragis) can pick up
/// `parse_cipso_sensitivity` and refuse to accept the packet
/// into a lower-cleared receiver. The wire bytes look like:
///
///     [type=0x86][len=10][DOI 4B="BBOS"][tag=1][taglen=4]
///       [align=0][sens] [NOP][NOP]
///
/// Admin / kernel context (`active_sensitivity() == Unclassified`)
/// keeps the historical no-options IHL=5 header — every existing
/// test passes unchanged, and routers that drop packets with IP
/// options stay happy in the default path.
pub fn send(dst_ip: u32, protocol: u8, payload: &[u8]) -> Result<(), &'static str> {
    // Outbound gate. NET_ISOLATED (default true at boot) refuses all
    // outbound traffic until the operator toggles NET to ROUTED.
    // Was display-only through Wave 7; now load-bearing.
    if super::is_isolated() {
        super::activity::push(
            super::activity::ActivityKind::FwDrop,
            "isolated (outbound refused)",
        );
        crate::drivers::uart::puts("[ip] isolated, refusing outbound\n");
        return Err("net isolated");
    }
    // After the global isolation gate, consult the firewall's
    // outbound policy. Today this is the wildcard "out:any" rule
    // installed by `firewall::init`; future per-cave or per-proto
    // outbound deny rules slot in here without another call site.
    if !super::firewall::allow_outbound(dst_ip, protocol) {
        crate::drivers::uart::puts("[ip] firewall denied outbound\n");
        return Err("outbound blocked by firewall");
    }

    // Confidentiality is handled at TCP layer by TLS. The earlier
    // Tor(VPN(payload)) pipeline was retired: `tor` was 3 layers of
    // CTR with hardcoded keys (not real Tor), and `psk_overlay`
    // (formerly `vpn`) was an AES-CTR envelope with no replay
    // window. Real overlay encryption only ships through the
    // `wireguard` module now (Noise IK + replay window).

    let src_ip = our_ip();
    let id = IP_ID.load(Ordering::Relaxed); IP_ID.store(id.wrapping_add(1), Ordering::Relaxed);

    // SECMARK decision: emit CIPSO only when the active cave has
    // raised its sensitivity above Unclassified. Otherwise the
    // header stays at IHL=5 / 20 bytes (no behavioural change).
    use crate::caves::cave::Sensitivity;
    let active_sens = crate::caves::cave::active_sensitivity();
    let emit_cipso  = active_sens != Sensitivity::Unclassified;
    let opt_bytes   = if emit_cipso { SECMARK_OPT_LEN } else { 0 };
    let header_len  = IP_HDR_SIZE + opt_bytes;
    let ihl_words   = (header_len / 4) as u8;

    // Build IP header
    let total_len = (header_len + payload.len()) as u16;
    let mut ip_pkt = [0u8; 1500];

    ip_pkt[0] = 0x40 | ihl_words; // Version 4, IHL
    ip_pkt[1] = 0;    // DSCP/ECN
    ip_pkt[2..4].copy_from_slice(&total_len.to_be_bytes());
    ip_pkt[4..6].copy_from_slice(&id.to_be_bytes());
    ip_pkt[6] = 0x40; // Don't fragment
    ip_pkt[7] = 0;
    ip_pkt[8] = 64;   // TTL
    ip_pkt[9] = protocol;
    // Checksum at [10..12] — computed below
    ip_pkt[12..16].copy_from_slice(&src_ip.to_be_bytes());
    ip_pkt[16..20].copy_from_slice(&dst_ip.to_be_bytes());

    // CIPSO option block (when emitting).
    if emit_cipso {
        let o = IP_HDR_SIZE;
        ip_pkt[o]       = CIPSO_OPT_TYPE;
        ip_pkt[o + 1]   = 0x0a;             // option length = 10
        ip_pkt[o + 2..o + 6].copy_from_slice(&CIPSO_DOI_SPHRAGIS.to_be_bytes());
        ip_pkt[o + 6]   = 0x01;             // tag type = 1 (restrictive bitmap)
        ip_pkt[o + 7]   = 0x04;             // tag length = 4 bytes
        ip_pkt[o + 8]   = 0x00;             // alignment flags
        ip_pkt[o + 9]   = active_sens as u8;
        ip_pkt[o + 10]  = IP_OPT_NOP;
        ip_pkt[o + 11]  = IP_OPT_NOP;
    }

    // Copy payload after the header (+ options if present).
    ip_pkt[header_len..header_len + payload.len()].copy_from_slice(payload);

    // Compute header checksum across header + any options.
    let cksum = checksum(&ip_pkt[..header_len]);
    ip_pkt[10..12].copy_from_slice(&cksum.to_be_bytes());

    // Determine next-hop MAC
    let mask = SUBNET_MASK.load(Ordering::Relaxed);
    let next_hop = if (dst_ip & mask) == (src_ip & mask) {
        dst_ip // Same subnet, send directly
    } else {
        gateway() // Different subnet, send to gateway
    };

    let dst_mac = arp::resolve(next_hop).ok_or("ARP resolve failed")?;
    let src_mac = netdev::mac();

    // Build Ethernet frame
    let mut frame = [0u8; 1514];
    let frame_len = ethernet::EthFrame::build(
        &dst_mac, &src_mac, ethernet::ETHERTYPE_IPV4,
        &ip_pkt[..total_len as usize], &mut frame,
    );

    netdev::send(&frame[..frame_len])
}

/// Test-only: build the same IPv4 wire bytes `send` would emit
/// (header + CIPSO option per active cave + payload) into `out`,
/// returning the number of bytes written. No psk-overlay, no ARP,
/// no NIC — useful for selftests that need to inspect the
/// SECMARK CIPSO emission without owning a real NIC. Returns 0 on
/// any sizing error.
pub fn build_test_packet(dst_ip: u32, protocol: u8, payload: &[u8], out: &mut [u8]) -> usize {
    use crate::caves::cave::Sensitivity;
    let active_sens = crate::caves::cave::active_sensitivity();
    let emit_cipso  = active_sens != Sensitivity::Unclassified;
    let opt_bytes   = if emit_cipso { SECMARK_OPT_LEN } else { 0 };
    let header_len  = IP_HDR_SIZE + opt_bytes;
    let total_len   = header_len + payload.len();
    if total_len > out.len() { return 0; }
    let ihl_words   = (header_len / 4) as u8;

    let src_ip = our_ip();
    let id = IP_ID.load(Ordering::Relaxed); IP_ID.store(id.wrapping_add(1), Ordering::Relaxed);

    out[0] = 0x40 | ihl_words;
    out[1] = 0;
    out[2..4].copy_from_slice(&(total_len as u16).to_be_bytes());
    out[4..6].copy_from_slice(&id.to_be_bytes());
    out[6] = 0x40;
    out[7] = 0;
    out[8] = 64;
    out[9] = protocol;
    out[10] = 0; out[11] = 0;
    out[12..16].copy_from_slice(&src_ip.to_be_bytes());
    out[16..20].copy_from_slice(&dst_ip.to_be_bytes());
    if emit_cipso {
        let o = IP_HDR_SIZE;
        out[o]      = CIPSO_OPT_TYPE;
        out[o + 1]  = 0x0a;
        out[o + 2..o + 6].copy_from_slice(&CIPSO_DOI_SPHRAGIS.to_be_bytes());
        out[o + 6]  = 0x01;
        out[o + 7]  = 0x04;
        out[o + 8]  = 0x00;
        out[o + 9]  = active_sens as u8;
        out[o + 10] = IP_OPT_NOP;
        out[o + 11] = IP_OPT_NOP;
    }
    out[header_len..header_len + payload.len()].copy_from_slice(payload);
    let cksum = checksum(&out[..header_len]);
    out[10..12].copy_from_slice(&cksum.to_be_bytes());
    total_len
}

/// Kernel-wide ceiling on inbound CIPSO-labeled packets — gov-grade
/// §3.2 SECMARK receiver slice. A packet whose CIPSO sensitivity
/// byte exceeds this value is dropped at `ip::handle` before any
/// transport handler sees it. Default at boot: Unclassified (0),
/// meaning we reject any inbound packet that carries a non-zero
/// label. Admins raise the ceiling via `secmark-set-ceiling <level>`.
pub static INBOUND_CIPSO_CEILING: core::sync::atomic::AtomicU8 =
    core::sync::atomic::AtomicU8::new(0);
pub static INBOUND_SECMARK_DROPS: core::sync::atomic::AtomicU32 =
    core::sync::atomic::AtomicU32::new(0);

/// Read the current inbound CIPSO ceiling.
pub fn inbound_cipso_ceiling() -> u8 {
    INBOUND_CIPSO_CEILING.load(Ordering::Relaxed)
}

/// Set the ceiling. Used by the `secmark-set-ceiling` shell command.
pub fn set_inbound_cipso_ceiling(level: u8) {
    INBOUND_CIPSO_CEILING.store(level, Ordering::Relaxed);
}

/// Handle an incoming IP packet.
pub fn handle(data: &[u8]) {
    if let Some(pkt) = IpPacket::parse(data) {
        // AUDIT-NET-F3 (2026-05-15): filter inbound packets by dst-IP.
        // The prior code dispatched to transport handlers without
        // verifying pkt.dst == our_ip(). With promiscuous-mode NICs or
        // misconfigured switches, packets destined for OTHER hosts on
        // the segment reached our TCP / ICMP handlers. Combined with
        // conntrack matching on (proto, remote_ip, remote_port,
        // local_port) (Net-F6) this gave an L2-adjacent attacker
        // injection into any of our PCBs without addressing our IP.
        // Broadcast (255.255.255.255) and multicast (224.0.0.0/4) also
        // unconditionally reached ICMP echo — classic smurf amplifier.
        //
        // Policy: accept only unicast packets addressed to us. Drop
        // broadcast and multicast outright; if we ever need multicast
        // (mDNS / SSDP / etc.) the receiver side wires explicit joined-
        // group state and accepts only those addresses.
        if pkt.dst != our_ip() {
            return;
        }

        // Receiver-side SECMARK enforcement (§3.2). If the packet
        // carries a CIPSO label, gate delivery on the kernel-wide
        // ceiling. A sensitivity above the ceiling means "this
        // information is too classified to land in our caves" — drop
        // before any transport handler can see the payload. The
        // ceiling defaults to Unclassified (0) so unlabeled traffic
        // (the common case today) still flows, but anything tagged
        // gets refused until an admin opts in.
        if let Some(level) = parse_cipso_sensitivity(data) {
            let ceiling = inbound_cipso_ceiling();
            if level > ceiling {
                INBOUND_SECMARK_DROPS.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        // Check firewall
        if !super::firewall::allow_inbound(pkt.src, pkt.dst, pkt.protocol) {
            // Debug: show blocked packets during TCP connect
            if pkt.protocol == 6 {
                crate::drivers::uart::puts("[fw] BLOCKED TCP from ");
                crate::kernel::mm::print_num(((pkt.src >> 24) & 0xFF) as usize);
                crate::drivers::uart::putc(b'.');
                crate::kernel::mm::print_num(((pkt.src >> 16) & 0xFF) as usize);
                crate::drivers::uart::putc(b'.');
                crate::kernel::mm::print_num(((pkt.src >> 8) & 0xFF) as usize);
                crate::drivers::uart::putc(b'.');
                crate::kernel::mm::print_num((pkt.src & 0xFF) as usize);
                crate::drivers::uart::puts("\n");
            }
            return;
        }

        // PSK-overlay decrypt path retired (see send-side comment).
        // Decryption now happens only at higher layers (TLS for TCP,
        // WireGuard for explicit tunnels).
        match pkt.protocol {
            PROTO_ICMP => super::icmp::handle(&pkt),
            PROTO_UDP => super::udp::handle(&pkt),
            PROTO_TCP => super::tcp::handle_incoming(&pkt),
            _ => {}
        }
    }
}

pub fn checksum(data: &[u8]) -> u16 {
    // V8-ROOT-3: one's-complement summation tolerates wrap-around by
    // construction (the carry is folded back below). Use wrapping_add so
    // overflow-checks=true does not panic on long buffers.
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum = sum.wrapping_add(u16::from_be_bytes([data[i], data[i + 1]]) as u32);
        i += 2;
    }
    if i < data.len() {
        sum = sum.wrapping_add((data[i] as u32) << 8);
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

pub fn ip_to_str(ip: u32, buf: &mut [u8; 16]) -> usize {
    let octets = ip.to_be_bytes();
    let mut pos = 0;
    for (i, &oct) in octets.iter().enumerate() {
        let mut n = oct;
        if n >= 100 { buf[pos] = b'0' + n / 100; pos += 1; n %= 100; }
        if n >= 10 || oct >= 100 { buf[pos] = b'0' + n / 10; pos += 1; n %= 10; }
        buf[pos] = b'0' + n; pos += 1;
        if i < 3 { buf[pos] = b'.'; pos += 1; }
    }
    pos
}
