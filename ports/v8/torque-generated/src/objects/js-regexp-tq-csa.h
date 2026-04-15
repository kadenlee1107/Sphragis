#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_REGEXP_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_REGEXP_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=17&c=1
TNode<RegExpData> Cast_RegExpData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=28&c=1
TNode<AtomRegExpData> Cast_AtomRegExpData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=38&c=1
TNode<IrRegExpData> Cast_IrRegExpData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=54&c=1
TNode<RegExpDataWrapper> Cast_RegExpDataWrapper_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=59&c=1
TNode<JSRegExp> Cast_JSRegExp_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=89&c=1
TNode<String> RegExpSource_0(compiler::CodeAssemblerState* state_, TNode<JSRegExp> p_regexp);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=19&c=3
TNode<Smi> LoadRegExpDataTypeTag_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=19&c=3
void StoreRegExpDataTypeTag_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=20&c=3
TNode<String> LoadRegExpDataOriginalSource_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=20&c=3
void StoreRegExpDataOriginalSource_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o, TNode<String> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=21&c=3
TNode<String> LoadRegExpDataEscapedSource_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=21&c=3
void StoreRegExpDataEscapedSource_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o, TNode<String> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=22&c=3
TNode<Smi> LoadRegExpDataFlags_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=22&c=3
void StoreRegExpDataFlags_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=23&c=3
TNode<RegExpDataWrapper> LoadRegExpDataWrapper_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=23&c=3
void StoreRegExpDataWrapper_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o, TNode<RegExpDataWrapper> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=24&c=3
TNode<Uint32T> LoadRegExpDataQuickCheckMask_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=24&c=3
void StoreRegExpDataQuickCheckMask_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=25&c=3
TNode<Uint32T> LoadRegExpDataQuickCheckValue_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=25&c=3
void StoreRegExpDataQuickCheckValue_0(compiler::CodeAssemblerState* state_, TNode<RegExpData> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=30&c=3
TNode<String> LoadAtomRegExpDataPattern_0(compiler::CodeAssemblerState* state_, TNode<AtomRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=30&c=3
void StoreAtomRegExpDataPattern_0(compiler::CodeAssemblerState* state_, TNode<AtomRegExpData> p_o, TNode<String> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=42&c=3
TNode<MaybeObject> LoadIrRegExpDataLatin1Bytecode_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=42&c=3
void StoreIrRegExpDataLatin1Bytecode_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=43&c=3
TNode<MaybeObject> LoadIrRegExpDataUc16Bytecode_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=43&c=3
void StoreIrRegExpDataUc16Bytecode_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=44&c=3
TNode<MaybeObject> LoadIrRegExpDataCaptureNameMap_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=44&c=3
void StoreIrRegExpDataCaptureNameMap_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=45&c=3
TNode<TrustedPointerT> LoadIrRegExpDataLatin1Code_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=45&c=3
void StoreIrRegExpDataLatin1Code_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=46&c=3
TNode<TrustedPointerT> LoadIrRegExpDataUc16Code_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=46&c=3
void StoreIrRegExpDataUc16Code_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=47&c=3
TNode<Smi> LoadIrRegExpDataMaxRegisterCount_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=47&c=3
void StoreIrRegExpDataMaxRegisterCount_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=48&c=3
TNode<Smi> LoadIrRegExpDataCaptureCount_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=48&c=3
void StoreIrRegExpDataCaptureCount_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=49&c=3
TNode<Smi> LoadIrRegExpDataTicksUntilTierUp_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=49&c=3
void StoreIrRegExpDataTicksUntilTierUp_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=50&c=3
TNode<Smi> LoadIrRegExpDataBacktrackLimit_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=50&c=3
void StoreIrRegExpDataBacktrackLimit_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=51&c=3
TNode<Smi> LoadIrRegExpDataBitField_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=51&c=3
void StoreIrRegExpDataBitField_0(compiler::CodeAssemblerState* state_, TNode<IrRegExpData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=56&c=3
TNode<TrustedPointerT> LoadRegExpDataWrapperData_0(compiler::CodeAssemblerState* state_, TNode<RegExpDataWrapper> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=56&c=3
void StoreRegExpDataWrapperData_0(compiler::CodeAssemblerState* state_, TNode<RegExpDataWrapper> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=60&c=3
TNode<TrustedPointerT> LoadJSRegExpData_0(compiler::CodeAssemblerState* state_, TNode<JSRegExp> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=60&c=3
void StoreJSRegExpData_0(compiler::CodeAssemblerState* state_, TNode<JSRegExp> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=61&c=3
TNode<Union<Smi, Undefined>> LoadJSRegExpFlags_0(compiler::CodeAssemblerState* state_, TNode<JSRegExp> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=61&c=3
void StoreJSRegExpFlags_0(compiler::CodeAssemblerState* state_, TNode<JSRegExp> p_o, TNode<Union<Smi, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=102&c=3
TNode<JSAny> LoadJSRegExpResultIndex_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=102&c=3
void StoreJSRegExpResultIndex_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=103&c=3
TNode<JSAny> LoadJSRegExpResultInput_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=103&c=3
void StoreJSRegExpResultInput_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=104&c=3
TNode<JSAny> LoadJSRegExpResultGroups_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=104&c=3
void StoreJSRegExpResultGroups_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=107&c=3
TNode<String> LoadJSRegExpResultRegexpInput_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=107&c=3
void StoreJSRegExpResultRegexpInput_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o, TNode<String> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=108&c=3
TNode<Smi> LoadJSRegExpResultRegexpLastIndex_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=108&c=3
void StoreJSRegExpResultRegexpLastIndex_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=112&c=3
TNode<JSAny> LoadJSRegExpResultWithIndicesIndices_0(compiler::CodeAssemblerState* state_, TNode<JSRegExpResult> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=112&c=3
void StoreJSRegExpResultWithIndicesIndices_0(compiler::CodeAssemblerState* state_, TNode<JSRegExpResult> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=118&c=3
TNode<JSAny> LoadJSRegExpResultIndicesGroups_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=118&c=3
void StoreJSRegExpResultIndicesGroups_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=17&c=1
TNode<RegExpData> DownCastForTorqueClass_RegExpData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=28&c=1
TNode<AtomRegExpData> DownCastForTorqueClass_AtomRegExpData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=38&c=1
TNode<IrRegExpData> DownCastForTorqueClass_IrRegExpData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=54&c=1
TNode<RegExpDataWrapper> DownCastForTorqueClass_RegExpDataWrapper_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=59&c=1
TNode<JSRegExp> DownCastForTorqueClass_JSRegExp_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_REGEXP_TQ_CSA_H_
