// Bat_OS — Minimal C++ Runtime (ABI layer)
// Provides just enough C++ support for Skia and other C++ libraries
// compiled with -fno-exceptions -fno-rtti
//
// Skia specifically uses: operator new/delete, std::nothrow,
// pure virtual handlers, and basic type info stubs.

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// ═══════════════════════════════════════════════════════════════════
// operator new / operator delete — backed by our malloc/free
// ═══════════════════════════════════════════════════════════════════

void *_Znwm(unsigned long size) { // operator new(size_t)
    void *p = malloc(size);
    if (!p) {
        // malloc failed — try brk-based fallback
        // Our mmap-based malloc might not work in all contexts
        // Fall back to a simple bump allocator
        static char fallback_heap[1048576]; // 1MB fallback heap
        static unsigned long fallback_pos = 0;
        unsigned long aligned = (size + 15) & ~15UL;
        if (fallback_pos + aligned <= sizeof(fallback_heap)) {
            p = &fallback_heap[fallback_pos];
            fallback_pos += aligned;
            return p;
        }
        printf("[cxxrt] operator new(%lu) failed!\n", size);
        abort();
    }
    return p;
}

void *_Znam(unsigned long size) { // operator new[](size_t)
    void *p = malloc(size);
    if (!p) p = _Znwm(size); // use fallback
    return p;
}

void _ZdlPv(void *ptr) { // operator delete(void*)
    free(ptr);
}

void _ZdaPv(void *ptr) { // operator delete[](void*)
    free(ptr);
}

// Sized delete (C++14)
void _ZdlPvm(void *ptr, unsigned long size) { // operator delete(void*, size_t)
    (void)size;
    free(ptr);
}

void _ZdaPvm(void *ptr, unsigned long size) { // operator delete[](void*, size_t)
    (void)size;
    free(ptr);
}

// nothrow variants
void *_ZnwmRKSt9nothrow_t(unsigned long size, void *nothrow) {
    (void)nothrow;
    return malloc(size);
}

void *_ZnamRKSt9nothrow_t(unsigned long size, void *nothrow) {
    (void)nothrow;
    return malloc(size);
}

// ═══════════════════════════════════════════════════════════════════
// C++ ABI functions
// ═══════════════════════════════════════════════════════════════════

// Pure virtual function call handler
void __cxa_pure_virtual(void) {
    printf("[cxxrt] FATAL: pure virtual function called!\n");
    abort();
}

// Deleted virtual function call handler
void __cxa_deleted_virtual(void) {
    printf("[cxxrt] FATAL: deleted virtual function called!\n");
    abort();
}

// Guard variables for static local initialization (thread-safe statics)
// __cxa_guard_acquire returns 1 if initialization should proceed
int __cxa_guard_acquire(long long *guard) {
    if (*(char *)guard) return 0; // already initialized
    return 1; // needs initialization
}

void __cxa_guard_release(long long *guard) {
    *(char *)guard = 1; // mark as initialized
}

void __cxa_guard_abort(long long *guard) {
    *(char *)guard = 0; // initialization failed
}

// atexit for static destructors (we don't support these in bare metal)
int __cxa_atexit(void (*destructor)(void*), void *arg, void *dso_handle) {
    (void)destructor; (void)arg; (void)dso_handle;
    return 0;
}

void *__dso_handle = 0;

// Thread-safe static initialization (single-threaded simplification)
int __cxa_thread_atexit_impl(void (*dtor)(void*), void *obj, void *dso_handle) {
    (void)dtor; (void)obj; (void)dso_handle;
    return 0;
}

// Exception handling stubs (for -fno-exceptions mode)
void *__cxa_allocate_exception(unsigned long size) {
    (void)size;
    abort();
    return 0;
}

void __cxa_throw(void *thrown_exception, void *tinfo, void (*dest)(void*)) {
    (void)thrown_exception; (void)tinfo; (void)dest;
    printf("[cxxrt] FATAL: exception thrown (no exception support)!\n");
    abort();
}

void __cxa_rethrow(void) {
    abort();
}

void *__cxa_begin_catch(void *exn) {
    (void)exn;
    return 0;
}

void __cxa_end_catch(void) {}

// ═══════════════════════════════════════════════════════════════════
// RTTI stubs (for -fno-rtti mode, some code still references these)
// ═══════════════════════════════════════════════════════════════════

// Type info base class vtable
void *_ZTVN10__cxxabiv117__class_type_infoE[4] = {0};
void *_ZTVN10__cxxabiv120__si_class_type_infoE[4] = {0};
void *_ZTVN10__cxxabiv121__vmi_class_type_infoE[4] = {0};

// ═══════════════════════════════════════════════════════════════════
// Unwinding stubs (for -fno-exceptions, but some code references)
// ═══════════════════════════════════════════════════════════════════

typedef int _Unwind_Reason_Code;
typedef int _Unwind_Action;

struct _Unwind_Exception {
    long exception_class;
    void (*exception_cleanup)(int, struct _Unwind_Exception *);
    long private_1;
    long private_2;
};

struct _Unwind_Context;

_Unwind_Reason_Code _Unwind_RaiseException(struct _Unwind_Exception *e) {
    (void)e;
    abort();
    return 0;
}

_Unwind_Reason_Code _Unwind_Resume(struct _Unwind_Exception *e) {
    (void)e;
    abort();
    return 0;
}

void *_Unwind_GetLanguageSpecificData(struct _Unwind_Context *ctx) {
    (void)ctx;
    return 0;
}

unsigned long _Unwind_GetIP(struct _Unwind_Context *ctx) {
    (void)ctx;
    return 0;
}

unsigned long _Unwind_GetRegionStart(struct _Unwind_Context *ctx) {
    (void)ctx;
    return 0;
}

void _Unwind_SetGR(struct _Unwind_Context *ctx, int reg, unsigned long val) {
    (void)ctx; (void)reg; (void)val;
}

void _Unwind_SetIP(struct _Unwind_Context *ctx, unsigned long val) {
    (void)ctx; (void)val;
}

// Personality function (referenced by exception tables)
_Unwind_Reason_Code __gxx_personality_v0(
    int version, _Unwind_Action actions,
    long exception_class, struct _Unwind_Exception *ue_header,
    struct _Unwind_Context *context)
{
    (void)version; (void)actions; (void)exception_class;
    (void)ue_header; (void)context;
    abort();
    return 0;
}

// ═══════════════════════════════════════════════════════════════════
// std::terminate and friends
// ═══════════════════════════════════════════════════════════════════

void _ZSt9terminatev(void) { // std::terminate()
    printf("[cxxrt] std::terminate() called!\n");
    abort();
}

void _ZSt17__throw_bad_allocv(void) { // std::__throw_bad_alloc()
    printf("[cxxrt] std::bad_alloc!\n");
    abort();
}

void _ZSt20__throw_length_errorPKc(const char *what) {
    printf("[cxxrt] std::length_error: %s\n", what);
    abort();
}

void _ZSt20__throw_out_of_rangePKc(const char *what) {
    printf("[cxxrt] std::out_of_range: %s\n", what);
    abort();
}

// ═══════════════════════════════════════════════════════════════════
// Stack protector (GCC/Clang stack canary)
// ═══════════════════════════════════════════════════════════════════

unsigned long __stack_chk_guard = 0xDEADBEEFCAFEBABE;

void __stack_chk_fail(void) {
    printf("[cxxrt] FATAL: stack smashing detected!\n");
    abort();
}
