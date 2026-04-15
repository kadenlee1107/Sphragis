// Minimal Abseil flat_hash_map stub for V8 compilation on Bat_OS
// Aliases to std::unordered_map

#ifndef ABSL_CONTAINER_FLAT_HASH_MAP_H_
#define ABSL_CONTAINER_FLAT_HASH_MAP_H_

#include <functional>
#include <unordered_map>

namespace absl {

template <class K, class V,
          class Hash = std::hash<K>,
          class Eq = std::equal_to<K>,
          class Allocator = std::allocator<std::pair<const K, V>>>
using flat_hash_map = std::unordered_map<K, V, Hash, Eq, Allocator>;

}  // namespace absl

#endif  // ABSL_CONTAINER_FLAT_HASH_MAP_H_
