class JSSegmentIterator;
class JSSegmentDataObject;
class JSSegmentDataObjectWithIsWordLike;
// Alias for IsJSSegmentIterator() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=11&c=1
V8_EXPORT_PRIVATE bool IsJSSegmentIterator_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=11&c=1
template <class D, class P>
class TorqueGeneratedJSSegmentIterator : public P {
  static_assert(
      std::is_same_v<JSSegmentIterator, D>,
      "Use this class as direct base for JSSegmentIterator.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSSegmentIterator.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSSegmentIterator<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=12&c=3
  inline Tagged<Foreign> icu_iterator_with_text() const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=12&c=3
  inline Tagged<Foreign> icu_iterator_with_text(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=12&c=3
  inline void set_icu_iterator_with_text(Tagged<Foreign> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=13&c=3
  inline Tagged<String> raw_string() const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=13&c=3
  inline Tagged<String> raw_string(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=13&c=3
  inline void set_raw_string(Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=14&c=3
  inline int flags() const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=14&c=3
  inline void set_flags(int value);

  DECL_PRINTER(JSSegmentIterator)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=11&c=1
  V8_EXPORT_PRIVATE void JSSegmentIteratorVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=12&c=3
  static constexpr int kIcuIteratorWithTextOffset = P::kHeaderSize;
  static constexpr int kIcuIteratorWithTextOffsetEnd = kIcuIteratorWithTextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=13&c=3
  static constexpr int kRawStringOffset = kIcuIteratorWithTextOffsetEnd + 1;
  static constexpr int kRawStringOffsetEnd = kRawStringOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=14&c=3
  static constexpr int kFlagsOffset = kRawStringOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSSegmentIterator() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSSegmentIterator, DAlias>,
        "class TorqueGeneratedJSSegmentIterator should be used as direct base for JSSegmentIterator.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSSegmentIterator(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSSegmentDataObject() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=17&c=1
V8_EXPORT_PRIVATE bool IsJSSegmentDataObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=17&c=1
template <class D, class P>
class TorqueGeneratedJSSegmentDataObject : public P {
  static_assert(
      std::is_same_v<JSSegmentDataObject, D>,
      "Use this class as direct base for JSSegmentDataObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSSegmentDataObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSSegmentDataObject<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=20&c=3
  inline Tagged<String> segment() const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=20&c=3
  inline Tagged<String> segment(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=20&c=3
  inline void set_segment(Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: Number
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=21&c=3
  inline Tagged<Number> index() const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=21&c=3
  inline Tagged<Number> index(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=21&c=3
  inline void set_index(Tagged<Number> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=22&c=3
  inline Tagged<String> input() const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=22&c=3
  inline Tagged<String> input(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=22&c=3
  inline void set_input(Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSSegmentDataObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=17&c=1
  V8_EXPORT_PRIVATE void JSSegmentDataObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=20&c=3
  static constexpr int kSegmentOffset = P::kHeaderSize;
  static constexpr int kSegmentOffsetEnd = kSegmentOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=21&c=3
  static constexpr int kIndexOffset = kSegmentOffsetEnd + 1;
  static constexpr int kIndexOffsetEnd = kIndexOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=22&c=3
  static constexpr int kInputOffset = kIndexOffsetEnd + 1;
  static constexpr int kInputOffsetEnd = kInputOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kInputOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kInputOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kInputOffsetEnd + 1;
  static constexpr int kHeaderSize = kInputOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSSegmentDataObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSSegmentDataObject, DAlias>,
        "class TorqueGeneratedJSSegmentDataObject should be used as direct base for JSSegmentDataObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSSegmentDataObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSSegmentDataObjectWithIsWordLike() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=25&c=1
V8_EXPORT_PRIVATE bool IsJSSegmentDataObjectWithIsWordLike_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=25&c=1
template <class D, class P>
class TorqueGeneratedJSSegmentDataObjectWithIsWordLike : public P {
  static_assert(
      std::is_same_v<JSSegmentDataObjectWithIsWordLike, D>,
      "Use this class as direct base for JSSegmentDataObjectWithIsWordLike.");
  static_assert(
      std::is_same_v<JSSegmentDataObject, P>,
      "Pass in JSSegmentDataObject as second template parameter for TorqueGeneratedJSSegmentDataObjectWithIsWordLike.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSSegmentDataObjectWithIsWordLike<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=28&c=3
  inline Tagged<Boolean> is_word_like() const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=28&c=3
  inline Tagged<Boolean> is_word_like(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=28&c=3
  inline void set_is_word_like(Tagged<Boolean> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSSegmentDataObjectWithIsWordLike)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=25&c=1
  V8_EXPORT_PRIVATE void JSSegmentDataObjectWithIsWordLikeVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-segment-iterator.tq?l=28&c=3
  static constexpr int kIsWordLikeOffset = P::kHeaderSize;
  static constexpr int kIsWordLikeOffsetEnd = kIsWordLikeOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kIsWordLikeOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kIsWordLikeOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kIsWordLikeOffsetEnd + 1;
  static constexpr int kHeaderSize = kIsWordLikeOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSSegmentDataObjectWithIsWordLike() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSSegmentDataObjectWithIsWordLike, DAlias>,
        "class TorqueGeneratedJSSegmentDataObjectWithIsWordLike should be used as direct base for JSSegmentDataObjectWithIsWordLike.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSSegmentDataObjectWithIsWordLike(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

