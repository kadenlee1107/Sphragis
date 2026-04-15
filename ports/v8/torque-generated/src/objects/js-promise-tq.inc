class JSPromise;
// Alias for IsJSPromise() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-promise.tq?l=12&c=1
V8_EXPORT_PRIVATE bool IsJSPromise_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-promise.tq?l=12&c=1
template <class D, class P>
class TorqueGeneratedJSPromise : public P {
  static_assert(
      std::is_same_v<JSPromise, D>,
      "Use this class as direct base for JSPromise.");
  static_assert(
      std::is_same_v<JSObjectWithEmbedderSlots, P>,
      "Pass in JSObjectWithEmbedderSlots as second template parameter for TorqueGeneratedJSPromise.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSPromise<D,P>;

  // Torque type: (BigInt | class JSReceiver | class Undefined | Smi | class HeapNumber | class String | class Symbol | class Boolean | class Null | class PromiseReaction)
  // https://crsrc.org/c/v8/src/objects/js-promise.tq?l=34&c=3
  inline Tagged<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, PromiseReaction, Smi, String, Symbol, Undefined>> reactions_or_result() const;
  // https://crsrc.org/c/v8/src/objects/js-promise.tq?l=34&c=3
  inline Tagged<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, PromiseReaction, Smi, String, Symbol, Undefined>> reactions_or_result(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-promise.tq?l=34&c=3
  inline void set_reactions_or_result(Tagged<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, PromiseReaction, Smi, String, Symbol, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-promise.tq?l=35&c=3
  inline int flags() const;
  // https://crsrc.org/c/v8/src/objects/js-promise.tq?l=35&c=3
  inline void set_flags(int value);

  DECL_PRINTER(JSPromise)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-promise.tq?l=12&c=1
  V8_EXPORT_PRIVATE void JSPromiseVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-promise.tq?l=34&c=3
  static constexpr int kReactionsOrResultOffset = P::kHeaderSize;
  static constexpr int kReactionsOrResultOffsetEnd = kReactionsOrResultOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-promise.tq?l=35&c=3
  static constexpr int kFlagsOffset = kReactionsOrResultOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSPromise() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSPromise, DAlias>,
        "class TorqueGeneratedJSPromise should be used as direct base for JSPromise.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSPromise(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

