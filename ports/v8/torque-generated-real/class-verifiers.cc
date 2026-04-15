#ifdef VERIFY_HEAP
#include "torque-generated/class-verifiers.h"

#include "src/objects/all-objects-inl.h"

// Has to be the last include (doesn't have include guards):
#include "src/objects/object-macros.h"
namespace v8 {
namespace internal {
#include "torque-generated/test/torque/test-torque-tq-inl.inc"
void TorqueGeneratedClassVerifiers::TrustedForeignVerify(Tagged<TrustedForeign> o, Isolate* isolate) {
  CHECK(IsTrustedForeign(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSReceiverVerify(Tagged<JSReceiver> o, Isolate* isolate) {
  CHECK(IsJSReceiver(o, isolate));
  {
    Tagged<Object> properties_or_hash__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, properties_or_hash__value);
    CHECK(IsFixedArrayBase(properties_or_hash__value) || IsSwissNameDictionary(properties_or_hash__value) || IsSmi(properties_or_hash__value) || IsPropertyArray(properties_or_hash__value));
  }
}
void TorqueGeneratedClassVerifiers::JSObjectVerify(Tagged<JSObject> o, Isolate* isolate) {
  o->JSReceiverVerify(isolate);
  CHECK(IsJSObject(o, isolate));
  {
    Tagged<Object> elements__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, elements__value);
    CHECK(IsFixedArrayBase(elements__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmInstanceObjectVerify(Tagged<WasmInstanceObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsWasmInstanceObject(o, isolate));
  {
    Tagged<Object> module_object__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, module_object__value);
    CHECK(IsWasmModuleObject(module_object__value));
  }
  {
    Tagged<Object> exports_object__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, exports_object__value);
    CHECK(IsJSObject(exports_object__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmImportDataVerify(Tagged<WasmImportData> o, Isolate* isolate) {
  CHECK(IsWasmImportData(o, isolate));
  {
    Tagged<Object> native_context__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, native_context__value);
    CHECK(IsNativeContext(native_context__value));
  }
  {
    Tagged<Object> callable__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, callable__value);
    CHECK(IsJSReceiver(callable__value) || IsUndefined(callable__value));
  }
  {
    Tagged<Object> wrapper_budget__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, wrapper_budget__value);
    CHECK(IsCell(wrapper_budget__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmFastApiCallDataVerify(Tagged<WasmFastApiCallData> o, Isolate* isolate) {
  CHECK(IsWasmFastApiCallData(o, isolate));
  {
    Tagged<Object> signature__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, signature__value);
    CHECK(IsHeapObject(signature__value));
  }
  {
    Tagged<Object> callback_data__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, callback_data__value);
  }
  {
    Tagged<MaybeObject> cached_map__value = TaggedField<MaybeObject>::load(o, 12);
    Object::VerifyMaybeObjectPointer(isolate, cached_map__value);
    CHECK(cached_map__value.IsCleared() || (!cached_map__value.IsWeak() && IsNull(cached_map__value.GetHeapObjectOrSmi())) || (cached_map__value.IsWeak() && IsMap(cached_map__value.GetHeapObjectOrSmi())));
  }
}
void TorqueGeneratedClassVerifiers::WasmInternalFunctionVerify(Tagged<WasmInternalFunction> o, Isolate* isolate) {
  CHECK(IsWasmInternalFunction(o, isolate));
  {
    Tagged<Object> external__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, external__value);
    CHECK(IsJSFunction(external__value) || IsUndefined(external__value));
  }
  {
    Tagged<Object> function_index__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, function_index__value);
    CHECK(IsSmi(function_index__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmFuncRefVerify(Tagged<WasmFuncRef> o, Isolate* isolate) {
  CHECK(IsWasmFuncRef(o, isolate));
}
void TorqueGeneratedClassVerifiers::WasmFunctionDataVerify(Tagged<WasmFunctionData> o, Isolate* isolate) {
  CHECK(IsWasmFunctionData(o, isolate));
  {
    Tagged<Object> func_ref__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, func_ref__value);
    CHECK(IsWasmFuncRef(func_ref__value));
  }
  {
    Tagged<Object> js_promise_flags__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, js_promise_flags__value);
    CHECK(IsSmi(js_promise_flags__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmExportedFunctionDataVerify(Tagged<WasmExportedFunctionData> o, Isolate* isolate) {
  o->WasmFunctionDataVerify(isolate);
  CHECK(IsWasmExportedFunctionData(o, isolate));
  {
    Tagged<Object> function_index__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, function_index__value);
    CHECK(IsSmi(function_index__value));
  }
  {
    Tagged<Object> wrapper_budget__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, wrapper_budget__value);
    CHECK(IsCell(wrapper_budget__value));
  }
  {
    Tagged<Object> packed_args_size__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, packed_args_size__value);
    CHECK(IsSmi(packed_args_size__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmJSFunctionDataVerify(Tagged<WasmJSFunctionData> o, Isolate* isolate) {
  o->WasmFunctionDataVerify(isolate);
  CHECK(IsWasmJSFunctionData(o, isolate));
}
void TorqueGeneratedClassVerifiers::WasmCapiFunctionDataVerify(Tagged<WasmCapiFunctionData> o, Isolate* isolate) {
  o->WasmFunctionDataVerify(isolate);
  CHECK(IsWasmCapiFunctionData(o, isolate));
  {
    Tagged<Object> embedder_data__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, embedder_data__value);
    CHECK(IsForeign(embedder_data__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmResumeDataVerify(Tagged<WasmResumeData> o, Isolate* isolate) {
  CHECK(IsWasmResumeData(o, isolate));
  {
    Tagged<Object> on_resume__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, on_resume__value);
    CHECK(IsSmi(on_resume__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmSuspenderObjectVerify(Tagged<WasmSuspenderObject> o, Isolate* isolate) {
  CHECK(IsWasmSuspenderObject(o, isolate));
  {
    Tagged<Object> promise__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, promise__value);
    CHECK(IsUndefined(promise__value) || IsJSPromise(promise__value));
  }
  {
    Tagged<Object> resume__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, resume__value);
    CHECK(IsJSObject(resume__value) || IsUndefined(resume__value));
  }
  {
    Tagged<Object> reject__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, reject__value);
    CHECK(IsJSObject(reject__value) || IsUndefined(reject__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmContinuationObjectVerify(Tagged<WasmContinuationObject> o, Isolate* isolate) {
  CHECK(IsWasmContinuationObject(o, isolate));
  {
    Tagged<Object> stack_obj__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, stack_obj__value);
    CHECK(IsWasmStackObject(stack_obj__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmStackObjectVerify(Tagged<WasmStackObject> o, Isolate* isolate) {
  CHECK(IsWasmStackObject(o, isolate));
}
void TorqueGeneratedClassVerifiers::StructVerify(Tagged<Struct> o, Isolate* isolate) {
  CHECK(IsStruct(o, isolate));
}
void TorqueGeneratedClassVerifiers::WasmExceptionTagVerify(Tagged<WasmExceptionTag> o, Isolate* isolate) {
  o->StructVerify(isolate);
  CHECK(IsWasmExceptionTag(o, isolate));
  {
    Tagged<Object> index__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, index__value);
    CHECK(IsSmi(index__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmModuleObjectVerify(Tagged<WasmModuleObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsWasmModuleObject(o, isolate));
  {
    Tagged<Object> managed_native_module__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, managed_native_module__value);
    CHECK(IsForeign(managed_native_module__value));
  }
  {
    Tagged<Object> script__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, script__value);
    CHECK(IsScript(script__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmTableObjectVerify(Tagged<WasmTableObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsWasmTableObject(o, isolate));
  {
    Tagged<Object> entries__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, entries__value);
    CHECK(IsFixedArray(entries__value));
  }
  {
    Tagged<Object> current_length__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, current_length__value);
    CHECK(IsSmi(current_length__value));
  }
  {
    Tagged<Object> maximum_length__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, maximum_length__value);
    CHECK(IsBigInt(maximum_length__value) || IsUndefined(maximum_length__value) || IsSmi(maximum_length__value) || IsHeapNumber(maximum_length__value));
  }
  {
    Tagged<Object> raw_type__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, raw_type__value);
    CHECK(IsSmi(raw_type__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmMemoryObjectVerify(Tagged<WasmMemoryObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsWasmMemoryObject(o, isolate));
  {
    Tagged<Object> array_buffer__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, array_buffer__value);
    CHECK(IsUndefined(array_buffer__value) || IsJSArrayBuffer(array_buffer__value));
  }
  {
    Tagged<Object> managed_backing_store__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, managed_backing_store__value);
    CHECK(IsForeign(managed_backing_store__value));
  }
  {
    Tagged<Object> maximum_pages__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, maximum_pages__value);
    CHECK(IsSmi(maximum_pages__value));
  }
  {
    Tagged<Object> instances__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, instances__value);
    CHECK(IsWeakArrayList(instances__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmMemoryMapDescriptorVerify(Tagged<WasmMemoryMapDescriptor> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsWasmMemoryMapDescriptor(o, isolate));
  {
    Tagged<MaybeObject> memory__value = TaggedField<MaybeObject>::load(o, 12);
    Object::VerifyMaybeObjectPointer(isolate, memory__value);
    CHECK(memory__value.IsCleared() || (memory__value.IsWeak() && IsWasmMemoryObject(memory__value.GetHeapObjectOrSmi())));
  }
}
void TorqueGeneratedClassVerifiers::WasmGlobalObjectVerify(Tagged<WasmGlobalObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsWasmGlobalObject(o, isolate));
  {
    Tagged<Object> buffer__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, buffer__value);
    CHECK(IsByteArray(buffer__value) || IsFixedArray(buffer__value));
  }
  {
    Tagged<Object> offset__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, offset__value);
    CHECK(IsSmi(offset__value));
  }
  {
    Tagged<Object> raw_type__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, raw_type__value);
    CHECK(IsSmi(raw_type__value));
  }
  {
    Tagged<Object> is_mutable__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, is_mutable__value);
    CHECK(IsSmi(is_mutable__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmTagObjectVerify(Tagged<WasmTagObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsWasmTagObject(o, isolate));
  {
    Tagged<Object> tag__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, tag__value);
    CHECK(IsHeapObject(tag__value));
  }
  {
    Tagged<Object> canonical_type_index__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, canonical_type_index__value);
    CHECK(IsSmi(canonical_type_index__value));
  }
}
void TorqueGeneratedClassVerifiers::JSFunctionOrBoundFunctionOrWrappedFunctionVerify(Tagged<JSFunctionOrBoundFunctionOrWrappedFunction> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSFunctionOrBoundFunctionOrWrappedFunction(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSFunctionVerify(Tagged<JSFunction> o, Isolate* isolate) {
  o->JSFunctionOrBoundFunctionOrWrappedFunctionVerify(isolate);
  CHECK(IsJSFunction(o, isolate));
  {
    Tagged<Object> shared_function_info__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, shared_function_info__value);
    CHECK(IsSharedFunctionInfo(shared_function_info__value));
  }
  {
    Tagged<Object> context__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, context__value);
    CHECK(IsContext(context__value));
  }
  {
    Tagged<Object> feedback_cell__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, feedback_cell__value);
    CHECK(IsFeedbackCell(feedback_cell__value));
  }
}
void TorqueGeneratedClassVerifiers::AsmWasmDataVerify(Tagged<AsmWasmData> o, Isolate* isolate) {
  o->StructVerify(isolate);
  CHECK(IsAsmWasmData(o, isolate));
  {
    Tagged<Object> managed_native_module__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, managed_native_module__value);
    CHECK(IsForeign(managed_native_module__value));
  }
  {
    Tagged<Object> uses_bitset__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, uses_bitset__value);
    CHECK(IsHeapNumber(uses_bitset__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmTypeInfoVerify(Tagged<WasmTypeInfo> o, Isolate* isolate) {
  CHECK(IsWasmTypeInfo(o, isolate));
  {
    Tagged<Object> supertypes_length__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, supertypes_length__value);
    CHECK(IsSmi(supertypes_length__value));
  }
  intptr_t supertypes__offset, supertypes__length;
  std::tie(std::ignore, supertypes__offset, supertypes__length) = TqRuntimeFieldSliceWasmTypeInfoSupertypes(o);
  CHECK_EQ(supertypes__offset, static_cast<int>(supertypes__offset));
  CHECK_EQ(supertypes__length, static_cast<int>(supertypes__length));
  for (int i = 0; i < static_cast<int>(supertypes__length); ++i) {
    Tagged<Object> supertypes__value = TaggedField<Object>::load(o, static_cast<int>(supertypes__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, supertypes__value);
  }
}
void TorqueGeneratedClassVerifiers::WasmObjectVerify(Tagged<WasmObject> o, Isolate* isolate) {
  o->JSReceiverVerify(isolate);
  CHECK(IsWasmObject(o, isolate));
}
void TorqueGeneratedClassVerifiers::WasmStructVerify(Tagged<WasmStruct> o, Isolate* isolate) {
  o->WasmObjectVerify(isolate);
  CHECK(IsWasmStruct(o, isolate));
}
void TorqueGeneratedClassVerifiers::WasmArrayVerify(Tagged<WasmArray> o, Isolate* isolate) {
  o->WasmObjectVerify(isolate);
  CHECK(IsWasmArray(o, isolate));
}
void TorqueGeneratedClassVerifiers::WasmStringViewIterVerify(Tagged<WasmStringViewIter> o, Isolate* isolate) {
  CHECK(IsWasmStringViewIter(o, isolate));
  {
    Tagged<Object> string__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, string__value);
    CHECK(IsString(string__value));
  }
}
void TorqueGeneratedClassVerifiers::WasmNullVerify(Tagged<WasmNull> o, Isolate* isolate) {
  CHECK(IsWasmNull(o, isolate));
}
void TorqueGeneratedClassVerifiers::WasmSuspendingObjectVerify(Tagged<WasmSuspendingObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsWasmSuspendingObject(o, isolate));
  {
    Tagged<Object> callable__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, callable__value);
    CHECK(IsJSReceiver(callable__value));
  }
}
void TorqueGeneratedClassVerifiers::JSGeneratorObjectVerify(Tagged<JSGeneratorObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSGeneratorObject(o, isolate));
  {
    Tagged<Object> function__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, function__value);
    CHECK(IsJSFunction(function__value));
  }
  {
    Tagged<Object> context__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, context__value);
    CHECK(IsContext(context__value));
  }
  {
    Tagged<Object> receiver__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, receiver__value);
    CHECK(IsJSReceiver(receiver__value) || IsBigInt(receiver__value) || IsUndefined(receiver__value) || IsSmi(receiver__value) || IsHeapNumber(receiver__value) || IsString(receiver__value) || IsSymbol(receiver__value) || IsBoolean(receiver__value) || IsNull(receiver__value));
  }
  {
    Tagged<Object> input_or_debug_pos__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, input_or_debug_pos__value);
  }
  {
    Tagged<Object> resume_mode__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, resume_mode__value);
    CHECK(IsSmi(resume_mode__value));
  }
  {
    Tagged<Object> continuation__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, continuation__value);
    CHECK(IsSmi(continuation__value));
  }
  {
    Tagged<Object> parameters_and_registers__value = TaggedField<Object>::load(o, 36);
    Object::VerifyPointer(isolate, parameters_and_registers__value);
    CHECK(IsFixedArray(parameters_and_registers__value));
  }
}
void TorqueGeneratedClassVerifiers::JSAsyncFunctionObjectVerify(Tagged<JSAsyncFunctionObject> o, Isolate* isolate) {
  o->JSGeneratorObjectVerify(isolate);
  CHECK(IsJSAsyncFunctionObject(o, isolate));
  {
    Tagged<Object> promise__value = TaggedField<Object>::load(o, 40);
    Object::VerifyPointer(isolate, promise__value);
    CHECK(IsJSPromise(promise__value));
  }
  {
    Tagged<Object> await_resolve_closure__value = TaggedField<Object>::load(o, 44);
    Object::VerifyPointer(isolate, await_resolve_closure__value);
    CHECK(IsJSFunction(await_resolve_closure__value) || IsUndefined(await_resolve_closure__value));
  }
  {
    Tagged<Object> await_reject_closure__value = TaggedField<Object>::load(o, 48);
    Object::VerifyPointer(isolate, await_reject_closure__value);
    CHECK(IsJSFunction(await_reject_closure__value) || IsUndefined(await_reject_closure__value));
  }
}
void TorqueGeneratedClassVerifiers::JSAsyncGeneratorObjectVerify(Tagged<JSAsyncGeneratorObject> o, Isolate* isolate) {
  o->JSGeneratorObjectVerify(isolate);
  CHECK(IsJSAsyncGeneratorObject(o, isolate));
  {
    Tagged<Object> queue__value = TaggedField<Object>::load(o, 40);
    Object::VerifyPointer(isolate, queue__value);
    CHECK(IsHeapObject(queue__value));
  }
  {
    Tagged<Object> is_awaiting__value = TaggedField<Object>::load(o, 44);
    Object::VerifyPointer(isolate, is_awaiting__value);
    CHECK(IsSmi(is_awaiting__value));
  }
}
void TorqueGeneratedClassVerifiers::JSRegExpVerify(Tagged<JSRegExp> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSRegExp(o, isolate));
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsUndefined(flags__value) || IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::JSFunctionWithPrototypeVerify(Tagged<JSFunctionWithPrototype> o, Isolate* isolate) {
  o->JSFunctionVerify(isolate);
  CHECK(IsJSFunctionWithPrototype(o, isolate));
  {
    Tagged<Object> prototype_or_initial_map__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, prototype_or_initial_map__value);
    CHECK(IsJSReceiver(prototype_or_initial_map__value) || IsMap(prototype_or_initial_map__value) || IsTheHole(prototype_or_initial_map__value));
  }
}
void TorqueGeneratedClassVerifiers::JSArrayVerify(Tagged<JSArray> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSArray(o, isolate));
  {
    Tagged<Object> length__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, length__value);
    CHECK(IsSmi(length__value) || IsHeapNumber(length__value));
  }
}
void TorqueGeneratedClassVerifiers::JSListFormatVerify(Tagged<JSListFormat> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSListFormat(o, isolate));
  {
    Tagged<Object> locale__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, locale__value);
    CHECK(IsString(locale__value));
  }
  {
    Tagged<Object> icu_formatter__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, icu_formatter__value);
    CHECK(IsForeign(icu_formatter__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::JSSegmenterVerify(Tagged<JSSegmenter> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSSegmenter(o, isolate));
  {
    Tagged<Object> locale__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, locale__value);
    CHECK(IsString(locale__value));
  }
  {
    Tagged<Object> icu_break_iterator__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, icu_break_iterator__value);
    CHECK(IsForeign(icu_break_iterator__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::MapVerify(Tagged<Map> o, Isolate* isolate) {
  CHECK(IsMap(o, isolate));
  {
    Tagged<Object> prototype__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, prototype__value);
    CHECK(IsJSReceiver(prototype__value) || IsNull(prototype__value));
  }
  {
    Tagged<Object> constructor_or_back_pointer_or_native_context__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, constructor_or_back_pointer_or_native_context__value);
  }
  {
    Tagged<Object> instance_descriptors__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, instance_descriptors__value);
    CHECK(IsWasmStruct(instance_descriptors__value) || IsDescriptorArray(instance_descriptors__value));
  }
  {
    Tagged<Object> dependent_code__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, dependent_code__value);
    CHECK(IsMap(dependent_code__value) || IsDependentCode(dependent_code__value));
  }
  {
    Tagged<Object> prototype_validity_cell__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, prototype_validity_cell__value);
    CHECK(IsCell(prototype_validity_cell__value) || IsZero(prototype_validity_cell__value));
  }
  {
    Tagged<MaybeObject> transitions_or_prototype_info__value = TaggedField<MaybeObject>::load(o, 36);
    Object::VerifyMaybeObjectPointer(isolate, transitions_or_prototype_info__value);
    CHECK(transitions_or_prototype_info__value.IsCleared() || (!transitions_or_prototype_info__value.IsWeak() && IsPrototypeInfo(transitions_or_prototype_info__value.GetHeapObjectOrSmi())) || (!transitions_or_prototype_info__value.IsWeak() && IsMap(transitions_or_prototype_info__value.GetHeapObjectOrSmi())) || (transitions_or_prototype_info__value.IsWeak() && IsMap(transitions_or_prototype_info__value.GetHeapObjectOrSmi())) || (!transitions_or_prototype_info__value.IsWeak() && IsZero(transitions_or_prototype_info__value.GetHeapObjectOrSmi())) || (!transitions_or_prototype_info__value.IsWeak() && IsPrototypeSharedClosureInfo(transitions_or_prototype_info__value.GetHeapObjectOrSmi())) || (!transitions_or_prototype_info__value.IsWeak() && IsTransitionArray(transitions_or_prototype_info__value.GetHeapObjectOrSmi())));
  }
}
void TorqueGeneratedClassVerifiers::AccessorInfoVerify(Tagged<AccessorInfo> o, Isolate* isolate) {
  CHECK(IsAccessorInfo(o, isolate));
  {
    Tagged<Object> data__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, data__value);
  }
  {
    Tagged<Object> name__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, name__value);
    CHECK(IsName(name__value));
  }
}
void TorqueGeneratedClassVerifiers::DescriptorArrayVerify(Tagged<DescriptorArray> o, Isolate* isolate) {
  CHECK(IsDescriptorArray(o, isolate));
  {
    Tagged<Object> enum_cache__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, enum_cache__value);
    CHECK(IsEnumCache(enum_cache__value));
  }
  intptr_t descriptors__offset, descriptors__length;
  std::tie(std::ignore, descriptors__offset, descriptors__length) = TqRuntimeFieldSliceDescriptorArrayDescriptors(o);
  CHECK_EQ(descriptors__offset, static_cast<int>(descriptors__offset));
  CHECK_EQ(descriptors__length, static_cast<int>(descriptors__length));
  for (int i = 0; i < static_cast<int>(descriptors__length); ++i) {
    Tagged<Object> key__value = TaggedField<Object>::load(o, static_cast<int>(descriptors__offset) + 0 + i * 12);
    Object::VerifyPointer(isolate, key__value);
    CHECK(IsName(key__value) || IsUndefined(key__value));
    Tagged<Object> details__value = TaggedField<Object>::load(o, static_cast<int>(descriptors__offset) + 4 + i * 12);
    Object::VerifyPointer(isolate, details__value);
    CHECK(IsUndefined(details__value) || IsSmi(details__value));
    Tagged<MaybeObject> value__value = TaggedField<MaybeObject>::load(o, static_cast<int>(descriptors__offset) + 8 + i * 12);
    Object::VerifyMaybeObjectPointer(isolate, value__value);
    CHECK(value__value.IsCleared() || (!value__value.IsWeak() && IsJSReceiver(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsBigInt(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsUndefined(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsSmi(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsHeapNumber(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsString(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsSymbol(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsBoolean(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsNull(value__value.GetHeapObjectOrSmi())) || (value__value.IsWeak() && IsMap(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsAccessorInfo(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsAccessorPair(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsClassPositions(value__value.GetHeapObjectOrSmi())) || (!value__value.IsWeak() && IsNumberDictionary(value__value.GetHeapObjectOrSmi())));
  }
}
void TorqueGeneratedClassVerifiers::StrongDescriptorArrayVerify(Tagged<StrongDescriptorArray> o, Isolate* isolate) {
  o->DescriptorArrayVerify(isolate);
  CHECK(IsStrongDescriptorArray(o, isolate));
}
void TorqueGeneratedClassVerifiers::InterceptorInfoVerify(Tagged<InterceptorInfo> o, Isolate* isolate) {
  CHECK(IsInterceptorInfo(o, isolate));
  {
    Tagged<Object> data__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, data__value);
  }
}
void TorqueGeneratedClassVerifiers::JSArgumentsObjectVerify(Tagged<JSArgumentsObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSArgumentsObject(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSObjectWithEmbedderSlotsVerify(Tagged<JSObjectWithEmbedderSlots> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSObjectWithEmbedderSlots(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSPromiseVerify(Tagged<JSPromise> o, Isolate* isolate) {
  o->JSObjectWithEmbedderSlotsVerify(isolate);
  CHECK(IsJSPromise(o, isolate));
  {
    Tagged<Object> reactions_or_result__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, reactions_or_result__value);
    CHECK(IsJSReceiver(reactions_or_result__value) || IsBigInt(reactions_or_result__value) || IsUndefined(reactions_or_result__value) || IsSmi(reactions_or_result__value) || IsHeapNumber(reactions_or_result__value) || IsString(reactions_or_result__value) || IsSymbol(reactions_or_result__value) || IsBoolean(reactions_or_result__value) || IsNull(reactions_or_result__value) || IsPromiseReaction(reactions_or_result__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::FeedbackVectorVerify(Tagged<FeedbackVector> o, Isolate* isolate) {
  CHECK(IsFeedbackVector(o, isolate));
  {
    Tagged<Object> shared_function_info__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, shared_function_info__value);
    CHECK(IsSharedFunctionInfo(shared_function_info__value));
  }
  {
    Tagged<Object> closure_feedback_cell_array__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, closure_feedback_cell_array__value);
    CHECK(IsClosureFeedbackCellArray(closure_feedback_cell_array__value));
  }
  {
    Tagged<Object> parent_feedback_cell__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, parent_feedback_cell__value);
    CHECK(IsFeedbackCell(parent_feedback_cell__value));
  }
  intptr_t raw_feedback_slots__offset, raw_feedback_slots__length;
  std::tie(std::ignore, raw_feedback_slots__offset, raw_feedback_slots__length) = TqRuntimeFieldSliceFeedbackVectorRawFeedbackSlots(o);
  CHECK_EQ(raw_feedback_slots__offset, static_cast<int>(raw_feedback_slots__offset));
  CHECK_EQ(raw_feedback_slots__length, static_cast<int>(raw_feedback_slots__length));
  for (int i = 0; i < static_cast<int>(raw_feedback_slots__length); ++i) {
    Tagged<MaybeObject> raw_feedback_slots__value = TaggedField<MaybeObject>::load(o, static_cast<int>(raw_feedback_slots__offset) + i * kTaggedSize);
    Object::VerifyMaybeObjectPointer(isolate, raw_feedback_slots__value);
    CHECK(raw_feedback_slots__value.IsCleared() || (!raw_feedback_slots__value.IsWeak() && IsHeapObject(raw_feedback_slots__value.GetHeapObjectOrSmi())) || (!raw_feedback_slots__value.IsWeak() && IsSmi(raw_feedback_slots__value.GetHeapObjectOrSmi())) || raw_feedback_slots__value.IsWeak());
  }
}
void TorqueGeneratedClassVerifiers::JSLocaleVerify(Tagged<JSLocale> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSLocale(o, isolate));
  {
    Tagged<Object> icu_locale__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, icu_locale__value);
    CHECK(IsForeign(icu_locale__value));
  }
}
void TorqueGeneratedClassVerifiers::JSBoundFunctionVerify(Tagged<JSBoundFunction> o, Isolate* isolate) {
  o->JSFunctionOrBoundFunctionOrWrappedFunctionVerify(isolate);
  CHECK(IsJSBoundFunction(o, isolate));
  {
    Tagged<Object> bound_target_function__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, bound_target_function__value);
    CHECK(IsJSFunction(bound_target_function__value) || IsJSBoundFunction(bound_target_function__value) || IsJSWrappedFunction(bound_target_function__value) || IsCallableJSProxy(bound_target_function__value) || IsCallableApiObject(bound_target_function__value));
  }
  {
    Tagged<Object> bound_this__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, bound_this__value);
    CHECK(IsJSReceiver(bound_this__value) || IsBigInt(bound_this__value) || IsUndefined(bound_this__value) || IsSmi(bound_this__value) || IsHeapNumber(bound_this__value) || IsString(bound_this__value) || IsSymbol(bound_this__value) || IsBoolean(bound_this__value) || IsNull(bound_this__value) || IsSourceTextModule(bound_this__value));
  }
  {
    Tagged<Object> bound_arguments__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, bound_arguments__value);
    CHECK(IsFixedArray(bound_arguments__value));
  }
}
void TorqueGeneratedClassVerifiers::JSWrappedFunctionVerify(Tagged<JSWrappedFunction> o, Isolate* isolate) {
  o->JSFunctionOrBoundFunctionOrWrappedFunctionVerify(isolate);
  CHECK(IsJSWrappedFunction(o, isolate));
  {
    Tagged<Object> wrapped_target_function__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, wrapped_target_function__value);
    CHECK(IsJSFunction(wrapped_target_function__value) || IsJSBoundFunction(wrapped_target_function__value) || IsJSWrappedFunction(wrapped_target_function__value) || IsCallableJSProxy(wrapped_target_function__value) || IsCallableApiObject(wrapped_target_function__value));
  }
  {
    Tagged<Object> context__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, context__value);
    CHECK(IsNativeContext(context__value));
  }
}
void TorqueGeneratedClassVerifiers::JSFunctionWithoutPrototypeVerify(Tagged<JSFunctionWithoutPrototype> o, Isolate* isolate) {
  o->JSFunctionVerify(isolate);
  CHECK(IsJSFunctionWithoutPrototype(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSArrayIteratorVerify(Tagged<JSArrayIterator> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSArrayIterator(o, isolate));
  {
    Tagged<Object> iterated_object__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, iterated_object__value);
    CHECK(IsJSReceiver(iterated_object__value));
  }
  {
    Tagged<Object> next_index__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, next_index__value);
    CHECK(IsSmi(next_index__value) || IsHeapNumber(next_index__value));
  }
  {
    Tagged<Object> kind__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, kind__value);
    CHECK(IsSmi(kind__value));
  }
}
void TorqueGeneratedClassVerifiers::TemplateLiteralObjectVerify(Tagged<TemplateLiteralObject> o, Isolate* isolate) {
  o->JSArrayVerify(isolate);
  CHECK(IsTemplateLiteralObject(o, isolate));
  {
    Tagged<Object> raw__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, raw__value);
    CHECK(IsJSArray(raw__value));
  }
  {
    Tagged<Object> function_literal_id__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, function_literal_id__value);
    CHECK(IsSmi(function_literal_id__value));
  }
  {
    Tagged<Object> slot_id__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, slot_id__value);
    CHECK(IsSmi(slot_id__value));
  }
}
void TorqueGeneratedClassVerifiers::JSRawJsonVerify(Tagged<JSRawJson> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSRawJson(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSNumberFormatVerify(Tagged<JSNumberFormat> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSNumberFormat(o, isolate));
  {
    Tagged<Object> locale__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, locale__value);
    CHECK(IsString(locale__value));
  }
  {
    Tagged<Object> icu_number_formatter__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, icu_number_formatter__value);
    CHECK(IsForeign(icu_number_formatter__value));
  }
  {
    Tagged<Object> bound_format__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, bound_format__value);
    CHECK(IsJSFunction(bound_format__value) || IsUndefined(bound_format__value));
  }
}
void TorqueGeneratedClassVerifiers::JSPluralRulesVerify(Tagged<JSPluralRules> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSPluralRules(o, isolate));
  {
    Tagged<Object> locale__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, locale__value);
    CHECK(IsString(locale__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
  {
    Tagged<Object> icu_plural_rules__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, icu_plural_rules__value);
    CHECK(IsForeign(icu_plural_rules__value));
  }
  {
    Tagged<Object> icu_number_formatter__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, icu_number_formatter__value);
    CHECK(IsForeign(icu_number_formatter__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorHelperVerify(Tagged<JSIteratorHelper> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSIteratorHelper(o, isolate));
  {
    Tagged<Object> state__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, state__value);
    CHECK(IsSmi(state__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorHelperSimpleVerify(Tagged<JSIteratorHelperSimple> o, Isolate* isolate) {
  o->JSIteratorHelperVerify(isolate);
  CHECK(IsJSIteratorHelperSimple(o, isolate));
  {
    Tagged<Object> object__value = TaggedField<Object>::load(o, 16 + 0);
    Object::VerifyPointer(isolate, object__value);
    CHECK(IsJSReceiver(object__value));
    Tagged<Object> next__value = TaggedField<Object>::load(o, 16 + 4);
    Object::VerifyPointer(isolate, next__value);
    CHECK(IsJSReceiver(next__value) || IsBigInt(next__value) || IsUndefined(next__value) || IsSmi(next__value) || IsHeapNumber(next__value) || IsString(next__value) || IsSymbol(next__value) || IsBoolean(next__value) || IsNull(next__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorMapHelperVerify(Tagged<JSIteratorMapHelper> o, Isolate* isolate) {
  o->JSIteratorHelperSimpleVerify(isolate);
  CHECK(IsJSIteratorMapHelper(o, isolate));
  {
    Tagged<Object> mapper__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, mapper__value);
    CHECK(IsJSFunction(mapper__value) || IsJSBoundFunction(mapper__value) || IsJSWrappedFunction(mapper__value) || IsCallableJSProxy(mapper__value) || IsCallableApiObject(mapper__value));
  }
  {
    Tagged<Object> counter__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, counter__value);
    CHECK(IsSmi(counter__value) || IsHeapNumber(counter__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorFilterHelperVerify(Tagged<JSIteratorFilterHelper> o, Isolate* isolate) {
  o->JSIteratorHelperSimpleVerify(isolate);
  CHECK(IsJSIteratorFilterHelper(o, isolate));
  {
    Tagged<Object> predicate__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, predicate__value);
    CHECK(IsJSFunction(predicate__value) || IsJSBoundFunction(predicate__value) || IsJSWrappedFunction(predicate__value) || IsCallableJSProxy(predicate__value) || IsCallableApiObject(predicate__value));
  }
  {
    Tagged<Object> counter__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, counter__value);
    CHECK(IsSmi(counter__value) || IsHeapNumber(counter__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorTakeHelperVerify(Tagged<JSIteratorTakeHelper> o, Isolate* isolate) {
  o->JSIteratorHelperSimpleVerify(isolate);
  CHECK(IsJSIteratorTakeHelper(o, isolate));
  {
    Tagged<Object> remaining__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, remaining__value);
    CHECK(IsSmi(remaining__value) || IsHeapNumber(remaining__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorDropHelperVerify(Tagged<JSIteratorDropHelper> o, Isolate* isolate) {
  o->JSIteratorHelperSimpleVerify(isolate);
  CHECK(IsJSIteratorDropHelper(o, isolate));
  {
    Tagged<Object> remaining__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, remaining__value);
    CHECK(IsSmi(remaining__value) || IsHeapNumber(remaining__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorFlatMapHelperVerify(Tagged<JSIteratorFlatMapHelper> o, Isolate* isolate) {
  o->JSIteratorHelperSimpleVerify(isolate);
  CHECK(IsJSIteratorFlatMapHelper(o, isolate));
  {
    Tagged<Object> mapper__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, mapper__value);
    CHECK(IsJSFunction(mapper__value) || IsJSBoundFunction(mapper__value) || IsJSWrappedFunction(mapper__value) || IsCallableJSProxy(mapper__value) || IsCallableApiObject(mapper__value));
  }
  {
    Tagged<Object> counter__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, counter__value);
    CHECK(IsSmi(counter__value) || IsHeapNumber(counter__value));
  }
  {
    Tagged<Object> object__value = TaggedField<Object>::load(o, 32 + 0);
    Object::VerifyPointer(isolate, object__value);
    CHECK(IsJSReceiver(object__value));
    Tagged<Object> next__value = TaggedField<Object>::load(o, 32 + 4);
    Object::VerifyPointer(isolate, next__value);
    CHECK(IsJSReceiver(next__value) || IsBigInt(next__value) || IsUndefined(next__value) || IsSmi(next__value) || IsHeapNumber(next__value) || IsString(next__value) || IsSymbol(next__value) || IsBoolean(next__value) || IsNull(next__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorConcatHelperVerify(Tagged<JSIteratorConcatHelper> o, Isolate* isolate) {
  o->JSIteratorHelperSimpleVerify(isolate);
  CHECK(IsJSIteratorConcatHelper(o, isolate));
  {
    Tagged<Object> iterables__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, iterables__value);
    CHECK(IsFixedArray(iterables__value));
  }
  {
    Tagged<Object> current__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, current__value);
    CHECK(IsSmi(current__value));
  }
}
void TorqueGeneratedClassVerifiers::JSIteratorZipHelperVerify(Tagged<JSIteratorZipHelper> o, Isolate* isolate) {
  o->JSIteratorHelperVerify(isolate);
  CHECK(IsJSIteratorZipHelper(o, isolate));
  {
    Tagged<Object> underlying_iterators__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, underlying_iterators__value);
    CHECK(IsFixedArray(underlying_iterators__value));
  }
  {
    Tagged<Object> mode__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, mode__value);
    CHECK(IsSmi(mode__value));
  }
  {
    Tagged<Object> active_count__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, active_count__value);
    CHECK(IsSmi(active_count__value));
  }
  {
    Tagged<Object> padding__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, padding__value);
    CHECK(IsFixedArray(padding__value));
  }
}
void TorqueGeneratedClassVerifiers::JSRelativeTimeFormatVerify(Tagged<JSRelativeTimeFormat> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSRelativeTimeFormat(o, isolate));
  {
    Tagged<Object> locale__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, locale__value);
    CHECK(IsString(locale__value));
  }
  {
    Tagged<Object> numberingSystem__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, numberingSystem__value);
    CHECK(IsString(numberingSystem__value));
  }
  {
    Tagged<Object> icu_formatter__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, icu_formatter__value);
    CHECK(IsForeign(icu_formatter__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::JSV8BreakIteratorVerify(Tagged<JSV8BreakIterator> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSV8BreakIterator(o, isolate));
  {
    Tagged<Object> locale__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, locale__value);
    CHECK(IsString(locale__value));
  }
  {
    Tagged<Object> icu_iterator_with_text__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, icu_iterator_with_text__value);
    CHECK(IsForeign(icu_iterator_with_text__value));
  }
  {
    Tagged<Object> bound_adopt_text__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, bound_adopt_text__value);
    CHECK(IsJSFunction(bound_adopt_text__value) || IsUndefined(bound_adopt_text__value));
  }
  {
    Tagged<Object> bound_first__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, bound_first__value);
    CHECK(IsJSFunction(bound_first__value) || IsUndefined(bound_first__value));
  }
  {
    Tagged<Object> bound_next__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, bound_next__value);
    CHECK(IsJSFunction(bound_next__value) || IsUndefined(bound_next__value));
  }
  {
    Tagged<Object> bound_current__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, bound_current__value);
    CHECK(IsJSFunction(bound_current__value) || IsUndefined(bound_current__value));
  }
  {
    Tagged<Object> bound_break_type__value = TaggedField<Object>::load(o, 36);
    Object::VerifyPointer(isolate, bound_break_type__value);
    CHECK(IsJSFunction(bound_break_type__value) || IsUndefined(bound_break_type__value));
  }
}
void TorqueGeneratedClassVerifiers::JSDisplayNamesVerify(Tagged<JSDisplayNames> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSDisplayNames(o, isolate));
  {
    Tagged<Object> internal__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, internal__value);
    CHECK(IsForeign(internal__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::AlwaysSharedSpaceJSObjectVerify(Tagged<AlwaysSharedSpaceJSObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsAlwaysSharedSpaceJSObject(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSSharedStructVerify(Tagged<JSSharedStruct> o, Isolate* isolate) {
  o->AlwaysSharedSpaceJSObjectVerify(isolate);
  CHECK(IsJSSharedStruct(o, isolate));
}
void TorqueGeneratedClassVerifiers::CppHeapExternalObjectVerify(Tagged<CppHeapExternalObject> o, Isolate* isolate) {
  CHECK(IsCppHeapExternalObject(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSSegmentsVerify(Tagged<JSSegments> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSSegments(o, isolate));
  {
    Tagged<Object> icu_iterator_with_text__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, icu_iterator_with_text__value);
    CHECK(IsForeign(icu_iterator_with_text__value));
  }
  {
    Tagged<Object> raw_string__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, raw_string__value);
    CHECK(IsString(raw_string__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::JSCollectionIteratorVerify(Tagged<JSCollectionIterator> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSCollectionIterator(o, isolate));
  {
    Tagged<Object> table__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, table__value);
  }
  {
    Tagged<Object> index__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, index__value);
  }
}
void TorqueGeneratedClassVerifiers::JSCollatorVerify(Tagged<JSCollator> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSCollator(o, isolate));
  {
    Tagged<Object> icu_collator__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, icu_collator__value);
    CHECK(IsForeign(icu_collator__value));
  }
  {
    Tagged<Object> bound_compare__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, bound_compare__value);
    CHECK(IsJSFunction(bound_compare__value) || IsUndefined(bound_compare__value));
  }
  {
    Tagged<Object> locale__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, locale__value);
    CHECK(IsString(locale__value));
  }
}
void TorqueGeneratedClassVerifiers::PropertyArrayVerify(Tagged<PropertyArray> o, Isolate* isolate) {
  CHECK(IsPropertyArray(o, isolate));
  {
    Tagged<Object> length_and_hash__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, length_and_hash__value);
    CHECK(IsSmi(length_and_hash__value));
  }
}
void TorqueGeneratedClassVerifiers::JSDisposableStackBaseVerify(Tagged<JSDisposableStackBase> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSDisposableStackBase(o, isolate));
  {
    Tagged<Object> stack__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, stack__value);
    CHECK(IsFixedArray(stack__value));
  }
  {
    Tagged<Object> status__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, status__value);
    CHECK(IsSmi(status__value));
  }
  {
    Tagged<Object> error__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, error__value);
  }
  {
    Tagged<Object> error_message__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, error_message__value);
  }
}
void TorqueGeneratedClassVerifiers::JSSyncDisposableStackVerify(Tagged<JSSyncDisposableStack> o, Isolate* isolate) {
  o->JSDisposableStackBaseVerify(isolate);
  CHECK(IsJSSyncDisposableStack(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSAsyncDisposableStackVerify(Tagged<JSAsyncDisposableStack> o, Isolate* isolate) {
  o->JSDisposableStackBaseVerify(isolate);
  CHECK(IsJSAsyncDisposableStack(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSAPIObjectWithEmbedderSlotsVerify(Tagged<JSAPIObjectWithEmbedderSlots> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSAPIObjectWithEmbedderSlots(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSArrayBufferVerify(Tagged<JSArrayBuffer> o, Isolate* isolate) {
  o->JSAPIObjectWithEmbedderSlotsVerify(isolate);
  CHECK(IsJSArrayBuffer(o, isolate));
  {
    Tagged<MaybeObject> views_or_detach_key__value = TaggedField<MaybeObject>::load(o, 16);
    Object::VerifyMaybeObjectPointer(isolate, views_or_detach_key__value);
    CHECK(views_or_detach_key__value.IsCleared() || (!views_or_detach_key__value.IsWeak() && IsSmi(views_or_detach_key__value.GetHeapObjectOrSmi())) || (!views_or_detach_key__value.IsWeak() && IsCell(views_or_detach_key__value.GetHeapObjectOrSmi())) || (views_or_detach_key__value.IsWeak() && IsJSArrayBufferView(views_or_detach_key__value.GetHeapObjectOrSmi())));
  }
}
void TorqueGeneratedClassVerifiers::JSArrayBufferViewVerify(Tagged<JSArrayBufferView> o, Isolate* isolate) {
  o->JSAPIObjectWithEmbedderSlotsVerify(isolate);
  CHECK(IsJSArrayBufferView(o, isolate));
  {
    Tagged<Object> buffer__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, buffer__value);
    CHECK(IsJSArrayBuffer(buffer__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTypedArrayVerify(Tagged<JSTypedArray> o, Isolate* isolate) {
  o->JSArrayBufferViewVerify(isolate);
  CHECK(IsJSTypedArray(o, isolate));
  {
    Tagged<Object> base_pointer__value = TaggedField<Object>::load(o, 56);
    Object::VerifyPointer(isolate, base_pointer__value);
    CHECK(IsByteArray(base_pointer__value) || IsSmi(base_pointer__value));
  }
}
void TorqueGeneratedClassVerifiers::JSDetachedTypedArrayVerify(Tagged<JSDetachedTypedArray> o, Isolate* isolate) {
  o->JSTypedArrayVerify(isolate);
  CHECK(IsJSDetachedTypedArray(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSDataViewOrRabGsabDataViewVerify(Tagged<JSDataViewOrRabGsabDataView> o, Isolate* isolate) {
  o->JSArrayBufferViewVerify(isolate);
  CHECK(IsJSDataViewOrRabGsabDataView(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSDataViewVerify(Tagged<JSDataView> o, Isolate* isolate) {
  o->JSDataViewOrRabGsabDataViewVerify(isolate);
  CHECK(IsJSDataView(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSRabGsabDataViewVerify(Tagged<JSRabGsabDataView> o, Isolate* isolate) {
  o->JSDataViewOrRabGsabDataViewVerify(isolate);
  CHECK(IsJSRabGsabDataView(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSFinalizationRegistryVerify(Tagged<JSFinalizationRegistry> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSFinalizationRegistry(o, isolate));
  {
    Tagged<Object> native_context__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, native_context__value);
    CHECK(IsNativeContext(native_context__value));
  }
  {
    Tagged<Object> cleanup__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, cleanup__value);
    CHECK(IsJSFunction(cleanup__value) || IsJSBoundFunction(cleanup__value) || IsJSWrappedFunction(cleanup__value) || IsCallableJSProxy(cleanup__value) || IsCallableApiObject(cleanup__value));
  }
  {
    Tagged<Object> active_cells__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, active_cells__value);
    CHECK(IsUndefined(active_cells__value) || IsWeakCell(active_cells__value));
  }
  {
    Tagged<Object> cleared_cells__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, cleared_cells__value);
    CHECK(IsUndefined(cleared_cells__value) || IsWeakCell(cleared_cells__value));
  }
  {
    Tagged<Object> key_map__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, key_map__value);
  }
  {
    Tagged<Object> next_dirty__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, next_dirty__value);
    CHECK(IsUndefined(next_dirty__value) || IsJSFinalizationRegistry(next_dirty__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 36);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::JSWeakRefVerify(Tagged<JSWeakRef> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSWeakRef(o, isolate));
  {
    Tagged<Object> target__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, target__value);
    CHECK(IsJSReceiver(target__value) || IsUndefined(target__value) || IsSymbol(target__value));
  }
}
void TorqueGeneratedClassVerifiers::SharedFunctionInfoVerify(Tagged<SharedFunctionInfo> o, Isolate* isolate) {
  CHECK(IsSharedFunctionInfo(o, isolate));
  {
    Tagged<Object> untrusted_function_data__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, untrusted_function_data__value);
  }
  {
    Tagged<Object> name_or_scope_info__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, name_or_scope_info__value);
    CHECK(IsString(name_or_scope_info__value) || IsScopeInfo(name_or_scope_info__value) || IsNoSharedNameSentinel(name_or_scope_info__value));
  }
  {
    Tagged<Object> outer_scope_info_or_feedback_metadata__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, outer_scope_info_or_feedback_metadata__value);
    CHECK(IsFeedbackMetadata(outer_scope_info_or_feedback_metadata__value) || IsTheHole(outer_scope_info_or_feedback_metadata__value) || IsScopeInfo(outer_scope_info_or_feedback_metadata__value));
  }
  {
    Tagged<Object> script__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, script__value);
    CHECK(IsUndefined(script__value) || IsScript(script__value));
  }
}
void TorqueGeneratedClassVerifiers::OnHeapBasicBlockProfilerDataVerify(Tagged<OnHeapBasicBlockProfilerData> o, Isolate* isolate) {
  CHECK(IsOnHeapBasicBlockProfilerData(o, isolate));
  {
    Tagged<Object> block_ids__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, block_ids__value);
    CHECK(IsByteArray(block_ids__value));
  }
  {
    Tagged<Object> counts__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, counts__value);
    CHECK(IsByteArray(counts__value));
  }
  {
    Tagged<Object> branches__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, branches__value);
    CHECK(IsByteArray(branches__value));
  }
  {
    Tagged<Object> name__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, name__value);
    CHECK(IsString(name__value));
  }
  {
    Tagged<Object> schedule__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, schedule__value);
    CHECK(IsString(schedule__value));
  }
  {
    Tagged<Object> code__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, code__value);
    CHECK(IsString(code__value));
  }
  {
    Tagged<Object> hash__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, hash__value);
    CHECK(IsSmi(hash__value));
  }
}
void TorqueGeneratedClassVerifiers::JSSynchronizationPrimitiveVerify(Tagged<JSSynchronizationPrimitive> o, Isolate* isolate) {
  o->AlwaysSharedSpaceJSObjectVerify(isolate);
  CHECK(IsJSSynchronizationPrimitive(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSAtomicsMutexVerify(Tagged<JSAtomicsMutex> o, Isolate* isolate) {
  o->JSSynchronizationPrimitiveVerify(isolate);
  CHECK(IsJSAtomicsMutex(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSAtomicsConditionVerify(Tagged<JSAtomicsCondition> o, Isolate* isolate) {
  o->JSSynchronizationPrimitiveVerify(isolate);
  CHECK(IsJSAtomicsCondition(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSCustomElementsObjectVerify(Tagged<JSCustomElementsObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSCustomElementsObject(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSSpecialObjectVerify(Tagged<JSSpecialObject> o, Isolate* isolate) {
  o->JSCustomElementsObjectVerify(isolate);
  CHECK(IsJSSpecialObject(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSModuleNamespaceVerify(Tagged<JSModuleNamespace> o, Isolate* isolate) {
  o->JSSpecialObjectVerify(isolate);
  CHECK(IsJSModuleNamespace(o, isolate));
  {
    Tagged<Object> module__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, module__value);
    CHECK(IsModule(module__value));
  }
}
void TorqueGeneratedClassVerifiers::JSDeferredModuleNamespaceVerify(Tagged<JSDeferredModuleNamespace> o, Isolate* isolate) {
  o->JSModuleNamespaceVerify(isolate);
  CHECK(IsJSDeferredModuleNamespace(o, isolate));
}
void TorqueGeneratedClassVerifiers::ContextVerify(Tagged<Context> o, Isolate* isolate) {
  CHECK(IsContext(o, isolate));
  {
    Tagged<Object> length__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, length__value);
    CHECK(IsSmi(length__value));
  }
  intptr_t elements__offset, elements__length;
  std::tie(std::ignore, elements__offset, elements__length) = TqRuntimeFieldSliceContextElements(o);
  CHECK_EQ(elements__offset, static_cast<int>(elements__offset));
  CHECK_EQ(elements__length, static_cast<int>(elements__length));
  for (int i = 0; i < static_cast<int>(elements__length); ++i) {
    Tagged<Object> elements__value = TaggedField<Object>::load(o, static_cast<int>(elements__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, elements__value);
  }
}
void TorqueGeneratedClassVerifiers::ScopeInfoVerify(Tagged<ScopeInfo> o, Isolate* isolate) {
  CHECK(IsScopeInfo(o, isolate));
  {
    Tagged<Object> parameter_count__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, parameter_count__value);
    CHECK(IsSmi(parameter_count__value));
  }
  {
    Tagged<Object> context_local_count__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, context_local_count__value);
    CHECK(IsSmi(context_local_count__value));
  }
  {
    Tagged<Object> start__value = TaggedField<Object>::load(o, 16 + 0);
    Object::VerifyPointer(isolate, start__value);
    CHECK(IsSmi(start__value));
    Tagged<Object> end__value = TaggedField<Object>::load(o, 16 + 4);
    Object::VerifyPointer(isolate, end__value);
    CHECK(IsSmi(end__value));
  }
  intptr_t module_variable_count__offset, module_variable_count__length;
  std::tie(std::ignore, module_variable_count__offset, module_variable_count__length) = TqRuntimeFieldSliceScopeInfoModuleVariableCount(o);
  CHECK_EQ(module_variable_count__offset, static_cast<int>(module_variable_count__offset));
  CHECK_EQ(module_variable_count__length, static_cast<int>(module_variable_count__length));
  for (int i = 0; i < static_cast<int>(module_variable_count__length); ++i) {
    Tagged<Object> module_variable_count__value = TaggedField<Object>::load(o, static_cast<int>(module_variable_count__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, module_variable_count__value);
    CHECK(IsSmi(module_variable_count__value));
  }
  intptr_t context_local_names__offset, context_local_names__length;
  std::tie(std::ignore, context_local_names__offset, context_local_names__length) = TqRuntimeFieldSliceScopeInfoContextLocalNames(o);
  CHECK_EQ(context_local_names__offset, static_cast<int>(context_local_names__offset));
  CHECK_EQ(context_local_names__length, static_cast<int>(context_local_names__length));
  for (int i = 0; i < static_cast<int>(context_local_names__length); ++i) {
    Tagged<Object> context_local_names__value = TaggedField<Object>::load(o, static_cast<int>(context_local_names__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, context_local_names__value);
    CHECK(IsString(context_local_names__value));
  }
  intptr_t context_local_names_hashtable__offset, context_local_names_hashtable__length;
  std::tie(std::ignore, context_local_names_hashtable__offset, context_local_names_hashtable__length) = TqRuntimeFieldSliceScopeInfoContextLocalNamesHashtable(o);
  CHECK_EQ(context_local_names_hashtable__offset, static_cast<int>(context_local_names_hashtable__offset));
  CHECK_EQ(context_local_names_hashtable__length, static_cast<int>(context_local_names_hashtable__length));
  for (int i = 0; i < static_cast<int>(context_local_names_hashtable__length); ++i) {
    Tagged<Object> context_local_names_hashtable__value = TaggedField<Object>::load(o, static_cast<int>(context_local_names_hashtable__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, context_local_names_hashtable__value);
    CHECK(IsNameToIndexHashTable(context_local_names_hashtable__value));
  }
  intptr_t context_local_infos__offset, context_local_infos__length;
  std::tie(std::ignore, context_local_infos__offset, context_local_infos__length) = TqRuntimeFieldSliceScopeInfoContextLocalInfos(o);
  CHECK_EQ(context_local_infos__offset, static_cast<int>(context_local_infos__offset));
  CHECK_EQ(context_local_infos__length, static_cast<int>(context_local_infos__length));
  for (int i = 0; i < static_cast<int>(context_local_infos__length); ++i) {
    Tagged<Object> context_local_infos__value = TaggedField<Object>::load(o, static_cast<int>(context_local_infos__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, context_local_infos__value);
    CHECK(IsSmi(context_local_infos__value));
  }
  intptr_t saved_class_variable_info__offset, saved_class_variable_info__length;
  std::tie(std::ignore, saved_class_variable_info__offset, saved_class_variable_info__length) = TqRuntimeFieldSliceScopeInfoSavedClassVariableInfo(o);
  CHECK_EQ(saved_class_variable_info__offset, static_cast<int>(saved_class_variable_info__offset));
  CHECK_EQ(saved_class_variable_info__length, static_cast<int>(saved_class_variable_info__length));
  for (int i = 0; i < static_cast<int>(saved_class_variable_info__length); ++i) {
    Tagged<Object> saved_class_variable_info__value = TaggedField<Object>::load(o, static_cast<int>(saved_class_variable_info__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, saved_class_variable_info__value);
    CHECK(IsName(saved_class_variable_info__value) || IsSmi(saved_class_variable_info__value));
  }
  intptr_t function_variable_info__offset, function_variable_info__length;
  std::tie(std::ignore, function_variable_info__offset, function_variable_info__length) = TqRuntimeFieldSliceScopeInfoFunctionVariableInfo(o);
  CHECK_EQ(function_variable_info__offset, static_cast<int>(function_variable_info__offset));
  CHECK_EQ(function_variable_info__length, static_cast<int>(function_variable_info__length));
  for (int i = 0; i < static_cast<int>(function_variable_info__length); ++i) {
    Tagged<Object> name__value = TaggedField<Object>::load(o, static_cast<int>(function_variable_info__offset) + 0 + i * 8);
    Object::VerifyPointer(isolate, name__value);
    CHECK(IsString(name__value) || IsZero(name__value));
    Tagged<Object> context_or_stack_slot_index__value = TaggedField<Object>::load(o, static_cast<int>(function_variable_info__offset) + 4 + i * 8);
    Object::VerifyPointer(isolate, context_or_stack_slot_index__value);
    CHECK(IsSmi(context_or_stack_slot_index__value));
  }
  intptr_t inferred_function_name__offset, inferred_function_name__length;
  std::tie(std::ignore, inferred_function_name__offset, inferred_function_name__length) = TqRuntimeFieldSliceScopeInfoInferredFunctionName(o);
  CHECK_EQ(inferred_function_name__offset, static_cast<int>(inferred_function_name__offset));
  CHECK_EQ(inferred_function_name__length, static_cast<int>(inferred_function_name__length));
  for (int i = 0; i < static_cast<int>(inferred_function_name__length); ++i) {
    Tagged<Object> inferred_function_name__value = TaggedField<Object>::load(o, static_cast<int>(inferred_function_name__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, inferred_function_name__value);
    CHECK(IsUndefined(inferred_function_name__value) || IsString(inferred_function_name__value));
  }
  intptr_t outer_scope_info__offset, outer_scope_info__length;
  std::tie(std::ignore, outer_scope_info__offset, outer_scope_info__length) = TqRuntimeFieldSliceScopeInfoOuterScopeInfo(o);
  CHECK_EQ(outer_scope_info__offset, static_cast<int>(outer_scope_info__offset));
  CHECK_EQ(outer_scope_info__length, static_cast<int>(outer_scope_info__length));
  for (int i = 0; i < static_cast<int>(outer_scope_info__length); ++i) {
    Tagged<Object> outer_scope_info__value = TaggedField<Object>::load(o, static_cast<int>(outer_scope_info__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, outer_scope_info__value);
    CHECK(IsScopeInfo(outer_scope_info__value));
  }
  intptr_t module_info__offset, module_info__length;
  std::tie(std::ignore, module_info__offset, module_info__length) = TqRuntimeFieldSliceScopeInfoModuleInfo(o);
  CHECK_EQ(module_info__offset, static_cast<int>(module_info__offset));
  CHECK_EQ(module_info__length, static_cast<int>(module_info__length));
  for (int i = 0; i < static_cast<int>(module_info__length); ++i) {
    Tagged<Object> module_info__value = TaggedField<Object>::load(o, static_cast<int>(module_info__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, module_info__value);
    CHECK(IsSourceTextModuleInfo(module_info__value));
  }
  intptr_t module_variables__offset, module_variables__length;
  std::tie(std::ignore, module_variables__offset, module_variables__length) = TqRuntimeFieldSliceScopeInfoModuleVariables(o);
  CHECK_EQ(module_variables__offset, static_cast<int>(module_variables__offset));
  CHECK_EQ(module_variables__length, static_cast<int>(module_variables__length));
  for (int i = 0; i < static_cast<int>(module_variables__length); ++i) {
    Tagged<Object> name__value = TaggedField<Object>::load(o, static_cast<int>(module_variables__offset) + 0 + i * 12);
    Object::VerifyPointer(isolate, name__value);
    CHECK(IsString(name__value));
    Tagged<Object> index__value = TaggedField<Object>::load(o, static_cast<int>(module_variables__offset) + 4 + i * 12);
    Object::VerifyPointer(isolate, index__value);
    CHECK(IsSmi(index__value));
    Tagged<Object> properties__value = TaggedField<Object>::load(o, static_cast<int>(module_variables__offset) + 8 + i * 12);
    Object::VerifyPointer(isolate, properties__value);
    CHECK(IsSmi(properties__value));
  }
  intptr_t dependent_code__offset, dependent_code__length;
  std::tie(std::ignore, dependent_code__offset, dependent_code__length) = TqRuntimeFieldSliceScopeInfoDependentCode(o);
  CHECK_EQ(dependent_code__offset, static_cast<int>(dependent_code__offset));
  CHECK_EQ(dependent_code__length, static_cast<int>(dependent_code__length));
  for (int i = 0; i < static_cast<int>(dependent_code__length); ++i) {
    Tagged<Object> dependent_code__value = TaggedField<Object>::load(o, static_cast<int>(dependent_code__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, dependent_code__value);
    CHECK(IsDependentCode(dependent_code__value));
  }
  intptr_t unused_parameter_bits__offset, unused_parameter_bits__length;
  std::tie(std::ignore, unused_parameter_bits__offset, unused_parameter_bits__length) = TqRuntimeFieldSliceScopeInfoUnusedParameterBits(o);
  CHECK_EQ(unused_parameter_bits__offset, static_cast<int>(unused_parameter_bits__offset));
  CHECK_EQ(unused_parameter_bits__length, static_cast<int>(unused_parameter_bits__length));
  for (int i = 0; i < static_cast<int>(unused_parameter_bits__length); ++i) {
    Tagged<Object> unused_parameter_bits__value = TaggedField<Object>::load(o, static_cast<int>(unused_parameter_bits__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, unused_parameter_bits__value);
    CHECK(IsSmi(unused_parameter_bits__value));
  }
}
void TorqueGeneratedClassVerifiers::JSProxyVerify(Tagged<JSProxy> o, Isolate* isolate) {
  o->JSReceiverVerify(isolate);
  CHECK(IsJSProxy(o, isolate));
  {
    Tagged<Object> target__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, target__value);
    CHECK(IsJSReceiver(target__value) || IsNull(target__value));
  }
  {
    Tagged<Object> handler__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, handler__value);
    CHECK(IsJSReceiver(handler__value) || IsNull(handler__value));
  }
}
void TorqueGeneratedClassVerifiers::EmbedderDataArrayVerify(Tagged<EmbedderDataArray> o, Isolate* isolate) {
  CHECK(IsEmbedderDataArray(o, isolate));
  {
    Tagged<Object> length__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, length__value);
    CHECK(IsSmi(length__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTemporalDurationVerify(Tagged<JSTemporalDuration> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSTemporalDuration(o, isolate));
  {
    Tagged<Object> duration__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, duration__value);
    CHECK(IsForeign(duration__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTemporalInstantVerify(Tagged<JSTemporalInstant> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSTemporalInstant(o, isolate));
  {
    Tagged<Object> instant__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, instant__value);
    CHECK(IsForeign(instant__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTemporalPlainDateTimeVerify(Tagged<JSTemporalPlainDateTime> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSTemporalPlainDateTime(o, isolate));
  {
    Tagged<Object> date_time__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, date_time__value);
    CHECK(IsForeign(date_time__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTemporalPlainDateVerify(Tagged<JSTemporalPlainDate> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSTemporalPlainDate(o, isolate));
  {
    Tagged<Object> date__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, date__value);
    CHECK(IsForeign(date__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTemporalPlainMonthDayVerify(Tagged<JSTemporalPlainMonthDay> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSTemporalPlainMonthDay(o, isolate));
  {
    Tagged<Object> month_day__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, month_day__value);
    CHECK(IsForeign(month_day__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTemporalPlainTimeVerify(Tagged<JSTemporalPlainTime> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSTemporalPlainTime(o, isolate));
  {
    Tagged<Object> time__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, time__value);
    CHECK(IsForeign(time__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTemporalPlainYearMonthVerify(Tagged<JSTemporalPlainYearMonth> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSTemporalPlainYearMonth(o, isolate));
  {
    Tagged<Object> year_month__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, year_month__value);
    CHECK(IsForeign(year_month__value));
  }
}
void TorqueGeneratedClassVerifiers::JSTemporalZonedDateTimeVerify(Tagged<JSTemporalZonedDateTime> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSTemporalZonedDateTime(o, isolate));
  {
    Tagged<Object> zoned_date_time__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, zoned_date_time__value);
    CHECK(IsForeign(zoned_date_time__value));
  }
}
void TorqueGeneratedClassVerifiers::JSRegExpStringIteratorVerify(Tagged<JSRegExpStringIterator> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSRegExpStringIterator(o, isolate));
  {
    Tagged<Object> iterating_reg_exp__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, iterating_reg_exp__value);
    CHECK(IsJSReceiver(iterating_reg_exp__value));
  }
  {
    Tagged<Object> iterated_string__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, iterated_string__value);
    CHECK(IsString(iterated_string__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::JSSharedArrayVerify(Tagged<JSSharedArray> o, Isolate* isolate) {
  o->AlwaysSharedSpaceJSObjectVerify(isolate);
  CHECK(IsJSSharedArray(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSShadowRealmVerify(Tagged<JSShadowRealm> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSShadowRealm(o, isolate));
  {
    Tagged<Object> native_context__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, native_context__value);
    CHECK(IsNativeContext(native_context__value));
  }
}
void TorqueGeneratedClassVerifiers::JSDateTimeFormatVerify(Tagged<JSDateTimeFormat> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSDateTimeFormat(o, isolate));
  {
    Tagged<Object> locale__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, locale__value);
    CHECK(IsString(locale__value));
  }
  {
    Tagged<Object> icu_locale__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, icu_locale__value);
    CHECK(IsForeign(icu_locale__value));
  }
  {
    Tagged<Object> icu_simple_date_format__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, icu_simple_date_format__value);
    CHECK(IsForeign(icu_simple_date_format__value));
  }
  {
    Tagged<Object> icu_date_interval_format__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, icu_date_interval_format__value);
    CHECK(IsForeign(icu_date_interval_format__value));
  }
  {
    Tagged<Object> bound_format__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, bound_format__value);
    CHECK(IsJSFunction(bound_format__value) || IsUndefined(bound_format__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::MegaDomHandlerVerify(Tagged<MegaDomHandler> o, Isolate* isolate) {
  CHECK(IsMegaDomHandler(o, isolate));
  {
    Tagged<MaybeObject> accessor__value = TaggedField<MaybeObject>::load(o, 4);
    Object::VerifyMaybeObjectPointer(isolate, accessor__value);
    CHECK(accessor__value.IsCleared() || (!accessor__value.IsWeak() && IsHeapObject(accessor__value.GetHeapObjectOrSmi())) || (!accessor__value.IsWeak() && IsSmi(accessor__value.GetHeapObjectOrSmi())) || accessor__value.IsWeak());
  }
  {
    Tagged<MaybeObject> context__value = TaggedField<MaybeObject>::load(o, 8);
    Object::VerifyMaybeObjectPointer(isolate, context__value);
    CHECK(context__value.IsCleared() || (!context__value.IsWeak() && IsHeapObject(context__value.GetHeapObjectOrSmi())) || (!context__value.IsWeak() && IsSmi(context__value.GetHeapObjectOrSmi())) || context__value.IsWeak());
  }
}
void TorqueGeneratedClassVerifiers::CoverageInfoVerify(Tagged<CoverageInfo> o, Isolate* isolate) {
  CHECK(IsCoverageInfo(o, isolate));
  intptr_t slots__offset, slots__length;
  std::tie(std::ignore, slots__offset, slots__length) = TqRuntimeFieldSliceCoverageInfoSlots(o);
  CHECK_EQ(slots__offset, static_cast<int>(slots__offset));
  CHECK_EQ(slots__length, static_cast<int>(slots__length));
  for (int i = 0; i < static_cast<int>(slots__length); ++i) {
  }
}
void TorqueGeneratedClassVerifiers::JSDurationFormatVerify(Tagged<JSDurationFormat> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSDurationFormat(o, isolate));
  {
    Tagged<Object> style_flags__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, style_flags__value);
    CHECK(IsSmi(style_flags__value));
  }
  {
    Tagged<Object> display_flags__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, display_flags__value);
    CHECK(IsSmi(display_flags__value));
  }
  {
    Tagged<Object> icu_locale__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, icu_locale__value);
    CHECK(IsForeign(icu_locale__value));
  }
  {
    Tagged<Object> icu_number_formatter__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, icu_number_formatter__value);
    CHECK(IsForeign(icu_number_formatter__value));
  }
}
void TorqueGeneratedClassVerifiers::JSSegmentIteratorVerify(Tagged<JSSegmentIterator> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSSegmentIterator(o, isolate));
  {
    Tagged<Object> icu_iterator_with_text__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, icu_iterator_with_text__value);
    CHECK(IsForeign(icu_iterator_with_text__value));
  }
  {
    Tagged<Object> raw_string__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, raw_string__value);
    CHECK(IsString(raw_string__value));
  }
  {
    Tagged<Object> flags__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, flags__value);
    CHECK(IsSmi(flags__value));
  }
}
void TorqueGeneratedClassVerifiers::JSSegmentDataObjectVerify(Tagged<JSSegmentDataObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSSegmentDataObject(o, isolate));
  {
    Tagged<Object> segment__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, segment__value);
    CHECK(IsString(segment__value));
  }
  {
    Tagged<Object> index__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, index__value);
    CHECK(IsSmi(index__value) || IsHeapNumber(index__value));
  }
  {
    Tagged<Object> input__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, input__value);
    CHECK(IsString(input__value));
  }
}
void TorqueGeneratedClassVerifiers::JSSegmentDataObjectWithIsWordLikeVerify(Tagged<JSSegmentDataObjectWithIsWordLike> o, Isolate* isolate) {
  o->JSSegmentDataObjectVerify(isolate);
  CHECK(IsJSSegmentDataObjectWithIsWordLike(o, isolate));
  {
    Tagged<Object> is_word_like__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, is_word_like__value);
    CHECK(IsBoolean(is_word_like__value));
  }
}
void TorqueGeneratedClassVerifiers::JSCollectionVerify(Tagged<JSCollection> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSCollection(o, isolate));
  {
    Tagged<Object> table__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, table__value);
  }
}
void TorqueGeneratedClassVerifiers::JSSetVerify(Tagged<JSSet> o, Isolate* isolate) {
  o->JSCollectionVerify(isolate);
  CHECK(IsJSSet(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSMapVerify(Tagged<JSMap> o, Isolate* isolate) {
  o->JSCollectionVerify(isolate);
  CHECK(IsJSMap(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSWeakCollectionVerify(Tagged<JSWeakCollection> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSWeakCollection(o, isolate));
  {
    Tagged<Object> table__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, table__value);
  }
}
void TorqueGeneratedClassVerifiers::JSWeakSetVerify(Tagged<JSWeakSet> o, Isolate* isolate) {
  o->JSWeakCollectionVerify(isolate);
  CHECK(IsJSWeakSet(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSWeakMapVerify(Tagged<JSWeakMap> o, Isolate* isolate) {
  o->JSWeakCollectionVerify(isolate);
  CHECK(IsJSWeakMap(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSExternalObjectVerify(Tagged<JSExternalObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSExternalObject(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSGlobalProxyVerify(Tagged<JSGlobalProxy> o, Isolate* isolate) {
  o->JSSpecialObjectVerify(isolate);
  CHECK(IsJSGlobalProxy(o, isolate));
}
void TorqueGeneratedClassVerifiers::JSGlobalObjectVerify(Tagged<JSGlobalObject> o, Isolate* isolate) {
  o->JSSpecialObjectVerify(isolate);
  CHECK(IsJSGlobalObject(o, isolate));
  {
    Tagged<Object> global_proxy__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, global_proxy__value);
    CHECK(IsJSGlobalProxy(global_proxy__value));
  }
  {
    Tagged<Object> global_proxy_for_api__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, global_proxy_for_api__value);
    CHECK(IsJSGlobalProxy(global_proxy_for_api__value));
  }
}
void TorqueGeneratedClassVerifiers::JSPrimitiveWrapperVerify(Tagged<JSPrimitiveWrapper> o, Isolate* isolate) {
  o->JSCustomElementsObjectVerify(isolate);
  CHECK(IsJSPrimitiveWrapper(o, isolate));
  {
    Tagged<Object> value__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, value__value);
    CHECK(IsJSReceiver(value__value) || IsBigInt(value__value) || IsUndefined(value__value) || IsSmi(value__value) || IsHeapNumber(value__value) || IsString(value__value) || IsSymbol(value__value) || IsBoolean(value__value) || IsNull(value__value));
  }
}
void TorqueGeneratedClassVerifiers::JSMessageObjectVerify(Tagged<JSMessageObject> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSMessageObject(o, isolate));
  {
    Tagged<Object> message_type__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, message_type__value);
    CHECK(IsSmi(message_type__value));
  }
  {
    Tagged<Object> argument__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, argument__value);
  }
  {
    Tagged<Object> script__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, script__value);
    CHECK(IsScript(script__value));
  }
  {
    Tagged<Object> stack_trace__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, stack_trace__value);
    CHECK(IsTheHole(stack_trace__value) || IsStackTraceInfo(stack_trace__value));
  }
  {
    Tagged<Object> shared_info__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, shared_info__value);
    CHECK(IsSmi(shared_info__value) || IsSharedFunctionInfo(shared_info__value));
  }
  {
    Tagged<Object> bytecode_offset__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, bytecode_offset__value);
    CHECK(IsSmi(bytecode_offset__value));
  }
  {
    Tagged<Object> start_position__value = TaggedField<Object>::load(o, 36);
    Object::VerifyPointer(isolate, start_position__value);
    CHECK(IsSmi(start_position__value));
  }
  {
    Tagged<Object> end_position__value = TaggedField<Object>::load(o, 40);
    Object::VerifyPointer(isolate, end_position__value);
    CHECK(IsSmi(end_position__value));
  }
  {
    Tagged<Object> error_level__value = TaggedField<Object>::load(o, 44);
    Object::VerifyPointer(isolate, error_level__value);
    CHECK(IsSmi(error_level__value));
  }
}
void TorqueGeneratedClassVerifiers::JSDateVerify(Tagged<JSDate> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSDate(o, isolate));
  {
    Tagged<Object> year__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, year__value);
    CHECK(IsSmi(year__value) || IsNaN(year__value));
  }
  {
    Tagged<Object> month__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, month__value);
    CHECK(IsSmi(month__value) || IsNaN(month__value));
  }
  {
    Tagged<Object> day__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, day__value);
    CHECK(IsSmi(day__value) || IsNaN(day__value));
  }
  {
    Tagged<Object> weekday__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, weekday__value);
    CHECK(IsSmi(weekday__value) || IsNaN(weekday__value));
  }
  {
    Tagged<Object> hour__value = TaggedField<Object>::load(o, 36);
    Object::VerifyPointer(isolate, hour__value);
    CHECK(IsSmi(hour__value) || IsNaN(hour__value));
  }
  {
    Tagged<Object> min__value = TaggedField<Object>::load(o, 40);
    Object::VerifyPointer(isolate, min__value);
    CHECK(IsSmi(min__value) || IsNaN(min__value));
  }
  {
    Tagged<Object> sec__value = TaggedField<Object>::load(o, 44);
    Object::VerifyPointer(isolate, sec__value);
    CHECK(IsSmi(sec__value) || IsNaN(sec__value));
  }
  {
    Tagged<Object> cache_stamp__value = TaggedField<Object>::load(o, 48);
    Object::VerifyPointer(isolate, cache_stamp__value);
    CHECK(IsSmi(cache_stamp__value) || IsNaN(cache_stamp__value));
  }
}
void TorqueGeneratedClassVerifiers::JSAsyncFromSyncIteratorVerify(Tagged<JSAsyncFromSyncIterator> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSAsyncFromSyncIterator(o, isolate));
  {
    Tagged<Object> sync_iterator__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, sync_iterator__value);
    CHECK(IsJSReceiver(sync_iterator__value));
  }
  {
    Tagged<Object> next__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, next__value);
  }
}
void TorqueGeneratedClassVerifiers::JSStringIteratorVerify(Tagged<JSStringIterator> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSStringIterator(o, isolate));
  {
    Tagged<Object> string__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, string__value);
    CHECK(IsString(string__value));
  }
  {
    Tagged<Object> index__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, index__value);
    CHECK(IsSmi(index__value));
  }
}
void TorqueGeneratedClassVerifiers::JSValidIteratorWrapperVerify(Tagged<JSValidIteratorWrapper> o, Isolate* isolate) {
  o->JSObjectVerify(isolate);
  CHECK(IsJSValidIteratorWrapper(o, isolate));
  {
    Tagged<Object> object__value = TaggedField<Object>::load(o, 12 + 0);
    Object::VerifyPointer(isolate, object__value);
    CHECK(IsJSReceiver(object__value));
    Tagged<Object> next__value = TaggedField<Object>::load(o, 12 + 4);
    Object::VerifyPointer(isolate, next__value);
    CHECK(IsJSReceiver(next__value) || IsBigInt(next__value) || IsUndefined(next__value) || IsSmi(next__value) || IsHeapNumber(next__value) || IsString(next__value) || IsSymbol(next__value) || IsBoolean(next__value) || IsNull(next__value));
  }
}
void TorqueGeneratedClassVerifiers::TurboshaftTypeVerify(Tagged<TurboshaftType> o, Isolate* isolate) {
  CHECK(IsTurboshaftType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftWord32TypeVerify(Tagged<TurboshaftWord32Type> o, Isolate* isolate) {
  o->TurboshaftTypeVerify(isolate);
  CHECK(IsTurboshaftWord32Type(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftWord32RangeTypeVerify(Tagged<TurboshaftWord32RangeType> o, Isolate* isolate) {
  o->TurboshaftWord32TypeVerify(isolate);
  CHECK(IsTurboshaftWord32RangeType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftWord32SetTypeVerify(Tagged<TurboshaftWord32SetType> o, Isolate* isolate) {
  o->TurboshaftWord32TypeVerify(isolate);
  CHECK(IsTurboshaftWord32SetType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftWord64TypeVerify(Tagged<TurboshaftWord64Type> o, Isolate* isolate) {
  o->TurboshaftTypeVerify(isolate);
  CHECK(IsTurboshaftWord64Type(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftWord64RangeTypeVerify(Tagged<TurboshaftWord64RangeType> o, Isolate* isolate) {
  o->TurboshaftWord64TypeVerify(isolate);
  CHECK(IsTurboshaftWord64RangeType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftWord64SetTypeVerify(Tagged<TurboshaftWord64SetType> o, Isolate* isolate) {
  o->TurboshaftWord64TypeVerify(isolate);
  CHECK(IsTurboshaftWord64SetType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftFloat64TypeVerify(Tagged<TurboshaftFloat64Type> o, Isolate* isolate) {
  o->TurboshaftTypeVerify(isolate);
  CHECK(IsTurboshaftFloat64Type(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftFloat64RangeTypeVerify(Tagged<TurboshaftFloat64RangeType> o, Isolate* isolate) {
  o->TurboshaftFloat64TypeVerify(isolate);
  CHECK(IsTurboshaftFloat64RangeType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurboshaftFloat64SetTypeVerify(Tagged<TurboshaftFloat64SetType> o, Isolate* isolate) {
  o->TurboshaftFloat64TypeVerify(isolate);
  CHECK(IsTurboshaftFloat64SetType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TemplateInfoVerify(Tagged<TemplateInfo> o, Isolate* isolate) {
  CHECK(IsTemplateInfo(o, isolate));
  {
    Tagged<Object> template_info_flags__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, template_info_flags__value);
    CHECK(IsSmi(template_info_flags__value));
  }
}
void TorqueGeneratedClassVerifiers::TemplateInfoWithPropertiesVerify(Tagged<TemplateInfoWithProperties> o, Isolate* isolate) {
  o->TemplateInfoVerify(isolate);
  CHECK(IsTemplateInfoWithProperties(o, isolate));
  {
    Tagged<Object> number_of_properties__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, number_of_properties__value);
    CHECK(IsSmi(number_of_properties__value));
  }
  {
    Tagged<Object> property_list__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, property_list__value);
    CHECK(IsUndefined(property_list__value) || IsArrayList(property_list__value));
  }
  {
    Tagged<Object> property_accessors__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, property_accessors__value);
    CHECK(IsUndefined(property_accessors__value) || IsArrayList(property_accessors__value));
  }
}
void TorqueGeneratedClassVerifiers::FunctionTemplateInfoVerify(Tagged<FunctionTemplateInfo> o, Isolate* isolate) {
  o->TemplateInfoWithPropertiesVerify(isolate);
  CHECK(IsFunctionTemplateInfo(o, isolate));
  {
    Tagged<Object> class_name__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, class_name__value);
    CHECK(IsUndefined(class_name__value) || IsString(class_name__value));
  }
  {
    Tagged<Object> interface_name__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, interface_name__value);
    CHECK(IsUndefined(interface_name__value) || IsString(interface_name__value));
  }
  {
    Tagged<Object> signature__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, signature__value);
    CHECK(IsUndefined(signature__value) || IsFunctionTemplateInfo(signature__value));
  }
  {
    Tagged<Object> rare_data__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, rare_data__value);
    CHECK(IsUndefined(rare_data__value) || IsFunctionTemplateRareData(rare_data__value));
  }
  {
    Tagged<Object> shared_function_info__value = TaggedField<Object>::load(o, 36);
    Object::VerifyPointer(isolate, shared_function_info__value);
    CHECK(IsUndefined(shared_function_info__value) || IsSharedFunctionInfo(shared_function_info__value));
  }
  {
    Tagged<Object> cached_property_name__value = TaggedField<Object>::load(o, 40);
    Object::VerifyPointer(isolate, cached_property_name__value);
  }
  {
    Tagged<Object> callback_data__value = TaggedField<Object>::load(o, 44);
    Object::VerifyPointer(isolate, callback_data__value);
  }
}
void TorqueGeneratedClassVerifiers::ObjectTemplateInfoVerify(Tagged<ObjectTemplateInfo> o, Isolate* isolate) {
  o->TemplateInfoWithPropertiesVerify(isolate);
  CHECK(IsObjectTemplateInfo(o, isolate));
  {
    Tagged<Object> constructor__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, constructor__value);
    CHECK(IsUndefined(constructor__value) || IsFunctionTemplateInfo(constructor__value));
  }
  {
    Tagged<Object> data__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, data__value);
    CHECK(IsSmi(data__value));
  }
}
void TorqueGeneratedClassVerifiers::DictionaryTemplateInfoVerify(Tagged<DictionaryTemplateInfo> o, Isolate* isolate) {
  o->TemplateInfoVerify(isolate);
  CHECK(IsDictionaryTemplateInfo(o, isolate));
  {
    Tagged<Object> property_names__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, property_names__value);
    CHECK(IsFixedArray(property_names__value));
  }
}
void TorqueGeneratedClassVerifiers::TurbofanTypeVerify(Tagged<TurbofanType> o, Isolate* isolate) {
  CHECK(IsTurbofanType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurbofanBitsetTypeVerify(Tagged<TurbofanBitsetType> o, Isolate* isolate) {
  o->TurbofanTypeVerify(isolate);
  CHECK(IsTurbofanBitsetType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurbofanUnionTypeVerify(Tagged<TurbofanUnionType> o, Isolate* isolate) {
  o->TurbofanTypeVerify(isolate);
  CHECK(IsTurbofanUnionType(o, isolate));
  {
    Tagged<Object> type1__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, type1__value);
    CHECK(IsTurbofanType(type1__value));
  }
  {
    Tagged<Object> type2__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, type2__value);
    CHECK(IsTurbofanType(type2__value));
  }
}
void TorqueGeneratedClassVerifiers::TurbofanRangeTypeVerify(Tagged<TurbofanRangeType> o, Isolate* isolate) {
  o->TurbofanTypeVerify(isolate);
  CHECK(IsTurbofanRangeType(o, isolate));
}
void TorqueGeneratedClassVerifiers::TurbofanHeapConstantTypeVerify(Tagged<TurbofanHeapConstantType> o, Isolate* isolate) {
  o->TurbofanTypeVerify(isolate);
  CHECK(IsTurbofanHeapConstantType(o, isolate));
  {
    Tagged<Object> constant__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, constant__value);
    CHECK(IsHeapObject(constant__value));
  }
}
void TorqueGeneratedClassVerifiers::TurbofanOtherNumberConstantTypeVerify(Tagged<TurbofanOtherNumberConstantType> o, Isolate* isolate) {
  o->TurbofanTypeVerify(isolate);
  CHECK(IsTurbofanOtherNumberConstantType(o, isolate));
}
void TorqueGeneratedClassVerifiers::SortStateVerify(Tagged<SortState> o, Isolate* isolate) {
  CHECK(IsSortState(o, isolate));
  {
    Tagged<Object> receiver__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, receiver__value);
    CHECK(IsJSReceiver(receiver__value));
  }
  {
    Tagged<Object> initialReceiverMap__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, initialReceiverMap__value);
    CHECK(IsMap(initialReceiverMap__value));
  }
  {
    Tagged<Object> initialReceiverLength__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, initialReceiverLength__value);
    CHECK(IsSmi(initialReceiverLength__value) || IsHeapNumber(initialReceiverLength__value));
  }
  {
    Tagged<Object> userCmpFn__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, userCmpFn__value);
    CHECK(IsJSFunction(userCmpFn__value) || IsUndefined(userCmpFn__value) || IsJSBoundFunction(userCmpFn__value) || IsJSWrappedFunction(userCmpFn__value) || IsCallableJSProxy(userCmpFn__value) || IsCallableApiObject(userCmpFn__value));
  }
  {
    Tagged<Object> isResetToGeneric__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, isResetToGeneric__value);
    CHECK(IsBoolean(isResetToGeneric__value));
  }
  {
    Tagged<Object> minGallop__value = TaggedField<Object>::load(o, 24);
    Object::VerifyPointer(isolate, minGallop__value);
    CHECK(IsSmi(minGallop__value));
  }
  {
    Tagged<Object> pendingRunsSize__value = TaggedField<Object>::load(o, 28);
    Object::VerifyPointer(isolate, pendingRunsSize__value);
    CHECK(IsSmi(pendingRunsSize__value));
  }
  {
    Tagged<Object> pendingRuns__value = TaggedField<Object>::load(o, 32);
    Object::VerifyPointer(isolate, pendingRuns__value);
    CHECK(IsFixedArray(pendingRuns__value));
  }
  {
    Tagged<Object> workArray__value = TaggedField<Object>::load(o, 36);
    Object::VerifyPointer(isolate, workArray__value);
    CHECK(IsFixedArray(workArray__value));
  }
  {
    Tagged<Object> tempArray__value = TaggedField<Object>::load(o, 40);
    Object::VerifyPointer(isolate, tempArray__value);
    CHECK(IsFixedArray(tempArray__value));
  }
  {
    Tagged<Object> sortLength__value = TaggedField<Object>::load(o, 44);
    Object::VerifyPointer(isolate, sortLength__value);
    CHECK(IsSmi(sortLength__value));
  }
  {
    Tagged<Object> numberOfUndefined__value = TaggedField<Object>::load(o, 48);
    Object::VerifyPointer(isolate, numberOfUndefined__value);
    CHECK(IsSmi(numberOfUndefined__value));
  }
}
void TorqueGeneratedClassVerifiers::InternalClassVerify(Tagged<InternalClass> o, Isolate* isolate) {
  CHECK(IsInternalClass(o, isolate));
  {
    Tagged<Object> a__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, a__value);
    CHECK(IsSmi(a__value));
  }
  {
    Tagged<Object> b__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, b__value);
    CHECK(IsSmi(b__value) || IsHeapNumber(b__value));
  }
}
void TorqueGeneratedClassVerifiers::SmiPairVerify(Tagged<SmiPair> o, Isolate* isolate) {
  CHECK(IsSmiPair(o, isolate));
  {
    Tagged<Object> a__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, a__value);
    CHECK(IsSmi(a__value));
  }
  {
    Tagged<Object> b__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, b__value);
    CHECK(IsSmi(b__value));
  }
}
void TorqueGeneratedClassVerifiers::SmiBoxVerify(Tagged<SmiBox> o, Isolate* isolate) {
  CHECK(IsSmiBox(o, isolate));
  {
    Tagged<Object> value__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, value__value);
    CHECK(IsSmi(value__value));
  }
  {
    Tagged<Object> unrelated__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, unrelated__value);
    CHECK(IsSmi(unrelated__value));
  }
}
void TorqueGeneratedClassVerifiers::ExportedSubClassBaseVerify(Tagged<ExportedSubClassBase> o, Isolate* isolate) {
  CHECK(IsExportedSubClassBase(o, isolate));
  {
    Tagged<Object> a__value = TaggedField<Object>::load(o, 4);
    Object::VerifyPointer(isolate, a__value);
    CHECK(IsHeapObject(a__value));
  }
  {
    Tagged<Object> b__value = TaggedField<Object>::load(o, 8);
    Object::VerifyPointer(isolate, b__value);
    CHECK(IsHeapObject(b__value));
  }
}
void TorqueGeneratedClassVerifiers::ExportedSubClassVerify(Tagged<ExportedSubClass> o, Isolate* isolate) {
  o->ExportedSubClassBaseVerify(isolate);
  CHECK(IsExportedSubClass(o, isolate));
  {
    Tagged<Object> e_field__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, e_field__value);
    CHECK(IsSmi(e_field__value));
  }
}
void TorqueGeneratedClassVerifiers::AbstractInternalClassVerify(Tagged<AbstractInternalClass> o, Isolate* isolate) {
  CHECK(IsAbstractInternalClass(o, isolate));
}
void TorqueGeneratedClassVerifiers::AbstractInternalClassSubclass1Verify(Tagged<AbstractInternalClassSubclass1> o, Isolate* isolate) {
  o->AbstractInternalClassVerify(isolate);
  CHECK(IsAbstractInternalClassSubclass1(o, isolate));
}
void TorqueGeneratedClassVerifiers::AbstractInternalClassSubclass2Verify(Tagged<AbstractInternalClassSubclass2> o, Isolate* isolate) {
  o->AbstractInternalClassVerify(isolate);
  CHECK(IsAbstractInternalClassSubclass2(o, isolate));
}
void TorqueGeneratedClassVerifiers::InternalClassWithStructElementsVerify(Tagged<InternalClassWithStructElements> o, Isolate* isolate) {
  CHECK(IsInternalClassWithStructElements(o, isolate));
  {
    Tagged<Object> count__value = TaggedField<Object>::load(o, 12);
    Object::VerifyPointer(isolate, count__value);
    CHECK(IsSmi(count__value));
  }
  {
    Tagged<Object> data__value = TaggedField<Object>::load(o, 16);
    Object::VerifyPointer(isolate, data__value);
    CHECK(IsSmi(data__value));
  }
  {
    Tagged<Object> object__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, object__value);
  }
  intptr_t entries__offset, entries__length;
  std::tie(std::ignore, entries__offset, entries__length) = TqRuntimeFieldSliceInternalClassWithStructElementsEntries(o);
  CHECK_EQ(entries__offset, static_cast<int>(entries__offset));
  CHECK_EQ(entries__length, static_cast<int>(entries__length));
  for (int i = 0; i < static_cast<int>(entries__length); ++i) {
    Tagged<Object> entries__value = TaggedField<Object>::load(o, static_cast<int>(entries__offset) + i * kTaggedSize);
    Object::VerifyPointer(isolate, entries__value);
    CHECK(IsSmi(entries__value));
  }
  intptr_t more_entries__offset, more_entries__length;
  std::tie(std::ignore, more_entries__offset, more_entries__length) = TqRuntimeFieldSliceInternalClassWithStructElementsMoreEntries(o);
  CHECK_EQ(more_entries__offset, static_cast<int>(more_entries__offset));
  CHECK_EQ(more_entries__length, static_cast<int>(more_entries__length));
  for (int i = 0; i < static_cast<int>(more_entries__length); ++i) {
    Tagged<Object> a__value = TaggedField<Object>::load(o, static_cast<int>(more_entries__offset) + 0 + i * 8);
    Object::VerifyPointer(isolate, a__value);
    CHECK(IsSmi(a__value));
    Tagged<Object> b__value = TaggedField<Object>::load(o, static_cast<int>(more_entries__offset) + 4 + i * 8);
    Object::VerifyPointer(isolate, b__value);
    CHECK(IsSmi(b__value));
  }
}
void TorqueGeneratedClassVerifiers::ExportedSubClass2Verify(Tagged<ExportedSubClass2> o, Isolate* isolate) {
  o->ExportedSubClassBaseVerify(isolate);
  CHECK(IsExportedSubClass2(o, isolate));
  {
    Tagged<Object> z_field__value = TaggedField<Object>::load(o, 20);
    Object::VerifyPointer(isolate, z_field__value);
    CHECK(IsSmi(z_field__value));
  }
}
}  // namespace internal
}  // namespace v8

#include "src/objects/object-macros-undef.h"
#endif  // VERIFY_HEAP
