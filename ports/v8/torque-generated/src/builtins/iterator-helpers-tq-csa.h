#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ITERATOR_HELPERS_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ITERATOR_HELPERS_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=9&c=1
TNode<BoolT> IsIteratorHelperExhausted_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=14&c=1
void MarkIteratorHelperAsExhausted_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=19&c=1
TNode<BoolT> IsIteratorHelperExecuting_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=27&c=1
void ThrowIfIteratorHelperExecuting_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSIteratorHelper> p_helper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=34&c=1
void MarkIteratorHelperAsExecuting_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=40&c=1
TNode<BoolT> IteratorHelperIsSuspendedStart_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=45&c=1
void MarkIteratorHelperAsFinishedExecuting_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=52&c=1
TorqueStructIteratorRecord GetIteratorDirect_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_obj);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=207&c=1
TNode<JSIteratorMapHelper> NewJSIteratorMapHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_mapper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=313&c=1
TNode<JSIteratorFilterHelper> NewJSIteratorFilterHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_predicate);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=427&c=1
TNode<JSIteratorTakeHelper> NewJSIteratorTakeHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Number> p_limit);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=548&c=1
TNode<JSIteratorDropHelper> NewJSIteratorDropHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Number> p_limit);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=669&c=1
const char* kFlatMapMethodName_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=671&c=1
TNode<JSIteratorFlatMapHelper> NewJSIteratorFlatMapHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_mapper);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1238&c=1
const char* kConcatMethodName_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1240&c=1
TNode<JSIteratorConcatHelper> NewJSIteratorConcatHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArray> p_iterables);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1456&c=1
void IteratorZipCloseAll_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArray> p_iterators, bool p_propagate);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1491&c=1
TNode<JSIteratorZipHelper> NewJSIteratorZipHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArray> p_iterators, TNode<FixedArray> p_padding, TNode<Uint32T> p_zipMode);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=11&c=7
TNode<Smi> SmiTag_JSIteratorHelperState_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_value);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=72&c=18
TNode<JSIteratorHelper> Cast_JSIteratorHelper_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1502&c=11
TNode<Smi> SmiTag_JSIteratorZipHelperMode_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_value);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1612&c=40
TorqueStructConstantIterator_Undefined_0 ConstantIterator_Undefined_0(compiler::CodeAssemblerState* state_, TNode<Undefined> p_value);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1612&c=15
TNode<FixedArray> NewFixedArray_ConstantIterator_Undefined_0(compiler::CodeAssemblerState* state_, TNode<IntPtrT> p_length, TorqueStructConstantIterator_Undefined_0 p_it);

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1695&c=16
TNode<Uint32T> SmiUntag_JSIteratorZipHelperMode_0(compiler::CodeAssemblerState* state_, TNode<Smi> p_value);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ITERATOR_HELPERS_TQ_CSA_H_
