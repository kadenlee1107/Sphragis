#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_CREATETYPEDARRAY_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_CREATETYPEDARRAY_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=29&c=1
void AttachView_0(compiler::CodeAssemblerState* state_, TNode<JSArrayBuffer> p_buffer, TNode<JSArrayBufferView> p_view);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=51&c=1
TNode<JSTypedArray> AllocateTypedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, bool p_isOnHeap, TNode<Map> p_map, TNode<JSArrayBuffer> p_buffer, TNode<UintPtrT> p_byteOffset, TNode<UintPtrT> p_byteLength, TNode<BoolT> p_isLengthTracking);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=115&c=1
TNode<JSTypedArray> TypedArrayInitialize_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, bool p_initialize, TNode<Map> p_map, TNode<UintPtrT> p_length, TorqueStructTypedArrayElementsInfo p_elementsInfo, compiler::CodeAssemblerLabel* label_IfRangeError);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=164&c=1
TNode<JSTypedArray> ConstructByLength_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p_target, TNode<JSReceiver> p_newTarget, TNode<JSAny> p_lengthObj);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=185&c=1
TNode<JSTypedArray> ConstructByArrayLike_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Map> p_map, TNode<HeapObject> p_arrayLike, TNode<UintPtrT> p_length, TorqueStructTypedArrayElementsInfo p_elementsInfo);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=239&c=1
void ConstructByIterable_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_iterable, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_iteratorFn, compiler::CodeAssemblerLabel* label_IfConstructByArrayLike, compiler::TypedCodeAssemblerVariable<JSArray>* label_IfConstructByArrayLike_parameter_0, compiler::TypedCodeAssemblerVariable<UintPtrT>* label_IfConstructByArrayLike_parameter_1);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=277&c=1
void ConstructByTypedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSTypedArray> p_srcTypedArray, compiler::CodeAssemblerLabel* label_IfConstructByArrayLike, compiler::TypedCodeAssemblerVariable<JSTypedArray>* label_IfConstructByArrayLike_parameter_0, compiler::TypedCodeAssemblerVariable<UintPtrT>* label_IfConstructByArrayLike_parameter_1);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=294&c=1
TNode<JSTypedArray> ConstructByArrayBuffer_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p_target, TNode<JSReceiver> p_newTarget, TNode<JSArrayBuffer> p_buffer, TNode<JSAny> p_byteOffset, TNode<JSAny> p_length);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=401&c=1
TNode<JSTypedArray> TypedArrayCreateByLength_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_constructor, TNode<Number> p_length, const char* p_methodName, TypedArrayAccessMode p_accessMode);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=442&c=1
void ConstructByJSReceiver_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_obj, compiler::CodeAssemblerLabel* label_IfConstructByArrayLike, compiler::TypedCodeAssemblerVariable<JSReceiver>* label_IfConstructByArrayLike_parameter_0, compiler::TypedCodeAssemblerVariable<UintPtrT>* label_IfConstructByArrayLike_parameter_1, compiler::CodeAssemblerLabel* label_IfIteratorNotCallable, compiler::TypedCodeAssemblerVariable<JSAny>* label_IfIteratorNotCallable_parameter_0);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=512&c=1
TNode<JSTypedArray> TypedArraySpeciesCreateMaybeOOB_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, const char* p_methodName, int31_t p_numArgs, TNode<JSTypedArray> p_exemplar, TNode<JSAny> p_arg0, TNode<JSAny> p_arg1, TNode<JSAny> p_arg2, TypedArrayAccessMode p_accessMode);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=552&c=1
TNode<JSTypedArray> TypedArraySpeciesCreateByLength_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, const char* p_methodName, TNode<JSTypedArray> p_exemplar, TNode<UintPtrT> p_length, TypedArrayAccessMode p_accessMode);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=575&c=1
TNode<JSTypedArray> TypedArraySpeciesCreateByBuffer_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, const char* p_methodName, TNode<JSTypedArray> p_exemplar, TNode<JSArrayBuffer> p_buffer, TNode<UintPtrT> p_beginByteOffset, TNode<Union<HeapNumber, Smi, Undefined>> p_newLength, TypedArrayAccessMode p_accessMode);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=588&c=1
TNode<JSTypedArray> TypedArrayCreateSameType_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSTypedArray> p_exemplar, TNode<UintPtrT> p_newLength);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=39&c=5
TNode<Weak<HeapObject>> Cast_Weak_JSArrayBufferView_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, Smi, Weak<HeapObject>>> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/typed-array-createtypedarray.tq?l=48&c=32
TNode<Weak<HeapObject>> StrongToWeak_JSArrayBufferView_0(compiler::CodeAssemblerState* state_, TNode<JSArrayBufferView> p_x);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_TYPED_ARRAY_CREATETYPEDARRAY_TQ_CSA_H_
