// Auto-generated instance type checkers for Bat_OS V8 port
// Generated from Torque INSTANCE_CHECKERS_MULTIPLE macros
#ifndef V8_BATOS_INSTANCE_TYPE_CHECKERS_GEN_H_
#define V8_BATOS_INSTANCE_TYPE_CHECKERS_GEN_H_

#include "src/objects/instance-type.h"
#include "src/objects/map.h"
#include "src/objects/tagged.h"

namespace v8 { namespace internal { namespace InstanceTypeChecker {

V8_INLINE constexpr bool IsUncompiledDataWithPreparseData(InstanceType type) {
  return type == UNCOMPILED_DATA_WITH_PREPARSE_DATA_TYPE;
}
V8_INLINE bool IsUncompiledDataWithPreparseData(Tagged<Map> map) {
  return IsUncompiledDataWithPreparseData(map->instance_type());
}

V8_INLINE constexpr bool IsUncompiledDataWithoutPreparseData(InstanceType type) {
  return type == UNCOMPILED_DATA_WITHOUT_PREPARSE_DATA_TYPE;
}
V8_INLINE bool IsUncompiledDataWithoutPreparseData(Tagged<Map> map) {
  return IsUncompiledDataWithoutPreparseData(map->instance_type());
}

V8_INLINE constexpr bool IsRegExpData(InstanceType type) {
  return type == REG_EXP_DATA_TYPE;
}
V8_INLINE bool IsRegExpData(Tagged<Map> map) {
  return IsRegExpData(map->instance_type());
}

V8_INLINE constexpr bool IsExportedSubClassBase(InstanceType type) {
  return type == EXPORTED_SUB_CLASS_BASE_TYPE;
}
V8_INLINE bool IsExportedSubClassBase(Tagged<Map> map) {
  return IsExportedSubClassBase(map->instance_type());
}

V8_INLINE constexpr bool IsFixedArray(InstanceType type) {
  return type == FIXED_ARRAY_TYPE;
}
V8_INLINE bool IsFixedArray(Tagged<Map> map) {
  return IsFixedArray(map->instance_type());
}

V8_INLINE constexpr bool IsTurboshaftFloat64Type(InstanceType type) {
  return type == TURBOSHAFT_FLOAT64_TYPE_TYPE;
}
V8_INLINE bool IsTurboshaftFloat64Type(Tagged<Map> map) {
  return IsTurboshaftFloat64Type(map->instance_type());
}

V8_INLINE constexpr bool IsTurboshaftWord32Type(InstanceType type) {
  return type == TURBOSHAFT_WORD32_TYPE_TYPE;
}
V8_INLINE bool IsTurboshaftWord32Type(Tagged<Map> map) {
  return IsTurboshaftWord32Type(map->instance_type());
}

V8_INLINE constexpr bool IsTurboshaftWord64Type(InstanceType type) {
  return type == TURBOSHAFT_WORD64_TYPE_TYPE;
}
V8_INLINE bool IsTurboshaftWord64Type(Tagged<Map> map) {
  return IsTurboshaftWord64Type(map->instance_type());
}

V8_INLINE constexpr bool IsDescriptorArray(InstanceType type) {
  return type == DESCRIPTOR_ARRAY_TYPE;
}
V8_INLINE bool IsDescriptorArray(Tagged<Map> map) {
  return IsDescriptorArray(map->instance_type());
}

V8_INLINE constexpr bool IsWeakFixedArray(InstanceType type) {
  return type == WEAK_FIXED_ARRAY_TYPE;
}
V8_INLINE bool IsWeakFixedArray(Tagged<Map> map) {
  return IsWeakFixedArray(map->instance_type());
}

V8_INLINE constexpr bool IsJSObject(InstanceType type) {
  return type == JS_OBJECT_TYPE;
}
V8_INLINE bool IsJSObject(Tagged<Map> map) {
  return IsJSObject(map->instance_type());
}

V8_INLINE constexpr bool IsJSModuleNamespace(InstanceType type) {
  return type == JS_MODULE_NAMESPACE_TYPE;
}
V8_INLINE bool IsJSModuleNamespace(Tagged<Map> map) {
  return IsJSModuleNamespace(map->instance_type());
}

V8_INLINE constexpr bool IsJSTypedArray(InstanceType type) {
  return type == JS_TYPED_ARRAY_TYPE;
}
V8_INLINE bool IsJSTypedArray(Tagged<Map> map) {
  return IsJSTypedArray(map->instance_type());
}

V8_INLINE constexpr bool IsJSFunctionWithPrototype(InstanceType type) {
  return type == JS_FUNCTION_WITH_PROTOTYPE_TYPE;
}
V8_INLINE bool IsJSFunctionWithPrototype(Tagged<Map> map) {
  return IsJSFunctionWithPrototype(map->instance_type());
}

V8_INLINE constexpr bool IsJSDisposableStackBase(InstanceType type) {
  return type == JS_DISPOSABLE_STACK_BASE_TYPE;
}
V8_INLINE bool IsJSDisposableStackBase(Tagged<Map> map) {
  return IsJSDisposableStackBase(map->instance_type());
}

V8_INLINE constexpr bool IsJSGeneratorObject(InstanceType type) {
  return type == JS_GENERATOR_OBJECT_TYPE;
}
V8_INLINE bool IsJSGeneratorObject(Tagged<Map> map) {
  return IsJSGeneratorObject(map->instance_type());
}

V8_INLINE constexpr bool IsJSObjectWithEmbedderSlots(InstanceType type) {
  return type == JS_OBJECT_WITH_EMBEDDER_SLOTS_TYPE;
}
V8_INLINE bool IsJSObjectWithEmbedderSlots(Tagged<Map> map) {
  return IsJSObjectWithEmbedderSlots(map->instance_type());
}

V8_INLINE constexpr bool IsHashTable(InstanceType type) {
  return type == HASH_TABLE_TYPE;
}
V8_INLINE bool IsHashTable(Tagged<Map> map) {
  return IsHashTable(map->instance_type());
}

V8_INLINE constexpr bool IsJSApiObject(InstanceType type) {
  return type == JS_API_OBJECT_TYPE;
}
V8_INLINE bool IsJSApiObject(Tagged<Map> map) {
  return IsJSApiObject(map->instance_type());
}

} } } // namespace v8::internal::InstanceTypeChecker

namespace v8 { namespace internal {

inline bool IsUncompiledDataWithPreparseData(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsUncompiledDataWithPreparseData(obj->map());
}
inline bool IsUncompiledDataWithPreparseData(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsUncompiledDataWithPreparseData(obj);
}

inline bool IsUncompiledDataWithoutPreparseData(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsUncompiledDataWithoutPreparseData(obj->map());
}
inline bool IsUncompiledDataWithoutPreparseData(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsUncompiledDataWithoutPreparseData(obj);
}

inline bool IsRegExpData(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsRegExpData(obj->map());
}
inline bool IsRegExpData(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsRegExpData(obj);
}

inline bool IsExportedSubClassBase(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsExportedSubClassBase(obj->map());
}
inline bool IsExportedSubClassBase(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsExportedSubClassBase(obj);
}

inline bool IsFixedArray(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsFixedArray(obj->map());
}
inline bool IsFixedArray(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsFixedArray(obj);
}

inline bool IsTurboshaftFloat64Type(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsTurboshaftFloat64Type(obj->map());
}
inline bool IsTurboshaftFloat64Type(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsTurboshaftFloat64Type(obj);
}

inline bool IsTurboshaftWord32Type(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsTurboshaftWord32Type(obj->map());
}
inline bool IsTurboshaftWord32Type(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsTurboshaftWord32Type(obj);
}

inline bool IsTurboshaftWord64Type(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsTurboshaftWord64Type(obj->map());
}
inline bool IsTurboshaftWord64Type(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsTurboshaftWord64Type(obj);
}

inline bool IsDescriptorArray(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsDescriptorArray(obj->map());
}
inline bool IsDescriptorArray(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsDescriptorArray(obj);
}

inline bool IsWeakFixedArray(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsWeakFixedArray(obj->map());
}
inline bool IsWeakFixedArray(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsWeakFixedArray(obj);
}

inline bool IsJSObject(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsJSObject(obj->map());
}
inline bool IsJSObject(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsJSObject(obj);
}

inline bool IsJSModuleNamespace(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsJSModuleNamespace(obj->map());
}
inline bool IsJSModuleNamespace(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsJSModuleNamespace(obj);
}

inline bool IsJSTypedArray(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsJSTypedArray(obj->map());
}
inline bool IsJSTypedArray(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsJSTypedArray(obj);
}

inline bool IsJSFunctionWithPrototype(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsJSFunctionWithPrototype(obj->map());
}
inline bool IsJSFunctionWithPrototype(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsJSFunctionWithPrototype(obj);
}

inline bool IsJSDisposableStackBase(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsJSDisposableStackBase(obj->map());
}
inline bool IsJSDisposableStackBase(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsJSDisposableStackBase(obj);
}

inline bool IsJSGeneratorObject(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsJSGeneratorObject(obj->map());
}
inline bool IsJSGeneratorObject(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsJSGeneratorObject(obj);
}

inline bool IsJSObjectWithEmbedderSlots(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsJSObjectWithEmbedderSlots(obj->map());
}
inline bool IsJSObjectWithEmbedderSlots(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsJSObjectWithEmbedderSlots(obj);
}

inline bool IsHashTable(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsHashTable(obj->map());
}
inline bool IsHashTable(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsHashTable(obj);
}

inline bool IsJSApiObject(Tagged<HeapObject> obj) {
  return InstanceTypeChecker::IsJSApiObject(obj->map());
}
inline bool IsJSApiObject(Tagged<HeapObject> obj, PtrComprCageBase) {
  return IsJSApiObject(obj);
}

} } // namespace v8::internal

#endif  // V8_BATOS_INSTANCE_TYPE_CHECKERS_GEN_H_
