class JSModuleNamespace;
class JSDeferredModuleNamespace;
// Alias for IsJSModuleNamespace() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/module.tq?l=22&c=1
V8_EXPORT_PRIVATE bool IsJSModuleNamespace_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/module.tq?l=22&c=1
template <class D, class P>
class TorqueGeneratedJSModuleNamespace : public P {
  static_assert(
      std::is_same_v<JSModuleNamespace, D>,
      "Use this class as direct base for JSModuleNamespace.");
  static_assert(
      std::is_same_v<JSSpecialObject, P>,
      "Pass in JSSpecialObject as second template parameter for TorqueGeneratedJSModuleNamespace.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSModuleNamespace<D,P>;

  // https://crsrc.org/c/v8/src/objects/module.tq?l=23&c=3
  inline Tagged<Module> module() const;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=23&c=3
  inline Tagged<Module> module(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=23&c=3
  inline void set_module(Tagged<Module> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(JSModuleNamespace)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/module.tq?l=22&c=1
  V8_EXPORT_PRIVATE void JSModuleNamespaceVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=23&c=3
  static constexpr int kModuleOffset = P::kHeaderSize;
  static constexpr int kModuleOffsetEnd = kModuleOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kModuleOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kModuleOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kModuleOffsetEnd + 1;
  static constexpr int kHeaderSize = kModuleOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSModuleNamespace() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSModuleNamespace, DAlias>,
        "class TorqueGeneratedJSModuleNamespace should be used as direct base for JSModuleNamespace.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSModuleNamespace(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsJSDeferredModuleNamespace() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/module.tq?l=26&c=1
V8_EXPORT_PRIVATE bool IsJSDeferredModuleNamespace_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/module.tq?l=26&c=1
template <class D, class P>
class TorqueGeneratedJSDeferredModuleNamespace : public P {
  static_assert(
      std::is_same_v<JSDeferredModuleNamespace, D>,
      "Use this class as direct base for JSDeferredModuleNamespace.");
  static_assert(
      std::is_same_v<JSModuleNamespace, P>,
      "Pass in JSModuleNamespace as second template parameter for TorqueGeneratedJSDeferredModuleNamespace.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedJSDeferredModuleNamespace<D,P>;

  DECL_PRINTER(JSDeferredModuleNamespace)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/module.tq?l=26&c=1
  V8_EXPORT_PRIVATE void JSDeferredModuleNamespaceVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedJSDeferredModuleNamespace() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedJSDeferredModuleNamespace, DAlias>,
        "class TorqueGeneratedJSDeferredModuleNamespace should be used as direct base for JSDeferredModuleNamespace.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedJSDeferredModuleNamespace(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

