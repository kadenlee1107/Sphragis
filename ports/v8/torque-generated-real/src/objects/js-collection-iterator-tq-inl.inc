// https://crsrc.org/c/v8/src/objects/js-collection-iterator.tq?l=8&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedJSCollectionIterator<D, P>::table() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSCollectionIterator::table(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-collection-iterator.tq?l=8&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedJSCollectionIterator<D, P>::table(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kTableOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-collection-iterator.tq?l=8&c=3
template<class D, class P>
void TorqueGeneratedJSCollectionIterator<D, P>::set_table(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kTableOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kTableOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-collection-iterator.tq?l=10&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedJSCollectionIterator<D, P>::index() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSCollectionIterator::index(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-collection-iterator.tq?l=10&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedJSCollectionIterator<D, P>::index(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kIndexOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-collection-iterator.tq?l=10&c=3
template<class D, class P>
void TorqueGeneratedJSCollectionIterator<D, P>::set_index(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kIndexOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIndexOffset, value, mode);
}

