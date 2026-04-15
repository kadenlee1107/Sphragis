class MegaDomHandler;
// Alias for IsMegaDomHandler() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=5&c=1
V8_EXPORT_PRIVATE bool IsMegaDomHandler_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=5&c=1
template <class D, class P>
class TorqueGeneratedMegaDomHandler : public P {
  static_assert(
      std::is_same_v<MegaDomHandler, D>,
      "Use this class as direct base for MegaDomHandler.");
  static_assert(
      std::is_same_v<HeapObject, P>,
      "Pass in HeapObject as second template parameter for TorqueGeneratedMegaDomHandler.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedMegaDomHandler<D,P>;

  // Torque type: MaybeObject
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=7&c=3
  inline Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> accessor() const;
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=7&c=3
  inline Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> accessor(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=7&c=3
  inline void set_accessor(Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  // Torque type: MaybeObject
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=8&c=3
  inline Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> context() const;
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=8&c=3
  inline Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> context(PtrComprCageBase cage_base) const;
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=8&c=3
  inline void set_context(Tagged<Union<HeapObject, Smi, Weak<HeapObject>>> value, WriteBarrierMode mode = UPDATE_WRITE_BARRIER);

  DECL_PRINTER(MegaDomHandler)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=5&c=1
  V8_EXPORT_PRIVATE void MegaDomHandlerVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  static constexpr int kStartOfWeakFieldsOffset = P::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=7&c=3
  static constexpr int kAccessorOffset = P::kHeaderSize;
  static constexpr int kAccessorOffsetEnd = kAccessorOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=8&c=3
  static constexpr int kContextOffset = kAccessorOffsetEnd + 1;
  static constexpr int kContextOffsetEnd = kContextOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kContextOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kContextOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kContextOffsetEnd + 1;
  static constexpr int kHeaderSize = kContextOffsetEnd + 1;
  static constexpr int kSize = kContextOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=5&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/megadom-handler.tq?l=5&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedMegaDomHandler() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedMegaDomHandler, DAlias>,
        "class TorqueGeneratedMegaDomHandler should be used as direct base for MegaDomHandler.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedMegaDomHandler(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

