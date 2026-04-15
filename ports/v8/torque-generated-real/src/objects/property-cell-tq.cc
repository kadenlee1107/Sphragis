#include "src/objects/property-cell.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/property-cell.tq?l=6&c=1
class TorqueGeneratedPropertyCellAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(HeapObjectLayout);
  // https://crsrc.org/c/v8/src/objects/property-cell.tq?l=8&c=3
  static constexpr int kNameOffset = sizeof(HeapObjectLayout);
  static constexpr int kNameOffsetEnd = kNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/property-cell.tq?l=9&c=3
  static constexpr int kPropertyDetailsRawOffset = kNameOffsetEnd + 1;
  static constexpr int kPropertyDetailsRawOffsetEnd = kPropertyDetailsRawOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/property-cell.tq?l=10&c=3
  static constexpr int kValueOffset = kPropertyDetailsRawOffsetEnd + 1;
  static constexpr int kValueOffsetEnd = kValueOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/property-cell.tq?l=11&c=3
  static constexpr int kDependentCodeOffset = kValueOffsetEnd + 1;
  static constexpr int kDependentCodeOffsetEnd = kDependentCodeOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kDependentCodeOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kDependentCodeOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kDependentCodeOffsetEnd + 1;
  static constexpr int kHeaderSize = kDependentCodeOffsetEnd + 1;
  static constexpr int kSize = kDependentCodeOffsetEnd + 1;

  static_assert(kNameOffset == offsetof(PropertyCell, name_),
                "Value of PropertyCell::kNameOffset defined in Torque and offset of field PropertyCell::name in C++ do not match");
  static_assert(kPropertyDetailsRawOffset == offsetof(PropertyCell, property_details_raw_),
                "Value of PropertyCell::kPropertyDetailsRawOffset defined in Torque and offset of field PropertyCell::property_details_raw in C++ do not match");
  static_assert(kValueOffset == offsetof(PropertyCell, value_),
                "Value of PropertyCell::kValueOffset defined in Torque and offset of field PropertyCell::value in C++ do not match");
  static_assert(kDependentCodeOffset == offsetof(PropertyCell, dependent_code_),
                "Value of PropertyCell::kDependentCodeOffset defined in Torque and offset of field PropertyCell::dependent_code in C++ do not match");
  static_assert(kSize == sizeof(PropertyCell));
};

} // namespace internal
} // namespace v8
