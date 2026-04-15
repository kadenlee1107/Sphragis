#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_FUNCTION_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_FUNCTION_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/function.tq?l=30&c=1
void CheckAccessor_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<DescriptorArray> p_array, int32_t p_index, TNode<Name> p_name, compiler::CodeAssemblerLabel* label_Slow);

// https://crsrc.org/c/v8/src/builtins/function.tq?l=38&c=3
TNode<AccessorInfo> Cast_AccessorInfo_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<HeapObject, Smi, Weak<HeapObject>>> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/function.tq?l=66&c=13
TNode<DescriptorArray> UnsafeCast_DescriptorArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_FUNCTION_TQ_CSA_H_
