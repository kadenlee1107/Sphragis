// https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=19&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedJSDisplayNames<D, P>::internal() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSDisplayNames::internal(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=19&c=3
template<class D, class P>
Tagged<Foreign> TorqueGeneratedJSDisplayNames<D, P>::internal(PtrComprCageBase cage_base) const {
  Tagged<Foreign> value = TaggedField<Tagged<Foreign>>::load(cage_base, *this, kInternalOffset);
  DCHECK(IsForeign(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=19&c=3
template<class D, class P>
void TorqueGeneratedJSDisplayNames<D, P>::set_internal(Tagged<Foreign> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsForeign(value)));
  WRITE_FIELD(*this, kInternalOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInternalOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=20&c=3
template<class D, class P>
int TorqueGeneratedJSDisplayNames<D, P>::flags() const {
  int value = TaggedField<Smi>::load(*this, kFlagsOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=20&c=3
template<class D, class P>
void TorqueGeneratedJSDisplayNames<D, P>::set_flags(int value) {
  // bitfield struct JSDisplayNamesFlags
  WRITE_FIELD(*this, kFlagsOffset, Smi::From31BitPattern(value));
}

