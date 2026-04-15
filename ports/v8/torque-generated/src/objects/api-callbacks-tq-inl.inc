// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=48&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedAccessorInfo<D, P>::data() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedAccessorInfo::data(cage_base);
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=48&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedAccessorInfo<D, P>::data(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kDataOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=48&c=3
template<class D, class P>
void TorqueGeneratedAccessorInfo<D, P>::set_data(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kDataOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kDataOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=57&c=3
template<class D, class P>
Tagged<Name> TorqueGeneratedAccessorInfo<D, P>::name() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedAccessorInfo::name(cage_base);
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=57&c=3
template<class D, class P>
Tagged<Name> TorqueGeneratedAccessorInfo<D, P>::name(PtrComprCageBase cage_base) const {
  Tagged<Name> value = TaggedField<Tagged<Name>>::load(cage_base, *this, kNameOffset);
  DCHECK(IsName(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=57&c=3
template<class D, class P>
void TorqueGeneratedAccessorInfo<D, P>::set_name(Tagged<Name> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsName(value)));
  WRITE_FIELD(*this, kNameOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kNameOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=63&c=3
template<class D, class P>
uint32_t TorqueGeneratedAccessorInfo<D, P>::flags() const {
  uint32_t value = this->template ReadField<uint32_t>(kFlagsOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=63&c=3
template<class D, class P>
void TorqueGeneratedAccessorInfo<D, P>::set_flags(uint32_t value) {
  this->template WriteField<uint32_t>(kFlagsOffset, value);
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=14&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedInterceptorInfo<D, P>::data() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedInterceptorInfo::data(cage_base);
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=14&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedInterceptorInfo<D, P>::data(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kDataOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=14&c=3
template<class D, class P>
void TorqueGeneratedInterceptorInfo<D, P>::set_data(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kDataOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kDataOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=15&c=3
template<class D, class P>
uint32_t TorqueGeneratedInterceptorInfo<D, P>::flags() const {
  uint32_t value = this->template ReadField<uint32_t>(kFlagsOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=15&c=3
template<class D, class P>
void TorqueGeneratedInterceptorInfo<D, P>::set_flags(uint32_t value) {
  this->template WriteField<uint32_t>(kFlagsOffset, value);
}

