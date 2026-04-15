#include "src/objects/property-descriptor-object.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=19&c=1
class TorqueGeneratedPropertyDescriptorObjectAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=48&c=3
  static constexpr int kFlagsOffset = sizeof(StructLayout);
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=49&c=3
  static constexpr int kValueOffset = kFlagsOffsetEnd + 1;
  static constexpr int kValueOffsetEnd = kValueOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=50&c=3
  static constexpr int kGetOffset = kValueOffsetEnd + 1;
  static constexpr int kGetOffsetEnd = kGetOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=51&c=3
  static constexpr int kSetOffset = kGetOffsetEnd + 1;
  static constexpr int kSetOffsetEnd = kSetOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kSetOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kSetOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kSetOffsetEnd + 1;
  static constexpr int kHeaderSize = kSetOffsetEnd + 1;
  static constexpr int kSize = kSetOffsetEnd + 1;

  static_assert(kFlagsOffset == offsetof(PropertyDescriptorObject, flags_),
                "Value of PropertyDescriptorObject::kFlagsOffset defined in Torque and offset of field PropertyDescriptorObject::flags in C++ do not match");
  static_assert(kValueOffset == offsetof(PropertyDescriptorObject, value_),
                "Value of PropertyDescriptorObject::kValueOffset defined in Torque and offset of field PropertyDescriptorObject::value in C++ do not match");
  static_assert(kGetOffset == offsetof(PropertyDescriptorObject, get_),
                "Value of PropertyDescriptorObject::kGetOffset defined in Torque and offset of field PropertyDescriptorObject::get in C++ do not match");
  static_assert(kSetOffset == offsetof(PropertyDescriptorObject, set_),
                "Value of PropertyDescriptorObject::kSetOffset defined in Torque and offset of field PropertyDescriptorObject::set in C++ do not match");
  static_assert(kSize == sizeof(PropertyDescriptorObject));
};

} // namespace internal
} // namespace v8
