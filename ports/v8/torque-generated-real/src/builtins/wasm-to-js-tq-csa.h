#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_WASM_TO_JS_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_WASM_TO_JS_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=28&c=1
void HandleF32Returns_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TorqueStructLocationAllocator_0 p_locationAllocator, TorqueStructReference_intptr_0 p_toRef, TNode<JSAny> p_retVal);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=50&c=1
TorqueStructWasmToJSResult WasmToJSWrapper_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_data);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=33&c=8
TorqueStructReference_float64_0 RefCast_float64_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_i);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=35&c=8
TorqueStructReference_float32_0 RefCast_float32_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_i);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=53&c=10
TNode<BoolT> Is_WasmImportData_WasmImportData_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<WasmImportData> p_o);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=58&c=4
TorqueStructReference_RawPtr_0 GetRefAt_RawPtr_RawPtr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=60&c=33
int31_t SizeOf_intptr_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=73&c=24
TorqueStructReference_intptr_0 NewOffHeapReference_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_ptr);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=78&c=30
TorqueStructReference_RawPtr_uint32_0 NewOffHeapReference_RawPtr_uint32_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_ptr);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=82&c=7
TorqueStructSlice_uint32_ConstReference_uint32_0 NewOffHeapConstSlice_uint32_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_startPointer, TNode<IntPtrT> p_length);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=85&c=7
TorqueStructSlice_uint32_ConstReference_uint32_0 Subslice_uint32_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_uint32_ConstReference_uint32_0 p_slice, TNode<IntPtrT> p_start, TNode<IntPtrT> p_length, compiler::CodeAssemblerLabel* label_OutOfBounds);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=116&c=37
TorqueStructReference_int64_0 RefCast_int64_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_i);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=118&c=16
TorqueStructReference_int32_0 RefCast_int32_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_i);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=186&c=4
TorqueStructReference_intptr_0 GetRefAt_intptr_RawPtr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=303&c=15
TorqueStructReference_intptr_0 GetRefAt_intptr_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset);

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=305&c=15
TorqueStructReference_float64_0 GetRefAt_float64_RawPtr_float64_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_WASM_TO_JS_TQ_CSA_H_
