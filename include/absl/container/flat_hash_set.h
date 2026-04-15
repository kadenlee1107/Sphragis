// Minimal Abseil flat_hash_set stub for V8 compilation on Bat_OS
// Aliases to std::unordered_set

#ifndef ABSL_CONTAINER_FLAT_HASH_SET_H_
#define ABSL_CONTAINER_FLAT_HASH_SET_H_

#include <functional>
#include <unordered_set>

namespace absl {

template <class T,
          class Hash = std::hash<T>,
          class Eq = std::equal_to<T>,
          class Allocator = std::allocator<T>>
using flat_hash_set = std::unordered_set<T, Hash, Eq, Allocator>;

}  // namespace absl

#endif  // ABSL_CONTAINER_FLAT_HASH_SET_H_
