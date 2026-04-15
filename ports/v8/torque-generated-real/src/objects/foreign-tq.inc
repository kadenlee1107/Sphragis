class TrustedForeign;
// Alias for IsTrustedForeign() that avoids inlining.
// https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1
V8_EXPORT_PRIVATE bool IsTrustedForeign_NonInline(Tagged<HeapObject> o);

// Definition https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1
template <class D, class P>
class TorqueGeneratedTrustedForeign : public P {
  static_assert(
      std::is_same_v<TrustedForeign, D>,
      "Use this class as direct base for TrustedForeign.");
  static_assert(
      std::is_same_v<TrustedObject, P>,
      "Pass in TrustedObject as second template parameter for TorqueGeneratedTrustedForeign.");

 public: 
  using Super = P;
  using TorqueGeneratedClass = TorqueGeneratedTrustedForeign<D,P>;

  // https://crsrc.org/c/v8/src/objects/foreign.tq?l=12&c=3
  inline Address foreign_address() const;
  // https://crsrc.org/c/v8/src/objects/foreign.tq?l=12&c=3
  inline void set_foreign_address(Address value);

  DECL_PRINTER(TrustedForeign)

#ifdef VERIFY_HEAP
  // https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1
  V8_EXPORT_PRIVATE void TrustedForeignVerify(Isolate* isolate);
#endif  // VERIFY_HEAP

  // https://crsrc.org/c/v8/src/objects/foreign.tq?l=12&c=3
  static constexpr int kForeignAddressOffset = P::kHeaderSize;
  static constexpr int kForeignAddressOffsetEnd = kForeignAddressOffset + kSystemPointerSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kForeignAddressOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kForeignAddressOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kForeignAddressOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kForeignAddressOffsetEnd + 1;
  static constexpr int kHeaderSize = kForeignAddressOffsetEnd + 1;
  static constexpr int kSize = kForeignAddressOffsetEnd + 1;

  // https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1
  V8_INLINE static constexpr int32_t SizeFor() {
    DCHECK(kHeaderSize == kSize && kHeaderSize == 12);
    int32_t size = kHeaderSize;
    return size;
  }

  // https://crsrc.org/c/v8/src/objects/foreign.tq?l=11&c=1
  V8_INLINE int32_t AllocatedSize() const {
    return SizeFor();
  }

  friend class Factory;

  template <class DAlias = D>
  constexpr TorqueGeneratedTrustedForeign() : P() {
    static_assert(
        std::is_base_of_v<TorqueGeneratedTrustedForeign, DAlias>,
        "class TorqueGeneratedTrustedForeign should be used as direct base for TrustedForeign.");
  }

 protected:
  inline explicit constexpr TorqueGeneratedTrustedForeign(Address ptr, typename P::SkipTypeCheckTag
)    : P(ptr, typename P::SkipTypeCheckTag{}) {}
};

