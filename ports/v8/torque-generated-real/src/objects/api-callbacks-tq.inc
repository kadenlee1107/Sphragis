class AccessorInfo;
class InterceptorInfo;
// Alias for IsAccessorInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1
V8_EXPORT_PRIVATE bool IsAccessorInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1
template <class D, class P>
class TorqueGeneratedAccessorInfo : public P {
  static_assert(
      std::is_same_v<AccessorInfo, D>,
      "Use this class as direct base for AccessorInfo.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedAccessorInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedAccessorInfo<D,P>;

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=48&c=3
  inline Tagged<Object> data() const;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=48&c=3
  inline Tagged<Object> data(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=48&c=3
  inline void set_data(Tagged<Object> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=57&c=3
  inline Tagged<Name> name() const;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=57&c=3
  inline Tagged<Name> name(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=57&c=3
  inline void set_name(Tagged<Name> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=63&c=3
  inline uint32_t flags() const;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=63&c=3
  inline void set_flags(uint32_t value);

  DECL_PRINTER(AccessorInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1
  V8_EXPORT_PRIVATE void AccessorInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=48&c=3
  static constexpr int kDataOffset = P::kHeaderSize;
  static constexpr int kDataOffsetEnd = kDataOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=57&c=3
  static constexpr int kNameOffset = kDataOffsetEnd + 1;
  static constexpr int kNameOffsetEnd = kNameOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kNameOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=61&c=3
  static constexpr int kGetterOffset = kNameOffsetEnd + 1;
  static constexpr int kGetterOffsetEnd = kGetterOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=62&c=3
  static constexpr int kSetterOffset = kGetterOffsetEnd + 1;
  static constexpr int kSetterOffsetEnd = kSetterOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=63&c=3
  static constexpr int kFlagsOffset = kSetterOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=65&c=31
  static constexpr int kOptionalPaddingOffset = kFlagsOffsetEnd + 1;
  static constexpr int kOptionalPaddingOffsetEnd = kOptionalPaddingOffset + 0 - 1;
  static constexpr int kStartOfWeakFieldsOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kHeaderSize = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kSize = kOptionalPaddingOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 32);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=47&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedAccessorInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedAccessorInfo, DAlias>,
        "class TorqueGeneratedAccessorInfo should be used as direct base for AccessorInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedAccessorInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsInterceptorInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1
V8_EXPORT_PRIVATE bool IsInterceptorInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1
template <class D, class P>
class TorqueGeneratedInterceptorInfo : public P {
  static_assert(
      std::is_same_v<InterceptorInfo, D>,
      "Use this class as direct base for InterceptorInfo.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedInterceptorInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedInterceptorInfo<D,P>;

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=14&c=3
  inline Tagged<Object> data() const;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=14&c=3
  inline Tagged<Object> data(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=14&c=3
  inline void set_data(Tagged<Object> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=15&c=3
  inline uint32_t flags() const;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=15&c=3
  inline void set_flags(uint32_t value);

  DECL_PRINTER(InterceptorInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1
  V8_EXPORT_PRIVATE void InterceptorInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=14&c=3
  static constexpr int kDataOffset = P::kHeaderSize;
  static constexpr int kDataOffsetEnd = kDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kDataOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=15&c=3
  static constexpr int kFlagsOffset = kDataOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=17&c=31
  static constexpr int kOptionalPaddingOffset = kFlagsOffsetEnd + 1;
  static constexpr int kOptionalPaddingOffsetEnd = kOptionalPaddingOffset + 0 - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=18&c=3
  static constexpr int kGetterOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kGetterOffsetEnd = kGetterOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=19&c=3
  static constexpr int kSetterOffset = kGetterOffsetEnd + 1;
  static constexpr int kSetterOffsetEnd = kSetterOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=20&c=3
  static constexpr int kQueryOffset = kSetterOffsetEnd + 1;
  static constexpr int kQueryOffsetEnd = kQueryOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=21&c=3
  static constexpr int kDescriptorOffset = kQueryOffsetEnd + 1;
  static constexpr int kDescriptorOffsetEnd = kDescriptorOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=22&c=3
  static constexpr int kDeleterOffset = kDescriptorOffsetEnd + 1;
  static constexpr int kDeleterOffsetEnd = kDeleterOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=23&c=3
  static constexpr int kEnumeratorOffset = kDeleterOffsetEnd + 1;
  static constexpr int kEnumeratorOffsetEnd = kEnumeratorOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=24&c=3
  static constexpr int kDefinerOffset = kEnumeratorOffsetEnd + 1;
  static constexpr int kDefinerOffsetEnd = kDefinerOffset + kExternalPointerSlotSize - 1;
  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=26&c=3
  static constexpr int kIndexOfOffset = kDefinerOffsetEnd + 1;
  static constexpr int kIndexOfOffsetEnd = kIndexOfOffset + kExternalPointerSlotSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kIndexOfOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kIndexOfOffsetEnd + 1;
  static constexpr int kHeaderSize = kIndexOfOffsetEnd + 1;
  static constexpr int kSize = kIndexOfOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 76);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/api-callbacks.tq?l=13&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedInterceptorInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedInterceptorInfo, DAlias>,
        "class TorqueGeneratedInterceptorInfo should be used as direct base for InterceptorInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedInterceptorInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

