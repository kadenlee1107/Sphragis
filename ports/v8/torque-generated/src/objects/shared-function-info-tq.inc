class SharedFunctionInfo;
class OnHeapBasicBlockProfilerData;
// Alias for IsSharedFunctionInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1
V8_EXPORT_PRIVATE bool IsSharedFunctionInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1
template <class D, class P>
class TorqueGeneratedSharedFunctionInfo : public P {
  static_assert(
      std::is_same_v<SharedFunctionInfo, D>,
      "Use this class as direct base for SharedFunctionInfo.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedSharedFunctionInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedSharedFunctionInfo<D,P>;

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=79&c=3
  inline Tagged<Object> untrusted_function_data() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=79&c=3
  inline Tagged<Object> untrusted_function_data(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=79&c=3
  inline void set_untrusted_function_data(Tagged<Object> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class String | class ScopeInfo | NoSharedNameSentinel)
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=80&c=3
  inline Tagged<Union<ScopeInfo, Smi, String>> name_or_scope_info() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=80&c=3
  inline Tagged<Union<ScopeInfo, Smi, String>> name_or_scope_info(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=80&c=3
  inline void set_name_or_scope_info(Tagged<Union<ScopeInfo, Smi, String>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class FeedbackMetadata | class TheHole | class ScopeInfo)
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=81&c=3
  inline Tagged<Union<FeedbackMetadata, ScopeInfo, TheHole>> outer_scope_info_or_feedback_metadata() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=81&c=3
  inline Tagged<Union<FeedbackMetadata, ScopeInfo, TheHole>> outer_scope_info_or_feedback_metadata(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=81&c=3
  inline void set_outer_scope_info_or_feedback_metadata(Tagged<Union<FeedbackMetadata, ScopeInfo, TheHole>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class Script)
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=82&c=3
  inline Tagged<Union<Script, Undefined>> script() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=82&c=3
  inline Tagged<Union<Script, Undefined>> script(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=82&c=3
  inline void set_script(Tagged<Union<Script, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=86&c=3
  inline uint16_t length() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=86&c=3
  inline void set_length(uint16_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=102&c=3
  inline uint16_t formal_parameter_count() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=102&c=3
  inline void set_formal_parameter_count(uint16_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=103&c=3
  inline uint16_t function_token_offset() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=103&c=3
  inline void set_function_token_offset(uint16_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=106&c=3
  inline uint8_t expected_nof_properties() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=106&c=3
  inline void set_expected_nof_properties(uint8_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=107&c=3
  inline uint8_t flags2() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=107&c=3
  inline void set_flags2(uint8_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=108&c=3
  inline uint32_t flags() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=108&c=3
  inline void set_flags(uint32_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=112&c=3
  inline int32_t function_literal_id() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=112&c=3
  inline void set_function_literal_id(int32_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=116&c=3
  inline int32_t unique_id() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=116&c=3
  inline void set_unique_id(int32_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=119&c=3
  inline uint16_t age() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=119&c=3
  inline void set_age(uint16_t value);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=123&c=3
  inline uint16_t feedback_slot() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=123&c=3
  inline void set_feedback_slot(uint16_t value);

  DECL_PRINTER(SharedFunctionInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1
  V8_EXPORT_PRIVATE void SharedFunctionInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=75&c=3
  static constexpr int kTrustedFunctionDataOffset = P::kHeaderSize;
  static constexpr int kTrustedFunctionDataOffsetEnd = kTrustedFunctionDataOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfStrongFieldsOffset = kTrustedFunctionDataOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=79&c=3
  static constexpr int kUntrustedFunctionDataOffset = kTrustedFunctionDataOffsetEnd + 1;
  static constexpr int kUntrustedFunctionDataOffsetEnd = kUntrustedFunctionDataOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=80&c=3
  static constexpr int kNameOrScopeInfoOffset = kUntrustedFunctionDataOffsetEnd + 1;
  static constexpr int kNameOrScopeInfoOffsetEnd = kNameOrScopeInfoOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=81&c=3
  static constexpr int kOuterScopeInfoOrFeedbackMetadataOffset = kNameOrScopeInfoOffsetEnd + 1;
  static constexpr int kOuterScopeInfoOrFeedbackMetadataOffsetEnd = kOuterScopeInfoOrFeedbackMetadataOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=82&c=3
  static constexpr int kScriptOffset = kOuterScopeInfoOrFeedbackMetadataOffsetEnd + 1;
  static constexpr int kScriptOffsetEnd = kScriptOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kScriptOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=86&c=3
  static constexpr int kLengthOffset = kScriptOffsetEnd + 1;
  static constexpr int kLengthOffsetEnd = kLengthOffset + kUInt16Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=102&c=3
  static constexpr int kFormalParameterCountOffset = kLengthOffsetEnd + 1;
  static constexpr int kFormalParameterCountOffsetEnd = kFormalParameterCountOffset + kUInt16Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=103&c=3
  static constexpr int kFunctionTokenOffsetOffset = kFormalParameterCountOffsetEnd + 1;
  static constexpr int kFunctionTokenOffsetOffsetEnd = kFunctionTokenOffsetOffset + kUInt16Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=106&c=3
  static constexpr int kExpectedNofPropertiesOffset = kFunctionTokenOffsetOffsetEnd + 1;
  static constexpr int kExpectedNofPropertiesOffsetEnd = kExpectedNofPropertiesOffset + kUInt8Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=107&c=3
  static constexpr int kFlags2Offset = kExpectedNofPropertiesOffsetEnd + 1;
  static constexpr int kFlags2OffsetEnd = kFlags2Offset + kUInt8Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=108&c=3
  static constexpr int kFlagsOffset = kFlags2OffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=112&c=3
  static constexpr int kFunctionLiteralIdOffset = kFlagsOffsetEnd + 1;
  static constexpr int kFunctionLiteralIdOffsetEnd = kFunctionLiteralIdOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=116&c=3
  static constexpr int kUniqueIdOffset = kFunctionLiteralIdOffsetEnd + 1;
  static constexpr int kUniqueIdOffsetEnd = kUniqueIdOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=119&c=3
  static constexpr int kAgeOffset = kUniqueIdOffsetEnd + 1;
  static constexpr int kAgeOffsetEnd = kAgeOffset + kUInt16Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=123&c=3
  static constexpr int kFeedbackSlotOffset = kAgeOffsetEnd + 1;
  static constexpr int kFeedbackSlotOffsetEnd = kFeedbackSlotOffset + kUInt16Size - 1;
  static constexpr int kStartOfWeakFieldsOffset = kFeedbackSlotOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFeedbackSlotOffsetEnd + 1;
  static constexpr int kHeaderSize = kFeedbackSlotOffsetEnd + 1;
  static constexpr int kSize = kFeedbackSlotOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 48);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedSharedFunctionInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedSharedFunctionInfo, DAlias>,
        "class TorqueGeneratedSharedFunctionInfo should be used as direct base for SharedFunctionInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedSharedFunctionInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsOnHeapBasicBlockProfilerData() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1
V8_EXPORT_PRIVATE bool IsOnHeapBasicBlockProfilerData_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1
template <class D, class P>
class TorqueGeneratedOnHeapBasicBlockProfilerData : public P {
  static_assert(
      std::is_same_v<OnHeapBasicBlockProfilerData, D>,
      "Use this class as direct base for OnHeapBasicBlockProfilerData.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedOnHeapBasicBlockProfilerData.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedOnHeapBasicBlockProfilerData<D,P>;

  // Torque type: FixedInt32Array
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=201&c=3
  inline Tagged<FixedInt32Array> block_ids() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=201&c=3
  inline Tagged<FixedInt32Array> block_ids(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=201&c=3
  inline void set_block_ids(Tagged<FixedInt32Array> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: FixedUInt32Array
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=202&c=3
  inline Tagged<FixedUInt32Array> counts() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=202&c=3
  inline Tagged<FixedUInt32Array> counts(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=202&c=3
  inline void set_counts(Tagged<FixedUInt32Array> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: PodArrayOfIntegerPairs
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=203&c=3
  inline Tagged<PodArray<std::pair<int32_t, int32_t>>> branches() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=203&c=3
  inline Tagged<PodArray<std::pair<int32_t, int32_t>>> branches(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=203&c=3
  inline void set_branches(Tagged<PodArray<std::pair<int32_t, int32_t>>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=204&c=3
  inline Tagged<String> name() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=204&c=3
  inline Tagged<String> name(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=204&c=3
  inline void set_name(Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=205&c=3
  inline Tagged<String> schedule() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=205&c=3
  inline Tagged<String> schedule(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=205&c=3
  inline void set_schedule(Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=206&c=3
  inline Tagged<String> code() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=206&c=3
  inline Tagged<String> code(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=206&c=3
  inline void set_code(Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=207&c=3
  inline int hash() const;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=207&c=3
  inline void set_hash(int value);

  DECL_PRINTER(OnHeapBasicBlockProfilerData)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1
  V8_EXPORT_PRIVATE void OnHeapBasicBlockProfilerDataVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=201&c=3
  static constexpr int kBlockIdsOffset = P::kHeaderSize;
  static constexpr int kBlockIdsOffsetEnd = kBlockIdsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=202&c=3
  static constexpr int kCountsOffset = kBlockIdsOffsetEnd + 1;
  static constexpr int kCountsOffsetEnd = kCountsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=203&c=3
  static constexpr int kBranchesOffset = kCountsOffsetEnd + 1;
  static constexpr int kBranchesOffsetEnd = kBranchesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=204&c=3
  static constexpr int kNameOffset = kBranchesOffsetEnd + 1;
  static constexpr int kNameOffsetEnd = kNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=205&c=3
  static constexpr int kScheduleOffset = kNameOffsetEnd + 1;
  static constexpr int kScheduleOffsetEnd = kScheduleOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=206&c=3
  static constexpr int kCodeOffset = kScheduleOffsetEnd + 1;
  static constexpr int kCodeOffsetEnd = kCodeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=207&c=3
  static constexpr int kHashOffset = kCodeOffsetEnd + 1;
  static constexpr int kHashOffsetEnd = kHashOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kHashOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kHashOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kHashOffsetEnd + 1;
  static constexpr int kHeaderSize = kHashOffsetEnd + 1;
  static constexpr int kSize = kHashOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 32);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedOnHeapBasicBlockProfilerData() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedOnHeapBasicBlockProfilerData, DAlias>,
        "class TorqueGeneratedOnHeapBasicBlockProfilerData should be used as direct base for OnHeapBasicBlockProfilerData.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedOnHeapBasicBlockProfilerData(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class OnHeapBasicBlockProfilerData : public TorqueGeneratedOnHeapBasicBlockProfilerData<OnHeapBasicBlockProfilerData, HeapObject> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(OnHeapBasicBlockProfilerData)
};

