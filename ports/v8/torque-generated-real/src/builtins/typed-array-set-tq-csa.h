#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_SET_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_SET_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/typed-array-set.tq?l=8&c=1
const char* kBuiltinNameSet_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/typed-array-set.tq?l=114&c=1
void TypedArrayPrototypeSetArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_receiver, TNode<JSTypedArray> p_target, TNode<UintPtrT> p_targetLength, TNode<JSAny> p_arrayArg, TNode<UintPtrT> p_targetOffset, TNode<BoolT> p_targetOffsetOverflowed, compiler::CodeAssemblerLabel* label_IfOffsetOutOfBounds);

// https://crsrc.org/c/v8/src/builtins/typed-array-set.tq?l=175&c=1
void TypedArrayPrototypeSetTypedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_receiver, TorqueStructValidJSTypedArrayAndLength_0 p_attachedTargetAndLength, TorqueStructValidJSTypedArrayAndLength_0 p_attachedSourceAndLength, TNode<UintPtrT> p_targetOffset, TNode<BoolT> p_targetOffsetOverflowed, compiler::CodeAssemblerLabel* label_IfOffsetOutOfBounds);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_SET_TQ_CSA_H_
