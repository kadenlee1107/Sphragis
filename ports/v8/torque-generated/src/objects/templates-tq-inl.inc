// https://crsrc.org/c/v8/src/objects/templates.tq?l=13&c=3
template<class D, class P>
int TorqueGeneratedTemplateInfo<D, P>::template_info_flags() const {
  int value = TaggedField<Smi>::load(*this, kTemplateInfoFlagsOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=13&c=3
template<class D, class P>
void TorqueGeneratedTemplateInfo<D, P>::set_template_info_flags(int value) {
  // bitfield struct TemplateInfoFlags
  WRITE_FIELD(*this, kTemplateInfoFlagsOffset, Smi::From31BitPattern(value));
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=18&c=3
template<class D, class P>
int TorqueGeneratedTemplateInfoWithProperties<D, P>::number_of_properties() const {
  int value = TaggedField<Smi>::load(*this, kNumberOfPropertiesOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=18&c=3
template<class D, class P>
void TorqueGeneratedTemplateInfoWithProperties<D, P>::set_number_of_properties(int value) {
  WRITE_FIELD(*this, kNumberOfPropertiesOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=19&c=3
template<class D, class P>
Tagged<Union<ArrayList, Undefined>> TorqueGeneratedTemplateInfoWithProperties<D, P>::property_list() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedTemplateInfoWithProperties::property_list(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=19&c=3
template<class D, class P>
Tagged<Union<ArrayList, Undefined>> TorqueGeneratedTemplateInfoWithProperties<D, P>::property_list(PtrComprCageBase cage_base) const {
  Tagged<Union<ArrayList, Undefined>> value = TaggedField<Tagged<Union<ArrayList, Undefined>>>::load(cage_base, *this, kPropertyListOffset);
  DCHECK(IsUndefined(value) || IsArrayList(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=19&c=3
template<class D, class P>
void TorqueGeneratedTemplateInfoWithProperties<D, P>::set_property_list(Tagged<Union<ArrayList, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsArrayList(value)));
  WRITE_FIELD(*this, kPropertyListOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPropertyListOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=20&c=3
template<class D, class P>
Tagged<Union<ArrayList, Undefined>> TorqueGeneratedTemplateInfoWithProperties<D, P>::property_accessors() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedTemplateInfoWithProperties::property_accessors(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=20&c=3
template<class D, class P>
Tagged<Union<ArrayList, Undefined>> TorqueGeneratedTemplateInfoWithProperties<D, P>::property_accessors(PtrComprCageBase cage_base) const {
  Tagged<Union<ArrayList, Undefined>> value = TaggedField<Tagged<Union<ArrayList, Undefined>>>::load(cage_base, *this, kPropertyAccessorsOffset);
  DCHECK(IsUndefined(value) || IsArrayList(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=20&c=3
template<class D, class P>
void TorqueGeneratedTemplateInfoWithProperties<D, P>::set_property_accessors(Tagged<Union<ArrayList, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsArrayList(value)));
  WRITE_FIELD(*this, kPropertyAccessorsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPropertyAccessorsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=57&c=3
template<class D, class P>
Tagged<Union<String, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::class_name() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedFunctionTemplateInfo::class_name(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=57&c=3
template<class D, class P>
Tagged<Union<String, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::class_name(PtrComprCageBase cage_base) const {
  Tagged<Union<String, Undefined>> value = TaggedField<Tagged<Union<String, Undefined>>>::load(cage_base, *this, kClassNameOffset);
  DCHECK(IsUndefined(value) || IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=57&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_class_name(Tagged<Union<String, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsString(value)));
  WRITE_FIELD(*this, kClassNameOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kClassNameOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=61&c=3
template<class D, class P>
Tagged<Union<String, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::interface_name() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedFunctionTemplateInfo::interface_name(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=61&c=3
template<class D, class P>
Tagged<Union<String, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::interface_name(PtrComprCageBase cage_base) const {
  Tagged<Union<String, Undefined>> value = TaggedField<Tagged<Union<String, Undefined>>>::load(cage_base, *this, kInterfaceNameOffset);
  DCHECK(IsUndefined(value) || IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=61&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_interface_name(Tagged<Union<String, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsString(value)));
  WRITE_FIELD(*this, kInterfaceNameOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInterfaceNameOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=66&c=3
template<class D, class P>
Tagged<Union<FunctionTemplateInfo, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::signature() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedFunctionTemplateInfo::signature(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=66&c=3
template<class D, class P>
Tagged<Union<FunctionTemplateInfo, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::signature(PtrComprCageBase cage_base) const {
  Tagged<Union<FunctionTemplateInfo, Undefined>> value = TaggedField<Tagged<Union<FunctionTemplateInfo, Undefined>>>::load(cage_base, *this, kSignatureOffset);
  DCHECK(IsUndefined(value) || IsFunctionTemplateInfo(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=66&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_signature(Tagged<Union<FunctionTemplateInfo, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsFunctionTemplateInfo(value)));
  WRITE_FIELD(*this, kSignatureOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kSignatureOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=72&c=3
template<class D, class P>
Tagged<Union<FunctionTemplateRareData, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::rare_data(AcquireLoadTag) const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedFunctionTemplateInfo::rare_data(cage_base, kAcquireLoad);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=72&c=3
template<class D, class P>
Tagged<Union<FunctionTemplateRareData, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::rare_data(PtrComprCageBase cage_base, AcquireLoadTag) const {
  Tagged<Union<FunctionTemplateRareData, Undefined>> value = TaggedField<Tagged<Union<FunctionTemplateRareData, Undefined>>>::Acquire_Load(cage_base, *this, kRareDataOffset);
  DCHECK(IsUndefined(value) || IsFunctionTemplateRareData(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=72&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_rare_data(Tagged<Union<FunctionTemplateRareData, Undefined>> value, ReleaseStoreTag, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsFunctionTemplateRareData(value)));
  RELEASE_WRITE_FIELD(*this, kRareDataOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kRareDataOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=73&c=3
template<class D, class P>
Tagged<Union<SharedFunctionInfo, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::shared_function_info() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedFunctionTemplateInfo::shared_function_info(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=73&c=3
template<class D, class P>
Tagged<Union<SharedFunctionInfo, Undefined>> TorqueGeneratedFunctionTemplateInfo<D, P>::shared_function_info(PtrComprCageBase cage_base) const {
  Tagged<Union<SharedFunctionInfo, Undefined>> value = TaggedField<Tagged<Union<SharedFunctionInfo, Undefined>>>::load(cage_base, *this, kSharedFunctionInfoOffset);
  DCHECK(IsUndefined(value) || IsSharedFunctionInfo(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=73&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_shared_function_info(Tagged<Union<SharedFunctionInfo, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsSharedFunctionInfo(value)));
  WRITE_FIELD(*this, kSharedFunctionInfoOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kSharedFunctionInfoOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=77&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedFunctionTemplateInfo<D, P>::cached_property_name() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedFunctionTemplateInfo::cached_property_name(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=77&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedFunctionTemplateInfo<D, P>::cached_property_name(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kCachedPropertyNameOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=77&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_cached_property_name(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kCachedPropertyNameOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCachedPropertyNameOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=84&c=36
template<class D, class P>
Tagged<Object> TorqueGeneratedFunctionTemplateInfo<D, P>::callback_data(AcquireLoadTag) const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedFunctionTemplateInfo::callback_data(cage_base, kAcquireLoad);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=84&c=36
template<class D, class P>
Tagged<Object> TorqueGeneratedFunctionTemplateInfo<D, P>::callback_data(PtrComprCageBase cage_base, AcquireLoadTag) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::Acquire_Load(cage_base, *this, kCallbackDataOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=84&c=36
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_callback_data(Tagged<Object> value, ReleaseStoreTag, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  RELEASE_WRITE_FIELD(*this, kCallbackDataOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCallbackDataOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=87&c=3
template<class D, class P>
uint32_t TorqueGeneratedFunctionTemplateInfo<D, P>::flag() const {
  uint32_t value = this->template ReadField<uint32_t>(kFlagOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=87&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_flag(uint32_t value) {
  this->template WriteField<uint32_t>(kFlagOffset, value);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=89&c=3
template<class D, class P>
int16_t TorqueGeneratedFunctionTemplateInfo<D, P>::length() const {
  int16_t value = this->template ReadField<int16_t>(kLengthOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=89&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_length(int16_t value) {
  this->template WriteField<int16_t>(kLengthOffset, value);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=92&c=3
template<class D, class P>
InstanceType TorqueGeneratedFunctionTemplateInfo<D, P>::instance_type() const {
  InstanceType value = this->template ReadField<InstanceType>(kInstanceTypeOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=92&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_instance_type(InstanceType value) {
  this->template WriteField<InstanceType>(kInstanceTypeOffset, value);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=97&c=3
template<class D, class P>
uint32_t TorqueGeneratedFunctionTemplateInfo<D, P>::exception_context() const {
  uint32_t value = this->template ReadField<uint32_t>(kExceptionContextOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=97&c=3
template<class D, class P>
void TorqueGeneratedFunctionTemplateInfo<D, P>::set_exception_context(uint32_t value) {
  this->template WriteField<uint32_t>(kExceptionContextOffset, value);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=117&c=3
template<class D, class P>
Tagged<Union<FunctionTemplateInfo, Undefined>> TorqueGeneratedObjectTemplateInfo<D, P>::constructor() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedObjectTemplateInfo::constructor(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=117&c=3
template<class D, class P>
Tagged<Union<FunctionTemplateInfo, Undefined>> TorqueGeneratedObjectTemplateInfo<D, P>::constructor(PtrComprCageBase cage_base) const {
  Tagged<Union<FunctionTemplateInfo, Undefined>> value = TaggedField<Tagged<Union<FunctionTemplateInfo, Undefined>>>::load(cage_base, *this, kConstructorOffset);
  DCHECK(IsUndefined(value) || IsFunctionTemplateInfo(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=117&c=3
template<class D, class P>
void TorqueGeneratedObjectTemplateInfo<D, P>::set_constructor(Tagged<Union<FunctionTemplateInfo, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsFunctionTemplateInfo(value)));
  WRITE_FIELD(*this, kConstructorOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kConstructorOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=118&c=3
template<class D, class P>
int TorqueGeneratedObjectTemplateInfo<D, P>::data() const {
  int value = TaggedField<Smi>::load(*this, kDataOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=118&c=3
template<class D, class P>
void TorqueGeneratedObjectTemplateInfo<D, P>::set_data(int value) {
  // bitfield struct ObjectTemplateInfoFlags
  WRITE_FIELD(*this, kDataOffset, Smi::From31BitPattern(value));
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=123&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedDictionaryTemplateInfo<D, P>::property_names() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedDictionaryTemplateInfo::property_names(cage_base);
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=123&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedDictionaryTemplateInfo<D, P>::property_names(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kPropertyNamesOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/templates.tq?l=123&c=3
template<class D, class P>
void TorqueGeneratedDictionaryTemplateInfo<D, P>::set_property_names(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kPropertyNamesOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPropertyNamesOffset, value, mode);
}

