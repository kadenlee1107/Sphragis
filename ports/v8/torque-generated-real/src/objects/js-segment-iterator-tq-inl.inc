// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=12&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedJSSegmentIterator<D, P>::icu_iterator_with_text() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSSegmentIterator::icu_iterator_with_text(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=12&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedJSSegmentIterator<D, P>::icu_iterator_with_text(PtrComprCageBase cage_base) const {
  Tagged<Foreign> value = TaggedField<Tagged<Foreign>>::load(cage_base, *this, kIcuIteratorWithTextOffset);
  DCHECK(IsForeign(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=12&c=3
template<class D, class P>
void TorqueGeneratedJSSegmentIterator<D, P>::set_icu_iterator_with_text(Tagged<Foreign> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsForeign(value)));
  WRITE_FIELD(*this, kIcuIteratorWithTextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIcuIteratorWithTextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=13&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSSegmentIterator<D, P>::raw_string() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSSegmentIterator::raw_string(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=13&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSSegmentIterator<D, P>::raw_string(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kRawStringOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=13&c=3
template<class D, class P>
void TorqueGeneratedJSSegmentIterator<D, P>::set_raw_string(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kRawStringOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kRawStringOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=14&c=3
template<class D, class P>
int TorqueGeneratedJSSegmentIterator<D, P>::flags() const {
  int value = TaggedField<Smi>::load(*this, kFlagsOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=14&c=3
template<class D, class P>
void TorqueGeneratedJSSegmentIterator<D, P>::set_flags(int value) {
  // bitfield struct JSSegmentIteratorFlags
  WRITE_FIELD(*this, kFlagsOffset, Smi::From31BitPattern(value));
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=20&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSSegmentDataObject<D, P>::segment() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSSegmentDataObject::segment(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=20&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSSegmentDataObject<D, P>::segment(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kSegmentOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=20&c=3
template<class D, class P>
void TorqueGeneratedJSSegmentDataObject<D, P>::set_segment(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kSegmentOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kSegmentOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=21&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSSegmentDataObject<D, P>::index() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSSegmentDataObject::index(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=21&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSSegmentDataObject<D, P>::index(PtrComprCageBase cage_base) const {
  Tagged<Number> value = TaggedField<Tagged<Number>>::load(cage_base, *this, kIndexOffset);
  DCHECK(IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=21&c=3
template<class D, class P>
void TorqueGeneratedJSSegmentDataObject<D, P>::set_index(Tagged<Number> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kIndexOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIndexOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=22&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSSegmentDataObject<D, P>::input() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSSegmentDataObject::input(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=22&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSSegmentDataObject<D, P>::input(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kInputOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=22&c=3
template<class D, class P>
void TorqueGeneratedJSSegmentDataObject<D, P>::set_input(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kInputOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInputOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=28&c=3
template<class D, class P>
Tagged<Boolean> TorqueGeneratedJSSegmentDataObjectWithIsWordLike<D, P>::is_word_like() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSSegmentDataObjectWithIsWordLike::is_word_like(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=28&c=3
template<class D, class P>
Tagged<Boolean> TorqueGeneratedJSSegmentDataObjectWithIsWordLike<D, P>::is_word_like(PtrComprCageBase cage_base) const {
  Tagged<Boolean> value = TaggedField<Tagged<Boolean>>::load(cage_base, *this, kIsWordLikeOffset);
  DCHECK(IsBoolean(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=28&c=3
template<class D, class P>
void TorqueGeneratedJSSegmentDataObjectWithIsWordLike<D, P>::set_is_word_like(Tagged<Boolean> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsBoolean(value)));
  WRITE_FIELD(*this, kIsWordLikeOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIsWordLikeOffset, value, mode);
}

