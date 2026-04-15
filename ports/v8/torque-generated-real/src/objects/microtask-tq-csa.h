#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_MICROTASK_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_MICROTASK_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=5&c=1
TNode<Microtask> Cast_Microtask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=12&c=1
TNode<CallbackTask> Cast_CallbackTask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=18&c=1
TNode<CallableTask> Cast_CallableTask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=27&c=1
TNode<AsyncResumeTask> Cast_AsyncResumeTask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=14&c=3
TNode<Foreign> LoadCallbackTaskCallback_0(compiler::CodeAssemblerState* state_, TNode<CallbackTask> p_o);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=14&c=3
void StoreCallbackTaskCallback_0(compiler::CodeAssemblerState* state_, TNode<CallbackTask> p_o, TNode<Foreign> p_v);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=15&c=3
TNode<Foreign> LoadCallbackTaskData_0(compiler::CodeAssemblerState* state_, TNode<CallbackTask> p_o);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=15&c=3
void StoreCallbackTaskData_0(compiler::CodeAssemblerState* state_, TNode<CallbackTask> p_o, TNode<Foreign> p_v);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=20&c=3
TNode<JSReceiver> LoadCallableTaskCallable_0(compiler::CodeAssemblerState* state_, TNode<CallableTask> p_o);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=20&c=3
void StoreCallableTaskCallable_0(compiler::CodeAssemblerState* state_, TNode<CallableTask> p_o, TNode<JSReceiver> p_v);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=21&c=3
TNode<NativeContext> LoadCallableTaskContext_0(compiler::CodeAssemblerState* state_, TNode<CallableTask> p_o);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=21&c=3
void StoreCallableTaskContext_0(compiler::CodeAssemblerState* state_, TNode<CallableTask> p_o, TNode<NativeContext> p_v);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=29&c=3
TNode<JSGeneratorObject> LoadAsyncResumeTaskGenerator_0(compiler::CodeAssemblerState* state_, TNode<AsyncResumeTask> p_o);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=29&c=3
void StoreAsyncResumeTaskGenerator_0(compiler::CodeAssemblerState* state_, TNode<AsyncResumeTask> p_o, TNode<JSGeneratorObject> p_v);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=30&c=3
TNode<Object> LoadAsyncResumeTaskValue_0(compiler::CodeAssemblerState* state_, TNode<AsyncResumeTask> p_o);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=30&c=3
void StoreAsyncResumeTaskValue_0(compiler::CodeAssemblerState* state_, TNode<AsyncResumeTask> p_o, TNode<Object> p_v);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=31&c=3
TNode<Smi> LoadAsyncResumeTaskKind_0(compiler::CodeAssemblerState* state_, TNode<AsyncResumeTask> p_o);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=31&c=3
void StoreAsyncResumeTaskKind_0(compiler::CodeAssemblerState* state_, TNode<AsyncResumeTask> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=5&c=1
TNode<Microtask> DownCastForTorqueClass_Microtask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=12&c=1
TNode<CallbackTask> DownCastForTorqueClass_CallbackTask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=18&c=1
TNode<CallableTask> DownCastForTorqueClass_CallableTask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/microtask.tq?l=27&c=1
TNode<AsyncResumeTask> DownCastForTorqueClass_AsyncResumeTask_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_MICROTASK_TQ_CSA_H_
