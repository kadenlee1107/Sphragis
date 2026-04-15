// https://crsrc.org/c/v8/src/objects/embedder-data-array.tq?l=7&c=3
template<class D, class P>
int TorqueGeneratedEmbedderDataArray<D, P>::length() const {
  int value = TaggedField<Smi>::load(*this, kLengthOffset).value();
  return value;
}

// https://crsrc.org/c/v8/src/objects/embedder-data-array.tq?l=7&c=3
template<class D, class P>
void TorqueGeneratedEmbedderDataArray<D, P>::set_length(int value) {
  WRITE_FIELD(*this, kLengthOffset, Smi::FromInt(value));
}

