class JSIteratorHelper;
class JSIteratorHelperSimple;
class JSIteratorMapHelper;
class JSIteratorFilterHelper;
class JSIteratorTakeHelper;
class JSIteratorDropHelper;
class JSIteratorFlatMapHelper;
class JSIteratorConcatHelper;
class JSIteratorZipHelper;
// Alias for IsJSIteratorHelper() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=14&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorHelper_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=14&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorHelper : public P {
  static_assert(
      std::is_same_v<JSIteratorHelper, D>,
      "Use this class as direct base for JSIteratorHelper.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSIteratorHelper.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorHelper<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=16&c=3
  inline int state() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=16&c=3
  inline void set_state(int value);

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=14&c=1
  V8_EXPORT_PRIVATE void JSIteratorHelperVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=16&c=3
  static constexpr int kStateOffset = P::kHeaderSize;
  static constexpr int kStateOffsetEnd = kStateOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kStateOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kStateOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kStateOffsetEnd + 1;
  static constexpr int kHeaderSize = kStateOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorHelper() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorHelper, DAlias>,
        "class TorqueGeneratedJSIteratorHelper should be used as direct base for JSIteratorHelper.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorHelper(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSIteratorHelperSimple() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=19&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorHelperSimple_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=19&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorHelperSimple : public P {
  static_assert(
      std::is_same_v<JSIteratorHelperSimple, D>,
      "Use this class as direct base for JSIteratorHelperSimple.");
  static_assert(
      std::is_same_v<JSIteratorHelper, P>,
      "Pass in JSIteratorHelper as second template parameter for TorqueGeneratedJSIteratorHelperSimple.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorHelperSimple<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
  inline Tagged<JSReceiver> underlying_iterator_object() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
  inline Tagged<JSReceiver> underlying_iterator_object(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
  inline void set_underlying_iterator_object(Tagged<JSReceiver> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: JSAny
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
  inline Tagged<JSAny> underlying_iterator_next() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
  inline Tagged<JSAny> underlying_iterator_next(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
  inline void set_underlying_iterator_next(Tagged<JSAny> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=19&c=1
  V8_EXPORT_PRIVATE void JSIteratorHelperSimpleVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=21&c=3
  static constexpr int kUnderlyingIteratorOffset = P::kHeaderSize;
  static constexpr int kUnderlyingIteratorOffsetEnd = kUnderlyingIteratorOffset + 8 - 1;
  static constexpr int kEndOfStrongFieldsOffset = kUnderlyingIteratorOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kUnderlyingIteratorOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kUnderlyingIteratorOffsetEnd + 1;
  static constexpr int kHeaderSize = kUnderlyingIteratorOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorHelperSimple() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorHelperSimple, DAlias>,
        "class TorqueGeneratedJSIteratorHelperSimple should be used as direct base for JSIteratorHelperSimple.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorHelperSimple(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSIteratorMapHelper() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=24&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorMapHelper_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=24&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorMapHelper : public P {
  static_assert(
      std::is_same_v<JSIteratorMapHelper, D>,
      "Use this class as direct base for JSIteratorMapHelper.");
  static_assert(
      std::is_same_v<JSIteratorHelperSimple, P>,
      "Pass in JSIteratorHelperSimple as second template parameter for TorqueGeneratedJSIteratorMapHelper.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorMapHelper<D,P>;

  // Torque type: Callable
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=25&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> mapper() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=25&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> mapper(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=25&c=3
  inline void set_mapper(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: Number
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=26&c=3
  inline Tagged<Number> counter() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=26&c=3
  inline Tagged<Number> counter(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=26&c=3
  inline void set_counter(Tagged<Number> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSIteratorMapHelper)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=24&c=1
  V8_EXPORT_PRIVATE void JSIteratorMapHelperVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=25&c=3
  static constexpr int kMapperOffset = P::kHeaderSize;
  static constexpr int kMapperOffsetEnd = kMapperOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=26&c=3
  static constexpr int kCounterOffset = kMapperOffsetEnd + 1;
  static constexpr int kCounterOffsetEnd = kCounterOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCounterOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kCounterOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kCounterOffsetEnd + 1;
  static constexpr int kHeaderSize = kCounterOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorMapHelper() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorMapHelper, DAlias>,
        "class TorqueGeneratedJSIteratorMapHelper should be used as direct base for JSIteratorMapHelper.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorMapHelper(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSIteratorFilterHelper() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=29&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorFilterHelper_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=29&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorFilterHelper : public P {
  static_assert(
      std::is_same_v<JSIteratorFilterHelper, D>,
      "Use this class as direct base for JSIteratorFilterHelper.");
  static_assert(
      std::is_same_v<JSIteratorHelperSimple, P>,
      "Pass in JSIteratorHelperSimple as second template parameter for TorqueGeneratedJSIteratorFilterHelper.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorFilterHelper<D,P>;

  // Torque type: Callable
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=30&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> predicate() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=30&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> predicate(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=30&c=3
  inline void set_predicate(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: Number
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=31&c=3
  inline Tagged<Number> counter() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=31&c=3
  inline Tagged<Number> counter(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=31&c=3
  inline void set_counter(Tagged<Number> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSIteratorFilterHelper)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=29&c=1
  V8_EXPORT_PRIVATE void JSIteratorFilterHelperVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=30&c=3
  static constexpr int kPredicateOffset = P::kHeaderSize;
  static constexpr int kPredicateOffsetEnd = kPredicateOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=31&c=3
  static constexpr int kCounterOffset = kPredicateOffsetEnd + 1;
  static constexpr int kCounterOffsetEnd = kCounterOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCounterOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kCounterOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kCounterOffsetEnd + 1;
  static constexpr int kHeaderSize = kCounterOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorFilterHelper() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorFilterHelper, DAlias>,
        "class TorqueGeneratedJSIteratorFilterHelper should be used as direct base for JSIteratorFilterHelper.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorFilterHelper(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSIteratorTakeHelper() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=34&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorTakeHelper_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=34&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorTakeHelper : public P {
  static_assert(
      std::is_same_v<JSIteratorTakeHelper, D>,
      "Use this class as direct base for JSIteratorTakeHelper.");
  static_assert(
      std::is_same_v<JSIteratorHelperSimple, P>,
      "Pass in JSIteratorHelperSimple as second template parameter for TorqueGeneratedJSIteratorTakeHelper.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorTakeHelper<D,P>;

  // Torque type: Number
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=35&c=3
  inline Tagged<Number> remaining() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=35&c=3
  inline Tagged<Number> remaining(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=35&c=3
  inline void set_remaining(Tagged<Number> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSIteratorTakeHelper)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=34&c=1
  V8_EXPORT_PRIVATE void JSIteratorTakeHelperVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=35&c=3
  static constexpr int kRemainingOffset = P::kHeaderSize;
  static constexpr int kRemainingOffsetEnd = kRemainingOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kRemainingOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kRemainingOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kRemainingOffsetEnd + 1;
  static constexpr int kHeaderSize = kRemainingOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorTakeHelper() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorTakeHelper, DAlias>,
        "class TorqueGeneratedJSIteratorTakeHelper should be used as direct base for JSIteratorTakeHelper.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorTakeHelper(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSIteratorDropHelper() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=38&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorDropHelper_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=38&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorDropHelper : public P {
  static_assert(
      std::is_same_v<JSIteratorDropHelper, D>,
      "Use this class as direct base for JSIteratorDropHelper.");
  static_assert(
      std::is_same_v<JSIteratorHelperSimple, P>,
      "Pass in JSIteratorHelperSimple as second template parameter for TorqueGeneratedJSIteratorDropHelper.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorDropHelper<D,P>;

  // Torque type: Number
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=39&c=3
  inline Tagged<Number> remaining() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=39&c=3
  inline Tagged<Number> remaining(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=39&c=3
  inline void set_remaining(Tagged<Number> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSIteratorDropHelper)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=38&c=1
  V8_EXPORT_PRIVATE void JSIteratorDropHelperVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=39&c=3
  static constexpr int kRemainingOffset = P::kHeaderSize;
  static constexpr int kRemainingOffsetEnd = kRemainingOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kRemainingOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kRemainingOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kRemainingOffsetEnd + 1;
  static constexpr int kHeaderSize = kRemainingOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorDropHelper() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorDropHelper, DAlias>,
        "class TorqueGeneratedJSIteratorDropHelper should be used as direct base for JSIteratorDropHelper.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorDropHelper(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSIteratorFlatMapHelper() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=42&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorFlatMapHelper_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=42&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorFlatMapHelper : public P {
  static_assert(
      std::is_same_v<JSIteratorFlatMapHelper, D>,
      "Use this class as direct base for JSIteratorFlatMapHelper.");
  static_assert(
      std::is_same_v<JSIteratorHelperSimple, P>,
      "Pass in JSIteratorHelperSimple as second template parameter for TorqueGeneratedJSIteratorFlatMapHelper.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorFlatMapHelper<D,P>;

  // Torque type: Callable
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=43&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> mapper() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=43&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> mapper(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=43&c=3
  inline void set_mapper(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: Number
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=44&c=3
  inline Tagged<Number> counter() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=44&c=3
  inline Tagged<Number> counter(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=44&c=3
  inline void set_counter(Tagged<Number> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
  inline Tagged<JSReceiver> inner_iterator_object() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
  inline Tagged<JSReceiver> inner_iterator_object(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
  inline void set_inner_iterator_object(Tagged<JSReceiver> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: JSAny
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
  inline Tagged<JSAny> inner_iterator_next() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
  inline Tagged<JSAny> inner_iterator_next(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
  inline void set_inner_iterator_next(Tagged<JSAny> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSIteratorFlatMapHelper)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=42&c=1
  V8_EXPORT_PRIVATE void JSIteratorFlatMapHelperVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=43&c=3
  static constexpr int kMapperOffset = P::kHeaderSize;
  static constexpr int kMapperOffsetEnd = kMapperOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=44&c=3
  static constexpr int kCounterOffset = kMapperOffsetEnd + 1;
  static constexpr int kCounterOffsetEnd = kCounterOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=45&c=3
  static constexpr int kInnerIteratorOffset = kCounterOffsetEnd + 1;
  static constexpr int kInnerIteratorOffsetEnd = kInnerIteratorOffset + 8 - 1;
  static constexpr int kEndOfStrongFieldsOffset = kInnerIteratorOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kInnerIteratorOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kInnerIteratorOffsetEnd + 1;
  static constexpr int kHeaderSize = kInnerIteratorOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorFlatMapHelper() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorFlatMapHelper, DAlias>,
        "class TorqueGeneratedJSIteratorFlatMapHelper should be used as direct base for JSIteratorFlatMapHelper.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorFlatMapHelper(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSIteratorConcatHelper() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=48&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorConcatHelper_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=48&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorConcatHelper : public P {
  static_assert(
      std::is_same_v<JSIteratorConcatHelper, D>,
      "Use this class as direct base for JSIteratorConcatHelper.");
  static_assert(
      std::is_same_v<JSIteratorHelperSimple, P>,
      "Pass in JSIteratorHelperSimple as second template parameter for TorqueGeneratedJSIteratorConcatHelper.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorConcatHelper<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=49&c=3
  inline Tagged<FixedArray> iterables() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=49&c=3
  inline Tagged<FixedArray> iterables(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=49&c=3
  inline void set_iterables(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=50&c=3
  inline int current() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=50&c=3
  inline void set_current(int value);

  DECL_PRINTER(JSIteratorConcatHelper)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=48&c=1
  V8_EXPORT_PRIVATE void JSIteratorConcatHelperVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=49&c=3
  static constexpr int kIterablesOffset = P::kHeaderSize;
  static constexpr int kIterablesOffsetEnd = kIterablesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=50&c=3
  static constexpr int kCurrentOffset = kIterablesOffsetEnd + 1;
  static constexpr int kCurrentOffsetEnd = kCurrentOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCurrentOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kCurrentOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kCurrentOffsetEnd + 1;
  static constexpr int kHeaderSize = kCurrentOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorConcatHelper() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorConcatHelper, DAlias>,
        "class TorqueGeneratedJSIteratorConcatHelper should be used as direct base for JSIteratorConcatHelper.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorConcatHelper(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSIteratorZipHelper() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=59&c=1
V8_EXPORT_PRIVATE bool IsJSIteratorZipHelper_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=59&c=1
template <class D, class P>
class TorqueGeneratedJSIteratorZipHelper : public P {
  static_assert(
      std::is_same_v<JSIteratorZipHelper, D>,
      "Use this class as direct base for JSIteratorZipHelper.");
  static_assert(
      std::is_same_v<JSIteratorHelper, P>,
      "Pass in JSIteratorHelper as second template parameter for TorqueGeneratedJSIteratorZipHelper.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSIteratorZipHelper<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=60&c=3
  inline Tagged<FixedArray> underlying_iterators() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=60&c=3
  inline Tagged<FixedArray> underlying_iterators(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=60&c=3
  inline void set_underlying_iterators(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=61&c=3
  inline int mode() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=61&c=3
  inline void set_mode(int value);

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=62&c=3
  inline int active_count() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=62&c=3
  inline void set_active_count(int value);

  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=63&c=3
  inline Tagged<FixedArray> padding() const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=63&c=3
  inline Tagged<FixedArray> padding(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=63&c=3
  inline void set_padding(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSIteratorZipHelper)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=59&c=1
  V8_EXPORT_PRIVATE void JSIteratorZipHelperVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=60&c=3
  static constexpr int kUnderlyingIteratorsOffset = P::kHeaderSize;
  static constexpr int kUnderlyingIteratorsOffsetEnd = kUnderlyingIteratorsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=61&c=3
  static constexpr int kModeOffset = kUnderlyingIteratorsOffsetEnd + 1;
  static constexpr int kModeOffsetEnd = kModeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=62&c=3
  static constexpr int kActiveCountOffset = kModeOffsetEnd + 1;
  static constexpr int kActiveCountOffsetEnd = kActiveCountOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-iterator-helpers.tq?l=63&c=3
  static constexpr int kPaddingOffset = kActiveCountOffsetEnd + 1;
  static constexpr int kPaddingOffsetEnd = kPaddingOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPaddingOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kPaddingOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPaddingOffsetEnd + 1;
  static constexpr int kHeaderSize = kPaddingOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSIteratorZipHelper() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSIteratorZipHelper, DAlias>,
        "class TorqueGeneratedJSIteratorZipHelper should be used as direct base for JSIteratorZipHelper.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSIteratorZipHelper(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

