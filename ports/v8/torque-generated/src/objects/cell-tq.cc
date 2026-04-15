#include "src/objects/cell.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/cell.tq?l=5&c=1
class TorqueGeneratedCellAsserts {
  static constexpr int kStartOfWeakFieldsOffset = sizeof(HeapObjectLayout);
  // https://crsrc.org/c/v8/src/objects/cell.tq?l=7&c=3
  static constexpr int kMaybeValueOffset = sizeof(HeapObjectLayout);
  static constexpr int kMaybeValueOffsetEnd = kMaybeValueOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kMaybeValueOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kMaybeValueOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kMaybeValueOffsetEnd + 1;
  static constexpr int kHeaderSize = kMaybeValueOffsetEnd + 1;
  static constexpr int kSize = kMaybeValueOffsetEnd + 1;

  static_assert(kMaybeValueOffset == offsetof(Cell, maybe_value_),
                "Value of Cell::kMaybeValueOffset defined in Torque and offset of field Cell::maybe_value in C++ do not match");
  static_assert(kSize == sizeof(Cell));
};

} // namespace internal
} // namespace v8
