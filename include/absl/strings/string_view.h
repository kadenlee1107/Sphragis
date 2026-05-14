// Minimal Abseil string_view stub for V8 compilation on Sphragis
// Aliases to std::string_view (C++17)

#ifndef ABSL_STRINGS_STRING_VIEW_H_
#define ABSL_STRINGS_STRING_VIEW_H_

#include <string_view>
#include <string>
#include <ostream>

namespace absl {

using string_view = std::string_view;

// NullSafeStringView: returns empty view for nullptr
inline string_view NullSafeStringView(const char* p) {
  return p ? string_view(p) : string_view();
}

}  // namespace absl

#endif  // ABSL_STRINGS_STRING_VIEW_H_
