// Bat_OS — VirtIO MMIO Transport Layer
// All virtio devices on QEMU virt use Memory-Mapped I/O.
// This implements the common registers and device negotiation.
// Reference: VirtIO Spec v1.2, Section 4.2 (MMIO)

/// VirtIO MMIO register offsets
const MAGIC_VALUE: usize = 0x000;
const VERSION: usize = 0x004;
const DEVICE_ID: usize = 0x008;
const VENDOR_ID: usize = 0x00C;
const DEVICE_FEATURES: usize = 0x010;
const DEVICE_FEATURES_SEL: usize = 0x014;
const DRIVER_FEATURES: usize = 0x020;
const DRIVER_FEATURES_SEL: usize = 0x024;
const QUEUE_SEL: usize = 0x030;
const QUEUE_NUM_MAX: usize = 0x034;
const QUEUE_NUM: usize = 0x038;
const QUEUE_READY: usize = 0x044;
const QUEUE_NOTIFY: usize = 0x050;
const INTERRUPT_STATUS: usize = 0x060;
const INTERRUPT_ACK: usize = 0x064;
const STATUS: usize = 0x070;
const QUEUE_DESC_LOW: usize = 0x080;
const QUEUE_DESC_HIGH: usize = 0x084;
const QUEUE_AVAIL_LOW: usize = 0x090;
const QUEUE_AVAIL_HIGH: usize = 0x094;
const QUEUE_USED_LOW: usize = 0x0A0;
const QUEUE_USED_HIGH: usize = 0x0A4;

/// VirtIO device status bits
const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_FEATURES_OK: u32 = 8;
const STATUS_FAILED: u32 = 128;

/// VirtIO magic value
const VIRTIO_MAGIC: u32 = 0x74726976; // "virt"

/// VirtIO device types
pub const DEVICE_NET: u32 = 1;
pub const DEVICE_BLK: u32 = 2;
pub const DEVICE_CONSOLE: u32 = 3;
pub const DEVICE_GPU: u32 = 16;
pub const DEVICE_INPUT: u32 = 18;

/// QEMU virt machine places virtio MMIO devices here
const VIRTIO_MMIO_BASE: usize = 0x0a000000;
const VIRTIO_MMIO_STRIDE: usize = 0x200;
const VIRTIO_MMIO_COUNT: usize = 32;

pub struct VirtioMmio {
    base: usize,
}

impl VirtioMmio {
    pub fn new(base: usize) -> Self {
        Self { base }
    }

    fn read32(&self, offset: usize) -> u32 {
        let addr = self.base + offset;
        let val: u32;
        // Use explicit ldr to ensure ISV is set for HVF
        unsafe {
            core::arch::asm!(
                "ldr {val:w}, [{addr}]",
                addr = in(reg) addr,
                val = out(reg) val,
            );
        }
        val
    }

    fn write32(&self, offset: usize, val: u32) {
        let addr = self.base + offset;
        // Use explicit str to ensure ISV is set for HVF
        unsafe {
            core::arch::asm!(
                "str {val:w}, [{addr}]",
                addr = in(reg) addr,
                val = in(reg) val,
            );
        }
    }

    /// Check if this MMIO region contains a valid virtio device.
    pub fn is_valid(&self) -> bool {
        let magic = self.read32(MAGIC_VALUE);
        let version = self.read32(VERSION);
        magic == VIRTIO_MAGIC && (version == 1 || version == 2)
    }

    pub fn version(&self) -> u32 {
        self.read32(VERSION)
    }

    pub fn device_id(&self) -> u32 {
        self.read32(DEVICE_ID)
    }

    /// Initialize the device through the standard virtio negotiation.
    pub fn init_device(&self) -> Result<(), &'static str> {
        let version = self.read32(VERSION);

        // 1. Reset
        self.write32(STATUS, 0);

        // 2. Acknowledge
        self.write32(STATUS, STATUS_ACKNOWLEDGE);

        // 3. Driver
        self.write32(STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER);

        // 4. Read device features
        if version >= 2 {
            self.write32(DEVICE_FEATURES_SEL, 0);
        }
        let _features = self.read32(DEVICE_FEATURES);

        // 5. Accept features
        if version >= 2 {
            self.write32(DRIVER_FEATURES_SEL, 0);
        }
        self.write32(DRIVER_FEATURES, 0);

        if version >= 2 {
            // 6. Features OK (v2 only)
            self.write32(STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK);

            let status = self.read32(STATUS);
            if status & STATUS_FEATURES_OK == 0 {
                self.write32(STATUS, STATUS_FAILED);
                return Err("device rejected features");
            }
        }

        Ok(())
    }

    /// Set up a virtqueue for this device.
    pub fn setup_queue(&self, queue_index: u32, queue: &super::virtqueue::Virtqueue) {
        let version = self.read32(VERSION);
        self.write32(QUEUE_SEL, queue_index);

        if version == 1 {
            // Legacy: set queue size and page-aligned address
            let queue_pfn = queue.desc_addr() as u32 / 4096;
            self.write32(QUEUE_NUM, queue.size() as u32);
            self.write32(0x028, 4096); // GuestPageSize
            self.write32(0x040, queue_pfn); // QueuePFN
        } else {
            // Modern: separate desc/avail/used addresses
            let desc_addr = queue.desc_addr();
            let avail_addr = queue.avail_addr();
            let used_addr = queue.used_addr();

            self.write32(QUEUE_NUM, queue.size() as u32);
            self.write32(QUEUE_DESC_LOW, desc_addr as u32);
            self.write32(QUEUE_DESC_HIGH, (desc_addr >> 32) as u32);
            self.write32(QUEUE_AVAIL_LOW, avail_addr as u32);
            self.write32(QUEUE_AVAIL_HIGH, (avail_addr >> 32) as u32);
            self.write32(QUEUE_USED_LOW, used_addr as u32);
            self.write32(QUEUE_USED_HIGH, (used_addr >> 32) as u32);
            self.write32(QUEUE_READY, 1);
        }
    }

    /// Mark device as fully initialized.
    pub fn driver_ok(&self) {
        let version = self.read32(VERSION);
        if version == 1 {
            self.write32(STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK);
        } else {
            self.write32(
                STATUS,
                STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK,
            );
        }
    }

    /// Notify the device that a queue has new buffers.
    pub fn notify(&self, queue_index: u32) {
        self.write32(QUEUE_NOTIFY, queue_index);
    }

    /// Read and acknowledge interrupts.
    pub fn ack_interrupt(&self) -> u32 {
        let status = self.read32(INTERRUPT_STATUS);
        self.write32(INTERRUPT_ACK, status);
        status
    }

    pub fn queue_max_size(&self, queue_index: u32) -> u32 {
        self.write32(QUEUE_SEL, queue_index);
        self.read32(QUEUE_NUM_MAX)
    }

    pub fn base(&self) -> usize {
        self.base
    }
}

/// Probe all MMIO slots and return addresses of devices matching a given type.
pub fn probe(device_type: u32) -> [Option<usize>; 8] {
    let mut found = [None; 8];
    let mut count = 0;

    for i in 0..VIRTIO_MMIO_COUNT {
        let base = VIRTIO_MMIO_BASE + i * VIRTIO_MMIO_STRIDE;
        let dev = VirtioMmio::new(base);

        if dev.is_valid() && dev.device_id() == device_type && count < 8 {
            found[count] = Some(base);
            count += 1;
        }
    }

    found
}
