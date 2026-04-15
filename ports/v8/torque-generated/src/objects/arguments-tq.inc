class JSArgumentsObject;
class JSSloppyArgumentsObject;
class JSStrictArgumentsObject;
// Alias for IsJSArgumentsObject() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1
V8_EXPORT_PRIVATE bool IsJSArgumentsObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1
template <class D, class P>
class TorqueGeneratedJSArgumentsObject : public P {
  static_assert(
      std::is_same_v<JSArgumentsObject, D>,
      "Use this class as direct base for JSArgumentsObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSArgumentsObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSArgumentsObject<D,P>;

  DECL_PRINTER(JSArgumentsObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=5&c=1
  V8_EXPORT_PRIVATE void JSArgumentsObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSArgumentsObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSArgumentsObject, DAlias>,
        "class TorqueGeneratedJSArgumentsObject should be used as direct base for JSArgumentsObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSArgumentsObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Definition https://crsrc.org/c/v8/src/objects/arguments.tq?l=17&c=1
template <class D, class P>
class TorqueGeneratedJSSloppyArgumentsObject : public P {
  static_assert(
      std::is_same_v<JSSloppyArgumentsObject, D>,
      "Use this class as direct base for JSSloppyArgumentsObject.");
  static_assert(
      std::is_same_v<JSArgumentsObject, P>,
      "Pass in JSArgumentsObject as second template parameter for TorqueGeneratedJSSloppyArgumentsObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSSloppyArgumentsObject<D,P>;

  // Torque type: JSAny
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=18&c=3
  inline Tagged<JSAny> length() const;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=18&c=3
  inline Tagged<JSAny> length(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=18&c=3
  inline void set_length(Tagged<JSAny> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: JSAny
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=19&c=3
  inline Tagged<JSAny> callee() const;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=19&c=3
  inline Tagged<JSAny> callee(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=19&c=3
  inline void set_callee(Tagged<JSAny> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSSloppyArgumentsObject)


  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=18&c=3
  static constexpr int kLengthOffset = P::kHeaderSize;
  static constexpr int kLengthOffsetEnd = kLengthOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=19&c=3
  static constexpr int kCalleeOffset = kLengthOffsetEnd + 1;
  static constexpr int kCalleeOffsetEnd = kCalleeOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCalleeOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kCalleeOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kCalleeOffsetEnd + 1;
  static constexpr int kSize = kCalleeOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSSloppyArgumentsObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSSloppyArgumentsObject, DAlias>,
        "class TorqueGeneratedJSSloppyArgumentsObject should be used as direct base for JSSloppyArgumentsObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSSloppyArgumentsObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Definition https://crsrc.org/c/v8/src/objects/arguments.tq?l=23&c=1
template <class D, class P>
class TorqueGeneratedJSStrictArgumentsObject : public P {
  static_assert(
      std::is_same_v<JSStrictArgumentsObject, D>,
      "Use this class as direct base for JSStrictArgumentsObject.");
  static_assert(
      std::is_same_v<JSArgumentsObject, P>,
      "Pass in JSArgumentsObject as second template parameter for TorqueGeneratedJSStrictArgumentsObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSStrictArgumentsObject<D,P>;

  // Torque type: JSAny
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=24&c=3
  inline Tagged<JSAny> length() const;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=24&c=3
  inline Tagged<JSAny> length(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=24&c=3
  inline void set_length(Tagged<JSAny> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSStrictArgumentsObject)


  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/arguments.tq?l=24&c=3
  static constexpr int kLengthOffset = P::kHeaderSize;
  static constexpr int kLengthOffsetEnd = kLengthOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kLengthOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kLengthOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kLengthOffsetEnd + 1;
  static constexpr int kSize = kLengthOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSStrictArgumentsObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSStrictArgumentsObject, DAlias>,
        "class TorqueGeneratedJSStrictArgumentsObject should be used as direct base for JSStrictArgumentsObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSStrictArgumentsObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

