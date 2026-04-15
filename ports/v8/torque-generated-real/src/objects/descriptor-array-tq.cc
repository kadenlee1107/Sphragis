#include "src/objects/descriptor-array-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/descriptor-array.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=5&c=1
class TorqueGeneratedEnumCacheAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=7&c=3
  static constexpr int kKeysOffset = sizeof(StructLayout);
  static constexpr int kKeysOffsetEnd = kKeysOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=8&c=3
  static constexpr int kIndicesOffset = kKeysOffsetEnd + 1;
  static constexpr int kIndicesOffsetEnd = kIndicesOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kIndicesOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kIndicesOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kIndicesOffsetEnd + 1;
  static constexpr int kHeaderSize = kIndicesOffsetEnd + 1;
  static constexpr int kSize = kIndicesOffsetEnd + 1;

  static_assert(kKeysOffset == offsetof(EnumCache, keys_),
                "Value of EnumCache::kKeysOffset defined in Torque and offset of field EnumCache::keys in C++ do not match");
  static_assert(kIndicesOffset == offsetof(EnumCache, indices_),
                "Value of EnumCache::kIndicesOffset defined in Torque and offset of field EnumCache::indices in C++ do not match");
  static_assert(kSize == sizeof(EnumCache));
};

// https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1
bool IsDescriptorArray_NonInline(Tagged<HeapObject> o) {
  return IsDescriptorArray(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedDescriptorArray<DescriptorArray, HeapObject>::DescriptorArrayVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::DescriptorArrayVerify(TrustedCast<DescriptorArray>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
