#ifndef V8_GEN_TORQUE_GENERATED_INSTANCE_TYPES_H_
#define V8_GEN_TORQUE_GENERATED_INSTANCE_TYPES_H_

// Instance types for all classes except for those that use InstanceType as flags.
#define TORQUE_ASSIGNED_INSTANCE_TYPES(V) \
  V(FIRST_HEAP_OBJECT_TYPE, 0) \
    V(FIRST_PRIMITIVE_HEAP_OBJECT_TYPE, 0) \
      V(FIRST_NAME_TYPE, 0) \
        V(FIRST_STRING_TYPE, 0) \
        V(SYMBOL_TYPE, 128) /* https://crsrc.org/c/v8/src/objects/name.tq?l=35&c=1 */\
      V(LAST_NAME_TYPE, 128) \
      V(BIG_INT_BASE_TYPE, 129) /* https://crsrc.org/c/v8/src/objects/bigint.tq?l=7&c=1 */\
      V(HEAP_NUMBER_TYPE, 130) /* https://crsrc.org/c/v8/src/objects/heap-number.tq?l=5&c=1 */\
      V(ODDBALL_TYPE, 131) /* https://crsrc.org/c/v8/src/objects/oddball.tq?l=5&c=1 */\
    V(LAST_PRIMITIVE_HEAP_OBJECT_TYPE, 131) \
    V(FIRST_STRUCT_TYPE, 132) \
      V(FIRST_MICROTASK_TYPE, 132) \
        V(FIRST_PROMISE_REACTION_JOB_TASK_TYPE, 132) \
          V(PROMISE_FULFILL_REACTION_JOB_TASK_TYPE, 132) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=70&c=1 */\
          V(PROMISE_REJECT_REACTION_JOB_TASK_TYPE, 133) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=73&c=1 */\
        V(LAST_PROMISE_REACTION_JOB_TASK_TYPE, 133) \
        V(ASYNC_RESUME_TASK_TYPE, 134) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=27&c=1 */\
        V(CALLABLE_TASK_TYPE, 135) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=18&c=1 */\
        V(CALLBACK_TASK_TYPE, 136) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=12&c=1 */\
        V(PROMISE_RESOLVE_THENABLE_JOB_TASK_TYPE, 137) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=76&c=1 */\
      V(LAST_MICROTASK_TYPE, 137) \
      V(FIRST_DATA_HANDLER_TYPE, 138) \
        V(LOAD_HANDLER_TYPE, 138) /* https://crsrc.org/c/v8/src/ic/handler-configuration.tq?l=7&c=1 */\
        V(STORE_HANDLER_TYPE, 139) /* https://crsrc.org/c/v8/src/ic/handler-configuration.tq?l=8&c=1 */\
      V(LAST_DATA_HANDLER_TYPE, 139) \
      V(ACCESS_CHECK_INFO_TYPE, 140) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=29&c=1 */\
      V(ACCESSOR_PAIR_TYPE, 141) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=12&c=1 */\
      V(ALIASED_ARGUMENTS_ENTRY_TYPE, 142) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=41&c=1 */\
      V(ALLOCATION_MEMENTO_TYPE, 143) /* https://crsrc.org/c/v8/src/objects/allocation-site.tq?l=7&c=1 */\
      V(ARRAY_BOILERPLATE_DESCRIPTION_TYPE, 144) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=13&c=1 */\
      V(ASM_WASM_DATA_TYPE, 145) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1 */\
      V(ASYNC_GENERATOR_REQUEST_TYPE, 146) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=44&c=1 */\
      V(BREAK_POINT_TYPE, 147) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=5&c=1 */\
      V(BREAK_POINT_INFO_TYPE, 148) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=11&c=1 */\
      V(BYTECODE_WRAPPER_TYPE, 149) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=26&c=1 */\
      V(CALL_SITE_INFO_TYPE, 150) /* https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=24&c=1 */\
      V(CLASS_BOILERPLATE_TYPE, 151) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=25&c=1 */\
      V(CLASS_POSITIONS_TYPE, 152) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=10&c=1 */\
      V(CODE_WRAPPER_TYPE, 153) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=33&c=1 */\
      V(DEBUG_INFO_TYPE, 154) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=35&c=1 */\
      V(ENUM_CACHE_TYPE, 155) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=5&c=1 */\
      V(ERROR_STACK_DATA_TYPE, 156) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=109&c=1 */\
      V(FEEDBACK_CELL_TYPE, 157) /* https://crsrc.org/c/v8/src/objects/feedback-cell.tq?l=5&c=1 */\
      V(FUNCTION_TEMPLATE_RARE_DATA_TYPE, 158) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=23&c=1 */\
      V(MODULE_REQUEST_TYPE, 159) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=59&c=1 */\
      V(PROMISE_CAPABILITY_TYPE, 160) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=5&c=1 */\
      V(PROMISE_REACTION_TYPE, 161) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=34&c=1 */\
      V(PROPERTY_DESCRIPTOR_OBJECT_TYPE, 162) /* https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=19&c=1 */\
      V(PROTOTYPE_INFO_TYPE, 163) /* https://crsrc.org/c/v8/src/objects/prototype-info.tq?l=12&c=1 */\
      V(PROTOTYPE_SHARED_CLOSURE_INFO_TYPE, 164) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=36&c=1 */\
      V(REG_EXP_BOILERPLATE_DESCRIPTION_TYPE, 165) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=19&c=1 */\
      V(REG_EXP_DATA_WRAPPER_TYPE, 166) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=54&c=1 */\
      V(SCRIPT_TYPE, 167) /* https://crsrc.org/c/v8/src/objects/script.tq?l=19&c=1 */\
      V(SCRIPT_OR_MODULE_TYPE, 168) /* https://crsrc.org/c/v8/src/objects/module.tq?l=28&c=1 */\
      V(SOURCE_TEXT_MODULE_INFO_ENTRY_TYPE, 169) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=70&c=1 */\
      V(STACK_FRAME_INFO_TYPE, 170) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=75&c=1 */\
      V(STACK_TRACE_INFO_TYPE, 171) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=88&c=1 */\
      V(TEMPLATE_OBJECT_DESCRIPTION_TYPE, 172) /* https://crsrc.org/c/v8/src/objects/template-objects.tq?l=5&c=1 */\
      V(TUPLE2_TYPE, 173) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=8&c=1 */\
      V(WASM_EXCEPTION_TAG_TYPE, 174) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1 */\
    V(LAST_STRUCT_TYPE, 174) \
    V(FIRST_TRUSTED_OBJECT_TYPE, 175) \
      V(FIRST_EXPOSED_TRUSTED_OBJECT_TYPE, 175) \
        V(FIRST_UNCOMPILED_DATA_TYPE, 175) \
          V(FIRST_UNCOMPILED_DATA_WITH_PREPARSE_DATA_TYPE, 175) \
            V(UNCOMPILED_DATA_WITH_PREPARSE_DATA_TYPE, 175) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=172&c=1 */\
            V(UNCOMPILED_DATA_WITH_PREPARSE_DATA_AND_JOB_TYPE, 176) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=183&c=1 */\
          V(LAST_UNCOMPILED_DATA_WITH_PREPARSE_DATA_TYPE, 176) \
          V(FIRST_UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_TYPE, 177) \
            V(UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_TYPE, 177) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=169&c=1 */\
            V(UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_WITH_JOB_TYPE, 178) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=177&c=1 */\
          V(LAST_UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_TYPE, 178) \
        V(LAST_UNCOMPILED_DATA_TYPE, 178) \
        V(FIRST_WASM_FUNCTION_DATA_TYPE, 179) \
          V(WASM_FUNCTION_DATA_TYPE, 179) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1 */\
          V(WASM_CAPI_FUNCTION_DATA_TYPE, 180) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1 */\
          V(WASM_EXPORTED_FUNCTION_DATA_TYPE, 181) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1 */\
          V(WASM_JS_FUNCTION_DATA_TYPE, 182) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1 */\
        V(LAST_WASM_FUNCTION_DATA_TYPE, 182) \
        V(FIRST_REG_EXP_DATA_TYPE, 183) \
          V(REG_EXP_DATA_TYPE, 183) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=17&c=1 */\
          V(ATOM_REG_EXP_DATA_TYPE, 184) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=28&c=1 */\
          V(IR_REG_EXP_DATA_TYPE, 185) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=38&c=1 */\
        V(LAST_REG_EXP_DATA_TYPE, 185) \
        V(BYTECODE_ARRAY_TYPE, 186) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=7&c=1 */\
        V(CODE_TYPE, 187) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=31&c=1 */\
        V(INTERPRETER_DATA_TYPE, 188) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=14&c=1 */\
        V(WASM_DISPATCH_TABLE_TYPE, 189) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=202&c=1 */\
        V(WASM_INTERNAL_FUNCTION_TYPE, 190) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1 */\
        V(WASM_SUSPENDER_OBJECT_TYPE, 191) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1 */\
        V(WASM_TRUSTED_INSTANCE_DATA_TYPE, 192) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=37&c=1 */\
      V(LAST_EXPOSED_TRUSTED_OBJECT_TYPE, 192) \
      V(INSTRUCTION_STREAM_TYPE, 193) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=293&c=1 */\
      V(PROTECTED_FIXED_ARRAY_TYPE, 194) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=25&c=1 */\
      V(PROTECTED_WEAK_FIXED_ARRAY_TYPE, 195) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=54&c=1 */\
      V(SHARED_FUNCTION_INFO_WRAPPER_TYPE, 196) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=130&c=1 */\
      V(TRUSTED_BYTE_ARRAY_TYPE, 197) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=67&c=1 */\
      V(TRUSTED_FIXED_ARRAY_TYPE, 198) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=19&c=1 */\
      V(TRUSTED_FOREIGN_TYPE, 199) /* https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1 */\
      V(TRUSTED_WEAK_FIXED_ARRAY_TYPE, 200) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=48&c=1 */\
      V(WASM_DISPATCH_TABLE_FOR_IMPORTS_TYPE, 201) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=203&c=1 */\
      V(WASM_IMPORT_DATA_TYPE, 202) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1 */\
    V(LAST_TRUSTED_OBJECT_TYPE, 202) \
    V(ACCESSOR_INFO_TYPE, 203) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1 */\
    V(FOREIGN_TYPE, 204) /* https://crsrc.org/c/v8/src/objects/foreign.tq?l=5&c=1 */\
    V(FIRST_FIXED_ARRAY_BASE_TYPE, 205) \
      V(FIRST_FIXED_ARRAY_TYPE, 205) \
        V(FIXED_ARRAY_TYPE, 205) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=11&c=1 */\
        V(FIRST_HASH_TABLE_TYPE, 206) \
          V(HASH_TABLE_TYPE, 206) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=264&c=1 */\
          V(EPHEMERON_HASH_TABLE_TYPE, 207) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=275&c=1 */\
          V(GLOBAL_DICTIONARY_TYPE, 208) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=272&c=1 */\
          V(NAME_DICTIONARY_TYPE, 209) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=271&c=1 */\
          V(NAME_TO_INDEX_HASH_TABLE_TYPE, 210) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=269&c=1 */\
          V(NUMBER_DICTIONARY_TYPE, 211) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=278&c=1 */\
          V(ORDERED_HASH_MAP_TYPE, 212) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=266&c=1 */\
          V(ORDERED_HASH_SET_TYPE, 213) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=267&c=1 */\
          V(ORDERED_NAME_DICTIONARY_TYPE, 214) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=268&c=1 */\
          V(REGISTERED_SYMBOL_TABLE_TYPE, 215) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=270&c=1 */\
          V(SIMPLE_NAME_DICTIONARY_TYPE, 216) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=273&c=1 */\
          V(SIMPLE_NUMBER_DICTIONARY_TYPE, 217) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=274&c=1 */\
        V(LAST_HASH_TABLE_TYPE, 217) \
      V(LAST_FIXED_ARRAY_TYPE, 217) \
      V(BYTE_ARRAY_TYPE, 218) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=62&c=1 */\
      V(FIXED_DOUBLE_ARRAY_TYPE, 219) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=31&c=1 */\
      V(SLOPPY_ARGUMENTS_ELEMENTS_TYPE, 220) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=27&c=1 */\
    V(LAST_FIXED_ARRAY_BASE_TYPE, 220) \
    V(FIRST_CONTEXT_TYPE, 221) \
      V(AWAIT_CONTEXT_TYPE, 221) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=49&c=1 */\
      V(BLOCK_CONTEXT_TYPE, 222) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=50&c=1 */\
      V(CATCH_CONTEXT_TYPE, 223) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=51&c=1 */\
      V(DEBUG_EVALUATE_CONTEXT_TYPE, 224) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=52&c=1 */\
      V(EVAL_CONTEXT_TYPE, 225) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=54&c=1 */\
      V(FUNCTION_CONTEXT_TYPE, 226) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=59&c=1 */\
      V(MODULE_CONTEXT_TYPE, 227) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=55&c=1 */\
      V(NATIVE_CONTEXT_TYPE, 228) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=92&c=1 */\
      V(SCRIPT_CONTEXT_TYPE, 229) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=56&c=1 */\
      V(WITH_CONTEXT_TYPE, 230) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=57&c=1 */\
    V(LAST_CONTEXT_TYPE, 230) \
    V(FIRST_TURBOSHAFT_TYPE_TYPE, 231) \
      V(FIRST_TURBOSHAFT_FLOAT64_TYPE_TYPE, 231) \
        V(TURBOSHAFT_FLOAT64_TYPE_TYPE, 231) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=65&c=1 */\
        V(TURBOSHAFT_FLOAT64_RANGE_TYPE_TYPE, 232) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=72&c=1 */\
        V(TURBOSHAFT_FLOAT64_SET_TYPE_TYPE, 233) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=81&c=1 */\
      V(LAST_TURBOSHAFT_FLOAT64_TYPE_TYPE, 233) \
      V(FIRST_TURBOSHAFT_WORD32_TYPE_TYPE, 234) \
        V(TURBOSHAFT_WORD32_TYPE_TYPE, 234) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=20&c=1 */\
        V(TURBOSHAFT_WORD32_RANGE_TYPE_TYPE, 235) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=25&c=1 */\
        V(TURBOSHAFT_WORD32_SET_TYPE_TYPE, 236) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=33&c=1 */\
      V(LAST_TURBOSHAFT_WORD32_TYPE_TYPE, 236) \
      V(FIRST_TURBOSHAFT_WORD64_TYPE_TYPE, 237) \
        V(TURBOSHAFT_WORD64_TYPE_TYPE, 237) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=41&c=1 */\
        V(TURBOSHAFT_WORD64_RANGE_TYPE_TYPE, 238) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=46&c=1 */\
        V(TURBOSHAFT_WORD64_SET_TYPE_TYPE, 239) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=56&c=1 */\
      V(LAST_TURBOSHAFT_WORD64_TYPE_TYPE, 239) \
    V(LAST_TURBOSHAFT_TYPE_TYPE, 239) \
    V(FIRST_TURBOFAN_TYPE_TYPE, 240) \
      V(TURBOFAN_BITSET_TYPE_TYPE, 240) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=61&c=1 */\
      V(TURBOFAN_HEAP_CONSTANT_TYPE_TYPE, 241) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=79&c=1 */\
      V(TURBOFAN_OTHER_NUMBER_CONSTANT_TYPE_TYPE, 242) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=84&c=1 */\
      V(TURBOFAN_RANGE_TYPE_TYPE, 243) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=73&c=1 */\
      V(TURBOFAN_UNION_TYPE_TYPE, 244) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=67&c=1 */\
    V(LAST_TURBOFAN_TYPE_TYPE, 244) \
    V(FIRST_EXPORTED_SUB_CLASS_BASE_TYPE, 245) \
      V(EXPORTED_SUB_CLASS_BASE_TYPE, 245) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1181&c=1 */\
      V(EXPORTED_SUB_CLASS_TYPE, 246) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1174&c=1 */\
      V(EXPORTED_SUB_CLASS2_TYPE, 247) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1216&c=1 */\
    V(LAST_EXPORTED_SUB_CLASS_BASE_TYPE, 247) \
    V(FIRST_SMALL_ORDERED_HASH_TABLE_TYPE, 248) \
      V(SMALL_ORDERED_HASH_MAP_TYPE, 248) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=64&c=1 */\
      V(SMALL_ORDERED_HASH_SET_TYPE, 249) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=25&c=1 */\
      V(SMALL_ORDERED_NAME_DICTIONARY_TYPE, 250) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=101&c=1 */\
    V(LAST_SMALL_ORDERED_HASH_TABLE_TYPE, 250) \
    V(FIRST_TEMPLATE_INFO_TYPE, 251) \
      V(FIRST_TEMPLATE_INFO_WITH_PROPERTIES_TYPE, 251) \
        V(FUNCTION_TEMPLATE_INFO_TYPE, 251) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1 */\
        V(OBJECT_TEMPLATE_INFO_TYPE, 252) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1 */\
      V(LAST_TEMPLATE_INFO_WITH_PROPERTIES_TYPE, 252) \
      V(DICTIONARY_TEMPLATE_INFO_TYPE, 253) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1 */\
    V(LAST_TEMPLATE_INFO_TYPE, 253) \
    V(FIRST_ABSTRACT_INTERNAL_CLASS_TYPE, 254) \
      V(ABSTRACT_INTERNAL_CLASS_SUBCLASS1_TYPE, 254) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1190&c=1 */\
      V(ABSTRACT_INTERNAL_CLASS_SUBCLASS2_TYPE, 255) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1192&c=1 */\
    V(LAST_ABSTRACT_INTERNAL_CLASS_TYPE, 255) \
    V(FIRST_DESCRIPTOR_ARRAY_TYPE, 256) \
      V(DESCRIPTOR_ARRAY_TYPE, 256) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1 */\
      V(STRONG_DESCRIPTOR_ARRAY_TYPE, 257) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=44&c=1 */\
    V(LAST_DESCRIPTOR_ARRAY_TYPE, 257) \
    V(FIRST_MODULE_TYPE, 258) \
      V(SOURCE_TEXT_MODULE_TYPE, 258) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=12&c=1 */\
      V(SYNTHETIC_MODULE_TYPE, 259) /* https://crsrc.org/c/v8/src/objects/synthetic-module.tq?l=5&c=1 */\
    V(LAST_MODULE_TYPE, 259) \
    V(FIRST_WEAK_FIXED_ARRAY_TYPE, 260) \
      V(WEAK_FIXED_ARRAY_TYPE, 260) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=36&c=1 */\
      V(TRANSITION_ARRAY_TYPE, 261) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=299&c=1 */\
    V(LAST_WEAK_FIXED_ARRAY_TYPE, 261) \
    V(ALLOCATION_SITE_TYPE, 262) /* https://crsrc.org/c/v8/src/objects/allocation-site.tq?l=5&c=1 */\
    V(ARRAY_LIST_TYPE, 263) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=75&c=1 */\
    V(CELL_TYPE, 264) /* https://crsrc.org/c/v8/src/objects/cell.tq?l=5&c=1 */\
    V(CLOSURE_FEEDBACK_CELL_ARRAY_TYPE, 265) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=27&c=1 */\
    V(CONTEXT_CELL_TYPE, 266) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=25&c=1 */\
    V(COVERAGE_INFO_TYPE, 267) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1 */\
    V(CPP_HEAP_EXTERNAL_OBJECT_TYPE, 268) /* https://crsrc.org/c/v8/src/objects/cpp-heap-external-object.tq?l=5&c=1 */\
    V(DOUBLE_STRING_CACHE_TYPE, 269) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=279&c=1 */\
    V(EMBEDDER_DATA_ARRAY_TYPE, 270) /* https://crsrc.org/c/v8/src/objects/embedder-data-array.tq?l=5&c=1 */\
    V(FEEDBACK_METADATA_TYPE, 271) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=47&c=1 */\
    V(FEEDBACK_VECTOR_TYPE, 272) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=33&c=1 */\
    V(FILLER_TYPE, 273) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=690&c=1 */\
    V(FREE_SPACE_TYPE, 274) /* https://crsrc.org/c/v8/src/objects/free-space.tq?l=5&c=1 */\
    V(HOLE_TYPE, 275) /* https://crsrc.org/c/v8/src/objects/hole.tq?l=5&c=1 */\
    V(INTERCEPTOR_INFO_TYPE, 276) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1 */\
    V(INTERNAL_CLASS_TYPE, 277) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=763&c=1 */\
    V(INTERNAL_CLASS_WITH_STRUCT_ELEMENTS_TYPE, 278) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1199&c=1 */\
    V(MAP_TYPE, 279) /* https://crsrc.org/c/v8/src/objects/map.tq?l=37&c=1 */\
    V(MEGA_DOM_HANDLER_TYPE, 280) /* https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=5&c=1 */\
    V(OBJECT_BOILERPLATE_DESCRIPTION_TYPE, 281) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=5&c=1 */\
    V(ON_HEAP_BASIC_BLOCK_PROFILER_DATA_TYPE, 282) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1 */\
    V(PREPARSE_DATA_TYPE, 283) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=7&c=1 */\
    V(PROPERTY_ARRAY_TYPE, 284) /* https://crsrc.org/c/v8/src/objects/property-array.tq?l=5&c=1 */\
    V(PROPERTY_CELL_TYPE, 285) /* https://crsrc.org/c/v8/src/objects/property-cell.tq?l=6&c=1 */\
    V(REG_EXP_MATCH_INFO_TYPE, 286) /* https://crsrc.org/c/v8/src/objects/regexp-match-info.tq?l=5&c=1 */\
    V(SCOPE_INFO_TYPE, 287) /* https://crsrc.org/c/v8/src/objects/scope-info.tq?l=122&c=1 */\
    V(SCRIPT_CONTEXT_TABLE_TYPE, 288) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=17&c=1 */\
    V(SHARED_FUNCTION_INFO_TYPE, 289) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1 */\
    V(SMI_BOX_TYPE, 290) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=930&c=1 */\
    V(SMI_PAIR_TYPE, 291) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=829&c=1 */\
    V(SORT_STATE_TYPE, 292) /* https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=19&c=1 */\
    V(SWISS_NAME_DICTIONARY_TYPE, 293) /* https://crsrc.org/c/v8/src/objects/swiss-name-dictionary.tq?l=7&c=1 */\
    V(WASM_CONTINUATION_OBJECT_TYPE, 294) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1 */\
    V(WASM_FAST_API_CALL_DATA_TYPE, 295) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=73&c=1 */\
    V(WASM_FUNC_REF_TYPE, 296) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1 */\
    V(WASM_NULL_TYPE, 297) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1 */\
    V(WASM_RESUME_DATA_TYPE, 298) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1 */\
    V(WASM_STACK_OBJECT_TYPE, 299) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1 */\
    V(WASM_STRING_VIEW_ITER_TYPE, 300) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=325&c=1 */\
    V(WASM_TYPE_INFO_TYPE, 301) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1 */\
    V(WEAK_ARRAY_LIST_TYPE, 302) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=82&c=1 */\
    V(WEAK_CELL_TYPE, 303) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=21&c=1 */\
    V(WEAK_HOMOMORPHIC_FIXED_ARRAY_TYPE, 304) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=42&c=1 */\
    V(FIRST_JS_RECEIVER_TYPE, 305) \
      V(FIRST_WASM_OBJECT_TYPE, 305) \
        V(WASM_ARRAY_TYPE, 305) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1 */\
        V(WASM_STRUCT_TYPE, 306) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1 */\
      V(LAST_WASM_OBJECT_TYPE, 306) \
      V(JS_PROXY_TYPE, 307) /* https://crsrc.org/c/v8/src/objects/js-proxy.tq?l=5&c=1 */\
      V(FIRST_JS_OBJECT_TYPE, 308) \
        V(JS_OBJECT_TYPE, 1057) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=14&c=1 */\
        V(FIRST_JS_CUSTOM_ELEMENTS_OBJECT_TYPE, 308) \
          V(FIRST_JS_SPECIAL_OBJECT_TYPE, 308) \
            V(FIRST_JS_MODULE_NAMESPACE_TYPE, 308) \
              V(JS_MODULE_NAMESPACE_TYPE, 308) /* https://crsrc.org/c/v8/src/objects/module.tq?l=22&c=1 */\
              V(JS_DEFERRED_MODULE_NAMESPACE_TYPE, 309) /* https://crsrc.org/c/v8/src/objects/module.tq?l=26&c=1 */\
            V(LAST_JS_MODULE_NAMESPACE_TYPE, 309) \
            V(JS_GLOBAL_OBJECT_TYPE, 310) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=119&c=1 */\
            V(JS_GLOBAL_PROXY_TYPE, 311) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=117&c=1 */\
            V(JS_SPECIAL_API_OBJECT_TYPE, 1040) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=710&c=1 */\
          V(LAST_JS_SPECIAL_OBJECT_TYPE, 1040) \
          V(JS_PRIMITIVE_WRAPPER_TYPE, 1041) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=145&c=1 */\
        V(LAST_JS_CUSTOM_ELEMENTS_OBJECT_TYPE, 1041) \
        V(FIRST_JS_APIOBJECT_WITH_EMBEDDER_SLOTS_TYPE, 1058) \
          V(FIRST_JS_API_OBJECT_TYPE, 1058) \
            V(JS_API_OBJECT_TYPE, 1058) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=696&c=1 */\
            V(JS_LAST_DUMMY_API_OBJECT_TYPE, 2058) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=703&c=1 */\
          V(LAST_JS_API_OBJECT_TYPE, 2058) \
          V(FIRST_JS_ARRAY_BUFFER_VIEW_TYPE, 2059) \
            V(FIRST_JS_DATA_VIEW_OR_RAB_GSAB_DATA_VIEW_TYPE, 2059) \
              V(JS_DATA_VIEW_TYPE, 2059) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=145&c=1 */\
              V(JS_RAB_GSAB_DATA_VIEW_TYPE, 2060) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=147&c=1 */\
            V(LAST_JS_DATA_VIEW_OR_RAB_GSAB_DATA_VIEW_TYPE, 2060) \
            V(FIRST_JS_TYPED_ARRAY_TYPE, 2061) \
              V(JS_TYPED_ARRAY_TYPE, 2061) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=118&c=1 */\
              V(JS_DETACHED_TYPED_ARRAY_TYPE, 2062) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=129&c=1 */\
            V(LAST_JS_TYPED_ARRAY_TYPE, 2062) \
          V(LAST_JS_ARRAY_BUFFER_VIEW_TYPE, 2062) \
          V(JS_ARRAY_BUFFER_TYPE, 2063) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=18&c=1 */\
        V(LAST_JS_APIOBJECT_WITH_EMBEDDER_SLOTS_TYPE, 2063) \
        V(FIRST_JS_FUNCTION_OR_BOUND_FUNCTION_OR_WRAPPED_FUNCTION_TYPE, 2064) \
          V(FIRST_JS_FUNCTION_TYPE, 2064) \
            V(JS_FUNCTION_WITHOUT_PROTOTYPE_TYPE, 2064) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=52&c=1 */\
            V(FIRST_JS_FUNCTION_WITH_PROTOTYPE_TYPE, 2065) \
              V(JS_FUNCTION_WITH_PROTOTYPE_TYPE, 2065) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=55&c=1 */\
              V(FIRST_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2066) \
                V(BIGINT64_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2066) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=186&c=1 */\
                V(BIGUINT64_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2067) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=183&c=1 */\
                V(FLOAT16_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2068) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=171&c=1 */\
                V(FLOAT32_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2069) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=174&c=1 */\
                V(FLOAT64_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2070) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=177&c=1 */\
                V(INT16_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2071) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=162&c=1 */\
                V(INT32_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2072) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=168&c=1 */\
                V(INT8_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2073) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=156&c=1 */\
                V(UINT16_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2074) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=159&c=1 */\
                V(UINT32_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2075) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=165&c=1 */\
                V(UINT8_CLAMPED_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2076) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=180&c=1 */\
                V(UINT8_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2077) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=153&c=1 */\
              V(LAST_TYPED_ARRAY_CONSTRUCTOR_TYPE, 2077) \
              V(JS_ARRAY_CONSTRUCTOR_TYPE, 2078) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=76&c=1 */\
              V(JS_PROMISE_CONSTRUCTOR_TYPE, 2079) /* https://crsrc.org/c/v8/src/objects/js-promise.tq?l=38&c=1 */\
              V(JS_REG_EXP_CONSTRUCTOR_TYPE, 2080) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=95&c=1 */\
              V(JS_CLASS_CONSTRUCTOR_TYPE, 2081) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=63&c=1 */\
            V(LAST_JS_FUNCTION_WITH_PROTOTYPE_TYPE, 2081) \
          V(LAST_JS_FUNCTION_TYPE, 2081) \
          V(JS_BOUND_FUNCTION_TYPE, 2082) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=8&c=1 */\
          V(JS_WRAPPED_FUNCTION_TYPE, 2083) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=20&c=1 */\
        V(LAST_JS_FUNCTION_OR_BOUND_FUNCTION_OR_WRAPPED_FUNCTION_TYPE, 2083) \
        V(FIRST_JS_INTERNAL_PROTOTYPE_BASE_TYPE, 2084) \
          V(JS_ARRAY_ITERATOR_PROTOTYPE_TYPE, 2084) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=248&c=1 */\
          V(JS_ITERATOR_PROTOTYPE_TYPE, 2085) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=245&c=1 */\
          V(JS_MAP_ITERATOR_PROTOTYPE_TYPE, 2086) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=251&c=1 */\
          V(JS_OBJECT_PROTOTYPE_TYPE, 2087) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=230&c=1 */\
          V(JS_PROMISE_PROTOTYPE_TYPE, 2088) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=236&c=1 */\
          V(JS_REG_EXP_PROTOTYPE_TYPE, 2089) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=233&c=1 */\
          V(JS_SET_ITERATOR_PROTOTYPE_TYPE, 2090) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=254&c=1 */\
          V(JS_SET_PROTOTYPE_TYPE, 2091) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=242&c=1 */\
          V(JS_STRING_ITERATOR_PROTOTYPE_TYPE, 2092) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=257&c=1 */\
          V(JS_TYPED_ARRAY_PROTOTYPE_TYPE, 2093) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=239&c=1 */\
        V(LAST_JS_INTERNAL_PROTOTYPE_BASE_TYPE, 2093) \
        V(FIRST_JS_ITERATOR_HELPER_TYPE, 2094) \
          V(FIRST_JS_ITERATOR_HELPER_SIMPLE_TYPE, 2094) \
            V(JS_ITERATOR_CONCAT_HELPER_TYPE, 2094) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=48&c=1 */\
            V(JS_ITERATOR_DROP_HELPER_TYPE, 2095) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=38&c=1 */\
            V(JS_ITERATOR_FILTER_HELPER_TYPE, 2096) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=29&c=1 */\
            V(JS_ITERATOR_FLAT_MAP_HELPER_TYPE, 2097) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=42&c=1 */\
            V(JS_ITERATOR_MAP_HELPER_TYPE, 2098) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=24&c=1 */\
            V(JS_ITERATOR_TAKE_HELPER_TYPE, 2099) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=34&c=1 */\
          V(LAST_JS_ITERATOR_HELPER_SIMPLE_TYPE, 2099) \
          V(JS_ITERATOR_ZIP_HELPER_TYPE, 2100) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=59&c=1 */\
        V(LAST_JS_ITERATOR_HELPER_TYPE, 2100) \
        V(FIRST_JS_COLLECTION_ITERATOR_TYPE, 2101) \
          V(FIRST_JS_MAP_ITERATOR_TYPE, 2101) \
            V(JS_MAP_KEY_ITERATOR_TYPE, 2101) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=24&c=1 */\
            V(JS_MAP_KEY_VALUE_ITERATOR_TYPE, 2102) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=26&c=1 */\
            V(JS_MAP_VALUE_ITERATOR_TYPE, 2103) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=28&c=1 */\
          V(LAST_JS_MAP_ITERATOR_TYPE, 2103) \
          V(FIRST_JS_SET_ITERATOR_TYPE, 2104) \
            V(JS_SET_KEY_VALUE_ITERATOR_TYPE, 2104) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=32&c=1 */\
            V(JS_SET_VALUE_ITERATOR_TYPE, 2105) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=34&c=1 */\
          V(LAST_JS_SET_ITERATOR_TYPE, 2105) \
        V(LAST_JS_COLLECTION_ITERATOR_TYPE, 2105) \
        V(FIRST_ALWAYS_SHARED_SPACE_JS_OBJECT_TYPE, 2106) \
          V(FIRST_JS_SYNCHRONIZATION_PRIMITIVE_TYPE, 2106) \
            V(JS_ATOMICS_CONDITION_TYPE, 2106) /* https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=15&c=1 */\
            V(JS_ATOMICS_MUTEX_TYPE, 2107) /* https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=11&c=1 */\
          V(LAST_JS_SYNCHRONIZATION_PRIMITIVE_TYPE, 2107) \
          V(JS_SHARED_ARRAY_TYPE, 2108) /* https://crsrc.org/c/v8/src/objects/js-shared-array.tq?l=5&c=1 */\
          V(JS_SHARED_STRUCT_TYPE, 2109) /* https://crsrc.org/c/v8/src/objects/js-struct.tq?l=11&c=1 */\
        V(LAST_ALWAYS_SHARED_SPACE_JS_OBJECT_TYPE, 2109) \
        V(FIRST_JS_DISPOSABLE_STACK_BASE_TYPE, 2110) \
          V(JS_DISPOSABLE_STACK_BASE_TYPE, 2110) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=24&c=1 */\
          V(JS_ASYNC_DISPOSABLE_STACK_TYPE, 2111) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=38&c=1 */\
          V(JS_SYNC_DISPOSABLE_STACK_TYPE, 2112) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=36&c=1 */\
        V(LAST_JS_DISPOSABLE_STACK_BASE_TYPE, 2112) \
        V(FIRST_JS_GENERATOR_OBJECT_TYPE, 2113) \
          V(JS_GENERATOR_OBJECT_TYPE, 2113) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=5&c=1 */\
          V(JS_ASYNC_FUNCTION_OBJECT_TYPE, 2114) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=28&c=1 */\
          V(JS_ASYNC_GENERATOR_OBJECT_TYPE, 2115) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=36&c=1 */\
        V(LAST_JS_GENERATOR_OBJECT_TYPE, 2115) \
        V(FIRST_JS_COLLECTION_TYPE, 2116) \
          V(JS_MAP_TYPE, 2116) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=13&c=1 */\
          V(JS_SET_TYPE, 2117) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=12&c=1 */\
        V(LAST_JS_COLLECTION_TYPE, 2117) \
        V(FIRST_JS_OBJECT_WITH_EMBEDDER_SLOTS_TYPE, 2118) \
          V(JS_OBJECT_WITH_EMBEDDER_SLOTS_TYPE, 2118) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=50&c=1 */\
          V(JS_PROMISE_TYPE, 2119) /* https://crsrc.org/c/v8/src/objects/js-promise.tq?l=12&c=1 */\
        V(LAST_JS_OBJECT_WITH_EMBEDDER_SLOTS_TYPE, 2119) \
        V(FIRST_JS_WEAK_COLLECTION_TYPE, 2120) \
          V(JS_WEAK_MAP_TYPE, 2120) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=21&c=1 */\
          V(JS_WEAK_SET_TYPE, 2121) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=20&c=1 */\
        V(LAST_JS_WEAK_COLLECTION_TYPE, 2121) \
        V(JS_ARGUMENTS_OBJECT_TYPE, 2122) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1 */\
        V(JS_ARRAY_TYPE, 2123) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=61&c=1 */\
        V(JS_ARRAY_ITERATOR_TYPE, 2124) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=7&c=1 */\
        V(JS_ASYNC_FROM_SYNC_ITERATOR_TYPE, 2125) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=188&c=1 */\
        V(JS_COLLATOR_TYPE, 2126) /* https://crsrc.org/c/v8/src/objects/js-collator.tq?l=7&c=1 */\
        V(JS_CONTEXT_EXTENSION_OBJECT_TYPE, 2127) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=713&c=1 */\
        V(JS_DATE_TYPE, 2128) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=168&c=1 */\
        V(JS_DATE_TIME_FORMAT_TYPE, 2129) /* https://crsrc.org/c/v8/src/objects/js-date-time-format.tq?l=19&c=1 */\
        V(JS_DISPLAY_NAMES_TYPE, 2130) /* https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=18&c=1 */\
        V(JS_DURATION_FORMAT_TYPE, 2131) /* https://crsrc.org/c/v8/src/objects/js-duration-format.tq?l=43&c=1 */\
        V(JS_ERROR_TYPE, 2132) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=715&c=1 */\
        V(JS_EXTERNAL_OBJECT_TYPE, 2133) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=43&c=1 */\
        V(JS_FINALIZATION_REGISTRY_TYPE, 2134) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=9&c=1 */\
        V(JS_LIST_FORMAT_TYPE, 2135) /* https://crsrc.org/c/v8/src/objects/js-list-format.tq?l=14&c=1 */\
        V(JS_LOCALE_TYPE, 2136) /* https://crsrc.org/c/v8/src/objects/js-locale.tq?l=7&c=1 */\
        V(JS_MESSAGE_OBJECT_TYPE, 2137) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=149&c=1 */\
        V(JS_NUMBER_FORMAT_TYPE, 2138) /* https://crsrc.org/c/v8/src/objects/js-number-format.tq?l=7&c=1 */\
        V(JS_PLURAL_RULES_TYPE, 2139) /* https://crsrc.org/c/v8/src/objects/js-plural-rules.tq?l=12&c=1 */\
        V(JS_RAW_JSON_TYPE, 2140) /* https://crsrc.org/c/v8/src/objects/js-raw-json.tq?l=7&c=1 */\
        V(JS_REG_EXP_TYPE, 2141) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=59&c=1 */\
        V(JS_REG_EXP_STRING_ITERATOR_TYPE, 2142) /* https://crsrc.org/c/v8/src/objects/js-regexp-string-iterator.tq?l=11&c=1 */\
        V(JS_RELATIVE_TIME_FORMAT_TYPE, 2143) /* https://crsrc.org/c/v8/src/objects/js-relative-time-format.tq?l=13&c=1 */\
        V(JS_SEGMENT_ITERATOR_TYPE, 2144) /* https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=11&c=1 */\
        V(JS_SEGMENTER_TYPE, 2145) /* https://crsrc.org/c/v8/src/objects/js-segmenter.tq?l=13&c=1 */\
        V(JS_SEGMENTS_TYPE, 2146) /* https://crsrc.org/c/v8/src/objects/js-segments.tq?l=11&c=1 */\
        V(JS_SHADOW_REALM_TYPE, 2147) /* https://crsrc.org/c/v8/src/objects/js-shadow-realm.tq?l=5&c=1 */\
        V(JS_STRING_ITERATOR_TYPE, 2148) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=195&c=1 */\
        V(JS_TEMPORAL_DURATION_TYPE, 2149) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=7&c=1 */\
        V(JS_TEMPORAL_INSTANT_TYPE, 2150) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=11&c=1 */\
        V(JS_TEMPORAL_PLAIN_DATE_TYPE, 2151) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=19&c=1 */\
        V(JS_TEMPORAL_PLAIN_DATE_TIME_TYPE, 2152) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=15&c=1 */\
        V(JS_TEMPORAL_PLAIN_MONTH_DAY_TYPE, 2153) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=23&c=1 */\
        V(JS_TEMPORAL_PLAIN_TIME_TYPE, 2154) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=27&c=1 */\
        V(JS_TEMPORAL_PLAIN_YEAR_MONTH_TYPE, 2155) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=31&c=1 */\
        V(JS_TEMPORAL_ZONED_DATE_TIME_TYPE, 2156) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=35&c=1 */\
        V(JS_V8_BREAK_ITERATOR_TYPE, 2157) /* https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=7&c=1 */\
        V(JS_VALID_ITERATOR_WRAPPER_TYPE, 2158) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=204&c=1 */\
        V(JS_WEAK_REF_TYPE, 2159) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=43&c=1 */\
        V(WASM_EXCEPTION_PACKAGE_TYPE, 2160) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=195&c=1 */\
        V(WASM_GLOBAL_OBJECT_TYPE, 2161) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1 */\
        V(WASM_INSTANCE_OBJECT_TYPE, 2162) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1 */\
        V(WASM_MEMORY_MAP_DESCRIPTOR_TYPE, 2163) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1 */\
        V(WASM_MEMORY_OBJECT_TYPE, 2164) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1 */\
        V(WASM_MODULE_OBJECT_TYPE, 2165) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1 */\
        V(WASM_SUSPENDING_OBJECT_TYPE, 2166) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1 */\
        V(WASM_TABLE_OBJECT_TYPE, 2167) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1 */\
        V(WASM_TAG_OBJECT_TYPE, 2168) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1 */\
        V(WASM_VALUE_OBJECT_TYPE, 2169) /* https://crsrc.org/c/v8/src/debug/debug-wasm-objects.tq?l=7&c=1 */\
      V(LAST_JS_OBJECT_TYPE, 2169) \
    V(LAST_JS_RECEIVER_TYPE, 2169) \
  V(LAST_HEAP_OBJECT_TYPE, 2169) \

// Instance types for all classes except for those that use
// InstanceType as flags.
#define TORQUE_ASSIGNED_INSTANCE_TYPE_LIST(V) \
  V(SYMBOL_TYPE) /* https://crsrc.org/c/v8/src/objects/name.tq?l=35&c=1 */\
  V(BIG_INT_BASE_TYPE) /* https://crsrc.org/c/v8/src/objects/bigint.tq?l=7&c=1 */\
  V(HEAP_NUMBER_TYPE) /* https://crsrc.org/c/v8/src/objects/heap-number.tq?l=5&c=1 */\
  V(ODDBALL_TYPE) /* https://crsrc.org/c/v8/src/objects/oddball.tq?l=5&c=1 */\
  V(PROMISE_FULFILL_REACTION_JOB_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=70&c=1 */\
  V(PROMISE_REJECT_REACTION_JOB_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=73&c=1 */\
  V(ASYNC_RESUME_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=27&c=1 */\
  V(CALLABLE_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=18&c=1 */\
  V(CALLBACK_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=12&c=1 */\
  V(PROMISE_RESOLVE_THENABLE_JOB_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=76&c=1 */\
  V(LOAD_HANDLER_TYPE) /* https://crsrc.org/c/v8/src/ic/handler-configuration.tq?l=7&c=1 */\
  V(STORE_HANDLER_TYPE) /* https://crsrc.org/c/v8/src/ic/handler-configuration.tq?l=8&c=1 */\
  V(ACCESS_CHECK_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=29&c=1 */\
  V(ACCESSOR_PAIR_TYPE) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=12&c=1 */\
  V(ALIASED_ARGUMENTS_ENTRY_TYPE) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=41&c=1 */\
  V(ALLOCATION_MEMENTO_TYPE) /* https://crsrc.org/c/v8/src/objects/allocation-site.tq?l=7&c=1 */\
  V(ARRAY_BOILERPLATE_DESCRIPTION_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=13&c=1 */\
  V(ASM_WASM_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1 */\
  V(ASYNC_GENERATOR_REQUEST_TYPE) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=44&c=1 */\
  V(BREAK_POINT_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=5&c=1 */\
  V(BREAK_POINT_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=11&c=1 */\
  V(BYTECODE_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=26&c=1 */\
  V(CALL_SITE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=24&c=1 */\
  V(CLASS_BOILERPLATE_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=25&c=1 */\
  V(CLASS_POSITIONS_TYPE) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=10&c=1 */\
  V(CODE_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=33&c=1 */\
  V(DEBUG_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=35&c=1 */\
  V(ENUM_CACHE_TYPE) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=5&c=1 */\
  V(ERROR_STACK_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=109&c=1 */\
  V(FEEDBACK_CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/feedback-cell.tq?l=5&c=1 */\
  V(FUNCTION_TEMPLATE_RARE_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=23&c=1 */\
  V(MODULE_REQUEST_TYPE) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=59&c=1 */\
  V(PROMISE_CAPABILITY_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=5&c=1 */\
  V(PROMISE_REACTION_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=34&c=1 */\
  V(PROPERTY_DESCRIPTOR_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=19&c=1 */\
  V(PROTOTYPE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/prototype-info.tq?l=12&c=1 */\
  V(PROTOTYPE_SHARED_CLOSURE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=36&c=1 */\
  V(REG_EXP_BOILERPLATE_DESCRIPTION_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=19&c=1 */\
  V(REG_EXP_DATA_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=54&c=1 */\
  V(SCRIPT_TYPE) /* https://crsrc.org/c/v8/src/objects/script.tq?l=19&c=1 */\
  V(SCRIPT_OR_MODULE_TYPE) /* https://crsrc.org/c/v8/src/objects/module.tq?l=28&c=1 */\
  V(SOURCE_TEXT_MODULE_INFO_ENTRY_TYPE) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=70&c=1 */\
  V(STACK_FRAME_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=75&c=1 */\
  V(STACK_TRACE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=88&c=1 */\
  V(TEMPLATE_OBJECT_DESCRIPTION_TYPE) /* https://crsrc.org/c/v8/src/objects/template-objects.tq?l=5&c=1 */\
  V(TUPLE2_TYPE) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=8&c=1 */\
  V(WASM_EXCEPTION_TAG_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1 */\
  V(UNCOMPILED_DATA_WITH_PREPARSE_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=172&c=1 */\
  V(UNCOMPILED_DATA_WITH_PREPARSE_DATA_AND_JOB_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=183&c=1 */\
  V(UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=169&c=1 */\
  V(UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_WITH_JOB_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=177&c=1 */\
  V(WASM_FUNCTION_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1 */\
  V(WASM_CAPI_FUNCTION_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1 */\
  V(WASM_EXPORTED_FUNCTION_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1 */\
  V(WASM_JS_FUNCTION_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1 */\
  V(REG_EXP_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=17&c=1 */\
  V(ATOM_REG_EXP_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=28&c=1 */\
  V(IR_REG_EXP_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=38&c=1 */\
  V(BYTECODE_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=7&c=1 */\
  V(CODE_TYPE) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=31&c=1 */\
  V(INTERPRETER_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=14&c=1 */\
  V(WASM_DISPATCH_TABLE_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=202&c=1 */\
  V(WASM_INTERNAL_FUNCTION_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1 */\
  V(WASM_SUSPENDER_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1 */\
  V(WASM_TRUSTED_INSTANCE_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=37&c=1 */\
  V(INSTRUCTION_STREAM_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=293&c=1 */\
  V(PROTECTED_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=25&c=1 */\
  V(PROTECTED_WEAK_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=54&c=1 */\
  V(SHARED_FUNCTION_INFO_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=130&c=1 */\
  V(TRUSTED_BYTE_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=67&c=1 */\
  V(TRUSTED_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=19&c=1 */\
  V(TRUSTED_FOREIGN_TYPE) /* https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1 */\
  V(TRUSTED_WEAK_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=48&c=1 */\
  V(WASM_DISPATCH_TABLE_FOR_IMPORTS_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=203&c=1 */\
  V(WASM_IMPORT_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1 */\
  V(ACCESSOR_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1 */\
  V(FOREIGN_TYPE) /* https://crsrc.org/c/v8/src/objects/foreign.tq?l=5&c=1 */\
  V(FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=11&c=1 */\
  V(HASH_TABLE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=264&c=1 */\
  V(EPHEMERON_HASH_TABLE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=275&c=1 */\
  V(GLOBAL_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=272&c=1 */\
  V(NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=271&c=1 */\
  V(NAME_TO_INDEX_HASH_TABLE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=269&c=1 */\
  V(NUMBER_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=278&c=1 */\
  V(ORDERED_HASH_MAP_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=266&c=1 */\
  V(ORDERED_HASH_SET_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=267&c=1 */\
  V(ORDERED_NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=268&c=1 */\
  V(REGISTERED_SYMBOL_TABLE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=270&c=1 */\
  V(SIMPLE_NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=273&c=1 */\
  V(SIMPLE_NUMBER_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=274&c=1 */\
  V(BYTE_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=62&c=1 */\
  V(FIXED_DOUBLE_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=31&c=1 */\
  V(SLOPPY_ARGUMENTS_ELEMENTS_TYPE) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=27&c=1 */\
  V(AWAIT_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=49&c=1 */\
  V(BLOCK_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=50&c=1 */\
  V(CATCH_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=51&c=1 */\
  V(DEBUG_EVALUATE_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=52&c=1 */\
  V(EVAL_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=54&c=1 */\
  V(FUNCTION_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=59&c=1 */\
  V(MODULE_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=55&c=1 */\
  V(NATIVE_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=92&c=1 */\
  V(SCRIPT_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=56&c=1 */\
  V(WITH_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=57&c=1 */\
  V(TURBOSHAFT_FLOAT64_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=65&c=1 */\
  V(TURBOSHAFT_FLOAT64_RANGE_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=72&c=1 */\
  V(TURBOSHAFT_FLOAT64_SET_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=81&c=1 */\
  V(TURBOSHAFT_WORD32_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=20&c=1 */\
  V(TURBOSHAFT_WORD32_RANGE_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=25&c=1 */\
  V(TURBOSHAFT_WORD32_SET_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=33&c=1 */\
  V(TURBOSHAFT_WORD64_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=41&c=1 */\
  V(TURBOSHAFT_WORD64_RANGE_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=46&c=1 */\
  V(TURBOSHAFT_WORD64_SET_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=56&c=1 */\
  V(TURBOFAN_BITSET_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=61&c=1 */\
  V(TURBOFAN_HEAP_CONSTANT_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=79&c=1 */\
  V(TURBOFAN_OTHER_NUMBER_CONSTANT_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=84&c=1 */\
  V(TURBOFAN_RANGE_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=73&c=1 */\
  V(TURBOFAN_UNION_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=67&c=1 */\
  V(EXPORTED_SUB_CLASS_BASE_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1181&c=1 */\
  V(EXPORTED_SUB_CLASS_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1174&c=1 */\
  V(EXPORTED_SUB_CLASS2_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1216&c=1 */\
  V(SMALL_ORDERED_HASH_MAP_TYPE) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=64&c=1 */\
  V(SMALL_ORDERED_HASH_SET_TYPE) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=25&c=1 */\
  V(SMALL_ORDERED_NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=101&c=1 */\
  V(FUNCTION_TEMPLATE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1 */\
  V(OBJECT_TEMPLATE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1 */\
  V(DICTIONARY_TEMPLATE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1 */\
  V(ABSTRACT_INTERNAL_CLASS_SUBCLASS1_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1190&c=1 */\
  V(ABSTRACT_INTERNAL_CLASS_SUBCLASS2_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1192&c=1 */\
  V(DESCRIPTOR_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1 */\
  V(STRONG_DESCRIPTOR_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=44&c=1 */\
  V(SOURCE_TEXT_MODULE_TYPE) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=12&c=1 */\
  V(SYNTHETIC_MODULE_TYPE) /* https://crsrc.org/c/v8/src/objects/synthetic-module.tq?l=5&c=1 */\
  V(WEAK_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=36&c=1 */\
  V(TRANSITION_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=299&c=1 */\
  V(ALLOCATION_SITE_TYPE) /* https://crsrc.org/c/v8/src/objects/allocation-site.tq?l=5&c=1 */\
  V(ARRAY_LIST_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=75&c=1 */\
  V(CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/cell.tq?l=5&c=1 */\
  V(CLOSURE_FEEDBACK_CELL_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=27&c=1 */\
  V(CONTEXT_CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=25&c=1 */\
  V(COVERAGE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1 */\
  V(CPP_HEAP_EXTERNAL_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/cpp-heap-external-object.tq?l=5&c=1 */\
  V(DOUBLE_STRING_CACHE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=279&c=1 */\
  V(EMBEDDER_DATA_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/embedder-data-array.tq?l=5&c=1 */\
  V(FEEDBACK_METADATA_TYPE) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=47&c=1 */\
  V(FEEDBACK_VECTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=33&c=1 */\
  V(FILLER_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=690&c=1 */\
  V(FREE_SPACE_TYPE) /* https://crsrc.org/c/v8/src/objects/free-space.tq?l=5&c=1 */\
  V(HOLE_TYPE) /* https://crsrc.org/c/v8/src/objects/hole.tq?l=5&c=1 */\
  V(INTERCEPTOR_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1 */\
  V(INTERNAL_CLASS_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=763&c=1 */\
  V(INTERNAL_CLASS_WITH_STRUCT_ELEMENTS_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1199&c=1 */\
  V(MAP_TYPE) /* https://crsrc.org/c/v8/src/objects/map.tq?l=37&c=1 */\
  V(MEGA_DOM_HANDLER_TYPE) /* https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=5&c=1 */\
  V(OBJECT_BOILERPLATE_DESCRIPTION_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=5&c=1 */\
  V(ON_HEAP_BASIC_BLOCK_PROFILER_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1 */\
  V(PREPARSE_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=7&c=1 */\
  V(PROPERTY_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/property-array.tq?l=5&c=1 */\
  V(PROPERTY_CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/property-cell.tq?l=6&c=1 */\
  V(REG_EXP_MATCH_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/regexp-match-info.tq?l=5&c=1 */\
  V(SCOPE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/scope-info.tq?l=122&c=1 */\
  V(SCRIPT_CONTEXT_TABLE_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=17&c=1 */\
  V(SHARED_FUNCTION_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1 */\
  V(SMI_BOX_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=930&c=1 */\
  V(SMI_PAIR_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=829&c=1 */\
  V(SORT_STATE_TYPE) /* https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=19&c=1 */\
  V(SWISS_NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/objects/swiss-name-dictionary.tq?l=7&c=1 */\
  V(WASM_CONTINUATION_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1 */\
  V(WASM_FAST_API_CALL_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=73&c=1 */\
  V(WASM_FUNC_REF_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1 */\
  V(WASM_NULL_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1 */\
  V(WASM_RESUME_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1 */\
  V(WASM_STACK_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1 */\
  V(WASM_STRING_VIEW_ITER_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=325&c=1 */\
  V(WASM_TYPE_INFO_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1 */\
  V(WEAK_ARRAY_LIST_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=82&c=1 */\
  V(WEAK_CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=21&c=1 */\
  V(WEAK_HOMOMORPHIC_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=42&c=1 */\
  V(WASM_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1 */\
  V(WASM_STRUCT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1 */\
  V(JS_PROXY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-proxy.tq?l=5&c=1 */\
  V(JS_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=14&c=1 */\
  V(JS_MODULE_NAMESPACE_TYPE) /* https://crsrc.org/c/v8/src/objects/module.tq?l=22&c=1 */\
  V(JS_DEFERRED_MODULE_NAMESPACE_TYPE) /* https://crsrc.org/c/v8/src/objects/module.tq?l=26&c=1 */\
  V(JS_GLOBAL_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=119&c=1 */\
  V(JS_GLOBAL_PROXY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=117&c=1 */\
  V(JS_SPECIAL_API_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=710&c=1 */\
  V(JS_PRIMITIVE_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=145&c=1 */\
  V(JS_API_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=696&c=1 */\
  V(JS_LAST_DUMMY_API_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=703&c=1 */\
  V(JS_DATA_VIEW_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=145&c=1 */\
  V(JS_RAB_GSAB_DATA_VIEW_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=147&c=1 */\
  V(JS_TYPED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=118&c=1 */\
  V(JS_DETACHED_TYPED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=129&c=1 */\
  V(JS_ARRAY_BUFFER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=18&c=1 */\
  V(JS_FUNCTION_WITHOUT_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=52&c=1 */\
  V(JS_FUNCTION_WITH_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=55&c=1 */\
  V(BIGINT64_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=186&c=1 */\
  V(BIGUINT64_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=183&c=1 */\
  V(FLOAT16_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=171&c=1 */\
  V(FLOAT32_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=174&c=1 */\
  V(FLOAT64_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=177&c=1 */\
  V(INT16_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=162&c=1 */\
  V(INT32_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=168&c=1 */\
  V(INT8_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=156&c=1 */\
  V(UINT16_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=159&c=1 */\
  V(UINT32_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=165&c=1 */\
  V(UINT8_CLAMPED_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=180&c=1 */\
  V(UINT8_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=153&c=1 */\
  V(JS_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=76&c=1 */\
  V(JS_PROMISE_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-promise.tq?l=38&c=1 */\
  V(JS_REG_EXP_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=95&c=1 */\
  V(JS_CLASS_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=63&c=1 */\
  V(JS_BOUND_FUNCTION_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=8&c=1 */\
  V(JS_WRAPPED_FUNCTION_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=20&c=1 */\
  V(JS_ARRAY_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=248&c=1 */\
  V(JS_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=245&c=1 */\
  V(JS_MAP_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=251&c=1 */\
  V(JS_OBJECT_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=230&c=1 */\
  V(JS_PROMISE_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=236&c=1 */\
  V(JS_REG_EXP_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=233&c=1 */\
  V(JS_SET_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=254&c=1 */\
  V(JS_SET_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=242&c=1 */\
  V(JS_STRING_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=257&c=1 */\
  V(JS_TYPED_ARRAY_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=239&c=1 */\
  V(JS_ITERATOR_CONCAT_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=48&c=1 */\
  V(JS_ITERATOR_DROP_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=38&c=1 */\
  V(JS_ITERATOR_FILTER_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=29&c=1 */\
  V(JS_ITERATOR_FLAT_MAP_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=42&c=1 */\
  V(JS_ITERATOR_MAP_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=24&c=1 */\
  V(JS_ITERATOR_TAKE_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=34&c=1 */\
  V(JS_ITERATOR_ZIP_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=59&c=1 */\
  V(JS_MAP_KEY_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=24&c=1 */\
  V(JS_MAP_KEY_VALUE_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=26&c=1 */\
  V(JS_MAP_VALUE_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=28&c=1 */\
  V(JS_SET_KEY_VALUE_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=32&c=1 */\
  V(JS_SET_VALUE_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=34&c=1 */\
  V(JS_ATOMICS_CONDITION_TYPE) /* https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=15&c=1 */\
  V(JS_ATOMICS_MUTEX_TYPE) /* https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=11&c=1 */\
  V(JS_SHARED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-shared-array.tq?l=5&c=1 */\
  V(JS_SHARED_STRUCT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-struct.tq?l=11&c=1 */\
  V(JS_DISPOSABLE_STACK_BASE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=24&c=1 */\
  V(JS_ASYNC_DISPOSABLE_STACK_TYPE) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=38&c=1 */\
  V(JS_SYNC_DISPOSABLE_STACK_TYPE) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=36&c=1 */\
  V(JS_GENERATOR_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=5&c=1 */\
  V(JS_ASYNC_FUNCTION_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=28&c=1 */\
  V(JS_ASYNC_GENERATOR_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=36&c=1 */\
  V(JS_MAP_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=13&c=1 */\
  V(JS_SET_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=12&c=1 */\
  V(JS_OBJECT_WITH_EMBEDDER_SLOTS_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=50&c=1 */\
  V(JS_PROMISE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-promise.tq?l=12&c=1 */\
  V(JS_WEAK_MAP_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=21&c=1 */\
  V(JS_WEAK_SET_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=20&c=1 */\
  V(JS_ARGUMENTS_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1 */\
  V(JS_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=61&c=1 */\
  V(JS_ARRAY_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=7&c=1 */\
  V(JS_ASYNC_FROM_SYNC_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=188&c=1 */\
  V(JS_COLLATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collator.tq?l=7&c=1 */\
  V(JS_CONTEXT_EXTENSION_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=713&c=1 */\
  V(JS_DATE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=168&c=1 */\
  V(JS_DATE_TIME_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-date-time-format.tq?l=19&c=1 */\
  V(JS_DISPLAY_NAMES_TYPE) /* https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=18&c=1 */\
  V(JS_DURATION_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-duration-format.tq?l=43&c=1 */\
  V(JS_ERROR_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=715&c=1 */\
  V(JS_EXTERNAL_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=43&c=1 */\
  V(JS_FINALIZATION_REGISTRY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=9&c=1 */\
  V(JS_LIST_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-list-format.tq?l=14&c=1 */\
  V(JS_LOCALE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-locale.tq?l=7&c=1 */\
  V(JS_MESSAGE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=149&c=1 */\
  V(JS_NUMBER_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-number-format.tq?l=7&c=1 */\
  V(JS_PLURAL_RULES_TYPE) /* https://crsrc.org/c/v8/src/objects/js-plural-rules.tq?l=12&c=1 */\
  V(JS_RAW_JSON_TYPE) /* https://crsrc.org/c/v8/src/objects/js-raw-json.tq?l=7&c=1 */\
  V(JS_REG_EXP_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=59&c=1 */\
  V(JS_REG_EXP_STRING_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp-string-iterator.tq?l=11&c=1 */\
  V(JS_RELATIVE_TIME_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-relative-time-format.tq?l=13&c=1 */\
  V(JS_SEGMENT_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=11&c=1 */\
  V(JS_SEGMENTER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-segmenter.tq?l=13&c=1 */\
  V(JS_SEGMENTS_TYPE) /* https://crsrc.org/c/v8/src/objects/js-segments.tq?l=11&c=1 */\
  V(JS_SHADOW_REALM_TYPE) /* https://crsrc.org/c/v8/src/objects/js-shadow-realm.tq?l=5&c=1 */\
  V(JS_STRING_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=195&c=1 */\
  V(JS_TEMPORAL_DURATION_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=7&c=1 */\
  V(JS_TEMPORAL_INSTANT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=11&c=1 */\
  V(JS_TEMPORAL_PLAIN_DATE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=19&c=1 */\
  V(JS_TEMPORAL_PLAIN_DATE_TIME_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=15&c=1 */\
  V(JS_TEMPORAL_PLAIN_MONTH_DAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=23&c=1 */\
  V(JS_TEMPORAL_PLAIN_TIME_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=27&c=1 */\
  V(JS_TEMPORAL_PLAIN_YEAR_MONTH_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=31&c=1 */\
  V(JS_TEMPORAL_ZONED_DATE_TIME_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=35&c=1 */\
  V(JS_V8_BREAK_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=7&c=1 */\
  V(JS_VALID_ITERATOR_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=204&c=1 */\
  V(JS_WEAK_REF_TYPE) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=43&c=1 */\
  V(WASM_EXCEPTION_PACKAGE_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=195&c=1 */\
  V(WASM_GLOBAL_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1 */\
  V(WASM_INSTANCE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1 */\
  V(WASM_MEMORY_MAP_DESCRIPTOR_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1 */\
  V(WASM_MEMORY_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1 */\
  V(WASM_MODULE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1 */\
  V(WASM_SUSPENDING_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1 */\
  V(WASM_TABLE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1 */\
  V(WASM_TAG_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1 */\
  V(WASM_VALUE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/debug/debug-wasm-objects.tq?l=7&c=1 */\

// Pairs of (ClassName, INSTANCE_TYPE) for classes that have
// full Torque definitions.
#define TORQUE_INSTANCE_CHECKERS_SINGLE_FULLY_DEFINED(V) \
  V(Symbol, SYMBOL_TYPE) /* https://crsrc.org/c/v8/src/objects/name.tq?l=35&c=1 */ \
  V(HeapNumber, HEAP_NUMBER_TYPE) /* https://crsrc.org/c/v8/src/objects/heap-number.tq?l=5&c=1 */ \
  V(Oddball, ODDBALL_TYPE) /* https://crsrc.org/c/v8/src/objects/oddball.tq?l=5&c=1 */ \
  V(PromiseFulfillReactionJobTask, PROMISE_FULFILL_REACTION_JOB_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=70&c=1 */ \
  V(PromiseRejectReactionJobTask, PROMISE_REJECT_REACTION_JOB_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=73&c=1 */ \
  V(AsyncResumeTask, ASYNC_RESUME_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=27&c=1 */ \
  V(CallableTask, CALLABLE_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=18&c=1 */ \
  V(CallbackTask, CALLBACK_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/microtask.tq?l=12&c=1 */ \
  V(PromiseResolveThenableJobTask, PROMISE_RESOLVE_THENABLE_JOB_TASK_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=76&c=1 */ \
  V(AccessCheckInfo, ACCESS_CHECK_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=29&c=1 */ \
  V(AliasedArgumentsEntry, ALIASED_ARGUMENTS_ENTRY_TYPE) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=41&c=1 */ \
  V(AllocationMemento, ALLOCATION_MEMENTO_TYPE) /* https://crsrc.org/c/v8/src/objects/allocation-site.tq?l=7&c=1 */ \
  V(ArrayBoilerplateDescription, ARRAY_BOILERPLATE_DESCRIPTION_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=13&c=1 */ \
  V(AsmWasmData, ASM_WASM_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1 */ \
  V(AsyncGeneratorRequest, ASYNC_GENERATOR_REQUEST_TYPE) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=44&c=1 */ \
  V(BreakPoint, BREAK_POINT_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=5&c=1 */ \
  V(BreakPointInfo, BREAK_POINT_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=11&c=1 */ \
  V(BytecodeWrapper, BYTECODE_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=26&c=1 */ \
  V(CallSiteInfo, CALL_SITE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=24&c=1 */ \
  V(ClassBoilerplate, CLASS_BOILERPLATE_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=25&c=1 */ \
  V(DebugInfo, DEBUG_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=35&c=1 */ \
  V(EnumCache, ENUM_CACHE_TYPE) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=5&c=1 */ \
  V(ErrorStackData, ERROR_STACK_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=109&c=1 */ \
  V(FeedbackCell, FEEDBACK_CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/feedback-cell.tq?l=5&c=1 */ \
  V(FunctionTemplateRareData, FUNCTION_TEMPLATE_RARE_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=23&c=1 */ \
  V(ModuleRequest, MODULE_REQUEST_TYPE) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=59&c=1 */ \
  V(PromiseCapability, PROMISE_CAPABILITY_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=5&c=1 */ \
  V(PromiseReaction, PROMISE_REACTION_TYPE) /* https://crsrc.org/c/v8/src/objects/promise.tq?l=34&c=1 */ \
  V(PropertyDescriptorObject, PROPERTY_DESCRIPTOR_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=19&c=1 */ \
  V(PrototypeInfo, PROTOTYPE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/prototype-info.tq?l=12&c=1 */ \
  V(PrototypeSharedClosureInfo, PROTOTYPE_SHARED_CLOSURE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=36&c=1 */ \
  V(RegExpBoilerplateDescription, REG_EXP_BOILERPLATE_DESCRIPTION_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=19&c=1 */ \
  V(RegExpDataWrapper, REG_EXP_DATA_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=54&c=1 */ \
  V(Script, SCRIPT_TYPE) /* https://crsrc.org/c/v8/src/objects/script.tq?l=19&c=1 */ \
  V(ScriptOrModule, SCRIPT_OR_MODULE_TYPE) /* https://crsrc.org/c/v8/src/objects/module.tq?l=28&c=1 */ \
  V(SourceTextModuleInfoEntry, SOURCE_TEXT_MODULE_INFO_ENTRY_TYPE) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=70&c=1 */ \
  V(StackFrameInfo, STACK_FRAME_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=75&c=1 */ \
  V(StackTraceInfo, STACK_TRACE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=88&c=1 */ \
  V(TemplateObjectDescription, TEMPLATE_OBJECT_DESCRIPTION_TYPE) /* https://crsrc.org/c/v8/src/objects/template-objects.tq?l=5&c=1 */ \
  V(WasmExceptionTag, WASM_EXCEPTION_TAG_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1 */ \
  V(UncompiledDataWithPreparseDataAndJob, UNCOMPILED_DATA_WITH_PREPARSE_DATA_AND_JOB_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=183&c=1 */ \
  V(UncompiledDataWithoutPreparseDataWithJob, UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_WITH_JOB_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=177&c=1 */ \
  V(WasmCapiFunctionData, WASM_CAPI_FUNCTION_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1 */ \
  V(WasmExportedFunctionData, WASM_EXPORTED_FUNCTION_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1 */ \
  V(WasmJSFunctionData, WASM_JS_FUNCTION_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1 */ \
  V(AtomRegExpData, ATOM_REG_EXP_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=28&c=1 */ \
  V(IrRegExpData, IR_REG_EXP_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=38&c=1 */ \
  V(BytecodeArray, BYTECODE_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=7&c=1 */ \
  V(InterpreterData, INTERPRETER_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=14&c=1 */ \
  V(WasmInternalFunction, WASM_INTERNAL_FUNCTION_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1 */ \
  V(WasmSuspenderObject, WASM_SUSPENDER_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1 */ \
  V(ProtectedFixedArray, PROTECTED_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=25&c=1 */ \
  V(ProtectedWeakFixedArray, PROTECTED_WEAK_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=54&c=1 */ \
  V(SharedFunctionInfoWrapper, SHARED_FUNCTION_INFO_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=130&c=1 */ \
  V(TrustedByteArray, TRUSTED_BYTE_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=67&c=1 */ \
  V(TrustedFixedArray, TRUSTED_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=19&c=1 */ \
  V(TrustedForeign, TRUSTED_FOREIGN_TYPE) /* https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1 */ \
  V(TrustedWeakFixedArray, TRUSTED_WEAK_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=48&c=1 */ \
  V(WasmImportData, WASM_IMPORT_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1 */ \
  V(AccessorInfo, ACCESSOR_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1 */ \
  V(Foreign, FOREIGN_TYPE) /* https://crsrc.org/c/v8/src/objects/foreign.tq?l=5&c=1 */ \
  V(ByteArray, BYTE_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=62&c=1 */ \
  V(FixedDoubleArray, FIXED_DOUBLE_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=31&c=1 */ \
  V(SloppyArgumentsElements, SLOPPY_ARGUMENTS_ELEMENTS_TYPE) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=27&c=1 */ \
  V(TurboshaftFloat64RangeType, TURBOSHAFT_FLOAT64_RANGE_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=72&c=1 */ \
  V(TurboshaftFloat64SetType, TURBOSHAFT_FLOAT64_SET_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=81&c=1 */ \
  V(TurboshaftWord32RangeType, TURBOSHAFT_WORD32_RANGE_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=25&c=1 */ \
  V(TurboshaftWord32SetType, TURBOSHAFT_WORD32_SET_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=33&c=1 */ \
  V(TurboshaftWord64RangeType, TURBOSHAFT_WORD64_RANGE_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=46&c=1 */ \
  V(TurboshaftWord64SetType, TURBOSHAFT_WORD64_SET_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=56&c=1 */ \
  V(TurbofanBitsetType, TURBOFAN_BITSET_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=61&c=1 */ \
  V(TurbofanHeapConstantType, TURBOFAN_HEAP_CONSTANT_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=79&c=1 */ \
  V(TurbofanOtherNumberConstantType, TURBOFAN_OTHER_NUMBER_CONSTANT_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=84&c=1 */ \
  V(TurbofanRangeType, TURBOFAN_RANGE_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=73&c=1 */ \
  V(TurbofanUnionType, TURBOFAN_UNION_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turbofan-types.tq?l=67&c=1 */ \
  V(ExportedSubClass, EXPORTED_SUB_CLASS_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1174&c=1 */ \
  V(ExportedSubClass2, EXPORTED_SUB_CLASS2_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1216&c=1 */ \
  V(SmallOrderedHashMap, SMALL_ORDERED_HASH_MAP_TYPE) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=64&c=1 */ \
  V(SmallOrderedHashSet, SMALL_ORDERED_HASH_SET_TYPE) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=25&c=1 */ \
  V(SmallOrderedNameDictionary, SMALL_ORDERED_NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=101&c=1 */ \
  V(FunctionTemplateInfo, FUNCTION_TEMPLATE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1 */ \
  V(ObjectTemplateInfo, OBJECT_TEMPLATE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1 */ \
  V(DictionaryTemplateInfo, DICTIONARY_TEMPLATE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1 */ \
  V(AbstractInternalClassSubclass1, ABSTRACT_INTERNAL_CLASS_SUBCLASS1_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1190&c=1 */ \
  V(AbstractInternalClassSubclass2, ABSTRACT_INTERNAL_CLASS_SUBCLASS2_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1192&c=1 */ \
  V(StrongDescriptorArray, STRONG_DESCRIPTOR_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=44&c=1 */ \
  V(SourceTextModule, SOURCE_TEXT_MODULE_TYPE) /* https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=12&c=1 */ \
  V(SyntheticModule, SYNTHETIC_MODULE_TYPE) /* https://crsrc.org/c/v8/src/objects/synthetic-module.tq?l=5&c=1 */ \
  V(ArrayList, ARRAY_LIST_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=75&c=1 */ \
  V(Cell, CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/cell.tq?l=5&c=1 */ \
  V(ClosureFeedbackCellArray, CLOSURE_FEEDBACK_CELL_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=27&c=1 */ \
  V(ContextCell, CONTEXT_CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=25&c=1 */ \
  V(CoverageInfo, COVERAGE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1 */ \
  V(CppHeapExternalObject, CPP_HEAP_EXTERNAL_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/cpp-heap-external-object.tq?l=5&c=1 */ \
  V(EmbedderDataArray, EMBEDDER_DATA_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/embedder-data-array.tq?l=5&c=1 */ \
  V(FeedbackVector, FEEDBACK_VECTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=33&c=1 */ \
  V(InterceptorInfo, INTERCEPTOR_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1 */ \
  V(InternalClass, INTERNAL_CLASS_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=763&c=1 */ \
  V(InternalClassWithStructElements, INTERNAL_CLASS_WITH_STRUCT_ELEMENTS_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1199&c=1 */ \
  V(Map, MAP_TYPE) /* https://crsrc.org/c/v8/src/objects/map.tq?l=37&c=1 */ \
  V(MegaDomHandler, MEGA_DOM_HANDLER_TYPE) /* https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=5&c=1 */ \
  V(ObjectBoilerplateDescription, OBJECT_BOILERPLATE_DESCRIPTION_TYPE) /* https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=5&c=1 */ \
  V(OnHeapBasicBlockProfilerData, ON_HEAP_BASIC_BLOCK_PROFILER_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1 */ \
  V(PreparseData, PREPARSE_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=7&c=1 */ \
  V(PropertyArray, PROPERTY_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/property-array.tq?l=5&c=1 */ \
  V(PropertyCell, PROPERTY_CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/property-cell.tq?l=6&c=1 */ \
  V(RegExpMatchInfo, REG_EXP_MATCH_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/regexp-match-info.tq?l=5&c=1 */ \
  V(ScopeInfo, SCOPE_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/scope-info.tq?l=122&c=1 */ \
  V(ScriptContextTable, SCRIPT_CONTEXT_TABLE_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=17&c=1 */ \
  V(SharedFunctionInfo, SHARED_FUNCTION_INFO_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1 */ \
  V(SmiBox, SMI_BOX_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=930&c=1 */ \
  V(SmiPair, SMI_PAIR_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=829&c=1 */ \
  V(SortState, SORT_STATE_TYPE) /* https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=19&c=1 */ \
  V(SwissNameDictionary, SWISS_NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/objects/swiss-name-dictionary.tq?l=7&c=1 */ \
  V(WasmContinuationObject, WASM_CONTINUATION_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1 */ \
  V(WasmFastApiCallData, WASM_FAST_API_CALL_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=73&c=1 */ \
  V(WasmFuncRef, WASM_FUNC_REF_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1 */ \
  V(WasmNull, WASM_NULL_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1 */ \
  V(WasmResumeData, WASM_RESUME_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1 */ \
  V(WasmStackObject, WASM_STACK_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1 */ \
  V(WasmStringViewIter, WASM_STRING_VIEW_ITER_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=325&c=1 */ \
  V(WasmTypeInfo, WASM_TYPE_INFO_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1 */ \
  V(WeakArrayList, WEAK_ARRAY_LIST_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=82&c=1 */ \
  V(WeakCell, WEAK_CELL_TYPE) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=21&c=1 */ \
  V(WeakHomomorphicFixedArray, WEAK_HOMOMORPHIC_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=42&c=1 */ \
  V(WasmArray, WASM_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1 */ \
  V(WasmStruct, WASM_STRUCT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1 */ \
  V(JSProxy, JS_PROXY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-proxy.tq?l=5&c=1 */ \
  V(JSDeferredModuleNamespace, JS_DEFERRED_MODULE_NAMESPACE_TYPE) /* https://crsrc.org/c/v8/src/objects/module.tq?l=26&c=1 */ \
  V(JSGlobalObject, JS_GLOBAL_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=119&c=1 */ \
  V(JSGlobalProxy, JS_GLOBAL_PROXY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=117&c=1 */ \
  V(JSPrimitiveWrapper, JS_PRIMITIVE_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=145&c=1 */ \
  V(JSDataView, JS_DATA_VIEW_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=145&c=1 */ \
  V(JSRabGsabDataView, JS_RAB_GSAB_DATA_VIEW_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=147&c=1 */ \
  V(JSDetachedTypedArray, JS_DETACHED_TYPED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=129&c=1 */ \
  V(JSArrayBuffer, JS_ARRAY_BUFFER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=18&c=1 */ \
  V(JSFunctionWithoutPrototype, JS_FUNCTION_WITHOUT_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=52&c=1 */ \
  V(JSBoundFunction, JS_BOUND_FUNCTION_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=8&c=1 */ \
  V(JSWrappedFunction, JS_WRAPPED_FUNCTION_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=20&c=1 */ \
  V(JSIteratorConcatHelper, JS_ITERATOR_CONCAT_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=48&c=1 */ \
  V(JSIteratorDropHelper, JS_ITERATOR_DROP_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=38&c=1 */ \
  V(JSIteratorFilterHelper, JS_ITERATOR_FILTER_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=29&c=1 */ \
  V(JSIteratorFlatMapHelper, JS_ITERATOR_FLAT_MAP_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=42&c=1 */ \
  V(JSIteratorMapHelper, JS_ITERATOR_MAP_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=24&c=1 */ \
  V(JSIteratorTakeHelper, JS_ITERATOR_TAKE_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=34&c=1 */ \
  V(JSIteratorZipHelper, JS_ITERATOR_ZIP_HELPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=59&c=1 */ \
  V(JSAtomicsCondition, JS_ATOMICS_CONDITION_TYPE) /* https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=15&c=1 */ \
  V(JSAtomicsMutex, JS_ATOMICS_MUTEX_TYPE) /* https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=11&c=1 */ \
  V(JSSharedArray, JS_SHARED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-shared-array.tq?l=5&c=1 */ \
  V(JSSharedStruct, JS_SHARED_STRUCT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-struct.tq?l=11&c=1 */ \
  V(JSAsyncDisposableStack, JS_ASYNC_DISPOSABLE_STACK_TYPE) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=38&c=1 */ \
  V(JSSyncDisposableStack, JS_SYNC_DISPOSABLE_STACK_TYPE) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=36&c=1 */ \
  V(JSAsyncFunctionObject, JS_ASYNC_FUNCTION_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=28&c=1 */ \
  V(JSAsyncGeneratorObject, JS_ASYNC_GENERATOR_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=36&c=1 */ \
  V(JSMap, JS_MAP_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=13&c=1 */ \
  V(JSSet, JS_SET_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=12&c=1 */ \
  V(JSPromise, JS_PROMISE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-promise.tq?l=12&c=1 */ \
  V(JSWeakMap, JS_WEAK_MAP_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=21&c=1 */ \
  V(JSWeakSet, JS_WEAK_SET_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=20&c=1 */ \
  V(JSArgumentsObject, JS_ARGUMENTS_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1 */ \
  V(JSArray, JS_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=61&c=1 */ \
  V(JSArrayIterator, JS_ARRAY_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=7&c=1 */ \
  V(JSAsyncFromSyncIterator, JS_ASYNC_FROM_SYNC_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=188&c=1 */ \
  V(JSCollator, JS_COLLATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collator.tq?l=7&c=1 */ \
  V(JSDate, JS_DATE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=168&c=1 */ \
  V(JSDateTimeFormat, JS_DATE_TIME_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-date-time-format.tq?l=19&c=1 */ \
  V(JSDisplayNames, JS_DISPLAY_NAMES_TYPE) /* https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=18&c=1 */ \
  V(JSDurationFormat, JS_DURATION_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-duration-format.tq?l=43&c=1 */ \
  V(JSExternalObject, JS_EXTERNAL_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=43&c=1 */ \
  V(JSFinalizationRegistry, JS_FINALIZATION_REGISTRY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=9&c=1 */ \
  V(JSListFormat, JS_LIST_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-list-format.tq?l=14&c=1 */ \
  V(JSLocale, JS_LOCALE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-locale.tq?l=7&c=1 */ \
  V(JSMessageObject, JS_MESSAGE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=149&c=1 */ \
  V(JSNumberFormat, JS_NUMBER_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-number-format.tq?l=7&c=1 */ \
  V(JSPluralRules, JS_PLURAL_RULES_TYPE) /* https://crsrc.org/c/v8/src/objects/js-plural-rules.tq?l=12&c=1 */ \
  V(JSRawJson, JS_RAW_JSON_TYPE) /* https://crsrc.org/c/v8/src/objects/js-raw-json.tq?l=7&c=1 */ \
  V(JSRegExp, JS_REG_EXP_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=59&c=1 */ \
  V(JSRegExpStringIterator, JS_REG_EXP_STRING_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp-string-iterator.tq?l=11&c=1 */ \
  V(JSRelativeTimeFormat, JS_RELATIVE_TIME_FORMAT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-relative-time-format.tq?l=13&c=1 */ \
  V(JSSegmentIterator, JS_SEGMENT_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=11&c=1 */ \
  V(JSSegmenter, JS_SEGMENTER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-segmenter.tq?l=13&c=1 */ \
  V(JSSegments, JS_SEGMENTS_TYPE) /* https://crsrc.org/c/v8/src/objects/js-segments.tq?l=11&c=1 */ \
  V(JSShadowRealm, JS_SHADOW_REALM_TYPE) /* https://crsrc.org/c/v8/src/objects/js-shadow-realm.tq?l=5&c=1 */ \
  V(JSStringIterator, JS_STRING_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=195&c=1 */ \
  V(JSTemporalDuration, JS_TEMPORAL_DURATION_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=7&c=1 */ \
  V(JSTemporalInstant, JS_TEMPORAL_INSTANT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=11&c=1 */ \
  V(JSTemporalPlainDate, JS_TEMPORAL_PLAIN_DATE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=19&c=1 */ \
  V(JSTemporalPlainDateTime, JS_TEMPORAL_PLAIN_DATE_TIME_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=15&c=1 */ \
  V(JSTemporalPlainMonthDay, JS_TEMPORAL_PLAIN_MONTH_DAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=23&c=1 */ \
  V(JSTemporalPlainTime, JS_TEMPORAL_PLAIN_TIME_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=27&c=1 */ \
  V(JSTemporalPlainYearMonth, JS_TEMPORAL_PLAIN_YEAR_MONTH_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=31&c=1 */ \
  V(JSTemporalZonedDateTime, JS_TEMPORAL_ZONED_DATE_TIME_TYPE) /* https://crsrc.org/c/v8/src/objects/js-temporal-objects.tq?l=35&c=1 */ \
  V(JSV8BreakIterator, JS_V8_BREAK_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=7&c=1 */ \
  V(JSValidIteratorWrapper, JS_VALID_ITERATOR_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=204&c=1 */ \
  V(JSWeakRef, JS_WEAK_REF_TYPE) /* https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=43&c=1 */ \
  V(WasmGlobalObject, WASM_GLOBAL_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1 */ \
  V(WasmInstanceObject, WASM_INSTANCE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1 */ \
  V(WasmMemoryMapDescriptor, WASM_MEMORY_MAP_DESCRIPTOR_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1 */ \
  V(WasmMemoryObject, WASM_MEMORY_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1 */ \
  V(WasmModuleObject, WASM_MODULE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1 */ \
  V(WasmSuspendingObject, WASM_SUSPENDING_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1 */ \
  V(WasmTableObject, WASM_TABLE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1 */ \
  V(WasmTagObject, WASM_TAG_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1 */ \

// Pairs of (ClassName, INSTANCE_TYPE) for classes that have
// full Torque definitions and subclasses.
#define TORQUE_INSTANCE_CHECKERS_MULTIPLE_FULLY_DEFINED(V) \
  V(UncompiledDataWithPreparseData, UNCOMPILED_DATA_WITH_PREPARSE_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=172&c=1 */ \
  V(UncompiledDataWithoutPreparseData, UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=169&c=1 */ \
  V(WasmFunctionData, WASM_FUNCTION_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1 */ \
  V(RegExpData, REG_EXP_DATA_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=17&c=1 */ \
  V(FixedArray, FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=11&c=1 */ \
  V(TurboshaftFloat64Type, TURBOSHAFT_FLOAT64_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=65&c=1 */ \
  V(TurboshaftWord32Type, TURBOSHAFT_WORD32_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=20&c=1 */ \
  V(TurboshaftWord64Type, TURBOSHAFT_WORD64_TYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=41&c=1 */ \
  V(ExportedSubClassBase, EXPORTED_SUB_CLASS_BASE_TYPE) /* https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1181&c=1 */ \
  V(DescriptorArray, DESCRIPTOR_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1 */ \
  V(WeakFixedArray, WEAK_FIXED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=36&c=1 */ \
  V(JSObject, JS_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=14&c=1 */ \
  V(JSModuleNamespace, JS_MODULE_NAMESPACE_TYPE) /* https://crsrc.org/c/v8/src/objects/module.tq?l=22&c=1 */ \
  V(JSTypedArray, JS_TYPED_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=118&c=1 */ \
  V(JSFunctionWithPrototype, JS_FUNCTION_WITH_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=55&c=1 */ \
  V(JSDisposableStackBase, JS_DISPOSABLE_STACK_BASE_TYPE) /* https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=24&c=1 */ \
  V(JSGeneratorObject, JS_GENERATOR_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/objects/js-generator.tq?l=5&c=1 */ \
  V(JSObjectWithEmbedderSlots, JS_OBJECT_WITH_EMBEDDER_SLOTS_TYPE) /* https://crsrc.org/c/v8/src/objects/js-objects.tq?l=50&c=1 */ \

// Pairs of (ClassName, INSTANCE_TYPE) for classes that are
// declared but not defined in Torque. These classes may
// correspond with actual C++ classes, but they are not
// guaranteed to.
#define TORQUE_INSTANCE_CHECKERS_SINGLE_ONLY_DECLARED(V) \
  V(BigIntBase, BIG_INT_BASE_TYPE) /* https://crsrc.org/c/v8/src/objects/bigint.tq?l=7&c=1 */ \
  V(LoadHandler, LOAD_HANDLER_TYPE) /* https://crsrc.org/c/v8/src/ic/handler-configuration.tq?l=7&c=1 */ \
  V(StoreHandler, STORE_HANDLER_TYPE) /* https://crsrc.org/c/v8/src/ic/handler-configuration.tq?l=8&c=1 */ \
  V(AccessorPair, ACCESSOR_PAIR_TYPE) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=12&c=1 */ \
  V(ClassPositions, CLASS_POSITIONS_TYPE) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=10&c=1 */ \
  V(CodeWrapper, CODE_WRAPPER_TYPE) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=33&c=1 */ \
  V(Tuple2, TUPLE2_TYPE) /* https://crsrc.org/c/v8/src/objects/struct.tq?l=8&c=1 */ \
  V(Code, CODE_TYPE) /* https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=31&c=1 */ \
  V(WasmDispatchTable, WASM_DISPATCH_TABLE_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=202&c=1 */ \
  V(WasmTrustedInstanceData, WASM_TRUSTED_INSTANCE_DATA_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=37&c=1 */ \
  V(InstructionStream, INSTRUCTION_STREAM_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=293&c=1 */ \
  V(WasmDispatchTableForImports, WASM_DISPATCH_TABLE_FOR_IMPORTS_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=203&c=1 */ \
  V(EphemeronHashTable, EPHEMERON_HASH_TABLE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=275&c=1 */ \
  V(GlobalDictionary, GLOBAL_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=272&c=1 */ \
  V(NameDictionary, NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=271&c=1 */ \
  V(NameToIndexHashTable, NAME_TO_INDEX_HASH_TABLE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=269&c=1 */ \
  V(NumberDictionary, NUMBER_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=278&c=1 */ \
  V(OrderedHashMap, ORDERED_HASH_MAP_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=266&c=1 */ \
  V(OrderedHashSet, ORDERED_HASH_SET_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=267&c=1 */ \
  V(OrderedNameDictionary, ORDERED_NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=268&c=1 */ \
  V(RegisteredSymbolTable, REGISTERED_SYMBOL_TABLE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=270&c=1 */ \
  V(SimpleNameDictionary, SIMPLE_NAME_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=273&c=1 */ \
  V(SimpleNumberDictionary, SIMPLE_NUMBER_DICTIONARY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=274&c=1 */ \
  V(AwaitContext, AWAIT_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=49&c=1 */ \
  V(BlockContext, BLOCK_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=50&c=1 */ \
  V(CatchContext, CATCH_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=51&c=1 */ \
  V(DebugEvaluateContext, DEBUG_EVALUATE_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=52&c=1 */ \
  V(EvalContext, EVAL_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=54&c=1 */ \
  V(FunctionContext, FUNCTION_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=59&c=1 */ \
  V(ModuleContext, MODULE_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=55&c=1 */ \
  V(NativeContext, NATIVE_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=92&c=1 */ \
  V(ScriptContext, SCRIPT_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=56&c=1 */ \
  V(WithContext, WITH_CONTEXT_TYPE) /* https://crsrc.org/c/v8/src/objects/contexts.tq?l=57&c=1 */ \
  V(TransitionArray, TRANSITION_ARRAY_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=299&c=1 */ \
  V(AllocationSite, ALLOCATION_SITE_TYPE) /* https://crsrc.org/c/v8/src/objects/allocation-site.tq?l=5&c=1 */ \
  V(DoubleStringCache, DOUBLE_STRING_CACHE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=279&c=1 */ \
  V(FeedbackMetadata, FEEDBACK_METADATA_TYPE) /* https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=47&c=1 */ \
  V(Filler, FILLER_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=690&c=1 */ \
  V(FreeSpace, FREE_SPACE_TYPE) /* https://crsrc.org/c/v8/src/objects/free-space.tq?l=5&c=1 */ \
  V(JSSpecialApiObject, JS_SPECIAL_API_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=710&c=1 */ \
  V(JSLastDummyApiObject, JS_LAST_DUMMY_API_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=703&c=1 */ \
  V(Bigint64TypedArrayConstructor, BIGINT64_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=186&c=1 */ \
  V(Biguint64TypedArrayConstructor, BIGUINT64_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=183&c=1 */ \
  V(Float16TypedArrayConstructor, FLOAT16_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=171&c=1 */ \
  V(Float32TypedArrayConstructor, FLOAT32_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=174&c=1 */ \
  V(Float64TypedArrayConstructor, FLOAT64_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=177&c=1 */ \
  V(Int16TypedArrayConstructor, INT16_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=162&c=1 */ \
  V(Int32TypedArrayConstructor, INT32_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=168&c=1 */ \
  V(Int8TypedArrayConstructor, INT8_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=156&c=1 */ \
  V(Uint16TypedArrayConstructor, UINT16_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=159&c=1 */ \
  V(Uint32TypedArrayConstructor, UINT32_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=165&c=1 */ \
  V(Uint8ClampedTypedArrayConstructor, UINT8_CLAMPED_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=180&c=1 */ \
  V(Uint8TypedArrayConstructor, UINT8_TYPED_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=153&c=1 */ \
  V(JSArrayConstructor, JS_ARRAY_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-array.tq?l=76&c=1 */ \
  V(JSPromiseConstructor, JS_PROMISE_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-promise.tq?l=38&c=1 */ \
  V(JSRegExpConstructor, JS_REG_EXP_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=95&c=1 */ \
  V(JSClassConstructor, JS_CLASS_CONSTRUCTOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-function.tq?l=63&c=1 */ \
  V(JSArrayIteratorPrototype, JS_ARRAY_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=248&c=1 */ \
  V(JSIteratorPrototype, JS_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=245&c=1 */ \
  V(JSMapIteratorPrototype, JS_MAP_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=251&c=1 */ \
  V(JSObjectPrototype, JS_OBJECT_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=230&c=1 */ \
  V(JSPromisePrototype, JS_PROMISE_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=236&c=1 */ \
  V(JSRegExpPrototype, JS_REG_EXP_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=233&c=1 */ \
  V(JSSetIteratorPrototype, JS_SET_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=254&c=1 */ \
  V(JSSetPrototype, JS_SET_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=242&c=1 */ \
  V(JSStringIteratorPrototype, JS_STRING_ITERATOR_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=257&c=1 */ \
  V(JSTypedArrayPrototype, JS_TYPED_ARRAY_PROTOTYPE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=239&c=1 */ \
  V(JSMapKeyIterator, JS_MAP_KEY_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=24&c=1 */ \
  V(JSMapKeyValueIterator, JS_MAP_KEY_VALUE_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=26&c=1 */ \
  V(JSMapValueIterator, JS_MAP_VALUE_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=28&c=1 */ \
  V(JSSetKeyValueIterator, JS_SET_KEY_VALUE_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=32&c=1 */ \
  V(JSSetValueIterator, JS_SET_VALUE_ITERATOR_TYPE) /* https://crsrc.org/c/v8/src/objects/js-collection.tq?l=34&c=1 */ \
  V(JSContextExtensionObject, JS_CONTEXT_EXTENSION_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=713&c=1 */ \
  V(JSError, JS_ERROR_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=715&c=1 */ \
  V(WasmExceptionPackage, WASM_EXCEPTION_PACKAGE_TYPE) /* https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=195&c=1 */ \
  V(WasmValueObject, WASM_VALUE_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/debug/debug-wasm-objects.tq?l=7&c=1 */ \

// Pairs of (ClassName, INSTANCE_TYPE) for classes that are
// declared but not defined in Torque, and have subclasses.
// These classes may correspond with actual C++ classes, but
// they are not guaranteed to.
#define TORQUE_INSTANCE_CHECKERS_MULTIPLE_ONLY_DECLARED(V) \
  V(HashTable, HASH_TABLE_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=264&c=1 */ \
  V(JSApiObject, JS_API_OBJECT_TYPE) /* https://crsrc.org/c/v8/src/builtins/base.tq?l=696&c=1 */ \

// Triples of (ClassName, FIRST_TYPE, LAST_TYPE) for classes
// that have full Torque definitions.
#define TORQUE_INSTANCE_CHECKERS_RANGE_FULLY_DEFINED(V) \
  V(String, FIRST_STRING_TYPE, LAST_STRING_TYPE) \
  V(Name, FIRST_NAME_TYPE, LAST_NAME_TYPE) \
  V(PrimitiveHeapObject, FIRST_PRIMITIVE_HEAP_OBJECT_TYPE, LAST_PRIMITIVE_HEAP_OBJECT_TYPE) \
  V(PromiseReactionJobTask, FIRST_PROMISE_REACTION_JOB_TASK_TYPE, LAST_PROMISE_REACTION_JOB_TASK_TYPE) \
  V(Microtask, FIRST_MICROTASK_TYPE, LAST_MICROTASK_TYPE) \
  V(DataHandler, FIRST_DATA_HANDLER_TYPE, LAST_DATA_HANDLER_TYPE) \
  V(Struct, FIRST_STRUCT_TYPE, LAST_STRUCT_TYPE) \
  V(UncompiledDataWithPreparseData, FIRST_UNCOMPILED_DATA_WITH_PREPARSE_DATA_TYPE, LAST_UNCOMPILED_DATA_WITH_PREPARSE_DATA_TYPE) \
  V(UncompiledDataWithoutPreparseData, FIRST_UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_TYPE, LAST_UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_TYPE) \
  V(UncompiledData, FIRST_UNCOMPILED_DATA_TYPE, LAST_UNCOMPILED_DATA_TYPE) \
  V(WasmFunctionData, FIRST_WASM_FUNCTION_DATA_TYPE, LAST_WASM_FUNCTION_DATA_TYPE) \
  V(RegExpData, FIRST_REG_EXP_DATA_TYPE, LAST_REG_EXP_DATA_TYPE) \
  V(ExposedTrustedObject, FIRST_EXPOSED_TRUSTED_OBJECT_TYPE, LAST_EXPOSED_TRUSTED_OBJECT_TYPE) \
  V(TrustedObject, FIRST_TRUSTED_OBJECT_TYPE, LAST_TRUSTED_OBJECT_TYPE) \
  V(FixedArray, FIRST_FIXED_ARRAY_TYPE, LAST_FIXED_ARRAY_TYPE) \
  V(FixedArrayBase, FIRST_FIXED_ARRAY_BASE_TYPE, LAST_FIXED_ARRAY_BASE_TYPE) \
  V(Context, FIRST_CONTEXT_TYPE, LAST_CONTEXT_TYPE) \
  V(TurboshaftFloat64Type, FIRST_TURBOSHAFT_FLOAT64_TYPE_TYPE, LAST_TURBOSHAFT_FLOAT64_TYPE_TYPE) \
  V(TurboshaftWord32Type, FIRST_TURBOSHAFT_WORD32_TYPE_TYPE, LAST_TURBOSHAFT_WORD32_TYPE_TYPE) \
  V(TurboshaftWord64Type, FIRST_TURBOSHAFT_WORD64_TYPE_TYPE, LAST_TURBOSHAFT_WORD64_TYPE_TYPE) \
  V(TurboshaftType, FIRST_TURBOSHAFT_TYPE_TYPE, LAST_TURBOSHAFT_TYPE_TYPE) \
  V(TurbofanType, FIRST_TURBOFAN_TYPE_TYPE, LAST_TURBOFAN_TYPE_TYPE) \
  V(ExportedSubClassBase, FIRST_EXPORTED_SUB_CLASS_BASE_TYPE, LAST_EXPORTED_SUB_CLASS_BASE_TYPE) \
  V(SmallOrderedHashTable, FIRST_SMALL_ORDERED_HASH_TABLE_TYPE, LAST_SMALL_ORDERED_HASH_TABLE_TYPE) \
  V(TemplateInfoWithProperties, FIRST_TEMPLATE_INFO_WITH_PROPERTIES_TYPE, LAST_TEMPLATE_INFO_WITH_PROPERTIES_TYPE) \
  V(TemplateInfo, FIRST_TEMPLATE_INFO_TYPE, LAST_TEMPLATE_INFO_TYPE) \
  V(AbstractInternalClass, FIRST_ABSTRACT_INTERNAL_CLASS_TYPE, LAST_ABSTRACT_INTERNAL_CLASS_TYPE) \
  V(DescriptorArray, FIRST_DESCRIPTOR_ARRAY_TYPE, LAST_DESCRIPTOR_ARRAY_TYPE) \
  V(Module, FIRST_MODULE_TYPE, LAST_MODULE_TYPE) \
  V(WeakFixedArray, FIRST_WEAK_FIXED_ARRAY_TYPE, LAST_WEAK_FIXED_ARRAY_TYPE) \
  V(WasmObject, FIRST_WASM_OBJECT_TYPE, LAST_WASM_OBJECT_TYPE) \
  V(JSModuleNamespace, FIRST_JS_MODULE_NAMESPACE_TYPE, LAST_JS_MODULE_NAMESPACE_TYPE) \
  V(JSSpecialObject, FIRST_JS_SPECIAL_OBJECT_TYPE, LAST_JS_SPECIAL_OBJECT_TYPE) \
  V(JSCustomElementsObject, FIRST_JS_CUSTOM_ELEMENTS_OBJECT_TYPE, LAST_JS_CUSTOM_ELEMENTS_OBJECT_TYPE) \
  V(JSDataViewOrRabGsabDataView, FIRST_JS_DATA_VIEW_OR_RAB_GSAB_DATA_VIEW_TYPE, LAST_JS_DATA_VIEW_OR_RAB_GSAB_DATA_VIEW_TYPE) \
  V(JSTypedArray, FIRST_JS_TYPED_ARRAY_TYPE, LAST_JS_TYPED_ARRAY_TYPE) \
  V(JSArrayBufferView, FIRST_JS_ARRAY_BUFFER_VIEW_TYPE, LAST_JS_ARRAY_BUFFER_VIEW_TYPE) \
  V(JSAPIObjectWithEmbedderSlots, FIRST_JS_APIOBJECT_WITH_EMBEDDER_SLOTS_TYPE, LAST_JS_APIOBJECT_WITH_EMBEDDER_SLOTS_TYPE) \
  V(JSFunctionWithPrototype, FIRST_JS_FUNCTION_WITH_PROTOTYPE_TYPE, LAST_JS_FUNCTION_WITH_PROTOTYPE_TYPE) \
  V(JSFunction, FIRST_JS_FUNCTION_TYPE, LAST_JS_FUNCTION_TYPE) \
  V(JSFunctionOrBoundFunctionOrWrappedFunction, FIRST_JS_FUNCTION_OR_BOUND_FUNCTION_OR_WRAPPED_FUNCTION_TYPE, LAST_JS_FUNCTION_OR_BOUND_FUNCTION_OR_WRAPPED_FUNCTION_TYPE) \
  V(JSIteratorHelperSimple, FIRST_JS_ITERATOR_HELPER_SIMPLE_TYPE, LAST_JS_ITERATOR_HELPER_SIMPLE_TYPE) \
  V(JSIteratorHelper, FIRST_JS_ITERATOR_HELPER_TYPE, LAST_JS_ITERATOR_HELPER_TYPE) \
  V(JSCollectionIterator, FIRST_JS_COLLECTION_ITERATOR_TYPE, LAST_JS_COLLECTION_ITERATOR_TYPE) \
  V(JSSynchronizationPrimitive, FIRST_JS_SYNCHRONIZATION_PRIMITIVE_TYPE, LAST_JS_SYNCHRONIZATION_PRIMITIVE_TYPE) \
  V(AlwaysSharedSpaceJSObject, FIRST_ALWAYS_SHARED_SPACE_JS_OBJECT_TYPE, LAST_ALWAYS_SHARED_SPACE_JS_OBJECT_TYPE) \
  V(JSDisposableStackBase, FIRST_JS_DISPOSABLE_STACK_BASE_TYPE, LAST_JS_DISPOSABLE_STACK_BASE_TYPE) \
  V(JSGeneratorObject, FIRST_JS_GENERATOR_OBJECT_TYPE, LAST_JS_GENERATOR_OBJECT_TYPE) \
  V(JSCollection, FIRST_JS_COLLECTION_TYPE, LAST_JS_COLLECTION_TYPE) \
  V(JSObjectWithEmbedderSlots, FIRST_JS_OBJECT_WITH_EMBEDDER_SLOTS_TYPE, LAST_JS_OBJECT_WITH_EMBEDDER_SLOTS_TYPE) \
  V(JSWeakCollection, FIRST_JS_WEAK_COLLECTION_TYPE, LAST_JS_WEAK_COLLECTION_TYPE) \
  V(JSObject, FIRST_JS_OBJECT_TYPE, LAST_JS_OBJECT_TYPE) \
  V(JSReceiver, FIRST_JS_RECEIVER_TYPE, LAST_JS_RECEIVER_TYPE) \

// Triples of (ClassName, FIRST_TYPE, LAST_TYPE) for classes
// that are declared but not defined in Torque. These classes
// may correspond with actual C++ classes, but they are not
// guaranteed to.
#define TORQUE_INSTANCE_CHECKERS_RANGE_ONLY_DECLARED(V) \
  V(HashTable, FIRST_HASH_TABLE_TYPE, LAST_HASH_TABLE_TYPE) \
  V(JSApiObject, FIRST_JS_API_OBJECT_TYPE, LAST_JS_API_OBJECT_TYPE) \
  V(TypedArrayConstructor, FIRST_TYPED_ARRAY_CONSTRUCTOR_TYPE, LAST_TYPED_ARRAY_CONSTRUCTOR_TYPE) \
  V(JSInternalPrototypeBase, FIRST_JS_INTERNAL_PROTOTYPE_BASE_TYPE, LAST_JS_INTERNAL_PROTOTYPE_BASE_TYPE) \
  V(JSMapIterator, FIRST_JS_MAP_ITERATOR_TYPE, LAST_JS_MAP_ITERATOR_TYPE) \
  V(JSSetIterator, FIRST_JS_SET_ITERATOR_TYPE, LAST_JS_SET_ITERATOR_TYPE) \

// Fully Torque-defined classes (both internal and exported).
#define TORQUE_DEFINED_CLASS_LIST(V) \
  V(SloppyArgumentsElements) \
  V(StrongDescriptorArray) \
  V(OnHeapBasicBlockProfilerData) \
  V(TurbofanType) \
  V(TurbofanBitsetType) \
  V(TurbofanUnionType) \
  V(TurbofanRangeType) \
  V(TurbofanHeapConstantType) \
  V(TurbofanOtherNumberConstantType) \
  V(WasmFastApiCallData) \
  V(WasmStringViewIter) \
  V(InternalClass) \
  V(SmiPair) \
  V(SmiBox) \
  V(ExportedSubClassBase) \
  V(ExportedSubClass) \
  V(AbstractInternalClass) \
  V(AbstractInternalClassSubclass1) \
  V(AbstractInternalClassSubclass2) \
  V(InternalClassWithStructElements) \
  V(ExportedSubClass2) \
  V(SortState) \

#define TORQUE_DEFINED_VARSIZE_INSTANCE_TYPE_LIST(V) \
  V(SLOPPY_ARGUMENTS_ELEMENTS_TYPE, SloppyArgumentsElements, sloppy_arguments_elements) \
  V(DESCRIPTOR_ARRAY_TYPE, DescriptorArray, descriptor_array) \
  V(STRONG_DESCRIPTOR_ARRAY_TYPE, StrongDescriptorArray, strong_descriptor_array) \
  V(TURBOSHAFT_WORD32_SET_TYPE_TYPE, TurboshaftWord32SetType, turboshaft_word32set_type) \
  V(TURBOSHAFT_WORD64_SET_TYPE_TYPE, TurboshaftWord64SetType, turboshaft_word64set_type) \
  V(TURBOSHAFT_FLOAT64_SET_TYPE_TYPE, TurboshaftFloat64SetType, turboshaft_float64set_type) \
  V(INTERNAL_CLASS_WITH_STRUCT_ELEMENTS_TYPE, InternalClassWithStructElements, internal_class_with_struct_elements) \

#define TORQUE_DEFINED_FIXED_INSTANCE_TYPE_LIST(V) \
  V(FUNCTION_TEMPLATE_INFO_TYPE, FunctionTemplateInfo, function_template_info) \
  V(ON_HEAP_BASIC_BLOCK_PROFILER_DATA_TYPE, OnHeapBasicBlockProfilerData, on_heap_basic_block_profiler_data) \
  V(OBJECT_TEMPLATE_INFO_TYPE, ObjectTemplateInfo, object_template_info) \
  V(TURBOFAN_BITSET_TYPE_TYPE, TurbofanBitsetType, turbofan_bitset_type) \
  V(TURBOFAN_UNION_TYPE_TYPE, TurbofanUnionType, turbofan_union_type) \
  V(TURBOFAN_RANGE_TYPE_TYPE, TurbofanRangeType, turbofan_range_type) \
  V(TURBOFAN_HEAP_CONSTANT_TYPE_TYPE, TurbofanHeapConstantType, turbofan_heap_constant_type) \
  V(TURBOFAN_OTHER_NUMBER_CONSTANT_TYPE_TYPE, TurbofanOtherNumberConstantType, turbofan_other_number_constant_type) \
  V(TURBOSHAFT_WORD32_TYPE_TYPE, TurboshaftWord32Type, turboshaft_word32type) \
  V(TURBOSHAFT_WORD32_RANGE_TYPE_TYPE, TurboshaftWord32RangeType, turboshaft_word32range_type) \
  V(TURBOSHAFT_WORD64_TYPE_TYPE, TurboshaftWord64Type, turboshaft_word64type) \
  V(TURBOSHAFT_WORD64_RANGE_TYPE_TYPE, TurboshaftWord64RangeType, turboshaft_word64range_type) \
  V(TURBOSHAFT_FLOAT64_TYPE_TYPE, TurboshaftFloat64Type, turboshaft_float64type) \
  V(TURBOSHAFT_FLOAT64_RANGE_TYPE_TYPE, TurboshaftFloat64RangeType, turboshaft_float64range_type) \
  V(WASM_FAST_API_CALL_DATA_TYPE, WasmFastApiCallData, wasm_fast_api_call_data) \
  V(WASM_STRING_VIEW_ITER_TYPE, WasmStringViewIter, wasm_string_view_iter) \
  V(INTERNAL_CLASS_TYPE, InternalClass, internal_class) \
  V(SMI_PAIR_TYPE, SmiPair, smi_pair) \
  V(SMI_BOX_TYPE, SmiBox, smi_box) \
  V(EXPORTED_SUB_CLASS_BASE_TYPE, ExportedSubClassBase, exported_sub_class_base) \
  V(EXPORTED_SUB_CLASS_TYPE, ExportedSubClass, exported_sub_class) \
  V(ABSTRACT_INTERNAL_CLASS_SUBCLASS1_TYPE, AbstractInternalClassSubclass1, abstract_internal_class_subclass1) \
  V(ABSTRACT_INTERNAL_CLASS_SUBCLASS2_TYPE, AbstractInternalClassSubclass2, abstract_internal_class_subclass2) \
  V(EXPORTED_SUB_CLASS2_TYPE, ExportedSubClass2, exported_sub_class2) \
  V(SORT_STATE_TYPE, SortState, sort_state) \

#define TORQUE_DEFINED_INSTANCE_TYPE_LIST(V) \
  TORQUE_DEFINED_VARSIZE_INSTANCE_TYPE_LIST(V) \
  TORQUE_DEFINED_FIXED_INSTANCE_TYPE_LIST(V) \

#define TORQUE_DEFINED_MAP_CSA_LIST_GENERATOR(V, _) \
  V(_, FunctionTemplateInfoMap, function_template_info_map, FunctionTemplateInfo) \
  V(_, SloppyArgumentsElementsMap, sloppy_arguments_elements_map, SloppyArgumentsElements) \
  V(_, DescriptorArrayMap, descriptor_array_map, DescriptorArray) \
  V(_, StrongDescriptorArrayMap, strong_descriptor_array_map, StrongDescriptorArray) \
  V(_, OnHeapBasicBlockProfilerDataMap, on_heap_basic_block_profiler_data_map, OnHeapBasicBlockProfilerData) \
  V(_, ObjectTemplateInfoMap, object_template_info_map, ObjectTemplateInfo) \
  V(_, TurbofanBitsetTypeMap, turbofan_bitset_type_map, TurbofanBitsetType) \
  V(_, TurbofanUnionTypeMap, turbofan_union_type_map, TurbofanUnionType) \
  V(_, TurbofanRangeTypeMap, turbofan_range_type_map, TurbofanRangeType) \
  V(_, TurbofanHeapConstantTypeMap, turbofan_heap_constant_type_map, TurbofanHeapConstantType) \
  V(_, TurbofanOtherNumberConstantTypeMap, turbofan_other_number_constant_type_map, TurbofanOtherNumberConstantType) \
  V(_, TurboshaftWord32TypeMap, turboshaft_word32type_map, TurboshaftWord32Type) \
  V(_, TurboshaftWord32RangeTypeMap, turboshaft_word32range_type_map, TurboshaftWord32RangeType) \
  V(_, TurboshaftWord32SetTypeMap, turboshaft_word32set_type_map, TurboshaftWord32SetType) \
  V(_, TurboshaftWord64TypeMap, turboshaft_word64type_map, TurboshaftWord64Type) \
  V(_, TurboshaftWord64RangeTypeMap, turboshaft_word64range_type_map, TurboshaftWord64RangeType) \
  V(_, TurboshaftWord64SetTypeMap, turboshaft_word64set_type_map, TurboshaftWord64SetType) \
  V(_, TurboshaftFloat64TypeMap, turboshaft_float64type_map, TurboshaftFloat64Type) \
  V(_, TurboshaftFloat64RangeTypeMap, turboshaft_float64range_type_map, TurboshaftFloat64RangeType) \
  V(_, TurboshaftFloat64SetTypeMap, turboshaft_float64set_type_map, TurboshaftFloat64SetType) \
  V(_, WasmFastApiCallDataMap, wasm_fast_api_call_data_map, WasmFastApiCallData) \
  V(_, WasmStringViewIterMap, wasm_string_view_iter_map, WasmStringViewIter) \
  V(_, InternalClassMap, internal_class_map, InternalClass) \
  V(_, SmiPairMap, smi_pair_map, SmiPair) \
  V(_, SmiBoxMap, smi_box_map, SmiBox) \
  V(_, ExportedSubClassBaseMap, exported_sub_class_base_map, ExportedSubClassBase) \
  V(_, ExportedSubClassMap, exported_sub_class_map, ExportedSubClass) \
  V(_, AbstractInternalClassSubclass1Map, abstract_internal_class_subclass1_map, AbstractInternalClassSubclass1) \
  V(_, AbstractInternalClassSubclass2Map, abstract_internal_class_subclass2_map, AbstractInternalClassSubclass2) \
  V(_, InternalClassWithStructElementsMap, internal_class_with_struct_elements_map, InternalClassWithStructElements) \
  V(_, ExportedSubClass2Map, exported_sub_class2_map, ExportedSubClass2) \
  V(_, SortStateMap, sort_state_map, SortState) \

#define TORQUE_DEFINED_MAP_ROOT_LIST(V) \
  V(Map, function_template_info_map, FunctionTemplateInfoMap) \
  V(Map, sloppy_arguments_elements_map, SloppyArgumentsElementsMap) \
  V(Map, descriptor_array_map, DescriptorArrayMap) \
  V(Map, strong_descriptor_array_map, StrongDescriptorArrayMap) \
  V(Map, on_heap_basic_block_profiler_data_map, OnHeapBasicBlockProfilerDataMap) \
  V(Map, object_template_info_map, ObjectTemplateInfoMap) \
  V(Map, turbofan_bitset_type_map, TurbofanBitsetTypeMap) \
  V(Map, turbofan_union_type_map, TurbofanUnionTypeMap) \
  V(Map, turbofan_range_type_map, TurbofanRangeTypeMap) \
  V(Map, turbofan_heap_constant_type_map, TurbofanHeapConstantTypeMap) \
  V(Map, turbofan_other_number_constant_type_map, TurbofanOtherNumberConstantTypeMap) \
  V(Map, turboshaft_word32type_map, TurboshaftWord32TypeMap) \
  V(Map, turboshaft_word32range_type_map, TurboshaftWord32RangeTypeMap) \
  V(Map, turboshaft_word32set_type_map, TurboshaftWord32SetTypeMap) \
  V(Map, turboshaft_word64type_map, TurboshaftWord64TypeMap) \
  V(Map, turboshaft_word64range_type_map, TurboshaftWord64RangeTypeMap) \
  V(Map, turboshaft_word64set_type_map, TurboshaftWord64SetTypeMap) \
  V(Map, turboshaft_float64type_map, TurboshaftFloat64TypeMap) \
  V(Map, turboshaft_float64range_type_map, TurboshaftFloat64RangeTypeMap) \
  V(Map, turboshaft_float64set_type_map, TurboshaftFloat64SetTypeMap) \
  V(Map, wasm_fast_api_call_data_map, WasmFastApiCallDataMap) \
  V(Map, wasm_string_view_iter_map, WasmStringViewIterMap) \
  V(Map, internal_class_map, InternalClassMap) \
  V(Map, smi_pair_map, SmiPairMap) \
  V(Map, smi_box_map, SmiBoxMap) \
  V(Map, exported_sub_class_base_map, ExportedSubClassBaseMap) \
  V(Map, exported_sub_class_map, ExportedSubClassMap) \
  V(Map, abstract_internal_class_subclass1_map, AbstractInternalClassSubclass1Map) \
  V(Map, abstract_internal_class_subclass2_map, AbstractInternalClassSubclass2Map) \
  V(Map, internal_class_with_struct_elements_map, InternalClassWithStructElementsMap) \
  V(Map, exported_sub_class2_map, ExportedSubClass2Map) \
  V(Map, sort_state_map, SortStateMap) \

#endif  // V8_GEN_TORQUE_GENERATED_INSTANCE_TYPES_H_
