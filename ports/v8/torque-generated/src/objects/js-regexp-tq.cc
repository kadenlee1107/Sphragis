#include "src/objects/js-regexp-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/js-regexp.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=17&c=1
class TorqueGeneratedRegExpDataAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(ExposedTrustedObjectLayout);
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=19&c=3
  static constexpr int kTypeTagOffset = sizeof(ExposedTrustedObjectLayout);
  static constexpr int kTypeTagOffsetEnd = kTypeTagOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=20&c=3
  static constexpr int kOriginalSourceOffset = kTypeTagOffsetEnd + 1;
  static constexpr int kOriginalSourceOffsetEnd = kOriginalSourceOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=21&c=3
  static constexpr int kEscapedSourceOffset = kOriginalSourceOffsetEnd + 1;
  static constexpr int kEscapedSourceOffsetEnd = kEscapedSourceOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=22&c=3
  static constexpr int kFlagsOffset = kEscapedSourceOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=23&c=3
  static constexpr int kWrapperOffset = kFlagsOffsetEnd + 1;
  static constexpr int kWrapperOffsetEnd = kWrapperOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kWrapperOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=24&c=3
  static constexpr int kQuickCheckMaskOffset = kWrapperOffsetEnd + 1;
  static constexpr int kQuickCheckMaskOffsetEnd = kQuickCheckMaskOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=25&c=3
  static constexpr int kQuickCheckValueOffset = kQuickCheckMaskOffsetEnd + 1;
  static constexpr int kQuickCheckValueOffsetEnd = kQuickCheckValueOffset + kInt32Size - 1;
  static constexpr int kStartOfWeakFieldsOffset = kQuickCheckValueOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kQuickCheckValueOffsetEnd + 1;
  static constexpr int kHeaderSize = kQuickCheckValueOffsetEnd + 1;
  static constexpr int kSize = kQuickCheckValueOffsetEnd + 1;

  static_assert(kTypeTagOffset == offsetof(RegExpData, type_tag_),
                "Value of RegExpData::kTypeTagOffset defined in Torque and offset of field RegExpData::type_tag in C++ do not match");
  static_assert(kOriginalSourceOffset == offsetof(RegExpData, original_source_),
                "Value of RegExpData::kOriginalSourceOffset defined in Torque and offset of field RegExpData::original_source in C++ do not match");
  static_assert(kEscapedSourceOffset == offsetof(RegExpData, escaped_source_),
                "Value of RegExpData::kEscapedSourceOffset defined in Torque and offset of field RegExpData::escaped_source in C++ do not match");
  static_assert(kFlagsOffset == offsetof(RegExpData, flags_),
                "Value of RegExpData::kFlagsOffset defined in Torque and offset of field RegExpData::flags in C++ do not match");
  static_assert(kWrapperOffset == offsetof(RegExpData, wrapper_),
                "Value of RegExpData::kWrapperOffset defined in Torque and offset of field RegExpData::wrapper in C++ do not match");
  static_assert(kQuickCheckMaskOffset == offsetof(RegExpData, quick_check_mask_),
                "Value of RegExpData::kQuickCheckMaskOffset defined in Torque and offset of field RegExpData::quick_check_mask in C++ do not match");
  static_assert(kQuickCheckValueOffset == offsetof(RegExpData, quick_check_value_),
                "Value of RegExpData::kQuickCheckValueOffset defined in Torque and offset of field RegExpData::quick_check_value in C++ do not match");
  static_assert(kSize == sizeof(RegExpData));
};

// Definition https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=28&c=1
class TorqueGeneratedAtomRegExpDataAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(RegExpData);
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=30&c=3
  static constexpr int kPatternOffset = sizeof(RegExpData);
  static constexpr int kPatternOffsetEnd = kPatternOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPatternOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kPatternOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPatternOffsetEnd + 1;
  static constexpr int kHeaderSize = kPatternOffsetEnd + 1;
  static constexpr int kSize = kPatternOffsetEnd + 1;

  static_assert(kPatternOffset == offsetof(AtomRegExpData, pattern_),
                "Value of AtomRegExpData::kPatternOffset defined in Torque and offset of field AtomRegExpData::pattern in C++ do not match");
  static_assert(kSize == sizeof(AtomRegExpData));
};

// Definition https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=38&c=1
class TorqueGeneratedIrRegExpDataAsserts {
  static constexpr int kStartOfWeakFieldsOffset = sizeof(RegExpData);
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=42&c=3
  static constexpr int kLatin1BytecodeOffset = sizeof(RegExpData);
  static constexpr int kLatin1BytecodeOffsetEnd = kLatin1BytecodeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=43&c=3
  static constexpr int kUc16BytecodeOffset = kLatin1BytecodeOffsetEnd + 1;
  static constexpr int kUc16BytecodeOffsetEnd = kUc16BytecodeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=44&c=3
  static constexpr int kCaptureNameMapOffset = kUc16BytecodeOffsetEnd + 1;
  static constexpr int kCaptureNameMapOffsetEnd = kCaptureNameMapOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kCaptureNameMapOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=45&c=3
  static constexpr int kLatin1CodeOffset = kCaptureNameMapOffsetEnd + 1;
  static constexpr int kLatin1CodeOffsetEnd = kLatin1CodeOffset + kTrustedPointerSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=46&c=3
  static constexpr int kUc16CodeOffset = kLatin1CodeOffsetEnd + 1;
  static constexpr int kUc16CodeOffsetEnd = kUc16CodeOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfStrongFieldsOffset = kUc16CodeOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=47&c=3
  static constexpr int kMaxRegisterCountOffset = kUc16CodeOffsetEnd + 1;
  static constexpr int kMaxRegisterCountOffsetEnd = kMaxRegisterCountOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=48&c=3
  static constexpr int kCaptureCountOffset = kMaxRegisterCountOffsetEnd + 1;
  static constexpr int kCaptureCountOffsetEnd = kCaptureCountOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=49&c=3
  static constexpr int kTicksUntilTierUpOffset = kCaptureCountOffsetEnd + 1;
  static constexpr int kTicksUntilTierUpOffsetEnd = kTicksUntilTierUpOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=50&c=3
  static constexpr int kBacktrackLimitOffset = kTicksUntilTierUpOffsetEnd + 1;
  static constexpr int kBacktrackLimitOffsetEnd = kBacktrackLimitOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=51&c=3
  static constexpr int kBitFieldOffset = kBacktrackLimitOffsetEnd + 1;
  static constexpr int kBitFieldOffsetEnd = kBitFieldOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kBitFieldOffsetEnd + 1;
  static constexpr int kHeaderSize = kBitFieldOffsetEnd + 1;
  static constexpr int kSize = kBitFieldOffsetEnd + 1;

  static_assert(kLatin1BytecodeOffset == offsetof(IrRegExpData, latin1_bytecode_),
                "Value of IrRegExpData::kLatin1BytecodeOffset defined in Torque and offset of field IrRegExpData::latin1_bytecode in C++ do not match");
  static_assert(kUc16BytecodeOffset == offsetof(IrRegExpData, uc16_bytecode_),
                "Value of IrRegExpData::kUc16BytecodeOffset defined in Torque and offset of field IrRegExpData::uc16_bytecode in C++ do not match");
  static_assert(kCaptureNameMapOffset == offsetof(IrRegExpData, capture_name_map_),
                "Value of IrRegExpData::kCaptureNameMapOffset defined in Torque and offset of field IrRegExpData::capture_name_map in C++ do not match");
  static_assert(kLatin1CodeOffset == offsetof(IrRegExpData, latin1_code_),
                "Value of IrRegExpData::kLatin1CodeOffset defined in Torque and offset of field IrRegExpData::latin1_code in C++ do not match");
  static_assert(kUc16CodeOffset == offsetof(IrRegExpData, uc16_code_),
                "Value of IrRegExpData::kUc16CodeOffset defined in Torque and offset of field IrRegExpData::uc16_code in C++ do not match");
  static_assert(kMaxRegisterCountOffset == offsetof(IrRegExpData, max_register_count_),
                "Value of IrRegExpData::kMaxRegisterCountOffset defined in Torque and offset of field IrRegExpData::max_register_count in C++ do not match");
  static_assert(kCaptureCountOffset == offsetof(IrRegExpData, capture_count_),
                "Value of IrRegExpData::kCaptureCountOffset defined in Torque and offset of field IrRegExpData::capture_count in C++ do not match");
  static_assert(kTicksUntilTierUpOffset == offsetof(IrRegExpData, ticks_until_tier_up_),
                "Value of IrRegExpData::kTicksUntilTierUpOffset defined in Torque and offset of field IrRegExpData::ticks_until_tier_up in C++ do not match");
  static_assert(kBacktrackLimitOffset == offsetof(IrRegExpData, backtrack_limit_),
                "Value of IrRegExpData::kBacktrackLimitOffset defined in Torque and offset of field IrRegExpData::backtrack_limit in C++ do not match");
  static_assert(kBitFieldOffset == offsetof(IrRegExpData, bit_field_),
                "Value of IrRegExpData::kBitFieldOffset defined in Torque and offset of field IrRegExpData::bit_field in C++ do not match");
  static_assert(kSize == sizeof(IrRegExpData));
};

// Definition https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=54&c=1
class TorqueGeneratedRegExpDataWrapperAsserts {
  // https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=56&c=3
  static constexpr int kDataOffset = sizeof(StructLayout);
  static constexpr int kDataOffsetEnd = kDataOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kHeaderSize = kDataOffsetEnd + 1;
  static constexpr int kSize = kDataOffsetEnd + 1;

  static_assert(kDataOffset == offsetof(RegExpDataWrapper, data_),
                "Value of RegExpDataWrapper::kDataOffset defined in Torque and offset of field RegExpDataWrapper::data in C++ do not match");
  static_assert(kSize == sizeof(RegExpDataWrapper));
};

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=59&c=1
bool IsJSRegExp_NonInline(Tagged<HeapObject> o) {
  return IsJSRegExp(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSRegExp<JSRegExp, JSObject>::JSRegExpVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSRegExpVerify(TrustedCast<JSRegExp>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
