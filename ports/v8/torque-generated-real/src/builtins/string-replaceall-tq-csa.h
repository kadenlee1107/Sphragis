#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_STRING_REPLACEALL_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_STRING_REPLACEALL_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/string-replaceall.tq?l=12&c=1
void ThrowIfNotGlobal_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_searchValue);

// https://crsrc.org/c/v8/src/builtins/string-replaceall.tq?l=16&c=5
TNode<JSRegExp> Cast_FastJSRegExp_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/string-replaceall.tq?l=42&c=7
TNode<BoolT> Is_JSReceiver_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_o);

// https://crsrc.org/c/v8/src/builtins/string-replaceall.tq?l=115&c=34
TNode<String> UnsafeCast_String_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_STRING_REPLACEALL_TQ_CSA_H_
