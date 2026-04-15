// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=61&c=3
template<class D, class P>
Tagged<Union<Smi, Undefined>> TorqueGeneratedJSRegExp<D, P>::flags() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSRegExp::flags(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=61&c=3
template<class D, class P>
Tagged<Union<Smi, Undefined>> TorqueGeneratedJSRegExp<D, P>::flags(PtrComprCageBase cage_base) const {
  Tagged<Union<Smi, Undefined>> value = TaggedField<Tagged<Union<Smi, Undefined>>>::load(cage_base, *this, kFlagsOffset);
  DCHECK(IsUndefined(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=61&c=3
template<class D, class P>
void TorqueGeneratedJSRegExp<D, P>::set_flags(Tagged<Union<Smi, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsSmi(value)));
  WRITE_FIELD(*this, kFlagsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kFlagsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=102&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResult<D, P>::index() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSRegExpResult::index(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=102&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResult<D, P>::index(PtrComprCageBase cage_base) const {
  Tagged<JSAny> value = TaggedField<Tagged<JSAny>>::load(cage_base, *this, kIndexOffset);
  DCHECK(IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=102&c=3
template<class D, class P>
void TorqueGeneratedJSRegExpResult<D, P>::set_index(Tagged<JSAny> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value)));
  WRITE_FIELD(*this, kIndexOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIndexOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=103&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResult<D, P>::input() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSRegExpResult::input(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=103&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResult<D, P>::input(PtrComprCageBase cage_base) const {
  Tagged<JSAny> value = TaggedField<Tagged<JSAny>>::load(cage_base, *this, kInputOffset);
  DCHECK(IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=103&c=3
template<class D, class P>
void TorqueGeneratedJSRegExpResult<D, P>::set_input(Tagged<JSAny> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value)));
  WRITE_FIELD(*this, kInputOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kInputOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=104&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResult<D, P>::groups() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSRegExpResult::groups(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=104&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResult<D, P>::groups(PtrComprCageBase cage_base) const {
  Tagged<JSAny> value = TaggedField<Tagged<JSAny>>::load(cage_base, *this, kGroupsOffset);
  DCHECK(IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=104&c=3
template<class D, class P>
void TorqueGeneratedJSRegExpResult<D, P>::set_groups(Tagged<JSAny> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value)));
  WRITE_FIELD(*this, kGroupsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kGroupsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=107&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSRegExpResult<D, P>::regexp_input() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSRegExpResult::regexp_input(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=107&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedJSRegExpResult<D, P>::regexp_input(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kRegexpInputOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=107&c=3
template<class D, class P>
void TorqueGeneratedJSRegExpResult<D, P>::set_regexp_input(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kRegexpInputOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kRegexpInputOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=108&c=3
template<class D, class P>
int TorqueGeneratedJSRegExpResult<D, P>::regexp_last_index() const {
  int value = TaggedField<Smi>::load(*this, kRegexpLastIndexOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=108&c=3
template<class D, class P>
void TorqueGeneratedJSRegExpResult<D, P>::set_regexp_last_index(int value) {
  WRITE_FIELD(*this, kRegexpLastIndexOffset, Smi::FromInt(value));
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=112&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResultWithIndices<D, P>::indices() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSRegExpResultWithIndices::indices(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=112&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResultWithIndices<D, P>::indices(PtrComprCageBase cage_base) const {
  Tagged<JSAny> value = TaggedField<Tagged<JSAny>>::load(cage_base, *this, kIndicesOffset);
  DCHECK(IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=112&c=3
template<class D, class P>
void TorqueGeneratedJSRegExpResultWithIndices<D, P>::set_indices(Tagged<JSAny> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value)));
  WRITE_FIELD(*this, kIndicesOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kIndicesOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=118&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResultIndices<D, P>::groups() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedJSRegExpResultIndices::groups(cage_base);
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=118&c=3
template<class D, class P>
Tagged<JSAny> TorqueGeneratedJSRegExpResultIndices<D, P>::groups(PtrComprCageBase cage_base) const {
  Tagged<JSAny> value = TaggedField<Tagged<JSAny>>::load(cage_base, *this, kGroupsOffset);
  DCHECK(IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-regexp.tq?l=118&c=3
template<class D, class P>
void TorqueGeneratedJSRegExpResultIndices<D, P>::set_groups(Tagged<JSAny> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsJSReceiver(value) || IsBigInt(value) || IsUndefined(value) || IsSmi(value) || IsHeapNumber(value) || IsString(value) || IsSymbol(value) || IsBoolean(value) || IsNull(value)));
  WRITE_FIELD(*this, kGroupsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kGroupsOffset, value, mode);
}

