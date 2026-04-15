// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=10&c=3
template<class D, class P>
Tagged<NativeContext> TorqueGeneratedJSFinalizationRegistry<D, P>::native_context() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFinalizationRegistry::native_context(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=10&c=3
template<class D, class P>
Tagged<NativeContext> TorqueGeneratedJSFinalizationRegistry<D, P>::native_context(PtrComprCageBase cage_base) const {
  Tagged<NativeContext> value = TaggedField<Tagged<NativeContext>>::load(cage_base, *this, kNativeContextOffset);
  DCHECK(IsNativeContext(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=10&c=3
template<class D, class P>
void TorqueGeneratedJSFinalizationRegistry<D, P>::set_native_context(Tagged<NativeContext> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsNativeContext(value)));
  WRITE_FIELD(*this, kNativeContextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kNativeContextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=11&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSFinalizationRegistry<D, P>::cleanup() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFinalizationRegistry::cleanup(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=11&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSFinalizationRegistry<D, P>::cleanup(PtrComprCageBase cage_base) const {
  Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value = TaggedField<Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>>::load(cage_base, *this, kCleanupOffset);
  DCHECK(IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=11&c=3
template<class D, class P>
void TorqueGeneratedJSFinalizationRegistry<D, P>::set_cleanup(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value)));
  WRITE_FIELD(*this, kCleanupOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCleanupOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=12&c=3
template<class D, class P>
Tagged<Union<Undefined, WeakCell>> TorqueGeneratedJSFinalizationRegistry<D, P>::active_cells() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFinalizationRegistry::active_cells(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=12&c=3
template<class D, class P>
Tagged<Union<Undefined, WeakCell>> TorqueGeneratedJSFinalizationRegistry<D, P>::active_cells(PtrComprCageBase cage_base) const {
  Tagged<Union<Undefined, WeakCell>> value = TaggedField<Tagged<Union<Undefined, WeakCell>>>::load(cage_base, *this, kActiveCellsOffset);
  DCHECK(IsUndefined(value) || IsWeakCell(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=12&c=3
template<class D, class P>
void TorqueGeneratedJSFinalizationRegistry<D, P>::set_active_cells(Tagged<Union<Undefined, WeakCell>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsWeakCell(value)));
  WRITE_FIELD(*this, kActiveCellsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kActiveCellsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=13&c=3
template<class D, class P>
Tagged<Union<Undefined, WeakCell>> TorqueGeneratedJSFinalizationRegistry<D, P>::cleared_cells() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFinalizationRegistry::cleared_cells(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=13&c=3
template<class D, class P>
Tagged<Union<Undefined, WeakCell>> TorqueGeneratedJSFinalizationRegistry<D, P>::cleared_cells(PtrComprCageBase cage_base) const {
  Tagged<Union<Undefined, WeakCell>> value = TaggedField<Tagged<Union<Undefined, WeakCell>>>::load(cage_base, *this, kClearedCellsOffset);
  DCHECK(IsUndefined(value) || IsWeakCell(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=13&c=3
template<class D, class P>
void TorqueGeneratedJSFinalizationRegistry<D, P>::set_cleared_cells(Tagged<Union<Undefined, WeakCell>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsWeakCell(value)));
  WRITE_FIELD(*this, kClearedCellsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kClearedCellsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=14&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedJSFinalizationRegistry<D, P>::key_map() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFinalizationRegistry::key_map(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=14&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedJSFinalizationRegistry<D, P>::key_map(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kKeyMapOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=14&c=3
template<class D, class P>
void TorqueGeneratedJSFinalizationRegistry<D, P>::set_key_map(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kKeyMapOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kKeyMapOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=17&c=3
template<class D, class P>
Tagged<Union<JSFinalizationRegistry, Undefined>> TorqueGeneratedJSFinalizationRegistry<D, P>::next_dirty() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFinalizationRegistry::next_dirty(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=17&c=3
template<class D, class P>
Tagged<Union<JSFinalizationRegistry, Undefined>> TorqueGeneratedJSFinalizationRegistry<D, P>::next_dirty(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFinalizationRegistry, Undefined>> value = TaggedField<Tagged<Union<JSFinalizationRegistry, Undefined>>>::load(cage_base, *this, kNextDirtyOffset);
  DCHECK(IsUndefined(value) || IsJSFinalizationRegistry(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=17&c=3
template<class D, class P>
void TorqueGeneratedJSFinalizationRegistry<D, P>::set_next_dirty(Tagged<Union<JSFinalizationRegistry, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsJSFinalizationRegistry(value)));
  WRITE_FIELD(*this, kNextDirtyOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kNextDirtyOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=18&c=3
template<class D, class P>
int TorqueGeneratedJSFinalizationRegistry<D, P>::flags() const {
  int value = TaggedField<Smi>::load(*this, kFlagsOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=18&c=3
template<class D, class P>
void TorqueGeneratedJSFinalizationRegistry<D, P>::set_flags(int value) {
  // bitfield struct FinalizationRegistryFlags
  WRITE_FIELD(*this, kFlagsOffset, Smi::From31BitPattern(value));
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=44&c=3
template<class D, class P>
Tagged<Union<JSReceiver, Symbol, Undefined>> TorqueGeneratedJSWeakRef<D, P>::target() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSWeakRef::target(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=44&c=3
template<class D, class P>
Tagged<Union<JSReceiver, Symbol, Undefined>> TorqueGeneratedJSWeakRef<D, P>::target(PtrComprCageBase cage_base) const {
  Tagged<Union<JSReceiver, Symbol, Undefined>> value = TaggedField<Tagged<Union<JSReceiver, Symbol, Undefined>>>::load(cage_base, *this, kTargetOffset);
  DCHECK(IsJSReceiver(value) || IsUndefined(value) || IsSymbol(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=44&c=3
template<class D, class P>
void TorqueGeneratedJSWeakRef<D, P>::set_target(Tagged<Union<JSReceiver, Symbol, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsUndefined(value) || IsSymbol(value)));
  WRITE_FIELD(*this, kTargetOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kTargetOffset, value, mode);
}

