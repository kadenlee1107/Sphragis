// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=59&c=3
template<class D, class P>
Tagged<JSReceiver> TorqueGeneratedSortState<D, P>::receiver() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSortState::receiver(cage_base);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=59&c=3
template<class D, class P>
Tagged<JSReceiver> TorqueGeneratedSortState<D, P>::receiver(PtrComprCageBase cage_base) const {
  Tagged<JSReceiver> value = TaggedField<Tagged<JSReceiver>>::load(cage_base, *this, kReceiverOffset);
  DCHECK(IsJSReceiver(value));
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=59&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_receiver(Tagged<JSReceiver> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value)));
  WRITE_FIELD(*this, kReceiverOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kReceiverOffset, value, mode);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=64&c=3
template<class D, class P>
Tagged<Map> TorqueGeneratedSortState<D, P>::initialReceiverMap() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSortState::initialReceiverMap(cage_base);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=64&c=3
template<class D, class P>
Tagged<Map> TorqueGeneratedSortState<D, P>::initialReceiverMap(PtrComprCageBase cage_base) const {
  Tagged<Map> value = TaggedField<Tagged<Map>>::load(cage_base, *this, kInitialReceiverMapOffset);
  DCHECK(IsMap(value));
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=64&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_initialReceiverMap(Tagged<Map> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsMap(value)));
  WRITE_FIELD(*this, kInitialReceiverMapOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInitialReceiverMapOffset, value, mode);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=65&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedSortState<D, P>::initialReceiverLength() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSortState::initialReceiverLength(cage_base);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=65&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedSortState<D, P>::initialReceiverLength(PtrComprCageBase cage_base) const {
  Tagged<Number> value = TaggedField<Tagged<Number>>::load(cage_base, *this, kInitialReceiverLengthOffset);
  DCHECK(IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=65&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_initialReceiverLength(Tagged<Number> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kInitialReceiverLengthOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInitialReceiverLengthOffset, value, mode);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=68&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> TorqueGeneratedSortState<D, P>::userCmpFn() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSortState::userCmpFn(cage_base);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=68&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> TorqueGeneratedSortState<D, P>::userCmpFn(PtrComprCageBase cage_base) const {
  Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> value = TaggedField<Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>>>::load(cage_base, *this, kUserCmpFnOffset);
  DCHECK(IsJSFunction(value) || IsUndefined(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value));
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=68&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_userCmpFn(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsUndefined(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value)));
  WRITE_FIELD(*this, kUserCmpFnOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kUserCmpFnOffset, value, mode);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=70&c=3
template<class D, class P>
Tagged<Boolean> TorqueGeneratedSortState<D, P>::isResetToGeneric() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSortState::isResetToGeneric(cage_base);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=70&c=3
template<class D, class P>
Tagged<Boolean> TorqueGeneratedSortState<D, P>::isResetToGeneric(PtrComprCageBase cage_base) const {
  Tagged<Boolean> value = TaggedField<Tagged<Boolean>>::load(cage_base, *this, kIsResetToGenericOffset);
  DCHECK(IsBoolean(value));
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=70&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_isResetToGeneric(Tagged<Boolean> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsBoolean(value)));
  WRITE_FIELD(*this, kIsResetToGenericOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIsResetToGenericOffset, value, mode);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=75&c=3
template<class D, class P>
int TorqueGeneratedSortState<D, P>::minGallop() const {
  int value = TaggedField<Smi>::load(*this, kMinGallopOffset).value();
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=75&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_minGallop(int value) {
  WRITE_FIELD(*this, kMinGallopOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=87&c=3
template<class D, class P>
int TorqueGeneratedSortState<D, P>::pendingRunsSize() const {
  int value = TaggedField<Smi>::load(*this, kPendingRunsSizeOffset).value();
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=87&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_pendingRunsSize(int value) {
  WRITE_FIELD(*this, kPendingRunsSizeOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=88&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedSortState<D, P>::pendingRuns() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSortState::pendingRuns(cage_base);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=88&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedSortState<D, P>::pendingRuns(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kPendingRunsOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=88&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_pendingRuns(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kPendingRunsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPendingRunsOffset, value, mode);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=93&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedSortState<D, P>::workArray() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSortState::workArray(cage_base);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=93&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedSortState<D, P>::workArray(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kWorkArrayOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=93&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_workArray(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kWorkArrayOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kWorkArrayOffset, value, mode);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=96&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedSortState<D, P>::tempArray() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSortState::tempArray(cage_base);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=96&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedSortState<D, P>::tempArray(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kTempArrayOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=96&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_tempArray(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kTempArrayOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kTempArrayOffset, value, mode);
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=99&c=3
template<class D, class P>
int TorqueGeneratedSortState<D, P>::sortLength() const {
  int value = TaggedField<Smi>::load(*this, kSortLengthOffset).value();
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=99&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_sortLength(int value) {
  WRITE_FIELD(*this, kSortLengthOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=103&c=3
template<class D, class P>
int TorqueGeneratedSortState<D, P>::numberOfUndefined() const {
  int value = TaggedField<Smi>::load(*this, kNumberOfUndefinedOffset).value();
  return value;
}

// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=103&c=3
template<class D, class P>
void TorqueGeneratedSortState<D, P>::set_numberOfUndefined(int value) {
  WRITE_FIELD(*this, kNumberOfUndefinedOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=769&c=3
template<class D, class P>
int TorqueGeneratedInternalClass<D, P>::a() const {
  int value = TaggedField<Smi>::load(*this, kAOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=769&c=3
template<class D, class P>
void TorqueGeneratedInternalClass<D, P>::set_a(int value) {
  WRITE_FIELD(*this, kAOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=770&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedInternalClass<D, P>::b() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedInternalClass::b(cage_base);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=770&c=3
template<class D, class P>
Tagged<Number> TorqueGeneratedInternalClass<D, P>::b(PtrComprCageBase cage_base) const {
  Tagged<Number> value = TaggedField<Tagged<Number>>::load(cage_base, *this, kBOffset);
  DCHECK(IsSmi(value) || IsHeapNumber(value));
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=770&c=3
template<class D, class P>
void TorqueGeneratedInternalClass<D, P>::set_b(Tagged<Number> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSmi(value) || IsHeapNumber(value)));
  WRITE_FIELD(*this, kBOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBOffset, value, mode);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=833&c=3
template<class D, class P>
int TorqueGeneratedSmiPair<D, P>::a() const {
  int value = TaggedField<Smi>::load(*this, kAOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=833&c=3
template<class D, class P>
void TorqueGeneratedSmiPair<D, P>::set_a(int value) {
  WRITE_FIELD(*this, kAOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=834&c=3
template<class D, class P>
int TorqueGeneratedSmiPair<D, P>::b() const {
  int value = TaggedField<Smi>::load(*this, kBOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=834&c=3
template<class D, class P>
void TorqueGeneratedSmiPair<D, P>::set_b(int value) {
  WRITE_FIELD(*this, kBOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=931&c=3
template<class D, class P>
int TorqueGeneratedSmiBox<D, P>::value() const {
  int value = TaggedField<Smi>::load(*this, kValueOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=931&c=3
template<class D, class P>
void TorqueGeneratedSmiBox<D, P>::set_value(int value) {
  WRITE_FIELD(*this, kValueOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=932&c=3
template<class D, class P>
int TorqueGeneratedSmiBox<D, P>::unrelated() const {
  int value = TaggedField<Smi>::load(*this, kUnrelatedOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=932&c=3
template<class D, class P>
void TorqueGeneratedSmiBox<D, P>::set_unrelated(int value) {
  WRITE_FIELD(*this, kUnrelatedOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1183&c=3
template<class D, class P>
Tagged<HeapObject> TorqueGeneratedExportedSubClassBase<D, P>::a() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedExportedSubClassBase::a(cage_base);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1183&c=3
template<class D, class P>
Tagged<HeapObject> TorqueGeneratedExportedSubClassBase<D, P>::a(PtrComprCageBase cage_base) const {
  Tagged<HeapObject> value = TaggedField<Tagged<HeapObject>>::load(cage_base, *this, kAOffset);
  DCHECK(IsHeapObject(value));
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1183&c=3
template<class D, class P>
void TorqueGeneratedExportedSubClassBase<D, P>::set_a(Tagged<HeapObject> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value)));
  WRITE_FIELD(*this, kAOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kAOffset, value, mode);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1184&c=3
template<class D, class P>
Tagged<HeapObject> TorqueGeneratedExportedSubClassBase<D, P>::b() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedExportedSubClassBase::b(cage_base);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1184&c=3
template<class D, class P>
Tagged<HeapObject> TorqueGeneratedExportedSubClassBase<D, P>::b(PtrComprCageBase cage_base) const {
  Tagged<HeapObject> value = TaggedField<Tagged<HeapObject>>::load(cage_base, *this, kBOffset);
  DCHECK(IsHeapObject(value));
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1184&c=3
template<class D, class P>
void TorqueGeneratedExportedSubClassBase<D, P>::set_b(Tagged<HeapObject> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value)));
  WRITE_FIELD(*this, kBOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBOffset, value, mode);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1176&c=3
template<class D, class P>
int32_t TorqueGeneratedExportedSubClass<D, P>::c_field() const {
  int32_t value = this->template ReadField<int32_t>(kCFieldOffset);
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1176&c=3
template<class D, class P>
void TorqueGeneratedExportedSubClass<D, P>::set_c_field(int32_t value) {
  this->template WriteField<int32_t>(kCFieldOffset, value);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1177&c=3
template<class D, class P>
int32_t TorqueGeneratedExportedSubClass<D, P>::d_field() const {
  int32_t value = this->template ReadField<int32_t>(kDFieldOffset);
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1177&c=3
template<class D, class P>
void TorqueGeneratedExportedSubClass<D, P>::set_d_field(int32_t value) {
  this->template WriteField<int32_t>(kDFieldOffset, value);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1178&c=3
template<class D, class P>
int TorqueGeneratedExportedSubClass<D, P>::e_field() const {
  int value = TaggedField<Smi>::load(*this, kEFieldOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1178&c=3
template<class D, class P>
void TorqueGeneratedExportedSubClass<D, P>::set_e_field(int value) {
  WRITE_FIELD(*this, kEFieldOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1200&c=3
template<class D, class P>
int32_t TorqueGeneratedInternalClassWithStructElements<D, P>::dummy1() const {
  int32_t value = this->template ReadField<int32_t>(kDummy1Offset);
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1200&c=3
template<class D, class P>
void TorqueGeneratedInternalClassWithStructElements<D, P>::set_dummy1(int32_t value) {
  this->template WriteField<int32_t>(kDummy1Offset, value);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1201&c=3
template<class D, class P>
int32_t TorqueGeneratedInternalClassWithStructElements<D, P>::dummy2() const {
  int32_t value = this->template ReadField<int32_t>(kDummy2Offset);
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1201&c=3
template<class D, class P>
void TorqueGeneratedInternalClassWithStructElements<D, P>::set_dummy2(int32_t value) {
  this->template WriteField<int32_t>(kDummy2Offset, value);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1202&c=9
template<class D, class P>
int TorqueGeneratedInternalClassWithStructElements<D, P>::count() const {
  int value = TaggedField<Smi>::load(*this, kCountOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1202&c=9
template<class D, class P>
void TorqueGeneratedInternalClassWithStructElements<D, P>::set_count(int value) {
  WRITE_FIELD(*this, kCountOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1203&c=3
template<class D, class P>
int TorqueGeneratedInternalClassWithStructElements<D, P>::data() const {
  int value = TaggedField<Smi>::load(*this, kDataOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1203&c=3
template<class D, class P>
void TorqueGeneratedInternalClassWithStructElements<D, P>::set_data(int value) {
  WRITE_FIELD(*this, kDataOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1204&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedInternalClassWithStructElements<D, P>::object() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedInternalClassWithStructElements::object(cage_base);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1204&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedInternalClassWithStructElements<D, P>::object(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kObjectOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1204&c=3
template<class D, class P>
void TorqueGeneratedInternalClassWithStructElements<D, P>::set_object(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kObjectOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kObjectOffset, value, mode);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1205&c=3
template<class D, class P>
int TorqueGeneratedInternalClassWithStructElements<D, P>::entries(int i) const {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->count());
  int offset = kEntriesOffset + i * kTaggedSize;
  int value = TaggedField<Smi>::load(*this, offset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1205&c=3
template<class D, class P>
void TorqueGeneratedInternalClassWithStructElements<D, P>::set_entries(int i, int value) {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->count());
  int offset = kEntriesOffset + i * kTaggedSize;
  WRITE_FIELD(*this, offset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
template<class D, class P>
int TorqueGeneratedInternalClassWithStructElements<D, P>::more_entries_a(int i) const {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->count());
  int offset = MoreEntriesOffset() + 0 + i * 8;
  int value = TaggedField<Smi>::load(*this, offset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
template<class D, class P>
void TorqueGeneratedInternalClassWithStructElements<D, P>::set_more_entries_a(int i, int value) {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->count());
  int offset = MoreEntriesOffset() + 0 + i * 8;
  WRITE_FIELD(*this, offset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
template<class D, class P>
int TorqueGeneratedInternalClassWithStructElements<D, P>::more_entries_b(int i) const {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->count());
  int offset = MoreEntriesOffset() + 4 + i * 8;
  int value = TaggedField<Smi>::load(*this, offset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
template<class D, class P>
void TorqueGeneratedInternalClassWithStructElements<D, P>::set_more_entries_b(int i, int value) {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->count());
  int offset = MoreEntriesOffset() + 4 + i * 8;
  WRITE_FIELD(*this, offset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
template<class D, class P>
int TorqueGeneratedInternalClassWithStructElements<D, P>::MoreEntriesOffset() const {
  return static_cast<int>(std::get<1>(TqRuntimeFieldSliceInternalClassWithStructElementsMoreEntries(*static_cast<const D*>(this))));
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1218&c=3
template<class D, class P>
int32_t TorqueGeneratedExportedSubClass2<D, P>::x_field() const {
  int32_t value = this->template ReadField<int32_t>(kXFieldOffset);
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1218&c=3
template<class D, class P>
void TorqueGeneratedExportedSubClass2<D, P>::set_x_field(int32_t value) {
  this->template WriteField<int32_t>(kXFieldOffset, value);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1219&c=3
template<class D, class P>
int32_t TorqueGeneratedExportedSubClass2<D, P>::y_field() const {
  int32_t value = this->template ReadField<int32_t>(kYFieldOffset);
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1219&c=3
template<class D, class P>
void TorqueGeneratedExportedSubClass2<D, P>::set_y_field(int32_t value) {
  this->template WriteField<int32_t>(kYFieldOffset, value);
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1220&c=3
template<class D, class P>
int TorqueGeneratedExportedSubClass2<D, P>::z_field() const {
  int value = TaggedField<Smi>::load(*this, kZFieldOffset).value();
  return value;
}

// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1220&c=3
template<class D, class P>
void TorqueGeneratedExportedSubClass2<D, P>::set_z_field(int value) {
  WRITE_FIELD(*this, kZFieldOffset, Smi::FromInt(value));
}

