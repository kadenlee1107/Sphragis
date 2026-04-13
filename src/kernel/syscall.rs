// Bat_OS — System Call Handler
// Tasks use SVC instruction to request kernel services.
// Each syscall is capability-checked before execution.

use crate::kernel::arch::TrapFrame;
use crate::drivers::uart;

// Syscall numbers
pub const SYS_YIELD: u16 = 0;
pub const SYS_SEND: u16 = 1;
pub const SYS_RECV: u16 = 2;
pub const SYS_PRINT: u16 = 3; // Debug only — will be removed in hardened build
pub const SYS_EXIT: u16 = 4;

pub fn handle(num: u16, frame: &mut TrapFrame) {
    match num {
        SYS_YIELD => {
            crate::kernel::scheduler::yield_now();
        }
        SYS_PRINT => {
            // Debug syscall: print a byte to UART
            let byte = frame.x[0] as u8;
            uart::putc(byte);
        }
        SYS_EXIT => {
            let current = crate::kernel::process::current();
            current.state = crate::kernel::process::TaskState::Dead;
            uart::puts("[syscall] Task exited: ");
            uart::puts(current.name_str());
            uart::puts("\n");
            crate::kernel::scheduler::yield_now();
        }
        _ => {
            uart::puts("[syscall] Unknown syscall: ");
            uart::putc(b'0' + (num / 10) as u8);
            uart::putc(b'0' + (num % 10) as u8);
            uart::puts("\n");
        }
    }
}
