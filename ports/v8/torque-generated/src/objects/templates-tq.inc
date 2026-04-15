class TemplateInfo;
class TemplateInfoWithProperties;
class FunctionTemplateInfo;
class ObjectTemplateInfo;
class DictionaryTemplateInfo;
// Alias for IsTemplateInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/templates.tq?l=11&c=1
V8_EXPORT_PRIVATE bool IsTemplateInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/templates.tq?l=11&c=1
template <class D, class P>
class TorqueGeneratedTemplateInfo : public P {
  static_assert(
      std::is_same_v<TemplateInfo, D>,
      "Use this class as direct base for TemplateInfo.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedTemplateInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedTemplateInfo<D,P>;

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=13&c=3
  inline int template_info_flags() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=13&c=3
  inline void set_template_info_flags(int value);

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=11&c=1
  V8_EXPORT_PRIVATE void TemplateInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=13&c=3
  static constexpr int kTemplateInfoFlagsOffset = P::kHeaderSize;
  static constexpr int kTemplateInfoFlagsOffsetEnd = kTemplateInfoFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kTemplateInfoFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kTemplateInfoFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kTemplateInfoFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kTemplateInfoFlagsOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedTemplateInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedTemplateInfo, DAlias>,
        "class TorqueGeneratedTemplateInfo should be used as direct base for TemplateInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedTemplateInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsTemplateInfoWithProperties() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/templates.tq?l=16&c=1
V8_EXPORT_PRIVATE bool IsTemplateInfoWithProperties_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/templates.tq?l=16&c=1
template <class D, class P>
class TorqueGeneratedTemplateInfoWithProperties : public P {
  static_assert(
      std::is_same_v<TemplateInfoWithProperties, D>,
      "Use this class as direct base for TemplateInfoWithProperties.");
  static_assert(
      std::is_same_v<TemplateInfo, P>,
      "Pass in TemplateInfo as second template parameter for TorqueGeneratedTemplateInfoWithProperties.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedTemplateInfoWithProperties<D,P>;

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=18&c=3
  inline int number_of_properties() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=18&c=3
  inline void set_number_of_properties(int value);

  // Torque type: (class Undefined | class ArrayList)
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=19&c=3
  inline Tagged<Union<ArrayList, Undefined>> property_list() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=19&c=3
  inline Tagged<Union<ArrayList, Undefined>> property_list(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=19&c=3
  inline void set_property_list(Tagged<Union<ArrayList, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class ArrayList)
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=20&c=3
  inline Tagged<Union<ArrayList, Undefined>> property_accessors() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=20&c=3
  inline Tagged<Union<ArrayList, Undefined>> property_accessors(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=20&c=3
  inline void set_property_accessors(Tagged<Union<ArrayList, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=16&c=1
  V8_EXPORT_PRIVATE void TemplateInfoWithPropertiesVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=18&c=3
  static constexpr int kNumberOfPropertiesOffset = P::kHeaderSize;
  static constexpr int kNumberOfPropertiesOffsetEnd = kNumberOfPropertiesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=19&c=3
  static constexpr int kPropertyListOffset = kNumberOfPropertiesOffsetEnd + 1;
  static constexpr int kPropertyListOffsetEnd = kPropertyListOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=20&c=3
  static constexpr int kPropertyAccessorsOffset = kPropertyListOffsetEnd + 1;
  static constexpr int kPropertyAccessorsOffsetEnd = kPropertyAccessorsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPropertyAccessorsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kPropertyAccessorsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPropertyAccessorsOffsetEnd + 1;
  static constexpr int kHeaderSize = kPropertyAccessorsOffsetEnd + 1;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedTemplateInfoWithProperties() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedTemplateInfoWithProperties, DAlias>,
        "class TorqueGeneratedTemplateInfoWithProperties should be used as direct base for TemplateInfoWithProperties.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedTemplateInfoWithProperties(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsFunctionTemplateInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1
V8_EXPORT_PRIVATE bool IsFunctionTemplateInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1
template <class D, class P>
class TorqueGeneratedFunctionTemplateInfo : public P {
  static_assert(
      std::is_same_v<FunctionTemplateInfo, D>,
      "Use this class as direct base for FunctionTemplateInfo.");
  static_assert(
      std::is_same_v<TemplateInfoWithProperties, P>,
      "Pass in TemplateInfoWithProperties as second template parameter for TorqueGeneratedFunctionTemplateInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedFunctionTemplateInfo<D,P>;

  // Torque type: (class Undefined | class String)
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=57&c=3
  inline Tagged<Union<String, Undefined>> class_name() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=57&c=3
  inline Tagged<Union<String, Undefined>> class_name(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=57&c=3
  inline void set_class_name(Tagged<Union<String, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class String)
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=61&c=3
  inline Tagged<Union<String, Undefined>> interface_name() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=61&c=3
  inline Tagged<Union<String, Undefined>> interface_name(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=61&c=3
  inline void set_interface_name(Tagged<Union<String, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class FunctionTemplateInfo)
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=66&c=3
  inline Tagged<Union<FunctionTemplateInfo, Undefined>> signature() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=66&c=3
  inline Tagged<Union<FunctionTemplateInfo, Undefined>> signature(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=66&c=3
  inline void set_signature(Tagged<Union<FunctionTemplateInfo, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class FunctionTemplateRareData)
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=72&c=3
  inline Tagged<Union<FunctionTemplateRareData, Undefined>> rare_data(AcquireLoadTag) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=72&c=3
  inline Tagged<Union<FunctionTemplateRareData, Undefined>> rare_data(PtrComprCageBase cage_base, AcquireLoadTag) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=72&c=3
  inline void set_rare_data(Tagged<Union<FunctionTemplateRareData, Undefined>> value, ReleaseStoreTag, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class Undefined | class SharedFunctionInfo)
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=73&c=3
  inline Tagged<Union<SharedFunctionInfo, Undefined>> shared_function_info() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=73&c=3
  inline Tagged<Union<SharedFunctionInfo, Undefined>> shared_function_info(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=73&c=3
  inline void set_shared_function_info(Tagged<Union<SharedFunctionInfo, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=77&c=3
  inline Tagged<Object> cached_property_name() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=77&c=3
  inline Tagged<Object> cached_property_name(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=77&c=3
  inline void set_cached_property_name(Tagged<Object> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=84&c=36
  inline Tagged<Object> callback_data(AcquireLoadTag) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=84&c=36
  inline Tagged<Object> callback_data(PtrComprCageBase cage_base, AcquireLoadTag) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=84&c=36
  inline void set_callback_data(Tagged<Object> value, ReleaseStoreTag, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=87&c=3
  inline uint32_t flag() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=87&c=3
  inline void set_flag(uint32_t value);

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=89&c=3
  inline int16_t length() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=89&c=3
  inline void set_length(int16_t value);

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=92&c=3
  inline InstanceType instance_type() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=92&c=3
  inline void set_instance_type(InstanceType value);

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=97&c=3
  inline uint32_t exception_context() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=97&c=3
  inline void set_exception_context(uint32_t value);

  DECL_PRINTER(FunctionTemplateInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1
  V8_EXPORT_PRIVATE void FunctionTemplateInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=57&c=3
  static constexpr int kClassNameOffset = P::kHeaderSize;
  static constexpr int kClassNameOffsetEnd = kClassNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=61&c=3
  static constexpr int kInterfaceNameOffset = kClassNameOffsetEnd + 1;
  static constexpr int kInterfaceNameOffsetEnd = kInterfaceNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=66&c=3
  static constexpr int kSignatureOffset = kInterfaceNameOffsetEnd + 1;
  static constexpr int kSignatureOffsetEnd = kSignatureOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=72&c=3
  static constexpr int kRareDataOffset = kSignatureOffsetEnd + 1;
  static constexpr int kRareDataOffsetEnd = kRareDataOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=73&c=3
  static constexpr int kSharedFunctionInfoOffset = kRareDataOffsetEnd + 1;
  static constexpr int kSharedFunctionInfoOffsetEnd = kSharedFunctionInfoOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=77&c=3
  static constexpr int kCachedPropertyNameOffset = kSharedFunctionInfoOffsetEnd + 1;
  static constexpr int kCachedPropertyNameOffsetEnd = kCachedPropertyNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=84&c=36
  static constexpr int kCallbackDataOffset = kCachedPropertyNameOffsetEnd + 1;
  static constexpr int kCallbackDataOffsetEnd = kCallbackDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCallbackDataOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=87&c=3
  static constexpr int kFlagOffset = kCallbackDataOffsetEnd + 1;
  static constexpr int kFlagOffsetEnd = kFlagOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=89&c=3
  static constexpr int kLengthOffset = kFlagOffsetEnd + 1;
  static constexpr int kLengthOffsetEnd = kLengthOffset + kUInt16Size - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=92&c=3
  static constexpr int kInstanceTypeOffset = kLengthOffsetEnd + 1;
  static constexpr int kInstanceTypeOffsetEnd = kInstanceTypeOffset + kUInt16Size - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=97&c=3
  static constexpr int kExceptionContextOffset = kInstanceTypeOffsetEnd + 1;
  static constexpr int kExceptionContextOffsetEnd = kExceptionContextOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=100&c=31
  static constexpr int kOptionalPaddingOffset = kExceptionContextOffsetEnd + 1;
  static constexpr int kOptionalPaddingOffsetEnd = kOptionalPaddingOffset + 0 - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=106&c=3
  static constexpr int kCallbackOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kCallbackOffsetEnd = kCallbackOffset + kExternalPointerSlotSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kCallbackOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kCallbackOffsetEnd + 1;
  static constexpr int kHeaderSize = kCallbackOffsetEnd + 1;
  static constexpr int kSize = kCallbackOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 68);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedFunctionTemplateInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedFunctionTemplateInfo, DAlias>,
        "class TorqueGeneratedFunctionTemplateInfo should be used as direct base for FunctionTemplateInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedFunctionTemplateInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsObjectTemplateInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1
V8_EXPORT_PRIVATE bool IsObjectTemplateInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1
template <class D, class P>
class TorqueGeneratedObjectTemplateInfo : public P {
  static_assert(
      std::is_same_v<ObjectTemplateInfo, D>,
      "Use this class as direct base for ObjectTemplateInfo.");
  static_assert(
      std::is_same_v<TemplateInfoWithProperties, P>,
      "Pass in TemplateInfoWithProperties as second template parameter for TorqueGeneratedObjectTemplateInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedObjectTemplateInfo<D,P>;

  // Torque type: (class Undefined | class FunctionTemplateInfo)
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=117&c=3
  inline Tagged<Union<FunctionTemplateInfo, Undefined>> constructor() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=117&c=3
  inline Tagged<Union<FunctionTemplateInfo, Undefined>> constructor(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=117&c=3
  inline void set_constructor(Tagged<Union<FunctionTemplateInfo, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=118&c=3
  inline int data() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=118&c=3
  inline void set_data(int value);

  DECL_PRINTER(ObjectTemplateInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1
  V8_EXPORT_PRIVATE void ObjectTemplateInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=117&c=3
  static constexpr int kConstructorOffset = P::kHeaderSize;
  static constexpr int kConstructorOffsetEnd = kConstructorOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=118&c=3
  static constexpr int kDataOffset = kConstructorOffsetEnd + 1;
  static constexpr int kDataOffsetEnd = kDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kHeaderSize = kDataOffsetEnd + 1;
  static constexpr int kSize = kDataOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 28);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedObjectTemplateInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedObjectTemplateInfo, DAlias>,
        "class TorqueGeneratedObjectTemplateInfo should be used as direct base for ObjectTemplateInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedObjectTemplateInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

// Alias for IsDictionaryTemplateInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1
V8_EXPORT_PRIVATE bool IsDictionaryTemplateInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1
template <class D, class P>
class TorqueGeneratedDictionaryTemplateInfo : public P {
  static_assert(
      std::is_same_v<DictionaryTemplateInfo, D>,
      "Use this class as direct base for DictionaryTemplateInfo.");
  static_assert(
      std::is_same_v<TemplateInfo, P>,
      "Pass in TemplateInfo as second template parameter for TorqueGeneratedDictionaryTemplateInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedDictionaryTemplateInfo<D,P>;

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=123&c=3
  inline Tagged<FixedArray> property_names() const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=123&c=3
  inline Tagged<FixedArray> property_names(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=123&c=3
  inline void set_property_names(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(DictionaryTemplateInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1
  V8_EXPORT_PRIVATE void DictionaryTemplateInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=123&c=3
  static constexpr int kPropertyNamesOffset = P::kHeaderSize;
  static constexpr int kPropertyNamesOffsetEnd = kPropertyNamesOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPropertyNamesOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kPropertyNamesOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPropertyNamesOffsetEnd + 1;
  static constexpr int kHeaderSize = kPropertyNamesOffsetEnd + 1;
  static constexpr int kSize = kPropertyNamesOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedDictionaryTemplateInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedDictionaryTemplateInfo, DAlias>,
        "class TorqueGeneratedDictionaryTemplateInfo should be used as direct base for DictionaryTemplateInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedDictionaryTemplateInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

