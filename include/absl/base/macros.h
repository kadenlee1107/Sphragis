// Minimal Abseil base macros stub for V8 compilation on Sphragis

#ifndef ABSL_BASE_MACROS_H_
#define ABSL_BASE_MACROS_H_

#include <cassert>
#include <cstddef>

// ABSL_ARRAYSIZE: compile-time array size
#ifndef ABSL_ARRAYSIZE
#define ABSL_ARRAYSIZE(array) \
  (sizeof(::absl::macros_internal::ArraySizeHelper(array)))
#endif

namespace absl {
namespace macros_internal {

// Template trick to get array size at compile time with type safety
template <typename T, std::size_t N>
auto constexpr ArraySizeHelper(const T (&)[N]) -> char (&)[N];

}  // namespace macros_internal
}  // namespace absl

// ABSL_ASSERT: debug assertion
#ifndef ABSL_ASSERT
#define ABSL_ASSERT(expr) assert(expr)
#endif

// ABSL_HARDENING_ASSERT: same as ABSL_ASSERT in debug, may be active in opt
#ifndef ABSL_HARDENING_ASSERT
#define ABSL_HARDENING_ASSERT(expr) assert(expr)
#endif

// Attribute macros V8 may reference
#ifndef ABSL_ATTRIBUTE_ALWAYS_INLINE
#if defined(__GNUC__) || defined(__clang__)
#define ABSL_ATTRIBUTE_ALWAYS_INLINE __attribute__((always_inline))
#else
#define ABSL_ATTRIBUTE_ALWAYS_INLINE
#endif
#endif

#ifndef ABSL_ATTRIBUTE_NOINLINE
#if defined(__GNUC__) || defined(__clang__)
#define ABSL_ATTRIBUTE_NOINLINE __attribute__((noinline))
#else
#define ABSL_ATTRIBUTE_NOINLINE
#endif
#endif

#ifndef ABSL_ATTRIBUTE_UNUSED
#if defined(__GNUC__) || defined(__clang__)
#define ABSL_ATTRIBUTE_UNUSED __attribute__((unused))
#else
#define ABSL_ATTRIBUTE_UNUSED
#endif
#endif

#ifndef ABSL_MUST_USE_RESULT
#if defined(__GNUC__) || defined(__clang__)
#define ABSL_MUST_USE_RESULT __attribute__((warn_unused_result))
#else
#define ABSL_MUST_USE_RESULT
#endif
#endif

// ABSL_PREDICT_TRUE / ABSL_PREDICT_FALSE: branch prediction hints
#ifndef ABSL_PREDICT_TRUE
#if defined(__GNUC__) || defined(__clang__)
#define ABSL_PREDICT_TRUE(x) (__builtin_expect(!!(x), 1))
#define ABSL_PREDICT_FALSE(x) (__builtin_expect(!!(x), 0))
#else
#define ABSL_PREDICT_TRUE(x) (x)
#define ABSL_PREDICT_FALSE(x) (x)
#endif
#endif

// ABSL_DEPRECATED
#ifndef ABSL_DEPRECATED
#define ABSL_DEPRECATED(msg) __attribute__((deprecated(msg)))
#endif

// ABSL_CONST_INIT
#ifndef ABSL_CONST_INIT
#if defined(__clang__)
#define ABSL_CONST_INIT [[clang::require_constant_initialization]]
#else
#define ABSL_CONST_INIT
#endif
#endif

// ABSL_FALLTHROUGH_INTENDED
#ifndef ABSL_FALLTHROUGH_INTENDED
#define ABSL_FALLTHROUGH_INTENDED [[fallthrough]]
#endif

// ABSL_BAD_CALL_IF: used to generate compile errors on bad calls
#ifndef ABSL_BAD_CALL_IF
#define ABSL_BAD_CALL_IF(expr, msg)
#endif

#endif  // ABSL_BASE_MACROS_H_
