#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_OBJECT_GROUPBY_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_OBJECT_GROUPBY_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/object-groupby.tq?l=16&c=1
TNode<JSAny> CoerceGroupKey_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_key, TNode<Boolean> p_coerceToProperty);

// https://crsrc.org/c/v8/src/builtins/object-groupby.tq?l=95&c=1
TNode<OrderedHashMap> GroupByImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_items, TNode<JSAny> p_callback, TNode<Boolean> p_coerceToProperty, const char* p_methodName);

// https://crsrc.org/c/v8/src/builtins/object-groupby.tq?l=110&c=7
TNode<JSArray> Cast_FastJSArrayForReadWithNoCustomIteration_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/object-groupby.tq?l=198&c=42
TNode<ArrayList> UnsafeCast_ArrayList_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_OBJECT_GROUPBY_TQ_CSA_H_
