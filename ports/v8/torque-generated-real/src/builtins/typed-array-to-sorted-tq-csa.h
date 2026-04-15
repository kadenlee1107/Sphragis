#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_TO_SORTED_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_TO_SORTED_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/typed-array-to-sorted.tq?l=8&c=1
const char* kBuiltinNameToSorted_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/typed-array-to-sorted.tq?l=16&c=21
TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> Cast_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/typed-array-to-sorted.tq?l=27&c=31
TNode<JSTypedArray> UnsafeCast_JSTypedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_TO_SORTED_TQ_CSA_H_
