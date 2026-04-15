#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=93&c=1
TNode<Smi> kStoreSucceded_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=94&c=1
TNode<Smi> kStoreFailureArrayDetachedOrImmutableOrOutOfBounds_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=145&c=1
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_0(compiler::CodeAssemblerState* state_, TNode<Int32T> p_elementsKindParam);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=201&c=1
TNode<JSTypedArray> EnsureValid_0(compiler::CodeAssemblerState* state_, TNode<JSTypedArray> p_array, TypedArrayAccessMode p_mode, compiler::CodeAssemblerLabel* label_Fail);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=222&c=1
TorqueStructValidJSTypedArrayAndLength_0 EnsureValidAndReadLength_0(compiler::CodeAssemblerState* state_, TNode<JSTypedArray> p_array, TypedArrayAccessMode p_mode, compiler::CodeAssemblerLabel* label_Fail);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=258&c=1
TorqueStructAttachedJSTypedArrayWitness_0 NewAttachedJSTypedArrayWitness_0(compiler::CodeAssemblerState* state_, TNode<JSTypedArray> p_array);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=271&c=1
ElementsKind KindForArrayType_Uint8Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=274&c=1
ElementsKind KindForArrayType_Int8Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=277&c=1
ElementsKind KindForArrayType_Uint16Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=280&c=1
ElementsKind KindForArrayType_Int16Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=283&c=1
ElementsKind KindForArrayType_Uint32Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=286&c=1
ElementsKind KindForArrayType_Int32Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=289&c=1
ElementsKind KindForArrayType_Float16Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=292&c=1
ElementsKind KindForArrayType_Float32Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=295&c=1
ElementsKind KindForArrayType_Float64Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=298&c=1
ElementsKind KindForArrayType_Uint8ClampedElements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=301&c=1
ElementsKind KindForArrayType_BigUint64Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=304&c=1
ElementsKind KindForArrayType_BigInt64Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=156&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Int32Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=158&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Float16Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=160&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Float32Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=162&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Float64Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=164&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Uint8ClampedElements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=166&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_BigUint64Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=168&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_BigInt64Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=172&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Uint8Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=174&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Int8Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=176&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Uint16Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=178&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Int16Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=180&c=14
TorqueStructTypedArrayAccessor_0 GetTypedArrayAccessor_Uint32Elements_0(compiler::CodeAssemblerState* state_);

// https://crsrc.org/c/v8/src/builtins/typed-array.tq?l=318&c=32
TNode<JSArrayBuffer> UnsafeCast_JSArrayBuffer_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_TQ_CSA_H_
