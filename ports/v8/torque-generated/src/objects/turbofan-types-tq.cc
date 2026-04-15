#include "src/objects/turbofan-types-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=12&c=1
bool IsTurbofanType_NonInline(Tagged<HeapObject> o) {
  return IsTurbofanType(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTurbofanType<TurbofanType, HeapObject>::TurbofanTypeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TurbofanTypeVerify(TrustedCast<TurbofanType>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=61&c=1
bool IsTurbofanBitsetType_NonInline(Tagged<HeapObject> o) {
  return IsTurbofanBitsetType(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTurbofanBitsetType<TurbofanBitsetType, TurbofanType>::TurbofanBitsetTypeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TurbofanBitsetTypeVerify(TrustedCast<TurbofanBitsetType>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=67&c=1
bool IsTurbofanUnionType_NonInline(Tagged<HeapObject> o) {
  return IsTurbofanUnionType(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTurbofanUnionType<TurbofanUnionType, TurbofanType>::TurbofanUnionTypeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TurbofanUnionTypeVerify(TrustedCast<TurbofanUnionType>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=73&c=1
bool IsTurbofanRangeType_NonInline(Tagged<HeapObject> o) {
  return IsTurbofanRangeType(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTurbofanRangeType<TurbofanRangeType, TurbofanType>::TurbofanRangeTypeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TurbofanRangeTypeVerify(TrustedCast<TurbofanRangeType>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=79&c=1
bool IsTurbofanHeapConstantType_NonInline(Tagged<HeapObject> o) {
  return IsTurbofanHeapConstantType(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTurbofanHeapConstantType<TurbofanHeapConstantType, TurbofanType>::TurbofanHeapConstantTypeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TurbofanHeapConstantTypeVerify(TrustedCast<TurbofanHeapConstantType>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=84&c=1
bool IsTurbofanOtherNumberConstantType_NonInline(Tagged<HeapObject> o) {
  return IsTurbofanOtherNumberConstantType(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTurbofanOtherNumberConstantType<TurbofanOtherNumberConstantType, TurbofanType>::TurbofanOtherNumberConstantTypeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TurbofanOtherNumberConstantTypeVerify(TrustedCast<TurbofanOtherNumberConstantType>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
