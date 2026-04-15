// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=6&c=3
template<class D, class P>
Tagged<JSFunction> TorqueGeneratedJSGeneratorObject<D, P>::function() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSGeneratorObject::function(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=6&c=3
template<class D, class P>
Tagged<JSFunction> TorqueGeneratedJSGeneratorObject<D, P>::function(PtrComprCageBase cage_base) const {
  Tagged<JSFunction> value = TaggedField<Tagged<JSFunction>>::load(cage_base, *this, kFunctionOffset);
  DCHECK(IsJSFunction(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=6&c=3
template<class D, class P>
void TorqueGeneratedJSGeneratorObject<D, P>::set_function(Tagged<JSFunction> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value)));
  WRITE_FIELD(*this, kFunctionOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kFunctionOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=7&c=3
template<class D, class P>
Tagged<Context> TorqueGeneratedJSGeneratorObject<D, P>::context() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSGeneratorObject::context(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=7&c=3
template<class D, class P>
Tagged<Context> TorqueGeneratedJSGeneratorObject<D, P>::context(PtrComprCageBase cage_base) const {
  Tagged<Context> value = TaggedField<Tagged<Context>>::load(cage_base, *this, kContextOffset);
  DCHECK(IsContext(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=7&c=3
template<class D, class P>
void TorqueGeneratedJSGeneratorObject<D, P>::set_context(Tagged<Context> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsContext(value)));
  WRITE_FIELD(*this, kContextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kContextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=8&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSGeneratorObject<D, P>::receiver() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSGeneratorObject::receiver(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=8&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSGeneratorObject<D, P>::receiver(PtrComprCageBase cage_base) const {
  Tagged<JSAny> value = TaggedField<Tagged<JSAny>>::load(cage_base, *this, kReceiverOffset);
  DCHECK(IsBigInt(value) || IsJSReceiver(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=8&c=3
template<class D, class P>
void TorqueGeneratedJSGeneratorObject<D, P>::set_receiver(Tagged<JSAny> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsBigInt(value) || IsJSReceiver(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value)));
  WRITE_FIELD(*this, kReceiverOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kReceiverOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=14&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedJSGeneratorObject<D, P>::input_or_debug_pos() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSGeneratorObject::input_or_debug_pos(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=14&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedJSGeneratorObject<D, P>::input_or_debug_pos(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kInputOrDebugPosOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=14&c=3
template<class D, class P>
void TorqueGeneratedJSGeneratorObject<D, P>::set_input_or_debug_pos(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kInputOrDebugPosOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInputOrDebugPosOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=17&c=3
template<class D, class P>
int TorqueGeneratedJSGeneratorObject<D, P>::resume_mode() const {
  int value = TaggedField<Smi>::load(*this, kResumeModeOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=17&c=3
template<class D, class P>
void TorqueGeneratedJSGeneratorObject<D, P>::set_resume_mode(int value) {
  WRITE_FIELD(*this, kResumeModeOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=22&c=3
template<class D, class P>
int TorqueGeneratedJSGeneratorObject<D, P>::continuation() const {
  int value = TaggedField<Smi>::load(*this, kContinuationOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=22&c=3
template<class D, class P>
void TorqueGeneratedJSGeneratorObject<D, P>::set_continuation(int value) {
  WRITE_FIELD(*this, kContinuationOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=25&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSGeneratorObject<D, P>::parameters_and_registers() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSGeneratorObject::parameters_and_registers(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=25&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSGeneratorObject<D, P>::parameters_and_registers(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kParametersAndRegistersOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=25&c=3
template<class D, class P>
void TorqueGeneratedJSGeneratorObject<D, P>::set_parameters_and_registers(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kParametersAndRegistersOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kParametersAndRegistersOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=29&c=3
template<class D, class P>
Tagged<JSPromise> TorqueGeneratedJSAsyncFunctionObject<D, P>::promise() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSAsyncFunctionObject::promise(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=29&c=3
template<class D, class P>
Tagged<JSPromise> TorqueGeneratedJSAsyncFunctionObject<D, P>::promise(PtrComprCageBase cage_base) const {
  Tagged<JSPromise> value = TaggedField<Tagged<JSPromise>>::load(cage_base, *this, kPromiseOffset);
  DCHECK(IsJSPromise(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=29&c=3
template<class D, class P>
void TorqueGeneratedJSAsyncFunctionObject<D, P>::set_promise(Tagged<JSPromise> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSPromise(value)));
  WRITE_FIELD(*this, kPromiseOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPromiseOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=32&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSAsyncFunctionObject<D, P>::await_resolve_closure() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSAsyncFunctionObject::await_resolve_closure(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=32&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSAsyncFunctionObject<D, P>::await_resolve_closure(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFunction, Undefined>> value = TaggedField<Tagged<Union<JSFunction, Undefined>>>::load(cage_base, *this, kAwaitResolveClosureOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=32&c=3
template<class D, class P>
void TorqueGeneratedJSAsyncFunctionObject<D, P>::set_await_resolve_closure(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kAwaitResolveClosureOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kAwaitResolveClosureOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=33&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSAsyncFunctionObject<D, P>::await_reject_closure() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSAsyncFunctionObject::await_reject_closure(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=33&c=3
template<class D, class P>
Tagged<Union<JSFunction, Undefined>> TorqueGeneratedJSAsyncFunctionObject<D, P>::await_reject_closure(PtrComprCageBase cage_base) const {
  Tagged<Union<JSFunction, Undefined>> value = TaggedField<Tagged<Union<JSFunction, Undefined>>>::load(cage_base, *this, kAwaitRejectClosureOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=33&c=3
template<class D, class P>
void TorqueGeneratedJSAsyncFunctionObject<D, P>::set_await_reject_closure(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value)));
  WRITE_FIELD(*this, kAwaitRejectClosureOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kAwaitRejectClosureOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=39&c=3
template<class D, class P>
Tagged<HeapObject> TorqueGeneratedJSAsyncGeneratorObject<D, P>::queue() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSAsyncGeneratorObject::queue(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=39&c=3
template<class D, class P>
Tagged<HeapObject> TorqueGeneratedJSAsyncGeneratorObject<D, P>::queue(PtrComprCageBase cage_base) const {
  Tagged<HeapObject> value = TaggedField<Tagged<HeapObject>>::load(cage_base, *this, kQueueOffset);
  DCHECK(IsHeapObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=39&c=3
template<class D, class P>
void TorqueGeneratedJSAsyncGeneratorObject<D, P>::set_queue(Tagged<HeapObject> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value)));
  WRITE_FIELD(*this, kQueueOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kQueueOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=41&c=3
template<class D, class P>
int TorqueGeneratedJSAsyncGeneratorObject<D, P>::is_awaiting() const {
  int value = TaggedField<Smi>::load(*this, kIsAwaitingOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=41&c=3
template<class D, class P>
void TorqueGeneratedJSAsyncGeneratorObject<D, P>::set_is_awaiting(int value) {
  WRITE_FIELD(*this, kIsAwaitingOffset, Smi::FromInt(value));
}

