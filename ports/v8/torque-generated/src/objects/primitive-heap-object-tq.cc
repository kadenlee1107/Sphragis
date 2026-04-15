#include "src/objects/primitive-heap-object.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/primitive-heap-object.tq?l=5&c=1
class TorqueGeneratedPrimitiveHeapObjectAsserts {
  static constexpr int kStartOfWeakFieldsOffset = sizeof(HeapObjectLayout);
  static constexpr int kEndOfWeakFieldsOffset = sizeof(HeapObjectLayout);
  static constexpr int kStartOfStrongFieldsOffset = sizeof(HeapObjectLayout);
  static constexpr int kEndOfStrongFieldsOffset = sizeof(HeapObjectLayout);
  static constexpr int kHeaderSize = sizeof(HeapObjectLayout);

};

} // namespace internal
} // namespace v8
