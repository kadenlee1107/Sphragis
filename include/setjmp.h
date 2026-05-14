/*
 * Sphragis — setjmp.h stub for NetSurf
 * AArch64: callee-saved x19-x30, sp, d8-d15 = 22 registers.
 */
#ifndef _SPHRAGIS_SETJMP_H
#define _SPHRAGIS_SETJMP_H

typedef long jmp_buf[32];  /* generous buffer for register save area */

int  setjmp(jmp_buf env);
void longjmp(jmp_buf env, int val) __attribute__((__noreturn__));

#endif /* _SPHRAGIS_SETJMP_H */
