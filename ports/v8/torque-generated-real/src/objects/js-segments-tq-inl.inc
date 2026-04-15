// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=12&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedJSSegments<D, P>::icu_iterator_with_text() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSSegments::icu_iterator_with_text(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=12&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedJSSegments<D, P>::icu_iterator_with_text(PtrComprCageBase cage_base) const {
  Tagged<Foreign> value = TaggedField<Tagged<Foreign>>::load(cage_base, *this, kIcuIteratorWithTextOffset);
  DCHECK(IsForeign(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=12&c=3
template<class D, class P>
void TorqueGeneratedJSSegments<D, P>::set_icu_iterator_with_text(Tagged<Foreign> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsForeign(value)));
  WRITE_FIELD(*this, kIcuIteratorWithTextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIcuIteratorWithTextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=13&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSSegments<D, P>::raw_string() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSSegments::raw_string(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=13&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSSegments<D, P>::raw_string(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kRawStringOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=13&c=3
template<class D, class P>
void TorqueGeneratedJSSegments<D, P>::set_raw_string(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kRawStringOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kRawStringOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=14&c=3
template<class D, class P>
int TorqueGeneratedJSSegments<D, P>::flags() const {
  int value = TaggedField<Smi>::load(*this, kFlagsOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=14&c=3
template<class D, class P>
void TorqueGeneratedJSSegments<D, P>::set_flags(int value) {
  // bitfield struct JSSegmentsFlags
  WRITE_FIELD(*this, kFlagsOffset, Smi::From31BitPattern(value));
}

