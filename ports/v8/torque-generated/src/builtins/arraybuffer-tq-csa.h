#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAYBUFFER_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAYBUFFER_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/arraybuffer.tq?l=15&c=13
TNode<JSArrayBuffer> Cast_JSArrayBuffer_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/arraybuffer.tq?l=59&c=28
TNode<WasmMemoryObject> Cast_WasmMemoryObject_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/arraybuffer.tq?l=201&c=5
TNode<JSArrayBufferView> Cast_JSArrayBufferView_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_ARRAYBUFFER_TQ_CSA_H_
