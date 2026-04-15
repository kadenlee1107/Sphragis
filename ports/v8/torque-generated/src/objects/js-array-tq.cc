#include "src/objects/js-array-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-array.tq?l=61&c=1
bool IsJSArray_NonInline(Tagged<HeapObject> o) {
  return IsJSArray(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSArray<JSArray, JSObject>::JSArrayVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSArrayVerify(TrustedCast<JSArray>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-array.tq?l=7&c=1
bool IsJSArrayIterator_NonInline(Tagged<HeapObject> o) {
  return IsJSArrayIterator(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSArrayIterator<JSArrayIterator, JSObject>::JSArrayIteratorVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSArrayIteratorVerify(TrustedCast<JSArrayIterator>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-array.tq?l=68&c=1
bool IsTemplateLiteralObject_NonInline(Tagged<HeapObject> o) {
  return IsTemplateLiteralObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTemplateLiteralObject<TemplateLiteralObject, JSArray>::TemplateLiteralObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TemplateLiteralObjectVerify(TrustedCast<TemplateLiteralObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
