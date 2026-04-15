#include "src/ast/ast.h"
#include "src/builtins/builtins-array-gen.h"
#include "src/builtins/builtins-bigint-gen.h"
#include "src/builtins/builtins-call-gen.h"
#include "src/builtins/builtins-collections-gen.h"
#include "src/builtins/builtins-constructor-gen.h"
#include "src/builtins/builtins-data-view-gen.h"
#include "src/builtins/builtins-iterator-gen.h"
#include "src/builtins/builtins-object-gen.h"
#include "src/builtins/builtins-promise-gen.h"
#include "src/builtins/builtins-promise.h"
#include "src/builtins/builtins-proxy-gen.h"
#include "src/builtins/builtins-regexp-gen.h"
#include "src/builtins/builtins-string-gen.h"
#include "src/builtins/builtins-string-gen.h"
#include "src/builtins/builtins-typed-array-gen.h"
#include "src/builtins/builtins-utils-gen.h"
#include "src/builtins/builtins-wasm-gen.h"
#include "src/builtins/builtins.h"
#include "src/codegen/code-factory.h"
#include "src/debug/debug-wasm-objects.h"
#include "src/heap/factory-inl.h"
#include "src/ic/binary-op-assembler.h"
#include "src/ic/handler-configuration-inl.h"
#include "src/objects/arguments.h"
#include "src/objects/bigint.h"
#include "src/objects/call-site-info.h"
#include "src/objects/elements-kind.h"
#include "src/objects/free-space.h"
#include "src/objects/intl-objects.h"
#include "src/objects/js-atomics-synchronization.h"
#include "src/objects/js-break-iterator.h"
#include "src/objects/js-collator.h"
#include "src/objects/js-date-time-format.h"
#include "src/objects/js-display-names.h"
#include "src/objects/js-disposable-stack.h"
#include "src/objects/js-duration-format.h"
#include "src/objects/js-function.h"
#include "src/objects/js-generator.h"
#include "src/objects/js-iterator-helpers.h"
#include "src/objects/js-list-format.h"
#include "src/objects/js-locale.h"
#include "src/objects/js-number-format.h"
#include "src/objects/js-objects.h"
#include "src/objects/js-plural-rules.h"
#include "src/objects/js-promise.h"
#include "src/objects/js-raw-json.h"
#include "src/objects/js-regexp-string-iterator.h"
#include "src/objects/js-relative-time-format.h"
#include "src/objects/js-segment-iterator-inl.h"
#include "src/objects/js-segmenter.h"
#include "src/objects/js-segments.h"
#include "src/objects/js-shadow-realm.h"
#include "src/objects/js-shared-array.h"
#include "src/objects/js-struct.h"
#include "src/objects/js-temporal-objects.h"
#include "src/objects/js-weak-refs.h"
#include "src/objects/objects.h"
#include "src/objects/ordered-hash-table.h"
#include "src/objects/property-array.h"
#include "src/objects/property-descriptor-object.h"
#include "src/objects/source-text-module.h"
#include "src/objects/swiss-hash-table-helpers.h"
#include "src/objects/swiss-name-dictionary.h"
#include "src/objects/synthetic-module.h"
#include "src/objects/template-objects.h"
#include "src/objects/torque-defined-classes.h"
#include "src/objects/turbofan-types.h"
#include "src/objects/turboshaft-types.h"
#include "src/torque/runtime-support.h"
#include "src/wasm/value-type.h"
#include "src/wasm/wasm-linkage.h"
#include "src/wasm/wasm-module.h"
#include "src/codegen/code-stub-assembler-inl.h"
// Required Builtins:
#include "torque-generated/src/builtins/js-to-wasm-tq-csa.h"
#include "torque-generated/src/wasm/wasm-objects-tq-csa.h"
#include "torque-generated/src/objects/arguments-tq-csa.h"
#include "torque-generated/src/objects/cell-tq-csa.h"
#include "torque-generated/src/objects/js-function-tq-csa.h"
#include "torque-generated/src/objects/js-array-tq-csa.h"
#include "torque-generated/src/objects/fixed-array-tq-csa.h"
#include "torque-generated/src/objects/intl-objects-tq-csa.h"
#include "torque-generated/src/objects/ordered-hash-table-tq-csa.h"
#include "torque-generated/src/objects/contexts-tq-csa.h"
#include "torque-generated/src/builtins/promise-resolve-tq-csa.h"
#include "torque-generated/src/builtins/convert-tq-csa.h"
#include "torque-generated/src/builtins/frame-arguments-tq-csa.h"
#include "torque-generated/src/builtins/string-replaceall-tq-csa.h"
#include "torque-generated/src/builtins/regexp-replace-tq-csa.h"
#include "torque-generated/src/builtins/builtins-bigint-tq-csa.h"
#include "torque-generated/src/builtins/torque-internal-tq-csa.h"
#include "torque-generated/src/builtins/cast-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"
#include "torque-generated/src/builtins/wasm-to-js-tq-csa.h"
#include "torque-generated/src/builtins/js-to-wasm-tq-csa.h"
#include "torque-generated/src/builtins/wasm-tq-csa.h"
#include "torque-generated/src/builtins/frames-tq-csa.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=68&c=1
TNode<Int32T> FromConstexpr_Promise_constexpr_kPromise_0(compiler::CodeAssemblerState* state_, wasm::Promise p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Int32T> tmp0;
  TNode<Int32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Int32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Int32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Int32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=68&c=1
TNode<Int32T> FromConstexpr_Promise_constexpr_kNoPromise_0(compiler::CodeAssemblerState* state_, wasm::Promise p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Int32T> tmp0;
  TNode<Int32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Int32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Int32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Int32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=68&c=1
TNode<Int32T> FromConstexpr_Promise_constexpr_kStressSwitch_0(compiler::CodeAssemblerState* state_, wasm::Promise p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Int32T> tmp0;
  TNode<Int32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Int32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Int32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Int32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=74&c=1
TNode<Uint32T> FromConstexpr_StandardType_constexpr_kExtern_0(compiler::CodeAssemblerState* state_, wasm::StandardType p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=74&c=1
TNode<Uint32T> FromConstexpr_StandardType_constexpr_kNoExtern_0(compiler::CodeAssemblerState* state_, wasm::StandardType p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=74&c=1
TNode<Uint32T> FromConstexpr_StandardType_constexpr_kString_0(compiler::CodeAssemblerState* state_, wasm::StandardType p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=74&c=1
TNode<Uint32T> FromConstexpr_StandardType_constexpr_kEq_0(compiler::CodeAssemblerState* state_, wasm::StandardType p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=74&c=1
TNode<Uint32T> FromConstexpr_StandardType_constexpr_kI31_0(compiler::CodeAssemblerState* state_, wasm::StandardType p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=74&c=1
TNode<Uint32T> FromConstexpr_StandardType_constexpr_kAny_0(compiler::CodeAssemblerState* state_, wasm::StandardType p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=84&c=1
TNode<Uint32T> FromConstexpr_RefTypeKind_constexpr_kStruct_0(compiler::CodeAssemblerState* state_, wasm::RefTypeKind p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=84&c=1
TNode<Uint32T> FromConstexpr_RefTypeKind_constexpr_kArray_0(compiler::CodeAssemblerState* state_, wasm::RefTypeKind p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=84&c=1
TNode<Uint32T> FromConstexpr_RefTypeKind_constexpr_kFunction_0(compiler::CodeAssemblerState* state_, wasm::RefTypeKind p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ca_.Uint32Constant(CastToUnderlyingTypeIfEnum(p_o));
    tmp1 = (TNode<Uint32T>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=149&c=1
TNode<Uint32T> Bitcast_uint32_float32_0(compiler::CodeAssemblerState* state_, TNode<Float32T> p_v) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).BitcastFloat32ToInt32(TNode<Float32T>{p_v});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp0};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=158&c=1
TNode<IntPtrT> TruncateBigIntToI64_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_input) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BigInt> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<BoolT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).ToBigInt(TNode<Context>{p_context}, TNode<JSAny>{p_input});
    tmp1 = BigIntBuiltinsAssembler(state_).ReadBigIntLength(TNode<BigInt>{tmp0});
    tmp2 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp3 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp1}, TNode<IntPtrT>{tmp2});
    ca_.Branch(tmp3, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp4;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp4 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block1, tmp4);
  }

  TNode<IntPtrT> tmp5;
  TNode<UintPtrT> tmp6;
  TNode<Uint32T> tmp7;
  TNode<Uint32T> tmp8;
  TNode<BoolT> tmp9;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp5 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp6 = CodeStubAssembler(state_).LoadBigIntDigit(TNode<BigInt>{tmp0}, TNode<IntPtrT>{tmp5});
    tmp7 = BigIntBuiltinsAssembler(state_).ReadBigIntSign(TNode<BigInt>{tmp0});
    tmp8 = kPositiveSign_0(state_);
    tmp9 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp7}, TNode<Uint32T>{tmp8});
    ca_.Branch(tmp9, &block8, std::vector<compiler::Node*>{}, &block9, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp10;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    tmp10 = CodeStubAssembler(state_).Signed(TNode<UintPtrT>{tmp6});
    ca_.Goto(&block1, tmp10);
  }

  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp11 = CodeStubAssembler(state_).Signed(TNode<UintPtrT>{tmp6});
    tmp12 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp13 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp12}, TNode<IntPtrT>{tmp11});
    ca_.Goto(&block1, tmp13);
  }

  TNode<IntPtrT> phi_bb1_2;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_2);
    ca_.Goto(&block10, phi_bb1_2);
  }

  TNode<IntPtrT> phi_bb10_2;
    ca_.Bind(&block10, &phi_bb10_2);
  return TNode<IntPtrT>{phi_bb10_2};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=272&c=1
TorqueStructReturnSlotAllocator_0 NewReturnSlotAllocator_0(compiler::CodeAssemblerState* state_) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<BoolT> tmp3;
  TNode<BoolT> tmp4;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_intptr_0(state_, arraysize(wasm::kGpReturnRegisters));
    tmp1 = FromConstexpr_intptr_constexpr_intptr_0(state_, arraysize(wasm::kFpReturnRegisters));
    tmp2 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp3 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp4 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReturnSlotAllocator_0{TNode<IntPtrT>{tmp0}, TNode<IntPtrT>{tmp1}, TNode<BoolT>{tmp3}, TNode<BoolT>{tmp4}, TNode<IntPtrT>{tmp2}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=405&c=1
TorqueStructLocationAllocator_0 LocationAllocatorForParams_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_paramBuffer, TNode<IntPtrT> p_paramBufferSize) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).UniqueIntPtrConstant(arraysize(wasm::kGpParamRegisters) - 1);
    tmp1 = CodeStubAssembler(state_).UniqueIntPtrConstant(arraysize(wasm::kFpParamRegisters));
    tmp2 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp3 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp0}, TNode<IntPtrT>{tmp2});
    if (((CodeStubAssembler(state_).ConstexprBoolNot((CodeStubAssembler(state_).Is64()))))) {
      ca_.Goto(&block2);
    } else {
      ca_.Goto(&block3);
    }
  }

  TNode<IntPtrT> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp4 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp5 = CodeStubAssembler(state_).WordAnd(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp4});
    tmp6 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp5});
    ca_.Goto(&block4, tmp6);
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block4, tmp3);
  }

  TNode<IntPtrT> phi_bb4_7;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<BoolT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<BoolT> tmp14;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_7);
    tmp7 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb4_7}, TNode<IntPtrT>{p_paramBuffer.offset});
    tmp8 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp9 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp1}, TNode<IntPtrT>{tmp8});
    tmp10 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp7}, TNode<IntPtrT>{tmp9});
    tmp11 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp12 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp13 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp14 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{p_paramBufferSize}, TNode<IntPtrT>{tmp13});
    ca_.Branch(tmp14, &block9, std::vector<compiler::Node*>{}, &block10, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp15;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp15 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_paramBuffer.offset}, TNode<IntPtrT>{p_paramBufferSize});
    ca_.Goto(&block11, tmp15);
  }

  TNode<IntPtrT> tmp16;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp16 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block11, tmp16);
  }

  TNode<IntPtrT> phi_bb11_10;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_10);
    ca_.Goto(&block12);
  }

    ca_.Bind(&block12);
  return TorqueStructLocationAllocator_0{TNode<Union<HeapObject, TaggedIndex>>{p_paramBuffer.object}, TNode<IntPtrT>{tmp0}, TNode<IntPtrT>{tmp1}, TNode<IntPtrT>{p_paramBuffer.offset}, TNode<IntPtrT>{tmp7}, TNode<IntPtrT>{tmp10}, TNode<IntPtrT>{tmp10}, TNode<IntPtrT>{phi_bb11_10}, TNode<IntPtrT>{tmp11}, TNode<BoolT>{tmp12}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=432&c=1
TorqueStructLocationAllocator_0 LocationAllocatorForReturns_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_gpRegs, TNode<RawPtrT> p_fpRegs, TNode<RawPtrT> p_stack) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<TaggedIndex> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<BoolT> tmp13;
  TNode<IntPtrT> tmp14;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = kZeroBitPattern_0(state_);
    tmp1 = FromConstexpr_intptr_constexpr_intptr_0(state_, arraysize(wasm::kGpReturnRegisters));
    tmp2 = FromConstexpr_intptr_constexpr_intptr_0(state_, arraysize(wasm::kFpReturnRegisters));
    tmp3 = Convert_intptr_RawPtr_0(state_, TNode<RawPtrT>{p_gpRegs});
    tmp4 = FromConstexpr_intptr_constexpr_int31_0(state_, kHeapObjectTag);
    tmp5 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp4});
    tmp6 = Convert_intptr_RawPtr_0(state_, TNode<RawPtrT>{p_fpRegs});
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, kHeapObjectTag);
    tmp8 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp6}, TNode<IntPtrT>{tmp7});
    tmp9 = Convert_intptr_RawPtr_0(state_, TNode<RawPtrT>{p_stack});
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, kHeapObjectTag);
    tmp11 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp9}, TNode<IntPtrT>{tmp10});
    tmp12 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp13 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp14 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructLocationAllocator_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TNode<IntPtrT>{tmp2}, TNode<IntPtrT>{tmp5}, TNode<IntPtrT>{tmp8}, TNode<IntPtrT>{tmp11}, TNode<IntPtrT>{tmp11}, TNode<IntPtrT>{tmp14}, TNode<IntPtrT>{tmp12}, TNode<BoolT>{tmp13}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=450&c=1
TNode<Object> JSToWasmObject_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<Uint32T> p_targetType, TNode<JSAny> p_value) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Object> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Object> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  TNode<Uint32T> tmp2;
  TNode<BoolT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kHasIndexBit);
    tmp1 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{p_targetType}, TNode<Uint32T>{tmp0});
    tmp2 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp3 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp1}, TNode<Uint32T>{tmp2});
    ca_.Branch(tmp3, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<Uint32T> tmp4;
  TNode<Uint32T> tmp5;
  TNode<Uint32T> tmp6;
  TNode<Uint32T> tmp7;
  TNode<Uint32T> tmp8;
  TNode<Uint32T> tmp9;
  TNode<Uint32T> tmp10;
  TNode<BoolT> tmp11;
  TNode<Uint32T> tmp12;
  TNode<BoolT> tmp13;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp4 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIndexBits);
    tmp5 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{p_targetType}, TNode<Uint32T>{tmp4});
    tmp6 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIndexShift);
    tmp7 = CodeStubAssembler(state_).Word32Shr(TNode<Uint32T>{tmp5}, TNode<Uint32T>{tmp6});
    tmp8 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsNullableBit);
    tmp9 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{p_targetType}, TNode<Uint32T>{tmp8});
    tmp10 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp11 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp9}, TNode<Uint32T>{tmp10});
    tmp12 = FromConstexpr_uint32_constexpr_uint32_0(state_, CastIfEnumClass<uint32_t>(wasm::StandardType::kExtern));
    tmp13 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp7}, TNode<Uint32T>{tmp12});
    ca_.Branch(tmp13, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp14;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp14 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp11});
    ca_.Branch(tmp14, &block8, std::vector<compiler::Node*>{}, &block9, std::vector<compiler::Node*>{});
  }

  TNode<Null> tmp15;
  TNode<BoolT> tmp16;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    tmp15 = Null_0(state_);
    tmp16 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{p_value}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp15});
    ca_.Goto(&block10, tmp16);
  }

  TNode<BoolT> tmp17;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp17 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block10, tmp17);
  }

  TNode<BoolT> phi_bb10_7;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_7);
    ca_.Branch(phi_bb10_7, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{p_context}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kWasmTrapJSTypeError));
  }

  TNode<Uint32T> tmp18;
  TNode<Uint32T> tmp19;
  TNode<Uint32T> tmp20;
  TNode<BoolT> tmp21;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp18 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsSharedBit);
    tmp19 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{p_targetType}, TNode<Uint32T>{tmp18});
    tmp20 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp21 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp19}, TNode<Uint32T>{tmp20});
    ca_.Branch(tmp21, &block11, std::vector<compiler::Node*>{}, &block12, std::vector<compiler::Node*>{});
  }

  if (block11.is_used()) {
    ca_.Bind(&block11);
    ca_.Goto(&block1, p_value);
  }

  if (block12.is_used()) {
    ca_.Bind(&block12);
    ca_.Goto(&block5);
  }

  TNode<Uint32T> tmp22;
  TNode<BoolT> tmp23;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp22 = FromConstexpr_uint32_constexpr_uint32_0(state_, CastIfEnumClass<uint32_t>(wasm::StandardType::kString));
    tmp23 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp7}, TNode<Uint32T>{tmp22});
    ca_.Branch(tmp23, &block13, std::vector<compiler::Node*>{}, &block14, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp24;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp24 = CodeStubAssembler(state_).TaggedIsSmi(TNode<Object>{p_value});
    ca_.Branch(tmp24, &block15, std::vector<compiler::Node*>{}, &block16, std::vector<compiler::Node*>{});
  }

  if (block15.is_used()) {
    ca_.Bind(&block15);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{p_context}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kWasmTrapJSTypeError));
  }

  TNode<HeapObject> tmp25;
  TNode<BoolT> tmp26;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp25 = UnsafeCast_HeapObject_0(state_, TNode<Context>{p_context}, TNode<Object>{p_value});
    tmp26 = CodeStubAssembler(state_).IsString(TNode<HeapObject>{tmp25});
    ca_.Branch(tmp26, &block17, std::vector<compiler::Node*>{}, &block18, std::vector<compiler::Node*>{});
  }

  if (block17.is_used()) {
    ca_.Bind(&block17);
    ca_.Goto(&block1, p_value);
  }

  if (block18.is_used()) {
    ca_.Bind(&block18);
    ca_.Branch(tmp11, &block21, std::vector<compiler::Node*>{}, &block22, std::vector<compiler::Node*>{});
  }

  TNode<Null> tmp27;
  TNode<BoolT> tmp28;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp27 = Null_0(state_);
    tmp28 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{p_value}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp27});
    ca_.Goto(&block23, tmp28);
  }

  TNode<BoolT> tmp29;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    tmp29 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block23, tmp29);
  }

  TNode<BoolT> phi_bb23_7;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_7);
    ca_.Branch(phi_bb23_7, &block19, std::vector<compiler::Node*>{}, &block20, std::vector<compiler::Node*>{});
  }

  TNode<WasmNull> tmp30;
  if (block19.is_used()) {
    ca_.Bind(&block19);
    tmp30 = kWasmNull_0(state_);
    ca_.Goto(&block1, tmp30);
  }

  if (block20.is_used()) {
    ca_.Bind(&block20);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{p_context}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kWasmTrapJSTypeError));
  }

  if (block14.is_used()) {
    ca_.Bind(&block14);
    ca_.Goto(&block3);
  }

  TNode<Smi> tmp31;
  TNode<JSAny> tmp32;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp31 = Convert_Smi_uint32_0(state_, TNode<Uint32T>{p_targetType});
    tmp32 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kWasmJSToWasmObject, p_context, p_value, tmp31)); 
    ca_.Goto(&block1, tmp32);
  }

  TNode<Object> phi_bb1_3;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_3);
    ca_.Goto(&block24, phi_bb1_3);
  }

  TNode<Object> phi_bb24_3;
    ca_.Bind(&block24, &phi_bb24_3);
  return TNode<Object>{phi_bb24_3};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=478&c=1
void HandleF32Params_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TorqueStructLocationAllocator_0 p_locationAllocator, TorqueStructReference_intptr_0 p_toRef, TNode<JSAny> p_param) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  if (block0.is_used()) {
    ca_.Bind(&block0);
    if ((wasm::kIsFpAlwaysDouble)) {
      ca_.Goto(&block2);
    } else {
      ca_.Goto(&block3);
    }
  }

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).IntPtrGreaterThanOrEqual(TNode<IntPtrT>{p_locationAllocator.remainingFPRegs}, TNode<IntPtrT>{tmp0});
    ca_.Branch(tmp1, &block5, std::vector<compiler::Node*>{}, &block6, std::vector<compiler::Node*>{});
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<Float32T> tmp4;
  TNode<Float64T> tmp5;
  TNode<Float64T> tmp6;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    std::tie(tmp2, tmp3) = RefCast_float64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{p_toRef.object}, TNode<IntPtrT>{p_toRef.offset}, TorqueStructUnsafe_0{}}).Flatten();
    tmp4 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_param);
    tmp5 = CodeStubAssembler(state_).ChangeFloat32ToFloat64(TNode<Float32T>{tmp4});
    tmp6 = CodeStubAssembler(state_).Float64SilenceNaN(TNode<Float64T>{tmp5});
    CodeStubAssembler(state_).StoreReference<Float64T>(CodeStubAssembler::Reference{tmp2, tmp3}, tmp6);
    ca_.Goto(&block8);
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<Float32T> tmp9;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    std::tie(tmp7, tmp8) = RefCast_float32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{p_toRef.object}, TNode<IntPtrT>{p_toRef.offset}, TorqueStructUnsafe_0{}}).Flatten();
    tmp9 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_param);
    CodeStubAssembler(state_).StoreReference<Float32T>(CodeStubAssembler::Reference{tmp7, tmp8}, tmp9);
    ca_.Goto(&block8);
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    ca_.Goto(&block4);
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    if ((wasm::kIsBigEndian)) {
      ca_.Goto(&block9);
    } else {
      ca_.Goto(&block10);
    }
  }

  TNode<Float32T> tmp10;
  TNode<Uint32T> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<IntPtrT> tmp14;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp10 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_param);
    tmp11 = Bitcast_uint32_float32_0(state_, TNode<Float32T>{tmp10});
    tmp12 = Convert_intptr_uint32_0(state_, TNode<Uint32T>{tmp11});
    tmp13 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x20ull));
    tmp14 = CodeStubAssembler(state_).WordShl(TNode<IntPtrT>{tmp12}, TNode<IntPtrT>{tmp13});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{p_toRef.object, p_toRef.offset}, tmp14);
    ca_.Goto(&block11);
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    if ((wasm::kIsBigEndianOnSim)) {
      ca_.Goto(&block12);
    } else {
      ca_.Goto(&block13);
    }
  }

  TNode<IntPtrT> tmp15;
  TNode<BoolT> tmp16;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp15 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp16 = CodeStubAssembler(state_).IntPtrGreaterThanOrEqual(TNode<IntPtrT>{p_locationAllocator.remainingFPRegs}, TNode<IntPtrT>{tmp15});
    ca_.Branch(tmp16, &block15, std::vector<compiler::Node*>{}, &block16, std::vector<compiler::Node*>{});
  }

  TNode<Float32T> tmp17;
  TNode<Uint32T> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<IntPtrT> tmp20;
  TNode<IntPtrT> tmp21;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp17 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_param);
    tmp18 = Bitcast_uint32_float32_0(state_, TNode<Float32T>{tmp17});
    tmp19 = Convert_intptr_uint32_0(state_, TNode<Uint32T>{tmp18});
    tmp20 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x20ull));
    tmp21 = CodeStubAssembler(state_).WordShl(TNode<IntPtrT>{tmp19}, TNode<IntPtrT>{tmp20});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{p_toRef.object, p_toRef.offset}, tmp21);
    ca_.Goto(&block18);
  }

  TNode<Float32T> tmp22;
  TNode<Uint32T> tmp23;
  TNode<IntPtrT> tmp24;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp22 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_param);
    tmp23 = Bitcast_uint32_float32_0(state_, TNode<Float32T>{tmp22});
    tmp24 = Convert_intptr_uint32_0(state_, TNode<Uint32T>{tmp23});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{p_toRef.object, p_toRef.offset}, tmp24);
    ca_.Goto(&block18);
  }

  if (block18.is_used()) {
    ca_.Bind(&block18);
    ca_.Goto(&block14);
  }

  if (block13.is_used()) {
    ca_.Bind(&block13);
    ca_.Goto(&block14);
  }

  if (block14.is_used()) {
    ca_.Bind(&block14);
    ca_.Goto(&block11);
  }

  if (block11.is_used()) {
    ca_.Bind(&block11);
    ca_.Goto(&block4);
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    ca_.Goto(&block19);
  }

    ca_.Bind(&block19);
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=505&c=1
TNode<JSAny> JSToWasmWrapperHelper_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<JSAny> p__receiver, TNode<JSFunction> p_target, TorqueStructArguments p_arguments, wasm::Promise p_promise) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block42(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block52(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block45(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block60(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block62(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block63(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block66(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block67(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block72(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block73(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block75(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block76(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block78(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block71(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block81(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block85(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block86(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block87(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block88(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block89(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block84(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block82(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block91(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block90(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block83(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block70(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block44(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block92(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block95(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block96(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block100(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block98(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block109(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block112(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block113(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block115(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block116(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block118(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block119(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block120(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block117(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block111(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT> block110(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT> block99(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT, IntPtrT> block93(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT, IntPtrT> block122(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT, IntPtrT> block123(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, IntPtrT, IntPtrT, BoolT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block121(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT> block128(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT> block127(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT> block129(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT> block130(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block131(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block142(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block140(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block145(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block149(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block150(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block152(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block153(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block155(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block156(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block151(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block148(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block160(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block159(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block157(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block146(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block161(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block165(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block166(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block168(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block169(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block171(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block172(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block167(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block164(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block173(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block174(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block175(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block162(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block176(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block180(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block181(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block182(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block186(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block187(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block189(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block190(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block185(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block183(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block179(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block177(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block191(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block194(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block198(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block199(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block201(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block202(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block204(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block205(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block200(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block197(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block195(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block207(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block208(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block210(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block211(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block213(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block214(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block209(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block206(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block216(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block217(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block219(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block220(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block222(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block223(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block218(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block215(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block196(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block192(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block225(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block224(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, Union<FixedArray, Smi>, IntPtrT> block226(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, Union<FixedArray, Smi>, IntPtrT> block227(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block232(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block233(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block193(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block178(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block163(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block147(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block141(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block236(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block239(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block240(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block237(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, JSAny> block238(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block241(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block244(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block245(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block248(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block246(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block251(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block257(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block258(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block262(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block263(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block265(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block266(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block268(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block269(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block264(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block261(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block252(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block247(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block242(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block272(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, IntPtrT> block273(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block277(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block278(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, BoolT> block279(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block276(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block275(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block280(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block281(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block283(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block284(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block286(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>> block287(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, JSAny> block285(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<FixedArray, Smi>, JSAny> block282(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block289(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<SharedFunctionInfo> tmp1;
  TNode<WasmExportedFunctionData> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp1 = CodeStubAssembler(state_).LoadReference<SharedFunctionInfo>(CodeStubAssembler::Reference{p_target, tmp0});
    tmp2 = CodeStubAssembler(state_).LoadSharedFunctionInfoWasmExportedFunctionData(TNode<SharedFunctionInfo>{tmp1});
    if (((CodeStubAssembler(state_).ConstexprBoolNot(((CodeStubAssembler(state_).ConstexprInt32Equal(CastIfEnumClass<int32_t>(p_promise), CastIfEnumClass<int32_t>(wasm::Promise::kPromise))) || (CodeStubAssembler(state_).ConstexprInt32Equal(CastIfEnumClass<int32_t>(p_promise), CastIfEnumClass<int32_t>(wasm::Promise::kStressSwitch)))))))) {
      ca_.Goto(&block2);
    } else {
      ca_.Goto(&block3);
    }
  }

  TNode<IntPtrT> tmp3;
  TNode<Cell> tmp4;
  TNode<Object> tmp5;
  TNode<Smi> tmp6;
  TNode<Smi> tmp7;
  TNode<Smi> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<Cell> tmp10;
  TNode<Smi> tmp11;
  TNode<BoolT> tmp12;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    tmp4 = CodeStubAssembler(state_).LoadReference<Cell>(CodeStubAssembler::Reference{tmp2, tmp3});
    tmp5 = LoadCellValue_0(state_, TNode<Cell>{tmp4});
    tmp6 = UnsafeCast_Smi_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp5});
    tmp7 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp8 = CodeStubAssembler(state_).SmiSub(TNode<Smi>{tmp6}, TNode<Smi>{tmp7});
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    tmp10 = CodeStubAssembler(state_).LoadReference<Cell>(CodeStubAssembler::Reference{tmp2, tmp9});
    StoreCellValue_0(state_, TNode<Cell>{tmp10}, TNode<Object>{tmp8});
    tmp11 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp12 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp8}, TNode<Smi>{tmp11});
    ca_.Branch(tmp12, &block9, std::vector<compiler::Node*>{}, &block10, std::vector<compiler::Node*>{});
  }

  TNode<Smi> tmp13;
  TNode<JSAny> tmp14;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp13 = kNoContext_0(state_);
    tmp14 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kTierUpJSToWasmWrapper, tmp13, tmp2)); 
    ca_.Goto(&block10);
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    ca_.Goto(&block4);
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block4);
  }

  TNode<WasmInternalFunction> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<RawPtrT> tmp17;
  TNode<Union<WasmImportData, WasmTrustedInstanceData>> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<Union<HeapObject, TaggedIndex>> tmp20;
  TNode<IntPtrT> tmp21;
  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<Union<HeapObject, TaggedIndex>> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<IntPtrT> tmp26;
  TNode<IntPtrT> tmp27;
  TNode<Union<HeapObject, TaggedIndex>> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<RawPtrT> tmp30;
  TNode<RawPtrT> tmp31;
  TNode<IntPtrT> tmp32;
  TNode<Union<HeapObject, TaggedIndex>> tmp33;
  TNode<IntPtrT> tmp34;
  TNode<IntPtrT> tmp35;
  TNode<Undefined> tmp36;
  TNode<IntPtrT> tmp37;
  TNode<BoolT> tmp38;
  TNode<IntPtrT> tmp39;
  TNode<BoolT> tmp40;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp15 = CodeStubAssembler(state_).LoadWasmInternalFunctionFromFunctionData(TNode<WasmFunctionData>{tmp2});
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp17 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{tmp15, tmp16});
    tmp18 = CodeStubAssembler(state_).LoadImplicitArgFromWasmInternalFunction(TNode<WasmInternalFunction>{tmp15});
    tmp19 = FromConstexpr_intptr_constexpr_intptr_0(state_, wasm::FunctionSig::kParameterCountOffset);
    std::tie(tmp20, tmp21) = GetRefAt_intptr_RawFunctionSigPtr_0(state_, TNode<RawPtrT>{tmp17}, TNode<IntPtrT>{tmp19}).Flatten();
    tmp22 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp20, tmp21});
    tmp23 = FromConstexpr_intptr_constexpr_intptr_0(state_, wasm::FunctionSig::kReturnCountOffset);
    std::tie(tmp24, tmp25) = GetRefAt_intptr_RawFunctionSigPtr_0(state_, TNode<RawPtrT>{tmp17}, TNode<IntPtrT>{tmp23}).Flatten();
    tmp26 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp24, tmp25});
    tmp27 = FromConstexpr_intptr_constexpr_intptr_0(state_, wasm::FunctionSig::kRepsOffset);
    std::tie(tmp28, tmp29) = GetRefAt_RawPtr_RawFunctionSigPtr_0(state_, TNode<RawPtrT>{tmp17}, TNode<IntPtrT>{tmp27}).Flatten();
    tmp30 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{tmp28, tmp29});
    tmp31 = (TNode<RawPtrT>{tmp30});
    tmp32 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp22}, TNode<IntPtrT>{tmp26});
    std::tie(tmp33, tmp34, tmp35) = NewOffHeapConstSlice_uint32_0(state_, TNode<RawPtrT>{tmp31}, TNode<IntPtrT>{tmp32}).Flatten();
    tmp36 = Undefined_0(state_);
    tmp37 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp38 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp39 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp40 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp26}, TNode<IntPtrT>{tmp39});
    ca_.Branch(tmp40, &block11, std::vector<compiler::Node*>{}, &block12, std::vector<compiler::Node*>{tmp36, tmp37, tmp38});
  }

  TNode<Smi> tmp41;
  TNode<JSArray> tmp42;
  TNode<IntPtrT> tmp43;
  TNode<Union<HeapObject, TaggedIndex>> tmp44;
  TNode<IntPtrT> tmp45;
  TNode<IntPtrT> tmp46;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp41 = CodeStubAssembler(state_).SmiFromIntPtr(TNode<IntPtrT>{tmp26});
    tmp42 = ca_.CallBuiltin<JSArray>(Builtin::kWasmAllocateJSArray, p_context, tmp41);
    tmp43 = Convert_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    compiler::CodeAssemblerLabel label47(&ca_);
    std::tie(tmp44, tmp45, tmp46) = Subslice_uint32_0(state_, TorqueStructSlice_uint32_ConstReference_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp33}, TNode<IntPtrT>{tmp34}, TNode<IntPtrT>{tmp35}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{tmp43}, TNode<IntPtrT>{tmp26}, &label47).Flatten();
    ca_.Goto(&block15);
    if (label47.is_used()) {
      ca_.Bind(&label47);
      ca_.Goto(&block16);
    }
  }

  if (block16.is_used()) {
    ca_.Bind(&block16);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> tmp48;
  TNode<IntPtrT> tmp49;
  TNode<BoolT> tmp50;
  TNode<BoolT> tmp51;
  TNode<IntPtrT> tmp52;
  TNode<IntPtrT> tmp53;
  TNode<IntPtrT> tmp54;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    std::tie(tmp48, tmp49, tmp50, tmp51, tmp52) = NewReturnSlotAllocator_0(state_).Flatten();
    tmp53 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{tmp46});
    tmp54 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp45}, TNode<IntPtrT>{tmp53});
    ca_.Goto(&block20, tmp38, tmp48, tmp49, tmp50, tmp51, tmp52, tmp45);
  }

  TNode<BoolT> phi_bb20_19;
  TNode<IntPtrT> phi_bb20_23;
  TNode<IntPtrT> phi_bb20_24;
  TNode<BoolT> phi_bb20_25;
  TNode<BoolT> phi_bb20_26;
  TNode<IntPtrT> phi_bb20_27;
  TNode<IntPtrT> phi_bb20_29;
  TNode<BoolT> tmp55;
  TNode<BoolT> tmp56;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_19, &phi_bb20_23, &phi_bb20_24, &phi_bb20_25, &phi_bb20_26, &phi_bb20_27, &phi_bb20_29);
    tmp55 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb20_29}, TNode<IntPtrT>{tmp54});
    tmp56 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp55});
    ca_.Branch(tmp56, &block18, std::vector<compiler::Node*>{phi_bb20_19, phi_bb20_23, phi_bb20_24, phi_bb20_25, phi_bb20_26, phi_bb20_27, phi_bb20_29}, &block19, std::vector<compiler::Node*>{phi_bb20_19, phi_bb20_23, phi_bb20_24, phi_bb20_25, phi_bb20_26, phi_bb20_27, phi_bb20_29});
  }

  TNode<BoolT> phi_bb18_19;
  TNode<IntPtrT> phi_bb18_23;
  TNode<IntPtrT> phi_bb18_24;
  TNode<BoolT> phi_bb18_25;
  TNode<BoolT> phi_bb18_26;
  TNode<IntPtrT> phi_bb18_27;
  TNode<IntPtrT> phi_bb18_29;
  TNode<Union<HeapObject, TaggedIndex>> tmp57;
  TNode<IntPtrT> tmp58;
  TNode<IntPtrT> tmp59;
  TNode<IntPtrT> tmp60;
  TNode<Uint32T> tmp61;
  TNode<Uint32T> tmp62;
  TNode<BoolT> tmp63;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_19, &phi_bb18_23, &phi_bb18_24, &phi_bb18_25, &phi_bb18_26, &phi_bb18_27, &phi_bb18_29);
    std::tie(tmp57, tmp58) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp44}, TNode<IntPtrT>{phi_bb18_29}).Flatten();
    tmp59 = FromConstexpr_intptr_constexpr_int31_0(state_, kInt32Size);
    tmp60 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb18_29}, TNode<IntPtrT>{tmp59});
    tmp61 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp57, tmp58});
    tmp62 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI32.raw_bit_field());
    tmp63 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp61}, TNode<Uint32T>{tmp62});
    ca_.Branch(tmp63, &block29, std::vector<compiler::Node*>{phi_bb18_19, phi_bb18_23, phi_bb18_24, phi_bb18_25, phi_bb18_26, phi_bb18_27}, &block30, std::vector<compiler::Node*>{phi_bb18_19, phi_bb18_23, phi_bb18_24, phi_bb18_25, phi_bb18_26, phi_bb18_27});
  }

  TNode<BoolT> phi_bb29_19;
  TNode<IntPtrT> phi_bb29_23;
  TNode<IntPtrT> phi_bb29_24;
  TNode<BoolT> phi_bb29_25;
  TNode<BoolT> phi_bb29_26;
  TNode<IntPtrT> phi_bb29_27;
  TNode<IntPtrT> tmp64;
  TNode<BoolT> tmp65;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_19, &phi_bb29_23, &phi_bb29_24, &phi_bb29_25, &phi_bb29_26, &phi_bb29_27);
    tmp64 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp65 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb29_23}, TNode<IntPtrT>{tmp64});
    ca_.Branch(tmp65, &block33, std::vector<compiler::Node*>{phi_bb29_19, phi_bb29_23, phi_bb29_24, phi_bb29_25, phi_bb29_26, phi_bb29_27}, &block34, std::vector<compiler::Node*>{phi_bb29_19, phi_bb29_23, phi_bb29_24, phi_bb29_25, phi_bb29_26, phi_bb29_27});
  }

  TNode<BoolT> phi_bb33_19;
  TNode<IntPtrT> phi_bb33_23;
  TNode<IntPtrT> phi_bb33_24;
  TNode<BoolT> phi_bb33_25;
  TNode<BoolT> phi_bb33_26;
  TNode<IntPtrT> phi_bb33_27;
  TNode<IntPtrT> tmp66;
  TNode<IntPtrT> tmp67;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_19, &phi_bb33_23, &phi_bb33_24, &phi_bb33_25, &phi_bb33_26, &phi_bb33_27);
    tmp66 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp67 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb33_23}, TNode<IntPtrT>{tmp66});
    ca_.Goto(&block32, phi_bb33_19, tmp67, phi_bb33_24, phi_bb33_25, phi_bb33_26, phi_bb33_27);
  }

  TNode<BoolT> phi_bb34_19;
  TNode<IntPtrT> phi_bb34_23;
  TNode<IntPtrT> phi_bb34_24;
  TNode<BoolT> phi_bb34_25;
  TNode<BoolT> phi_bb34_26;
  TNode<IntPtrT> phi_bb34_27;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_19, &phi_bb34_23, &phi_bb34_24, &phi_bb34_25, &phi_bb34_26, &phi_bb34_27);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block36, phi_bb34_19, phi_bb34_23, phi_bb34_24, phi_bb34_25, phi_bb34_26, phi_bb34_27);
    } else {
      ca_.Goto(&block37, phi_bb34_19, phi_bb34_23, phi_bb34_24, phi_bb34_25, phi_bb34_26, phi_bb34_27);
    }
  }

  TNode<BoolT> phi_bb36_19;
  TNode<IntPtrT> phi_bb36_23;
  TNode<IntPtrT> phi_bb36_24;
  TNode<BoolT> phi_bb36_25;
  TNode<BoolT> phi_bb36_26;
  TNode<IntPtrT> phi_bb36_27;
  TNode<IntPtrT> tmp68;
  TNode<IntPtrT> tmp69;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_19, &phi_bb36_23, &phi_bb36_24, &phi_bb36_25, &phi_bb36_26, &phi_bb36_27);
    tmp68 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp69 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb36_27}, TNode<IntPtrT>{tmp68});
    ca_.Goto(&block38, phi_bb36_19, phi_bb36_23, phi_bb36_24, phi_bb36_25, phi_bb36_26, tmp69);
  }

  TNode<BoolT> phi_bb37_19;
  TNode<IntPtrT> phi_bb37_23;
  TNode<IntPtrT> phi_bb37_24;
  TNode<BoolT> phi_bb37_25;
  TNode<BoolT> phi_bb37_26;
  TNode<IntPtrT> phi_bb37_27;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_19, &phi_bb37_23, &phi_bb37_24, &phi_bb37_25, &phi_bb37_26, &phi_bb37_27);
    ca_.Branch(phi_bb37_25, &block39, std::vector<compiler::Node*>{phi_bb37_19, phi_bb37_23, phi_bb37_24, phi_bb37_25, phi_bb37_26, phi_bb37_27}, &block40, std::vector<compiler::Node*>{phi_bb37_19, phi_bb37_23, phi_bb37_24, phi_bb37_25, phi_bb37_26, phi_bb37_27});
  }

  TNode<BoolT> phi_bb39_19;
  TNode<IntPtrT> phi_bb39_23;
  TNode<IntPtrT> phi_bb39_24;
  TNode<BoolT> phi_bb39_25;
  TNode<BoolT> phi_bb39_26;
  TNode<IntPtrT> phi_bb39_27;
  TNode<BoolT> tmp70;
  TNode<BoolT> tmp71;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_19, &phi_bb39_23, &phi_bb39_24, &phi_bb39_25, &phi_bb39_26, &phi_bb39_27);
    tmp70 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp71 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block41, phi_bb39_19, phi_bb39_23, phi_bb39_24, tmp70, tmp71, phi_bb39_27);
  }

  TNode<BoolT> phi_bb40_19;
  TNode<IntPtrT> phi_bb40_23;
  TNode<IntPtrT> phi_bb40_24;
  TNode<BoolT> phi_bb40_25;
  TNode<BoolT> phi_bb40_26;
  TNode<IntPtrT> phi_bb40_27;
  TNode<IntPtrT> tmp72;
  TNode<IntPtrT> tmp73;
  TNode<BoolT> tmp74;
  TNode<BoolT> tmp75;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_19, &phi_bb40_23, &phi_bb40_24, &phi_bb40_25, &phi_bb40_26, &phi_bb40_27);
    tmp72 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp73 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb40_27}, TNode<IntPtrT>{tmp72});
    tmp74 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp75 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block41, phi_bb40_19, phi_bb40_23, phi_bb40_24, tmp74, tmp75, tmp73);
  }

  TNode<BoolT> phi_bb41_19;
  TNode<IntPtrT> phi_bb41_23;
  TNode<IntPtrT> phi_bb41_24;
  TNode<BoolT> phi_bb41_25;
  TNode<BoolT> phi_bb41_26;
  TNode<IntPtrT> phi_bb41_27;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_19, &phi_bb41_23, &phi_bb41_24, &phi_bb41_25, &phi_bb41_26, &phi_bb41_27);
    ca_.Goto(&block38, phi_bb41_19, phi_bb41_23, phi_bb41_24, phi_bb41_25, phi_bb41_26, phi_bb41_27);
  }

  TNode<BoolT> phi_bb38_19;
  TNode<IntPtrT> phi_bb38_23;
  TNode<IntPtrT> phi_bb38_24;
  TNode<BoolT> phi_bb38_25;
  TNode<BoolT> phi_bb38_26;
  TNode<IntPtrT> phi_bb38_27;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_19, &phi_bb38_23, &phi_bb38_24, &phi_bb38_25, &phi_bb38_26, &phi_bb38_27);
    ca_.Goto(&block32, phi_bb38_19, phi_bb38_23, phi_bb38_24, phi_bb38_25, phi_bb38_26, phi_bb38_27);
  }

  TNode<BoolT> phi_bb32_19;
  TNode<IntPtrT> phi_bb32_23;
  TNode<IntPtrT> phi_bb32_24;
  TNode<BoolT> phi_bb32_25;
  TNode<BoolT> phi_bb32_26;
  TNode<IntPtrT> phi_bb32_27;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_19, &phi_bb32_23, &phi_bb32_24, &phi_bb32_25, &phi_bb32_26, &phi_bb32_27);
    ca_.Goto(&block31, phi_bb32_19, phi_bb32_23, phi_bb32_24, phi_bb32_25, phi_bb32_26, phi_bb32_27);
  }

  TNode<BoolT> phi_bb30_19;
  TNode<IntPtrT> phi_bb30_23;
  TNode<IntPtrT> phi_bb30_24;
  TNode<BoolT> phi_bb30_25;
  TNode<BoolT> phi_bb30_26;
  TNode<IntPtrT> phi_bb30_27;
  TNode<Uint32T> tmp76;
  TNode<BoolT> tmp77;
  if (block30.is_used()) {
    ca_.Bind(&block30, &phi_bb30_19, &phi_bb30_23, &phi_bb30_24, &phi_bb30_25, &phi_bb30_26, &phi_bb30_27);
    tmp76 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI64.raw_bit_field());
    tmp77 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp61}, TNode<Uint32T>{tmp76});
    ca_.Branch(tmp77, &block42, std::vector<compiler::Node*>{phi_bb30_19, phi_bb30_23, phi_bb30_24, phi_bb30_25, phi_bb30_26, phi_bb30_27}, &block43, std::vector<compiler::Node*>{phi_bb30_19, phi_bb30_23, phi_bb30_24, phi_bb30_25, phi_bb30_26, phi_bb30_27});
  }

  TNode<BoolT> phi_bb42_19;
  TNode<IntPtrT> phi_bb42_23;
  TNode<IntPtrT> phi_bb42_24;
  TNode<BoolT> phi_bb42_25;
  TNode<BoolT> phi_bb42_26;
  TNode<IntPtrT> phi_bb42_27;
  TNode<IntPtrT> tmp78;
  TNode<BoolT> tmp79;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_19, &phi_bb42_23, &phi_bb42_24, &phi_bb42_25, &phi_bb42_26, &phi_bb42_27);
    tmp78 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp79 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb42_23}, TNode<IntPtrT>{tmp78});
    ca_.Branch(tmp79, &block46, std::vector<compiler::Node*>{phi_bb42_19, phi_bb42_23, phi_bb42_24, phi_bb42_25, phi_bb42_26, phi_bb42_27}, &block47, std::vector<compiler::Node*>{phi_bb42_19, phi_bb42_23, phi_bb42_24, phi_bb42_25, phi_bb42_26, phi_bb42_27});
  }

  TNode<BoolT> phi_bb46_19;
  TNode<IntPtrT> phi_bb46_23;
  TNode<IntPtrT> phi_bb46_24;
  TNode<BoolT> phi_bb46_25;
  TNode<BoolT> phi_bb46_26;
  TNode<IntPtrT> phi_bb46_27;
  TNode<IntPtrT> tmp80;
  TNode<IntPtrT> tmp81;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_19, &phi_bb46_23, &phi_bb46_24, &phi_bb46_25, &phi_bb46_26, &phi_bb46_27);
    tmp80 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp81 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb46_23}, TNode<IntPtrT>{tmp80});
    ca_.Goto(&block45, phi_bb46_19, tmp81, phi_bb46_24, phi_bb46_25, phi_bb46_26, phi_bb46_27);
  }

  TNode<BoolT> phi_bb47_19;
  TNode<IntPtrT> phi_bb47_23;
  TNode<IntPtrT> phi_bb47_24;
  TNode<BoolT> phi_bb47_25;
  TNode<BoolT> phi_bb47_26;
  TNode<IntPtrT> phi_bb47_27;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_19, &phi_bb47_23, &phi_bb47_24, &phi_bb47_25, &phi_bb47_26, &phi_bb47_27);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block49, phi_bb47_19, phi_bb47_23, phi_bb47_24, phi_bb47_25, phi_bb47_26, phi_bb47_27);
    } else {
      ca_.Goto(&block50, phi_bb47_19, phi_bb47_23, phi_bb47_24, phi_bb47_25, phi_bb47_26, phi_bb47_27);
    }
  }

  TNode<BoolT> phi_bb49_19;
  TNode<IntPtrT> phi_bb49_23;
  TNode<IntPtrT> phi_bb49_24;
  TNode<BoolT> phi_bb49_25;
  TNode<BoolT> phi_bb49_26;
  TNode<IntPtrT> phi_bb49_27;
  TNode<IntPtrT> tmp82;
  TNode<IntPtrT> tmp83;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_19, &phi_bb49_23, &phi_bb49_24, &phi_bb49_25, &phi_bb49_26, &phi_bb49_27);
    tmp82 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp83 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb49_27}, TNode<IntPtrT>{tmp82});
    ca_.Goto(&block51, phi_bb49_19, phi_bb49_23, phi_bb49_24, phi_bb49_25, phi_bb49_26, tmp83);
  }

  TNode<BoolT> phi_bb50_19;
  TNode<IntPtrT> phi_bb50_23;
  TNode<IntPtrT> phi_bb50_24;
  TNode<BoolT> phi_bb50_25;
  TNode<BoolT> phi_bb50_26;
  TNode<IntPtrT> phi_bb50_27;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_19, &phi_bb50_23, &phi_bb50_24, &phi_bb50_25, &phi_bb50_26, &phi_bb50_27);
    ca_.Branch(phi_bb50_25, &block52, std::vector<compiler::Node*>{phi_bb50_19, phi_bb50_23, phi_bb50_24, phi_bb50_25, phi_bb50_26, phi_bb50_27}, &block53, std::vector<compiler::Node*>{phi_bb50_19, phi_bb50_23, phi_bb50_24, phi_bb50_25, phi_bb50_26, phi_bb50_27});
  }

  TNode<BoolT> phi_bb52_19;
  TNode<IntPtrT> phi_bb52_23;
  TNode<IntPtrT> phi_bb52_24;
  TNode<BoolT> phi_bb52_25;
  TNode<BoolT> phi_bb52_26;
  TNode<IntPtrT> phi_bb52_27;
  TNode<BoolT> tmp84;
  TNode<BoolT> tmp85;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_19, &phi_bb52_23, &phi_bb52_24, &phi_bb52_25, &phi_bb52_26, &phi_bb52_27);
    tmp84 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp85 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block54, phi_bb52_19, phi_bb52_23, phi_bb52_24, tmp84, tmp85, phi_bb52_27);
  }

  TNode<BoolT> phi_bb53_19;
  TNode<IntPtrT> phi_bb53_23;
  TNode<IntPtrT> phi_bb53_24;
  TNode<BoolT> phi_bb53_25;
  TNode<BoolT> phi_bb53_26;
  TNode<IntPtrT> phi_bb53_27;
  TNode<IntPtrT> tmp86;
  TNode<IntPtrT> tmp87;
  TNode<BoolT> tmp88;
  TNode<BoolT> tmp89;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_19, &phi_bb53_23, &phi_bb53_24, &phi_bb53_25, &phi_bb53_26, &phi_bb53_27);
    tmp86 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp87 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb53_27}, TNode<IntPtrT>{tmp86});
    tmp88 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp89 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block54, phi_bb53_19, phi_bb53_23, phi_bb53_24, tmp88, tmp89, tmp87);
  }

  TNode<BoolT> phi_bb54_19;
  TNode<IntPtrT> phi_bb54_23;
  TNode<IntPtrT> phi_bb54_24;
  TNode<BoolT> phi_bb54_25;
  TNode<BoolT> phi_bb54_26;
  TNode<IntPtrT> phi_bb54_27;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_19, &phi_bb54_23, &phi_bb54_24, &phi_bb54_25, &phi_bb54_26, &phi_bb54_27);
    ca_.Goto(&block51, phi_bb54_19, phi_bb54_23, phi_bb54_24, phi_bb54_25, phi_bb54_26, phi_bb54_27);
  }

  TNode<BoolT> phi_bb51_19;
  TNode<IntPtrT> phi_bb51_23;
  TNode<IntPtrT> phi_bb51_24;
  TNode<BoolT> phi_bb51_25;
  TNode<BoolT> phi_bb51_26;
  TNode<IntPtrT> phi_bb51_27;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_19, &phi_bb51_23, &phi_bb51_24, &phi_bb51_25, &phi_bb51_26, &phi_bb51_27);
    ca_.Goto(&block45, phi_bb51_19, phi_bb51_23, phi_bb51_24, phi_bb51_25, phi_bb51_26, phi_bb51_27);
  }

  TNode<BoolT> phi_bb45_19;
  TNode<IntPtrT> phi_bb45_23;
  TNode<IntPtrT> phi_bb45_24;
  TNode<BoolT> phi_bb45_25;
  TNode<BoolT> phi_bb45_26;
  TNode<IntPtrT> phi_bb45_27;
  if (block45.is_used()) {
    ca_.Bind(&block45, &phi_bb45_19, &phi_bb45_23, &phi_bb45_24, &phi_bb45_25, &phi_bb45_26, &phi_bb45_27);
    if (((CodeStubAssembler(state_).ConstexprBoolNot((CodeStubAssembler(state_).Is64()))))) {
      ca_.Goto(&block55, phi_bb45_19, phi_bb45_23, phi_bb45_24, phi_bb45_25, phi_bb45_26, phi_bb45_27);
    } else {
      ca_.Goto(&block56, phi_bb45_19, phi_bb45_23, phi_bb45_24, phi_bb45_25, phi_bb45_26, phi_bb45_27);
    }
  }

  TNode<BoolT> phi_bb55_19;
  TNode<IntPtrT> phi_bb55_23;
  TNode<IntPtrT> phi_bb55_24;
  TNode<BoolT> phi_bb55_25;
  TNode<BoolT> phi_bb55_26;
  TNode<IntPtrT> phi_bb55_27;
  TNode<IntPtrT> tmp90;
  TNode<BoolT> tmp91;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_19, &phi_bb55_23, &phi_bb55_24, &phi_bb55_25, &phi_bb55_26, &phi_bb55_27);
    tmp90 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp91 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb55_23}, TNode<IntPtrT>{tmp90});
    ca_.Branch(tmp91, &block59, std::vector<compiler::Node*>{phi_bb55_19, phi_bb55_23, phi_bb55_24, phi_bb55_25, phi_bb55_26, phi_bb55_27}, &block60, std::vector<compiler::Node*>{phi_bb55_19, phi_bb55_23, phi_bb55_24, phi_bb55_25, phi_bb55_26, phi_bb55_27});
  }

  TNode<BoolT> phi_bb59_19;
  TNode<IntPtrT> phi_bb59_23;
  TNode<IntPtrT> phi_bb59_24;
  TNode<BoolT> phi_bb59_25;
  TNode<BoolT> phi_bb59_26;
  TNode<IntPtrT> phi_bb59_27;
  TNode<IntPtrT> tmp92;
  TNode<IntPtrT> tmp93;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_19, &phi_bb59_23, &phi_bb59_24, &phi_bb59_25, &phi_bb59_26, &phi_bb59_27);
    tmp92 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp93 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb59_23}, TNode<IntPtrT>{tmp92});
    ca_.Goto(&block58, phi_bb59_19, tmp93, phi_bb59_24, phi_bb59_25, phi_bb59_26, phi_bb59_27);
  }

  TNode<BoolT> phi_bb60_19;
  TNode<IntPtrT> phi_bb60_23;
  TNode<IntPtrT> phi_bb60_24;
  TNode<BoolT> phi_bb60_25;
  TNode<BoolT> phi_bb60_26;
  TNode<IntPtrT> phi_bb60_27;
  if (block60.is_used()) {
    ca_.Bind(&block60, &phi_bb60_19, &phi_bb60_23, &phi_bb60_24, &phi_bb60_25, &phi_bb60_26, &phi_bb60_27);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block62, phi_bb60_19, phi_bb60_23, phi_bb60_24, phi_bb60_25, phi_bb60_26, phi_bb60_27);
    } else {
      ca_.Goto(&block63, phi_bb60_19, phi_bb60_23, phi_bb60_24, phi_bb60_25, phi_bb60_26, phi_bb60_27);
    }
  }

  TNode<BoolT> phi_bb62_19;
  TNode<IntPtrT> phi_bb62_23;
  TNode<IntPtrT> phi_bb62_24;
  TNode<BoolT> phi_bb62_25;
  TNode<BoolT> phi_bb62_26;
  TNode<IntPtrT> phi_bb62_27;
  TNode<IntPtrT> tmp94;
  TNode<IntPtrT> tmp95;
  if (block62.is_used()) {
    ca_.Bind(&block62, &phi_bb62_19, &phi_bb62_23, &phi_bb62_24, &phi_bb62_25, &phi_bb62_26, &phi_bb62_27);
    tmp94 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp95 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb62_27}, TNode<IntPtrT>{tmp94});
    ca_.Goto(&block64, phi_bb62_19, phi_bb62_23, phi_bb62_24, phi_bb62_25, phi_bb62_26, tmp95);
  }

  TNode<BoolT> phi_bb63_19;
  TNode<IntPtrT> phi_bb63_23;
  TNode<IntPtrT> phi_bb63_24;
  TNode<BoolT> phi_bb63_25;
  TNode<BoolT> phi_bb63_26;
  TNode<IntPtrT> phi_bb63_27;
  if (block63.is_used()) {
    ca_.Bind(&block63, &phi_bb63_19, &phi_bb63_23, &phi_bb63_24, &phi_bb63_25, &phi_bb63_26, &phi_bb63_27);
    ca_.Branch(phi_bb63_25, &block65, std::vector<compiler::Node*>{phi_bb63_19, phi_bb63_23, phi_bb63_24, phi_bb63_25, phi_bb63_26, phi_bb63_27}, &block66, std::vector<compiler::Node*>{phi_bb63_19, phi_bb63_23, phi_bb63_24, phi_bb63_25, phi_bb63_26, phi_bb63_27});
  }

  TNode<BoolT> phi_bb65_19;
  TNode<IntPtrT> phi_bb65_23;
  TNode<IntPtrT> phi_bb65_24;
  TNode<BoolT> phi_bb65_25;
  TNode<BoolT> phi_bb65_26;
  TNode<IntPtrT> phi_bb65_27;
  TNode<BoolT> tmp96;
  TNode<BoolT> tmp97;
  if (block65.is_used()) {
    ca_.Bind(&block65, &phi_bb65_19, &phi_bb65_23, &phi_bb65_24, &phi_bb65_25, &phi_bb65_26, &phi_bb65_27);
    tmp96 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp97 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block67, phi_bb65_19, phi_bb65_23, phi_bb65_24, tmp96, tmp97, phi_bb65_27);
  }

  TNode<BoolT> phi_bb66_19;
  TNode<IntPtrT> phi_bb66_23;
  TNode<IntPtrT> phi_bb66_24;
  TNode<BoolT> phi_bb66_25;
  TNode<BoolT> phi_bb66_26;
  TNode<IntPtrT> phi_bb66_27;
  TNode<IntPtrT> tmp98;
  TNode<IntPtrT> tmp99;
  TNode<BoolT> tmp100;
  TNode<BoolT> tmp101;
  if (block66.is_used()) {
    ca_.Bind(&block66, &phi_bb66_19, &phi_bb66_23, &phi_bb66_24, &phi_bb66_25, &phi_bb66_26, &phi_bb66_27);
    tmp98 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp99 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb66_27}, TNode<IntPtrT>{tmp98});
    tmp100 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp101 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block67, phi_bb66_19, phi_bb66_23, phi_bb66_24, tmp100, tmp101, tmp99);
  }

  TNode<BoolT> phi_bb67_19;
  TNode<IntPtrT> phi_bb67_23;
  TNode<IntPtrT> phi_bb67_24;
  TNode<BoolT> phi_bb67_25;
  TNode<BoolT> phi_bb67_26;
  TNode<IntPtrT> phi_bb67_27;
  if (block67.is_used()) {
    ca_.Bind(&block67, &phi_bb67_19, &phi_bb67_23, &phi_bb67_24, &phi_bb67_25, &phi_bb67_26, &phi_bb67_27);
    ca_.Goto(&block64, phi_bb67_19, phi_bb67_23, phi_bb67_24, phi_bb67_25, phi_bb67_26, phi_bb67_27);
  }

  TNode<BoolT> phi_bb64_19;
  TNode<IntPtrT> phi_bb64_23;
  TNode<IntPtrT> phi_bb64_24;
  TNode<BoolT> phi_bb64_25;
  TNode<BoolT> phi_bb64_26;
  TNode<IntPtrT> phi_bb64_27;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_19, &phi_bb64_23, &phi_bb64_24, &phi_bb64_25, &phi_bb64_26, &phi_bb64_27);
    ca_.Goto(&block58, phi_bb64_19, phi_bb64_23, phi_bb64_24, phi_bb64_25, phi_bb64_26, phi_bb64_27);
  }

  TNode<BoolT> phi_bb58_19;
  TNode<IntPtrT> phi_bb58_23;
  TNode<IntPtrT> phi_bb58_24;
  TNode<BoolT> phi_bb58_25;
  TNode<BoolT> phi_bb58_26;
  TNode<IntPtrT> phi_bb58_27;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_19, &phi_bb58_23, &phi_bb58_24, &phi_bb58_25, &phi_bb58_26, &phi_bb58_27);
    ca_.Goto(&block57, phi_bb58_19, phi_bb58_23, phi_bb58_24, phi_bb58_25, phi_bb58_26, phi_bb58_27);
  }

  TNode<BoolT> phi_bb56_19;
  TNode<IntPtrT> phi_bb56_23;
  TNode<IntPtrT> phi_bb56_24;
  TNode<BoolT> phi_bb56_25;
  TNode<BoolT> phi_bb56_26;
  TNode<IntPtrT> phi_bb56_27;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_19, &phi_bb56_23, &phi_bb56_24, &phi_bb56_25, &phi_bb56_26, &phi_bb56_27);
    ca_.Goto(&block57, phi_bb56_19, phi_bb56_23, phi_bb56_24, phi_bb56_25, phi_bb56_26, phi_bb56_27);
  }

  TNode<BoolT> phi_bb57_19;
  TNode<IntPtrT> phi_bb57_23;
  TNode<IntPtrT> phi_bb57_24;
  TNode<BoolT> phi_bb57_25;
  TNode<BoolT> phi_bb57_26;
  TNode<IntPtrT> phi_bb57_27;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_19, &phi_bb57_23, &phi_bb57_24, &phi_bb57_25, &phi_bb57_26, &phi_bb57_27);
    ca_.Goto(&block44, phi_bb57_19, phi_bb57_23, phi_bb57_24, phi_bb57_25, phi_bb57_26, phi_bb57_27);
  }

  TNode<BoolT> phi_bb43_19;
  TNode<IntPtrT> phi_bb43_23;
  TNode<IntPtrT> phi_bb43_24;
  TNode<BoolT> phi_bb43_25;
  TNode<BoolT> phi_bb43_26;
  TNode<IntPtrT> phi_bb43_27;
  TNode<Uint32T> tmp102;
  TNode<BoolT> tmp103;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_19, &phi_bb43_23, &phi_bb43_24, &phi_bb43_25, &phi_bb43_26, &phi_bb43_27);
    tmp102 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF32.raw_bit_field());
    tmp103 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp61}, TNode<Uint32T>{tmp102});
    ca_.Branch(tmp103, &block68, std::vector<compiler::Node*>{phi_bb43_19, phi_bb43_23, phi_bb43_24, phi_bb43_25, phi_bb43_26, phi_bb43_27}, &block69, std::vector<compiler::Node*>{phi_bb43_19, phi_bb43_23, phi_bb43_24, phi_bb43_25, phi_bb43_26, phi_bb43_27});
  }

  TNode<BoolT> phi_bb68_19;
  TNode<IntPtrT> phi_bb68_23;
  TNode<IntPtrT> phi_bb68_24;
  TNode<BoolT> phi_bb68_25;
  TNode<BoolT> phi_bb68_26;
  TNode<IntPtrT> phi_bb68_27;
  TNode<IntPtrT> tmp104;
  TNode<BoolT> tmp105;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_19, &phi_bb68_23, &phi_bb68_24, &phi_bb68_25, &phi_bb68_26, &phi_bb68_27);
    tmp104 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp105 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb68_24}, TNode<IntPtrT>{tmp104});
    ca_.Branch(tmp105, &block72, std::vector<compiler::Node*>{phi_bb68_19, phi_bb68_23, phi_bb68_24, phi_bb68_25, phi_bb68_26, phi_bb68_27}, &block73, std::vector<compiler::Node*>{phi_bb68_19, phi_bb68_23, phi_bb68_24, phi_bb68_25, phi_bb68_26, phi_bb68_27});
  }

  TNode<BoolT> phi_bb72_19;
  TNode<IntPtrT> phi_bb72_23;
  TNode<IntPtrT> phi_bb72_24;
  TNode<BoolT> phi_bb72_25;
  TNode<BoolT> phi_bb72_26;
  TNode<IntPtrT> phi_bb72_27;
  TNode<IntPtrT> tmp106;
  TNode<IntPtrT> tmp107;
  if (block72.is_used()) {
    ca_.Bind(&block72, &phi_bb72_19, &phi_bb72_23, &phi_bb72_24, &phi_bb72_25, &phi_bb72_26, &phi_bb72_27);
    tmp106 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp107 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb72_24}, TNode<IntPtrT>{tmp106});
    ca_.Goto(&block71, phi_bb72_19, phi_bb72_23, tmp107, phi_bb72_25, phi_bb72_26, phi_bb72_27);
  }

  TNode<BoolT> phi_bb73_19;
  TNode<IntPtrT> phi_bb73_23;
  TNode<IntPtrT> phi_bb73_24;
  TNode<BoolT> phi_bb73_25;
  TNode<BoolT> phi_bb73_26;
  TNode<IntPtrT> phi_bb73_27;
  if (block73.is_used()) {
    ca_.Bind(&block73, &phi_bb73_19, &phi_bb73_23, &phi_bb73_24, &phi_bb73_25, &phi_bb73_26, &phi_bb73_27);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block75, phi_bb73_19, phi_bb73_23, phi_bb73_24, phi_bb73_25, phi_bb73_26, phi_bb73_27);
    } else {
      ca_.Goto(&block76, phi_bb73_19, phi_bb73_23, phi_bb73_24, phi_bb73_25, phi_bb73_26, phi_bb73_27);
    }
  }

  TNode<BoolT> phi_bb75_19;
  TNode<IntPtrT> phi_bb75_23;
  TNode<IntPtrT> phi_bb75_24;
  TNode<BoolT> phi_bb75_25;
  TNode<BoolT> phi_bb75_26;
  TNode<IntPtrT> phi_bb75_27;
  TNode<IntPtrT> tmp108;
  TNode<IntPtrT> tmp109;
  if (block75.is_used()) {
    ca_.Bind(&block75, &phi_bb75_19, &phi_bb75_23, &phi_bb75_24, &phi_bb75_25, &phi_bb75_26, &phi_bb75_27);
    tmp108 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp109 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb75_27}, TNode<IntPtrT>{tmp108});
    ca_.Goto(&block77, phi_bb75_19, phi_bb75_23, phi_bb75_24, phi_bb75_25, phi_bb75_26, tmp109);
  }

  TNode<BoolT> phi_bb76_19;
  TNode<IntPtrT> phi_bb76_23;
  TNode<IntPtrT> phi_bb76_24;
  TNode<BoolT> phi_bb76_25;
  TNode<BoolT> phi_bb76_26;
  TNode<IntPtrT> phi_bb76_27;
  if (block76.is_used()) {
    ca_.Bind(&block76, &phi_bb76_19, &phi_bb76_23, &phi_bb76_24, &phi_bb76_25, &phi_bb76_26, &phi_bb76_27);
    ca_.Branch(phi_bb76_25, &block78, std::vector<compiler::Node*>{phi_bb76_19, phi_bb76_23, phi_bb76_24, phi_bb76_25, phi_bb76_26, phi_bb76_27}, &block79, std::vector<compiler::Node*>{phi_bb76_19, phi_bb76_23, phi_bb76_24, phi_bb76_25, phi_bb76_26, phi_bb76_27});
  }

  TNode<BoolT> phi_bb78_19;
  TNode<IntPtrT> phi_bb78_23;
  TNode<IntPtrT> phi_bb78_24;
  TNode<BoolT> phi_bb78_25;
  TNode<BoolT> phi_bb78_26;
  TNode<IntPtrT> phi_bb78_27;
  TNode<BoolT> tmp110;
  TNode<BoolT> tmp111;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_19, &phi_bb78_23, &phi_bb78_24, &phi_bb78_25, &phi_bb78_26, &phi_bb78_27);
    tmp110 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp111 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block80, phi_bb78_19, phi_bb78_23, phi_bb78_24, tmp110, tmp111, phi_bb78_27);
  }

  TNode<BoolT> phi_bb79_19;
  TNode<IntPtrT> phi_bb79_23;
  TNode<IntPtrT> phi_bb79_24;
  TNode<BoolT> phi_bb79_25;
  TNode<BoolT> phi_bb79_26;
  TNode<IntPtrT> phi_bb79_27;
  TNode<IntPtrT> tmp112;
  TNode<IntPtrT> tmp113;
  TNode<BoolT> tmp114;
  TNode<BoolT> tmp115;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_19, &phi_bb79_23, &phi_bb79_24, &phi_bb79_25, &phi_bb79_26, &phi_bb79_27);
    tmp112 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp113 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb79_27}, TNode<IntPtrT>{tmp112});
    tmp114 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp115 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block80, phi_bb79_19, phi_bb79_23, phi_bb79_24, tmp114, tmp115, tmp113);
  }

  TNode<BoolT> phi_bb80_19;
  TNode<IntPtrT> phi_bb80_23;
  TNode<IntPtrT> phi_bb80_24;
  TNode<BoolT> phi_bb80_25;
  TNode<BoolT> phi_bb80_26;
  TNode<IntPtrT> phi_bb80_27;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_19, &phi_bb80_23, &phi_bb80_24, &phi_bb80_25, &phi_bb80_26, &phi_bb80_27);
    ca_.Goto(&block77, phi_bb80_19, phi_bb80_23, phi_bb80_24, phi_bb80_25, phi_bb80_26, phi_bb80_27);
  }

  TNode<BoolT> phi_bb77_19;
  TNode<IntPtrT> phi_bb77_23;
  TNode<IntPtrT> phi_bb77_24;
  TNode<BoolT> phi_bb77_25;
  TNode<BoolT> phi_bb77_26;
  TNode<IntPtrT> phi_bb77_27;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_19, &phi_bb77_23, &phi_bb77_24, &phi_bb77_25, &phi_bb77_26, &phi_bb77_27);
    ca_.Goto(&block71, phi_bb77_19, phi_bb77_23, phi_bb77_24, phi_bb77_25, phi_bb77_26, phi_bb77_27);
  }

  TNode<BoolT> phi_bb71_19;
  TNode<IntPtrT> phi_bb71_23;
  TNode<IntPtrT> phi_bb71_24;
  TNode<BoolT> phi_bb71_25;
  TNode<BoolT> phi_bb71_26;
  TNode<IntPtrT> phi_bb71_27;
  if (block71.is_used()) {
    ca_.Bind(&block71, &phi_bb71_19, &phi_bb71_23, &phi_bb71_24, &phi_bb71_25, &phi_bb71_26, &phi_bb71_27);
    ca_.Goto(&block70, phi_bb71_19, phi_bb71_23, phi_bb71_24, phi_bb71_25, phi_bb71_26, phi_bb71_27);
  }

  TNode<BoolT> phi_bb69_19;
  TNode<IntPtrT> phi_bb69_23;
  TNode<IntPtrT> phi_bb69_24;
  TNode<BoolT> phi_bb69_25;
  TNode<BoolT> phi_bb69_26;
  TNode<IntPtrT> phi_bb69_27;
  TNode<Uint32T> tmp116;
  TNode<BoolT> tmp117;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_19, &phi_bb69_23, &phi_bb69_24, &phi_bb69_25, &phi_bb69_26, &phi_bb69_27);
    tmp116 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF64.raw_bit_field());
    tmp117 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp61}, TNode<Uint32T>{tmp116});
    ca_.Branch(tmp117, &block81, std::vector<compiler::Node*>{phi_bb69_19, phi_bb69_23, phi_bb69_24, phi_bb69_25, phi_bb69_26, phi_bb69_27}, &block82, std::vector<compiler::Node*>{phi_bb69_19, phi_bb69_23, phi_bb69_24, phi_bb69_25, phi_bb69_26, phi_bb69_27});
  }

  TNode<BoolT> phi_bb81_19;
  TNode<IntPtrT> phi_bb81_23;
  TNode<IntPtrT> phi_bb81_24;
  TNode<BoolT> phi_bb81_25;
  TNode<BoolT> phi_bb81_26;
  TNode<IntPtrT> phi_bb81_27;
  TNode<IntPtrT> tmp118;
  TNode<BoolT> tmp119;
  if (block81.is_used()) {
    ca_.Bind(&block81, &phi_bb81_19, &phi_bb81_23, &phi_bb81_24, &phi_bb81_25, &phi_bb81_26, &phi_bb81_27);
    tmp118 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp119 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb81_24}, TNode<IntPtrT>{tmp118});
    ca_.Branch(tmp119, &block85, std::vector<compiler::Node*>{phi_bb81_19, phi_bb81_23, phi_bb81_24, phi_bb81_25, phi_bb81_26, phi_bb81_27}, &block86, std::vector<compiler::Node*>{phi_bb81_19, phi_bb81_23, phi_bb81_24, phi_bb81_25, phi_bb81_26, phi_bb81_27});
  }

  TNode<BoolT> phi_bb85_19;
  TNode<IntPtrT> phi_bb85_23;
  TNode<IntPtrT> phi_bb85_24;
  TNode<BoolT> phi_bb85_25;
  TNode<BoolT> phi_bb85_26;
  TNode<IntPtrT> phi_bb85_27;
  TNode<IntPtrT> tmp120;
  TNode<IntPtrT> tmp121;
  if (block85.is_used()) {
    ca_.Bind(&block85, &phi_bb85_19, &phi_bb85_23, &phi_bb85_24, &phi_bb85_25, &phi_bb85_26, &phi_bb85_27);
    tmp120 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp121 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb85_24}, TNode<IntPtrT>{tmp120});
    ca_.Goto(&block84, phi_bb85_19, phi_bb85_23, tmp121, phi_bb85_25, phi_bb85_26, phi_bb85_27);
  }

  TNode<BoolT> phi_bb86_19;
  TNode<IntPtrT> phi_bb86_23;
  TNode<IntPtrT> phi_bb86_24;
  TNode<BoolT> phi_bb86_25;
  TNode<BoolT> phi_bb86_26;
  TNode<IntPtrT> phi_bb86_27;
  if (block86.is_used()) {
    ca_.Bind(&block86, &phi_bb86_19, &phi_bb86_23, &phi_bb86_24, &phi_bb86_25, &phi_bb86_26, &phi_bb86_27);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block87, phi_bb86_19, phi_bb86_23, phi_bb86_24, phi_bb86_25, phi_bb86_26, phi_bb86_27);
    } else {
      ca_.Goto(&block88, phi_bb86_19, phi_bb86_23, phi_bb86_24, phi_bb86_25, phi_bb86_26, phi_bb86_27);
    }
  }

  TNode<BoolT> phi_bb87_19;
  TNode<IntPtrT> phi_bb87_23;
  TNode<IntPtrT> phi_bb87_24;
  TNode<BoolT> phi_bb87_25;
  TNode<BoolT> phi_bb87_26;
  TNode<IntPtrT> phi_bb87_27;
  TNode<IntPtrT> tmp122;
  TNode<IntPtrT> tmp123;
  if (block87.is_used()) {
    ca_.Bind(&block87, &phi_bb87_19, &phi_bb87_23, &phi_bb87_24, &phi_bb87_25, &phi_bb87_26, &phi_bb87_27);
    tmp122 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp123 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb87_27}, TNode<IntPtrT>{tmp122});
    ca_.Goto(&block89, phi_bb87_19, phi_bb87_23, phi_bb87_24, phi_bb87_25, phi_bb87_26, tmp123);
  }

  TNode<BoolT> phi_bb88_19;
  TNode<IntPtrT> phi_bb88_23;
  TNode<IntPtrT> phi_bb88_24;
  TNode<BoolT> phi_bb88_25;
  TNode<BoolT> phi_bb88_26;
  TNode<IntPtrT> phi_bb88_27;
  TNode<IntPtrT> tmp124;
  TNode<IntPtrT> tmp125;
  TNode<BoolT> tmp126;
  if (block88.is_used()) {
    ca_.Bind(&block88, &phi_bb88_19, &phi_bb88_23, &phi_bb88_24, &phi_bb88_25, &phi_bb88_26, &phi_bb88_27);
    tmp124 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp125 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb88_27}, TNode<IntPtrT>{tmp124});
    tmp126 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block89, phi_bb88_19, phi_bb88_23, phi_bb88_24, phi_bb88_25, tmp126, tmp125);
  }

  TNode<BoolT> phi_bb89_19;
  TNode<IntPtrT> phi_bb89_23;
  TNode<IntPtrT> phi_bb89_24;
  TNode<BoolT> phi_bb89_25;
  TNode<BoolT> phi_bb89_26;
  TNode<IntPtrT> phi_bb89_27;
  if (block89.is_used()) {
    ca_.Bind(&block89, &phi_bb89_19, &phi_bb89_23, &phi_bb89_24, &phi_bb89_25, &phi_bb89_26, &phi_bb89_27);
    ca_.Goto(&block84, phi_bb89_19, phi_bb89_23, phi_bb89_24, phi_bb89_25, phi_bb89_26, phi_bb89_27);
  }

  TNode<BoolT> phi_bb84_19;
  TNode<IntPtrT> phi_bb84_23;
  TNode<IntPtrT> phi_bb84_24;
  TNode<BoolT> phi_bb84_25;
  TNode<BoolT> phi_bb84_26;
  TNode<IntPtrT> phi_bb84_27;
  if (block84.is_used()) {
    ca_.Bind(&block84, &phi_bb84_19, &phi_bb84_23, &phi_bb84_24, &phi_bb84_25, &phi_bb84_26, &phi_bb84_27);
    ca_.Goto(&block83, phi_bb84_19, phi_bb84_23, phi_bb84_24, phi_bb84_25, phi_bb84_26, phi_bb84_27);
  }

  TNode<BoolT> phi_bb82_19;
  TNode<IntPtrT> phi_bb82_23;
  TNode<IntPtrT> phi_bb82_24;
  TNode<BoolT> phi_bb82_25;
  TNode<BoolT> phi_bb82_26;
  TNode<IntPtrT> phi_bb82_27;
  TNode<Uint32T> tmp127;
  TNode<Uint32T> tmp128;
  TNode<Uint32T> tmp129;
  TNode<BoolT> tmp130;
  if (block82.is_used()) {
    ca_.Bind(&block82, &phi_bb82_19, &phi_bb82_23, &phi_bb82_24, &phi_bb82_25, &phi_bb82_26, &phi_bb82_27);
    tmp127 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp128 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp61}, TNode<Uint32T>{tmp127});
    tmp129 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp130 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp128}, TNode<Uint32T>{tmp129});
    ca_.Branch(tmp130, &block90, std::vector<compiler::Node*>{phi_bb82_19, phi_bb82_23, phi_bb82_24, phi_bb82_25, phi_bb82_26, phi_bb82_27}, &block91, std::vector<compiler::Node*>{phi_bb82_19, phi_bb82_23, phi_bb82_24, phi_bb82_25, phi_bb82_26, phi_bb82_27});
  }

  TNode<BoolT> phi_bb91_19;
  TNode<IntPtrT> phi_bb91_23;
  TNode<IntPtrT> phi_bb91_24;
  TNode<BoolT> phi_bb91_25;
  TNode<BoolT> phi_bb91_26;
  TNode<IntPtrT> phi_bb91_27;
  if (block91.is_used()) {
    ca_.Bind(&block91, &phi_bb91_19, &phi_bb91_23, &phi_bb91_24, &phi_bb91_25, &phi_bb91_26, &phi_bb91_27);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/js-to-wasm.tq", 570});
      CodeStubAssembler(state_).FailAssert("Torque assert '(retType & kValueTypeIsRefBit) != 0' failed", pos_stack);
    }
  }

  TNode<BoolT> phi_bb90_19;
  TNode<IntPtrT> phi_bb90_23;
  TNode<IntPtrT> phi_bb90_24;
  TNode<BoolT> phi_bb90_25;
  TNode<BoolT> phi_bb90_26;
  TNode<IntPtrT> phi_bb90_27;
  TNode<BoolT> tmp131;
  if (block90.is_used()) {
    ca_.Bind(&block90, &phi_bb90_19, &phi_bb90_23, &phi_bb90_24, &phi_bb90_25, &phi_bb90_26, &phi_bb90_27);
    tmp131 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block83, tmp131, phi_bb90_23, phi_bb90_24, phi_bb90_25, phi_bb90_26, phi_bb90_27);
  }

  TNode<BoolT> phi_bb83_19;
  TNode<IntPtrT> phi_bb83_23;
  TNode<IntPtrT> phi_bb83_24;
  TNode<BoolT> phi_bb83_25;
  TNode<BoolT> phi_bb83_26;
  TNode<IntPtrT> phi_bb83_27;
  if (block83.is_used()) {
    ca_.Bind(&block83, &phi_bb83_19, &phi_bb83_23, &phi_bb83_24, &phi_bb83_25, &phi_bb83_26, &phi_bb83_27);
    ca_.Goto(&block70, phi_bb83_19, phi_bb83_23, phi_bb83_24, phi_bb83_25, phi_bb83_26, phi_bb83_27);
  }

  TNode<BoolT> phi_bb70_19;
  TNode<IntPtrT> phi_bb70_23;
  TNode<IntPtrT> phi_bb70_24;
  TNode<BoolT> phi_bb70_25;
  TNode<BoolT> phi_bb70_26;
  TNode<IntPtrT> phi_bb70_27;
  if (block70.is_used()) {
    ca_.Bind(&block70, &phi_bb70_19, &phi_bb70_23, &phi_bb70_24, &phi_bb70_25, &phi_bb70_26, &phi_bb70_27);
    ca_.Goto(&block44, phi_bb70_19, phi_bb70_23, phi_bb70_24, phi_bb70_25, phi_bb70_26, phi_bb70_27);
  }

  TNode<BoolT> phi_bb44_19;
  TNode<IntPtrT> phi_bb44_23;
  TNode<IntPtrT> phi_bb44_24;
  TNode<BoolT> phi_bb44_25;
  TNode<BoolT> phi_bb44_26;
  TNode<IntPtrT> phi_bb44_27;
  if (block44.is_used()) {
    ca_.Bind(&block44, &phi_bb44_19, &phi_bb44_23, &phi_bb44_24, &phi_bb44_25, &phi_bb44_26, &phi_bb44_27);
    ca_.Goto(&block31, phi_bb44_19, phi_bb44_23, phi_bb44_24, phi_bb44_25, phi_bb44_26, phi_bb44_27);
  }

  TNode<BoolT> phi_bb31_19;
  TNode<IntPtrT> phi_bb31_23;
  TNode<IntPtrT> phi_bb31_24;
  TNode<BoolT> phi_bb31_25;
  TNode<BoolT> phi_bb31_26;
  TNode<IntPtrT> phi_bb31_27;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_19, &phi_bb31_23, &phi_bb31_24, &phi_bb31_25, &phi_bb31_26, &phi_bb31_27);
    ca_.Goto(&block20, phi_bb31_19, phi_bb31_23, phi_bb31_24, phi_bb31_25, phi_bb31_26, phi_bb31_27, tmp60);
  }

  TNode<BoolT> phi_bb19_19;
  TNode<IntPtrT> phi_bb19_23;
  TNode<IntPtrT> phi_bb19_24;
  TNode<BoolT> phi_bb19_25;
  TNode<BoolT> phi_bb19_26;
  TNode<IntPtrT> phi_bb19_27;
  TNode<IntPtrT> phi_bb19_29;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_19, &phi_bb19_23, &phi_bb19_24, &phi_bb19_25, &phi_bb19_26, &phi_bb19_27, &phi_bb19_29);
    ca_.Branch(phi_bb19_19, &block92, std::vector<compiler::Node*>{phi_bb19_19, phi_bb19_23, phi_bb19_24, phi_bb19_25, phi_bb19_26, phi_bb19_27, phi_bb19_29}, &block93, std::vector<compiler::Node*>{phi_bb19_19, phi_bb19_23, phi_bb19_24, phi_bb19_25, phi_bb19_26, phi_bb19_27, phi_bb19_29, tmp54});
  }

  TNode<BoolT> phi_bb92_19;
  TNode<IntPtrT> phi_bb92_23;
  TNode<IntPtrT> phi_bb92_24;
  TNode<BoolT> phi_bb92_25;
  TNode<BoolT> phi_bb92_26;
  TNode<IntPtrT> phi_bb92_27;
  TNode<IntPtrT> phi_bb92_29;
  TNode<BoolT> tmp132;
  if (block92.is_used()) {
    ca_.Bind(&block92, &phi_bb92_19, &phi_bb92_23, &phi_bb92_24, &phi_bb92_25, &phi_bb92_26, &phi_bb92_27, &phi_bb92_29);
    tmp132 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{phi_bb92_26});
    ca_.Branch(tmp132, &block95, std::vector<compiler::Node*>{phi_bb92_19, phi_bb92_23, phi_bb92_24, phi_bb92_25, phi_bb92_26, phi_bb92_27, phi_bb92_29}, &block96, std::vector<compiler::Node*>{phi_bb92_19, phi_bb92_23, phi_bb92_24, phi_bb92_25, phi_bb92_26, phi_bb92_27, phi_bb92_29});
  }

  TNode<BoolT> phi_bb95_19;
  TNode<IntPtrT> phi_bb95_23;
  TNode<IntPtrT> phi_bb95_24;
  TNode<BoolT> phi_bb95_25;
  TNode<BoolT> phi_bb95_26;
  TNode<IntPtrT> phi_bb95_27;
  TNode<IntPtrT> phi_bb95_29;
  TNode<BoolT> tmp133;
  if (block95.is_used()) {
    ca_.Bind(&block95, &phi_bb95_19, &phi_bb95_23, &phi_bb95_24, &phi_bb95_25, &phi_bb95_26, &phi_bb95_27, &phi_bb95_29);
    tmp133 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block96, phi_bb95_19, phi_bb95_23, phi_bb95_24, tmp133, phi_bb95_26, phi_bb95_27, phi_bb95_29);
  }

  TNode<BoolT> phi_bb96_19;
  TNode<IntPtrT> phi_bb96_23;
  TNode<IntPtrT> phi_bb96_24;
  TNode<BoolT> phi_bb96_25;
  TNode<BoolT> phi_bb96_26;
  TNode<IntPtrT> phi_bb96_27;
  TNode<IntPtrT> phi_bb96_29;
  TNode<IntPtrT> tmp134;
  TNode<IntPtrT> tmp135;
  if (block96.is_used()) {
    ca_.Bind(&block96, &phi_bb96_19, &phi_bb96_23, &phi_bb96_24, &phi_bb96_25, &phi_bb96_26, &phi_bb96_27, &phi_bb96_29);
    tmp134 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{tmp46});
    tmp135 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp45}, TNode<IntPtrT>{tmp134});
    ca_.Goto(&block100, phi_bb96_19, phi_bb96_23, phi_bb96_24, phi_bb96_25, phi_bb96_26, phi_bb96_27, tmp45);
  }

  TNode<BoolT> phi_bb100_19;
  TNode<IntPtrT> phi_bb100_23;
  TNode<IntPtrT> phi_bb100_24;
  TNode<BoolT> phi_bb100_25;
  TNode<BoolT> phi_bb100_26;
  TNode<IntPtrT> phi_bb100_27;
  TNode<IntPtrT> phi_bb100_29;
  TNode<BoolT> tmp136;
  TNode<BoolT> tmp137;
  if (block100.is_used()) {
    ca_.Bind(&block100, &phi_bb100_19, &phi_bb100_23, &phi_bb100_24, &phi_bb100_25, &phi_bb100_26, &phi_bb100_27, &phi_bb100_29);
    tmp136 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb100_29}, TNode<IntPtrT>{tmp135});
    tmp137 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp136});
    ca_.Branch(tmp137, &block98, std::vector<compiler::Node*>{phi_bb100_19, phi_bb100_23, phi_bb100_24, phi_bb100_25, phi_bb100_26, phi_bb100_27, phi_bb100_29}, &block99, std::vector<compiler::Node*>{phi_bb100_19, phi_bb100_23, phi_bb100_24, phi_bb100_25, phi_bb100_26, phi_bb100_27, phi_bb100_29});
  }

  TNode<BoolT> phi_bb98_19;
  TNode<IntPtrT> phi_bb98_23;
  TNode<IntPtrT> phi_bb98_24;
  TNode<BoolT> phi_bb98_25;
  TNode<BoolT> phi_bb98_26;
  TNode<IntPtrT> phi_bb98_27;
  TNode<IntPtrT> phi_bb98_29;
  TNode<Union<HeapObject, TaggedIndex>> tmp138;
  TNode<IntPtrT> tmp139;
  TNode<IntPtrT> tmp140;
  TNode<IntPtrT> tmp141;
  TNode<Uint32T> tmp142;
  TNode<Uint32T> tmp143;
  TNode<Uint32T> tmp144;
  TNode<Uint32T> tmp145;
  TNode<BoolT> tmp146;
  if (block98.is_used()) {
    ca_.Bind(&block98, &phi_bb98_19, &phi_bb98_23, &phi_bb98_24, &phi_bb98_25, &phi_bb98_26, &phi_bb98_27, &phi_bb98_29);
    std::tie(tmp138, tmp139) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp44}, TNode<IntPtrT>{phi_bb98_29}).Flatten();
    tmp140 = FromConstexpr_intptr_constexpr_int31_0(state_, kInt32Size);
    tmp141 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb98_29}, TNode<IntPtrT>{tmp140});
    tmp142 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp138, tmp139});
    tmp143 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp144 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp142}, TNode<Uint32T>{tmp143});
    tmp145 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp146 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp144}, TNode<Uint32T>{tmp145});
    ca_.Branch(tmp146, &block109, std::vector<compiler::Node*>{phi_bb98_19, phi_bb98_23, phi_bb98_24, phi_bb98_25, phi_bb98_26, phi_bb98_27}, &block110, std::vector<compiler::Node*>{phi_bb98_19, phi_bb98_23, phi_bb98_24, phi_bb98_25, phi_bb98_26, phi_bb98_27});
  }

  TNode<BoolT> phi_bb109_19;
  TNode<IntPtrT> phi_bb109_23;
  TNode<IntPtrT> phi_bb109_24;
  TNode<BoolT> phi_bb109_25;
  TNode<BoolT> phi_bb109_26;
  TNode<IntPtrT> phi_bb109_27;
  TNode<IntPtrT> tmp147;
  TNode<BoolT> tmp148;
  if (block109.is_used()) {
    ca_.Bind(&block109, &phi_bb109_19, &phi_bb109_23, &phi_bb109_24, &phi_bb109_25, &phi_bb109_26, &phi_bb109_27);
    tmp147 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp148 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb109_23}, TNode<IntPtrT>{tmp147});
    ca_.Branch(tmp148, &block112, std::vector<compiler::Node*>{phi_bb109_19, phi_bb109_23, phi_bb109_24, phi_bb109_25, phi_bb109_26, phi_bb109_27}, &block113, std::vector<compiler::Node*>{phi_bb109_19, phi_bb109_23, phi_bb109_24, phi_bb109_25, phi_bb109_26, phi_bb109_27});
  }

  TNode<BoolT> phi_bb112_19;
  TNode<IntPtrT> phi_bb112_23;
  TNode<IntPtrT> phi_bb112_24;
  TNode<BoolT> phi_bb112_25;
  TNode<BoolT> phi_bb112_26;
  TNode<IntPtrT> phi_bb112_27;
  TNode<IntPtrT> tmp149;
  TNode<IntPtrT> tmp150;
  if (block112.is_used()) {
    ca_.Bind(&block112, &phi_bb112_19, &phi_bb112_23, &phi_bb112_24, &phi_bb112_25, &phi_bb112_26, &phi_bb112_27);
    tmp149 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp150 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb112_23}, TNode<IntPtrT>{tmp149});
    ca_.Goto(&block111, phi_bb112_19, tmp150, phi_bb112_24, phi_bb112_25, phi_bb112_26, phi_bb112_27);
  }

  TNode<BoolT> phi_bb113_19;
  TNode<IntPtrT> phi_bb113_23;
  TNode<IntPtrT> phi_bb113_24;
  TNode<BoolT> phi_bb113_25;
  TNode<BoolT> phi_bb113_26;
  TNode<IntPtrT> phi_bb113_27;
  if (block113.is_used()) {
    ca_.Bind(&block113, &phi_bb113_19, &phi_bb113_23, &phi_bb113_24, &phi_bb113_25, &phi_bb113_26, &phi_bb113_27);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block115, phi_bb113_19, phi_bb113_23, phi_bb113_24, phi_bb113_25, phi_bb113_26, phi_bb113_27);
    } else {
      ca_.Goto(&block116, phi_bb113_19, phi_bb113_23, phi_bb113_24, phi_bb113_25, phi_bb113_26, phi_bb113_27);
    }
  }

  TNode<BoolT> phi_bb115_19;
  TNode<IntPtrT> phi_bb115_23;
  TNode<IntPtrT> phi_bb115_24;
  TNode<BoolT> phi_bb115_25;
  TNode<BoolT> phi_bb115_26;
  TNode<IntPtrT> phi_bb115_27;
  TNode<IntPtrT> tmp151;
  TNode<IntPtrT> tmp152;
  if (block115.is_used()) {
    ca_.Bind(&block115, &phi_bb115_19, &phi_bb115_23, &phi_bb115_24, &phi_bb115_25, &phi_bb115_26, &phi_bb115_27);
    tmp151 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp152 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb115_27}, TNode<IntPtrT>{tmp151});
    ca_.Goto(&block117, phi_bb115_19, phi_bb115_23, phi_bb115_24, phi_bb115_25, phi_bb115_26, tmp152);
  }

  TNode<BoolT> phi_bb116_19;
  TNode<IntPtrT> phi_bb116_23;
  TNode<IntPtrT> phi_bb116_24;
  TNode<BoolT> phi_bb116_25;
  TNode<BoolT> phi_bb116_26;
  TNode<IntPtrT> phi_bb116_27;
  if (block116.is_used()) {
    ca_.Bind(&block116, &phi_bb116_19, &phi_bb116_23, &phi_bb116_24, &phi_bb116_25, &phi_bb116_26, &phi_bb116_27);
    ca_.Branch(phi_bb116_25, &block118, std::vector<compiler::Node*>{phi_bb116_19, phi_bb116_23, phi_bb116_24, phi_bb116_25, phi_bb116_26, phi_bb116_27}, &block119, std::vector<compiler::Node*>{phi_bb116_19, phi_bb116_23, phi_bb116_24, phi_bb116_25, phi_bb116_26, phi_bb116_27});
  }

  TNode<BoolT> phi_bb118_19;
  TNode<IntPtrT> phi_bb118_23;
  TNode<IntPtrT> phi_bb118_24;
  TNode<BoolT> phi_bb118_25;
  TNode<BoolT> phi_bb118_26;
  TNode<IntPtrT> phi_bb118_27;
  TNode<BoolT> tmp153;
  TNode<BoolT> tmp154;
  if (block118.is_used()) {
    ca_.Bind(&block118, &phi_bb118_19, &phi_bb118_23, &phi_bb118_24, &phi_bb118_25, &phi_bb118_26, &phi_bb118_27);
    tmp153 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp154 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block120, phi_bb118_19, phi_bb118_23, phi_bb118_24, tmp153, tmp154, phi_bb118_27);
  }

  TNode<BoolT> phi_bb119_19;
  TNode<IntPtrT> phi_bb119_23;
  TNode<IntPtrT> phi_bb119_24;
  TNode<BoolT> phi_bb119_25;
  TNode<BoolT> phi_bb119_26;
  TNode<IntPtrT> phi_bb119_27;
  TNode<IntPtrT> tmp155;
  TNode<IntPtrT> tmp156;
  TNode<BoolT> tmp157;
  TNode<BoolT> tmp158;
  if (block119.is_used()) {
    ca_.Bind(&block119, &phi_bb119_19, &phi_bb119_23, &phi_bb119_24, &phi_bb119_25, &phi_bb119_26, &phi_bb119_27);
    tmp155 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp156 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb119_27}, TNode<IntPtrT>{tmp155});
    tmp157 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp158 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block120, phi_bb119_19, phi_bb119_23, phi_bb119_24, tmp157, tmp158, tmp156);
  }

  TNode<BoolT> phi_bb120_19;
  TNode<IntPtrT> phi_bb120_23;
  TNode<IntPtrT> phi_bb120_24;
  TNode<BoolT> phi_bb120_25;
  TNode<BoolT> phi_bb120_26;
  TNode<IntPtrT> phi_bb120_27;
  if (block120.is_used()) {
    ca_.Bind(&block120, &phi_bb120_19, &phi_bb120_23, &phi_bb120_24, &phi_bb120_25, &phi_bb120_26, &phi_bb120_27);
    ca_.Goto(&block117, phi_bb120_19, phi_bb120_23, phi_bb120_24, phi_bb120_25, phi_bb120_26, phi_bb120_27);
  }

  TNode<BoolT> phi_bb117_19;
  TNode<IntPtrT> phi_bb117_23;
  TNode<IntPtrT> phi_bb117_24;
  TNode<BoolT> phi_bb117_25;
  TNode<BoolT> phi_bb117_26;
  TNode<IntPtrT> phi_bb117_27;
  if (block117.is_used()) {
    ca_.Bind(&block117, &phi_bb117_19, &phi_bb117_23, &phi_bb117_24, &phi_bb117_25, &phi_bb117_26, &phi_bb117_27);
    ca_.Goto(&block111, phi_bb117_19, phi_bb117_23, phi_bb117_24, phi_bb117_25, phi_bb117_26, phi_bb117_27);
  }

  TNode<BoolT> phi_bb111_19;
  TNode<IntPtrT> phi_bb111_23;
  TNode<IntPtrT> phi_bb111_24;
  TNode<BoolT> phi_bb111_25;
  TNode<BoolT> phi_bb111_26;
  TNode<IntPtrT> phi_bb111_27;
  if (block111.is_used()) {
    ca_.Bind(&block111, &phi_bb111_19, &phi_bb111_23, &phi_bb111_24, &phi_bb111_25, &phi_bb111_26, &phi_bb111_27);
    ca_.Goto(&block110, phi_bb111_19, phi_bb111_23, phi_bb111_24, phi_bb111_25, phi_bb111_26, phi_bb111_27);
  }

  TNode<BoolT> phi_bb110_19;
  TNode<IntPtrT> phi_bb110_23;
  TNode<IntPtrT> phi_bb110_24;
  TNode<BoolT> phi_bb110_25;
  TNode<BoolT> phi_bb110_26;
  TNode<IntPtrT> phi_bb110_27;
  if (block110.is_used()) {
    ca_.Bind(&block110, &phi_bb110_19, &phi_bb110_23, &phi_bb110_24, &phi_bb110_25, &phi_bb110_26, &phi_bb110_27);
    ca_.Goto(&block100, phi_bb110_19, phi_bb110_23, phi_bb110_24, phi_bb110_25, phi_bb110_26, phi_bb110_27, tmp141);
  }

  TNode<BoolT> phi_bb99_19;
  TNode<IntPtrT> phi_bb99_23;
  TNode<IntPtrT> phi_bb99_24;
  TNode<BoolT> phi_bb99_25;
  TNode<BoolT> phi_bb99_26;
  TNode<IntPtrT> phi_bb99_27;
  TNode<IntPtrT> phi_bb99_29;
  if (block99.is_used()) {
    ca_.Bind(&block99, &phi_bb99_19, &phi_bb99_23, &phi_bb99_24, &phi_bb99_25, &phi_bb99_26, &phi_bb99_27, &phi_bb99_29);
    ca_.Goto(&block93, phi_bb99_19, phi_bb99_23, phi_bb99_24, phi_bb99_25, phi_bb99_26, phi_bb99_27, phi_bb99_29, tmp135);
  }

  TNode<BoolT> phi_bb93_19;
  TNode<IntPtrT> phi_bb93_23;
  TNode<IntPtrT> phi_bb93_24;
  TNode<BoolT> phi_bb93_25;
  TNode<BoolT> phi_bb93_26;
  TNode<IntPtrT> phi_bb93_27;
  TNode<IntPtrT> phi_bb93_29;
  TNode<IntPtrT> phi_bb93_30;
  if (block93.is_used()) {
    ca_.Bind(&block93, &phi_bb93_19, &phi_bb93_23, &phi_bb93_24, &phi_bb93_25, &phi_bb93_26, &phi_bb93_27, &phi_bb93_29, &phi_bb93_30);
    ca_.Branch(phi_bb93_26, &block122, std::vector<compiler::Node*>{phi_bb93_19, phi_bb93_23, phi_bb93_24, phi_bb93_25, phi_bb93_26, phi_bb93_27, phi_bb93_29, phi_bb93_30}, &block123, std::vector<compiler::Node*>{phi_bb93_19, phi_bb93_23, phi_bb93_24, phi_bb93_25, phi_bb93_26, phi_bb93_27, phi_bb93_29, phi_bb93_30});
  }

  TNode<BoolT> phi_bb122_19;
  TNode<IntPtrT> phi_bb122_23;
  TNode<IntPtrT> phi_bb122_24;
  TNode<BoolT> phi_bb122_25;
  TNode<BoolT> phi_bb122_26;
  TNode<IntPtrT> phi_bb122_27;
  TNode<IntPtrT> phi_bb122_29;
  TNode<IntPtrT> phi_bb122_30;
  TNode<IntPtrT> tmp159;
  TNode<IntPtrT> tmp160;
  if (block122.is_used()) {
    ca_.Bind(&block122, &phi_bb122_19, &phi_bb122_23, &phi_bb122_24, &phi_bb122_25, &phi_bb122_26, &phi_bb122_27, &phi_bb122_29, &phi_bb122_30);
    tmp159 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp160 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb122_27}, TNode<IntPtrT>{tmp159});
    ca_.Goto(&block121, phi_bb122_19, phi_bb122_23, phi_bb122_24, phi_bb122_25, phi_bb122_26, phi_bb122_27, phi_bb122_29, phi_bb122_30, tmp160);
  }

  TNode<BoolT> phi_bb123_19;
  TNode<IntPtrT> phi_bb123_23;
  TNode<IntPtrT> phi_bb123_24;
  TNode<BoolT> phi_bb123_25;
  TNode<BoolT> phi_bb123_26;
  TNode<IntPtrT> phi_bb123_27;
  TNode<IntPtrT> phi_bb123_29;
  TNode<IntPtrT> phi_bb123_30;
  if (block123.is_used()) {
    ca_.Bind(&block123, &phi_bb123_19, &phi_bb123_23, &phi_bb123_24, &phi_bb123_25, &phi_bb123_26, &phi_bb123_27, &phi_bb123_29, &phi_bb123_30);
    ca_.Goto(&block121, phi_bb123_19, phi_bb123_23, phi_bb123_24, phi_bb123_25, phi_bb123_26, phi_bb123_27, phi_bb123_29, phi_bb123_30, phi_bb123_27);
  }

  TNode<BoolT> phi_bb121_19;
  TNode<IntPtrT> phi_bb121_23;
  TNode<IntPtrT> phi_bb121_24;
  TNode<BoolT> phi_bb121_25;
  TNode<BoolT> phi_bb121_26;
  TNode<IntPtrT> phi_bb121_27;
  TNode<IntPtrT> phi_bb121_29;
  TNode<IntPtrT> phi_bb121_30;
  TNode<IntPtrT> phi_bb121_31;
  if (block121.is_used()) {
    ca_.Bind(&block121, &phi_bb121_19, &phi_bb121_23, &phi_bb121_24, &phi_bb121_25, &phi_bb121_26, &phi_bb121_27, &phi_bb121_29, &phi_bb121_30, &phi_bb121_31);
    ca_.Goto(&block12, tmp42, phi_bb121_31, phi_bb121_19);
  }

  TNode<JSAny> phi_bb12_17;
  TNode<IntPtrT> phi_bb12_18;
  TNode<BoolT> phi_bb12_19;
  TNode<Union<HeapObject, TaggedIndex>> tmp161;
  TNode<IntPtrT> tmp162;
  TNode<IntPtrT> tmp163;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_17, &phi_bb12_18, &phi_bb12_19);
    compiler::CodeAssemblerLabel label164(&ca_);
    std::tie(tmp161, tmp162, tmp163) = Subslice_uint32_0(state_, TorqueStructSlice_uint32_ConstReference_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp33}, TNode<IntPtrT>{tmp34}, TNode<IntPtrT>{tmp35}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{tmp26}, TNode<IntPtrT>{tmp22}, &label164).Flatten();
    ca_.Goto(&block127, phi_bb12_17, phi_bb12_18, phi_bb12_19);
    if (label164.is_used()) {
      ca_.Bind(&label164);
      ca_.Goto(&block128, phi_bb12_17, phi_bb12_18, phi_bb12_19);
    }
  }

  TNode<JSAny> phi_bb128_17;
  TNode<IntPtrT> phi_bb128_18;
  TNode<BoolT> phi_bb128_19;
  if (block128.is_used()) {
    ca_.Bind(&block128, &phi_bb128_17, &phi_bb128_18, &phi_bb128_19);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb127_17;
  TNode<IntPtrT> phi_bb127_18;
  TNode<BoolT> phi_bb127_19;
  TNode<IntPtrT> tmp165;
  TNode<BoolT> tmp166;
  if (block127.is_used()) {
    ca_.Bind(&block127, &phi_bb127_17, &phi_bb127_18, &phi_bb127_19);
    tmp165 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0xaull));
    tmp166 = CodeStubAssembler(state_).IntPtrLessThanOrEqual(TNode<IntPtrT>{tmp22}, TNode<IntPtrT>{tmp165});
    ca_.Branch(tmp166, &block129, std::vector<compiler::Node*>{phi_bb127_17, phi_bb127_18, phi_bb127_19}, &block130, std::vector<compiler::Node*>{phi_bb127_17, phi_bb127_18, phi_bb127_19});
  }

  TNode<JSAny> phi_bb129_17;
  TNode<IntPtrT> phi_bb129_18;
  TNode<BoolT> phi_bb129_19;
  TNode<RawPtrT> tmp167;
  TNode<RawPtrT> tmp168;
  TNode<IntPtrT> tmp169;
  TNode<Union<HeapObject, TaggedIndex>> tmp170;
  TNode<IntPtrT> tmp171;
  if (block129.is_used()) {
    ca_.Bind(&block129, &phi_bb129_17, &phi_bb129_18, &phi_bb129_19);
    tmp167 = CodeStubAssembler(state_).StackSlotPtr(CastIfEnumClass<int32_t>((CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, (CodeStubAssembler(state_).ConstexprIntegerLiteralAdd((CodeStubAssembler(state_).ConstexprIntegerLiteralAdd(IntegerLiteral(false, 0xaull), IntegerLiteral(false, 0x8ull))), IntegerLiteral(false, 0x2ull))))), (SizeOf_float64_0(state_))))), CastIfEnumClass<int32_t>((SizeOf_float64_0(state_))));
    tmp168 = (TNode<RawPtrT>{tmp167});
    tmp169 = Convert_intptr_constexpr_int31_0(state_, (CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, (CodeStubAssembler(state_).ConstexprIntegerLiteralAdd((CodeStubAssembler(state_).ConstexprIntegerLiteralAdd(IntegerLiteral(false, 0xaull), IntegerLiteral(false, 0x8ull))), IntegerLiteral(false, 0x2ull))))), (SizeOf_float64_0(state_)))));
    std::tie(tmp170, tmp171) = NewOffHeapReference_intptr_0(state_, TNode<RawPtrT>{tmp168}).Flatten();
    ca_.Goto(&block131, phi_bb129_17, phi_bb129_18, phi_bb129_19, tmp170, tmp171, tmp169);
  }

  TNode<JSAny> phi_bb130_17;
  TNode<IntPtrT> phi_bb130_18;
  TNode<BoolT> phi_bb130_19;
  TNode<IntPtrT> tmp172;
  TNode<IntPtrT> tmp173;
  TNode<IntPtrT> tmp174;
  TNode<IntPtrT> tmp175;
  TNode<IntPtrT> tmp176;
  TNode<IntPtrT> tmp177;
  TNode<RawPtrT> tmp178;
  TNode<Union<HeapObject, TaggedIndex>> tmp179;
  TNode<IntPtrT> tmp180;
  if (block130.is_used()) {
    ca_.Bind(&block130, &phi_bb130_17, &phi_bb130_18, &phi_bb130_19);
    tmp172 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp173 = CodeStubAssembler(state_).UniqueIntPtrConstant(arraysize(wasm::kFpParamRegisters));
    tmp174 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp173}, TNode<IntPtrT>{tmp22});
    tmp175 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp176 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp174}, TNode<IntPtrT>{tmp175});
    tmp177 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp176}, TNode<IntPtrT>{tmp172});
    tmp178 = CodeStubAssembler(state_).AllocateBuffer(TNode<IntPtrT>{tmp177});
    std::tie(tmp179, tmp180) = NewOffHeapReference_intptr_0(state_, TNode<RawPtrT>{tmp178}).Flatten();
    ca_.Goto(&block131, phi_bb130_17, phi_bb130_18, phi_bb130_19, tmp179, tmp180, tmp177);
  }

  TNode<JSAny> phi_bb131_17;
  TNode<IntPtrT> phi_bb131_18;
  TNode<BoolT> phi_bb131_19;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb131_23;
  TNode<IntPtrT> phi_bb131_24;
  TNode<IntPtrT> phi_bb131_25;
  TNode<Union<HeapObject, TaggedIndex>> tmp181;
  TNode<IntPtrT> tmp182;
  TNode<IntPtrT> tmp183;
  TNode<IntPtrT> tmp184;
  TNode<IntPtrT> tmp185;
  TNode<IntPtrT> tmp186;
  TNode<IntPtrT> tmp187;
  TNode<IntPtrT> tmp188;
  TNode<IntPtrT> tmp189;
  TNode<BoolT> tmp190;
  TNode<BoolT> tmp191;
  TNode<Smi> tmp192;
  TNode<IntPtrT> tmp193;
  if (block131.is_used()) {
    ca_.Bind(&block131, &phi_bb131_17, &phi_bb131_18, &phi_bb131_19, &phi_bb131_23, &phi_bb131_24, &phi_bb131_25);
    std::tie(tmp181, tmp182, tmp183, tmp184, tmp185, tmp186, tmp187, tmp188, tmp189, tmp190) = LocationAllocatorForParams_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb131_23}, TNode<IntPtrT>{phi_bb131_24}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb131_25}).Flatten();
    tmp191 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp192 = Convert_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp193 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block142, phi_bb131_17, phi_bb131_18, phi_bb131_19, tmp182, tmp183, tmp184, tmp185, tmp186, tmp189, tmp190, tmp191, tmp192, tmp193);
  }

  TNode<JSAny> phi_bb142_17;
  TNode<IntPtrT> phi_bb142_18;
  TNode<BoolT> phi_bb142_19;
  TNode<IntPtrT> phi_bb142_27;
  TNode<IntPtrT> phi_bb142_28;
  TNode<IntPtrT> phi_bb142_29;
  TNode<IntPtrT> phi_bb142_30;
  TNode<IntPtrT> phi_bb142_31;
  TNode<IntPtrT> phi_bb142_34;
  TNode<BoolT> phi_bb142_35;
  TNode<BoolT> phi_bb142_36;
  TNode<Union<FixedArray, Smi>> phi_bb142_37;
  TNode<IntPtrT> phi_bb142_38;
  TNode<BoolT> tmp194;
  if (block142.is_used()) {
    ca_.Bind(&block142, &phi_bb142_17, &phi_bb142_18, &phi_bb142_19, &phi_bb142_27, &phi_bb142_28, &phi_bb142_29, &phi_bb142_30, &phi_bb142_31, &phi_bb142_34, &phi_bb142_35, &phi_bb142_36, &phi_bb142_37, &phi_bb142_38);
    tmp194 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb142_38}, TNode<IntPtrT>{tmp22});
    ca_.Branch(tmp194, &block140, std::vector<compiler::Node*>{phi_bb142_17, phi_bb142_18, phi_bb142_19, phi_bb142_27, phi_bb142_28, phi_bb142_29, phi_bb142_30, phi_bb142_31, phi_bb142_34, phi_bb142_35, phi_bb142_36, phi_bb142_37, phi_bb142_38}, &block141, std::vector<compiler::Node*>{phi_bb142_17, phi_bb142_18, phi_bb142_19, phi_bb142_27, phi_bb142_28, phi_bb142_29, phi_bb142_30, phi_bb142_31, phi_bb142_34, phi_bb142_35, phi_bb142_36, phi_bb142_37, phi_bb142_38});
  }

  TNode<JSAny> phi_bb140_17;
  TNode<IntPtrT> phi_bb140_18;
  TNode<BoolT> phi_bb140_19;
  TNode<IntPtrT> phi_bb140_27;
  TNode<IntPtrT> phi_bb140_28;
  TNode<IntPtrT> phi_bb140_29;
  TNode<IntPtrT> phi_bb140_30;
  TNode<IntPtrT> phi_bb140_31;
  TNode<IntPtrT> phi_bb140_34;
  TNode<BoolT> phi_bb140_35;
  TNode<BoolT> phi_bb140_36;
  TNode<Union<FixedArray, Smi>> phi_bb140_37;
  TNode<IntPtrT> phi_bb140_38;
  TNode<JSAny> tmp195;
  TNode<IntPtrT> tmp196;
  TNode<IntPtrT> tmp197;
  TNode<Union<HeapObject, TaggedIndex>> tmp198;
  TNode<IntPtrT> tmp199;
  TNode<Uint32T> tmp200;
  TNode<Uint32T> tmp201;
  TNode<BoolT> tmp202;
  if (block140.is_used()) {
    ca_.Bind(&block140, &phi_bb140_17, &phi_bb140_18, &phi_bb140_19, &phi_bb140_27, &phi_bb140_28, &phi_bb140_29, &phi_bb140_30, &phi_bb140_31, &phi_bb140_34, &phi_bb140_35, &phi_bb140_36, &phi_bb140_37, &phi_bb140_38);
    tmp195 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{p_arguments.frame}, TNode<RawPtrT>{p_arguments.base}, TNode<IntPtrT>{p_arguments.length}, TNode<IntPtrT>{p_arguments.actual_count}}, TNode<IntPtrT>{phi_bb140_38});
    tmp196 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{phi_bb140_38});
    tmp197 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp162}, TNode<IntPtrT>{tmp196});
    std::tie(tmp198, tmp199) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp161}, TNode<IntPtrT>{tmp197}).Flatten();
    tmp200 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp198, tmp199});
    tmp201 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI32.raw_bit_field());
    tmp202 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp200}, TNode<Uint32T>{tmp201});
    ca_.Branch(tmp202, &block145, std::vector<compiler::Node*>{phi_bb140_17, phi_bb140_18, phi_bb140_19, phi_bb140_27, phi_bb140_28, phi_bb140_29, phi_bb140_30, phi_bb140_31, phi_bb140_34, phi_bb140_35, phi_bb140_36, phi_bb140_37, phi_bb140_38}, &block146, std::vector<compiler::Node*>{phi_bb140_17, phi_bb140_18, phi_bb140_19, phi_bb140_27, phi_bb140_28, phi_bb140_29, phi_bb140_30, phi_bb140_31, phi_bb140_34, phi_bb140_35, phi_bb140_36, phi_bb140_37, phi_bb140_38});
  }

  TNode<JSAny> phi_bb145_17;
  TNode<IntPtrT> phi_bb145_18;
  TNode<BoolT> phi_bb145_19;
  TNode<IntPtrT> phi_bb145_27;
  TNode<IntPtrT> phi_bb145_28;
  TNode<IntPtrT> phi_bb145_29;
  TNode<IntPtrT> phi_bb145_30;
  TNode<IntPtrT> phi_bb145_31;
  TNode<IntPtrT> phi_bb145_34;
  TNode<BoolT> phi_bb145_35;
  TNode<BoolT> phi_bb145_36;
  TNode<Union<FixedArray, Smi>> phi_bb145_37;
  TNode<IntPtrT> phi_bb145_38;
  TNode<IntPtrT> tmp203;
  TNode<IntPtrT> tmp204;
  TNode<IntPtrT> tmp205;
  TNode<BoolT> tmp206;
  if (block145.is_used()) {
    ca_.Bind(&block145, &phi_bb145_17, &phi_bb145_18, &phi_bb145_19, &phi_bb145_27, &phi_bb145_28, &phi_bb145_29, &phi_bb145_30, &phi_bb145_31, &phi_bb145_34, &phi_bb145_35, &phi_bb145_36, &phi_bb145_37, &phi_bb145_38);
    tmp203 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp204 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb145_27}, TNode<IntPtrT>{tmp203});
    tmp205 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp206 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb145_27}, TNode<IntPtrT>{tmp205});
    ca_.Branch(tmp206, &block149, std::vector<compiler::Node*>{phi_bb145_17, phi_bb145_18, phi_bb145_19, phi_bb145_28, phi_bb145_29, phi_bb145_30, phi_bb145_31, phi_bb145_34, phi_bb145_35, phi_bb145_36, phi_bb145_37, phi_bb145_38}, &block150, std::vector<compiler::Node*>{phi_bb145_17, phi_bb145_18, phi_bb145_19, phi_bb145_28, phi_bb145_29, phi_bb145_30, phi_bb145_31, phi_bb145_34, phi_bb145_35, phi_bb145_36, phi_bb145_37, phi_bb145_38});
  }

  TNode<JSAny> phi_bb149_17;
  TNode<IntPtrT> phi_bb149_18;
  TNode<BoolT> phi_bb149_19;
  TNode<IntPtrT> phi_bb149_28;
  TNode<IntPtrT> phi_bb149_29;
  TNode<IntPtrT> phi_bb149_30;
  TNode<IntPtrT> phi_bb149_31;
  TNode<IntPtrT> phi_bb149_34;
  TNode<BoolT> phi_bb149_35;
  TNode<BoolT> phi_bb149_36;
  TNode<Union<FixedArray, Smi>> phi_bb149_37;
  TNode<IntPtrT> phi_bb149_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp207;
  TNode<IntPtrT> tmp208;
  TNode<IntPtrT> tmp209;
  TNode<IntPtrT> tmp210;
  if (block149.is_used()) {
    ca_.Bind(&block149, &phi_bb149_17, &phi_bb149_18, &phi_bb149_19, &phi_bb149_28, &phi_bb149_29, &phi_bb149_30, &phi_bb149_31, &phi_bb149_34, &phi_bb149_35, &phi_bb149_36, &phi_bb149_37, &phi_bb149_38);
    std::tie(tmp207, tmp208) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb149_29}).Flatten();
    tmp209 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp210 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb149_29}, TNode<IntPtrT>{tmp209});
    ca_.Goto(&block148, phi_bb149_17, phi_bb149_18, phi_bb149_19, phi_bb149_28, tmp210, phi_bb149_30, phi_bb149_31, phi_bb149_34, phi_bb149_35, phi_bb149_36, phi_bb149_37, phi_bb149_38, tmp207, tmp208);
  }

  TNode<JSAny> phi_bb150_17;
  TNode<IntPtrT> phi_bb150_18;
  TNode<BoolT> phi_bb150_19;
  TNode<IntPtrT> phi_bb150_28;
  TNode<IntPtrT> phi_bb150_29;
  TNode<IntPtrT> phi_bb150_30;
  TNode<IntPtrT> phi_bb150_31;
  TNode<IntPtrT> phi_bb150_34;
  TNode<BoolT> phi_bb150_35;
  TNode<BoolT> phi_bb150_36;
  TNode<Union<FixedArray, Smi>> phi_bb150_37;
  TNode<IntPtrT> phi_bb150_38;
  if (block150.is_used()) {
    ca_.Bind(&block150, &phi_bb150_17, &phi_bb150_18, &phi_bb150_19, &phi_bb150_28, &phi_bb150_29, &phi_bb150_30, &phi_bb150_31, &phi_bb150_34, &phi_bb150_35, &phi_bb150_36, &phi_bb150_37, &phi_bb150_38);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block152, phi_bb150_17, phi_bb150_18, phi_bb150_19, phi_bb150_28, phi_bb150_29, phi_bb150_30, phi_bb150_31, phi_bb150_34, phi_bb150_35, phi_bb150_36, phi_bb150_37, phi_bb150_38);
    } else {
      ca_.Goto(&block153, phi_bb150_17, phi_bb150_18, phi_bb150_19, phi_bb150_28, phi_bb150_29, phi_bb150_30, phi_bb150_31, phi_bb150_34, phi_bb150_35, phi_bb150_36, phi_bb150_37, phi_bb150_38);
    }
  }

  TNode<JSAny> phi_bb152_17;
  TNode<IntPtrT> phi_bb152_18;
  TNode<BoolT> phi_bb152_19;
  TNode<IntPtrT> phi_bb152_28;
  TNode<IntPtrT> phi_bb152_29;
  TNode<IntPtrT> phi_bb152_30;
  TNode<IntPtrT> phi_bb152_31;
  TNode<IntPtrT> phi_bb152_34;
  TNode<BoolT> phi_bb152_35;
  TNode<BoolT> phi_bb152_36;
  TNode<Union<FixedArray, Smi>> phi_bb152_37;
  TNode<IntPtrT> phi_bb152_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp211;
  TNode<IntPtrT> tmp212;
  TNode<IntPtrT> tmp213;
  TNode<IntPtrT> tmp214;
  if (block152.is_used()) {
    ca_.Bind(&block152, &phi_bb152_17, &phi_bb152_18, &phi_bb152_19, &phi_bb152_28, &phi_bb152_29, &phi_bb152_30, &phi_bb152_31, &phi_bb152_34, &phi_bb152_35, &phi_bb152_36, &phi_bb152_37, &phi_bb152_38);
    std::tie(tmp211, tmp212) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb152_31}).Flatten();
    tmp213 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp214 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb152_31}, TNode<IntPtrT>{tmp213});
    ca_.Goto(&block151, phi_bb152_17, phi_bb152_18, phi_bb152_19, phi_bb152_28, phi_bb152_29, phi_bb152_30, tmp214, phi_bb152_34, phi_bb152_35, phi_bb152_36, phi_bb152_37, phi_bb152_38, tmp211, tmp212);
  }

  TNode<JSAny> phi_bb153_17;
  TNode<IntPtrT> phi_bb153_18;
  TNode<BoolT> phi_bb153_19;
  TNode<IntPtrT> phi_bb153_28;
  TNode<IntPtrT> phi_bb153_29;
  TNode<IntPtrT> phi_bb153_30;
  TNode<IntPtrT> phi_bb153_31;
  TNode<IntPtrT> phi_bb153_34;
  TNode<BoolT> phi_bb153_35;
  TNode<BoolT> phi_bb153_36;
  TNode<Union<FixedArray, Smi>> phi_bb153_37;
  TNode<IntPtrT> phi_bb153_38;
  TNode<IntPtrT> tmp215;
  TNode<BoolT> tmp216;
  if (block153.is_used()) {
    ca_.Bind(&block153, &phi_bb153_17, &phi_bb153_18, &phi_bb153_19, &phi_bb153_28, &phi_bb153_29, &phi_bb153_30, &phi_bb153_31, &phi_bb153_34, &phi_bb153_35, &phi_bb153_36, &phi_bb153_37, &phi_bb153_38);
    tmp215 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp216 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb153_34}, TNode<IntPtrT>{tmp215});
    ca_.Branch(tmp216, &block155, std::vector<compiler::Node*>{phi_bb153_17, phi_bb153_18, phi_bb153_19, phi_bb153_28, phi_bb153_29, phi_bb153_30, phi_bb153_31, phi_bb153_34, phi_bb153_35, phi_bb153_36, phi_bb153_37, phi_bb153_38}, &block156, std::vector<compiler::Node*>{phi_bb153_17, phi_bb153_18, phi_bb153_19, phi_bb153_28, phi_bb153_29, phi_bb153_30, phi_bb153_31, phi_bb153_34, phi_bb153_35, phi_bb153_36, phi_bb153_37, phi_bb153_38});
  }

  TNode<JSAny> phi_bb155_17;
  TNode<IntPtrT> phi_bb155_18;
  TNode<BoolT> phi_bb155_19;
  TNode<IntPtrT> phi_bb155_28;
  TNode<IntPtrT> phi_bb155_29;
  TNode<IntPtrT> phi_bb155_30;
  TNode<IntPtrT> phi_bb155_31;
  TNode<IntPtrT> phi_bb155_34;
  TNode<BoolT> phi_bb155_35;
  TNode<BoolT> phi_bb155_36;
  TNode<Union<FixedArray, Smi>> phi_bb155_37;
  TNode<IntPtrT> phi_bb155_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp217;
  TNode<IntPtrT> tmp218;
  TNode<IntPtrT> tmp219;
  TNode<BoolT> tmp220;
  if (block155.is_used()) {
    ca_.Bind(&block155, &phi_bb155_17, &phi_bb155_18, &phi_bb155_19, &phi_bb155_28, &phi_bb155_29, &phi_bb155_30, &phi_bb155_31, &phi_bb155_34, &phi_bb155_35, &phi_bb155_36, &phi_bb155_37, &phi_bb155_38);
    std::tie(tmp217, tmp218) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb155_34}).Flatten();
    tmp219 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp220 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block151, phi_bb155_17, phi_bb155_18, phi_bb155_19, phi_bb155_28, phi_bb155_29, phi_bb155_30, phi_bb155_31, tmp219, tmp220, phi_bb155_36, phi_bb155_37, phi_bb155_38, tmp217, tmp218);
  }

  TNode<JSAny> phi_bb156_17;
  TNode<IntPtrT> phi_bb156_18;
  TNode<BoolT> phi_bb156_19;
  TNode<IntPtrT> phi_bb156_28;
  TNode<IntPtrT> phi_bb156_29;
  TNode<IntPtrT> phi_bb156_30;
  TNode<IntPtrT> phi_bb156_31;
  TNode<IntPtrT> phi_bb156_34;
  TNode<BoolT> phi_bb156_35;
  TNode<BoolT> phi_bb156_36;
  TNode<Union<FixedArray, Smi>> phi_bb156_37;
  TNode<IntPtrT> phi_bb156_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp221;
  TNode<IntPtrT> tmp222;
  TNode<IntPtrT> tmp223;
  TNode<IntPtrT> tmp224;
  TNode<IntPtrT> tmp225;
  TNode<IntPtrT> tmp226;
  TNode<BoolT> tmp227;
  if (block156.is_used()) {
    ca_.Bind(&block156, &phi_bb156_17, &phi_bb156_18, &phi_bb156_19, &phi_bb156_28, &phi_bb156_29, &phi_bb156_30, &phi_bb156_31, &phi_bb156_34, &phi_bb156_35, &phi_bb156_36, &phi_bb156_37, &phi_bb156_38);
    std::tie(tmp221, tmp222) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb156_31}).Flatten();
    tmp223 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp224 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb156_31}, TNode<IntPtrT>{tmp223});
    tmp225 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp226 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp224}, TNode<IntPtrT>{tmp225});
    tmp227 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block151, phi_bb156_17, phi_bb156_18, phi_bb156_19, phi_bb156_28, phi_bb156_29, phi_bb156_30, tmp226, tmp224, tmp227, phi_bb156_36, phi_bb156_37, phi_bb156_38, tmp221, tmp222);
  }

  TNode<JSAny> phi_bb151_17;
  TNode<IntPtrT> phi_bb151_18;
  TNode<BoolT> phi_bb151_19;
  TNode<IntPtrT> phi_bb151_28;
  TNode<IntPtrT> phi_bb151_29;
  TNode<IntPtrT> phi_bb151_30;
  TNode<IntPtrT> phi_bb151_31;
  TNode<IntPtrT> phi_bb151_34;
  TNode<BoolT> phi_bb151_35;
  TNode<BoolT> phi_bb151_36;
  TNode<Union<FixedArray, Smi>> phi_bb151_37;
  TNode<IntPtrT> phi_bb151_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb151_41;
  TNode<IntPtrT> phi_bb151_42;
  if (block151.is_used()) {
    ca_.Bind(&block151, &phi_bb151_17, &phi_bb151_18, &phi_bb151_19, &phi_bb151_28, &phi_bb151_29, &phi_bb151_30, &phi_bb151_31, &phi_bb151_34, &phi_bb151_35, &phi_bb151_36, &phi_bb151_37, &phi_bb151_38, &phi_bb151_41, &phi_bb151_42);
    ca_.Goto(&block148, phi_bb151_17, phi_bb151_18, phi_bb151_19, phi_bb151_28, phi_bb151_29, phi_bb151_30, phi_bb151_31, phi_bb151_34, phi_bb151_35, phi_bb151_36, phi_bb151_37, phi_bb151_38, phi_bb151_41, phi_bb151_42);
  }

  TNode<JSAny> phi_bb148_17;
  TNode<IntPtrT> phi_bb148_18;
  TNode<BoolT> phi_bb148_19;
  TNode<IntPtrT> phi_bb148_28;
  TNode<IntPtrT> phi_bb148_29;
  TNode<IntPtrT> phi_bb148_30;
  TNode<IntPtrT> phi_bb148_31;
  TNode<IntPtrT> phi_bb148_34;
  TNode<BoolT> phi_bb148_35;
  TNode<BoolT> phi_bb148_36;
  TNode<Union<FixedArray, Smi>> phi_bb148_37;
  TNode<IntPtrT> phi_bb148_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb148_41;
  TNode<IntPtrT> phi_bb148_42;
  TNode<Smi> tmp228;
  if (block148.is_used()) {
    ca_.Bind(&block148, &phi_bb148_17, &phi_bb148_18, &phi_bb148_19, &phi_bb148_28, &phi_bb148_29, &phi_bb148_30, &phi_bb148_31, &phi_bb148_34, &phi_bb148_35, &phi_bb148_36, &phi_bb148_37, &phi_bb148_38, &phi_bb148_41, &phi_bb148_42);
    compiler::CodeAssemblerLabel label229(&ca_);
    tmp228 = Cast_Smi_0(state_, TNode<Object>{tmp195}, &label229);
    ca_.Goto(&block159, phi_bb148_17, phi_bb148_18, phi_bb148_19, phi_bb148_28, phi_bb148_29, phi_bb148_30, phi_bb148_31, phi_bb148_34, phi_bb148_35, phi_bb148_36, phi_bb148_37, phi_bb148_38, phi_bb148_41, phi_bb148_42);
    if (label229.is_used()) {
      ca_.Bind(&label229);
      ca_.Goto(&block160, phi_bb148_17, phi_bb148_18, phi_bb148_19, phi_bb148_28, phi_bb148_29, phi_bb148_30, phi_bb148_31, phi_bb148_34, phi_bb148_35, phi_bb148_36, phi_bb148_37, phi_bb148_38, phi_bb148_41, phi_bb148_42);
    }
  }

  TNode<JSAny> phi_bb160_17;
  TNode<IntPtrT> phi_bb160_18;
  TNode<BoolT> phi_bb160_19;
  TNode<IntPtrT> phi_bb160_28;
  TNode<IntPtrT> phi_bb160_29;
  TNode<IntPtrT> phi_bb160_30;
  TNode<IntPtrT> phi_bb160_31;
  TNode<IntPtrT> phi_bb160_34;
  TNode<BoolT> phi_bb160_35;
  TNode<BoolT> phi_bb160_36;
  TNode<Union<FixedArray, Smi>> phi_bb160_37;
  TNode<IntPtrT> phi_bb160_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb160_41;
  TNode<IntPtrT> phi_bb160_42;
  TNode<Int32T> tmp230;
  TNode<IntPtrT> tmp231;
  if (block160.is_used()) {
    ca_.Bind(&block160, &phi_bb160_17, &phi_bb160_18, &phi_bb160_19, &phi_bb160_28, &phi_bb160_29, &phi_bb160_30, &phi_bb160_31, &phi_bb160_34, &phi_bb160_35, &phi_bb160_36, &phi_bb160_37, &phi_bb160_38, &phi_bb160_41, &phi_bb160_42);
    tmp230 = ca_.CallBuiltin<Int32T>(Builtin::kWasmTaggedNonSmiToInt32, p_context, ca_.UncheckedCast<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, String, Symbol, Undefined>>(tmp195));
    tmp231 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp230});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb160_41, phi_bb160_42}, tmp231);
    ca_.Goto(&block157, phi_bb160_17, phi_bb160_18, phi_bb160_19, phi_bb160_28, phi_bb160_29, phi_bb160_30, phi_bb160_31, phi_bb160_34, phi_bb160_35, phi_bb160_36, phi_bb160_37, phi_bb160_38, phi_bb160_41, phi_bb160_42);
  }

  TNode<JSAny> phi_bb159_17;
  TNode<IntPtrT> phi_bb159_18;
  TNode<BoolT> phi_bb159_19;
  TNode<IntPtrT> phi_bb159_28;
  TNode<IntPtrT> phi_bb159_29;
  TNode<IntPtrT> phi_bb159_30;
  TNode<IntPtrT> phi_bb159_31;
  TNode<IntPtrT> phi_bb159_34;
  TNode<BoolT> phi_bb159_35;
  TNode<BoolT> phi_bb159_36;
  TNode<Union<FixedArray, Smi>> phi_bb159_37;
  TNode<IntPtrT> phi_bb159_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb159_41;
  TNode<IntPtrT> phi_bb159_42;
  TNode<Int32T> tmp232;
  TNode<IntPtrT> tmp233;
  if (block159.is_used()) {
    ca_.Bind(&block159, &phi_bb159_17, &phi_bb159_18, &phi_bb159_19, &phi_bb159_28, &phi_bb159_29, &phi_bb159_30, &phi_bb159_31, &phi_bb159_34, &phi_bb159_35, &phi_bb159_36, &phi_bb159_37, &phi_bb159_38, &phi_bb159_41, &phi_bb159_42);
    tmp232 = CodeStubAssembler(state_).SmiToInt32(TNode<Smi>{tmp228});
    tmp233 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp232});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb159_41, phi_bb159_42}, tmp233);
    ca_.Goto(&block157, phi_bb159_17, phi_bb159_18, phi_bb159_19, phi_bb159_28, phi_bb159_29, phi_bb159_30, phi_bb159_31, phi_bb159_34, phi_bb159_35, phi_bb159_36, phi_bb159_37, phi_bb159_38, phi_bb159_41, phi_bb159_42);
  }

  TNode<JSAny> phi_bb157_17;
  TNode<IntPtrT> phi_bb157_18;
  TNode<BoolT> phi_bb157_19;
  TNode<IntPtrT> phi_bb157_28;
  TNode<IntPtrT> phi_bb157_29;
  TNode<IntPtrT> phi_bb157_30;
  TNode<IntPtrT> phi_bb157_31;
  TNode<IntPtrT> phi_bb157_34;
  TNode<BoolT> phi_bb157_35;
  TNode<BoolT> phi_bb157_36;
  TNode<Union<FixedArray, Smi>> phi_bb157_37;
  TNode<IntPtrT> phi_bb157_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb157_41;
  TNode<IntPtrT> phi_bb157_42;
  if (block157.is_used()) {
    ca_.Bind(&block157, &phi_bb157_17, &phi_bb157_18, &phi_bb157_19, &phi_bb157_28, &phi_bb157_29, &phi_bb157_30, &phi_bb157_31, &phi_bb157_34, &phi_bb157_35, &phi_bb157_36, &phi_bb157_37, &phi_bb157_38, &phi_bb157_41, &phi_bb157_42);
    ca_.Goto(&block147, phi_bb157_17, phi_bb157_18, phi_bb157_19, tmp204, phi_bb157_28, phi_bb157_29, phi_bb157_30, phi_bb157_31, phi_bb157_34, phi_bb157_35, phi_bb157_36, phi_bb157_37, phi_bb157_38);
  }

  TNode<JSAny> phi_bb146_17;
  TNode<IntPtrT> phi_bb146_18;
  TNode<BoolT> phi_bb146_19;
  TNode<IntPtrT> phi_bb146_27;
  TNode<IntPtrT> phi_bb146_28;
  TNode<IntPtrT> phi_bb146_29;
  TNode<IntPtrT> phi_bb146_30;
  TNode<IntPtrT> phi_bb146_31;
  TNode<IntPtrT> phi_bb146_34;
  TNode<BoolT> phi_bb146_35;
  TNode<BoolT> phi_bb146_36;
  TNode<Union<FixedArray, Smi>> phi_bb146_37;
  TNode<IntPtrT> phi_bb146_38;
  TNode<Uint32T> tmp234;
  TNode<BoolT> tmp235;
  if (block146.is_used()) {
    ca_.Bind(&block146, &phi_bb146_17, &phi_bb146_18, &phi_bb146_19, &phi_bb146_27, &phi_bb146_28, &phi_bb146_29, &phi_bb146_30, &phi_bb146_31, &phi_bb146_34, &phi_bb146_35, &phi_bb146_36, &phi_bb146_37, &phi_bb146_38);
    tmp234 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF32.raw_bit_field());
    tmp235 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp200}, TNode<Uint32T>{tmp234});
    ca_.Branch(tmp235, &block161, std::vector<compiler::Node*>{phi_bb146_17, phi_bb146_18, phi_bb146_19, phi_bb146_27, phi_bb146_28, phi_bb146_29, phi_bb146_30, phi_bb146_31, phi_bb146_34, phi_bb146_35, phi_bb146_36, phi_bb146_37, phi_bb146_38}, &block162, std::vector<compiler::Node*>{phi_bb146_17, phi_bb146_18, phi_bb146_19, phi_bb146_27, phi_bb146_28, phi_bb146_29, phi_bb146_30, phi_bb146_31, phi_bb146_34, phi_bb146_35, phi_bb146_36, phi_bb146_37, phi_bb146_38});
  }

  TNode<JSAny> phi_bb161_17;
  TNode<IntPtrT> phi_bb161_18;
  TNode<BoolT> phi_bb161_19;
  TNode<IntPtrT> phi_bb161_27;
  TNode<IntPtrT> phi_bb161_28;
  TNode<IntPtrT> phi_bb161_29;
  TNode<IntPtrT> phi_bb161_30;
  TNode<IntPtrT> phi_bb161_31;
  TNode<IntPtrT> phi_bb161_34;
  TNode<BoolT> phi_bb161_35;
  TNode<BoolT> phi_bb161_36;
  TNode<Union<FixedArray, Smi>> phi_bb161_37;
  TNode<IntPtrT> phi_bb161_38;
  TNode<IntPtrT> tmp236;
  TNode<IntPtrT> tmp237;
  TNode<IntPtrT> tmp238;
  TNode<BoolT> tmp239;
  if (block161.is_used()) {
    ca_.Bind(&block161, &phi_bb161_17, &phi_bb161_18, &phi_bb161_19, &phi_bb161_27, &phi_bb161_28, &phi_bb161_29, &phi_bb161_30, &phi_bb161_31, &phi_bb161_34, &phi_bb161_35, &phi_bb161_36, &phi_bb161_37, &phi_bb161_38);
    tmp236 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp237 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb161_28}, TNode<IntPtrT>{tmp236});
    tmp238 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp239 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb161_28}, TNode<IntPtrT>{tmp238});
    ca_.Branch(tmp239, &block165, std::vector<compiler::Node*>{phi_bb161_17, phi_bb161_18, phi_bb161_19, phi_bb161_27, phi_bb161_29, phi_bb161_30, phi_bb161_31, phi_bb161_34, phi_bb161_35, phi_bb161_36, phi_bb161_37, phi_bb161_38}, &block166, std::vector<compiler::Node*>{phi_bb161_17, phi_bb161_18, phi_bb161_19, phi_bb161_27, phi_bb161_29, phi_bb161_30, phi_bb161_31, phi_bb161_34, phi_bb161_35, phi_bb161_36, phi_bb161_37, phi_bb161_38});
  }

  TNode<JSAny> phi_bb165_17;
  TNode<IntPtrT> phi_bb165_18;
  TNode<BoolT> phi_bb165_19;
  TNode<IntPtrT> phi_bb165_27;
  TNode<IntPtrT> phi_bb165_29;
  TNode<IntPtrT> phi_bb165_30;
  TNode<IntPtrT> phi_bb165_31;
  TNode<IntPtrT> phi_bb165_34;
  TNode<BoolT> phi_bb165_35;
  TNode<BoolT> phi_bb165_36;
  TNode<Union<FixedArray, Smi>> phi_bb165_37;
  TNode<IntPtrT> phi_bb165_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp240;
  TNode<IntPtrT> tmp241;
  TNode<IntPtrT> tmp242;
  TNode<IntPtrT> tmp243;
  if (block165.is_used()) {
    ca_.Bind(&block165, &phi_bb165_17, &phi_bb165_18, &phi_bb165_19, &phi_bb165_27, &phi_bb165_29, &phi_bb165_30, &phi_bb165_31, &phi_bb165_34, &phi_bb165_35, &phi_bb165_36, &phi_bb165_37, &phi_bb165_38);
    std::tie(tmp240, tmp241) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb165_30}).Flatten();
    tmp242 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp243 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb165_30}, TNode<IntPtrT>{tmp242});
    ca_.Goto(&block164, phi_bb165_17, phi_bb165_18, phi_bb165_19, phi_bb165_27, phi_bb165_29, tmp243, phi_bb165_31, phi_bb165_34, phi_bb165_35, phi_bb165_36, phi_bb165_37, phi_bb165_38, tmp240, tmp241);
  }

  TNode<JSAny> phi_bb166_17;
  TNode<IntPtrT> phi_bb166_18;
  TNode<BoolT> phi_bb166_19;
  TNode<IntPtrT> phi_bb166_27;
  TNode<IntPtrT> phi_bb166_29;
  TNode<IntPtrT> phi_bb166_30;
  TNode<IntPtrT> phi_bb166_31;
  TNode<IntPtrT> phi_bb166_34;
  TNode<BoolT> phi_bb166_35;
  TNode<BoolT> phi_bb166_36;
  TNode<Union<FixedArray, Smi>> phi_bb166_37;
  TNode<IntPtrT> phi_bb166_38;
  if (block166.is_used()) {
    ca_.Bind(&block166, &phi_bb166_17, &phi_bb166_18, &phi_bb166_19, &phi_bb166_27, &phi_bb166_29, &phi_bb166_30, &phi_bb166_31, &phi_bb166_34, &phi_bb166_35, &phi_bb166_36, &phi_bb166_37, &phi_bb166_38);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block168, phi_bb166_17, phi_bb166_18, phi_bb166_19, phi_bb166_27, phi_bb166_29, phi_bb166_30, phi_bb166_31, phi_bb166_34, phi_bb166_35, phi_bb166_36, phi_bb166_37, phi_bb166_38);
    } else {
      ca_.Goto(&block169, phi_bb166_17, phi_bb166_18, phi_bb166_19, phi_bb166_27, phi_bb166_29, phi_bb166_30, phi_bb166_31, phi_bb166_34, phi_bb166_35, phi_bb166_36, phi_bb166_37, phi_bb166_38);
    }
  }

  TNode<JSAny> phi_bb168_17;
  TNode<IntPtrT> phi_bb168_18;
  TNode<BoolT> phi_bb168_19;
  TNode<IntPtrT> phi_bb168_27;
  TNode<IntPtrT> phi_bb168_29;
  TNode<IntPtrT> phi_bb168_30;
  TNode<IntPtrT> phi_bb168_31;
  TNode<IntPtrT> phi_bb168_34;
  TNode<BoolT> phi_bb168_35;
  TNode<BoolT> phi_bb168_36;
  TNode<Union<FixedArray, Smi>> phi_bb168_37;
  TNode<IntPtrT> phi_bb168_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp244;
  TNode<IntPtrT> tmp245;
  TNode<IntPtrT> tmp246;
  TNode<IntPtrT> tmp247;
  if (block168.is_used()) {
    ca_.Bind(&block168, &phi_bb168_17, &phi_bb168_18, &phi_bb168_19, &phi_bb168_27, &phi_bb168_29, &phi_bb168_30, &phi_bb168_31, &phi_bb168_34, &phi_bb168_35, &phi_bb168_36, &phi_bb168_37, &phi_bb168_38);
    std::tie(tmp244, tmp245) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb168_31}).Flatten();
    tmp246 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp247 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb168_31}, TNode<IntPtrT>{tmp246});
    ca_.Goto(&block167, phi_bb168_17, phi_bb168_18, phi_bb168_19, phi_bb168_27, phi_bb168_29, phi_bb168_30, tmp247, phi_bb168_34, phi_bb168_35, phi_bb168_36, phi_bb168_37, phi_bb168_38, tmp244, tmp245);
  }

  TNode<JSAny> phi_bb169_17;
  TNode<IntPtrT> phi_bb169_18;
  TNode<BoolT> phi_bb169_19;
  TNode<IntPtrT> phi_bb169_27;
  TNode<IntPtrT> phi_bb169_29;
  TNode<IntPtrT> phi_bb169_30;
  TNode<IntPtrT> phi_bb169_31;
  TNode<IntPtrT> phi_bb169_34;
  TNode<BoolT> phi_bb169_35;
  TNode<BoolT> phi_bb169_36;
  TNode<Union<FixedArray, Smi>> phi_bb169_37;
  TNode<IntPtrT> phi_bb169_38;
  TNode<IntPtrT> tmp248;
  TNode<BoolT> tmp249;
  if (block169.is_used()) {
    ca_.Bind(&block169, &phi_bb169_17, &phi_bb169_18, &phi_bb169_19, &phi_bb169_27, &phi_bb169_29, &phi_bb169_30, &phi_bb169_31, &phi_bb169_34, &phi_bb169_35, &phi_bb169_36, &phi_bb169_37, &phi_bb169_38);
    tmp248 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp249 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb169_34}, TNode<IntPtrT>{tmp248});
    ca_.Branch(tmp249, &block171, std::vector<compiler::Node*>{phi_bb169_17, phi_bb169_18, phi_bb169_19, phi_bb169_27, phi_bb169_29, phi_bb169_30, phi_bb169_31, phi_bb169_34, phi_bb169_35, phi_bb169_36, phi_bb169_37, phi_bb169_38}, &block172, std::vector<compiler::Node*>{phi_bb169_17, phi_bb169_18, phi_bb169_19, phi_bb169_27, phi_bb169_29, phi_bb169_30, phi_bb169_31, phi_bb169_34, phi_bb169_35, phi_bb169_36, phi_bb169_37, phi_bb169_38});
  }

  TNode<JSAny> phi_bb171_17;
  TNode<IntPtrT> phi_bb171_18;
  TNode<BoolT> phi_bb171_19;
  TNode<IntPtrT> phi_bb171_27;
  TNode<IntPtrT> phi_bb171_29;
  TNode<IntPtrT> phi_bb171_30;
  TNode<IntPtrT> phi_bb171_31;
  TNode<IntPtrT> phi_bb171_34;
  TNode<BoolT> phi_bb171_35;
  TNode<BoolT> phi_bb171_36;
  TNode<Union<FixedArray, Smi>> phi_bb171_37;
  TNode<IntPtrT> phi_bb171_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp250;
  TNode<IntPtrT> tmp251;
  TNode<IntPtrT> tmp252;
  TNode<BoolT> tmp253;
  if (block171.is_used()) {
    ca_.Bind(&block171, &phi_bb171_17, &phi_bb171_18, &phi_bb171_19, &phi_bb171_27, &phi_bb171_29, &phi_bb171_30, &phi_bb171_31, &phi_bb171_34, &phi_bb171_35, &phi_bb171_36, &phi_bb171_37, &phi_bb171_38);
    std::tie(tmp250, tmp251) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb171_34}).Flatten();
    tmp252 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp253 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block167, phi_bb171_17, phi_bb171_18, phi_bb171_19, phi_bb171_27, phi_bb171_29, phi_bb171_30, phi_bb171_31, tmp252, tmp253, phi_bb171_36, phi_bb171_37, phi_bb171_38, tmp250, tmp251);
  }

  TNode<JSAny> phi_bb172_17;
  TNode<IntPtrT> phi_bb172_18;
  TNode<BoolT> phi_bb172_19;
  TNode<IntPtrT> phi_bb172_27;
  TNode<IntPtrT> phi_bb172_29;
  TNode<IntPtrT> phi_bb172_30;
  TNode<IntPtrT> phi_bb172_31;
  TNode<IntPtrT> phi_bb172_34;
  TNode<BoolT> phi_bb172_35;
  TNode<BoolT> phi_bb172_36;
  TNode<Union<FixedArray, Smi>> phi_bb172_37;
  TNode<IntPtrT> phi_bb172_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp254;
  TNode<IntPtrT> tmp255;
  TNode<IntPtrT> tmp256;
  TNode<IntPtrT> tmp257;
  TNode<IntPtrT> tmp258;
  TNode<IntPtrT> tmp259;
  TNode<BoolT> tmp260;
  if (block172.is_used()) {
    ca_.Bind(&block172, &phi_bb172_17, &phi_bb172_18, &phi_bb172_19, &phi_bb172_27, &phi_bb172_29, &phi_bb172_30, &phi_bb172_31, &phi_bb172_34, &phi_bb172_35, &phi_bb172_36, &phi_bb172_37, &phi_bb172_38);
    std::tie(tmp254, tmp255) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb172_31}).Flatten();
    tmp256 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp257 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb172_31}, TNode<IntPtrT>{tmp256});
    tmp258 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp259 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp257}, TNode<IntPtrT>{tmp258});
    tmp260 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block167, phi_bb172_17, phi_bb172_18, phi_bb172_19, phi_bb172_27, phi_bb172_29, phi_bb172_30, tmp259, tmp257, tmp260, phi_bb172_36, phi_bb172_37, phi_bb172_38, tmp254, tmp255);
  }

  TNode<JSAny> phi_bb167_17;
  TNode<IntPtrT> phi_bb167_18;
  TNode<BoolT> phi_bb167_19;
  TNode<IntPtrT> phi_bb167_27;
  TNode<IntPtrT> phi_bb167_29;
  TNode<IntPtrT> phi_bb167_30;
  TNode<IntPtrT> phi_bb167_31;
  TNode<IntPtrT> phi_bb167_34;
  TNode<BoolT> phi_bb167_35;
  TNode<BoolT> phi_bb167_36;
  TNode<Union<FixedArray, Smi>> phi_bb167_37;
  TNode<IntPtrT> phi_bb167_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb167_41;
  TNode<IntPtrT> phi_bb167_42;
  if (block167.is_used()) {
    ca_.Bind(&block167, &phi_bb167_17, &phi_bb167_18, &phi_bb167_19, &phi_bb167_27, &phi_bb167_29, &phi_bb167_30, &phi_bb167_31, &phi_bb167_34, &phi_bb167_35, &phi_bb167_36, &phi_bb167_37, &phi_bb167_38, &phi_bb167_41, &phi_bb167_42);
    ca_.Goto(&block164, phi_bb167_17, phi_bb167_18, phi_bb167_19, phi_bb167_27, phi_bb167_29, phi_bb167_30, phi_bb167_31, phi_bb167_34, phi_bb167_35, phi_bb167_36, phi_bb167_37, phi_bb167_38, phi_bb167_41, phi_bb167_42);
  }

  TNode<JSAny> phi_bb164_17;
  TNode<IntPtrT> phi_bb164_18;
  TNode<BoolT> phi_bb164_19;
  TNode<IntPtrT> phi_bb164_27;
  TNode<IntPtrT> phi_bb164_29;
  TNode<IntPtrT> phi_bb164_30;
  TNode<IntPtrT> phi_bb164_31;
  TNode<IntPtrT> phi_bb164_34;
  TNode<BoolT> phi_bb164_35;
  TNode<BoolT> phi_bb164_36;
  TNode<Union<FixedArray, Smi>> phi_bb164_37;
  TNode<IntPtrT> phi_bb164_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb164_41;
  TNode<IntPtrT> phi_bb164_42;
  if (block164.is_used()) {
    ca_.Bind(&block164, &phi_bb164_17, &phi_bb164_18, &phi_bb164_19, &phi_bb164_27, &phi_bb164_29, &phi_bb164_30, &phi_bb164_31, &phi_bb164_34, &phi_bb164_35, &phi_bb164_36, &phi_bb164_37, &phi_bb164_38, &phi_bb164_41, &phi_bb164_42);
    if ((((wasm::kIsFpAlwaysDouble || wasm::kIsBigEndian) || wasm::kIsBigEndianOnSim))) {
      ca_.Goto(&block173, phi_bb164_17, phi_bb164_18, phi_bb164_19, phi_bb164_27, phi_bb164_29, phi_bb164_30, phi_bb164_31, phi_bb164_34, phi_bb164_35, phi_bb164_36, phi_bb164_37, phi_bb164_38, phi_bb164_41, phi_bb164_42);
    } else {
      ca_.Goto(&block174, phi_bb164_17, phi_bb164_18, phi_bb164_19, phi_bb164_27, phi_bb164_29, phi_bb164_30, phi_bb164_31, phi_bb164_34, phi_bb164_35, phi_bb164_36, phi_bb164_37, phi_bb164_38, phi_bb164_41, phi_bb164_42);
    }
  }

  TNode<JSAny> phi_bb173_17;
  TNode<IntPtrT> phi_bb173_18;
  TNode<BoolT> phi_bb173_19;
  TNode<IntPtrT> phi_bb173_27;
  TNode<IntPtrT> phi_bb173_29;
  TNode<IntPtrT> phi_bb173_30;
  TNode<IntPtrT> phi_bb173_31;
  TNode<IntPtrT> phi_bb173_34;
  TNode<BoolT> phi_bb173_35;
  TNode<BoolT> phi_bb173_36;
  TNode<Union<FixedArray, Smi>> phi_bb173_37;
  TNode<IntPtrT> phi_bb173_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb173_41;
  TNode<IntPtrT> phi_bb173_42;
  if (block173.is_used()) {
    ca_.Bind(&block173, &phi_bb173_17, &phi_bb173_18, &phi_bb173_19, &phi_bb173_27, &phi_bb173_29, &phi_bb173_30, &phi_bb173_31, &phi_bb173_34, &phi_bb173_35, &phi_bb173_36, &phi_bb173_37, &phi_bb173_38, &phi_bb173_41, &phi_bb173_42);
    HandleF32Params_0(state_, TNode<NativeContext>{p_context}, TorqueStructLocationAllocator_0{TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb173_27}, TNode<IntPtrT>{tmp237}, TNode<IntPtrT>{phi_bb173_29}, TNode<IntPtrT>{phi_bb173_30}, TNode<IntPtrT>{phi_bb173_31}, TNode<IntPtrT>{tmp187}, TNode<IntPtrT>{tmp188}, TNode<IntPtrT>{phi_bb173_34}, TNode<BoolT>{phi_bb173_35}}, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb173_41}, TNode<IntPtrT>{phi_bb173_42}, TorqueStructUnsafe_0{}}, TNode<JSAny>{tmp195});
    ca_.Goto(&block175, phi_bb173_17, phi_bb173_18, phi_bb173_19, phi_bb173_27, phi_bb173_29, phi_bb173_30, phi_bb173_31, phi_bb173_34, phi_bb173_35, phi_bb173_36, phi_bb173_37, phi_bb173_38, phi_bb173_41, phi_bb173_42);
  }

  TNode<JSAny> phi_bb174_17;
  TNode<IntPtrT> phi_bb174_18;
  TNode<BoolT> phi_bb174_19;
  TNode<IntPtrT> phi_bb174_27;
  TNode<IntPtrT> phi_bb174_29;
  TNode<IntPtrT> phi_bb174_30;
  TNode<IntPtrT> phi_bb174_31;
  TNode<IntPtrT> phi_bb174_34;
  TNode<BoolT> phi_bb174_35;
  TNode<BoolT> phi_bb174_36;
  TNode<Union<FixedArray, Smi>> phi_bb174_37;
  TNode<IntPtrT> phi_bb174_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb174_41;
  TNode<IntPtrT> phi_bb174_42;
  TNode<Float32T> tmp261;
  TNode<Uint32T> tmp262;
  TNode<IntPtrT> tmp263;
  if (block174.is_used()) {
    ca_.Bind(&block174, &phi_bb174_17, &phi_bb174_18, &phi_bb174_19, &phi_bb174_27, &phi_bb174_29, &phi_bb174_30, &phi_bb174_31, &phi_bb174_34, &phi_bb174_35, &phi_bb174_36, &phi_bb174_37, &phi_bb174_38, &phi_bb174_41, &phi_bb174_42);
    tmp261 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, tmp195);
    tmp262 = Bitcast_uint32_float32_0(state_, TNode<Float32T>{tmp261});
    tmp263 = Convert_intptr_uint32_0(state_, TNode<Uint32T>{tmp262});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb174_41, phi_bb174_42}, tmp263);
    ca_.Goto(&block175, phi_bb174_17, phi_bb174_18, phi_bb174_19, phi_bb174_27, phi_bb174_29, phi_bb174_30, phi_bb174_31, phi_bb174_34, phi_bb174_35, phi_bb174_36, phi_bb174_37, phi_bb174_38, phi_bb174_41, phi_bb174_42);
  }

  TNode<JSAny> phi_bb175_17;
  TNode<IntPtrT> phi_bb175_18;
  TNode<BoolT> phi_bb175_19;
  TNode<IntPtrT> phi_bb175_27;
  TNode<IntPtrT> phi_bb175_29;
  TNode<IntPtrT> phi_bb175_30;
  TNode<IntPtrT> phi_bb175_31;
  TNode<IntPtrT> phi_bb175_34;
  TNode<BoolT> phi_bb175_35;
  TNode<BoolT> phi_bb175_36;
  TNode<Union<FixedArray, Smi>> phi_bb175_37;
  TNode<IntPtrT> phi_bb175_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb175_41;
  TNode<IntPtrT> phi_bb175_42;
  if (block175.is_used()) {
    ca_.Bind(&block175, &phi_bb175_17, &phi_bb175_18, &phi_bb175_19, &phi_bb175_27, &phi_bb175_29, &phi_bb175_30, &phi_bb175_31, &phi_bb175_34, &phi_bb175_35, &phi_bb175_36, &phi_bb175_37, &phi_bb175_38, &phi_bb175_41, &phi_bb175_42);
    ca_.Goto(&block163, phi_bb175_17, phi_bb175_18, phi_bb175_19, phi_bb175_27, tmp237, phi_bb175_29, phi_bb175_30, phi_bb175_31, phi_bb175_34, phi_bb175_35, phi_bb175_36, phi_bb175_37, phi_bb175_38);
  }

  TNode<JSAny> phi_bb162_17;
  TNode<IntPtrT> phi_bb162_18;
  TNode<BoolT> phi_bb162_19;
  TNode<IntPtrT> phi_bb162_27;
  TNode<IntPtrT> phi_bb162_28;
  TNode<IntPtrT> phi_bb162_29;
  TNode<IntPtrT> phi_bb162_30;
  TNode<IntPtrT> phi_bb162_31;
  TNode<IntPtrT> phi_bb162_34;
  TNode<BoolT> phi_bb162_35;
  TNode<BoolT> phi_bb162_36;
  TNode<Union<FixedArray, Smi>> phi_bb162_37;
  TNode<IntPtrT> phi_bb162_38;
  TNode<Uint32T> tmp264;
  TNode<BoolT> tmp265;
  if (block162.is_used()) {
    ca_.Bind(&block162, &phi_bb162_17, &phi_bb162_18, &phi_bb162_19, &phi_bb162_27, &phi_bb162_28, &phi_bb162_29, &phi_bb162_30, &phi_bb162_31, &phi_bb162_34, &phi_bb162_35, &phi_bb162_36, &phi_bb162_37, &phi_bb162_38);
    tmp264 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF64.raw_bit_field());
    tmp265 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp200}, TNode<Uint32T>{tmp264});
    ca_.Branch(tmp265, &block176, std::vector<compiler::Node*>{phi_bb162_17, phi_bb162_18, phi_bb162_19, phi_bb162_27, phi_bb162_28, phi_bb162_29, phi_bb162_30, phi_bb162_31, phi_bb162_34, phi_bb162_35, phi_bb162_36, phi_bb162_37, phi_bb162_38}, &block177, std::vector<compiler::Node*>{phi_bb162_17, phi_bb162_18, phi_bb162_19, phi_bb162_27, phi_bb162_28, phi_bb162_29, phi_bb162_30, phi_bb162_31, phi_bb162_34, phi_bb162_35, phi_bb162_36, phi_bb162_37, phi_bb162_38});
  }

  TNode<JSAny> phi_bb176_17;
  TNode<IntPtrT> phi_bb176_18;
  TNode<BoolT> phi_bb176_19;
  TNode<IntPtrT> phi_bb176_27;
  TNode<IntPtrT> phi_bb176_28;
  TNode<IntPtrT> phi_bb176_29;
  TNode<IntPtrT> phi_bb176_30;
  TNode<IntPtrT> phi_bb176_31;
  TNode<IntPtrT> phi_bb176_34;
  TNode<BoolT> phi_bb176_35;
  TNode<BoolT> phi_bb176_36;
  TNode<Union<FixedArray, Smi>> phi_bb176_37;
  TNode<IntPtrT> phi_bb176_38;
  TNode<IntPtrT> tmp266;
  TNode<IntPtrT> tmp267;
  TNode<IntPtrT> tmp268;
  TNode<BoolT> tmp269;
  if (block176.is_used()) {
    ca_.Bind(&block176, &phi_bb176_17, &phi_bb176_18, &phi_bb176_19, &phi_bb176_27, &phi_bb176_28, &phi_bb176_29, &phi_bb176_30, &phi_bb176_31, &phi_bb176_34, &phi_bb176_35, &phi_bb176_36, &phi_bb176_37, &phi_bb176_38);
    tmp266 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp267 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb176_28}, TNode<IntPtrT>{tmp266});
    tmp268 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp269 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb176_28}, TNode<IntPtrT>{tmp268});
    ca_.Branch(tmp269, &block180, std::vector<compiler::Node*>{phi_bb176_17, phi_bb176_18, phi_bb176_19, phi_bb176_27, phi_bb176_29, phi_bb176_30, phi_bb176_31, phi_bb176_34, phi_bb176_35, phi_bb176_36, phi_bb176_37, phi_bb176_38}, &block181, std::vector<compiler::Node*>{phi_bb176_17, phi_bb176_18, phi_bb176_19, phi_bb176_27, phi_bb176_29, phi_bb176_30, phi_bb176_31, phi_bb176_34, phi_bb176_35, phi_bb176_36, phi_bb176_37, phi_bb176_38});
  }

  TNode<JSAny> phi_bb180_17;
  TNode<IntPtrT> phi_bb180_18;
  TNode<BoolT> phi_bb180_19;
  TNode<IntPtrT> phi_bb180_27;
  TNode<IntPtrT> phi_bb180_29;
  TNode<IntPtrT> phi_bb180_30;
  TNode<IntPtrT> phi_bb180_31;
  TNode<IntPtrT> phi_bb180_34;
  TNode<BoolT> phi_bb180_35;
  TNode<BoolT> phi_bb180_36;
  TNode<Union<FixedArray, Smi>> phi_bb180_37;
  TNode<IntPtrT> phi_bb180_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp270;
  TNode<IntPtrT> tmp271;
  TNode<IntPtrT> tmp272;
  TNode<IntPtrT> tmp273;
  if (block180.is_used()) {
    ca_.Bind(&block180, &phi_bb180_17, &phi_bb180_18, &phi_bb180_19, &phi_bb180_27, &phi_bb180_29, &phi_bb180_30, &phi_bb180_31, &phi_bb180_34, &phi_bb180_35, &phi_bb180_36, &phi_bb180_37, &phi_bb180_38);
    std::tie(tmp270, tmp271) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb180_30}).Flatten();
    tmp272 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp273 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb180_30}, TNode<IntPtrT>{tmp272});
    ca_.Goto(&block179, phi_bb180_17, phi_bb180_18, phi_bb180_19, phi_bb180_27, phi_bb180_29, tmp273, phi_bb180_31, phi_bb180_34, phi_bb180_35, phi_bb180_36, phi_bb180_37, phi_bb180_38, tmp270, tmp271);
  }

  TNode<JSAny> phi_bb181_17;
  TNode<IntPtrT> phi_bb181_18;
  TNode<BoolT> phi_bb181_19;
  TNode<IntPtrT> phi_bb181_27;
  TNode<IntPtrT> phi_bb181_29;
  TNode<IntPtrT> phi_bb181_30;
  TNode<IntPtrT> phi_bb181_31;
  TNode<IntPtrT> phi_bb181_34;
  TNode<BoolT> phi_bb181_35;
  TNode<BoolT> phi_bb181_36;
  TNode<Union<FixedArray, Smi>> phi_bb181_37;
  TNode<IntPtrT> phi_bb181_38;
  if (block181.is_used()) {
    ca_.Bind(&block181, &phi_bb181_17, &phi_bb181_18, &phi_bb181_19, &phi_bb181_27, &phi_bb181_29, &phi_bb181_30, &phi_bb181_31, &phi_bb181_34, &phi_bb181_35, &phi_bb181_36, &phi_bb181_37, &phi_bb181_38);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block182, phi_bb181_17, phi_bb181_18, phi_bb181_19, phi_bb181_27, phi_bb181_29, phi_bb181_30, phi_bb181_31, phi_bb181_34, phi_bb181_35, phi_bb181_36, phi_bb181_37, phi_bb181_38);
    } else {
      ca_.Goto(&block183, phi_bb181_17, phi_bb181_18, phi_bb181_19, phi_bb181_27, phi_bb181_29, phi_bb181_30, phi_bb181_31, phi_bb181_34, phi_bb181_35, phi_bb181_36, phi_bb181_37, phi_bb181_38);
    }
  }

  TNode<JSAny> phi_bb182_17;
  TNode<IntPtrT> phi_bb182_18;
  TNode<BoolT> phi_bb182_19;
  TNode<IntPtrT> phi_bb182_27;
  TNode<IntPtrT> phi_bb182_29;
  TNode<IntPtrT> phi_bb182_30;
  TNode<IntPtrT> phi_bb182_31;
  TNode<IntPtrT> phi_bb182_34;
  TNode<BoolT> phi_bb182_35;
  TNode<BoolT> phi_bb182_36;
  TNode<Union<FixedArray, Smi>> phi_bb182_37;
  TNode<IntPtrT> phi_bb182_38;
  if (block182.is_used()) {
    ca_.Bind(&block182, &phi_bb182_17, &phi_bb182_18, &phi_bb182_19, &phi_bb182_27, &phi_bb182_29, &phi_bb182_30, &phi_bb182_31, &phi_bb182_34, &phi_bb182_35, &phi_bb182_36, &phi_bb182_37, &phi_bb182_38);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block186, phi_bb182_17, phi_bb182_18, phi_bb182_19, phi_bb182_27, phi_bb182_29, phi_bb182_30, phi_bb182_31, phi_bb182_34, phi_bb182_35, phi_bb182_36, phi_bb182_37, phi_bb182_38);
    } else {
      ca_.Goto(&block187, phi_bb182_17, phi_bb182_18, phi_bb182_19, phi_bb182_27, phi_bb182_29, phi_bb182_30, phi_bb182_31, phi_bb182_34, phi_bb182_35, phi_bb182_36, phi_bb182_37, phi_bb182_38);
    }
  }

  TNode<JSAny> phi_bb186_17;
  TNode<IntPtrT> phi_bb186_18;
  TNode<BoolT> phi_bb186_19;
  TNode<IntPtrT> phi_bb186_27;
  TNode<IntPtrT> phi_bb186_29;
  TNode<IntPtrT> phi_bb186_30;
  TNode<IntPtrT> phi_bb186_31;
  TNode<IntPtrT> phi_bb186_34;
  TNode<BoolT> phi_bb186_35;
  TNode<BoolT> phi_bb186_36;
  TNode<Union<FixedArray, Smi>> phi_bb186_37;
  TNode<IntPtrT> phi_bb186_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp274;
  TNode<IntPtrT> tmp275;
  TNode<IntPtrT> tmp276;
  TNode<IntPtrT> tmp277;
  if (block186.is_used()) {
    ca_.Bind(&block186, &phi_bb186_17, &phi_bb186_18, &phi_bb186_19, &phi_bb186_27, &phi_bb186_29, &phi_bb186_30, &phi_bb186_31, &phi_bb186_34, &phi_bb186_35, &phi_bb186_36, &phi_bb186_37, &phi_bb186_38);
    std::tie(tmp274, tmp275) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb186_31}).Flatten();
    tmp276 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp277 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb186_31}, TNode<IntPtrT>{tmp276});
    ca_.Goto(&block185, phi_bb186_17, phi_bb186_18, phi_bb186_19, phi_bb186_27, phi_bb186_29, phi_bb186_30, tmp277, phi_bb186_34, phi_bb186_35, phi_bb186_36, phi_bb186_37, phi_bb186_38, tmp274, tmp275);
  }

  TNode<JSAny> phi_bb187_17;
  TNode<IntPtrT> phi_bb187_18;
  TNode<BoolT> phi_bb187_19;
  TNode<IntPtrT> phi_bb187_27;
  TNode<IntPtrT> phi_bb187_29;
  TNode<IntPtrT> phi_bb187_30;
  TNode<IntPtrT> phi_bb187_31;
  TNode<IntPtrT> phi_bb187_34;
  TNode<BoolT> phi_bb187_35;
  TNode<BoolT> phi_bb187_36;
  TNode<Union<FixedArray, Smi>> phi_bb187_37;
  TNode<IntPtrT> phi_bb187_38;
  TNode<IntPtrT> tmp278;
  TNode<BoolT> tmp279;
  if (block187.is_used()) {
    ca_.Bind(&block187, &phi_bb187_17, &phi_bb187_18, &phi_bb187_19, &phi_bb187_27, &phi_bb187_29, &phi_bb187_30, &phi_bb187_31, &phi_bb187_34, &phi_bb187_35, &phi_bb187_36, &phi_bb187_37, &phi_bb187_38);
    tmp278 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp279 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb187_34}, TNode<IntPtrT>{tmp278});
    ca_.Branch(tmp279, &block189, std::vector<compiler::Node*>{phi_bb187_17, phi_bb187_18, phi_bb187_19, phi_bb187_27, phi_bb187_29, phi_bb187_30, phi_bb187_31, phi_bb187_34, phi_bb187_35, phi_bb187_36, phi_bb187_37, phi_bb187_38}, &block190, std::vector<compiler::Node*>{phi_bb187_17, phi_bb187_18, phi_bb187_19, phi_bb187_27, phi_bb187_29, phi_bb187_30, phi_bb187_31, phi_bb187_34, phi_bb187_35, phi_bb187_36, phi_bb187_37, phi_bb187_38});
  }

  TNode<JSAny> phi_bb189_17;
  TNode<IntPtrT> phi_bb189_18;
  TNode<BoolT> phi_bb189_19;
  TNode<IntPtrT> phi_bb189_27;
  TNode<IntPtrT> phi_bb189_29;
  TNode<IntPtrT> phi_bb189_30;
  TNode<IntPtrT> phi_bb189_31;
  TNode<IntPtrT> phi_bb189_34;
  TNode<BoolT> phi_bb189_35;
  TNode<BoolT> phi_bb189_36;
  TNode<Union<FixedArray, Smi>> phi_bb189_37;
  TNode<IntPtrT> phi_bb189_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp280;
  TNode<IntPtrT> tmp281;
  TNode<IntPtrT> tmp282;
  TNode<BoolT> tmp283;
  if (block189.is_used()) {
    ca_.Bind(&block189, &phi_bb189_17, &phi_bb189_18, &phi_bb189_19, &phi_bb189_27, &phi_bb189_29, &phi_bb189_30, &phi_bb189_31, &phi_bb189_34, &phi_bb189_35, &phi_bb189_36, &phi_bb189_37, &phi_bb189_38);
    std::tie(tmp280, tmp281) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb189_34}).Flatten();
    tmp282 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp283 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block185, phi_bb189_17, phi_bb189_18, phi_bb189_19, phi_bb189_27, phi_bb189_29, phi_bb189_30, phi_bb189_31, tmp282, tmp283, phi_bb189_36, phi_bb189_37, phi_bb189_38, tmp280, tmp281);
  }

  TNode<JSAny> phi_bb190_17;
  TNode<IntPtrT> phi_bb190_18;
  TNode<BoolT> phi_bb190_19;
  TNode<IntPtrT> phi_bb190_27;
  TNode<IntPtrT> phi_bb190_29;
  TNode<IntPtrT> phi_bb190_30;
  TNode<IntPtrT> phi_bb190_31;
  TNode<IntPtrT> phi_bb190_34;
  TNode<BoolT> phi_bb190_35;
  TNode<BoolT> phi_bb190_36;
  TNode<Union<FixedArray, Smi>> phi_bb190_37;
  TNode<IntPtrT> phi_bb190_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp284;
  TNode<IntPtrT> tmp285;
  TNode<IntPtrT> tmp286;
  TNode<IntPtrT> tmp287;
  TNode<IntPtrT> tmp288;
  TNode<IntPtrT> tmp289;
  TNode<BoolT> tmp290;
  if (block190.is_used()) {
    ca_.Bind(&block190, &phi_bb190_17, &phi_bb190_18, &phi_bb190_19, &phi_bb190_27, &phi_bb190_29, &phi_bb190_30, &phi_bb190_31, &phi_bb190_34, &phi_bb190_35, &phi_bb190_36, &phi_bb190_37, &phi_bb190_38);
    std::tie(tmp284, tmp285) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb190_31}).Flatten();
    tmp286 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp287 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb190_31}, TNode<IntPtrT>{tmp286});
    tmp288 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp289 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp287}, TNode<IntPtrT>{tmp288});
    tmp290 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block185, phi_bb190_17, phi_bb190_18, phi_bb190_19, phi_bb190_27, phi_bb190_29, phi_bb190_30, tmp289, tmp287, tmp290, phi_bb190_36, phi_bb190_37, phi_bb190_38, tmp284, tmp285);
  }

  TNode<JSAny> phi_bb185_17;
  TNode<IntPtrT> phi_bb185_18;
  TNode<BoolT> phi_bb185_19;
  TNode<IntPtrT> phi_bb185_27;
  TNode<IntPtrT> phi_bb185_29;
  TNode<IntPtrT> phi_bb185_30;
  TNode<IntPtrT> phi_bb185_31;
  TNode<IntPtrT> phi_bb185_34;
  TNode<BoolT> phi_bb185_35;
  TNode<BoolT> phi_bb185_36;
  TNode<Union<FixedArray, Smi>> phi_bb185_37;
  TNode<IntPtrT> phi_bb185_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb185_41;
  TNode<IntPtrT> phi_bb185_42;
  if (block185.is_used()) {
    ca_.Bind(&block185, &phi_bb185_17, &phi_bb185_18, &phi_bb185_19, &phi_bb185_27, &phi_bb185_29, &phi_bb185_30, &phi_bb185_31, &phi_bb185_34, &phi_bb185_35, &phi_bb185_36, &phi_bb185_37, &phi_bb185_38, &phi_bb185_41, &phi_bb185_42);
    ca_.Goto(&block179, phi_bb185_17, phi_bb185_18, phi_bb185_19, phi_bb185_27, phi_bb185_29, phi_bb185_30, phi_bb185_31, phi_bb185_34, phi_bb185_35, phi_bb185_36, phi_bb185_37, phi_bb185_38, phi_bb185_41, phi_bb185_42);
  }

  TNode<JSAny> phi_bb183_17;
  TNode<IntPtrT> phi_bb183_18;
  TNode<BoolT> phi_bb183_19;
  TNode<IntPtrT> phi_bb183_27;
  TNode<IntPtrT> phi_bb183_29;
  TNode<IntPtrT> phi_bb183_30;
  TNode<IntPtrT> phi_bb183_31;
  TNode<IntPtrT> phi_bb183_34;
  TNode<BoolT> phi_bb183_35;
  TNode<BoolT> phi_bb183_36;
  TNode<Union<FixedArray, Smi>> phi_bb183_37;
  TNode<IntPtrT> phi_bb183_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp291;
  TNode<IntPtrT> tmp292;
  TNode<IntPtrT> tmp293;
  TNode<IntPtrT> tmp294;
  TNode<BoolT> tmp295;
  if (block183.is_used()) {
    ca_.Bind(&block183, &phi_bb183_17, &phi_bb183_18, &phi_bb183_19, &phi_bb183_27, &phi_bb183_29, &phi_bb183_30, &phi_bb183_31, &phi_bb183_34, &phi_bb183_35, &phi_bb183_36, &phi_bb183_37, &phi_bb183_38);
    std::tie(tmp291, tmp292) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb183_31}).Flatten();
    tmp293 = FromConstexpr_intptr_constexpr_int31_0(state_, (CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull))), (SizeOf_intptr_0(state_)))));
    tmp294 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb183_31}, TNode<IntPtrT>{tmp293});
    tmp295 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block179, phi_bb183_17, phi_bb183_18, phi_bb183_19, phi_bb183_27, phi_bb183_29, phi_bb183_30, tmp294, phi_bb183_34, tmp295, phi_bb183_36, phi_bb183_37, phi_bb183_38, tmp291, tmp292);
  }

  TNode<JSAny> phi_bb179_17;
  TNode<IntPtrT> phi_bb179_18;
  TNode<BoolT> phi_bb179_19;
  TNode<IntPtrT> phi_bb179_27;
  TNode<IntPtrT> phi_bb179_29;
  TNode<IntPtrT> phi_bb179_30;
  TNode<IntPtrT> phi_bb179_31;
  TNode<IntPtrT> phi_bb179_34;
  TNode<BoolT> phi_bb179_35;
  TNode<BoolT> phi_bb179_36;
  TNode<Union<FixedArray, Smi>> phi_bb179_37;
  TNode<IntPtrT> phi_bb179_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb179_41;
  TNode<IntPtrT> phi_bb179_42;
  TNode<Union<HeapObject, TaggedIndex>> tmp296;
  TNode<IntPtrT> tmp297;
  TNode<Float64T> tmp298;
  TNode<Float64T> tmp299;
  if (block179.is_used()) {
    ca_.Bind(&block179, &phi_bb179_17, &phi_bb179_18, &phi_bb179_19, &phi_bb179_27, &phi_bb179_29, &phi_bb179_30, &phi_bb179_31, &phi_bb179_34, &phi_bb179_35, &phi_bb179_36, &phi_bb179_37, &phi_bb179_38, &phi_bb179_41, &phi_bb179_42);
    std::tie(tmp296, tmp297) = RefCast_float64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb179_41}, TNode<IntPtrT>{phi_bb179_42}, TorqueStructUnsafe_0{}}).Flatten();
    tmp298 = CodeStubAssembler(state_).ChangeTaggedToFloat64(TNode<Context>{p_context}, TNode<JSAny>{tmp195});
    tmp299 = CodeStubAssembler(state_).Float64SilenceNaN(TNode<Float64T>{tmp298});
    CodeStubAssembler(state_).StoreReference<Float64T>(CodeStubAssembler::Reference{tmp296, tmp297}, tmp299);
    ca_.Goto(&block178, phi_bb179_17, phi_bb179_18, phi_bb179_19, phi_bb179_27, tmp267, phi_bb179_29, phi_bb179_30, phi_bb179_31, phi_bb179_34, phi_bb179_35, phi_bb179_36, phi_bb179_37, phi_bb179_38);
  }

  TNode<JSAny> phi_bb177_17;
  TNode<IntPtrT> phi_bb177_18;
  TNode<BoolT> phi_bb177_19;
  TNode<IntPtrT> phi_bb177_27;
  TNode<IntPtrT> phi_bb177_28;
  TNode<IntPtrT> phi_bb177_29;
  TNode<IntPtrT> phi_bb177_30;
  TNode<IntPtrT> phi_bb177_31;
  TNode<IntPtrT> phi_bb177_34;
  TNode<BoolT> phi_bb177_35;
  TNode<BoolT> phi_bb177_36;
  TNode<Union<FixedArray, Smi>> phi_bb177_37;
  TNode<IntPtrT> phi_bb177_38;
  TNode<Uint32T> tmp300;
  TNode<BoolT> tmp301;
  if (block177.is_used()) {
    ca_.Bind(&block177, &phi_bb177_17, &phi_bb177_18, &phi_bb177_19, &phi_bb177_27, &phi_bb177_28, &phi_bb177_29, &phi_bb177_30, &phi_bb177_31, &phi_bb177_34, &phi_bb177_35, &phi_bb177_36, &phi_bb177_37, &phi_bb177_38);
    tmp300 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI64.raw_bit_field());
    tmp301 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp200}, TNode<Uint32T>{tmp300});
    ca_.Branch(tmp301, &block191, std::vector<compiler::Node*>{phi_bb177_17, phi_bb177_18, phi_bb177_19, phi_bb177_27, phi_bb177_28, phi_bb177_29, phi_bb177_30, phi_bb177_31, phi_bb177_34, phi_bb177_35, phi_bb177_36, phi_bb177_37, phi_bb177_38}, &block192, std::vector<compiler::Node*>{phi_bb177_17, phi_bb177_18, phi_bb177_19, phi_bb177_27, phi_bb177_28, phi_bb177_29, phi_bb177_30, phi_bb177_31, phi_bb177_34, phi_bb177_35, phi_bb177_36, phi_bb177_37, phi_bb177_38});
  }

  TNode<JSAny> phi_bb191_17;
  TNode<IntPtrT> phi_bb191_18;
  TNode<BoolT> phi_bb191_19;
  TNode<IntPtrT> phi_bb191_27;
  TNode<IntPtrT> phi_bb191_28;
  TNode<IntPtrT> phi_bb191_29;
  TNode<IntPtrT> phi_bb191_30;
  TNode<IntPtrT> phi_bb191_31;
  TNode<IntPtrT> phi_bb191_34;
  TNode<BoolT> phi_bb191_35;
  TNode<BoolT> phi_bb191_36;
  TNode<Union<FixedArray, Smi>> phi_bb191_37;
  TNode<IntPtrT> phi_bb191_38;
  if (block191.is_used()) {
    ca_.Bind(&block191, &phi_bb191_17, &phi_bb191_18, &phi_bb191_19, &phi_bb191_27, &phi_bb191_28, &phi_bb191_29, &phi_bb191_30, &phi_bb191_31, &phi_bb191_34, &phi_bb191_35, &phi_bb191_36, &phi_bb191_37, &phi_bb191_38);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block194, phi_bb191_17, phi_bb191_18, phi_bb191_19, phi_bb191_27, phi_bb191_28, phi_bb191_29, phi_bb191_30, phi_bb191_31, phi_bb191_34, phi_bb191_35, phi_bb191_36, phi_bb191_37, phi_bb191_38);
    } else {
      ca_.Goto(&block195, phi_bb191_17, phi_bb191_18, phi_bb191_19, phi_bb191_27, phi_bb191_28, phi_bb191_29, phi_bb191_30, phi_bb191_31, phi_bb191_34, phi_bb191_35, phi_bb191_36, phi_bb191_37, phi_bb191_38);
    }
  }

  TNode<JSAny> phi_bb194_17;
  TNode<IntPtrT> phi_bb194_18;
  TNode<BoolT> phi_bb194_19;
  TNode<IntPtrT> phi_bb194_27;
  TNode<IntPtrT> phi_bb194_28;
  TNode<IntPtrT> phi_bb194_29;
  TNode<IntPtrT> phi_bb194_30;
  TNode<IntPtrT> phi_bb194_31;
  TNode<IntPtrT> phi_bb194_34;
  TNode<BoolT> phi_bb194_35;
  TNode<BoolT> phi_bb194_36;
  TNode<Union<FixedArray, Smi>> phi_bb194_37;
  TNode<IntPtrT> phi_bb194_38;
  TNode<IntPtrT> tmp302;
  TNode<IntPtrT> tmp303;
  TNode<IntPtrT> tmp304;
  TNode<BoolT> tmp305;
  if (block194.is_used()) {
    ca_.Bind(&block194, &phi_bb194_17, &phi_bb194_18, &phi_bb194_19, &phi_bb194_27, &phi_bb194_28, &phi_bb194_29, &phi_bb194_30, &phi_bb194_31, &phi_bb194_34, &phi_bb194_35, &phi_bb194_36, &phi_bb194_37, &phi_bb194_38);
    tmp302 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp303 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb194_27}, TNode<IntPtrT>{tmp302});
    tmp304 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp305 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb194_27}, TNode<IntPtrT>{tmp304});
    ca_.Branch(tmp305, &block198, std::vector<compiler::Node*>{phi_bb194_17, phi_bb194_18, phi_bb194_19, phi_bb194_28, phi_bb194_29, phi_bb194_30, phi_bb194_31, phi_bb194_34, phi_bb194_35, phi_bb194_36, phi_bb194_37, phi_bb194_38}, &block199, std::vector<compiler::Node*>{phi_bb194_17, phi_bb194_18, phi_bb194_19, phi_bb194_28, phi_bb194_29, phi_bb194_30, phi_bb194_31, phi_bb194_34, phi_bb194_35, phi_bb194_36, phi_bb194_37, phi_bb194_38});
  }

  TNode<JSAny> phi_bb198_17;
  TNode<IntPtrT> phi_bb198_18;
  TNode<BoolT> phi_bb198_19;
  TNode<IntPtrT> phi_bb198_28;
  TNode<IntPtrT> phi_bb198_29;
  TNode<IntPtrT> phi_bb198_30;
  TNode<IntPtrT> phi_bb198_31;
  TNode<IntPtrT> phi_bb198_34;
  TNode<BoolT> phi_bb198_35;
  TNode<BoolT> phi_bb198_36;
  TNode<Union<FixedArray, Smi>> phi_bb198_37;
  TNode<IntPtrT> phi_bb198_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp306;
  TNode<IntPtrT> tmp307;
  TNode<IntPtrT> tmp308;
  TNode<IntPtrT> tmp309;
  if (block198.is_used()) {
    ca_.Bind(&block198, &phi_bb198_17, &phi_bb198_18, &phi_bb198_19, &phi_bb198_28, &phi_bb198_29, &phi_bb198_30, &phi_bb198_31, &phi_bb198_34, &phi_bb198_35, &phi_bb198_36, &phi_bb198_37, &phi_bb198_38);
    std::tie(tmp306, tmp307) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb198_29}).Flatten();
    tmp308 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp309 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb198_29}, TNode<IntPtrT>{tmp308});
    ca_.Goto(&block197, phi_bb198_17, phi_bb198_18, phi_bb198_19, phi_bb198_28, tmp309, phi_bb198_30, phi_bb198_31, phi_bb198_34, phi_bb198_35, phi_bb198_36, phi_bb198_37, phi_bb198_38, tmp306, tmp307);
  }

  TNode<JSAny> phi_bb199_17;
  TNode<IntPtrT> phi_bb199_18;
  TNode<BoolT> phi_bb199_19;
  TNode<IntPtrT> phi_bb199_28;
  TNode<IntPtrT> phi_bb199_29;
  TNode<IntPtrT> phi_bb199_30;
  TNode<IntPtrT> phi_bb199_31;
  TNode<IntPtrT> phi_bb199_34;
  TNode<BoolT> phi_bb199_35;
  TNode<BoolT> phi_bb199_36;
  TNode<Union<FixedArray, Smi>> phi_bb199_37;
  TNode<IntPtrT> phi_bb199_38;
  if (block199.is_used()) {
    ca_.Bind(&block199, &phi_bb199_17, &phi_bb199_18, &phi_bb199_19, &phi_bb199_28, &phi_bb199_29, &phi_bb199_30, &phi_bb199_31, &phi_bb199_34, &phi_bb199_35, &phi_bb199_36, &phi_bb199_37, &phi_bb199_38);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block201, phi_bb199_17, phi_bb199_18, phi_bb199_19, phi_bb199_28, phi_bb199_29, phi_bb199_30, phi_bb199_31, phi_bb199_34, phi_bb199_35, phi_bb199_36, phi_bb199_37, phi_bb199_38);
    } else {
      ca_.Goto(&block202, phi_bb199_17, phi_bb199_18, phi_bb199_19, phi_bb199_28, phi_bb199_29, phi_bb199_30, phi_bb199_31, phi_bb199_34, phi_bb199_35, phi_bb199_36, phi_bb199_37, phi_bb199_38);
    }
  }

  TNode<JSAny> phi_bb201_17;
  TNode<IntPtrT> phi_bb201_18;
  TNode<BoolT> phi_bb201_19;
  TNode<IntPtrT> phi_bb201_28;
  TNode<IntPtrT> phi_bb201_29;
  TNode<IntPtrT> phi_bb201_30;
  TNode<IntPtrT> phi_bb201_31;
  TNode<IntPtrT> phi_bb201_34;
  TNode<BoolT> phi_bb201_35;
  TNode<BoolT> phi_bb201_36;
  TNode<Union<FixedArray, Smi>> phi_bb201_37;
  TNode<IntPtrT> phi_bb201_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp310;
  TNode<IntPtrT> tmp311;
  TNode<IntPtrT> tmp312;
  TNode<IntPtrT> tmp313;
  if (block201.is_used()) {
    ca_.Bind(&block201, &phi_bb201_17, &phi_bb201_18, &phi_bb201_19, &phi_bb201_28, &phi_bb201_29, &phi_bb201_30, &phi_bb201_31, &phi_bb201_34, &phi_bb201_35, &phi_bb201_36, &phi_bb201_37, &phi_bb201_38);
    std::tie(tmp310, tmp311) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb201_31}).Flatten();
    tmp312 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp313 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb201_31}, TNode<IntPtrT>{tmp312});
    ca_.Goto(&block200, phi_bb201_17, phi_bb201_18, phi_bb201_19, phi_bb201_28, phi_bb201_29, phi_bb201_30, tmp313, phi_bb201_34, phi_bb201_35, phi_bb201_36, phi_bb201_37, phi_bb201_38, tmp310, tmp311);
  }

  TNode<JSAny> phi_bb202_17;
  TNode<IntPtrT> phi_bb202_18;
  TNode<BoolT> phi_bb202_19;
  TNode<IntPtrT> phi_bb202_28;
  TNode<IntPtrT> phi_bb202_29;
  TNode<IntPtrT> phi_bb202_30;
  TNode<IntPtrT> phi_bb202_31;
  TNode<IntPtrT> phi_bb202_34;
  TNode<BoolT> phi_bb202_35;
  TNode<BoolT> phi_bb202_36;
  TNode<Union<FixedArray, Smi>> phi_bb202_37;
  TNode<IntPtrT> phi_bb202_38;
  TNode<IntPtrT> tmp314;
  TNode<BoolT> tmp315;
  if (block202.is_used()) {
    ca_.Bind(&block202, &phi_bb202_17, &phi_bb202_18, &phi_bb202_19, &phi_bb202_28, &phi_bb202_29, &phi_bb202_30, &phi_bb202_31, &phi_bb202_34, &phi_bb202_35, &phi_bb202_36, &phi_bb202_37, &phi_bb202_38);
    tmp314 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp315 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb202_34}, TNode<IntPtrT>{tmp314});
    ca_.Branch(tmp315, &block204, std::vector<compiler::Node*>{phi_bb202_17, phi_bb202_18, phi_bb202_19, phi_bb202_28, phi_bb202_29, phi_bb202_30, phi_bb202_31, phi_bb202_34, phi_bb202_35, phi_bb202_36, phi_bb202_37, phi_bb202_38}, &block205, std::vector<compiler::Node*>{phi_bb202_17, phi_bb202_18, phi_bb202_19, phi_bb202_28, phi_bb202_29, phi_bb202_30, phi_bb202_31, phi_bb202_34, phi_bb202_35, phi_bb202_36, phi_bb202_37, phi_bb202_38});
  }

  TNode<JSAny> phi_bb204_17;
  TNode<IntPtrT> phi_bb204_18;
  TNode<BoolT> phi_bb204_19;
  TNode<IntPtrT> phi_bb204_28;
  TNode<IntPtrT> phi_bb204_29;
  TNode<IntPtrT> phi_bb204_30;
  TNode<IntPtrT> phi_bb204_31;
  TNode<IntPtrT> phi_bb204_34;
  TNode<BoolT> phi_bb204_35;
  TNode<BoolT> phi_bb204_36;
  TNode<Union<FixedArray, Smi>> phi_bb204_37;
  TNode<IntPtrT> phi_bb204_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp316;
  TNode<IntPtrT> tmp317;
  TNode<IntPtrT> tmp318;
  TNode<BoolT> tmp319;
  if (block204.is_used()) {
    ca_.Bind(&block204, &phi_bb204_17, &phi_bb204_18, &phi_bb204_19, &phi_bb204_28, &phi_bb204_29, &phi_bb204_30, &phi_bb204_31, &phi_bb204_34, &phi_bb204_35, &phi_bb204_36, &phi_bb204_37, &phi_bb204_38);
    std::tie(tmp316, tmp317) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb204_34}).Flatten();
    tmp318 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp319 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block200, phi_bb204_17, phi_bb204_18, phi_bb204_19, phi_bb204_28, phi_bb204_29, phi_bb204_30, phi_bb204_31, tmp318, tmp319, phi_bb204_36, phi_bb204_37, phi_bb204_38, tmp316, tmp317);
  }

  TNode<JSAny> phi_bb205_17;
  TNode<IntPtrT> phi_bb205_18;
  TNode<BoolT> phi_bb205_19;
  TNode<IntPtrT> phi_bb205_28;
  TNode<IntPtrT> phi_bb205_29;
  TNode<IntPtrT> phi_bb205_30;
  TNode<IntPtrT> phi_bb205_31;
  TNode<IntPtrT> phi_bb205_34;
  TNode<BoolT> phi_bb205_35;
  TNode<BoolT> phi_bb205_36;
  TNode<Union<FixedArray, Smi>> phi_bb205_37;
  TNode<IntPtrT> phi_bb205_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp320;
  TNode<IntPtrT> tmp321;
  TNode<IntPtrT> tmp322;
  TNode<IntPtrT> tmp323;
  TNode<IntPtrT> tmp324;
  TNode<IntPtrT> tmp325;
  TNode<BoolT> tmp326;
  if (block205.is_used()) {
    ca_.Bind(&block205, &phi_bb205_17, &phi_bb205_18, &phi_bb205_19, &phi_bb205_28, &phi_bb205_29, &phi_bb205_30, &phi_bb205_31, &phi_bb205_34, &phi_bb205_35, &phi_bb205_36, &phi_bb205_37, &phi_bb205_38);
    std::tie(tmp320, tmp321) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb205_31}).Flatten();
    tmp322 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp323 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb205_31}, TNode<IntPtrT>{tmp322});
    tmp324 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp325 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp323}, TNode<IntPtrT>{tmp324});
    tmp326 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block200, phi_bb205_17, phi_bb205_18, phi_bb205_19, phi_bb205_28, phi_bb205_29, phi_bb205_30, tmp325, tmp323, tmp326, phi_bb205_36, phi_bb205_37, phi_bb205_38, tmp320, tmp321);
  }

  TNode<JSAny> phi_bb200_17;
  TNode<IntPtrT> phi_bb200_18;
  TNode<BoolT> phi_bb200_19;
  TNode<IntPtrT> phi_bb200_28;
  TNode<IntPtrT> phi_bb200_29;
  TNode<IntPtrT> phi_bb200_30;
  TNode<IntPtrT> phi_bb200_31;
  TNode<IntPtrT> phi_bb200_34;
  TNode<BoolT> phi_bb200_35;
  TNode<BoolT> phi_bb200_36;
  TNode<Union<FixedArray, Smi>> phi_bb200_37;
  TNode<IntPtrT> phi_bb200_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb200_41;
  TNode<IntPtrT> phi_bb200_42;
  if (block200.is_used()) {
    ca_.Bind(&block200, &phi_bb200_17, &phi_bb200_18, &phi_bb200_19, &phi_bb200_28, &phi_bb200_29, &phi_bb200_30, &phi_bb200_31, &phi_bb200_34, &phi_bb200_35, &phi_bb200_36, &phi_bb200_37, &phi_bb200_38, &phi_bb200_41, &phi_bb200_42);
    ca_.Goto(&block197, phi_bb200_17, phi_bb200_18, phi_bb200_19, phi_bb200_28, phi_bb200_29, phi_bb200_30, phi_bb200_31, phi_bb200_34, phi_bb200_35, phi_bb200_36, phi_bb200_37, phi_bb200_38, phi_bb200_41, phi_bb200_42);
  }

  TNode<JSAny> phi_bb197_17;
  TNode<IntPtrT> phi_bb197_18;
  TNode<BoolT> phi_bb197_19;
  TNode<IntPtrT> phi_bb197_28;
  TNode<IntPtrT> phi_bb197_29;
  TNode<IntPtrT> phi_bb197_30;
  TNode<IntPtrT> phi_bb197_31;
  TNode<IntPtrT> phi_bb197_34;
  TNode<BoolT> phi_bb197_35;
  TNode<BoolT> phi_bb197_36;
  TNode<Union<FixedArray, Smi>> phi_bb197_37;
  TNode<IntPtrT> phi_bb197_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb197_41;
  TNode<IntPtrT> phi_bb197_42;
  TNode<IntPtrT> tmp327;
  if (block197.is_used()) {
    ca_.Bind(&block197, &phi_bb197_17, &phi_bb197_18, &phi_bb197_19, &phi_bb197_28, &phi_bb197_29, &phi_bb197_30, &phi_bb197_31, &phi_bb197_34, &phi_bb197_35, &phi_bb197_36, &phi_bb197_37, &phi_bb197_38, &phi_bb197_41, &phi_bb197_42);
    tmp327 = TruncateBigIntToI64_0(state_, TNode<Context>{p_context}, TNode<JSAny>{tmp195});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb197_41, phi_bb197_42}, tmp327);
    ca_.Goto(&block196, phi_bb197_17, phi_bb197_18, phi_bb197_19, tmp303, phi_bb197_28, phi_bb197_29, phi_bb197_30, phi_bb197_31, phi_bb197_34, phi_bb197_35, phi_bb197_36, phi_bb197_37, phi_bb197_38);
  }

  TNode<JSAny> phi_bb195_17;
  TNode<IntPtrT> phi_bb195_18;
  TNode<BoolT> phi_bb195_19;
  TNode<IntPtrT> phi_bb195_27;
  TNode<IntPtrT> phi_bb195_28;
  TNode<IntPtrT> phi_bb195_29;
  TNode<IntPtrT> phi_bb195_30;
  TNode<IntPtrT> phi_bb195_31;
  TNode<IntPtrT> phi_bb195_34;
  TNode<BoolT> phi_bb195_35;
  TNode<BoolT> phi_bb195_36;
  TNode<Union<FixedArray, Smi>> phi_bb195_37;
  TNode<IntPtrT> phi_bb195_38;
  TNode<IntPtrT> tmp328;
  TNode<IntPtrT> tmp329;
  TNode<IntPtrT> tmp330;
  TNode<BoolT> tmp331;
  if (block195.is_used()) {
    ca_.Bind(&block195, &phi_bb195_17, &phi_bb195_18, &phi_bb195_19, &phi_bb195_27, &phi_bb195_28, &phi_bb195_29, &phi_bb195_30, &phi_bb195_31, &phi_bb195_34, &phi_bb195_35, &phi_bb195_36, &phi_bb195_37, &phi_bb195_38);
    tmp328 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp329 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb195_27}, TNode<IntPtrT>{tmp328});
    tmp330 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp331 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb195_27}, TNode<IntPtrT>{tmp330});
    ca_.Branch(tmp331, &block207, std::vector<compiler::Node*>{phi_bb195_17, phi_bb195_18, phi_bb195_19, phi_bb195_28, phi_bb195_29, phi_bb195_30, phi_bb195_31, phi_bb195_34, phi_bb195_35, phi_bb195_36, phi_bb195_37, phi_bb195_38}, &block208, std::vector<compiler::Node*>{phi_bb195_17, phi_bb195_18, phi_bb195_19, phi_bb195_28, phi_bb195_29, phi_bb195_30, phi_bb195_31, phi_bb195_34, phi_bb195_35, phi_bb195_36, phi_bb195_37, phi_bb195_38});
  }

  TNode<JSAny> phi_bb207_17;
  TNode<IntPtrT> phi_bb207_18;
  TNode<BoolT> phi_bb207_19;
  TNode<IntPtrT> phi_bb207_28;
  TNode<IntPtrT> phi_bb207_29;
  TNode<IntPtrT> phi_bb207_30;
  TNode<IntPtrT> phi_bb207_31;
  TNode<IntPtrT> phi_bb207_34;
  TNode<BoolT> phi_bb207_35;
  TNode<BoolT> phi_bb207_36;
  TNode<Union<FixedArray, Smi>> phi_bb207_37;
  TNode<IntPtrT> phi_bb207_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp332;
  TNode<IntPtrT> tmp333;
  TNode<IntPtrT> tmp334;
  TNode<IntPtrT> tmp335;
  if (block207.is_used()) {
    ca_.Bind(&block207, &phi_bb207_17, &phi_bb207_18, &phi_bb207_19, &phi_bb207_28, &phi_bb207_29, &phi_bb207_30, &phi_bb207_31, &phi_bb207_34, &phi_bb207_35, &phi_bb207_36, &phi_bb207_37, &phi_bb207_38);
    std::tie(tmp332, tmp333) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb207_29}).Flatten();
    tmp334 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp335 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb207_29}, TNode<IntPtrT>{tmp334});
    ca_.Goto(&block206, phi_bb207_17, phi_bb207_18, phi_bb207_19, phi_bb207_28, tmp335, phi_bb207_30, phi_bb207_31, phi_bb207_34, phi_bb207_35, phi_bb207_36, phi_bb207_37, phi_bb207_38, tmp332, tmp333);
  }

  TNode<JSAny> phi_bb208_17;
  TNode<IntPtrT> phi_bb208_18;
  TNode<BoolT> phi_bb208_19;
  TNode<IntPtrT> phi_bb208_28;
  TNode<IntPtrT> phi_bb208_29;
  TNode<IntPtrT> phi_bb208_30;
  TNode<IntPtrT> phi_bb208_31;
  TNode<IntPtrT> phi_bb208_34;
  TNode<BoolT> phi_bb208_35;
  TNode<BoolT> phi_bb208_36;
  TNode<Union<FixedArray, Smi>> phi_bb208_37;
  TNode<IntPtrT> phi_bb208_38;
  if (block208.is_used()) {
    ca_.Bind(&block208, &phi_bb208_17, &phi_bb208_18, &phi_bb208_19, &phi_bb208_28, &phi_bb208_29, &phi_bb208_30, &phi_bb208_31, &phi_bb208_34, &phi_bb208_35, &phi_bb208_36, &phi_bb208_37, &phi_bb208_38);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block210, phi_bb208_17, phi_bb208_18, phi_bb208_19, phi_bb208_28, phi_bb208_29, phi_bb208_30, phi_bb208_31, phi_bb208_34, phi_bb208_35, phi_bb208_36, phi_bb208_37, phi_bb208_38);
    } else {
      ca_.Goto(&block211, phi_bb208_17, phi_bb208_18, phi_bb208_19, phi_bb208_28, phi_bb208_29, phi_bb208_30, phi_bb208_31, phi_bb208_34, phi_bb208_35, phi_bb208_36, phi_bb208_37, phi_bb208_38);
    }
  }

  TNode<JSAny> phi_bb210_17;
  TNode<IntPtrT> phi_bb210_18;
  TNode<BoolT> phi_bb210_19;
  TNode<IntPtrT> phi_bb210_28;
  TNode<IntPtrT> phi_bb210_29;
  TNode<IntPtrT> phi_bb210_30;
  TNode<IntPtrT> phi_bb210_31;
  TNode<IntPtrT> phi_bb210_34;
  TNode<BoolT> phi_bb210_35;
  TNode<BoolT> phi_bb210_36;
  TNode<Union<FixedArray, Smi>> phi_bb210_37;
  TNode<IntPtrT> phi_bb210_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp336;
  TNode<IntPtrT> tmp337;
  TNode<IntPtrT> tmp338;
  TNode<IntPtrT> tmp339;
  if (block210.is_used()) {
    ca_.Bind(&block210, &phi_bb210_17, &phi_bb210_18, &phi_bb210_19, &phi_bb210_28, &phi_bb210_29, &phi_bb210_30, &phi_bb210_31, &phi_bb210_34, &phi_bb210_35, &phi_bb210_36, &phi_bb210_37, &phi_bb210_38);
    std::tie(tmp336, tmp337) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb210_31}).Flatten();
    tmp338 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp339 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb210_31}, TNode<IntPtrT>{tmp338});
    ca_.Goto(&block209, phi_bb210_17, phi_bb210_18, phi_bb210_19, phi_bb210_28, phi_bb210_29, phi_bb210_30, tmp339, phi_bb210_34, phi_bb210_35, phi_bb210_36, phi_bb210_37, phi_bb210_38, tmp336, tmp337);
  }

  TNode<JSAny> phi_bb211_17;
  TNode<IntPtrT> phi_bb211_18;
  TNode<BoolT> phi_bb211_19;
  TNode<IntPtrT> phi_bb211_28;
  TNode<IntPtrT> phi_bb211_29;
  TNode<IntPtrT> phi_bb211_30;
  TNode<IntPtrT> phi_bb211_31;
  TNode<IntPtrT> phi_bb211_34;
  TNode<BoolT> phi_bb211_35;
  TNode<BoolT> phi_bb211_36;
  TNode<Union<FixedArray, Smi>> phi_bb211_37;
  TNode<IntPtrT> phi_bb211_38;
  TNode<IntPtrT> tmp340;
  TNode<BoolT> tmp341;
  if (block211.is_used()) {
    ca_.Bind(&block211, &phi_bb211_17, &phi_bb211_18, &phi_bb211_19, &phi_bb211_28, &phi_bb211_29, &phi_bb211_30, &phi_bb211_31, &phi_bb211_34, &phi_bb211_35, &phi_bb211_36, &phi_bb211_37, &phi_bb211_38);
    tmp340 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp341 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb211_34}, TNode<IntPtrT>{tmp340});
    ca_.Branch(tmp341, &block213, std::vector<compiler::Node*>{phi_bb211_17, phi_bb211_18, phi_bb211_19, phi_bb211_28, phi_bb211_29, phi_bb211_30, phi_bb211_31, phi_bb211_34, phi_bb211_35, phi_bb211_36, phi_bb211_37, phi_bb211_38}, &block214, std::vector<compiler::Node*>{phi_bb211_17, phi_bb211_18, phi_bb211_19, phi_bb211_28, phi_bb211_29, phi_bb211_30, phi_bb211_31, phi_bb211_34, phi_bb211_35, phi_bb211_36, phi_bb211_37, phi_bb211_38});
  }

  TNode<JSAny> phi_bb213_17;
  TNode<IntPtrT> phi_bb213_18;
  TNode<BoolT> phi_bb213_19;
  TNode<IntPtrT> phi_bb213_28;
  TNode<IntPtrT> phi_bb213_29;
  TNode<IntPtrT> phi_bb213_30;
  TNode<IntPtrT> phi_bb213_31;
  TNode<IntPtrT> phi_bb213_34;
  TNode<BoolT> phi_bb213_35;
  TNode<BoolT> phi_bb213_36;
  TNode<Union<FixedArray, Smi>> phi_bb213_37;
  TNode<IntPtrT> phi_bb213_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp342;
  TNode<IntPtrT> tmp343;
  TNode<IntPtrT> tmp344;
  TNode<BoolT> tmp345;
  if (block213.is_used()) {
    ca_.Bind(&block213, &phi_bb213_17, &phi_bb213_18, &phi_bb213_19, &phi_bb213_28, &phi_bb213_29, &phi_bb213_30, &phi_bb213_31, &phi_bb213_34, &phi_bb213_35, &phi_bb213_36, &phi_bb213_37, &phi_bb213_38);
    std::tie(tmp342, tmp343) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb213_34}).Flatten();
    tmp344 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp345 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block209, phi_bb213_17, phi_bb213_18, phi_bb213_19, phi_bb213_28, phi_bb213_29, phi_bb213_30, phi_bb213_31, tmp344, tmp345, phi_bb213_36, phi_bb213_37, phi_bb213_38, tmp342, tmp343);
  }

  TNode<JSAny> phi_bb214_17;
  TNode<IntPtrT> phi_bb214_18;
  TNode<BoolT> phi_bb214_19;
  TNode<IntPtrT> phi_bb214_28;
  TNode<IntPtrT> phi_bb214_29;
  TNode<IntPtrT> phi_bb214_30;
  TNode<IntPtrT> phi_bb214_31;
  TNode<IntPtrT> phi_bb214_34;
  TNode<BoolT> phi_bb214_35;
  TNode<BoolT> phi_bb214_36;
  TNode<Union<FixedArray, Smi>> phi_bb214_37;
  TNode<IntPtrT> phi_bb214_38;
  TNode<Union<HeapObject, TaggedIndex>> tmp346;
  TNode<IntPtrT> tmp347;
  TNode<IntPtrT> tmp348;
  TNode<IntPtrT> tmp349;
  TNode<IntPtrT> tmp350;
  TNode<IntPtrT> tmp351;
  TNode<BoolT> tmp352;
  if (block214.is_used()) {
    ca_.Bind(&block214, &phi_bb214_17, &phi_bb214_18, &phi_bb214_19, &phi_bb214_28, &phi_bb214_29, &phi_bb214_30, &phi_bb214_31, &phi_bb214_34, &phi_bb214_35, &phi_bb214_36, &phi_bb214_37, &phi_bb214_38);
    std::tie(tmp346, tmp347) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb214_31}).Flatten();
    tmp348 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp349 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb214_31}, TNode<IntPtrT>{tmp348});
    tmp350 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp351 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp349}, TNode<IntPtrT>{tmp350});
    tmp352 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block209, phi_bb214_17, phi_bb214_18, phi_bb214_19, phi_bb214_28, phi_bb214_29, phi_bb214_30, tmp351, tmp349, tmp352, phi_bb214_36, phi_bb214_37, phi_bb214_38, tmp346, tmp347);
  }

  TNode<JSAny> phi_bb209_17;
  TNode<IntPtrT> phi_bb209_18;
  TNode<BoolT> phi_bb209_19;
  TNode<IntPtrT> phi_bb209_28;
  TNode<IntPtrT> phi_bb209_29;
  TNode<IntPtrT> phi_bb209_30;
  TNode<IntPtrT> phi_bb209_31;
  TNode<IntPtrT> phi_bb209_34;
  TNode<BoolT> phi_bb209_35;
  TNode<BoolT> phi_bb209_36;
  TNode<Union<FixedArray, Smi>> phi_bb209_37;
  TNode<IntPtrT> phi_bb209_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb209_41;
  TNode<IntPtrT> phi_bb209_42;
  if (block209.is_used()) {
    ca_.Bind(&block209, &phi_bb209_17, &phi_bb209_18, &phi_bb209_19, &phi_bb209_28, &phi_bb209_29, &phi_bb209_30, &phi_bb209_31, &phi_bb209_34, &phi_bb209_35, &phi_bb209_36, &phi_bb209_37, &phi_bb209_38, &phi_bb209_41, &phi_bb209_42);
    ca_.Goto(&block206, phi_bb209_17, phi_bb209_18, phi_bb209_19, phi_bb209_28, phi_bb209_29, phi_bb209_30, phi_bb209_31, phi_bb209_34, phi_bb209_35, phi_bb209_36, phi_bb209_37, phi_bb209_38, phi_bb209_41, phi_bb209_42);
  }

  TNode<JSAny> phi_bb206_17;
  TNode<IntPtrT> phi_bb206_18;
  TNode<BoolT> phi_bb206_19;
  TNode<IntPtrT> phi_bb206_28;
  TNode<IntPtrT> phi_bb206_29;
  TNode<IntPtrT> phi_bb206_30;
  TNode<IntPtrT> phi_bb206_31;
  TNode<IntPtrT> phi_bb206_34;
  TNode<BoolT> phi_bb206_35;
  TNode<BoolT> phi_bb206_36;
  TNode<Union<FixedArray, Smi>> phi_bb206_37;
  TNode<IntPtrT> phi_bb206_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb206_41;
  TNode<IntPtrT> phi_bb206_42;
  TNode<IntPtrT> tmp353;
  TNode<IntPtrT> tmp354;
  TNode<IntPtrT> tmp355;
  TNode<BoolT> tmp356;
  if (block206.is_used()) {
    ca_.Bind(&block206, &phi_bb206_17, &phi_bb206_18, &phi_bb206_19, &phi_bb206_28, &phi_bb206_29, &phi_bb206_30, &phi_bb206_31, &phi_bb206_34, &phi_bb206_35, &phi_bb206_36, &phi_bb206_37, &phi_bb206_38, &phi_bb206_41, &phi_bb206_42);
    tmp353 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp354 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp329}, TNode<IntPtrT>{tmp353});
    tmp355 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp356 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp329}, TNode<IntPtrT>{tmp355});
    ca_.Branch(tmp356, &block216, std::vector<compiler::Node*>{phi_bb206_17, phi_bb206_18, phi_bb206_19, phi_bb206_28, phi_bb206_29, phi_bb206_30, phi_bb206_31, phi_bb206_34, phi_bb206_35, phi_bb206_36, phi_bb206_37, phi_bb206_38, phi_bb206_41, phi_bb206_42}, &block217, std::vector<compiler::Node*>{phi_bb206_17, phi_bb206_18, phi_bb206_19, phi_bb206_28, phi_bb206_29, phi_bb206_30, phi_bb206_31, phi_bb206_34, phi_bb206_35, phi_bb206_36, phi_bb206_37, phi_bb206_38, phi_bb206_41, phi_bb206_42});
  }

  TNode<JSAny> phi_bb216_17;
  TNode<IntPtrT> phi_bb216_18;
  TNode<BoolT> phi_bb216_19;
  TNode<IntPtrT> phi_bb216_28;
  TNode<IntPtrT> phi_bb216_29;
  TNode<IntPtrT> phi_bb216_30;
  TNode<IntPtrT> phi_bb216_31;
  TNode<IntPtrT> phi_bb216_34;
  TNode<BoolT> phi_bb216_35;
  TNode<BoolT> phi_bb216_36;
  TNode<Union<FixedArray, Smi>> phi_bb216_37;
  TNode<IntPtrT> phi_bb216_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb216_41;
  TNode<IntPtrT> phi_bb216_42;
  TNode<Union<HeapObject, TaggedIndex>> tmp357;
  TNode<IntPtrT> tmp358;
  TNode<IntPtrT> tmp359;
  TNode<IntPtrT> tmp360;
  if (block216.is_used()) {
    ca_.Bind(&block216, &phi_bb216_17, &phi_bb216_18, &phi_bb216_19, &phi_bb216_28, &phi_bb216_29, &phi_bb216_30, &phi_bb216_31, &phi_bb216_34, &phi_bb216_35, &phi_bb216_36, &phi_bb216_37, &phi_bb216_38, &phi_bb216_41, &phi_bb216_42);
    std::tie(tmp357, tmp358) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb216_29}).Flatten();
    tmp359 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp360 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb216_29}, TNode<IntPtrT>{tmp359});
    ca_.Goto(&block215, phi_bb216_17, phi_bb216_18, phi_bb216_19, phi_bb216_28, tmp360, phi_bb216_30, phi_bb216_31, phi_bb216_34, phi_bb216_35, phi_bb216_36, phi_bb216_37, phi_bb216_38, phi_bb216_41, phi_bb216_42, tmp357, tmp358);
  }

  TNode<JSAny> phi_bb217_17;
  TNode<IntPtrT> phi_bb217_18;
  TNode<BoolT> phi_bb217_19;
  TNode<IntPtrT> phi_bb217_28;
  TNode<IntPtrT> phi_bb217_29;
  TNode<IntPtrT> phi_bb217_30;
  TNode<IntPtrT> phi_bb217_31;
  TNode<IntPtrT> phi_bb217_34;
  TNode<BoolT> phi_bb217_35;
  TNode<BoolT> phi_bb217_36;
  TNode<Union<FixedArray, Smi>> phi_bb217_37;
  TNode<IntPtrT> phi_bb217_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb217_41;
  TNode<IntPtrT> phi_bb217_42;
  if (block217.is_used()) {
    ca_.Bind(&block217, &phi_bb217_17, &phi_bb217_18, &phi_bb217_19, &phi_bb217_28, &phi_bb217_29, &phi_bb217_30, &phi_bb217_31, &phi_bb217_34, &phi_bb217_35, &phi_bb217_36, &phi_bb217_37, &phi_bb217_38, &phi_bb217_41, &phi_bb217_42);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block219, phi_bb217_17, phi_bb217_18, phi_bb217_19, phi_bb217_28, phi_bb217_29, phi_bb217_30, phi_bb217_31, phi_bb217_34, phi_bb217_35, phi_bb217_36, phi_bb217_37, phi_bb217_38, phi_bb217_41, phi_bb217_42);
    } else {
      ca_.Goto(&block220, phi_bb217_17, phi_bb217_18, phi_bb217_19, phi_bb217_28, phi_bb217_29, phi_bb217_30, phi_bb217_31, phi_bb217_34, phi_bb217_35, phi_bb217_36, phi_bb217_37, phi_bb217_38, phi_bb217_41, phi_bb217_42);
    }
  }

  TNode<JSAny> phi_bb219_17;
  TNode<IntPtrT> phi_bb219_18;
  TNode<BoolT> phi_bb219_19;
  TNode<IntPtrT> phi_bb219_28;
  TNode<IntPtrT> phi_bb219_29;
  TNode<IntPtrT> phi_bb219_30;
  TNode<IntPtrT> phi_bb219_31;
  TNode<IntPtrT> phi_bb219_34;
  TNode<BoolT> phi_bb219_35;
  TNode<BoolT> phi_bb219_36;
  TNode<Union<FixedArray, Smi>> phi_bb219_37;
  TNode<IntPtrT> phi_bb219_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb219_41;
  TNode<IntPtrT> phi_bb219_42;
  TNode<Union<HeapObject, TaggedIndex>> tmp361;
  TNode<IntPtrT> tmp362;
  TNode<IntPtrT> tmp363;
  TNode<IntPtrT> tmp364;
  if (block219.is_used()) {
    ca_.Bind(&block219, &phi_bb219_17, &phi_bb219_18, &phi_bb219_19, &phi_bb219_28, &phi_bb219_29, &phi_bb219_30, &phi_bb219_31, &phi_bb219_34, &phi_bb219_35, &phi_bb219_36, &phi_bb219_37, &phi_bb219_38, &phi_bb219_41, &phi_bb219_42);
    std::tie(tmp361, tmp362) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb219_31}).Flatten();
    tmp363 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp364 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb219_31}, TNode<IntPtrT>{tmp363});
    ca_.Goto(&block218, phi_bb219_17, phi_bb219_18, phi_bb219_19, phi_bb219_28, phi_bb219_29, phi_bb219_30, tmp364, phi_bb219_34, phi_bb219_35, phi_bb219_36, phi_bb219_37, phi_bb219_38, phi_bb219_41, phi_bb219_42, tmp361, tmp362);
  }

  TNode<JSAny> phi_bb220_17;
  TNode<IntPtrT> phi_bb220_18;
  TNode<BoolT> phi_bb220_19;
  TNode<IntPtrT> phi_bb220_28;
  TNode<IntPtrT> phi_bb220_29;
  TNode<IntPtrT> phi_bb220_30;
  TNode<IntPtrT> phi_bb220_31;
  TNode<IntPtrT> phi_bb220_34;
  TNode<BoolT> phi_bb220_35;
  TNode<BoolT> phi_bb220_36;
  TNode<Union<FixedArray, Smi>> phi_bb220_37;
  TNode<IntPtrT> phi_bb220_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb220_41;
  TNode<IntPtrT> phi_bb220_42;
  TNode<IntPtrT> tmp365;
  TNode<BoolT> tmp366;
  if (block220.is_used()) {
    ca_.Bind(&block220, &phi_bb220_17, &phi_bb220_18, &phi_bb220_19, &phi_bb220_28, &phi_bb220_29, &phi_bb220_30, &phi_bb220_31, &phi_bb220_34, &phi_bb220_35, &phi_bb220_36, &phi_bb220_37, &phi_bb220_38, &phi_bb220_41, &phi_bb220_42);
    tmp365 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp366 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb220_34}, TNode<IntPtrT>{tmp365});
    ca_.Branch(tmp366, &block222, std::vector<compiler::Node*>{phi_bb220_17, phi_bb220_18, phi_bb220_19, phi_bb220_28, phi_bb220_29, phi_bb220_30, phi_bb220_31, phi_bb220_34, phi_bb220_35, phi_bb220_36, phi_bb220_37, phi_bb220_38, phi_bb220_41, phi_bb220_42}, &block223, std::vector<compiler::Node*>{phi_bb220_17, phi_bb220_18, phi_bb220_19, phi_bb220_28, phi_bb220_29, phi_bb220_30, phi_bb220_31, phi_bb220_34, phi_bb220_35, phi_bb220_36, phi_bb220_37, phi_bb220_38, phi_bb220_41, phi_bb220_42});
  }

  TNode<JSAny> phi_bb222_17;
  TNode<IntPtrT> phi_bb222_18;
  TNode<BoolT> phi_bb222_19;
  TNode<IntPtrT> phi_bb222_28;
  TNode<IntPtrT> phi_bb222_29;
  TNode<IntPtrT> phi_bb222_30;
  TNode<IntPtrT> phi_bb222_31;
  TNode<IntPtrT> phi_bb222_34;
  TNode<BoolT> phi_bb222_35;
  TNode<BoolT> phi_bb222_36;
  TNode<Union<FixedArray, Smi>> phi_bb222_37;
  TNode<IntPtrT> phi_bb222_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb222_41;
  TNode<IntPtrT> phi_bb222_42;
  TNode<Union<HeapObject, TaggedIndex>> tmp367;
  TNode<IntPtrT> tmp368;
  TNode<IntPtrT> tmp369;
  TNode<BoolT> tmp370;
  if (block222.is_used()) {
    ca_.Bind(&block222, &phi_bb222_17, &phi_bb222_18, &phi_bb222_19, &phi_bb222_28, &phi_bb222_29, &phi_bb222_30, &phi_bb222_31, &phi_bb222_34, &phi_bb222_35, &phi_bb222_36, &phi_bb222_37, &phi_bb222_38, &phi_bb222_41, &phi_bb222_42);
    std::tie(tmp367, tmp368) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb222_34}).Flatten();
    tmp369 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp370 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block218, phi_bb222_17, phi_bb222_18, phi_bb222_19, phi_bb222_28, phi_bb222_29, phi_bb222_30, phi_bb222_31, tmp369, tmp370, phi_bb222_36, phi_bb222_37, phi_bb222_38, phi_bb222_41, phi_bb222_42, tmp367, tmp368);
  }

  TNode<JSAny> phi_bb223_17;
  TNode<IntPtrT> phi_bb223_18;
  TNode<BoolT> phi_bb223_19;
  TNode<IntPtrT> phi_bb223_28;
  TNode<IntPtrT> phi_bb223_29;
  TNode<IntPtrT> phi_bb223_30;
  TNode<IntPtrT> phi_bb223_31;
  TNode<IntPtrT> phi_bb223_34;
  TNode<BoolT> phi_bb223_35;
  TNode<BoolT> phi_bb223_36;
  TNode<Union<FixedArray, Smi>> phi_bb223_37;
  TNode<IntPtrT> phi_bb223_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb223_41;
  TNode<IntPtrT> phi_bb223_42;
  TNode<Union<HeapObject, TaggedIndex>> tmp371;
  TNode<IntPtrT> tmp372;
  TNode<IntPtrT> tmp373;
  TNode<IntPtrT> tmp374;
  TNode<IntPtrT> tmp375;
  TNode<IntPtrT> tmp376;
  TNode<BoolT> tmp377;
  if (block223.is_used()) {
    ca_.Bind(&block223, &phi_bb223_17, &phi_bb223_18, &phi_bb223_19, &phi_bb223_28, &phi_bb223_29, &phi_bb223_30, &phi_bb223_31, &phi_bb223_34, &phi_bb223_35, &phi_bb223_36, &phi_bb223_37, &phi_bb223_38, &phi_bb223_41, &phi_bb223_42);
    std::tie(tmp371, tmp372) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb223_31}).Flatten();
    tmp373 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp374 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb223_31}, TNode<IntPtrT>{tmp373});
    tmp375 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp376 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp374}, TNode<IntPtrT>{tmp375});
    tmp377 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block218, phi_bb223_17, phi_bb223_18, phi_bb223_19, phi_bb223_28, phi_bb223_29, phi_bb223_30, tmp376, tmp374, tmp377, phi_bb223_36, phi_bb223_37, phi_bb223_38, phi_bb223_41, phi_bb223_42, tmp371, tmp372);
  }

  TNode<JSAny> phi_bb218_17;
  TNode<IntPtrT> phi_bb218_18;
  TNode<BoolT> phi_bb218_19;
  TNode<IntPtrT> phi_bb218_28;
  TNode<IntPtrT> phi_bb218_29;
  TNode<IntPtrT> phi_bb218_30;
  TNode<IntPtrT> phi_bb218_31;
  TNode<IntPtrT> phi_bb218_34;
  TNode<BoolT> phi_bb218_35;
  TNode<BoolT> phi_bb218_36;
  TNode<Union<FixedArray, Smi>> phi_bb218_37;
  TNode<IntPtrT> phi_bb218_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb218_41;
  TNode<IntPtrT> phi_bb218_42;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb218_43;
  TNode<IntPtrT> phi_bb218_44;
  if (block218.is_used()) {
    ca_.Bind(&block218, &phi_bb218_17, &phi_bb218_18, &phi_bb218_19, &phi_bb218_28, &phi_bb218_29, &phi_bb218_30, &phi_bb218_31, &phi_bb218_34, &phi_bb218_35, &phi_bb218_36, &phi_bb218_37, &phi_bb218_38, &phi_bb218_41, &phi_bb218_42, &phi_bb218_43, &phi_bb218_44);
    ca_.Goto(&block215, phi_bb218_17, phi_bb218_18, phi_bb218_19, phi_bb218_28, phi_bb218_29, phi_bb218_30, phi_bb218_31, phi_bb218_34, phi_bb218_35, phi_bb218_36, phi_bb218_37, phi_bb218_38, phi_bb218_41, phi_bb218_42, phi_bb218_43, phi_bb218_44);
  }

  TNode<JSAny> phi_bb215_17;
  TNode<IntPtrT> phi_bb215_18;
  TNode<BoolT> phi_bb215_19;
  TNode<IntPtrT> phi_bb215_28;
  TNode<IntPtrT> phi_bb215_29;
  TNode<IntPtrT> phi_bb215_30;
  TNode<IntPtrT> phi_bb215_31;
  TNode<IntPtrT> phi_bb215_34;
  TNode<BoolT> phi_bb215_35;
  TNode<BoolT> phi_bb215_36;
  TNode<Union<FixedArray, Smi>> phi_bb215_37;
  TNode<IntPtrT> phi_bb215_38;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb215_41;
  TNode<IntPtrT> phi_bb215_42;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb215_43;
  TNode<IntPtrT> phi_bb215_44;
  TNode<BigInt> tmp378;
  TNode<UintPtrT> tmp379;
  TNode<UintPtrT> tmp380;
  TNode<IntPtrT> tmp381;
  TNode<IntPtrT> tmp382;
  if (block215.is_used()) {
    ca_.Bind(&block215, &phi_bb215_17, &phi_bb215_18, &phi_bb215_19, &phi_bb215_28, &phi_bb215_29, &phi_bb215_30, &phi_bb215_31, &phi_bb215_34, &phi_bb215_35, &phi_bb215_36, &phi_bb215_37, &phi_bb215_38, &phi_bb215_41, &phi_bb215_42, &phi_bb215_43, &phi_bb215_44);
    tmp378 = CodeStubAssembler(state_).ToBigInt(TNode<Context>{p_context}, TNode<JSAny>{tmp195});
    std::tie(tmp379, tmp380) = CodeStubAssembler(state_).BigIntToRawBytes(TNode<BigInt>{tmp378}).Flatten();
    tmp381 = CodeStubAssembler(state_).Signed(TNode<UintPtrT>{tmp379});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb215_41, phi_bb215_42}, tmp381);
    tmp382 = CodeStubAssembler(state_).Signed(TNode<UintPtrT>{tmp380});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb215_43, phi_bb215_44}, tmp382);
    ca_.Goto(&block196, phi_bb215_17, phi_bb215_18, phi_bb215_19, tmp354, phi_bb215_28, phi_bb215_29, phi_bb215_30, phi_bb215_31, phi_bb215_34, phi_bb215_35, phi_bb215_36, phi_bb215_37, phi_bb215_38);
  }

  TNode<JSAny> phi_bb196_17;
  TNode<IntPtrT> phi_bb196_18;
  TNode<BoolT> phi_bb196_19;
  TNode<IntPtrT> phi_bb196_27;
  TNode<IntPtrT> phi_bb196_28;
  TNode<IntPtrT> phi_bb196_29;
  TNode<IntPtrT> phi_bb196_30;
  TNode<IntPtrT> phi_bb196_31;
  TNode<IntPtrT> phi_bb196_34;
  TNode<BoolT> phi_bb196_35;
  TNode<BoolT> phi_bb196_36;
  TNode<Union<FixedArray, Smi>> phi_bb196_37;
  TNode<IntPtrT> phi_bb196_38;
  if (block196.is_used()) {
    ca_.Bind(&block196, &phi_bb196_17, &phi_bb196_18, &phi_bb196_19, &phi_bb196_27, &phi_bb196_28, &phi_bb196_29, &phi_bb196_30, &phi_bb196_31, &phi_bb196_34, &phi_bb196_35, &phi_bb196_36, &phi_bb196_37, &phi_bb196_38);
    ca_.Goto(&block193, phi_bb196_17, phi_bb196_18, phi_bb196_19, phi_bb196_27, phi_bb196_28, phi_bb196_29, phi_bb196_30, phi_bb196_31, phi_bb196_34, phi_bb196_35, phi_bb196_36, phi_bb196_37, phi_bb196_38);
  }

  TNode<JSAny> phi_bb192_17;
  TNode<IntPtrT> phi_bb192_18;
  TNode<BoolT> phi_bb192_19;
  TNode<IntPtrT> phi_bb192_27;
  TNode<IntPtrT> phi_bb192_28;
  TNode<IntPtrT> phi_bb192_29;
  TNode<IntPtrT> phi_bb192_30;
  TNode<IntPtrT> phi_bb192_31;
  TNode<IntPtrT> phi_bb192_34;
  TNode<BoolT> phi_bb192_35;
  TNode<BoolT> phi_bb192_36;
  TNode<Union<FixedArray, Smi>> phi_bb192_37;
  TNode<IntPtrT> phi_bb192_38;
  TNode<Uint32T> tmp383;
  TNode<Uint32T> tmp384;
  TNode<Uint32T> tmp385;
  TNode<BoolT> tmp386;
  if (block192.is_used()) {
    ca_.Bind(&block192, &phi_bb192_17, &phi_bb192_18, &phi_bb192_19, &phi_bb192_27, &phi_bb192_28, &phi_bb192_29, &phi_bb192_30, &phi_bb192_31, &phi_bb192_34, &phi_bb192_35, &phi_bb192_36, &phi_bb192_37, &phi_bb192_38);
    tmp383 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp384 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp200}, TNode<Uint32T>{tmp383});
    tmp385 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp386 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp384}, TNode<Uint32T>{tmp385});
    ca_.Branch(tmp386, &block224, std::vector<compiler::Node*>{phi_bb192_17, phi_bb192_18, phi_bb192_19, phi_bb192_27, phi_bb192_28, phi_bb192_29, phi_bb192_30, phi_bb192_31, phi_bb192_34, phi_bb192_35, phi_bb192_36, phi_bb192_37, phi_bb192_38}, &block225, std::vector<compiler::Node*>{phi_bb192_17, phi_bb192_18, phi_bb192_19, phi_bb192_27, phi_bb192_28, phi_bb192_29, phi_bb192_30, phi_bb192_31, phi_bb192_34, phi_bb192_35, phi_bb192_36, phi_bb192_37, phi_bb192_38});
  }

  TNode<JSAny> phi_bb225_17;
  TNode<IntPtrT> phi_bb225_18;
  TNode<BoolT> phi_bb225_19;
  TNode<IntPtrT> phi_bb225_27;
  TNode<IntPtrT> phi_bb225_28;
  TNode<IntPtrT> phi_bb225_29;
  TNode<IntPtrT> phi_bb225_30;
  TNode<IntPtrT> phi_bb225_31;
  TNode<IntPtrT> phi_bb225_34;
  TNode<BoolT> phi_bb225_35;
  TNode<BoolT> phi_bb225_36;
  TNode<Union<FixedArray, Smi>> phi_bb225_37;
  TNode<IntPtrT> phi_bb225_38;
  if (block225.is_used()) {
    ca_.Bind(&block225, &phi_bb225_17, &phi_bb225_18, &phi_bb225_19, &phi_bb225_27, &phi_bb225_28, &phi_bb225_29, &phi_bb225_30, &phi_bb225_31, &phi_bb225_34, &phi_bb225_35, &phi_bb225_36, &phi_bb225_37, &phi_bb225_38);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/js-to-wasm.tq", 740});
      CodeStubAssembler(state_).FailAssert("Torque assert '(paramType & kValueTypeIsRefBit) != 0' failed", pos_stack);
    }
  }

  TNode<JSAny> phi_bb224_17;
  TNode<IntPtrT> phi_bb224_18;
  TNode<BoolT> phi_bb224_19;
  TNode<IntPtrT> phi_bb224_27;
  TNode<IntPtrT> phi_bb224_28;
  TNode<IntPtrT> phi_bb224_29;
  TNode<IntPtrT> phi_bb224_30;
  TNode<IntPtrT> phi_bb224_31;
  TNode<IntPtrT> phi_bb224_34;
  TNode<BoolT> phi_bb224_35;
  TNode<BoolT> phi_bb224_36;
  TNode<Union<FixedArray, Smi>> phi_bb224_37;
  TNode<IntPtrT> phi_bb224_38;
  TNode<BoolT> tmp387;
  TNode<BoolT> tmp388;
  if (block224.is_used()) {
    ca_.Bind(&block224, &phi_bb224_17, &phi_bb224_18, &phi_bb224_19, &phi_bb224_27, &phi_bb224_28, &phi_bb224_29, &phi_bb224_30, &phi_bb224_31, &phi_bb224_34, &phi_bb224_35, &phi_bb224_36, &phi_bb224_37, &phi_bb224_38);
    tmp387 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp388 = CodeStubAssembler(state_).TaggedIsSmi(TNode<Object>{phi_bb224_37});
    ca_.Branch(tmp388, &block226, std::vector<compiler::Node*>{phi_bb224_17, phi_bb224_18, phi_bb224_19, phi_bb224_27, phi_bb224_28, phi_bb224_29, phi_bb224_30, phi_bb224_31, phi_bb224_34, phi_bb224_35, phi_bb224_37, phi_bb224_38}, &block227, std::vector<compiler::Node*>{phi_bb224_17, phi_bb224_18, phi_bb224_19, phi_bb224_27, phi_bb224_28, phi_bb224_29, phi_bb224_30, phi_bb224_31, phi_bb224_34, phi_bb224_35, phi_bb224_37, phi_bb224_38});
  }

  TNode<JSAny> phi_bb226_17;
  TNode<IntPtrT> phi_bb226_18;
  TNode<BoolT> phi_bb226_19;
  TNode<IntPtrT> phi_bb226_27;
  TNode<IntPtrT> phi_bb226_28;
  TNode<IntPtrT> phi_bb226_29;
  TNode<IntPtrT> phi_bb226_30;
  TNode<IntPtrT> phi_bb226_31;
  TNode<IntPtrT> phi_bb226_34;
  TNode<BoolT> phi_bb226_35;
  TNode<Union<FixedArray, Smi>> phi_bb226_37;
  TNode<IntPtrT> phi_bb226_38;
  TNode<FixedArray> tmp389;
  if (block226.is_used()) {
    ca_.Bind(&block226, &phi_bb226_17, &phi_bb226_18, &phi_bb226_19, &phi_bb226_27, &phi_bb226_28, &phi_bb226_29, &phi_bb226_30, &phi_bb226_31, &phi_bb226_34, &phi_bb226_35, &phi_bb226_37, &phi_bb226_38);
    tmp389 = ca_.CallBuiltin<FixedArray>(Builtin::kWasmAllocateZeroedFixedArray, TNode<Object>(), tmp22);
    ca_.Goto(&block227, phi_bb226_17, phi_bb226_18, phi_bb226_19, phi_bb226_27, phi_bb226_28, phi_bb226_29, phi_bb226_30, phi_bb226_31, phi_bb226_34, phi_bb226_35, tmp389, phi_bb226_38);
  }

  TNode<JSAny> phi_bb227_17;
  TNode<IntPtrT> phi_bb227_18;
  TNode<BoolT> phi_bb227_19;
  TNode<IntPtrT> phi_bb227_27;
  TNode<IntPtrT> phi_bb227_28;
  TNode<IntPtrT> phi_bb227_29;
  TNode<IntPtrT> phi_bb227_30;
  TNode<IntPtrT> phi_bb227_31;
  TNode<IntPtrT> phi_bb227_34;
  TNode<BoolT> phi_bb227_35;
  TNode<Union<FixedArray, Smi>> phi_bb227_37;
  TNode<IntPtrT> phi_bb227_38;
  TNode<FixedArray> tmp390;
  TNode<Union<HeapObject, TaggedIndex>> tmp391;
  TNode<IntPtrT> tmp392;
  TNode<IntPtrT> tmp393;
  TNode<UintPtrT> tmp394;
  TNode<UintPtrT> tmp395;
  TNode<BoolT> tmp396;
  if (block227.is_used()) {
    ca_.Bind(&block227, &phi_bb227_17, &phi_bb227_18, &phi_bb227_19, &phi_bb227_27, &phi_bb227_28, &phi_bb227_29, &phi_bb227_30, &phi_bb227_31, &phi_bb227_34, &phi_bb227_35, &phi_bb227_37, &phi_bb227_38);
    tmp390 = UnsafeCast_FixedArray_0(state_, TNode<Context>{p_context}, TNode<Object>{phi_bb227_37});
    std::tie(tmp391, tmp392, tmp393) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp390}).Flatten();
    tmp394 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb227_38});
    tmp395 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp393});
    tmp396 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp394}, TNode<UintPtrT>{tmp395});
    ca_.Branch(tmp396, &block232, std::vector<compiler::Node*>{phi_bb227_17, phi_bb227_18, phi_bb227_19, phi_bb227_27, phi_bb227_28, phi_bb227_29, phi_bb227_30, phi_bb227_31, phi_bb227_34, phi_bb227_35, phi_bb227_38, phi_bb227_38, phi_bb227_38, phi_bb227_38, phi_bb227_38}, &block233, std::vector<compiler::Node*>{phi_bb227_17, phi_bb227_18, phi_bb227_19, phi_bb227_27, phi_bb227_28, phi_bb227_29, phi_bb227_30, phi_bb227_31, phi_bb227_34, phi_bb227_35, phi_bb227_38, phi_bb227_38, phi_bb227_38, phi_bb227_38, phi_bb227_38});
  }

  TNode<JSAny> phi_bb232_17;
  TNode<IntPtrT> phi_bb232_18;
  TNode<BoolT> phi_bb232_19;
  TNode<IntPtrT> phi_bb232_27;
  TNode<IntPtrT> phi_bb232_28;
  TNode<IntPtrT> phi_bb232_29;
  TNode<IntPtrT> phi_bb232_30;
  TNode<IntPtrT> phi_bb232_31;
  TNode<IntPtrT> phi_bb232_34;
  TNode<BoolT> phi_bb232_35;
  TNode<IntPtrT> phi_bb232_38;
  TNode<IntPtrT> phi_bb232_46;
  TNode<IntPtrT> phi_bb232_47;
  TNode<IntPtrT> phi_bb232_51;
  TNode<IntPtrT> phi_bb232_52;
  TNode<IntPtrT> tmp397;
  TNode<IntPtrT> tmp398;
  TNode<Union<HeapObject, TaggedIndex>> tmp399;
  TNode<IntPtrT> tmp400;
  TNode<Object> tmp401;
  if (block232.is_used()) {
    ca_.Bind(&block232, &phi_bb232_17, &phi_bb232_18, &phi_bb232_19, &phi_bb232_27, &phi_bb232_28, &phi_bb232_29, &phi_bb232_30, &phi_bb232_31, &phi_bb232_34, &phi_bb232_35, &phi_bb232_38, &phi_bb232_46, &phi_bb232_47, &phi_bb232_51, &phi_bb232_52);
    tmp397 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb232_52});
    tmp398 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp392}, TNode<IntPtrT>{tmp397});
    std::tie(tmp399, tmp400) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp391}, TNode<IntPtrT>{tmp398}).Flatten();
    tmp401 = JSToWasmObject_0(state_, TNode<NativeContext>{p_context}, TNode<Uint32T>{tmp200}, TNode<JSAny>{tmp195});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp399, tmp400}, tmp401);
    ca_.Goto(&block193, phi_bb232_17, phi_bb232_18, phi_bb232_19, phi_bb232_27, phi_bb232_28, phi_bb232_29, phi_bb232_30, phi_bb232_31, phi_bb232_34, phi_bb232_35, tmp387, phi_bb227_37, phi_bb232_38);
  }

  TNode<JSAny> phi_bb233_17;
  TNode<IntPtrT> phi_bb233_18;
  TNode<BoolT> phi_bb233_19;
  TNode<IntPtrT> phi_bb233_27;
  TNode<IntPtrT> phi_bb233_28;
  TNode<IntPtrT> phi_bb233_29;
  TNode<IntPtrT> phi_bb233_30;
  TNode<IntPtrT> phi_bb233_31;
  TNode<IntPtrT> phi_bb233_34;
  TNode<BoolT> phi_bb233_35;
  TNode<IntPtrT> phi_bb233_38;
  TNode<IntPtrT> phi_bb233_46;
  TNode<IntPtrT> phi_bb233_47;
  TNode<IntPtrT> phi_bb233_51;
  TNode<IntPtrT> phi_bb233_52;
  if (block233.is_used()) {
    ca_.Bind(&block233, &phi_bb233_17, &phi_bb233_18, &phi_bb233_19, &phi_bb233_27, &phi_bb233_28, &phi_bb233_29, &phi_bb233_30, &phi_bb233_31, &phi_bb233_34, &phi_bb233_35, &phi_bb233_38, &phi_bb233_46, &phi_bb233_47, &phi_bb233_51, &phi_bb233_52);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb193_17;
  TNode<IntPtrT> phi_bb193_18;
  TNode<BoolT> phi_bb193_19;
  TNode<IntPtrT> phi_bb193_27;
  TNode<IntPtrT> phi_bb193_28;
  TNode<IntPtrT> phi_bb193_29;
  TNode<IntPtrT> phi_bb193_30;
  TNode<IntPtrT> phi_bb193_31;
  TNode<IntPtrT> phi_bb193_34;
  TNode<BoolT> phi_bb193_35;
  TNode<BoolT> phi_bb193_36;
  TNode<Union<FixedArray, Smi>> phi_bb193_37;
  TNode<IntPtrT> phi_bb193_38;
  if (block193.is_used()) {
    ca_.Bind(&block193, &phi_bb193_17, &phi_bb193_18, &phi_bb193_19, &phi_bb193_27, &phi_bb193_28, &phi_bb193_29, &phi_bb193_30, &phi_bb193_31, &phi_bb193_34, &phi_bb193_35, &phi_bb193_36, &phi_bb193_37, &phi_bb193_38);
    ca_.Goto(&block178, phi_bb193_17, phi_bb193_18, phi_bb193_19, phi_bb193_27, phi_bb193_28, phi_bb193_29, phi_bb193_30, phi_bb193_31, phi_bb193_34, phi_bb193_35, phi_bb193_36, phi_bb193_37, phi_bb193_38);
  }

  TNode<JSAny> phi_bb178_17;
  TNode<IntPtrT> phi_bb178_18;
  TNode<BoolT> phi_bb178_19;
  TNode<IntPtrT> phi_bb178_27;
  TNode<IntPtrT> phi_bb178_28;
  TNode<IntPtrT> phi_bb178_29;
  TNode<IntPtrT> phi_bb178_30;
  TNode<IntPtrT> phi_bb178_31;
  TNode<IntPtrT> phi_bb178_34;
  TNode<BoolT> phi_bb178_35;
  TNode<BoolT> phi_bb178_36;
  TNode<Union<FixedArray, Smi>> phi_bb178_37;
  TNode<IntPtrT> phi_bb178_38;
  if (block178.is_used()) {
    ca_.Bind(&block178, &phi_bb178_17, &phi_bb178_18, &phi_bb178_19, &phi_bb178_27, &phi_bb178_28, &phi_bb178_29, &phi_bb178_30, &phi_bb178_31, &phi_bb178_34, &phi_bb178_35, &phi_bb178_36, &phi_bb178_37, &phi_bb178_38);
    ca_.Goto(&block163, phi_bb178_17, phi_bb178_18, phi_bb178_19, phi_bb178_27, phi_bb178_28, phi_bb178_29, phi_bb178_30, phi_bb178_31, phi_bb178_34, phi_bb178_35, phi_bb178_36, phi_bb178_37, phi_bb178_38);
  }

  TNode<JSAny> phi_bb163_17;
  TNode<IntPtrT> phi_bb163_18;
  TNode<BoolT> phi_bb163_19;
  TNode<IntPtrT> phi_bb163_27;
  TNode<IntPtrT> phi_bb163_28;
  TNode<IntPtrT> phi_bb163_29;
  TNode<IntPtrT> phi_bb163_30;
  TNode<IntPtrT> phi_bb163_31;
  TNode<IntPtrT> phi_bb163_34;
  TNode<BoolT> phi_bb163_35;
  TNode<BoolT> phi_bb163_36;
  TNode<Union<FixedArray, Smi>> phi_bb163_37;
  TNode<IntPtrT> phi_bb163_38;
  if (block163.is_used()) {
    ca_.Bind(&block163, &phi_bb163_17, &phi_bb163_18, &phi_bb163_19, &phi_bb163_27, &phi_bb163_28, &phi_bb163_29, &phi_bb163_30, &phi_bb163_31, &phi_bb163_34, &phi_bb163_35, &phi_bb163_36, &phi_bb163_37, &phi_bb163_38);
    ca_.Goto(&block147, phi_bb163_17, phi_bb163_18, phi_bb163_19, phi_bb163_27, phi_bb163_28, phi_bb163_29, phi_bb163_30, phi_bb163_31, phi_bb163_34, phi_bb163_35, phi_bb163_36, phi_bb163_37, phi_bb163_38);
  }

  TNode<JSAny> phi_bb147_17;
  TNode<IntPtrT> phi_bb147_18;
  TNode<BoolT> phi_bb147_19;
  TNode<IntPtrT> phi_bb147_27;
  TNode<IntPtrT> phi_bb147_28;
  TNode<IntPtrT> phi_bb147_29;
  TNode<IntPtrT> phi_bb147_30;
  TNode<IntPtrT> phi_bb147_31;
  TNode<IntPtrT> phi_bb147_34;
  TNode<BoolT> phi_bb147_35;
  TNode<BoolT> phi_bb147_36;
  TNode<Union<FixedArray, Smi>> phi_bb147_37;
  TNode<IntPtrT> phi_bb147_38;
  TNode<IntPtrT> tmp402;
  TNode<IntPtrT> tmp403;
  if (block147.is_used()) {
    ca_.Bind(&block147, &phi_bb147_17, &phi_bb147_18, &phi_bb147_19, &phi_bb147_27, &phi_bb147_28, &phi_bb147_29, &phi_bb147_30, &phi_bb147_31, &phi_bb147_34, &phi_bb147_35, &phi_bb147_36, &phi_bb147_37, &phi_bb147_38);
    tmp402 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp403 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb147_38}, TNode<IntPtrT>{tmp402});
    ca_.Goto(&block142, phi_bb147_17, phi_bb147_18, phi_bb147_19, phi_bb147_27, phi_bb147_28, phi_bb147_29, phi_bb147_30, phi_bb147_31, phi_bb147_34, phi_bb147_35, phi_bb147_36, phi_bb147_37, tmp403);
  }

  TNode<JSAny> phi_bb141_17;
  TNode<IntPtrT> phi_bb141_18;
  TNode<BoolT> phi_bb141_19;
  TNode<IntPtrT> phi_bb141_27;
  TNode<IntPtrT> phi_bb141_28;
  TNode<IntPtrT> phi_bb141_29;
  TNode<IntPtrT> phi_bb141_30;
  TNode<IntPtrT> phi_bb141_31;
  TNode<IntPtrT> phi_bb141_34;
  TNode<BoolT> phi_bb141_35;
  TNode<BoolT> phi_bb141_36;
  TNode<Union<FixedArray, Smi>> phi_bb141_37;
  TNode<IntPtrT> phi_bb141_38;
  TNode<Undefined> tmp404;
  if (block141.is_used()) {
    ca_.Bind(&block141, &phi_bb141_17, &phi_bb141_18, &phi_bb141_19, &phi_bb141_27, &phi_bb141_28, &phi_bb141_29, &phi_bb141_30, &phi_bb141_31, &phi_bb141_34, &phi_bb141_35, &phi_bb141_36, &phi_bb141_37, &phi_bb141_38);
    tmp404 = Undefined_0(state_);
    if ((((CodeStubAssembler(state_).ConstexprInt32Equal(CastIfEnumClass<int32_t>(p_promise), CastIfEnumClass<int32_t>(wasm::Promise::kPromise))) || (CodeStubAssembler(state_).ConstexprInt32Equal(CastIfEnumClass<int32_t>(p_promise), CastIfEnumClass<int32_t>(wasm::Promise::kStressSwitch)))))) {
      ca_.Goto(&block236, phi_bb141_17, phi_bb141_18, phi_bb141_19, phi_bb141_27, phi_bb141_28, phi_bb141_29, phi_bb141_30, phi_bb141_31, phi_bb141_34, phi_bb141_35, phi_bb141_36, phi_bb141_37);
    } else {
      ca_.Goto(&block237, phi_bb141_17, phi_bb141_18, phi_bb141_19, phi_bb141_27, phi_bb141_28, phi_bb141_29, phi_bb141_30, phi_bb141_31, phi_bb141_34, phi_bb141_35, phi_bb141_36, phi_bb141_37);
    }
  }

  TNode<JSAny> phi_bb236_17;
  TNode<IntPtrT> phi_bb236_18;
  TNode<BoolT> phi_bb236_19;
  TNode<IntPtrT> phi_bb236_27;
  TNode<IntPtrT> phi_bb236_28;
  TNode<IntPtrT> phi_bb236_29;
  TNode<IntPtrT> phi_bb236_30;
  TNode<IntPtrT> phi_bb236_31;
  TNode<IntPtrT> phi_bb236_34;
  TNode<BoolT> phi_bb236_35;
  TNode<BoolT> phi_bb236_36;
  TNode<Union<FixedArray, Smi>> phi_bb236_37;
  TNode<JSAny> tmp405;
  TNode<BoolT> tmp406;
  if (block236.is_used()) {
    ca_.Bind(&block236, &phi_bb236_17, &phi_bb236_18, &phi_bb236_19, &phi_bb236_27, &phi_bb236_28, &phi_bb236_29, &phi_bb236_30, &phi_bb236_31, &phi_bb236_34, &phi_bb236_35, &phi_bb236_36, &phi_bb236_37);
    tmp405 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kWasmAllocateSuspender, p_context)); 
    tmp406 = FromConstexpr_bool_constexpr_bool_0(state_, (CodeStubAssembler(state_).ConstexprInt32Equal(CastIfEnumClass<int32_t>(p_promise), CastIfEnumClass<int32_t>(wasm::Promise::kStressSwitch))));
    ca_.Branch(tmp406, &block239, std::vector<compiler::Node*>{phi_bb236_17, phi_bb236_18, phi_bb236_19, phi_bb236_27, phi_bb236_28, phi_bb236_29, phi_bb236_30, phi_bb236_31, phi_bb236_34, phi_bb236_35, phi_bb236_36, phi_bb236_37}, &block240, std::vector<compiler::Node*>{phi_bb236_17, phi_bb236_18, phi_bb236_19, phi_bb236_27, phi_bb236_28, phi_bb236_29, phi_bb236_30, phi_bb236_31, phi_bb236_34, phi_bb236_35, phi_bb236_36, phi_bb236_37});
  }

  TNode<JSAny> phi_bb239_17;
  TNode<IntPtrT> phi_bb239_18;
  TNode<BoolT> phi_bb239_19;
  TNode<IntPtrT> phi_bb239_27;
  TNode<IntPtrT> phi_bb239_28;
  TNode<IntPtrT> phi_bb239_29;
  TNode<IntPtrT> phi_bb239_30;
  TNode<IntPtrT> phi_bb239_31;
  TNode<IntPtrT> phi_bb239_34;
  TNode<BoolT> phi_bb239_35;
  TNode<BoolT> phi_bb239_36;
  TNode<Union<FixedArray, Smi>> phi_bb239_37;
  TNode<JSAny> tmp407;
  if (block239.is_used()) {
    ca_.Bind(&block239, &phi_bb239_17, &phi_bb239_18, &phi_bb239_19, &phi_bb239_27, &phi_bb239_28, &phi_bb239_29, &phi_bb239_30, &phi_bb239_31, &phi_bb239_34, &phi_bb239_35, &phi_bb239_36, &phi_bb239_37);
    tmp407 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kClearWasmSuspenderResumeField, p_context, tmp405)); 
    ca_.Goto(&block240, phi_bb239_17, phi_bb239_18, phi_bb239_19, phi_bb239_27, phi_bb239_28, phi_bb239_29, phi_bb239_30, phi_bb239_31, phi_bb239_34, phi_bb239_35, phi_bb239_36, phi_bb239_37);
  }

  TNode<JSAny> phi_bb240_17;
  TNode<IntPtrT> phi_bb240_18;
  TNode<BoolT> phi_bb240_19;
  TNode<IntPtrT> phi_bb240_27;
  TNode<IntPtrT> phi_bb240_28;
  TNode<IntPtrT> phi_bb240_29;
  TNode<IntPtrT> phi_bb240_30;
  TNode<IntPtrT> phi_bb240_31;
  TNode<IntPtrT> phi_bb240_34;
  TNode<BoolT> phi_bb240_35;
  TNode<BoolT> phi_bb240_36;
  TNode<Union<FixedArray, Smi>> phi_bb240_37;
  if (block240.is_used()) {
    ca_.Bind(&block240, &phi_bb240_17, &phi_bb240_18, &phi_bb240_19, &phi_bb240_27, &phi_bb240_28, &phi_bb240_29, &phi_bb240_30, &phi_bb240_31, &phi_bb240_34, &phi_bb240_35, &phi_bb240_36, &phi_bb240_37);
    ca_.Goto(&block238, phi_bb240_17, phi_bb240_18, phi_bb240_19, phi_bb240_27, phi_bb240_28, phi_bb240_29, phi_bb240_30, phi_bb240_31, phi_bb240_34, phi_bb240_35, phi_bb240_36, phi_bb240_37, tmp405);
  }

  TNode<JSAny> phi_bb237_17;
  TNode<IntPtrT> phi_bb237_18;
  TNode<BoolT> phi_bb237_19;
  TNode<IntPtrT> phi_bb237_27;
  TNode<IntPtrT> phi_bb237_28;
  TNode<IntPtrT> phi_bb237_29;
  TNode<IntPtrT> phi_bb237_30;
  TNode<IntPtrT> phi_bb237_31;
  TNode<IntPtrT> phi_bb237_34;
  TNode<BoolT> phi_bb237_35;
  TNode<BoolT> phi_bb237_36;
  TNode<Union<FixedArray, Smi>> phi_bb237_37;
  if (block237.is_used()) {
    ca_.Bind(&block237, &phi_bb237_17, &phi_bb237_18, &phi_bb237_19, &phi_bb237_27, &phi_bb237_28, &phi_bb237_29, &phi_bb237_30, &phi_bb237_31, &phi_bb237_34, &phi_bb237_35, &phi_bb237_36, &phi_bb237_37);
    ca_.Goto(&block238, phi_bb237_17, phi_bb237_18, phi_bb237_19, phi_bb237_27, phi_bb237_28, phi_bb237_29, phi_bb237_30, phi_bb237_31, phi_bb237_34, phi_bb237_35, phi_bb237_36, phi_bb237_37, tmp404);
  }

  TNode<JSAny> phi_bb238_17;
  TNode<IntPtrT> phi_bb238_18;
  TNode<BoolT> phi_bb238_19;
  TNode<IntPtrT> phi_bb238_27;
  TNode<IntPtrT> phi_bb238_28;
  TNode<IntPtrT> phi_bb238_29;
  TNode<IntPtrT> phi_bb238_30;
  TNode<IntPtrT> phi_bb238_31;
  TNode<IntPtrT> phi_bb238_34;
  TNode<BoolT> phi_bb238_35;
  TNode<BoolT> phi_bb238_36;
  TNode<Union<FixedArray, Smi>> phi_bb238_37;
  TNode<JSAny> phi_bb238_38;
  if (block238.is_used()) {
    ca_.Bind(&block238, &phi_bb238_17, &phi_bb238_18, &phi_bb238_19, &phi_bb238_27, &phi_bb238_28, &phi_bb238_29, &phi_bb238_30, &phi_bb238_31, &phi_bb238_34, &phi_bb238_35, &phi_bb238_36, &phi_bb238_37, &phi_bb238_38);
    ca_.Branch(phi_bb238_36, &block241, std::vector<compiler::Node*>{phi_bb238_17, phi_bb238_18, phi_bb238_19, phi_bb238_27, phi_bb238_28, phi_bb238_29, phi_bb238_30, phi_bb238_31, phi_bb238_34, phi_bb238_35, phi_bb238_36, phi_bb238_37}, &block242, std::vector<compiler::Node*>{phi_bb238_17, phi_bb238_18, phi_bb238_19, phi_bb238_27, phi_bb238_28, phi_bb238_29, phi_bb238_30, phi_bb238_31, phi_bb238_34, phi_bb238_35, phi_bb238_36, phi_bb238_37});
  }

  TNode<JSAny> phi_bb241_17;
  TNode<IntPtrT> phi_bb241_18;
  TNode<BoolT> phi_bb241_19;
  TNode<IntPtrT> phi_bb241_27;
  TNode<IntPtrT> phi_bb241_28;
  TNode<IntPtrT> phi_bb241_29;
  TNode<IntPtrT> phi_bb241_30;
  TNode<IntPtrT> phi_bb241_31;
  TNode<IntPtrT> phi_bb241_34;
  TNode<BoolT> phi_bb241_35;
  TNode<BoolT> phi_bb241_36;
  TNode<Union<FixedArray, Smi>> phi_bb241_37;
  TNode<BoolT> tmp408;
  if (block241.is_used()) {
    ca_.Bind(&block241, &phi_bb241_17, &phi_bb241_18, &phi_bb241_19, &phi_bb241_27, &phi_bb241_28, &phi_bb241_29, &phi_bb241_30, &phi_bb241_31, &phi_bb241_34, &phi_bb241_35, &phi_bb241_36, &phi_bb241_37);
    tmp408 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{phi_bb241_35});
    ca_.Branch(tmp408, &block244, std::vector<compiler::Node*>{phi_bb241_17, phi_bb241_18, phi_bb241_19, phi_bb241_27, phi_bb241_28, phi_bb241_29, phi_bb241_30, phi_bb241_31, phi_bb241_34, phi_bb241_35, phi_bb241_36, phi_bb241_37}, &block245, std::vector<compiler::Node*>{phi_bb241_17, phi_bb241_18, phi_bb241_19, phi_bb241_27, phi_bb241_28, phi_bb241_29, phi_bb241_30, phi_bb241_31, phi_bb241_34, phi_bb241_35, phi_bb241_36, phi_bb241_37});
  }

  TNode<JSAny> phi_bb244_17;
  TNode<IntPtrT> phi_bb244_18;
  TNode<BoolT> phi_bb244_19;
  TNode<IntPtrT> phi_bb244_27;
  TNode<IntPtrT> phi_bb244_28;
  TNode<IntPtrT> phi_bb244_29;
  TNode<IntPtrT> phi_bb244_30;
  TNode<IntPtrT> phi_bb244_31;
  TNode<IntPtrT> phi_bb244_34;
  TNode<BoolT> phi_bb244_35;
  TNode<BoolT> phi_bb244_36;
  TNode<Union<FixedArray, Smi>> phi_bb244_37;
  TNode<IntPtrT> tmp409;
  if (block244.is_used()) {
    ca_.Bind(&block244, &phi_bb244_17, &phi_bb244_18, &phi_bb244_19, &phi_bb244_27, &phi_bb244_28, &phi_bb244_29, &phi_bb244_30, &phi_bb244_31, &phi_bb244_34, &phi_bb244_35, &phi_bb244_36, &phi_bb244_37);
    tmp409 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block245, phi_bb244_17, phi_bb244_18, phi_bb244_19, phi_bb244_27, phi_bb244_28, phi_bb244_29, phi_bb244_30, phi_bb244_31, tmp409, phi_bb244_35, phi_bb244_36, phi_bb244_37);
  }

  TNode<JSAny> phi_bb245_17;
  TNode<IntPtrT> phi_bb245_18;
  TNode<BoolT> phi_bb245_19;
  TNode<IntPtrT> phi_bb245_27;
  TNode<IntPtrT> phi_bb245_28;
  TNode<IntPtrT> phi_bb245_29;
  TNode<IntPtrT> phi_bb245_30;
  TNode<IntPtrT> phi_bb245_31;
  TNode<IntPtrT> phi_bb245_34;
  TNode<BoolT> phi_bb245_35;
  TNode<BoolT> phi_bb245_36;
  TNode<Union<FixedArray, Smi>> phi_bb245_37;
  TNode<IntPtrT> tmp410;
  if (block245.is_used()) {
    ca_.Bind(&block245, &phi_bb245_17, &phi_bb245_18, &phi_bb245_19, &phi_bb245_27, &phi_bb245_28, &phi_bb245_29, &phi_bb245_30, &phi_bb245_31, &phi_bb245_34, &phi_bb245_35, &phi_bb245_36, &phi_bb245_37);
    tmp410 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block248, phi_bb245_17, phi_bb245_18, phi_bb245_19, phi_bb245_27, phi_bb245_28, phi_bb245_29, phi_bb245_30, phi_bb245_31, phi_bb245_34, phi_bb245_35, phi_bb245_36, phi_bb245_37, tmp410);
  }

  TNode<JSAny> phi_bb248_17;
  TNode<IntPtrT> phi_bb248_18;
  TNode<BoolT> phi_bb248_19;
  TNode<IntPtrT> phi_bb248_27;
  TNode<IntPtrT> phi_bb248_28;
  TNode<IntPtrT> phi_bb248_29;
  TNode<IntPtrT> phi_bb248_30;
  TNode<IntPtrT> phi_bb248_31;
  TNode<IntPtrT> phi_bb248_34;
  TNode<BoolT> phi_bb248_35;
  TNode<BoolT> phi_bb248_36;
  TNode<Union<FixedArray, Smi>> phi_bb248_37;
  TNode<IntPtrT> phi_bb248_39;
  TNode<BoolT> tmp411;
  if (block248.is_used()) {
    ca_.Bind(&block248, &phi_bb248_17, &phi_bb248_18, &phi_bb248_19, &phi_bb248_27, &phi_bb248_28, &phi_bb248_29, &phi_bb248_30, &phi_bb248_31, &phi_bb248_34, &phi_bb248_35, &phi_bb248_36, &phi_bb248_37, &phi_bb248_39);
    tmp411 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb248_39}, TNode<IntPtrT>{tmp22});
    ca_.Branch(tmp411, &block246, std::vector<compiler::Node*>{phi_bb248_17, phi_bb248_18, phi_bb248_19, phi_bb248_27, phi_bb248_28, phi_bb248_29, phi_bb248_30, phi_bb248_31, phi_bb248_34, phi_bb248_35, phi_bb248_36, phi_bb248_37, phi_bb248_39}, &block247, std::vector<compiler::Node*>{phi_bb248_17, phi_bb248_18, phi_bb248_19, phi_bb248_27, phi_bb248_28, phi_bb248_29, phi_bb248_30, phi_bb248_31, phi_bb248_34, phi_bb248_35, phi_bb248_36, phi_bb248_37, phi_bb248_39});
  }

  TNode<JSAny> phi_bb246_17;
  TNode<IntPtrT> phi_bb246_18;
  TNode<BoolT> phi_bb246_19;
  TNode<IntPtrT> phi_bb246_27;
  TNode<IntPtrT> phi_bb246_28;
  TNode<IntPtrT> phi_bb246_29;
  TNode<IntPtrT> phi_bb246_30;
  TNode<IntPtrT> phi_bb246_31;
  TNode<IntPtrT> phi_bb246_34;
  TNode<BoolT> phi_bb246_35;
  TNode<BoolT> phi_bb246_36;
  TNode<Union<FixedArray, Smi>> phi_bb246_37;
  TNode<IntPtrT> phi_bb246_39;
  TNode<IntPtrT> tmp412;
  TNode<IntPtrT> tmp413;
  TNode<Union<HeapObject, TaggedIndex>> tmp414;
  TNode<IntPtrT> tmp415;
  TNode<Uint32T> tmp416;
  TNode<Uint32T> tmp417;
  TNode<Uint32T> tmp418;
  TNode<Uint32T> tmp419;
  TNode<BoolT> tmp420;
  if (block246.is_used()) {
    ca_.Bind(&block246, &phi_bb246_17, &phi_bb246_18, &phi_bb246_19, &phi_bb246_27, &phi_bb246_28, &phi_bb246_29, &phi_bb246_30, &phi_bb246_31, &phi_bb246_34, &phi_bb246_35, &phi_bb246_36, &phi_bb246_37, &phi_bb246_39);
    tmp412 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{phi_bb246_39});
    tmp413 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp162}, TNode<IntPtrT>{tmp412});
    std::tie(tmp414, tmp415) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp161}, TNode<IntPtrT>{tmp413}).Flatten();
    tmp416 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp414, tmp415});
    tmp417 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp418 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp416}, TNode<Uint32T>{tmp417});
    tmp419 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp420 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp418}, TNode<Uint32T>{tmp419});
    ca_.Branch(tmp420, &block251, std::vector<compiler::Node*>{phi_bb246_17, phi_bb246_18, phi_bb246_19, phi_bb246_27, phi_bb246_28, phi_bb246_29, phi_bb246_30, phi_bb246_31, phi_bb246_34, phi_bb246_35, phi_bb246_36, phi_bb246_37, phi_bb246_39}, &block252, std::vector<compiler::Node*>{phi_bb246_17, phi_bb246_18, phi_bb246_19, phi_bb246_27, phi_bb246_28, phi_bb246_29, phi_bb246_30, phi_bb246_31, phi_bb246_34, phi_bb246_35, phi_bb246_36, phi_bb246_37, phi_bb246_39});
  }

  TNode<JSAny> phi_bb251_17;
  TNode<IntPtrT> phi_bb251_18;
  TNode<BoolT> phi_bb251_19;
  TNode<IntPtrT> phi_bb251_27;
  TNode<IntPtrT> phi_bb251_28;
  TNode<IntPtrT> phi_bb251_29;
  TNode<IntPtrT> phi_bb251_30;
  TNode<IntPtrT> phi_bb251_31;
  TNode<IntPtrT> phi_bb251_34;
  TNode<BoolT> phi_bb251_35;
  TNode<BoolT> phi_bb251_36;
  TNode<Union<FixedArray, Smi>> phi_bb251_37;
  TNode<IntPtrT> phi_bb251_39;
  TNode<FixedArray> tmp421;
  TNode<Union<HeapObject, TaggedIndex>> tmp422;
  TNode<IntPtrT> tmp423;
  TNode<IntPtrT> tmp424;
  TNode<UintPtrT> tmp425;
  TNode<UintPtrT> tmp426;
  TNode<BoolT> tmp427;
  if (block251.is_used()) {
    ca_.Bind(&block251, &phi_bb251_17, &phi_bb251_18, &phi_bb251_19, &phi_bb251_27, &phi_bb251_28, &phi_bb251_29, &phi_bb251_30, &phi_bb251_31, &phi_bb251_34, &phi_bb251_35, &phi_bb251_36, &phi_bb251_37, &phi_bb251_39);
    tmp421 = UnsafeCast_FixedArray_0(state_, TNode<Context>{p_context}, TNode<Object>{phi_bb251_37});
    std::tie(tmp422, tmp423, tmp424) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp421}).Flatten();
    tmp425 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb251_39});
    tmp426 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp424});
    tmp427 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp425}, TNode<UintPtrT>{tmp426});
    ca_.Branch(tmp427, &block257, std::vector<compiler::Node*>{phi_bb251_17, phi_bb251_18, phi_bb251_19, phi_bb251_27, phi_bb251_28, phi_bb251_29, phi_bb251_30, phi_bb251_31, phi_bb251_34, phi_bb251_35, phi_bb251_36, phi_bb251_37, phi_bb251_39, phi_bb251_39, phi_bb251_39, phi_bb251_39, phi_bb251_39}, &block258, std::vector<compiler::Node*>{phi_bb251_17, phi_bb251_18, phi_bb251_19, phi_bb251_27, phi_bb251_28, phi_bb251_29, phi_bb251_30, phi_bb251_31, phi_bb251_34, phi_bb251_35, phi_bb251_36, phi_bb251_37, phi_bb251_39, phi_bb251_39, phi_bb251_39, phi_bb251_39, phi_bb251_39});
  }

  TNode<JSAny> phi_bb257_17;
  TNode<IntPtrT> phi_bb257_18;
  TNode<BoolT> phi_bb257_19;
  TNode<IntPtrT> phi_bb257_27;
  TNode<IntPtrT> phi_bb257_28;
  TNode<IntPtrT> phi_bb257_29;
  TNode<IntPtrT> phi_bb257_30;
  TNode<IntPtrT> phi_bb257_31;
  TNode<IntPtrT> phi_bb257_34;
  TNode<BoolT> phi_bb257_35;
  TNode<BoolT> phi_bb257_36;
  TNode<Union<FixedArray, Smi>> phi_bb257_37;
  TNode<IntPtrT> phi_bb257_39;
  TNode<IntPtrT> phi_bb257_46;
  TNode<IntPtrT> phi_bb257_47;
  TNode<IntPtrT> phi_bb257_51;
  TNode<IntPtrT> phi_bb257_52;
  TNode<IntPtrT> tmp428;
  TNode<IntPtrT> tmp429;
  TNode<Union<HeapObject, TaggedIndex>> tmp430;
  TNode<IntPtrT> tmp431;
  TNode<Object> tmp432;
  TNode<IntPtrT> tmp433;
  TNode<IntPtrT> tmp434;
  TNode<IntPtrT> tmp435;
  TNode<BoolT> tmp436;
  if (block257.is_used()) {
    ca_.Bind(&block257, &phi_bb257_17, &phi_bb257_18, &phi_bb257_19, &phi_bb257_27, &phi_bb257_28, &phi_bb257_29, &phi_bb257_30, &phi_bb257_31, &phi_bb257_34, &phi_bb257_35, &phi_bb257_36, &phi_bb257_37, &phi_bb257_39, &phi_bb257_46, &phi_bb257_47, &phi_bb257_51, &phi_bb257_52);
    tmp428 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb257_52});
    tmp429 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp423}, TNode<IntPtrT>{tmp428});
    std::tie(tmp430, tmp431) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp422}, TNode<IntPtrT>{tmp429}).Flatten();
    tmp432 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp430, tmp431});
    tmp433 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp434 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb257_27}, TNode<IntPtrT>{tmp433});
    tmp435 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp436 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb257_27}, TNode<IntPtrT>{tmp435});
    ca_.Branch(tmp436, &block262, std::vector<compiler::Node*>{phi_bb257_17, phi_bb257_18, phi_bb257_19, phi_bb257_28, phi_bb257_29, phi_bb257_30, phi_bb257_31, phi_bb257_34, phi_bb257_35, phi_bb257_36, phi_bb257_37, phi_bb257_39}, &block263, std::vector<compiler::Node*>{phi_bb257_17, phi_bb257_18, phi_bb257_19, phi_bb257_28, phi_bb257_29, phi_bb257_30, phi_bb257_31, phi_bb257_34, phi_bb257_35, phi_bb257_36, phi_bb257_37, phi_bb257_39});
  }

  TNode<JSAny> phi_bb258_17;
  TNode<IntPtrT> phi_bb258_18;
  TNode<BoolT> phi_bb258_19;
  TNode<IntPtrT> phi_bb258_27;
  TNode<IntPtrT> phi_bb258_28;
  TNode<IntPtrT> phi_bb258_29;
  TNode<IntPtrT> phi_bb258_30;
  TNode<IntPtrT> phi_bb258_31;
  TNode<IntPtrT> phi_bb258_34;
  TNode<BoolT> phi_bb258_35;
  TNode<BoolT> phi_bb258_36;
  TNode<Union<FixedArray, Smi>> phi_bb258_37;
  TNode<IntPtrT> phi_bb258_39;
  TNode<IntPtrT> phi_bb258_46;
  TNode<IntPtrT> phi_bb258_47;
  TNode<IntPtrT> phi_bb258_51;
  TNode<IntPtrT> phi_bb258_52;
  if (block258.is_used()) {
    ca_.Bind(&block258, &phi_bb258_17, &phi_bb258_18, &phi_bb258_19, &phi_bb258_27, &phi_bb258_28, &phi_bb258_29, &phi_bb258_30, &phi_bb258_31, &phi_bb258_34, &phi_bb258_35, &phi_bb258_36, &phi_bb258_37, &phi_bb258_39, &phi_bb258_46, &phi_bb258_47, &phi_bb258_51, &phi_bb258_52);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb262_17;
  TNode<IntPtrT> phi_bb262_18;
  TNode<BoolT> phi_bb262_19;
  TNode<IntPtrT> phi_bb262_28;
  TNode<IntPtrT> phi_bb262_29;
  TNode<IntPtrT> phi_bb262_30;
  TNode<IntPtrT> phi_bb262_31;
  TNode<IntPtrT> phi_bb262_34;
  TNode<BoolT> phi_bb262_35;
  TNode<BoolT> phi_bb262_36;
  TNode<Union<FixedArray, Smi>> phi_bb262_37;
  TNode<IntPtrT> phi_bb262_39;
  TNode<Union<HeapObject, TaggedIndex>> tmp437;
  TNode<IntPtrT> tmp438;
  TNode<IntPtrT> tmp439;
  TNode<IntPtrT> tmp440;
  if (block262.is_used()) {
    ca_.Bind(&block262, &phi_bb262_17, &phi_bb262_18, &phi_bb262_19, &phi_bb262_28, &phi_bb262_29, &phi_bb262_30, &phi_bb262_31, &phi_bb262_34, &phi_bb262_35, &phi_bb262_36, &phi_bb262_37, &phi_bb262_39);
    std::tie(tmp437, tmp438) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb262_29}).Flatten();
    tmp439 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp440 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb262_29}, TNode<IntPtrT>{tmp439});
    ca_.Goto(&block261, phi_bb262_17, phi_bb262_18, phi_bb262_19, phi_bb262_28, tmp440, phi_bb262_30, phi_bb262_31, phi_bb262_34, phi_bb262_35, phi_bb262_36, phi_bb262_37, phi_bb262_39, tmp437, tmp438);
  }

  TNode<JSAny> phi_bb263_17;
  TNode<IntPtrT> phi_bb263_18;
  TNode<BoolT> phi_bb263_19;
  TNode<IntPtrT> phi_bb263_28;
  TNode<IntPtrT> phi_bb263_29;
  TNode<IntPtrT> phi_bb263_30;
  TNode<IntPtrT> phi_bb263_31;
  TNode<IntPtrT> phi_bb263_34;
  TNode<BoolT> phi_bb263_35;
  TNode<BoolT> phi_bb263_36;
  TNode<Union<FixedArray, Smi>> phi_bb263_37;
  TNode<IntPtrT> phi_bb263_39;
  if (block263.is_used()) {
    ca_.Bind(&block263, &phi_bb263_17, &phi_bb263_18, &phi_bb263_19, &phi_bb263_28, &phi_bb263_29, &phi_bb263_30, &phi_bb263_31, &phi_bb263_34, &phi_bb263_35, &phi_bb263_36, &phi_bb263_37, &phi_bb263_39);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block265, phi_bb263_17, phi_bb263_18, phi_bb263_19, phi_bb263_28, phi_bb263_29, phi_bb263_30, phi_bb263_31, phi_bb263_34, phi_bb263_35, phi_bb263_36, phi_bb263_37, phi_bb263_39);
    } else {
      ca_.Goto(&block266, phi_bb263_17, phi_bb263_18, phi_bb263_19, phi_bb263_28, phi_bb263_29, phi_bb263_30, phi_bb263_31, phi_bb263_34, phi_bb263_35, phi_bb263_36, phi_bb263_37, phi_bb263_39);
    }
  }

  TNode<JSAny> phi_bb265_17;
  TNode<IntPtrT> phi_bb265_18;
  TNode<BoolT> phi_bb265_19;
  TNode<IntPtrT> phi_bb265_28;
  TNode<IntPtrT> phi_bb265_29;
  TNode<IntPtrT> phi_bb265_30;
  TNode<IntPtrT> phi_bb265_31;
  TNode<IntPtrT> phi_bb265_34;
  TNode<BoolT> phi_bb265_35;
  TNode<BoolT> phi_bb265_36;
  TNode<Union<FixedArray, Smi>> phi_bb265_37;
  TNode<IntPtrT> phi_bb265_39;
  TNode<Union<HeapObject, TaggedIndex>> tmp441;
  TNode<IntPtrT> tmp442;
  TNode<IntPtrT> tmp443;
  TNode<IntPtrT> tmp444;
  if (block265.is_used()) {
    ca_.Bind(&block265, &phi_bb265_17, &phi_bb265_18, &phi_bb265_19, &phi_bb265_28, &phi_bb265_29, &phi_bb265_30, &phi_bb265_31, &phi_bb265_34, &phi_bb265_35, &phi_bb265_36, &phi_bb265_37, &phi_bb265_39);
    std::tie(tmp441, tmp442) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb265_31}).Flatten();
    tmp443 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp444 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb265_31}, TNode<IntPtrT>{tmp443});
    ca_.Goto(&block264, phi_bb265_17, phi_bb265_18, phi_bb265_19, phi_bb265_28, phi_bb265_29, phi_bb265_30, tmp444, phi_bb265_34, phi_bb265_35, phi_bb265_36, phi_bb265_37, phi_bb265_39, tmp441, tmp442);
  }

  TNode<JSAny> phi_bb266_17;
  TNode<IntPtrT> phi_bb266_18;
  TNode<BoolT> phi_bb266_19;
  TNode<IntPtrT> phi_bb266_28;
  TNode<IntPtrT> phi_bb266_29;
  TNode<IntPtrT> phi_bb266_30;
  TNode<IntPtrT> phi_bb266_31;
  TNode<IntPtrT> phi_bb266_34;
  TNode<BoolT> phi_bb266_35;
  TNode<BoolT> phi_bb266_36;
  TNode<Union<FixedArray, Smi>> phi_bb266_37;
  TNode<IntPtrT> phi_bb266_39;
  TNode<IntPtrT> tmp445;
  TNode<BoolT> tmp446;
  if (block266.is_used()) {
    ca_.Bind(&block266, &phi_bb266_17, &phi_bb266_18, &phi_bb266_19, &phi_bb266_28, &phi_bb266_29, &phi_bb266_30, &phi_bb266_31, &phi_bb266_34, &phi_bb266_35, &phi_bb266_36, &phi_bb266_37, &phi_bb266_39);
    tmp445 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp446 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb266_34}, TNode<IntPtrT>{tmp445});
    ca_.Branch(tmp446, &block268, std::vector<compiler::Node*>{phi_bb266_17, phi_bb266_18, phi_bb266_19, phi_bb266_28, phi_bb266_29, phi_bb266_30, phi_bb266_31, phi_bb266_34, phi_bb266_35, phi_bb266_36, phi_bb266_37, phi_bb266_39}, &block269, std::vector<compiler::Node*>{phi_bb266_17, phi_bb266_18, phi_bb266_19, phi_bb266_28, phi_bb266_29, phi_bb266_30, phi_bb266_31, phi_bb266_34, phi_bb266_35, phi_bb266_36, phi_bb266_37, phi_bb266_39});
  }

  TNode<JSAny> phi_bb268_17;
  TNode<IntPtrT> phi_bb268_18;
  TNode<BoolT> phi_bb268_19;
  TNode<IntPtrT> phi_bb268_28;
  TNode<IntPtrT> phi_bb268_29;
  TNode<IntPtrT> phi_bb268_30;
  TNode<IntPtrT> phi_bb268_31;
  TNode<IntPtrT> phi_bb268_34;
  TNode<BoolT> phi_bb268_35;
  TNode<BoolT> phi_bb268_36;
  TNode<Union<FixedArray, Smi>> phi_bb268_37;
  TNode<IntPtrT> phi_bb268_39;
  TNode<Union<HeapObject, TaggedIndex>> tmp447;
  TNode<IntPtrT> tmp448;
  TNode<IntPtrT> tmp449;
  TNode<BoolT> tmp450;
  if (block268.is_used()) {
    ca_.Bind(&block268, &phi_bb268_17, &phi_bb268_18, &phi_bb268_19, &phi_bb268_28, &phi_bb268_29, &phi_bb268_30, &phi_bb268_31, &phi_bb268_34, &phi_bb268_35, &phi_bb268_36, &phi_bb268_37, &phi_bb268_39);
    std::tie(tmp447, tmp448) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb268_34}).Flatten();
    tmp449 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp450 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block264, phi_bb268_17, phi_bb268_18, phi_bb268_19, phi_bb268_28, phi_bb268_29, phi_bb268_30, phi_bb268_31, tmp449, tmp450, phi_bb268_36, phi_bb268_37, phi_bb268_39, tmp447, tmp448);
  }

  TNode<JSAny> phi_bb269_17;
  TNode<IntPtrT> phi_bb269_18;
  TNode<BoolT> phi_bb269_19;
  TNode<IntPtrT> phi_bb269_28;
  TNode<IntPtrT> phi_bb269_29;
  TNode<IntPtrT> phi_bb269_30;
  TNode<IntPtrT> phi_bb269_31;
  TNode<IntPtrT> phi_bb269_34;
  TNode<BoolT> phi_bb269_35;
  TNode<BoolT> phi_bb269_36;
  TNode<Union<FixedArray, Smi>> phi_bb269_37;
  TNode<IntPtrT> phi_bb269_39;
  TNode<Union<HeapObject, TaggedIndex>> tmp451;
  TNode<IntPtrT> tmp452;
  TNode<IntPtrT> tmp453;
  TNode<IntPtrT> tmp454;
  TNode<IntPtrT> tmp455;
  TNode<IntPtrT> tmp456;
  TNode<BoolT> tmp457;
  if (block269.is_used()) {
    ca_.Bind(&block269, &phi_bb269_17, &phi_bb269_18, &phi_bb269_19, &phi_bb269_28, &phi_bb269_29, &phi_bb269_30, &phi_bb269_31, &phi_bb269_34, &phi_bb269_35, &phi_bb269_36, &phi_bb269_37, &phi_bb269_39);
    std::tie(tmp451, tmp452) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb269_31}).Flatten();
    tmp453 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp454 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb269_31}, TNode<IntPtrT>{tmp453});
    tmp455 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp456 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp454}, TNode<IntPtrT>{tmp455});
    tmp457 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block264, phi_bb269_17, phi_bb269_18, phi_bb269_19, phi_bb269_28, phi_bb269_29, phi_bb269_30, tmp456, tmp454, tmp457, phi_bb269_36, phi_bb269_37, phi_bb269_39, tmp451, tmp452);
  }

  TNode<JSAny> phi_bb264_17;
  TNode<IntPtrT> phi_bb264_18;
  TNode<BoolT> phi_bb264_19;
  TNode<IntPtrT> phi_bb264_28;
  TNode<IntPtrT> phi_bb264_29;
  TNode<IntPtrT> phi_bb264_30;
  TNode<IntPtrT> phi_bb264_31;
  TNode<IntPtrT> phi_bb264_34;
  TNode<BoolT> phi_bb264_35;
  TNode<BoolT> phi_bb264_36;
  TNode<Union<FixedArray, Smi>> phi_bb264_37;
  TNode<IntPtrT> phi_bb264_39;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb264_42;
  TNode<IntPtrT> phi_bb264_43;
  if (block264.is_used()) {
    ca_.Bind(&block264, &phi_bb264_17, &phi_bb264_18, &phi_bb264_19, &phi_bb264_28, &phi_bb264_29, &phi_bb264_30, &phi_bb264_31, &phi_bb264_34, &phi_bb264_35, &phi_bb264_36, &phi_bb264_37, &phi_bb264_39, &phi_bb264_42, &phi_bb264_43);
    ca_.Goto(&block261, phi_bb264_17, phi_bb264_18, phi_bb264_19, phi_bb264_28, phi_bb264_29, phi_bb264_30, phi_bb264_31, phi_bb264_34, phi_bb264_35, phi_bb264_36, phi_bb264_37, phi_bb264_39, phi_bb264_42, phi_bb264_43);
  }

  TNode<JSAny> phi_bb261_17;
  TNode<IntPtrT> phi_bb261_18;
  TNode<BoolT> phi_bb261_19;
  TNode<IntPtrT> phi_bb261_28;
  TNode<IntPtrT> phi_bb261_29;
  TNode<IntPtrT> phi_bb261_30;
  TNode<IntPtrT> phi_bb261_31;
  TNode<IntPtrT> phi_bb261_34;
  TNode<BoolT> phi_bb261_35;
  TNode<BoolT> phi_bb261_36;
  TNode<Union<FixedArray, Smi>> phi_bb261_37;
  TNode<IntPtrT> phi_bb261_39;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb261_42;
  TNode<IntPtrT> phi_bb261_43;
  TNode<IntPtrT> tmp458;
  if (block261.is_used()) {
    ca_.Bind(&block261, &phi_bb261_17, &phi_bb261_18, &phi_bb261_19, &phi_bb261_28, &phi_bb261_29, &phi_bb261_30, &phi_bb261_31, &phi_bb261_34, &phi_bb261_35, &phi_bb261_36, &phi_bb261_37, &phi_bb261_39, &phi_bb261_42, &phi_bb261_43);
    tmp458 = CodeStubAssembler(state_).BitcastTaggedToWord(TNode<Object>{tmp432});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb261_42, phi_bb261_43}, tmp458);
    ca_.Goto(&block252, phi_bb261_17, phi_bb261_18, phi_bb261_19, tmp434, phi_bb261_28, phi_bb261_29, phi_bb261_30, phi_bb261_31, phi_bb261_34, phi_bb261_35, phi_bb261_36, phi_bb261_37, phi_bb261_39);
  }

  TNode<JSAny> phi_bb252_17;
  TNode<IntPtrT> phi_bb252_18;
  TNode<BoolT> phi_bb252_19;
  TNode<IntPtrT> phi_bb252_27;
  TNode<IntPtrT> phi_bb252_28;
  TNode<IntPtrT> phi_bb252_29;
  TNode<IntPtrT> phi_bb252_30;
  TNode<IntPtrT> phi_bb252_31;
  TNode<IntPtrT> phi_bb252_34;
  TNode<BoolT> phi_bb252_35;
  TNode<BoolT> phi_bb252_36;
  TNode<Union<FixedArray, Smi>> phi_bb252_37;
  TNode<IntPtrT> phi_bb252_39;
  TNode<IntPtrT> tmp459;
  TNode<IntPtrT> tmp460;
  if (block252.is_used()) {
    ca_.Bind(&block252, &phi_bb252_17, &phi_bb252_18, &phi_bb252_19, &phi_bb252_27, &phi_bb252_28, &phi_bb252_29, &phi_bb252_30, &phi_bb252_31, &phi_bb252_34, &phi_bb252_35, &phi_bb252_36, &phi_bb252_37, &phi_bb252_39);
    tmp459 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp460 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb252_39}, TNode<IntPtrT>{tmp459});
    ca_.Goto(&block248, phi_bb252_17, phi_bb252_18, phi_bb252_19, phi_bb252_27, phi_bb252_28, phi_bb252_29, phi_bb252_30, phi_bb252_31, phi_bb252_34, phi_bb252_35, phi_bb252_36, phi_bb252_37, tmp460);
  }

  TNode<JSAny> phi_bb247_17;
  TNode<IntPtrT> phi_bb247_18;
  TNode<BoolT> phi_bb247_19;
  TNode<IntPtrT> phi_bb247_27;
  TNode<IntPtrT> phi_bb247_28;
  TNode<IntPtrT> phi_bb247_29;
  TNode<IntPtrT> phi_bb247_30;
  TNode<IntPtrT> phi_bb247_31;
  TNode<IntPtrT> phi_bb247_34;
  TNode<BoolT> phi_bb247_35;
  TNode<BoolT> phi_bb247_36;
  TNode<Union<FixedArray, Smi>> phi_bb247_37;
  TNode<IntPtrT> phi_bb247_39;
  if (block247.is_used()) {
    ca_.Bind(&block247, &phi_bb247_17, &phi_bb247_18, &phi_bb247_19, &phi_bb247_27, &phi_bb247_28, &phi_bb247_29, &phi_bb247_30, &phi_bb247_31, &phi_bb247_34, &phi_bb247_35, &phi_bb247_36, &phi_bb247_37, &phi_bb247_39);
    ca_.Goto(&block242, phi_bb247_17, phi_bb247_18, phi_bb247_19, phi_bb247_27, phi_bb247_28, phi_bb247_29, phi_bb247_30, phi_bb247_31, phi_bb247_34, phi_bb247_35, phi_bb247_36, phi_bb247_37);
  }

  TNode<JSAny> phi_bb242_17;
  TNode<IntPtrT> phi_bb242_18;
  TNode<BoolT> phi_bb242_19;
  TNode<IntPtrT> phi_bb242_27;
  TNode<IntPtrT> phi_bb242_28;
  TNode<IntPtrT> phi_bb242_29;
  TNode<IntPtrT> phi_bb242_30;
  TNode<IntPtrT> phi_bb242_31;
  TNode<IntPtrT> phi_bb242_34;
  TNode<BoolT> phi_bb242_35;
  TNode<BoolT> phi_bb242_36;
  TNode<Union<FixedArray, Smi>> phi_bb242_37;
  TNode<RawPtrT> tmp461;
  TNode<RawPtrT> tmp462;
  if (block242.is_used()) {
    ca_.Bind(&block242, &phi_bb242_17, &phi_bb242_18, &phi_bb242_19, &phi_bb242_27, &phi_bb242_28, &phi_bb242_29, &phi_bb242_30, &phi_bb242_31, &phi_bb242_34, &phi_bb242_35, &phi_bb242_36, &phi_bb242_37);
    tmp461 = CodeStubAssembler(state_).GCUnsafeReferenceToRawPtr(TNode<Union<HeapObject, TaggedIndex>>{phi_bb131_23}, TNode<IntPtrT>{phi_bb131_24});
    tmp462 = (TNode<RawPtrT>{tmp461});
    ca_.Branch(phi_bb242_35, &block272, std::vector<compiler::Node*>{phi_bb242_17, phi_bb242_18, phi_bb242_19, phi_bb242_27, phi_bb242_28, phi_bb242_29, phi_bb242_30, phi_bb242_31, phi_bb242_34, phi_bb242_35, phi_bb242_36, phi_bb242_37, phi_bb242_31}, &block273, std::vector<compiler::Node*>{phi_bb242_17, phi_bb242_18, phi_bb242_19, phi_bb242_27, phi_bb242_28, phi_bb242_29, phi_bb242_30, phi_bb242_31, phi_bb242_34, phi_bb242_35, phi_bb242_36, phi_bb242_37, phi_bb242_31});
  }

  TNode<JSAny> phi_bb272_17;
  TNode<IntPtrT> phi_bb272_18;
  TNode<BoolT> phi_bb272_19;
  TNode<IntPtrT> phi_bb272_27;
  TNode<IntPtrT> phi_bb272_28;
  TNode<IntPtrT> phi_bb272_29;
  TNode<IntPtrT> phi_bb272_30;
  TNode<IntPtrT> phi_bb272_31;
  TNode<IntPtrT> phi_bb272_34;
  TNode<BoolT> phi_bb272_35;
  TNode<BoolT> phi_bb272_36;
  TNode<Union<FixedArray, Smi>> phi_bb272_37;
  TNode<IntPtrT> phi_bb272_40;
  TNode<IntPtrT> tmp463;
  TNode<IntPtrT> tmp464;
  if (block272.is_used()) {
    ca_.Bind(&block272, &phi_bb272_17, &phi_bb272_18, &phi_bb272_19, &phi_bb272_27, &phi_bb272_28, &phi_bb272_29, &phi_bb272_30, &phi_bb272_31, &phi_bb272_34, &phi_bb272_35, &phi_bb272_36, &phi_bb272_37, &phi_bb272_40);
    tmp463 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp464 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb272_40}, TNode<IntPtrT>{tmp463});
    ca_.Goto(&block273, phi_bb272_17, phi_bb272_18, phi_bb272_19, phi_bb272_27, phi_bb272_28, phi_bb272_29, phi_bb272_30, phi_bb272_31, phi_bb272_34, phi_bb272_35, phi_bb272_36, phi_bb272_37, tmp464);
  }

  TNode<JSAny> phi_bb273_17;
  TNode<IntPtrT> phi_bb273_18;
  TNode<BoolT> phi_bb273_19;
  TNode<IntPtrT> phi_bb273_27;
  TNode<IntPtrT> phi_bb273_28;
  TNode<IntPtrT> phi_bb273_29;
  TNode<IntPtrT> phi_bb273_30;
  TNode<IntPtrT> phi_bb273_31;
  TNode<IntPtrT> phi_bb273_34;
  TNode<BoolT> phi_bb273_35;
  TNode<BoolT> phi_bb273_36;
  TNode<Union<FixedArray, Smi>> phi_bb273_37;
  TNode<IntPtrT> phi_bb273_40;
  TNode<RawPtrT> tmp465;
  TNode<IntPtrT> tmp466;
  TNode<BoolT> tmp467;
  if (block273.is_used()) {
    ca_.Bind(&block273, &phi_bb273_17, &phi_bb273_18, &phi_bb273_19, &phi_bb273_27, &phi_bb273_28, &phi_bb273_29, &phi_bb273_30, &phi_bb273_31, &phi_bb273_34, &phi_bb273_35, &phi_bb273_36, &phi_bb273_37, &phi_bb273_40);
    tmp465 = CodeStubAssembler(state_).GCUnsafeReferenceToRawPtr(TNode<Union<HeapObject, TaggedIndex>>{tmp181}, TNode<IntPtrT>{phi_bb273_40});
    tmp466 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp467 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp188}, TNode<IntPtrT>{tmp466});
    ca_.Branch(tmp467, &block277, std::vector<compiler::Node*>{phi_bb273_17, phi_bb273_18, phi_bb273_19, phi_bb273_27, phi_bb273_28, phi_bb273_29, phi_bb273_30, phi_bb273_31, phi_bb273_34, phi_bb273_35, phi_bb273_36, phi_bb273_37}, &block278, std::vector<compiler::Node*>{phi_bb273_17, phi_bb273_18, phi_bb273_19, phi_bb273_27, phi_bb273_28, phi_bb273_29, phi_bb273_30, phi_bb273_31, phi_bb273_34, phi_bb273_35, phi_bb273_36, phi_bb273_37});
  }

  TNode<JSAny> phi_bb277_17;
  TNode<IntPtrT> phi_bb277_18;
  TNode<BoolT> phi_bb277_19;
  TNode<IntPtrT> phi_bb277_27;
  TNode<IntPtrT> phi_bb277_28;
  TNode<IntPtrT> phi_bb277_29;
  TNode<IntPtrT> phi_bb277_30;
  TNode<IntPtrT> phi_bb277_31;
  TNode<IntPtrT> phi_bb277_34;
  TNode<BoolT> phi_bb277_35;
  TNode<BoolT> phi_bb277_36;
  TNode<Union<FixedArray, Smi>> phi_bb277_37;
  TNode<BoolT> tmp468;
  if (block277.is_used()) {
    ca_.Bind(&block277, &phi_bb277_17, &phi_bb277_18, &phi_bb277_19, &phi_bb277_27, &phi_bb277_28, &phi_bb277_29, &phi_bb277_30, &phi_bb277_31, &phi_bb277_34, &phi_bb277_35, &phi_bb277_36, &phi_bb277_37);
    tmp468 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block279, phi_bb277_17, phi_bb277_18, phi_bb277_19, phi_bb277_27, phi_bb277_28, phi_bb277_29, phi_bb277_30, phi_bb277_31, phi_bb277_34, phi_bb277_35, phi_bb277_36, phi_bb277_37, tmp468);
  }

  TNode<JSAny> phi_bb278_17;
  TNode<IntPtrT> phi_bb278_18;
  TNode<BoolT> phi_bb278_19;
  TNode<IntPtrT> phi_bb278_27;
  TNode<IntPtrT> phi_bb278_28;
  TNode<IntPtrT> phi_bb278_29;
  TNode<IntPtrT> phi_bb278_30;
  TNode<IntPtrT> phi_bb278_31;
  TNode<IntPtrT> phi_bb278_34;
  TNode<BoolT> phi_bb278_35;
  TNode<BoolT> phi_bb278_36;
  TNode<Union<FixedArray, Smi>> phi_bb278_37;
  TNode<BoolT> tmp469;
  if (block278.is_used()) {
    ca_.Bind(&block278, &phi_bb278_17, &phi_bb278_18, &phi_bb278_19, &phi_bb278_27, &phi_bb278_28, &phi_bb278_29, &phi_bb278_30, &phi_bb278_31, &phi_bb278_34, &phi_bb278_35, &phi_bb278_36, &phi_bb278_37);
    tmp469 = CodeStubAssembler(state_).IntPtrLessThanOrEqual(TNode<IntPtrT>{phi_bb278_31}, TNode<IntPtrT>{tmp188});
    ca_.Goto(&block279, phi_bb278_17, phi_bb278_18, phi_bb278_19, phi_bb278_27, phi_bb278_28, phi_bb278_29, phi_bb278_30, phi_bb278_31, phi_bb278_34, phi_bb278_35, phi_bb278_36, phi_bb278_37, tmp469);
  }

  TNode<JSAny> phi_bb279_17;
  TNode<IntPtrT> phi_bb279_18;
  TNode<BoolT> phi_bb279_19;
  TNode<IntPtrT> phi_bb279_27;
  TNode<IntPtrT> phi_bb279_28;
  TNode<IntPtrT> phi_bb279_29;
  TNode<IntPtrT> phi_bb279_30;
  TNode<IntPtrT> phi_bb279_31;
  TNode<IntPtrT> phi_bb279_34;
  TNode<BoolT> phi_bb279_35;
  TNode<BoolT> phi_bb279_36;
  TNode<Union<FixedArray, Smi>> phi_bb279_37;
  TNode<BoolT> phi_bb279_42;
  if (block279.is_used()) {
    ca_.Bind(&block279, &phi_bb279_17, &phi_bb279_18, &phi_bb279_19, &phi_bb279_27, &phi_bb279_28, &phi_bb279_29, &phi_bb279_30, &phi_bb279_31, &phi_bb279_34, &phi_bb279_35, &phi_bb279_36, &phi_bb279_37, &phi_bb279_42);
    ca_.Branch(phi_bb279_42, &block275, std::vector<compiler::Node*>{phi_bb279_17, phi_bb279_18, phi_bb279_19, phi_bb279_27, phi_bb279_28, phi_bb279_29, phi_bb279_30, phi_bb279_31, phi_bb279_34, phi_bb279_35, phi_bb279_36, phi_bb279_37}, &block276, std::vector<compiler::Node*>{phi_bb279_17, phi_bb279_18, phi_bb279_19, phi_bb279_27, phi_bb279_28, phi_bb279_29, phi_bb279_30, phi_bb279_31, phi_bb279_34, phi_bb279_35, phi_bb279_36, phi_bb279_37});
  }

  TNode<JSAny> phi_bb276_17;
  TNode<IntPtrT> phi_bb276_18;
  TNode<BoolT> phi_bb276_19;
  TNode<IntPtrT> phi_bb276_27;
  TNode<IntPtrT> phi_bb276_28;
  TNode<IntPtrT> phi_bb276_29;
  TNode<IntPtrT> phi_bb276_30;
  TNode<IntPtrT> phi_bb276_31;
  TNode<IntPtrT> phi_bb276_34;
  TNode<BoolT> phi_bb276_35;
  TNode<BoolT> phi_bb276_36;
  TNode<Union<FixedArray, Smi>> phi_bb276_37;
  if (block276.is_used()) {
    ca_.Bind(&block276, &phi_bb276_17, &phi_bb276_18, &phi_bb276_19, &phi_bb276_27, &phi_bb276_28, &phi_bb276_29, &phi_bb276_30, &phi_bb276_31, &phi_bb276_34, &phi_bb276_35, &phi_bb276_36, &phi_bb276_37);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/js-to-wasm.tq", 379});
      CodeStubAssembler(state_).FailAssert("Torque assert 'this.paramBufferEnd == 0 || this.nextStack <= this.paramBufferEnd' failed", pos_stack);
    }
  }

  TNode<JSAny> phi_bb275_17;
  TNode<IntPtrT> phi_bb275_18;
  TNode<BoolT> phi_bb275_19;
  TNode<IntPtrT> phi_bb275_27;
  TNode<IntPtrT> phi_bb275_28;
  TNode<IntPtrT> phi_bb275_29;
  TNode<IntPtrT> phi_bb275_30;
  TNode<IntPtrT> phi_bb275_31;
  TNode<IntPtrT> phi_bb275_34;
  TNode<BoolT> phi_bb275_35;
  TNode<BoolT> phi_bb275_36;
  TNode<Union<FixedArray, Smi>> phi_bb275_37;
  TNode<WasmInternalFunction> tmp470;
  TNode<IntPtrT> tmp471;
  TNode<Uint32T> tmp472;
  TNode<RawPtrT> tmp473;
  TNode<RawPtrT> tmp474;
  TNode<IntPtrT> tmp475;
  TNode<Union<HeapObject, TaggedIndex>> tmp476;
  TNode<IntPtrT> tmp477;
  TNode<Int32T> tmp478;
  TNode<IntPtrT> tmp479;
  TNode<Union<HeapObject, TaggedIndex>> tmp480;
  TNode<IntPtrT> tmp481;
  TNode<IntPtrT> tmp482;
  TNode<Union<HeapObject, TaggedIndex>> tmp483;
  TNode<IntPtrT> tmp484;
  TNode<IntPtrT> tmp485;
  TNode<Union<HeapObject, TaggedIndex>> tmp486;
  TNode<IntPtrT> tmp487;
  TNode<IntPtrT> tmp488;
  TNode<Union<HeapObject, TaggedIndex>> tmp489;
  TNode<IntPtrT> tmp490;
  TNode<IntPtrT> tmp491;
  TNode<Union<HeapObject, TaggedIndex>> tmp492;
  TNode<IntPtrT> tmp493;
  TNode<IntPtrT> tmp494;
  TNode<Union<HeapObject, TaggedIndex>> tmp495;
  TNode<IntPtrT> tmp496;
  if (block275.is_used()) {
    ca_.Bind(&block275, &phi_bb275_17, &phi_bb275_18, &phi_bb275_19, &phi_bb275_27, &phi_bb275_28, &phi_bb275_29, &phi_bb275_30, &phi_bb275_31, &phi_bb275_34, &phi_bb275_35, &phi_bb275_36, &phi_bb275_37);
    tmp470 = CodeStubAssembler(state_).LoadWasmInternalFunctionFromFunctionData(TNode<WasmFunctionData>{tmp2});
    tmp471 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp472 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp470, tmp471});
    tmp473 = CodeStubAssembler(state_).StackSlotPtr(JSToWasmWrapperFrameConstants::kWrapperBufferSize, CastIfEnumClass<int32_t>((SizeOf_intptr_0(state_))));
    tmp474 = (TNode<RawPtrT>{tmp473});
    tmp475 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferReturnCount);
    std::tie(tmp476, tmp477) = GetRefAt_int32_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp474}, TNode<IntPtrT>{tmp475}).Flatten();
    tmp478 = CodeStubAssembler(state_).TruncateIntPtrToInt32(TNode<IntPtrT>{tmp26});
    CodeStubAssembler(state_).StoreReference<Int32T>(CodeStubAssembler::Reference{tmp476, tmp477}, tmp478);
    tmp479 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferRefReturnCount);
    std::tie(tmp480, tmp481) = GetRefAt_bool_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp474}, TNode<IntPtrT>{tmp479}).Flatten();
    CodeStubAssembler(state_).StoreReference<BoolT>(CodeStubAssembler::Reference{tmp480, tmp481}, phi_bb275_19);
    tmp482 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferSigRepresentationArray);
    std::tie(tmp483, tmp484) = GetRefAt_RawPtr_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp474}, TNode<IntPtrT>{tmp482}).Flatten();
    CodeStubAssembler(state_).StoreReference<RawPtrT>(CodeStubAssembler::Reference{tmp483, tmp484}, tmp30);
    tmp485 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferStackReturnBufferSize);
    std::tie(tmp486, tmp487) = GetRefAt_intptr_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp474}, TNode<IntPtrT>{tmp485}).Flatten();
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{tmp486, tmp487}, phi_bb275_18);
    tmp488 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferCallTarget);
    std::tie(tmp489, tmp490) = GetRefAt_WasmCodePointer_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp474}, TNode<IntPtrT>{tmp488}).Flatten();
    CodeStubAssembler(state_).StoreReference<Uint32T>(CodeStubAssembler::Reference{tmp489, tmp490}, tmp472);
    tmp491 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferParamStart);
    std::tie(tmp492, tmp493) = GetRefAt_RawPtr_intptr_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp474}, TNode<IntPtrT>{tmp491}).Flatten();
    CodeStubAssembler(state_).StoreReference<RawPtrT>(CodeStubAssembler::Reference{tmp492, tmp493}, tmp462);
    tmp494 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferParamEnd);
    std::tie(tmp495, tmp496) = GetRefAt_RawPtr_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp474}, TNode<IntPtrT>{tmp494}).Flatten();
    CodeStubAssembler(state_).StoreReference<RawPtrT>(CodeStubAssembler::Reference{tmp495, tmp496}, tmp465);
    if (((CodeStubAssembler(state_).ConstexprInt32Equal(CastIfEnumClass<int32_t>(p_promise), CastIfEnumClass<int32_t>(wasm::Promise::kPromise))))) {
      ca_.Goto(&block280, phi_bb275_17, phi_bb275_18, phi_bb275_19, phi_bb275_27, phi_bb275_28, phi_bb275_29, phi_bb275_30, phi_bb275_31, phi_bb275_34, phi_bb275_35, phi_bb275_36, phi_bb275_37);
    } else {
      ca_.Goto(&block281, phi_bb275_17, phi_bb275_18, phi_bb275_19, phi_bb275_27, phi_bb275_28, phi_bb275_29, phi_bb275_30, phi_bb275_31, phi_bb275_34, phi_bb275_35, phi_bb275_36, phi_bb275_37);
    }
  }

  TNode<JSAny> phi_bb280_17;
  TNode<IntPtrT> phi_bb280_18;
  TNode<BoolT> phi_bb280_19;
  TNode<IntPtrT> phi_bb280_27;
  TNode<IntPtrT> phi_bb280_28;
  TNode<IntPtrT> phi_bb280_29;
  TNode<IntPtrT> phi_bb280_30;
  TNode<IntPtrT> phi_bb280_31;
  TNode<IntPtrT> phi_bb280_34;
  TNode<BoolT> phi_bb280_35;
  TNode<BoolT> phi_bb280_36;
  TNode<Union<FixedArray, Smi>> phi_bb280_37;
  TNode<JSAny> tmp497;
  if (block280.is_used()) {
    ca_.Bind(&block280, &phi_bb280_17, &phi_bb280_18, &phi_bb280_19, &phi_bb280_27, &phi_bb280_28, &phi_bb280_29, &phi_bb280_30, &phi_bb280_31, &phi_bb280_34, &phi_bb280_35, &phi_bb280_36, &phi_bb280_37);
    tmp497 = ca_.CallBuiltin<JSAny>(Builtin::kWasmReturnPromiseOnSuspendAsm, TNode<Object>(), tmp474, tmp18, phi_bb280_17);
    ca_.Goto(&block282, phi_bb280_17, phi_bb280_18, phi_bb280_19, phi_bb280_27, phi_bb280_28, phi_bb280_29, phi_bb280_30, phi_bb280_31, phi_bb280_34, phi_bb280_35, phi_bb280_36, phi_bb280_37, tmp497);
  }

  TNode<JSAny> phi_bb281_17;
  TNode<IntPtrT> phi_bb281_18;
  TNode<BoolT> phi_bb281_19;
  TNode<IntPtrT> phi_bb281_27;
  TNode<IntPtrT> phi_bb281_28;
  TNode<IntPtrT> phi_bb281_29;
  TNode<IntPtrT> phi_bb281_30;
  TNode<IntPtrT> phi_bb281_31;
  TNode<IntPtrT> phi_bb281_34;
  TNode<BoolT> phi_bb281_35;
  TNode<BoolT> phi_bb281_36;
  TNode<Union<FixedArray, Smi>> phi_bb281_37;
  TNode<BoolT> tmp498;
  if (block281.is_used()) {
    ca_.Bind(&block281, &phi_bb281_17, &phi_bb281_18, &phi_bb281_19, &phi_bb281_27, &phi_bb281_28, &phi_bb281_29, &phi_bb281_30, &phi_bb281_31, &phi_bb281_34, &phi_bb281_35, &phi_bb281_36, &phi_bb281_37);
    tmp498 = FromConstexpr_bool_constexpr_bool_0(state_, (CodeStubAssembler(state_).ConstexprInt32Equal(CastIfEnumClass<int32_t>(p_promise), CastIfEnumClass<int32_t>(wasm::Promise::kNoPromise))));
    ca_.Branch(tmp498, &block283, std::vector<compiler::Node*>{phi_bb281_17, phi_bb281_18, phi_bb281_19, phi_bb281_27, phi_bb281_28, phi_bb281_29, phi_bb281_30, phi_bb281_31, phi_bb281_34, phi_bb281_35, phi_bb281_36, phi_bb281_37}, &block284, std::vector<compiler::Node*>{phi_bb281_17, phi_bb281_18, phi_bb281_19, phi_bb281_27, phi_bb281_28, phi_bb281_29, phi_bb281_30, phi_bb281_31, phi_bb281_34, phi_bb281_35, phi_bb281_36, phi_bb281_37});
  }

  TNode<JSAny> phi_bb283_17;
  TNode<IntPtrT> phi_bb283_18;
  TNode<BoolT> phi_bb283_19;
  TNode<IntPtrT> phi_bb283_27;
  TNode<IntPtrT> phi_bb283_28;
  TNode<IntPtrT> phi_bb283_29;
  TNode<IntPtrT> phi_bb283_30;
  TNode<IntPtrT> phi_bb283_31;
  TNode<IntPtrT> phi_bb283_34;
  TNode<BoolT> phi_bb283_35;
  TNode<BoolT> phi_bb283_36;
  TNode<Union<FixedArray, Smi>> phi_bb283_37;
  TNode<JSAny> tmp499;
  if (block283.is_used()) {
    ca_.Bind(&block283, &phi_bb283_17, &phi_bb283_18, &phi_bb283_19, &phi_bb283_27, &phi_bb283_28, &phi_bb283_29, &phi_bb283_30, &phi_bb283_31, &phi_bb283_34, &phi_bb283_35, &phi_bb283_36, &phi_bb283_37);
    tmp499 = ca_.CallBuiltin<JSAny>(Builtin::kJSToWasmWrapperAsm, TNode<Object>(), tmp474, tmp18, phi_bb283_17);
    ca_.Goto(&block285, phi_bb283_17, phi_bb283_18, phi_bb283_19, phi_bb283_27, phi_bb283_28, phi_bb283_29, phi_bb283_30, phi_bb283_31, phi_bb283_34, phi_bb283_35, phi_bb283_36, phi_bb283_37, tmp499);
  }

  TNode<JSAny> phi_bb284_17;
  TNode<IntPtrT> phi_bb284_18;
  TNode<BoolT> phi_bb284_19;
  TNode<IntPtrT> phi_bb284_27;
  TNode<IntPtrT> phi_bb284_28;
  TNode<IntPtrT> phi_bb284_29;
  TNode<IntPtrT> phi_bb284_30;
  TNode<IntPtrT> phi_bb284_31;
  TNode<IntPtrT> phi_bb284_34;
  TNode<BoolT> phi_bb284_35;
  TNode<BoolT> phi_bb284_36;
  TNode<Union<FixedArray, Smi>> phi_bb284_37;
  TNode<BoolT> tmp500;
  if (block284.is_used()) {
    ca_.Bind(&block284, &phi_bb284_17, &phi_bb284_18, &phi_bb284_19, &phi_bb284_27, &phi_bb284_28, &phi_bb284_29, &phi_bb284_30, &phi_bb284_31, &phi_bb284_34, &phi_bb284_35, &phi_bb284_36, &phi_bb284_37);
    tmp500 = FromConstexpr_bool_constexpr_bool_0(state_, (CodeStubAssembler(state_).ConstexprInt32Equal(CastIfEnumClass<int32_t>(p_promise), CastIfEnumClass<int32_t>(wasm::Promise::kStressSwitch))));
    ca_.Branch(tmp500, &block286, std::vector<compiler::Node*>{phi_bb284_17, phi_bb284_18, phi_bb284_19, phi_bb284_27, phi_bb284_28, phi_bb284_29, phi_bb284_30, phi_bb284_31, phi_bb284_34, phi_bb284_35, phi_bb284_36, phi_bb284_37}, &block287, std::vector<compiler::Node*>{phi_bb284_17, phi_bb284_18, phi_bb284_19, phi_bb284_27, phi_bb284_28, phi_bb284_29, phi_bb284_30, phi_bb284_31, phi_bb284_34, phi_bb284_35, phi_bb284_36, phi_bb284_37});
  }

  TNode<JSAny> phi_bb286_17;
  TNode<IntPtrT> phi_bb286_18;
  TNode<BoolT> phi_bb286_19;
  TNode<IntPtrT> phi_bb286_27;
  TNode<IntPtrT> phi_bb286_28;
  TNode<IntPtrT> phi_bb286_29;
  TNode<IntPtrT> phi_bb286_30;
  TNode<IntPtrT> phi_bb286_31;
  TNode<IntPtrT> phi_bb286_34;
  TNode<BoolT> phi_bb286_35;
  TNode<BoolT> phi_bb286_36;
  TNode<Union<FixedArray, Smi>> phi_bb286_37;
  TNode<JSAny> tmp501;
  if (block286.is_used()) {
    ca_.Bind(&block286, &phi_bb286_17, &phi_bb286_18, &phi_bb286_19, &phi_bb286_27, &phi_bb286_28, &phi_bb286_29, &phi_bb286_30, &phi_bb286_31, &phi_bb286_34, &phi_bb286_35, &phi_bb286_36, &phi_bb286_37);
    tmp501 = ca_.CallBuiltin<JSAny>(Builtin::kJSToWasmStressSwitchStacksAsm, TNode<Object>(), tmp474, tmp18, phi_bb286_17);
    ca_.Goto(&block285, phi_bb286_17, phi_bb286_18, phi_bb286_19, phi_bb286_27, phi_bb286_28, phi_bb286_29, phi_bb286_30, phi_bb286_31, phi_bb286_34, phi_bb286_35, phi_bb286_36, phi_bb286_37, tmp501);
  }

  TNode<JSAny> phi_bb287_17;
  TNode<IntPtrT> phi_bb287_18;
  TNode<BoolT> phi_bb287_19;
  TNode<IntPtrT> phi_bb287_27;
  TNode<IntPtrT> phi_bb287_28;
  TNode<IntPtrT> phi_bb287_29;
  TNode<IntPtrT> phi_bb287_30;
  TNode<IntPtrT> phi_bb287_31;
  TNode<IntPtrT> phi_bb287_34;
  TNode<BoolT> phi_bb287_35;
  TNode<BoolT> phi_bb287_36;
  TNode<Union<FixedArray, Smi>> phi_bb287_37;
  if (block287.is_used()) {
    ca_.Bind(&block287, &phi_bb287_17, &phi_bb287_18, &phi_bb287_19, &phi_bb287_27, &phi_bb287_28, &phi_bb287_29, &phi_bb287_30, &phi_bb287_31, &phi_bb287_34, &phi_bb287_35, &phi_bb287_36, &phi_bb287_37);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb285_17;
  TNode<IntPtrT> phi_bb285_18;
  TNode<BoolT> phi_bb285_19;
  TNode<IntPtrT> phi_bb285_27;
  TNode<IntPtrT> phi_bb285_28;
  TNode<IntPtrT> phi_bb285_29;
  TNode<IntPtrT> phi_bb285_30;
  TNode<IntPtrT> phi_bb285_31;
  TNode<IntPtrT> phi_bb285_34;
  TNode<BoolT> phi_bb285_35;
  TNode<BoolT> phi_bb285_36;
  TNode<Union<FixedArray, Smi>> phi_bb285_37;
  TNode<JSAny> phi_bb285_43;
  if (block285.is_used()) {
    ca_.Bind(&block285, &phi_bb285_17, &phi_bb285_18, &phi_bb285_19, &phi_bb285_27, &phi_bb285_28, &phi_bb285_29, &phi_bb285_30, &phi_bb285_31, &phi_bb285_34, &phi_bb285_35, &phi_bb285_36, &phi_bb285_37, &phi_bb285_43);
    ca_.Goto(&block282, phi_bb285_17, phi_bb285_18, phi_bb285_19, phi_bb285_27, phi_bb285_28, phi_bb285_29, phi_bb285_30, phi_bb285_31, phi_bb285_34, phi_bb285_35, phi_bb285_36, phi_bb285_37, phi_bb285_43);
  }

  TNode<JSAny> phi_bb282_17;
  TNode<IntPtrT> phi_bb282_18;
  TNode<BoolT> phi_bb282_19;
  TNode<IntPtrT> phi_bb282_27;
  TNode<IntPtrT> phi_bb282_28;
  TNode<IntPtrT> phi_bb282_29;
  TNode<IntPtrT> phi_bb282_30;
  TNode<IntPtrT> phi_bb282_31;
  TNode<IntPtrT> phi_bb282_34;
  TNode<BoolT> phi_bb282_35;
  TNode<BoolT> phi_bb282_36;
  TNode<Union<FixedArray, Smi>> phi_bb282_37;
  TNode<JSAny> phi_bb282_43;
  if (block282.is_used()) {
    ca_.Bind(&block282, &phi_bb282_17, &phi_bb282_18, &phi_bb282_19, &phi_bb282_27, &phi_bb282_28, &phi_bb282_29, &phi_bb282_30, &phi_bb282_31, &phi_bb282_34, &phi_bb282_35, &phi_bb282_36, &phi_bb282_37, &phi_bb282_43);
    ca_.Goto(&block289, phi_bb282_43);
  }

  TNode<JSAny> phi_bb289_7;
    ca_.Bind(&block289, &phi_bb289_7);
  return TNode<JSAny>{phi_bb289_7};
}

TF_BUILTIN(JSToWasmWrapper, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Word32T> argc = UncheckedParameter<Word32T>(Descriptor::kJSActualArgumentsCount);
  TNode<IntPtrT> arguments_length(ChangeInt32ToIntPtr(UncheckedCast<Int32T>(argc)));
  TNode<RawPtrT> arguments_frame = UncheckedCast<RawPtrT>(LoadFramePointer());
  TorqueStructArguments torque_arguments(GetFrameArguments(arguments_frame, arguments_length, FrameArgumentsArgcType::kCountIncludesReceiver));
  CodeStubArguments arguments(this, torque_arguments);
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = arguments.GetReceiver();
  USE(parameter1);
  TNode<JSFunction> parameter2 = UncheckedParameter<JSFunction>(Descriptor::kJSTarget);
  USE(parameter2);
  TNode<JSDispatchHandleT> parameter3 = ReinterpretCast<JSDispatchHandleT>(LoadJSFunctionDispatchHandle(UncheckedParameter<JSFunction>(Descriptor::kJSTarget)));
  USE(parameter3);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    CodeStubAssembler(state_).SetSupportsDynamicParameterCount(TNode<JSFunction>{parameter2}, TNode<JSDispatchHandleT>{parameter3});
    tmp0 = JSToWasmWrapperHelper_0(state_, TNode<NativeContext>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSFunction>{parameter2}, TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, CastIfEnumClass<wasm::Promise>(wasm::Promise::kNoPromise));
    arguments.PopAndReturn(tmp0);
  }
}

TF_BUILTIN(WasmPromising, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Word32T> argc = UncheckedParameter<Word32T>(Descriptor::kJSActualArgumentsCount);
  TNode<IntPtrT> arguments_length(ChangeInt32ToIntPtr(UncheckedCast<Int32T>(argc)));
  TNode<RawPtrT> arguments_frame = UncheckedCast<RawPtrT>(LoadFramePointer());
  TorqueStructArguments torque_arguments(GetFrameArguments(arguments_frame, arguments_length, FrameArgumentsArgcType::kCountIncludesReceiver));
  CodeStubArguments arguments(this, torque_arguments);
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = arguments.GetReceiver();
  USE(parameter1);
  TNode<JSFunction> parameter2 = UncheckedParameter<JSFunction>(Descriptor::kJSTarget);
  USE(parameter2);
  TNode<JSDispatchHandleT> parameter3 = ReinterpretCast<JSDispatchHandleT>(LoadJSFunctionDispatchHandle(UncheckedParameter<JSFunction>(Descriptor::kJSTarget)));
  USE(parameter3);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    CodeStubAssembler(state_).SetSupportsDynamicParameterCount(TNode<JSFunction>{parameter2}, TNode<JSDispatchHandleT>{parameter3});
    tmp0 = JSToWasmWrapperHelper_0(state_, TNode<NativeContext>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSFunction>{parameter2}, TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, CastIfEnumClass<wasm::Promise>(wasm::Promise::kPromise));
    arguments.PopAndReturn(tmp0);
  }
}

TF_BUILTIN(WasmStressSwitch, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Word32T> argc = UncheckedParameter<Word32T>(Descriptor::kJSActualArgumentsCount);
  TNode<IntPtrT> arguments_length(ChangeInt32ToIntPtr(UncheckedCast<Int32T>(argc)));
  TNode<RawPtrT> arguments_frame = UncheckedCast<RawPtrT>(LoadFramePointer());
  TorqueStructArguments torque_arguments(GetFrameArguments(arguments_frame, arguments_length, FrameArgumentsArgcType::kCountIncludesReceiver));
  CodeStubArguments arguments(this, torque_arguments);
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = arguments.GetReceiver();
  USE(parameter1);
  TNode<JSFunction> parameter2 = UncheckedParameter<JSFunction>(Descriptor::kJSTarget);
  USE(parameter2);
  TNode<JSDispatchHandleT> parameter3 = ReinterpretCast<JSDispatchHandleT>(LoadJSFunctionDispatchHandle(UncheckedParameter<JSFunction>(Descriptor::kJSTarget)));
  USE(parameter3);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    CodeStubAssembler(state_).SetSupportsDynamicParameterCount(TNode<JSFunction>{parameter2}, TNode<JSDispatchHandleT>{parameter3});
    tmp0 = JSToWasmWrapperHelper_0(state_, TNode<NativeContext>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSFunction>{parameter2}, TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, CastIfEnumClass<wasm::Promise>(wasm::Promise::kStressSwitch));
    arguments.PopAndReturn(tmp0);
  }
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=860&c=1
TNode<JSAny> WasmToJSObject_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<Object> p_value) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<WasmNull> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = kWasmNull_0(state_);
    tmp1 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{p_value}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp0});
    ca_.Branch(tmp1, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<Null> tmp2;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp2 = Null_0(state_);
    ca_.Goto(&block1, tmp2);
  }

  TNode<BoolT> tmp3;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp3 = Is_WasmFuncRef_Object_0(state_, TNode<Context>{p_context}, TNode<Object>{p_value});
    ca_.Branch(tmp3, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<WasmFuncRef> tmp4;
  TNode<WasmInternalFunction> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<Union<JSFunction, Undefined>> tmp7;
  TNode<BoolT> tmp8;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp4 = UnsafeCast_WasmFuncRef_0(state_, TNode<Context>{p_context}, TNode<Object>{p_value});
    tmp5 = CodeStubAssembler(state_).LoadWasmInternalFunctionFromFuncRef(TNode<WasmFuncRef>{tmp4});
    tmp6 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp7 = CodeStubAssembler(state_).LoadReference<Union<JSFunction, Undefined>>(CodeStubAssembler::Reference{tmp5, tmp6});
    tmp8 = Is_JSFunction_JSFunction_OR_Undefined_0(state_, TNode<Context>{p_context}, TNode<Union<JSFunction, Undefined>>{tmp7});
    ca_.Branch(tmp8, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  TNode<JSFunction> tmp9;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp9 = UnsafeCast_JSFunction_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp7});
    ca_.Goto(&block1, tmp9);
  }

  TNode<JSAny> tmp10;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp10 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kWasmWasmToJSObject, p_context, p_value)); 
    ca_.Goto(&block1, tmp10);
  }

  TNode<BoolT> tmp11;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp11 = Is_String_Object_0(state_, TNode<Context>{p_context}, TNode<Object>{p_value});
    ca_.Branch(tmp11, &block11, std::vector<compiler::Node*>{}, &block12, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp12;
  TNode<BoolT> tmp13;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp12 = UnsafeCast_String_0(state_, TNode<Context>{p_context}, TNode<Object>{p_value});
    tmp13 = WasmBuiltinsAssembler(state_).InSharedSpace(TNode<HeapObject>{tmp12});
    ca_.Goto(&block13, tmp13);
  }

  TNode<BoolT> tmp14;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp14 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block13, tmp14);
  }

  TNode<BoolT> phi_bb13_3;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_3);
    ca_.Branch(phi_bb13_3, &block9, std::vector<compiler::Node*>{}, &block10, std::vector<compiler::Node*>{});
  }

  TNode<JSAny> tmp15;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp15 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kWasmWasmToJSObject, p_context, p_value)); 
    ca_.Goto(&block1, tmp15);
  }

  TNode<JSAny> tmp16;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp16 = UnsafeCast_JSAny_0(state_, TNode<Context>{p_context}, TNode<Object>{p_value});
    ca_.Goto(&block1, tmp16);
  }

  TNode<JSAny> phi_bb1_2;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_2);
    ca_.Goto(&block14, phi_bb1_2);
  }

  TNode<JSAny> phi_bb14_2;
    ca_.Bind(&block14, &phi_bb14_2);
  return TNode<JSAny>{phi_bb14_2};
}

TF_BUILTIN(JSToWasmHandleReturns, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kJsContext);
  USE(parameter0);
  TNode<JSArray> parameter1 = UncheckedParameter<JSArray>(Descriptor::kResultArray);
  USE(parameter1);
  TNode<RawPtrT> parameter2 = UncheckedParameter<RawPtrT>(Descriptor::kWrapperBuffer);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Int32T> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block44(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block61(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block62(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block52(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block63(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block67(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block70(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block71(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block73(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block74(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block66(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block75(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block82(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block83(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block85(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block86(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block81(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block78(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block87(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block91(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block92(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block94(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block95(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block97(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block98(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block93(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block90(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block88(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block89(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block76(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block99(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block102(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block103(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block104(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block108(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block109(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block111(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block112(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block107(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block105(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block101(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block100(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block114(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block115(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block118(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block116(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block121(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block124(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block125(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block127(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block128(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block130(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block131(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block126(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block123(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block136(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block137(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block122(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block117(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block45(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block143(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block144(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block145(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block142(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block141(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block148(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block146(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block151(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block155(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block156(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block158(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block159(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block161(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block162(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block157(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block154(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block163(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block164(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Int32T> block165(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block170(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block171(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block152(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block174(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block178(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block179(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block181(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block182(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block184(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block185(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block180(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block177(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block186(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block189(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block190(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Float32T> block192(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block187(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block193(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block196(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block197(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Float32T> block199(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block194(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Float32T> block195(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Float32T> block188(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block204(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block205(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block175(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block208(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block211(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block215(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block216(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block218(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block219(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block221(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block222(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block217(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block214(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block227(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block228(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block212(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block232(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block233(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block235(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block236(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block238(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block239(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block234(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block231(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block241(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block242(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block244(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block245(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block247(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block248(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block243(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block240(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block253(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block254(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block213(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block209(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block257(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block261(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block262(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block263(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block267(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block268(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block270(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block271(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block266(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block264(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block260(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block276(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block277(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block258(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block281(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block280(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block286(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block287(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block294(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block295(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block259(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block210(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block176(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block153(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT> block147(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block301(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block302(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block303(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block300(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT> block299(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Int32T> tmp3;
  TNode<Int32T> tmp4;
  TNode<BoolT> tmp5;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferReturnCount);
    std::tie(tmp1, tmp2) = GetRefAt_int32_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = FromConstexpr_int32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp5 = CodeStubAssembler(state_).Word32Equal(TNode<Int32T>{tmp3}, TNode<Int32T>{tmp4});
    ca_.Branch(tmp5, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  TNode<Undefined> tmp6;
  if (block1.is_used()) {
    ca_.Bind(&block1);
    tmp6 = Undefined_0(state_);
    CodeStubAssembler(state_).Return(tmp6);
  }

  TNode<Int32T> tmp7;
  TNode<BoolT> tmp8;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp7 = FromConstexpr_int32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp8 = CodeStubAssembler(state_).Word32Equal(TNode<Int32T>{tmp3}, TNode<Int32T>{tmp7});
    ca_.Branch(tmp8, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp9;
  TNode<Union<HeapObject, TaggedIndex>> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<RawPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<Union<HeapObject, TaggedIndex>> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<Uint32T> tmp16;
  TNode<Uint32T> tmp17;
  TNode<BoolT> tmp18;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp9 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferSigRepresentationArray);
    std::tie(tmp10, tmp11) = GetRefAt_RawPtr_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp9}).Flatten();
    tmp12 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{tmp10, tmp11});
    tmp13 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp14, tmp15) = GetRefAt_uint32_RawPtr_0(state_, TNode<RawPtrT>{tmp12}, TNode<IntPtrT>{tmp13}).Flatten();
    tmp16 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp14, tmp15});
    tmp17 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI32.raw_bit_field());
    tmp18 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp16}, TNode<Uint32T>{tmp17});
    ca_.Branch(tmp18, &block5, std::vector<compiler::Node*>{}, &block6, std::vector<compiler::Node*>{});
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    if ((wasm::kIsBigEndian)) {
      ca_.Goto(&block8);
    } else {
      ca_.Goto(&block9);
    }
  }

  TNode<IntPtrT> tmp19;
  TNode<Union<HeapObject, TaggedIndex>> tmp20;
  TNode<IntPtrT> tmp21;
  TNode<Int64T> tmp22;
  TNode<Int32T> tmp23;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    tmp19 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferGPReturnRegister1);
    std::tie(tmp20, tmp21) = GetRefAt_int64_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp19}).Flatten();
    tmp22 = CodeStubAssembler(state_).LoadReference<Int64T>(CodeStubAssembler::Reference{tmp20, tmp21});
    tmp23 = CodeStubAssembler(state_).TruncateInt64ToInt32(TNode<Int64T>{tmp22});
    ca_.Goto(&block10, tmp23);
  }

  TNode<IntPtrT> tmp24;
  TNode<Union<HeapObject, TaggedIndex>> tmp25;
  TNode<IntPtrT> tmp26;
  TNode<Int32T> tmp27;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp24 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferGPReturnRegister1);
    std::tie(tmp25, tmp26) = GetRefAt_int32_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp24}).Flatten();
    tmp27 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp25, tmp26});
    ca_.Goto(&block10, tmp27);
  }

  TNode<Int32T> phi_bb10_6;
  TNode<Number> tmp28;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_6);
    tmp28 = Convert_Number_int32_0(state_, TNode<Int32T>{phi_bb10_6});
    CodeStubAssembler(state_).Return(tmp28);
  }

  TNode<Uint32T> tmp29;
  TNode<BoolT> tmp30;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp29 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF32.raw_bit_field());
    tmp30 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp16}, TNode<Uint32T>{tmp29});
    ca_.Branch(tmp30, &block11, std::vector<compiler::Node*>{}, &block12, std::vector<compiler::Node*>{});
  }

  if (block11.is_used()) {
    ca_.Bind(&block11);
    if ((wasm::kIsFpAlwaysDouble)) {
      ca_.Goto(&block14);
    } else {
      ca_.Goto(&block15);
    }
  }

  TNode<IntPtrT> tmp31;
  TNode<Union<HeapObject, TaggedIndex>> tmp32;
  TNode<IntPtrT> tmp33;
  TNode<Float64T> tmp34;
  TNode<Float32T> tmp35;
  TNode<Number> tmp36;
  if (block14.is_used()) {
    ca_.Bind(&block14);
    tmp31 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferFPReturnRegister1);
    std::tie(tmp32, tmp33) = GetRefAt_float64_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp31}).Flatten();
    tmp34 = CodeStubAssembler(state_).LoadReference<Float64T>(CodeStubAssembler::Reference{tmp32, tmp33});
    tmp35 = CodeStubAssembler(state_).TruncateFloat64ToFloat32(TNode<Float64T>{tmp34});
    tmp36 = Convert_Number_float32_0(state_, TNode<Float32T>{tmp35});
    CodeStubAssembler(state_).Return(tmp36);
  }

  if (block15.is_used()) {
    ca_.Bind(&block15);
    if ((wasm::kIsBigEndianOnSim)) {
      ca_.Goto(&block17);
    } else {
      ca_.Goto(&block18);
    }
  }

  TNode<IntPtrT> tmp37;
  TNode<Union<HeapObject, TaggedIndex>> tmp38;
  TNode<IntPtrT> tmp39;
  TNode<Int64T> tmp40;
  TNode<Int64T> tmp41;
  TNode<Int64T> tmp42;
  TNode<Int32T> tmp43;
  TNode<Float32T> tmp44;
  TNode<Number> tmp45;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp37 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferFPReturnRegister1);
    std::tie(tmp38, tmp39) = GetRefAt_int64_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp37}).Flatten();
    tmp40 = CodeStubAssembler(state_).LoadReference<Int64T>(CodeStubAssembler::Reference{tmp38, tmp39});
    tmp41 = FromConstexpr_int64_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x20ull));
    tmp42 = CodeStubAssembler(state_).Word64Sar(TNode<Int64T>{tmp40}, TNode<Int64T>{tmp41});
    tmp43 = CodeStubAssembler(state_).TruncateInt64ToInt32(TNode<Int64T>{tmp42});
    tmp44 = CodeStubAssembler(state_).BitcastInt32ToFloat32(TNode<Int32T>{tmp43});
    tmp45 = Convert_Number_float32_0(state_, TNode<Float32T>{tmp44});
    CodeStubAssembler(state_).Return(tmp45);
  }

  TNode<IntPtrT> tmp46;
  TNode<Union<HeapObject, TaggedIndex>> tmp47;
  TNode<IntPtrT> tmp48;
  TNode<Float32T> tmp49;
  TNode<Number> tmp50;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp46 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferFPReturnRegister1);
    std::tie(tmp47, tmp48) = GetRefAt_float32_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp46}).Flatten();
    tmp49 = CodeStubAssembler(state_).LoadReference<Float32T>(CodeStubAssembler::Reference{tmp47, tmp48});
    tmp50 = Convert_Number_float32_0(state_, TNode<Float32T>{tmp49});
    CodeStubAssembler(state_).Return(tmp50);
  }

  TNode<Uint32T> tmp51;
  TNode<BoolT> tmp52;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp51 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF64.raw_bit_field());
    tmp52 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp16}, TNode<Uint32T>{tmp51});
    ca_.Branch(tmp52, &block20, std::vector<compiler::Node*>{}, &block21, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp53;
  TNode<Union<HeapObject, TaggedIndex>> tmp54;
  TNode<IntPtrT> tmp55;
  TNode<Float64T> tmp56;
  TNode<Number> tmp57;
  if (block20.is_used()) {
    ca_.Bind(&block20);
    tmp53 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferFPReturnRegister1);
    std::tie(tmp54, tmp55) = GetRefAt_float64_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp53}).Flatten();
    tmp56 = CodeStubAssembler(state_).LoadReference<Float64T>(CodeStubAssembler::Reference{tmp54, tmp55});
    tmp57 = Convert_Number_float64_0(state_, TNode<Float64T>{tmp56});
    CodeStubAssembler(state_).Return(tmp57);
  }

  TNode<Uint32T> tmp58;
  TNode<BoolT> tmp59;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp58 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI64.raw_bit_field());
    tmp59 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp16}, TNode<Uint32T>{tmp58});
    ca_.Branch(tmp59, &block23, std::vector<compiler::Node*>{}, &block24, std::vector<compiler::Node*>{});
  }

  if (block23.is_used()) {
    ca_.Bind(&block23);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block26);
    } else {
      ca_.Goto(&block27);
    }
  }

  TNode<IntPtrT> tmp60;
  TNode<Union<HeapObject, TaggedIndex>> tmp61;
  TNode<IntPtrT> tmp62;
  TNode<IntPtrT> tmp63;
  TNode<BigInt> tmp64;
  if (block26.is_used()) {
    ca_.Bind(&block26);
    tmp60 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferGPReturnRegister1);
    std::tie(tmp61, tmp62) = GetRefAt_intptr_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp60}).Flatten();
    tmp63 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp61, tmp62});
    tmp64 = ca_.CallBuiltin<BigInt>(Builtin::kI64ToBigInt, TNode<Object>(), tmp63);
    CodeStubAssembler(state_).Return(tmp64);
  }

  TNode<IntPtrT> tmp65;
  TNode<Union<HeapObject, TaggedIndex>> tmp66;
  TNode<IntPtrT> tmp67;
  TNode<IntPtrT> tmp68;
  TNode<IntPtrT> tmp69;
  TNode<Union<HeapObject, TaggedIndex>> tmp70;
  TNode<IntPtrT> tmp71;
  TNode<IntPtrT> tmp72;
  TNode<BigInt> tmp73;
  if (block27.is_used()) {
    ca_.Bind(&block27);
    tmp65 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferGPReturnRegister1);
    std::tie(tmp66, tmp67) = GetRefAt_intptr_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp65}).Flatten();
    tmp68 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp66, tmp67});
    tmp69 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferGPReturnRegister2);
    std::tie(tmp70, tmp71) = GetRefAt_intptr_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp69}).Flatten();
    tmp72 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp70, tmp71});
    tmp73 = ca_.CallBuiltin<BigInt>(Builtin::kI32PairToBigInt, TNode<Object>(), tmp68, tmp72);
    CodeStubAssembler(state_).Return(tmp73);
  }

  TNode<Uint32T> tmp74;
  TNode<Uint32T> tmp75;
  TNode<Uint32T> tmp76;
  TNode<BoolT> tmp77;
  if (block24.is_used()) {
    ca_.Bind(&block24);
    tmp74 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp75 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp16}, TNode<Uint32T>{tmp74});
    tmp76 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp77 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp75}, TNode<Uint32T>{tmp76});
    ca_.Branch(tmp77, &block29, std::vector<compiler::Node*>{}, &block30, std::vector<compiler::Node*>{});
  }

  if (block30.is_used()) {
    ca_.Bind(&block30);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/js-to-wasm.tq", 935});
      CodeStubAssembler(state_).FailAssert("Torque assert '(retType & kValueTypeIsRefBit) != 0' failed", pos_stack);
    }
  }

  TNode<IntPtrT> tmp78;
  TNode<RawPtrT> tmp79;
  TNode<RawPtrT> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<Union<HeapObject, TaggedIndex>> tmp82;
  TNode<IntPtrT> tmp83;
  TNode<UintPtrT> tmp84;
  TNode<Object> tmp85;
  TNode<JSAny> tmp86;
  if (block29.is_used()) {
    ca_.Bind(&block29);
    tmp78 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferGPReturnRegister1);
    tmp79 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp78});
    tmp80 = (TNode<RawPtrT>{tmp79});
    tmp81 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp82, tmp83) = GetRefAt_uintptr_RawPtr_uintptr_0(state_, TNode<RawPtrT>{tmp80}, TNode<IntPtrT>{tmp81}).Flatten();
    tmp84 = CodeStubAssembler(state_).LoadReference<UintPtrT>(CodeStubAssembler::Reference{tmp82, tmp83});
    tmp85 = CodeStubAssembler(state_).BitcastWordToTagged(TNode<UintPtrT>{tmp84});
    tmp86 = WasmToJSObject_0(state_, TNode<NativeContext>{parameter0}, TNode<Object>{tmp85});
    CodeStubAssembler(state_).Return(tmp86);
  }

  TNode<IntPtrT> tmp87;
  TNode<FixedArrayBase> tmp88;
  TNode<FixedArray> tmp89;
  TNode<IntPtrT> tmp90;
  TNode<Union<HeapObject, TaggedIndex>> tmp91;
  TNode<IntPtrT> tmp92;
  TNode<RawPtrT> tmp93;
  TNode<IntPtrT> tmp94;
  TNode<RawPtrT> tmp95;
  TNode<IntPtrT> tmp96;
  TNode<RawPtrT> tmp97;
  TNode<Union<HeapObject, TaggedIndex>> tmp98;
  TNode<IntPtrT> tmp99;
  TNode<IntPtrT> tmp100;
  TNode<IntPtrT> tmp101;
  TNode<IntPtrT> tmp102;
  TNode<IntPtrT> tmp103;
  TNode<IntPtrT> tmp104;
  TNode<IntPtrT> tmp105;
  TNode<IntPtrT> tmp106;
  TNode<BoolT> tmp107;
  TNode<IntPtrT> tmp108;
  TNode<Union<HeapObject, TaggedIndex>> tmp109;
  TNode<IntPtrT> tmp110;
  TNode<RawPtrT> tmp111;
  TNode<RawPtrT> tmp112;
  TNode<IntPtrT> tmp113;
  TNode<Union<HeapObject, TaggedIndex>> tmp114;
  TNode<IntPtrT> tmp115;
  TNode<IntPtrT> tmp116;
  TNode<IntPtrT> tmp117;
  TNode<Union<HeapObject, TaggedIndex>> tmp118;
  TNode<IntPtrT> tmp119;
  TNode<BoolT> tmp120;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp87 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp88 = CodeStubAssembler(state_).LoadReference<FixedArrayBase>(CodeStubAssembler::Reference{parameter1, tmp87});
    tmp89 = TORQUE_CAST(TNode<Object>{tmp88});
    tmp90 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferStackReturnBufferStart);
    std::tie(tmp91, tmp92) = GetRefAt_RawPtr_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp90}).Flatten();
    tmp93 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{tmp91, tmp92});
    tmp94 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferGPReturnRegister1);
    tmp95 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp94});
    tmp96 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferFPReturnRegister1);
    tmp97 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp96});
    std::tie(tmp98, tmp99, tmp100, tmp101, tmp102, tmp103, tmp104, tmp105, tmp106, tmp107) = LocationAllocatorForReturns_0(state_, TNode<RawPtrT>{tmp95}, TNode<RawPtrT>{tmp97}, TNode<RawPtrT>{tmp93}).Flatten();
    tmp108 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferSigRepresentationArray);
    std::tie(tmp109, tmp110) = GetRefAt_RawPtr_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp108}).Flatten();
    tmp111 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{tmp109, tmp110});
    tmp112 = (TNode<RawPtrT>{tmp111});
    tmp113 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp3});
    std::tie(tmp114, tmp115, tmp116) = NewOffHeapConstSlice_uint32_0(state_, TNode<RawPtrT>{tmp112}, TNode<IntPtrT>{tmp113}).Flatten();
    tmp117 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferRefReturnCount);
    std::tie(tmp118, tmp119) = GetRefAt_bool_RawPtr_intptr_0(state_, TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp117}).Flatten();
    tmp120 = CodeStubAssembler(state_).LoadReference<BoolT>(CodeStubAssembler::Reference{tmp118, tmp119});
    ca_.Branch(tmp120, &block44, std::vector<compiler::Node*>{}, &block45, std::vector<compiler::Node*>{tmp99, tmp100, tmp101, tmp102, tmp103, tmp106, tmp107});
  }

  TNode<IntPtrT> tmp121;
  if (block44.is_used()) {
    ca_.Bind(&block44);
    tmp121 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block48, tmp99, tmp100, tmp101, tmp102, tmp103, tmp106, tmp107, tmp121);
  }

  TNode<IntPtrT> phi_bb48_7;
  TNode<IntPtrT> phi_bb48_8;
  TNode<IntPtrT> phi_bb48_9;
  TNode<IntPtrT> phi_bb48_10;
  TNode<IntPtrT> phi_bb48_11;
  TNode<IntPtrT> phi_bb48_14;
  TNode<BoolT> phi_bb48_15;
  TNode<IntPtrT> phi_bb48_21;
  TNode<IntPtrT> tmp122;
  TNode<BoolT> tmp123;
  if (block48.is_used()) {
    ca_.Bind(&block48, &phi_bb48_7, &phi_bb48_8, &phi_bb48_9, &phi_bb48_10, &phi_bb48_11, &phi_bb48_14, &phi_bb48_15, &phi_bb48_21);
    tmp122 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp3});
    tmp123 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb48_21}, TNode<IntPtrT>{tmp122});
    ca_.Branch(tmp123, &block46, std::vector<compiler::Node*>{phi_bb48_7, phi_bb48_8, phi_bb48_9, phi_bb48_10, phi_bb48_11, phi_bb48_14, phi_bb48_15, phi_bb48_21}, &block47, std::vector<compiler::Node*>{phi_bb48_7, phi_bb48_8, phi_bb48_9, phi_bb48_10, phi_bb48_11, phi_bb48_14, phi_bb48_15, phi_bb48_21});
  }

  TNode<IntPtrT> phi_bb46_7;
  TNode<IntPtrT> phi_bb46_8;
  TNode<IntPtrT> phi_bb46_9;
  TNode<IntPtrT> phi_bb46_10;
  TNode<IntPtrT> phi_bb46_11;
  TNode<IntPtrT> phi_bb46_14;
  TNode<BoolT> phi_bb46_15;
  TNode<IntPtrT> phi_bb46_21;
  TNode<IntPtrT> tmp124;
  TNode<IntPtrT> tmp125;
  TNode<Union<HeapObject, TaggedIndex>> tmp126;
  TNode<IntPtrT> tmp127;
  TNode<Uint32T> tmp128;
  TNode<Uint32T> tmp129;
  TNode<BoolT> tmp130;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_7, &phi_bb46_8, &phi_bb46_9, &phi_bb46_10, &phi_bb46_11, &phi_bb46_14, &phi_bb46_15, &phi_bb46_21);
    tmp124 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{phi_bb46_21});
    tmp125 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp115}, TNode<IntPtrT>{tmp124});
    std::tie(tmp126, tmp127) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp114}, TNode<IntPtrT>{tmp125}).Flatten();
    tmp128 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp126, tmp127});
    tmp129 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI32.raw_bit_field());
    tmp130 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp128}, TNode<Uint32T>{tmp129});
    ca_.Branch(tmp130, &block51, std::vector<compiler::Node*>{phi_bb46_7, phi_bb46_8, phi_bb46_9, phi_bb46_10, phi_bb46_11, phi_bb46_14, phi_bb46_15, phi_bb46_21}, &block52, std::vector<compiler::Node*>{phi_bb46_7, phi_bb46_8, phi_bb46_9, phi_bb46_10, phi_bb46_11, phi_bb46_14, phi_bb46_15, phi_bb46_21});
  }

  TNode<IntPtrT> phi_bb51_7;
  TNode<IntPtrT> phi_bb51_8;
  TNode<IntPtrT> phi_bb51_9;
  TNode<IntPtrT> phi_bb51_10;
  TNode<IntPtrT> phi_bb51_11;
  TNode<IntPtrT> phi_bb51_14;
  TNode<BoolT> phi_bb51_15;
  TNode<IntPtrT> phi_bb51_21;
  TNode<IntPtrT> tmp131;
  TNode<IntPtrT> tmp132;
  TNode<IntPtrT> tmp133;
  TNode<BoolT> tmp134;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_7, &phi_bb51_8, &phi_bb51_9, &phi_bb51_10, &phi_bb51_11, &phi_bb51_14, &phi_bb51_15, &phi_bb51_21);
    tmp131 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp132 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb51_7}, TNode<IntPtrT>{tmp131});
    tmp133 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp134 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb51_7}, TNode<IntPtrT>{tmp133});
    ca_.Branch(tmp134, &block55, std::vector<compiler::Node*>{phi_bb51_8, phi_bb51_9, phi_bb51_10, phi_bb51_11, phi_bb51_14, phi_bb51_15, phi_bb51_21}, &block56, std::vector<compiler::Node*>{phi_bb51_8, phi_bb51_9, phi_bb51_10, phi_bb51_11, phi_bb51_14, phi_bb51_15, phi_bb51_21});
  }

  TNode<IntPtrT> phi_bb55_8;
  TNode<IntPtrT> phi_bb55_9;
  TNode<IntPtrT> phi_bb55_10;
  TNode<IntPtrT> phi_bb55_11;
  TNode<IntPtrT> phi_bb55_14;
  TNode<BoolT> phi_bb55_15;
  TNode<IntPtrT> phi_bb55_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp135;
  TNode<IntPtrT> tmp136;
  TNode<IntPtrT> tmp137;
  TNode<IntPtrT> tmp138;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_8, &phi_bb55_9, &phi_bb55_10, &phi_bb55_11, &phi_bb55_14, &phi_bb55_15, &phi_bb55_21);
    std::tie(tmp135, tmp136) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb55_9}).Flatten();
    tmp137 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp138 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb55_9}, TNode<IntPtrT>{tmp137});
    ca_.Goto(&block54, phi_bb55_8, tmp138, phi_bb55_10, phi_bb55_11, phi_bb55_14, phi_bb55_15, phi_bb55_21, tmp135, tmp136);
  }

  TNode<IntPtrT> phi_bb56_8;
  TNode<IntPtrT> phi_bb56_9;
  TNode<IntPtrT> phi_bb56_10;
  TNode<IntPtrT> phi_bb56_11;
  TNode<IntPtrT> phi_bb56_14;
  TNode<BoolT> phi_bb56_15;
  TNode<IntPtrT> phi_bb56_21;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_8, &phi_bb56_9, &phi_bb56_10, &phi_bb56_11, &phi_bb56_14, &phi_bb56_15, &phi_bb56_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block58, phi_bb56_8, phi_bb56_9, phi_bb56_10, phi_bb56_11, phi_bb56_14, phi_bb56_15, phi_bb56_21);
    } else {
      ca_.Goto(&block59, phi_bb56_8, phi_bb56_9, phi_bb56_10, phi_bb56_11, phi_bb56_14, phi_bb56_15, phi_bb56_21);
    }
  }

  TNode<IntPtrT> phi_bb58_8;
  TNode<IntPtrT> phi_bb58_9;
  TNode<IntPtrT> phi_bb58_10;
  TNode<IntPtrT> phi_bb58_11;
  TNode<IntPtrT> phi_bb58_14;
  TNode<BoolT> phi_bb58_15;
  TNode<IntPtrT> phi_bb58_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp139;
  TNode<IntPtrT> tmp140;
  TNode<IntPtrT> tmp141;
  TNode<IntPtrT> tmp142;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_8, &phi_bb58_9, &phi_bb58_10, &phi_bb58_11, &phi_bb58_14, &phi_bb58_15, &phi_bb58_21);
    std::tie(tmp139, tmp140) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb58_11}).Flatten();
    tmp141 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp142 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb58_11}, TNode<IntPtrT>{tmp141});
    ca_.Goto(&block57, phi_bb58_8, phi_bb58_9, phi_bb58_10, tmp142, phi_bb58_14, phi_bb58_15, phi_bb58_21, tmp139, tmp140);
  }

  TNode<IntPtrT> phi_bb59_8;
  TNode<IntPtrT> phi_bb59_9;
  TNode<IntPtrT> phi_bb59_10;
  TNode<IntPtrT> phi_bb59_11;
  TNode<IntPtrT> phi_bb59_14;
  TNode<BoolT> phi_bb59_15;
  TNode<IntPtrT> phi_bb59_21;
  TNode<IntPtrT> tmp143;
  TNode<BoolT> tmp144;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_8, &phi_bb59_9, &phi_bb59_10, &phi_bb59_11, &phi_bb59_14, &phi_bb59_15, &phi_bb59_21);
    tmp143 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp144 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb59_14}, TNode<IntPtrT>{tmp143});
    ca_.Branch(tmp144, &block61, std::vector<compiler::Node*>{phi_bb59_8, phi_bb59_9, phi_bb59_10, phi_bb59_11, phi_bb59_14, phi_bb59_15, phi_bb59_21}, &block62, std::vector<compiler::Node*>{phi_bb59_8, phi_bb59_9, phi_bb59_10, phi_bb59_11, phi_bb59_14, phi_bb59_15, phi_bb59_21});
  }

  TNode<IntPtrT> phi_bb61_8;
  TNode<IntPtrT> phi_bb61_9;
  TNode<IntPtrT> phi_bb61_10;
  TNode<IntPtrT> phi_bb61_11;
  TNode<IntPtrT> phi_bb61_14;
  TNode<BoolT> phi_bb61_15;
  TNode<IntPtrT> phi_bb61_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp145;
  TNode<IntPtrT> tmp146;
  TNode<IntPtrT> tmp147;
  TNode<BoolT> tmp148;
  if (block61.is_used()) {
    ca_.Bind(&block61, &phi_bb61_8, &phi_bb61_9, &phi_bb61_10, &phi_bb61_11, &phi_bb61_14, &phi_bb61_15, &phi_bb61_21);
    std::tie(tmp145, tmp146) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb61_14}).Flatten();
    tmp147 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp148 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block57, phi_bb61_8, phi_bb61_9, phi_bb61_10, phi_bb61_11, tmp147, tmp148, phi_bb61_21, tmp145, tmp146);
  }

  TNode<IntPtrT> phi_bb62_8;
  TNode<IntPtrT> phi_bb62_9;
  TNode<IntPtrT> phi_bb62_10;
  TNode<IntPtrT> phi_bb62_11;
  TNode<IntPtrT> phi_bb62_14;
  TNode<BoolT> phi_bb62_15;
  TNode<IntPtrT> phi_bb62_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp149;
  TNode<IntPtrT> tmp150;
  TNode<IntPtrT> tmp151;
  TNode<IntPtrT> tmp152;
  TNode<IntPtrT> tmp153;
  TNode<IntPtrT> tmp154;
  TNode<BoolT> tmp155;
  if (block62.is_used()) {
    ca_.Bind(&block62, &phi_bb62_8, &phi_bb62_9, &phi_bb62_10, &phi_bb62_11, &phi_bb62_14, &phi_bb62_15, &phi_bb62_21);
    std::tie(tmp149, tmp150) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb62_11}).Flatten();
    tmp151 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp152 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb62_11}, TNode<IntPtrT>{tmp151});
    tmp153 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp154 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp152}, TNode<IntPtrT>{tmp153});
    tmp155 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block57, phi_bb62_8, phi_bb62_9, phi_bb62_10, tmp154, tmp152, tmp155, phi_bb62_21, tmp149, tmp150);
  }

  TNode<IntPtrT> phi_bb57_8;
  TNode<IntPtrT> phi_bb57_9;
  TNode<IntPtrT> phi_bb57_10;
  TNode<IntPtrT> phi_bb57_11;
  TNode<IntPtrT> phi_bb57_14;
  TNode<BoolT> phi_bb57_15;
  TNode<IntPtrT> phi_bb57_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb57_23;
  TNode<IntPtrT> phi_bb57_24;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_8, &phi_bb57_9, &phi_bb57_10, &phi_bb57_11, &phi_bb57_14, &phi_bb57_15, &phi_bb57_21, &phi_bb57_23, &phi_bb57_24);
    ca_.Goto(&block54, phi_bb57_8, phi_bb57_9, phi_bb57_10, phi_bb57_11, phi_bb57_14, phi_bb57_15, phi_bb57_21, phi_bb57_23, phi_bb57_24);
  }

  TNode<IntPtrT> phi_bb54_8;
  TNode<IntPtrT> phi_bb54_9;
  TNode<IntPtrT> phi_bb54_10;
  TNode<IntPtrT> phi_bb54_11;
  TNode<IntPtrT> phi_bb54_14;
  TNode<BoolT> phi_bb54_15;
  TNode<IntPtrT> phi_bb54_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb54_23;
  TNode<IntPtrT> phi_bb54_24;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_8, &phi_bb54_9, &phi_bb54_10, &phi_bb54_11, &phi_bb54_14, &phi_bb54_15, &phi_bb54_21, &phi_bb54_23, &phi_bb54_24);
    ca_.Goto(&block53, tmp132, phi_bb54_8, phi_bb54_9, phi_bb54_10, phi_bb54_11, phi_bb54_14, phi_bb54_15, phi_bb54_21);
  }

  TNode<IntPtrT> phi_bb52_7;
  TNode<IntPtrT> phi_bb52_8;
  TNode<IntPtrT> phi_bb52_9;
  TNode<IntPtrT> phi_bb52_10;
  TNode<IntPtrT> phi_bb52_11;
  TNode<IntPtrT> phi_bb52_14;
  TNode<BoolT> phi_bb52_15;
  TNode<IntPtrT> phi_bb52_21;
  TNode<Uint32T> tmp156;
  TNode<BoolT> tmp157;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_7, &phi_bb52_8, &phi_bb52_9, &phi_bb52_10, &phi_bb52_11, &phi_bb52_14, &phi_bb52_15, &phi_bb52_21);
    tmp156 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF32.raw_bit_field());
    tmp157 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp128}, TNode<Uint32T>{tmp156});
    ca_.Branch(tmp157, &block63, std::vector<compiler::Node*>{phi_bb52_7, phi_bb52_8, phi_bb52_9, phi_bb52_10, phi_bb52_11, phi_bb52_14, phi_bb52_15, phi_bb52_21}, &block64, std::vector<compiler::Node*>{phi_bb52_7, phi_bb52_8, phi_bb52_9, phi_bb52_10, phi_bb52_11, phi_bb52_14, phi_bb52_15, phi_bb52_21});
  }

  TNode<IntPtrT> phi_bb63_7;
  TNode<IntPtrT> phi_bb63_8;
  TNode<IntPtrT> phi_bb63_9;
  TNode<IntPtrT> phi_bb63_10;
  TNode<IntPtrT> phi_bb63_11;
  TNode<IntPtrT> phi_bb63_14;
  TNode<BoolT> phi_bb63_15;
  TNode<IntPtrT> phi_bb63_21;
  TNode<IntPtrT> tmp158;
  TNode<IntPtrT> tmp159;
  TNode<IntPtrT> tmp160;
  TNode<BoolT> tmp161;
  if (block63.is_used()) {
    ca_.Bind(&block63, &phi_bb63_7, &phi_bb63_8, &phi_bb63_9, &phi_bb63_10, &phi_bb63_11, &phi_bb63_14, &phi_bb63_15, &phi_bb63_21);
    tmp158 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp159 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb63_8}, TNode<IntPtrT>{tmp158});
    tmp160 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp161 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb63_8}, TNode<IntPtrT>{tmp160});
    ca_.Branch(tmp161, &block67, std::vector<compiler::Node*>{phi_bb63_7, phi_bb63_9, phi_bb63_10, phi_bb63_11, phi_bb63_14, phi_bb63_15, phi_bb63_21}, &block68, std::vector<compiler::Node*>{phi_bb63_7, phi_bb63_9, phi_bb63_10, phi_bb63_11, phi_bb63_14, phi_bb63_15, phi_bb63_21});
  }

  TNode<IntPtrT> phi_bb67_7;
  TNode<IntPtrT> phi_bb67_9;
  TNode<IntPtrT> phi_bb67_10;
  TNode<IntPtrT> phi_bb67_11;
  TNode<IntPtrT> phi_bb67_14;
  TNode<BoolT> phi_bb67_15;
  TNode<IntPtrT> phi_bb67_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp162;
  TNode<IntPtrT> tmp163;
  TNode<IntPtrT> tmp164;
  TNode<IntPtrT> tmp165;
  if (block67.is_used()) {
    ca_.Bind(&block67, &phi_bb67_7, &phi_bb67_9, &phi_bb67_10, &phi_bb67_11, &phi_bb67_14, &phi_bb67_15, &phi_bb67_21);
    std::tie(tmp162, tmp163) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb67_10}).Flatten();
    tmp164 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp165 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb67_10}, TNode<IntPtrT>{tmp164});
    ca_.Goto(&block66, phi_bb67_7, phi_bb67_9, tmp165, phi_bb67_11, phi_bb67_14, phi_bb67_15, phi_bb67_21, tmp162, tmp163);
  }

  TNode<IntPtrT> phi_bb68_7;
  TNode<IntPtrT> phi_bb68_9;
  TNode<IntPtrT> phi_bb68_10;
  TNode<IntPtrT> phi_bb68_11;
  TNode<IntPtrT> phi_bb68_14;
  TNode<BoolT> phi_bb68_15;
  TNode<IntPtrT> phi_bb68_21;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_7, &phi_bb68_9, &phi_bb68_10, &phi_bb68_11, &phi_bb68_14, &phi_bb68_15, &phi_bb68_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block70, phi_bb68_7, phi_bb68_9, phi_bb68_10, phi_bb68_11, phi_bb68_14, phi_bb68_15, phi_bb68_21);
    } else {
      ca_.Goto(&block71, phi_bb68_7, phi_bb68_9, phi_bb68_10, phi_bb68_11, phi_bb68_14, phi_bb68_15, phi_bb68_21);
    }
  }

  TNode<IntPtrT> phi_bb70_7;
  TNode<IntPtrT> phi_bb70_9;
  TNode<IntPtrT> phi_bb70_10;
  TNode<IntPtrT> phi_bb70_11;
  TNode<IntPtrT> phi_bb70_14;
  TNode<BoolT> phi_bb70_15;
  TNode<IntPtrT> phi_bb70_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp166;
  TNode<IntPtrT> tmp167;
  TNode<IntPtrT> tmp168;
  TNode<IntPtrT> tmp169;
  if (block70.is_used()) {
    ca_.Bind(&block70, &phi_bb70_7, &phi_bb70_9, &phi_bb70_10, &phi_bb70_11, &phi_bb70_14, &phi_bb70_15, &phi_bb70_21);
    std::tie(tmp166, tmp167) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb70_11}).Flatten();
    tmp168 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp169 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb70_11}, TNode<IntPtrT>{tmp168});
    ca_.Goto(&block69, phi_bb70_7, phi_bb70_9, phi_bb70_10, tmp169, phi_bb70_14, phi_bb70_15, phi_bb70_21, tmp166, tmp167);
  }

  TNode<IntPtrT> phi_bb71_7;
  TNode<IntPtrT> phi_bb71_9;
  TNode<IntPtrT> phi_bb71_10;
  TNode<IntPtrT> phi_bb71_11;
  TNode<IntPtrT> phi_bb71_14;
  TNode<BoolT> phi_bb71_15;
  TNode<IntPtrT> phi_bb71_21;
  TNode<IntPtrT> tmp170;
  TNode<BoolT> tmp171;
  if (block71.is_used()) {
    ca_.Bind(&block71, &phi_bb71_7, &phi_bb71_9, &phi_bb71_10, &phi_bb71_11, &phi_bb71_14, &phi_bb71_15, &phi_bb71_21);
    tmp170 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp171 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb71_14}, TNode<IntPtrT>{tmp170});
    ca_.Branch(tmp171, &block73, std::vector<compiler::Node*>{phi_bb71_7, phi_bb71_9, phi_bb71_10, phi_bb71_11, phi_bb71_14, phi_bb71_15, phi_bb71_21}, &block74, std::vector<compiler::Node*>{phi_bb71_7, phi_bb71_9, phi_bb71_10, phi_bb71_11, phi_bb71_14, phi_bb71_15, phi_bb71_21});
  }

  TNode<IntPtrT> phi_bb73_7;
  TNode<IntPtrT> phi_bb73_9;
  TNode<IntPtrT> phi_bb73_10;
  TNode<IntPtrT> phi_bb73_11;
  TNode<IntPtrT> phi_bb73_14;
  TNode<BoolT> phi_bb73_15;
  TNode<IntPtrT> phi_bb73_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp172;
  TNode<IntPtrT> tmp173;
  TNode<IntPtrT> tmp174;
  TNode<BoolT> tmp175;
  if (block73.is_used()) {
    ca_.Bind(&block73, &phi_bb73_7, &phi_bb73_9, &phi_bb73_10, &phi_bb73_11, &phi_bb73_14, &phi_bb73_15, &phi_bb73_21);
    std::tie(tmp172, tmp173) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb73_14}).Flatten();
    tmp174 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp175 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block69, phi_bb73_7, phi_bb73_9, phi_bb73_10, phi_bb73_11, tmp174, tmp175, phi_bb73_21, tmp172, tmp173);
  }

  TNode<IntPtrT> phi_bb74_7;
  TNode<IntPtrT> phi_bb74_9;
  TNode<IntPtrT> phi_bb74_10;
  TNode<IntPtrT> phi_bb74_11;
  TNode<IntPtrT> phi_bb74_14;
  TNode<BoolT> phi_bb74_15;
  TNode<IntPtrT> phi_bb74_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp176;
  TNode<IntPtrT> tmp177;
  TNode<IntPtrT> tmp178;
  TNode<IntPtrT> tmp179;
  TNode<IntPtrT> tmp180;
  TNode<IntPtrT> tmp181;
  TNode<BoolT> tmp182;
  if (block74.is_used()) {
    ca_.Bind(&block74, &phi_bb74_7, &phi_bb74_9, &phi_bb74_10, &phi_bb74_11, &phi_bb74_14, &phi_bb74_15, &phi_bb74_21);
    std::tie(tmp176, tmp177) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb74_11}).Flatten();
    tmp178 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp179 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb74_11}, TNode<IntPtrT>{tmp178});
    tmp180 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp181 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp179}, TNode<IntPtrT>{tmp180});
    tmp182 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block69, phi_bb74_7, phi_bb74_9, phi_bb74_10, tmp181, tmp179, tmp182, phi_bb74_21, tmp176, tmp177);
  }

  TNode<IntPtrT> phi_bb69_7;
  TNode<IntPtrT> phi_bb69_9;
  TNode<IntPtrT> phi_bb69_10;
  TNode<IntPtrT> phi_bb69_11;
  TNode<IntPtrT> phi_bb69_14;
  TNode<BoolT> phi_bb69_15;
  TNode<IntPtrT> phi_bb69_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb69_23;
  TNode<IntPtrT> phi_bb69_24;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_7, &phi_bb69_9, &phi_bb69_10, &phi_bb69_11, &phi_bb69_14, &phi_bb69_15, &phi_bb69_21, &phi_bb69_23, &phi_bb69_24);
    ca_.Goto(&block66, phi_bb69_7, phi_bb69_9, phi_bb69_10, phi_bb69_11, phi_bb69_14, phi_bb69_15, phi_bb69_21, phi_bb69_23, phi_bb69_24);
  }

  TNode<IntPtrT> phi_bb66_7;
  TNode<IntPtrT> phi_bb66_9;
  TNode<IntPtrT> phi_bb66_10;
  TNode<IntPtrT> phi_bb66_11;
  TNode<IntPtrT> phi_bb66_14;
  TNode<BoolT> phi_bb66_15;
  TNode<IntPtrT> phi_bb66_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb66_23;
  TNode<IntPtrT> phi_bb66_24;
  if (block66.is_used()) {
    ca_.Bind(&block66, &phi_bb66_7, &phi_bb66_9, &phi_bb66_10, &phi_bb66_11, &phi_bb66_14, &phi_bb66_15, &phi_bb66_21, &phi_bb66_23, &phi_bb66_24);
    ca_.Goto(&block65, phi_bb66_7, tmp159, phi_bb66_9, phi_bb66_10, phi_bb66_11, phi_bb66_14, phi_bb66_15, phi_bb66_21);
  }

  TNode<IntPtrT> phi_bb64_7;
  TNode<IntPtrT> phi_bb64_8;
  TNode<IntPtrT> phi_bb64_9;
  TNode<IntPtrT> phi_bb64_10;
  TNode<IntPtrT> phi_bb64_11;
  TNode<IntPtrT> phi_bb64_14;
  TNode<BoolT> phi_bb64_15;
  TNode<IntPtrT> phi_bb64_21;
  TNode<Uint32T> tmp183;
  TNode<BoolT> tmp184;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_7, &phi_bb64_8, &phi_bb64_9, &phi_bb64_10, &phi_bb64_11, &phi_bb64_14, &phi_bb64_15, &phi_bb64_21);
    tmp183 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI64.raw_bit_field());
    tmp184 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp128}, TNode<Uint32T>{tmp183});
    ca_.Branch(tmp184, &block75, std::vector<compiler::Node*>{phi_bb64_7, phi_bb64_8, phi_bb64_9, phi_bb64_10, phi_bb64_11, phi_bb64_14, phi_bb64_15, phi_bb64_21}, &block76, std::vector<compiler::Node*>{phi_bb64_7, phi_bb64_8, phi_bb64_9, phi_bb64_10, phi_bb64_11, phi_bb64_14, phi_bb64_15, phi_bb64_21});
  }

  TNode<IntPtrT> phi_bb75_7;
  TNode<IntPtrT> phi_bb75_8;
  TNode<IntPtrT> phi_bb75_9;
  TNode<IntPtrT> phi_bb75_10;
  TNode<IntPtrT> phi_bb75_11;
  TNode<IntPtrT> phi_bb75_14;
  TNode<BoolT> phi_bb75_15;
  TNode<IntPtrT> phi_bb75_21;
  TNode<IntPtrT> tmp185;
  TNode<IntPtrT> tmp186;
  TNode<IntPtrT> tmp187;
  TNode<BoolT> tmp188;
  if (block75.is_used()) {
    ca_.Bind(&block75, &phi_bb75_7, &phi_bb75_8, &phi_bb75_9, &phi_bb75_10, &phi_bb75_11, &phi_bb75_14, &phi_bb75_15, &phi_bb75_21);
    tmp185 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp186 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb75_7}, TNode<IntPtrT>{tmp185});
    tmp187 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp188 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb75_7}, TNode<IntPtrT>{tmp187});
    ca_.Branch(tmp188, &block79, std::vector<compiler::Node*>{phi_bb75_8, phi_bb75_9, phi_bb75_10, phi_bb75_11, phi_bb75_14, phi_bb75_15, phi_bb75_21}, &block80, std::vector<compiler::Node*>{phi_bb75_8, phi_bb75_9, phi_bb75_10, phi_bb75_11, phi_bb75_14, phi_bb75_15, phi_bb75_21});
  }

  TNode<IntPtrT> phi_bb79_8;
  TNode<IntPtrT> phi_bb79_9;
  TNode<IntPtrT> phi_bb79_10;
  TNode<IntPtrT> phi_bb79_11;
  TNode<IntPtrT> phi_bb79_14;
  TNode<BoolT> phi_bb79_15;
  TNode<IntPtrT> phi_bb79_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp189;
  TNode<IntPtrT> tmp190;
  TNode<IntPtrT> tmp191;
  TNode<IntPtrT> tmp192;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_8, &phi_bb79_9, &phi_bb79_10, &phi_bb79_11, &phi_bb79_14, &phi_bb79_15, &phi_bb79_21);
    std::tie(tmp189, tmp190) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb79_9}).Flatten();
    tmp191 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp192 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb79_9}, TNode<IntPtrT>{tmp191});
    ca_.Goto(&block78, phi_bb79_8, tmp192, phi_bb79_10, phi_bb79_11, phi_bb79_14, phi_bb79_15, phi_bb79_21, tmp189, tmp190);
  }

  TNode<IntPtrT> phi_bb80_8;
  TNode<IntPtrT> phi_bb80_9;
  TNode<IntPtrT> phi_bb80_10;
  TNode<IntPtrT> phi_bb80_11;
  TNode<IntPtrT> phi_bb80_14;
  TNode<BoolT> phi_bb80_15;
  TNode<IntPtrT> phi_bb80_21;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_8, &phi_bb80_9, &phi_bb80_10, &phi_bb80_11, &phi_bb80_14, &phi_bb80_15, &phi_bb80_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block82, phi_bb80_8, phi_bb80_9, phi_bb80_10, phi_bb80_11, phi_bb80_14, phi_bb80_15, phi_bb80_21);
    } else {
      ca_.Goto(&block83, phi_bb80_8, phi_bb80_9, phi_bb80_10, phi_bb80_11, phi_bb80_14, phi_bb80_15, phi_bb80_21);
    }
  }

  TNode<IntPtrT> phi_bb82_8;
  TNode<IntPtrT> phi_bb82_9;
  TNode<IntPtrT> phi_bb82_10;
  TNode<IntPtrT> phi_bb82_11;
  TNode<IntPtrT> phi_bb82_14;
  TNode<BoolT> phi_bb82_15;
  TNode<IntPtrT> phi_bb82_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp193;
  TNode<IntPtrT> tmp194;
  TNode<IntPtrT> tmp195;
  TNode<IntPtrT> tmp196;
  if (block82.is_used()) {
    ca_.Bind(&block82, &phi_bb82_8, &phi_bb82_9, &phi_bb82_10, &phi_bb82_11, &phi_bb82_14, &phi_bb82_15, &phi_bb82_21);
    std::tie(tmp193, tmp194) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb82_11}).Flatten();
    tmp195 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp196 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb82_11}, TNode<IntPtrT>{tmp195});
    ca_.Goto(&block81, phi_bb82_8, phi_bb82_9, phi_bb82_10, tmp196, phi_bb82_14, phi_bb82_15, phi_bb82_21, tmp193, tmp194);
  }

  TNode<IntPtrT> phi_bb83_8;
  TNode<IntPtrT> phi_bb83_9;
  TNode<IntPtrT> phi_bb83_10;
  TNode<IntPtrT> phi_bb83_11;
  TNode<IntPtrT> phi_bb83_14;
  TNode<BoolT> phi_bb83_15;
  TNode<IntPtrT> phi_bb83_21;
  TNode<IntPtrT> tmp197;
  TNode<BoolT> tmp198;
  if (block83.is_used()) {
    ca_.Bind(&block83, &phi_bb83_8, &phi_bb83_9, &phi_bb83_10, &phi_bb83_11, &phi_bb83_14, &phi_bb83_15, &phi_bb83_21);
    tmp197 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp198 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb83_14}, TNode<IntPtrT>{tmp197});
    ca_.Branch(tmp198, &block85, std::vector<compiler::Node*>{phi_bb83_8, phi_bb83_9, phi_bb83_10, phi_bb83_11, phi_bb83_14, phi_bb83_15, phi_bb83_21}, &block86, std::vector<compiler::Node*>{phi_bb83_8, phi_bb83_9, phi_bb83_10, phi_bb83_11, phi_bb83_14, phi_bb83_15, phi_bb83_21});
  }

  TNode<IntPtrT> phi_bb85_8;
  TNode<IntPtrT> phi_bb85_9;
  TNode<IntPtrT> phi_bb85_10;
  TNode<IntPtrT> phi_bb85_11;
  TNode<IntPtrT> phi_bb85_14;
  TNode<BoolT> phi_bb85_15;
  TNode<IntPtrT> phi_bb85_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp199;
  TNode<IntPtrT> tmp200;
  TNode<IntPtrT> tmp201;
  TNode<BoolT> tmp202;
  if (block85.is_used()) {
    ca_.Bind(&block85, &phi_bb85_8, &phi_bb85_9, &phi_bb85_10, &phi_bb85_11, &phi_bb85_14, &phi_bb85_15, &phi_bb85_21);
    std::tie(tmp199, tmp200) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb85_14}).Flatten();
    tmp201 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp202 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block81, phi_bb85_8, phi_bb85_9, phi_bb85_10, phi_bb85_11, tmp201, tmp202, phi_bb85_21, tmp199, tmp200);
  }

  TNode<IntPtrT> phi_bb86_8;
  TNode<IntPtrT> phi_bb86_9;
  TNode<IntPtrT> phi_bb86_10;
  TNode<IntPtrT> phi_bb86_11;
  TNode<IntPtrT> phi_bb86_14;
  TNode<BoolT> phi_bb86_15;
  TNode<IntPtrT> phi_bb86_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp203;
  TNode<IntPtrT> tmp204;
  TNode<IntPtrT> tmp205;
  TNode<IntPtrT> tmp206;
  TNode<IntPtrT> tmp207;
  TNode<IntPtrT> tmp208;
  TNode<BoolT> tmp209;
  if (block86.is_used()) {
    ca_.Bind(&block86, &phi_bb86_8, &phi_bb86_9, &phi_bb86_10, &phi_bb86_11, &phi_bb86_14, &phi_bb86_15, &phi_bb86_21);
    std::tie(tmp203, tmp204) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb86_11}).Flatten();
    tmp205 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp206 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb86_11}, TNode<IntPtrT>{tmp205});
    tmp207 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp208 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp206}, TNode<IntPtrT>{tmp207});
    tmp209 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block81, phi_bb86_8, phi_bb86_9, phi_bb86_10, tmp208, tmp206, tmp209, phi_bb86_21, tmp203, tmp204);
  }

  TNode<IntPtrT> phi_bb81_8;
  TNode<IntPtrT> phi_bb81_9;
  TNode<IntPtrT> phi_bb81_10;
  TNode<IntPtrT> phi_bb81_11;
  TNode<IntPtrT> phi_bb81_14;
  TNode<BoolT> phi_bb81_15;
  TNode<IntPtrT> phi_bb81_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb81_23;
  TNode<IntPtrT> phi_bb81_24;
  if (block81.is_used()) {
    ca_.Bind(&block81, &phi_bb81_8, &phi_bb81_9, &phi_bb81_10, &phi_bb81_11, &phi_bb81_14, &phi_bb81_15, &phi_bb81_21, &phi_bb81_23, &phi_bb81_24);
    ca_.Goto(&block78, phi_bb81_8, phi_bb81_9, phi_bb81_10, phi_bb81_11, phi_bb81_14, phi_bb81_15, phi_bb81_21, phi_bb81_23, phi_bb81_24);
  }

  TNode<IntPtrT> phi_bb78_8;
  TNode<IntPtrT> phi_bb78_9;
  TNode<IntPtrT> phi_bb78_10;
  TNode<IntPtrT> phi_bb78_11;
  TNode<IntPtrT> phi_bb78_14;
  TNode<BoolT> phi_bb78_15;
  TNode<IntPtrT> phi_bb78_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb78_23;
  TNode<IntPtrT> phi_bb78_24;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_8, &phi_bb78_9, &phi_bb78_10, &phi_bb78_11, &phi_bb78_14, &phi_bb78_15, &phi_bb78_21, &phi_bb78_23, &phi_bb78_24);
    if (((CodeStubAssembler(state_).ConstexprBoolNot((CodeStubAssembler(state_).Is64()))))) {
      ca_.Goto(&block87, phi_bb78_8, phi_bb78_9, phi_bb78_10, phi_bb78_11, phi_bb78_14, phi_bb78_15, phi_bb78_21);
    } else {
      ca_.Goto(&block88, phi_bb78_8, phi_bb78_9, phi_bb78_10, phi_bb78_11, phi_bb78_14, phi_bb78_15, phi_bb78_21);
    }
  }

  TNode<IntPtrT> phi_bb87_8;
  TNode<IntPtrT> phi_bb87_9;
  TNode<IntPtrT> phi_bb87_10;
  TNode<IntPtrT> phi_bb87_11;
  TNode<IntPtrT> phi_bb87_14;
  TNode<BoolT> phi_bb87_15;
  TNode<IntPtrT> phi_bb87_21;
  TNode<IntPtrT> tmp210;
  TNode<IntPtrT> tmp211;
  TNode<IntPtrT> tmp212;
  TNode<BoolT> tmp213;
  if (block87.is_used()) {
    ca_.Bind(&block87, &phi_bb87_8, &phi_bb87_9, &phi_bb87_10, &phi_bb87_11, &phi_bb87_14, &phi_bb87_15, &phi_bb87_21);
    tmp210 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp211 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp186}, TNode<IntPtrT>{tmp210});
    tmp212 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp213 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp186}, TNode<IntPtrT>{tmp212});
    ca_.Branch(tmp213, &block91, std::vector<compiler::Node*>{phi_bb87_8, phi_bb87_9, phi_bb87_10, phi_bb87_11, phi_bb87_14, phi_bb87_15, phi_bb87_21}, &block92, std::vector<compiler::Node*>{phi_bb87_8, phi_bb87_9, phi_bb87_10, phi_bb87_11, phi_bb87_14, phi_bb87_15, phi_bb87_21});
  }

  TNode<IntPtrT> phi_bb91_8;
  TNode<IntPtrT> phi_bb91_9;
  TNode<IntPtrT> phi_bb91_10;
  TNode<IntPtrT> phi_bb91_11;
  TNode<IntPtrT> phi_bb91_14;
  TNode<BoolT> phi_bb91_15;
  TNode<IntPtrT> phi_bb91_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp214;
  TNode<IntPtrT> tmp215;
  TNode<IntPtrT> tmp216;
  TNode<IntPtrT> tmp217;
  if (block91.is_used()) {
    ca_.Bind(&block91, &phi_bb91_8, &phi_bb91_9, &phi_bb91_10, &phi_bb91_11, &phi_bb91_14, &phi_bb91_15, &phi_bb91_21);
    std::tie(tmp214, tmp215) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb91_9}).Flatten();
    tmp216 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp217 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb91_9}, TNode<IntPtrT>{tmp216});
    ca_.Goto(&block90, phi_bb91_8, tmp217, phi_bb91_10, phi_bb91_11, phi_bb91_14, phi_bb91_15, phi_bb91_21, tmp214, tmp215);
  }

  TNode<IntPtrT> phi_bb92_8;
  TNode<IntPtrT> phi_bb92_9;
  TNode<IntPtrT> phi_bb92_10;
  TNode<IntPtrT> phi_bb92_11;
  TNode<IntPtrT> phi_bb92_14;
  TNode<BoolT> phi_bb92_15;
  TNode<IntPtrT> phi_bb92_21;
  if (block92.is_used()) {
    ca_.Bind(&block92, &phi_bb92_8, &phi_bb92_9, &phi_bb92_10, &phi_bb92_11, &phi_bb92_14, &phi_bb92_15, &phi_bb92_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block94, phi_bb92_8, phi_bb92_9, phi_bb92_10, phi_bb92_11, phi_bb92_14, phi_bb92_15, phi_bb92_21);
    } else {
      ca_.Goto(&block95, phi_bb92_8, phi_bb92_9, phi_bb92_10, phi_bb92_11, phi_bb92_14, phi_bb92_15, phi_bb92_21);
    }
  }

  TNode<IntPtrT> phi_bb94_8;
  TNode<IntPtrT> phi_bb94_9;
  TNode<IntPtrT> phi_bb94_10;
  TNode<IntPtrT> phi_bb94_11;
  TNode<IntPtrT> phi_bb94_14;
  TNode<BoolT> phi_bb94_15;
  TNode<IntPtrT> phi_bb94_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp218;
  TNode<IntPtrT> tmp219;
  TNode<IntPtrT> tmp220;
  TNode<IntPtrT> tmp221;
  if (block94.is_used()) {
    ca_.Bind(&block94, &phi_bb94_8, &phi_bb94_9, &phi_bb94_10, &phi_bb94_11, &phi_bb94_14, &phi_bb94_15, &phi_bb94_21);
    std::tie(tmp218, tmp219) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb94_11}).Flatten();
    tmp220 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp221 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb94_11}, TNode<IntPtrT>{tmp220});
    ca_.Goto(&block93, phi_bb94_8, phi_bb94_9, phi_bb94_10, tmp221, phi_bb94_14, phi_bb94_15, phi_bb94_21, tmp218, tmp219);
  }

  TNode<IntPtrT> phi_bb95_8;
  TNode<IntPtrT> phi_bb95_9;
  TNode<IntPtrT> phi_bb95_10;
  TNode<IntPtrT> phi_bb95_11;
  TNode<IntPtrT> phi_bb95_14;
  TNode<BoolT> phi_bb95_15;
  TNode<IntPtrT> phi_bb95_21;
  TNode<IntPtrT> tmp222;
  TNode<BoolT> tmp223;
  if (block95.is_used()) {
    ca_.Bind(&block95, &phi_bb95_8, &phi_bb95_9, &phi_bb95_10, &phi_bb95_11, &phi_bb95_14, &phi_bb95_15, &phi_bb95_21);
    tmp222 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp223 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb95_14}, TNode<IntPtrT>{tmp222});
    ca_.Branch(tmp223, &block97, std::vector<compiler::Node*>{phi_bb95_8, phi_bb95_9, phi_bb95_10, phi_bb95_11, phi_bb95_14, phi_bb95_15, phi_bb95_21}, &block98, std::vector<compiler::Node*>{phi_bb95_8, phi_bb95_9, phi_bb95_10, phi_bb95_11, phi_bb95_14, phi_bb95_15, phi_bb95_21});
  }

  TNode<IntPtrT> phi_bb97_8;
  TNode<IntPtrT> phi_bb97_9;
  TNode<IntPtrT> phi_bb97_10;
  TNode<IntPtrT> phi_bb97_11;
  TNode<IntPtrT> phi_bb97_14;
  TNode<BoolT> phi_bb97_15;
  TNode<IntPtrT> phi_bb97_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp224;
  TNode<IntPtrT> tmp225;
  TNode<IntPtrT> tmp226;
  TNode<BoolT> tmp227;
  if (block97.is_used()) {
    ca_.Bind(&block97, &phi_bb97_8, &phi_bb97_9, &phi_bb97_10, &phi_bb97_11, &phi_bb97_14, &phi_bb97_15, &phi_bb97_21);
    std::tie(tmp224, tmp225) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb97_14}).Flatten();
    tmp226 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp227 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block93, phi_bb97_8, phi_bb97_9, phi_bb97_10, phi_bb97_11, tmp226, tmp227, phi_bb97_21, tmp224, tmp225);
  }

  TNode<IntPtrT> phi_bb98_8;
  TNode<IntPtrT> phi_bb98_9;
  TNode<IntPtrT> phi_bb98_10;
  TNode<IntPtrT> phi_bb98_11;
  TNode<IntPtrT> phi_bb98_14;
  TNode<BoolT> phi_bb98_15;
  TNode<IntPtrT> phi_bb98_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp228;
  TNode<IntPtrT> tmp229;
  TNode<IntPtrT> tmp230;
  TNode<IntPtrT> tmp231;
  TNode<IntPtrT> tmp232;
  TNode<IntPtrT> tmp233;
  TNode<BoolT> tmp234;
  if (block98.is_used()) {
    ca_.Bind(&block98, &phi_bb98_8, &phi_bb98_9, &phi_bb98_10, &phi_bb98_11, &phi_bb98_14, &phi_bb98_15, &phi_bb98_21);
    std::tie(tmp228, tmp229) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb98_11}).Flatten();
    tmp230 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp231 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb98_11}, TNode<IntPtrT>{tmp230});
    tmp232 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp233 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp231}, TNode<IntPtrT>{tmp232});
    tmp234 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block93, phi_bb98_8, phi_bb98_9, phi_bb98_10, tmp233, tmp231, tmp234, phi_bb98_21, tmp228, tmp229);
  }

  TNode<IntPtrT> phi_bb93_8;
  TNode<IntPtrT> phi_bb93_9;
  TNode<IntPtrT> phi_bb93_10;
  TNode<IntPtrT> phi_bb93_11;
  TNode<IntPtrT> phi_bb93_14;
  TNode<BoolT> phi_bb93_15;
  TNode<IntPtrT> phi_bb93_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb93_23;
  TNode<IntPtrT> phi_bb93_24;
  if (block93.is_used()) {
    ca_.Bind(&block93, &phi_bb93_8, &phi_bb93_9, &phi_bb93_10, &phi_bb93_11, &phi_bb93_14, &phi_bb93_15, &phi_bb93_21, &phi_bb93_23, &phi_bb93_24);
    ca_.Goto(&block90, phi_bb93_8, phi_bb93_9, phi_bb93_10, phi_bb93_11, phi_bb93_14, phi_bb93_15, phi_bb93_21, phi_bb93_23, phi_bb93_24);
  }

  TNode<IntPtrT> phi_bb90_8;
  TNode<IntPtrT> phi_bb90_9;
  TNode<IntPtrT> phi_bb90_10;
  TNode<IntPtrT> phi_bb90_11;
  TNode<IntPtrT> phi_bb90_14;
  TNode<BoolT> phi_bb90_15;
  TNode<IntPtrT> phi_bb90_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb90_23;
  TNode<IntPtrT> phi_bb90_24;
  if (block90.is_used()) {
    ca_.Bind(&block90, &phi_bb90_8, &phi_bb90_9, &phi_bb90_10, &phi_bb90_11, &phi_bb90_14, &phi_bb90_15, &phi_bb90_21, &phi_bb90_23, &phi_bb90_24);
    ca_.Goto(&block89, tmp211, phi_bb90_8, phi_bb90_9, phi_bb90_10, phi_bb90_11, phi_bb90_14, phi_bb90_15, phi_bb90_21);
  }

  TNode<IntPtrT> phi_bb88_8;
  TNode<IntPtrT> phi_bb88_9;
  TNode<IntPtrT> phi_bb88_10;
  TNode<IntPtrT> phi_bb88_11;
  TNode<IntPtrT> phi_bb88_14;
  TNode<BoolT> phi_bb88_15;
  TNode<IntPtrT> phi_bb88_21;
  if (block88.is_used()) {
    ca_.Bind(&block88, &phi_bb88_8, &phi_bb88_9, &phi_bb88_10, &phi_bb88_11, &phi_bb88_14, &phi_bb88_15, &phi_bb88_21);
    ca_.Goto(&block89, tmp186, phi_bb88_8, phi_bb88_9, phi_bb88_10, phi_bb88_11, phi_bb88_14, phi_bb88_15, phi_bb88_21);
  }

  TNode<IntPtrT> phi_bb89_7;
  TNode<IntPtrT> phi_bb89_8;
  TNode<IntPtrT> phi_bb89_9;
  TNode<IntPtrT> phi_bb89_10;
  TNode<IntPtrT> phi_bb89_11;
  TNode<IntPtrT> phi_bb89_14;
  TNode<BoolT> phi_bb89_15;
  TNode<IntPtrT> phi_bb89_21;
  if (block89.is_used()) {
    ca_.Bind(&block89, &phi_bb89_7, &phi_bb89_8, &phi_bb89_9, &phi_bb89_10, &phi_bb89_11, &phi_bb89_14, &phi_bb89_15, &phi_bb89_21);
    ca_.Goto(&block77, phi_bb89_7, phi_bb89_8, phi_bb89_9, phi_bb89_10, phi_bb89_11, phi_bb89_14, phi_bb89_15, phi_bb89_21);
  }

  TNode<IntPtrT> phi_bb76_7;
  TNode<IntPtrT> phi_bb76_8;
  TNode<IntPtrT> phi_bb76_9;
  TNode<IntPtrT> phi_bb76_10;
  TNode<IntPtrT> phi_bb76_11;
  TNode<IntPtrT> phi_bb76_14;
  TNode<BoolT> phi_bb76_15;
  TNode<IntPtrT> phi_bb76_21;
  TNode<Uint32T> tmp235;
  TNode<BoolT> tmp236;
  if (block76.is_used()) {
    ca_.Bind(&block76, &phi_bb76_7, &phi_bb76_8, &phi_bb76_9, &phi_bb76_10, &phi_bb76_11, &phi_bb76_14, &phi_bb76_15, &phi_bb76_21);
    tmp235 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF64.raw_bit_field());
    tmp236 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp128}, TNode<Uint32T>{tmp235});
    ca_.Branch(tmp236, &block99, std::vector<compiler::Node*>{phi_bb76_7, phi_bb76_8, phi_bb76_9, phi_bb76_10, phi_bb76_11, phi_bb76_14, phi_bb76_15, phi_bb76_21}, &block100, std::vector<compiler::Node*>{phi_bb76_7, phi_bb76_8, phi_bb76_9, phi_bb76_10, phi_bb76_11, phi_bb76_14, phi_bb76_15, phi_bb76_21});
  }

  TNode<IntPtrT> phi_bb99_7;
  TNode<IntPtrT> phi_bb99_8;
  TNode<IntPtrT> phi_bb99_9;
  TNode<IntPtrT> phi_bb99_10;
  TNode<IntPtrT> phi_bb99_11;
  TNode<IntPtrT> phi_bb99_14;
  TNode<BoolT> phi_bb99_15;
  TNode<IntPtrT> phi_bb99_21;
  TNode<IntPtrT> tmp237;
  TNode<IntPtrT> tmp238;
  TNode<IntPtrT> tmp239;
  TNode<BoolT> tmp240;
  if (block99.is_used()) {
    ca_.Bind(&block99, &phi_bb99_7, &phi_bb99_8, &phi_bb99_9, &phi_bb99_10, &phi_bb99_11, &phi_bb99_14, &phi_bb99_15, &phi_bb99_21);
    tmp237 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp238 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb99_8}, TNode<IntPtrT>{tmp237});
    tmp239 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp240 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb99_8}, TNode<IntPtrT>{tmp239});
    ca_.Branch(tmp240, &block102, std::vector<compiler::Node*>{phi_bb99_7, phi_bb99_9, phi_bb99_10, phi_bb99_11, phi_bb99_14, phi_bb99_15, phi_bb99_21}, &block103, std::vector<compiler::Node*>{phi_bb99_7, phi_bb99_9, phi_bb99_10, phi_bb99_11, phi_bb99_14, phi_bb99_15, phi_bb99_21});
  }

  TNode<IntPtrT> phi_bb102_7;
  TNode<IntPtrT> phi_bb102_9;
  TNode<IntPtrT> phi_bb102_10;
  TNode<IntPtrT> phi_bb102_11;
  TNode<IntPtrT> phi_bb102_14;
  TNode<BoolT> phi_bb102_15;
  TNode<IntPtrT> phi_bb102_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp241;
  TNode<IntPtrT> tmp242;
  TNode<IntPtrT> tmp243;
  TNode<IntPtrT> tmp244;
  if (block102.is_used()) {
    ca_.Bind(&block102, &phi_bb102_7, &phi_bb102_9, &phi_bb102_10, &phi_bb102_11, &phi_bb102_14, &phi_bb102_15, &phi_bb102_21);
    std::tie(tmp241, tmp242) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb102_10}).Flatten();
    tmp243 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp244 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb102_10}, TNode<IntPtrT>{tmp243});
    ca_.Goto(&block101, phi_bb102_7, phi_bb102_9, tmp244, phi_bb102_11, phi_bb102_14, phi_bb102_15, phi_bb102_21, tmp241, tmp242);
  }

  TNode<IntPtrT> phi_bb103_7;
  TNode<IntPtrT> phi_bb103_9;
  TNode<IntPtrT> phi_bb103_10;
  TNode<IntPtrT> phi_bb103_11;
  TNode<IntPtrT> phi_bb103_14;
  TNode<BoolT> phi_bb103_15;
  TNode<IntPtrT> phi_bb103_21;
  if (block103.is_used()) {
    ca_.Bind(&block103, &phi_bb103_7, &phi_bb103_9, &phi_bb103_10, &phi_bb103_11, &phi_bb103_14, &phi_bb103_15, &phi_bb103_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block104, phi_bb103_7, phi_bb103_9, phi_bb103_10, phi_bb103_11, phi_bb103_14, phi_bb103_15, phi_bb103_21);
    } else {
      ca_.Goto(&block105, phi_bb103_7, phi_bb103_9, phi_bb103_10, phi_bb103_11, phi_bb103_14, phi_bb103_15, phi_bb103_21);
    }
  }

  TNode<IntPtrT> phi_bb104_7;
  TNode<IntPtrT> phi_bb104_9;
  TNode<IntPtrT> phi_bb104_10;
  TNode<IntPtrT> phi_bb104_11;
  TNode<IntPtrT> phi_bb104_14;
  TNode<BoolT> phi_bb104_15;
  TNode<IntPtrT> phi_bb104_21;
  if (block104.is_used()) {
    ca_.Bind(&block104, &phi_bb104_7, &phi_bb104_9, &phi_bb104_10, &phi_bb104_11, &phi_bb104_14, &phi_bb104_15, &phi_bb104_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block108, phi_bb104_7, phi_bb104_9, phi_bb104_10, phi_bb104_11, phi_bb104_14, phi_bb104_15, phi_bb104_21);
    } else {
      ca_.Goto(&block109, phi_bb104_7, phi_bb104_9, phi_bb104_10, phi_bb104_11, phi_bb104_14, phi_bb104_15, phi_bb104_21);
    }
  }

  TNode<IntPtrT> phi_bb108_7;
  TNode<IntPtrT> phi_bb108_9;
  TNode<IntPtrT> phi_bb108_10;
  TNode<IntPtrT> phi_bb108_11;
  TNode<IntPtrT> phi_bb108_14;
  TNode<BoolT> phi_bb108_15;
  TNode<IntPtrT> phi_bb108_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp245;
  TNode<IntPtrT> tmp246;
  TNode<IntPtrT> tmp247;
  TNode<IntPtrT> tmp248;
  if (block108.is_used()) {
    ca_.Bind(&block108, &phi_bb108_7, &phi_bb108_9, &phi_bb108_10, &phi_bb108_11, &phi_bb108_14, &phi_bb108_15, &phi_bb108_21);
    std::tie(tmp245, tmp246) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb108_11}).Flatten();
    tmp247 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp248 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb108_11}, TNode<IntPtrT>{tmp247});
    ca_.Goto(&block107, phi_bb108_7, phi_bb108_9, phi_bb108_10, tmp248, phi_bb108_14, phi_bb108_15, phi_bb108_21, tmp245, tmp246);
  }

  TNode<IntPtrT> phi_bb109_7;
  TNode<IntPtrT> phi_bb109_9;
  TNode<IntPtrT> phi_bb109_10;
  TNode<IntPtrT> phi_bb109_11;
  TNode<IntPtrT> phi_bb109_14;
  TNode<BoolT> phi_bb109_15;
  TNode<IntPtrT> phi_bb109_21;
  TNode<IntPtrT> tmp249;
  TNode<BoolT> tmp250;
  if (block109.is_used()) {
    ca_.Bind(&block109, &phi_bb109_7, &phi_bb109_9, &phi_bb109_10, &phi_bb109_11, &phi_bb109_14, &phi_bb109_15, &phi_bb109_21);
    tmp249 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp250 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb109_14}, TNode<IntPtrT>{tmp249});
    ca_.Branch(tmp250, &block111, std::vector<compiler::Node*>{phi_bb109_7, phi_bb109_9, phi_bb109_10, phi_bb109_11, phi_bb109_14, phi_bb109_15, phi_bb109_21}, &block112, std::vector<compiler::Node*>{phi_bb109_7, phi_bb109_9, phi_bb109_10, phi_bb109_11, phi_bb109_14, phi_bb109_15, phi_bb109_21});
  }

  TNode<IntPtrT> phi_bb111_7;
  TNode<IntPtrT> phi_bb111_9;
  TNode<IntPtrT> phi_bb111_10;
  TNode<IntPtrT> phi_bb111_11;
  TNode<IntPtrT> phi_bb111_14;
  TNode<BoolT> phi_bb111_15;
  TNode<IntPtrT> phi_bb111_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp251;
  TNode<IntPtrT> tmp252;
  TNode<IntPtrT> tmp253;
  TNode<BoolT> tmp254;
  if (block111.is_used()) {
    ca_.Bind(&block111, &phi_bb111_7, &phi_bb111_9, &phi_bb111_10, &phi_bb111_11, &phi_bb111_14, &phi_bb111_15, &phi_bb111_21);
    std::tie(tmp251, tmp252) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb111_14}).Flatten();
    tmp253 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp254 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block107, phi_bb111_7, phi_bb111_9, phi_bb111_10, phi_bb111_11, tmp253, tmp254, phi_bb111_21, tmp251, tmp252);
  }

  TNode<IntPtrT> phi_bb112_7;
  TNode<IntPtrT> phi_bb112_9;
  TNode<IntPtrT> phi_bb112_10;
  TNode<IntPtrT> phi_bb112_11;
  TNode<IntPtrT> phi_bb112_14;
  TNode<BoolT> phi_bb112_15;
  TNode<IntPtrT> phi_bb112_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp255;
  TNode<IntPtrT> tmp256;
  TNode<IntPtrT> tmp257;
  TNode<IntPtrT> tmp258;
  TNode<IntPtrT> tmp259;
  TNode<IntPtrT> tmp260;
  TNode<BoolT> tmp261;
  if (block112.is_used()) {
    ca_.Bind(&block112, &phi_bb112_7, &phi_bb112_9, &phi_bb112_10, &phi_bb112_11, &phi_bb112_14, &phi_bb112_15, &phi_bb112_21);
    std::tie(tmp255, tmp256) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb112_11}).Flatten();
    tmp257 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp258 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb112_11}, TNode<IntPtrT>{tmp257});
    tmp259 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp260 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp258}, TNode<IntPtrT>{tmp259});
    tmp261 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block107, phi_bb112_7, phi_bb112_9, phi_bb112_10, tmp260, tmp258, tmp261, phi_bb112_21, tmp255, tmp256);
  }

  TNode<IntPtrT> phi_bb107_7;
  TNode<IntPtrT> phi_bb107_9;
  TNode<IntPtrT> phi_bb107_10;
  TNode<IntPtrT> phi_bb107_11;
  TNode<IntPtrT> phi_bb107_14;
  TNode<BoolT> phi_bb107_15;
  TNode<IntPtrT> phi_bb107_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb107_23;
  TNode<IntPtrT> phi_bb107_24;
  if (block107.is_used()) {
    ca_.Bind(&block107, &phi_bb107_7, &phi_bb107_9, &phi_bb107_10, &phi_bb107_11, &phi_bb107_14, &phi_bb107_15, &phi_bb107_21, &phi_bb107_23, &phi_bb107_24);
    ca_.Goto(&block101, phi_bb107_7, phi_bb107_9, phi_bb107_10, phi_bb107_11, phi_bb107_14, phi_bb107_15, phi_bb107_21, phi_bb107_23, phi_bb107_24);
  }

  TNode<IntPtrT> phi_bb105_7;
  TNode<IntPtrT> phi_bb105_9;
  TNode<IntPtrT> phi_bb105_10;
  TNode<IntPtrT> phi_bb105_11;
  TNode<IntPtrT> phi_bb105_14;
  TNode<BoolT> phi_bb105_15;
  TNode<IntPtrT> phi_bb105_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp262;
  TNode<IntPtrT> tmp263;
  TNode<IntPtrT> tmp264;
  TNode<IntPtrT> tmp265;
  TNode<BoolT> tmp266;
  if (block105.is_used()) {
    ca_.Bind(&block105, &phi_bb105_7, &phi_bb105_9, &phi_bb105_10, &phi_bb105_11, &phi_bb105_14, &phi_bb105_15, &phi_bb105_21);
    std::tie(tmp262, tmp263) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb105_11}).Flatten();
    tmp264 = FromConstexpr_intptr_constexpr_int31_0(state_, (CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull))), (SizeOf_intptr_0(state_)))));
    tmp265 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb105_11}, TNode<IntPtrT>{tmp264});
    tmp266 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block101, phi_bb105_7, phi_bb105_9, phi_bb105_10, tmp265, phi_bb105_14, tmp266, phi_bb105_21, tmp262, tmp263);
  }

  TNode<IntPtrT> phi_bb101_7;
  TNode<IntPtrT> phi_bb101_9;
  TNode<IntPtrT> phi_bb101_10;
  TNode<IntPtrT> phi_bb101_11;
  TNode<IntPtrT> phi_bb101_14;
  TNode<BoolT> phi_bb101_15;
  TNode<IntPtrT> phi_bb101_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb101_23;
  TNode<IntPtrT> phi_bb101_24;
  if (block101.is_used()) {
    ca_.Bind(&block101, &phi_bb101_7, &phi_bb101_9, &phi_bb101_10, &phi_bb101_11, &phi_bb101_14, &phi_bb101_15, &phi_bb101_21, &phi_bb101_23, &phi_bb101_24);
    ca_.Goto(&block100, phi_bb101_7, tmp238, phi_bb101_9, phi_bb101_10, phi_bb101_11, phi_bb101_14, phi_bb101_15, phi_bb101_21);
  }

  TNode<IntPtrT> phi_bb100_7;
  TNode<IntPtrT> phi_bb100_8;
  TNode<IntPtrT> phi_bb100_9;
  TNode<IntPtrT> phi_bb100_10;
  TNode<IntPtrT> phi_bb100_11;
  TNode<IntPtrT> phi_bb100_14;
  TNode<BoolT> phi_bb100_15;
  TNode<IntPtrT> phi_bb100_21;
  if (block100.is_used()) {
    ca_.Bind(&block100, &phi_bb100_7, &phi_bb100_8, &phi_bb100_9, &phi_bb100_10, &phi_bb100_11, &phi_bb100_14, &phi_bb100_15, &phi_bb100_21);
    ca_.Goto(&block77, phi_bb100_7, phi_bb100_8, phi_bb100_9, phi_bb100_10, phi_bb100_11, phi_bb100_14, phi_bb100_15, phi_bb100_21);
  }

  TNode<IntPtrT> phi_bb77_7;
  TNode<IntPtrT> phi_bb77_8;
  TNode<IntPtrT> phi_bb77_9;
  TNode<IntPtrT> phi_bb77_10;
  TNode<IntPtrT> phi_bb77_11;
  TNode<IntPtrT> phi_bb77_14;
  TNode<BoolT> phi_bb77_15;
  TNode<IntPtrT> phi_bb77_21;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_7, &phi_bb77_8, &phi_bb77_9, &phi_bb77_10, &phi_bb77_11, &phi_bb77_14, &phi_bb77_15, &phi_bb77_21);
    ca_.Goto(&block65, phi_bb77_7, phi_bb77_8, phi_bb77_9, phi_bb77_10, phi_bb77_11, phi_bb77_14, phi_bb77_15, phi_bb77_21);
  }

  TNode<IntPtrT> phi_bb65_7;
  TNode<IntPtrT> phi_bb65_8;
  TNode<IntPtrT> phi_bb65_9;
  TNode<IntPtrT> phi_bb65_10;
  TNode<IntPtrT> phi_bb65_11;
  TNode<IntPtrT> phi_bb65_14;
  TNode<BoolT> phi_bb65_15;
  TNode<IntPtrT> phi_bb65_21;
  if (block65.is_used()) {
    ca_.Bind(&block65, &phi_bb65_7, &phi_bb65_8, &phi_bb65_9, &phi_bb65_10, &phi_bb65_11, &phi_bb65_14, &phi_bb65_15, &phi_bb65_21);
    ca_.Goto(&block53, phi_bb65_7, phi_bb65_8, phi_bb65_9, phi_bb65_10, phi_bb65_11, phi_bb65_14, phi_bb65_15, phi_bb65_21);
  }

  TNode<IntPtrT> phi_bb53_7;
  TNode<IntPtrT> phi_bb53_8;
  TNode<IntPtrT> phi_bb53_9;
  TNode<IntPtrT> phi_bb53_10;
  TNode<IntPtrT> phi_bb53_11;
  TNode<IntPtrT> phi_bb53_14;
  TNode<BoolT> phi_bb53_15;
  TNode<IntPtrT> phi_bb53_21;
  TNode<IntPtrT> tmp267;
  TNode<IntPtrT> tmp268;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_7, &phi_bb53_8, &phi_bb53_9, &phi_bb53_10, &phi_bb53_11, &phi_bb53_14, &phi_bb53_15, &phi_bb53_21);
    tmp267 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp268 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb53_21}, TNode<IntPtrT>{tmp267});
    ca_.Goto(&block48, phi_bb53_7, phi_bb53_8, phi_bb53_9, phi_bb53_10, phi_bb53_11, phi_bb53_14, phi_bb53_15, tmp268);
  }

  TNode<IntPtrT> phi_bb47_7;
  TNode<IntPtrT> phi_bb47_8;
  TNode<IntPtrT> phi_bb47_9;
  TNode<IntPtrT> phi_bb47_10;
  TNode<IntPtrT> phi_bb47_11;
  TNode<IntPtrT> phi_bb47_14;
  TNode<BoolT> phi_bb47_15;
  TNode<IntPtrT> phi_bb47_21;
  TNode<BoolT> tmp269;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_7, &phi_bb47_8, &phi_bb47_9, &phi_bb47_10, &phi_bb47_11, &phi_bb47_14, &phi_bb47_15, &phi_bb47_21);
    tmp269 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{phi_bb47_15});
    ca_.Branch(tmp269, &block114, std::vector<compiler::Node*>{phi_bb47_7, phi_bb47_8, phi_bb47_9, phi_bb47_10, phi_bb47_11, phi_bb47_14, phi_bb47_15}, &block115, std::vector<compiler::Node*>{phi_bb47_7, phi_bb47_8, phi_bb47_9, phi_bb47_10, phi_bb47_11, phi_bb47_14, phi_bb47_15});
  }

  TNode<IntPtrT> phi_bb114_7;
  TNode<IntPtrT> phi_bb114_8;
  TNode<IntPtrT> phi_bb114_9;
  TNode<IntPtrT> phi_bb114_10;
  TNode<IntPtrT> phi_bb114_11;
  TNode<IntPtrT> phi_bb114_14;
  TNode<BoolT> phi_bb114_15;
  TNode<IntPtrT> tmp270;
  if (block114.is_used()) {
    ca_.Bind(&block114, &phi_bb114_7, &phi_bb114_8, &phi_bb114_9, &phi_bb114_10, &phi_bb114_11, &phi_bb114_14, &phi_bb114_15);
    tmp270 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block115, phi_bb114_7, phi_bb114_8, phi_bb114_9, phi_bb114_10, phi_bb114_11, tmp270, phi_bb114_15);
  }

  TNode<IntPtrT> phi_bb115_7;
  TNode<IntPtrT> phi_bb115_8;
  TNode<IntPtrT> phi_bb115_9;
  TNode<IntPtrT> phi_bb115_10;
  TNode<IntPtrT> phi_bb115_11;
  TNode<IntPtrT> phi_bb115_14;
  TNode<BoolT> phi_bb115_15;
  TNode<IntPtrT> tmp271;
  if (block115.is_used()) {
    ca_.Bind(&block115, &phi_bb115_7, &phi_bb115_8, &phi_bb115_9, &phi_bb115_10, &phi_bb115_11, &phi_bb115_14, &phi_bb115_15);
    tmp271 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block118, phi_bb115_7, phi_bb115_8, phi_bb115_9, phi_bb115_10, phi_bb115_11, phi_bb115_14, phi_bb115_15, tmp271);
  }

  TNode<IntPtrT> phi_bb118_7;
  TNode<IntPtrT> phi_bb118_8;
  TNode<IntPtrT> phi_bb118_9;
  TNode<IntPtrT> phi_bb118_10;
  TNode<IntPtrT> phi_bb118_11;
  TNode<IntPtrT> phi_bb118_14;
  TNode<BoolT> phi_bb118_15;
  TNode<IntPtrT> phi_bb118_21;
  TNode<IntPtrT> tmp272;
  TNode<BoolT> tmp273;
  if (block118.is_used()) {
    ca_.Bind(&block118, &phi_bb118_7, &phi_bb118_8, &phi_bb118_9, &phi_bb118_10, &phi_bb118_11, &phi_bb118_14, &phi_bb118_15, &phi_bb118_21);
    tmp272 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp3});
    tmp273 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb118_21}, TNode<IntPtrT>{tmp272});
    ca_.Branch(tmp273, &block116, std::vector<compiler::Node*>{phi_bb118_7, phi_bb118_8, phi_bb118_9, phi_bb118_10, phi_bb118_11, phi_bb118_14, phi_bb118_15, phi_bb118_21}, &block117, std::vector<compiler::Node*>{phi_bb118_7, phi_bb118_8, phi_bb118_9, phi_bb118_10, phi_bb118_11, phi_bb118_14, phi_bb118_15, phi_bb118_21});
  }

  TNode<IntPtrT> phi_bb116_7;
  TNode<IntPtrT> phi_bb116_8;
  TNode<IntPtrT> phi_bb116_9;
  TNode<IntPtrT> phi_bb116_10;
  TNode<IntPtrT> phi_bb116_11;
  TNode<IntPtrT> phi_bb116_14;
  TNode<BoolT> phi_bb116_15;
  TNode<IntPtrT> phi_bb116_21;
  TNode<IntPtrT> tmp274;
  TNode<IntPtrT> tmp275;
  TNode<Union<HeapObject, TaggedIndex>> tmp276;
  TNode<IntPtrT> tmp277;
  TNode<Uint32T> tmp278;
  TNode<Uint32T> tmp279;
  TNode<Uint32T> tmp280;
  TNode<Uint32T> tmp281;
  TNode<BoolT> tmp282;
  if (block116.is_used()) {
    ca_.Bind(&block116, &phi_bb116_7, &phi_bb116_8, &phi_bb116_9, &phi_bb116_10, &phi_bb116_11, &phi_bb116_14, &phi_bb116_15, &phi_bb116_21);
    tmp274 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{phi_bb116_21});
    tmp275 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp115}, TNode<IntPtrT>{tmp274});
    std::tie(tmp276, tmp277) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp114}, TNode<IntPtrT>{tmp275}).Flatten();
    tmp278 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp276, tmp277});
    tmp279 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp280 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp278}, TNode<Uint32T>{tmp279});
    tmp281 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp282 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp280}, TNode<Uint32T>{tmp281});
    ca_.Branch(tmp282, &block121, std::vector<compiler::Node*>{phi_bb116_7, phi_bb116_8, phi_bb116_9, phi_bb116_10, phi_bb116_11, phi_bb116_14, phi_bb116_15, phi_bb116_21}, &block122, std::vector<compiler::Node*>{phi_bb116_7, phi_bb116_8, phi_bb116_9, phi_bb116_10, phi_bb116_11, phi_bb116_14, phi_bb116_15, phi_bb116_21});
  }

  TNode<IntPtrT> phi_bb121_7;
  TNode<IntPtrT> phi_bb121_8;
  TNode<IntPtrT> phi_bb121_9;
  TNode<IntPtrT> phi_bb121_10;
  TNode<IntPtrT> phi_bb121_11;
  TNode<IntPtrT> phi_bb121_14;
  TNode<BoolT> phi_bb121_15;
  TNode<IntPtrT> phi_bb121_21;
  TNode<IntPtrT> tmp283;
  TNode<IntPtrT> tmp284;
  TNode<IntPtrT> tmp285;
  TNode<BoolT> tmp286;
  if (block121.is_used()) {
    ca_.Bind(&block121, &phi_bb121_7, &phi_bb121_8, &phi_bb121_9, &phi_bb121_10, &phi_bb121_11, &phi_bb121_14, &phi_bb121_15, &phi_bb121_21);
    tmp283 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp284 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb121_7}, TNode<IntPtrT>{tmp283});
    tmp285 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp286 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb121_7}, TNode<IntPtrT>{tmp285});
    ca_.Branch(tmp286, &block124, std::vector<compiler::Node*>{phi_bb121_8, phi_bb121_9, phi_bb121_10, phi_bb121_11, phi_bb121_14, phi_bb121_15, phi_bb121_21}, &block125, std::vector<compiler::Node*>{phi_bb121_8, phi_bb121_9, phi_bb121_10, phi_bb121_11, phi_bb121_14, phi_bb121_15, phi_bb121_21});
  }

  TNode<IntPtrT> phi_bb124_8;
  TNode<IntPtrT> phi_bb124_9;
  TNode<IntPtrT> phi_bb124_10;
  TNode<IntPtrT> phi_bb124_11;
  TNode<IntPtrT> phi_bb124_14;
  TNode<BoolT> phi_bb124_15;
  TNode<IntPtrT> phi_bb124_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp287;
  TNode<IntPtrT> tmp288;
  TNode<IntPtrT> tmp289;
  TNode<IntPtrT> tmp290;
  if (block124.is_used()) {
    ca_.Bind(&block124, &phi_bb124_8, &phi_bb124_9, &phi_bb124_10, &phi_bb124_11, &phi_bb124_14, &phi_bb124_15, &phi_bb124_21);
    std::tie(tmp287, tmp288) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb124_9}).Flatten();
    tmp289 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp290 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb124_9}, TNode<IntPtrT>{tmp289});
    ca_.Goto(&block123, phi_bb124_8, tmp290, phi_bb124_10, phi_bb124_11, phi_bb124_14, phi_bb124_15, phi_bb124_21, tmp287, tmp288);
  }

  TNode<IntPtrT> phi_bb125_8;
  TNode<IntPtrT> phi_bb125_9;
  TNode<IntPtrT> phi_bb125_10;
  TNode<IntPtrT> phi_bb125_11;
  TNode<IntPtrT> phi_bb125_14;
  TNode<BoolT> phi_bb125_15;
  TNode<IntPtrT> phi_bb125_21;
  if (block125.is_used()) {
    ca_.Bind(&block125, &phi_bb125_8, &phi_bb125_9, &phi_bb125_10, &phi_bb125_11, &phi_bb125_14, &phi_bb125_15, &phi_bb125_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block127, phi_bb125_8, phi_bb125_9, phi_bb125_10, phi_bb125_11, phi_bb125_14, phi_bb125_15, phi_bb125_21);
    } else {
      ca_.Goto(&block128, phi_bb125_8, phi_bb125_9, phi_bb125_10, phi_bb125_11, phi_bb125_14, phi_bb125_15, phi_bb125_21);
    }
  }

  TNode<IntPtrT> phi_bb127_8;
  TNode<IntPtrT> phi_bb127_9;
  TNode<IntPtrT> phi_bb127_10;
  TNode<IntPtrT> phi_bb127_11;
  TNode<IntPtrT> phi_bb127_14;
  TNode<BoolT> phi_bb127_15;
  TNode<IntPtrT> phi_bb127_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp291;
  TNode<IntPtrT> tmp292;
  TNode<IntPtrT> tmp293;
  TNode<IntPtrT> tmp294;
  if (block127.is_used()) {
    ca_.Bind(&block127, &phi_bb127_8, &phi_bb127_9, &phi_bb127_10, &phi_bb127_11, &phi_bb127_14, &phi_bb127_15, &phi_bb127_21);
    std::tie(tmp291, tmp292) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb127_11}).Flatten();
    tmp293 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp294 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb127_11}, TNode<IntPtrT>{tmp293});
    ca_.Goto(&block126, phi_bb127_8, phi_bb127_9, phi_bb127_10, tmp294, phi_bb127_14, phi_bb127_15, phi_bb127_21, tmp291, tmp292);
  }

  TNode<IntPtrT> phi_bb128_8;
  TNode<IntPtrT> phi_bb128_9;
  TNode<IntPtrT> phi_bb128_10;
  TNode<IntPtrT> phi_bb128_11;
  TNode<IntPtrT> phi_bb128_14;
  TNode<BoolT> phi_bb128_15;
  TNode<IntPtrT> phi_bb128_21;
  TNode<IntPtrT> tmp295;
  TNode<BoolT> tmp296;
  if (block128.is_used()) {
    ca_.Bind(&block128, &phi_bb128_8, &phi_bb128_9, &phi_bb128_10, &phi_bb128_11, &phi_bb128_14, &phi_bb128_15, &phi_bb128_21);
    tmp295 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp296 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb128_14}, TNode<IntPtrT>{tmp295});
    ca_.Branch(tmp296, &block130, std::vector<compiler::Node*>{phi_bb128_8, phi_bb128_9, phi_bb128_10, phi_bb128_11, phi_bb128_14, phi_bb128_15, phi_bb128_21}, &block131, std::vector<compiler::Node*>{phi_bb128_8, phi_bb128_9, phi_bb128_10, phi_bb128_11, phi_bb128_14, phi_bb128_15, phi_bb128_21});
  }

  TNode<IntPtrT> phi_bb130_8;
  TNode<IntPtrT> phi_bb130_9;
  TNode<IntPtrT> phi_bb130_10;
  TNode<IntPtrT> phi_bb130_11;
  TNode<IntPtrT> phi_bb130_14;
  TNode<BoolT> phi_bb130_15;
  TNode<IntPtrT> phi_bb130_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp297;
  TNode<IntPtrT> tmp298;
  TNode<IntPtrT> tmp299;
  TNode<BoolT> tmp300;
  if (block130.is_used()) {
    ca_.Bind(&block130, &phi_bb130_8, &phi_bb130_9, &phi_bb130_10, &phi_bb130_11, &phi_bb130_14, &phi_bb130_15, &phi_bb130_21);
    std::tie(tmp297, tmp298) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb130_14}).Flatten();
    tmp299 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp300 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block126, phi_bb130_8, phi_bb130_9, phi_bb130_10, phi_bb130_11, tmp299, tmp300, phi_bb130_21, tmp297, tmp298);
  }

  TNode<IntPtrT> phi_bb131_8;
  TNode<IntPtrT> phi_bb131_9;
  TNode<IntPtrT> phi_bb131_10;
  TNode<IntPtrT> phi_bb131_11;
  TNode<IntPtrT> phi_bb131_14;
  TNode<BoolT> phi_bb131_15;
  TNode<IntPtrT> phi_bb131_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp301;
  TNode<IntPtrT> tmp302;
  TNode<IntPtrT> tmp303;
  TNode<IntPtrT> tmp304;
  TNode<IntPtrT> tmp305;
  TNode<IntPtrT> tmp306;
  TNode<BoolT> tmp307;
  if (block131.is_used()) {
    ca_.Bind(&block131, &phi_bb131_8, &phi_bb131_9, &phi_bb131_10, &phi_bb131_11, &phi_bb131_14, &phi_bb131_15, &phi_bb131_21);
    std::tie(tmp301, tmp302) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp98}, TNode<IntPtrT>{phi_bb131_11}).Flatten();
    tmp303 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp304 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb131_11}, TNode<IntPtrT>{tmp303});
    tmp305 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp306 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp304}, TNode<IntPtrT>{tmp305});
    tmp307 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block126, phi_bb131_8, phi_bb131_9, phi_bb131_10, tmp306, tmp304, tmp307, phi_bb131_21, tmp301, tmp302);
  }

  TNode<IntPtrT> phi_bb126_8;
  TNode<IntPtrT> phi_bb126_9;
  TNode<IntPtrT> phi_bb126_10;
  TNode<IntPtrT> phi_bb126_11;
  TNode<IntPtrT> phi_bb126_14;
  TNode<BoolT> phi_bb126_15;
  TNode<IntPtrT> phi_bb126_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb126_23;
  TNode<IntPtrT> phi_bb126_24;
  if (block126.is_used()) {
    ca_.Bind(&block126, &phi_bb126_8, &phi_bb126_9, &phi_bb126_10, &phi_bb126_11, &phi_bb126_14, &phi_bb126_15, &phi_bb126_21, &phi_bb126_23, &phi_bb126_24);
    ca_.Goto(&block123, phi_bb126_8, phi_bb126_9, phi_bb126_10, phi_bb126_11, phi_bb126_14, phi_bb126_15, phi_bb126_21, phi_bb126_23, phi_bb126_24);
  }

  TNode<IntPtrT> phi_bb123_8;
  TNode<IntPtrT> phi_bb123_9;
  TNode<IntPtrT> phi_bb123_10;
  TNode<IntPtrT> phi_bb123_11;
  TNode<IntPtrT> phi_bb123_14;
  TNode<BoolT> phi_bb123_15;
  TNode<IntPtrT> phi_bb123_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb123_23;
  TNode<IntPtrT> phi_bb123_24;
  TNode<IntPtrT> tmp308;
  TNode<Object> tmp309;
  TNode<Union<HeapObject, TaggedIndex>> tmp310;
  TNode<IntPtrT> tmp311;
  TNode<IntPtrT> tmp312;
  TNode<UintPtrT> tmp313;
  TNode<UintPtrT> tmp314;
  TNode<BoolT> tmp315;
  if (block123.is_used()) {
    ca_.Bind(&block123, &phi_bb123_8, &phi_bb123_9, &phi_bb123_10, &phi_bb123_11, &phi_bb123_14, &phi_bb123_15, &phi_bb123_21, &phi_bb123_23, &phi_bb123_24);
    tmp308 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb123_23, phi_bb123_24});
    tmp309 = CodeStubAssembler(state_).BitcastWordToTagged(TNode<IntPtrT>{tmp308});
    std::tie(tmp310, tmp311, tmp312) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp89}).Flatten();
    tmp313 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb123_21});
    tmp314 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp312});
    tmp315 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp313}, TNode<UintPtrT>{tmp314});
    ca_.Branch(tmp315, &block136, std::vector<compiler::Node*>{phi_bb123_8, phi_bb123_9, phi_bb123_10, phi_bb123_11, phi_bb123_14, phi_bb123_15, phi_bb123_21, phi_bb123_23, phi_bb123_24, phi_bb123_21, phi_bb123_21, phi_bb123_21, phi_bb123_21}, &block137, std::vector<compiler::Node*>{phi_bb123_8, phi_bb123_9, phi_bb123_10, phi_bb123_11, phi_bb123_14, phi_bb123_15, phi_bb123_21, phi_bb123_23, phi_bb123_24, phi_bb123_21, phi_bb123_21, phi_bb123_21, phi_bb123_21});
  }

  TNode<IntPtrT> phi_bb136_8;
  TNode<IntPtrT> phi_bb136_9;
  TNode<IntPtrT> phi_bb136_10;
  TNode<IntPtrT> phi_bb136_11;
  TNode<IntPtrT> phi_bb136_14;
  TNode<BoolT> phi_bb136_15;
  TNode<IntPtrT> phi_bb136_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb136_23;
  TNode<IntPtrT> phi_bb136_24;
  TNode<IntPtrT> phi_bb136_31;
  TNode<IntPtrT> phi_bb136_32;
  TNode<IntPtrT> phi_bb136_36;
  TNode<IntPtrT> phi_bb136_37;
  TNode<IntPtrT> tmp316;
  TNode<IntPtrT> tmp317;
  TNode<Union<HeapObject, TaggedIndex>> tmp318;
  TNode<IntPtrT> tmp319;
  if (block136.is_used()) {
    ca_.Bind(&block136, &phi_bb136_8, &phi_bb136_9, &phi_bb136_10, &phi_bb136_11, &phi_bb136_14, &phi_bb136_15, &phi_bb136_21, &phi_bb136_23, &phi_bb136_24, &phi_bb136_31, &phi_bb136_32, &phi_bb136_36, &phi_bb136_37);
    tmp316 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb136_37});
    tmp317 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp311}, TNode<IntPtrT>{tmp316});
    std::tie(tmp318, tmp319) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp310}, TNode<IntPtrT>{tmp317}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp318, tmp319}, tmp309);
    ca_.Goto(&block122, tmp284, phi_bb136_8, phi_bb136_9, phi_bb136_10, phi_bb136_11, phi_bb136_14, phi_bb136_15, phi_bb136_21);
  }

  TNode<IntPtrT> phi_bb137_8;
  TNode<IntPtrT> phi_bb137_9;
  TNode<IntPtrT> phi_bb137_10;
  TNode<IntPtrT> phi_bb137_11;
  TNode<IntPtrT> phi_bb137_14;
  TNode<BoolT> phi_bb137_15;
  TNode<IntPtrT> phi_bb137_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb137_23;
  TNode<IntPtrT> phi_bb137_24;
  TNode<IntPtrT> phi_bb137_31;
  TNode<IntPtrT> phi_bb137_32;
  TNode<IntPtrT> phi_bb137_36;
  TNode<IntPtrT> phi_bb137_37;
  if (block137.is_used()) {
    ca_.Bind(&block137, &phi_bb137_8, &phi_bb137_9, &phi_bb137_10, &phi_bb137_11, &phi_bb137_14, &phi_bb137_15, &phi_bb137_21, &phi_bb137_23, &phi_bb137_24, &phi_bb137_31, &phi_bb137_32, &phi_bb137_36, &phi_bb137_37);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb122_7;
  TNode<IntPtrT> phi_bb122_8;
  TNode<IntPtrT> phi_bb122_9;
  TNode<IntPtrT> phi_bb122_10;
  TNode<IntPtrT> phi_bb122_11;
  TNode<IntPtrT> phi_bb122_14;
  TNode<BoolT> phi_bb122_15;
  TNode<IntPtrT> phi_bb122_21;
  TNode<IntPtrT> tmp320;
  TNode<IntPtrT> tmp321;
  if (block122.is_used()) {
    ca_.Bind(&block122, &phi_bb122_7, &phi_bb122_8, &phi_bb122_9, &phi_bb122_10, &phi_bb122_11, &phi_bb122_14, &phi_bb122_15, &phi_bb122_21);
    tmp320 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp321 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb122_21}, TNode<IntPtrT>{tmp320});
    ca_.Goto(&block118, phi_bb122_7, phi_bb122_8, phi_bb122_9, phi_bb122_10, phi_bb122_11, phi_bb122_14, phi_bb122_15, tmp321);
  }

  TNode<IntPtrT> phi_bb117_7;
  TNode<IntPtrT> phi_bb117_8;
  TNode<IntPtrT> phi_bb117_9;
  TNode<IntPtrT> phi_bb117_10;
  TNode<IntPtrT> phi_bb117_11;
  TNode<IntPtrT> phi_bb117_14;
  TNode<BoolT> phi_bb117_15;
  TNode<IntPtrT> phi_bb117_21;
  if (block117.is_used()) {
    ca_.Bind(&block117, &phi_bb117_7, &phi_bb117_8, &phi_bb117_9, &phi_bb117_10, &phi_bb117_11, &phi_bb117_14, &phi_bb117_15, &phi_bb117_21);
    ca_.Goto(&block45, phi_bb117_7, phi_bb117_8, phi_bb117_9, phi_bb117_10, phi_bb117_11, phi_bb117_14, phi_bb117_15);
  }

  TNode<IntPtrT> phi_bb45_7;
  TNode<IntPtrT> phi_bb45_8;
  TNode<IntPtrT> phi_bb45_9;
  TNode<IntPtrT> phi_bb45_10;
  TNode<IntPtrT> phi_bb45_11;
  TNode<IntPtrT> phi_bb45_14;
  TNode<BoolT> phi_bb45_15;
  TNode<IntPtrT> tmp322;
  TNode<BoolT> tmp323;
  if (block45.is_used()) {
    ca_.Bind(&block45, &phi_bb45_7, &phi_bb45_8, &phi_bb45_9, &phi_bb45_10, &phi_bb45_11, &phi_bb45_14, &phi_bb45_15);
    tmp322 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp323 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp105}, TNode<IntPtrT>{tmp322});
    ca_.Branch(tmp323, &block143, std::vector<compiler::Node*>{phi_bb45_7, phi_bb45_8, phi_bb45_9, phi_bb45_10, phi_bb45_11, phi_bb45_14, phi_bb45_15}, &block144, std::vector<compiler::Node*>{phi_bb45_7, phi_bb45_8, phi_bb45_9, phi_bb45_10, phi_bb45_11, phi_bb45_14, phi_bb45_15});
  }

  TNode<IntPtrT> phi_bb143_7;
  TNode<IntPtrT> phi_bb143_8;
  TNode<IntPtrT> phi_bb143_9;
  TNode<IntPtrT> phi_bb143_10;
  TNode<IntPtrT> phi_bb143_11;
  TNode<IntPtrT> phi_bb143_14;
  TNode<BoolT> phi_bb143_15;
  TNode<BoolT> tmp324;
  if (block143.is_used()) {
    ca_.Bind(&block143, &phi_bb143_7, &phi_bb143_8, &phi_bb143_9, &phi_bb143_10, &phi_bb143_11, &phi_bb143_14, &phi_bb143_15);
    tmp324 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block145, phi_bb143_7, phi_bb143_8, phi_bb143_9, phi_bb143_10, phi_bb143_11, phi_bb143_14, phi_bb143_15, tmp324);
  }

  TNode<IntPtrT> phi_bb144_7;
  TNode<IntPtrT> phi_bb144_8;
  TNode<IntPtrT> phi_bb144_9;
  TNode<IntPtrT> phi_bb144_10;
  TNode<IntPtrT> phi_bb144_11;
  TNode<IntPtrT> phi_bb144_14;
  TNode<BoolT> phi_bb144_15;
  TNode<BoolT> tmp325;
  if (block144.is_used()) {
    ca_.Bind(&block144, &phi_bb144_7, &phi_bb144_8, &phi_bb144_9, &phi_bb144_10, &phi_bb144_11, &phi_bb144_14, &phi_bb144_15);
    tmp325 = CodeStubAssembler(state_).IntPtrLessThanOrEqual(TNode<IntPtrT>{phi_bb144_11}, TNode<IntPtrT>{tmp105});
    ca_.Goto(&block145, phi_bb144_7, phi_bb144_8, phi_bb144_9, phi_bb144_10, phi_bb144_11, phi_bb144_14, phi_bb144_15, tmp325);
  }

  TNode<IntPtrT> phi_bb145_7;
  TNode<IntPtrT> phi_bb145_8;
  TNode<IntPtrT> phi_bb145_9;
  TNode<IntPtrT> phi_bb145_10;
  TNode<IntPtrT> phi_bb145_11;
  TNode<IntPtrT> phi_bb145_14;
  TNode<BoolT> phi_bb145_15;
  TNode<BoolT> phi_bb145_22;
  if (block145.is_used()) {
    ca_.Bind(&block145, &phi_bb145_7, &phi_bb145_8, &phi_bb145_9, &phi_bb145_10, &phi_bb145_11, &phi_bb145_14, &phi_bb145_15, &phi_bb145_22);
    ca_.Branch(phi_bb145_22, &block141, std::vector<compiler::Node*>{phi_bb145_7, phi_bb145_8, phi_bb145_9, phi_bb145_10, phi_bb145_11, phi_bb145_14, phi_bb145_15}, &block142, std::vector<compiler::Node*>{phi_bb145_7, phi_bb145_8, phi_bb145_9, phi_bb145_10, phi_bb145_11, phi_bb145_14, phi_bb145_15});
  }

  TNode<IntPtrT> phi_bb142_7;
  TNode<IntPtrT> phi_bb142_8;
  TNode<IntPtrT> phi_bb142_9;
  TNode<IntPtrT> phi_bb142_10;
  TNode<IntPtrT> phi_bb142_11;
  TNode<IntPtrT> phi_bb142_14;
  TNode<BoolT> phi_bb142_15;
  if (block142.is_used()) {
    ca_.Bind(&block142, &phi_bb142_7, &phi_bb142_8, &phi_bb142_9, &phi_bb142_10, &phi_bb142_11, &phi_bb142_14, &phi_bb142_15);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/js-to-wasm.tq", 379});
      CodeStubAssembler(state_).FailAssert("Torque assert 'this.paramBufferEnd == 0 || this.nextStack <= this.paramBufferEnd' failed", pos_stack);
    }
  }

  TNode<IntPtrT> phi_bb141_7;
  TNode<IntPtrT> phi_bb141_8;
  TNode<IntPtrT> phi_bb141_9;
  TNode<IntPtrT> phi_bb141_10;
  TNode<IntPtrT> phi_bb141_11;
  TNode<IntPtrT> phi_bb141_14;
  TNode<BoolT> phi_bb141_15;
  TNode<IntPtrT> tmp326;
  TNode<RawPtrT> tmp327;
  TNode<IntPtrT> tmp328;
  TNode<RawPtrT> tmp329;
  TNode<Union<HeapObject, TaggedIndex>> tmp330;
  TNode<IntPtrT> tmp331;
  TNode<IntPtrT> tmp332;
  TNode<IntPtrT> tmp333;
  TNode<IntPtrT> tmp334;
  TNode<IntPtrT> tmp335;
  TNode<IntPtrT> tmp336;
  TNode<IntPtrT> tmp337;
  TNode<IntPtrT> tmp338;
  TNode<BoolT> tmp339;
  TNode<IntPtrT> tmp340;
  if (block141.is_used()) {
    ca_.Bind(&block141, &phi_bb141_7, &phi_bb141_8, &phi_bb141_9, &phi_bb141_10, &phi_bb141_11, &phi_bb141_14, &phi_bb141_15);
    tmp326 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferGPReturnRegister1);
    tmp327 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp326});
    tmp328 = FromConstexpr_intptr_constexpr_intptr_0(state_, JSToWasmWrapperFrameConstants::kWrapperBufferFPReturnRegister1);
    tmp329 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{parameter2}, TNode<IntPtrT>{tmp328});
    std::tie(tmp330, tmp331, tmp332, tmp333, tmp334, tmp335, tmp336, tmp337, tmp338, tmp339) = LocationAllocatorForReturns_0(state_, TNode<RawPtrT>{tmp327}, TNode<RawPtrT>{tmp329}, TNode<RawPtrT>{tmp93}).Flatten();
    tmp340 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block148, tmp331, tmp332, tmp333, tmp334, tmp335, tmp338, tmp339, tmp340);
  }

  TNode<IntPtrT> phi_bb148_7;
  TNode<IntPtrT> phi_bb148_8;
  TNode<IntPtrT> phi_bb148_9;
  TNode<IntPtrT> phi_bb148_10;
  TNode<IntPtrT> phi_bb148_11;
  TNode<IntPtrT> phi_bb148_14;
  TNode<BoolT> phi_bb148_15;
  TNode<IntPtrT> phi_bb148_21;
  TNode<IntPtrT> tmp341;
  TNode<BoolT> tmp342;
  if (block148.is_used()) {
    ca_.Bind(&block148, &phi_bb148_7, &phi_bb148_8, &phi_bb148_9, &phi_bb148_10, &phi_bb148_11, &phi_bb148_14, &phi_bb148_15, &phi_bb148_21);
    tmp341 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp3});
    tmp342 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb148_21}, TNode<IntPtrT>{tmp341});
    ca_.Branch(tmp342, &block146, std::vector<compiler::Node*>{phi_bb148_7, phi_bb148_8, phi_bb148_9, phi_bb148_10, phi_bb148_11, phi_bb148_14, phi_bb148_15, phi_bb148_21}, &block147, std::vector<compiler::Node*>{phi_bb148_7, phi_bb148_8, phi_bb148_9, phi_bb148_10, phi_bb148_11, phi_bb148_14, phi_bb148_15, phi_bb148_21});
  }

  TNode<IntPtrT> phi_bb146_7;
  TNode<IntPtrT> phi_bb146_8;
  TNode<IntPtrT> phi_bb146_9;
  TNode<IntPtrT> phi_bb146_10;
  TNode<IntPtrT> phi_bb146_11;
  TNode<IntPtrT> phi_bb146_14;
  TNode<BoolT> phi_bb146_15;
  TNode<IntPtrT> phi_bb146_21;
  TNode<IntPtrT> tmp343;
  TNode<IntPtrT> tmp344;
  TNode<Union<HeapObject, TaggedIndex>> tmp345;
  TNode<IntPtrT> tmp346;
  TNode<Uint32T> tmp347;
  TNode<Uint32T> tmp348;
  TNode<BoolT> tmp349;
  if (block146.is_used()) {
    ca_.Bind(&block146, &phi_bb146_7, &phi_bb146_8, &phi_bb146_9, &phi_bb146_10, &phi_bb146_11, &phi_bb146_14, &phi_bb146_15, &phi_bb146_21);
    tmp343 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{phi_bb146_21});
    tmp344 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp115}, TNode<IntPtrT>{tmp343});
    std::tie(tmp345, tmp346) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp114}, TNode<IntPtrT>{tmp344}).Flatten();
    tmp347 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp345, tmp346});
    tmp348 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI32.raw_bit_field());
    tmp349 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp347}, TNode<Uint32T>{tmp348});
    ca_.Branch(tmp349, &block151, std::vector<compiler::Node*>{phi_bb146_7, phi_bb146_8, phi_bb146_9, phi_bb146_10, phi_bb146_11, phi_bb146_14, phi_bb146_15, phi_bb146_21}, &block152, std::vector<compiler::Node*>{phi_bb146_7, phi_bb146_8, phi_bb146_9, phi_bb146_10, phi_bb146_11, phi_bb146_14, phi_bb146_15, phi_bb146_21});
  }

  TNode<IntPtrT> phi_bb151_7;
  TNode<IntPtrT> phi_bb151_8;
  TNode<IntPtrT> phi_bb151_9;
  TNode<IntPtrT> phi_bb151_10;
  TNode<IntPtrT> phi_bb151_11;
  TNode<IntPtrT> phi_bb151_14;
  TNode<BoolT> phi_bb151_15;
  TNode<IntPtrT> phi_bb151_21;
  TNode<IntPtrT> tmp350;
  TNode<IntPtrT> tmp351;
  TNode<IntPtrT> tmp352;
  TNode<BoolT> tmp353;
  if (block151.is_used()) {
    ca_.Bind(&block151, &phi_bb151_7, &phi_bb151_8, &phi_bb151_9, &phi_bb151_10, &phi_bb151_11, &phi_bb151_14, &phi_bb151_15, &phi_bb151_21);
    tmp350 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp351 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb151_7}, TNode<IntPtrT>{tmp350});
    tmp352 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp353 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb151_7}, TNode<IntPtrT>{tmp352});
    ca_.Branch(tmp353, &block155, std::vector<compiler::Node*>{phi_bb151_8, phi_bb151_9, phi_bb151_10, phi_bb151_11, phi_bb151_14, phi_bb151_15, phi_bb151_21}, &block156, std::vector<compiler::Node*>{phi_bb151_8, phi_bb151_9, phi_bb151_10, phi_bb151_11, phi_bb151_14, phi_bb151_15, phi_bb151_21});
  }

  TNode<IntPtrT> phi_bb155_8;
  TNode<IntPtrT> phi_bb155_9;
  TNode<IntPtrT> phi_bb155_10;
  TNode<IntPtrT> phi_bb155_11;
  TNode<IntPtrT> phi_bb155_14;
  TNode<BoolT> phi_bb155_15;
  TNode<IntPtrT> phi_bb155_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp354;
  TNode<IntPtrT> tmp355;
  TNode<IntPtrT> tmp356;
  TNode<IntPtrT> tmp357;
  if (block155.is_used()) {
    ca_.Bind(&block155, &phi_bb155_8, &phi_bb155_9, &phi_bb155_10, &phi_bb155_11, &phi_bb155_14, &phi_bb155_15, &phi_bb155_21);
    std::tie(tmp354, tmp355) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb155_9}).Flatten();
    tmp356 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp357 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb155_9}, TNode<IntPtrT>{tmp356});
    ca_.Goto(&block154, phi_bb155_8, tmp357, phi_bb155_10, phi_bb155_11, phi_bb155_14, phi_bb155_15, phi_bb155_21, tmp354, tmp355);
  }

  TNode<IntPtrT> phi_bb156_8;
  TNode<IntPtrT> phi_bb156_9;
  TNode<IntPtrT> phi_bb156_10;
  TNode<IntPtrT> phi_bb156_11;
  TNode<IntPtrT> phi_bb156_14;
  TNode<BoolT> phi_bb156_15;
  TNode<IntPtrT> phi_bb156_21;
  if (block156.is_used()) {
    ca_.Bind(&block156, &phi_bb156_8, &phi_bb156_9, &phi_bb156_10, &phi_bb156_11, &phi_bb156_14, &phi_bb156_15, &phi_bb156_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block158, phi_bb156_8, phi_bb156_9, phi_bb156_10, phi_bb156_11, phi_bb156_14, phi_bb156_15, phi_bb156_21);
    } else {
      ca_.Goto(&block159, phi_bb156_8, phi_bb156_9, phi_bb156_10, phi_bb156_11, phi_bb156_14, phi_bb156_15, phi_bb156_21);
    }
  }

  TNode<IntPtrT> phi_bb158_8;
  TNode<IntPtrT> phi_bb158_9;
  TNode<IntPtrT> phi_bb158_10;
  TNode<IntPtrT> phi_bb158_11;
  TNode<IntPtrT> phi_bb158_14;
  TNode<BoolT> phi_bb158_15;
  TNode<IntPtrT> phi_bb158_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp358;
  TNode<IntPtrT> tmp359;
  TNode<IntPtrT> tmp360;
  TNode<IntPtrT> tmp361;
  if (block158.is_used()) {
    ca_.Bind(&block158, &phi_bb158_8, &phi_bb158_9, &phi_bb158_10, &phi_bb158_11, &phi_bb158_14, &phi_bb158_15, &phi_bb158_21);
    std::tie(tmp358, tmp359) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb158_11}).Flatten();
    tmp360 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp361 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb158_11}, TNode<IntPtrT>{tmp360});
    ca_.Goto(&block157, phi_bb158_8, phi_bb158_9, phi_bb158_10, tmp361, phi_bb158_14, phi_bb158_15, phi_bb158_21, tmp358, tmp359);
  }

  TNode<IntPtrT> phi_bb159_8;
  TNode<IntPtrT> phi_bb159_9;
  TNode<IntPtrT> phi_bb159_10;
  TNode<IntPtrT> phi_bb159_11;
  TNode<IntPtrT> phi_bb159_14;
  TNode<BoolT> phi_bb159_15;
  TNode<IntPtrT> phi_bb159_21;
  TNode<IntPtrT> tmp362;
  TNode<BoolT> tmp363;
  if (block159.is_used()) {
    ca_.Bind(&block159, &phi_bb159_8, &phi_bb159_9, &phi_bb159_10, &phi_bb159_11, &phi_bb159_14, &phi_bb159_15, &phi_bb159_21);
    tmp362 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp363 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb159_14}, TNode<IntPtrT>{tmp362});
    ca_.Branch(tmp363, &block161, std::vector<compiler::Node*>{phi_bb159_8, phi_bb159_9, phi_bb159_10, phi_bb159_11, phi_bb159_14, phi_bb159_15, phi_bb159_21}, &block162, std::vector<compiler::Node*>{phi_bb159_8, phi_bb159_9, phi_bb159_10, phi_bb159_11, phi_bb159_14, phi_bb159_15, phi_bb159_21});
  }

  TNode<IntPtrT> phi_bb161_8;
  TNode<IntPtrT> phi_bb161_9;
  TNode<IntPtrT> phi_bb161_10;
  TNode<IntPtrT> phi_bb161_11;
  TNode<IntPtrT> phi_bb161_14;
  TNode<BoolT> phi_bb161_15;
  TNode<IntPtrT> phi_bb161_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp364;
  TNode<IntPtrT> tmp365;
  TNode<IntPtrT> tmp366;
  TNode<BoolT> tmp367;
  if (block161.is_used()) {
    ca_.Bind(&block161, &phi_bb161_8, &phi_bb161_9, &phi_bb161_10, &phi_bb161_11, &phi_bb161_14, &phi_bb161_15, &phi_bb161_21);
    std::tie(tmp364, tmp365) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb161_14}).Flatten();
    tmp366 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp367 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block157, phi_bb161_8, phi_bb161_9, phi_bb161_10, phi_bb161_11, tmp366, tmp367, phi_bb161_21, tmp364, tmp365);
  }

  TNode<IntPtrT> phi_bb162_8;
  TNode<IntPtrT> phi_bb162_9;
  TNode<IntPtrT> phi_bb162_10;
  TNode<IntPtrT> phi_bb162_11;
  TNode<IntPtrT> phi_bb162_14;
  TNode<BoolT> phi_bb162_15;
  TNode<IntPtrT> phi_bb162_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp368;
  TNode<IntPtrT> tmp369;
  TNode<IntPtrT> tmp370;
  TNode<IntPtrT> tmp371;
  TNode<IntPtrT> tmp372;
  TNode<IntPtrT> tmp373;
  TNode<BoolT> tmp374;
  if (block162.is_used()) {
    ca_.Bind(&block162, &phi_bb162_8, &phi_bb162_9, &phi_bb162_10, &phi_bb162_11, &phi_bb162_14, &phi_bb162_15, &phi_bb162_21);
    std::tie(tmp368, tmp369) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb162_11}).Flatten();
    tmp370 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp371 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb162_11}, TNode<IntPtrT>{tmp370});
    tmp372 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp373 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp371}, TNode<IntPtrT>{tmp372});
    tmp374 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block157, phi_bb162_8, phi_bb162_9, phi_bb162_10, tmp373, tmp371, tmp374, phi_bb162_21, tmp368, tmp369);
  }

  TNode<IntPtrT> phi_bb157_8;
  TNode<IntPtrT> phi_bb157_9;
  TNode<IntPtrT> phi_bb157_10;
  TNode<IntPtrT> phi_bb157_11;
  TNode<IntPtrT> phi_bb157_14;
  TNode<BoolT> phi_bb157_15;
  TNode<IntPtrT> phi_bb157_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb157_23;
  TNode<IntPtrT> phi_bb157_24;
  if (block157.is_used()) {
    ca_.Bind(&block157, &phi_bb157_8, &phi_bb157_9, &phi_bb157_10, &phi_bb157_11, &phi_bb157_14, &phi_bb157_15, &phi_bb157_21, &phi_bb157_23, &phi_bb157_24);
    ca_.Goto(&block154, phi_bb157_8, phi_bb157_9, phi_bb157_10, phi_bb157_11, phi_bb157_14, phi_bb157_15, phi_bb157_21, phi_bb157_23, phi_bb157_24);
  }

  TNode<IntPtrT> phi_bb154_8;
  TNode<IntPtrT> phi_bb154_9;
  TNode<IntPtrT> phi_bb154_10;
  TNode<IntPtrT> phi_bb154_11;
  TNode<IntPtrT> phi_bb154_14;
  TNode<BoolT> phi_bb154_15;
  TNode<IntPtrT> phi_bb154_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb154_23;
  TNode<IntPtrT> phi_bb154_24;
  if (block154.is_used()) {
    ca_.Bind(&block154, &phi_bb154_8, &phi_bb154_9, &phi_bb154_10, &phi_bb154_11, &phi_bb154_14, &phi_bb154_15, &phi_bb154_21, &phi_bb154_23, &phi_bb154_24);
    if ((wasm::kIsBigEndian)) {
      ca_.Goto(&block163, phi_bb154_8, phi_bb154_9, phi_bb154_10, phi_bb154_11, phi_bb154_14, phi_bb154_15, phi_bb154_21, phi_bb154_23, phi_bb154_24);
    } else {
      ca_.Goto(&block164, phi_bb154_8, phi_bb154_9, phi_bb154_10, phi_bb154_11, phi_bb154_14, phi_bb154_15, phi_bb154_21, phi_bb154_23, phi_bb154_24);
    }
  }

  TNode<IntPtrT> phi_bb163_8;
  TNode<IntPtrT> phi_bb163_9;
  TNode<IntPtrT> phi_bb163_10;
  TNode<IntPtrT> phi_bb163_11;
  TNode<IntPtrT> phi_bb163_14;
  TNode<BoolT> phi_bb163_15;
  TNode<IntPtrT> phi_bb163_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb163_23;
  TNode<IntPtrT> phi_bb163_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp375;
  TNode<IntPtrT> tmp376;
  TNode<Int64T> tmp377;
  TNode<Int32T> tmp378;
  if (block163.is_used()) {
    ca_.Bind(&block163, &phi_bb163_8, &phi_bb163_9, &phi_bb163_10, &phi_bb163_11, &phi_bb163_14, &phi_bb163_15, &phi_bb163_21, &phi_bb163_23, &phi_bb163_24);
    std::tie(tmp375, tmp376) = RefCast_int64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb163_23}, TNode<IntPtrT>{phi_bb163_24}, TorqueStructUnsafe_0{}}).Flatten();
    tmp377 = CodeStubAssembler(state_).LoadReference<Int64T>(CodeStubAssembler::Reference{tmp375, tmp376});
    tmp378 = CodeStubAssembler(state_).TruncateInt64ToInt32(TNode<Int64T>{tmp377});
    ca_.Goto(&block165, phi_bb163_8, phi_bb163_9, phi_bb163_10, phi_bb163_11, phi_bb163_14, phi_bb163_15, phi_bb163_21, phi_bb163_23, phi_bb163_24, tmp378);
  }

  TNode<IntPtrT> phi_bb164_8;
  TNode<IntPtrT> phi_bb164_9;
  TNode<IntPtrT> phi_bb164_10;
  TNode<IntPtrT> phi_bb164_11;
  TNode<IntPtrT> phi_bb164_14;
  TNode<BoolT> phi_bb164_15;
  TNode<IntPtrT> phi_bb164_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb164_23;
  TNode<IntPtrT> phi_bb164_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp379;
  TNode<IntPtrT> tmp380;
  TNode<Int32T> tmp381;
  if (block164.is_used()) {
    ca_.Bind(&block164, &phi_bb164_8, &phi_bb164_9, &phi_bb164_10, &phi_bb164_11, &phi_bb164_14, &phi_bb164_15, &phi_bb164_21, &phi_bb164_23, &phi_bb164_24);
    std::tie(tmp379, tmp380) = RefCast_int32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb164_23}, TNode<IntPtrT>{phi_bb164_24}, TorqueStructUnsafe_0{}}).Flatten();
    tmp381 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp379, tmp380});
    ca_.Goto(&block165, phi_bb164_8, phi_bb164_9, phi_bb164_10, phi_bb164_11, phi_bb164_14, phi_bb164_15, phi_bb164_21, phi_bb164_23, phi_bb164_24, tmp381);
  }

  TNode<IntPtrT> phi_bb165_8;
  TNode<IntPtrT> phi_bb165_9;
  TNode<IntPtrT> phi_bb165_10;
  TNode<IntPtrT> phi_bb165_11;
  TNode<IntPtrT> phi_bb165_14;
  TNode<BoolT> phi_bb165_15;
  TNode<IntPtrT> phi_bb165_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb165_23;
  TNode<IntPtrT> phi_bb165_24;
  TNode<Int32T> phi_bb165_25;
  TNode<Union<HeapObject, TaggedIndex>> tmp382;
  TNode<IntPtrT> tmp383;
  TNode<IntPtrT> tmp384;
  TNode<UintPtrT> tmp385;
  TNode<UintPtrT> tmp386;
  TNode<BoolT> tmp387;
  if (block165.is_used()) {
    ca_.Bind(&block165, &phi_bb165_8, &phi_bb165_9, &phi_bb165_10, &phi_bb165_11, &phi_bb165_14, &phi_bb165_15, &phi_bb165_21, &phi_bb165_23, &phi_bb165_24, &phi_bb165_25);
    std::tie(tmp382, tmp383, tmp384) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp89}).Flatten();
    tmp385 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb165_21});
    tmp386 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp384});
    tmp387 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp385}, TNode<UintPtrT>{tmp386});
    ca_.Branch(tmp387, &block170, std::vector<compiler::Node*>{phi_bb165_8, phi_bb165_9, phi_bb165_10, phi_bb165_11, phi_bb165_14, phi_bb165_15, phi_bb165_21, phi_bb165_23, phi_bb165_24, phi_bb165_21, phi_bb165_21, phi_bb165_21, phi_bb165_21}, &block171, std::vector<compiler::Node*>{phi_bb165_8, phi_bb165_9, phi_bb165_10, phi_bb165_11, phi_bb165_14, phi_bb165_15, phi_bb165_21, phi_bb165_23, phi_bb165_24, phi_bb165_21, phi_bb165_21, phi_bb165_21, phi_bb165_21});
  }

  TNode<IntPtrT> phi_bb170_8;
  TNode<IntPtrT> phi_bb170_9;
  TNode<IntPtrT> phi_bb170_10;
  TNode<IntPtrT> phi_bb170_11;
  TNode<IntPtrT> phi_bb170_14;
  TNode<BoolT> phi_bb170_15;
  TNode<IntPtrT> phi_bb170_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb170_23;
  TNode<IntPtrT> phi_bb170_24;
  TNode<IntPtrT> phi_bb170_30;
  TNode<IntPtrT> phi_bb170_31;
  TNode<IntPtrT> phi_bb170_35;
  TNode<IntPtrT> phi_bb170_36;
  TNode<IntPtrT> tmp388;
  TNode<IntPtrT> tmp389;
  TNode<Union<HeapObject, TaggedIndex>> tmp390;
  TNode<IntPtrT> tmp391;
  TNode<Number> tmp392;
  if (block170.is_used()) {
    ca_.Bind(&block170, &phi_bb170_8, &phi_bb170_9, &phi_bb170_10, &phi_bb170_11, &phi_bb170_14, &phi_bb170_15, &phi_bb170_21, &phi_bb170_23, &phi_bb170_24, &phi_bb170_30, &phi_bb170_31, &phi_bb170_35, &phi_bb170_36);
    tmp388 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb170_36});
    tmp389 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp383}, TNode<IntPtrT>{tmp388});
    std::tie(tmp390, tmp391) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp382}, TNode<IntPtrT>{tmp389}).Flatten();
    tmp392 = Convert_Number_int32_0(state_, TNode<Int32T>{phi_bb165_25});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp390, tmp391}, tmp392);
    ca_.Goto(&block153, tmp351, phi_bb170_8, phi_bb170_9, phi_bb170_10, phi_bb170_11, phi_bb170_14, phi_bb170_15, phi_bb170_21);
  }

  TNode<IntPtrT> phi_bb171_8;
  TNode<IntPtrT> phi_bb171_9;
  TNode<IntPtrT> phi_bb171_10;
  TNode<IntPtrT> phi_bb171_11;
  TNode<IntPtrT> phi_bb171_14;
  TNode<BoolT> phi_bb171_15;
  TNode<IntPtrT> phi_bb171_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb171_23;
  TNode<IntPtrT> phi_bb171_24;
  TNode<IntPtrT> phi_bb171_30;
  TNode<IntPtrT> phi_bb171_31;
  TNode<IntPtrT> phi_bb171_35;
  TNode<IntPtrT> phi_bb171_36;
  if (block171.is_used()) {
    ca_.Bind(&block171, &phi_bb171_8, &phi_bb171_9, &phi_bb171_10, &phi_bb171_11, &phi_bb171_14, &phi_bb171_15, &phi_bb171_21, &phi_bb171_23, &phi_bb171_24, &phi_bb171_30, &phi_bb171_31, &phi_bb171_35, &phi_bb171_36);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb152_7;
  TNode<IntPtrT> phi_bb152_8;
  TNode<IntPtrT> phi_bb152_9;
  TNode<IntPtrT> phi_bb152_10;
  TNode<IntPtrT> phi_bb152_11;
  TNode<IntPtrT> phi_bb152_14;
  TNode<BoolT> phi_bb152_15;
  TNode<IntPtrT> phi_bb152_21;
  TNode<Uint32T> tmp393;
  TNode<BoolT> tmp394;
  if (block152.is_used()) {
    ca_.Bind(&block152, &phi_bb152_7, &phi_bb152_8, &phi_bb152_9, &phi_bb152_10, &phi_bb152_11, &phi_bb152_14, &phi_bb152_15, &phi_bb152_21);
    tmp393 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF32.raw_bit_field());
    tmp394 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp347}, TNode<Uint32T>{tmp393});
    ca_.Branch(tmp394, &block174, std::vector<compiler::Node*>{phi_bb152_7, phi_bb152_8, phi_bb152_9, phi_bb152_10, phi_bb152_11, phi_bb152_14, phi_bb152_15, phi_bb152_21}, &block175, std::vector<compiler::Node*>{phi_bb152_7, phi_bb152_8, phi_bb152_9, phi_bb152_10, phi_bb152_11, phi_bb152_14, phi_bb152_15, phi_bb152_21});
  }

  TNode<IntPtrT> phi_bb174_7;
  TNode<IntPtrT> phi_bb174_8;
  TNode<IntPtrT> phi_bb174_9;
  TNode<IntPtrT> phi_bb174_10;
  TNode<IntPtrT> phi_bb174_11;
  TNode<IntPtrT> phi_bb174_14;
  TNode<BoolT> phi_bb174_15;
  TNode<IntPtrT> phi_bb174_21;
  TNode<IntPtrT> tmp395;
  TNode<IntPtrT> tmp396;
  TNode<IntPtrT> tmp397;
  TNode<BoolT> tmp398;
  if (block174.is_used()) {
    ca_.Bind(&block174, &phi_bb174_7, &phi_bb174_8, &phi_bb174_9, &phi_bb174_10, &phi_bb174_11, &phi_bb174_14, &phi_bb174_15, &phi_bb174_21);
    tmp395 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp396 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb174_8}, TNode<IntPtrT>{tmp395});
    tmp397 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp398 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb174_8}, TNode<IntPtrT>{tmp397});
    ca_.Branch(tmp398, &block178, std::vector<compiler::Node*>{phi_bb174_7, phi_bb174_9, phi_bb174_10, phi_bb174_11, phi_bb174_14, phi_bb174_15, phi_bb174_21}, &block179, std::vector<compiler::Node*>{phi_bb174_7, phi_bb174_9, phi_bb174_10, phi_bb174_11, phi_bb174_14, phi_bb174_15, phi_bb174_21});
  }

  TNode<IntPtrT> phi_bb178_7;
  TNode<IntPtrT> phi_bb178_9;
  TNode<IntPtrT> phi_bb178_10;
  TNode<IntPtrT> phi_bb178_11;
  TNode<IntPtrT> phi_bb178_14;
  TNode<BoolT> phi_bb178_15;
  TNode<IntPtrT> phi_bb178_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp399;
  TNode<IntPtrT> tmp400;
  TNode<IntPtrT> tmp401;
  TNode<IntPtrT> tmp402;
  if (block178.is_used()) {
    ca_.Bind(&block178, &phi_bb178_7, &phi_bb178_9, &phi_bb178_10, &phi_bb178_11, &phi_bb178_14, &phi_bb178_15, &phi_bb178_21);
    std::tie(tmp399, tmp400) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb178_10}).Flatten();
    tmp401 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp402 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb178_10}, TNode<IntPtrT>{tmp401});
    ca_.Goto(&block177, phi_bb178_7, phi_bb178_9, tmp402, phi_bb178_11, phi_bb178_14, phi_bb178_15, phi_bb178_21, tmp399, tmp400);
  }

  TNode<IntPtrT> phi_bb179_7;
  TNode<IntPtrT> phi_bb179_9;
  TNode<IntPtrT> phi_bb179_10;
  TNode<IntPtrT> phi_bb179_11;
  TNode<IntPtrT> phi_bb179_14;
  TNode<BoolT> phi_bb179_15;
  TNode<IntPtrT> phi_bb179_21;
  if (block179.is_used()) {
    ca_.Bind(&block179, &phi_bb179_7, &phi_bb179_9, &phi_bb179_10, &phi_bb179_11, &phi_bb179_14, &phi_bb179_15, &phi_bb179_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block181, phi_bb179_7, phi_bb179_9, phi_bb179_10, phi_bb179_11, phi_bb179_14, phi_bb179_15, phi_bb179_21);
    } else {
      ca_.Goto(&block182, phi_bb179_7, phi_bb179_9, phi_bb179_10, phi_bb179_11, phi_bb179_14, phi_bb179_15, phi_bb179_21);
    }
  }

  TNode<IntPtrT> phi_bb181_7;
  TNode<IntPtrT> phi_bb181_9;
  TNode<IntPtrT> phi_bb181_10;
  TNode<IntPtrT> phi_bb181_11;
  TNode<IntPtrT> phi_bb181_14;
  TNode<BoolT> phi_bb181_15;
  TNode<IntPtrT> phi_bb181_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp403;
  TNode<IntPtrT> tmp404;
  TNode<IntPtrT> tmp405;
  TNode<IntPtrT> tmp406;
  if (block181.is_used()) {
    ca_.Bind(&block181, &phi_bb181_7, &phi_bb181_9, &phi_bb181_10, &phi_bb181_11, &phi_bb181_14, &phi_bb181_15, &phi_bb181_21);
    std::tie(tmp403, tmp404) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb181_11}).Flatten();
    tmp405 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp406 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb181_11}, TNode<IntPtrT>{tmp405});
    ca_.Goto(&block180, phi_bb181_7, phi_bb181_9, phi_bb181_10, tmp406, phi_bb181_14, phi_bb181_15, phi_bb181_21, tmp403, tmp404);
  }

  TNode<IntPtrT> phi_bb182_7;
  TNode<IntPtrT> phi_bb182_9;
  TNode<IntPtrT> phi_bb182_10;
  TNode<IntPtrT> phi_bb182_11;
  TNode<IntPtrT> phi_bb182_14;
  TNode<BoolT> phi_bb182_15;
  TNode<IntPtrT> phi_bb182_21;
  TNode<IntPtrT> tmp407;
  TNode<BoolT> tmp408;
  if (block182.is_used()) {
    ca_.Bind(&block182, &phi_bb182_7, &phi_bb182_9, &phi_bb182_10, &phi_bb182_11, &phi_bb182_14, &phi_bb182_15, &phi_bb182_21);
    tmp407 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp408 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb182_14}, TNode<IntPtrT>{tmp407});
    ca_.Branch(tmp408, &block184, std::vector<compiler::Node*>{phi_bb182_7, phi_bb182_9, phi_bb182_10, phi_bb182_11, phi_bb182_14, phi_bb182_15, phi_bb182_21}, &block185, std::vector<compiler::Node*>{phi_bb182_7, phi_bb182_9, phi_bb182_10, phi_bb182_11, phi_bb182_14, phi_bb182_15, phi_bb182_21});
  }

  TNode<IntPtrT> phi_bb184_7;
  TNode<IntPtrT> phi_bb184_9;
  TNode<IntPtrT> phi_bb184_10;
  TNode<IntPtrT> phi_bb184_11;
  TNode<IntPtrT> phi_bb184_14;
  TNode<BoolT> phi_bb184_15;
  TNode<IntPtrT> phi_bb184_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp409;
  TNode<IntPtrT> tmp410;
  TNode<IntPtrT> tmp411;
  TNode<BoolT> tmp412;
  if (block184.is_used()) {
    ca_.Bind(&block184, &phi_bb184_7, &phi_bb184_9, &phi_bb184_10, &phi_bb184_11, &phi_bb184_14, &phi_bb184_15, &phi_bb184_21);
    std::tie(tmp409, tmp410) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb184_14}).Flatten();
    tmp411 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp412 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block180, phi_bb184_7, phi_bb184_9, phi_bb184_10, phi_bb184_11, tmp411, tmp412, phi_bb184_21, tmp409, tmp410);
  }

  TNode<IntPtrT> phi_bb185_7;
  TNode<IntPtrT> phi_bb185_9;
  TNode<IntPtrT> phi_bb185_10;
  TNode<IntPtrT> phi_bb185_11;
  TNode<IntPtrT> phi_bb185_14;
  TNode<BoolT> phi_bb185_15;
  TNode<IntPtrT> phi_bb185_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp413;
  TNode<IntPtrT> tmp414;
  TNode<IntPtrT> tmp415;
  TNode<IntPtrT> tmp416;
  TNode<IntPtrT> tmp417;
  TNode<IntPtrT> tmp418;
  TNode<BoolT> tmp419;
  if (block185.is_used()) {
    ca_.Bind(&block185, &phi_bb185_7, &phi_bb185_9, &phi_bb185_10, &phi_bb185_11, &phi_bb185_14, &phi_bb185_15, &phi_bb185_21);
    std::tie(tmp413, tmp414) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb185_11}).Flatten();
    tmp415 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp416 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb185_11}, TNode<IntPtrT>{tmp415});
    tmp417 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp418 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp416}, TNode<IntPtrT>{tmp417});
    tmp419 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block180, phi_bb185_7, phi_bb185_9, phi_bb185_10, tmp418, tmp416, tmp419, phi_bb185_21, tmp413, tmp414);
  }

  TNode<IntPtrT> phi_bb180_7;
  TNode<IntPtrT> phi_bb180_9;
  TNode<IntPtrT> phi_bb180_10;
  TNode<IntPtrT> phi_bb180_11;
  TNode<IntPtrT> phi_bb180_14;
  TNode<BoolT> phi_bb180_15;
  TNode<IntPtrT> phi_bb180_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb180_23;
  TNode<IntPtrT> phi_bb180_24;
  if (block180.is_used()) {
    ca_.Bind(&block180, &phi_bb180_7, &phi_bb180_9, &phi_bb180_10, &phi_bb180_11, &phi_bb180_14, &phi_bb180_15, &phi_bb180_21, &phi_bb180_23, &phi_bb180_24);
    ca_.Goto(&block177, phi_bb180_7, phi_bb180_9, phi_bb180_10, phi_bb180_11, phi_bb180_14, phi_bb180_15, phi_bb180_21, phi_bb180_23, phi_bb180_24);
  }

  TNode<IntPtrT> phi_bb177_7;
  TNode<IntPtrT> phi_bb177_9;
  TNode<IntPtrT> phi_bb177_10;
  TNode<IntPtrT> phi_bb177_11;
  TNode<IntPtrT> phi_bb177_14;
  TNode<BoolT> phi_bb177_15;
  TNode<IntPtrT> phi_bb177_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb177_23;
  TNode<IntPtrT> phi_bb177_24;
  if (block177.is_used()) {
    ca_.Bind(&block177, &phi_bb177_7, &phi_bb177_9, &phi_bb177_10, &phi_bb177_11, &phi_bb177_14, &phi_bb177_15, &phi_bb177_21, &phi_bb177_23, &phi_bb177_24);
    if ((wasm::kIsFpAlwaysDouble)) {
      ca_.Goto(&block186, phi_bb177_7, phi_bb177_9, phi_bb177_10, phi_bb177_11, phi_bb177_14, phi_bb177_15, phi_bb177_21, phi_bb177_23, phi_bb177_24);
    } else {
      ca_.Goto(&block187, phi_bb177_7, phi_bb177_9, phi_bb177_10, phi_bb177_11, phi_bb177_14, phi_bb177_15, phi_bb177_21, phi_bb177_23, phi_bb177_24);
    }
  }

  TNode<IntPtrT> phi_bb186_7;
  TNode<IntPtrT> phi_bb186_9;
  TNode<IntPtrT> phi_bb186_10;
  TNode<IntPtrT> phi_bb186_11;
  TNode<IntPtrT> phi_bb186_14;
  TNode<BoolT> phi_bb186_15;
  TNode<IntPtrT> phi_bb186_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb186_23;
  TNode<IntPtrT> phi_bb186_24;
  TNode<IntPtrT> tmp420;
  TNode<BoolT> tmp421;
  if (block186.is_used()) {
    ca_.Bind(&block186, &phi_bb186_7, &phi_bb186_9, &phi_bb186_10, &phi_bb186_11, &phi_bb186_14, &phi_bb186_15, &phi_bb186_21, &phi_bb186_23, &phi_bb186_24);
    tmp420 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp421 = CodeStubAssembler(state_).IntPtrGreaterThanOrEqual(TNode<IntPtrT>{tmp396}, TNode<IntPtrT>{tmp420});
    ca_.Branch(tmp421, &block189, std::vector<compiler::Node*>{phi_bb186_7, phi_bb186_9, phi_bb186_10, phi_bb186_11, phi_bb186_14, phi_bb186_15, phi_bb186_21, phi_bb186_23, phi_bb186_24}, &block190, std::vector<compiler::Node*>{phi_bb186_7, phi_bb186_9, phi_bb186_10, phi_bb186_11, phi_bb186_14, phi_bb186_15, phi_bb186_21, phi_bb186_23, phi_bb186_24});
  }

  TNode<IntPtrT> phi_bb189_7;
  TNode<IntPtrT> phi_bb189_9;
  TNode<IntPtrT> phi_bb189_10;
  TNode<IntPtrT> phi_bb189_11;
  TNode<IntPtrT> phi_bb189_14;
  TNode<BoolT> phi_bb189_15;
  TNode<IntPtrT> phi_bb189_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb189_23;
  TNode<IntPtrT> phi_bb189_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp422;
  TNode<IntPtrT> tmp423;
  TNode<Float64T> tmp424;
  TNode<Float32T> tmp425;
  if (block189.is_used()) {
    ca_.Bind(&block189, &phi_bb189_7, &phi_bb189_9, &phi_bb189_10, &phi_bb189_11, &phi_bb189_14, &phi_bb189_15, &phi_bb189_21, &phi_bb189_23, &phi_bb189_24);
    std::tie(tmp422, tmp423) = RefCast_float64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb189_23}, TNode<IntPtrT>{phi_bb189_24}, TorqueStructUnsafe_0{}}).Flatten();
    tmp424 = CodeStubAssembler(state_).LoadReference<Float64T>(CodeStubAssembler::Reference{tmp422, tmp423});
    tmp425 = CodeStubAssembler(state_).TruncateFloat64ToFloat32(TNode<Float64T>{tmp424});
    ca_.Goto(&block192, phi_bb189_7, phi_bb189_9, phi_bb189_10, phi_bb189_11, phi_bb189_14, phi_bb189_15, phi_bb189_21, phi_bb189_23, phi_bb189_24, tmp425);
  }

  TNode<IntPtrT> phi_bb190_7;
  TNode<IntPtrT> phi_bb190_9;
  TNode<IntPtrT> phi_bb190_10;
  TNode<IntPtrT> phi_bb190_11;
  TNode<IntPtrT> phi_bb190_14;
  TNode<BoolT> phi_bb190_15;
  TNode<IntPtrT> phi_bb190_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb190_23;
  TNode<IntPtrT> phi_bb190_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp426;
  TNode<IntPtrT> tmp427;
  TNode<Float32T> tmp428;
  if (block190.is_used()) {
    ca_.Bind(&block190, &phi_bb190_7, &phi_bb190_9, &phi_bb190_10, &phi_bb190_11, &phi_bb190_14, &phi_bb190_15, &phi_bb190_21, &phi_bb190_23, &phi_bb190_24);
    std::tie(tmp426, tmp427) = RefCast_float32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb190_23}, TNode<IntPtrT>{phi_bb190_24}, TorqueStructUnsafe_0{}}).Flatten();
    tmp428 = CodeStubAssembler(state_).LoadReference<Float32T>(CodeStubAssembler::Reference{tmp426, tmp427});
    ca_.Goto(&block192, phi_bb190_7, phi_bb190_9, phi_bb190_10, phi_bb190_11, phi_bb190_14, phi_bb190_15, phi_bb190_21, phi_bb190_23, phi_bb190_24, tmp428);
  }

  TNode<IntPtrT> phi_bb192_7;
  TNode<IntPtrT> phi_bb192_9;
  TNode<IntPtrT> phi_bb192_10;
  TNode<IntPtrT> phi_bb192_11;
  TNode<IntPtrT> phi_bb192_14;
  TNode<BoolT> phi_bb192_15;
  TNode<IntPtrT> phi_bb192_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb192_23;
  TNode<IntPtrT> phi_bb192_24;
  TNode<Float32T> phi_bb192_25;
  if (block192.is_used()) {
    ca_.Bind(&block192, &phi_bb192_7, &phi_bb192_9, &phi_bb192_10, &phi_bb192_11, &phi_bb192_14, &phi_bb192_15, &phi_bb192_21, &phi_bb192_23, &phi_bb192_24, &phi_bb192_25);
    ca_.Goto(&block188, phi_bb192_7, phi_bb192_9, phi_bb192_10, phi_bb192_11, phi_bb192_14, phi_bb192_15, phi_bb192_21, phi_bb192_23, phi_bb192_24, phi_bb192_25);
  }

  TNode<IntPtrT> phi_bb187_7;
  TNode<IntPtrT> phi_bb187_9;
  TNode<IntPtrT> phi_bb187_10;
  TNode<IntPtrT> phi_bb187_11;
  TNode<IntPtrT> phi_bb187_14;
  TNode<BoolT> phi_bb187_15;
  TNode<IntPtrT> phi_bb187_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb187_23;
  TNode<IntPtrT> phi_bb187_24;
  if (block187.is_used()) {
    ca_.Bind(&block187, &phi_bb187_7, &phi_bb187_9, &phi_bb187_10, &phi_bb187_11, &phi_bb187_14, &phi_bb187_15, &phi_bb187_21, &phi_bb187_23, &phi_bb187_24);
    if ((wasm::kIsBigEndianOnSim)) {
      ca_.Goto(&block193, phi_bb187_7, phi_bb187_9, phi_bb187_10, phi_bb187_11, phi_bb187_14, phi_bb187_15, phi_bb187_21, phi_bb187_23, phi_bb187_24);
    } else {
      ca_.Goto(&block194, phi_bb187_7, phi_bb187_9, phi_bb187_10, phi_bb187_11, phi_bb187_14, phi_bb187_15, phi_bb187_21, phi_bb187_23, phi_bb187_24);
    }
  }

  TNode<IntPtrT> phi_bb193_7;
  TNode<IntPtrT> phi_bb193_9;
  TNode<IntPtrT> phi_bb193_10;
  TNode<IntPtrT> phi_bb193_11;
  TNode<IntPtrT> phi_bb193_14;
  TNode<BoolT> phi_bb193_15;
  TNode<IntPtrT> phi_bb193_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb193_23;
  TNode<IntPtrT> phi_bb193_24;
  TNode<IntPtrT> tmp429;
  TNode<BoolT> tmp430;
  if (block193.is_used()) {
    ca_.Bind(&block193, &phi_bb193_7, &phi_bb193_9, &phi_bb193_10, &phi_bb193_11, &phi_bb193_14, &phi_bb193_15, &phi_bb193_21, &phi_bb193_23, &phi_bb193_24);
    tmp429 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp430 = CodeStubAssembler(state_).IntPtrGreaterThanOrEqual(TNode<IntPtrT>{tmp396}, TNode<IntPtrT>{tmp429});
    ca_.Branch(tmp430, &block196, std::vector<compiler::Node*>{phi_bb193_7, phi_bb193_9, phi_bb193_10, phi_bb193_11, phi_bb193_14, phi_bb193_15, phi_bb193_21, phi_bb193_23, phi_bb193_24}, &block197, std::vector<compiler::Node*>{phi_bb193_7, phi_bb193_9, phi_bb193_10, phi_bb193_11, phi_bb193_14, phi_bb193_15, phi_bb193_21, phi_bb193_23, phi_bb193_24});
  }

  TNode<IntPtrT> phi_bb196_7;
  TNode<IntPtrT> phi_bb196_9;
  TNode<IntPtrT> phi_bb196_10;
  TNode<IntPtrT> phi_bb196_11;
  TNode<IntPtrT> phi_bb196_14;
  TNode<BoolT> phi_bb196_15;
  TNode<IntPtrT> phi_bb196_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb196_23;
  TNode<IntPtrT> phi_bb196_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp431;
  TNode<IntPtrT> tmp432;
  TNode<Int64T> tmp433;
  TNode<Int64T> tmp434;
  TNode<Int64T> tmp435;
  TNode<Int32T> tmp436;
  TNode<Float32T> tmp437;
  if (block196.is_used()) {
    ca_.Bind(&block196, &phi_bb196_7, &phi_bb196_9, &phi_bb196_10, &phi_bb196_11, &phi_bb196_14, &phi_bb196_15, &phi_bb196_21, &phi_bb196_23, &phi_bb196_24);
    std::tie(tmp431, tmp432) = RefCast_int64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb196_23}, TNode<IntPtrT>{phi_bb196_24}, TorqueStructUnsafe_0{}}).Flatten();
    tmp433 = CodeStubAssembler(state_).LoadReference<Int64T>(CodeStubAssembler::Reference{tmp431, tmp432});
    tmp434 = FromConstexpr_int64_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x20ull));
    tmp435 = CodeStubAssembler(state_).Word64Sar(TNode<Int64T>{tmp433}, TNode<Int64T>{tmp434});
    tmp436 = CodeStubAssembler(state_).TruncateInt64ToInt32(TNode<Int64T>{tmp435});
    tmp437 = CodeStubAssembler(state_).BitcastInt32ToFloat32(TNode<Int32T>{tmp436});
    ca_.Goto(&block199, phi_bb196_7, phi_bb196_9, phi_bb196_10, phi_bb196_11, phi_bb196_14, phi_bb196_15, phi_bb196_21, phi_bb196_23, phi_bb196_24, tmp437);
  }

  TNode<IntPtrT> phi_bb197_7;
  TNode<IntPtrT> phi_bb197_9;
  TNode<IntPtrT> phi_bb197_10;
  TNode<IntPtrT> phi_bb197_11;
  TNode<IntPtrT> phi_bb197_14;
  TNode<BoolT> phi_bb197_15;
  TNode<IntPtrT> phi_bb197_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb197_23;
  TNode<IntPtrT> phi_bb197_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp438;
  TNode<IntPtrT> tmp439;
  TNode<Float32T> tmp440;
  if (block197.is_used()) {
    ca_.Bind(&block197, &phi_bb197_7, &phi_bb197_9, &phi_bb197_10, &phi_bb197_11, &phi_bb197_14, &phi_bb197_15, &phi_bb197_21, &phi_bb197_23, &phi_bb197_24);
    std::tie(tmp438, tmp439) = RefCast_float32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb197_23}, TNode<IntPtrT>{phi_bb197_24}, TorqueStructUnsafe_0{}}).Flatten();
    tmp440 = CodeStubAssembler(state_).LoadReference<Float32T>(CodeStubAssembler::Reference{tmp438, tmp439});
    ca_.Goto(&block199, phi_bb197_7, phi_bb197_9, phi_bb197_10, phi_bb197_11, phi_bb197_14, phi_bb197_15, phi_bb197_21, phi_bb197_23, phi_bb197_24, tmp440);
  }

  TNode<IntPtrT> phi_bb199_7;
  TNode<IntPtrT> phi_bb199_9;
  TNode<IntPtrT> phi_bb199_10;
  TNode<IntPtrT> phi_bb199_11;
  TNode<IntPtrT> phi_bb199_14;
  TNode<BoolT> phi_bb199_15;
  TNode<IntPtrT> phi_bb199_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb199_23;
  TNode<IntPtrT> phi_bb199_24;
  TNode<Float32T> phi_bb199_25;
  if (block199.is_used()) {
    ca_.Bind(&block199, &phi_bb199_7, &phi_bb199_9, &phi_bb199_10, &phi_bb199_11, &phi_bb199_14, &phi_bb199_15, &phi_bb199_21, &phi_bb199_23, &phi_bb199_24, &phi_bb199_25);
    ca_.Goto(&block195, phi_bb199_7, phi_bb199_9, phi_bb199_10, phi_bb199_11, phi_bb199_14, phi_bb199_15, phi_bb199_21, phi_bb199_23, phi_bb199_24, phi_bb199_25);
  }

  TNode<IntPtrT> phi_bb194_7;
  TNode<IntPtrT> phi_bb194_9;
  TNode<IntPtrT> phi_bb194_10;
  TNode<IntPtrT> phi_bb194_11;
  TNode<IntPtrT> phi_bb194_14;
  TNode<BoolT> phi_bb194_15;
  TNode<IntPtrT> phi_bb194_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb194_23;
  TNode<IntPtrT> phi_bb194_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp441;
  TNode<IntPtrT> tmp442;
  TNode<Float32T> tmp443;
  if (block194.is_used()) {
    ca_.Bind(&block194, &phi_bb194_7, &phi_bb194_9, &phi_bb194_10, &phi_bb194_11, &phi_bb194_14, &phi_bb194_15, &phi_bb194_21, &phi_bb194_23, &phi_bb194_24);
    std::tie(tmp441, tmp442) = RefCast_float32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb194_23}, TNode<IntPtrT>{phi_bb194_24}, TorqueStructUnsafe_0{}}).Flatten();
    tmp443 = CodeStubAssembler(state_).LoadReference<Float32T>(CodeStubAssembler::Reference{tmp441, tmp442});
    ca_.Goto(&block195, phi_bb194_7, phi_bb194_9, phi_bb194_10, phi_bb194_11, phi_bb194_14, phi_bb194_15, phi_bb194_21, phi_bb194_23, phi_bb194_24, tmp443);
  }

  TNode<IntPtrT> phi_bb195_7;
  TNode<IntPtrT> phi_bb195_9;
  TNode<IntPtrT> phi_bb195_10;
  TNode<IntPtrT> phi_bb195_11;
  TNode<IntPtrT> phi_bb195_14;
  TNode<BoolT> phi_bb195_15;
  TNode<IntPtrT> phi_bb195_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb195_23;
  TNode<IntPtrT> phi_bb195_24;
  TNode<Float32T> phi_bb195_25;
  if (block195.is_used()) {
    ca_.Bind(&block195, &phi_bb195_7, &phi_bb195_9, &phi_bb195_10, &phi_bb195_11, &phi_bb195_14, &phi_bb195_15, &phi_bb195_21, &phi_bb195_23, &phi_bb195_24, &phi_bb195_25);
    ca_.Goto(&block188, phi_bb195_7, phi_bb195_9, phi_bb195_10, phi_bb195_11, phi_bb195_14, phi_bb195_15, phi_bb195_21, phi_bb195_23, phi_bb195_24, phi_bb195_25);
  }

  TNode<IntPtrT> phi_bb188_7;
  TNode<IntPtrT> phi_bb188_9;
  TNode<IntPtrT> phi_bb188_10;
  TNode<IntPtrT> phi_bb188_11;
  TNode<IntPtrT> phi_bb188_14;
  TNode<BoolT> phi_bb188_15;
  TNode<IntPtrT> phi_bb188_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb188_23;
  TNode<IntPtrT> phi_bb188_24;
  TNode<Float32T> phi_bb188_25;
  TNode<Union<HeapObject, TaggedIndex>> tmp444;
  TNode<IntPtrT> tmp445;
  TNode<IntPtrT> tmp446;
  TNode<UintPtrT> tmp447;
  TNode<UintPtrT> tmp448;
  TNode<BoolT> tmp449;
  if (block188.is_used()) {
    ca_.Bind(&block188, &phi_bb188_7, &phi_bb188_9, &phi_bb188_10, &phi_bb188_11, &phi_bb188_14, &phi_bb188_15, &phi_bb188_21, &phi_bb188_23, &phi_bb188_24, &phi_bb188_25);
    std::tie(tmp444, tmp445, tmp446) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp89}).Flatten();
    tmp447 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb188_21});
    tmp448 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp446});
    tmp449 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp447}, TNode<UintPtrT>{tmp448});
    ca_.Branch(tmp449, &block204, std::vector<compiler::Node*>{phi_bb188_7, phi_bb188_9, phi_bb188_10, phi_bb188_11, phi_bb188_14, phi_bb188_15, phi_bb188_21, phi_bb188_23, phi_bb188_24, phi_bb188_21, phi_bb188_21, phi_bb188_21, phi_bb188_21}, &block205, std::vector<compiler::Node*>{phi_bb188_7, phi_bb188_9, phi_bb188_10, phi_bb188_11, phi_bb188_14, phi_bb188_15, phi_bb188_21, phi_bb188_23, phi_bb188_24, phi_bb188_21, phi_bb188_21, phi_bb188_21, phi_bb188_21});
  }

  TNode<IntPtrT> phi_bb204_7;
  TNode<IntPtrT> phi_bb204_9;
  TNode<IntPtrT> phi_bb204_10;
  TNode<IntPtrT> phi_bb204_11;
  TNode<IntPtrT> phi_bb204_14;
  TNode<BoolT> phi_bb204_15;
  TNode<IntPtrT> phi_bb204_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb204_23;
  TNode<IntPtrT> phi_bb204_24;
  TNode<IntPtrT> phi_bb204_30;
  TNode<IntPtrT> phi_bb204_31;
  TNode<IntPtrT> phi_bb204_35;
  TNode<IntPtrT> phi_bb204_36;
  TNode<IntPtrT> tmp450;
  TNode<IntPtrT> tmp451;
  TNode<Union<HeapObject, TaggedIndex>> tmp452;
  TNode<IntPtrT> tmp453;
  TNode<Number> tmp454;
  if (block204.is_used()) {
    ca_.Bind(&block204, &phi_bb204_7, &phi_bb204_9, &phi_bb204_10, &phi_bb204_11, &phi_bb204_14, &phi_bb204_15, &phi_bb204_21, &phi_bb204_23, &phi_bb204_24, &phi_bb204_30, &phi_bb204_31, &phi_bb204_35, &phi_bb204_36);
    tmp450 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb204_36});
    tmp451 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp445}, TNode<IntPtrT>{tmp450});
    std::tie(tmp452, tmp453) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp444}, TNode<IntPtrT>{tmp451}).Flatten();
    tmp454 = Convert_Number_float32_0(state_, TNode<Float32T>{phi_bb188_25});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp452, tmp453}, tmp454);
    ca_.Goto(&block176, phi_bb204_7, tmp396, phi_bb204_9, phi_bb204_10, phi_bb204_11, phi_bb204_14, phi_bb204_15, phi_bb204_21);
  }

  TNode<IntPtrT> phi_bb205_7;
  TNode<IntPtrT> phi_bb205_9;
  TNode<IntPtrT> phi_bb205_10;
  TNode<IntPtrT> phi_bb205_11;
  TNode<IntPtrT> phi_bb205_14;
  TNode<BoolT> phi_bb205_15;
  TNode<IntPtrT> phi_bb205_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb205_23;
  TNode<IntPtrT> phi_bb205_24;
  TNode<IntPtrT> phi_bb205_30;
  TNode<IntPtrT> phi_bb205_31;
  TNode<IntPtrT> phi_bb205_35;
  TNode<IntPtrT> phi_bb205_36;
  if (block205.is_used()) {
    ca_.Bind(&block205, &phi_bb205_7, &phi_bb205_9, &phi_bb205_10, &phi_bb205_11, &phi_bb205_14, &phi_bb205_15, &phi_bb205_21, &phi_bb205_23, &phi_bb205_24, &phi_bb205_30, &phi_bb205_31, &phi_bb205_35, &phi_bb205_36);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb175_7;
  TNode<IntPtrT> phi_bb175_8;
  TNode<IntPtrT> phi_bb175_9;
  TNode<IntPtrT> phi_bb175_10;
  TNode<IntPtrT> phi_bb175_11;
  TNode<IntPtrT> phi_bb175_14;
  TNode<BoolT> phi_bb175_15;
  TNode<IntPtrT> phi_bb175_21;
  TNode<Uint32T> tmp455;
  TNode<BoolT> tmp456;
  if (block175.is_used()) {
    ca_.Bind(&block175, &phi_bb175_7, &phi_bb175_8, &phi_bb175_9, &phi_bb175_10, &phi_bb175_11, &phi_bb175_14, &phi_bb175_15, &phi_bb175_21);
    tmp455 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI64.raw_bit_field());
    tmp456 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp347}, TNode<Uint32T>{tmp455});
    ca_.Branch(tmp456, &block208, std::vector<compiler::Node*>{phi_bb175_7, phi_bb175_8, phi_bb175_9, phi_bb175_10, phi_bb175_11, phi_bb175_14, phi_bb175_15, phi_bb175_21}, &block209, std::vector<compiler::Node*>{phi_bb175_7, phi_bb175_8, phi_bb175_9, phi_bb175_10, phi_bb175_11, phi_bb175_14, phi_bb175_15, phi_bb175_21});
  }

  TNode<IntPtrT> phi_bb208_7;
  TNode<IntPtrT> phi_bb208_8;
  TNode<IntPtrT> phi_bb208_9;
  TNode<IntPtrT> phi_bb208_10;
  TNode<IntPtrT> phi_bb208_11;
  TNode<IntPtrT> phi_bb208_14;
  TNode<BoolT> phi_bb208_15;
  TNode<IntPtrT> phi_bb208_21;
  if (block208.is_used()) {
    ca_.Bind(&block208, &phi_bb208_7, &phi_bb208_8, &phi_bb208_9, &phi_bb208_10, &phi_bb208_11, &phi_bb208_14, &phi_bb208_15, &phi_bb208_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block211, phi_bb208_7, phi_bb208_8, phi_bb208_9, phi_bb208_10, phi_bb208_11, phi_bb208_14, phi_bb208_15, phi_bb208_21);
    } else {
      ca_.Goto(&block212, phi_bb208_7, phi_bb208_8, phi_bb208_9, phi_bb208_10, phi_bb208_11, phi_bb208_14, phi_bb208_15, phi_bb208_21);
    }
  }

  TNode<IntPtrT> phi_bb211_7;
  TNode<IntPtrT> phi_bb211_8;
  TNode<IntPtrT> phi_bb211_9;
  TNode<IntPtrT> phi_bb211_10;
  TNode<IntPtrT> phi_bb211_11;
  TNode<IntPtrT> phi_bb211_14;
  TNode<BoolT> phi_bb211_15;
  TNode<IntPtrT> phi_bb211_21;
  TNode<IntPtrT> tmp457;
  TNode<IntPtrT> tmp458;
  TNode<IntPtrT> tmp459;
  TNode<BoolT> tmp460;
  if (block211.is_used()) {
    ca_.Bind(&block211, &phi_bb211_7, &phi_bb211_8, &phi_bb211_9, &phi_bb211_10, &phi_bb211_11, &phi_bb211_14, &phi_bb211_15, &phi_bb211_21);
    tmp457 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp458 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb211_7}, TNode<IntPtrT>{tmp457});
    tmp459 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp460 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb211_7}, TNode<IntPtrT>{tmp459});
    ca_.Branch(tmp460, &block215, std::vector<compiler::Node*>{phi_bb211_8, phi_bb211_9, phi_bb211_10, phi_bb211_11, phi_bb211_14, phi_bb211_15, phi_bb211_21}, &block216, std::vector<compiler::Node*>{phi_bb211_8, phi_bb211_9, phi_bb211_10, phi_bb211_11, phi_bb211_14, phi_bb211_15, phi_bb211_21});
  }

  TNode<IntPtrT> phi_bb215_8;
  TNode<IntPtrT> phi_bb215_9;
  TNode<IntPtrT> phi_bb215_10;
  TNode<IntPtrT> phi_bb215_11;
  TNode<IntPtrT> phi_bb215_14;
  TNode<BoolT> phi_bb215_15;
  TNode<IntPtrT> phi_bb215_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp461;
  TNode<IntPtrT> tmp462;
  TNode<IntPtrT> tmp463;
  TNode<IntPtrT> tmp464;
  if (block215.is_used()) {
    ca_.Bind(&block215, &phi_bb215_8, &phi_bb215_9, &phi_bb215_10, &phi_bb215_11, &phi_bb215_14, &phi_bb215_15, &phi_bb215_21);
    std::tie(tmp461, tmp462) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb215_9}).Flatten();
    tmp463 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp464 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb215_9}, TNode<IntPtrT>{tmp463});
    ca_.Goto(&block214, phi_bb215_8, tmp464, phi_bb215_10, phi_bb215_11, phi_bb215_14, phi_bb215_15, phi_bb215_21, tmp461, tmp462);
  }

  TNode<IntPtrT> phi_bb216_8;
  TNode<IntPtrT> phi_bb216_9;
  TNode<IntPtrT> phi_bb216_10;
  TNode<IntPtrT> phi_bb216_11;
  TNode<IntPtrT> phi_bb216_14;
  TNode<BoolT> phi_bb216_15;
  TNode<IntPtrT> phi_bb216_21;
  if (block216.is_used()) {
    ca_.Bind(&block216, &phi_bb216_8, &phi_bb216_9, &phi_bb216_10, &phi_bb216_11, &phi_bb216_14, &phi_bb216_15, &phi_bb216_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block218, phi_bb216_8, phi_bb216_9, phi_bb216_10, phi_bb216_11, phi_bb216_14, phi_bb216_15, phi_bb216_21);
    } else {
      ca_.Goto(&block219, phi_bb216_8, phi_bb216_9, phi_bb216_10, phi_bb216_11, phi_bb216_14, phi_bb216_15, phi_bb216_21);
    }
  }

  TNode<IntPtrT> phi_bb218_8;
  TNode<IntPtrT> phi_bb218_9;
  TNode<IntPtrT> phi_bb218_10;
  TNode<IntPtrT> phi_bb218_11;
  TNode<IntPtrT> phi_bb218_14;
  TNode<BoolT> phi_bb218_15;
  TNode<IntPtrT> phi_bb218_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp465;
  TNode<IntPtrT> tmp466;
  TNode<IntPtrT> tmp467;
  TNode<IntPtrT> tmp468;
  if (block218.is_used()) {
    ca_.Bind(&block218, &phi_bb218_8, &phi_bb218_9, &phi_bb218_10, &phi_bb218_11, &phi_bb218_14, &phi_bb218_15, &phi_bb218_21);
    std::tie(tmp465, tmp466) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb218_11}).Flatten();
    tmp467 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp468 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb218_11}, TNode<IntPtrT>{tmp467});
    ca_.Goto(&block217, phi_bb218_8, phi_bb218_9, phi_bb218_10, tmp468, phi_bb218_14, phi_bb218_15, phi_bb218_21, tmp465, tmp466);
  }

  TNode<IntPtrT> phi_bb219_8;
  TNode<IntPtrT> phi_bb219_9;
  TNode<IntPtrT> phi_bb219_10;
  TNode<IntPtrT> phi_bb219_11;
  TNode<IntPtrT> phi_bb219_14;
  TNode<BoolT> phi_bb219_15;
  TNode<IntPtrT> phi_bb219_21;
  TNode<IntPtrT> tmp469;
  TNode<BoolT> tmp470;
  if (block219.is_used()) {
    ca_.Bind(&block219, &phi_bb219_8, &phi_bb219_9, &phi_bb219_10, &phi_bb219_11, &phi_bb219_14, &phi_bb219_15, &phi_bb219_21);
    tmp469 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp470 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb219_14}, TNode<IntPtrT>{tmp469});
    ca_.Branch(tmp470, &block221, std::vector<compiler::Node*>{phi_bb219_8, phi_bb219_9, phi_bb219_10, phi_bb219_11, phi_bb219_14, phi_bb219_15, phi_bb219_21}, &block222, std::vector<compiler::Node*>{phi_bb219_8, phi_bb219_9, phi_bb219_10, phi_bb219_11, phi_bb219_14, phi_bb219_15, phi_bb219_21});
  }

  TNode<IntPtrT> phi_bb221_8;
  TNode<IntPtrT> phi_bb221_9;
  TNode<IntPtrT> phi_bb221_10;
  TNode<IntPtrT> phi_bb221_11;
  TNode<IntPtrT> phi_bb221_14;
  TNode<BoolT> phi_bb221_15;
  TNode<IntPtrT> phi_bb221_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp471;
  TNode<IntPtrT> tmp472;
  TNode<IntPtrT> tmp473;
  TNode<BoolT> tmp474;
  if (block221.is_used()) {
    ca_.Bind(&block221, &phi_bb221_8, &phi_bb221_9, &phi_bb221_10, &phi_bb221_11, &phi_bb221_14, &phi_bb221_15, &phi_bb221_21);
    std::tie(tmp471, tmp472) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb221_14}).Flatten();
    tmp473 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp474 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block217, phi_bb221_8, phi_bb221_9, phi_bb221_10, phi_bb221_11, tmp473, tmp474, phi_bb221_21, tmp471, tmp472);
  }

  TNode<IntPtrT> phi_bb222_8;
  TNode<IntPtrT> phi_bb222_9;
  TNode<IntPtrT> phi_bb222_10;
  TNode<IntPtrT> phi_bb222_11;
  TNode<IntPtrT> phi_bb222_14;
  TNode<BoolT> phi_bb222_15;
  TNode<IntPtrT> phi_bb222_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp475;
  TNode<IntPtrT> tmp476;
  TNode<IntPtrT> tmp477;
  TNode<IntPtrT> tmp478;
  TNode<IntPtrT> tmp479;
  TNode<IntPtrT> tmp480;
  TNode<BoolT> tmp481;
  if (block222.is_used()) {
    ca_.Bind(&block222, &phi_bb222_8, &phi_bb222_9, &phi_bb222_10, &phi_bb222_11, &phi_bb222_14, &phi_bb222_15, &phi_bb222_21);
    std::tie(tmp475, tmp476) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb222_11}).Flatten();
    tmp477 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp478 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb222_11}, TNode<IntPtrT>{tmp477});
    tmp479 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp480 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp478}, TNode<IntPtrT>{tmp479});
    tmp481 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block217, phi_bb222_8, phi_bb222_9, phi_bb222_10, tmp480, tmp478, tmp481, phi_bb222_21, tmp475, tmp476);
  }

  TNode<IntPtrT> phi_bb217_8;
  TNode<IntPtrT> phi_bb217_9;
  TNode<IntPtrT> phi_bb217_10;
  TNode<IntPtrT> phi_bb217_11;
  TNode<IntPtrT> phi_bb217_14;
  TNode<BoolT> phi_bb217_15;
  TNode<IntPtrT> phi_bb217_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb217_23;
  TNode<IntPtrT> phi_bb217_24;
  if (block217.is_used()) {
    ca_.Bind(&block217, &phi_bb217_8, &phi_bb217_9, &phi_bb217_10, &phi_bb217_11, &phi_bb217_14, &phi_bb217_15, &phi_bb217_21, &phi_bb217_23, &phi_bb217_24);
    ca_.Goto(&block214, phi_bb217_8, phi_bb217_9, phi_bb217_10, phi_bb217_11, phi_bb217_14, phi_bb217_15, phi_bb217_21, phi_bb217_23, phi_bb217_24);
  }

  TNode<IntPtrT> phi_bb214_8;
  TNode<IntPtrT> phi_bb214_9;
  TNode<IntPtrT> phi_bb214_10;
  TNode<IntPtrT> phi_bb214_11;
  TNode<IntPtrT> phi_bb214_14;
  TNode<BoolT> phi_bb214_15;
  TNode<IntPtrT> phi_bb214_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb214_23;
  TNode<IntPtrT> phi_bb214_24;
  TNode<IntPtrT> tmp482;
  TNode<Union<HeapObject, TaggedIndex>> tmp483;
  TNode<IntPtrT> tmp484;
  TNode<IntPtrT> tmp485;
  TNode<UintPtrT> tmp486;
  TNode<UintPtrT> tmp487;
  TNode<BoolT> tmp488;
  if (block214.is_used()) {
    ca_.Bind(&block214, &phi_bb214_8, &phi_bb214_9, &phi_bb214_10, &phi_bb214_11, &phi_bb214_14, &phi_bb214_15, &phi_bb214_21, &phi_bb214_23, &phi_bb214_24);
    tmp482 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb214_23, phi_bb214_24});
    std::tie(tmp483, tmp484, tmp485) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp89}).Flatten();
    tmp486 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb214_21});
    tmp487 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp485});
    tmp488 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp486}, TNode<UintPtrT>{tmp487});
    ca_.Branch(tmp488, &block227, std::vector<compiler::Node*>{phi_bb214_8, phi_bb214_9, phi_bb214_10, phi_bb214_11, phi_bb214_14, phi_bb214_15, phi_bb214_21, phi_bb214_23, phi_bb214_24, phi_bb214_21, phi_bb214_21, phi_bb214_21, phi_bb214_21}, &block228, std::vector<compiler::Node*>{phi_bb214_8, phi_bb214_9, phi_bb214_10, phi_bb214_11, phi_bb214_14, phi_bb214_15, phi_bb214_21, phi_bb214_23, phi_bb214_24, phi_bb214_21, phi_bb214_21, phi_bb214_21, phi_bb214_21});
  }

  TNode<IntPtrT> phi_bb227_8;
  TNode<IntPtrT> phi_bb227_9;
  TNode<IntPtrT> phi_bb227_10;
  TNode<IntPtrT> phi_bb227_11;
  TNode<IntPtrT> phi_bb227_14;
  TNode<BoolT> phi_bb227_15;
  TNode<IntPtrT> phi_bb227_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb227_23;
  TNode<IntPtrT> phi_bb227_24;
  TNode<IntPtrT> phi_bb227_30;
  TNode<IntPtrT> phi_bb227_31;
  TNode<IntPtrT> phi_bb227_35;
  TNode<IntPtrT> phi_bb227_36;
  TNode<IntPtrT> tmp489;
  TNode<IntPtrT> tmp490;
  TNode<Union<HeapObject, TaggedIndex>> tmp491;
  TNode<IntPtrT> tmp492;
  TNode<BigInt> tmp493;
  if (block227.is_used()) {
    ca_.Bind(&block227, &phi_bb227_8, &phi_bb227_9, &phi_bb227_10, &phi_bb227_11, &phi_bb227_14, &phi_bb227_15, &phi_bb227_21, &phi_bb227_23, &phi_bb227_24, &phi_bb227_30, &phi_bb227_31, &phi_bb227_35, &phi_bb227_36);
    tmp489 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb227_36});
    tmp490 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp484}, TNode<IntPtrT>{tmp489});
    std::tie(tmp491, tmp492) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp483}, TNode<IntPtrT>{tmp490}).Flatten();
    tmp493 = ca_.CallBuiltin<BigInt>(Builtin::kI64ToBigInt, TNode<Object>(), tmp482);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp491, tmp492}, tmp493);
    ca_.Goto(&block213, tmp458, phi_bb227_8, phi_bb227_9, phi_bb227_10, phi_bb227_11, phi_bb227_14, phi_bb227_15, phi_bb227_21);
  }

  TNode<IntPtrT> phi_bb228_8;
  TNode<IntPtrT> phi_bb228_9;
  TNode<IntPtrT> phi_bb228_10;
  TNode<IntPtrT> phi_bb228_11;
  TNode<IntPtrT> phi_bb228_14;
  TNode<BoolT> phi_bb228_15;
  TNode<IntPtrT> phi_bb228_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb228_23;
  TNode<IntPtrT> phi_bb228_24;
  TNode<IntPtrT> phi_bb228_30;
  TNode<IntPtrT> phi_bb228_31;
  TNode<IntPtrT> phi_bb228_35;
  TNode<IntPtrT> phi_bb228_36;
  if (block228.is_used()) {
    ca_.Bind(&block228, &phi_bb228_8, &phi_bb228_9, &phi_bb228_10, &phi_bb228_11, &phi_bb228_14, &phi_bb228_15, &phi_bb228_21, &phi_bb228_23, &phi_bb228_24, &phi_bb228_30, &phi_bb228_31, &phi_bb228_35, &phi_bb228_36);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb212_7;
  TNode<IntPtrT> phi_bb212_8;
  TNode<IntPtrT> phi_bb212_9;
  TNode<IntPtrT> phi_bb212_10;
  TNode<IntPtrT> phi_bb212_11;
  TNode<IntPtrT> phi_bb212_14;
  TNode<BoolT> phi_bb212_15;
  TNode<IntPtrT> phi_bb212_21;
  TNode<IntPtrT> tmp494;
  TNode<IntPtrT> tmp495;
  TNode<IntPtrT> tmp496;
  TNode<BoolT> tmp497;
  if (block212.is_used()) {
    ca_.Bind(&block212, &phi_bb212_7, &phi_bb212_8, &phi_bb212_9, &phi_bb212_10, &phi_bb212_11, &phi_bb212_14, &phi_bb212_15, &phi_bb212_21);
    tmp494 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp495 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb212_7}, TNode<IntPtrT>{tmp494});
    tmp496 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp497 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb212_7}, TNode<IntPtrT>{tmp496});
    ca_.Branch(tmp497, &block232, std::vector<compiler::Node*>{phi_bb212_8, phi_bb212_9, phi_bb212_10, phi_bb212_11, phi_bb212_14, phi_bb212_15, phi_bb212_21}, &block233, std::vector<compiler::Node*>{phi_bb212_8, phi_bb212_9, phi_bb212_10, phi_bb212_11, phi_bb212_14, phi_bb212_15, phi_bb212_21});
  }

  TNode<IntPtrT> phi_bb232_8;
  TNode<IntPtrT> phi_bb232_9;
  TNode<IntPtrT> phi_bb232_10;
  TNode<IntPtrT> phi_bb232_11;
  TNode<IntPtrT> phi_bb232_14;
  TNode<BoolT> phi_bb232_15;
  TNode<IntPtrT> phi_bb232_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp498;
  TNode<IntPtrT> tmp499;
  TNode<IntPtrT> tmp500;
  TNode<IntPtrT> tmp501;
  if (block232.is_used()) {
    ca_.Bind(&block232, &phi_bb232_8, &phi_bb232_9, &phi_bb232_10, &phi_bb232_11, &phi_bb232_14, &phi_bb232_15, &phi_bb232_21);
    std::tie(tmp498, tmp499) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb232_9}).Flatten();
    tmp500 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp501 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb232_9}, TNode<IntPtrT>{tmp500});
    ca_.Goto(&block231, phi_bb232_8, tmp501, phi_bb232_10, phi_bb232_11, phi_bb232_14, phi_bb232_15, phi_bb232_21, tmp498, tmp499);
  }

  TNode<IntPtrT> phi_bb233_8;
  TNode<IntPtrT> phi_bb233_9;
  TNode<IntPtrT> phi_bb233_10;
  TNode<IntPtrT> phi_bb233_11;
  TNode<IntPtrT> phi_bb233_14;
  TNode<BoolT> phi_bb233_15;
  TNode<IntPtrT> phi_bb233_21;
  if (block233.is_used()) {
    ca_.Bind(&block233, &phi_bb233_8, &phi_bb233_9, &phi_bb233_10, &phi_bb233_11, &phi_bb233_14, &phi_bb233_15, &phi_bb233_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block235, phi_bb233_8, phi_bb233_9, phi_bb233_10, phi_bb233_11, phi_bb233_14, phi_bb233_15, phi_bb233_21);
    } else {
      ca_.Goto(&block236, phi_bb233_8, phi_bb233_9, phi_bb233_10, phi_bb233_11, phi_bb233_14, phi_bb233_15, phi_bb233_21);
    }
  }

  TNode<IntPtrT> phi_bb235_8;
  TNode<IntPtrT> phi_bb235_9;
  TNode<IntPtrT> phi_bb235_10;
  TNode<IntPtrT> phi_bb235_11;
  TNode<IntPtrT> phi_bb235_14;
  TNode<BoolT> phi_bb235_15;
  TNode<IntPtrT> phi_bb235_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp502;
  TNode<IntPtrT> tmp503;
  TNode<IntPtrT> tmp504;
  TNode<IntPtrT> tmp505;
  if (block235.is_used()) {
    ca_.Bind(&block235, &phi_bb235_8, &phi_bb235_9, &phi_bb235_10, &phi_bb235_11, &phi_bb235_14, &phi_bb235_15, &phi_bb235_21);
    std::tie(tmp502, tmp503) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb235_11}).Flatten();
    tmp504 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp505 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb235_11}, TNode<IntPtrT>{tmp504});
    ca_.Goto(&block234, phi_bb235_8, phi_bb235_9, phi_bb235_10, tmp505, phi_bb235_14, phi_bb235_15, phi_bb235_21, tmp502, tmp503);
  }

  TNode<IntPtrT> phi_bb236_8;
  TNode<IntPtrT> phi_bb236_9;
  TNode<IntPtrT> phi_bb236_10;
  TNode<IntPtrT> phi_bb236_11;
  TNode<IntPtrT> phi_bb236_14;
  TNode<BoolT> phi_bb236_15;
  TNode<IntPtrT> phi_bb236_21;
  TNode<IntPtrT> tmp506;
  TNode<BoolT> tmp507;
  if (block236.is_used()) {
    ca_.Bind(&block236, &phi_bb236_8, &phi_bb236_9, &phi_bb236_10, &phi_bb236_11, &phi_bb236_14, &phi_bb236_15, &phi_bb236_21);
    tmp506 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp507 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb236_14}, TNode<IntPtrT>{tmp506});
    ca_.Branch(tmp507, &block238, std::vector<compiler::Node*>{phi_bb236_8, phi_bb236_9, phi_bb236_10, phi_bb236_11, phi_bb236_14, phi_bb236_15, phi_bb236_21}, &block239, std::vector<compiler::Node*>{phi_bb236_8, phi_bb236_9, phi_bb236_10, phi_bb236_11, phi_bb236_14, phi_bb236_15, phi_bb236_21});
  }

  TNode<IntPtrT> phi_bb238_8;
  TNode<IntPtrT> phi_bb238_9;
  TNode<IntPtrT> phi_bb238_10;
  TNode<IntPtrT> phi_bb238_11;
  TNode<IntPtrT> phi_bb238_14;
  TNode<BoolT> phi_bb238_15;
  TNode<IntPtrT> phi_bb238_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp508;
  TNode<IntPtrT> tmp509;
  TNode<IntPtrT> tmp510;
  TNode<BoolT> tmp511;
  if (block238.is_used()) {
    ca_.Bind(&block238, &phi_bb238_8, &phi_bb238_9, &phi_bb238_10, &phi_bb238_11, &phi_bb238_14, &phi_bb238_15, &phi_bb238_21);
    std::tie(tmp508, tmp509) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb238_14}).Flatten();
    tmp510 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp511 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block234, phi_bb238_8, phi_bb238_9, phi_bb238_10, phi_bb238_11, tmp510, tmp511, phi_bb238_21, tmp508, tmp509);
  }

  TNode<IntPtrT> phi_bb239_8;
  TNode<IntPtrT> phi_bb239_9;
  TNode<IntPtrT> phi_bb239_10;
  TNode<IntPtrT> phi_bb239_11;
  TNode<IntPtrT> phi_bb239_14;
  TNode<BoolT> phi_bb239_15;
  TNode<IntPtrT> phi_bb239_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp512;
  TNode<IntPtrT> tmp513;
  TNode<IntPtrT> tmp514;
  TNode<IntPtrT> tmp515;
  TNode<IntPtrT> tmp516;
  TNode<IntPtrT> tmp517;
  TNode<BoolT> tmp518;
  if (block239.is_used()) {
    ca_.Bind(&block239, &phi_bb239_8, &phi_bb239_9, &phi_bb239_10, &phi_bb239_11, &phi_bb239_14, &phi_bb239_15, &phi_bb239_21);
    std::tie(tmp512, tmp513) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb239_11}).Flatten();
    tmp514 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp515 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb239_11}, TNode<IntPtrT>{tmp514});
    tmp516 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp517 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp515}, TNode<IntPtrT>{tmp516});
    tmp518 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block234, phi_bb239_8, phi_bb239_9, phi_bb239_10, tmp517, tmp515, tmp518, phi_bb239_21, tmp512, tmp513);
  }

  TNode<IntPtrT> phi_bb234_8;
  TNode<IntPtrT> phi_bb234_9;
  TNode<IntPtrT> phi_bb234_10;
  TNode<IntPtrT> phi_bb234_11;
  TNode<IntPtrT> phi_bb234_14;
  TNode<BoolT> phi_bb234_15;
  TNode<IntPtrT> phi_bb234_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb234_23;
  TNode<IntPtrT> phi_bb234_24;
  if (block234.is_used()) {
    ca_.Bind(&block234, &phi_bb234_8, &phi_bb234_9, &phi_bb234_10, &phi_bb234_11, &phi_bb234_14, &phi_bb234_15, &phi_bb234_21, &phi_bb234_23, &phi_bb234_24);
    ca_.Goto(&block231, phi_bb234_8, phi_bb234_9, phi_bb234_10, phi_bb234_11, phi_bb234_14, phi_bb234_15, phi_bb234_21, phi_bb234_23, phi_bb234_24);
  }

  TNode<IntPtrT> phi_bb231_8;
  TNode<IntPtrT> phi_bb231_9;
  TNode<IntPtrT> phi_bb231_10;
  TNode<IntPtrT> phi_bb231_11;
  TNode<IntPtrT> phi_bb231_14;
  TNode<BoolT> phi_bb231_15;
  TNode<IntPtrT> phi_bb231_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb231_23;
  TNode<IntPtrT> phi_bb231_24;
  TNode<IntPtrT> tmp519;
  TNode<IntPtrT> tmp520;
  TNode<IntPtrT> tmp521;
  TNode<BoolT> tmp522;
  if (block231.is_used()) {
    ca_.Bind(&block231, &phi_bb231_8, &phi_bb231_9, &phi_bb231_10, &phi_bb231_11, &phi_bb231_14, &phi_bb231_15, &phi_bb231_21, &phi_bb231_23, &phi_bb231_24);
    tmp519 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp520 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp495}, TNode<IntPtrT>{tmp519});
    tmp521 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp522 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp495}, TNode<IntPtrT>{tmp521});
    ca_.Branch(tmp522, &block241, std::vector<compiler::Node*>{phi_bb231_8, phi_bb231_9, phi_bb231_10, phi_bb231_11, phi_bb231_14, phi_bb231_15, phi_bb231_21, phi_bb231_23, phi_bb231_24}, &block242, std::vector<compiler::Node*>{phi_bb231_8, phi_bb231_9, phi_bb231_10, phi_bb231_11, phi_bb231_14, phi_bb231_15, phi_bb231_21, phi_bb231_23, phi_bb231_24});
  }

  TNode<IntPtrT> phi_bb241_8;
  TNode<IntPtrT> phi_bb241_9;
  TNode<IntPtrT> phi_bb241_10;
  TNode<IntPtrT> phi_bb241_11;
  TNode<IntPtrT> phi_bb241_14;
  TNode<BoolT> phi_bb241_15;
  TNode<IntPtrT> phi_bb241_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb241_23;
  TNode<IntPtrT> phi_bb241_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp523;
  TNode<IntPtrT> tmp524;
  TNode<IntPtrT> tmp525;
  TNode<IntPtrT> tmp526;
  if (block241.is_used()) {
    ca_.Bind(&block241, &phi_bb241_8, &phi_bb241_9, &phi_bb241_10, &phi_bb241_11, &phi_bb241_14, &phi_bb241_15, &phi_bb241_21, &phi_bb241_23, &phi_bb241_24);
    std::tie(tmp523, tmp524) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb241_9}).Flatten();
    tmp525 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp526 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb241_9}, TNode<IntPtrT>{tmp525});
    ca_.Goto(&block240, phi_bb241_8, tmp526, phi_bb241_10, phi_bb241_11, phi_bb241_14, phi_bb241_15, phi_bb241_21, phi_bb241_23, phi_bb241_24, tmp523, tmp524);
  }

  TNode<IntPtrT> phi_bb242_8;
  TNode<IntPtrT> phi_bb242_9;
  TNode<IntPtrT> phi_bb242_10;
  TNode<IntPtrT> phi_bb242_11;
  TNode<IntPtrT> phi_bb242_14;
  TNode<BoolT> phi_bb242_15;
  TNode<IntPtrT> phi_bb242_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb242_23;
  TNode<IntPtrT> phi_bb242_24;
  if (block242.is_used()) {
    ca_.Bind(&block242, &phi_bb242_8, &phi_bb242_9, &phi_bb242_10, &phi_bb242_11, &phi_bb242_14, &phi_bb242_15, &phi_bb242_21, &phi_bb242_23, &phi_bb242_24);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block244, phi_bb242_8, phi_bb242_9, phi_bb242_10, phi_bb242_11, phi_bb242_14, phi_bb242_15, phi_bb242_21, phi_bb242_23, phi_bb242_24);
    } else {
      ca_.Goto(&block245, phi_bb242_8, phi_bb242_9, phi_bb242_10, phi_bb242_11, phi_bb242_14, phi_bb242_15, phi_bb242_21, phi_bb242_23, phi_bb242_24);
    }
  }

  TNode<IntPtrT> phi_bb244_8;
  TNode<IntPtrT> phi_bb244_9;
  TNode<IntPtrT> phi_bb244_10;
  TNode<IntPtrT> phi_bb244_11;
  TNode<IntPtrT> phi_bb244_14;
  TNode<BoolT> phi_bb244_15;
  TNode<IntPtrT> phi_bb244_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb244_23;
  TNode<IntPtrT> phi_bb244_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp527;
  TNode<IntPtrT> tmp528;
  TNode<IntPtrT> tmp529;
  TNode<IntPtrT> tmp530;
  if (block244.is_used()) {
    ca_.Bind(&block244, &phi_bb244_8, &phi_bb244_9, &phi_bb244_10, &phi_bb244_11, &phi_bb244_14, &phi_bb244_15, &phi_bb244_21, &phi_bb244_23, &phi_bb244_24);
    std::tie(tmp527, tmp528) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb244_11}).Flatten();
    tmp529 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp530 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb244_11}, TNode<IntPtrT>{tmp529});
    ca_.Goto(&block243, phi_bb244_8, phi_bb244_9, phi_bb244_10, tmp530, phi_bb244_14, phi_bb244_15, phi_bb244_21, phi_bb244_23, phi_bb244_24, tmp527, tmp528);
  }

  TNode<IntPtrT> phi_bb245_8;
  TNode<IntPtrT> phi_bb245_9;
  TNode<IntPtrT> phi_bb245_10;
  TNode<IntPtrT> phi_bb245_11;
  TNode<IntPtrT> phi_bb245_14;
  TNode<BoolT> phi_bb245_15;
  TNode<IntPtrT> phi_bb245_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb245_23;
  TNode<IntPtrT> phi_bb245_24;
  TNode<IntPtrT> tmp531;
  TNode<BoolT> tmp532;
  if (block245.is_used()) {
    ca_.Bind(&block245, &phi_bb245_8, &phi_bb245_9, &phi_bb245_10, &phi_bb245_11, &phi_bb245_14, &phi_bb245_15, &phi_bb245_21, &phi_bb245_23, &phi_bb245_24);
    tmp531 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp532 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb245_14}, TNode<IntPtrT>{tmp531});
    ca_.Branch(tmp532, &block247, std::vector<compiler::Node*>{phi_bb245_8, phi_bb245_9, phi_bb245_10, phi_bb245_11, phi_bb245_14, phi_bb245_15, phi_bb245_21, phi_bb245_23, phi_bb245_24}, &block248, std::vector<compiler::Node*>{phi_bb245_8, phi_bb245_9, phi_bb245_10, phi_bb245_11, phi_bb245_14, phi_bb245_15, phi_bb245_21, phi_bb245_23, phi_bb245_24});
  }

  TNode<IntPtrT> phi_bb247_8;
  TNode<IntPtrT> phi_bb247_9;
  TNode<IntPtrT> phi_bb247_10;
  TNode<IntPtrT> phi_bb247_11;
  TNode<IntPtrT> phi_bb247_14;
  TNode<BoolT> phi_bb247_15;
  TNode<IntPtrT> phi_bb247_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb247_23;
  TNode<IntPtrT> phi_bb247_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp533;
  TNode<IntPtrT> tmp534;
  TNode<IntPtrT> tmp535;
  TNode<BoolT> tmp536;
  if (block247.is_used()) {
    ca_.Bind(&block247, &phi_bb247_8, &phi_bb247_9, &phi_bb247_10, &phi_bb247_11, &phi_bb247_14, &phi_bb247_15, &phi_bb247_21, &phi_bb247_23, &phi_bb247_24);
    std::tie(tmp533, tmp534) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb247_14}).Flatten();
    tmp535 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp536 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block243, phi_bb247_8, phi_bb247_9, phi_bb247_10, phi_bb247_11, tmp535, tmp536, phi_bb247_21, phi_bb247_23, phi_bb247_24, tmp533, tmp534);
  }

  TNode<IntPtrT> phi_bb248_8;
  TNode<IntPtrT> phi_bb248_9;
  TNode<IntPtrT> phi_bb248_10;
  TNode<IntPtrT> phi_bb248_11;
  TNode<IntPtrT> phi_bb248_14;
  TNode<BoolT> phi_bb248_15;
  TNode<IntPtrT> phi_bb248_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb248_23;
  TNode<IntPtrT> phi_bb248_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp537;
  TNode<IntPtrT> tmp538;
  TNode<IntPtrT> tmp539;
  TNode<IntPtrT> tmp540;
  TNode<IntPtrT> tmp541;
  TNode<IntPtrT> tmp542;
  TNode<BoolT> tmp543;
  if (block248.is_used()) {
    ca_.Bind(&block248, &phi_bb248_8, &phi_bb248_9, &phi_bb248_10, &phi_bb248_11, &phi_bb248_14, &phi_bb248_15, &phi_bb248_21, &phi_bb248_23, &phi_bb248_24);
    std::tie(tmp537, tmp538) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb248_11}).Flatten();
    tmp539 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp540 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb248_11}, TNode<IntPtrT>{tmp539});
    tmp541 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp542 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp540}, TNode<IntPtrT>{tmp541});
    tmp543 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block243, phi_bb248_8, phi_bb248_9, phi_bb248_10, tmp542, tmp540, tmp543, phi_bb248_21, phi_bb248_23, phi_bb248_24, tmp537, tmp538);
  }

  TNode<IntPtrT> phi_bb243_8;
  TNode<IntPtrT> phi_bb243_9;
  TNode<IntPtrT> phi_bb243_10;
  TNode<IntPtrT> phi_bb243_11;
  TNode<IntPtrT> phi_bb243_14;
  TNode<BoolT> phi_bb243_15;
  TNode<IntPtrT> phi_bb243_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb243_23;
  TNode<IntPtrT> phi_bb243_24;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb243_25;
  TNode<IntPtrT> phi_bb243_26;
  if (block243.is_used()) {
    ca_.Bind(&block243, &phi_bb243_8, &phi_bb243_9, &phi_bb243_10, &phi_bb243_11, &phi_bb243_14, &phi_bb243_15, &phi_bb243_21, &phi_bb243_23, &phi_bb243_24, &phi_bb243_25, &phi_bb243_26);
    ca_.Goto(&block240, phi_bb243_8, phi_bb243_9, phi_bb243_10, phi_bb243_11, phi_bb243_14, phi_bb243_15, phi_bb243_21, phi_bb243_23, phi_bb243_24, phi_bb243_25, phi_bb243_26);
  }

  TNode<IntPtrT> phi_bb240_8;
  TNode<IntPtrT> phi_bb240_9;
  TNode<IntPtrT> phi_bb240_10;
  TNode<IntPtrT> phi_bb240_11;
  TNode<IntPtrT> phi_bb240_14;
  TNode<BoolT> phi_bb240_15;
  TNode<IntPtrT> phi_bb240_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb240_23;
  TNode<IntPtrT> phi_bb240_24;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb240_25;
  TNode<IntPtrT> phi_bb240_26;
  TNode<IntPtrT> tmp544;
  TNode<IntPtrT> tmp545;
  TNode<Union<HeapObject, TaggedIndex>> tmp546;
  TNode<IntPtrT> tmp547;
  TNode<IntPtrT> tmp548;
  TNode<UintPtrT> tmp549;
  TNode<UintPtrT> tmp550;
  TNode<BoolT> tmp551;
  if (block240.is_used()) {
    ca_.Bind(&block240, &phi_bb240_8, &phi_bb240_9, &phi_bb240_10, &phi_bb240_11, &phi_bb240_14, &phi_bb240_15, &phi_bb240_21, &phi_bb240_23, &phi_bb240_24, &phi_bb240_25, &phi_bb240_26);
    tmp544 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb240_23, phi_bb240_24});
    tmp545 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb240_25, phi_bb240_26});
    std::tie(tmp546, tmp547, tmp548) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp89}).Flatten();
    tmp549 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb240_21});
    tmp550 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp548});
    tmp551 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp549}, TNode<UintPtrT>{tmp550});
    ca_.Branch(tmp551, &block253, std::vector<compiler::Node*>{phi_bb240_8, phi_bb240_9, phi_bb240_10, phi_bb240_11, phi_bb240_14, phi_bb240_15, phi_bb240_21, phi_bb240_23, phi_bb240_24, phi_bb240_25, phi_bb240_26, phi_bb240_21, phi_bb240_21, phi_bb240_21, phi_bb240_21}, &block254, std::vector<compiler::Node*>{phi_bb240_8, phi_bb240_9, phi_bb240_10, phi_bb240_11, phi_bb240_14, phi_bb240_15, phi_bb240_21, phi_bb240_23, phi_bb240_24, phi_bb240_25, phi_bb240_26, phi_bb240_21, phi_bb240_21, phi_bb240_21, phi_bb240_21});
  }

  TNode<IntPtrT> phi_bb253_8;
  TNode<IntPtrT> phi_bb253_9;
  TNode<IntPtrT> phi_bb253_10;
  TNode<IntPtrT> phi_bb253_11;
  TNode<IntPtrT> phi_bb253_14;
  TNode<BoolT> phi_bb253_15;
  TNode<IntPtrT> phi_bb253_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb253_23;
  TNode<IntPtrT> phi_bb253_24;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb253_25;
  TNode<IntPtrT> phi_bb253_26;
  TNode<IntPtrT> phi_bb253_33;
  TNode<IntPtrT> phi_bb253_34;
  TNode<IntPtrT> phi_bb253_38;
  TNode<IntPtrT> phi_bb253_39;
  TNode<IntPtrT> tmp552;
  TNode<IntPtrT> tmp553;
  TNode<Union<HeapObject, TaggedIndex>> tmp554;
  TNode<IntPtrT> tmp555;
  TNode<BigInt> tmp556;
  if (block253.is_used()) {
    ca_.Bind(&block253, &phi_bb253_8, &phi_bb253_9, &phi_bb253_10, &phi_bb253_11, &phi_bb253_14, &phi_bb253_15, &phi_bb253_21, &phi_bb253_23, &phi_bb253_24, &phi_bb253_25, &phi_bb253_26, &phi_bb253_33, &phi_bb253_34, &phi_bb253_38, &phi_bb253_39);
    tmp552 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb253_39});
    tmp553 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp547}, TNode<IntPtrT>{tmp552});
    std::tie(tmp554, tmp555) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp546}, TNode<IntPtrT>{tmp553}).Flatten();
    tmp556 = ca_.CallBuiltin<BigInt>(Builtin::kI32PairToBigInt, TNode<Object>(), tmp544, tmp545);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp554, tmp555}, tmp556);
    ca_.Goto(&block213, tmp520, phi_bb253_8, phi_bb253_9, phi_bb253_10, phi_bb253_11, phi_bb253_14, phi_bb253_15, phi_bb253_21);
  }

  TNode<IntPtrT> phi_bb254_8;
  TNode<IntPtrT> phi_bb254_9;
  TNode<IntPtrT> phi_bb254_10;
  TNode<IntPtrT> phi_bb254_11;
  TNode<IntPtrT> phi_bb254_14;
  TNode<BoolT> phi_bb254_15;
  TNode<IntPtrT> phi_bb254_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb254_23;
  TNode<IntPtrT> phi_bb254_24;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb254_25;
  TNode<IntPtrT> phi_bb254_26;
  TNode<IntPtrT> phi_bb254_33;
  TNode<IntPtrT> phi_bb254_34;
  TNode<IntPtrT> phi_bb254_38;
  TNode<IntPtrT> phi_bb254_39;
  if (block254.is_used()) {
    ca_.Bind(&block254, &phi_bb254_8, &phi_bb254_9, &phi_bb254_10, &phi_bb254_11, &phi_bb254_14, &phi_bb254_15, &phi_bb254_21, &phi_bb254_23, &phi_bb254_24, &phi_bb254_25, &phi_bb254_26, &phi_bb254_33, &phi_bb254_34, &phi_bb254_38, &phi_bb254_39);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb213_7;
  TNode<IntPtrT> phi_bb213_8;
  TNode<IntPtrT> phi_bb213_9;
  TNode<IntPtrT> phi_bb213_10;
  TNode<IntPtrT> phi_bb213_11;
  TNode<IntPtrT> phi_bb213_14;
  TNode<BoolT> phi_bb213_15;
  TNode<IntPtrT> phi_bb213_21;
  if (block213.is_used()) {
    ca_.Bind(&block213, &phi_bb213_7, &phi_bb213_8, &phi_bb213_9, &phi_bb213_10, &phi_bb213_11, &phi_bb213_14, &phi_bb213_15, &phi_bb213_21);
    ca_.Goto(&block210, phi_bb213_7, phi_bb213_8, phi_bb213_9, phi_bb213_10, phi_bb213_11, phi_bb213_14, phi_bb213_15, phi_bb213_21);
  }

  TNode<IntPtrT> phi_bb209_7;
  TNode<IntPtrT> phi_bb209_8;
  TNode<IntPtrT> phi_bb209_9;
  TNode<IntPtrT> phi_bb209_10;
  TNode<IntPtrT> phi_bb209_11;
  TNode<IntPtrT> phi_bb209_14;
  TNode<BoolT> phi_bb209_15;
  TNode<IntPtrT> phi_bb209_21;
  TNode<Uint32T> tmp557;
  TNode<BoolT> tmp558;
  if (block209.is_used()) {
    ca_.Bind(&block209, &phi_bb209_7, &phi_bb209_8, &phi_bb209_9, &phi_bb209_10, &phi_bb209_11, &phi_bb209_14, &phi_bb209_15, &phi_bb209_21);
    tmp557 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF64.raw_bit_field());
    tmp558 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp347}, TNode<Uint32T>{tmp557});
    ca_.Branch(tmp558, &block257, std::vector<compiler::Node*>{phi_bb209_7, phi_bb209_8, phi_bb209_9, phi_bb209_10, phi_bb209_11, phi_bb209_14, phi_bb209_15, phi_bb209_21}, &block258, std::vector<compiler::Node*>{phi_bb209_7, phi_bb209_8, phi_bb209_9, phi_bb209_10, phi_bb209_11, phi_bb209_14, phi_bb209_15, phi_bb209_21});
  }

  TNode<IntPtrT> phi_bb257_7;
  TNode<IntPtrT> phi_bb257_8;
  TNode<IntPtrT> phi_bb257_9;
  TNode<IntPtrT> phi_bb257_10;
  TNode<IntPtrT> phi_bb257_11;
  TNode<IntPtrT> phi_bb257_14;
  TNode<BoolT> phi_bb257_15;
  TNode<IntPtrT> phi_bb257_21;
  TNode<IntPtrT> tmp559;
  TNode<IntPtrT> tmp560;
  TNode<IntPtrT> tmp561;
  TNode<BoolT> tmp562;
  if (block257.is_used()) {
    ca_.Bind(&block257, &phi_bb257_7, &phi_bb257_8, &phi_bb257_9, &phi_bb257_10, &phi_bb257_11, &phi_bb257_14, &phi_bb257_15, &phi_bb257_21);
    tmp559 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp560 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb257_8}, TNode<IntPtrT>{tmp559});
    tmp561 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp562 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb257_8}, TNode<IntPtrT>{tmp561});
    ca_.Branch(tmp562, &block261, std::vector<compiler::Node*>{phi_bb257_7, phi_bb257_9, phi_bb257_10, phi_bb257_11, phi_bb257_14, phi_bb257_15, phi_bb257_21}, &block262, std::vector<compiler::Node*>{phi_bb257_7, phi_bb257_9, phi_bb257_10, phi_bb257_11, phi_bb257_14, phi_bb257_15, phi_bb257_21});
  }

  TNode<IntPtrT> phi_bb261_7;
  TNode<IntPtrT> phi_bb261_9;
  TNode<IntPtrT> phi_bb261_10;
  TNode<IntPtrT> phi_bb261_11;
  TNode<IntPtrT> phi_bb261_14;
  TNode<BoolT> phi_bb261_15;
  TNode<IntPtrT> phi_bb261_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp563;
  TNode<IntPtrT> tmp564;
  TNode<IntPtrT> tmp565;
  TNode<IntPtrT> tmp566;
  if (block261.is_used()) {
    ca_.Bind(&block261, &phi_bb261_7, &phi_bb261_9, &phi_bb261_10, &phi_bb261_11, &phi_bb261_14, &phi_bb261_15, &phi_bb261_21);
    std::tie(tmp563, tmp564) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb261_10}).Flatten();
    tmp565 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp566 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb261_10}, TNode<IntPtrT>{tmp565});
    ca_.Goto(&block260, phi_bb261_7, phi_bb261_9, tmp566, phi_bb261_11, phi_bb261_14, phi_bb261_15, phi_bb261_21, tmp563, tmp564);
  }

  TNode<IntPtrT> phi_bb262_7;
  TNode<IntPtrT> phi_bb262_9;
  TNode<IntPtrT> phi_bb262_10;
  TNode<IntPtrT> phi_bb262_11;
  TNode<IntPtrT> phi_bb262_14;
  TNode<BoolT> phi_bb262_15;
  TNode<IntPtrT> phi_bb262_21;
  if (block262.is_used()) {
    ca_.Bind(&block262, &phi_bb262_7, &phi_bb262_9, &phi_bb262_10, &phi_bb262_11, &phi_bb262_14, &phi_bb262_15, &phi_bb262_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block263, phi_bb262_7, phi_bb262_9, phi_bb262_10, phi_bb262_11, phi_bb262_14, phi_bb262_15, phi_bb262_21);
    } else {
      ca_.Goto(&block264, phi_bb262_7, phi_bb262_9, phi_bb262_10, phi_bb262_11, phi_bb262_14, phi_bb262_15, phi_bb262_21);
    }
  }

  TNode<IntPtrT> phi_bb263_7;
  TNode<IntPtrT> phi_bb263_9;
  TNode<IntPtrT> phi_bb263_10;
  TNode<IntPtrT> phi_bb263_11;
  TNode<IntPtrT> phi_bb263_14;
  TNode<BoolT> phi_bb263_15;
  TNode<IntPtrT> phi_bb263_21;
  if (block263.is_used()) {
    ca_.Bind(&block263, &phi_bb263_7, &phi_bb263_9, &phi_bb263_10, &phi_bb263_11, &phi_bb263_14, &phi_bb263_15, &phi_bb263_21);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block267, phi_bb263_7, phi_bb263_9, phi_bb263_10, phi_bb263_11, phi_bb263_14, phi_bb263_15, phi_bb263_21);
    } else {
      ca_.Goto(&block268, phi_bb263_7, phi_bb263_9, phi_bb263_10, phi_bb263_11, phi_bb263_14, phi_bb263_15, phi_bb263_21);
    }
  }

  TNode<IntPtrT> phi_bb267_7;
  TNode<IntPtrT> phi_bb267_9;
  TNode<IntPtrT> phi_bb267_10;
  TNode<IntPtrT> phi_bb267_11;
  TNode<IntPtrT> phi_bb267_14;
  TNode<BoolT> phi_bb267_15;
  TNode<IntPtrT> phi_bb267_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp567;
  TNode<IntPtrT> tmp568;
  TNode<IntPtrT> tmp569;
  TNode<IntPtrT> tmp570;
  if (block267.is_used()) {
    ca_.Bind(&block267, &phi_bb267_7, &phi_bb267_9, &phi_bb267_10, &phi_bb267_11, &phi_bb267_14, &phi_bb267_15, &phi_bb267_21);
    std::tie(tmp567, tmp568) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb267_11}).Flatten();
    tmp569 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp570 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb267_11}, TNode<IntPtrT>{tmp569});
    ca_.Goto(&block266, phi_bb267_7, phi_bb267_9, phi_bb267_10, tmp570, phi_bb267_14, phi_bb267_15, phi_bb267_21, tmp567, tmp568);
  }

  TNode<IntPtrT> phi_bb268_7;
  TNode<IntPtrT> phi_bb268_9;
  TNode<IntPtrT> phi_bb268_10;
  TNode<IntPtrT> phi_bb268_11;
  TNode<IntPtrT> phi_bb268_14;
  TNode<BoolT> phi_bb268_15;
  TNode<IntPtrT> phi_bb268_21;
  TNode<IntPtrT> tmp571;
  TNode<BoolT> tmp572;
  if (block268.is_used()) {
    ca_.Bind(&block268, &phi_bb268_7, &phi_bb268_9, &phi_bb268_10, &phi_bb268_11, &phi_bb268_14, &phi_bb268_15, &phi_bb268_21);
    tmp571 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp572 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb268_14}, TNode<IntPtrT>{tmp571});
    ca_.Branch(tmp572, &block270, std::vector<compiler::Node*>{phi_bb268_7, phi_bb268_9, phi_bb268_10, phi_bb268_11, phi_bb268_14, phi_bb268_15, phi_bb268_21}, &block271, std::vector<compiler::Node*>{phi_bb268_7, phi_bb268_9, phi_bb268_10, phi_bb268_11, phi_bb268_14, phi_bb268_15, phi_bb268_21});
  }

  TNode<IntPtrT> phi_bb270_7;
  TNode<IntPtrT> phi_bb270_9;
  TNode<IntPtrT> phi_bb270_10;
  TNode<IntPtrT> phi_bb270_11;
  TNode<IntPtrT> phi_bb270_14;
  TNode<BoolT> phi_bb270_15;
  TNode<IntPtrT> phi_bb270_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp573;
  TNode<IntPtrT> tmp574;
  TNode<IntPtrT> tmp575;
  TNode<BoolT> tmp576;
  if (block270.is_used()) {
    ca_.Bind(&block270, &phi_bb270_7, &phi_bb270_9, &phi_bb270_10, &phi_bb270_11, &phi_bb270_14, &phi_bb270_15, &phi_bb270_21);
    std::tie(tmp573, tmp574) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb270_14}).Flatten();
    tmp575 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp576 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block266, phi_bb270_7, phi_bb270_9, phi_bb270_10, phi_bb270_11, tmp575, tmp576, phi_bb270_21, tmp573, tmp574);
  }

  TNode<IntPtrT> phi_bb271_7;
  TNode<IntPtrT> phi_bb271_9;
  TNode<IntPtrT> phi_bb271_10;
  TNode<IntPtrT> phi_bb271_11;
  TNode<IntPtrT> phi_bb271_14;
  TNode<BoolT> phi_bb271_15;
  TNode<IntPtrT> phi_bb271_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp577;
  TNode<IntPtrT> tmp578;
  TNode<IntPtrT> tmp579;
  TNode<IntPtrT> tmp580;
  TNode<IntPtrT> tmp581;
  TNode<IntPtrT> tmp582;
  TNode<BoolT> tmp583;
  if (block271.is_used()) {
    ca_.Bind(&block271, &phi_bb271_7, &phi_bb271_9, &phi_bb271_10, &phi_bb271_11, &phi_bb271_14, &phi_bb271_15, &phi_bb271_21);
    std::tie(tmp577, tmp578) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb271_11}).Flatten();
    tmp579 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp580 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb271_11}, TNode<IntPtrT>{tmp579});
    tmp581 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp582 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp580}, TNode<IntPtrT>{tmp581});
    tmp583 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block266, phi_bb271_7, phi_bb271_9, phi_bb271_10, tmp582, tmp580, tmp583, phi_bb271_21, tmp577, tmp578);
  }

  TNode<IntPtrT> phi_bb266_7;
  TNode<IntPtrT> phi_bb266_9;
  TNode<IntPtrT> phi_bb266_10;
  TNode<IntPtrT> phi_bb266_11;
  TNode<IntPtrT> phi_bb266_14;
  TNode<BoolT> phi_bb266_15;
  TNode<IntPtrT> phi_bb266_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb266_23;
  TNode<IntPtrT> phi_bb266_24;
  if (block266.is_used()) {
    ca_.Bind(&block266, &phi_bb266_7, &phi_bb266_9, &phi_bb266_10, &phi_bb266_11, &phi_bb266_14, &phi_bb266_15, &phi_bb266_21, &phi_bb266_23, &phi_bb266_24);
    ca_.Goto(&block260, phi_bb266_7, phi_bb266_9, phi_bb266_10, phi_bb266_11, phi_bb266_14, phi_bb266_15, phi_bb266_21, phi_bb266_23, phi_bb266_24);
  }

  TNode<IntPtrT> phi_bb264_7;
  TNode<IntPtrT> phi_bb264_9;
  TNode<IntPtrT> phi_bb264_10;
  TNode<IntPtrT> phi_bb264_11;
  TNode<IntPtrT> phi_bb264_14;
  TNode<BoolT> phi_bb264_15;
  TNode<IntPtrT> phi_bb264_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp584;
  TNode<IntPtrT> tmp585;
  TNode<IntPtrT> tmp586;
  TNode<IntPtrT> tmp587;
  TNode<BoolT> tmp588;
  if (block264.is_used()) {
    ca_.Bind(&block264, &phi_bb264_7, &phi_bb264_9, &phi_bb264_10, &phi_bb264_11, &phi_bb264_14, &phi_bb264_15, &phi_bb264_21);
    std::tie(tmp584, tmp585) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp330}, TNode<IntPtrT>{phi_bb264_11}).Flatten();
    tmp586 = FromConstexpr_intptr_constexpr_int31_0(state_, (CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull))), (SizeOf_intptr_0(state_)))));
    tmp587 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb264_11}, TNode<IntPtrT>{tmp586});
    tmp588 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block260, phi_bb264_7, phi_bb264_9, phi_bb264_10, tmp587, phi_bb264_14, tmp588, phi_bb264_21, tmp584, tmp585);
  }

  TNode<IntPtrT> phi_bb260_7;
  TNode<IntPtrT> phi_bb260_9;
  TNode<IntPtrT> phi_bb260_10;
  TNode<IntPtrT> phi_bb260_11;
  TNode<IntPtrT> phi_bb260_14;
  TNode<BoolT> phi_bb260_15;
  TNode<IntPtrT> phi_bb260_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb260_23;
  TNode<IntPtrT> phi_bb260_24;
  TNode<Union<HeapObject, TaggedIndex>> tmp589;
  TNode<IntPtrT> tmp590;
  TNode<Float64T> tmp591;
  TNode<Union<HeapObject, TaggedIndex>> tmp592;
  TNode<IntPtrT> tmp593;
  TNode<IntPtrT> tmp594;
  TNode<UintPtrT> tmp595;
  TNode<UintPtrT> tmp596;
  TNode<BoolT> tmp597;
  if (block260.is_used()) {
    ca_.Bind(&block260, &phi_bb260_7, &phi_bb260_9, &phi_bb260_10, &phi_bb260_11, &phi_bb260_14, &phi_bb260_15, &phi_bb260_21, &phi_bb260_23, &phi_bb260_24);
    std::tie(tmp589, tmp590) = RefCast_float64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb260_23}, TNode<IntPtrT>{phi_bb260_24}, TorqueStructUnsafe_0{}}).Flatten();
    tmp591 = CodeStubAssembler(state_).LoadReference<Float64T>(CodeStubAssembler::Reference{tmp589, tmp590});
    std::tie(tmp592, tmp593, tmp594) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp89}).Flatten();
    tmp595 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb260_21});
    tmp596 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp594});
    tmp597 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp595}, TNode<UintPtrT>{tmp596});
    ca_.Branch(tmp597, &block276, std::vector<compiler::Node*>{phi_bb260_7, phi_bb260_9, phi_bb260_10, phi_bb260_11, phi_bb260_14, phi_bb260_15, phi_bb260_21, phi_bb260_23, phi_bb260_24, phi_bb260_21, phi_bb260_21, phi_bb260_21, phi_bb260_21}, &block277, std::vector<compiler::Node*>{phi_bb260_7, phi_bb260_9, phi_bb260_10, phi_bb260_11, phi_bb260_14, phi_bb260_15, phi_bb260_21, phi_bb260_23, phi_bb260_24, phi_bb260_21, phi_bb260_21, phi_bb260_21, phi_bb260_21});
  }

  TNode<IntPtrT> phi_bb276_7;
  TNode<IntPtrT> phi_bb276_9;
  TNode<IntPtrT> phi_bb276_10;
  TNode<IntPtrT> phi_bb276_11;
  TNode<IntPtrT> phi_bb276_14;
  TNode<BoolT> phi_bb276_15;
  TNode<IntPtrT> phi_bb276_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb276_23;
  TNode<IntPtrT> phi_bb276_24;
  TNode<IntPtrT> phi_bb276_30;
  TNode<IntPtrT> phi_bb276_31;
  TNode<IntPtrT> phi_bb276_35;
  TNode<IntPtrT> phi_bb276_36;
  TNode<IntPtrT> tmp598;
  TNode<IntPtrT> tmp599;
  TNode<Union<HeapObject, TaggedIndex>> tmp600;
  TNode<IntPtrT> tmp601;
  TNode<Number> tmp602;
  if (block276.is_used()) {
    ca_.Bind(&block276, &phi_bb276_7, &phi_bb276_9, &phi_bb276_10, &phi_bb276_11, &phi_bb276_14, &phi_bb276_15, &phi_bb276_21, &phi_bb276_23, &phi_bb276_24, &phi_bb276_30, &phi_bb276_31, &phi_bb276_35, &phi_bb276_36);
    tmp598 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb276_36});
    tmp599 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp593}, TNode<IntPtrT>{tmp598});
    std::tie(tmp600, tmp601) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp592}, TNode<IntPtrT>{tmp599}).Flatten();
    tmp602 = Convert_Number_float64_0(state_, TNode<Float64T>{tmp591});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp600, tmp601}, tmp602);
    ca_.Goto(&block259, phi_bb276_7, tmp560, phi_bb276_9, phi_bb276_10, phi_bb276_11, phi_bb276_14, phi_bb276_15, phi_bb276_21);
  }

  TNode<IntPtrT> phi_bb277_7;
  TNode<IntPtrT> phi_bb277_9;
  TNode<IntPtrT> phi_bb277_10;
  TNode<IntPtrT> phi_bb277_11;
  TNode<IntPtrT> phi_bb277_14;
  TNode<BoolT> phi_bb277_15;
  TNode<IntPtrT> phi_bb277_21;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb277_23;
  TNode<IntPtrT> phi_bb277_24;
  TNode<IntPtrT> phi_bb277_30;
  TNode<IntPtrT> phi_bb277_31;
  TNode<IntPtrT> phi_bb277_35;
  TNode<IntPtrT> phi_bb277_36;
  if (block277.is_used()) {
    ca_.Bind(&block277, &phi_bb277_7, &phi_bb277_9, &phi_bb277_10, &phi_bb277_11, &phi_bb277_14, &phi_bb277_15, &phi_bb277_21, &phi_bb277_23, &phi_bb277_24, &phi_bb277_30, &phi_bb277_31, &phi_bb277_35, &phi_bb277_36);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb258_7;
  TNode<IntPtrT> phi_bb258_8;
  TNode<IntPtrT> phi_bb258_9;
  TNode<IntPtrT> phi_bb258_10;
  TNode<IntPtrT> phi_bb258_11;
  TNode<IntPtrT> phi_bb258_14;
  TNode<BoolT> phi_bb258_15;
  TNode<IntPtrT> phi_bb258_21;
  TNode<Uint32T> tmp603;
  TNode<Uint32T> tmp604;
  TNode<Uint32T> tmp605;
  TNode<BoolT> tmp606;
  if (block258.is_used()) {
    ca_.Bind(&block258, &phi_bb258_7, &phi_bb258_8, &phi_bb258_9, &phi_bb258_10, &phi_bb258_11, &phi_bb258_14, &phi_bb258_15, &phi_bb258_21);
    tmp603 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp604 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp347}, TNode<Uint32T>{tmp603});
    tmp605 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp606 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp604}, TNode<Uint32T>{tmp605});
    ca_.Branch(tmp606, &block280, std::vector<compiler::Node*>{phi_bb258_7, phi_bb258_8, phi_bb258_9, phi_bb258_10, phi_bb258_11, phi_bb258_14, phi_bb258_15, phi_bb258_21}, &block281, std::vector<compiler::Node*>{phi_bb258_7, phi_bb258_8, phi_bb258_9, phi_bb258_10, phi_bb258_11, phi_bb258_14, phi_bb258_15, phi_bb258_21});
  }

  TNode<IntPtrT> phi_bb281_7;
  TNode<IntPtrT> phi_bb281_8;
  TNode<IntPtrT> phi_bb281_9;
  TNode<IntPtrT> phi_bb281_10;
  TNode<IntPtrT> phi_bb281_11;
  TNode<IntPtrT> phi_bb281_14;
  TNode<BoolT> phi_bb281_15;
  TNode<IntPtrT> phi_bb281_21;
  if (block281.is_used()) {
    ca_.Bind(&block281, &phi_bb281_7, &phi_bb281_8, &phi_bb281_9, &phi_bb281_10, &phi_bb281_11, &phi_bb281_14, &phi_bb281_15, &phi_bb281_21);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/js-to-wasm.tq", 1051});
      CodeStubAssembler(state_).FailAssert("Torque assert '(retType & kValueTypeIsRefBit) != 0' failed", pos_stack);
    }
  }

  TNode<IntPtrT> phi_bb280_7;
  TNode<IntPtrT> phi_bb280_8;
  TNode<IntPtrT> phi_bb280_9;
  TNode<IntPtrT> phi_bb280_10;
  TNode<IntPtrT> phi_bb280_11;
  TNode<IntPtrT> phi_bb280_14;
  TNode<BoolT> phi_bb280_15;
  TNode<IntPtrT> phi_bb280_21;
  TNode<Union<HeapObject, TaggedIndex>> tmp607;
  TNode<IntPtrT> tmp608;
  TNode<IntPtrT> tmp609;
  TNode<UintPtrT> tmp610;
  TNode<UintPtrT> tmp611;
  TNode<BoolT> tmp612;
  if (block280.is_used()) {
    ca_.Bind(&block280, &phi_bb280_7, &phi_bb280_8, &phi_bb280_9, &phi_bb280_10, &phi_bb280_11, &phi_bb280_14, &phi_bb280_15, &phi_bb280_21);
    std::tie(tmp607, tmp608, tmp609) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp89}).Flatten();
    tmp610 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb280_21});
    tmp611 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp609});
    tmp612 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp610}, TNode<UintPtrT>{tmp611});
    ca_.Branch(tmp612, &block286, std::vector<compiler::Node*>{phi_bb280_7, phi_bb280_8, phi_bb280_9, phi_bb280_10, phi_bb280_11, phi_bb280_14, phi_bb280_15, phi_bb280_21, phi_bb280_21, phi_bb280_21, phi_bb280_21, phi_bb280_21}, &block287, std::vector<compiler::Node*>{phi_bb280_7, phi_bb280_8, phi_bb280_9, phi_bb280_10, phi_bb280_11, phi_bb280_14, phi_bb280_15, phi_bb280_21, phi_bb280_21, phi_bb280_21, phi_bb280_21, phi_bb280_21});
  }

  TNode<IntPtrT> phi_bb286_7;
  TNode<IntPtrT> phi_bb286_8;
  TNode<IntPtrT> phi_bb286_9;
  TNode<IntPtrT> phi_bb286_10;
  TNode<IntPtrT> phi_bb286_11;
  TNode<IntPtrT> phi_bb286_14;
  TNode<BoolT> phi_bb286_15;
  TNode<IntPtrT> phi_bb286_21;
  TNode<IntPtrT> phi_bb286_27;
  TNode<IntPtrT> phi_bb286_28;
  TNode<IntPtrT> phi_bb286_32;
  TNode<IntPtrT> phi_bb286_33;
  TNode<IntPtrT> tmp613;
  TNode<IntPtrT> tmp614;
  TNode<Union<HeapObject, TaggedIndex>> tmp615;
  TNode<IntPtrT> tmp616;
  TNode<Object> tmp617;
  TNode<Union<HeapObject, TaggedIndex>> tmp618;
  TNode<IntPtrT> tmp619;
  TNode<IntPtrT> tmp620;
  TNode<UintPtrT> tmp621;
  TNode<UintPtrT> tmp622;
  TNode<BoolT> tmp623;
  if (block286.is_used()) {
    ca_.Bind(&block286, &phi_bb286_7, &phi_bb286_8, &phi_bb286_9, &phi_bb286_10, &phi_bb286_11, &phi_bb286_14, &phi_bb286_15, &phi_bb286_21, &phi_bb286_27, &phi_bb286_28, &phi_bb286_32, &phi_bb286_33);
    tmp613 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb286_33});
    tmp614 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp608}, TNode<IntPtrT>{tmp613});
    std::tie(tmp615, tmp616) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp607}, TNode<IntPtrT>{tmp614}).Flatten();
    tmp617 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp615, tmp616});
    std::tie(tmp618, tmp619, tmp620) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp89}).Flatten();
    tmp621 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb286_21});
    tmp622 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp620});
    tmp623 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp621}, TNode<UintPtrT>{tmp622});
    ca_.Branch(tmp623, &block294, std::vector<compiler::Node*>{phi_bb286_7, phi_bb286_8, phi_bb286_9, phi_bb286_10, phi_bb286_11, phi_bb286_14, phi_bb286_15, phi_bb286_21, phi_bb286_21, phi_bb286_21, phi_bb286_21, phi_bb286_21}, &block295, std::vector<compiler::Node*>{phi_bb286_7, phi_bb286_8, phi_bb286_9, phi_bb286_10, phi_bb286_11, phi_bb286_14, phi_bb286_15, phi_bb286_21, phi_bb286_21, phi_bb286_21, phi_bb286_21, phi_bb286_21});
  }

  TNode<IntPtrT> phi_bb287_7;
  TNode<IntPtrT> phi_bb287_8;
  TNode<IntPtrT> phi_bb287_9;
  TNode<IntPtrT> phi_bb287_10;
  TNode<IntPtrT> phi_bb287_11;
  TNode<IntPtrT> phi_bb287_14;
  TNode<BoolT> phi_bb287_15;
  TNode<IntPtrT> phi_bb287_21;
  TNode<IntPtrT> phi_bb287_27;
  TNode<IntPtrT> phi_bb287_28;
  TNode<IntPtrT> phi_bb287_32;
  TNode<IntPtrT> phi_bb287_33;
  if (block287.is_used()) {
    ca_.Bind(&block287, &phi_bb287_7, &phi_bb287_8, &phi_bb287_9, &phi_bb287_10, &phi_bb287_11, &phi_bb287_14, &phi_bb287_15, &phi_bb287_21, &phi_bb287_27, &phi_bb287_28, &phi_bb287_32, &phi_bb287_33);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb294_7;
  TNode<IntPtrT> phi_bb294_8;
  TNode<IntPtrT> phi_bb294_9;
  TNode<IntPtrT> phi_bb294_10;
  TNode<IntPtrT> phi_bb294_11;
  TNode<IntPtrT> phi_bb294_14;
  TNode<BoolT> phi_bb294_15;
  TNode<IntPtrT> phi_bb294_21;
  TNode<IntPtrT> phi_bb294_28;
  TNode<IntPtrT> phi_bb294_29;
  TNode<IntPtrT> phi_bb294_33;
  TNode<IntPtrT> phi_bb294_34;
  TNode<IntPtrT> tmp624;
  TNode<IntPtrT> tmp625;
  TNode<Union<HeapObject, TaggedIndex>> tmp626;
  TNode<IntPtrT> tmp627;
  TNode<JSAny> tmp628;
  if (block294.is_used()) {
    ca_.Bind(&block294, &phi_bb294_7, &phi_bb294_8, &phi_bb294_9, &phi_bb294_10, &phi_bb294_11, &phi_bb294_14, &phi_bb294_15, &phi_bb294_21, &phi_bb294_28, &phi_bb294_29, &phi_bb294_33, &phi_bb294_34);
    tmp624 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb294_34});
    tmp625 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp619}, TNode<IntPtrT>{tmp624});
    std::tie(tmp626, tmp627) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp618}, TNode<IntPtrT>{tmp625}).Flatten();
    tmp628 = WasmToJSObject_0(state_, TNode<NativeContext>{parameter0}, TNode<Object>{tmp617});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp626, tmp627}, tmp628);
    ca_.Goto(&block259, phi_bb294_7, phi_bb294_8, phi_bb294_9, phi_bb294_10, phi_bb294_11, phi_bb294_14, phi_bb294_15, phi_bb294_21);
  }

  TNode<IntPtrT> phi_bb295_7;
  TNode<IntPtrT> phi_bb295_8;
  TNode<IntPtrT> phi_bb295_9;
  TNode<IntPtrT> phi_bb295_10;
  TNode<IntPtrT> phi_bb295_11;
  TNode<IntPtrT> phi_bb295_14;
  TNode<BoolT> phi_bb295_15;
  TNode<IntPtrT> phi_bb295_21;
  TNode<IntPtrT> phi_bb295_28;
  TNode<IntPtrT> phi_bb295_29;
  TNode<IntPtrT> phi_bb295_33;
  TNode<IntPtrT> phi_bb295_34;
  if (block295.is_used()) {
    ca_.Bind(&block295, &phi_bb295_7, &phi_bb295_8, &phi_bb295_9, &phi_bb295_10, &phi_bb295_11, &phi_bb295_14, &phi_bb295_15, &phi_bb295_21, &phi_bb295_28, &phi_bb295_29, &phi_bb295_33, &phi_bb295_34);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb259_7;
  TNode<IntPtrT> phi_bb259_8;
  TNode<IntPtrT> phi_bb259_9;
  TNode<IntPtrT> phi_bb259_10;
  TNode<IntPtrT> phi_bb259_11;
  TNode<IntPtrT> phi_bb259_14;
  TNode<BoolT> phi_bb259_15;
  TNode<IntPtrT> phi_bb259_21;
  if (block259.is_used()) {
    ca_.Bind(&block259, &phi_bb259_7, &phi_bb259_8, &phi_bb259_9, &phi_bb259_10, &phi_bb259_11, &phi_bb259_14, &phi_bb259_15, &phi_bb259_21);
    ca_.Goto(&block210, phi_bb259_7, phi_bb259_8, phi_bb259_9, phi_bb259_10, phi_bb259_11, phi_bb259_14, phi_bb259_15, phi_bb259_21);
  }

  TNode<IntPtrT> phi_bb210_7;
  TNode<IntPtrT> phi_bb210_8;
  TNode<IntPtrT> phi_bb210_9;
  TNode<IntPtrT> phi_bb210_10;
  TNode<IntPtrT> phi_bb210_11;
  TNode<IntPtrT> phi_bb210_14;
  TNode<BoolT> phi_bb210_15;
  TNode<IntPtrT> phi_bb210_21;
  if (block210.is_used()) {
    ca_.Bind(&block210, &phi_bb210_7, &phi_bb210_8, &phi_bb210_9, &phi_bb210_10, &phi_bb210_11, &phi_bb210_14, &phi_bb210_15, &phi_bb210_21);
    ca_.Goto(&block176, phi_bb210_7, phi_bb210_8, phi_bb210_9, phi_bb210_10, phi_bb210_11, phi_bb210_14, phi_bb210_15, phi_bb210_21);
  }

  TNode<IntPtrT> phi_bb176_7;
  TNode<IntPtrT> phi_bb176_8;
  TNode<IntPtrT> phi_bb176_9;
  TNode<IntPtrT> phi_bb176_10;
  TNode<IntPtrT> phi_bb176_11;
  TNode<IntPtrT> phi_bb176_14;
  TNode<BoolT> phi_bb176_15;
  TNode<IntPtrT> phi_bb176_21;
  if (block176.is_used()) {
    ca_.Bind(&block176, &phi_bb176_7, &phi_bb176_8, &phi_bb176_9, &phi_bb176_10, &phi_bb176_11, &phi_bb176_14, &phi_bb176_15, &phi_bb176_21);
    ca_.Goto(&block153, phi_bb176_7, phi_bb176_8, phi_bb176_9, phi_bb176_10, phi_bb176_11, phi_bb176_14, phi_bb176_15, phi_bb176_21);
  }

  TNode<IntPtrT> phi_bb153_7;
  TNode<IntPtrT> phi_bb153_8;
  TNode<IntPtrT> phi_bb153_9;
  TNode<IntPtrT> phi_bb153_10;
  TNode<IntPtrT> phi_bb153_11;
  TNode<IntPtrT> phi_bb153_14;
  TNode<BoolT> phi_bb153_15;
  TNode<IntPtrT> phi_bb153_21;
  TNode<IntPtrT> tmp629;
  TNode<IntPtrT> tmp630;
  if (block153.is_used()) {
    ca_.Bind(&block153, &phi_bb153_7, &phi_bb153_8, &phi_bb153_9, &phi_bb153_10, &phi_bb153_11, &phi_bb153_14, &phi_bb153_15, &phi_bb153_21);
    tmp629 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp630 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb153_21}, TNode<IntPtrT>{tmp629});
    ca_.Goto(&block148, phi_bb153_7, phi_bb153_8, phi_bb153_9, phi_bb153_10, phi_bb153_11, phi_bb153_14, phi_bb153_15, tmp630);
  }

  TNode<IntPtrT> phi_bb147_7;
  TNode<IntPtrT> phi_bb147_8;
  TNode<IntPtrT> phi_bb147_9;
  TNode<IntPtrT> phi_bb147_10;
  TNode<IntPtrT> phi_bb147_11;
  TNode<IntPtrT> phi_bb147_14;
  TNode<BoolT> phi_bb147_15;
  TNode<IntPtrT> phi_bb147_21;
  TNode<IntPtrT> tmp631;
  TNode<BoolT> tmp632;
  if (block147.is_used()) {
    ca_.Bind(&block147, &phi_bb147_7, &phi_bb147_8, &phi_bb147_9, &phi_bb147_10, &phi_bb147_11, &phi_bb147_14, &phi_bb147_15, &phi_bb147_21);
    tmp631 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp632 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp337}, TNode<IntPtrT>{tmp631});
    ca_.Branch(tmp632, &block301, std::vector<compiler::Node*>{phi_bb147_7, phi_bb147_8, phi_bb147_9, phi_bb147_10, phi_bb147_11, phi_bb147_14, phi_bb147_15}, &block302, std::vector<compiler::Node*>{phi_bb147_7, phi_bb147_8, phi_bb147_9, phi_bb147_10, phi_bb147_11, phi_bb147_14, phi_bb147_15});
  }

  TNode<IntPtrT> phi_bb301_7;
  TNode<IntPtrT> phi_bb301_8;
  TNode<IntPtrT> phi_bb301_9;
  TNode<IntPtrT> phi_bb301_10;
  TNode<IntPtrT> phi_bb301_11;
  TNode<IntPtrT> phi_bb301_14;
  TNode<BoolT> phi_bb301_15;
  TNode<BoolT> tmp633;
  if (block301.is_used()) {
    ca_.Bind(&block301, &phi_bb301_7, &phi_bb301_8, &phi_bb301_9, &phi_bb301_10, &phi_bb301_11, &phi_bb301_14, &phi_bb301_15);
    tmp633 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block303, phi_bb301_7, phi_bb301_8, phi_bb301_9, phi_bb301_10, phi_bb301_11, phi_bb301_14, phi_bb301_15, tmp633);
  }

  TNode<IntPtrT> phi_bb302_7;
  TNode<IntPtrT> phi_bb302_8;
  TNode<IntPtrT> phi_bb302_9;
  TNode<IntPtrT> phi_bb302_10;
  TNode<IntPtrT> phi_bb302_11;
  TNode<IntPtrT> phi_bb302_14;
  TNode<BoolT> phi_bb302_15;
  TNode<BoolT> tmp634;
  if (block302.is_used()) {
    ca_.Bind(&block302, &phi_bb302_7, &phi_bb302_8, &phi_bb302_9, &phi_bb302_10, &phi_bb302_11, &phi_bb302_14, &phi_bb302_15);
    tmp634 = CodeStubAssembler(state_).IntPtrLessThanOrEqual(TNode<IntPtrT>{phi_bb302_11}, TNode<IntPtrT>{tmp337});
    ca_.Goto(&block303, phi_bb302_7, phi_bb302_8, phi_bb302_9, phi_bb302_10, phi_bb302_11, phi_bb302_14, phi_bb302_15, tmp634);
  }

  TNode<IntPtrT> phi_bb303_7;
  TNode<IntPtrT> phi_bb303_8;
  TNode<IntPtrT> phi_bb303_9;
  TNode<IntPtrT> phi_bb303_10;
  TNode<IntPtrT> phi_bb303_11;
  TNode<IntPtrT> phi_bb303_14;
  TNode<BoolT> phi_bb303_15;
  TNode<BoolT> phi_bb303_22;
  if (block303.is_used()) {
    ca_.Bind(&block303, &phi_bb303_7, &phi_bb303_8, &phi_bb303_9, &phi_bb303_10, &phi_bb303_11, &phi_bb303_14, &phi_bb303_15, &phi_bb303_22);
    ca_.Branch(phi_bb303_22, &block299, std::vector<compiler::Node*>{phi_bb303_7, phi_bb303_8, phi_bb303_9, phi_bb303_10, phi_bb303_11, phi_bb303_14, phi_bb303_15}, &block300, std::vector<compiler::Node*>{phi_bb303_7, phi_bb303_8, phi_bb303_9, phi_bb303_10, phi_bb303_11, phi_bb303_14, phi_bb303_15});
  }

  TNode<IntPtrT> phi_bb300_7;
  TNode<IntPtrT> phi_bb300_8;
  TNode<IntPtrT> phi_bb300_9;
  TNode<IntPtrT> phi_bb300_10;
  TNode<IntPtrT> phi_bb300_11;
  TNode<IntPtrT> phi_bb300_14;
  TNode<BoolT> phi_bb300_15;
  if (block300.is_used()) {
    ca_.Bind(&block300, &phi_bb300_7, &phi_bb300_8, &phi_bb300_9, &phi_bb300_10, &phi_bb300_11, &phi_bb300_14, &phi_bb300_15);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/js-to-wasm.tq", 379});
      CodeStubAssembler(state_).FailAssert("Torque assert 'this.paramBufferEnd == 0 || this.nextStack <= this.paramBufferEnd' failed", pos_stack);
    }
  }

  TNode<IntPtrT> phi_bb299_7;
  TNode<IntPtrT> phi_bb299_8;
  TNode<IntPtrT> phi_bb299_9;
  TNode<IntPtrT> phi_bb299_10;
  TNode<IntPtrT> phi_bb299_11;
  TNode<IntPtrT> phi_bb299_14;
  TNode<BoolT> phi_bb299_15;
  if (block299.is_used()) {
    ca_.Bind(&block299, &phi_bb299_7, &phi_bb299_8, &phi_bb299_9, &phi_bb299_10, &phi_bb299_11, &phi_bb299_14, &phi_bb299_15);
    CodeStubAssembler(state_).Return(parameter1);
  }
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=308&c=22
TorqueStructReference_intptr_0 NewReference_intptr_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, TaggedIndex>> p_object, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = (TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{p_object}, TNode<IntPtrT>{p_offset}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=320&c=25
int31_t SizeOf_float64_0(compiler::CodeAssemblerState* state_) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  if (block0.is_used()) {
    ca_.Bind(&block0);
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return kDoubleSize;
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=528&c=23
TorqueStructReference_intptr_0 GetRefAt_intptr_RawFunctionSigPtr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_intptr_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=532&c=17
TorqueStructReference_RawPtr_0 GetRefAt_RawPtr_RawFunctionSigPtr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_RawPtr_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_RawPtr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=796&c=4
TorqueStructReference_int32_0 GetRefAt_int32_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_int32_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_int32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=798&c=4
TorqueStructReference_bool_0 GetRefAt_bool_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_bool_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_bool_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=799&c=4
TorqueStructReference_RawPtr_0 GetRefAt_RawPtr_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_RawPtr_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_RawPtr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=802&c=4
TorqueStructReference_WasmCodePointer_0 GetRefAt_WasmCodePointer_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_WasmCodePointer_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_WasmCodePointer_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=804&c=4
TorqueStructReference_RawPtr_intptr_0 GetRefAt_RawPtr_intptr_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_RawPtr_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=864&c=7
TNode<BoolT> Is_WasmFuncRef_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<WasmFuncRef> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_WasmFuncRef_1(state_, TNode<Context>{p_context}, TNode<Object>{p_o}, &label1);
    ca_.Goto(&block4);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block5);
    }
  }

  TNode<BoolT> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp2 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block1, tmp2);
  }

  TNode<BoolT> tmp3;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp3 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp3);
  }

  TNode<BoolT> phi_bb1_2;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_2);
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<BoolT>{phi_bb1_2};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=865&c=22
TNode<WasmFuncRef> UnsafeCast_WasmFuncRef_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<WasmFuncRef> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = TORQUE_CAST(TNode<Object>{p_o});
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<WasmFuncRef>{tmp0};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=867&c=9
TNode<BoolT> Is_JSFunction_JSFunction_OR_Undefined_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSFunction, Undefined>> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSFunction> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSFunction_0(state_, TNode<HeapObject>{p_o}, &label1);
    ca_.Goto(&block4);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block5);
    }
  }

  TNode<BoolT> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp2 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block1, tmp2);
  }

  TNode<BoolT> tmp3;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp3 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp3);
  }

  TNode<BoolT> phi_bb1_2;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_2);
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<BoolT>{phi_bb1_2};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=868&c=14
TNode<JSFunction> UnsafeCast_JSFunction_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSFunction> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = TORQUE_CAST(TNode<Object>{p_o});
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<JSFunction>{tmp0};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=874&c=7
TNode<BoolT> Is_String_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<String> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_String_1(state_, TNode<Context>{p_context}, TNode<Object>{p_o}, &label1);
    ca_.Goto(&block4);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block5);
    }
  }

  TNode<BoolT> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp2 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block1, tmp2);
  }

  TNode<BoolT> tmp3;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp3 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp3);
  }

  TNode<BoolT> phi_bb1_2;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_2);
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<BoolT>{phi_bb1_2};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=892&c=22
TorqueStructReference_uint32_0 GetRefAt_uint32_RawPtr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_uint32_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=896&c=37
TorqueStructReference_int64_0 GetRefAt_int64_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_int64_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_int64_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=905&c=58
TorqueStructReference_float64_0 GetRefAt_float64_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_float64_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_float64_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=915&c=13
TorqueStructReference_float32_0 GetRefAt_float32_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_float32_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_float32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=938&c=23
TorqueStructReference_uintptr_0 GetRefAt_uintptr_RawPtr_uintptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{p_base}, TNode<IntPtrT>{p_offset});
    tmp1 = (TNode<RawPtrT>{tmp0});
    std::tie(tmp2, tmp3) = NewOffHeapReference_uintptr_0(state_, TNode<RawPtrT>{tmp1}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_uintptr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp3}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=155&c=10
TorqueStructReference_float32_0 NewReference_float32_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, TaggedIndex>> p_object, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = (TorqueStructReference_float32_0{TNode<Union<HeapObject, TaggedIndex>>{p_object}, TNode<IntPtrT>{p_offset}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_float32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=155&c=10
TorqueStructReference_int64_0 NewReference_int64_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, TaggedIndex>> p_object, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = (TorqueStructReference_int64_0{TNode<Union<HeapObject, TaggedIndex>>{p_object}, TNode<IntPtrT>{p_offset}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_int64_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/js-to-wasm.tq?l=155&c=10
TorqueStructReference_int32_0 NewReference_int32_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, TaggedIndex>> p_object, TNode<IntPtrT> p_offset) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = (TorqueStructReference_int32_0{TNode<Union<HeapObject, TaggedIndex>>{p_object}, TNode<IntPtrT>{p_offset}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_int32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

} // namespace internal
} // namespace v8
