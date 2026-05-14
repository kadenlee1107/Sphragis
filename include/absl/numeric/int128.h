// Minimal Abseil int128 stub for V8 compilation on Sphragis
// Uses compiler __int128 intrinsic (supported on ARM64)

#ifndef ABSL_NUMERIC_INT128_H_
#define ABSL_NUMERIC_INT128_H_

#include <cstdint>
#include <ostream>

namespace absl {

using uint128 = unsigned __int128;
using int128 = __int128;

// Helper functions V8 may reference

inline uint128 MakeUint128(uint64_t high, uint64_t low) {
  return (static_cast<uint128>(high) << 64) | low;
}

inline uint128 Uint128Max() {
  return MakeUint128(~uint64_t{0}, ~uint64_t{0});
}

inline uint64_t Uint128High64(uint128 v) {
  return static_cast<uint64_t>(v >> 64);
}

inline uint64_t Uint128Low64(uint128 v) {
  return static_cast<uint64_t>(v);
}

inline int128 MakeInt128(int64_t high, uint64_t low) {
  return (static_cast<int128>(high) << 64) | low;
}

inline int64_t Int128High64(int128 v) {
  return static_cast<int64_t>(static_cast<uint128>(v) >> 64);
}

inline uint64_t Int128Low64(int128 v) {
  return static_cast<uint64_t>(v);
}

}  // namespace absl

#endif  // ABSL_NUMERIC_INT128_H_
