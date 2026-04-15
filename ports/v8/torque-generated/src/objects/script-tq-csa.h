#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_SCRIPT_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_SCRIPT_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/script.tq?l=19&c=1
TNode<Script> Cast_Script_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/script.tq?l=22&c=3
TNode<Union<String, Undefined>> LoadScriptSource_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=22&c=3
void StoreScriptSource_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<String, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=25&c=3
TNode<Object> LoadScriptName_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=25&c=3
void StoreScriptName_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Object> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=28&c=3
TNode<Smi> LoadScriptLineOffset_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=28&c=3
void StoreScriptLineOffset_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=32&c=3
TNode<Smi> LoadScriptColumnOffset_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=32&c=3
void StoreScriptColumnOffset_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=35&c=3
TNode<Union<Smi, Symbol, Undefined>> LoadScriptContextData_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=35&c=3
void StoreScriptContextData_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<Smi, Symbol, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=37&c=3
TNode<Smi> LoadScriptScriptType_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=37&c=3
void StoreScriptScriptType_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=40&c=3
TNode<Union<FixedArray, Smi>> LoadScriptLineEnds_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=40&c=3
void StoreScriptLineEnds_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<FixedArray, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=43&c=3
TNode<Smi> LoadScriptId_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=43&c=3
void StoreScriptId_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=48&c=3
TNode<Union<FixedArray, SharedFunctionInfo, Undefined>> LoadScriptEvalFromSharedOrWrappedArguments_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=48&c=3
void StoreScriptEvalFromSharedOrWrappedArguments_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<FixedArray, SharedFunctionInfo, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=50&c=3
TNode<Union<Foreign, Smi>> LoadScriptEvalFromPosition_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=50&c=3
void StoreScriptEvalFromPosition_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<Foreign, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=51&c=3
TNode<Union<ScopeInfo, Undefined>> LoadScriptEvalFromScopeInfo_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=51&c=3
void StoreScriptEvalFromScopeInfo_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<ScopeInfo, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=52&c=3
TNode<Union<WeakArrayList, WeakFixedArray>> LoadScriptInfos_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=52&c=3
void StoreScriptInfos_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<WeakArrayList, WeakFixedArray>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=56&c=3
TNode<Union<ArrayList, Undefined>> LoadScriptCompiledLazyFunctionPositions_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=56&c=3
void StoreScriptCompiledLazyFunctionPositions_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<ArrayList, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=59&c=3
TNode<Smi> LoadScriptFlags_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=59&c=3
void StoreScriptFlags_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=62&c=3
TNode<Union<String, Undefined>> LoadScriptSourceUrl_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=62&c=3
void StoreScriptSourceUrl_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<String, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=65&c=3
TNode<Object> LoadScriptSourceMappingUrl_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=65&c=3
void StoreScriptSourceMappingUrl_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Object> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=68&c=3
TNode<Union<String, Undefined>> LoadScriptDebugId_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=68&c=3
void StoreScriptDebugId_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<String, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=71&c=3
TNode<FixedArray> LoadScriptHostDefinedOptions_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=71&c=3
void StoreScriptHostDefinedOptions_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<FixedArray> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=80&c=3
TNode<Union<String, Undefined>> LoadScriptSourceHash_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o);

// https://crsrc.org/c/v8/src/objects/script.tq?l=80&c=3
void StoreScriptSourceHash_0(compiler::CodeAssemblerState* state_, TNode<Script> p_o, TNode<Union<String, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/script.tq?l=19&c=1
TNode<Script> DownCastForTorqueClass_Script_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_SCRIPT_TQ_CSA_H_
