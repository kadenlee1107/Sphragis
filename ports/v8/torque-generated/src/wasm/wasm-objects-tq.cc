#include "src/wasm/wasm-objects-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1
bool IsWasmInstanceObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmInstanceObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmInstanceObject<WasmInstanceObject, JSObject>::WasmInstanceObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmInstanceObjectVerify(TrustedCast<WasmInstanceObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1
bool IsWasmImportData_NonInline(Tagged<HeapObject> o) {
  return IsWasmImportData(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmImportData<WasmImportData, TrustedObject>::WasmImportDataVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmImportDataVerify(TrustedCast<WasmImportData>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1
bool IsWasmInternalFunction_NonInline(Tagged<HeapObject> o) {
  return IsWasmInternalFunction(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmInternalFunction<WasmInternalFunction, ExposedTrustedObject>::WasmInternalFunctionVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmInternalFunctionVerify(TrustedCast<WasmInternalFunction>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1
bool IsWasmFuncRef_NonInline(Tagged<HeapObject> o) {
  return IsWasmFuncRef(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmFuncRef<WasmFuncRef, HeapObject>::WasmFuncRefVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmFuncRefVerify(TrustedCast<WasmFuncRef>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1
bool IsWasmFunctionData_NonInline(Tagged<HeapObject> o) {
  return IsWasmFunctionData(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmFunctionData<WasmFunctionData, ExposedTrustedObject>::WasmFunctionDataVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmFunctionDataVerify(TrustedCast<WasmFunctionData>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1
bool IsWasmExportedFunctionData_NonInline(Tagged<HeapObject> o) {
  return IsWasmExportedFunctionData(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmExportedFunctionData<WasmExportedFunctionData, WasmFunctionData>::WasmExportedFunctionDataVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmExportedFunctionDataVerify(TrustedCast<WasmExportedFunctionData>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1
bool IsWasmJSFunctionData_NonInline(Tagged<HeapObject> o) {
  return IsWasmJSFunctionData(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmJSFunctionData<WasmJSFunctionData, WasmFunctionData>::WasmJSFunctionDataVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmJSFunctionDataVerify(TrustedCast<WasmJSFunctionData>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1
bool IsWasmCapiFunctionData_NonInline(Tagged<HeapObject> o) {
  return IsWasmCapiFunctionData(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmCapiFunctionData<WasmCapiFunctionData, WasmFunctionData>::WasmCapiFunctionDataVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmCapiFunctionDataVerify(TrustedCast<WasmCapiFunctionData>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1
bool IsWasmResumeData_NonInline(Tagged<HeapObject> o) {
  return IsWasmResumeData(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmResumeData<WasmResumeData, HeapObject>::WasmResumeDataVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmResumeDataVerify(TrustedCast<WasmResumeData>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1
bool IsWasmSuspenderObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmSuspenderObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmSuspenderObject<WasmSuspenderObject, ExposedTrustedObject>::WasmSuspenderObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmSuspenderObjectVerify(TrustedCast<WasmSuspenderObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1
bool IsWasmContinuationObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmContinuationObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmContinuationObject<WasmContinuationObject, HeapObject>::WasmContinuationObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmContinuationObjectVerify(TrustedCast<WasmContinuationObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1
bool IsWasmStackObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmStackObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmStackObject<WasmStackObject, HeapObject>::WasmStackObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmStackObjectVerify(TrustedCast<WasmStackObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1
bool IsWasmExceptionTag_NonInline(Tagged<HeapObject> o) {
  return IsWasmExceptionTag(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmExceptionTag<WasmExceptionTag, Struct>::WasmExceptionTagVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmExceptionTagVerify(TrustedCast<WasmExceptionTag>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1
bool IsWasmModuleObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmModuleObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmModuleObject<WasmModuleObject, JSObject>::WasmModuleObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmModuleObjectVerify(TrustedCast<WasmModuleObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1
bool IsWasmTableObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmTableObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmTableObject<WasmTableObject, JSObject>::WasmTableObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmTableObjectVerify(TrustedCast<WasmTableObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1
bool IsWasmMemoryObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmMemoryObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmMemoryObject<WasmMemoryObject, JSObject>::WasmMemoryObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmMemoryObjectVerify(TrustedCast<WasmMemoryObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1
bool IsWasmMemoryMapDescriptor_NonInline(Tagged<HeapObject> o) {
  return IsWasmMemoryMapDescriptor(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmMemoryMapDescriptor<WasmMemoryMapDescriptor, JSObject>::WasmMemoryMapDescriptorVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmMemoryMapDescriptorVerify(TrustedCast<WasmMemoryMapDescriptor>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1
bool IsWasmGlobalObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmGlobalObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmGlobalObject<WasmGlobalObject, JSObject>::WasmGlobalObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmGlobalObjectVerify(TrustedCast<WasmGlobalObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1
bool IsWasmTagObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmTagObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmTagObject<WasmTagObject, JSObject>::WasmTagObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmTagObjectVerify(TrustedCast<WasmTagObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1
bool IsAsmWasmData_NonInline(Tagged<HeapObject> o) {
  return IsAsmWasmData(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedAsmWasmData<AsmWasmData, Struct>::AsmWasmDataVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::AsmWasmDataVerify(TrustedCast<AsmWasmData>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1
bool IsWasmTypeInfo_NonInline(Tagged<HeapObject> o) {
  return IsWasmTypeInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmTypeInfo<WasmTypeInfo, HeapObject>::WasmTypeInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmTypeInfoVerify(TrustedCast<WasmTypeInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=311&c=1
bool IsWasmObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmObject<WasmObject, JSReceiver>::WasmObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmObjectVerify(TrustedCast<WasmObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1
bool IsWasmStruct_NonInline(Tagged<HeapObject> o) {
  return IsWasmStruct(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmStruct<WasmStruct, WasmObject>::WasmStructVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmStructVerify(TrustedCast<WasmStruct>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1
bool IsWasmArray_NonInline(Tagged<HeapObject> o) {
  return IsWasmArray(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmArray<WasmArray, WasmObject>::WasmArrayVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmArrayVerify(TrustedCast<WasmArray>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1
bool IsWasmNull_NonInline(Tagged<HeapObject> o) {
  return IsWasmNull(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmNull<WasmNull, HeapObject>::WasmNullVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmNullVerify(TrustedCast<WasmNull>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1
bool IsWasmSuspendingObject_NonInline(Tagged<HeapObject> o) {
  return IsWasmSuspendingObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedWasmSuspendingObject<WasmSuspendingObject, JSObject>::WasmSuspendingObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::WasmSuspendingObjectVerify(TrustedCast<WasmSuspendingObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
