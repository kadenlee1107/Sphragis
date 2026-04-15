// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=79&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedSharedFunctionInfo<D, P>::untrusted_function_data() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSharedFunctionInfo::untrusted_function_data(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=79&c=3
template<class D, class P>
Tagged<Object> TorqueGeneratedSharedFunctionInfo<D, P>::untrusted_function_data(PtrComprCageBase cage_base) const {
  Tagged<Object> value = TaggedField<Tagged<Object>>::load(cage_base, *this, kUntrustedFunctionDataOffset);
  DCHECK(IsHeapObject(value) || IsSmi(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=79&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_untrusted_function_data(Tagged<Object> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsHeapObject(value) || IsSmi(value)));
  WRITE_FIELD(*this, kUntrustedFunctionDataOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kUntrustedFunctionDataOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=80&c=3
template<class D, class P>
Tagged<Union<ScopeInfo, Smi, String>> TorqueGeneratedSharedFunctionInfo<D, P>::name_or_scope_info() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSharedFunctionInfo::name_or_scope_info(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=80&c=3
template<class D, class P>
Tagged<Union<ScopeInfo, Smi, String>> TorqueGeneratedSharedFunctionInfo<D, P>::name_or_scope_info(PtrComprCageBase cage_base) const {
  Tagged<Union<ScopeInfo, Smi, String>> value = TaggedField<Tagged<Union<ScopeInfo, Smi, String>>>::load(cage_base, *this, kNameOrScopeInfoOffset);
  DCHECK(IsString(value) || IsScopeInfo(value) || IsNoSharedNameSentinel(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=80&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_name_or_scope_info(Tagged<Union<ScopeInfo, Smi, String>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value) || IsScopeInfo(value) || IsNoSharedNameSentinel(value)));
  WRITE_FIELD(*this, kNameOrScopeInfoOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kNameOrScopeInfoOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=81&c=3
template<class D, class P>
Tagged<Union<FeedbackMetadata, ScopeInfo, TheHole>> TorqueGeneratedSharedFunctionInfo<D, P>::outer_scope_info_or_feedback_metadata() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSharedFunctionInfo::outer_scope_info_or_feedback_metadata(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=81&c=3
template<class D, class P>
Tagged<Union<FeedbackMetadata, ScopeInfo, TheHole>> TorqueGeneratedSharedFunctionInfo<D, P>::outer_scope_info_or_feedback_metadata(PtrComprCageBase cage_base) const {
  Tagged<Union<FeedbackMetadata, ScopeInfo, TheHole>> value = TaggedField<Tagged<Union<FeedbackMetadata, ScopeInfo, TheHole>>>::load(cage_base, *this, kOuterScopeInfoOrFeedbackMetadataOffset);
  DCHECK(IsFeedbackMetadata(value) || IsTheHole(value) || IsScopeInfo(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=81&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_outer_scope_info_or_feedback_metadata(Tagged<Union<FeedbackMetadata, ScopeInfo, TheHole>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsFeedbackMetadata(value) || IsTheHole(value) || IsScopeInfo(value)));
  WRITE_FIELD(*this, kOuterScopeInfoOrFeedbackMetadataOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kOuterScopeInfoOrFeedbackMetadataOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=82&c=3
template<class D, class P>
Tagged<Union<Script, Undefined>> TorqueGeneratedSharedFunctionInfo<D, P>::script() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedSharedFunctionInfo::script(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=82&c=3
template<class D, class P>
Tagged<Union<Script, Undefined>> TorqueGeneratedSharedFunctionInfo<D, P>::script(PtrComprCageBase cage_base) const {
  Tagged<Union<Script, Undefined>> value = TaggedField<Tagged<Union<Script, Undefined>>>::load(cage_base, *this, kScriptOffset);
  DCHECK(IsUndefined(value) || IsScript(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=82&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_script(Tagged<Union<Script, Undefined>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsUndefined(value) || IsScript(value)));
  WRITE_FIELD(*this, kScriptOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kScriptOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=86&c=3
template<class D, class P>
uint16_t TorqueGeneratedSharedFunctionInfo<D, P>::length() const {
  uint16_t value = this->template ReadField<uint16_t>(kLengthOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=86&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_length(uint16_t value) {
  this->template WriteField<uint16_t>(kLengthOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=102&c=3
template<class D, class P>
uint16_t TorqueGeneratedSharedFunctionInfo<D, P>::formal_parameter_count() const {
  uint16_t value = this->template ReadField<uint16_t>(kFormalParameterCountOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=102&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_formal_parameter_count(uint16_t value) {
  this->template WriteField<uint16_t>(kFormalParameterCountOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=103&c=3
template<class D, class P>
uint16_t TorqueGeneratedSharedFunctionInfo<D, P>::function_token_offset() const {
  uint16_t value = this->template ReadField<uint16_t>(kFunctionTokenOffsetOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=103&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_function_token_offset(uint16_t value) {
  this->template WriteField<uint16_t>(kFunctionTokenOffsetOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=106&c=3
template<class D, class P>
uint8_t TorqueGeneratedSharedFunctionInfo<D, P>::expected_nof_properties() const {
  uint8_t value = this->template ReadField<uint8_t>(kExpectedNofPropertiesOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=106&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_expected_nof_properties(uint8_t value) {
  this->template WriteField<uint8_t>(kExpectedNofPropertiesOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=107&c=3
template<class D, class P>
uint8_t TorqueGeneratedSharedFunctionInfo<D, P>::flags2() const {
  uint8_t value = this->template ReadField<uint8_t>(kFlags2Offset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=107&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_flags2(uint8_t value) {
  this->template WriteField<uint8_t>(kFlags2Offset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=108&c=3
template<class D, class P>
uint32_t TorqueGeneratedSharedFunctionInfo<D, P>::flags() const {
  uint32_t value = this->template ReadField<uint32_t>(kFlagsOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=108&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_flags(uint32_t value) {
  this->template WriteField<uint32_t>(kFlagsOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=112&c=3
template<class D, class P>
int32_t TorqueGeneratedSharedFunctionInfo<D, P>::function_literal_id() const {
  int32_t value = this->template ReadField<int32_t>(kFunctionLiteralIdOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=112&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_function_literal_id(int32_t value) {
  this->template WriteField<int32_t>(kFunctionLiteralIdOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=116&c=3
template<class D, class P>
int32_t TorqueGeneratedSharedFunctionInfo<D, P>::unique_id() const {
  int32_t value = this->template ReadField<int32_t>(kUniqueIdOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=116&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_unique_id(int32_t value) {
  this->template WriteField<int32_t>(kUniqueIdOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=119&c=3
template<class D, class P>
uint16_t TorqueGeneratedSharedFunctionInfo<D, P>::age() const {
  uint16_t value = this->template ReadField<uint16_t>(kAgeOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=119&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_age(uint16_t value) {
  this->template WriteField<uint16_t>(kAgeOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=123&c=3
template<class D, class P>
uint16_t TorqueGeneratedSharedFunctionInfo<D, P>::feedback_slot() const {
  uint16_t value = this->template ReadField<uint16_t>(kFeedbackSlotOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=123&c=3
template<class D, class P>
void TorqueGeneratedSharedFunctionInfo<D, P>::set_feedback_slot(uint16_t value) {
  this->template WriteField<uint16_t>(kFeedbackSlotOffset, value);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=201&c=3
template<class D, class P>
Tagged<FixedInt32Array> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::block_ids() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedOnHeapBasicBlockProfilerData::block_ids(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=201&c=3
template<class D, class P>
Tagged<FixedInt32Array> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::block_ids(PtrComprCageBase cage_base) const {
  Tagged<FixedInt32Array> value = TaggedField<Tagged<FixedInt32Array>>::load(cage_base, *this, kBlockIdsOffset);
  DCHECK(IsByteArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=201&c=3
template<class D, class P>
void TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::set_block_ids(Tagged<FixedInt32Array> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsByteArray(value)));
  WRITE_FIELD(*this, kBlockIdsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBlockIdsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=202&c=3
template<class D, class P>
Tagged<FixedUInt32Array> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::counts() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedOnHeapBasicBlockProfilerData::counts(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=202&c=3
template<class D, class P>
Tagged<FixedUInt32Array> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::counts(PtrComprCageBase cage_base) const {
  Tagged<FixedUInt32Array> value = TaggedField<Tagged<FixedUInt32Array>>::load(cage_base, *this, kCountsOffset);
  DCHECK(IsByteArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=202&c=3
template<class D, class P>
void TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::set_counts(Tagged<FixedUInt32Array> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsByteArray(value)));
  WRITE_FIELD(*this, kCountsOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCountsOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=203&c=3
template<class D, class P>
Tagged<PodArray<std::pair<int32_t, int32_t>>> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::branches() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedOnHeapBasicBlockProfilerData::branches(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=203&c=3
template<class D, class P>
Tagged<PodArray<std::pair<int32_t, int32_t>>> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::branches(PtrComprCageBase cage_base) const {
  Tagged<PodArray<std::pair<int32_t, int32_t>>> value = TaggedField<Tagged<PodArray<std::pair<int32_t, int32_t>>>>::load(cage_base, *this, kBranchesOffset);
  DCHECK(IsByteArray(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=203&c=3
template<class D, class P>
void TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::set_branches(Tagged<PodArray<std::pair<int32_t, int32_t>>> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsByteArray(value)));
  WRITE_FIELD(*this, kBranchesOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kBranchesOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=204&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::name() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedOnHeapBasicBlockProfilerData::name(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=204&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::name(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kNameOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=204&c=3
template<class D, class P>
void TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::set_name(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kNameOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kNameOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=205&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::schedule() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedOnHeapBasicBlockProfilerData::schedule(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=205&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::schedule(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kScheduleOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=205&c=3
template<class D, class P>
void TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::set_schedule(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kScheduleOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kScheduleOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=206&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::code() const {
  PtrComprCageBase cage_base = GetPtrComprCageBase(*this);
  return TorqueGeneratedOnHeapBasicBlockProfilerData::code(cage_base);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=206&c=3
template<class D, class P>
Tagged<String> TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::code(PtrComprCageBase cage_base) const {
  Tagged<String> value = TaggedField<Tagged<String>>::load(cage_base, *this, kCodeOffset);
  DCHECK(IsString(value));
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=206&c=3
template<class D, class P>
void TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::set_code(Tagged<String> value, WriteBarrierMode mode) {
  SLOW_DCHECK(!IsolateGroup::current()->shared_read_only_heap()->roots_init_complete() || (IsString(value)));
  WRITE_FIELD(*this, kCodeOffset, value);
  CONDITIONAL_WRITE_BARRIER(*this, kCodeOffset, value, mode);
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=207&c=3
template<class D, class P>
int TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::hash() const {
  int value = TaggedField<Smi>::load(*this, kHashOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=207&c=3
template<class D, class P>
void TorqueGeneratedOnHeapBasicBlockProfilerData<D, P>::set_hash(int value) {
  WRITE_FIELD(*this, kHashOffset, Smi::FromInt(value));
}

