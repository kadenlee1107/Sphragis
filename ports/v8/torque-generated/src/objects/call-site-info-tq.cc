#include "src/objects/call-site-info.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=24&c=1
class TorqueGeneratedCallSiteInfoAsserts {
  // https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=28&c=3
  static constexpr int kCodeObjectOffset = sizeof(StructLayout);
  static constexpr int kCodeObjectOffsetEnd = kCodeObjectOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfStrongFieldsOffset = kCodeObjectOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=29&c=3
  static constexpr int kReceiverOrInstanceOffset = kCodeObjectOffsetEnd + 1;
  static constexpr int kReceiverOrInstanceOffsetEnd = kReceiverOrInstanceOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=30&c=3
  static constexpr int kFunctionOffset = kReceiverOrInstanceOffsetEnd + 1;
  static constexpr int kFunctionOffsetEnd = kFunctionOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=31&c=3
  static constexpr int kCodeOffsetOrSourcePositionOffset = kFunctionOffsetEnd + 1;
  static constexpr int kCodeOffsetOrSourcePositionOffsetEnd = kCodeOffsetOrSourcePositionOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=32&c=3
  static constexpr int kFlagsOffset = kCodeOffsetOrSourcePositionOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;
  static constexpr int kSize = kFlagsOffsetEnd + 1;

  static_assert(kCodeObjectOffset == offsetof(CallSiteInfo, code_object_),
                "Value of CallSiteInfo::kCodeObjectOffset defined in Torque and offset of field CallSiteInfo::code_object in C++ do not match");
  static_assert(kReceiverOrInstanceOffset == offsetof(CallSiteInfo, receiver_or_instance_),
                "Value of CallSiteInfo::kReceiverOrInstanceOffset defined in Torque and offset of field CallSiteInfo::receiver_or_instance in C++ do not match");
  static_assert(kFunctionOffset == offsetof(CallSiteInfo, function_),
                "Value of CallSiteInfo::kFunctionOffset defined in Torque and offset of field CallSiteInfo::function in C++ do not match");
  static_assert(kCodeOffsetOrSourcePositionOffset == offsetof(CallSiteInfo, code_offset_or_source_position_),
                "Value of CallSiteInfo::kCodeOffsetOrSourcePositionOffset defined in Torque and offset of field CallSiteInfo::code_offset_or_source_position in C++ do not match");
  static_assert(kFlagsOffset == offsetof(CallSiteInfo, flags_),
                "Value of CallSiteInfo::kFlagsOffset defined in Torque and offset of field CallSiteInfo::flags in C++ do not match");
  static_assert(kSize == sizeof(CallSiteInfo));
};

} // namespace internal
} // namespace v8
