#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_MISC_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_MISC_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=42&c=1
TNode<BoolT> PromiseHasHandler_0(compiler::CodeAssemblerState* state_, TNode<JSPromise> p_promise);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=47&c=1
void PromiseInit_0(compiler::CodeAssemblerState* state_, TNode<JSPromise> p_promise);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=59&c=1
TNode<JSPromise> InnerNewJSPromise_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=78&c=1
TNode<PromiseFulfillReactionJobTask> NewPromiseFulfillReactionJobTask_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Context> p_handlerContext, TNode<Object> p_argument, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_handler, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=106&c=1
TNode<PromiseRejectReactionJobTask> NewPromiseRejectReactionJobTask_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Context> p_handlerContext, TNode<Object> p_argument, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_handler, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=134&c=1
void RunContextPromiseHookInit_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise, TNode<Object> p_parent);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=150&c=1
void RunContextPromiseHookResolve_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=162&c=1
void RunContextPromiseHookResolve_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise, TNode<Uint32T> p_flags);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=169&c=1
void RunContextPromiseHookBefore_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=182&c=1
void RunContextPromiseHookBefore_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability, TNode<Uint32T> p_flags);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=192&c=1
void RunContextPromiseHookAfter_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=205&c=1
void RunContextPromiseHookAfter_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability, TNode<Uint32T> p_flags);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=215&c=1
void RunContextPromiseHook_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_slot, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability, TNode<Uint32T> p_flags);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=249&c=1
void RunAnyPromiseHookInit_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise, TNode<Object> p_parent);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=269&c=1
TNode<JSPromise> NewJSPromise_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_parent);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=280&c=1
TNode<JSPromise> NewJSPromise_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=287&c=1
TNode<JSPromise> NewJSPromise_2(compiler::CodeAssemblerState* state_, TNode<Context> p_context, Promise::PromiseState p_status, TNode<JSAny> p_result);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=301&c=1
TNode<PromiseReaction> NewPromiseReaction_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<PromiseReaction, Smi>> p_next, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_fulfillHandler, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_rejectHandler);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=332&c=1
TNode<PromiseResolveThenableJobTask> NewPromiseResolveThenableJobTask_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promiseToResolve, TNode<JSReceiver> p_thenable, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_then);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=409&c=1
TNode<JSAny> InvokeThen_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_receiver, TNode<JSAny> p_arg);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=416&c=1
TNode<JSAny> InvokeThen_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_receiver, TNode<JSAny> p_arg1, TNode<JSAny> p_arg2);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=423&c=1
void BranchIfAccessCheckFailed_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_promiseConstructor, TNode<JSAny> p_executor, compiler::CodeAssemblerLabel* label_IfNoAccess);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=50&c=19
TNode<Smi> SmiTag_JSPromiseFlags_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_value);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=61&c=10
TNode<BoolT> Is_JSFunctionWithPrototype_JSFunction_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p_o);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=64&c=4
TorqueStructReference_Map_0 UnsafeConstCast_Map_0(compiler::CodeAssemblerState* state_, TorqueStructReference_Map_0 p_r);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=137&c=22
TorqueStructReference_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0 NativeContextSlot_Context_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_index);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=140&c=24
TNode<BoolT> Is_JSPromise_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=412&c=10
TNode<JSAny> InvokeThen_InvokeThenOneArgFunctor_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_receiver, TNode<JSAny> p_arg1, TNode<JSAny> p_arg2, TorqueStructInvokeThenOneArgFunctor_0 p_callFunctor);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=419&c=10
TNode<JSAny> InvokeThen_InvokeThenTwoArgFunctor_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_receiver, TNode<JSAny> p_arg1, TNode<JSAny> p_arg2, TorqueStructInvokeThenTwoArgFunctor_0 p_callFunctor);

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=396&c=8
TNode<BoolT> Is_Smi_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_MISC_TQ_CSA_H_
