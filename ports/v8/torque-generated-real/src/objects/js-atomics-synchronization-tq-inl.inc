// https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=8&c=3
template<class D, class P>
uint32_t TorqueGeneratedJSSynchronizationPrimitive<D, P>::state() const {
  uint32_t value = this->template ReadField<uint32_t>(kStateOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=8&c=3
template<class D, class P>
void TorqueGeneratedJSSynchronizationPrimitive<D, P>::set_state(uint32_t value) {
  this->template WriteField<uint32_t>(kStateOffset, value);
}

// https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=12&c=3
template<class D, class P>
int32_t TorqueGeneratedJSAtomicsMutex<D, P>::owner_thread_id() const {
  int32_t value = this->template ReadField<int32_t>(kOwnerThreadIdOffset);
  return value;
}

// https://crsrc.org/c/v8/src/objects/js-atomics-synchronization.tq?l=12&c=3
template<class D, class P>
void TorqueGeneratedJSAtomicsMutex<D, P>::set_owner_thread_id(int32_t value) {
  this->template WriteField<int32_t>(kOwnerThreadIdOffset, value);
}

