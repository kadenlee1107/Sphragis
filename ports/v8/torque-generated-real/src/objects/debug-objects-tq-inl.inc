// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
inline std::tuple<Tagged<Union<HeapObject, TaggedIndex>>, intptr_t, intptr_t> TqRuntimeFieldSliceCoverageInfoSlots(Tagged<CoverageInfo> p_o);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
inline intptr_t TqRuntimeFromConstexpr_intptr_constexpr_int31_0(int31_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=212&c=1
inline intptr_t TqRuntimeConvert_intptr_int32_0(int32_t p_i);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
inline intptr_t TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0(IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
inline std::tuple<Tagged<Union<HeapObject, TaggedIndex>>, intptr_t, intptr_t> TqRuntimeNewMutableSlice_CoverageInfoSlot_0(Tagged<Union<HeapObject, TaggedIndex>> p_object, intptr_t p_offset, intptr_t p_length);

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
inline intptr_t TqRuntimeFromConstexpr_intptr_constexpr_intptr_0(intptr_t p_i);

#ifndef V8_INTERNAL_DEFINED_TqRuntimeFieldSliceCoverageInfoSlots
#define V8_INTERNAL_DEFINED_TqRuntimeFieldSliceCoverageInfoSlots

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
std::tuple<Tagged<Union<HeapObject, TaggedIndex>>, intptr_t, intptr_t> TqRuntimeFieldSliceCoverageInfoSlots(Tagged<CoverageInfo> p_o) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  int32_t tmp1{}; USE(tmp1);
  intptr_t tmp2{}; USE(tmp2);
  intptr_t tmp3{}; USE(tmp3);
  Tagged<Union<HeapObject, TaggedIndex>> tmp4{}; USE(tmp4);
  intptr_t tmp5{}; USE(tmp5);
  intptr_t tmp6{}; USE(tmp6);
  goto block0;

  block0:
  tmp0 = TqRuntimeFromConstexpr_intptr_constexpr_int31_0(4);
  tmp1 = (p_o)->ReadField<int32_t>(tmp0);
  tmp2 = TqRuntimeConvert_intptr_int32_0(tmp1);
  tmp3 = TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0(IntegerLiteral(false, 0x8ull));
  std::tie(tmp4, tmp5, tmp6) = TqRuntimeNewMutableSlice_CoverageInfoSlot_0(p_o, tmp3, tmp2);
  goto block2;

  block2:
  return std::make_tuple(tmp4, tmp5, tmp6);
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeFieldSliceCoverageInfoSlots
#ifndef V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_int31_0
#define V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_int31_0

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=73&c=1
intptr_t TqRuntimeFromConstexpr_intptr_constexpr_int31_0(int31_t p_i) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  goto block0;

  block0:
  tmp0 = (CastToUnderlyingTypeIfEnum(p_i));
  goto block2;

  block2:
  return tmp0;
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_int31_0
#ifndef V8_INTERNAL_DEFINED_TqRuntimeConvert_intptr_int32_0
#define V8_INTERNAL_DEFINED_TqRuntimeConvert_intptr_int32_0

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=212&c=1
intptr_t TqRuntimeConvert_intptr_int32_0(int32_t p_i) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  goto block0;

  block0:
  tmp0 = TorqueRuntimeMacroShims::CodeStubAssembler::ChangeInt32ToIntPtr(p_i);
  goto block2;

  block2:
  return tmp0;
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeConvert_intptr_int32_0
#ifndef V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0
#define V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=8&c=1
intptr_t TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0(IntegerLiteral p_i) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  goto block0;

  block0:
  tmp0 = TqRuntimeFromConstexpr_intptr_constexpr_intptr_0((TorqueRuntimeMacroShims::CodeStubAssembler::ConstexprIntegerLiteralToIntptr(p_i)));
  goto block2;

  block2:
  return tmp0;
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_IntegerLiteral_0
#ifndef V8_INTERNAL_DEFINED_TqRuntimeNewMutableSlice_CoverageInfoSlot_0
#define V8_INTERNAL_DEFINED_TqRuntimeNewMutableSlice_CoverageInfoSlot_0

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
std::tuple<Tagged<Union<HeapObject, TaggedIndex>>, intptr_t, intptr_t> TqRuntimeNewMutableSlice_CoverageInfoSlot_0(Tagged<Union<HeapObject, TaggedIndex>> p_object, intptr_t p_offset, intptr_t p_length) {
  DisallowGarbageCollection no_gc;
  Tagged<Union<HeapObject, TaggedIndex>> tmp0{}; USE(tmp0);
  intptr_t tmp1{}; USE(tmp1);
  intptr_t tmp2{}; USE(tmp2);
  goto block0;

  block0:
  std::tie(tmp0, tmp1, tmp2) = (std::make_tuple(p_object, p_offset, p_length));
  goto block2;

  block2:
  return std::make_tuple(tmp0, tmp1, tmp2);
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeNewMutableSlice_CoverageInfoSlot_0
#ifndef V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_intptr_0
#define V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_intptr_0

// https://crsrc.org/c/v8/src/builtins/convert.tq?l=79&c=1
intptr_t TqRuntimeFromConstexpr_intptr_constexpr_intptr_0(intptr_t p_i) {
  DisallowGarbageCollection no_gc;
  intptr_t tmp0{}; USE(tmp0);
  goto block0;

  block0:
  tmp0 = (CastToUnderlyingTypeIfEnum(p_i));
  goto block2;

  block2:
  return tmp0;
}

#endif // V8_INTERNAL_DEFINED_TqRuntimeFromConstexpr_intptr_constexpr_intptr_0
// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=66&c=9
template<class D, class P>
int32_t TorqueGeneratedCoverageInfo<D, P>::slot_count() const {
  int32_t value = this->template ReadField<int32_t>(kSlotCountOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=66&c=9
template<class D, class P>
void TorqueGeneratedCoverageInfo<D, P>::set_slot_count(int32_t value) {
  this->template WriteField<int32_t>(kSlotCountOffset, value);
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
template<class D, class P>
int32_t TorqueGeneratedCoverageInfo<D, P>::slots_start_source_position(int i) const {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->slot_count());
  int offset = kSlotsOffset + 0 + i * 16;
  int32_t value = this->template ReadField<int32_t>(offset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
template<class D, class P>
void TorqueGeneratedCoverageInfo<D, P>::set_slots_start_source_position(int i, int32_t value) {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->slot_count());
  int offset = kSlotsOffset + 0 + i * 16;
  this->template WriteField<int32_t>(offset, value);
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
template<class D, class P>
int32_t TorqueGeneratedCoverageInfo<D, P>::slots_end_source_position(int i) const {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->slot_count());
  int offset = kSlotsOffset + 4 + i * 16;
  int32_t value = this->template ReadField<int32_t>(offset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
template<class D, class P>
void TorqueGeneratedCoverageInfo<D, P>::set_slots_end_source_position(int i, int32_t value) {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->slot_count());
  int offset = kSlotsOffset + 4 + i * 16;
  this->template WriteField<int32_t>(offset, value);
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
template<class D, class P>
int32_t TorqueGeneratedCoverageInfo<D, P>::slots_block_count(int i) const {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->slot_count());
  int offset = kSlotsOffset + 8 + i * 16;
  int32_t value = this->template ReadField<int32_t>(offset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
template<class D, class P>
void TorqueGeneratedCoverageInfo<D, P>::set_slots_block_count(int i, int32_t value) {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->slot_count());
  int offset = kSlotsOffset + 8 + i * 16;
  this->template WriteField<int32_t>(offset, value);
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
template<class D, class P>
int32_t TorqueGeneratedCoverageInfo<D, P>::slots_padding(int i) const {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->slot_count());
  int offset = kSlotsOffset + 12 + i * 16;
  int32_t value = this->template ReadField<int32_t>(offset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
template<class D, class P>
void TorqueGeneratedCoverageInfo<D, P>::set_slots_padding(int i, int32_t value) {
  DCHECK_GE(i, 0);
  DCHECK_LT(i, this ->slot_count());
  int offset = kSlotsOffset + 12 + i * 16;
  this->template WriteField<int32_t>(offset, value);
}

