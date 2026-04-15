#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAY_FLAT_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAY_FLAT_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=7&c=1
TNode<Boolean> ArrayIsArray_Inline_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_element);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=37&c=1
TorqueStructFlatVector_0 NewFlatVector_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Smi> p_length);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=52&c=1
TNode<IntPtrT> kMaxFlatFastStackEntries_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=54&c=1
TorqueStructFlattenedLengthResult_0 CalculateFlattenedLengthFast_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSArray> p_source, TNode<Smi> p_sourceLength, TNode<Smi> p_depth, compiler::CodeAssemblerLabel* label_Bailout);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=184&c=1
TNode<JSArray> TryFastFlat_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_receiver, TNode<Number> p_sourceLength, TNode<Smi> p_depth, compiler::CodeAssemblerLabel* label_Bailout);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=350&c=1
TNode<Number> FlattenIntoArrayFast_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_target, TNode<JSReceiver> p_source, TNode<Number> p_sourceLength, TNode<Number> p_start, TNode<Smi> p_depth, bool p_hasMapper, TNode<JSAny> p_mapfn, TNode<JSAny> p_thisArgs, compiler::CodeAssemblerLabel* label_Bailout, compiler::TypedCodeAssemblerVariable<Number>* label_Bailout_parameter_0, compiler::TypedCodeAssemblerVariable<Number>* label_Bailout_parameter_1);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=436&c=1
TNode<Number> FlattenIntoArraySlow_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_target, TNode<JSReceiver> p_source, TNode<Number> p_sourceIndex, TNode<Number> p_sourceLength, TNode<Number> p_start, TNode<Smi> p_depth, bool p_hasMapper, TNode<JSAny> p_mapfn, TNode<JSAny> p_thisArgs);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=498&c=1
TNode<Number> FlattenIntoArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_target, TNode<JSReceiver> p_source, TNode<Number> p_sourceLength, TNode<Number> p_start, TNode<Smi> p_depth, bool p_hasMapper, TNode<JSAny> p_mapfn, TNode<JSAny> p_thisArgs);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=9&c=7
TNode<BoolT> Is_JSArray_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_o);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=11&c=14
TNode<BoolT> Is_JSProxy_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_o);

// https://crsrc.org/c/v8/src/builtins/array-flat.tq?l=365&c=10
TNode<BoolT> Is_Smi_Number_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Number> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAY_FLAT_TQ_CSA_H_
