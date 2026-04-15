class JSFinalizationRegistry;
class JSWeakRef;
// Alias for IsJSFinalizationRegistry() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=9&c=1
V8_EXPORT_PRIVATE bool IsJSFinalizationRegistry_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=9&c=1
template <class D, class P>
class TorqueGeneratedJSFinalizationRegistry : public P {
  static_assert(
      std::is_same_v<JSFinalizationRegistry, D>,
      "Use this class as direct base for JSFinalizationRegistry.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSFinalizationRegistry.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSFinalizationRegistry<D,P>;

  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=10&c=3
  inline Tagged<NativeContext> native_context() const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=10&c=3
  inline Tagged<NativeContext> native_context(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=10&c=3
  inline void set_native_context(Tagged<NativeContext> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: Callable
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=11&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> cleanup() const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=11&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> cleanup(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=11&c=3
  inline void set_cleanup(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class WeakCell)
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=12&c=3
  inline Tagged<Union<Undefined, WeakCell>> active_cells() const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=12&c=3
  inline Tagged<Union<Undefined, WeakCell>> active_cells(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=12&c=3
  inline void set_active_cells(Tagged<Union<Undefined, WeakCell>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class WeakCell)
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=13&c=3
  inline Tagged<Union<Undefined, WeakCell>> cleared_cells() const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=13&c=3
  inline Tagged<Union<Undefined, WeakCell>> cleared_cells(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=13&c=3
  inline void set_cleared_cells(Tagged<Union<Undefined, WeakCell>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=14&c=3
  inline Tagged<Object> key_map() const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=14&c=3
  inline Tagged<Object> key_map(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=14&c=3
  inline void set_key_map(Tagged<Object> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class JSFinalizationRegistry)
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=17&c=3
  inline Tagged<Union<JSFinalizationRegistry, Undefined>> next_dirty() const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=17&c=3
  inline Tagged<Union<JSFinalizationRegistry, Undefined>> next_dirty(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=17&c=3
  inline void set_next_dirty(Tagged<Union<JSFinalizationRegistry, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=18&c=3
  inline int flags() const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=18&c=3
  inline void set_flags(int value);

  DECL_PRINTER(JSFinalizationRegistry)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=9&c=1
  V8_EXPORT_PRIVATE void JSFinalizationRegistryVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=10&c=3
  static constexpr int kNativeContextOffset = P::kHeaderSize;
  static constexpr int kNativeContextOffsetEnd = kNativeContextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=11&c=3
  static constexpr int kCleanupOffset = kNativeContextOffsetEnd + 1;
  static constexpr int kCleanupOffsetEnd = kCleanupOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=12&c=3
  static constexpr int kActiveCellsOffset = kCleanupOffsetEnd + 1;
  static constexpr int kActiveCellsOffsetEnd = kActiveCellsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=13&c=3
  static constexpr int kClearedCellsOffset = kActiveCellsOffsetEnd + 1;
  static constexpr int kClearedCellsOffsetEnd = kClearedCellsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=14&c=3
  static constexpr int kKeyMapOffset = kClearedCellsOffsetEnd + 1;
  static constexpr int kKeyMapOffsetEnd = kKeyMapOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=17&c=3
  static constexpr int kNextDirtyOffset = kKeyMapOffsetEnd + 1;
  static constexpr int kNextDirtyOffsetEnd = kNextDirtyOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=18&c=3
  static constexpr int kFlagsOffset = kNextDirtyOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSFinalizationRegistry() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSFinalizationRegistry, DAlias>,
        "class TorqueGeneratedJSFinalizationRegistry should be used as direct base for JSFinalizationRegistry.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSFinalizationRegistry(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSWeakRef() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=43&c=1
V8_EXPORT_PRIVATE bool IsJSWeakRef_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=43&c=1
template <class D, class P>
class TorqueGeneratedJSWeakRef : public P {
  static_assert(
      std::is_same_v<JSWeakRef, D>,
      "Use this class as direct base for JSWeakRef.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedJSWeakRef.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSWeakRef<D,P>;

  // Torque type: (class JSReceiver | class Undefined | class Symbol)
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=44&c=3
  inline Tagged<Union<JSReceiver, Symbol, Undefined>> target() const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=44&c=3
  inline Tagged<Union<JSReceiver, Symbol, Undefined>> target(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=44&c=3
  inline void set_target(Tagged<Union<JSReceiver, Symbol, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSWeakRef)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=43&c=1
  V8_EXPORT_PRIVATE void JSWeakRefVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/js-weak-refs.tq?l=44&c=3
  static constexpr int kTargetOffset = P::kHeaderSize;
  static constexpr int kTargetOffsetEnd = kTargetOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kTargetOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kTargetOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kTargetOffsetEnd + 1;
  static constexpr int kHeaderSize = kTargetOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSWeakRef() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSWeakRef, DAlias>,
        "class TorqueGeneratedJSWeakRef should be used as direct base for JSWeakRef.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSWeakRef(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

