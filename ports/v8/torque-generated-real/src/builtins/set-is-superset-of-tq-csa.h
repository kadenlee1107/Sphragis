#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_SET_IS_SUPERSET_OF_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_SET_IS_SUPERSET_OF_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/set-is-superset-of.tq?l=17&c=13
TNode<JSSet> Cast_JSSet_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/set-is-superset-of.tq?l=36&c=7
TNode<JSSet> Cast_JSSetWithNoCustomIteration_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/set-is-superset-of.tq?l=52&c=7
TNode<JSMap> Cast_JSMapWithNoCustomIteration_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/set-is-superset-of.tq?l=75&c=42
TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> UnsafeCast_Callable_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_SET_IS_SUPERSET_OF_TQ_CSA_H_
