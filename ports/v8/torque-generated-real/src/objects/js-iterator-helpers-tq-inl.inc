// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=16&c=3
template<class D, class P>
int TorqueGeneratedJSIteratorHelper<D, P>::state() const {
  int value = TaggedField<Smi>::load(*this, kStateOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=16&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorHelper<D, P>::set_state(int value) {
  // JSIteratorHelperState
  WRITE_FIELD(*this, kStateOffset, Smi::From31BitPattern(value));
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
template<class D, class P>
Tagged<JSReceiver> TorqueGeneratedJSIteratorHelperSimple<D, P>::underlying_iterator_object() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorHelperSimple::underlying_iterator_object(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
template<class D, class P>
Tagged<JSReceiver> TorqueGeneratedJSIteratorHelperSimple<D, P>::underlying_iterator_object(PtrComprCageBase cage_base) const {
  Tagged<JSReceiver> value = TaggedField<Tagged<JSReceiver>>::load(cage_base, *this, kUnderlyingIteratorOffset + 0);
  DCHECK(IsJSReceiver(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorHelperSimple<D, P>::set_underlying_iterator_object(Tagged<JSReceiver> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value)));
  WRITE_FIELD(*this, kUnderlyingIteratorOffset + 0, value);
  CONDITIONAL_WRITE_BARRIER(*this, kUnderlyingIteratorOffset + 0, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSIteratorHelperSimple<D, P>::underlying_iterator_next() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorHelperSimple::underlying_iterator_next(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSIteratorHelperSimple<D, P>::underlying_iterator_next(PtrComprCageBase cage_base) const {
  Tagged<JSAny> value = TaggedField<Tagged<JSAny>>::load(cage_base, *this, kUnderlyingIteratorOffset + 4);
  DCHECK(IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorHelperSimple<D, P>::set_underlying_iterator_next(Tagged<JSAny> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value)));
  WRITE_FIELD(*this, kUnderlyingIteratorOffset + 4, value);
  CONDITIONAL_WRITE_BARRIER(*this, kUnderlyingIteratorOffset + 4, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=25&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSIteratorMapHelper<D, P>::mapper() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorMapHelper::mapper(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=25&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSIteratorMapHelper<D, P>::mapper(PtrComprCageBase cage_base) const {
  Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value = TaggedField<Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>>::load(cage_base, *this, kMapperOffset);
  DCHECK(IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=25&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorMapHelper<D, P>::set_mapper(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value)));
  WRITE_FIELD(*this, kMapperOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kMapperOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=26&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorMapHelper<D, P>::counter() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorMapHelper::counter(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=26&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorMapHelper<D, P>::counter(PtrComprCageBase cage_base) const {
  Tagged<Number> value = TaggedField<Tagged<Number>>::load(cage_base, *this, kCounterOffset);
  DCHECK(IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=26&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorMapHelper<D, P>::set_counter(Tagged<Number> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kCounterOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCounterOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=30&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSIteratorFilterHelper<D, P>::predicate() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorFilterHelper::predicate(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=30&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSIteratorFilterHelper<D, P>::predicate(PtrComprCageBase cage_base) const {
  Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value = TaggedField<Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>>::load(cage_base, *this, kPredicateOffset);
  DCHECK(IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=30&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorFilterHelper<D, P>::set_predicate(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value)));
  WRITE_FIELD(*this, kPredicateOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPredicateOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=31&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorFilterHelper<D, P>::counter() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorFilterHelper::counter(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=31&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorFilterHelper<D, P>::counter(PtrComprCageBase cage_base) const {
  Tagged<Number> value = TaggedField<Tagged<Number>>::load(cage_base, *this, kCounterOffset);
  DCHECK(IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=31&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorFilterHelper<D, P>::set_counter(Tagged<Number> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kCounterOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCounterOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=35&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorTakeHelper<D, P>::remaining() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorTakeHelper::remaining(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=35&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorTakeHelper<D, P>::remaining(PtrComprCageBase cage_base) const {
  Tagged<Number> value = TaggedField<Tagged<Number>>::load(cage_base, *this, kRemainingOffset);
  DCHECK(IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=35&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorTakeHelper<D, P>::set_remaining(Tagged<Number> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kRemainingOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kRemainingOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=39&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorDropHelper<D, P>::remaining() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorDropHelper::remaining(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=39&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorDropHelper<D, P>::remaining(PtrComprCageBase cage_base) const {
  Tagged<Number> value = TaggedField<Tagged<Number>>::load(cage_base, *this, kRemainingOffset);
  DCHECK(IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=39&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorDropHelper<D, P>::set_remaining(Tagged<Number> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kRemainingOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kRemainingOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=43&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSIteratorFlatMapHelper<D, P>::mapper() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorFlatMapHelper::mapper(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=43&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSIteratorFlatMapHelper<D, P>::mapper(PtrComprCageBase cage_base) const {
  Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value = TaggedField<Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>>::load(cage_base, *this, kMapperOffset);
  DCHECK(IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=43&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorFlatMapHelper<D, P>::set_mapper(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value)));
  WRITE_FIELD(*this, kMapperOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kMapperOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=44&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorFlatMapHelper<D, P>::counter() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorFlatMapHelper::counter(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=44&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedJSIteratorFlatMapHelper<D, P>::counter(PtrComprCageBase cage_base) const {
  Tagged<Number> value = TaggedField<Tagged<Number>>::load(cage_base, *this, kCounterOffset);
  DCHECK(IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=44&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorFlatMapHelper<D, P>::set_counter(Tagged<Number> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kCounterOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCounterOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
template<class D, class P>
Tagged<JSReceiver> TorqueGeneratedJSIteratorFlatMapHelper<D, P>::inner_iterator_object() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorFlatMapHelper::inner_iterator_object(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
template<class D, class P>
Tagged<JSReceiver> TorqueGeneratedJSIteratorFlatMapHelper<D, P>::inner_iterator_object(PtrComprCageBase cage_base) const {
  Tagged<JSReceiver> value = TaggedField<Tagged<JSReceiver>>::load(cage_base, *this, kInnerIteratorOffset + 0);
  DCHECK(IsJSReceiver(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorFlatMapHelper<D, P>::set_inner_iterator_object(Tagged<JSReceiver> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value)));
  WRITE_FIELD(*this, kInnerIteratorOffset + 0, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInnerIteratorOffset + 0, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSIteratorFlatMapHelper<D, P>::inner_iterator_next() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorFlatMapHelper::inner_iterator_next(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSIteratorFlatMapHelper<D, P>::inner_iterator_next(PtrComprCageBase cage_base) const {
  Tagged<JSAny> value = TaggedField<Tagged<JSAny>>::load(cage_base, *this, kInnerIteratorOffset + 4);
  DCHECK(IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorFlatMapHelper<D, P>::set_inner_iterator_next(Tagged<JSAny> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value)));
  WRITE_FIELD(*this, kInnerIteratorOffset + 4, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInnerIteratorOffset + 4, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=49&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSIteratorConcatHelper<D, P>::iterables() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorConcatHelper::iterables(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=49&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSIteratorConcatHelper<D, P>::iterables(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kIterablesOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=49&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorConcatHelper<D, P>::set_iterables(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kIterablesOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIterablesOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=50&c=3
template<class D, class P>
int TorqueGeneratedJSIteratorConcatHelper<D, P>::current() const {
  int value = TaggedField<Smi>::load(*this, kCurrentOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=50&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorConcatHelper<D, P>::set_current(int value) {
  WRITE_FIELD(*this, kCurrentOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=60&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSIteratorZipHelper<D, P>::underlying_iterators() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorZipHelper::underlying_iterators(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=60&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSIteratorZipHelper<D, P>::underlying_iterators(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kUnderlyingIteratorsOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=60&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorZipHelper<D, P>::set_underlying_iterators(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kUnderlyingIteratorsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kUnderlyingIteratorsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=61&c=3
template<class D, class P>
int TorqueGeneratedJSIteratorZipHelper<D, P>::mode() const {
  int value = TaggedField<Smi>::load(*this, kModeOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=61&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorZipHelper<D, P>::set_mode(int value) {
  // JSIteratorZipHelperMode
  WRITE_FIELD(*this, kModeOffset, Smi::From31BitPattern(value));
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=62&c=3
template<class D, class P>
int TorqueGeneratedJSIteratorZipHelper<D, P>::active_count() const {
  int value = TaggedField<Smi>::load(*this, kActiveCountOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=62&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorZipHelper<D, P>::set_active_count(int value) {
  WRITE_FIELD(*this, kActiveCountOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=63&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSIteratorZipHelper<D, P>::padding() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSIteratorZipHelper::padding(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=63&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSIteratorZipHelper<D, P>::padding(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kPaddingOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=63&c=3
template<class D, class P>
void TorqueGeneratedJSIteratorZipHelper<D, P>::set_padding(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kPaddingOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPaddingOffset, value, mode);
}

