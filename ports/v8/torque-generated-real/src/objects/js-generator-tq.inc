class JSGeneratorObject;
class JSAsyncFunctionObject;
class JSAsyncGeneratorObject;
// Alias for IsJSGeneratorObject() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=5&c=1
V8_EXPORT_PRIVATE bool IsJSGeneratorObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-generator.tq?l=5&c=1
template <class D, class P>
class TorqueGeneratedJSGeneratorObject : public P {
  static_assert(
      std::is_same_v<JSGeneratorObject, D>,
      "Use this class as direct base for JSGeneratorObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSGeneratorObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSGeneratorObject<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=6&c=3
  inline Tagged<JSFunction> function() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=6&c=3
  inline Tagged<JSFunction> function(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=6&c=3
  inline void set_function(Tagged<JSFunction> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=7&c=3
  inline Tagged<Context> context() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=7&c=3
  inline Tagged<Context> context(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=7&c=3
  inline void set_context(Tagged<Context> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: JSAny
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=8&c=3
  inline Tagged<JSAny> receiver() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=8&c=3
  inline Tagged<JSAny> receiver(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=8&c=3
  inline void set_receiver(Tagged<JSAny> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=14&c=3
  inline Tagged<Object> input_or_debug_pos() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=14&c=3
  inline Tagged<Object> input_or_debug_pos(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=14&c=3
  inline void set_input_or_debug_pos(Tagged<Object> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=17&c=3
  inline int resume_mode() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=17&c=3
  inline void set_resume_mode(int value);

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=22&c=3
  inline int continuation() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=22&c=3
  inline void set_continuation(int value);

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=25&c=3
  inline Tagged<FixedArray> parameters_and_registers() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=25&c=3
  inline Tagged<FixedArray> parameters_and_registers(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=25&c=3
  inline void set_parameters_and_registers(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSGeneratorObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=5&c=1
  V8_EXPORT_PRIVATE void JSGeneratorObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=6&c=3
  static constexpr int kFunctionOffset = P::kHeaderSize;
  static constexpr int kFunctionOffsetEnd = kFunctionOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=7&c=3
  static constexpr int kContextOffset = kFunctionOffsetEnd + 1;
  static constexpr int kContextOffsetEnd = kContextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=8&c=3
  static constexpr int kReceiverOffset = kContextOffsetEnd + 1;
  static constexpr int kReceiverOffsetEnd = kReceiverOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=14&c=3
  static constexpr int kInputOrDebugPosOffset = kReceiverOffsetEnd + 1;
  static constexpr int kInputOrDebugPosOffsetEnd = kInputOrDebugPosOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=17&c=3
  static constexpr int kResumeModeOffset = kInputOrDebugPosOffsetEnd + 1;
  static constexpr int kResumeModeOffsetEnd = kResumeModeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=22&c=3
  static constexpr int kContinuationOffset = kResumeModeOffsetEnd + 1;
  static constexpr int kContinuationOffsetEnd = kContinuationOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=25&c=3
  static constexpr int kParametersAndRegistersOffset = kContinuationOffsetEnd + 1;
  static constexpr int kParametersAndRegistersOffsetEnd = kParametersAndRegistersOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kParametersAndRegistersOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kParametersAndRegistersOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kParametersAndRegistersOffsetEnd + 1;
  static constexpr int kHeaderSize = kParametersAndRegistersOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSGeneratorObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSGeneratorObject, DAlias>,
        "class TorqueGeneratedJSGeneratorObject should be used as direct base for JSGeneratorObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSGeneratorObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSAsyncFunctionObject() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=28&c=1
V8_EXPORT_PRIVATE bool IsJSAsyncFunctionObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-generator.tq?l=28&c=1
template <class D, class P>
class TorqueGeneratedJSAsyncFunctionObject : public P {
  static_assert(
      std::is_same_v<JSAsyncFunctionObject, D>,
      "Use this class as direct base for JSAsyncFunctionObject.");
  static_assert(
      std::is_same_v<JSGeneratorObject, P>,
      "Pass in JSGeneratorObject as second template parameter for TorqueGeneratedJSAsyncFunctionObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSAsyncFunctionObject<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=29&c=3
  inline Tagged<JSPromise> promise() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=29&c=3
  inline Tagged<JSPromise> promise(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=29&c=3
  inline void set_promise(Tagged<JSPromise> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSFunction | class Undefined)
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=32&c=3
  inline Tagged<Union<JSFunction, Undefined>> await_resolve_closure() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=32&c=3
  inline Tagged<Union<JSFunction, Undefined>> await_resolve_closure(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=32&c=3
  inline void set_await_resolve_closure(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSFunction | class Undefined)
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=33&c=3
  inline Tagged<Union<JSFunction, Undefined>> await_reject_closure() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=33&c=3
  inline Tagged<Union<JSFunction, Undefined>> await_reject_closure(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=33&c=3
  inline void set_await_reject_closure(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSAsyncFunctionObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=28&c=1
  V8_EXPORT_PRIVATE void JSAsyncFunctionObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=29&c=3
  static constexpr int kPromiseOffset = P::kHeaderSize;
  static constexpr int kPromiseOffsetEnd = kPromiseOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=32&c=3
  static constexpr int kAwaitResolveClosureOffset = kPromiseOffsetEnd + 1;
  static constexpr int kAwaitResolveClosureOffsetEnd = kAwaitResolveClosureOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=33&c=3
  static constexpr int kAwaitRejectClosureOffset = kAwaitResolveClosureOffsetEnd + 1;
  static constexpr int kAwaitRejectClosureOffsetEnd = kAwaitRejectClosureOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kAwaitRejectClosureOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kAwaitRejectClosureOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kAwaitRejectClosureOffsetEnd + 1;
  static constexpr int kHeaderSize = kAwaitRejectClosureOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSAsyncFunctionObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSAsyncFunctionObject, DAlias>,
        "class TorqueGeneratedJSAsyncFunctionObject should be used as direct base for JSAsyncFunctionObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSAsyncFunctionObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSAsyncGeneratorObject() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-generator.tq?l=36&c=1
V8_EXPORT_PRIVATE bool IsJSAsyncGeneratorObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-generator.tq?l=36&c=1
template <class D, class P>
class TorqueGeneratedJSAsyncGeneratorObject : public P {
  static_assert(
      std::is_same_v<JSAsyncGeneratorObject, D>,
      "Use this class as direct base for JSAsyncGeneratorObject.");
  static_assert(
      std::is_same_v<JSGeneratorObject, P>,
      "Pass in JSGeneratorObject as second template parameter for TorqueGeneratedJSAsyncGeneratorObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSAsyncGeneratorObject<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=39&c=3
  inline Tagged<HeapObject> queue() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=39&c=3
  inline Tagged<HeapObject> queue(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=39&c=3
  inline void set_queue(Tagged<HeapObject> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=41&c=3
  inline int is_awaiting() const;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=41&c=3
  inline void set_is_awaiting(int value);

  DECL_PRINTER(JSAsyncGeneratorObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=36&c=1
  V8_EXPORT_PRIVATE void JSAsyncGeneratorObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=39&c=3
  static constexpr int kQueueOffset = P::kHeaderSize;
  static constexpr int kQueueOffsetEnd = kQueueOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-generator.tq?l=41&c=3
  static constexpr int kIsAwaitingOffset = kQueueOffsetEnd + 1;
  static constexpr int kIsAwaitingOffsetEnd = kIsAwaitingOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kIsAwaitingOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kIsAwaitingOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kIsAwaitingOffsetEnd + 1;
  static constexpr int kHeaderSize = kIsAwaitingOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSAsyncGeneratorObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSAsyncGeneratorObject, DAlias>,
        "class TorqueGeneratedJSAsyncGeneratorObject should be used as direct base for JSAsyncGeneratorObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSAsyncGeneratorObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

