#include "src/objects/promise.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/promise.tq?l=5&c=1
class TorqueGeneratedPromiseCapabilityAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=7&c=3
  static constexpr int kPromiseOffset = sizeof(StructLayout);
  static constexpr int kPromiseOffsetEnd = kPromiseOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=13&c=3
  static constexpr int kResolveOffset = kPromiseOffsetEnd + 1;
  static constexpr int kResolveOffsetEnd = kResolveOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=14&c=3
  static constexpr int kRejectOffset = kResolveOffsetEnd + 1;
  static constexpr int kRejectOffsetEnd = kRejectOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kRejectOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kRejectOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kRejectOffsetEnd + 1;
  static constexpr int kHeaderSize = kRejectOffsetEnd + 1;
  static constexpr int kSize = kRejectOffsetEnd + 1;

  static_assert(kPromiseOffset == offsetof(PromiseCapability, promise_),
                "Value of PromiseCapability::kPromiseOffset defined in Torque and offset of field PromiseCapability::promise in C++ do not match");
  static_assert(kResolveOffset == offsetof(PromiseCapability, resolve_),
                "Value of PromiseCapability::kResolveOffset defined in Torque and offset of field PromiseCapability::resolve in C++ do not match");
  static_assert(kRejectOffset == offsetof(PromiseCapability, reject_),
                "Value of PromiseCapability::kRejectOffset defined in Torque and offset of field PromiseCapability::reject in C++ do not match");
  static_assert(kSize == sizeof(PromiseCapability));
};

// Definition https://crsrc.org/c/v8/src/objects/promise.tq?l=34&c=1
class TorqueGeneratedPromiseReactionAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=38&c=3
  static constexpr int kNextOffset = sizeof(StructLayout);
  static constexpr int kNextOffsetEnd = kNextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=39&c=3
  static constexpr int kRejectHandlerOffset = kNextOffsetEnd + 1;
  static constexpr int kRejectHandlerOffsetEnd = kRejectHandlerOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=40&c=3
  static constexpr int kFulfillHandlerOffset = kRejectHandlerOffsetEnd + 1;
  static constexpr int kFulfillHandlerOffsetEnd = kFulfillHandlerOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=43&c=3
  static constexpr int kPromiseOrCapabilityOffset = kFulfillHandlerOffsetEnd + 1;
  static constexpr int kPromiseOrCapabilityOffsetEnd = kPromiseOrCapabilityOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPromiseOrCapabilityOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kPromiseOrCapabilityOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPromiseOrCapabilityOffsetEnd + 1;
  static constexpr int kHeaderSize = kPromiseOrCapabilityOffsetEnd + 1;
  static constexpr int kSize = kPromiseOrCapabilityOffsetEnd + 1;

  static_assert(kNextOffset == offsetof(PromiseReaction, next_),
                "Value of PromiseReaction::kNextOffset defined in Torque and offset of field PromiseReaction::next in C++ do not match");
  static_assert(kRejectHandlerOffset == offsetof(PromiseReaction, reject_handler_),
                "Value of PromiseReaction::kRejectHandlerOffset defined in Torque and offset of field PromiseReaction::reject_handler in C++ do not match");
  static_assert(kFulfillHandlerOffset == offsetof(PromiseReaction, fulfill_handler_),
                "Value of PromiseReaction::kFulfillHandlerOffset defined in Torque and offset of field PromiseReaction::fulfill_handler in C++ do not match");
  static_assert(kPromiseOrCapabilityOffset == offsetof(PromiseReaction, promise_or_capability_),
                "Value of PromiseReaction::kPromiseOrCapabilityOffset defined in Torque and offset of field PromiseReaction::promise_or_capability in C++ do not match");
  static_assert(kSize == sizeof(PromiseReaction));
};

// Definition https://crsrc.org/c/v8/src/objects/promise.tq?l=59&c=1
class TorqueGeneratedPromiseReactionJobTaskAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(Microtask);
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=62&c=3
  static constexpr int kArgumentOffset = sizeof(Microtask);
  static constexpr int kArgumentOffsetEnd = kArgumentOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=63&c=3
  static constexpr int kContextOffset = kArgumentOffsetEnd + 1;
  static constexpr int kContextOffsetEnd = kContextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=64&c=3
  static constexpr int kHandlerOffset = kContextOffsetEnd + 1;
  static constexpr int kHandlerOffsetEnd = kHandlerOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=67&c=3
  static constexpr int kPromiseOrCapabilityOffset = kHandlerOffsetEnd + 1;
  static constexpr int kPromiseOrCapabilityOffsetEnd = kPromiseOrCapabilityOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPromiseOrCapabilityOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kPromiseOrCapabilityOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPromiseOrCapabilityOffsetEnd + 1;
  static constexpr int kHeaderSize = kPromiseOrCapabilityOffsetEnd + 1;

  static_assert(kArgumentOffset == offsetof(PromiseReactionJobTask, argument_),
                "Value of PromiseReactionJobTask::kArgumentOffset defined in Torque and offset of field PromiseReactionJobTask::argument in C++ do not match");
  static_assert(kContextOffset == offsetof(PromiseReactionJobTask, context_),
                "Value of PromiseReactionJobTask::kContextOffset defined in Torque and offset of field PromiseReactionJobTask::context in C++ do not match");
  static_assert(kHandlerOffset == offsetof(PromiseReactionJobTask, handler_),
                "Value of PromiseReactionJobTask::kHandlerOffset defined in Torque and offset of field PromiseReactionJobTask::handler in C++ do not match");
  static_assert(kPromiseOrCapabilityOffset == offsetof(PromiseReactionJobTask, promise_or_capability_),
                "Value of PromiseReactionJobTask::kPromiseOrCapabilityOffset defined in Torque and offset of field PromiseReactionJobTask::promise_or_capability in C++ do not match");
};

// Definition https://crsrc.org/c/v8/src/objects/promise.tq?l=70&c=1
class TorqueGeneratedPromiseFulfillReactionJobTaskAsserts {
  static constexpr int kStartOfWeakFieldsOffset = sizeof(PromiseReactionJobTask);
  static constexpr int kEndOfWeakFieldsOffset = sizeof(PromiseReactionJobTask);
  static constexpr int kStartOfStrongFieldsOffset = sizeof(PromiseReactionJobTask);
  static constexpr int kEndOfStrongFieldsOffset = sizeof(PromiseReactionJobTask);
  static constexpr int kHeaderSize = sizeof(PromiseReactionJobTask);
  static constexpr int kSize = sizeof(PromiseReactionJobTask);

  static_assert(kSize == sizeof(PromiseFulfillReactionJobTask));
};

// Definition https://crsrc.org/c/v8/src/objects/promise.tq?l=73&c=1
class TorqueGeneratedPromiseRejectReactionJobTaskAsserts {
  static constexpr int kStartOfWeakFieldsOffset = sizeof(PromiseReactionJobTask);
  static constexpr int kEndOfWeakFieldsOffset = sizeof(PromiseReactionJobTask);
  static constexpr int kStartOfStrongFieldsOffset = sizeof(PromiseReactionJobTask);
  static constexpr int kEndOfStrongFieldsOffset = sizeof(PromiseReactionJobTask);
  static constexpr int kHeaderSize = sizeof(PromiseReactionJobTask);
  static constexpr int kSize = sizeof(PromiseReactionJobTask);

  static_assert(kSize == sizeof(PromiseRejectReactionJobTask));
};

// Definition https://crsrc.org/c/v8/src/objects/promise.tq?l=76&c=1
class TorqueGeneratedPromiseResolveThenableJobTaskAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(Microtask);
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=78&c=3
  static constexpr int kContextOffset = sizeof(Microtask);
  static constexpr int kContextOffsetEnd = kContextOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=79&c=3
  static constexpr int kPromiseToResolveOffset = kContextOffsetEnd + 1;
  static constexpr int kPromiseToResolveOffsetEnd = kPromiseToResolveOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=80&c=3
  static constexpr int kThenableOffset = kPromiseToResolveOffsetEnd + 1;
  static constexpr int kThenableOffsetEnd = kThenableOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/promise.tq?l=81&c=3
  static constexpr int kThenOffset = kThenableOffsetEnd + 1;
  static constexpr int kThenOffsetEnd = kThenOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kThenOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kThenOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kThenOffsetEnd + 1;
  static constexpr int kHeaderSize = kThenOffsetEnd + 1;
  static constexpr int kSize = kThenOffsetEnd + 1;

  static_assert(kContextOffset == offsetof(PromiseResolveThenableJobTask, context_),
                "Value of PromiseResolveThenableJobTask::kContextOffset defined in Torque and offset of field PromiseResolveThenableJobTask::context in C++ do not match");
  static_assert(kPromiseToResolveOffset == offsetof(PromiseResolveThenableJobTask, promise_to_resolve_),
                "Value of PromiseResolveThenableJobTask::kPromiseToResolveOffset defined in Torque and offset of field PromiseResolveThenableJobTask::promise_to_resolve in C++ do not match");
  static_assert(kThenableOffset == offsetof(PromiseResolveThenableJobTask, thenable_),
                "Value of PromiseResolveThenableJobTask::kThenableOffset defined in Torque and offset of field PromiseResolveThenableJobTask::thenable in C++ do not match");
  static_assert(kThenOffset == offsetof(PromiseResolveThenableJobTask, then_),
                "Value of PromiseResolveThenableJobTask::kThenOffset defined in Torque and offset of field PromiseResolveThenableJobTask::then in C++ do not match");
  static_assert(kSize == sizeof(PromiseResolveThenableJobTask));
};

} // namespace internal
} // namespace v8
