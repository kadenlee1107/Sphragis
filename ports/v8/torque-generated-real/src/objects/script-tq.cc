#include "src/objects/script.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/script.tq?l=19&c=1
class TorqueGeneratedScriptAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/script.tq?l=22&c=3
  static constexpr int kSourceOffset = sizeof(StructLayout);
  static constexpr int kSourceOffsetEnd = kSourceOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=25&c=3
  static constexpr int kNameOffset = kSourceOffsetEnd + 1;
  static constexpr int kNameOffsetEnd = kNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=28&c=3
  static constexpr int kLineOffsetOffset = kNameOffsetEnd + 1;
  static constexpr int kLineOffsetOffsetEnd = kLineOffsetOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=32&c=3
  static constexpr int kColumnOffsetOffset = kLineOffsetOffsetEnd + 1;
  static constexpr int kColumnOffsetOffsetEnd = kColumnOffsetOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=35&c=3
  static constexpr int kContextDataOffset = kColumnOffsetOffsetEnd + 1;
  static constexpr int kContextDataOffsetEnd = kContextDataOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=37&c=3
  static constexpr int kScriptTypeOffset = kContextDataOffsetEnd + 1;
  static constexpr int kScriptTypeOffsetEnd = kScriptTypeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=40&c=3
  static constexpr int kLineEndsOffset = kScriptTypeOffsetEnd + 1;
  static constexpr int kLineEndsOffsetEnd = kLineEndsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=43&c=3
  static constexpr int kIdOffset = kLineEndsOffsetEnd + 1;
  static constexpr int kIdOffsetEnd = kIdOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=48&c=3
  static constexpr int kEvalFromSharedOrWrappedArgumentsOffset = kIdOffsetEnd + 1;
  static constexpr int kEvalFromSharedOrWrappedArgumentsOffsetEnd = kEvalFromSharedOrWrappedArgumentsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=50&c=3
  static constexpr int kEvalFromPositionOffset = kEvalFromSharedOrWrappedArgumentsOffsetEnd + 1;
  static constexpr int kEvalFromPositionOffsetEnd = kEvalFromPositionOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=51&c=3
  static constexpr int kEvalFromScopeInfoOffset = kEvalFromPositionOffsetEnd + 1;
  static constexpr int kEvalFromScopeInfoOffsetEnd = kEvalFromScopeInfoOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=52&c=3
  static constexpr int kInfosOffset = kEvalFromScopeInfoOffsetEnd + 1;
  static constexpr int kInfosOffsetEnd = kInfosOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=56&c=3
  static constexpr int kCompiledLazyFunctionPositionsOffset = kInfosOffsetEnd + 1;
  static constexpr int kCompiledLazyFunctionPositionsOffsetEnd = kCompiledLazyFunctionPositionsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=59&c=3
  static constexpr int kFlagsOffset = kCompiledLazyFunctionPositionsOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=62&c=3
  static constexpr int kSourceUrlOffset = kFlagsOffsetEnd + 1;
  static constexpr int kSourceUrlOffsetEnd = kSourceUrlOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=65&c=3
  static constexpr int kSourceMappingUrlOffset = kSourceUrlOffsetEnd + 1;
  static constexpr int kSourceMappingUrlOffsetEnd = kSourceMappingUrlOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=68&c=3
  static constexpr int kDebugIdOffset = kSourceMappingUrlOffsetEnd + 1;
  static constexpr int kDebugIdOffsetEnd = kDebugIdOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=71&c=3
  static constexpr int kHostDefinedOptionsOffset = kDebugIdOffsetEnd + 1;
  static constexpr int kHostDefinedOptionsOffsetEnd = kHostDefinedOptionsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/script.tq?l=80&c=3
  static constexpr int kSourceHashOffset = kHostDefinedOptionsOffsetEnd + 1;
  static constexpr int kSourceHashOffsetEnd = kSourceHashOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kSourceHashOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kSourceHashOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kSourceHashOffsetEnd + 1;
  static constexpr int kHeaderSize = kSourceHashOffsetEnd + 1;
  static constexpr int kSize = kSourceHashOffsetEnd + 1;

  static_assert(kSourceOffset == offsetof(Script, source_),
                "Value of Script::kSourceOffset defined in Torque and offset of field Script::source in C++ do not match");
  static_assert(kNameOffset == offsetof(Script, name_),
                "Value of Script::kNameOffset defined in Torque and offset of field Script::name in C++ do not match");
  static_assert(kLineOffsetOffset == offsetof(Script, line_offset_),
                "Value of Script::kLineOffsetOffset defined in Torque and offset of field Script::line_offset in C++ do not match");
  static_assert(kColumnOffsetOffset == offsetof(Script, column_offset_),
                "Value of Script::kColumnOffsetOffset defined in Torque and offset of field Script::column_offset in C++ do not match");
  static_assert(kContextDataOffset == offsetof(Script, context_data_),
                "Value of Script::kContextDataOffset defined in Torque and offset of field Script::context_data in C++ do not match");
  static_assert(kScriptTypeOffset == offsetof(Script, script_type_),
                "Value of Script::kScriptTypeOffset defined in Torque and offset of field Script::script_type in C++ do not match");
  static_assert(kLineEndsOffset == offsetof(Script, line_ends_),
                "Value of Script::kLineEndsOffset defined in Torque and offset of field Script::line_ends in C++ do not match");
  static_assert(kIdOffset == offsetof(Script, id_),
                "Value of Script::kIdOffset defined in Torque and offset of field Script::id in C++ do not match");
  static_assert(kEvalFromSharedOrWrappedArgumentsOffset == offsetof(Script, eval_from_shared_or_wrapped_arguments_),
                "Value of Script::kEvalFromSharedOrWrappedArgumentsOffset defined in Torque and offset of field Script::eval_from_shared_or_wrapped_arguments in C++ do not match");
  static_assert(kEvalFromPositionOffset == offsetof(Script, eval_from_position_),
                "Value of Script::kEvalFromPositionOffset defined in Torque and offset of field Script::eval_from_position in C++ do not match");
  static_assert(kEvalFromScopeInfoOffset == offsetof(Script, eval_from_scope_info_),
                "Value of Script::kEvalFromScopeInfoOffset defined in Torque and offset of field Script::eval_from_scope_info in C++ do not match");
  static_assert(kInfosOffset == offsetof(Script, infos_),
                "Value of Script::kInfosOffset defined in Torque and offset of field Script::infos in C++ do not match");
  static_assert(kCompiledLazyFunctionPositionsOffset == offsetof(Script, compiled_lazy_function_positions_),
                "Value of Script::kCompiledLazyFunctionPositionsOffset defined in Torque and offset of field Script::compiled_lazy_function_positions in C++ do not match");
  static_assert(kFlagsOffset == offsetof(Script, flags_),
                "Value of Script::kFlagsOffset defined in Torque and offset of field Script::flags in C++ do not match");
  static_assert(kSourceUrlOffset == offsetof(Script, source_url_),
                "Value of Script::kSourceUrlOffset defined in Torque and offset of field Script::source_url in C++ do not match");
  static_assert(kSourceMappingUrlOffset == offsetof(Script, source_mapping_url_),
                "Value of Script::kSourceMappingUrlOffset defined in Torque and offset of field Script::source_mapping_url in C++ do not match");
  static_assert(kDebugIdOffset == offsetof(Script, debug_id_),
                "Value of Script::kDebugIdOffset defined in Torque and offset of field Script::debug_id in C++ do not match");
  static_assert(kHostDefinedOptionsOffset == offsetof(Script, host_defined_options_),
                "Value of Script::kHostDefinedOptionsOffset defined in Torque and offset of field Script::host_defined_options in C++ do not match");
  static_assert(kSourceHashOffset == offsetof(Script, source_hash_),
                "Value of Script::kSourceHashOffset defined in Torque and offset of field Script::source_hash in C++ do not match");
  static_assert(kSize == sizeof(Script));
};

} // namespace internal
} // namespace v8
