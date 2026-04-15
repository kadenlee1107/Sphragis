class DescriptorArray;
// Alias for IsDescriptorArray() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1
V8_EXPORT_PRIVATE bool IsDescriptorArray_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1
template <class D, class P>
class TorqueGeneratedDescriptorArray : public P {
  static_assert(
      std::is_same_v<DescriptorArray, D>,
      "Use this class as direct base for DescriptorArray.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedDescriptorArray.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedDescriptorArray<D,P>;

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=29&c=9
  inline uint16_t number_of_all_descriptors() const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=29&c=9
  inline void set_number_of_all_descriptors(uint16_t value);

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=30&c=3
  inline uint16_t number_of_descriptors() const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=30&c=3
  inline void set_number_of_descriptors(uint16_t value);

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=33&c=3
  inline uint32_t raw_gc_state() const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=33&c=3
  inline void set_raw_gc_state(uint32_t value);

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=34&c=3
  inline uint32_t flags() const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=34&c=3
  inline void set_flags(uint32_t value);

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=39&c=3
  inline Tagged<EnumCache> enum_cache() const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=39&c=3
  inline Tagged<EnumCache> enum_cache(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=39&c=3
  inline void set_enum_cache(Tagged<EnumCache> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Name | class Undefined)
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline Tagged<Union<Name, Undefined>> descriptors_key(int i) const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline Tagged<Union<Name, Undefined>> descriptors_key(PtrComprCageBase cage_base, int i) const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline void set_descriptors_key(int i, Tagged<Union<Name, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | Smi)
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline Tagged<Union<Smi, Undefined>> descriptors_details(int i) const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline Tagged<Union<Smi, Undefined>> descriptors_details(PtrComprCageBase cage_base, int i) const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline void set_descriptors_details(int i, Tagged<Union<Smi, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSReceiver | BigInt | class Undefined | Smi | class HeapNumber | class String | class Symbol | class Boolean | class Null | Weak<class Map> | class AccessorInfo | class AccessorPair | class ClassPositions | class NumberDictionary)
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline Tagged<Union<AccessorInfo, AccessorPair, BigInt, Boolean, ClassPositions, HeapNumber, JSReceiver, Null, NumberDictionary, Smi, String, Symbol, Undefined, Weak<HeapObject>>> descriptors_value(int i) const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline Tagged<Union<AccessorInfo, AccessorPair, BigInt, Boolean, ClassPositions, HeapNumber, JSReceiver, Null, NumberDictionary, Smi, String, Symbol, Undefined, Weak<HeapObject>>> descriptors_value(PtrComprCageBase cage_base, int i) const;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  inline void set_descriptors_value(int i, Tagged<Union<AccessorInfo, AccessorPair, BigInt, Boolean, ClassPositions, HeapNumber, JSReceiver, Null, NumberDictionary, Smi, String, Symbol, Undefined, Weak<HeapObject>>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(DescriptorArray)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1
  V8_EXPORT_PRIVATE void DescriptorArrayVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=29&c=9
  static constexpr int kNumberOfAllDescriptorsOffset = P::kHeaderSize;
  static constexpr int kNumberOfAllDescriptorsOffsetEnd = kNumberOfAllDescriptorsOffset + kUInt16Size - 1;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=30&c=3
  static constexpr int kNumberOfDescriptorsOffset = kNumberOfAllDescriptorsOffsetEnd + 1;
  static constexpr int kNumberOfDescriptorsOffsetEnd = kNumberOfDescriptorsOffset + kUInt16Size - 1;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=33&c=3
  static constexpr int kRawGcStateOffset = kNumberOfDescriptorsOffsetEnd + 1;
  static constexpr int kRawGcStateOffsetEnd = kRawGcStateOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=34&c=3
  static constexpr int kFlagsOffset = kRawGcStateOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=37&c=31
  static constexpr int kOptionalPaddingOffset = kFlagsOffsetEnd + 1;
  static constexpr int kOptionalPaddingOffsetEnd = kOptionalPaddingOffset + 0 - 1;
  static constexpr int kStartOfStrongFieldsOffset = kOptionalPaddingOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=39&c=3
  static constexpr int kEnumCacheOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kEnumCacheOffsetEnd = kEnumCacheOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kEnumCacheOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kEnumCacheOffsetEnd + 1;
  static constexpr int kHeaderSize = kEnumCacheOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=40&c=3
  static constexpr int kDescriptorsOffset = kEnumCacheOffsetEnd + 1;
  static constexpr int kDescriptorsOffsetEnd = kDescriptorsOffset + 0 - 1;
  static constexpr int kEndOfWeakFieldsOffset = kDescriptorsOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1
  V8_INLINE static constexpr int32_t SizeFor(int number_of_all_descriptors) {
    int32_t size = kHeaderSize;
    size += number_of_all_descriptors * 12;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=26&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor(this->number_of_all_descriptors());
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedDescriptorArray() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedDescriptorArray, DAlias>,
        "class TorqueGeneratedDescriptorArray should be used as direct base for DescriptorArray.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedDescriptorArray(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

struct TorqueGeneratedDescriptorEntryOffsets {
  static constexpr int kKeyOffset = 0;
  static constexpr int kDetailsOffset = 4;
  static constexpr int kValueOffset = 8;
  static constexpr int kSize = 12;
};

