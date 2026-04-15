#ifndef ABSL_FUNCTIONAL_OVERLOAD_H_
#define ABSL_FUNCTIONAL_OVERLOAD_H_

namespace absl {
template<class... Ts>
struct Overload : Ts... {
    using Ts::operator()...;
};
template<class... Ts>
Overload(Ts...) -> Overload<Ts...>;

// Also lowercase alias
template<class... Ts>
using overload = Overload<Ts...>;
}

#endif
