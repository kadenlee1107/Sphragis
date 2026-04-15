// Minimal Abseil span stub for V8 compilation on Bat_OS
// Aliases to std::span (C++20)

#ifndef ABSL_TYPES_SPAN_H_
#define ABSL_TYPES_SPAN_H_

#if __cplusplus >= 202002L
#include <span>
#endif

#include <cstddef>
#include <type_traits>

namespace absl {

#if __cplusplus >= 202002L

// C++20: use std::span directly
template <typename T, std::size_t Extent = std::dynamic_extent>
using Span = std::span<T, Extent>;

#else

// Pre-C++20 fallback: minimal span implementation
inline constexpr std::size_t dynamic_extent =
    static_cast<std::size_t>(-1);

template <typename T, std::size_t Extent = dynamic_extent>
class Span {
 public:
  using element_type = T;
  using value_type = std::remove_cv_t<T>;
  using size_type = std::size_t;
  using pointer = T*;
  using const_pointer = const T*;
  using reference = T&;
  using iterator = pointer;
  using const_iterator = const_pointer;

  constexpr Span() noexcept : data_(nullptr), size_(0) {}

  constexpr Span(T* data, size_type size) noexcept
      : data_(data), size_(size) {}

  template <std::size_t N>
  constexpr Span(T (&arr)[N]) noexcept : data_(arr), size_(N) {}

  constexpr pointer data() const noexcept { return data_; }
  constexpr size_type size() const noexcept { return size_; }
  constexpr bool empty() const noexcept { return size_ == 0; }

  constexpr reference operator[](size_type idx) const { return data_[idx]; }

  constexpr iterator begin() const noexcept { return data_; }
  constexpr iterator end() const noexcept { return data_ + size_; }

  constexpr reference front() const { return data_[0]; }
  constexpr reference back() const { return data_[size_ - 1]; }

  constexpr Span subspan(size_type offset,
                         size_type count = dynamic_extent) const {
    return Span(data_ + offset,
                count == dynamic_extent ? size_ - offset : count);
  }

  constexpr Span first(size_type count) const {
    return Span(data_, count);
  }

  constexpr Span last(size_type count) const {
    return Span(data_ + (size_ - count), count);
  }

 private:
  pointer data_;
  size_type size_;
};

#endif  // __cplusplus >= 202002L

// MakeSpan helper
template <typename T>
constexpr Span<T> MakeSpan(T* data, std::size_t size) {
  return Span<T>(data, size);
}

template <typename T, std::size_t N>
constexpr Span<T> MakeSpan(T (&arr)[N]) {
  return Span<T>(arr, N);
}

// MakeConstSpan helper
template <typename T>
constexpr Span<const T> MakeConstSpan(const T* data, std::size_t size) {
  return Span<const T>(data, size);
}

template <typename T, std::size_t N>
constexpr Span<const T> MakeConstSpan(const T (&arr)[N]) {
  return Span<const T>(arr, N);
}

}  // namespace absl

#endif  // ABSL_TYPES_SPAN_H_
