#ifndef V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_BUILTINS_BIGINT_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_BUILTINS_BIGINT_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=9&c=1
TNode<Uint32T> kPositiveSign_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=10&c=1
TNode<Uint32T> kNegativeSign_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=11&c=1
TNode<IntPtrT> kGreaterThan_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=12&c=1
TNode<IntPtrT> kLessThan_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=14&c=1
TNode<Uint32T> kMustRoundDownBitShift_0(compiler::CodeAssemblerState* state_);
// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=62&c=1
TNode<BoolT> IsCanonicalized_0(compiler::CodeAssemblerState* state_, TNode<BigInt> p_bigint);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=72&c=1
TNode<Uint32T> InvertSign_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_sign);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=76&c=1
TNode<BigInt> AllocateEmptyBigIntNoThrow_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Uint32T> p_sign, TNode<IntPtrT> p_length, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=88&c=1
TNode<BigInt> AllocateEmptyBigInt_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Uint32T> p_sign, TNode<IntPtrT> p_length);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=99&c=1
TNode<IntPtrT> BigIntCompareAbsolute_0(compiler::CodeAssemblerState* state_, TNode<BigInt> p_x, TNode<BigInt> p_y);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=118&c=1
TNode<BigInt> MutableBigIntAbsoluteSub_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, TNode<Uint32T> p_resultSign);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=140&c=1
TNode<BigInt> MutableBigIntAbsoluteAdd_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_xBigint, TNode<BigInt> p_yBigint, TNode<Uint32T> p_resultSign, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=175&c=1
TNode<BigInt> BigIntAddImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=217&c=1
TNode<BigInt> BigIntSubtractImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=259&c=1
TNode<BigInt> BigIntMultiplyImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntTooBig, compiler::CodeAssemblerLabel* label_TerminationRequested);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=323&c=1
TNode<BigInt> BigIntDivideImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntDivZero, compiler::CodeAssemblerLabel* label_TerminationRequested);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=390&c=1
TNode<BigInt> BigIntModulusImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntDivZero, compiler::CodeAssemblerLabel* label_TerminationRequested);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=454&c=1
TNode<BigInt> BigIntBitwiseAndImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=521&c=1
TNode<BigInt> BigIntBitwiseOrImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=580&c=1
TNode<BigInt> BigIntBitwiseXorImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=649&c=1
TNode<BigInt> MutableBigIntLeftShiftByAbsolute_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=689&c=1
TNode<BigInt> RightShiftByMaximum_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Uint32T> p_sign);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=699&c=1
TNode<BigInt> MutableBigIntRightShiftByAbsolute_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=740&c=1
TNode<BigInt> BigIntShiftLeftImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=749&c=1
TNode<BigInt> BigIntShiftRightImpl_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y, compiler::CodeAssemblerLabel* label_BigIntTooBig);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=828&c=1
TNode<IntPtrT> BigIntCompare_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BigInt> p_x, TNode<BigInt> p_y);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=199&c=12
TNode<Smi> Convert_Smi_constexpr_IntegerLiteral_0(compiler::CodeAssemblerState* state_, IntegerLiteral p_i);

// https://crsrc.org/c/v8/src/builtins/builtins-bigint.tq?l=206&c=15
TNode<BigInt> Cast_BigInt_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_BUILTINS_BUILTINS_BIGINT_TQ_CSA_H_
