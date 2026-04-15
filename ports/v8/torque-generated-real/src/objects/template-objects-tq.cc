#include "src/objects/template-objects.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/template-objects.tq?l=5&c=1
class TorqueGeneratedTemplateObjectDescriptionAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/template-objects.tq?l=7&c=3
  static constexpr int kRawStringsOffset = sizeof(StructLayout);
  static constexpr int kRawStringsOffsetEnd = kRawStringsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/template-objects.tq?l=8&c=3
  static constexpr int kCookedStringsOffset = kRawStringsOffsetEnd + 1;
  static constexpr int kCookedStringsOffsetEnd = kCookedStringsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCookedStringsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kCookedStringsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kCookedStringsOffsetEnd + 1;
  static constexpr int kHeaderSize = kCookedStringsOffsetEnd + 1;
  static constexpr int kSize = kCookedStringsOffsetEnd + 1;

  static_assert(kRawStringsOffset == offsetof(TemplateObjectDescription, raw_strings_),
                "Value of TemplateObjectDescription::kRawStringsOffset defined in Torque and offset of field TemplateObjectDescription::raw_strings in C++ do not match");
  static_assert(kCookedStringsOffset == offsetof(TemplateObjectDescription, cooked_strings_),
                "Value of TemplateObjectDescription::kCookedStringsOffset defined in Torque and offset of field TemplateObjectDescription::cooked_strings in C++ do not match");
  static_assert(kSize == sizeof(TemplateObjectDescription));
};

} // namespace internal
} // namespace v8
