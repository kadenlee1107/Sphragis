#include "src/objects/microtask.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/microtask.tq?l=5&c=1
class TorqueGeneratedMicrotaskAsserts {
  static constexpr int kStartOfWeakFieldsOffset = sizeof(StructLayout);
  static constexpr int kEndOfWeakFieldsOffset = sizeof(StructLayout);
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  static constexpr int kEndOfStrongFieldsOffset = sizeof(StructLayout);
  static constexpr int kHeaderSize = sizeof(StructLayout);

};

// Definition https://crsrc.org/c/v8/src/objects/microtask.tq?l=12&c=1
class TorqueGeneratedCallbackTaskAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(Microtask);
  // https://crsrc.org/c/v8/src/objects/microtask.tq?l=14&c=3
  static constexpr int kCallbackOffset = sizeof(Microtask);
  static constexpr int kCallbackOffsetEnd = kCallbackOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/microtask.tq?l=15&c=3
  static constexpr int kDataOffset = kCallbackOffsetEnd + 1;
  static constexpr int kDataOffsetEnd = kDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kDataOffsetEnd + 1;
  static constexpr int kHeaderSize = kDataOffsetEnd + 1;
  static constexpr int kSize = kDataOffsetEnd + 1;

  static_assert(kCallbackOffset == offsetof(CallbackTask, callback_),
                "Value of CallbackTask::kCallbackOffset defined in Torque and offset of field CallbackTask::callback in C++ do not match");
  static_assert(kDataOffset == offsetof(CallbackTask, data_),
                "Value of CallbackTask::kDataOffset defined in Torque and offset of field CallbackTask::data in C++ do not match");
  static_assert(kSize == sizeof(CallbackTask));
};

// Definition https://crsrc.org/c/v8/src/objects/microtask.tq?l=18&c=1
class TorqueGeneratedCallableTaskAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(Microtask);
  // https://crsrc.org/c/v8/src/objects/microtask.tq?l=20&c=3
  static constexpr int kCallableOffset = sizeof(Microtask);
  static constexpr int kCallableOffsetEnd = kCallableOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/microtask.tq?l=21&c=3
  static constexpr int kContextOffset = kCallableOffsetEnd + 1;
  static constexpr int kContextOffsetEnd = kContextOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kContextOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kContextOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kContextOffsetEnd + 1;
  static constexpr int kHeaderSize = kContextOffsetEnd + 1;
  static constexpr int kSize = kContextOffsetEnd + 1;

  static_assert(kCallableOffset == offsetof(CallableTask, callable_),
                "Value of CallableTask::kCallableOffset defined in Torque and offset of field CallableTask::callable in C++ do not match");
  static_assert(kContextOffset == offsetof(CallableTask, context_),
                "Value of CallableTask::kContextOffset defined in Torque and offset of field CallableTask::context in C++ do not match");
  static_assert(kSize == sizeof(CallableTask));
};

// Definition https://crsrc.org/c/v8/src/objects/microtask.tq?l=27&c=1
class TorqueGeneratedAsyncResumeTaskAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(Microtask);
  // https://crsrc.org/c/v8/src/objects/microtask.tq?l=29&c=3
  static constexpr int kGeneratorOffset = sizeof(Microtask);
  static constexpr int kGeneratorOffsetEnd = kGeneratorOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/microtask.tq?l=30&c=3
  static constexpr int kValueOffset = kGeneratorOffsetEnd + 1;
  static constexpr int kValueOffsetEnd = kValueOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/microtask.tq?l=31&c=3
  static constexpr int kKindOffset = kValueOffsetEnd + 1;
  static constexpr int kKindOffsetEnd = kKindOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kKindOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kKindOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kKindOffsetEnd + 1;
  static constexpr int kHeaderSize = kKindOffsetEnd + 1;
  static constexpr int kSize = kKindOffsetEnd + 1;

  static_assert(kGeneratorOffset == offsetof(AsyncResumeTask, generator_),
                "Value of AsyncResumeTask::kGeneratorOffset defined in Torque and offset of field AsyncResumeTask::generator in C++ do not match");
  static_assert(kValueOffset == offsetof(AsyncResumeTask, value_),
                "Value of AsyncResumeTask::kValueOffset defined in Torque and offset of field AsyncResumeTask::value in C++ do not match");
  static_assert(kKindOffset == offsetof(AsyncResumeTask, kind_),
                "Value of AsyncResumeTask::kKindOffset defined in Torque and offset of field AsyncResumeTask::kind in C++ do not match");
  static_assert(kSize == sizeof(AsyncResumeTask));
};

} // namespace internal
} // namespace v8
