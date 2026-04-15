#include "src/objects/js-display-names-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=18&c=1
bool IsJSDisplayNames_NonInline(Tagged<HeapObject> o) {
  return IsJSDisplayNames(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSDisplayNames<JSDisplayNames, JSObject>::JSDisplayNamesVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSDisplayNamesVerify(TrustedCast<JSDisplayNames>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
