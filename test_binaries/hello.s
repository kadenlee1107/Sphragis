// Minimal ARM64 Linux "Hello Cave" program
// Uses only write() and exit() syscalls
// Assembled into a static ELF binary

.global _start
.section .text

_start:
    // write(1, msg, 23)
    mov     x8, #64         // syscall: write (ARM64 Linux)
    mov     x0, #1          // fd: stdout
    adr     x1, msg         // buf: message
    mov     x2, #23         // count
    svc     #0

    // exit(0)
    mov     x8, #93         // syscall: exit
    mov     x0, #0          // status: 0
    svc     #0

.section .rodata
msg:
    .ascii "Hello from Cave!\n\0\0\0"
