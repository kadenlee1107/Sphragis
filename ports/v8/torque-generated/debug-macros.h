#ifndef V8_GEN_TORQUE_GENERATED_DEBUG_MACROS_H_
#define V8_GEN_TORQUE_GENERATED_DEBUG_MACROS_H_

#include "tools/debug_helper/debug-helper-internal.h"
#include "src/numbers/integer-literal.h"

namespace v8 {
namespace internal {
namespace debug_helper_internal {

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=14&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceFixedArrayObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/contexts.tq?l=14&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceContextElements(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=39&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceWeakFixedArrayObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=31&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSloppyArgumentsElementsMappedEntries(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=86&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceWeakArrayListObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=20&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceBytecodeArrayBytes(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/contexts.tq?l=22&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScriptContextTableObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=142&c=9
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoModuleVariableCount(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=148&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoContextLocalNames(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=153&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoContextLocalNamesHashtable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=159&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoContextLocalInfos(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=164&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoSavedClassVariableInfo(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=170&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoFunctionVariableInfo(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=175&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoInferredFunctionName(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=177&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoOuterScopeInfo(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=181&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoModuleInfo(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoModuleVariables(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=187&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoDependentCode(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=191&c=9
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceScopeInfoUnusedParameterBits(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=33&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceFixedDoubleArrayValues(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/regexp-match-info.tq?l=19&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceRegExpMatchInfoObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceCoverageInfoSlots(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceDescriptorArrayDescriptors(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=30&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceClosureFeedbackCellArrayObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=44&c=36
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceFeedbackVectorRawFeedbackSlots(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=22&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceTrustedFixedArrayObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=28&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceProtectedFixedArrayObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=45&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceWeakHomomorphicFixedArrayObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=51&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceTrustedWeakFixedArrayObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=59&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceProtectedWeakFixedArrayObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=64&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceByteArrayValues(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=70&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceTrustedByteArrayValues(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=79&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceArrayListObjects(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/literal-objects.tq?l=10&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceObjectBoilerplateDescriptionRawEntries(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=31&c=31
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedHashSetPadding(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=32&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedHashSetDataTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=34&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedHashSetHashTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=35&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedHashSetChainTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=70&c=31
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedHashMapPadding(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=71&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedHashMapDataTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=73&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedHashMapHashTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=74&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedHashMapChainTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=110&c=31
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedNameDictionaryPadding1(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=111&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedNameDictionaryDataTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=113&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedNameDictionaryHashTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=114&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSmallOrderedNameDictionaryChainTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/prototype-info.tq?l=43&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSlicePrototypeInfoCachedHandler(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/string.tq?l=141&c=9
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSeqOneByteStringChars(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/string.tq?l=147&c=9
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSeqTwoByteStringChars(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/swiss-name-dictionary.tq?l=12&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSwissNameDictionaryDataTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/swiss-name-dictionary.tq?l=13&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSwissNameDictionaryCtrlTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/swiss-name-dictionary.tq?l=14&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceSwissNameDictionaryPropertyDetailsTable(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=38&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceTurboshaftWord32SetTypeElements(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=61&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceTurboshaftWord64SetTypeElementsHigh(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=62&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceTurboshaftWord64SetTypeElementsLow(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=86&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceTurboshaftFloat64SetTypeElements(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceWasmTypeInfoSupertypes(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1205&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceInternalClassWithStructElementsEntries(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugFieldSliceInternalClassWithStructElementsMoreEntries(d::MemoryAccessor accessor, uintptr_t p_o);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=14&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Object_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=31&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Smi_OR_TheHole_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=20&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_uint8_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=127&c=1
Value<uint32_t> TqDebugFromConstexpr_uint32_constexpr_uint32_0(d::MemoryAccessor accessor, uint32_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/builtins/builtins-string.tq?l=217&c=26
Value<intptr_t> TqDebugConvert_intptr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=142&c=9
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewConstSlice_Smi_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=76&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int32_0(d::MemoryAccessor accessor, int32_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=56&c=1
Value<uintptr_t> TqDebugFromConstexpr_Smi_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=148&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_String_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=153&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_NameToIndexHashTable_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=159&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_SmiTagged_VariableProperties_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=164&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Smi_OR_Name_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=27&c=1
Value<uint32_t> TqDebugFromConstexpr_VariableAllocationInfo_constexpr_NONE_0(d::MemoryAccessor accessor, VariableAllocationInfo p_o);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=170&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_FunctionVariableInfo_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=175&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_String_OR_Undefined_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=177&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_ScopeInfo_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=181&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_SourceTextModuleInfo_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/torque-internal.tq?l=125&c=23
Value<intptr_t> TqDebugConvert_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=408&c=1
Value<uintptr_t> TqDebugConvert_uintptr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/torque-internal.tq?l=109&c=36
Value<intptr_t> TqDebugTimesSizeOf_Smi_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/torque-internal.tq?l=108&c=12
Value<std::tuple<uintptr_t, intptr_t>> TqDebugNewReference_Smi_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_ModuleVariable_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=187&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_DependentCode_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=212&c=1
Value<intptr_t> TqDebugConvert_intptr_int32_0(d::MemoryAccessor accessor, int32_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_CoverageInfoSlot_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=239&c=1
Value<intptr_t> TqDebugConvert_intptr_uint16_0(d::MemoryAccessor accessor, uint16_t p_ui);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_DescriptorEntry_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=212&c=1
Value<intptr_t> TqDebugConvert_intptr_int32_0(d::MemoryAccessor accessor, int32_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=39&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_MaybeObject_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/base.tq?l=667&c=47
Value<intptr_t> TqDebugConvert_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=20&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_uint8_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=242&c=1
Value<intptr_t> TqDebugConvert_intptr_uint8_0(d::MemoryAccessor accessor, uint8_t p_ui);

// https://crsrc.org/c/v8/src/builtins/builtins-string.tq?l=217&c=26
Value<intptr_t> TqDebugConvert_intptr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=32&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Smi_OR_HeapNumber_OR_BigInt_OR_String_OR_Symbol_OR_Boolean_OR_Null_OR_Undefined_OR_JSReceiver_OR_TheHole_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=71&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_HashMapEntry_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=111&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_NameDictionaryEntry_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=212&c=1
Value<intptr_t> TqDebugConvert_intptr_int32_0(d::MemoryAccessor accessor, int32_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/builtins/builtins-string.tq?l=217&c=26
Value<intptr_t> TqDebugConvert_intptr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/objects/ordered-hash-table.tq?l=32&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Smi_OR_HeapNumber_OR_BigInt_OR_String_OR_Symbol_OR_Boolean_OR_Null_OR_Undefined_OR_JSReceiver_OR_TheHole_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=76&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int32_0(d::MemoryAccessor accessor, int32_t p_i);

// https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=20&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_uint8_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=218&c=1
Value<intptr_t> TqDebugConvert_intptr_uint32_0(d::MemoryAccessor accessor, uint32_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=38&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_uint32_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/turboshaft-types.tq?l=86&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_float64_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=14&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Object_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/regexp-match-info.tq?l=19&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Smi_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_InternalClassStructElement_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=57&c=10
Value<uintptr_t> TqDebugConvert_Smi_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=85&c=1
Value<uintptr_t> TqDebugFromConstexpr_Smi_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=14&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Object_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=39&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_MaybeObject_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/contexts.tq?l=22&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Context_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=33&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_float64_or_undefined_or_hole_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/regexp-match-info.tq?l=19&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Smi_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=30&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_FeedbackCell_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=28&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Smi_OR_TrustedObject_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/objects/bytecode-array.tq?l=20&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_uint8_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
Value<intptr_t> TqDebugConvert_intptr_Smi_0(d::MemoryAccessor accessor, uintptr_t p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/fixed-array.tq?l=14&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Object_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/torque-internal.tq?l=125&c=23
Value<intptr_t> TqDebugConvert_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/prototype-info.tq?l=43&c=3
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewMutableSlice_Zero_OR_LoadHandler_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=212&c=1
Value<intptr_t> TqDebugConvert_intptr_int32_0(d::MemoryAccessor accessor, int32_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_IntegerLiteral_0(d::MemoryAccessor accessor, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/string.tq?l=141&c=9
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewConstSlice_char8_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/wasm.tq?l=1158&c=26
Value<std::tuple<uintptr_t, intptr_t, intptr_t>> TqDebugNewConstSlice_char16_0(d::MemoryAccessor accessor, uintptr_t p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_int31_0(d::MemoryAccessor accessor, int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
Value<intptr_t> TqDebugFromConstexpr_intptr_constexpr_intptr_0(d::MemoryAccessor accessor, intptr_t p_i);


}  // namespace internal
}  // namespace v8
}  // namespace debug_helper_internal

#endif  // V8_GEN_TORQUE_GENERATED_DEBUG_MACROS_H_
