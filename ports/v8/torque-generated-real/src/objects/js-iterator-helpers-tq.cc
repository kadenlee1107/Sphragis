#include "src/objects/js-iterator-helpers-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=14&c=1
bool IsJSIteratorHelper_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorHelper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorHelper<JSIteratorHelper, JSObject>::JSIteratorHelperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorHelperVerify(TrustedCast<JSIteratorHelper>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=19&c=1
bool IsJSIteratorHelperSimple_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorHelperSimple(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorHelperSimple<JSIteratorHelperSimple, JSIteratorHelper>::JSIteratorHelperSimpleVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorHelperSimpleVerify(TrustedCast<JSIteratorHelperSimple>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=24&c=1
bool IsJSIteratorMapHelper_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorMapHelper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorMapHelper<JSIteratorMapHelper, JSIteratorHelperSimple>::JSIteratorMapHelperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorMapHelperVerify(TrustedCast<JSIteratorMapHelper>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=29&c=1
bool IsJSIteratorFilterHelper_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorFilterHelper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorFilterHelper<JSIteratorFilterHelper, JSIteratorHelperSimple>::JSIteratorFilterHelperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorFilterHelperVerify(TrustedCast<JSIteratorFilterHelper>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=34&c=1
bool IsJSIteratorTakeHelper_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorTakeHelper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorTakeHelper<JSIteratorTakeHelper, JSIteratorHelperSimple>::JSIteratorTakeHelperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorTakeHelperVerify(TrustedCast<JSIteratorTakeHelper>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=38&c=1
bool IsJSIteratorDropHelper_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorDropHelper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorDropHelper<JSIteratorDropHelper, JSIteratorHelperSimple>::JSIteratorDropHelperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorDropHelperVerify(TrustedCast<JSIteratorDropHelper>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=42&c=1
bool IsJSIteratorFlatMapHelper_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorFlatMapHelper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorFlatMapHelper<JSIteratorFlatMapHelper, JSIteratorHelperSimple>::JSIteratorFlatMapHelperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorFlatMapHelperVerify(TrustedCast<JSIteratorFlatMapHelper>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=48&c=1
bool IsJSIteratorConcatHelper_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorConcatHelper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorConcatHelper<JSIteratorConcatHelper, JSIteratorHelperSimple>::JSIteratorConcatHelperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorConcatHelperVerify(TrustedCast<JSIteratorConcatHelper>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=59&c=1
bool IsJSIteratorZipHelper_NonInline(Tagged<HeapObject> o) {
  return IsJSIteratorZipHelper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSIteratorZipHelper<JSIteratorZipHelper, JSIteratorHelper>::JSIteratorZipHelperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSIteratorZipHelperVerify(TrustedCast<JSIteratorZipHelper>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
