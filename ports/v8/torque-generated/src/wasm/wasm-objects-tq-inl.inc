// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
inline std::tuple<Tagged<Union<HeapObject, TaggedIndex>>, intptr_t, intptr_t> TqRuntimeFieldSliceWasmTypeInfoSupertypes(Tagged<WasmTypeInfo> p_o);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
inline intptr_t TqRuntimeFromConstexpr_intptr_constexpr_int31_0(int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
inline intptr_t TqRuntimeConvert_intptr_Smi_0(Tagged<Smi> p_s);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
inline intptr_t TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0(IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
inline std::tuple<Tagged<Union<HeapObject, TaggedIndex>>, intptr_t, intptr_t> TqRuntimeNewMutableSlice_Object_0(Tagged<Union<HeapObject, TaggedIndex>> p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
inline intptr_t TqRuntimeFromConstexpr_intptr_constexpr_intptr_0(intptr_t p_i);

#ifndef V8_INTERNAL_DEFINED_TqRuntimeFieldSliceWasmTypeInfoSupertypes
#define V8_INTERNAL_DEFINED_TqRuntimeFieldSliceWasmTypeInfoSupertypes

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
std::tuple<Tagged<Union<HeapObject, TaggedIndex>>, intptr_t, intptr_t> TqRuntimeFieldSliceWasmTypeInfoSupertypes(Tagged<WasmTypeInfo> p_o) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  Tagged<Smi> tmp1{}; USE(tmp1);
  intptr_t tmp2{}; USE(tmp2);
  intptr_t tmp3{}; USE(tmp3);
  Tagged<Union<HeapObject, TaggedIndex>> tmp4{}; USE(tmp4);
  intptr_t tmp5{}; USE(tmp5);
  intptr_t tmp6{}; USE(tmp6);
  goto block0;

  block0:
  tmp0 = TqRuntimeFromConstexpr_intptr_constexpr_int31_0(12);
  tmp1 = TaggedField<Tagged<Smi>>::load(UncheckedCast<HeapObject>(p_o), static_cast<int>(tmp0));
  tmp2 = TqRuntimeConvert_intptr_Smi_0(tmp1);
  tmp3 = TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0(IntegerLiteral(false, 0x10ull));
  std::tie(tmp4, tmp5, tmp6) = TqRuntimeNewMutableSlice_Object_0(p_o, tmp3, tmp2);
  goto block2;

  block2:
  return std::make_tuple(tmp4, tmp5, tmp6);
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeFieldSliceWasmTypeInfoSupertypes
#ifndef V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_int31_0
#define V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_int31_0

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
intptr_t TqRuntimeFromConstexpr_intptr_constexpr_int31_0(int31_t p_i) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  goto block0;

  block0:
  tmp0 = (CastToUnderlyingTypeIfEnum(p_i));
  goto block2;

  block2:
  return tmp0;
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_int31_0
#ifndef V8_INTERNAL_DEFINED_TqRuntimeConvert_intptr_Smi_0
#define V8_INTERNAL_DEFINED_TqRuntimeConvert_intptr_Smi_0

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=298&c=1
intptr_t TqRuntimeConvert_intptr_Smi_0(Tagged<Smi> p_s) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  goto block0;

  block0:
  tmp0 = TorqueRuntimeMacroShims::CodeStubAssembler::SmiUntag(p_s);
  goto block2;

  block2:
  return tmp0;
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeConvert_intptr_Smi_0
#ifndef V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0
#define V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
intptr_t TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0(IntegerLiteral p_i) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  goto block0;

  block0:
  tmp0 = TqRuntimeFromConstexpr_intptr_constexpr_intptr_0((TorqueRuntimeMacroShims::CodeStubAssembler::ConstexprIntegerLiteralToIntptr(p_i)));
  goto block2;

  block2:
  return tmp0;
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0
#ifndef V8_INTERNAL_DEFINED_TqRuntimeNewMutableSlice_Object_0
#define V8_INTERNAL_DEFINED_TqRuntimeNewMutableSlice_Object_0

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
std::tuple<Tagged<Union<HeapObject, TaggedIndex>>, intptr_t, intptr_t> TqRuntimeNewMutableSlice_Object_0(Tagged<Union<HeapObject, TaggedIndex>> p_object, intptr_t p_offset, intptr_t p_length) {
  DisallowGarbageCollection no_gc;
  Tagged<Union<HeapObject, TaggedIndex>> tmp0{}; USE(tmp0);
  intptr_t tmp1{}; USE(tmp1);
  intptr_t tmp2{}; USE(tmp2);
  goto block0;

  block0:
  std::tie(tmp0, tmp1, tmp2) = (std::make_tuple(p_object, p_offset, p_length));
  goto block2;

  block2:
  return std::make_tuple(tmp0, tmp1, tmp2);
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeNewMutableSlice_Object_0
#ifndef V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_intptr_0
#define V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_intptr_0

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
intptr_t TqRuntimeFromConstexpr_intptr_constexpr_intptr_0(intptr_t p_i) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  goto block0;

  block0:
  tmp0 = (CastToUnderlyingTypeIfEnum(p_i));
  goto block2;

  block2:
  return tmp0;
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_intptr_0
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
template<class D, class P>
Tagged<WasmModuleObject> TorqueGeneratedWasmInstanceObject<D, P>::module_object() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmInstanceObject::module_object(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
template<class D, class P>
Tagged<WasmModuleObject> TorqueGeneratedWasmInstanceObject<D, P>::module_object(PtrComprCageBase cage_base) const {
  Tagged<WasmModuleObject> value = TaggedField<Tagged<WasmModuleObject>>::load(cage_base, *this, kModuleObjectOffset);
  DCHECK(IsWasmModuleObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
template<class D, class P>
void TorqueGeneratedWasmInstanceObject<D, P>::set_module_object(Tagged<WasmModuleObject> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsWasmModuleObject(value)));
  WRITE_FIELD(*this, kModuleObjectOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kModuleObjectOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
template<class D, class P>
Tagged<JSObject> TorqueGeneratedWasmInstanceObject<D, P>::exports_object() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmInstanceObject::exports_object(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
template<class D, class P>
Tagged<JSObject> TorqueGeneratedWasmInstanceObject<D, P>::exports_object(PtrComprCageBase cage_base) const {
  Tagged<JSObject> value = TaggedField<Tagged<JSObject>>::load(cage_base, *this, kExportsObjectOffset);
  DCHECK(IsJSObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
template<class D, class P>
void TorqueGeneratedWasmInstanceObject<D, P>::set_exports_object(Tagged<JSObject> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSObject(value)));
  WRITE_FIELD(*this, kExportsObjectOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kExportsObjectOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
template<class D, class P>
Tagged<NativeContext> TorqueGeneratedWasmImportData<D, P>::native_context() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmImportData::native_context(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
template<class D, class P>
Tagged<NativeContext> TorqueGeneratedWasmImportData<D, P>::native_context(PtrComprCageBase cage_base) const {
  Tagged<NativeContext> value = TaggedField<Tagged<NativeContext>>::load(cage_base, *this, kNativeContextOffset);
  DCHECK(IsNativeContext(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
template<class D, class P>
void TorqueGeneratedWasmImportData<D, P>::set_native_context(Tagged<NativeContext> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsNativeContext(value)));
  WRITE_FIELD(*this, kNativeContextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kNativeContextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
template<class D, class P>
Tagged<Union<JSReceiver, Undefined>> TorqueGeneratedWasmImportData<D, P>::callable() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmImportData::callable(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
template<class D, class P>
Tagged<Union<JSReceiver, Undefined>> TorqueGeneratedWasmImportData<D, P>::callable(PtrComprCageBase cage_base) const {
  Tagged<Union<JSReceiver, Undefined>> value = TaggedField<Tagged<Union<JSReceiver, Undefined>>>::load(cage_base, *this, kCallableOffset);
  DCHECK(IsJSReceiver(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
template<class D, class P>
void TorqueGeneratedWasmImportData<D, P>::set_callable(Tagged<Union<JSReceiver, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kCallableOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCallableOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
template<class D, class P>
Tagged<Cell> TorqueGeneratedWasmImportData<D, P>::wrapper_budget() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmImportData::wrapper_budget(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
template<class D, class P>
Tagged<Cell> TorqueGeneratedWasmImportData<D, P>::wrapper_budget(PtrComprCageBase cage_base) const {
  Tagged<Cell> value = TaggedField<Tagged<Cell>>::load(cage_base, *this, kWrapperBudgetOffset);
  DCHECK(IsCell(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
template<class D, class P>
void TorqueGeneratedWasmImportData<D, P>::set_wrapper_budget(Tagged<Cell> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsCell(value)));
  WRITE_FIELD(*this, kWrapperBudgetOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kWrapperBudgetOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=64&c=3
template<class D, class P>
const wasm::CanonicalSig* TorqueGeneratedWasmImportData<D, P>::sig() const {
  const wasm::CanonicalSig* value = this->template ReadField<const wasm::CanonicalSig*>(kSigOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=64&c=3
template<class D, class P>
void TorqueGeneratedWasmImportData<D, P>::set_sig(const wasm::CanonicalSig* value) {
  this->template WriteField<const wasm::CanonicalSig*>(kSigOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=66&c=3
template<class D, class P>
uint32_t TorqueGeneratedWasmImportData<D, P>::bit_field() const {
  uint32_t value = this->template ReadField<uint32_t>(kBitFieldOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=66&c=3
template<class D, class P>
void TorqueGeneratedWasmImportData<D, P>::set_bit_field(uint32_t value) {
  this->template WriteField<uint32_t>(kBitFieldOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedWasmInternalFunction<D, P>::external() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmInternalFunction::external(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedWasmInternalFunction<D, P>::external(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFunction, Undefined>> value = TaggedField<Tagged<Union<JSFunction, Undefined>>>::load(cage_base, *this, kExternalOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
template<class D, class P>
void TorqueGeneratedWasmInternalFunction<D, P>::set_external(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kExternalOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kExternalOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=98&c=3
template<class D, class P>
int TorqueGeneratedWasmInternalFunction<D, P>::function_index() const {
  int value = TaggedField<Smi>::load(*this, kFunctionIndexOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=98&c=3
template<class D, class P>
void TorqueGeneratedWasmInternalFunction<D, P>::set_function_index(int value) {
  WRITE_FIELD(*this, kFunctionIndexOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=100&c=3
template<class D, class P>
uint32_t TorqueGeneratedWasmInternalFunction<D, P>::raw_call_target() const {
  uint32_t value = this->template ReadField<uint32_t>(kRawCallTargetOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=100&c=3
template<class D, class P>
void TorqueGeneratedWasmInternalFunction<D, P>::set_raw_call_target(uint32_t value) {
  this->template WriteField<uint32_t>(kRawCallTargetOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=103&c=3
template<class D, class P>
const wasm::CanonicalSig* TorqueGeneratedWasmInternalFunction<D, P>::sig() const {
  const wasm::CanonicalSig* value = this->template ReadField<const wasm::CanonicalSig*>(kSigOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=103&c=3
template<class D, class P>
void TorqueGeneratedWasmInternalFunction<D, P>::set_sig(const wasm::CanonicalSig* value) {
  this->template WriteField<const wasm::CanonicalSig*>(kSigOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
template<class D, class P>
Tagged<WasmFuncRef> TorqueGeneratedWasmFunctionData<D, P>::func_ref() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmFunctionData::func_ref(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
template<class D, class P>
Tagged<WasmFuncRef> TorqueGeneratedWasmFunctionData<D, P>::func_ref(PtrComprCageBase cage_base) const {
  Tagged<WasmFuncRef> value = TaggedField<Tagged<WasmFuncRef>>::load(cage_base, *this, kFuncRefOffset);
  DCHECK(IsWasmFuncRef(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
template<class D, class P>
void TorqueGeneratedWasmFunctionData<D, P>::set_func_ref(Tagged<WasmFuncRef> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsWasmFuncRef(value)));
  WRITE_FIELD(*this, kFuncRefOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kFuncRefOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=127&c=3
template<class D, class P>
int TorqueGeneratedWasmFunctionData<D, P>::js_promise_flags() const {
  int value = TaggedField<Smi>::load(*this, kJsPromiseFlagsOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=127&c=3
template<class D, class P>
void TorqueGeneratedWasmFunctionData<D, P>::set_js_promise_flags(int value) {
  WRITE_FIELD(*this, kJsPromiseFlagsOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=141&c=3
template<class D, class P>
int TorqueGeneratedWasmExportedFunctionData<D, P>::function_index() const {
  int value = TaggedField<Smi>::load(*this, kFunctionIndexOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=141&c=3
template<class D, class P>
void TorqueGeneratedWasmExportedFunctionData<D, P>::set_function_index(int value) {
  WRITE_FIELD(*this, kFunctionIndexOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
template<class D, class P>
Tagged<Cell> TorqueGeneratedWasmExportedFunctionData<D, P>::wrapper_budget() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmExportedFunctionData::wrapper_budget(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
template<class D, class P>
Tagged<Cell> TorqueGeneratedWasmExportedFunctionData<D, P>::wrapper_budget(PtrComprCageBase cage_base) const {
  Tagged<Cell> value = TaggedField<Tagged<Cell>>::load(cage_base, *this, kWrapperBudgetOffset);
  DCHECK(IsCell(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
template<class D, class P>
void TorqueGeneratedWasmExportedFunctionData<D, P>::set_wrapper_budget(Tagged<Cell> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsCell(value)));
  WRITE_FIELD(*this, kWrapperBudgetOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kWrapperBudgetOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=148&c=3
template<class D, class P>
int TorqueGeneratedWasmExportedFunctionData<D, P>::packed_args_size() const {
  int value = TaggedField<Smi>::load(*this, kPackedArgsSizeOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=148&c=3
template<class D, class P>
void TorqueGeneratedWasmExportedFunctionData<D, P>::set_packed_args_size(int value) {
  WRITE_FIELD(*this, kPackedArgsSizeOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedWasmCapiFunctionData<D, P>::embedder_data() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmCapiFunctionData::embedder_data(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedWasmCapiFunctionData<D, P>::embedder_data(PtrComprCageBase cage_base) const {
  Tagged<Foreign> value = TaggedField<Tagged<Foreign>>::load(cage_base, *this, kEmbedderDataOffset);
  DCHECK(IsForeign(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
template<class D, class P>
void TorqueGeneratedWasmCapiFunctionData<D, P>::set_embedder_data(Tagged<Foreign> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsForeign(value)));
  WRITE_FIELD(*this, kEmbedderDataOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kEmbedderDataOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=167&c=3
template<class D, class P>
int TorqueGeneratedWasmResumeData<D, P>::on_resume() const {
  int value = TaggedField<Smi>::load(*this, kOnResumeOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=167&c=3
template<class D, class P>
void TorqueGeneratedWasmResumeData<D, P>::set_on_resume(int value) {
  WRITE_FIELD(*this, kOnResumeOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
template<class D, class P>
Tagged<Union<JSPromise, Undefined>> TorqueGeneratedWasmSuspenderObject<D, P>::promise() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmSuspenderObject::promise(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
template<class D, class P>
Tagged<Union<JSPromise, Undefined>> TorqueGeneratedWasmSuspenderObject<D, P>::promise(PtrComprCageBase cage_base) const {
  Tagged<Union<JSPromise, Undefined>> value = TaggedField<Tagged<Union<JSPromise, Undefined>>>::load(cage_base, *this, kPromiseOffset);
  DCHECK(IsUndefined(value) || IsJSPromise(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
template<class D, class P>
void TorqueGeneratedWasmSuspenderObject<D, P>::set_promise(Tagged<Union<JSPromise, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsJSPromise(value)));
  WRITE_FIELD(*this, kPromiseOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPromiseOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
template<class D, class P>
Tagged<Union<JSObject, Undefined>> TorqueGeneratedWasmSuspenderObject<D, P>::resume() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmSuspenderObject::resume(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
template<class D, class P>
Tagged<Union<JSObject, Undefined>> TorqueGeneratedWasmSuspenderObject<D, P>::resume(PtrComprCageBase cage_base) const {
  Tagged<Union<JSObject, Undefined>> value = TaggedField<Tagged<Union<JSObject, Undefined>>>::load(cage_base, *this, kResumeOffset);
  DCHECK(IsJSObject(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
template<class D, class P>
void TorqueGeneratedWasmSuspenderObject<D, P>::set_resume(Tagged<Union<JSObject, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSObject(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kResumeOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kResumeOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
template<class D, class P>
Tagged<Union<JSObject, Undefined>> TorqueGeneratedWasmSuspenderObject<D, P>::reject() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmSuspenderObject::reject(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
template<class D, class P>
Tagged<Union<JSObject, Undefined>> TorqueGeneratedWasmSuspenderObject<D, P>::reject(PtrComprCageBase cage_base) const {
  Tagged<Union<JSObject, Undefined>> value = TaggedField<Tagged<Union<JSObject, Undefined>>>::load(cage_base, *this, kRejectOffset);
  DCHECK(IsJSObject(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
template<class D, class P>
void TorqueGeneratedWasmSuspenderObject<D, P>::set_reject(Tagged<Union<JSObject, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSObject(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kRejectOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kRejectOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
template<class D, class P>
Tagged<WasmStackObject> TorqueGeneratedWasmContinuationObject<D, P>::stack_obj() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmContinuationObject::stack_obj(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
template<class D, class P>
Tagged<WasmStackObject> TorqueGeneratedWasmContinuationObject<D, P>::stack_obj(PtrComprCageBase cage_base) const {
  Tagged<WasmStackObject> value = TaggedField<Tagged<WasmStackObject>>::load(cage_base, *this, kStackObjOffset);
  DCHECK(IsWasmStackObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
template<class D, class P>
void TorqueGeneratedWasmContinuationObject<D, P>::set_stack_obj(Tagged<WasmStackObject> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsWasmStackObject(value)));
  WRITE_FIELD(*this, kStackObjOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kStackObjOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=192&c=3
template<class D, class P>
int TorqueGeneratedWasmExceptionTag<D, P>::index() const {
  int value = TaggedField<Smi>::load(*this, kIndexOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=192&c=3
template<class D, class P>
void TorqueGeneratedWasmExceptionTag<D, P>::set_index(int value) {
  WRITE_FIELD(*this, kIndexOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
template<class D, class P>
Tagged<Managed<wasm::NativeModule>> TorqueGeneratedWasmModuleObject<D, P>::managed_native_module() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmModuleObject::managed_native_module(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
template<class D, class P>
Tagged<Managed<wasm::NativeModule>> TorqueGeneratedWasmModuleObject<D, P>::managed_native_module(PtrComprCageBase cage_base) const {
  Tagged<Managed<wasm::NativeModule>> value = TaggedField<Tagged<Managed<wasm::NativeModule>>>::load(cage_base, *this, kManagedNativeModuleOffset);
  DCHECK(IsForeign(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
template<class D, class P>
void TorqueGeneratedWasmModuleObject<D, P>::set_managed_native_module(Tagged<Managed<wasm::NativeModule>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsForeign(value)));
  WRITE_FIELD(*this, kManagedNativeModuleOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kManagedNativeModuleOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
template<class D, class P>
Tagged<Script> TorqueGeneratedWasmModuleObject<D, P>::script() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmModuleObject::script(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
template<class D, class P>
Tagged<Script> TorqueGeneratedWasmModuleObject<D, P>::script(PtrComprCageBase cage_base) const {
  Tagged<Script> value = TaggedField<Tagged<Script>>::load(cage_base, *this, kScriptOffset);
  DCHECK(IsScript(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
template<class D, class P>
void TorqueGeneratedWasmModuleObject<D, P>::set_script(Tagged<Script> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsScript(value)));
  WRITE_FIELD(*this, kScriptOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kScriptOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedWasmTableObject<D, P>::entries() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmTableObject::entries(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedWasmTableObject<D, P>::entries(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kEntriesOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
template<class D, class P>
void TorqueGeneratedWasmTableObject<D, P>::set_entries(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kEntriesOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kEntriesOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=215&c=3
template<class D, class P>
int TorqueGeneratedWasmTableObject<D, P>::current_length() const {
  int value = TaggedField<Smi>::load(*this, kCurrentLengthOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=215&c=3
template<class D, class P>
void TorqueGeneratedWasmTableObject<D, P>::set_current_length(int value) {
  WRITE_FIELD(*this, kCurrentLengthOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
template<class D, class P>
Tagged<Union<BigInt, HeapNumber, Smi, Undefined>> TorqueGeneratedWasmTableObject<D, P>::maximum_length() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmTableObject::maximum_length(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
template<class D, class P>
Tagged<Union<BigInt, HeapNumber, Smi, Undefined>> TorqueGeneratedWasmTableObject<D, P>::maximum_length(PtrComprCageBase cage_base) const {
  Tagged<Union<BigInt, HeapNumber, Smi, Undefined>> value = TaggedField<Tagged<Union<BigInt, HeapNumber, Smi, Undefined>>>::load(cage_base, *this, kMaximumLengthOffset);
  DCHECK(IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
template<class D, class P>
void TorqueGeneratedWasmTableObject<D, P>::set_maximum_length(Tagged<Union<BigInt, HeapNumber, Smi, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kMaximumLengthOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kMaximumLengthOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=224&c=3
template<class D, class P>
int TorqueGeneratedWasmTableObject<D, P>::raw_type() const {
  int value = TaggedField<Smi>::load(*this, kRawTypeOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=224&c=3
template<class D, class P>
void TorqueGeneratedWasmTableObject<D, P>::set_raw_type(int value) {
  WRITE_FIELD(*this, kRawTypeOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=231&c=3
template<class D, class P>
wasm::AddressType TorqueGeneratedWasmTableObject<D, P>::address_type() const {
  wasm::AddressType value = this->template ReadField<wasm::AddressType>(kAddressTypeOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=231&c=3
template<class D, class P>
void TorqueGeneratedWasmTableObject<D, P>::set_address_type(wasm::AddressType value) {
  this->template WriteField<wasm::AddressType>(kAddressTypeOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=233&c=3
template<class D, class P>
uint8_t TorqueGeneratedWasmTableObject<D, P>::padding_for_address_type_0() const {
  uint8_t value = this->template ReadField<uint8_t>(kPaddingForAddressType0Offset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=233&c=3
template<class D, class P>
void TorqueGeneratedWasmTableObject<D, P>::set_padding_for_address_type_0(uint8_t value) {
  this->template WriteField<uint8_t>(kPaddingForAddressType0Offset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=234&c=3
template<class D, class P>
uint16_t TorqueGeneratedWasmTableObject<D, P>::padding_for_address_type_1() const {
  uint16_t value = this->template ReadField<uint16_t>(kPaddingForAddressType1Offset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=234&c=3
template<class D, class P>
void TorqueGeneratedWasmTableObject<D, P>::set_padding_for_address_type_1(uint16_t value) {
  this->template WriteField<uint16_t>(kPaddingForAddressType1Offset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
template<class D, class P>
Tagged<Union<JSArrayBuffer, Undefined>> TorqueGeneratedWasmMemoryObject<D, P>::array_buffer() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmMemoryObject::array_buffer(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
template<class D, class P>
Tagged<Union<JSArrayBuffer, Undefined>> TorqueGeneratedWasmMemoryObject<D, P>::array_buffer(PtrComprCageBase cage_base) const {
  Tagged<Union<JSArrayBuffer, Undefined>> value = TaggedField<Tagged<Union<JSArrayBuffer, Undefined>>>::load(cage_base, *this, kArrayBufferOffset);
  DCHECK(IsUndefined(value) || IsJSArrayBuffer(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryObject<D, P>::set_array_buffer(Tagged<Union<JSArrayBuffer, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsJSArrayBuffer(value)));
  WRITE_FIELD(*this, kArrayBufferOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kArrayBufferOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
template<class D, class P>
Tagged<Managed<BackingStore>> TorqueGeneratedWasmMemoryObject<D, P>::managed_backing_store() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmMemoryObject::managed_backing_store(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
template<class D, class P>
Tagged<Managed<BackingStore>> TorqueGeneratedWasmMemoryObject<D, P>::managed_backing_store(PtrComprCageBase cage_base) const {
  Tagged<Managed<BackingStore>> value = TaggedField<Tagged<Managed<BackingStore>>>::load(cage_base, *this, kManagedBackingStoreOffset);
  DCHECK(IsForeign(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryObject<D, P>::set_managed_backing_store(Tagged<Managed<BackingStore>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsForeign(value)));
  WRITE_FIELD(*this, kManagedBackingStoreOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kManagedBackingStoreOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=248&c=3
template<class D, class P>
int TorqueGeneratedWasmMemoryObject<D, P>::maximum_pages() const {
  int value = TaggedField<Smi>::load(*this, kMaximumPagesOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=248&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryObject<D, P>::set_maximum_pages(int value) {
  WRITE_FIELD(*this, kMaximumPagesOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
template<class D, class P>
Tagged<WeakArrayList> TorqueGeneratedWasmMemoryObject<D, P>::instances() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmMemoryObject::instances(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
template<class D, class P>
Tagged<WeakArrayList> TorqueGeneratedWasmMemoryObject<D, P>::instances(PtrComprCageBase cage_base) const {
  Tagged<WeakArrayList> value = TaggedField<Tagged<WeakArrayList>>::load(cage_base, *this, kInstancesOffset);
  DCHECK(IsWeakArrayList(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryObject<D, P>::set_instances(Tagged<WeakArrayList> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsWeakArrayList(value)));
  WRITE_FIELD(*this, kInstancesOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInstancesOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=250&c=3
template<class D, class P>
wasm::AddressType TorqueGeneratedWasmMemoryObject<D, P>::address_type() const {
  wasm::AddressType value = this->template ReadField<wasm::AddressType>(kAddressTypeOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=250&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryObject<D, P>::set_address_type(wasm::AddressType value) {
  this->template WriteField<wasm::AddressType>(kAddressTypeOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=252&c=3
template<class D, class P>
uint8_t TorqueGeneratedWasmMemoryObject<D, P>::padding_for_flags_0() const {
  uint8_t value = this->template ReadField<uint8_t>(kPaddingForFlags0Offset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=252&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryObject<D, P>::set_padding_for_flags_0(uint8_t value) {
  this->template WriteField<uint8_t>(kPaddingForFlags0Offset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=253&c=3
template<class D, class P>
uint16_t TorqueGeneratedWasmMemoryObject<D, P>::padding_for_flags_1() const {
  uint16_t value = this->template ReadField<uint16_t>(kPaddingForFlags1Offset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=253&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryObject<D, P>::set_padding_for_flags_1(uint16_t value) {
  this->template WriteField<uint16_t>(kPaddingForFlags1Offset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
template<class D, class P>
Tagged<Weak<HeapObject>> TorqueGeneratedWasmMemoryMapDescriptor<D, P>::memory() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmMemoryMapDescriptor::memory(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
template<class D, class P>
Tagged<Weak<HeapObject>> TorqueGeneratedWasmMemoryMapDescriptor<D, P>::memory(PtrComprCageBase cage_base) const {
  Tagged<Weak<HeapObject>> value = TaggedField<Tagged<Weak<HeapObject>>>::load(cage_base, *this, kMemoryOffset);
  DCHECK(value.IsCleared() || (value.IsWeak() && IsWasmMemoryObject(value.GetHeapObjectOrSmi())));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryMapDescriptor<D, P>::set_memory(Tagged<Weak<HeapObject>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (value.IsCleared() || (value.IsWeak() && IsWasmMemoryObject(value.GetHeapObjectOrSmi()))));
  RELAXED_WRITE_WEAK_FIELD(*this, kMemoryOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kMemoryOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=263&c=3
template<class D, class P>
int32_t TorqueGeneratedWasmMemoryMapDescriptor<D, P>::file_descriptor() const {
  int32_t value = this->template ReadField<int32_t>(kFileDescriptorOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=263&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryMapDescriptor<D, P>::set_file_descriptor(int32_t value) {
  this->template WriteField<int32_t>(kFileDescriptorOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=265&c=3
template<class D, class P>
uint32_t TorqueGeneratedWasmMemoryMapDescriptor<D, P>::offset() const {
  uint32_t value = this->template ReadField<uint32_t>(kOffsetOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=265&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryMapDescriptor<D, P>::set_offset(uint32_t value) {
  this->template WriteField<uint32_t>(kOffsetOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=266&c=3
template<class D, class P>
uint32_t TorqueGeneratedWasmMemoryMapDescriptor<D, P>::size() const {
  uint32_t value = this->template ReadField<uint32_t>(kSizeOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=266&c=3
template<class D, class P>
void TorqueGeneratedWasmMemoryMapDescriptor<D, P>::set_size(uint32_t value) {
  this->template WriteField<uint32_t>(kSizeOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
template<class D, class P>
Tagged<Union<ByteArray, FixedArray>> TorqueGeneratedWasmGlobalObject<D, P>::buffer() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmGlobalObject::buffer(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
template<class D, class P>
Tagged<Union<ByteArray, FixedArray>> TorqueGeneratedWasmGlobalObject<D, P>::buffer(PtrComprCageBase cage_base) const {
  Tagged<Union<ByteArray, FixedArray>> value = TaggedField<Tagged<Union<ByteArray, FixedArray>>>::load(cage_base, *this, kBufferOffset);
  DCHECK(IsByteArray(value) || IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
template<class D, class P>
void TorqueGeneratedWasmGlobalObject<D, P>::set_buffer(Tagged<Union<ByteArray, FixedArray>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsByteArray(value) || IsFixedArray(value)));
  WRITE_FIELD(*this, kBufferOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBufferOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=283&c=3
template<class D, class P>
int TorqueGeneratedWasmGlobalObject<D, P>::offset() const {
  int value = TaggedField<Smi>::load(*this, kOffsetOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=283&c=3
template<class D, class P>
void TorqueGeneratedWasmGlobalObject<D, P>::set_offset(int value) {
  WRITE_FIELD(*this, kOffsetOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=284&c=3
template<class D, class P>
int TorqueGeneratedWasmGlobalObject<D, P>::raw_type() const {
  int value = TaggedField<Smi>::load(*this, kRawTypeOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=284&c=3
template<class D, class P>
void TorqueGeneratedWasmGlobalObject<D, P>::set_raw_type(int value) {
  WRITE_FIELD(*this, kRawTypeOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=287&c=3
template<class D, class P>
int TorqueGeneratedWasmGlobalObject<D, P>::is_mutable() const {
  int value = TaggedField<Smi>::load(*this, kIsMutableOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=287&c=3
template<class D, class P>
void TorqueGeneratedWasmGlobalObject<D, P>::set_is_mutable(int value) {
  WRITE_FIELD(*this, kIsMutableOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
template<class D, class P>
Tagged<HeapObject> TorqueGeneratedWasmTagObject<D, P>::tag() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmTagObject::tag(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
template<class D, class P>
Tagged<HeapObject> TorqueGeneratedWasmTagObject<D, P>::tag(PtrComprCageBase cage_base) const {
  Tagged<HeapObject> value = TaggedField<Tagged<HeapObject>>::load(cage_base, *this, kTagOffset);
  DCHECK(IsHeapObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
template<class D, class P>
void TorqueGeneratedWasmTagObject<D, P>::set_tag(Tagged<HeapObject> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value)));
  WRITE_FIELD(*this, kTagOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kTagOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=292&c=3
template<class D, class P>
int TorqueGeneratedWasmTagObject<D, P>::canonical_type_index() const {
  int value = TaggedField<Smi>::load(*this, kCanonicalTypeIndexOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=292&c=3
template<class D, class P>
void TorqueGeneratedWasmTagObject<D, P>::set_canonical_type_index(int value) {
  WRITE_FIELD(*this, kCanonicalTypeIndexOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
template<class D, class P>
Tagged<Managed<wasm::NativeModule>> TorqueGeneratedAsmWasmData<D, P>::managed_native_module() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedAsmWasmData::managed_native_module(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
template<class D, class P>
Tagged<Managed<wasm::NativeModule>> TorqueGeneratedAsmWasmData<D, P>::managed_native_module(PtrComprCageBase cage_base) const {
  Tagged<Managed<wasm::NativeModule>> value = TaggedField<Tagged<Managed<wasm::NativeModule>>>::load(cage_base, *this, kManagedNativeModuleOffset);
  DCHECK(IsForeign(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
template<class D, class P>
void TorqueGeneratedAsmWasmData<D, P>::set_managed_native_module(Tagged<Managed<wasm::NativeModule>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsForeign(value)));
  WRITE_FIELD(*this, kManagedNativeModuleOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kManagedNativeModuleOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
template<class D, class P>
Tagged<HeapNumber> TorqueGeneratedAsmWasmData<D, P>::uses_bitset() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedAsmWasmData::uses_bitset(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
template<class D, class P>
Tagged<HeapNumber> TorqueGeneratedAsmWasmData<D, P>::uses_bitset(PtrComprCageBase cage_base) const {
  Tagged<HeapNumber> value = TaggedField<Tagged<HeapNumber>>::load(cage_base, *this, kUsesBitsetOffset);
  DCHECK(IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
template<class D, class P>
void TorqueGeneratedAsmWasmData<D, P>::set_uses_bitset(Tagged<HeapNumber> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapNumber(value)));
  WRITE_FIELD(*this, kUsesBitsetOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kUsesBitsetOffset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=304&c=3
template<class D, class P>
uint32_t TorqueGeneratedWasmTypeInfo<D, P>::canonical_type() const {
  uint32_t value = this->template ReadField<uint32_t>(kCanonicalTypeOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=304&c=3
template<class D, class P>
void TorqueGeneratedWasmTypeInfo<D, P>::set_canonical_type(uint32_t value) {
  this->template WriteField<uint32_t>(kCanonicalTypeOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=305&c=3
template<class D, class P>
uint32_t TorqueGeneratedWasmTypeInfo<D, P>::canonical_element_type() const {
  uint32_t value = this->template ReadField<uint32_t>(kCanonicalElementTypeOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=305&c=3
template<class D, class P>
void TorqueGeneratedWasmTypeInfo<D, P>::set_canonical_element_type(uint32_t value) {
  this->template WriteField<uint32_t>(kCanonicalElementTypeOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=306&c=9
template<class D, class P>
int TorqueGeneratedWasmTypeInfo<D, P>::supertypes_length() const {
  int value = TaggedField<Smi>::load(*this, kSupertypesLengthOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=306&c=9
template<class D, class P>
void TorqueGeneratedWasmTypeInfo<D, P>::set_supertypes_length(int value) {
  WRITE_FIELD(*this, kSupertypesLengthOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedWasmTypeInfo<D, P>::supertypes(int i) const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmTypeInfo::supertypes(cage_base, i);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedWasmTypeInfo<D, P>::supertypes(PtrComprCageBase cage_base, int i) const {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->supertypes_length());
  int offset = kSupertypesOffset + i * kTaggedSize;
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, offset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
template<class D, class P>
void TorqueGeneratedWasmTypeInfo<D, P>::set_supertypes(int i, Tagged<Object> value, WriteBarrierMode mode) {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->supertypes_length());
  int offset = kSupertypesOffset + i * kTaggedSize;
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, offset, value);
  CONDITIONAL_WRITE_BARRIER(*this, offset, value, mode);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=319&c=3
template<class D, class P>
uint32_t TorqueGeneratedWasmArray<D, P>::length() const {
  uint32_t value = this->template ReadField<uint32_t>(kLengthOffset);
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=319&c=3
template<class D, class P>
void TorqueGeneratedWasmArray<D, P>::set_length(uint32_t value) {
  this->template WriteField<uint32_t>(kLengthOffset, value);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
template<class D, class P>
Tagged<JSReceiver> TorqueGeneratedWasmSuspendingObject<D, P>::callable() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedWasmSuspendingObject::callable(cage_base);
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
template<class D, class P>
Tagged<JSReceiver> TorqueGeneratedWasmSuspendingObject<D, P>::callable(PtrComprCageBase cage_base) const {
  Tagged<JSReceiver> value = TaggedField<Tagged<JSReceiver>>::load(cage_base, *this, kCallableOffset);
  DCHECK(IsJSReceiver(value));
  return value;
}

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
template<class D, class P>
void TorqueGeneratedWasmSuspendingObject<D, P>::set_callable(Tagged<JSReceiver> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value)));
  WRITE_FIELD(*this, kCallableOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCallableOffset, value, mode);
}

