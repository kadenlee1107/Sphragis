#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_ABSTRACT_OPERATIONS_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_ABSTRACT_OPERATIONS_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=29&c=1
TNode<Symbol> kPromiseForwardingHandlerSymbol_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=32&c=1
TNode<Symbol> kPromiseHandledBySymbol_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=34&c=1
TNode<String> kResolveString_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=47&c=1
TNode<Context> ExtractHandlerContextInternal_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_handler, compiler::CodeAssemblerLabel* label_NotFound);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=70&c=1
TNode<Context> ExtractHandlerContext_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_handler);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=79&c=1
TNode<Context> ExtractHandlerContext_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_primary, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_secondary);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=91&c=1
void MorphAndEnqueuePromiseReaction_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<PromiseReaction> p_promiseReaction, TNode<JSAny> p_argument, PromiseReaction::Type p_reactionType);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=156&c=1
void TriggerPromiseReactions_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<PromiseReaction, Smi>> p_reactions, TNode<JSAny> p_argument, PromiseReaction::Type p_reactionType);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=285&c=1
TNode<IntPtrT> FromConstexpr_PromiseResolvingFunctionContextSlot_constexpr_kPromiseSlot_0(compiler::CodeAssemblerState* state_, PromiseBuiltins::PromiseResolvingFunctionContextSlot p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=285&c=1
TNode<IntPtrT> FromConstexpr_PromiseResolvingFunctionContextSlot_constexpr_kAlreadyResolvedSlot_0(compiler::CodeAssemblerState* state_, PromiseBuiltins::PromiseResolvingFunctionContextSlot p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=285&c=1
TNode<IntPtrT> FromConstexpr_PromiseResolvingFunctionContextSlot_constexpr_kDebugEventSlot_0(compiler::CodeAssemblerState* state_, PromiseBuiltins::PromiseResolvingFunctionContextSlot p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=285&c=1
TNode<IntPtrT> FromConstexpr_PromiseResolvingFunctionContextSlot_constexpr_kPromiseContextLength_0(compiler::CodeAssemblerState* state_, PromiseBuiltins::PromiseResolvingFunctionContextSlot p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=285&c=1
TNode<IntPtrT> kPromiseSlot_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=285&c=1
TNode<IntPtrT> kAlreadyResolvedSlot_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=285&c=1
TNode<IntPtrT> kDebugEventSlot_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=294&c=1
TNode<IntPtrT> FromConstexpr_FunctionContextSlot_constexpr_kCapabilitySlot_0(compiler::CodeAssemblerState* state_, PromiseBuiltins::FunctionContextSlot p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=294&c=1
TNode<IntPtrT> FromConstexpr_FunctionContextSlot_constexpr_kCapabilitiesContextLength_0(compiler::CodeAssemblerState* state_, PromiseBuiltins::FunctionContextSlot p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=294&c=1
TNode<IntPtrT> kCapabilitySlot_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=300&c=1
TNode<Context> CreatePromiseCapabilitiesExecutorContext_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_nativeContext, TNode<PromiseCapability> p_capability);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=313&c=1
TNode<PromiseCapability> CreatePromiseCapability_0(compiler::CodeAssemblerState* state_, TNode<Union<JSReceiver, Undefined>> p_promise, TNode<Union<JSFunction, Undefined>> p_resolve, TNode<Union<JSFunction, Undefined>> p_reject);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=337&c=1
TorqueStructPromiseResolvingFunctions CreatePromiseResolvingFunctions_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise, TNode<Boolean> p_debugEvent, TNode<NativeContext> p_nativeContext);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=358&c=1
TNode<PromiseCapability> InnerNewPromiseCapability_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<HeapObject> p_constructor, TNode<Boolean> p_debugEvent);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=463&c=1
void PerformPromiseThenImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_onFulfilled, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_onRejected, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_resultPromiseOrCapability);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=576&c=1
TNode<BoolT> IsPromiseResolveLookupChainIntact_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSReceiver> p_constructor);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=586&c=1
TNode<JSAny> GetPromiseResolve_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSReceiver> p_constructor);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=613&c=1
TNode<JSAny> CallResolve_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_constructor, TNode<JSAny> p_resolve, TNode<JSAny> p_value);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=121&c=9
TNode<PromiseFulfillReactionJobTask> UnsafeCast_PromiseFulfillReactionJobTask_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=140&c=9
TNode<PromiseRejectReactionJobTask> UnsafeCast_PromiseRejectReactionJobTask_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=220&c=7
TNode<Union<PromiseReaction, Smi>> UnsafeCast_PromiseReaction_OR_Zero_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=308&c=3
void InitContextSlot_PromiseCapabilitiesExecutorContext_PromiseCapabilitiesExecutorContext_PromiseCapability_PromiseCapability_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_index, TNode<PromiseCapability> p_value);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=414&c=8
TorqueStructReference_JSPromise_0 ContextSlot_PromiseResolvingFunctionContext_PromiseResolvingFunctionContext_JSPromise_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_index);

// https://crsrc.org/c/v8/src/builtins/promise-abstract-operations.tq?l=565&c=8
TorqueStructReference_PromiseCapability_0 ContextSlot_PromiseCapabilitiesExecutorContext_PromiseCapabilitiesExecutorContext_PromiseCapability_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_index);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROMISE_ABSTRACT_OPERATIONS_TQ_CSA_H_
