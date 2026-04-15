#include "src/objects/js-segment-iterator-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=11&c=1
bool IsJSSegmentIterator_NonInline(Tagged<HeapObject> o) {
  return IsJSSegmentIterator(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSSegmentIterator<JSSegmentIterator, JSObject>::JSSegmentIteratorVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSSegmentIteratorVerify(TrustedCast<JSSegmentIterator>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=17&c=1
bool IsJSSegmentDataObject_NonInline(Tagged<HeapObject> o) {
  return IsJSSegmentDataObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSSegmentDataObject<JSSegmentDataObject, JSObject>::JSSegmentDataObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSSegmentDataObjectVerify(TrustedCast<JSSegmentDataObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=25&c=1
bool IsJSSegmentDataObjectWithIsWordLike_NonInline(Tagged<HeapObject> o) {
  return IsJSSegmentDataObjectWithIsWordLike(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSSegmentDataObjectWithIsWordLike<JSSegmentDataObjectWithIsWordLike, JSSegmentDataObject>::JSSegmentDataObjectWithIsWordLikeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSSegmentDataObjectWithIsWordLikeVerify(TrustedCast<JSSegmentDataObjectWithIsWordLike>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
