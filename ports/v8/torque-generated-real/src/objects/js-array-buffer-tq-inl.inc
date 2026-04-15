// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=19&c=3
template<class D, class P>
Tagged<Union<Cell, Smi, Weak<HeapObject>>> TorqueGeneratedJSArrayBuffer<D, P>::views_or_detach_key() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSArrayBuffer::views_or_detach_key(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=19&c=3
template<class D, class P>
Tagged<Union<Cell, Smi, Weak<HeapObject>>> TorqueGeneratedJSArrayBuffer<D, P>::views_or_detach_key(PtrComprCageBase cage_base) const {
  Tagged<Union<Cell, Smi, Weak<HeapObject>>> value = TaggedField<Tagged<Union<Cell, Smi, Weak<HeapObject>>>>::load(cage_base, *this, kViewsOrDetachKeyOffset);
  DCHECK(value.IsCleared() || (!value.IsWeak() && IsSmi(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsCell(value.GetHeapObjectOrSmi())) || (value.IsWeak() && IsJSArrayBufferView(value.GetHeapObjectOrSmi())));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=19&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBuffer<D, P>::set_views_or_detach_key(Tagged<Union<Cell, Smi, Weak<HeapObject>>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (value.IsCleared() || (!value.IsWeak() && IsSmi(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsCell(value.GetHeapObjectOrSmi())) || (value.IsWeak() && IsJSArrayBufferView(value.GetHeapObjectOrSmi()))));
  RELAXED_WRITE_WEAK_FIELD(*this, kViewsOrDetachKeyOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kViewsOrDetachKeyOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=21&c=3
template<class D, class P>
uintptr_t TorqueGeneratedJSArrayBuffer<D, P>::raw_byte_length() const {
  uintptr_t value = this->template ReadField<uintptr_t>(kRawByteLengthOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=21&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBuffer<D, P>::set_raw_byte_length(uintptr_t value) {
  this->template WriteField<uintptr_t>(kRawByteLengthOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=23&c=3
template<class D, class P>
uintptr_t TorqueGeneratedJSArrayBuffer<D, P>::raw_max_byte_length() const {
  uintptr_t value = this->template ReadField<uintptr_t>(kRawMaxByteLengthOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=23&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBuffer<D, P>::set_raw_max_byte_length(uintptr_t value) {
  this->template WriteField<uintptr_t>(kRawMaxByteLengthOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=25&c=3
template<class D, class P>
Address TorqueGeneratedJSArrayBuffer<D, P>::backing_store() const {
  Address value = this->template ReadField<Address>(kBackingStoreOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=25&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBuffer<D, P>::set_backing_store(Address value) {
  this->template WriteField<Address>(kBackingStoreOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=27&c=3
template<class D, class P>
uint32_t TorqueGeneratedJSArrayBuffer<D, P>::bit_field() const {
  uint32_t value = this->template ReadField<uint32_t>(kBitFieldOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=27&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBuffer<D, P>::set_bit_field(uint32_t value) {
  this->template WriteField<uint32_t>(kBitFieldOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=74&c=3
template<class D, class P>
Tagged<JSArrayBuffer> TorqueGeneratedJSArrayBufferView<D, P>::buffer() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSArrayBufferView::buffer(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=74&c=3
template<class D, class P>
Tagged<JSArrayBuffer> TorqueGeneratedJSArrayBufferView<D, P>::buffer(PtrComprCageBase cage_base) const {
  Tagged<JSArrayBuffer> value = TaggedField<Tagged<JSArrayBuffer>>::load(cage_base, *this, kBufferOffset);
  DCHECK(IsJSArrayBuffer(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=74&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBufferView<D, P>::set_buffer(Tagged<JSArrayBuffer> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSArrayBuffer(value)));
  WRITE_FIELD(*this, kBufferOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBufferOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=75&c=3
template<class D, class P>
uint32_t TorqueGeneratedJSArrayBufferView<D, P>::bit_field() const {
  uint32_t value = this->template ReadField<uint32_t>(kBitFieldOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=75&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBufferView<D, P>::set_bit_field(uint32_t value) {
  this->template WriteField<uint32_t>(kBitFieldOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=78&c=3
template<class D, class P>
uintptr_t TorqueGeneratedJSArrayBufferView<D, P>::raw_byte_offset() const {
  uintptr_t value = this->template ReadField<uintptr_t>(kRawByteOffsetOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=78&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBufferView<D, P>::set_raw_byte_offset(uintptr_t value) {
  this->template WriteField<uintptr_t>(kRawByteOffsetOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=80&c=3
template<class D, class P>
uintptr_t TorqueGeneratedJSArrayBufferView<D, P>::raw_byte_length() const {
  uintptr_t value = this->template ReadField<uintptr_t>(kRawByteLengthOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=80&c=3
template<class D, class P>
void TorqueGeneratedJSArrayBufferView<D, P>::set_raw_byte_length(uintptr_t value) {
  this->template WriteField<uintptr_t>(kRawByteLengthOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=120&c=3
template<class D, class P>
uintptr_t TorqueGeneratedJSTypedArray<D, P>::raw_length() const {
  uintptr_t value = this->template ReadField<uintptr_t>(kRawLengthOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=120&c=3
template<class D, class P>
void TorqueGeneratedJSTypedArray<D, P>::set_raw_length(uintptr_t value) {
  this->template WriteField<uintptr_t>(kRawLengthOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=122&c=3
template<class D, class P>
Address TorqueGeneratedJSTypedArray<D, P>::external_pointer() const {
  Address value = this->template ReadField<Address>(kExternalPointerOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=122&c=3
template<class D, class P>
void TorqueGeneratedJSTypedArray<D, P>::set_external_pointer(Address value) {
  this->template WriteField<Address>(kExternalPointerOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=123&c=3
template<class D, class P>
Tagged<Union<ByteArray, Smi>> TorqueGeneratedJSTypedArray<D, P>::base_pointer() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSTypedArray::base_pointer(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=123&c=3
template<class D, class P>
Tagged<Union<ByteArray, Smi>> TorqueGeneratedJSTypedArray<D, P>::base_pointer(PtrComprCageBase cage_base) const {
  Tagged<Union<ByteArray, Smi>> value = TaggedField<Tagged<Union<ByteArray, Smi>>>::load(cage_base, *this, kBasePointerOffset);
  DCHECK(IsByteArray(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=123&c=3
template<class D, class P>
void TorqueGeneratedJSTypedArray<D, P>::set_base_pointer(Tagged<Union<ByteArray, Smi>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsByteArray(value) || IsSmi(value)));
  WRITE_FIELD(*this, kBasePointerOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBasePointerOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=142&c=3
template<class D, class P>
Address TorqueGeneratedJSDataViewOrRabGsabDataView<D, P>::data_pointer() const {
  Address value = this->template ReadField<Address>(kDataPointerOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-array-buffer.tq?l=142&c=3
template<class D, class P>
void TorqueGeneratedJSDataViewOrRabGsabDataView<D, P>::set_data_pointer(Address value) {
  this->template WriteField<Address>(kDataPointerOffset, value);
}

