// Bat_OS — Capability System (seL4-inspired)
// Capabilities are unforgeable tokens granting specific access.
// Grant / Delegate / Revoke model.
// No ambient authority — zero caps = zero access.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CapType {
    None,
    IpcSend,    // Can send messages to a specific channel
    IpcRecv,    // Can receive messages on a specific channel
    Memory,     // Can access a specific memory region
    Interrupt,  // Can handle a specific interrupt
    DeviceMmio, // Can access a specific MMIO range
}

#[derive(Clone, Copy, Debug)]
pub struct Capability {
    pub cap_type: CapType,
    pub target: u64,      // Channel ID, memory address, IRQ number, etc.
    pub granted_by: u16,  // TaskId that granted this cap (0 = kernel)
    pub delegatable: bool, // Can this cap be passed to another task?
}

impl Capability {
    pub const fn empty() -> Self {
        Self {
            cap_type: CapType::None,
            target: 0,
            granted_by: 0,
            delegatable: false,
        }
    }
}

pub const MAX_CAPS_PER_TASK: usize = 32;

#[derive(Clone)]
pub struct CapabilitySet {
    caps: [Capability; MAX_CAPS_PER_TASK],
    count: usize,
}

impl CapabilitySet {
    pub const fn empty() -> Self {
        Self {
            caps: [Capability::empty(); MAX_CAPS_PER_TASK],
            count: 0,
        }
    }

    /// Grant a new capability to this set.
    pub fn grant(&mut self, cap: Capability) -> Result<(), &'static str> {
        if self.count >= MAX_CAPS_PER_TASK {
            return Err("capability set full");
        }
        self.caps[self.count] = cap;
        self.count += 1;
        Ok(())
    }

    /// Check if this set contains a capability matching the given type and target.
    pub fn has(&self, cap_type: CapType, target: u64) -> bool {
        self.caps[..self.count]
            .iter()
            .any(|c| c.cap_type == cap_type && c.target == target)
    }

    /// Revoke all capabilities granted by a specific task.
    pub fn revoke_from(&mut self, granter: u16) {
        let mut write = 0;
        for read in 0..self.count {
            if self.caps[read].granted_by != granter {
                self.caps[write] = self.caps[read];
                write += 1;
            }
        }
        self.count = write;
    }

    /// Revoke all capabilities of a specific type and target.
    pub fn revoke(&mut self, cap_type: CapType, target: u64) {
        let mut write = 0;
        for read in 0..self.count {
            if !(self.caps[read].cap_type == cap_type && self.caps[read].target == target) {
                self.caps[write] = self.caps[read];
                write += 1;
            }
        }
        self.count = write;
    }

    pub fn count(&self) -> usize {
        self.count
    }
}
