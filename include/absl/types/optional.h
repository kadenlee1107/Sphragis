// Minimal Abseil optional stub for V8 compilation on Sphragis
// Aliases to std::optional (C++17)

#ifndef ABSL_TYPES_OPTIONAL_H_
#define ABSL_TYPES_OPTIONAL_H_

#include <optional>
#include <utility>

namespace absl {

template <typename T>
using optional = std::optional<T>;

using std::nullopt_t;
using std::nullopt;

using std::make_optional;
using std::bad_optional_access;

// in_place tag
using std::in_place_t;
using std::in_place;

}  // namespace absl

#endif  // ABSL_TYPES_OPTIONAL_H_
