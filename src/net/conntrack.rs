//! Connection tracking (conntrack-class) — gap-audit item 045.
//!
//! Today's firewall (`src/net/firewall.rs`) is stateless: it matches
//! on (direction, protocol, src_ip, src_port, dst_port). That's
//! adequate for narrow allow-lists but leaves a permissive wildcard
//! inbound TCP rule open so that response packets to client-initiated
//! connections aren't dropped. A real-world attacker can send
//! unsolicited SYNs into any ephemeral dst_port the kernel happens to
//! be using.
//!
//! This module is the foundation for closing that gap: a stateful
//! flow table keyed on `(protocol, remote_ip, remote_port,
//! local_port)`. When Bat_OS initiates an outbound connection, the
//! flow is registered as `New`/`Established`; the inbound side can
//! query `lookup` to confirm an incoming packet matches an existing
//! Bat_OS-initiated flow before consulting the stateless rules.
//!
//! Today's scope (minimum-viable slice):
//!
//!   - 64-slot table (`MAX_FLOWS = 64`).
//!   - States: `New`, `Established`, `Closed`. TCP/UDP both supported.
//!   - `register_outbound`: called from `tcp::connect_blocking_pcb`
//!     when the SYN goes out; future `udp::send` adoption uses the
//!     same API.
//!   - `lookup_inbound(protocol, src_ip, src_port, dst_port)`: from
//!     the perspective of an incoming packet, src_ip/src_port is
//!     the REMOTE side and dst_port is OUR local_port. Matches the
//!     same fields we recorded on register.
//!   - `mark_established` / `mark_closed`: state transitions the
//!     TCP state machine drives.
//!   - `release_local_port`: explicit teardown on PCB close.
//!   - Sweep-on-write GC: every register call drops one stale
//!     Closed entry if the table starts filling.
//!
//! **What this does NOT do yet:** drive the firewall decision. The
//! existing wildcard inbound TCP rule still permits unsolicited
//! SYNs. Removing that wildcard and making `firewall::allow_inbound_tcp`
//! consult conntrack first is the next hardening pass. This module
//! ships the primitive, the wiring on outbound connect, and the
//! selftest that proves the table works end-to-end.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, Ordering};

const MAX_FLOWS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// SYN sent (TCP) or first UDP packet sent. Not yet acknowledged
    /// or replied to.
    New,
    /// 3-way handshake complete (TCP), or a reply observed (UDP).
    Established,
    /// FIN seen or PCB close requested. Slot reserved for late
    /// segments; reclaimed on the next sweep.
    Closed,
}

#[derive(Clone, Copy)]
struct Flow {
    in_use: bool,
    protocol: u8,        // 6 = TCP, 17 = UDP
    remote_ip: u32,
    remote_port: u16,
    local_port: u16,
    state_code: u8,      // 1 = New, 2 = Established, 3 = Closed
}

impl Flow {
    const fn empty() -> Self {
        Self {
            in_use: false, protocol: 0,
            remote_ip: 0, remote_port: 0, local_port: 0,
            state_code: 0,
        }
    }
}

static mut TABLE: [Flow; MAX_FLOWS] = [Flow::empty(); MAX_FLOWS];
static FLOW_COUNT: AtomicU32 = AtomicU32::new(0);
static REGISTER_COUNT: AtomicU32 = AtomicU32::new(0);
static LOOKUP_HITS: AtomicU32 = AtomicU32::new(0);
static LOOKUP_MISSES: AtomicU32 = AtomicU32::new(0);

fn state_to_code(s: State) -> u8 {
    match s { State::New => 1, State::Established => 2, State::Closed => 3 }
}

fn code_to_state(c: u8) -> Option<State> {
    match c { 1 => Some(State::New), 2 => Some(State::Established),
              3 => Some(State::Closed), _ => None }
}

fn sweep_one_closed() {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(TABLE);
        for i in 0..MAX_FLOWS {
            if (*ptr)[i].in_use && (*ptr)[i].state_code == 3 {
                (*ptr)[i] = Flow::empty();
                FLOW_COUNT.fetch_sub(1, Ordering::Relaxed);
                return;
            }
        }
    }
}

/// Register an outbound-initiated flow. Idempotent — re-registering
/// the same (proto, remote_ip, remote_port, local_port) tuple
/// upgrades the state in place rather than allocating a new slot.
///
/// Returns `Some(slot_idx)` on success, `None` if the table is full
/// (after a sweep attempt).
pub fn register_outbound(
    protocol: u8,
    remote_ip: u32,
    remote_port: u16,
    local_port: u16,
    state: State,
) -> Option<usize> {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(TABLE);
        // Update-in-place if the flow already exists.
        for i in 0..MAX_FLOWS {
            let f = &mut (*ptr)[i];
            if f.in_use
                && f.protocol == protocol
                && f.remote_ip == remote_ip
                && f.remote_port == remote_port
                && f.local_port == local_port
            {
                f.state_code = state_to_code(state);
                REGISTER_COUNT.fetch_add(1, Ordering::Relaxed);
                return Some(i);
            }
        }
        // Allocate a fresh slot. Sweep one Closed slot first if
        // we're tight on space.
        let count = FLOW_COUNT.load(Ordering::Relaxed) as usize;
        if count >= MAX_FLOWS {
            sweep_one_closed();
        }
        for i in 0..MAX_FLOWS {
            let f = &mut (*ptr)[i];
            if !f.in_use {
                *f = Flow {
                    in_use: true, protocol,
                    remote_ip, remote_port, local_port,
                    state_code: state_to_code(state),
                };
                FLOW_COUNT.fetch_add(1, Ordering::Relaxed);
                REGISTER_COUNT.fetch_add(1, Ordering::Relaxed);
                return Some(i);
            }
        }
    }
    None
}

/// Lookup an incoming packet against the flow table. From the
/// inbound side, `src_ip` / `src_port` are the REMOTE party we
/// recorded as `remote_ip` / `remote_port` on register, and
/// `dst_port` is OUR `local_port`.
///
/// Returns `Some(state)` if a matching flow exists, `None` if not.
/// `Closed` entries still match — the caller decides whether late
/// segments to a closed flow get a pass or are dropped.
pub fn lookup_inbound(
    protocol: u8,
    src_ip: u32,
    src_port: u16,
    dst_port: u16,
) -> Option<State> {
    unsafe {
        let ptr = core::ptr::addr_of!(TABLE);
        for i in 0..MAX_FLOWS {
            let f = &(*ptr)[i];
            if !f.in_use { continue; }
            if f.protocol == protocol
                && f.remote_ip == src_ip
                && f.remote_port == src_port
                && f.local_port == dst_port
            {
                LOOKUP_HITS.fetch_add(1, Ordering::Relaxed);
                return code_to_state(f.state_code);
            }
        }
    }
    LOOKUP_MISSES.fetch_add(1, Ordering::Relaxed);
    None
}

/// Promote a `New` flow to `Established`. Idempotent: no-op if the
/// flow is already Established or missing.
pub fn mark_established(
    protocol: u8,
    remote_ip: u32,
    remote_port: u16,
    local_port: u16,
) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(TABLE);
        for i in 0..MAX_FLOWS {
            let f = &mut (*ptr)[i];
            if f.in_use
                && f.protocol == protocol
                && f.remote_ip == remote_ip
                && f.remote_port == remote_port
                && f.local_port == local_port
            {
                f.state_code = state_to_code(State::Established);
                return;
            }
        }
    }
}

/// Mark a flow Closed. The slot stays reserved until the next
/// sweep so late segments (final ACK after FIN, retransmits) can
/// still be recognised.
pub fn mark_closed(
    protocol: u8,
    remote_ip: u32,
    remote_port: u16,
    local_port: u16,
) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(TABLE);
        for i in 0..MAX_FLOWS {
            let f = &mut (*ptr)[i];
            if f.in_use
                && f.protocol == protocol
                && f.remote_ip == remote_ip
                && f.remote_port == remote_port
                && f.local_port == local_port
            {
                f.state_code = state_to_code(State::Closed);
                return;
            }
        }
    }
}

/// Drop every flow bound to `local_port`. Called from `tcp::close`
/// when a PCB tears down — the local ephemeral port is about to be
/// reused, so its history must not survive.
pub fn release_local_port(local_port: u16) -> usize {
    let mut dropped = 0usize;
    unsafe {
        let ptr = core::ptr::addr_of_mut!(TABLE);
        for i in 0..MAX_FLOWS {
            let f = &mut (*ptr)[i];
            if f.in_use && f.local_port == local_port {
                *f = Flow::empty();
                FLOW_COUNT.fetch_sub(1, Ordering::Relaxed);
                dropped += 1;
            }
        }
    }
    dropped
}

/// Walk active flows. The callback sees one tuple per active slot:
/// `(protocol, remote_ip, remote_port, local_port, state)`.
pub fn for_each<F: FnMut(u8, u32, u16, u16, State)>(mut f: F) {
    unsafe {
        let ptr = core::ptr::addr_of!(TABLE);
        for i in 0..MAX_FLOWS {
            let fl = &(*ptr)[i];
            if !fl.in_use { continue; }
            if let Some(s) = code_to_state(fl.state_code) {
                f(fl.protocol, fl.remote_ip, fl.remote_port, fl.local_port, s);
            }
        }
    }
}

/// `(active_flows, lifetime_registers, lookup_hits, lookup_misses)`
pub fn stats() -> (u32, u32, u32, u32) {
    (
        FLOW_COUNT.load(Ordering::Relaxed),
        REGISTER_COUNT.load(Ordering::Relaxed),
        LOOKUP_HITS.load(Ordering::Relaxed),
        LOOKUP_MISSES.load(Ordering::Relaxed),
    )
}

/// Test-only: clear the table without resetting counters.
#[cfg(test)]
pub fn reset() {
    unsafe { TABLE = [Flow::empty(); MAX_FLOWS]; }
    FLOW_COUNT.store(0, Ordering::Relaxed);
}
