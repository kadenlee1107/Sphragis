#include "src/objects/js-number-format-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-number-format.tq?l=7&c=1
bool IsJSNumberFormat_NonInline(Tagged<HeapObject> o) {
  return IsJSNumberFormat(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSNumberFormat<JSNumberFormat, JSObject>::JSNumberFormatVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSNumberFormatVerify(TrustedCast<JSNumberFormat>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
