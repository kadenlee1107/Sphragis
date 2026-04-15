#ifndef V8_GEN_TORQUE_GENERATED_SRC_WASM_WASM_OBJECTS_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_WASM_WASM_OBJECTS_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=25&c=1
TNode<Uint8T> FromConstexpr_AddressType_constexpr_kI32_0(compiler::CodeAssemblerState* state_, wasm::AddressType p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=25&c=1
TNode<Uint8T> FromConstexpr_AddressType_constexpr_kI64_0(compiler::CodeAssemblerState* state_, wasm::AddressType p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=32&c=1
TNode<UintPtrT> Convert_uintptr_WasmCodePointer_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_p);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=37&c=1
TNode<WasmTrustedInstanceData> Cast_WasmTrustedInstanceData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1
TNode<WasmInstanceObject> Cast_WasmInstanceObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1
TNode<WasmImportData> Cast_WasmImportData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=73&c=1
TNode<WasmFastApiCallData> Cast_WasmFastApiCallData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1
TNode<WasmInternalFunction> Cast_WasmInternalFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1
TNode<WasmFuncRef> Cast_WasmFuncRef_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1
TNode<WasmFunctionData> Cast_WasmFunctionData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1
TNode<WasmExportedFunctionData> Cast_WasmExportedFunctionData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1
TNode<WasmJSFunctionData> Cast_WasmJSFunctionData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1
TNode<WasmCapiFunctionData> Cast_WasmCapiFunctionData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1
TNode<WasmResumeData> Cast_WasmResumeData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1
TNode<WasmSuspenderObject> Cast_WasmSuspenderObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1
TNode<WasmContinuationObject> Cast_WasmContinuationObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1
TNode<WasmStackObject> Cast_WasmStackObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1
TNode<WasmExceptionTag> Cast_WasmExceptionTag_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=195&c=1
TNode<WasmExceptionPackage> Cast_WasmExceptionPackage_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1
TNode<WasmModuleObject> Cast_WasmModuleObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=202&c=1
TNode<WasmDispatchTable> Cast_WasmDispatchTable_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=203&c=1
TNode<WasmDispatchTableForImports> Cast_WasmDispatchTableForImports_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1
TNode<WasmTableObject> Cast_WasmTableObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1
TNode<WasmMemoryObject> Cast_WasmMemoryObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1
TNode<WasmMemoryMapDescriptor> Cast_WasmMemoryMapDescriptor_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1
TNode<WasmGlobalObject> Cast_WasmGlobalObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1
TNode<WasmTagObject> Cast_WasmTagObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1
TNode<AsmWasmData> Cast_AsmWasmData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1
TNode<WasmTypeInfo> Cast_WasmTypeInfo_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=311&c=1
TNode<WasmObject> Cast_WasmObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1
TNode<WasmStruct> Cast_WasmStruct_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1
TNode<WasmArray> Cast_WasmArray_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=325&c=1
TNode<WasmStringViewIter> Cast_WasmStringViewIter_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1
TNode<WasmNull> Cast_WasmNull_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=336&c=1
TNode<WasmNull> kWasmNull_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1
TNode<WasmSuspendingObject> Cast_WasmSuspendingObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=40&c=3
TNode<TrustedPointerT> LoadWasmInstanceObjectTrustedData_0(compiler::CodeAssemblerState* state_, TNode<WasmInstanceObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=40&c=3
void StoreWasmInstanceObjectTrustedData_0(compiler::CodeAssemblerState* state_, TNode<WasmInstanceObject> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
TNode<WasmModuleObject> LoadWasmInstanceObjectModuleObject_0(compiler::CodeAssemblerState* state_, TNode<WasmInstanceObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=41&c=3
void StoreWasmInstanceObjectModuleObject_0(compiler::CodeAssemblerState* state_, TNode<WasmInstanceObject> p_o, TNode<WasmModuleObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
TNode<JSObject> LoadWasmInstanceObjectExportsObject_0(compiler::CodeAssemblerState* state_, TNode<WasmInstanceObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=42&c=3
void StoreWasmInstanceObjectExportsObject_0(compiler::CodeAssemblerState* state_, TNode<WasmInstanceObject> p_o, TNode<JSObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=53&c=3
TNode<MaybeObject> LoadWasmImportDataProtectedInstanceData_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=53&c=3
void StoreWasmImportDataProtectedInstanceData_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=57&c=3
TNode<MaybeObject> LoadWasmImportDataProtectedCallOrigin_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=57&c=3
void StoreWasmImportDataProtectedCallOrigin_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
TNode<NativeContext> LoadWasmImportDataNativeContext_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=59&c=3
void StoreWasmImportDataNativeContext_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o, TNode<NativeContext> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
TNode<Union<JSReceiver, Undefined>> LoadWasmImportDataCallable_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=60&c=3
void StoreWasmImportDataCallable_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o, TNode<Union<JSReceiver, Undefined>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
TNode<Cell> LoadWasmImportDataWrapperBudget_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=62&c=3
void StoreWasmImportDataWrapperBudget_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o, TNode<Cell> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=64&c=3
TNode<RawPtrT> LoadWasmImportDataSig_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=64&c=3
void StoreWasmImportDataSig_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o, TNode<RawPtrT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=66&c=3
TNode<Uint32T> LoadWasmImportDataBitField_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=66&c=3
void StoreWasmImportDataBitField_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=74&c=3
TNode<HeapObject> LoadWasmFastApiCallDataSignature_0(compiler::CodeAssemblerState* state_, TNode<WasmFastApiCallData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=74&c=3
void StoreWasmFastApiCallDataSignature_0(compiler::CodeAssemblerState* state_, TNode<WasmFastApiCallData> p_o, TNode<HeapObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=75&c=3
TNode<Object> LoadWasmFastApiCallDataCallbackData_0(compiler::CodeAssemblerState* state_, TNode<WasmFastApiCallData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=75&c=3
void StoreWasmFastApiCallDataCallbackData_0(compiler::CodeAssemblerState* state_, TNode<WasmFastApiCallData> p_o, TNode<Object> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=76&c=3
TNode<Union<Null, Weak<HeapObject>>> LoadWasmFastApiCallDataCachedMap_0(compiler::CodeAssemblerState* state_, TNode<WasmFastApiCallData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=76&c=3
void StoreWasmFastApiCallDataCachedMap_0(compiler::CodeAssemblerState* state_, TNode<WasmFastApiCallData> p_o, TNode<Union<Null, Weak<HeapObject>>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=88&c=3
TNode<MaybeObject> LoadWasmInternalFunctionProtectedImplicitArg_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=88&c=3
void StoreWasmInternalFunctionProtectedImplicitArg_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
TNode<Union<JSFunction, Undefined>> LoadWasmInternalFunctionExternal_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=91&c=3
void StoreWasmInternalFunctionExternal_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o, TNode<Union<JSFunction, Undefined>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=98&c=3
TNode<Smi> LoadWasmInternalFunctionFunctionIndex_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=98&c=3
void StoreWasmInternalFunctionFunctionIndex_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=100&c=3
TNode<Uint32T> LoadWasmInternalFunctionRawCallTarget_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=100&c=3
void StoreWasmInternalFunctionRawCallTarget_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=103&c=3
TNode<RawPtrT> LoadWasmInternalFunctionSig_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=103&c=3
void StoreWasmInternalFunctionSig_0(compiler::CodeAssemblerState* state_, TNode<WasmInternalFunction> p_o, TNode<RawPtrT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=113&c=3
TNode<TrustedPointerT> LoadWasmFuncRefTrustedInternal_0(compiler::CodeAssemblerState* state_, TNode<WasmFuncRef> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=113&c=3
void StoreWasmFuncRefTrustedInternal_0(compiler::CodeAssemblerState* state_, TNode<WasmFuncRef> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=122&c=3
TNode<TrustedPointerT> LoadWasmFunctionDataWrapperCode_0(compiler::CodeAssemblerState* state_, TNode<WasmFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=122&c=3
void StoreWasmFunctionDataWrapperCode_0(compiler::CodeAssemblerState* state_, TNode<WasmFunctionData> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
TNode<WasmFuncRef> LoadWasmFunctionDataFuncRef_0(compiler::CodeAssemblerState* state_, TNode<WasmFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=125&c=3
void StoreWasmFunctionDataFuncRef_0(compiler::CodeAssemblerState* state_, TNode<WasmFunctionData> p_o, TNode<WasmFuncRef> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=127&c=3
TNode<Smi> LoadWasmFunctionDataJsPromiseFlags_0(compiler::CodeAssemblerState* state_, TNode<WasmFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=127&c=3
void StoreWasmFunctionDataJsPromiseFlags_0(compiler::CodeAssemblerState* state_, TNode<WasmFunctionData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=130&c=3
TNode<MaybeObject> LoadWasmFunctionDataProtectedInternal_0(compiler::CodeAssemblerState* state_, TNode<WasmFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=130&c=3
void StoreWasmFunctionDataProtectedInternal_0(compiler::CodeAssemblerState* state_, TNode<WasmFunctionData> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=140&c=3
TNode<MaybeObject> LoadWasmExportedFunctionDataProtectedInstanceData_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=140&c=3
void StoreWasmExportedFunctionDataProtectedInstanceData_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=141&c=3
TNode<Smi> LoadWasmExportedFunctionDataFunctionIndex_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=141&c=3
void StoreWasmExportedFunctionDataFunctionIndex_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
TNode<Cell> LoadWasmExportedFunctionDataWrapperBudget_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=143&c=3
void StoreWasmExportedFunctionDataWrapperBudget_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o, TNode<Cell> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=148&c=3
TNode<Smi> LoadWasmExportedFunctionDataPackedArgsSize_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=148&c=3
void StoreWasmExportedFunctionDataPackedArgsSize_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=149&c=3
TNode<TrustedPointerT> LoadWasmExportedFunctionDataCWrapperCode_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=149&c=3
void StoreWasmExportedFunctionDataCWrapperCode_0(compiler::CodeAssemblerState* state_, TNode<WasmExportedFunctionData> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=157&c=3
TNode<MaybeObject> LoadWasmJSFunctionDataProtectedOffheapData_0(compiler::CodeAssemblerState* state_, TNode<WasmJSFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=157&c=3
void StoreWasmJSFunctionDataProtectedOffheapData_0(compiler::CodeAssemblerState* state_, TNode<WasmJSFunctionData> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
TNode<Foreign> LoadWasmCapiFunctionDataEmbedderData_0(compiler::CodeAssemblerState* state_, TNode<WasmCapiFunctionData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=162&c=3
void StoreWasmCapiFunctionDataEmbedderData_0(compiler::CodeAssemblerState* state_, TNode<WasmCapiFunctionData> p_o, TNode<Foreign> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=166&c=3
TNode<TrustedPointerT> LoadWasmResumeDataTrustedSuspender_0(compiler::CodeAssemblerState* state_, TNode<WasmResumeData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=166&c=3
void StoreWasmResumeDataTrustedSuspender_0(compiler::CodeAssemblerState* state_, TNode<WasmResumeData> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=167&c=3
TNode<Smi> LoadWasmResumeDataOnResume_0(compiler::CodeAssemblerState* state_, TNode<WasmResumeData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=167&c=3
void StoreWasmResumeDataOnResume_0(compiler::CodeAssemblerState* state_, TNode<WasmResumeData> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=171&c=3
TNode<ExternalPointerT> LoadWasmSuspenderObjectStack_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=171&c=3
void StoreWasmSuspenderObjectStack_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o, TNode<ExternalPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=172&c=3
TNode<MaybeObject> LoadWasmSuspenderObjectParent_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=172&c=3
void StoreWasmSuspenderObjectParent_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o, TNode<MaybeObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
TNode<Union<JSPromise, Undefined>> LoadWasmSuspenderObjectPromise_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=173&c=3
void StoreWasmSuspenderObjectPromise_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o, TNode<Union<JSPromise, Undefined>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
TNode<Union<JSObject, Undefined>> LoadWasmSuspenderObjectResume_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=174&c=3
void StoreWasmSuspenderObjectResume_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o, TNode<Union<JSObject, Undefined>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
TNode<Union<JSObject, Undefined>> LoadWasmSuspenderObjectReject_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=175&c=3
void StoreWasmSuspenderObjectReject_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspenderObject> p_o, TNode<Union<JSObject, Undefined>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
TNode<WasmStackObject> LoadWasmContinuationObjectStackObj_0(compiler::CodeAssemblerState* state_, TNode<WasmContinuationObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=179&c=3
void StoreWasmContinuationObjectStackObj_0(compiler::CodeAssemblerState* state_, TNode<WasmContinuationObject> p_o, TNode<WasmStackObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=185&c=3
TNode<ExternalPointerT> LoadWasmStackObjectStack_0(compiler::CodeAssemblerState* state_, TNode<WasmStackObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=185&c=3
void StoreWasmStackObjectStack_0(compiler::CodeAssemblerState* state_, TNode<WasmStackObject> p_o, TNode<ExternalPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=192&c=3
TNode<Smi> LoadWasmExceptionTagIndex_0(compiler::CodeAssemblerState* state_, TNode<WasmExceptionTag> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=192&c=3
void StoreWasmExceptionTagIndex_0(compiler::CodeAssemblerState* state_, TNode<WasmExceptionTag> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
TNode<Foreign> LoadWasmModuleObjectManagedNativeModule_0(compiler::CodeAssemblerState* state_, TNode<WasmModuleObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=198&c=3
void StoreWasmModuleObjectManagedNativeModule_0(compiler::CodeAssemblerState* state_, TNode<WasmModuleObject> p_o, TNode<Foreign> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
TNode<Script> LoadWasmModuleObjectScript_0(compiler::CodeAssemblerState* state_, TNode<WasmModuleObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=199&c=3
void StoreWasmModuleObjectScript_0(compiler::CodeAssemblerState* state_, TNode<WasmModuleObject> p_o, TNode<Script> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
TNode<FixedArray> LoadWasmTableObjectEntries_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=214&c=3
void StoreWasmTableObjectEntries_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<FixedArray> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=215&c=3
TNode<Smi> LoadWasmTableObjectCurrentLength_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=215&c=3
void StoreWasmTableObjectCurrentLength_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
TNode<Union<BigInt, HeapNumber, Smi, Undefined>> LoadWasmTableObjectMaximumLength_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=220&c=3
void StoreWasmTableObjectMaximumLength_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<Union<BigInt, HeapNumber, Smi, Undefined>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=224&c=3
TNode<Smi> LoadWasmTableObjectRawType_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=224&c=3
void StoreWasmTableObjectRawType_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=226&c=3
TNode<TrustedPointerT> LoadWasmTableObjectTrustedDispatchTable_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=226&c=3
void StoreWasmTableObjectTrustedDispatchTable_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=230&c=3
TNode<TrustedPointerT> LoadWasmTableObjectTrustedData_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=230&c=3
void StoreWasmTableObjectTrustedData_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=231&c=3
TNode<Uint8T> LoadWasmTableObjectAddressType_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=231&c=3
void StoreWasmTableObjectAddressType_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<Uint8T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=233&c=3
TNode<Uint8T> LoadWasmTableObjectPaddingForAddressType0_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=233&c=3
void StoreWasmTableObjectPaddingForAddressType0_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<Uint8T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=234&c=3
TNode<Uint16T> LoadWasmTableObjectPaddingForAddressType1_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=234&c=3
void StoreWasmTableObjectPaddingForAddressType1_0(compiler::CodeAssemblerState* state_, TNode<WasmTableObject> p_o, TNode<Uint16T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
TNode<Union<JSArrayBuffer, Undefined>> LoadWasmMemoryObjectArrayBuffer_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=241&c=3
void StoreWasmMemoryObjectArrayBuffer_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o, TNode<Union<JSArrayBuffer, Undefined>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
TNode<Foreign> LoadWasmMemoryObjectManagedBackingStore_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=246&c=3
void StoreWasmMemoryObjectManagedBackingStore_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o, TNode<Foreign> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=248&c=3
TNode<Smi> LoadWasmMemoryObjectMaximumPages_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=248&c=3
void StoreWasmMemoryObjectMaximumPages_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
TNode<WeakArrayList> LoadWasmMemoryObjectInstances_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=249&c=3
void StoreWasmMemoryObjectInstances_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o, TNode<WeakArrayList> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=250&c=3
TNode<Uint8T> LoadWasmMemoryObjectAddressType_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=250&c=3
void StoreWasmMemoryObjectAddressType_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o, TNode<Uint8T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=252&c=3
TNode<Uint8T> LoadWasmMemoryObjectPaddingForFlags0_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=252&c=3
void StoreWasmMemoryObjectPaddingForFlags0_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o, TNode<Uint8T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=253&c=3
TNode<Uint16T> LoadWasmMemoryObjectPaddingForFlags1_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=253&c=3
void StoreWasmMemoryObjectPaddingForFlags1_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryObject> p_o, TNode<Uint16T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
TNode<Weak<HeapObject>> LoadWasmMemoryMapDescriptorMemory_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryMapDescriptor> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=262&c=3
void StoreWasmMemoryMapDescriptorMemory_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryMapDescriptor> p_o, TNode<Weak<HeapObject>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=263&c=3
TNode<Int32T> LoadWasmMemoryMapDescriptorFileDescriptor_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryMapDescriptor> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=263&c=3
void StoreWasmMemoryMapDescriptorFileDescriptor_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryMapDescriptor> p_o, TNode<Int32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=265&c=3
TNode<Uint32T> LoadWasmMemoryMapDescriptorOffset_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryMapDescriptor> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=265&c=3
void StoreWasmMemoryMapDescriptorOffset_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryMapDescriptor> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=266&c=3
TNode<Uint32T> LoadWasmMemoryMapDescriptorSize_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryMapDescriptor> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=266&c=3
void StoreWasmMemoryMapDescriptorSize_0(compiler::CodeAssemblerState* state_, TNode<WasmMemoryMapDescriptor> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=274&c=3
TNode<TrustedPointerT> LoadWasmGlobalObjectTrustedData_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=274&c=3
void StoreWasmGlobalObjectTrustedData_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
TNode<Union<ByteArray, FixedArray>> LoadWasmGlobalObjectBuffer_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=279&c=3
void StoreWasmGlobalObjectBuffer_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o, TNode<Union<ByteArray, FixedArray>> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=283&c=3
TNode<Smi> LoadWasmGlobalObjectOffset_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=283&c=3
void StoreWasmGlobalObjectOffset_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=284&c=3
TNode<Smi> LoadWasmGlobalObjectRawType_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=284&c=3
void StoreWasmGlobalObjectRawType_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=287&c=3
TNode<Smi> LoadWasmGlobalObjectIsMutable_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=287&c=3
void StoreWasmGlobalObjectIsMutable_0(compiler::CodeAssemblerState* state_, TNode<WasmGlobalObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
TNode<HeapObject> LoadWasmTagObjectTag_0(compiler::CodeAssemblerState* state_, TNode<WasmTagObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=291&c=3
void StoreWasmTagObjectTag_0(compiler::CodeAssemblerState* state_, TNode<WasmTagObject> p_o, TNode<HeapObject> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=292&c=3
TNode<Smi> LoadWasmTagObjectCanonicalTypeIndex_0(compiler::CodeAssemblerState* state_, TNode<WasmTagObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=292&c=3
void StoreWasmTagObjectCanonicalTypeIndex_0(compiler::CodeAssemblerState* state_, TNode<WasmTagObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=293&c=3
TNode<TrustedPointerT> LoadWasmTagObjectTrustedData_0(compiler::CodeAssemblerState* state_, TNode<WasmTagObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=293&c=3
void StoreWasmTagObjectTrustedData_0(compiler::CodeAssemblerState* state_, TNode<WasmTagObject> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
TNode<Foreign> LoadAsmWasmDataManagedNativeModule_0(compiler::CodeAssemblerState* state_, TNode<AsmWasmData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=299&c=3
void StoreAsmWasmDataManagedNativeModule_0(compiler::CodeAssemblerState* state_, TNode<AsmWasmData> p_o, TNode<Foreign> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
TNode<HeapNumber> LoadAsmWasmDataUsesBitset_0(compiler::CodeAssemblerState* state_, TNode<AsmWasmData> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=300&c=3
void StoreAsmWasmDataUsesBitset_0(compiler::CodeAssemblerState* state_, TNode<AsmWasmData> p_o, TNode<HeapNumber> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=304&c=3
TNode<Uint32T> LoadWasmTypeInfoCanonicalType_0(compiler::CodeAssemblerState* state_, TNode<WasmTypeInfo> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=304&c=3
void StoreWasmTypeInfoCanonicalType_0(compiler::CodeAssemblerState* state_, TNode<WasmTypeInfo> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=305&c=3
TNode<Uint32T> LoadWasmTypeInfoCanonicalElementType_0(compiler::CodeAssemblerState* state_, TNode<WasmTypeInfo> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=305&c=3
void StoreWasmTypeInfoCanonicalElementType_0(compiler::CodeAssemblerState* state_, TNode<WasmTypeInfo> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=306&c=9
TNode<Smi> LoadWasmTypeInfoSupertypesLength_0(compiler::CodeAssemblerState* state_, TNode<WasmTypeInfo> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
TorqueStructSlice_Object_MutableReference_Object_0 FieldSliceWasmTypeInfoSupertypes_0(compiler::CodeAssemblerState* state_, TNode<WasmTypeInfo> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
TNode<Object> LoadWasmTypeInfoSupertypes_0(compiler::CodeAssemblerState* state_, TNode<WasmTypeInfo> p_o, TNode<IntPtrT> p_i);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
void StoreWasmTypeInfoSupertypes_0(compiler::CodeAssemblerState* state_, TNode<WasmTypeInfo> p_o, TNode<IntPtrT> p_i, TNode<Object> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=319&c=3
TNode<Uint32T> LoadWasmArrayLength_0(compiler::CodeAssemblerState* state_, TNode<WasmArray> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=319&c=3
void StoreWasmArrayLength_0(compiler::CodeAssemblerState* state_, TNode<WasmArray> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=326&c=3
TNode<String> LoadWasmStringViewIterString_0(compiler::CodeAssemblerState* state_, TNode<WasmStringViewIter> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=326&c=3
void StoreWasmStringViewIterString_0(compiler::CodeAssemblerState* state_, TNode<WasmStringViewIter> p_o, TNode<String> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=327&c=3
TNode<Uint32T> LoadWasmStringViewIterOffset_0(compiler::CodeAssemblerState* state_, TNode<WasmStringViewIter> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=327&c=3
void StoreWasmStringViewIterOffset_0(compiler::CodeAssemblerState* state_, TNode<WasmStringViewIter> p_o, TNode<Uint32T> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
TNode<JSReceiver> LoadWasmSuspendingObjectCallable_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspendingObject> p_o);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=339&c=3
void StoreWasmSuspendingObjectCallable_0(compiler::CodeAssemblerState* state_, TNode<WasmSuspendingObject> p_o, TNode<JSReceiver> p_v);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=33&c=27
TNode<Uint32T> Convert_uint32_WasmCodePointer_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_i);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=37&c=1
TNode<WasmTrustedInstanceData> DownCastForTorqueClass_WasmTrustedInstanceData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=39&c=1
TNode<WasmInstanceObject> DownCastForTorqueClass_WasmInstanceObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=49&c=1
TNode<WasmImportData> DownCastForTorqueClass_WasmImportData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=73&c=1
TNode<WasmFastApiCallData> DownCastForTorqueClass_WasmFastApiCallData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=81&c=1
TNode<WasmInternalFunction> DownCastForTorqueClass_WasmInternalFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=111&c=1
TNode<WasmFuncRef> DownCastForTorqueClass_WasmFuncRef_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=120&c=1
TNode<WasmFunctionData> DownCastForTorqueClass_WasmFunctionData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=136&c=1
TNode<WasmExportedFunctionData> DownCastForTorqueClass_WasmExportedFunctionData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=156&c=1
TNode<WasmJSFunctionData> DownCastForTorqueClass_WasmJSFunctionData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=161&c=1
TNode<WasmCapiFunctionData> DownCastForTorqueClass_WasmCapiFunctionData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=165&c=1
TNode<WasmResumeData> DownCastForTorqueClass_WasmResumeData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=170&c=1
TNode<WasmSuspenderObject> DownCastForTorqueClass_WasmSuspenderObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=178&c=1
TNode<WasmContinuationObject> DownCastForTorqueClass_WasmContinuationObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=184&c=1
TNode<WasmStackObject> DownCastForTorqueClass_WasmStackObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=188&c=1
TNode<WasmExceptionTag> DownCastForTorqueClass_WasmExceptionTag_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=195&c=1
TNode<WasmExceptionPackage> DownCastForTorqueClass_WasmExceptionPackage_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=197&c=1
TNode<WasmModuleObject> DownCastForTorqueClass_WasmModuleObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=202&c=1
TNode<WasmDispatchTable> DownCastForTorqueClass_WasmDispatchTable_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=203&c=1
TNode<WasmDispatchTableForImports> DownCastForTorqueClass_WasmDispatchTableForImports_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=205&c=1
TNode<WasmTableObject> DownCastForTorqueClass_WasmTableObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=238&c=1
TNode<WasmMemoryObject> DownCastForTorqueClass_WasmMemoryObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=257&c=1
TNode<WasmMemoryMapDescriptor> DownCastForTorqueClass_WasmMemoryMapDescriptor_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=270&c=1
TNode<WasmGlobalObject> DownCastForTorqueClass_WasmGlobalObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=290&c=1
TNode<WasmTagObject> DownCastForTorqueClass_WasmTagObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=298&c=1
TNode<AsmWasmData> DownCastForTorqueClass_AsmWasmData_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=303&c=1
TNode<WasmTypeInfo> DownCastForTorqueClass_WasmTypeInfo_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=311&c=1
TNode<WasmObject> DownCastForTorqueClass_WasmObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=314&c=1
TNode<WasmStruct> DownCastForTorqueClass_WasmStruct_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=317&c=1
TNode<WasmArray> DownCastForTorqueClass_WasmArray_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=325&c=1
TNode<WasmStringViewIter> DownCastForTorqueClass_WasmStringViewIter_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=333&c=1
TNode<WasmNull> DownCastForTorqueClass_WasmNull_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=338&c=1
TNode<WasmSuspendingObject> DownCastForTorqueClass_WasmSuspendingObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/wasm/wasm-objects.tq?l=307&c=3
TorqueStructSlice_Object_MutableReference_Object_0 NewMutableSlice_Object_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, TaggedIndex>> p_object, TNode<IntPtrT> p_offset, TNode<IntPtrT> p_length);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_WASM_WASM_OBJECTS_TQ_CSA_H_
