#include "src/objects/debug-objects-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/debug-objects.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=5&c=1
class TorqueGeneratedBreakPointAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=7&c=3
  static constexpr int kIdOffset = sizeof(StructLayout);
  static constexpr int kIdOffsetEnd = kIdOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=8&c=3
  static constexpr int kConditionOffset = kIdOffsetEnd + 1;
  static constexpr int kConditionOffsetEnd = kConditionOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kConditionOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kConditionOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kConditionOffsetEnd + 1;
  static constexpr int kHeaderSize = kConditionOffsetEnd + 1;
  static constexpr int kSize = kConditionOffsetEnd + 1;

  static_assert(kIdOffset == offsetof(BreakPoint, id_),
                "Value of BreakPoint::kIdOffset defined in Torque and offset of field BreakPoint::id in C++ do not match");
  static_assert(kConditionOffset == offsetof(BreakPoint, condition_),
                "Value of BreakPoint::kConditionOffset defined in Torque and offset of field BreakPoint::condition in C++ do not match");
  static_assert(kSize == sizeof(BreakPoint));
};

// Definition https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=11&c=1
class TorqueGeneratedBreakPointInfoAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=14&c=3
  static constexpr int kSourcePositionOffset = sizeof(StructLayout);
  static constexpr int kSourcePositionOffsetEnd = kSourcePositionOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=16&c=3
  static constexpr int kBreakPointsOffset = kSourcePositionOffsetEnd + 1;
  static constexpr int kBreakPointsOffsetEnd = kBreakPointsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kBreakPointsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kBreakPointsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kBreakPointsOffsetEnd + 1;
  static constexpr int kHeaderSize = kBreakPointsOffsetEnd + 1;
  static constexpr int kSize = kBreakPointsOffsetEnd + 1;

  static_assert(kSourcePositionOffset == offsetof(BreakPointInfo, source_position_),
                "Value of BreakPointInfo::kSourcePositionOffset defined in Torque and offset of field BreakPointInfo::source_position in C++ do not match");
  static_assert(kBreakPointsOffset == offsetof(BreakPointInfo, break_points_),
                "Value of BreakPointInfo::kBreakPointsOffset defined in Torque and offset of field BreakPointInfo::break_points in C++ do not match");
  static_assert(kSize == sizeof(BreakPointInfo));
};

// Definition https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=35&c=1
class TorqueGeneratedDebugInfoAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=37&c=3
  static constexpr int kSharedOffset = sizeof(StructLayout);
  static constexpr int kSharedOffsetEnd = kSharedOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=39&c=3
  static constexpr int kDebuggerHintsOffset = kSharedOffsetEnd + 1;
  static constexpr int kDebuggerHintsOffsetEnd = kDebuggerHintsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=41&c=3
  static constexpr int kBreakPointsOffset = kDebuggerHintsOffsetEnd + 1;
  static constexpr int kBreakPointsOffsetEnd = kBreakPointsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=43&c=36
  static constexpr int kFlagsOffset = kBreakPointsOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=44&c=3
  static constexpr int kCoverageInfoOffset = kFlagsOffsetEnd + 1;
  static constexpr int kCoverageInfoOffsetEnd = kCoverageInfoOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCoverageInfoOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=48&c=3
  static constexpr int kOriginalBytecodeArrayOffset = kCoverageInfoOffsetEnd + 1;
  static constexpr int kOriginalBytecodeArrayOffsetEnd = kOriginalBytecodeArrayOffset + kTrustedPointerSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=52&c=3
  static constexpr int kDebugBytecodeArrayOffset = kOriginalBytecodeArrayOffsetEnd + 1;
  static constexpr int kDebugBytecodeArrayOffsetEnd = kDebugBytecodeArrayOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kDebugBytecodeArrayOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kDebugBytecodeArrayOffsetEnd + 1;
  static constexpr int kHeaderSize = kDebugBytecodeArrayOffsetEnd + 1;
  static constexpr int kSize = kDebugBytecodeArrayOffsetEnd + 1;

  static_assert(kSharedOffset == offsetof(DebugInfo, shared_),
                "Value of DebugInfo::kSharedOffset defined in Torque and offset of field DebugInfo::shared in C++ do not match");
  static_assert(kDebuggerHintsOffset == offsetof(DebugInfo, debugger_hints_),
                "Value of DebugInfo::kDebuggerHintsOffset defined in Torque and offset of field DebugInfo::debugger_hints in C++ do not match");
  static_assert(kBreakPointsOffset == offsetof(DebugInfo, break_points_),
                "Value of DebugInfo::kBreakPointsOffset defined in Torque and offset of field DebugInfo::break_points in C++ do not match");
  static_assert(kFlagsOffset == offsetof(DebugInfo, flags_),
                "Value of DebugInfo::kFlagsOffset defined in Torque and offset of field DebugInfo::flags in C++ do not match");
  static_assert(kCoverageInfoOffset == offsetof(DebugInfo, coverage_info_),
                "Value of DebugInfo::kCoverageInfoOffset defined in Torque and offset of field DebugInfo::coverage_info in C++ do not match");
  static_assert(kOriginalBytecodeArrayOffset == offsetof(DebugInfo, original_bytecode_array_),
                "Value of DebugInfo::kOriginalBytecodeArrayOffset defined in Torque and offset of field DebugInfo::original_bytecode_array in C++ do not match");
  static_assert(kDebugBytecodeArrayOffset == offsetof(DebugInfo, debug_bytecode_array_),
                "Value of DebugInfo::kDebugBytecodeArrayOffset defined in Torque and offset of field DebugInfo::debug_bytecode_array in C++ do not match");
  static_assert(kSize == sizeof(DebugInfo));
};

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1
bool IsCoverageInfo_NonInline(Tagged<HeapObject> o) {
  return IsCoverageInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedCoverageInfo<CoverageInfo, HeapObject>::CoverageInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::CoverageInfoVerify(TrustedCast<CoverageInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
// Definition https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=75&c=1
class TorqueGeneratedStackFrameInfoAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=83&c=3
  static constexpr int kSharedOrScriptOffset = sizeof(StructLayout);
  static constexpr int kSharedOrScriptOffsetEnd = kSharedOrScriptOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=84&c=3
  static constexpr int kFunctionNameOffset = kSharedOrScriptOffsetEnd + 1;
  static constexpr int kFunctionNameOffsetEnd = kFunctionNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=85&c=3
  static constexpr int kFlagsOffset = kFunctionNameOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;
  static constexpr int kSize = kFlagsOffsetEnd + 1;

  static_assert(kSharedOrScriptOffset == offsetof(StackFrameInfo, shared_or_script_),
                "Value of StackFrameInfo::kSharedOrScriptOffset defined in Torque and offset of field StackFrameInfo::shared_or_script in C++ do not match");
  static_assert(kFunctionNameOffset == offsetof(StackFrameInfo, function_name_),
                "Value of StackFrameInfo::kFunctionNameOffset defined in Torque and offset of field StackFrameInfo::function_name in C++ do not match");
  static_assert(kFlagsOffset == offsetof(StackFrameInfo, flags_),
                "Value of StackFrameInfo::kFlagsOffset defined in Torque and offset of field StackFrameInfo::flags in C++ do not match");
  static_assert(kSize == sizeof(StackFrameInfo));
};

// Definition https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=88&c=1
class TorqueGeneratedStackTraceInfoAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=91&c=3
  static constexpr int kIdOffset = sizeof(StructLayout);
  static constexpr int kIdOffsetEnd = kIdOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=93&c=3
  static constexpr int kFramesOffset = kIdOffsetEnd + 1;
  static constexpr int kFramesOffsetEnd = kFramesOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFramesOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFramesOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFramesOffsetEnd + 1;
  static constexpr int kHeaderSize = kFramesOffsetEnd + 1;
  static constexpr int kSize = kFramesOffsetEnd + 1;

  static_assert(kIdOffset == offsetof(StackTraceInfo, id_),
                "Value of StackTraceInfo::kIdOffset defined in Torque and offset of field StackTraceInfo::id in C++ do not match");
  static_assert(kFramesOffset == offsetof(StackTraceInfo, frames_),
                "Value of StackTraceInfo::kFramesOffset defined in Torque and offset of field StackTraceInfo::frames in C++ do not match");
  static_assert(kSize == sizeof(StackTraceInfo));
};

// Definition https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=109&c=1
class TorqueGeneratedErrorStackDataAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=114&c=3
  static constexpr int kRawDataForCallSiteInfosOrFormattedStackOffset = sizeof(StructLayout);
  static constexpr int kRawDataForCallSiteInfosOrFormattedStackOffsetEnd = kRawDataForCallSiteInfosOrFormattedStackOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=116&c=3
  static constexpr int kStackTraceOffset = kRawDataForCallSiteInfosOrFormattedStackOffsetEnd + 1;
  static constexpr int kStackTraceOffsetEnd = kStackTraceOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kStackTraceOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kStackTraceOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kStackTraceOffsetEnd + 1;
  static constexpr int kHeaderSize = kStackTraceOffsetEnd + 1;
  static constexpr int kSize = kStackTraceOffsetEnd + 1;

  static_assert(kRawDataForCallSiteInfosOrFormattedStackOffset == offsetof(ErrorStackData, raw_data_for_call_site_infos_or_formatted_stack_),
                "Value of ErrorStackData::kRawDataForCallSiteInfosOrFormattedStackOffset defined in Torque and offset of field ErrorStackData::raw_data_for_call_site_infos_or_formatted_stack in C++ do not match");
  static_assert(kStackTraceOffset == offsetof(ErrorStackData, stack_trace_),
                "Value of ErrorStackData::kStackTraceOffset defined in Torque and offset of field ErrorStackData::stack_trace in C++ do not match");
  static_assert(kSize == sizeof(ErrorStackData));
};

} // namespace internal
} // namespace v8
