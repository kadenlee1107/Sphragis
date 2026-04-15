#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAY_TO_REVERSED_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAY_TO_REVERSED_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/array-to-reversed.tq?l=7&c=1
TNode<JSArray> FastPackedDoubleArrayToReversed_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedDoubleArray> p_elements, TNode<Smi> p_length);

// https://crsrc.org/c/v8/src/builtins/array-to-reversed.tq?l=81&c=1
TNode<JSArray> TryFastArrayToReversed_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_receiver, compiler::CodeAssemblerLabel* label_Slow);

// https://crsrc.org/c/v8/src/builtins/array-to-reversed.tq?l=83&c=30
TNode<JSArray> Cast_FastJSArray_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/array-to-reversed.tq?l=89&c=12
TNode<JSArray> FastArrayToReversed_FixedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, ElementsKind p_kind, TNode<FixedArray> p_elements, TNode<Smi> p_length, bool p_initializeArray);

// https://crsrc.org/c/v8/src/builtins/array-to-reversed.tq?l=109&c=14
TNode<JSArray> FastArrayToReversed_FixedDoubleArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, ElementsKind p_kind, TNode<FixedDoubleArray> p_elements, TNode<Smi> p_length, bool p_initializeArray);

// https://crsrc.org/c/v8/src/builtins/array-to-reversed.tq?l=52&c=12
TNode<BoolT> Is_FixedArray_FixedArrayBase_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArrayBase> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAY_TO_REVERSED_TQ_CSA_H_
