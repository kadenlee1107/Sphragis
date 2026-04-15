#ifndef ABSL_BTREE_MAP_H_
#define ABSL_BTREE_MAP_H_
#include <map>
namespace absl {
template<class K, class V, class C = std::less<K>, class A = std::allocator<std::pair<const K, V>>>
using btree_map = std::map<K, V, C, A>;
template<class K, class V, class C = std::less<K>, class A = std::allocator<std::pair<const K, V>>>
using btree_multimap = std::map<K, V, C, A>;
}
#endif
