// https://crsrc.org/c/v8/src/objects/map.tq?l=71&c=3
template<class D, class P>
uint8_t TorqueGeneratedMap<D, P>::instance_size_in_words() const {
  uint8_t value = this->template ReadField<uint8_t>(kInstanceSizeInWordsOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=71&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_instance_size_in_words(uint8_t value) {
  this->template WriteField<uint8_t>(kInstanceSizeInWordsOffset, value);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=72&c=3
template<class D, class P>
uint8_t TorqueGeneratedMap<D, P>::inobject_properties_start_or_constructor_function_index() const {
  uint8_t value = this->template ReadField<uint8_t>(kInobjectPropertiesStartOrConstructorFunctionIndexOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=72&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_inobject_properties_start_or_constructor_function_index(uint8_t value) {
  this->template WriteField<uint8_t>(kInobjectPropertiesStartOrConstructorFunctionIndexOffset, value);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=73&c=3
template<class D, class P>
uint8_t TorqueGeneratedMap<D, P>::used_or_unused_instance_size_in_words() const {
  uint8_t value = this->template ReadField<uint8_t>(kUsedOrUnusedInstanceSizeInWordsOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=73&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_used_or_unused_instance_size_in_words(uint8_t value) {
  this->template WriteField<uint8_t>(kUsedOrUnusedInstanceSizeInWordsOffset, value);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=74&c=3
template<class D, class P>
uint8_t TorqueGeneratedMap<D, P>::visitor_id() const {
  uint8_t value = this->template ReadField<uint8_t>(kVisitorIdOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=74&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_visitor_id(uint8_t value) {
  this->template WriteField<uint8_t>(kVisitorIdOffset, value);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=75&c=3
template<class D, class P>
InstanceType TorqueGeneratedMap<D, P>::instance_type() const {
  InstanceType value = this->template ReadField<InstanceType>(kInstanceTypeOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=75&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_instance_type(InstanceType value) {
  this->template WriteField<InstanceType>(kInstanceTypeOffset, value);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=76&c=3
template<class D, class P>
uint8_t TorqueGeneratedMap<D, P>::bit_field() const {
  uint8_t value = this->template ReadField<uint8_t>(kBitFieldOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=76&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_bit_field(uint8_t value) {
  this->template WriteField<uint8_t>(kBitFieldOffset, value);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=77&c=3
template<class D, class P>
uint8_t TorqueGeneratedMap<D, P>::bit_field2() const {
  uint8_t value = this->template ReadField<uint8_t>(kBitField2Offset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=77&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_bit_field2(uint8_t value) {
  this->template WriteField<uint8_t>(kBitField2Offset, value);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=78&c=3
template<class D, class P>
uint32_t TorqueGeneratedMap<D, P>::bit_field3() const {
  uint32_t value = this->template ReadField<uint32_t>(kBitField3Offset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=78&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_bit_field3(uint32_t value) {
  this->template WriteField<uint32_t>(kBitField3Offset, value);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=83&c=3
template<class D, class P>
Tagged<Union<JSReceiver, Null>> TorqueGeneratedMap<D, P>::prototype() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedMap::prototype(cage_base);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=83&c=3
template<class D, class P>
Tagged<Union<JSReceiver, Null>> TorqueGeneratedMap<D, P>::prototype(PtrComprCageBase cage_base) const {
  Tagged<Union<JSReceiver, Null>> value = TaggedField<Tagged<Union<JSReceiver, Null>>>::load(cage_base, *this, kPrototypeOffset);
  DCHECK(IsJSReceiver(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=83&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_prototype(Tagged<Union<JSReceiver, Null>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsNull(value)));
  WRITE_FIELD(*this, kPrototypeOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPrototypeOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=84&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedMap<D, P>::constructor_or_back_pointer_or_native_context() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedMap::constructor_or_back_pointer_or_native_context(cage_base);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=84&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedMap<D, P>::constructor_or_back_pointer_or_native_context(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kConstructorOrBackPointerOrNativeContextOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=84&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_constructor_or_back_pointer_or_native_context(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kConstructorOrBackPointerOrNativeContextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kConstructorOrBackPointerOrNativeContextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=88&c=33
template<class D, class P>
Tagged<DescriptorArray> TorqueGeneratedMap<D, P>::instance_descriptors() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedMap::instance_descriptors(cage_base);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=88&c=33
template<class D, class P>
Tagged<DescriptorArray> TorqueGeneratedMap<D, P>::instance_descriptors(PtrComprCageBase cage_base) const {
  Tagged<DescriptorArray> value = TaggedField<Tagged<DescriptorArray>>::load(cage_base, *this, kInstanceDescriptorsOffset);
  DCHECK(IsDescriptorArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=88&c=33
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_instance_descriptors(Tagged<DescriptorArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsDescriptorArray(value)));
  WRITE_FIELD(*this, kInstanceDescriptorsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInstanceDescriptorsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=93&c=33
template<class D, class P>
Tagged<WeakArrayList> TorqueGeneratedMap<D, P>::dependent_code() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedMap::dependent_code(cage_base);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=93&c=33
template<class D, class P>
Tagged<WeakArrayList> TorqueGeneratedMap<D, P>::dependent_code(PtrComprCageBase cage_base) const {
  Tagged<WeakArrayList> value = TaggedField<Tagged<WeakArrayList>>::load(cage_base, *this, kDependentCodeOffset);
  DCHECK(IsDependentCode(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=93&c=33
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_dependent_code(Tagged<WeakArrayList> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsDependentCode(value)));
  WRITE_FIELD(*this, kDependentCodeOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kDependentCodeOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=95&c=3
template<class D, class P>
Tagged<Union<Cell, Smi>> TorqueGeneratedMap<D, P>::prototype_validity_cell() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedMap::prototype_validity_cell(cage_base);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=95&c=3
template<class D, class P>
Tagged<Union<Cell, Smi>> TorqueGeneratedMap<D, P>::prototype_validity_cell(PtrComprCageBase cage_base) const {
  Tagged<Union<Cell, Smi>> value = TaggedField<Tagged<Union<Cell, Smi>>>::load(cage_base, *this, kPrototypeValidityCellOffset);
  DCHECK(IsCell(value) || IsZero(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=95&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_prototype_validity_cell(Tagged<Union<Cell, Smi>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsCell(value) || IsZero(value)));
  WRITE_FIELD(*this, kPrototypeValidityCellOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPrototypeValidityCellOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=96&c=3
template<class D, class P>
Tagged<Union<Map, PrototypeInfo, PrototypeSharedClosureInfo, Smi, TransitionArray, Weak<HeapObject>>> TorqueGeneratedMap<D, P>::transitions_or_prototype_info() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedMap::transitions_or_prototype_info(cage_base);
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=96&c=3
template<class D, class P>
Tagged<Union<Map, PrototypeInfo, PrototypeSharedClosureInfo, Smi, TransitionArray, Weak<HeapObject>>> TorqueGeneratedMap<D, P>::transitions_or_prototype_info(PtrComprCageBase cage_base) const {
  Tagged<Union<Map, PrototypeInfo, PrototypeSharedClosureInfo, Smi, TransitionArray, Weak<HeapObject>>> value = TaggedField<Tagged<Union<Map, PrototypeInfo, PrototypeSharedClosureInfo, Smi, TransitionArray, Weak<HeapObject>>>>::load(cage_base, *this, kTransitionsOrPrototypeInfoOffset);
  DCHECK(value.IsCleared() || (!value.IsWeak() && IsPrototypeInfo(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsMap(value.GetHeapObjectOrSmi())) || (value.IsWeak() && IsMap(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsZero(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsPrototypeSharedClosureInfo(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsTransitionArray(value.GetHeapObjectOrSmi())));
  return value;
}

// https://crsrc.org/c/v8/src/objects/map.tq?l=96&c=3
template<class D, class P>
void TorqueGeneratedMap<D, P>::set_transitions_or_prototype_info(Tagged<Union<Map, PrototypeInfo, PrototypeSharedClosureInfo, Smi, TransitionArray, Weak<HeapObject>>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (value.IsCleared() || (!value.IsWeak() && IsPrototypeInfo(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsMap(value.GetHeapObjectOrSmi())) || (value.IsWeak() && IsMap(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsZero(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsPrototypeSharedClosureInfo(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsTransitionArray(value.GetHeapObjectOrSmi()))));
  RELAXED_WRITE_WEAK_FIELD(*this, kTransitionsOrPrototypeInfoOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kTransitionsOrPrototypeInfoOffset, value, mode);
}

