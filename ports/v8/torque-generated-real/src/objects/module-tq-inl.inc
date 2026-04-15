// https://crsrc.org/c/v8/src/objects/module.tq?l=23&c=3
template<class D, class P>
Tagged<Module> TorqueGeneratedJSModuleNamespace<D, P>::module() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSModuleNamespace::module(cage_base);
}

// https://crsrc.org/c/v8/src/objects/module.tq?l=23&c=3
template<class D, class P>
Tagged<Module> TorqueGeneratedJSModuleNamespace<D, P>::module(PtrComprCageBase cage_base) const {
  Tagged<Module> value = TaggedField<Tagged<Module>>::load(cage_base, *this, kModuleOffset);
  DCHECK(IsModule(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/module.tq?l=23&c=3
template<class D, class P>
void TorqueGeneratedJSModuleNamespace<D, P>::set_module(Tagged<Module> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsModule(value)));
  WRITE_FIELD(*this, kModuleOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kModuleOffset, value, mode);
}

