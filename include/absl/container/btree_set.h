#ifndef ABSL_BTREE_SET_H_
#define ABSL_BTREE_SET_H_
#include <set>
namespace absl {
template<class T, class C = std::less<T>, class A = std::allocator<T>>
using btree_set = std::set<T, C, A>;
}
#endif
