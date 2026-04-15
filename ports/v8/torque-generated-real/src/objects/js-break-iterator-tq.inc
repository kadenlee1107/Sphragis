class JSV8BreakIterator;
// Alias for IsJSV8BreakIterator() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=7&c=1
V8_EXPORT_PRIVATE bool IsJSV8BreakIterator_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=7&c=1
template <class D, class P>
class TorqueGeneratedJSV8BreakIterator : public P {
  static_assert(
      std::is_same_v<JSV8BreakIterator, D>,
      "Use this class as direct base for JSV8BreakIterator.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSV8BreakIterator.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSV8BreakIterator<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=8&c=3
  inline Tagged<String> locale() const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=8&c=3
  inline Tagged<String> locale(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=8&c=3
  inline void set_locale(Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=9&c=3
  inline Tagged<Foreign> icu_iterator_with_text() const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=9&c=3
  inline Tagged<Foreign> icu_iterator_with_text(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=9&c=3
  inline void set_icu_iterator_with_text(Tagged<Foreign> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSFunction | class Undefined)
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=10&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_adopt_text() const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=10&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_adopt_text(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=10&c=3
  inline void set_bound_adopt_text(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSFunction | class Undefined)
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=11&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_first() const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=11&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_first(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=11&c=3
  inline void set_bound_first(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSFunction | class Undefined)
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=12&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_next() const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=12&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_next(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=12&c=3
  inline void set_bound_next(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSFunction | class Undefined)
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=13&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_current() const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=13&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_current(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=13&c=3
  inline void set_bound_current(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSFunction | class Undefined)
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=14&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_break_type() const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=14&c=3
  inline Tagged<Union<JSFunction, Undefined>> bound_break_type(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=14&c=3
  inline void set_bound_break_type(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSV8BreakIterator)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=7&c=1
  V8_EXPORT_PRIVATE void JSV8BreakIteratorVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=8&c=3
  static constexpr int kLocaleOffset = P::kHeaderSize;
  static constexpr int kLocaleOffsetEnd = kLocaleOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=9&c=3
  static constexpr int kIcuIteratorWithTextOffset = kLocaleOffsetEnd + 1;
  static constexpr int kIcuIteratorWithTextOffsetEnd = kIcuIteratorWithTextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=10&c=3
  static constexpr int kBoundAdoptTextOffset = kIcuIteratorWithTextOffsetEnd + 1;
  static constexpr int kBoundAdoptTextOffsetEnd = kBoundAdoptTextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=11&c=3
  static constexpr int kBoundFirstOffset = kBoundAdoptTextOffsetEnd + 1;
  static constexpr int kBoundFirstOffsetEnd = kBoundFirstOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=12&c=3
  static constexpr int kBoundNextOffset = kBoundFirstOffsetEnd + 1;
  static constexpr int kBoundNextOffsetEnd = kBoundNextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=13&c=3
  static constexpr int kBoundCurrentOffset = kBoundNextOffsetEnd + 1;
  static constexpr int kBoundCurrentOffsetEnd = kBoundCurrentOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-break-iterator.tq?l=14&c=3
  static constexpr int kBoundBreakTypeOffset = kBoundCurrentOffsetEnd + 1;
  static constexpr int kBoundBreakTypeOffsetEnd = kBoundBreakTypeOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kBoundBreakTypeOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kBoundBreakTypeOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kBoundBreakTypeOffsetEnd + 1;
  static constexpr int kHeaderSize = kBoundBreakTypeOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSV8BreakIterator() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSV8BreakIterator, DAlias>,
        "class TorqueGeneratedJSV8BreakIterator should be used as direct base for JSV8BreakIterator.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSV8BreakIterator(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

