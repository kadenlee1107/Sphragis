class WasmInstanceObject;
class WasmImportData;
class WasmInternalFunction;
class WasmFuncRef;
class WasmFunctionData;
class WasmExportedFunctionData;
class WasmJSFunctionData;
class WasmCapiFunctionData;
class WasmResumeData;
class WasmSuspenderObject;
class WasmContinuationObject;
class WasmStackObject;
class WasmExceptionTag;
class WasmModuleObject;
class WasmTableObject;
class WasmMemoryObject;
class WasmMemoryMapDescriptor;
class WasmGlobalObject;
class WasmTagObject;
class AsmWasmData;
class WasmTypeInfo;
class WasmObject;
class WasmStruct;
class WasmArray;
class WasmNull;
class WasmSuspendingObject;
// Alias for IsWasmInstanceObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1
V8_EXPORT_PRIVATE bool IsWasmInstanceObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1
template <class D, class P>
class TorqueGeneratedWasmInstanceObject : public P {
  static_assert(
      std::is_same_v<WasmInstanceObject, D>,
      "Use this class as direct base for WasmInstanceObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedWasmInstanceObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmInstanceObject<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
  inline Tagged<WasmModuleObject> module_object() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
  inline Tagged<WasmModuleObject> module_object(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
  inline void set_module_object(Tagged<WasmModuleObject> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
  inline Tagged<JSObject> exports_object() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
  inline Tagged<JSObject> exports_object(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
  inline void set_exports_object(Tagged<JSObject> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(WasmInstanceObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1
  V8_EXPORT_PRIVATE void WasmInstanceObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=40&c=3
  static constexpr int kTrustedDataOffset = P::kHeaderSize;
  static constexpr int kTrustedDataOffsetEnd = kTrustedDataOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfStrongFieldsOffset = kTrustedDataOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
  static constexpr int kModuleObjectOffset = kTrustedDataOffsetEnd + 1;
  static constexpr int kModuleObjectOffsetEnd = kModuleObjectOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
  static constexpr int kExportsObjectOffset = kModuleObjectOffsetEnd + 1;
  static constexpr int kExportsObjectOffsetEnd = kExportsObjectOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kExportsObjectOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kExportsObjectOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kExportsObjectOffsetEnd + 1;
  static constexpr int kHeaderSize = kExportsObjectOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmInstanceObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmInstanceObject, DAlias>,
        "class TorqueGeneratedWasmInstanceObject should be used as direct base for WasmInstanceObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmInstanceObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmImportData() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1
V8_EXPORT_PRIVATE bool IsWasmImportData_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1
template <class D, class P>
class TorqueGeneratedWasmImportData : public P {
  static_assert(
      std::is_same_v<WasmImportData, D>,
      "Use this class as direct base for WasmImportData.");
  static_assert(
      std::is_same_v<TrustedObject, P>,
      "Pass in TrustedObject as second template parameter for TorqueGeneratedWasmImportData.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmImportData<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
  inline Tagged<NativeContext> native_context() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
  inline Tagged<NativeContext> native_context(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
  inline void set_native_context(Tagged<NativeContext> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSReceiver | class Undefined)
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
  inline Tagged<Union<JSReceiver, Undefined>> callable() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
  inline Tagged<Union<JSReceiver, Undefined>> callable(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
  inline void set_callable(Tagged<Union<JSReceiver, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
  inline Tagged<Cell> wrapper_budget() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
  inline Tagged<Cell> wrapper_budget(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
  inline void set_wrapper_budget(Tagged<Cell> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=64&c=3
  inline const wasm::CanonicalSig* sig() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=64&c=3
  inline void set_sig(const wasm::CanonicalSig* value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=66&c=3
  inline uint32_t bit_field() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=66&c=3
  inline void set_bit_field(uint32_t value);

  DECL_PRINTER(WasmImportData)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1
  V8_EXPORT_PRIVATE void WasmImportDataVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=53&c=3
  static constexpr int kProtectedInstanceDataOffset = P::kHeaderSize;
  static constexpr int kProtectedInstanceDataOffsetEnd = kProtectedInstanceDataOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=57&c=3
  static constexpr int kProtectedCallOriginOffset = kProtectedInstanceDataOffsetEnd + 1;
  static constexpr int kProtectedCallOriginOffsetEnd = kProtectedCallOriginOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kProtectedCallOriginOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kProtectedCallOriginOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
  static constexpr int kNativeContextOffset = kProtectedCallOriginOffsetEnd + 1;
  static constexpr int kNativeContextOffsetEnd = kNativeContextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
  static constexpr int kCallableOffset = kNativeContextOffsetEnd + 1;
  static constexpr int kCallableOffsetEnd = kCallableOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
  static constexpr int kWrapperBudgetOffset = kCallableOffsetEnd + 1;
  static constexpr int kWrapperBudgetOffsetEnd = kWrapperBudgetOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kWrapperBudgetOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=64&c=3
  static constexpr int kSigOffset = kWrapperBudgetOffsetEnd + 1;
  static constexpr int kSigOffsetEnd = kSigOffset + kSystemPointerSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=66&c=3
  static constexpr int kBitFieldOffset = kSigOffsetEnd + 1;
  static constexpr int kBitFieldOffsetEnd = kBitFieldOffset + kInt32Size - 1;
  static constexpr int kHeaderSize = kBitFieldOffsetEnd + 1;
  static constexpr int kSize = kBitFieldOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 36);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmImportData() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmImportData, DAlias>,
        "class TorqueGeneratedWasmImportData should be used as direct base for WasmImportData.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmImportData(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmInternalFunction() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1
V8_EXPORT_PRIVATE bool IsWasmInternalFunction_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1
template <class D, class P>
class TorqueGeneratedWasmInternalFunction : public P {
  static_assert(
      std::is_same_v<WasmInternalFunction, D>,
      "Use this class as direct base for WasmInternalFunction.");
  static_assert(
      std::is_same_v<ExposedTrustedObject, P>,
      "Pass in ExposedTrustedObject as second template parameter for TorqueGeneratedWasmInternalFunction.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmInternalFunction<D,P>;

  // Torque type: (class JSFunction | class Undefined)
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
  inline Tagged<Union<JSFunction, Undefined>> external() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
  inline Tagged<Union<JSFunction, Undefined>> external(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
  inline void set_external(Tagged<Union<JSFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=98&c=3
  inline int function_index() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=98&c=3
  inline void set_function_index(int value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=100&c=3
  inline uint32_t raw_call_target() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=100&c=3
  inline void set_raw_call_target(uint32_t value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=103&c=3
  inline const wasm::CanonicalSig* sig() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=103&c=3
  inline void set_sig(const wasm::CanonicalSig* value);

  DECL_PRINTER(WasmInternalFunction)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1
  V8_EXPORT_PRIVATE void WasmInternalFunctionVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=88&c=3
  static constexpr int kProtectedImplicitArgOffset = P::kHeaderSize;
  static constexpr int kProtectedImplicitArgOffsetEnd = kProtectedImplicitArgOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kProtectedImplicitArgOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kProtectedImplicitArgOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
  static constexpr int kExternalOffset = kProtectedImplicitArgOffsetEnd + 1;
  static constexpr int kExternalOffsetEnd = kExternalOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=98&c=3
  static constexpr int kFunctionIndexOffset = kExternalOffsetEnd + 1;
  static constexpr int kFunctionIndexOffsetEnd = kFunctionIndexOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFunctionIndexOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=100&c=3
  static constexpr int kRawCallTargetOffset = kFunctionIndexOffsetEnd + 1;
  static constexpr int kRawCallTargetOffsetEnd = kRawCallTargetOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=102&c=43
  static constexpr int kOptionalPaddingOffset = kRawCallTargetOffsetEnd + 1;
  static constexpr int kOptionalPaddingOffsetEnd = kOptionalPaddingOffset + 0 - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=103&c=3
  static constexpr int kSigOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kSigOffsetEnd = kSigOffset + kSystemPointerSize - 1;
  static constexpr int kHeaderSize = kSigOffsetEnd + 1;
  static constexpr int kSize = kSigOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 28);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmInternalFunction() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmInternalFunction, DAlias>,
        "class TorqueGeneratedWasmInternalFunction should be used as direct base for WasmInternalFunction.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmInternalFunction(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmFuncRef() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1
V8_EXPORT_PRIVATE bool IsWasmFuncRef_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1
template <class D, class P>
class TorqueGeneratedWasmFuncRef : public P {
  static_assert(
      std::is_same_v<WasmFuncRef, D>,
      "Use this class as direct base for WasmFuncRef.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedWasmFuncRef.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmFuncRef<D,P>;

  DECL_PRINTER(WasmFuncRef)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1
  V8_EXPORT_PRIVATE void WasmFuncRefVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=113&c=3
  static constexpr int kTrustedInternalOffset = P::kHeaderSize;
  static constexpr int kTrustedInternalOffsetEnd = kTrustedInternalOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kTrustedInternalOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kTrustedInternalOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kTrustedInternalOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kTrustedInternalOffsetEnd + 1;
  static constexpr int kHeaderSize = kTrustedInternalOffsetEnd + 1;
  static constexpr int kSize = kTrustedInternalOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 8);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmFuncRef() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmFuncRef, DAlias>,
        "class TorqueGeneratedWasmFuncRef should be used as direct base for WasmFuncRef.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmFuncRef(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmFunctionData() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1
V8_EXPORT_PRIVATE bool IsWasmFunctionData_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1
template <class D, class P>
class TorqueGeneratedWasmFunctionData : public P {
  static_assert(
      std::is_same_v<WasmFunctionData, D>,
      "Use this class as direct base for WasmFunctionData.");
  static_assert(
      std::is_same_v<ExposedTrustedObject, P>,
      "Pass in ExposedTrustedObject as second template parameter for TorqueGeneratedWasmFunctionData.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmFunctionData<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
  inline Tagged<WasmFuncRef> func_ref() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
  inline Tagged<WasmFuncRef> func_ref(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
  inline void set_func_ref(Tagged<WasmFuncRef> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=127&c=3
  inline int js_promise_flags() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=127&c=3
  inline void set_js_promise_flags(int value);

  DECL_PRINTER(WasmFunctionData)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1
  V8_EXPORT_PRIVATE void WasmFunctionDataVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=122&c=3
  static constexpr int kWrapperCodeOffset = P::kHeaderSize;
  static constexpr int kWrapperCodeOffsetEnd = kWrapperCodeOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfStrongFieldsOffset = kWrapperCodeOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
  static constexpr int kFuncRefOffset = kWrapperCodeOffsetEnd + 1;
  static constexpr int kFuncRefOffsetEnd = kFuncRefOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=127&c=3
  static constexpr int kJsPromiseFlagsOffset = kFuncRefOffsetEnd + 1;
  static constexpr int kJsPromiseFlagsOffsetEnd = kJsPromiseFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kJsPromiseFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kJsPromiseFlagsOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=130&c=3
  static constexpr int kProtectedInternalOffset = kJsPromiseFlagsOffsetEnd + 1;
  static constexpr int kProtectedInternalOffsetEnd = kProtectedInternalOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kProtectedInternalOffsetEnd + 1;
  static constexpr int kHeaderSize = kProtectedInternalOffsetEnd + 1;
  static constexpr int kSize = kProtectedInternalOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 20);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmFunctionData() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmFunctionData, DAlias>,
        "class TorqueGeneratedWasmFunctionData should be used as direct base for WasmFunctionData.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmFunctionData(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmExportedFunctionData() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1
V8_EXPORT_PRIVATE bool IsWasmExportedFunctionData_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1
template <class D, class P>
class TorqueGeneratedWasmExportedFunctionData : public P {
  static_assert(
      std::is_same_v<WasmExportedFunctionData, D>,
      "Use this class as direct base for WasmExportedFunctionData.");
  static_assert(
      std::is_same_v<WasmFunctionData, P>,
      "Pass in WasmFunctionData as second template parameter for TorqueGeneratedWasmExportedFunctionData.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmExportedFunctionData<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=141&c=3
  inline int function_index() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=141&c=3
  inline void set_function_index(int value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
  inline Tagged<Cell> wrapper_budget() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
  inline Tagged<Cell> wrapper_budget(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
  inline void set_wrapper_budget(Tagged<Cell> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=148&c=3
  inline int packed_args_size() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=148&c=3
  inline void set_packed_args_size(int value);

  DECL_PRINTER(WasmExportedFunctionData)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1
  V8_EXPORT_PRIVATE void WasmExportedFunctionDataVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=140&c=3
  static constexpr int kProtectedInstanceDataOffset = P::kHeaderSize;
  static constexpr int kProtectedInstanceDataOffsetEnd = kProtectedInstanceDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kProtectedInstanceDataOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kProtectedInstanceDataOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=141&c=3
  static constexpr int kFunctionIndexOffset = kProtectedInstanceDataOffsetEnd + 1;
  static constexpr int kFunctionIndexOffsetEnd = kFunctionIndexOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
  static constexpr int kWrapperBudgetOffset = kFunctionIndexOffsetEnd + 1;
  static constexpr int kWrapperBudgetOffsetEnd = kWrapperBudgetOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=148&c=3
  static constexpr int kPackedArgsSizeOffset = kWrapperBudgetOffsetEnd + 1;
  static constexpr int kPackedArgsSizeOffsetEnd = kPackedArgsSizeOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPackedArgsSizeOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=149&c=3
  static constexpr int kCWrapperCodeOffset = kPackedArgsSizeOffsetEnd + 1;
  static constexpr int kCWrapperCodeOffsetEnd = kCWrapperCodeOffset + kTrustedPointerSize - 1;
  static constexpr int kHeaderSize = kCWrapperCodeOffsetEnd + 1;
  static constexpr int kSize = kCWrapperCodeOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 40);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmExportedFunctionData() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmExportedFunctionData, DAlias>,
        "class TorqueGeneratedWasmExportedFunctionData should be used as direct base for WasmExportedFunctionData.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmExportedFunctionData(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmJSFunctionData() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1
V8_EXPORT_PRIVATE bool IsWasmJSFunctionData_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1
template <class D, class P>
class TorqueGeneratedWasmJSFunctionData : public P {
  static_assert(
      std::is_same_v<WasmJSFunctionData, D>,
      "Use this class as direct base for WasmJSFunctionData.");
  static_assert(
      std::is_same_v<WasmFunctionData, P>,
      "Pass in WasmFunctionData as second template parameter for TorqueGeneratedWasmJSFunctionData.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmJSFunctionData<D,P>;

  DECL_PRINTER(WasmJSFunctionData)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1
  V8_EXPORT_PRIVATE void WasmJSFunctionDataVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=157&c=3
  static constexpr int kProtectedOffheapDataOffset = P::kHeaderSize;
  static constexpr int kProtectedOffheapDataOffsetEnd = kProtectedOffheapDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kProtectedOffheapDataOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kProtectedOffheapDataOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kProtectedOffheapDataOffsetEnd + 1;
  static constexpr int kHeaderSize = kProtectedOffheapDataOffsetEnd + 1;
  static constexpr int kSize = kProtectedOffheapDataOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 24);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmJSFunctionData() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmJSFunctionData, DAlias>,
        "class TorqueGeneratedWasmJSFunctionData should be used as direct base for WasmJSFunctionData.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmJSFunctionData(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmCapiFunctionData() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1
V8_EXPORT_PRIVATE bool IsWasmCapiFunctionData_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1
template <class D, class P>
class TorqueGeneratedWasmCapiFunctionData : public P {
  static_assert(
      std::is_same_v<WasmCapiFunctionData, D>,
      "Use this class as direct base for WasmCapiFunctionData.");
  static_assert(
      std::is_same_v<WasmFunctionData, P>,
      "Pass in WasmFunctionData as second template parameter for TorqueGeneratedWasmCapiFunctionData.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmCapiFunctionData<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
  inline Tagged<Foreign> embedder_data() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
  inline Tagged<Foreign> embedder_data(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
  inline void set_embedder_data(Tagged<Foreign> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(WasmCapiFunctionData)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1
  V8_EXPORT_PRIVATE void WasmCapiFunctionDataVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
  static constexpr int kEmbedderDataOffset = P::kHeaderSize;
  static constexpr int kEmbedderDataOffsetEnd = kEmbedderDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kEmbedderDataOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kEmbedderDataOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kEmbedderDataOffsetEnd + 1;
  static constexpr int kHeaderSize = kEmbedderDataOffsetEnd + 1;
  static constexpr int kSize = kEmbedderDataOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 24);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmCapiFunctionData() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmCapiFunctionData, DAlias>,
        "class TorqueGeneratedWasmCapiFunctionData should be used as direct base for WasmCapiFunctionData.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmCapiFunctionData(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmResumeData() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1
V8_EXPORT_PRIVATE bool IsWasmResumeData_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1
template <class D, class P>
class TorqueGeneratedWasmResumeData : public P {
  static_assert(
      std::is_same_v<WasmResumeData, D>,
      "Use this class as direct base for WasmResumeData.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedWasmResumeData.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmResumeData<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=167&c=3
  inline int on_resume() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=167&c=3
  inline void set_on_resume(int value);

  DECL_PRINTER(WasmResumeData)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1
  V8_EXPORT_PRIVATE void WasmResumeDataVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=166&c=3
  static constexpr int kTrustedSuspenderOffset = P::kHeaderSize;
  static constexpr int kTrustedSuspenderOffsetEnd = kTrustedSuspenderOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfStrongFieldsOffset = kTrustedSuspenderOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=167&c=3
  static constexpr int kOnResumeOffset = kTrustedSuspenderOffsetEnd + 1;
  static constexpr int kOnResumeOffsetEnd = kOnResumeOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kOnResumeOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kOnResumeOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kOnResumeOffsetEnd + 1;
  static constexpr int kHeaderSize = kOnResumeOffsetEnd + 1;
  static constexpr int kSize = kOnResumeOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmResumeData() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmResumeData, DAlias>,
        "class TorqueGeneratedWasmResumeData should be used as direct base for WasmResumeData.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmResumeData(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmSuspenderObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1
V8_EXPORT_PRIVATE bool IsWasmSuspenderObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1
template <class D, class P>
class TorqueGeneratedWasmSuspenderObject : public P {
  static_assert(
      std::is_same_v<WasmSuspenderObject, D>,
      "Use this class as direct base for WasmSuspenderObject.");
  static_assert(
      std::is_same_v<ExposedTrustedObject, P>,
      "Pass in ExposedTrustedObject as second template parameter for TorqueGeneratedWasmSuspenderObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmSuspenderObject<D,P>;

  // Torque type: (class Undefined | class JSPromise)
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
  inline Tagged<Union<JSPromise, Undefined>> promise() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
  inline Tagged<Union<JSPromise, Undefined>> promise(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
  inline void set_promise(Tagged<Union<JSPromise, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSObject | class Undefined)
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
  inline Tagged<Union<JSObject, Undefined>> resume() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
  inline Tagged<Union<JSObject, Undefined>> resume(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
  inline void set_resume(Tagged<Union<JSObject, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSObject | class Undefined)
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
  inline Tagged<Union<JSObject, Undefined>> reject() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
  inline Tagged<Union<JSObject, Undefined>> reject(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
  inline void set_reject(Tagged<Union<JSObject, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(WasmSuspenderObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1
  V8_EXPORT_PRIVATE void WasmSuspenderObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=171&c=3
  static constexpr int kStackOffset = P::kHeaderSize;
  static constexpr int kStackOffsetEnd = kStackOffset + kExternalPointerSlotSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kStackOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=172&c=3
  static constexpr int kParentOffset = kStackOffsetEnd + 1;
  static constexpr int kParentOffsetEnd = kParentOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kParentOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kParentOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
  static constexpr int kPromiseOffset = kParentOffsetEnd + 1;
  static constexpr int kPromiseOffsetEnd = kPromiseOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
  static constexpr int kResumeOffset = kPromiseOffsetEnd + 1;
  static constexpr int kResumeOffsetEnd = kResumeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
  static constexpr int kRejectOffset = kResumeOffsetEnd + 1;
  static constexpr int kRejectOffsetEnd = kRejectOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kRejectOffsetEnd + 1;
  static constexpr int kHeaderSize = kRejectOffsetEnd + 1;
  static constexpr int kSize = kRejectOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 28);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmSuspenderObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmSuspenderObject, DAlias>,
        "class TorqueGeneratedWasmSuspenderObject should be used as direct base for WasmSuspenderObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmSuspenderObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmContinuationObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1
V8_EXPORT_PRIVATE bool IsWasmContinuationObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1
template <class D, class P>
class TorqueGeneratedWasmContinuationObject : public P {
  static_assert(
      std::is_same_v<WasmContinuationObject, D>,
      "Use this class as direct base for WasmContinuationObject.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedWasmContinuationObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmContinuationObject<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
  inline Tagged<WasmStackObject> stack_obj() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
  inline Tagged<WasmStackObject> stack_obj(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
  inline void set_stack_obj(Tagged<WasmStackObject> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(WasmContinuationObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1
  V8_EXPORT_PRIVATE void WasmContinuationObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
  static constexpr int kStackObjOffset = P::kHeaderSize;
  static constexpr int kStackObjOffsetEnd = kStackObjOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kStackObjOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kStackObjOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kStackObjOffsetEnd + 1;
  static constexpr int kHeaderSize = kStackObjOffsetEnd + 1;
  static constexpr int kSize = kStackObjOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 8);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmContinuationObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmContinuationObject, DAlias>,
        "class TorqueGeneratedWasmContinuationObject should be used as direct base for WasmContinuationObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmContinuationObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmStackObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1
V8_EXPORT_PRIVATE bool IsWasmStackObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1
template <class D, class P>
class TorqueGeneratedWasmStackObject : public P {
  static_assert(
      std::is_same_v<WasmStackObject, D>,
      "Use this class as direct base for WasmStackObject.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedWasmStackObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmStackObject<D,P>;

  DECL_PRINTER(WasmStackObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1
  V8_EXPORT_PRIVATE void WasmStackObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=185&c=3
  static constexpr int kStackOffset = P::kHeaderSize;
  static constexpr int kStackOffsetEnd = kStackOffset + kExternalPointerSlotSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kStackOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kStackOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kStackOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kStackOffsetEnd + 1;
  static constexpr int kHeaderSize = kStackOffsetEnd + 1;
  static constexpr int kSize = kStackOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmStackObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmStackObject, DAlias>,
        "class TorqueGeneratedWasmStackObject should be used as direct base for WasmStackObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmStackObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmExceptionTag() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1
V8_EXPORT_PRIVATE bool IsWasmExceptionTag_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1
template <class D, class P>
class TorqueGeneratedWasmExceptionTag : public P {
  static_assert(
      std::is_same_v<WasmExceptionTag, D>,
      "Use this class as direct base for WasmExceptionTag.");
  static_assert(
      std::is_same_v<Struct, P>,
      "Pass in Struct as second template parameter for TorqueGeneratedWasmExceptionTag.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmExceptionTag<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=192&c=3
  inline int index() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=192&c=3
  inline void set_index(int value);

  DECL_PRINTER(WasmExceptionTag)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1
  V8_EXPORT_PRIVATE void WasmExceptionTagVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=192&c=3
  static constexpr int kIndexOffset = P::kHeaderSize;
  static constexpr int kIndexOffsetEnd = kIndexOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kIndexOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kIndexOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kIndexOffsetEnd + 1;
  static constexpr int kHeaderSize = kIndexOffsetEnd + 1;
  static constexpr int kSize = kIndexOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 8);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmExceptionTag() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmExceptionTag, DAlias>,
        "class TorqueGeneratedWasmExceptionTag should be used as direct base for WasmExceptionTag.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmExceptionTag(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmModuleObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1
V8_EXPORT_PRIVATE bool IsWasmModuleObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1
template <class D, class P>
class TorqueGeneratedWasmModuleObject : public P {
  static_assert(
      std::is_same_v<WasmModuleObject, D>,
      "Use this class as direct base for WasmModuleObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedWasmModuleObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmModuleObject<D,P>;

  // Torque type: ManagedWasmNativeModule
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
  inline Tagged<Managed<wasm::NativeModule>> managed_native_module() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
  inline Tagged<Managed<wasm::NativeModule>> managed_native_module(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
  inline void set_managed_native_module(Tagged<Managed<wasm::NativeModule>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
  inline Tagged<Script> script() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
  inline Tagged<Script> script(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
  inline void set_script(Tagged<Script> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(WasmModuleObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1
  V8_EXPORT_PRIVATE void WasmModuleObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
  static constexpr int kManagedNativeModuleOffset = P::kHeaderSize;
  static constexpr int kManagedNativeModuleOffsetEnd = kManagedNativeModuleOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
  static constexpr int kScriptOffset = kManagedNativeModuleOffsetEnd + 1;
  static constexpr int kScriptOffsetEnd = kScriptOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kScriptOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kScriptOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kScriptOffsetEnd + 1;
  static constexpr int kHeaderSize = kScriptOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmModuleObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmModuleObject, DAlias>,
        "class TorqueGeneratedWasmModuleObject should be used as direct base for WasmModuleObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmModuleObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmTableObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1
V8_EXPORT_PRIVATE bool IsWasmTableObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1
template <class D, class P>
class TorqueGeneratedWasmTableObject : public P {
  static_assert(
      std::is_same_v<WasmTableObject, D>,
      "Use this class as direct base for WasmTableObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedWasmTableObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmTableObject<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
  inline Tagged<FixedArray> entries() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
  inline Tagged<FixedArray> entries(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
  inline void set_entries(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=215&c=3
  inline int current_length() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=215&c=3
  inline void set_current_length(int value);

  // Torque type: (BigInt | class Undefined | Smi | class HeapNumber)
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
  inline Tagged<Union<BigInt, HeapNumber, Smi, Undefined>> maximum_length() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
  inline Tagged<Union<BigInt, HeapNumber, Smi, Undefined>> maximum_length(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
  inline void set_maximum_length(Tagged<Union<BigInt, HeapNumber, Smi, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=224&c=3
  inline int raw_type() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=224&c=3
  inline void set_raw_type(int value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=231&c=3
  inline wasm::AddressType address_type() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=231&c=3
  inline void set_address_type(wasm::AddressType value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=233&c=3
  inline uint8_t padding_for_address_type_0() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=233&c=3
  inline void set_padding_for_address_type_0(uint8_t value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=234&c=3
  inline uint16_t padding_for_address_type_1() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=234&c=3
  inline void set_padding_for_address_type_1(uint16_t value);

  DECL_PRINTER(WasmTableObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1
  V8_EXPORT_PRIVATE void WasmTableObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
  static constexpr int kEntriesOffset = P::kHeaderSize;
  static constexpr int kEntriesOffsetEnd = kEntriesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=215&c=3
  static constexpr int kCurrentLengthOffset = kEntriesOffsetEnd + 1;
  static constexpr int kCurrentLengthOffsetEnd = kCurrentLengthOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
  static constexpr int kMaximumLengthOffset = kCurrentLengthOffsetEnd + 1;
  static constexpr int kMaximumLengthOffsetEnd = kMaximumLengthOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=224&c=3
  static constexpr int kRawTypeOffset = kMaximumLengthOffsetEnd + 1;
  static constexpr int kRawTypeOffsetEnd = kRawTypeOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kRawTypeOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=226&c=3
  static constexpr int kTrustedDispatchTableOffset = kRawTypeOffsetEnd + 1;
  static constexpr int kTrustedDispatchTableOffsetEnd = kTrustedDispatchTableOffset + kTrustedPointerSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=230&c=3
  static constexpr int kTrustedDataOffset = kTrustedDispatchTableOffsetEnd + 1;
  static constexpr int kTrustedDataOffsetEnd = kTrustedDataOffset + kTrustedPointerSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=231&c=3
  static constexpr int kAddressTypeOffset = kTrustedDataOffsetEnd + 1;
  static constexpr int kAddressTypeOffsetEnd = kAddressTypeOffset + kUInt8Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=233&c=3
  static constexpr int kPaddingForAddressType0Offset = kAddressTypeOffsetEnd + 1;
  static constexpr int kPaddingForAddressType0OffsetEnd = kPaddingForAddressType0Offset + kUInt8Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=234&c=3
  static constexpr int kPaddingForAddressType1Offset = kPaddingForAddressType0OffsetEnd + 1;
  static constexpr int kPaddingForAddressType1OffsetEnd = kPaddingForAddressType1Offset + kUInt16Size - 1;
  static constexpr int kStartOfWeakFieldsOffset = kPaddingForAddressType1OffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPaddingForAddressType1OffsetEnd + 1;
  static constexpr int kHeaderSize = kPaddingForAddressType1OffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmTableObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmTableObject, DAlias>,
        "class TorqueGeneratedWasmTableObject should be used as direct base for WasmTableObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmTableObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmMemoryObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1
V8_EXPORT_PRIVATE bool IsWasmMemoryObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1
template <class D, class P>
class TorqueGeneratedWasmMemoryObject : public P {
  static_assert(
      std::is_same_v<WasmMemoryObject, D>,
      "Use this class as direct base for WasmMemoryObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedWasmMemoryObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmMemoryObject<D,P>;

  // Torque type: (class Undefined | class JSArrayBuffer)
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
  inline Tagged<Union<JSArrayBuffer, Undefined>> array_buffer() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
  inline Tagged<Union<JSArrayBuffer, Undefined>> array_buffer(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
  inline void set_array_buffer(Tagged<Union<JSArrayBuffer, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: ManagedBackingStore
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
  inline Tagged<Managed<BackingStore>> managed_backing_store() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
  inline Tagged<Managed<BackingStore>> managed_backing_store(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
  inline void set_managed_backing_store(Tagged<Managed<BackingStore>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=248&c=3
  inline int maximum_pages() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=248&c=3
  inline void set_maximum_pages(int value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
  inline Tagged<WeakArrayList> instances() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
  inline Tagged<WeakArrayList> instances(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
  inline void set_instances(Tagged<WeakArrayList> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=250&c=3
  inline wasm::AddressType address_type() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=250&c=3
  inline void set_address_type(wasm::AddressType value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=252&c=3
  inline uint8_t padding_for_flags_0() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=252&c=3
  inline void set_padding_for_flags_0(uint8_t value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=253&c=3
  inline uint16_t padding_for_flags_1() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=253&c=3
  inline void set_padding_for_flags_1(uint16_t value);

  DECL_PRINTER(WasmMemoryObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1
  V8_EXPORT_PRIVATE void WasmMemoryObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
  static constexpr int kArrayBufferOffset = P::kHeaderSize;
  static constexpr int kArrayBufferOffsetEnd = kArrayBufferOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
  static constexpr int kManagedBackingStoreOffset = kArrayBufferOffsetEnd + 1;
  static constexpr int kManagedBackingStoreOffsetEnd = kManagedBackingStoreOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=248&c=3
  static constexpr int kMaximumPagesOffset = kManagedBackingStoreOffsetEnd + 1;
  static constexpr int kMaximumPagesOffsetEnd = kMaximumPagesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
  static constexpr int kInstancesOffset = kMaximumPagesOffsetEnd + 1;
  static constexpr int kInstancesOffsetEnd = kInstancesOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kInstancesOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=250&c=3
  static constexpr int kAddressTypeOffset = kInstancesOffsetEnd + 1;
  static constexpr int kAddressTypeOffsetEnd = kAddressTypeOffset + kUInt8Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=252&c=3
  static constexpr int kPaddingForFlags0Offset = kAddressTypeOffsetEnd + 1;
  static constexpr int kPaddingForFlags0OffsetEnd = kPaddingForFlags0Offset + kUInt8Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=253&c=3
  static constexpr int kPaddingForFlags1Offset = kPaddingForFlags0OffsetEnd + 1;
  static constexpr int kPaddingForFlags1OffsetEnd = kPaddingForFlags1Offset + kUInt16Size - 1;
  static constexpr int kStartOfWeakFieldsOffset = kPaddingForFlags1OffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPaddingForFlags1OffsetEnd + 1;
  static constexpr int kHeaderSize = kPaddingForFlags1OffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmMemoryObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmMemoryObject, DAlias>,
        "class TorqueGeneratedWasmMemoryObject should be used as direct base for WasmMemoryObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmMemoryObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmMemoryMapDescriptor() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1
V8_EXPORT_PRIVATE bool IsWasmMemoryMapDescriptor_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1
template <class D, class P>
class TorqueGeneratedWasmMemoryMapDescriptor : public P {
  static_assert(
      std::is_same_v<WasmMemoryMapDescriptor, D>,
      "Use this class as direct base for WasmMemoryMapDescriptor.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedWasmMemoryMapDescriptor.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmMemoryMapDescriptor<D,P>;

  // Torque type: Weak<class WasmMemoryObject>
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
  inline Tagged<Weak<HeapObject>> memory() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
  inline Tagged<Weak<HeapObject>> memory(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
  inline void set_memory(Tagged<Weak<HeapObject>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=263&c=3
  inline int32_t file_descriptor() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=263&c=3
  inline void set_file_descriptor(int32_t value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=265&c=3
  inline uint32_t offset() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=265&c=3
  inline void set_offset(uint32_t value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=266&c=3
  inline uint32_t size() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=266&c=3
  inline void set_size(uint32_t value);

  DECL_PRINTER(WasmMemoryMapDescriptor)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1
  V8_EXPORT_PRIVATE void WasmMemoryMapDescriptorVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
  static constexpr int kMemoryOffset = P::kHeaderSize;
  static constexpr int kMemoryOffsetEnd = kMemoryOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kMemoryOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=263&c=3
  static constexpr int kFileDescriptorOffset = kMemoryOffsetEnd + 1;
  static constexpr int kFileDescriptorOffsetEnd = kFileDescriptorOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=265&c=3
  static constexpr int kOffsetOffset = kFileDescriptorOffsetEnd + 1;
  static constexpr int kOffsetOffsetEnd = kOffsetOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=266&c=3
  static constexpr int kSizeOffset = kOffsetOffsetEnd + 1;
  static constexpr int kSizeOffsetEnd = kSizeOffset + kInt32Size - 1;
  static constexpr int kStartOfStrongFieldsOffset = kSizeOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kSizeOffsetEnd + 1;
  static constexpr int kHeaderSize = kSizeOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmMemoryMapDescriptor() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmMemoryMapDescriptor, DAlias>,
        "class TorqueGeneratedWasmMemoryMapDescriptor should be used as direct base for WasmMemoryMapDescriptor.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmMemoryMapDescriptor(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmGlobalObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1
V8_EXPORT_PRIVATE bool IsWasmGlobalObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1
template <class D, class P>
class TorqueGeneratedWasmGlobalObject : public P {
  static_assert(
      std::is_same_v<WasmGlobalObject, D>,
      "Use this class as direct base for WasmGlobalObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedWasmGlobalObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmGlobalObject<D,P>;

  // Torque type: (class ByteArray | class FixedArray)
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
  inline Tagged<Union<ByteArray, FixedArray>> buffer() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
  inline Tagged<Union<ByteArray, FixedArray>> buffer(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
  inline void set_buffer(Tagged<Union<ByteArray, FixedArray>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=283&c=3
  inline int offset() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=283&c=3
  inline void set_offset(int value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=284&c=3
  inline int raw_type() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=284&c=3
  inline void set_raw_type(int value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=287&c=3
  inline int is_mutable() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=287&c=3
  inline void set_is_mutable(int value);

  DECL_PRINTER(WasmGlobalObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1
  V8_EXPORT_PRIVATE void WasmGlobalObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=274&c=3
  static constexpr int kTrustedDataOffset = P::kHeaderSize;
  static constexpr int kTrustedDataOffsetEnd = kTrustedDataOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfStrongFieldsOffset = kTrustedDataOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
  static constexpr int kBufferOffset = kTrustedDataOffsetEnd + 1;
  static constexpr int kBufferOffsetEnd = kBufferOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=283&c=3
  static constexpr int kOffsetOffset = kBufferOffsetEnd + 1;
  static constexpr int kOffsetOffsetEnd = kOffsetOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=284&c=3
  static constexpr int kRawTypeOffset = kOffsetOffsetEnd + 1;
  static constexpr int kRawTypeOffsetEnd = kRawTypeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=287&c=3
  static constexpr int kIsMutableOffset = kRawTypeOffsetEnd + 1;
  static constexpr int kIsMutableOffsetEnd = kIsMutableOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kIsMutableOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kIsMutableOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kIsMutableOffsetEnd + 1;
  static constexpr int kHeaderSize = kIsMutableOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmGlobalObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmGlobalObject, DAlias>,
        "class TorqueGeneratedWasmGlobalObject should be used as direct base for WasmGlobalObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmGlobalObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmTagObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1
V8_EXPORT_PRIVATE bool IsWasmTagObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1
template <class D, class P>
class TorqueGeneratedWasmTagObject : public P {
  static_assert(
      std::is_same_v<WasmTagObject, D>,
      "Use this class as direct base for WasmTagObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedWasmTagObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmTagObject<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
  inline Tagged<HeapObject> tag() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
  inline Tagged<HeapObject> tag(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
  inline void set_tag(Tagged<HeapObject> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=292&c=3
  inline int canonical_type_index() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=292&c=3
  inline void set_canonical_type_index(int value);

  DECL_PRINTER(WasmTagObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1
  V8_EXPORT_PRIVATE void WasmTagObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
  static constexpr int kTagOffset = P::kHeaderSize;
  static constexpr int kTagOffsetEnd = kTagOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=292&c=3
  static constexpr int kCanonicalTypeIndexOffset = kTagOffsetEnd + 1;
  static constexpr int kCanonicalTypeIndexOffsetEnd = kCanonicalTypeIndexOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCanonicalTypeIndexOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=293&c=3
  static constexpr int kTrustedDataOffset = kCanonicalTypeIndexOffsetEnd + 1;
  static constexpr int kTrustedDataOffsetEnd = kTrustedDataOffset + kTrustedPointerSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kTrustedDataOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kTrustedDataOffsetEnd + 1;
  static constexpr int kHeaderSize = kTrustedDataOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmTagObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmTagObject, DAlias>,
        "class TorqueGeneratedWasmTagObject should be used as direct base for WasmTagObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmTagObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsAsmWasmData() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1
V8_EXPORT_PRIVATE bool IsAsmWasmData_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1
template <class D, class P>
class TorqueGeneratedAsmWasmData : public P {
  static_assert(
      std::is_same_v<AsmWasmData, D>,
      "Use this class as direct base for AsmWasmData.");
  static_assert(
      std::is_same_v<Struct, P>,
      "Pass in Struct as second template parameter for TorqueGeneratedAsmWasmData.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedAsmWasmData<D,P>;

  // Torque type: ManagedWasmNativeModule
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
  inline Tagged<Managed<wasm::NativeModule>> managed_native_module() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
  inline Tagged<Managed<wasm::NativeModule>> managed_native_module(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
  inline void set_managed_native_module(Tagged<Managed<wasm::NativeModule>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
  inline Tagged<HeapNumber> uses_bitset() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
  inline Tagged<HeapNumber> uses_bitset(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
  inline void set_uses_bitset(Tagged<HeapNumber> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(AsmWasmData)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1
  V8_EXPORT_PRIVATE void AsmWasmDataVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
  static constexpr int kManagedNativeModuleOffset = P::kHeaderSize;
  static constexpr int kManagedNativeModuleOffsetEnd = kManagedNativeModuleOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
  static constexpr int kUsesBitsetOffset = kManagedNativeModuleOffsetEnd + 1;
  static constexpr int kUsesBitsetOffsetEnd = kUsesBitsetOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kUsesBitsetOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kUsesBitsetOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kUsesBitsetOffsetEnd + 1;
  static constexpr int kHeaderSize = kUsesBitsetOffsetEnd + 1;
  static constexpr int kSize = kUsesBitsetOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedAsmWasmData() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedAsmWasmData, DAlias>,
        "class TorqueGeneratedAsmWasmData should be used as direct base for AsmWasmData.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedAsmWasmData(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmTypeInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1
V8_EXPORT_PRIVATE bool IsWasmTypeInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1
template <class D, class P>
class TorqueGeneratedWasmTypeInfo : public P {
  static_assert(
      std::is_same_v<WasmTypeInfo, D>,
      "Use this class as direct base for WasmTypeInfo.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedWasmTypeInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmTypeInfo<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=304&c=3
  inline uint32_t canonical_type() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=304&c=3
  inline void set_canonical_type(uint32_t value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=305&c=3
  inline uint32_t canonical_element_type() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=305&c=3
  inline void set_canonical_element_type(uint32_t value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=306&c=9
  inline int supertypes_length() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=306&c=9
  inline void set_supertypes_length(int value);

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
  inline Tagged<Object> supertypes(int i) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
  inline Tagged<Object> supertypes(PtrComprCageBase cage_base, int i) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
  inline void set_supertypes(int i, Tagged<Object> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(WasmTypeInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1
  V8_EXPORT_PRIVATE void WasmTypeInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=304&c=3
  static constexpr int kCanonicalTypeOffset = P::kHeaderSize;
  static constexpr int kCanonicalTypeOffsetEnd = kCanonicalTypeOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=305&c=3
  static constexpr int kCanonicalElementTypeOffset = kCanonicalTypeOffsetEnd + 1;
  static constexpr int kCanonicalElementTypeOffsetEnd = kCanonicalElementTypeOffset + kInt32Size - 1;
  static constexpr int kStartOfStrongFieldsOffset = kCanonicalElementTypeOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=306&c=9
  static constexpr int kSupertypesLengthOffset = kCanonicalElementTypeOffsetEnd + 1;
  static constexpr int kSupertypesLengthOffsetEnd = kSupertypesLengthOffset + kTaggedSize - 1;
  static constexpr int kHeaderSize = kSupertypesLengthOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
  static constexpr int kSupertypesOffset = kSupertypesLengthOffsetEnd + 1;
  static constexpr int kSupertypesOffsetEnd = kSupertypesOffset + 0 - 1;
  static constexpr int kEndOfStrongFieldsOffset = kSupertypesOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kSupertypesOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kSupertypesOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1
  V8_INLINE static constexpr int32_t SizeFor(int supertypes_length) {
    int32_t size = kHeaderSize;
    size += supertypes_length * 4;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor(this->supertypes_length());
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmTypeInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmTypeInfo, DAlias>,
        "class TorqueGeneratedWasmTypeInfo should be used as direct base for WasmTypeInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmTypeInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=311&c=1
V8_EXPORT_PRIVATE bool IsWasmObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=311&c=1
template <class D, class P>
class TorqueGeneratedWasmObject : public P {
  static_assert(
      std::is_same_v<WasmObject, D>,
      "Use this class as direct base for WasmObject.");
  static_assert(
      std::is_same_v<JSReceiver, P>,
      "Pass in JSReceiver as second template parameter for TorqueGeneratedWasmObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmObject<D,P>;

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=311&c=1
  V8_EXPORT_PRIVATE void WasmObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmObject, DAlias>,
        "class TorqueGeneratedWasmObject should be used as direct base for WasmObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmStruct() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1
V8_EXPORT_PRIVATE bool IsWasmStruct_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1
template <class D, class P>
class TorqueGeneratedWasmStruct : public P {
  static_assert(
      std::is_same_v<WasmStruct, D>,
      "Use this class as direct base for WasmStruct.");
  static_assert(
      std::is_same_v<WasmObject, P>,
      "Pass in WasmObject as second template parameter for TorqueGeneratedWasmStruct.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmStruct<D,P>;

  DECL_PRINTER(WasmStruct)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1
  V8_EXPORT_PRIVATE void WasmStructVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;
  static constexpr int kSize = P::kHeaderSize;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 8);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmStruct() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmStruct, DAlias>,
        "class TorqueGeneratedWasmStruct should be used as direct base for WasmStruct.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmStruct(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmArray() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1
V8_EXPORT_PRIVATE bool IsWasmArray_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1
template <class D, class P>
class TorqueGeneratedWasmArray : public P {
  static_assert(
      std::is_same_v<WasmArray, D>,
      "Use this class as direct base for WasmArray.");
  static_assert(
      std::is_same_v<WasmObject, P>,
      "Pass in WasmObject as second template parameter for TorqueGeneratedWasmArray.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmArray<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=319&c=3
  inline uint32_t length() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=319&c=3
  inline void set_length(uint32_t value);

  DECL_PRINTER(WasmArray)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1
  V8_EXPORT_PRIVATE void WasmArrayVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=319&c=3
  static constexpr int kLengthOffset = P::kHeaderSize;
  static constexpr int kLengthOffsetEnd = kLengthOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=322&c=31
  static constexpr int kOptionalPaddingOffset = kLengthOffsetEnd + 1;
  static constexpr int kOptionalPaddingOffsetEnd = kOptionalPaddingOffset + 0 - 1;
  static constexpr int kStartOfWeakFieldsOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kHeaderSize = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kSize = kOptionalPaddingOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmArray() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmArray, DAlias>,
        "class TorqueGeneratedWasmArray should be used as direct base for WasmArray.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmArray(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmNull() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1
V8_EXPORT_PRIVATE bool IsWasmNull_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1
template <class D, class P>
class TorqueGeneratedWasmNull : public P {
  static_assert(
      std::is_same_v<WasmNull, D>,
      "Use this class as direct base for WasmNull.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedWasmNull.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmNull<D,P>;

  DECL_PRINTER(WasmNull)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1
  V8_EXPORT_PRIVATE void WasmNullVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;
  static constexpr int kSize = P::kHeaderSize;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 4);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmNull() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmNull, DAlias>,
        "class TorqueGeneratedWasmNull should be used as direct base for WasmNull.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmNull(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsWasmSuspendingObject() that avoids inlining.
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1
V8_EXPORT_PRIVATE bool IsWasmSuspendingObject_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1
template <class D, class P>
class TorqueGeneratedWasmSuspendingObject : public P {
  static_assert(
      std::is_same_v<WasmSuspendingObject, D>,
      "Use this class as direct base for WasmSuspendingObject.");
  static_assert(
      std::is_same_v<JSObject, P>,
      "Pass in JSObject as second template parameter for TorqueGeneratedWasmSuspendingObject.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedWasmSuspendingObject<D,P>;

  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
  inline Tagged<JSReceiver> callable() const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
  inline Tagged<JSReceiver> callable(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
  inline void set_callable(Tagged<JSReceiver> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(WasmSuspendingObject)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1
  V8_EXPORT_PRIVATE void WasmSuspendingObjectVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
  static constexpr int kCallableOffset = P::kHeaderSize;
  static constexpr int kCallableOffsetEnd = kCallableOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCallableOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kCallableOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kCallableOffsetEnd + 1;
  static constexpr int kHeaderSize = kCallableOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedWasmSuspendingObject() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedWasmSuspendingObject, DAlias>,
        "class TorqueGeneratedWasmSuspendingObject should be used as direct base for WasmSuspendingObject.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedWasmSuspendingObject(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

