// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=8&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSV8BreakIterator<D, P>::locale() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSV8BreakIterator::locale(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=8&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSV8BreakIterator<D, P>::locale(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kLocaleOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=8&c=3
template<class D, class P>
void TorqueGeneratedJSV8BreakIterator<D, P>::set_locale(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kLocaleOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kLocaleOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=9&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedJSV8BreakIterator<D, P>::icu_iterator_with_text() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSV8BreakIterator::icu_iterator_with_text(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=9&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedJSV8BreakIterator<D, P>::icu_iterator_with_text(PtrComprCageBase cage_base) const {
  Tagged<Foreign> value = TaggedField<Tagged<Foreign>>::load(cage_base, *this, kIcuIteratorWithTextOffset);
  DCHECK(IsForeign(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=9&c=3
template<class D, class P>
void TorqueGeneratedJSV8BreakIterator<D, P>::set_icu_iterator_with_text(Tagged<Foreign> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsForeign(value)));
  WRITE_FIELD(*this, kIcuIteratorWithTextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIcuIteratorWithTextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=10&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_adopt_text() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSV8BreakIterator::bound_adopt_text(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=10&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_adopt_text(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFunction, Undefined>> value = TaggedField<Tagged<Union<JSFunction, Undefined>>>::load(cage_base, *this, kBoundAdoptTextOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=10&c=3
template<class D, class P>
void TorqueGeneratedJSV8BreakIterator<D, P>::set_bound_adopt_text(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kBoundAdoptTextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBoundAdoptTextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=11&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_first() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSV8BreakIterator::bound_first(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=11&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_first(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFunction, Undefined>> value = TaggedField<Tagged<Union<JSFunction, Undefined>>>::load(cage_base, *this, kBoundFirstOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=11&c=3
template<class D, class P>
void TorqueGeneratedJSV8BreakIterator<D, P>::set_bound_first(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kBoundFirstOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBoundFirstOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=12&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_next() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSV8BreakIterator::bound_next(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=12&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_next(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFunction, Undefined>> value = TaggedField<Tagged<Union<JSFunction, Undefined>>>::load(cage_base, *this, kBoundNextOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=12&c=3
template<class D, class P>
void TorqueGeneratedJSV8BreakIterator<D, P>::set_bound_next(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kBoundNextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBoundNextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=13&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_current() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSV8BreakIterator::bound_current(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=13&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_current(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFunction, Undefined>> value = TaggedField<Tagged<Union<JSFunction, Undefined>>>::load(cage_base, *this, kBoundCurrentOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=13&c=3
template<class D, class P>
void TorqueGeneratedJSV8BreakIterator<D, P>::set_bound_current(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kBoundCurrentOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBoundCurrentOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=14&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_break_type() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSV8BreakIterator::bound_break_type(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=14&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSV8BreakIterator<D, P>::bound_break_type(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFunction, Undefined>> value = TaggedField<Tagged<Union<JSFunction, Undefined>>>::load(cage_base, *this, kBoundBreakTypeOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=14&c=3
template<class D, class P>
void TorqueGeneratedJSV8BreakIterator<D, P>::set_bound_break_type(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kBoundBreakTypeOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBoundBreakTypeOffset, value, mode);
}

