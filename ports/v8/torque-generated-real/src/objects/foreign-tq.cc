#include "src/objects/foreign-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/foreign.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/foreign.tq?l=5&c=1
class TorqueGeneratedForeignAsserts {
  // https://crsrc.org/c/v8/src/objects/foreign.tq?l=8&c=3
  static constexpr int kForeignAddressOffset = sizeof(HeapObjectLayout);
  static constexpr int kForeignAddressOffsetEnd = kForeignAddressOffset + kExternalPointerSlotSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kForeignAddressOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kForeignAddressOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kForeignAddressOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kForeignAddressOffsetEnd + 1;
  static constexpr int kHeaderSize = kForeignAddressOffsetEnd + 1;
  static constexpr int kSize = kForeignAddressOffsetEnd + 1;

  static_assert(kForeignAddressOffset == offsetof(Foreign, foreign_address_),
                "Value of Foreign::kForeignAddressOffset defined in Torque and offset of field Foreign::foreign_address in C++ do not match");
  static_assert(kSize == sizeof(Foreign));
};

// https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1
bool IsTrustedForeign_NonInline(Tagged<HeapObject> o) {
  return IsTrustedForeign(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTrustedForeign<TrustedForeign, TrustedObject>::TrustedForeignVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TrustedForeignVerify(TrustedCast<TrustedForeign>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
