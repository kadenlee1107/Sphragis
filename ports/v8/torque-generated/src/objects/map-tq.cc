#include "src/objects/map-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/map.tq?l=37&c=1
bool IsMap_NonInline(Tagged<HeapObject> o) {
  return IsMap(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedMap<Map, HeapObject>::MapVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::MapVerify(TrustedCast<Map>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
