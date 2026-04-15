// https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=7&c=3
template<class D, class P>
Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> TorqueGeneratedMegaDomHandler<D, P>::accessor() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedMegaDomHandler::accessor(cage_base);
}

// https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=7&c=3
template<class D, class P>
Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> TorqueGeneratedMegaDomHandler<D, P>::accessor(PtrComprCageBase cage_base) const {
  Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> value = TaggedField<Tagged<Union<HeapObject, Smi, Weak<HeapObject>>>>::load(cage_base, *this, kAccessorOffset);
  DCHECK(value.IsCleared() || (!value.IsWeak() && IsHeapObject(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsSmi(value.GetHeapObjectOrSmi())) || value.IsWeak());
  return value;
}

// https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=7&c=3
template<class D, class P>
void TorqueGeneratedMegaDomHandler<D, P>::set_accessor(Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (value.IsCleared() || (!value.IsWeak() && IsHeapObject(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsSmi(value.GetHeapObjectOrSmi())) || value.IsWeak()));
  RELAXED_WRITE_WEAK_FIELD(*this, kAccessorOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kAccessorOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=8&c=3
template<class D, class P>
Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> TorqueGeneratedMegaDomHandler<D, P>::context() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedMegaDomHandler::context(cage_base);
}

// https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=8&c=3
template<class D, class P>
Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> TorqueGeneratedMegaDomHandler<D, P>::context(PtrComprCageBase cage_base) const {
  Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> value = TaggedField<Tagged<Union<HeapObject, Smi, Weak<HeapObject>>>>::load(cage_base, *this, kContextOffset);
  DCHECK(value.IsCleared() || (!value.IsWeak() && IsHeapObject(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsSmi(value.GetHeapObjectOrSmi())) || value.IsWeak());
  return value;
}

// https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=8&c=3
template<class D, class P>
void TorqueGeneratedMegaDomHandler<D, P>::set_context(Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (value.IsCleared() || (!value.IsWeak() && IsHeapObject(value.GetHeapObjectOrSmi())) || (!value.IsWeak() && IsSmi(value.GetHeapObjectOrSmi())) || value.IsWeak()));
  RELAXED_WRITE_WEAK_FIELD(*this, kContextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kContextOffset, value, mode);
}

