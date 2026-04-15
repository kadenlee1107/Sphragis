#include "src/objects/synthetic-module.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/synthetic-module.tq?l=5&c=1
class TorqueGeneratedSyntheticModuleAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(Module);
  // https://crsrc.org/c/v8/src/objects/synthetic-module.tq?l=7&c=3
  static constexpr int kNameOffset = sizeof(Module);
  static constexpr int kNameOffsetEnd = kNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/synthetic-module.tq?l=8&c=3
  static constexpr int kExportNamesOffset = kNameOffsetEnd + 1;
  static constexpr int kExportNamesOffsetEnd = kExportNamesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/synthetic-module.tq?l=9&c=3
  static constexpr int kEvaluationStepsOffset = kExportNamesOffsetEnd + 1;
  static constexpr int kEvaluationStepsOffsetEnd = kEvaluationStepsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kEvaluationStepsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kEvaluationStepsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kEvaluationStepsOffsetEnd + 1;
  static constexpr int kHeaderSize = kEvaluationStepsOffsetEnd + 1;
  static constexpr int kSize = kEvaluationStepsOffsetEnd + 1;

  static_assert(kNameOffset == offsetof(SyntheticModule, name_),
                "Value of SyntheticModule::kNameOffset defined in Torque and offset of field SyntheticModule::name in C++ do not match");
  static_assert(kExportNamesOffset == offsetof(SyntheticModule, export_names_),
                "Value of SyntheticModule::kExportNamesOffset defined in Torque and offset of field SyntheticModule::export_names in C++ do not match");
  static_assert(kEvaluationStepsOffset == offsetof(SyntheticModule, evaluation_steps_),
                "Value of SyntheticModule::kEvaluationStepsOffset defined in Torque and offset of field SyntheticModule::evaluation_steps in C++ do not match");
  static_assert(kSize == sizeof(SyntheticModule));
};

} // namespace internal
} // namespace v8
