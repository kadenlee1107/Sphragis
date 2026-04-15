class StrongDescriptorArray;
class SortState;
class InternalClass;
class SmiPair;
class SmiBox;
class ExportedSubClassBase;
class ExportedSubClass;
class AbstractInternalClass;
class AbstractInternalClassSubclass1;
class AbstractInternalClassSubclass2;
class InternalClassWithStructElements;
class ExportedSubClass2;
// Alias for IsStrongDescriptorArray() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=44&c=1
V8_EXPORT_PRIVATE bool IsStrongDescriptorArray_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=44&c=1
template <class D, class P>
class TorqueGeneratedStrongDescriptorArray : public P {
  static_assert(
      std::is_same_v<StrongDescriptorArray, D>,
      "Use this class as direct base for StrongDescriptorArray.");
  static_assert(
      std::is_same_v<DescriptorArray, P>,
      "Pass in DescriptorArray as second template parameter for TorqueGeneratedStrongDescriptorArray.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedStrongDescriptorArray<D,P>;

 protected: // not extern or @export
 public:
  DECL_PRINTER(StrongDescriptorArray)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=44&c=1
  V8_EXPORT_PRIVATE void StrongDescriptorArrayVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=44&c=1
  V8_INLINE static constexpr int32_t SizeFor(int number_of_all_descriptors) {
    int32_t size = kHeaderSize;
    size += number_of_all_descriptors * 12;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/descriptor-array.tq?l=44&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor(this->number_of_all_descriptors());
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedStrongDescriptorArray() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedStrongDescriptorArray, DAlias>,
        "class TorqueGeneratedStrongDescriptorArray should be used as direct base for StrongDescriptorArray.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedStrongDescriptorArray(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class StrongDescriptorArray : public TorqueGeneratedStrongDescriptorArray<StrongDescriptorArray, DescriptorArray> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(StrongDescriptorArray)
};

// Alias for IsSortState() that avoids inlining.
// https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=19&c=1
V8_EXPORT_PRIVATE bool IsSortState_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=19&c=1
template <class D, class P>
class TorqueGeneratedSortState : public P {
  static_assert(
      std::is_same_v<SortState, D>,
      "Use this class as direct base for SortState.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedSortState.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedSortState<D,P>;

 protected: // not extern or @export
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=59&c=3
  inline Tagged<JSReceiver> receiver() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=59&c=3
  inline Tagged<JSReceiver> receiver(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=59&c=3
  inline void set_receiver(Tagged<JSReceiver> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=64&c=3
  inline Tagged<Map> initialReceiverMap() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=64&c=3
  inline Tagged<Map> initialReceiverMap(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=64&c=3
  inline void set_initialReceiverMap(Tagged<Map> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: Number
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=65&c=3
  inline Tagged<Number> initialReceiverLength() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=65&c=3
  inline Tagged<Number> initialReceiverLength(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=65&c=3
  inline void set_initialReceiverLength(Tagged<Number> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: (class JSFunction | class Undefined | class JSBoundFunction | class JSWrappedFunction | CallableJSProxy | CallableApiObject)
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=68&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> userCmpFn() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=68&c=3
  inline Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> userCmpFn(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=68&c=3
  inline void set_userCmpFn(Tagged<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=70&c=3
  inline Tagged<Boolean> isResetToGeneric() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=70&c=3
  inline Tagged<Boolean> isResetToGeneric(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=70&c=3
  inline void set_isResetToGeneric(Tagged<Boolean> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=75&c=3
  inline int minGallop() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=75&c=3
  inline void set_minGallop(int value);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=87&c=3
  inline int pendingRunsSize() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=87&c=3
  inline void set_pendingRunsSize(int value);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=88&c=3
  inline Tagged<FixedArray> pendingRuns() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=88&c=3
  inline Tagged<FixedArray> pendingRuns(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=88&c=3
  inline void set_pendingRuns(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=93&c=3
  inline Tagged<FixedArray> workArray() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=93&c=3
  inline Tagged<FixedArray> workArray(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=93&c=3
  inline void set_workArray(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=96&c=3
  inline Tagged<FixedArray> tempArray() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=96&c=3
  inline Tagged<FixedArray> tempArray(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=96&c=3
  inline void set_tempArray(Tagged<FixedArray> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=99&c=3
  inline int sortLength() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=99&c=3
  inline void set_sortLength(int value);

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=103&c=3
  inline int numberOfUndefined() const;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=103&c=3
  inline void set_numberOfUndefined(int value);

 public:
  DECL_PRINTER(SortState)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=19&c=1
  V8_EXPORT_PRIVATE void SortStateVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=59&c=3
  static constexpr int kReceiverOffset = P::kHeaderSize;
  static constexpr int kReceiverOffsetEnd = kReceiverOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=64&c=3
  static constexpr int kInitialReceiverMapOffset = kReceiverOffsetEnd + 1;
  static constexpr int kInitialReceiverMapOffsetEnd = kInitialReceiverMapOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=65&c=3
  static constexpr int kInitialReceiverLengthOffset = kInitialReceiverMapOffsetEnd + 1;
  static constexpr int kInitialReceiverLengthOffsetEnd = kInitialReceiverLengthOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=68&c=3
  static constexpr int kUserCmpFnOffset = kInitialReceiverLengthOffsetEnd + 1;
  static constexpr int kUserCmpFnOffsetEnd = kUserCmpFnOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=70&c=3
  static constexpr int kIsResetToGenericOffset = kUserCmpFnOffsetEnd + 1;
  static constexpr int kIsResetToGenericOffsetEnd = kIsResetToGenericOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=75&c=3
  static constexpr int kMinGallopOffset = kIsResetToGenericOffsetEnd + 1;
  static constexpr int kMinGallopOffsetEnd = kMinGallopOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=87&c=3
  static constexpr int kPendingRunsSizeOffset = kMinGallopOffsetEnd + 1;
  static constexpr int kPendingRunsSizeOffsetEnd = kPendingRunsSizeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=88&c=3
  static constexpr int kPendingRunsOffset = kPendingRunsSizeOffsetEnd + 1;
  static constexpr int kPendingRunsOffsetEnd = kPendingRunsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=93&c=3
  static constexpr int kWorkArrayOffset = kPendingRunsOffsetEnd + 1;
  static constexpr int kWorkArrayOffsetEnd = kWorkArrayOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=96&c=3
  static constexpr int kTempArrayOffset = kWorkArrayOffsetEnd + 1;
  static constexpr int kTempArrayOffsetEnd = kTempArrayOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=99&c=3
  static constexpr int kSortLengthOffset = kTempArrayOffsetEnd + 1;
  static constexpr int kSortLengthOffsetEnd = kSortLengthOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=103&c=3
  static constexpr int kNumberOfUndefinedOffset = kSortLengthOffsetEnd + 1;
  static constexpr int kNumberOfUndefinedOffsetEnd = kNumberOfUndefinedOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kNumberOfUndefinedOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kNumberOfUndefinedOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kNumberOfUndefinedOffsetEnd + 1;
  static constexpr int kHeaderSize = kNumberOfUndefinedOffsetEnd + 1;
  static constexpr int kSize = kNumberOfUndefinedOffsetEnd + 1;

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=19&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 52);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/third_party/v8/builtins/array-sort.tq?l=19&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedSortState() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedSortState, DAlias>,
        "class TorqueGeneratedSortState should be used as direct base for SortState.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedSortState(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class SortState : public TorqueGeneratedSortState<SortState, HeapObject> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(SortState)
};

// Alias for IsInternalClass() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=763&c=1
V8_EXPORT_PRIVATE bool IsInternalClass_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=763&c=1
template <class D, class P>
class TorqueGeneratedInternalClass : public P {
  static_assert(
      std::is_same_v<InternalClass, D>,
      "Use this class as direct base for InternalClass.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedInternalClass.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedInternalClass<D,P>;

 protected: // not extern or @export
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=769&c=3
  inline int a() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=769&c=3
  inline void set_a(int value);

  // Torque type: Number
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=770&c=3
  inline Tagged<Number> b() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=770&c=3
  inline Tagged<Number> b(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=770&c=3
  inline void set_b(Tagged<Number> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

 public:
  DECL_PRINTER(InternalClass)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=763&c=1
  V8_EXPORT_PRIVATE void InternalClassVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=769&c=3
  static constexpr int kAOffset = P::kHeaderSize;
  static constexpr int kAOffsetEnd = kAOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=770&c=3
  static constexpr int kBOffset = kAOffsetEnd + 1;
  static constexpr int kBOffsetEnd = kBOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kHeaderSize = kBOffsetEnd + 1;
  static constexpr int kSize = kBOffsetEnd + 1;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=763&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=763&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedInternalClass() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedInternalClass, DAlias>,
        "class TorqueGeneratedInternalClass should be used as direct base for InternalClass.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedInternalClass(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class InternalClass : public TorqueGeneratedInternalClass<InternalClass, HeapObject> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(InternalClass)
};

// Alias for IsSmiPair() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=829&c=1
V8_EXPORT_PRIVATE bool IsSmiPair_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=829&c=1
template <class D, class P>
class TorqueGeneratedSmiPair : public P {
  static_assert(
      std::is_same_v<SmiPair, D>,
      "Use this class as direct base for SmiPair.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedSmiPair.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedSmiPair<D,P>;

 protected: // not extern or @export
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=833&c=3
  inline int a() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=833&c=3
  inline void set_a(int value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=834&c=3
  inline int b() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=834&c=3
  inline void set_b(int value);

 public:
  DECL_PRINTER(SmiPair)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=829&c=1
  V8_EXPORT_PRIVATE void SmiPairVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=833&c=3
  static constexpr int kAOffset = P::kHeaderSize;
  static constexpr int kAOffsetEnd = kAOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=834&c=3
  static constexpr int kBOffset = kAOffsetEnd + 1;
  static constexpr int kBOffsetEnd = kBOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kHeaderSize = kBOffsetEnd + 1;
  static constexpr int kSize = kBOffsetEnd + 1;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=829&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=829&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedSmiPair() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedSmiPair, DAlias>,
        "class TorqueGeneratedSmiPair should be used as direct base for SmiPair.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedSmiPair(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class SmiPair : public TorqueGeneratedSmiPair<SmiPair, HeapObject> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(SmiPair)
};

// Alias for IsSmiBox() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=930&c=1
V8_EXPORT_PRIVATE bool IsSmiBox_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=930&c=1
template <class D, class P>
class TorqueGeneratedSmiBox : public P {
  static_assert(
      std::is_same_v<SmiBox, D>,
      "Use this class as direct base for SmiBox.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedSmiBox.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedSmiBox<D,P>;

 protected: // not extern or @export
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=931&c=3
  inline int value() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=931&c=3
  inline void set_value(int value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=932&c=3
  inline int unrelated() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=932&c=3
  inline void set_unrelated(int value);

 public:
  DECL_PRINTER(SmiBox)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=930&c=1
  V8_EXPORT_PRIVATE void SmiBoxVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=931&c=3
  static constexpr int kValueOffset = P::kHeaderSize;
  static constexpr int kValueOffsetEnd = kValueOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=932&c=3
  static constexpr int kUnrelatedOffset = kValueOffsetEnd + 1;
  static constexpr int kUnrelatedOffsetEnd = kUnrelatedOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kUnrelatedOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kUnrelatedOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kUnrelatedOffsetEnd + 1;
  static constexpr int kHeaderSize = kUnrelatedOffsetEnd + 1;
  static constexpr int kSize = kUnrelatedOffsetEnd + 1;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=930&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=930&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedSmiBox() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedSmiBox, DAlias>,
        "class TorqueGeneratedSmiBox should be used as direct base for SmiBox.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedSmiBox(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class SmiBox : public TorqueGeneratedSmiBox<SmiBox, HeapObject> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(SmiBox)
};

// Alias for IsExportedSubClassBase() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1181&c=1
V8_EXPORT_PRIVATE bool IsExportedSubClassBase_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1181&c=1
template <class D, class P>
class TorqueGeneratedExportedSubClassBase : public P {
  static_assert(
      std::is_same_v<ExportedSubClassBase, D>,
      "Use this class as direct base for ExportedSubClassBase.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedExportedSubClassBase.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedExportedSubClassBase<D,P>;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1183&c=3
  inline Tagged<HeapObject> a() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1183&c=3
  inline Tagged<HeapObject> a(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1183&c=3
  inline void set_a(Tagged<HeapObject> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1184&c=3
  inline Tagged<HeapObject> b() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1184&c=3
  inline Tagged<HeapObject> b(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1184&c=3
  inline void set_b(Tagged<HeapObject> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(ExportedSubClassBase)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1181&c=1
  V8_EXPORT_PRIVATE void ExportedSubClassBaseVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1183&c=3
  static constexpr int kAOffset = P::kHeaderSize;
  static constexpr int kAOffsetEnd = kAOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1184&c=3
  static constexpr int kBOffset = kAOffsetEnd + 1;
  static constexpr int kBOffsetEnd = kBOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kBOffsetEnd + 1;
  static constexpr int kHeaderSize = kBOffsetEnd + 1;
  static constexpr int kSize = kBOffsetEnd + 1;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1181&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1181&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedExportedSubClassBase() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedExportedSubClassBase, DAlias>,
        "class TorqueGeneratedExportedSubClassBase should be used as direct base for ExportedSubClassBase.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedExportedSubClassBase(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class ExportedSubClassBase : public TorqueGeneratedExportedSubClassBase<ExportedSubClassBase, HeapObject> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(ExportedSubClassBase)
};

// Alias for IsExportedSubClass() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1174&c=1
V8_EXPORT_PRIVATE bool IsExportedSubClass_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1174&c=1
template <class D, class P>
class TorqueGeneratedExportedSubClass : public P {
  static_assert(
      std::is_same_v<ExportedSubClass, D>,
      "Use this class as direct base for ExportedSubClass.");
  static_assert(
      std::is_same_v<ExportedSubClassBase, P>,
      "Pass in ExportedSubClassBase as second template parameter for TorqueGeneratedExportedSubClass.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedExportedSubClass<D,P>;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1176&c=3
  inline int32_t c_field() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1176&c=3
  inline void set_c_field(int32_t value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1177&c=3
  inline int32_t d_field() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1177&c=3
  inline void set_d_field(int32_t value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1178&c=3
  inline int e_field() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1178&c=3
  inline void set_e_field(int value);

  DECL_PRINTER(ExportedSubClass)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1174&c=1
  V8_EXPORT_PRIVATE void ExportedSubClassVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1176&c=3
  static constexpr int kCFieldOffset = P::kHeaderSize;
  static constexpr int kCFieldOffsetEnd = kCFieldOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1177&c=3
  static constexpr int kDFieldOffset = kCFieldOffsetEnd + 1;
  static constexpr int kDFieldOffsetEnd = kDFieldOffset + kInt32Size - 1;
  static constexpr int kStartOfStrongFieldsOffset = kDFieldOffsetEnd + 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1178&c=3
  static constexpr int kEFieldOffset = kDFieldOffsetEnd + 1;
  static constexpr int kEFieldOffsetEnd = kEFieldOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kEFieldOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kEFieldOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kEFieldOffsetEnd + 1;
  static constexpr int kHeaderSize = kEFieldOffsetEnd + 1;
  static constexpr int kSize = kEFieldOffsetEnd + 1;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1174&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 24);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1174&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedExportedSubClass() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedExportedSubClass, DAlias>,
        "class TorqueGeneratedExportedSubClass should be used as direct base for ExportedSubClass.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedExportedSubClass(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class ExportedSubClass : public TorqueGeneratedExportedSubClass<ExportedSubClass, ExportedSubClassBase> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(ExportedSubClass)
};

// Alias for IsAbstractInternalClass() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1187&c=1
V8_EXPORT_PRIVATE bool IsAbstractInternalClass_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1187&c=1
template <class D, class P>
class TorqueGeneratedAbstractInternalClass : public P {
  static_assert(
      std::is_same_v<AbstractInternalClass, D>,
      "Use this class as direct base for AbstractInternalClass.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedAbstractInternalClass.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedAbstractInternalClass<D,P>;

 protected: // not extern or @export
 public:
  DECL_PRINTER(AbstractInternalClass)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1187&c=1
  V8_EXPORT_PRIVATE void AbstractInternalClassVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedAbstractInternalClass() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedAbstractInternalClass, DAlias>,
        "class TorqueGeneratedAbstractInternalClass should be used as direct base for AbstractInternalClass.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedAbstractInternalClass(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class AbstractInternalClass : public TorqueGeneratedAbstractInternalClass<AbstractInternalClass, HeapObject> {
 public:
  TQ_OBJECT_CONSTRUCTORS(AbstractInternalClass)
};

// Alias for IsAbstractInternalClassSubclass1() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1190&c=1
V8_EXPORT_PRIVATE bool IsAbstractInternalClassSubclass1_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1190&c=1
template <class D, class P>
class TorqueGeneratedAbstractInternalClassSubclass1 : public P {
  static_assert(
      std::is_same_v<AbstractInternalClassSubclass1, D>,
      "Use this class as direct base for AbstractInternalClassSubclass1.");
  static_assert(
      std::is_same_v<AbstractInternalClass, P>,
      "Pass in AbstractInternalClass as second template parameter for TorqueGeneratedAbstractInternalClassSubclass1.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedAbstractInternalClassSubclass1<D,P>;

 protected: // not extern or @export
 public:
  DECL_PRINTER(AbstractInternalClassSubclass1)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1190&c=1
  V8_EXPORT_PRIVATE void AbstractInternalClassSubclass1Verify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;
  static constexpr int kSize = P::kHeaderSize;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1190&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 4);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1190&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedAbstractInternalClassSubclass1() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedAbstractInternalClassSubclass1, DAlias>,
        "class TorqueGeneratedAbstractInternalClassSubclass1 should be used as direct base for AbstractInternalClassSubclass1.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedAbstractInternalClassSubclass1(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class AbstractInternalClassSubclass1 : public TorqueGeneratedAbstractInternalClassSubclass1<AbstractInternalClassSubclass1, AbstractInternalClass> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(AbstractInternalClassSubclass1)
};

// Alias for IsAbstractInternalClassSubclass2() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1192&c=1
V8_EXPORT_PRIVATE bool IsAbstractInternalClassSubclass2_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1192&c=1
template <class D, class P>
class TorqueGeneratedAbstractInternalClassSubclass2 : public P {
  static_assert(
      std::is_same_v<AbstractInternalClassSubclass2, D>,
      "Use this class as direct base for AbstractInternalClassSubclass2.");
  static_assert(
      std::is_same_v<AbstractInternalClass, P>,
      "Pass in AbstractInternalClass as second template parameter for TorqueGeneratedAbstractInternalClassSubclass2.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedAbstractInternalClassSubclass2<D,P>;

 protected: // not extern or @export
 public:
  DECL_PRINTER(AbstractInternalClassSubclass2)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1192&c=1
  V8_EXPORT_PRIVATE void AbstractInternalClassSubclass2Verify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfWeakFieldsOffset = P::kHeaderSize;
  static constexpr int kStartOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kEndOfStrongFieldsOffset = P::kHeaderSize;
  static constexpr int kHeaderSize = P::kHeaderSize;
  static constexpr int kSize = P::kHeaderSize;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1192&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 4);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1192&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedAbstractInternalClassSubclass2() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedAbstractInternalClassSubclass2, DAlias>,
        "class TorqueGeneratedAbstractInternalClassSubclass2 should be used as direct base for AbstractInternalClassSubclass2.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedAbstractInternalClassSubclass2(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class AbstractInternalClassSubclass2 : public TorqueGeneratedAbstractInternalClassSubclass2<AbstractInternalClassSubclass2, AbstractInternalClass> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(AbstractInternalClassSubclass2)
};

// Alias for IsInternalClassWithStructElements() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1199&c=1
V8_EXPORT_PRIVATE bool IsInternalClassWithStructElements_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1199&c=1
template <class D, class P>
class TorqueGeneratedInternalClassWithStructElements : public P {
  static_assert(
      std::is_same_v<InternalClassWithStructElements, D>,
      "Use this class as direct base for InternalClassWithStructElements.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedInternalClassWithStructElements.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedInternalClassWithStructElements<D,P>;

 protected: // not extern or @export
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1200&c=3
  inline int32_t dummy1() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1200&c=3
  inline void set_dummy1(int32_t value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1201&c=3
  inline int32_t dummy2() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1201&c=3
  inline void set_dummy2(int32_t value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1202&c=9
  inline int count() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1202&c=9
  inline void set_count(int value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1203&c=3
  inline int data() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1203&c=3
  inline void set_data(int value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1204&c=3
  inline Tagged<Object> object() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1204&c=3
  inline Tagged<Object> object(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1204&c=3
  inline void set_object(Tagged<Object> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1205&c=3
  inline int entries(int i) const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1205&c=3
  inline void set_entries(int i, int value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
  inline int more_entries_a(int i) const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
  inline void set_more_entries_a(int i, int value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
  inline int more_entries_b(int i) const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
  inline void set_more_entries_b(int i, int value);

 public:
  DECL_PRINTER(InternalClassWithStructElements)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1199&c=1
  V8_EXPORT_PRIVATE void InternalClassWithStructElementsVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1200&c=3
  static constexpr int kDummy1Offset = P::kHeaderSize;
  static constexpr int kDummy1OffsetEnd = kDummy1Offset + kInt32Size - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1201&c=3
  static constexpr int kDummy2Offset = kDummy1OffsetEnd + 1;
  static constexpr int kDummy2OffsetEnd = kDummy2Offset + kInt32Size - 1;
  static constexpr int kStartOfStrongFieldsOffset = kDummy2OffsetEnd + 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1202&c=9
  static constexpr int kCountOffset = kDummy2OffsetEnd + 1;
  static constexpr int kCountOffsetEnd = kCountOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1203&c=3
  static constexpr int kDataOffset = kCountOffsetEnd + 1;
  static constexpr int kDataOffsetEnd = kDataOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1204&c=3
  static constexpr int kObjectOffset = kDataOffsetEnd + 1;
  static constexpr int kObjectOffsetEnd = kObjectOffset + kTaggedSize - 1;
  static constexpr int kHeaderSize = kObjectOffsetEnd + 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1205&c=3
  static constexpr int kEntriesOffset = kObjectOffsetEnd + 1;
  static constexpr int kEntriesOffsetEnd = kEntriesOffset + 0 - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1206&c=3
  inline int MoreEntriesOffset() const;
  static constexpr int kEndOfStrongFieldsOffset = kEntriesOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kEntriesOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kEntriesOffsetEnd + 1;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1199&c=1
  V8_INLINE static constexpr int32_t SizeFor(int count) {
    int32_t size = kHeaderSize;
    size += count * 4;
    size += count * 8;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1199&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor(this->count());
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedInternalClassWithStructElements() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedInternalClassWithStructElements, DAlias>,
        "class TorqueGeneratedInternalClassWithStructElements should be used as direct base for InternalClassWithStructElements.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedInternalClassWithStructElements(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class InternalClassWithStructElements : public TorqueGeneratedInternalClassWithStructElements<InternalClassWithStructElements, HeapObject> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(InternalClassWithStructElements)
};

// Alias for IsExportedSubClass2() that avoids inlining.
// https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1216&c=1
V8_EXPORT_PRIVATE bool IsExportedSubClass2_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1216&c=1
template <class D, class P>
class TorqueGeneratedExportedSubClass2 : public P {
  static_assert(
      std::is_same_v<ExportedSubClass2, D>,
      "Use this class as direct base for ExportedSubClass2.");
  static_assert(
      std::is_same_v<ExportedSubClassBase, P>,
      "Pass in ExportedSubClassBase as second template parameter for TorqueGeneratedExportedSubClass2.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedExportedSubClass2<D,P>;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1218&c=3
  inline int32_t x_field() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1218&c=3
  inline void set_x_field(int32_t value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1219&c=3
  inline int32_t y_field() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1219&c=3
  inline void set_y_field(int32_t value);

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1220&c=3
  inline int z_field() const;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1220&c=3
  inline void set_z_field(int value);

  DECL_PRINTER(ExportedSubClass2)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1216&c=1
  V8_EXPORT_PRIVATE void ExportedSubClass2Verify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1218&c=3
  static constexpr int kXFieldOffset = P::kHeaderSize;
  static constexpr int kXFieldOffsetEnd = kXFieldOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1219&c=3
  static constexpr int kYFieldOffset = kXFieldOffsetEnd + 1;
  static constexpr int kYFieldOffsetEnd = kYFieldOffset + kInt32Size - 1;
  static constexpr int kStartOfStrongFieldsOffset = kYFieldOffsetEnd + 1;
  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1220&c=3
  static constexpr int kZFieldOffset = kYFieldOffsetEnd + 1;
  static constexpr int kZFieldOffsetEnd = kZFieldOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kZFieldOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kZFieldOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kZFieldOffsetEnd + 1;
  static constexpr int kHeaderSize = kZFieldOffsetEnd + 1;
  static constexpr int kSize = kZFieldOffsetEnd + 1;

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1216&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 24);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/test/torque/test-torque.tq?l=1216&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedExportedSubClass2() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedExportedSubClass2, DAlias>,
        "class TorqueGeneratedExportedSubClass2 should be used as direct base for ExportedSubClass2.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedExportedSubClass2(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

class ExportedSubClass2 : public TorqueGeneratedExportedSubClass2<ExportedSubClass2, ExportedSubClassBase> {
 public:
  class BodyDescriptor;
  TQ_OBJECT_CONSTRUCTORS(ExportedSubClass2)
};

