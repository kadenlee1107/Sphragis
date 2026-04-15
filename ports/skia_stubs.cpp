// Bat_OS — Skia C++ symbol stubs
extern "C" {
    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>
}

#include "include/core/SkTypes.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkImage.h"
#include "include/core/SkPixmap.h"
#include "include/core/SkFlattenable.h"
#include "include/core/SkData.h"
#include "include/core/SkStream.h"
#include "include/core/SkColorFilter.h"

// Skia class stubs
void SkFlattenable::Register(const char*, sk_sp<SkFlattenable>(*)(SkReadBuffer&)) {}
SkDynamicMemoryWStream::~SkDynamicMemoryWStream() { reset(); }
sk_sp<SkData> SkDynamicMemoryWStream::detachAsData() { return SkData::MakeEmpty(); }
namespace SkImages {
    sk_sp<SkImage> RasterFromPixmap(const SkPixmap&, void(*)(const void*, void*), void*) { return nullptr; }
}

// Pthread stubs
extern "C" {
    int pthread_mutex_init(void*, const void*) { return 0; }
    int pthread_mutex_destroy(void*) { return 0; }
    int pthread_mutex_lock(void*) { return 0; }
    int pthread_mutex_unlock(void*) { return 0; }
    int pthread_mutex_trylock(void*) { return 0; }
    int pthread_once(void* once, void(*fn)(void)) { int*o=(int*)once; if(!*o){fn();*o=1;} return 0; }
    int pthread_key_create(unsigned int*, void(*)(void*)) { return 0; }
    void* pthread_getspecific(unsigned int) { return 0; }
    int pthread_setspecific(unsigned int, const void*) { return 0; }
    int pthread_rwlock_init(void*, const void*) { return 0; }
    int pthread_rwlock_destroy(void*) { return 0; }
    int pthread_rwlock_rdlock(void*) { return 0; }
    int pthread_rwlock_wrlock(void*) { return 0; }
    int pthread_rwlock_unlock(void*) { return 0; }
    unsigned long pthread_self(void) { return 1; }
    int fileno(void*) { return -1; }
    int fsync(int) { return 0; }
}
