// Minimal Abseil Time stub for V8 compilation on Bat_OS

#ifndef ABSL_TIME_TIME_H_
#define ABSL_TIME_TIME_H_

#include <cstdint>
#include <limits>

namespace absl {

// Duration: represents a span of time, stored as nanoseconds
class Duration {
 public:
  constexpr Duration() : nanos_(0) {}

  constexpr bool operator==(Duration other) const {
    return nanos_ == other.nanos_;
  }
  constexpr bool operator!=(Duration other) const {
    return nanos_ != other.nanos_;
  }
  constexpr bool operator<(Duration other) const {
    return nanos_ < other.nanos_;
  }
  constexpr bool operator<=(Duration other) const {
    return nanos_ <= other.nanos_;
  }
  constexpr bool operator>(Duration other) const {
    return nanos_ > other.nanos_;
  }
  constexpr bool operator>=(Duration other) const {
    return nanos_ >= other.nanos_;
  }

  constexpr Duration operator+(Duration other) const {
    return Duration(nanos_ + other.nanos_);
  }
  constexpr Duration operator-(Duration other) const {
    return Duration(nanos_ - other.nanos_);
  }

 private:
  friend constexpr Duration FromNanoseconds(int64_t n);
  friend constexpr Duration FromMicroseconds(int64_t n);
  friend constexpr Duration FromMilliseconds(int64_t n);
  friend constexpr Duration FromSeconds(int64_t n);
  friend constexpr Duration InfiniteDuration();
  friend constexpr int64_t ToInt64Nanoseconds(Duration d);
  friend constexpr int64_t ToInt64Microseconds(Duration d);
  friend constexpr int64_t ToInt64Milliseconds(Duration d);
  friend constexpr int64_t ToInt64Seconds(Duration d);

  constexpr explicit Duration(int64_t nanos) : nanos_(nanos) {}
  int64_t nanos_;
};

constexpr Duration FromNanoseconds(int64_t n) { return Duration(n); }
constexpr Duration FromMicroseconds(int64_t n) { return Duration(n * 1000); }
constexpr Duration FromMilliseconds(int64_t n) {
  return Duration(n * 1000000);
}
constexpr Duration FromSeconds(int64_t n) {
  return Duration(n * 1000000000);
}

// Aliases V8 may use
constexpr Duration Nanoseconds(int64_t n) { return FromNanoseconds(n); }
constexpr Duration Microseconds(int64_t n) { return FromMicroseconds(n); }
constexpr Duration Milliseconds(int64_t n) { return FromMilliseconds(n); }
constexpr Duration Seconds(int64_t n) { return FromSeconds(n); }
constexpr Duration Minutes(int64_t n) { return FromSeconds(n * 60); }
constexpr Duration Hours(int64_t n) { return FromSeconds(n * 3600); }

constexpr Duration InfiniteDuration() {
  return Duration(std::numeric_limits<int64_t>::max());
}

constexpr Duration ZeroDuration() { return Duration(0); }

constexpr int64_t ToInt64Nanoseconds(Duration d) { return d.nanos_; }
constexpr int64_t ToInt64Microseconds(Duration d) { return d.nanos_ / 1000; }
constexpr int64_t ToInt64Milliseconds(Duration d) {
  return d.nanos_ / 1000000;
}
constexpr int64_t ToInt64Seconds(Duration d) {
  return d.nanos_ / 1000000000;
}

inline double ToDoubleSeconds(Duration d) {
  return static_cast<double>(d.nanos_) / 1e9;
}

// Time: represents an absolute point in time, stored as nanos since epoch
class Time {
 public:
  constexpr Time() : nanos_(0) {}

  constexpr bool operator==(Time other) const {
    return nanos_ == other.nanos_;
  }
  constexpr bool operator!=(Time other) const {
    return nanos_ != other.nanos_;
  }
  constexpr bool operator<(Time other) const {
    return nanos_ < other.nanos_;
  }
  constexpr bool operator<=(Time other) const {
    return nanos_ <= other.nanos_;
  }
  constexpr bool operator>(Time other) const {
    return nanos_ > other.nanos_;
  }
  constexpr bool operator>=(Time other) const {
    return nanos_ >= other.nanos_;
  }

  constexpr Time operator+(Duration d) const {
    return Time(nanos_ + ToInt64Nanoseconds(d));
  }
  constexpr Time operator-(Duration d) const {
    return Time(nanos_ - ToInt64Nanoseconds(d));
  }
  constexpr Duration operator-(Time other) const {
    return FromNanoseconds(nanos_ - other.nanos_);
  }

 private:
  friend constexpr Time FromUnixNanos(int64_t ns);
  friend constexpr int64_t ToUnixNanos(Time t);
  friend inline Time Now();
  friend constexpr Time InfiniteFuture();
  friend constexpr Time InfinitePast();
  friend constexpr Time UnixEpoch();

  constexpr explicit Time(int64_t nanos) : nanos_(nanos) {}
  int64_t nanos_;
};

constexpr Time UnixEpoch() { return Time(0); }

constexpr Time FromUnixNanos(int64_t ns) { return Time(ns); }
constexpr int64_t ToUnixNanos(Time t) { return t.nanos_; }

constexpr Time InfiniteFuture() {
  return Time(std::numeric_limits<int64_t>::max());
}

constexpr Time InfinitePast() {
  return Time(std::numeric_limits<int64_t>::min());
}

// Stub: returns epoch (bare-metal has no clock yet)
inline Time Now() { return Time(0); }

}  // namespace absl

#endif  // ABSL_TIME_TIME_H_
