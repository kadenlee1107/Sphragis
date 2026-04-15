#include "src/objects/js-collection-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-collection.tq?l=5&c=1
bool IsJSCollection_NonInline(Tagged<HeapObject> o) {
  return IsJSCollection(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSCollection<JSCollection, JSObject>::JSCollectionVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSCollectionVerify(TrustedCast<JSCollection>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-collection.tq?l=12&c=1
bool IsJSSet_NonInline(Tagged<HeapObject> o) {
  return IsJSSet(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSSet<JSSet, JSCollection>::JSSetVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSSetVerify(TrustedCast<JSSet>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-collection.tq?l=13&c=1
bool IsJSMap_NonInline(Tagged<HeapObject> o) {
  return IsJSMap(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSMap<JSMap, JSCollection>::JSMapVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSMapVerify(TrustedCast<JSMap>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-collection.tq?l=15&c=1
bool IsJSWeakCollection_NonInline(Tagged<HeapObject> o) {
  return IsJSWeakCollection(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSWeakCollection<JSWeakCollection, JSObject>::JSWeakCollectionVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSWeakCollectionVerify(TrustedCast<JSWeakCollection>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-collection.tq?l=20&c=1
bool IsJSWeakSet_NonInline(Tagged<HeapObject> o) {
  return IsJSWeakSet(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSWeakSet<JSWeakSet, JSWeakCollection>::JSWeakSetVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSWeakSetVerify(TrustedCast<JSWeakSet>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-collection.tq?l=21&c=1
bool IsJSWeakMap_NonInline(Tagged<HeapObject> o) {
  return IsJSWeakMap(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSWeakMap<JSWeakMap, JSWeakCollection>::JSWeakMapVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSWeakMapVerify(TrustedCast<JSWeakMap>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
