#include "src/objects/property-array-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/property-array.tq?l=5&c=1
bool IsPropertyArray_NonInline(Tagged<HeapObject> o) {
  return IsPropertyArray(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedPropertyArray<PropertyArray, HeapObject>::PropertyArrayVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::PropertyArrayVerify(TrustedCast<PropertyArray>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
