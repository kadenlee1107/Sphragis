#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_CONVERSION_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_CONVERSION_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/conversion.tq?l=183&c=1
TNode<JSAny> TryGetExoticToPrimitive_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_input, compiler::CodeAssemblerLabel* label_OrdinaryToPrimitive);

// https://crsrc.org/c/v8/src/builtins/conversion.tq?l=194&c=1
TNode<JSPrimitive> CallExoticToPrimitive_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_input, TNode<JSAny> p_exoticToPrimitive, TNode<String> p_hint);

// https://crsrc.org/c/v8/src/builtins/conversion.tq?l=247&c=1
TNode<JSPrimitive> NonPrimitiveToPrimitive_String_Inline_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_input);

// https://crsrc.org/c/v8/src/builtins/conversion.tq?l=278&c=1
TNode<JSPrimitive> TryToPrimitiveMethod_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_input, TNode<String> p_name, compiler::CodeAssemblerLabel* label_Continue);

// https://crsrc.org/c/v8/src/builtins/conversion.tq?l=316&c=1
TNode<JSPrimitive> OrdinaryToPrimitive_String_Inline_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_input);

// https://crsrc.org/c/v8/src/builtins/conversion.tq?l=125&c=7
TNode<Name> Cast_Name_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/builtins/conversion.tq?l=172&c=22
TNode<Map> UnsafeCast_Map_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_CONVERSION_TQ_CSA_H_
