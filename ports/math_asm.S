// Bat_OS — ARM64 math function implementations
// These MUST be assembly to prevent the compiler from inlining builtins

.text

.global ceil
.type ceil, %function
ceil:
    frintp d0, d0
    ret

.global floor
.type floor, %function
floor:
    frintm d0, d0
    ret

.global fabs
.type fabs, %function
fabs:
    fabs d0, d0
    ret

.global sqrt
.type sqrt, %function
sqrt:
    fsqrt d0, d0
    ret

.global round
.type round, %function
round:
    frinta d0, d0
    ret

.global trunc
.type trunc, %function
trunc:
    frintz d0, d0
    ret

.global rint
.type rint, %function
rint:
    frintx d0, d0
    ret

.global ceilf
.type ceilf, %function
ceilf:
    frintp s0, s0
    ret

.global floorf
.type floorf, %function
floorf:
    frintm s0, s0
    ret

.global fabsf
.type fabsf, %function
fabsf:
    fabs s0, s0
    ret

.global sqrtf
.type sqrtf, %function
sqrtf:
    fsqrt s0, s0
    ret

.global roundf
.type roundf, %function
roundf:
    frinta s0, s0
    ret

.global truncf
.type truncf, %function
truncf:
    frintz s0, s0
    ret

// powf(float, float) — call pow via C
.global powf
.type powf, %function
powf:
    fcvt d0, s0
    fcvt d1, s1
    b pow  // tail call

.global tolower
.type tolower, %function
tolower:
    cmp w0, #'A'
    b.lt 1f
    cmp w0, #'Z'
    b.gt 1f
    add w0, w0, #32
1:  ret

.global toupper
.type toupper, %function
toupper:
    cmp w0, #'a'
    b.lt 1f
    cmp w0, #'z'
    b.gt 1f
    sub w0, w0, #32
1:  ret

.global pthread_self
.type pthread_self, %function
pthread_self:
    mov x0, #1
    ret

.global sem_destroy
.type sem_destroy, %function
sem_destroy:
    mov w0, #0
    ret

.global sem_init
.type sem_init, %function
sem_init:
    mov w0, #0
    ret

.global sem_wait
.type sem_wait, %function
sem_wait:
    mov w0, #0
    ret

.global sem_post
.type sem_post, %function
sem_post:
    mov w0, #0
    ret
