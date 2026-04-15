#include "src/objects/module-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/module.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/module.tq?l=5&c=1
class TorqueGeneratedModuleAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(HeapObjectLayout);
  // https://crsrc.org/c/v8/src/objects/module.tq?l=9&c=3
  static constexpr int kExportsOffset = sizeof(HeapObjectLayout);
  static constexpr int kExportsOffsetEnd = kExportsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=11&c=3
  static constexpr int kHashOffset = kExportsOffsetEnd + 1;
  static constexpr int kHashOffsetEnd = kHashOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=12&c=3
  static constexpr int kStatusOffset = kHashOffsetEnd + 1;
  static constexpr int kStatusOffsetEnd = kStatusOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=13&c=3
  static constexpr int kModuleNamespaceOffset = kStatusOffsetEnd + 1;
  static constexpr int kModuleNamespaceOffsetEnd = kModuleNamespaceOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=14&c=3
  static constexpr int kDeferredModuleNamespaceOffset = kModuleNamespaceOffsetEnd + 1;
  static constexpr int kDeferredModuleNamespaceOffsetEnd = kDeferredModuleNamespaceOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=16&c=3
  static constexpr int kExceptionOffset = kDeferredModuleNamespaceOffsetEnd + 1;
  static constexpr int kExceptionOffsetEnd = kExceptionOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=19&c=3
  static constexpr int kTopLevelCapabilityOffset = kExceptionOffsetEnd + 1;
  static constexpr int kTopLevelCapabilityOffsetEnd = kTopLevelCapabilityOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kTopLevelCapabilityOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kTopLevelCapabilityOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kTopLevelCapabilityOffsetEnd + 1;
  static constexpr int kHeaderSize = kTopLevelCapabilityOffsetEnd + 1;

  static_assert(kExportsOffset == offsetof(Module, exports_),
                "Value of Module::kExportsOffset defined in Torque and offset of field Module::exports in C++ do not match");
  static_assert(kHashOffset == offsetof(Module, hash_),
                "Value of Module::kHashOffset defined in Torque and offset of field Module::hash in C++ do not match");
  static_assert(kStatusOffset == offsetof(Module, status_),
                "Value of Module::kStatusOffset defined in Torque and offset of field Module::status in C++ do not match");
  static_assert(kModuleNamespaceOffset == offsetof(Module, module_namespace_),
                "Value of Module::kModuleNamespaceOffset defined in Torque and offset of field Module::module_namespace in C++ do not match");
  static_assert(kDeferredModuleNamespaceOffset == offsetof(Module, deferred_module_namespace_),
                "Value of Module::kDeferredModuleNamespaceOffset defined in Torque and offset of field Module::deferred_module_namespace in C++ do not match");
  static_assert(kExceptionOffset == offsetof(Module, exception_),
                "Value of Module::kExceptionOffset defined in Torque and offset of field Module::exception in C++ do not match");
  static_assert(kTopLevelCapabilityOffset == offsetof(Module, top_level_capability_),
                "Value of Module::kTopLevelCapabilityOffset defined in Torque and offset of field Module::top_level_capability in C++ do not match");
};

// https://crsrc.org/c/v8/src/objects/module.tq?l=22&c=1
bool IsJSModuleNamespace_NonInline(Tagged<HeapObject> o) {
  return IsJSModuleNamespace(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSModuleNamespace<JSModuleNamespace, JSSpecialObject>::JSModuleNamespaceVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSModuleNamespaceVerify(TrustedCast<JSModuleNamespace>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/module.tq?l=26&c=1
bool IsJSDeferredModuleNamespace_NonInline(Tagged<HeapObject> o) {
  return IsJSDeferredModuleNamespace(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSDeferredModuleNamespace<JSDeferredModuleNamespace, JSModuleNamespace>::JSDeferredModuleNamespaceVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSDeferredModuleNamespaceVerify(TrustedCast<JSDeferredModuleNamespace>(*this), isolate);
}


#endif  // VERIFY_HEAP
// Definition https://crsrc.org/c/v8/src/objects/module.tq?l=28&c=1
class TorqueGeneratedScriptOrModuleAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/module.tq?l=30&c=3
  static constexpr int kResourceNameOffset = sizeof(StructLayout);
  static constexpr int kResourceNameOffsetEnd = kResourceNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/module.tq?l=31&c=3
  static constexpr int kHostDefinedOptionsOffset = kResourceNameOffsetEnd + 1;
  static constexpr int kHostDefinedOptionsOffsetEnd = kHostDefinedOptionsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kHostDefinedOptionsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kHostDefinedOptionsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kHostDefinedOptionsOffsetEnd + 1;
  static constexpr int kHeaderSize = kHostDefinedOptionsOffsetEnd + 1;
  static constexpr int kSize = kHostDefinedOptionsOffsetEnd + 1;

  static_assert(kResourceNameOffset == offsetof(ScriptOrModule, resource_name_),
                "Value of ScriptOrModule::kResourceNameOffset defined in Torque and offset of field ScriptOrModule::resource_name in C++ do not match");
  static_assert(kHostDefinedOptionsOffset == offsetof(ScriptOrModule, host_defined_options_),
                "Value of ScriptOrModule::kHostDefinedOptionsOffset defined in Torque and offset of field ScriptOrModule::host_defined_options in C++ do not match");
  static_assert(kSize == sizeof(ScriptOrModule));
};

} // namespace internal
} // namespace v8
