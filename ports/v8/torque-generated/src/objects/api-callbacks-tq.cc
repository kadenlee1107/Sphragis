#include "src/objects/api-callbacks-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/api-callbacks.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1
bool IsAccessorInfo_NonInline(Tagged<HeapObject> o) {
  return IsAccessorInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedAccessorInfo<AccessorInfo, HeapObject>::AccessorInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::AccessorInfoVerify(TrustedCast<AccessorInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1
bool IsInterceptorInfo_NonInline(Tagged<HeapObject> o) {
  return IsInterceptorInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedInterceptorInfo<InterceptorInfo, HeapObject>::InterceptorInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::InterceptorInfoVerify(TrustedCast<InterceptorInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
// Definition https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=29&c=1
class TorqueGeneratedAccessCheckInfoAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=31&c=3
  static constexpr int kCallbackOffset = sizeof(StructLayout);
  static constexpr int kCallbackOffsetEnd = kCallbackOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=32&c=3
  static constexpr int kNamedInterceptorOffset = kCallbackOffsetEnd + 1;
  static constexpr int kNamedInterceptorOffsetEnd = kNamedInterceptorOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=33&c=3
  static constexpr int kIndexedInterceptorOffset = kNamedInterceptorOffsetEnd + 1;
  static constexpr int kIndexedInterceptorOffsetEnd = kIndexedInterceptorOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=34&c=3
  static constexpr int kDataOffset = kIndexedInterceptorOffsetEnd + 1;
  static constexpr int kDataOffsetEnd = kDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kHeaderSize = kDataOffsetEnd + 1;
  static constexpr int kSize = kDataOffsetEnd + 1;

  static_assert(kCallbackOffset == offsetof(AccessCheckInfo, callback_),
                "Value of AccessCheckInfo::kCallbackOffset defined in Torque and offset of field AccessCheckInfo::callback in C++ do not match");
  static_assert(kNamedInterceptorOffset == offsetof(AccessCheckInfo, named_interceptor_),
                "Value of AccessCheckInfo::kNamedInterceptorOffset defined in Torque and offset of field AccessCheckInfo::named_interceptor in C++ do not match");
  static_assert(kIndexedInterceptorOffset == offsetof(AccessCheckInfo, indexed_interceptor_),
                "Value of AccessCheckInfo::kIndexedInterceptorOffset defined in Torque and offset of field AccessCheckInfo::indexed_interceptor in C++ do not match");
  static_assert(kDataOffset == offsetof(AccessCheckInfo, data_),
                "Value of AccessCheckInfo::kDataOffset defined in Torque and offset of field AccessCheckInfo::data in C++ do not match");
  static_assert(kSize == sizeof(AccessCheckInfo));
};

} // namespace internal
} // namespace v8
