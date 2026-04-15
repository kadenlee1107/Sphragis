#include "src/objects/js-generator-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/js-generator.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=5&c=1
bool IsJSGeneratorObject_NonInline(Tagged<HeapObject> o) {
  return IsJSGeneratorObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSGeneratorObject<JSGeneratorObject, JSObject>::JSGeneratorObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSGeneratorObjectVerify(TrustedCast<JSGeneratorObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=28&c=1
bool IsJSAsyncFunctionObject_NonInline(Tagged<HeapObject> o) {
  return IsJSAsyncFunctionObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSAsyncFunctionObject<JSAsyncFunctionObject, JSGeneratorObject>::JSAsyncFunctionObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSAsyncFunctionObjectVerify(TrustedCast<JSAsyncFunctionObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=36&c=1
bool IsJSAsyncGeneratorObject_NonInline(Tagged<HeapObject> o) {
  return IsJSAsyncGeneratorObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSAsyncGeneratorObject<JSAsyncGeneratorObject, JSGeneratorObject>::JSAsyncGeneratorObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSAsyncGeneratorObjectVerify(TrustedCast<JSAsyncGeneratorObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// Definition https://crsrc.org/c/v8/src/objects/js-generator.tq?l=44&c=1
class TorqueGeneratedAsyncGeneratorRequestAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=46&c=3
  static constexpr int kNextOffset = sizeof(StructLayout);
  static constexpr int kNextOffsetEnd = kNextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=47&c=3
  static constexpr int kResumeModeOffset = kNextOffsetEnd + 1;
  static constexpr int kResumeModeOffsetEnd = kResumeModeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=48&c=3
  static constexpr int kValueOffset = kResumeModeOffsetEnd + 1;
  static constexpr int kValueOffsetEnd = kValueOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=49&c=3
  static constexpr int kPromiseOffset = kValueOffsetEnd + 1;
  static constexpr int kPromiseOffsetEnd = kPromiseOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPromiseOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kPromiseOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPromiseOffsetEnd + 1;
  static constexpr int kHeaderSize = kPromiseOffsetEnd + 1;
  static constexpr int kSize = kPromiseOffsetEnd + 1;

  static_assert(kNextOffset == offsetof(AsyncGeneratorRequest, next_),
                "Value of AsyncGeneratorRequest::kNextOffset defined in Torque and offset of field AsyncGeneratorRequest::next in C++ do not match");
  static_assert(kResumeModeOffset == offsetof(AsyncGeneratorRequest, resume_mode_),
                "Value of AsyncGeneratorRequest::kResumeModeOffset defined in Torque and offset of field AsyncGeneratorRequest::resume_mode in C++ do not match");
  static_assert(kValueOffset == offsetof(AsyncGeneratorRequest, value_),
                "Value of AsyncGeneratorRequest::kValueOffset defined in Torque and offset of field AsyncGeneratorRequest::value in C++ do not match");
  static_assert(kPromiseOffset == offsetof(AsyncGeneratorRequest, promise_),
                "Value of AsyncGeneratorRequest::kPromiseOffset defined in Torque and offset of field AsyncGeneratorRequest::promise in C++ do not match");
  static_assert(kSize == sizeof(AsyncGeneratorRequest));
};

} // namespace internal
} // namespace v8
