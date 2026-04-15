class JSSegments;
// Alias for IsJSSegments() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-segments.tq?l=11&c=1
V8_EXPORT_PRIVATE bool IsJSSegments_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-segments.tq?l=11&c=1
template <class D, class P>
class TorqueGeneratedJSSegments : public P {
  static_assert(
      std::is_same_v<JSSegments, D>,
      "Use this class as direct base for JSSegments.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSSegments.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSSegments<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=12&c=3
  inline Tagged<Foreign> icu_iterator_with_text() const;
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=12&c=3
  inline Tagged<Foreign> icu_iterator_with_text(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=12&c=3
  inline void set_icu_iterator_with_text(Tagged<Foreign> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=13&c=3
  inline Tagged<String> raw_string() const;
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=13&c=3
  inline Tagged<String> raw_string(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=13&c=3
  inline void set_raw_string(Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=14&c=3
  inline int flags() const;
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=14&c=3
  inline void set_flags(int value);

  DECL_PRINTER(JSSegments)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=11&c=1
  V8_EXPORT_PRIVATE void JSSegmentsVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=12&c=3
  static constexpr int kIcuIteratorWithTextOffset = P::kHeaderSize;
  static constexpr int kIcuIteratorWithTextOffsetEnd = kIcuIteratorWithTextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=13&c=3
  static constexpr int kRawStringOffset = kIcuIteratorWithTextOffsetEnd + 1;
  static constexpr int kRawStringOffsetEnd = kRawStringOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-segments.tq?l=14&c=3
  static constexpr int kFlagsOffset = kRawStringOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSSegments() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSSegments, DAlias>,
        "class TorqueGeneratedJSSegments should be used as direct base for JSSegments.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSSegments(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

