// Bat_OS — Fork save/restore via setjmp/longjmp mechanism
// fork_save: saves all callee-saved registers + SP + LR to buffer
// fork_restore: restores them, returning to the clone call site

.global fork_save
.global fork_restore

// fork_save(buf: *mut u64) -> 0
// Saves registers to buf, returns 0 (child return from clone)
fork_save:
    stp     x19, x20, [x0, #0]
    stp     x21, x22, [x0, #16]
    stp     x23, x24, [x0, #32]
    stp     x25, x26, [x0, #48]
    stp     x27, x28, [x0, #64]
    stp     x29, x30, [x0, #80]
    mov     x2, sp
    str     x2, [x0, #96]
    mov     x0, #0          // return 0 = child
    ret

// fork_restore(buf: *const u64, retval: u64)
// Restores registers from buf, returns retval (parent return from clone)
fork_restore:
    ldp     x19, x20, [x0, #0]
    ldp     x21, x22, [x0, #16]
    ldp     x23, x24, [x0, #32]
    ldp     x25, x26, [x0, #48]
    ldp     x27, x28, [x0, #64]
    ldp     x29, x30, [x0, #80]
    ldr     x2, [x0, #96]
    mov     sp, x2
    mov     x0, x1          // return retval = parent pid
    ret
