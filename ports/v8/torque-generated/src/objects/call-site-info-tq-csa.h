#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_CALL_SITE_INFO_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_CALL_SITE_INFO_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=24&c=1
TNode<CallSiteInfo> Cast_CallSiteInfo_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=28&c=3
TNode<TrustedPointerT> LoadCallSiteInfoCodeObject_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=28&c=3
void StoreCallSiteInfoCodeObject_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o, TNode<TrustedPointerT> p_v);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=29&c=3
TNode<JSAny> LoadCallSiteInfoReceiverOrInstance_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=29&c=3
void StoreCallSiteInfoReceiverOrInstance_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=30&c=3
TNode<Union<JSFunction, Smi>> LoadCallSiteInfoFunction_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=30&c=3
void StoreCallSiteInfoFunction_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o, TNode<Union<JSFunction, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=31&c=3
TNode<Smi> LoadCallSiteInfoCodeOffsetOrSourcePosition_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=31&c=3
void StoreCallSiteInfoCodeOffsetOrSourcePosition_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=32&c=3
TNode<Smi> LoadCallSiteInfoFlags_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=32&c=3
void StoreCallSiteInfoFlags_0(compiler::CodeAssemblerState* state_, TNode<CallSiteInfo> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/call-site-info.tq?l=24&c=1
TNode<CallSiteInfo> DownCastForTorqueClass_CallSiteInfo_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_CALL_SITE_INFO_TQ_CSA_H_
