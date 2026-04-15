#include "src/objects/js-weak-refs-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/js-weak-refs.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=9&c=1
bool IsJSFinalizationRegistry_NonInline(Tagged<HeapObject> o) {
  return IsJSFinalizationRegistry(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSFinalizationRegistry<JSFinalizationRegistry, JSObject>::JSFinalizationRegistryVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSFinalizationRegistryVerify(TrustedCast<JSFinalizationRegistry>(*this), isolate);
}


#endif  // VERIFY_HEAP
// Definition https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=21&c=1
class TorqueGeneratedWeakCellAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(HeapObjectLayout);
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=23&c=3
  static constexpr int kFinalizationRegistryOffset = sizeof(HeapObjectLayout);
  static constexpr int kFinalizationRegistryOffsetEnd = kFinalizationRegistryOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=24&c=3
  static constexpr int kHoldingsOffset = kFinalizationRegistryOffsetEnd + 1;
  static constexpr int kHoldingsOffsetEnd = kHoldingsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=25&c=3
  static constexpr int kTargetOffset = kHoldingsOffsetEnd + 1;
  static constexpr int kTargetOffsetEnd = kTargetOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=26&c=3
  static constexpr int kUnregisterTokenOffset = kTargetOffsetEnd + 1;
  static constexpr int kUnregisterTokenOffsetEnd = kUnregisterTokenOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=30&c=3
  static constexpr int kPrevOffset = kUnregisterTokenOffsetEnd + 1;
  static constexpr int kPrevOffsetEnd = kPrevOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=31&c=3
  static constexpr int kNextOffset = kPrevOffsetEnd + 1;
  static constexpr int kNextOffsetEnd = kNextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=39&c=3
  static constexpr int kKeyListPrevOffset = kNextOffsetEnd + 1;
  static constexpr int kKeyListPrevOffsetEnd = kKeyListPrevOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=40&c=3
  static constexpr int kKeyListNextOffset = kKeyListPrevOffsetEnd + 1;
  static constexpr int kKeyListNextOffsetEnd = kKeyListNextOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kKeyListNextOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kKeyListNextOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kKeyListNextOffsetEnd + 1;
  static constexpr int kHeaderSize = kKeyListNextOffsetEnd + 1;
  static constexpr int kSize = kKeyListNextOffsetEnd + 1;

  static_assert(kFinalizationRegistryOffset == offsetof(WeakCell, finalization_registry_),
                "Value of WeakCell::kFinalizationRegistryOffset defined in Torque and offset of field WeakCell::finalization_registry in C++ do not match");
  static_assert(kHoldingsOffset == offsetof(WeakCell, holdings_),
                "Value of WeakCell::kHoldingsOffset defined in Torque and offset of field WeakCell::holdings in C++ do not match");
  static_assert(kTargetOffset == offsetof(WeakCell, target_),
                "Value of WeakCell::kTargetOffset defined in Torque and offset of field WeakCell::target in C++ do not match");
  static_assert(kUnregisterTokenOffset == offsetof(WeakCell, unregister_token_),
                "Value of WeakCell::kUnregisterTokenOffset defined in Torque and offset of field WeakCell::unregister_token in C++ do not match");
  static_assert(kPrevOffset == offsetof(WeakCell, prev_),
                "Value of WeakCell::kPrevOffset defined in Torque and offset of field WeakCell::prev in C++ do not match");
  static_assert(kNextOffset == offsetof(WeakCell, next_),
                "Value of WeakCell::kNextOffset defined in Torque and offset of field WeakCell::next in C++ do not match");
  static_assert(kKeyListPrevOffset == offsetof(WeakCell, key_list_prev_),
                "Value of WeakCell::kKeyListPrevOffset defined in Torque and offset of field WeakCell::key_list_prev in C++ do not match");
  static_assert(kKeyListNextOffset == offsetof(WeakCell, key_list_next_),
                "Value of WeakCell::kKeyListNextOffset defined in Torque and offset of field WeakCell::key_list_next in C++ do not match");
  static_assert(kSize == sizeof(WeakCell));
};

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=43&c=1
bool IsJSWeakRef_NonInline(Tagged<HeapObject> o) {
  return IsJSWeakRef(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSWeakRef<JSWeakRef, JSObject>::JSWeakRefVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSWeakRefVerify(TrustedCast<JSWeakRef>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
