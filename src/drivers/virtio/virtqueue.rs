#![allow(dead_code)]
// Bat_OS — VirtIO Virtqueue Implementation (HVF-safe)
// All memory reads/writes use inline asm to ensure simple ldr/str
// instructions that set ISV for Apple Hypervisor.framework.

use crate::kernel::mm::frame;

const QUEUE_SIZE: u16 = 128;
const VRING_DESC_F_NEXT: u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;

// VringDesc: addr(u64) + len(u32) + flags(u16) + next(u16) = 16 bytes
// VringAvail: flags(u16) + idx(u16) + ring[128](u16) = 260 bytes
// VringUsed: flags(u16) + idx(u16) + ring[128](id:u32+len:u32) = 1028 bytes

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VringDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
pub struct VringAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; QUEUE_SIZE as usize],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VringUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
pub struct VringUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VringUsedElem; QUEUE_SIZE as usize],
}

// HVF-safe memory access — uses explicit ldr/str to ensure ISV bit is set
pub fn safe_write32(addr: usize, val: u32) {
    unsafe {
        core::arch::asm!("str {v:w}, [{a}]", a = in(reg) addr, v = in(reg) val);
    }
}

pub fn safe_read32(addr: usize) -> u32 {
    let val: u32;
    unsafe {
        core::arch::asm!("ldr {v:w}, [{a}]", a = in(reg) addr, v = out(reg) val);
    }
    val
}

pub fn safe_write64(addr: usize, val: u64) {
    unsafe {
        core::arch::asm!("str {v}, [{a}]", a = in(reg) addr, v = in(reg) val);
    }
}

pub fn safe_read64(addr: usize) -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("ldr {v}, [{a}]", a = in(reg) addr, v = out(reg) val);
    }
    val
}

pub fn safe_write16(addr: usize, val: u16) {
    unsafe {
        core::arch::asm!("strh {v:w}, [{a}]", a = in(reg) addr, v = in(reg) val as u32);
    }
}

pub fn safe_read16(addr: usize) -> u16 {
    let val: u32;
    unsafe {
        core::arch::asm!("ldrh {v:w}, [{a}]", a = in(reg) addr, v = out(reg) val);
    }
    val as u16
}

pub struct Virtqueue {
    base: usize,
    free_head: u16,
    last_used_idx: u16,
    num_free: u16,
}

impl Virtqueue {
    pub fn new() -> Option<Self> {
        let page0 = frame::alloc_frame()?;
        let _page1 = frame::alloc_frame()?;

        let vq = Self {
            base: page0,
            free_head: 0,
            last_used_idx: 0,
            num_free: QUEUE_SIZE,
        };

        // Init descriptor free chain
        for i in 0..QUEUE_SIZE as usize {
            let desc_addr = vq.base + i * 16;
            safe_write64(desc_addr, 0);           // addr
            safe_write32(desc_addr + 8, 0);       // len
            safe_write16(desc_addr + 12, 0);      // flags
            let next = if i + 1 < QUEUE_SIZE as usize { (i + 1) as u16 } else { 0 };
            safe_write16(desc_addr + 14, next);   // next
        }

        // Init avail ring
        let avail = vq.avail_addr() as usize;
        safe_write16(avail, 0);     // flags
        safe_write16(avail + 2, 0); // idx

        // Init used ring
        let used = vq.used_addr() as usize;
        safe_write16(used, 0);     // flags
        safe_write16(used + 2, 0); // idx

        Some(vq)
    }

    fn desc_base(&self) -> usize { self.base }

    fn avail_base(&self) -> usize {
        self.base + (QUEUE_SIZE as usize) * 16
    }

    fn used_base(&self) -> usize {
        let avail_end = self.avail_base() + 4 + (QUEUE_SIZE as usize) * 2;
        (avail_end + 4095) & !4095
    }

    pub fn desc_addr(&self) -> u64 { self.base as u64 }
    pub fn avail_addr(&self) -> u64 { self.avail_base() as u64 }
    pub fn used_addr(&self) -> u64 { self.used_base() as u64 }
    pub fn size(&self) -> u16 { QUEUE_SIZE }

    fn alloc_desc(&mut self) -> Option<u16> {
        if self.num_free == 0 { return None; }
        let idx = self.free_head;
        let desc = self.desc_base() + (idx as usize) * 16;
        self.free_head = safe_read16(desc + 14); // next
        self.num_free -= 1;
        Some(idx)
    }

    fn push_avail(&self, idx: u16) {
        let avail = self.avail_base();
        let avail_idx = safe_read16(avail + 2);
        let ring_pos = (avail_idx as usize % QUEUE_SIZE as usize) * 2;
        safe_write16(avail + 4 + ring_pos, idx);
        // Memory barrier
        unsafe { core::arch::asm!("dmb sy"); }
        safe_write16(avail + 2, avail_idx.wrapping_add(1));
    }

    pub fn add_writable(&mut self, buf: *mut u8, len: u32) -> Option<u16> {
        let idx = self.alloc_desc()?;
        let desc = self.desc_base() + (idx as usize) * 16;
        safe_write64(desc, buf as u64);
        safe_write32(desc + 8, len);
        safe_write16(desc + 12, VRING_DESC_F_WRITE);
        safe_write16(desc + 14, 0);
        self.push_avail(idx);
        Some(idx)
    }

    pub fn add_readable(&mut self, buf: *const u8, len: u32) -> Option<u16> {
        let idx = self.alloc_desc()?;
        let desc = self.desc_base() + (idx as usize) * 16;
        safe_write64(desc, buf as u64);
        safe_write32(desc + 8, len);
        safe_write16(desc + 12, 0);
        safe_write16(desc + 14, 0);
        self.push_avail(idx);
        Some(idx)
    }

    pub fn add_chain(
        &mut self,
        header: *const u8, header_len: u32,
        response: *mut u8, response_len: u32,
    ) -> Option<u16> {
        if header.is_null() || header_len == 0 {
            if response.is_null() || response_len == 0 { return None; }
            return self.add_writable(response, response_len);
        }
        if response.is_null() || response_len == 0 {
            return self.add_readable(header, header_len);
        }
        if self.num_free < 2 { return None; }

        let idx0 = self.alloc_desc()?;
        let idx1 = self.alloc_desc()?;

        let d0 = self.desc_base() + (idx0 as usize) * 16;
        safe_write64(d0, header as u64);
        safe_write32(d0 + 8, header_len);
        safe_write16(d0 + 12, VRING_DESC_F_NEXT);
        safe_write16(d0 + 14, idx1);

        let d1 = self.desc_base() + (idx1 as usize) * 16;
        safe_write64(d1, response as u64);
        safe_write32(d1 + 8, response_len);
        safe_write16(d1 + 12, VRING_DESC_F_WRITE);
        safe_write16(d1 + 14, 0);

        self.push_avail(idx0);
        Some(idx0)
    }

    pub fn last_used(&self) -> u16 { self.last_used_idx }

    pub fn poll_used(&mut self) -> Option<(u16, u32)> {
        let used = self.used_base();
        unsafe { core::arch::asm!("dmb sy"); }
        let used_idx = safe_read16(used + 2);

        if self.last_used_idx == used_idx { return None; }

        let entry_off = (self.last_used_idx as usize % QUEUE_SIZE as usize) * 8;
        let entry_id = safe_read32(used + 4 + entry_off);
        let entry_len = safe_read32(used + 4 + entry_off + 4);
        self.last_used_idx = self.last_used_idx.wrapping_add(1);

        // Return descriptors to free list (follow chain)
        let mut idx = entry_id as u16;
        loop {
            let desc = self.desc_base() + (idx as usize) * 16;
            let flags = safe_read16(desc + 12);
            let next = safe_read16(desc + 14);

            safe_write16(desc + 12, 0);
            safe_write16(desc + 14, self.free_head);
            self.free_head = idx;
            self.num_free += 1;

            if flags & VRING_DESC_F_NEXT != 0 { idx = next; } else { break; }
        }

        Some((entry_id as u16, entry_len))
    }
}
