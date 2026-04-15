#include "src/objects/arguments-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/arguments.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1
bool IsJSArgumentsObject_NonInline(Tagged<HeapObject> o) {
  return IsJSArgumentsObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSArgumentsObject<JSArgumentsObject, JSObject>::JSArgumentsObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSArgumentsObjectVerify(TrustedCast<JSArgumentsObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// Definition https://crsrc.org/c/v8/src/objects/arguments.tq?l=41&c=1
class TorqueGeneratedAliasedArgumentsEntryAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=43&c=3
  static constexpr int kAliasedContextSlotOffset = sizeof(StructLayout);
  static constexpr int kAliasedContextSlotOffsetEnd = kAliasedContextSlotOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kAliasedContextSlotOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kAliasedContextSlotOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kAliasedContextSlotOffsetEnd + 1;
  static constexpr int kHeaderSize = kAliasedContextSlotOffsetEnd + 1;
  static constexpr int kSize = kAliasedContextSlotOffsetEnd + 1;

  static_assert(kAliasedContextSlotOffset == offsetof(AliasedArgumentsEntry, aliased_context_slot_),
                "Value of AliasedArgumentsEntry::kAliasedContextSlotOffset defined in Torque and offset of field AliasedArgumentsEntry::aliased_context_slot in C++ do not match");
  static_assert(kSize == sizeof(AliasedArgumentsEntry));
};

} // namespace internal
} // namespace v8
