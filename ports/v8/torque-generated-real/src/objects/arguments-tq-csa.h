#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_ARGUMENTS_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_ARGUMENTS_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1
TNode<JSArgumentsObject> Cast_JSArgumentsObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=10&c=1
TNode<BoolT> IsJSArgumentsObjectWithLength_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=27&c=1
TNode<SloppyArgumentsElements> Cast_SloppyArgumentsElements_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=41&c=1
TNode<AliasedArgumentsEntry> Cast_AliasedArgumentsEntry_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=49&c=1
TNode<JSArgumentsObject> NewJSStrictArgumentsObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArray> p_elements);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=60&c=1
TNode<JSArgumentsObject> NewJSSloppyArgumentsObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArrayBase> p_elements, TNode<JSFunction> p_callee);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=73&c=1
TNode<JSArgumentsObject> NewJSFastAliasedArgumentsObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArrayBase> p_elements, TNode<Smi> p_length, TNode<JSFunction> p_callee);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=97&c=1
TorqueStructParameterMapIterator_0 NewParameterMapIterator_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_formalParameterCount, TNode<IntPtrT> p_mappedCount);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=137&c=1
TorqueStructParameterValueIterator_0 NewParameterValueIterator_0(compiler::CodeAssemblerState* state_, TNode<IntPtrT> p_mappedCount, TorqueStructArguments p_arguments);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=146&c=1
TNode<JSArray> NewAllArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<RawPtrT> p_frame, TNode<IntPtrT> p_argumentCount);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=156&c=1
TNode<FixedArray> NewRestArgumentsElements_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_frame, TNode<IntPtrT> p_formalParameterCount, TNode<IntPtrT> p_argumentCount);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=167&c=1
TNode<JSArray> NewRestArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructFrameWithArgumentsInfo_0 p_info);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=177&c=1
TNode<FixedArray> NewStrictArgumentsElements_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_frame, TNode<IntPtrT> p_argumentCount);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=184&c=1
TNode<JSArgumentsObject> NewStrictArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructFrameWithArgumentsInfo_0 p_info);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=192&c=1
TNode<FixedArray> NewSloppyArgumentsElements_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_frame, TNode<IntPtrT> p_formalParameterCount, TNode<IntPtrT> p_argumentCount);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=205&c=1
TNode<JSArgumentsObject> NewSloppyArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructFrameWithArgumentsInfo_0 p_info, TNode<JSFunction> p_callee);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=227&c=1
TNode<JSArray> EmitFastNewAllArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<RawPtrT> p_frame, TNode<IntPtrT> p_argc);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=234&c=1
TNode<JSArray> EmitFastNewRestArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p__f);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=241&c=1
TNode<JSArgumentsObject> EmitFastNewStrictArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p__f);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=248&c=1
TNode<JSArgumentsObject> EmitFastNewSloppyArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p_f);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=276&c=1
TNode<JSArray> NewRestArgumentsFromArguments_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructArguments p_arguments, TNode<IntPtrT> p_start);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=300&c=1
TorqueStructReference_Object_0 AccessSloppyArgumentsCommon_0(compiler::CodeAssemblerState* state_, TNode<JSObject> p_receiver, TNode<Object> p_keyObject, compiler::CodeAssemblerLabel* label_Bailout);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=331&c=1
TNode<JSAny> SloppyArgumentsLoad_0(compiler::CodeAssemblerState* state_, TNode<JSObject> p_receiver, TNode<Object> p_keyObject, compiler::CodeAssemblerLabel* label_Bailout);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=338&c=1
TNode<JSAny> SloppyArgumentsHas_0(compiler::CodeAssemblerState* state_, TNode<JSObject> p_receiver, TNode<Object> p_keyObject, compiler::CodeAssemblerLabel* label_Bailout);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=345&c=1
TNode<JSAny> SloppyArgumentsStore_0(compiler::CodeAssemblerState* state_, TNode<JSObject> p_receiver, TNode<Object> p_keyObject, TNode<JSAny> p_value, compiler::CodeAssemblerLabel* label_Bailout);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=18&c=3
TNode<JSAny> LoadJSSloppyArgumentsObjectLength_0(compiler::CodeAssemblerState* state_, TNode<JSArgumentsObject> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=18&c=3
void StoreJSSloppyArgumentsObjectLength_0(compiler::CodeAssemblerState* state_, TNode<JSArgumentsObject> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=19&c=3
TNode<JSAny> LoadJSSloppyArgumentsObjectCallee_0(compiler::CodeAssemblerState* state_, TNode<JSArgumentsObject> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=19&c=3
void StoreJSSloppyArgumentsObjectCallee_0(compiler::CodeAssemblerState* state_, TNode<JSArgumentsObject> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=24&c=3
TNode<JSAny> LoadJSStrictArgumentsObjectLength_0(compiler::CodeAssemblerState* state_, TNode<JSArgumentsObject> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=24&c=3
void StoreJSStrictArgumentsObjectLength_0(compiler::CodeAssemblerState* state_, TNode<JSArgumentsObject> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=29&c=3
TNode<Context> LoadSloppyArgumentsElementsContext_0(compiler::CodeAssemblerState* state_, TNode<SloppyArgumentsElements> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=29&c=3
void StoreSloppyArgumentsElementsContext_0(compiler::CodeAssemblerState* state_, TNode<SloppyArgumentsElements> p_o, TNode<Context> p_v);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=30&c=3
TNode<FixedArray> LoadSloppyArgumentsElementsArguments_0(compiler::CodeAssemblerState* state_, TNode<SloppyArgumentsElements> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=30&c=3
void StoreSloppyArgumentsElementsArguments_0(compiler::CodeAssemblerState* state_, TNode<SloppyArgumentsElements> p_o, TNode<FixedArray> p_v);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=31&c=3
TorqueStructSlice_Smi_OR_TheHole_MutableReference_Smi_OR_TheHole_0 FieldSliceSloppyArgumentsElementsMappedEntries_0(compiler::CodeAssemblerState* state_, TNode<SloppyArgumentsElements> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=31&c=3
TNode<Union<Smi, TheHole>> LoadSloppyArgumentsElementsMappedEntries_0(compiler::CodeAssemblerState* state_, TNode<SloppyArgumentsElements> p_o, TNode<IntPtrT> p_i);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=31&c=3
void StoreSloppyArgumentsElementsMappedEntries_0(compiler::CodeAssemblerState* state_, TNode<SloppyArgumentsElements> p_o, TNode<IntPtrT> p_i, TNode<Union<Smi, TheHole>> p_v);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=43&c=3
TNode<Smi> LoadAliasedArgumentsEntryAliasedContextSlot_0(compiler::CodeAssemblerState* state_, TNode<AliasedArgumentsEntry> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=43&c=3
void StoreAliasedArgumentsEntryAliasedContextSlot_0(compiler::CodeAssemblerState* state_, TNode<AliasedArgumentsEntry> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1
TNode<JSArgumentsObject> DownCastForTorqueClass_JSArgumentsObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=13&c=10
TNode<BoolT> Is_JSArgumentsObjectWithLength_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=27&c=1
TNode<SloppyArgumentsElements> DownCastForTorqueClass_SloppyArgumentsElements_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=41&c=1
TNode<AliasedArgumentsEntry> DownCastForTorqueClass_AliasedArgumentsEntry_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=152&c=20
TNode<FixedArray> NewFixedArray_ArgumentsIterator_0(compiler::CodeAssemblerState* state_, TNode<IntPtrT> p_length, TorqueStructArgumentsIterator_0 p_it);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=202&c=10
TNode<FixedArray> NewFixedArray_ParameterValueIterator_0(compiler::CodeAssemblerState* state_, TNode<IntPtrT> p_length, TorqueStructParameterValueIterator_0 p_it);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=219&c=20
TNode<SloppyArgumentsElements> NewSloppyArgumentsElements_ParameterMapIterator_0(compiler::CodeAssemblerState* state_, TNode<Smi> p_length, TNode<Context> p_context, TNode<FixedArray> p_arguments, TorqueStructParameterMapIterator_0 p_it);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=307&c=9
TNode<BoolT> OutOfBounds_Smi_Smi_0(compiler::CodeAssemblerState* state_, TNode<Smi> p_index, TNode<Smi> p_length);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=334&c=10
TNode<JSAny> UnsafeCast_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=31&c=3
TorqueStructSlice_Smi_OR_TheHole_MutableReference_Smi_OR_TheHole_0 NewMutableSlice_Smi_OR_TheHole_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, TaggedIndex>> p_object, TNode<IntPtrT> p_offset, TNode<IntPtrT> p_length);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=37&c=10
void InitializeFieldsFromIterator_Smi_OR_TheHole_ParameterMapIterator_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_Smi_OR_TheHole_MutableReference_Smi_OR_TheHole_0 p_target, TorqueStructParameterMapIterator_0 p_originIterator);

// https://crsrc.org/c/v8/src/objects/arguments.tq?l=37&c=10
void InitializeFieldsFromIterator_Smi_OR_TheHole_ConstantIterator_TheHole_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_Smi_OR_TheHole_MutableReference_Smi_OR_TheHole_0 p_target, TorqueStructConstantIterator_TheHole_0 p_originIterator);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_ARGUMENTS_TQ_CSA_H_
