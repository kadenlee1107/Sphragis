#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_CAST_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_CAST_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=12&c=1
TNode<BoolT> IsAccessorInfo_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=17&c=1
TNode<BoolT> IsAccessorPair_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=22&c=1
TNode<BoolT> IsAllocationSite_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=27&c=1
TNode<BoolT> IsCell_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=32&c=1
TNode<BoolT> IsInstructionStream_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=37&c=1
TNode<BoolT> IsCode_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=42&c=1
TNode<BoolT> IsContext_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=47&c=1
TNode<BoolT> IsCoverageInfo_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=52&c=1
TNode<BoolT> IsDebugInfo_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=57&c=1
TNode<BoolT> IsFixedDoubleArray_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=62&c=1
TNode<BoolT> IsFeedbackCell_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=67&c=1
TNode<BoolT> IsFeedbackVector_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=72&c=1
TNode<BoolT> IsHeapNumber_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=77&c=1
TNode<BoolT> IsJSModuleNamespace_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=82&c=1
TNode<BoolT> IsNativeContext_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=87&c=1
TNode<BoolT> IsNumber_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=92&c=1
TNode<BoolT> IsPrivateSymbol_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=97&c=1
TNode<BoolT> IsPromiseCapability_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=102&c=1
TNode<BoolT> IsPromiseFulfillReactionJobTask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=107&c=1
TNode<BoolT> IsPromiseReaction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=112&c=1
TNode<BoolT> IsPromiseRejectReactionJobTask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=117&c=1
TNode<BoolT> IsSharedFunctionInfo_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=122&c=1
TNode<BoolT> IsSymbol_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=158&c=1
TNode<Undefined> Cast_Undefined_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, Smi, Weak<HeapObject>>> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=170&c=1
TNode<Smi> Cast_Smi_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=175&c=1
TNode<Smi> Cast_PositiveSmi_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=180&c=1
TNode<Smi> Cast_Zero_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=185&c=1
TNode<Number> Cast_Number_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=200&c=1
TNode<Undefined> Cast_Undefined_1(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=206&c=1
TNode<Numeric> Cast_Numeric_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=220&c=1
TNode<TheHole> Cast_TheHole_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=225&c=1
TNode<TheHole> Cast_TheHole_1(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=230&c=1
TNode<True> Cast_True_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=235&c=1
TNode<True> Cast_True_1(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=240&c=1
TNode<False> Cast_False_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=245&c=1
TNode<False> Cast_False_1(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=250&c=1
TNode<Boolean> Cast_Boolean_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=264&c=1
TNode<Boolean> Cast_Boolean_1(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=272&c=1
TNode<JSPrimitive> Cast_JSPrimitive_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=298&c=1
TNode<JSAny> Cast_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=312&c=1
TNode<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, String, Symbol, TheHole, Undefined>> Cast_BigInt_OR_JSReceiver_OR_Undefined_OR_Smi_OR_HeapNumber_OR_String_OR_Symbol_OR_Boolean_OR_Null_OR_TheHole_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=326&c=1
TNode<Union<HeapNumber, Smi, TheHole>> Cast_Smi_OR_HeapNumber_OR_TheHole_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=359&c=1
TNode<Union<Context, Smi, Undefined>> Cast_Undefined_OR_Context_OR_Zero_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=380&c=1
TNode<HeapObject> Cast_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label__CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=385&c=1
TNode<Null> Cast_Null_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=391&c=1
TNode<Undefined> Cast_Undefined_2(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=397&c=1
TNode<FixedArray> Cast_EmptyFixedArray_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=402&c=1
TNode<FixedArray> Cast_EmptyFixedArray_1(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=408&c=1
TNode<Union<FixedArray, FixedDoubleArray>> Cast_EmptyFixedArray_OR_FixedDoubleArray_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=423&c=1
TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> Cast_Callable_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=428&c=1
TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> Cast_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=434&c=1
TNode<Union<JSFunction, Undefined>> Cast_JSFunction_OR_Undefined_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=441&c=1
TNode<Symbol> Cast_PublicSymbol_0(compiler::CodeAssemblerState* state_, TNode<Symbol> p_s, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=447&c=1
TNode<Symbol> Cast_PrivateSymbol_0(compiler::CodeAssemblerState* state_, TNode<Symbol> p_s, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=453&c=1
TNode<Symbol> Cast_PublicSymbol_1(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=457&c=1
TNode<Symbol> Cast_PrivateSymbol_1(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=462&c=1
TNode<String> Cast_DirectString_0(compiler::CodeAssemblerState* state_, TNode<String> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=467&c=1
TNode<JSReceiver> Cast_Constructor_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=472&c=1
TNode<BigInt> Cast_BigInt_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=477&c=1
TNode<JSArray> Cast_JSRegExpResult_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=483&c=1
TNode<JSArgumentsObject> Cast_JSSloppyArgumentsObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=494&c=1
TNode<JSArgumentsObject> Cast_JSStrictArgumentsObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=502&c=1
TNode<Union<JSArgumentsObject>> Cast_JSArgumentsObjectWithLength_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=518&c=1
TNode<JSRegExp> Cast_FastJSRegExp_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=535&c=1
TorqueStructCastResult_FastJSArray_0 CastFastJSArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=556&c=1
TNode<JSArray> Cast_FastJSArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=562&c=1
TorqueStructCastResult_FastJSArrayForRead_0 CastFastJSArrayForRead_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=583&c=1
TNode<JSArray> Cast_FastJSArrayForRead_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=590&c=1
TNode<JSArray> Cast_FastJSArrayForCopy_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=599&c=1
TNode<JSArray> Cast_FastJSArrayForConcat_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=607&c=1
TNode<JSArray> Cast_FastJSArrayWithNoCustomIteration_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=615&c=1
TNode<JSArray> Cast_FastJSArrayForReadWithNoCustomIteration_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=625&c=1
TNode<JSArray> Cast_JSArrayForFastToString_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=678&c=1
TNode<JSSet> Cast_JSSetWithNoCustomIteration_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=686&c=1
TNode<JSMap> Cast_JSMapWithNoCustomIteration_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=694&c=1
TNode<OrderedHashSet> Cast_StableOrderedHashSet_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=700&c=1
TNode<OrderedHashMap> Cast_StableOrderedHashMap_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=708&c=1
TNode<SeqOneByteString> Cast_SeqOneByteString_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=713&c=1
TNode<SeqOneByteString> Cast_SeqOneByteString_1(compiler::CodeAssemblerState* state_, TNode<String> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=723&c=1
TNode<SeqTwoByteString> Cast_SeqTwoByteString_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=728&c=1
TNode<SeqTwoByteString> Cast_SeqTwoByteString_1(compiler::CodeAssemblerState* state_, TNode<String> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=738&c=1
TNode<ThinString> Cast_ThinString_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=743&c=1
TNode<ThinString> Cast_ThinString_1(compiler::CodeAssemblerState* state_, TNode<String> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=751&c=1
TNode<ConsString> Cast_ConsString_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=756&c=1
TNode<ConsString> Cast_ConsString_1(compiler::CodeAssemblerState* state_, TNode<String> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=764&c=1
TNode<SlicedString> Cast_SlicedString_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=769&c=1
TNode<SlicedString> Cast_SlicedString_1(compiler::CodeAssemblerState* state_, TNode<String> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=778&c=1
TNode<ExternalOneByteString> Cast_ExternalOneByteString_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=784&c=1
TNode<ExternalOneByteString> Cast_ExternalOneByteString_1(compiler::CodeAssemblerState* state_, TNode<String> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=795&c=1
TNode<ExternalTwoByteString> Cast_ExternalTwoByteString_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=801&c=1
TNode<ExternalTwoByteString> Cast_ExternalTwoByteString_1(compiler::CodeAssemblerState* state_, TNode<String> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=812&c=1
TNode<Union<JSReceiver, Null>> Cast_JSReceiver_OR_Null_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=827&c=1
TNode<Union<JSReceiver, Symbol>> Cast_JSReceiver_OR_Symbol_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=843&c=1
TNode<Union<JSPromise, Undefined>> Cast_Undefined_OR_JSPromise_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=857&c=1
TNode<Union<PromiseReaction, Smi>> Cast_Smi_OR_PromiseReaction_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=871&c=1
TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, String>> Cast_JSFunction_OR_String_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=886&c=1
TNode<Union<PromiseReaction, Smi>> Cast_PromiseReaction_OR_Zero_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=901&c=1
TNode<Union<JSBoundFunction, JSFunction, JSWrappedFunction>> Cast_JSFunction_OR_JSBoundFunction_OR_JSWrappedFunction_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=920&c=1
TNode<Union<FixedArray, Undefined>> Cast_Undefined_OR_FixedArray_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=935&c=1
TNode<Union<JSProxy, Null>> Cast_Null_OR_JSProxy_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=988&c=1
TNode<Object> Cast_Object_0(compiler::CodeAssemblerState* state_, TNode<Object> p_o, compiler::CodeAssemblerLabel* label__CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSAny_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=969&c=10
TNode<BoolT> Is_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<HeapObject, Smi, Weak<HeapObject>>> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=957&c=3
TNode<Object> Cast_Object_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<HeapObject, Smi, Weak<HeapObject>>> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=149&c=5
TNode<Weak<HeapObject>> Cast_WeakHeapObject_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, Smi, Weak<HeapObject>>> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<ContextCell> Cast_ContextCell_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Context_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<Context> Cast_Context_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<HeapNumber> Cast_HeapNumber_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_SeqTwoByteString_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<SeqTwoByteString> Cast_SeqTwoByteString_2(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_FeedbackVector_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<FeedbackVector> Cast_FeedbackVector_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=14&c=10
TNode<BoolT> Is_AccessorInfo_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=19&c=10
TNode<BoolT> Is_AccessorPair_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=24&c=10
TNode<BoolT> Is_AllocationSite_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=29&c=10
TNode<BoolT> Is_Cell_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=34&c=10
TNode<BoolT> Is_InstructionStream_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=39&c=10
TNode<BoolT> Is_Code_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=44&c=10
TNode<BoolT> Is_Context_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=49&c=10
TNode<BoolT> Is_CoverageInfo_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=54&c=10
TNode<BoolT> Is_DebugInfo_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=59&c=10
TNode<BoolT> Is_FixedDoubleArray_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=64&c=10
TNode<BoolT> Is_FeedbackCell_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=69&c=10
TNode<BoolT> Is_FeedbackVector_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=74&c=10
TNode<BoolT> Is_HeapNumber_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=79&c=10
TNode<BoolT> Is_JSModuleNamespace_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<JSModuleNamespace> Cast_JSModuleNamespace_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=84&c=10
TNode<BoolT> Is_NativeContext_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=89&c=10
TNode<BoolT> Is_Number_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=94&c=10
TNode<BoolT> Is_PrivateSymbol_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=99&c=10
TNode<BoolT> Is_PromiseCapability_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=104&c=10
TNode<BoolT> Is_PromiseFulfillReactionJobTask_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=109&c=10
TNode<BoolT> Is_PromiseReaction_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=114&c=10
TNode<BoolT> Is_PromiseRejectReactionJobTask_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=119&c=10
TNode<BoolT> Is_SharedFunctionInfo_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=124&c=10
TNode<BoolT> Is_Symbol_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=539&c=8
TNode<BoolT> Is_JSArray_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=668&c=25
TorqueStructReference_Cell_0 NativeContextSlot_Cell_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<IntPtrT> p_index);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=834&c=5
TNode<Symbol> Cast_Symbol_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=892&c=5
TNode<PromiseReaction> Cast_PromiseReaction_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=908&c=5
TNode<JSBoundFunction> Cast_JSBoundFunction_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=911&c=5
TNode<JSWrappedFunction> Cast_JSWrappedFunction_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Smi_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_SeqOneByteString_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<SeqOneByteString> Cast_SeqOneByteString_2(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=153&c=14
TNode<PrototypeSharedClosureInfo> Cast_PrototypeSharedClosureInfo_2(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<Union<JSArgumentsObject>> Cast_JSArgumentsObjectWithLength_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_FixedArray_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_BigInt_OR_JSReceiver_OR_Undefined_OR_Smi_OR_HeapNumber_OR_String_OR_Symbol_OR_Boolean_OR_Null_OR_TheHole_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSReceiver_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Symbol_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_HeapObject_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_FixedDoubleArray_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<FixedDoubleArray> Cast_FixedDoubleArray_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_NativeContext_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Boolean_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSPrimitiveWrapper_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_ArrayList_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Constructor_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Map_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_String_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_FastJSRegExp_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSBoundFunction_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<AllocationSite> Cast_AllocationSite_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<FunctionTemplateInfo> Cast_FunctionTemplateInfo_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSTypedArray_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSArray_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Numeric_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSDataView_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSRabGsabDataView_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSRegExp_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSRegExpResult_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=153&c=14
TNode<AccessorInfo> Cast_AccessorInfo_2(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_DescriptorArray_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSReceiver_OR_Symbol_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSWeakRef_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_NumberDictionary_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Smi_OR_HeapNumber_OR_TheHole_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_PromiseFulfillReactionJobTask_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_PromiseRejectReactionJobTask_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_PromiseReaction_OR_Zero_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSFinalizationRegistry_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_PositiveSmi_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_ContextOrFrameType_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSObject_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<ExportedSubClassBase> Cast_ExportedSubClassBase_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<ExportedSubClass> Cast_ExportedSubClass_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<ExportedSubClass2> Cast_ExportedSubClass2_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<Map> Cast_Map_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<NativeContext> Cast_NativeContext_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<JSPrimitiveWrapper> Cast_JSPrimitiveWrapper_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<ArrayList> Cast_ArrayList_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_PromiseCapability_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<PromiseCapability> Cast_PromiseCapability_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_EmptyFixedArray_OR_FixedDoubleArray_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<DescriptorArray> Cast_DescriptorArray_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Null_OR_JSProxy_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<Union<JSProxy, Null>> Cast_Null_OR_JSProxy_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<NumberDictionary> Cast_NumberDictionary_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<PromiseFulfillReactionJobTask> Cast_PromiseFulfillReactionJobTask_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<PromiseRejectReactionJobTask> Cast_PromiseRejectReactionJobTask_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<JSObject> Cast_JSObject_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_ScopeInfo_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<ScopeInfo> Cast_ScopeInfo_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSFunction_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSArrayBuffer_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<Union<FixedArray, FixedDoubleArray>> Cast_EmptyFixedArray_OR_FixedDoubleArray_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Cell_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<Cell> Cast_Cell_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSFunction_OR_Undefined_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<Union<JSFunction, Undefined>> Cast_JSFunction_OR_Undefined_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_RegExpMatchInfo_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<RegExpMatchInfo> Cast_RegExpMatchInfo_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=963&c=10
TNode<BoolT> Is_Undefined_OR_FixedArray_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/cast.tq?l=951&c=3
TNode<Union<FixedArray, Undefined>> Cast_Undefined_OR_FixedArray_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_CAST_TQ_CSA_H_
