#include "src/objects/js-array-buffer-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=18&c=1
bool IsJSArrayBuffer_NonInline(Tagged<HeapObject> o) {
  return IsJSArrayBuffer(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSArrayBuffer<JSArrayBuffer, JSAPIObjectWithEmbedderSlots>::JSArrayBufferVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSArrayBufferVerify(TrustedCast<JSArrayBuffer>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=72&c=1
bool IsJSArrayBufferView_NonInline(Tagged<HeapObject> o) {
  return IsJSArrayBufferView(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSArrayBufferView<JSArrayBufferView, JSAPIObjectWithEmbedderSlots>::JSArrayBufferViewVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSArrayBufferViewVerify(TrustedCast<JSArrayBufferView>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=118&c=1
bool IsJSTypedArray_NonInline(Tagged<HeapObject> o) {
  return IsJSTypedArray(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSTypedArray<JSTypedArray, JSArrayBufferView>::JSTypedArrayVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSTypedArrayVerify(TrustedCast<JSTypedArray>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=129&c=1
bool IsJSDetachedTypedArray_NonInline(Tagged<HeapObject> o) {
  return IsJSDetachedTypedArray(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSDetachedTypedArray<JSDetachedTypedArray, JSTypedArray>::JSDetachedTypedArrayVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSDetachedTypedArrayVerify(TrustedCast<JSDetachedTypedArray>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=139&c=1
bool IsJSDataViewOrRabGsabDataView_NonInline(Tagged<HeapObject> o) {
  return IsJSDataViewOrRabGsabDataView(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSDataViewOrRabGsabDataView<JSDataViewOrRabGsabDataView, JSArrayBufferView>::JSDataViewOrRabGsabDataViewVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSDataViewOrRabGsabDataViewVerify(TrustedCast<JSDataViewOrRabGsabDataView>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=145&c=1
bool IsJSDataView_NonInline(Tagged<HeapObject> o) {
  return IsJSDataView(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSDataView<JSDataView, JSDataViewOrRabGsabDataView>::JSDataViewVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSDataViewVerify(TrustedCast<JSDataView>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=147&c=1
bool IsJSRabGsabDataView_NonInline(Tagged<HeapObject> o) {
  return IsJSRabGsabDataView(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSRabGsabDataView<JSRabGsabDataView, JSDataViewOrRabGsabDataView>::JSRabGsabDataViewVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSRabGsabDataViewVerify(TrustedCast<JSRabGsabDataView>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
