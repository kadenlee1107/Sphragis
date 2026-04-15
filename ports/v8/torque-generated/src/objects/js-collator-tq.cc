#include "src/objects/js-collator-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-collator.tq?l=7&c=1
bool IsJSCollator_NonInline(Tagged<HeapObject> o) {
  return IsJSCollator(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSCollator<JSCollator, JSObject>::JSCollatorVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSCollatorVerify(TrustedCast<JSCollator>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
