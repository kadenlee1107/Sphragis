#include "src/objects/feedback-cell.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/feedback-cell.tq?l=5&c=1
class TorqueGeneratedFeedbackCellAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/feedback-cell.tq?l=7&c=3
  static constexpr int kValueOffset = sizeof(StructLayout);
  static constexpr int kValueOffsetEnd = kValueOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kValueOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/feedback-cell.tq?l=8&c=3
  static constexpr int kDispatchHandleOffset = kValueOffsetEnd + 1;
  static constexpr int kDispatchHandleOffsetEnd = kDispatchHandleOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/feedback-cell.tq?l=9&c=3
  static constexpr int kInterruptBudgetOffset = kDispatchHandleOffsetEnd + 1;
  static constexpr int kInterruptBudgetOffsetEnd = kInterruptBudgetOffset + kInt32Size - 1;
  static constexpr int kStartOfWeakFieldsOffset = kInterruptBudgetOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kInterruptBudgetOffsetEnd + 1;
  static constexpr int kHeaderSize = kInterruptBudgetOffsetEnd + 1;
  static constexpr int kSize = kInterruptBudgetOffsetEnd + 1;

  static_assert(kValueOffset == offsetof(FeedbackCell, value_),
                "Value of FeedbackCell::kValueOffset defined in Torque and offset of field FeedbackCell::value in C++ do not match");
  static_assert(kDispatchHandleOffset == offsetof(FeedbackCell, dispatch_handle_),
                "Value of FeedbackCell::kDispatchHandleOffset defined in Torque and offset of field FeedbackCell::dispatch_handle in C++ do not match");
  static_assert(kInterruptBudgetOffset == offsetof(FeedbackCell, interrupt_budget_),
                "Value of FeedbackCell::kInterruptBudgetOffset defined in Torque and offset of field FeedbackCell::interrupt_budget in C++ do not match");
  static_assert(kSize == sizeof(FeedbackCell));
};

} // namespace internal
} // namespace v8
