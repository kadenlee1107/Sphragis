#include "src/objects/data-handler.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/data-handler.tq?l=5&c=1
class TorqueGeneratedDataHandlerAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/data-handler.tq?l=8&c=3
  static constexpr int kSmiHandlerOffset = sizeof(StructLayout);
  static constexpr int kSmiHandlerOffsetEnd = kSmiHandlerOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/data-handler.tq?l=9&c=3
  static constexpr int kValidityCellOffset = kSmiHandlerOffsetEnd + 1;
  static constexpr int kValidityCellOffsetEnd = kValidityCellOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kValidityCellOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kValidityCellOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kValidityCellOffsetEnd + 1;
  static constexpr int kHeaderSize = kValidityCellOffsetEnd + 1;

  static_assert(kSmiHandlerOffset == offsetof(DataHandler, smi_handler_),
                "Value of DataHandler::kSmiHandlerOffset defined in Torque and offset of field DataHandler::smi_handler in C++ do not match");
  static_assert(kValidityCellOffset == offsetof(DataHandler, validity_cell_),
                "Value of DataHandler::kValidityCellOffset defined in Torque and offset of field DataHandler::validity_cell in C++ do not match");
};

} // namespace internal
} // namespace v8
