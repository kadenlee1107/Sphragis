class JSDisplayNames;
// Alias for IsJSDisplayNames() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=18&c=1
V8_EXPORT_PRIVATE bool IsJSDisplayNames_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=18&c=1
template <class D, class P>
class TorqueGeneratedJSDisplayNames : public P {
  static_assert(
      std::is_same_v<JSDisplayNames, D>,
      "Use this class as direct base for JSDisplayNames.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSDisplayNames.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSDisplayNames<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=19&c=3
  inline Tagged<Foreign> internal() const;
  // https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=19&c=3
  inline Tagged<Foreign> internal(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=19&c=3
  inline void set_internal(Tagged<Foreign> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=20&c=3
  inline int flags() const;
  // https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=20&c=3
  inline void set_flags(int value);

  DECL_PRINTER(JSDisplayNames)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=18&c=1
  V8_EXPORT_PRIVATE void JSDisplayNamesVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=19&c=3
  static constexpr int kInternalOffset = P::kHeaderSize;
  static constexpr int kInternalOffsetEnd = kInternalOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-display-names.tq?l=20&c=3
  static constexpr int kFlagsOffset = kInternalOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSDisplayNames() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSDisplayNames, DAlias>,
        "class TorqueGeneratedJSDisplayNames should be used as direct base for JSDisplayNames.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSDisplayNames(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

