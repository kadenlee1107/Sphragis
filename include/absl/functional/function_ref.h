// Minimal Abseil FunctionRef stub for V8 compilation on Bat_OS
// Non-owning callable reference (like std::function_ref from C++26)

#ifndef ABSL_FUNCTIONAL_FUNCTION_REF_H_
#define ABSL_FUNCTIONAL_FUNCTION_REF_H_

#include <cstddef>
#include <type_traits>
#include <utility>

namespace absl {

// Primary template (undefined)
template <typename Sig>
class FunctionRef;

// Partial specialization for function types
template <typename R, typename... Args>
class FunctionRef<R(Args...)> {
 public:
  // Construct from any compatible callable (not a FunctionRef itself)
  template <typename F,
            typename = std::enable_if_t<
                !std::is_same_v<std::decay_t<F>, FunctionRef> &&
                std::is_invocable_r_v<R, F&, Args...>>>
  FunctionRef(F&& f) noexcept
      : obj_(const_cast<void*>(
            static_cast<const void*>(std::addressof(f)))),
        callback_([](void* obj, Args... args) -> R {
          return (*static_cast<std::add_pointer_t<F>>(obj))(
              std::forward<Args>(args)...);
        }) {}

  R operator()(Args... args) const {
    return callback_(obj_, std::forward<Args>(args)...);
  }

 private:
  void* obj_;
  R (*callback_)(void*, Args...);
};

}  // namespace absl

#endif  // ABSL_FUNCTIONAL_FUNCTION_REF_H_
