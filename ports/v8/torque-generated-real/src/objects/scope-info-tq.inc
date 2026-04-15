class ScopeInfo;
// Alias for IsScopeInfo() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/scope-info.tq?l=122&c=1
V8_EXPORT_PRIVATE bool IsScopeInfo_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/scope-info.tq?l=122&c=1
template <class D, class P>
class TorqueGeneratedScopeInfo : public P {
  static_assert(
      std::is_same_v<ScopeInfo, D>,
      "Use this class as direct base for ScopeInfo.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedScopeInfo.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedScopeInfo<D,P>;

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=124&c=42
  inline uint32_t flags(RelaxedLoadTag) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=124&c=42
  inline void set_flags(uint32_t value, RelaxedStoreTag);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=130&c=3
  inline int parameter_count() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=130&c=3
  inline void set_parameter_count(int value);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=134&c=9
  inline int context_local_count() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=134&c=9
  inline void set_context_local_count(int value);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=137&c=3
  inline int position_info_start() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=137&c=3
  inline void set_position_info_start(int value);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=137&c=3
  inline int position_info_end() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=137&c=3
  inline void set_position_info_end(int value);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=142&c=9
  inline int module_variable_count() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=142&c=9
  inline void set_module_variable_count(int value);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=148&c=3
  inline Tagged<String> context_local_names(int i) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=148&c=3
  inline Tagged<String> context_local_names(PtrComprCageBase cage_base, int i) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=148&c=3
  inline void set_context_local_names(int i, Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=153&c=3
  inline Tagged<NameToIndexHashTable> context_local_names_hashtable() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=153&c=3
  inline Tagged<NameToIndexHashTable> context_local_names_hashtable(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=153&c=3
  inline void set_context_local_names_hashtable(Tagged<NameToIndexHashTable> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=159&c=3
  inline int context_local_infos(int i) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=159&c=3
  inline void set_context_local_infos(int i, int value);

  // Torque type: (class Name | Smi)
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=164&c=3
  inline Tagged<Union<Name, Smi>> saved_class_variable_info() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=164&c=3
  inline Tagged<Union<Name, Smi>> saved_class_variable_info(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=164&c=3
  inline void set_saved_class_variable_info(Tagged<Union<Name, Smi>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class String | Zero)
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=170&c=3
  inline Tagged<Union<Smi, String>> function_variable_info_name() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=170&c=3
  inline Tagged<Union<Smi, String>> function_variable_info_name(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=170&c=3
  inline void set_function_variable_info_name(Tagged<Union<Smi, String>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=170&c=3
  inline int function_variable_info_context_or_stack_slot_index() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=170&c=3
  inline void set_function_variable_info_context_or_stack_slot_index(int value);

  // Torque type: (class Undefined | class String)
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=175&c=3
  inline Tagged<Union<String, Undefined>> inferred_function_name() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=175&c=3
  inline Tagged<Union<String, Undefined>> inferred_function_name(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=175&c=3
  inline void set_inferred_function_name(Tagged<Union<String, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=177&c=3
  inline Tagged<ScopeInfo> outer_scope_info() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=177&c=3
  inline Tagged<ScopeInfo> outer_scope_info(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=177&c=3
  inline void set_outer_scope_info(Tagged<ScopeInfo> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: SourceTextModuleInfo
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=181&c=3
  inline Tagged<FixedArray> module_info() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=181&c=3
  inline Tagged<FixedArray> module_info(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=181&c=3
  inline void set_module_info(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
  inline Tagged<String> module_variables_name(int i) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
  inline Tagged<String> module_variables_name(PtrComprCageBase cage_base, int i) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
  inline void set_module_variables_name(int i, Tagged<String> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
  inline int module_variables_index(int i) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
  inline void set_module_variables_index(int i, int value);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
  inline int module_variables_properties(int i) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
  inline void set_module_variables_properties(int i, int value);

  // Torque type: DependentCode
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=187&c=3
  inline Tagged<WeakArrayList> dependent_code() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=187&c=3
  inline Tagged<WeakArrayList> dependent_code(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=187&c=3
  inline void set_dependent_code(Tagged<WeakArrayList> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=191&c=9
  inline int unused_parameter_bits() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=191&c=9
  inline void set_unused_parameter_bits(int value);

  DECL_PRINTER(ScopeInfo)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=122&c=1
  V8_EXPORT_PRIVATE void ScopeInfoVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=124&c=42
  static constexpr int kFlagsOffset = P::kHeaderSize;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=127&c=31
  static constexpr int kOptionalPaddingOffset = kFlagsOffsetEnd + 1;
  static constexpr int kOptionalPaddingOffsetEnd = kOptionalPaddingOffset + 0 - 1;
  static constexpr int kStartOfStrongFieldsOffset = kOptionalPaddingOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=130&c=3
  static constexpr int kParameterCountOffset = kOptionalPaddingOffsetEnd + 1;
  static constexpr int kParameterCountOffsetEnd = kParameterCountOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=134&c=9
  static constexpr int kContextLocalCountOffset = kParameterCountOffsetEnd + 1;
  static constexpr int kContextLocalCountOffsetEnd = kContextLocalCountOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=137&c=3
  static constexpr int kPositionInfoOffset = kContextLocalCountOffsetEnd + 1;
  static constexpr int kPositionInfoOffsetEnd = kPositionInfoOffset + 8 - 1;
  static constexpr int kHeaderSize = kPositionInfoOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=142&c=9
  static constexpr int kModuleVariableCountOffset = kPositionInfoOffsetEnd + 1;
  static constexpr int kModuleVariableCountOffsetEnd = kModuleVariableCountOffset + 0 - 1;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=148&c=3
  inline int ContextLocalNamesOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=153&c=3
  inline int ContextLocalNamesHashtableOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=159&c=3
  inline int ContextLocalInfosOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=164&c=3
  inline int SavedClassVariableInfoOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=170&c=3
  inline int FunctionVariableInfoOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=175&c=3
  inline int InferredFunctionNameOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=177&c=3
  inline int OuterScopeInfoOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=181&c=3
  inline int ModuleInfoOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=183&c=3
  inline int ModuleVariablesOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=187&c=3
  inline int DependentCodeOffset() const;
  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=191&c=9
  inline int UnusedParameterBitsOffset() const;
  static constexpr int kEndOfStrongFieldsOffset = kModuleVariableCountOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kModuleVariableCountOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kModuleVariableCountOffsetEnd + 1;

  // SizeFor implementations not generated due to complex array lengths

  // https://crsrc.org/c/v8/src/objects/scope-info.tq?l=122&c=1
  inline int AllocatedSize() const;
  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedScopeInfo() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedScopeInfo, DAlias>,
        "class TorqueGeneratedScopeInfo should be used as direct base for ScopeInfo.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedScopeInfo(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

struct TorqueGeneratedPositionInfoOffsets {
  static constexpr int kStartOffset = 0;
  static constexpr int kEndOffset = 4;
  static constexpr int kSize = 8;
};

struct TorqueGeneratedFunctionVariableInfoOffsets {
  static constexpr int kNameOffset = 0;
  static constexpr int kContextOrStackSlotIndexOffset = 4;
  static constexpr int kSize = 8;
};

struct TorqueGeneratedModuleVariableOffsets {
  static constexpr int kNameOffset = 0;
  static constexpr int kIndexOffset = 4;
  static constexpr int kPropertiesOffset = 8;
  static constexpr int kSize = 12;
};

