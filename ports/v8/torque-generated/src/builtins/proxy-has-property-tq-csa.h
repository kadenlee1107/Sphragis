#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROXY_HAS_PROPERTY_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROXY_HAS_PROPERTY_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/proxy-has-property.tq?l=13&c=10
TNode<BoolT> Is_JSProxy_JSProxy_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSProxy> p_o);

// https://crsrc.org/c/v8/src/builtins/proxy-has-property.tq?l=18&c=10
TNode<BoolT> Is_Name_PropertyKey_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<String, Symbol>> p_o);

// https://crsrc.org/c/v8/src/builtins/proxy-has-property.tq?l=25&c=37
TNode<BoolT> Is_JSReceiver_JSReceiver_OR_Null_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSReceiver, Null>> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_PROXY_HAS_PROPERTY_TQ_CSA_H_
