class CoverageInfo;
// Alias for IsCoverageInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1
V8_EXPORT_PRIVATE bool IsCoverageInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1
template <class D, class P>
class TorqueGeneratedCoverageInfo : public P {
  static_assert(
      std::is_same_v<CoverageInfo, D>,
      "Use this class as direct base for CoverageInfo.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedCoverageInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedCoverageInfo<D,P>;

  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=66&c=9
  inline int32_t slot_count() const;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=66&c=9
  inline void set_slot_count(int32_t value);

  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  inline int32_t slots_start_source_position(int i) const;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  inline void set_slots_start_source_position(int i, int32_t value);

  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  inline int32_t slots_end_source_position(int i) const;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  inline void set_slots_end_source_position(int i, int32_t value);

  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  inline int32_t slots_block_count(int i) const;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  inline void set_slots_block_count(int i, int32_t value);

  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  inline int32_t slots_padding(int i) const;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  inline void set_slots_padding(int i, int32_t value);

  DECL_PRINTER(CoverageInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1
  V8_EXPORT_PRIVATE void CoverageInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=66&c=9
  static constexpr int kSlotCountOffset = P::kHeaderSize;
  static constexpr int kSlotCountOffsetEnd = kSlotCountOffset + kInt32Size - 1;
  static constexpr int kHeaderSize = kSlotCountOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=67&c=3
  static constexpr int kSlotsOffset = kSlotCountOffsetEnd + 1;
  static constexpr int kSlotsOffsetEnd = kSlotsOffset + 0 - 1;
  static constexpr int kStartOfWeakFieldsOffset = kSlotsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kSlotsOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kSlotsOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kSlotsOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1
  V8_INLINE static constexpr int32_t SizeFor(int slot_count) {
    int32_t size = kHeaderSize;
    size += slot_count * 16;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/debug-objects.tq?l=65&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor(this->slot_count());
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedCoverageInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedCoverageInfo, DAlias>,
        "class TorqueGeneratedCoverageInfo should be used as direct base for CoverageInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedCoverageInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

struct TorqueGeneratedCoverageInfoSlotOffsets {
  static constexpr int kStartSourcePositionOffset = 0;
  static constexpr int kEndSourcePositionOffset = 4;
  static constexpr int kBlockCountOffset = 8;
  static constexpr int kPaddingOffset = 12;
  static constexpr int kSize = 16;
};

