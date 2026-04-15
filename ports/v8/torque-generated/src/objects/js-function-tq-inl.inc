// https://crsrc.org/c/v8/src/objects/js-function.tq?l=34&c=3
template<class D, class P>
int32_t TorqueGeneratedJSFunction<D, P>::dispatch_handle() const {
  int32_t value = this->template ReadField<int32_t>(kDispatchHandleOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=34&c=3
template<class D, class P>
void TorqueGeneratedJSFunction<D, P>::set_dispatch_handle(int32_t value) {
  this->template WriteField<int32_t>(kDispatchHandleOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=36&c=3
template<class D, class P>
Tagged<SharedFunctionInfo> TorqueGeneratedJSFunction<D, P>::shared_function_info() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFunction::shared_function_info(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=36&c=3
template<class D, class P>
Tagged<SharedFunctionInfo> TorqueGeneratedJSFunction<D, P>::shared_function_info(PtrComprCageBase cage_base) const {
  Tagged<SharedFunctionInfo> value = TaggedField<Tagged<SharedFunctionInfo>>::load(cage_base, *this, kSharedFunctionInfoOffset);
  DCHECK(IsSharedFunctionInfo(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=36&c=3
template<class D, class P>
void TorqueGeneratedJSFunction<D, P>::set_shared_function_info(Tagged<SharedFunctionInfo> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsSharedFunctionInfo(value)));
  WRITE_FIELD(*this, kSharedFunctionInfoOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kSharedFunctionInfoOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=37&c=3
template<class D, class P>
Tagged<Context> TorqueGeneratedJSFunction<D, P>::context() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFunction::context(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=37&c=3
template<class D, class P>
Tagged<Context> TorqueGeneratedJSFunction<D, P>::context(PtrComprCageBase cage_base) const {
  Tagged<Context> value = TaggedField<Tagged<Context>>::load(cage_base, *this, kContextOffset);
  DCHECK(IsContext(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=37&c=3
template<class D, class P>
void TorqueGeneratedJSFunction<D, P>::set_context(Tagged<Context> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsContext(value)));
  WRITE_FIELD(*this, kContextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kContextOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=38&c=3
template<class D, class P>
Tagged<FeedbackCell> TorqueGeneratedJSFunction<D, P>::feedback_cell() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFunction::feedback_cell(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=38&c=3
template<class D, class P>
Tagged<FeedbackCell> TorqueGeneratedJSFunction<D, P>::feedback_cell(PtrComprCageBase cage_base) const {
  Tagged<FeedbackCell> value = TaggedField<Tagged<FeedbackCell>>::load(cage_base, *this, kFeedbackCellOffset);
  DCHECK(IsFeedbackCell(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=38&c=3
template<class D, class P>
void TorqueGeneratedJSFunction<D, P>::set_feedback_cell(Tagged<FeedbackCell> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFeedbackCell(value)));
  WRITE_FIELD(*this, kFeedbackCellOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kFeedbackCellOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=57&c=3
template<class D, class P>
Tagged<Union<JSReceiver, Map, TheHole>> TorqueGeneratedJSFunctionWithPrototype<D, P>::prototype_or_initial_map() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSFunctionWithPrototype::prototype_or_initial_map(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=57&c=3
template<class D, class P>
Tagged<Union<JSReceiver, Map, TheHole>> TorqueGeneratedJSFunctionWithPrototype<D, P>::prototype_or_initial_map(PtrComprCageBase cage_base) const {
  Tagged<Union<JSReceiver, Map, TheHole>> value = TaggedField<Tagged<Union<JSReceiver, Map, TheHole>>>::load(cage_base, *this, kPrototypeOrInitialMapOffset);
  DCHECK(IsJSReceiver(value) || IsMap(value) || IsTheHole(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=57&c=3
template<class D, class P>
void TorqueGeneratedJSFunctionWithPrototype<D, P>::set_prototype_or_initial_map(Tagged<Union<JSReceiver, Map, TheHole>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsMap(value) || IsTheHole(value)));
  WRITE_FIELD(*this, kPrototypeOrInitialMapOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kPrototypeOrInitialMapOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=11&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSBoundFunction<D, P>::bound_target_function() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSBoundFunction::bound_target_function(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=11&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSBoundFunction<D, P>::bound_target_function(PtrComprCageBase cage_base) const {
  Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value = TaggedField<Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>>::load(cage_base, *this, kBoundTargetFunctionOffset);
  DCHECK(IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=11&c=3
template<class D, class P>
void TorqueGeneratedJSBoundFunction<D, P>::set_bound_target_function(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value)));
  WRITE_FIELD(*this, kBoundTargetFunctionOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBoundTargetFunctionOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=14&c=3
template<class D, class P>
Tagged<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, SourceTextModule, String, Symbol, Undefined>> TorqueGeneratedJSBoundFunction<D, P>::bound_this() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSBoundFunction::bound_this(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=14&c=3
template<class D, class P>
Tagged<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, SourceTextModule, String, Symbol, Undefined>> TorqueGeneratedJSBoundFunction<D, P>::bound_this(PtrComprCageBase cage_base) const {
  Tagged<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, SourceTextModule, String, Symbol, Undefined>> value = TaggedField<Tagged<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, SourceTextModule, String, Symbol, Undefined>>>::load(cage_base, *this, kBoundThisOffset);
  DCHECK(IsBigInt(value) || IsJSReceiver(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value) || IsSourceTextModule(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=14&c=3
template<class D, class P>
void TorqueGeneratedJSBoundFunction<D, P>::set_bound_this(Tagged<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, SourceTextModule, String, Symbol, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsBigInt(value) || IsJSReceiver(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value) || IsSourceTextModule(value)));
  WRITE_FIELD(*this, kBoundThisOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBoundThisOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=17&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSBoundFunction<D, P>::bound_arguments() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSBoundFunction::bound_arguments(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=17&c=3
template<class D, class P>
Tagged<FixedArray> TorqueGeneratedJSBoundFunction<D, P>::bound_arguments(PtrComprCageBase cage_base) const {
  Tagged<FixedArray> value = TaggedField<Tagged<FixedArray>>::load(cage_base, *this, kBoundArgumentsOffset);
  DCHECK(IsFixedArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=17&c=3
template<class D, class P>
void TorqueGeneratedJSBoundFunction<D, P>::set_bound_arguments(Tagged<FixedArray> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFixedArray(value)));
  WRITE_FIELD(*this, kBoundArgumentsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBoundArgumentsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=23&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSWrappedFunction<D, P>::wrapped_target_function() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSWrappedFunction::wrapped_target_function(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=23&c=3
template<class D, class P>
Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> TorqueGeneratedJSWrappedFunction<D, P>::wrapped_target_function(PtrComprCageBase cage_base) const {
  Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value = TaggedField<Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>>::load(cage_base, *this, kWrappedTargetFunctionOffset);
  DCHECK(IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=23&c=3
template<class D, class P>
void TorqueGeneratedJSWrappedFunction<D, P>::set_wrapped_target_function(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSFunction(value) || IsJSBoundFunction(value) || IsJSWrappedFunction(value) || IsCallableJSProxy(value) || IsCallableApiObject(value)));
  WRITE_FIELD(*this, kWrappedTargetFunctionOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kWrappedTargetFunctionOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=25&c=3
template<class D, class P>
Tagged<NativeContext> TorqueGeneratedJSWrappedFunction<D, P>::context() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSWrappedFunction::context(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=25&c=3
template<class D, class P>
Tagged<NativeContext> TorqueGeneratedJSWrappedFunction<D, P>::context(PtrComprCageBase cage_base) const {
  Tagged<NativeContext> value = TaggedField<Tagged<NativeContext>>::load(cage_base, *this, kContextOffset);
  DCHECK(IsNativeContext(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=25&c=3
template<class D, class P>
void TorqueGeneratedJSWrappedFunction<D, P>::set_context(Tagged<NativeContext> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsNativeContext(value)));
  WRITE_FIELD(*this, kContextOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kContextOffset, value, mode);
}

