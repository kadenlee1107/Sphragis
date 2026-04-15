#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_RESOLVE_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_RESOLVE_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/promise-resolve.tq?l=13&c=1
TNode<String> kConstructorString_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-resolve.tq?l=86&c=1
TNode<String> kThenString_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-resolve.tq?l=30&c=23
TorqueStructReference_JSFunction_0 NativeContextSlot_JSFunction_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<IntPtrT> p_index);

// https://crsrc.org/c/v8/src/builtins/promise-resolve.tq?l=34&c=19
TNode<JSPromise> Cast_JSPromise_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/promise-resolve.tq?l=41&c=10
TorqueStructReference_JSObject_0 NativeContextSlot_JSObject_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<IntPtrT> p_index);

// https://crsrc.org/c/v8/src/builtins/promise-resolve.tq?l=113&c=28
TNode<HeapObject> UnsafeCast_HeapObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/promise-resolve.tq?l=141&c=12
TorqueStructReference_Map_0 NativeContextSlot_Map_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<IntPtrT> p_index);

// https://crsrc.org/c/v8/src/builtins/promise-resolve.tq?l=172&c=10
TNode<BoolT> Is_Callable_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_RESOLVE_TQ_CSA_H_
