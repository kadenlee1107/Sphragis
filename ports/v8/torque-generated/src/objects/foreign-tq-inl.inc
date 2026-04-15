// https://crsrc.org/c/v8/src/objects/foreign.tq?l=12&c=3
template<class D, class P>
Address TorqueGeneratedTrustedForeign<D, P>::foreign_address() const {
  Address value = this->template ReadField<Address>(kForeignAddressOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/foreign.tq?l=12&c=3
template<class D, class P>
void TorqueGeneratedTrustedForeign<D, P>::set_foreign_address(Address value) {
  this->template WriteField<Address>(kForeignAddressOffset, value);
}

