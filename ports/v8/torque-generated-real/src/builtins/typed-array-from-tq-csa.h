#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_FROM_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_FROM_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/typed-array-from.tq?l=8&c=1
const char* kBuiltinNameFrom_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/typed-array-from.tq?l=20&c=1
TNode<BoolT> CanCopyElementsFastNumber_0(compiler::CodeAssemblerState* state_, TNode<JSArray> p_source);

// https://crsrc.org/c/v8/src/builtins/typed-array-from.tq?l=40&c=25
TNode<JSReceiver> Cast_Constructor_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/typed-array-from.tq?l=47&c=21
TNode<BoolT> Is_Callable_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_o);

// https://crsrc.org/c/v8/src/builtins/typed-array-from.tq?l=101&c=11
TNode<JSArray> Cast_JSArray_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_FROM_TQ_CSA_H_
