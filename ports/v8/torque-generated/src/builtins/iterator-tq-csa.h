#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ITERATOR_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ITERATOR_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=18&c=1
int31_t kIteratorRecordFieldCount_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=47&c=1
TNode<JSAny> IteratorStepValue_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_iterated, TNode<Map> p_fastIteratorResultMap, compiler::CodeAssemblerLabel* label_Done, compiler::CodeAssemblerLabel* label_DoneWithException, compiler::TypedCodeAssemblerVariable<JSAny>* label_DoneWithException_parameter_0, compiler::TypedCodeAssemblerVariable<Union<JSMessageObject, TheHole>>* label_DoneWithException_parameter_1);

// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=168&c=1
TorqueStructIteratorRecord GetIteratorRecordAfterCreateAsyncFromSyncIterator_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_asyncIterator);

// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=180&c=1
TNode<JSAny> GetLazyReceiver_0(compiler::CodeAssemblerState* state_, TNode<JSAny> p_receiver);

// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=202&c=1
void IteratorCloseOnException_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_iteratorObject);

// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=224&c=1
void IteratorClose_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_iterator);

// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=269&c=1
TNode<JSAny> SetterThatIgnoresPrototypeProperties_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_receiver, TNode<JSObject> p_home, TNode<JSAny> p_key, TNode<JSAny> p_value, const char* p_methodName);

// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=378&c=1
intptr_t kAsyncIteratorPrototypeAsyncDisposeResolveContextLength_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=53&c=7
int31_t SizeOf_IteratorRecord_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/iterator.tq?l=160&c=20
TNode<IntPtrT> TimesSizeOf_intptr_0(compiler::CodeAssemblerState* state_, TNode<IntPtrT> p_i);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ITERATOR_TQ_CSA_H_
