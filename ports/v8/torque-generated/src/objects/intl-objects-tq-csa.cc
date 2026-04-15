#include "src/ast/ast.h"
#include "src/builtins/builtins-array-gen.h"
#include "src/builtins/builtins-bigint-gen.h"
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
#include "src/builtins/builtins.h"
#include "src/codegen/code-factory.h"
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
#include "src/codegen/code-stub-assembler-inl.h"
// Required Builtins:
#include "torque-generated/src/objects/intl-objects-tq-csa.h"
#include "torque-generated/src/objects/intl-objects-tq-csa.h"
#include "torque-generated/src/objects/string-tq-csa.h"
#include "torque-generated/src/builtins/convert-tq-csa.h"
#include "torque-generated/src/builtins/torque-internal-tq-csa.h"
#include "torque-generated/src/builtins/cast-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=13&c=1
TNode<Uint8T> IntlAsciiCollationWeightL1_0(compiler::CodeAssemblerState* state_, TNode<Uint8T> p_c, compiler::CodeAssemblerLabel* label__Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Uint8T> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_bool_constexpr_bool_0(state_, (CodeStubAssembler(state_).ConstexprInt31Equal(Intl::kAsciiCollationWeightsLength, (FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x100ull))))));
    CodeStubAssembler(state_).StaticAssert(TNode<BoolT>{tmp0}, "static_assert(kIntlAsciiCollationWeightsLength == 256) at https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=14&c=3");
    tmp1 = CodeStubAssembler(state_).IntlAsciiCollationWeightsL1();
    tmp2 = Convert_intptr_char8_0(state_, TNode<Uint8T>{p_c});
    tmp3 = CodeStubAssembler(state_).LoadUint8Ptr(TNode<RawPtrT>{tmp1}, TNode<IntPtrT>{tmp2});
    ca_.Goto(&block3);
  }

    ca_.Bind(&block3);
  return TNode<Uint8T>{tmp3};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=17&c=1
TNode<Uint8T> IntlAsciiCollationWeightL1_1(compiler::CodeAssemblerState* state_, TNode<Uint16T> p_c, compiler::CodeAssemblerLabel* label_Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  TNode<BoolT> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Convert_uint32_char16_0(state_, TNode<Uint16T>{p_c});
    tmp1 = FromConstexpr_uint32_constexpr_int31_0(state_, Intl::kAsciiCollationWeightsLength);
    tmp2 = CodeStubAssembler(state_).Uint32GreaterThanOrEqual(TNode<Uint32T>{tmp0}, TNode<Uint32T>{tmp1});
    ca_.Branch(tmp2, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(label_Bailout);
  }

  TNode<RawPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Uint8T> tmp5;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp3 = CodeStubAssembler(state_).IntlAsciiCollationWeightsL1();
    tmp4 = Convert_intptr_char16_0(state_, TNode<Uint16T>{p_c});
    tmp5 = CodeStubAssembler(state_).LoadUint8Ptr(TNode<RawPtrT>{tmp3}, TNode<IntPtrT>{tmp4});
    ca_.Goto(&block5);
  }

    ca_.Bind(&block5);
  return TNode<Uint8T>{tmp5};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=22&c=1
TNode<Uint8T> IntlAsciiCollationWeightL3_0(compiler::CodeAssemblerState* state_, TNode<Uint8T> p_c, compiler::CodeAssemblerLabel* label__Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Uint8T> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_bool_constexpr_bool_0(state_, (CodeStubAssembler(state_).ConstexprInt31Equal(Intl::kAsciiCollationWeightsLength, (FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x100ull))))));
    CodeStubAssembler(state_).StaticAssert(TNode<BoolT>{tmp0}, "static_assert(kIntlAsciiCollationWeightsLength == 256) at https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=23&c=3");
    tmp1 = CodeStubAssembler(state_).IntlAsciiCollationWeightsL3();
    tmp2 = Convert_intptr_char8_0(state_, TNode<Uint8T>{p_c});
    tmp3 = CodeStubAssembler(state_).LoadUint8Ptr(TNode<RawPtrT>{tmp1}, TNode<IntPtrT>{tmp2});
    ca_.Goto(&block3);
  }

    ca_.Bind(&block3);
  return TNode<Uint8T>{tmp3};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=26&c=1
TNode<Uint8T> IntlAsciiCollationWeightL3_1(compiler::CodeAssemblerState* state_, TNode<Uint16T> p_c, compiler::CodeAssemblerLabel* label_Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  TNode<BoolT> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Convert_uint32_char16_0(state_, TNode<Uint16T>{p_c});
    tmp1 = FromConstexpr_uint32_constexpr_int31_0(state_, Intl::kAsciiCollationWeightsLength);
    tmp2 = CodeStubAssembler(state_).Uint32GreaterThanOrEqual(TNode<Uint32T>{tmp0}, TNode<Uint32T>{tmp1});
    ca_.Branch(tmp2, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(label_Bailout);
  }

  TNode<RawPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Uint8T> tmp5;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp3 = CodeStubAssembler(state_).IntlAsciiCollationWeightsL3();
    tmp4 = Convert_intptr_char16_0(state_, TNode<Uint16T>{p_c});
    tmp5 = CodeStubAssembler(state_).LoadUint8Ptr(TNode<RawPtrT>{tmp3}, TNode<IntPtrT>{tmp4});
    ca_.Goto(&block5);
  }

    ca_.Bind(&block5);
  return TNode<Uint8T>{tmp5};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=32&c=1
void CheckNextIs1Byte_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_char8_ConstReference_char8_0 p__slice, TNode<IntPtrT> p__index, compiler::CodeAssemblerLabel* label__Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  if (block0.is_used()) {
    ca_.Bind(&block0);
    ca_.Goto(&block3);
  }

    ca_.Bind(&block3);
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=36&c=1
void CheckNextIs1Byte_1(compiler::CodeAssemblerState* state_, TorqueStructSlice_char16_ConstReference_char16_0 p_slice, TNode<IntPtrT> p_index, compiler::CodeAssemblerLabel* label_Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<BoolT> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp1 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_index}, TNode<IntPtrT>{tmp0});
    tmp2 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{tmp1}, TNode<IntPtrT>{p_slice.length});
    ca_.Branch(tmp2, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<Union<HeapObject, TaggedIndex>> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<Uint16T> tmp9;
  TNode<Uint32T> tmp10;
  TNode<Uint32T> tmp11;
  TNode<BoolT> tmp12;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp3 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp4 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_index}, TNode<IntPtrT>{tmp3});
    tmp5 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{tmp4});
    tmp6 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_slice.offset}, TNode<IntPtrT>{tmp5});
    std::tie(tmp7, tmp8) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_slice.object}, TNode<IntPtrT>{tmp6}).Flatten();
    tmp9 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp7, tmp8});
    tmp10 = Convert_uint32_char16_0(state_, TNode<Uint16T>{tmp9});
    tmp11 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0xffull));
    tmp12 = CodeStubAssembler(state_).Uint32GreaterThan(TNode<Uint32T>{tmp10}, TNode<Uint32T>{tmp11});
    ca_.Branch(tmp12, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    ca_.Goto(label_Bailout);
  }

  if (block7.is_used()) {
    ca_.Bind(&block7);
    ca_.Goto(&block4);
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    ca_.Goto(&block8);
  }

    ca_.Bind(&block8);
}

TF_BUILTIN(StringFastLocaleCompare, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSFunction> parameter1 = UncheckedParameter<JSFunction>(Descriptor::kLocaleCompareFn);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kLeft);
  USE(parameter2);
  TNode<JSAny> parameter3 = UncheckedParameter<JSAny>(Descriptor::kRight);
  USE(parameter3);
  TNode<JSAny> parameter4 = UncheckedParameter<JSAny>(Descriptor::kLocales);
  USE(parameter4);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, String> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, String> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, IntPtrT> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, IntPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block45(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block44(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, IntPtrT> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, IntPtrT> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block52(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block61(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block60(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, IntPtrT> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block73(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block72(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, IntPtrT> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String, IntPtrT> block76(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT, String> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block83(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block82(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT> block86(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT> block84(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, String> block90(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, String> block89(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block94(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block93(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block98(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block97(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block102(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block101(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block106(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block105(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block110(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block109(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block114(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block113(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block118(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block117(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block122(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block121(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block126(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block125(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block130(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block129(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block107(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block103(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT> block85(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block81(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block132(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block131(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block134(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block133(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block140(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block139(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT> block143(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT> block141(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, String> block147(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, String> block146(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block151(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block150(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block155(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block154(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block159(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block158(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block163(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block162(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block167(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block166(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block171(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block170(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block175(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block174(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block179(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block178(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block183(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block182(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block187(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String, IntPtrT> block186(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block164(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT, String> block160(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, String, IntPtrT> block142(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block138(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block189(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block188(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block136(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block191(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT> block190(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    ca_.Goto(&block0);

  TNode<String> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_String_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter2}, &label1);
    ca_.Goto(&block7);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block8);
    }
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    ca_.Goto(&block2);
  }

  TNode<String> tmp2;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_String_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter3}, &label3);
    ca_.Goto(&block9);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    ca_.Goto(&block2);
  }

  TNode<BoolT> tmp4;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp4 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<HeapObject, Smi, Weak<HeapObject>>>{tmp0}, TNode<Union<HeapObject, Smi, Weak<HeapObject>>>{tmp2});
    ca_.Branch(tmp4, &block11, std::vector<compiler::Node*>{}, &block12, std::vector<compiler::Node*>{});
  }

  TNode<Smi> tmp5;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp5 = SmiConstant_0(state_, IntegerLiteral(false, 0x0ull));
    CodeStubAssembler(state_).Return(tmp5);
  }

  TNode<SeqOneByteString> tmp6;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    compiler::CodeAssemblerLabel label7(&ca_);
    tmp6 = Cast_SeqOneByteString_1(state_, TNode<String>{tmp0}, &label7);
    ca_.Goto(&block15);
    if (label7.is_used()) {
      ca_.Bind(&label7);
      ca_.Goto(&block16);
    }
  }

  if (block16.is_used()) {
    ca_.Bind(&block16);
    ca_.Goto(&block13);
  }

  TNode<SeqOneByteString> tmp8;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    compiler::CodeAssemblerLabel label9(&ca_);
    tmp8 = Cast_SeqOneByteString_1(state_, TNode<String>{tmp2}, &label9);
    ca_.Goto(&block19);
    if (label9.is_used()) {
      ca_.Bind(&label9);
      ca_.Goto(&block20);
    }
  }

  if (block20.is_used()) {
    ca_.Bind(&block20);
    ca_.Goto(&block13);
  }

  TNode<IntPtrT> tmp10;
  TNode<Int32T> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<Int32T> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<Union<HeapObject, TaggedIndex>> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<Union<HeapObject, TaggedIndex>> tmp20;
  TNode<IntPtrT> tmp21;
  TNode<IntPtrT> tmp22;
  if (block19.is_used()) {
    ca_.Bind(&block19);
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp11 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp6, tmp10});
    tmp12 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp11});
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp14 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp8, tmp13});
    tmp15 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp14});
    std::tie(tmp16, tmp17, tmp18) = FieldSliceSeqOneByteStringChars_0(state_, TNode<SeqOneByteString>{tmp6}).Flatten();
    tmp19 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    compiler::CodeAssemblerLabel label23(&ca_);
    std::tie(tmp20, tmp21, tmp22) = Subslice_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp16}, TNode<IntPtrT>{tmp17}, TNode<IntPtrT>{tmp18}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{tmp19}, TNode<IntPtrT>{tmp12}, &label23).Flatten();
    ca_.Goto(&block23);
    if (label23.is_used()) {
      ca_.Bind(&label23);
      ca_.Goto(&block24);
    }
  }

  if (block24.is_used()) {
    ca_.Bind(&block24);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<IntPtrT> tmp26;
  TNode<IntPtrT> tmp27;
  TNode<Union<HeapObject, TaggedIndex>> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  if (block23.is_used()) {
    ca_.Bind(&block23);
    std::tie(tmp24, tmp25, tmp26) = FieldSliceSeqOneByteStringChars_0(state_, TNode<SeqOneByteString>{tmp8}).Flatten();
    tmp27 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    compiler::CodeAssemblerLabel label31(&ca_);
    std::tie(tmp28, tmp29, tmp30) = Subslice_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp24}, TNode<IntPtrT>{tmp25}, TNode<IntPtrT>{tmp26}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{tmp27}, TNode<IntPtrT>{tmp15}, &label31).Flatten();
    ca_.Goto(&block27);
    if (label31.is_used()) {
      ca_.Bind(&label31);
      ca_.Goto(&block28);
    }
  }

  if (block28.is_used()) {
    ca_.Bind(&block28);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Number> tmp32;
  if (block27.is_used()) {
    ca_.Bind(&block27);
    compiler::CodeAssemblerLabel label33(&ca_);
    tmp32 = LocaleCompareFastPath_char8_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp20}, TNode<IntPtrT>{tmp21}, TNode<IntPtrT>{tmp22}, TorqueStructUnsafe_0{}}, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp28}, TNode<IntPtrT>{tmp29}, TNode<IntPtrT>{tmp30}, TorqueStructUnsafe_0{}}, &label33);
    ca_.Goto(&block29);
    if (label33.is_used()) {
      ca_.Bind(&label33);
      ca_.Goto(&block30);
    }
  }

  if (block30.is_used()) {
    ca_.Bind(&block30);
    ca_.Goto(&block2);
  }

  if (block29.is_used()) {
    ca_.Bind(&block29);
    CodeStubAssembler(state_).Return(tmp32);
  }

  TNode<IntPtrT> tmp34;
  TNode<IntPtrT> tmp35;
  TNode<Int32T> tmp36;
  TNode<IntPtrT> tmp37;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp34 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp35 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp36 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp0, tmp35});
    tmp37 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp36});
    ca_.Goto(&block33, tmp0, tmp34);
  }

  TNode<String> phi_bb33_9;
  TNode<IntPtrT> phi_bb33_10;
  TNode<BoolT> tmp38;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_9, &phi_bb33_10);
    tmp38 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp38, &block31, std::vector<compiler::Node*>{phi_bb33_9, phi_bb33_10}, &block32, std::vector<compiler::Node*>{phi_bb33_9, phi_bb33_10});
  }

  TNode<String> phi_bb31_9;
  TNode<IntPtrT> phi_bb31_10;
  TNode<SeqOneByteString> tmp39;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_9, &phi_bb31_10);
    compiler::CodeAssemblerLabel label40(&ca_);
    tmp39 = Cast_SeqOneByteString_1(state_, TNode<String>{phi_bb31_9}, &label40);
    ca_.Goto(&block36, phi_bb31_9, phi_bb31_10, phi_bb31_9, phi_bb31_9);
    if (label40.is_used()) {
      ca_.Bind(&label40);
      ca_.Goto(&block37, phi_bb31_9, phi_bb31_10, phi_bb31_9, phi_bb31_9);
    }
  }

  TNode<String> phi_bb37_9;
  TNode<IntPtrT> phi_bb37_10;
  TNode<String> phi_bb37_12;
  TNode<String> phi_bb37_13;
  TNode<SeqTwoByteString> tmp41;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_9, &phi_bb37_10, &phi_bb37_12, &phi_bb37_13);
    compiler::CodeAssemblerLabel label42(&ca_);
    tmp41 = Cast_SeqTwoByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb37_12)}, &label42);
    ca_.Goto(&block44, phi_bb37_9, phi_bb37_10, phi_bb37_12);
    if (label42.is_used()) {
      ca_.Bind(&label42);
      ca_.Goto(&block45, phi_bb37_9, phi_bb37_10, phi_bb37_12);
    }
  }

  TNode<String> phi_bb36_9;
  TNode<IntPtrT> phi_bb36_10;
  TNode<String> phi_bb36_12;
  TNode<String> phi_bb36_13;
  TNode<Union<HeapObject, TaggedIndex>> tmp43;
  TNode<IntPtrT> tmp44;
  TNode<IntPtrT> tmp45;
  TNode<Union<HeapObject, TaggedIndex>> tmp46;
  TNode<IntPtrT> tmp47;
  TNode<IntPtrT> tmp48;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_9, &phi_bb36_10, &phi_bb36_12, &phi_bb36_13);
    std::tie(tmp43, tmp44, tmp45) = FieldSliceSeqOneByteStringChars_0(state_, TNode<SeqOneByteString>{tmp39}).Flatten();
    compiler::CodeAssemblerLabel label49(&ca_);
    std::tie(tmp46, tmp47, tmp48) = Subslice_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp43}, TNode<IntPtrT>{tmp44}, TNode<IntPtrT>{tmp45}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb36_10}, TNode<IntPtrT>{tmp37}, &label49).Flatten();
    ca_.Goto(&block40, phi_bb36_9, phi_bb36_10, phi_bb36_12, phi_bb36_10);
    if (label49.is_used()) {
      ca_.Bind(&label49);
      ca_.Goto(&block41, phi_bb36_9, phi_bb36_10, phi_bb36_12, phi_bb36_10);
    }
  }

  TNode<String> phi_bb41_9;
  TNode<IntPtrT> phi_bb41_10;
  TNode<String> phi_bb41_12;
  TNode<IntPtrT> phi_bb41_17;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_9, &phi_bb41_10, &phi_bb41_12, &phi_bb41_17);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> phi_bb40_9;
  TNode<IntPtrT> phi_bb40_10;
  TNode<String> phi_bb40_12;
  TNode<IntPtrT> phi_bb40_17;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_9, &phi_bb40_10, &phi_bb40_12, &phi_bb40_17);
    ca_.Goto(&block6, tmp46, tmp47, tmp48);
  }

  TNode<String> phi_bb45_9;
  TNode<IntPtrT> phi_bb45_10;
  TNode<String> phi_bb45_12;
  TNode<ThinString> tmp50;
  if (block45.is_used()) {
    ca_.Bind(&block45, &phi_bb45_9, &phi_bb45_10, &phi_bb45_12);
    compiler::CodeAssemblerLabel label51(&ca_);
    tmp50 = Cast_ThinString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb45_12)}, &label51);
    ca_.Goto(&block52, phi_bb45_9, phi_bb45_10, phi_bb45_12);
    if (label51.is_used()) {
      ca_.Bind(&label51);
      ca_.Goto(&block53, phi_bb45_9, phi_bb45_10, phi_bb45_12);
    }
  }

  TNode<String> phi_bb44_9;
  TNode<IntPtrT> phi_bb44_10;
  TNode<String> phi_bb44_12;
  TNode<Union<HeapObject, TaggedIndex>> tmp52;
  TNode<IntPtrT> tmp53;
  TNode<IntPtrT> tmp54;
  TNode<Union<HeapObject, TaggedIndex>> tmp55;
  TNode<IntPtrT> tmp56;
  TNode<IntPtrT> tmp57;
  if (block44.is_used()) {
    ca_.Bind(&block44, &phi_bb44_9, &phi_bb44_10, &phi_bb44_12);
    std::tie(tmp52, tmp53, tmp54) = FieldSliceSeqTwoByteStringChars_0(state_, TNode<SeqTwoByteString>{tmp41}).Flatten();
    compiler::CodeAssemblerLabel label58(&ca_);
    std::tie(tmp55, tmp56, tmp57) = Subslice_char16_0(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{tmp52}, TNode<IntPtrT>{tmp53}, TNode<IntPtrT>{tmp54}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb44_10}, TNode<IntPtrT>{tmp37}, &label58).Flatten();
    ca_.Goto(&block48, phi_bb44_9, phi_bb44_10, phi_bb44_12, phi_bb44_10);
    if (label58.is_used()) {
      ca_.Bind(&label58);
      ca_.Goto(&block49, phi_bb44_9, phi_bb44_10, phi_bb44_12, phi_bb44_10);
    }
  }

  TNode<String> phi_bb49_9;
  TNode<IntPtrT> phi_bb49_10;
  TNode<String> phi_bb49_12;
  TNode<IntPtrT> phi_bb49_17;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_9, &phi_bb49_10, &phi_bb49_12, &phi_bb49_17);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> phi_bb48_9;
  TNode<IntPtrT> phi_bb48_10;
  TNode<String> phi_bb48_12;
  TNode<IntPtrT> phi_bb48_17;
  if (block48.is_used()) {
    ca_.Bind(&block48, &phi_bb48_9, &phi_bb48_10, &phi_bb48_12, &phi_bb48_17);
    ca_.Goto(&block4, tmp55, tmp56, tmp57);
  }

  TNode<String> phi_bb53_9;
  TNode<IntPtrT> phi_bb53_10;
  TNode<String> phi_bb53_12;
  TNode<ConsString> tmp59;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_9, &phi_bb53_10, &phi_bb53_12);
    compiler::CodeAssemblerLabel label60(&ca_);
    tmp59 = Cast_ConsString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb53_12)}, &label60);
    ca_.Goto(&block56, phi_bb53_9, phi_bb53_10, phi_bb53_12);
    if (label60.is_used()) {
      ca_.Bind(&label60);
      ca_.Goto(&block57, phi_bb53_9, phi_bb53_10, phi_bb53_12);
    }
  }

  TNode<String> phi_bb52_9;
  TNode<IntPtrT> phi_bb52_10;
  TNode<String> phi_bb52_12;
  TNode<IntPtrT> tmp61;
  TNode<String> tmp62;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_9, &phi_bb52_10, &phi_bb52_12);
    tmp61 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp62 = CodeStubAssembler(state_).LoadReference<String>(CodeStubAssembler::Reference{tmp50, tmp61});
    ca_.Goto(&block50, tmp62, phi_bb52_10, phi_bb52_12);
  }

  TNode<String> phi_bb57_9;
  TNode<IntPtrT> phi_bb57_10;
  TNode<String> phi_bb57_12;
  TNode<SlicedString> tmp63;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_9, &phi_bb57_10, &phi_bb57_12);
    compiler::CodeAssemblerLabel label64(&ca_);
    tmp63 = Cast_SlicedString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb57_12)}, &label64);
    ca_.Goto(&block60, phi_bb57_9, phi_bb57_10, phi_bb57_12);
    if (label64.is_used()) {
      ca_.Bind(&label64);
      ca_.Goto(&block61, phi_bb57_9, phi_bb57_10, phi_bb57_12);
    }
  }

  TNode<String> phi_bb56_9;
  TNode<IntPtrT> phi_bb56_10;
  TNode<String> phi_bb56_12;
  TNode<String> tmp65;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_9, &phi_bb56_10, &phi_bb56_12);
    tmp65 = Flatten_1(state_, TNode<ConsString>{tmp59});
    ca_.Goto(&block54, tmp65, phi_bb56_10, phi_bb56_12);
  }

  TNode<String> phi_bb61_9;
  TNode<IntPtrT> phi_bb61_10;
  TNode<String> phi_bb61_12;
  TNode<ExternalOneByteString> tmp66;
  if (block61.is_used()) {
    ca_.Bind(&block61, &phi_bb61_9, &phi_bb61_10, &phi_bb61_12);
    compiler::CodeAssemblerLabel label67(&ca_);
    tmp66 = Cast_ExternalOneByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb61_12)}, &label67);
    ca_.Goto(&block64, phi_bb61_9, phi_bb61_10, phi_bb61_12);
    if (label67.is_used()) {
      ca_.Bind(&label67);
      ca_.Goto(&block65, phi_bb61_9, phi_bb61_10, phi_bb61_12);
    }
  }

  TNode<String> phi_bb60_9;
  TNode<IntPtrT> phi_bb60_10;
  TNode<String> phi_bb60_12;
  TNode<IntPtrT> tmp68;
  TNode<Smi> tmp69;
  TNode<IntPtrT> tmp70;
  TNode<IntPtrT> tmp71;
  TNode<IntPtrT> tmp72;
  TNode<String> tmp73;
  if (block60.is_used()) {
    ca_.Bind(&block60, &phi_bb60_9, &phi_bb60_10, &phi_bb60_12);
    tmp68 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp69 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{tmp63, tmp68});
    tmp70 = Convert_intptr_Smi_0(state_, TNode<Smi>{tmp69});
    tmp71 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb60_10}, TNode<IntPtrT>{tmp70});
    tmp72 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp73 = CodeStubAssembler(state_).LoadReference<String>(CodeStubAssembler::Reference{tmp63, tmp72});
    ca_.Goto(&block54, tmp73, tmp71, phi_bb60_12);
  }

  TNode<String> phi_bb65_9;
  TNode<IntPtrT> phi_bb65_10;
  TNode<String> phi_bb65_12;
  TNode<ExternalTwoByteString> tmp74;
  if (block65.is_used()) {
    ca_.Bind(&block65, &phi_bb65_9, &phi_bb65_10, &phi_bb65_12);
    compiler::CodeAssemblerLabel label75(&ca_);
    tmp74 = Cast_ExternalTwoByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb65_12)}, &label75);
    ca_.Goto(&block72, phi_bb65_9, phi_bb65_10, phi_bb65_12);
    if (label75.is_used()) {
      ca_.Bind(&label75);
      ca_.Goto(&block73, phi_bb65_9, phi_bb65_10, phi_bb65_12);
    }
  }

  TNode<String> phi_bb64_9;
  TNode<IntPtrT> phi_bb64_10;
  TNode<String> phi_bb64_12;
  TNode<RawPtrT> tmp76;
  TNode<IntPtrT> tmp77;
  TNode<Int32T> tmp78;
  TNode<IntPtrT> tmp79;
  TNode<Union<HeapObject, TaggedIndex>> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<IntPtrT> tmp82;
  TNode<Union<HeapObject, TaggedIndex>> tmp83;
  TNode<IntPtrT> tmp84;
  TNode<IntPtrT> tmp85;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_9, &phi_bb64_10, &phi_bb64_12);
    tmp76 = Method_ExternalOneByteString_GetChars_0(state_, TNode<ExternalOneByteString>{tmp66});
    tmp77 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp78 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp66, tmp77});
    tmp79 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp78});
    std::tie(tmp80, tmp81, tmp82) = NewOffHeapConstSlice_char8_0(state_, TNode<RawPtrT>{tmp76}, TNode<IntPtrT>{tmp79}).Flatten();
    compiler::CodeAssemblerLabel label86(&ca_);
    std::tie(tmp83, tmp84, tmp85) = Subslice_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp80}, TNode<IntPtrT>{tmp81}, TNode<IntPtrT>{tmp82}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb64_10}, TNode<IntPtrT>{tmp37}, &label86).Flatten();
    ca_.Goto(&block68, phi_bb64_9, phi_bb64_10, phi_bb64_12, phi_bb64_10);
    if (label86.is_used()) {
      ca_.Bind(&label86);
      ca_.Goto(&block69, phi_bb64_9, phi_bb64_10, phi_bb64_12, phi_bb64_10);
    }
  }

  TNode<String> phi_bb69_9;
  TNode<IntPtrT> phi_bb69_10;
  TNode<String> phi_bb69_12;
  TNode<IntPtrT> phi_bb69_20;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_9, &phi_bb69_10, &phi_bb69_12, &phi_bb69_20);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> phi_bb68_9;
  TNode<IntPtrT> phi_bb68_10;
  TNode<String> phi_bb68_12;
  TNode<IntPtrT> phi_bb68_20;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_9, &phi_bb68_10, &phi_bb68_12, &phi_bb68_20);
    ca_.Goto(&block6, tmp83, tmp84, tmp85);
  }

  TNode<String> phi_bb73_9;
  TNode<IntPtrT> phi_bb73_10;
  TNode<String> phi_bb73_12;
  if (block73.is_used()) {
    ca_.Bind(&block73, &phi_bb73_9, &phi_bb73_10, &phi_bb73_12);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> phi_bb72_9;
  TNode<IntPtrT> phi_bb72_10;
  TNode<String> phi_bb72_12;
  TNode<RawPtrT> tmp87;
  TNode<IntPtrT> tmp88;
  TNode<Int32T> tmp89;
  TNode<IntPtrT> tmp90;
  TNode<Union<HeapObject, TaggedIndex>> tmp91;
  TNode<IntPtrT> tmp92;
  TNode<IntPtrT> tmp93;
  TNode<Union<HeapObject, TaggedIndex>> tmp94;
  TNode<IntPtrT> tmp95;
  TNode<IntPtrT> tmp96;
  if (block72.is_used()) {
    ca_.Bind(&block72, &phi_bb72_9, &phi_bb72_10, &phi_bb72_12);
    tmp87 = Method_ExternalTwoByteString_GetChars_0(state_, TNode<ExternalTwoByteString>{tmp74});
    tmp88 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp89 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp74, tmp88});
    tmp90 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp89});
    std::tie(tmp91, tmp92, tmp93) = NewOffHeapConstSlice_char16_0(state_, TNode<RawPtrT>{tmp87}, TNode<IntPtrT>{tmp90}).Flatten();
    compiler::CodeAssemblerLabel label97(&ca_);
    std::tie(tmp94, tmp95, tmp96) = Subslice_char16_0(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{tmp91}, TNode<IntPtrT>{tmp92}, TNode<IntPtrT>{tmp93}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb72_10}, TNode<IntPtrT>{tmp37}, &label97).Flatten();
    ca_.Goto(&block76, phi_bb72_9, phi_bb72_10, phi_bb72_12, phi_bb72_10);
    if (label97.is_used()) {
      ca_.Bind(&label97);
      ca_.Goto(&block77, phi_bb72_9, phi_bb72_10, phi_bb72_12, phi_bb72_10);
    }
  }

  TNode<String> phi_bb77_9;
  TNode<IntPtrT> phi_bb77_10;
  TNode<String> phi_bb77_12;
  TNode<IntPtrT> phi_bb77_20;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_9, &phi_bb77_10, &phi_bb77_12, &phi_bb77_20);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> phi_bb76_9;
  TNode<IntPtrT> phi_bb76_10;
  TNode<String> phi_bb76_12;
  TNode<IntPtrT> phi_bb76_20;
  if (block76.is_used()) {
    ca_.Bind(&block76, &phi_bb76_9, &phi_bb76_10, &phi_bb76_12, &phi_bb76_20);
    ca_.Goto(&block4, tmp94, tmp95, tmp96);
  }

  TNode<String> phi_bb54_9;
  TNode<IntPtrT> phi_bb54_10;
  TNode<String> phi_bb54_12;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_9, &phi_bb54_10, &phi_bb54_12);
    ca_.Goto(&block50, phi_bb54_9, phi_bb54_10, phi_bb54_12);
  }

  TNode<String> phi_bb50_9;
  TNode<IntPtrT> phi_bb50_10;
  TNode<String> phi_bb50_12;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_9, &phi_bb50_10, &phi_bb50_12);
    ca_.Goto(&block33, phi_bb50_9, phi_bb50_10);
  }

  TNode<String> phi_bb32_9;
  TNode<IntPtrT> phi_bb32_10;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_9, &phi_bb32_10);
    VerifiedUnreachable_0(state_);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb6_5;
  TNode<IntPtrT> phi_bb6_6;
  TNode<IntPtrT> phi_bb6_7;
  TNode<String> tmp98;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_5, &phi_bb6_6, &phi_bb6_7);
    compiler::CodeAssemblerLabel label99(&ca_);
    tmp98 = Cast_String_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter3}, &label99);
    ca_.Goto(&block82, phi_bb6_5, phi_bb6_6, phi_bb6_7);
    if (label99.is_used()) {
      ca_.Bind(&label99);
      ca_.Goto(&block83, phi_bb6_5, phi_bb6_6, phi_bb6_7);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb83_5;
  TNode<IntPtrT> phi_bb83_6;
  TNode<IntPtrT> phi_bb83_7;
  if (block83.is_used()) {
    ca_.Bind(&block83, &phi_bb83_5, &phi_bb83_6, &phi_bb83_7);
    ca_.Goto(&block2);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb82_5;
  TNode<IntPtrT> phi_bb82_6;
  TNode<IntPtrT> phi_bb82_7;
  TNode<IntPtrT> tmp100;
  TNode<IntPtrT> tmp101;
  TNode<Int32T> tmp102;
  TNode<IntPtrT> tmp103;
  if (block82.is_used()) {
    ca_.Bind(&block82, &phi_bb82_5, &phi_bb82_6, &phi_bb82_7);
    tmp100 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp101 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp102 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp98, tmp101});
    tmp103 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp102});
    ca_.Goto(&block86, phi_bb82_5, phi_bb82_6, phi_bb82_7, tmp98, tmp100);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb86_5;
  TNode<IntPtrT> phi_bb86_6;
  TNode<IntPtrT> phi_bb86_7;
  TNode<String> phi_bb86_11;
  TNode<IntPtrT> phi_bb86_12;
  TNode<BoolT> tmp104;
  if (block86.is_used()) {
    ca_.Bind(&block86, &phi_bb86_5, &phi_bb86_6, &phi_bb86_7, &phi_bb86_11, &phi_bb86_12);
    tmp104 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp104, &block84, std::vector<compiler::Node*>{phi_bb86_5, phi_bb86_6, phi_bb86_7, phi_bb86_11, phi_bb86_12}, &block85, std::vector<compiler::Node*>{phi_bb86_5, phi_bb86_6, phi_bb86_7, phi_bb86_11, phi_bb86_12});
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb84_5;
  TNode<IntPtrT> phi_bb84_6;
  TNode<IntPtrT> phi_bb84_7;
  TNode<String> phi_bb84_11;
  TNode<IntPtrT> phi_bb84_12;
  TNode<SeqOneByteString> tmp105;
  if (block84.is_used()) {
    ca_.Bind(&block84, &phi_bb84_5, &phi_bb84_6, &phi_bb84_7, &phi_bb84_11, &phi_bb84_12);
    compiler::CodeAssemblerLabel label106(&ca_);
    tmp105 = Cast_SeqOneByteString_1(state_, TNode<String>{phi_bb84_11}, &label106);
    ca_.Goto(&block89, phi_bb84_5, phi_bb84_6, phi_bb84_7, phi_bb84_11, phi_bb84_12, phi_bb84_11, phi_bb84_11);
    if (label106.is_used()) {
      ca_.Bind(&label106);
      ca_.Goto(&block90, phi_bb84_5, phi_bb84_6, phi_bb84_7, phi_bb84_11, phi_bb84_12, phi_bb84_11, phi_bb84_11);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb90_5;
  TNode<IntPtrT> phi_bb90_6;
  TNode<IntPtrT> phi_bb90_7;
  TNode<String> phi_bb90_11;
  TNode<IntPtrT> phi_bb90_12;
  TNode<String> phi_bb90_14;
  TNode<String> phi_bb90_15;
  TNode<SeqTwoByteString> tmp107;
  if (block90.is_used()) {
    ca_.Bind(&block90, &phi_bb90_5, &phi_bb90_6, &phi_bb90_7, &phi_bb90_11, &phi_bb90_12, &phi_bb90_14, &phi_bb90_15);
    compiler::CodeAssemblerLabel label108(&ca_);
    tmp107 = Cast_SeqTwoByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb90_14)}, &label108);
    ca_.Goto(&block97, phi_bb90_5, phi_bb90_6, phi_bb90_7, phi_bb90_11, phi_bb90_12, phi_bb90_14);
    if (label108.is_used()) {
      ca_.Bind(&label108);
      ca_.Goto(&block98, phi_bb90_5, phi_bb90_6, phi_bb90_7, phi_bb90_11, phi_bb90_12, phi_bb90_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb89_5;
  TNode<IntPtrT> phi_bb89_6;
  TNode<IntPtrT> phi_bb89_7;
  TNode<String> phi_bb89_11;
  TNode<IntPtrT> phi_bb89_12;
  TNode<String> phi_bb89_14;
  TNode<String> phi_bb89_15;
  TNode<Union<HeapObject, TaggedIndex>> tmp109;
  TNode<IntPtrT> tmp110;
  TNode<IntPtrT> tmp111;
  TNode<Union<HeapObject, TaggedIndex>> tmp112;
  TNode<IntPtrT> tmp113;
  TNode<IntPtrT> tmp114;
  if (block89.is_used()) {
    ca_.Bind(&block89, &phi_bb89_5, &phi_bb89_6, &phi_bb89_7, &phi_bb89_11, &phi_bb89_12, &phi_bb89_14, &phi_bb89_15);
    std::tie(tmp109, tmp110, tmp111) = FieldSliceSeqOneByteStringChars_0(state_, TNode<SeqOneByteString>{tmp105}).Flatten();
    compiler::CodeAssemblerLabel label115(&ca_);
    std::tie(tmp112, tmp113, tmp114) = Subslice_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp109}, TNode<IntPtrT>{tmp110}, TNode<IntPtrT>{tmp111}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb89_12}, TNode<IntPtrT>{tmp103}, &label115).Flatten();
    ca_.Goto(&block93, phi_bb89_5, phi_bb89_6, phi_bb89_7, phi_bb89_11, phi_bb89_12, phi_bb89_14, phi_bb89_12);
    if (label115.is_used()) {
      ca_.Bind(&label115);
      ca_.Goto(&block94, phi_bb89_5, phi_bb89_6, phi_bb89_7, phi_bb89_11, phi_bb89_12, phi_bb89_14, phi_bb89_12);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb94_5;
  TNode<IntPtrT> phi_bb94_6;
  TNode<IntPtrT> phi_bb94_7;
  TNode<String> phi_bb94_11;
  TNode<IntPtrT> phi_bb94_12;
  TNode<String> phi_bb94_14;
  TNode<IntPtrT> phi_bb94_19;
  if (block94.is_used()) {
    ca_.Bind(&block94, &phi_bb94_5, &phi_bb94_6, &phi_bb94_7, &phi_bb94_11, &phi_bb94_12, &phi_bb94_14, &phi_bb94_19);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb93_5;
  TNode<IntPtrT> phi_bb93_6;
  TNode<IntPtrT> phi_bb93_7;
  TNode<String> phi_bb93_11;
  TNode<IntPtrT> phi_bb93_12;
  TNode<String> phi_bb93_14;
  TNode<IntPtrT> phi_bb93_19;
  if (block93.is_used()) {
    ca_.Bind(&block93, &phi_bb93_5, &phi_bb93_6, &phi_bb93_7, &phi_bb93_11, &phi_bb93_12, &phi_bb93_14, &phi_bb93_19);
    ca_.Goto(&block81, phi_bb93_5, phi_bb93_6, phi_bb93_7, tmp112, tmp113, tmp114);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb98_5;
  TNode<IntPtrT> phi_bb98_6;
  TNode<IntPtrT> phi_bb98_7;
  TNode<String> phi_bb98_11;
  TNode<IntPtrT> phi_bb98_12;
  TNode<String> phi_bb98_14;
  TNode<ThinString> tmp116;
  if (block98.is_used()) {
    ca_.Bind(&block98, &phi_bb98_5, &phi_bb98_6, &phi_bb98_7, &phi_bb98_11, &phi_bb98_12, &phi_bb98_14);
    compiler::CodeAssemblerLabel label117(&ca_);
    tmp116 = Cast_ThinString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb98_14)}, &label117);
    ca_.Goto(&block105, phi_bb98_5, phi_bb98_6, phi_bb98_7, phi_bb98_11, phi_bb98_12, phi_bb98_14);
    if (label117.is_used()) {
      ca_.Bind(&label117);
      ca_.Goto(&block106, phi_bb98_5, phi_bb98_6, phi_bb98_7, phi_bb98_11, phi_bb98_12, phi_bb98_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb97_5;
  TNode<IntPtrT> phi_bb97_6;
  TNode<IntPtrT> phi_bb97_7;
  TNode<String> phi_bb97_11;
  TNode<IntPtrT> phi_bb97_12;
  TNode<String> phi_bb97_14;
  TNode<Union<HeapObject, TaggedIndex>> tmp118;
  TNode<IntPtrT> tmp119;
  TNode<IntPtrT> tmp120;
  TNode<Union<HeapObject, TaggedIndex>> tmp121;
  TNode<IntPtrT> tmp122;
  TNode<IntPtrT> tmp123;
  if (block97.is_used()) {
    ca_.Bind(&block97, &phi_bb97_5, &phi_bb97_6, &phi_bb97_7, &phi_bb97_11, &phi_bb97_12, &phi_bb97_14);
    std::tie(tmp118, tmp119, tmp120) = FieldSliceSeqTwoByteStringChars_0(state_, TNode<SeqTwoByteString>{tmp107}).Flatten();
    compiler::CodeAssemblerLabel label124(&ca_);
    std::tie(tmp121, tmp122, tmp123) = Subslice_char16_0(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{tmp118}, TNode<IntPtrT>{tmp119}, TNode<IntPtrT>{tmp120}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb97_12}, TNode<IntPtrT>{tmp103}, &label124).Flatten();
    ca_.Goto(&block101, phi_bb97_5, phi_bb97_6, phi_bb97_7, phi_bb97_11, phi_bb97_12, phi_bb97_14, phi_bb97_12);
    if (label124.is_used()) {
      ca_.Bind(&label124);
      ca_.Goto(&block102, phi_bb97_5, phi_bb97_6, phi_bb97_7, phi_bb97_11, phi_bb97_12, phi_bb97_14, phi_bb97_12);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb102_5;
  TNode<IntPtrT> phi_bb102_6;
  TNode<IntPtrT> phi_bb102_7;
  TNode<String> phi_bb102_11;
  TNode<IntPtrT> phi_bb102_12;
  TNode<String> phi_bb102_14;
  TNode<IntPtrT> phi_bb102_19;
  if (block102.is_used()) {
    ca_.Bind(&block102, &phi_bb102_5, &phi_bb102_6, &phi_bb102_7, &phi_bb102_11, &phi_bb102_12, &phi_bb102_14, &phi_bb102_19);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb101_5;
  TNode<IntPtrT> phi_bb101_6;
  TNode<IntPtrT> phi_bb101_7;
  TNode<String> phi_bb101_11;
  TNode<IntPtrT> phi_bb101_12;
  TNode<String> phi_bb101_14;
  TNode<IntPtrT> phi_bb101_19;
  if (block101.is_used()) {
    ca_.Bind(&block101, &phi_bb101_5, &phi_bb101_6, &phi_bb101_7, &phi_bb101_11, &phi_bb101_12, &phi_bb101_14, &phi_bb101_19);
    ca_.Goto(&block79, phi_bb101_5, phi_bb101_6, phi_bb101_7, tmp121, tmp122, tmp123);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb106_5;
  TNode<IntPtrT> phi_bb106_6;
  TNode<IntPtrT> phi_bb106_7;
  TNode<String> phi_bb106_11;
  TNode<IntPtrT> phi_bb106_12;
  TNode<String> phi_bb106_14;
  TNode<ConsString> tmp125;
  if (block106.is_used()) {
    ca_.Bind(&block106, &phi_bb106_5, &phi_bb106_6, &phi_bb106_7, &phi_bb106_11, &phi_bb106_12, &phi_bb106_14);
    compiler::CodeAssemblerLabel label126(&ca_);
    tmp125 = Cast_ConsString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb106_14)}, &label126);
    ca_.Goto(&block109, phi_bb106_5, phi_bb106_6, phi_bb106_7, phi_bb106_11, phi_bb106_12, phi_bb106_14);
    if (label126.is_used()) {
      ca_.Bind(&label126);
      ca_.Goto(&block110, phi_bb106_5, phi_bb106_6, phi_bb106_7, phi_bb106_11, phi_bb106_12, phi_bb106_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb105_5;
  TNode<IntPtrT> phi_bb105_6;
  TNode<IntPtrT> phi_bb105_7;
  TNode<String> phi_bb105_11;
  TNode<IntPtrT> phi_bb105_12;
  TNode<String> phi_bb105_14;
  TNode<IntPtrT> tmp127;
  TNode<String> tmp128;
  if (block105.is_used()) {
    ca_.Bind(&block105, &phi_bb105_5, &phi_bb105_6, &phi_bb105_7, &phi_bb105_11, &phi_bb105_12, &phi_bb105_14);
    tmp127 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp128 = CodeStubAssembler(state_).LoadReference<String>(CodeStubAssembler::Reference{tmp116, tmp127});
    ca_.Goto(&block103, phi_bb105_5, phi_bb105_6, phi_bb105_7, tmp128, phi_bb105_12, phi_bb105_14);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb110_5;
  TNode<IntPtrT> phi_bb110_6;
  TNode<IntPtrT> phi_bb110_7;
  TNode<String> phi_bb110_11;
  TNode<IntPtrT> phi_bb110_12;
  TNode<String> phi_bb110_14;
  TNode<SlicedString> tmp129;
  if (block110.is_used()) {
    ca_.Bind(&block110, &phi_bb110_5, &phi_bb110_6, &phi_bb110_7, &phi_bb110_11, &phi_bb110_12, &phi_bb110_14);
    compiler::CodeAssemblerLabel label130(&ca_);
    tmp129 = Cast_SlicedString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb110_14)}, &label130);
    ca_.Goto(&block113, phi_bb110_5, phi_bb110_6, phi_bb110_7, phi_bb110_11, phi_bb110_12, phi_bb110_14);
    if (label130.is_used()) {
      ca_.Bind(&label130);
      ca_.Goto(&block114, phi_bb110_5, phi_bb110_6, phi_bb110_7, phi_bb110_11, phi_bb110_12, phi_bb110_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb109_5;
  TNode<IntPtrT> phi_bb109_6;
  TNode<IntPtrT> phi_bb109_7;
  TNode<String> phi_bb109_11;
  TNode<IntPtrT> phi_bb109_12;
  TNode<String> phi_bb109_14;
  TNode<String> tmp131;
  if (block109.is_used()) {
    ca_.Bind(&block109, &phi_bb109_5, &phi_bb109_6, &phi_bb109_7, &phi_bb109_11, &phi_bb109_12, &phi_bb109_14);
    tmp131 = Flatten_1(state_, TNode<ConsString>{tmp125});
    ca_.Goto(&block107, phi_bb109_5, phi_bb109_6, phi_bb109_7, tmp131, phi_bb109_12, phi_bb109_14);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb114_5;
  TNode<IntPtrT> phi_bb114_6;
  TNode<IntPtrT> phi_bb114_7;
  TNode<String> phi_bb114_11;
  TNode<IntPtrT> phi_bb114_12;
  TNode<String> phi_bb114_14;
  TNode<ExternalOneByteString> tmp132;
  if (block114.is_used()) {
    ca_.Bind(&block114, &phi_bb114_5, &phi_bb114_6, &phi_bb114_7, &phi_bb114_11, &phi_bb114_12, &phi_bb114_14);
    compiler::CodeAssemblerLabel label133(&ca_);
    tmp132 = Cast_ExternalOneByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb114_14)}, &label133);
    ca_.Goto(&block117, phi_bb114_5, phi_bb114_6, phi_bb114_7, phi_bb114_11, phi_bb114_12, phi_bb114_14);
    if (label133.is_used()) {
      ca_.Bind(&label133);
      ca_.Goto(&block118, phi_bb114_5, phi_bb114_6, phi_bb114_7, phi_bb114_11, phi_bb114_12, phi_bb114_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb113_5;
  TNode<IntPtrT> phi_bb113_6;
  TNode<IntPtrT> phi_bb113_7;
  TNode<String> phi_bb113_11;
  TNode<IntPtrT> phi_bb113_12;
  TNode<String> phi_bb113_14;
  TNode<IntPtrT> tmp134;
  TNode<Smi> tmp135;
  TNode<IntPtrT> tmp136;
  TNode<IntPtrT> tmp137;
  TNode<IntPtrT> tmp138;
  TNode<String> tmp139;
  if (block113.is_used()) {
    ca_.Bind(&block113, &phi_bb113_5, &phi_bb113_6, &phi_bb113_7, &phi_bb113_11, &phi_bb113_12, &phi_bb113_14);
    tmp134 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp135 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{tmp129, tmp134});
    tmp136 = Convert_intptr_Smi_0(state_, TNode<Smi>{tmp135});
    tmp137 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb113_12}, TNode<IntPtrT>{tmp136});
    tmp138 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp139 = CodeStubAssembler(state_).LoadReference<String>(CodeStubAssembler::Reference{tmp129, tmp138});
    ca_.Goto(&block107, phi_bb113_5, phi_bb113_6, phi_bb113_7, tmp139, tmp137, phi_bb113_14);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb118_5;
  TNode<IntPtrT> phi_bb118_6;
  TNode<IntPtrT> phi_bb118_7;
  TNode<String> phi_bb118_11;
  TNode<IntPtrT> phi_bb118_12;
  TNode<String> phi_bb118_14;
  TNode<ExternalTwoByteString> tmp140;
  if (block118.is_used()) {
    ca_.Bind(&block118, &phi_bb118_5, &phi_bb118_6, &phi_bb118_7, &phi_bb118_11, &phi_bb118_12, &phi_bb118_14);
    compiler::CodeAssemblerLabel label141(&ca_);
    tmp140 = Cast_ExternalTwoByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb118_14)}, &label141);
    ca_.Goto(&block125, phi_bb118_5, phi_bb118_6, phi_bb118_7, phi_bb118_11, phi_bb118_12, phi_bb118_14);
    if (label141.is_used()) {
      ca_.Bind(&label141);
      ca_.Goto(&block126, phi_bb118_5, phi_bb118_6, phi_bb118_7, phi_bb118_11, phi_bb118_12, phi_bb118_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb117_5;
  TNode<IntPtrT> phi_bb117_6;
  TNode<IntPtrT> phi_bb117_7;
  TNode<String> phi_bb117_11;
  TNode<IntPtrT> phi_bb117_12;
  TNode<String> phi_bb117_14;
  TNode<RawPtrT> tmp142;
  TNode<IntPtrT> tmp143;
  TNode<Int32T> tmp144;
  TNode<IntPtrT> tmp145;
  TNode<Union<HeapObject, TaggedIndex>> tmp146;
  TNode<IntPtrT> tmp147;
  TNode<IntPtrT> tmp148;
  TNode<Union<HeapObject, TaggedIndex>> tmp149;
  TNode<IntPtrT> tmp150;
  TNode<IntPtrT> tmp151;
  if (block117.is_used()) {
    ca_.Bind(&block117, &phi_bb117_5, &phi_bb117_6, &phi_bb117_7, &phi_bb117_11, &phi_bb117_12, &phi_bb117_14);
    tmp142 = Method_ExternalOneByteString_GetChars_0(state_, TNode<ExternalOneByteString>{tmp132});
    tmp143 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp144 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp132, tmp143});
    tmp145 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp144});
    std::tie(tmp146, tmp147, tmp148) = NewOffHeapConstSlice_char8_0(state_, TNode<RawPtrT>{tmp142}, TNode<IntPtrT>{tmp145}).Flatten();
    compiler::CodeAssemblerLabel label152(&ca_);
    std::tie(tmp149, tmp150, tmp151) = Subslice_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp146}, TNode<IntPtrT>{tmp147}, TNode<IntPtrT>{tmp148}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb117_12}, TNode<IntPtrT>{tmp103}, &label152).Flatten();
    ca_.Goto(&block121, phi_bb117_5, phi_bb117_6, phi_bb117_7, phi_bb117_11, phi_bb117_12, phi_bb117_14, phi_bb117_12);
    if (label152.is_used()) {
      ca_.Bind(&label152);
      ca_.Goto(&block122, phi_bb117_5, phi_bb117_6, phi_bb117_7, phi_bb117_11, phi_bb117_12, phi_bb117_14, phi_bb117_12);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb122_5;
  TNode<IntPtrT> phi_bb122_6;
  TNode<IntPtrT> phi_bb122_7;
  TNode<String> phi_bb122_11;
  TNode<IntPtrT> phi_bb122_12;
  TNode<String> phi_bb122_14;
  TNode<IntPtrT> phi_bb122_22;
  if (block122.is_used()) {
    ca_.Bind(&block122, &phi_bb122_5, &phi_bb122_6, &phi_bb122_7, &phi_bb122_11, &phi_bb122_12, &phi_bb122_14, &phi_bb122_22);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb121_5;
  TNode<IntPtrT> phi_bb121_6;
  TNode<IntPtrT> phi_bb121_7;
  TNode<String> phi_bb121_11;
  TNode<IntPtrT> phi_bb121_12;
  TNode<String> phi_bb121_14;
  TNode<IntPtrT> phi_bb121_22;
  if (block121.is_used()) {
    ca_.Bind(&block121, &phi_bb121_5, &phi_bb121_6, &phi_bb121_7, &phi_bb121_11, &phi_bb121_12, &phi_bb121_14, &phi_bb121_22);
    ca_.Goto(&block81, phi_bb121_5, phi_bb121_6, phi_bb121_7, tmp149, tmp150, tmp151);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb126_5;
  TNode<IntPtrT> phi_bb126_6;
  TNode<IntPtrT> phi_bb126_7;
  TNode<String> phi_bb126_11;
  TNode<IntPtrT> phi_bb126_12;
  TNode<String> phi_bb126_14;
  if (block126.is_used()) {
    ca_.Bind(&block126, &phi_bb126_5, &phi_bb126_6, &phi_bb126_7, &phi_bb126_11, &phi_bb126_12, &phi_bb126_14);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb125_5;
  TNode<IntPtrT> phi_bb125_6;
  TNode<IntPtrT> phi_bb125_7;
  TNode<String> phi_bb125_11;
  TNode<IntPtrT> phi_bb125_12;
  TNode<String> phi_bb125_14;
  TNode<RawPtrT> tmp153;
  TNode<IntPtrT> tmp154;
  TNode<Int32T> tmp155;
  TNode<IntPtrT> tmp156;
  TNode<Union<HeapObject, TaggedIndex>> tmp157;
  TNode<IntPtrT> tmp158;
  TNode<IntPtrT> tmp159;
  TNode<Union<HeapObject, TaggedIndex>> tmp160;
  TNode<IntPtrT> tmp161;
  TNode<IntPtrT> tmp162;
  if (block125.is_used()) {
    ca_.Bind(&block125, &phi_bb125_5, &phi_bb125_6, &phi_bb125_7, &phi_bb125_11, &phi_bb125_12, &phi_bb125_14);
    tmp153 = Method_ExternalTwoByteString_GetChars_0(state_, TNode<ExternalTwoByteString>{tmp140});
    tmp154 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp155 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp140, tmp154});
    tmp156 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp155});
    std::tie(tmp157, tmp158, tmp159) = NewOffHeapConstSlice_char16_0(state_, TNode<RawPtrT>{tmp153}, TNode<IntPtrT>{tmp156}).Flatten();
    compiler::CodeAssemblerLabel label163(&ca_);
    std::tie(tmp160, tmp161, tmp162) = Subslice_char16_0(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{tmp157}, TNode<IntPtrT>{tmp158}, TNode<IntPtrT>{tmp159}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb125_12}, TNode<IntPtrT>{tmp103}, &label163).Flatten();
    ca_.Goto(&block129, phi_bb125_5, phi_bb125_6, phi_bb125_7, phi_bb125_11, phi_bb125_12, phi_bb125_14, phi_bb125_12);
    if (label163.is_used()) {
      ca_.Bind(&label163);
      ca_.Goto(&block130, phi_bb125_5, phi_bb125_6, phi_bb125_7, phi_bb125_11, phi_bb125_12, phi_bb125_14, phi_bb125_12);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb130_5;
  TNode<IntPtrT> phi_bb130_6;
  TNode<IntPtrT> phi_bb130_7;
  TNode<String> phi_bb130_11;
  TNode<IntPtrT> phi_bb130_12;
  TNode<String> phi_bb130_14;
  TNode<IntPtrT> phi_bb130_22;
  if (block130.is_used()) {
    ca_.Bind(&block130, &phi_bb130_5, &phi_bb130_6, &phi_bb130_7, &phi_bb130_11, &phi_bb130_12, &phi_bb130_14, &phi_bb130_22);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb129_5;
  TNode<IntPtrT> phi_bb129_6;
  TNode<IntPtrT> phi_bb129_7;
  TNode<String> phi_bb129_11;
  TNode<IntPtrT> phi_bb129_12;
  TNode<String> phi_bb129_14;
  TNode<IntPtrT> phi_bb129_22;
  if (block129.is_used()) {
    ca_.Bind(&block129, &phi_bb129_5, &phi_bb129_6, &phi_bb129_7, &phi_bb129_11, &phi_bb129_12, &phi_bb129_14, &phi_bb129_22);
    ca_.Goto(&block79, phi_bb129_5, phi_bb129_6, phi_bb129_7, tmp160, tmp161, tmp162);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb107_5;
  TNode<IntPtrT> phi_bb107_6;
  TNode<IntPtrT> phi_bb107_7;
  TNode<String> phi_bb107_11;
  TNode<IntPtrT> phi_bb107_12;
  TNode<String> phi_bb107_14;
  if (block107.is_used()) {
    ca_.Bind(&block107, &phi_bb107_5, &phi_bb107_6, &phi_bb107_7, &phi_bb107_11, &phi_bb107_12, &phi_bb107_14);
    ca_.Goto(&block103, phi_bb107_5, phi_bb107_6, phi_bb107_7, phi_bb107_11, phi_bb107_12, phi_bb107_14);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb103_5;
  TNode<IntPtrT> phi_bb103_6;
  TNode<IntPtrT> phi_bb103_7;
  TNode<String> phi_bb103_11;
  TNode<IntPtrT> phi_bb103_12;
  TNode<String> phi_bb103_14;
  if (block103.is_used()) {
    ca_.Bind(&block103, &phi_bb103_5, &phi_bb103_6, &phi_bb103_7, &phi_bb103_11, &phi_bb103_12, &phi_bb103_14);
    ca_.Goto(&block86, phi_bb103_5, phi_bb103_6, phi_bb103_7, phi_bb103_11, phi_bb103_12);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb85_5;
  TNode<IntPtrT> phi_bb85_6;
  TNode<IntPtrT> phi_bb85_7;
  TNode<String> phi_bb85_11;
  TNode<IntPtrT> phi_bb85_12;
  if (block85.is_used()) {
    ca_.Bind(&block85, &phi_bb85_5, &phi_bb85_6, &phi_bb85_7, &phi_bb85_11, &phi_bb85_12);
    VerifiedUnreachable_0(state_);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb81_5;
  TNode<IntPtrT> phi_bb81_6;
  TNode<IntPtrT> phi_bb81_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb81_8;
  TNode<IntPtrT> phi_bb81_9;
  TNode<IntPtrT> phi_bb81_10;
  TNode<Number> tmp164;
  if (block81.is_used()) {
    ca_.Bind(&block81, &phi_bb81_5, &phi_bb81_6, &phi_bb81_7, &phi_bb81_8, &phi_bb81_9, &phi_bb81_10);
    compiler::CodeAssemblerLabel label165(&ca_);
    tmp164 = LocaleCompareFastPath_char8_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb81_5}, TNode<IntPtrT>{phi_bb81_6}, TNode<IntPtrT>{phi_bb81_7}, TorqueStructUnsafe_0{}}, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb81_8}, TNode<IntPtrT>{phi_bb81_9}, TNode<IntPtrT>{phi_bb81_10}, TorqueStructUnsafe_0{}}, &label165);
    ca_.Goto(&block131, phi_bb81_5, phi_bb81_6, phi_bb81_7, phi_bb81_8, phi_bb81_9, phi_bb81_10, phi_bb81_5, phi_bb81_6, phi_bb81_7, phi_bb81_8, phi_bb81_9, phi_bb81_10);
    if (label165.is_used()) {
      ca_.Bind(&label165);
      ca_.Goto(&block132, phi_bb81_5, phi_bb81_6, phi_bb81_7, phi_bb81_8, phi_bb81_9, phi_bb81_10, phi_bb81_5, phi_bb81_6, phi_bb81_7, phi_bb81_8, phi_bb81_9, phi_bb81_10);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb132_5;
  TNode<IntPtrT> phi_bb132_6;
  TNode<IntPtrT> phi_bb132_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb132_8;
  TNode<IntPtrT> phi_bb132_9;
  TNode<IntPtrT> phi_bb132_10;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb132_11;
  TNode<IntPtrT> phi_bb132_12;
  TNode<IntPtrT> phi_bb132_13;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb132_14;
  TNode<IntPtrT> phi_bb132_15;
  TNode<IntPtrT> phi_bb132_16;
  if (block132.is_used()) {
    ca_.Bind(&block132, &phi_bb132_5, &phi_bb132_6, &phi_bb132_7, &phi_bb132_8, &phi_bb132_9, &phi_bb132_10, &phi_bb132_11, &phi_bb132_12, &phi_bb132_13, &phi_bb132_14, &phi_bb132_15, &phi_bb132_16);
    ca_.Goto(&block2);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb131_5;
  TNode<IntPtrT> phi_bb131_6;
  TNode<IntPtrT> phi_bb131_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb131_8;
  TNode<IntPtrT> phi_bb131_9;
  TNode<IntPtrT> phi_bb131_10;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb131_11;
  TNode<IntPtrT> phi_bb131_12;
  TNode<IntPtrT> phi_bb131_13;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb131_14;
  TNode<IntPtrT> phi_bb131_15;
  TNode<IntPtrT> phi_bb131_16;
  if (block131.is_used()) {
    ca_.Bind(&block131, &phi_bb131_5, &phi_bb131_6, &phi_bb131_7, &phi_bb131_8, &phi_bb131_9, &phi_bb131_10, &phi_bb131_11, &phi_bb131_12, &phi_bb131_13, &phi_bb131_14, &phi_bb131_15, &phi_bb131_16);
    CodeStubAssembler(state_).Return(tmp164);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb79_5;
  TNode<IntPtrT> phi_bb79_6;
  TNode<IntPtrT> phi_bb79_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb79_8;
  TNode<IntPtrT> phi_bb79_9;
  TNode<IntPtrT> phi_bb79_10;
  TNode<Number> tmp166;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_5, &phi_bb79_6, &phi_bb79_7, &phi_bb79_8, &phi_bb79_9, &phi_bb79_10);
    compiler::CodeAssemblerLabel label167(&ca_);
    tmp166 = LocaleCompareFastPath_char8_char16_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb79_5}, TNode<IntPtrT>{phi_bb79_6}, TNode<IntPtrT>{phi_bb79_7}, TorqueStructUnsafe_0{}}, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb79_8}, TNode<IntPtrT>{phi_bb79_9}, TNode<IntPtrT>{phi_bb79_10}, TorqueStructUnsafe_0{}}, &label167);
    ca_.Goto(&block133, phi_bb79_5, phi_bb79_6, phi_bb79_7, phi_bb79_8, phi_bb79_9, phi_bb79_10, phi_bb79_5, phi_bb79_6, phi_bb79_7, phi_bb79_8, phi_bb79_9, phi_bb79_10);
    if (label167.is_used()) {
      ca_.Bind(&label167);
      ca_.Goto(&block134, phi_bb79_5, phi_bb79_6, phi_bb79_7, phi_bb79_8, phi_bb79_9, phi_bb79_10, phi_bb79_5, phi_bb79_6, phi_bb79_7, phi_bb79_8, phi_bb79_9, phi_bb79_10);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb134_5;
  TNode<IntPtrT> phi_bb134_6;
  TNode<IntPtrT> phi_bb134_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb134_8;
  TNode<IntPtrT> phi_bb134_9;
  TNode<IntPtrT> phi_bb134_10;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb134_11;
  TNode<IntPtrT> phi_bb134_12;
  TNode<IntPtrT> phi_bb134_13;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb134_14;
  TNode<IntPtrT> phi_bb134_15;
  TNode<IntPtrT> phi_bb134_16;
  if (block134.is_used()) {
    ca_.Bind(&block134, &phi_bb134_5, &phi_bb134_6, &phi_bb134_7, &phi_bb134_8, &phi_bb134_9, &phi_bb134_10, &phi_bb134_11, &phi_bb134_12, &phi_bb134_13, &phi_bb134_14, &phi_bb134_15, &phi_bb134_16);
    ca_.Goto(&block2);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb133_5;
  TNode<IntPtrT> phi_bb133_6;
  TNode<IntPtrT> phi_bb133_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb133_8;
  TNode<IntPtrT> phi_bb133_9;
  TNode<IntPtrT> phi_bb133_10;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb133_11;
  TNode<IntPtrT> phi_bb133_12;
  TNode<IntPtrT> phi_bb133_13;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb133_14;
  TNode<IntPtrT> phi_bb133_15;
  TNode<IntPtrT> phi_bb133_16;
  if (block133.is_used()) {
    ca_.Bind(&block133, &phi_bb133_5, &phi_bb133_6, &phi_bb133_7, &phi_bb133_8, &phi_bb133_9, &phi_bb133_10, &phi_bb133_11, &phi_bb133_12, &phi_bb133_13, &phi_bb133_14, &phi_bb133_15, &phi_bb133_16);
    CodeStubAssembler(state_).Return(tmp166);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb4_5;
  TNode<IntPtrT> phi_bb4_6;
  TNode<IntPtrT> phi_bb4_7;
  TNode<String> tmp168;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_5, &phi_bb4_6, &phi_bb4_7);
    compiler::CodeAssemblerLabel label169(&ca_);
    tmp168 = Cast_String_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter3}, &label169);
    ca_.Goto(&block139, phi_bb4_5, phi_bb4_6, phi_bb4_7);
    if (label169.is_used()) {
      ca_.Bind(&label169);
      ca_.Goto(&block140, phi_bb4_5, phi_bb4_6, phi_bb4_7);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb140_5;
  TNode<IntPtrT> phi_bb140_6;
  TNode<IntPtrT> phi_bb140_7;
  if (block140.is_used()) {
    ca_.Bind(&block140, &phi_bb140_5, &phi_bb140_6, &phi_bb140_7);
    ca_.Goto(&block2);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb139_5;
  TNode<IntPtrT> phi_bb139_6;
  TNode<IntPtrT> phi_bb139_7;
  TNode<IntPtrT> tmp170;
  TNode<IntPtrT> tmp171;
  TNode<Int32T> tmp172;
  TNode<IntPtrT> tmp173;
  if (block139.is_used()) {
    ca_.Bind(&block139, &phi_bb139_5, &phi_bb139_6, &phi_bb139_7);
    tmp170 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp171 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp172 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp168, tmp171});
    tmp173 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp172});
    ca_.Goto(&block143, phi_bb139_5, phi_bb139_6, phi_bb139_7, tmp168, tmp170);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb143_5;
  TNode<IntPtrT> phi_bb143_6;
  TNode<IntPtrT> phi_bb143_7;
  TNode<String> phi_bb143_11;
  TNode<IntPtrT> phi_bb143_12;
  TNode<BoolT> tmp174;
  if (block143.is_used()) {
    ca_.Bind(&block143, &phi_bb143_5, &phi_bb143_6, &phi_bb143_7, &phi_bb143_11, &phi_bb143_12);
    tmp174 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp174, &block141, std::vector<compiler::Node*>{phi_bb143_5, phi_bb143_6, phi_bb143_7, phi_bb143_11, phi_bb143_12}, &block142, std::vector<compiler::Node*>{phi_bb143_5, phi_bb143_6, phi_bb143_7, phi_bb143_11, phi_bb143_12});
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb141_5;
  TNode<IntPtrT> phi_bb141_6;
  TNode<IntPtrT> phi_bb141_7;
  TNode<String> phi_bb141_11;
  TNode<IntPtrT> phi_bb141_12;
  TNode<SeqOneByteString> tmp175;
  if (block141.is_used()) {
    ca_.Bind(&block141, &phi_bb141_5, &phi_bb141_6, &phi_bb141_7, &phi_bb141_11, &phi_bb141_12);
    compiler::CodeAssemblerLabel label176(&ca_);
    tmp175 = Cast_SeqOneByteString_1(state_, TNode<String>{phi_bb141_11}, &label176);
    ca_.Goto(&block146, phi_bb141_5, phi_bb141_6, phi_bb141_7, phi_bb141_11, phi_bb141_12, phi_bb141_11, phi_bb141_11);
    if (label176.is_used()) {
      ca_.Bind(&label176);
      ca_.Goto(&block147, phi_bb141_5, phi_bb141_6, phi_bb141_7, phi_bb141_11, phi_bb141_12, phi_bb141_11, phi_bb141_11);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb147_5;
  TNode<IntPtrT> phi_bb147_6;
  TNode<IntPtrT> phi_bb147_7;
  TNode<String> phi_bb147_11;
  TNode<IntPtrT> phi_bb147_12;
  TNode<String> phi_bb147_14;
  TNode<String> phi_bb147_15;
  TNode<SeqTwoByteString> tmp177;
  if (block147.is_used()) {
    ca_.Bind(&block147, &phi_bb147_5, &phi_bb147_6, &phi_bb147_7, &phi_bb147_11, &phi_bb147_12, &phi_bb147_14, &phi_bb147_15);
    compiler::CodeAssemblerLabel label178(&ca_);
    tmp177 = Cast_SeqTwoByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb147_14)}, &label178);
    ca_.Goto(&block154, phi_bb147_5, phi_bb147_6, phi_bb147_7, phi_bb147_11, phi_bb147_12, phi_bb147_14);
    if (label178.is_used()) {
      ca_.Bind(&label178);
      ca_.Goto(&block155, phi_bb147_5, phi_bb147_6, phi_bb147_7, phi_bb147_11, phi_bb147_12, phi_bb147_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb146_5;
  TNode<IntPtrT> phi_bb146_6;
  TNode<IntPtrT> phi_bb146_7;
  TNode<String> phi_bb146_11;
  TNode<IntPtrT> phi_bb146_12;
  TNode<String> phi_bb146_14;
  TNode<String> phi_bb146_15;
  TNode<Union<HeapObject, TaggedIndex>> tmp179;
  TNode<IntPtrT> tmp180;
  TNode<IntPtrT> tmp181;
  TNode<Union<HeapObject, TaggedIndex>> tmp182;
  TNode<IntPtrT> tmp183;
  TNode<IntPtrT> tmp184;
  if (block146.is_used()) {
    ca_.Bind(&block146, &phi_bb146_5, &phi_bb146_6, &phi_bb146_7, &phi_bb146_11, &phi_bb146_12, &phi_bb146_14, &phi_bb146_15);
    std::tie(tmp179, tmp180, tmp181) = FieldSliceSeqOneByteStringChars_0(state_, TNode<SeqOneByteString>{tmp175}).Flatten();
    compiler::CodeAssemblerLabel label185(&ca_);
    std::tie(tmp182, tmp183, tmp184) = Subslice_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp179}, TNode<IntPtrT>{tmp180}, TNode<IntPtrT>{tmp181}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb146_12}, TNode<IntPtrT>{tmp173}, &label185).Flatten();
    ca_.Goto(&block150, phi_bb146_5, phi_bb146_6, phi_bb146_7, phi_bb146_11, phi_bb146_12, phi_bb146_14, phi_bb146_12);
    if (label185.is_used()) {
      ca_.Bind(&label185);
      ca_.Goto(&block151, phi_bb146_5, phi_bb146_6, phi_bb146_7, phi_bb146_11, phi_bb146_12, phi_bb146_14, phi_bb146_12);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb151_5;
  TNode<IntPtrT> phi_bb151_6;
  TNode<IntPtrT> phi_bb151_7;
  TNode<String> phi_bb151_11;
  TNode<IntPtrT> phi_bb151_12;
  TNode<String> phi_bb151_14;
  TNode<IntPtrT> phi_bb151_19;
  if (block151.is_used()) {
    ca_.Bind(&block151, &phi_bb151_5, &phi_bb151_6, &phi_bb151_7, &phi_bb151_11, &phi_bb151_12, &phi_bb151_14, &phi_bb151_19);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb150_5;
  TNode<IntPtrT> phi_bb150_6;
  TNode<IntPtrT> phi_bb150_7;
  TNode<String> phi_bb150_11;
  TNode<IntPtrT> phi_bb150_12;
  TNode<String> phi_bb150_14;
  TNode<IntPtrT> phi_bb150_19;
  if (block150.is_used()) {
    ca_.Bind(&block150, &phi_bb150_5, &phi_bb150_6, &phi_bb150_7, &phi_bb150_11, &phi_bb150_12, &phi_bb150_14, &phi_bb150_19);
    ca_.Goto(&block138, phi_bb150_5, phi_bb150_6, phi_bb150_7, tmp182, tmp183, tmp184);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb155_5;
  TNode<IntPtrT> phi_bb155_6;
  TNode<IntPtrT> phi_bb155_7;
  TNode<String> phi_bb155_11;
  TNode<IntPtrT> phi_bb155_12;
  TNode<String> phi_bb155_14;
  TNode<ThinString> tmp186;
  if (block155.is_used()) {
    ca_.Bind(&block155, &phi_bb155_5, &phi_bb155_6, &phi_bb155_7, &phi_bb155_11, &phi_bb155_12, &phi_bb155_14);
    compiler::CodeAssemblerLabel label187(&ca_);
    tmp186 = Cast_ThinString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb155_14)}, &label187);
    ca_.Goto(&block162, phi_bb155_5, phi_bb155_6, phi_bb155_7, phi_bb155_11, phi_bb155_12, phi_bb155_14);
    if (label187.is_used()) {
      ca_.Bind(&label187);
      ca_.Goto(&block163, phi_bb155_5, phi_bb155_6, phi_bb155_7, phi_bb155_11, phi_bb155_12, phi_bb155_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb154_5;
  TNode<IntPtrT> phi_bb154_6;
  TNode<IntPtrT> phi_bb154_7;
  TNode<String> phi_bb154_11;
  TNode<IntPtrT> phi_bb154_12;
  TNode<String> phi_bb154_14;
  TNode<Union<HeapObject, TaggedIndex>> tmp188;
  TNode<IntPtrT> tmp189;
  TNode<IntPtrT> tmp190;
  TNode<Union<HeapObject, TaggedIndex>> tmp191;
  TNode<IntPtrT> tmp192;
  TNode<IntPtrT> tmp193;
  if (block154.is_used()) {
    ca_.Bind(&block154, &phi_bb154_5, &phi_bb154_6, &phi_bb154_7, &phi_bb154_11, &phi_bb154_12, &phi_bb154_14);
    std::tie(tmp188, tmp189, tmp190) = FieldSliceSeqTwoByteStringChars_0(state_, TNode<SeqTwoByteString>{tmp177}).Flatten();
    compiler::CodeAssemblerLabel label194(&ca_);
    std::tie(tmp191, tmp192, tmp193) = Subslice_char16_0(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{tmp188}, TNode<IntPtrT>{tmp189}, TNode<IntPtrT>{tmp190}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb154_12}, TNode<IntPtrT>{tmp173}, &label194).Flatten();
    ca_.Goto(&block158, phi_bb154_5, phi_bb154_6, phi_bb154_7, phi_bb154_11, phi_bb154_12, phi_bb154_14, phi_bb154_12);
    if (label194.is_used()) {
      ca_.Bind(&label194);
      ca_.Goto(&block159, phi_bb154_5, phi_bb154_6, phi_bb154_7, phi_bb154_11, phi_bb154_12, phi_bb154_14, phi_bb154_12);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb159_5;
  TNode<IntPtrT> phi_bb159_6;
  TNode<IntPtrT> phi_bb159_7;
  TNode<String> phi_bb159_11;
  TNode<IntPtrT> phi_bb159_12;
  TNode<String> phi_bb159_14;
  TNode<IntPtrT> phi_bb159_19;
  if (block159.is_used()) {
    ca_.Bind(&block159, &phi_bb159_5, &phi_bb159_6, &phi_bb159_7, &phi_bb159_11, &phi_bb159_12, &phi_bb159_14, &phi_bb159_19);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb158_5;
  TNode<IntPtrT> phi_bb158_6;
  TNode<IntPtrT> phi_bb158_7;
  TNode<String> phi_bb158_11;
  TNode<IntPtrT> phi_bb158_12;
  TNode<String> phi_bb158_14;
  TNode<IntPtrT> phi_bb158_19;
  if (block158.is_used()) {
    ca_.Bind(&block158, &phi_bb158_5, &phi_bb158_6, &phi_bb158_7, &phi_bb158_11, &phi_bb158_12, &phi_bb158_14, &phi_bb158_19);
    ca_.Goto(&block136, phi_bb158_5, phi_bb158_6, phi_bb158_7, tmp191, tmp192, tmp193);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb163_5;
  TNode<IntPtrT> phi_bb163_6;
  TNode<IntPtrT> phi_bb163_7;
  TNode<String> phi_bb163_11;
  TNode<IntPtrT> phi_bb163_12;
  TNode<String> phi_bb163_14;
  TNode<ConsString> tmp195;
  if (block163.is_used()) {
    ca_.Bind(&block163, &phi_bb163_5, &phi_bb163_6, &phi_bb163_7, &phi_bb163_11, &phi_bb163_12, &phi_bb163_14);
    compiler::CodeAssemblerLabel label196(&ca_);
    tmp195 = Cast_ConsString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb163_14)}, &label196);
    ca_.Goto(&block166, phi_bb163_5, phi_bb163_6, phi_bb163_7, phi_bb163_11, phi_bb163_12, phi_bb163_14);
    if (label196.is_used()) {
      ca_.Bind(&label196);
      ca_.Goto(&block167, phi_bb163_5, phi_bb163_6, phi_bb163_7, phi_bb163_11, phi_bb163_12, phi_bb163_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb162_5;
  TNode<IntPtrT> phi_bb162_6;
  TNode<IntPtrT> phi_bb162_7;
  TNode<String> phi_bb162_11;
  TNode<IntPtrT> phi_bb162_12;
  TNode<String> phi_bb162_14;
  TNode<IntPtrT> tmp197;
  TNode<String> tmp198;
  if (block162.is_used()) {
    ca_.Bind(&block162, &phi_bb162_5, &phi_bb162_6, &phi_bb162_7, &phi_bb162_11, &phi_bb162_12, &phi_bb162_14);
    tmp197 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp198 = CodeStubAssembler(state_).LoadReference<String>(CodeStubAssembler::Reference{tmp186, tmp197});
    ca_.Goto(&block160, phi_bb162_5, phi_bb162_6, phi_bb162_7, tmp198, phi_bb162_12, phi_bb162_14);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb167_5;
  TNode<IntPtrT> phi_bb167_6;
  TNode<IntPtrT> phi_bb167_7;
  TNode<String> phi_bb167_11;
  TNode<IntPtrT> phi_bb167_12;
  TNode<String> phi_bb167_14;
  TNode<SlicedString> tmp199;
  if (block167.is_used()) {
    ca_.Bind(&block167, &phi_bb167_5, &phi_bb167_6, &phi_bb167_7, &phi_bb167_11, &phi_bb167_12, &phi_bb167_14);
    compiler::CodeAssemblerLabel label200(&ca_);
    tmp199 = Cast_SlicedString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb167_14)}, &label200);
    ca_.Goto(&block170, phi_bb167_5, phi_bb167_6, phi_bb167_7, phi_bb167_11, phi_bb167_12, phi_bb167_14);
    if (label200.is_used()) {
      ca_.Bind(&label200);
      ca_.Goto(&block171, phi_bb167_5, phi_bb167_6, phi_bb167_7, phi_bb167_11, phi_bb167_12, phi_bb167_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb166_5;
  TNode<IntPtrT> phi_bb166_6;
  TNode<IntPtrT> phi_bb166_7;
  TNode<String> phi_bb166_11;
  TNode<IntPtrT> phi_bb166_12;
  TNode<String> phi_bb166_14;
  TNode<String> tmp201;
  if (block166.is_used()) {
    ca_.Bind(&block166, &phi_bb166_5, &phi_bb166_6, &phi_bb166_7, &phi_bb166_11, &phi_bb166_12, &phi_bb166_14);
    tmp201 = Flatten_1(state_, TNode<ConsString>{tmp195});
    ca_.Goto(&block164, phi_bb166_5, phi_bb166_6, phi_bb166_7, tmp201, phi_bb166_12, phi_bb166_14);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb171_5;
  TNode<IntPtrT> phi_bb171_6;
  TNode<IntPtrT> phi_bb171_7;
  TNode<String> phi_bb171_11;
  TNode<IntPtrT> phi_bb171_12;
  TNode<String> phi_bb171_14;
  TNode<ExternalOneByteString> tmp202;
  if (block171.is_used()) {
    ca_.Bind(&block171, &phi_bb171_5, &phi_bb171_6, &phi_bb171_7, &phi_bb171_11, &phi_bb171_12, &phi_bb171_14);
    compiler::CodeAssemblerLabel label203(&ca_);
    tmp202 = Cast_ExternalOneByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb171_14)}, &label203);
    ca_.Goto(&block174, phi_bb171_5, phi_bb171_6, phi_bb171_7, phi_bb171_11, phi_bb171_12, phi_bb171_14);
    if (label203.is_used()) {
      ca_.Bind(&label203);
      ca_.Goto(&block175, phi_bb171_5, phi_bb171_6, phi_bb171_7, phi_bb171_11, phi_bb171_12, phi_bb171_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb170_5;
  TNode<IntPtrT> phi_bb170_6;
  TNode<IntPtrT> phi_bb170_7;
  TNode<String> phi_bb170_11;
  TNode<IntPtrT> phi_bb170_12;
  TNode<String> phi_bb170_14;
  TNode<IntPtrT> tmp204;
  TNode<Smi> tmp205;
  TNode<IntPtrT> tmp206;
  TNode<IntPtrT> tmp207;
  TNode<IntPtrT> tmp208;
  TNode<String> tmp209;
  if (block170.is_used()) {
    ca_.Bind(&block170, &phi_bb170_5, &phi_bb170_6, &phi_bb170_7, &phi_bb170_11, &phi_bb170_12, &phi_bb170_14);
    tmp204 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp205 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{tmp199, tmp204});
    tmp206 = Convert_intptr_Smi_0(state_, TNode<Smi>{tmp205});
    tmp207 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb170_12}, TNode<IntPtrT>{tmp206});
    tmp208 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp209 = CodeStubAssembler(state_).LoadReference<String>(CodeStubAssembler::Reference{tmp199, tmp208});
    ca_.Goto(&block164, phi_bb170_5, phi_bb170_6, phi_bb170_7, tmp209, tmp207, phi_bb170_14);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb175_5;
  TNode<IntPtrT> phi_bb175_6;
  TNode<IntPtrT> phi_bb175_7;
  TNode<String> phi_bb175_11;
  TNode<IntPtrT> phi_bb175_12;
  TNode<String> phi_bb175_14;
  TNode<ExternalTwoByteString> tmp210;
  if (block175.is_used()) {
    ca_.Bind(&block175, &phi_bb175_5, &phi_bb175_6, &phi_bb175_7, &phi_bb175_11, &phi_bb175_12, &phi_bb175_14);
    compiler::CodeAssemblerLabel label211(&ca_);
    tmp210 = Cast_ExternalTwoByteString_1(state_, TNode<String>{ca_.UncheckedCast<String>(phi_bb175_14)}, &label211);
    ca_.Goto(&block182, phi_bb175_5, phi_bb175_6, phi_bb175_7, phi_bb175_11, phi_bb175_12, phi_bb175_14);
    if (label211.is_used()) {
      ca_.Bind(&label211);
      ca_.Goto(&block183, phi_bb175_5, phi_bb175_6, phi_bb175_7, phi_bb175_11, phi_bb175_12, phi_bb175_14);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb174_5;
  TNode<IntPtrT> phi_bb174_6;
  TNode<IntPtrT> phi_bb174_7;
  TNode<String> phi_bb174_11;
  TNode<IntPtrT> phi_bb174_12;
  TNode<String> phi_bb174_14;
  TNode<RawPtrT> tmp212;
  TNode<IntPtrT> tmp213;
  TNode<Int32T> tmp214;
  TNode<IntPtrT> tmp215;
  TNode<Union<HeapObject, TaggedIndex>> tmp216;
  TNode<IntPtrT> tmp217;
  TNode<IntPtrT> tmp218;
  TNode<Union<HeapObject, TaggedIndex>> tmp219;
  TNode<IntPtrT> tmp220;
  TNode<IntPtrT> tmp221;
  if (block174.is_used()) {
    ca_.Bind(&block174, &phi_bb174_5, &phi_bb174_6, &phi_bb174_7, &phi_bb174_11, &phi_bb174_12, &phi_bb174_14);
    tmp212 = Method_ExternalOneByteString_GetChars_0(state_, TNode<ExternalOneByteString>{tmp202});
    tmp213 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp214 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp202, tmp213});
    tmp215 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp214});
    std::tie(tmp216, tmp217, tmp218) = NewOffHeapConstSlice_char8_0(state_, TNode<RawPtrT>{tmp212}, TNode<IntPtrT>{tmp215}).Flatten();
    compiler::CodeAssemblerLabel label222(&ca_);
    std::tie(tmp219, tmp220, tmp221) = Subslice_char8_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp216}, TNode<IntPtrT>{tmp217}, TNode<IntPtrT>{tmp218}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb174_12}, TNode<IntPtrT>{tmp173}, &label222).Flatten();
    ca_.Goto(&block178, phi_bb174_5, phi_bb174_6, phi_bb174_7, phi_bb174_11, phi_bb174_12, phi_bb174_14, phi_bb174_12);
    if (label222.is_used()) {
      ca_.Bind(&label222);
      ca_.Goto(&block179, phi_bb174_5, phi_bb174_6, phi_bb174_7, phi_bb174_11, phi_bb174_12, phi_bb174_14, phi_bb174_12);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb179_5;
  TNode<IntPtrT> phi_bb179_6;
  TNode<IntPtrT> phi_bb179_7;
  TNode<String> phi_bb179_11;
  TNode<IntPtrT> phi_bb179_12;
  TNode<String> phi_bb179_14;
  TNode<IntPtrT> phi_bb179_22;
  if (block179.is_used()) {
    ca_.Bind(&block179, &phi_bb179_5, &phi_bb179_6, &phi_bb179_7, &phi_bb179_11, &phi_bb179_12, &phi_bb179_14, &phi_bb179_22);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb178_5;
  TNode<IntPtrT> phi_bb178_6;
  TNode<IntPtrT> phi_bb178_7;
  TNode<String> phi_bb178_11;
  TNode<IntPtrT> phi_bb178_12;
  TNode<String> phi_bb178_14;
  TNode<IntPtrT> phi_bb178_22;
  if (block178.is_used()) {
    ca_.Bind(&block178, &phi_bb178_5, &phi_bb178_6, &phi_bb178_7, &phi_bb178_11, &phi_bb178_12, &phi_bb178_14, &phi_bb178_22);
    ca_.Goto(&block138, phi_bb178_5, phi_bb178_6, phi_bb178_7, tmp219, tmp220, tmp221);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb183_5;
  TNode<IntPtrT> phi_bb183_6;
  TNode<IntPtrT> phi_bb183_7;
  TNode<String> phi_bb183_11;
  TNode<IntPtrT> phi_bb183_12;
  TNode<String> phi_bb183_14;
  if (block183.is_used()) {
    ca_.Bind(&block183, &phi_bb183_5, &phi_bb183_6, &phi_bb183_7, &phi_bb183_11, &phi_bb183_12, &phi_bb183_14);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb182_5;
  TNode<IntPtrT> phi_bb182_6;
  TNode<IntPtrT> phi_bb182_7;
  TNode<String> phi_bb182_11;
  TNode<IntPtrT> phi_bb182_12;
  TNode<String> phi_bb182_14;
  TNode<RawPtrT> tmp223;
  TNode<IntPtrT> tmp224;
  TNode<Int32T> tmp225;
  TNode<IntPtrT> tmp226;
  TNode<Union<HeapObject, TaggedIndex>> tmp227;
  TNode<IntPtrT> tmp228;
  TNode<IntPtrT> tmp229;
  TNode<Union<HeapObject, TaggedIndex>> tmp230;
  TNode<IntPtrT> tmp231;
  TNode<IntPtrT> tmp232;
  if (block182.is_used()) {
    ca_.Bind(&block182, &phi_bb182_5, &phi_bb182_6, &phi_bb182_7, &phi_bb182_11, &phi_bb182_12, &phi_bb182_14);
    tmp223 = Method_ExternalTwoByteString_GetChars_0(state_, TNode<ExternalTwoByteString>{tmp210});
    tmp224 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp225 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp210, tmp224});
    tmp226 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp225});
    std::tie(tmp227, tmp228, tmp229) = NewOffHeapConstSlice_char16_0(state_, TNode<RawPtrT>{tmp223}, TNode<IntPtrT>{tmp226}).Flatten();
    compiler::CodeAssemblerLabel label233(&ca_);
    std::tie(tmp230, tmp231, tmp232) = Subslice_char16_0(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{tmp227}, TNode<IntPtrT>{tmp228}, TNode<IntPtrT>{tmp229}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb182_12}, TNode<IntPtrT>{tmp173}, &label233).Flatten();
    ca_.Goto(&block186, phi_bb182_5, phi_bb182_6, phi_bb182_7, phi_bb182_11, phi_bb182_12, phi_bb182_14, phi_bb182_12);
    if (label233.is_used()) {
      ca_.Bind(&label233);
      ca_.Goto(&block187, phi_bb182_5, phi_bb182_6, phi_bb182_7, phi_bb182_11, phi_bb182_12, phi_bb182_14, phi_bb182_12);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb187_5;
  TNode<IntPtrT> phi_bb187_6;
  TNode<IntPtrT> phi_bb187_7;
  TNode<String> phi_bb187_11;
  TNode<IntPtrT> phi_bb187_12;
  TNode<String> phi_bb187_14;
  TNode<IntPtrT> phi_bb187_22;
  if (block187.is_used()) {
    ca_.Bind(&block187, &phi_bb187_5, &phi_bb187_6, &phi_bb187_7, &phi_bb187_11, &phi_bb187_12, &phi_bb187_14, &phi_bb187_22);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb186_5;
  TNode<IntPtrT> phi_bb186_6;
  TNode<IntPtrT> phi_bb186_7;
  TNode<String> phi_bb186_11;
  TNode<IntPtrT> phi_bb186_12;
  TNode<String> phi_bb186_14;
  TNode<IntPtrT> phi_bb186_22;
  if (block186.is_used()) {
    ca_.Bind(&block186, &phi_bb186_5, &phi_bb186_6, &phi_bb186_7, &phi_bb186_11, &phi_bb186_12, &phi_bb186_14, &phi_bb186_22);
    ca_.Goto(&block136, phi_bb186_5, phi_bb186_6, phi_bb186_7, tmp230, tmp231, tmp232);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb164_5;
  TNode<IntPtrT> phi_bb164_6;
  TNode<IntPtrT> phi_bb164_7;
  TNode<String> phi_bb164_11;
  TNode<IntPtrT> phi_bb164_12;
  TNode<String> phi_bb164_14;
  if (block164.is_used()) {
    ca_.Bind(&block164, &phi_bb164_5, &phi_bb164_6, &phi_bb164_7, &phi_bb164_11, &phi_bb164_12, &phi_bb164_14);
    ca_.Goto(&block160, phi_bb164_5, phi_bb164_6, phi_bb164_7, phi_bb164_11, phi_bb164_12, phi_bb164_14);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb160_5;
  TNode<IntPtrT> phi_bb160_6;
  TNode<IntPtrT> phi_bb160_7;
  TNode<String> phi_bb160_11;
  TNode<IntPtrT> phi_bb160_12;
  TNode<String> phi_bb160_14;
  if (block160.is_used()) {
    ca_.Bind(&block160, &phi_bb160_5, &phi_bb160_6, &phi_bb160_7, &phi_bb160_11, &phi_bb160_12, &phi_bb160_14);
    ca_.Goto(&block143, phi_bb160_5, phi_bb160_6, phi_bb160_7, phi_bb160_11, phi_bb160_12);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb142_5;
  TNode<IntPtrT> phi_bb142_6;
  TNode<IntPtrT> phi_bb142_7;
  TNode<String> phi_bb142_11;
  TNode<IntPtrT> phi_bb142_12;
  if (block142.is_used()) {
    ca_.Bind(&block142, &phi_bb142_5, &phi_bb142_6, &phi_bb142_7, &phi_bb142_11, &phi_bb142_12);
    VerifiedUnreachable_0(state_);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb138_5;
  TNode<IntPtrT> phi_bb138_6;
  TNode<IntPtrT> phi_bb138_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb138_8;
  TNode<IntPtrT> phi_bb138_9;
  TNode<IntPtrT> phi_bb138_10;
  TNode<Number> tmp234;
  if (block138.is_used()) {
    ca_.Bind(&block138, &phi_bb138_5, &phi_bb138_6, &phi_bb138_7, &phi_bb138_8, &phi_bb138_9, &phi_bb138_10);
    compiler::CodeAssemblerLabel label235(&ca_);
    tmp234 = LocaleCompareFastPath_char16_char8_0(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb138_5}, TNode<IntPtrT>{phi_bb138_6}, TNode<IntPtrT>{phi_bb138_7}, TorqueStructUnsafe_0{}}, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb138_8}, TNode<IntPtrT>{phi_bb138_9}, TNode<IntPtrT>{phi_bb138_10}, TorqueStructUnsafe_0{}}, &label235);
    ca_.Goto(&block188, phi_bb138_5, phi_bb138_6, phi_bb138_7, phi_bb138_8, phi_bb138_9, phi_bb138_10, phi_bb138_5, phi_bb138_6, phi_bb138_7, phi_bb138_8, phi_bb138_9, phi_bb138_10);
    if (label235.is_used()) {
      ca_.Bind(&label235);
      ca_.Goto(&block189, phi_bb138_5, phi_bb138_6, phi_bb138_7, phi_bb138_8, phi_bb138_9, phi_bb138_10, phi_bb138_5, phi_bb138_6, phi_bb138_7, phi_bb138_8, phi_bb138_9, phi_bb138_10);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb189_5;
  TNode<IntPtrT> phi_bb189_6;
  TNode<IntPtrT> phi_bb189_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb189_8;
  TNode<IntPtrT> phi_bb189_9;
  TNode<IntPtrT> phi_bb189_10;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb189_11;
  TNode<IntPtrT> phi_bb189_12;
  TNode<IntPtrT> phi_bb189_13;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb189_14;
  TNode<IntPtrT> phi_bb189_15;
  TNode<IntPtrT> phi_bb189_16;
  if (block189.is_used()) {
    ca_.Bind(&block189, &phi_bb189_5, &phi_bb189_6, &phi_bb189_7, &phi_bb189_8, &phi_bb189_9, &phi_bb189_10, &phi_bb189_11, &phi_bb189_12, &phi_bb189_13, &phi_bb189_14, &phi_bb189_15, &phi_bb189_16);
    ca_.Goto(&block2);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb188_5;
  TNode<IntPtrT> phi_bb188_6;
  TNode<IntPtrT> phi_bb188_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb188_8;
  TNode<IntPtrT> phi_bb188_9;
  TNode<IntPtrT> phi_bb188_10;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb188_11;
  TNode<IntPtrT> phi_bb188_12;
  TNode<IntPtrT> phi_bb188_13;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb188_14;
  TNode<IntPtrT> phi_bb188_15;
  TNode<IntPtrT> phi_bb188_16;
  if (block188.is_used()) {
    ca_.Bind(&block188, &phi_bb188_5, &phi_bb188_6, &phi_bb188_7, &phi_bb188_8, &phi_bb188_9, &phi_bb188_10, &phi_bb188_11, &phi_bb188_12, &phi_bb188_13, &phi_bb188_14, &phi_bb188_15, &phi_bb188_16);
    CodeStubAssembler(state_).Return(tmp234);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb136_5;
  TNode<IntPtrT> phi_bb136_6;
  TNode<IntPtrT> phi_bb136_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb136_8;
  TNode<IntPtrT> phi_bb136_9;
  TNode<IntPtrT> phi_bb136_10;
  TNode<Number> tmp236;
  if (block136.is_used()) {
    ca_.Bind(&block136, &phi_bb136_5, &phi_bb136_6, &phi_bb136_7, &phi_bb136_8, &phi_bb136_9, &phi_bb136_10);
    compiler::CodeAssemblerLabel label237(&ca_);
    tmp236 = LocaleCompareFastPath_char16_char16_0(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb136_5}, TNode<IntPtrT>{phi_bb136_6}, TNode<IntPtrT>{phi_bb136_7}, TorqueStructUnsafe_0{}}, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb136_8}, TNode<IntPtrT>{phi_bb136_9}, TNode<IntPtrT>{phi_bb136_10}, TorqueStructUnsafe_0{}}, &label237);
    ca_.Goto(&block190, phi_bb136_5, phi_bb136_6, phi_bb136_7, phi_bb136_8, phi_bb136_9, phi_bb136_10, phi_bb136_5, phi_bb136_6, phi_bb136_7, phi_bb136_8, phi_bb136_9, phi_bb136_10);
    if (label237.is_used()) {
      ca_.Bind(&label237);
      ca_.Goto(&block191, phi_bb136_5, phi_bb136_6, phi_bb136_7, phi_bb136_8, phi_bb136_9, phi_bb136_10, phi_bb136_5, phi_bb136_6, phi_bb136_7, phi_bb136_8, phi_bb136_9, phi_bb136_10);
    }
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb191_5;
  TNode<IntPtrT> phi_bb191_6;
  TNode<IntPtrT> phi_bb191_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb191_8;
  TNode<IntPtrT> phi_bb191_9;
  TNode<IntPtrT> phi_bb191_10;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb191_11;
  TNode<IntPtrT> phi_bb191_12;
  TNode<IntPtrT> phi_bb191_13;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb191_14;
  TNode<IntPtrT> phi_bb191_15;
  TNode<IntPtrT> phi_bb191_16;
  if (block191.is_used()) {
    ca_.Bind(&block191, &phi_bb191_5, &phi_bb191_6, &phi_bb191_7, &phi_bb191_8, &phi_bb191_9, &phi_bb191_10, &phi_bb191_11, &phi_bb191_12, &phi_bb191_13, &phi_bb191_14, &phi_bb191_15, &phi_bb191_16);
    ca_.Goto(&block2);
  }

  TNode<Union<HeapObject, TaggedIndex>> phi_bb190_5;
  TNode<IntPtrT> phi_bb190_6;
  TNode<IntPtrT> phi_bb190_7;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb190_8;
  TNode<IntPtrT> phi_bb190_9;
  TNode<IntPtrT> phi_bb190_10;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb190_11;
  TNode<IntPtrT> phi_bb190_12;
  TNode<IntPtrT> phi_bb190_13;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb190_14;
  TNode<IntPtrT> phi_bb190_15;
  TNode<IntPtrT> phi_bb190_16;
  if (block190.is_used()) {
    ca_.Bind(&block190, &phi_bb190_5, &phi_bb190_6, &phi_bb190_7, &phi_bb190_8, &phi_bb190_9, &phi_bb190_10, &phi_bb190_11, &phi_bb190_12, &phi_bb190_13, &phi_bb190_14, &phi_bb190_15, &phi_bb190_16);
    CodeStubAssembler(state_).Return(tmp236);
  }

  TNode<JSAny> tmp238;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp238 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSAny>{parameter2}, TNode<JSAny>{parameter3}, TNode<JSAny>{parameter4});
    CodeStubAssembler(state_).Return(tmp238);
  }
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=18&c=7
TNode<Uint32T> Convert_uint32_char16_0(compiler::CodeAssemblerState* state_, TNode<Uint16T> p_i) {
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
  return TNode<Uint32T>{p_i};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=129&c=21
TNode<String> Cast_String_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<HeapObject> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = CodeStubAssembler(state_).TaggedToHeapObject(TNode<Object>{p_o}, &label1);
    ca_.Goto(&block3);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block4);
    }
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    ca_.Goto(&block1);
  }

  TNode<String> tmp2;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_String_0(state_, TNode<HeapObject>{tmp0}, &label3);
    ca_.Goto(&block5);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    ca_.Goto(&block1);
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block7);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(label_CastError);
  }

    ca_.Bind(&block7);
  return TNode<String>{tmp2};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=142&c=17
TorqueStructSlice_char8_ConstReference_char8_0 Subslice_char8_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_char8_ConstReference_char8_0 p_slice, TNode<IntPtrT> p_start, TNode<IntPtrT> p_length, compiler::CodeAssemblerLabel* label_OutOfBounds) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<UintPtrT> tmp0;
  TNode<UintPtrT> tmp1;
  TNode<BoolT> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{p_length});
    tmp1 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{p_slice.length});
    tmp2 = CodeStubAssembler(state_).UintPtrGreaterThan(TNode<UintPtrT>{tmp0}, TNode<UintPtrT>{tmp1});
    ca_.Branch(tmp2, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block1);
  }

  TNode<UintPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<UintPtrT> tmp5;
  TNode<BoolT> tmp6;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp3 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{p_start});
    tmp4 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{p_slice.length}, TNode<IntPtrT>{p_length});
    tmp5 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{tmp4});
    tmp6 = CodeStubAssembler(state_).UintPtrGreaterThan(TNode<UintPtrT>{tmp3}, TNode<UintPtrT>{tmp5});
    ca_.Branch(tmp6, &block5, std::vector<compiler::Node*>{}, &block6, std::vector<compiler::Node*>{});
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<Union<HeapObject, TaggedIndex>> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<IntPtrT> tmp11;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp7 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{p_start});
    tmp8 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_slice.offset}, TNode<IntPtrT>{tmp7});
    std::tie(tmp9, tmp10, tmp11) = NewConstSlice_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_slice.object}, TNode<IntPtrT>{tmp8}, TNode<IntPtrT>{p_length}).Flatten();
    ca_.Goto(&block7);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(label_OutOfBounds);
  }

    ca_.Bind(&block7);
  return TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{tmp9}, TNode<IntPtrT>{tmp10}, TNode<IntPtrT>{tmp11}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=145&c=20
TNode<Number> LocaleCompareFastPath_char8_char8_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_char8_ConstReference_char8_0 p_left, TorqueStructSlice_char8_ConstReference_char8_0 p_right, compiler::CodeAssemblerLabel* label_Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BoolT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block42(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block78(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block81(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{p_left.length}, TNode<IntPtrT>{p_right.length});
    ca_.Branch(tmp1, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block5, p_left.length);
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    ca_.Goto(&block5, p_right.length);
  }

  TNode<IntPtrT> phi_bb5_7;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_7);
    ca_.Goto(&block9, tmp0);
  }

  TNode<IntPtrT> phi_bb9_6;
  TNode<BoolT> tmp2;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_6);
    tmp2 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb9_6}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp2, &block7, std::vector<compiler::Node*>{phi_bb9_6}, &block8, std::vector<compiler::Node*>{phi_bb9_6});
  }

  TNode<IntPtrT> phi_bb7_6;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Union<HeapObject, TaggedIndex>> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<Uint8T> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<Union<HeapObject, TaggedIndex>> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<Uint8T> tmp12;
  TNode<BoolT> tmp13;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_6);
    tmp3 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb7_6});
    tmp4 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp3});
    std::tie(tmp5, tmp6) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp4}).Flatten();
    tmp7 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp5, tmp6});
    tmp8 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb7_6});
    tmp9 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp8});
    std::tie(tmp10, tmp11) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp9}).Flatten();
    tmp12 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp10, tmp11});
    tmp13 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp7}, TNode<Uint32T>{tmp12});
    ca_.Branch(tmp13, &block12, std::vector<compiler::Node*>{phi_bb7_6}, &block13, std::vector<compiler::Node*>{phi_bb7_6});
  }

  TNode<IntPtrT> phi_bb12_6;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_6);
    ca_.Goto(&block8, phi_bb12_6);
  }

  TNode<IntPtrT> phi_bb13_6;
  TNode<Uint32T> tmp14;
  TNode<Uint32T> tmp15;
  TNode<BoolT> tmp16;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_6);
    tmp14 = Convert_uint32_char8_0(state_, TNode<Uint8T>{tmp7});
    tmp15 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x80ull));
    tmp16 = CodeStubAssembler(state_).Uint32GreaterThanOrEqual(TNode<Uint32T>{tmp14}, TNode<Uint32T>{tmp15});
    ca_.Branch(tmp16, &block14, std::vector<compiler::Node*>{phi_bb13_6}, &block15, std::vector<compiler::Node*>{phi_bb13_6});
  }

  TNode<IntPtrT> phi_bb14_6;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_6);
    ca_.Goto(&block8, phi_bb14_6);
  }

  TNode<IntPtrT> phi_bb15_6;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_6);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb15_6}, TNode<IntPtrT>{tmp17});
    ca_.Goto(&block9, tmp18);
  }

  TNode<IntPtrT> phi_bb8_6;
  TNode<BoolT> tmp19;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_6);
    tmp19 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb8_6}, TNode<IntPtrT>{p_left.length});
    ca_.Branch(tmp19, &block18, std::vector<compiler::Node*>{phi_bb8_6}, &block19, std::vector<compiler::Node*>{phi_bb8_6});
  }

  TNode<IntPtrT> phi_bb18_6;
  TNode<BoolT> tmp20;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_6);
    tmp20 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb18_6}, TNode<IntPtrT>{p_right.length});
    ca_.Goto(&block20, phi_bb18_6, tmp20);
  }

  TNode<IntPtrT> phi_bb19_6;
  TNode<BoolT> tmp21;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_6);
    tmp21 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block20, phi_bb19_6, tmp21);
  }

  TNode<IntPtrT> phi_bb20_6;
  TNode<BoolT> phi_bb20_9;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_6, &phi_bb20_9);
    ca_.Branch(phi_bb20_9, &block16, std::vector<compiler::Node*>{phi_bb20_6}, &block17, std::vector<compiler::Node*>{phi_bb20_6});
  }

  TNode<IntPtrT> phi_bb16_6;
  TNode<Number> tmp22;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_6);
    tmp22 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2, tmp22);
  }

  TNode<IntPtrT> phi_bb17_6;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_6);
    ca_.Goto(&block23, phi_bb17_6, phi_bb17_6);
  }

  TNode<IntPtrT> phi_bb23_6;
  TNode<IntPtrT> phi_bb23_8;
  TNode<BoolT> tmp23;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_6, &phi_bb23_8);
    tmp23 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb23_8}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp23, &block21, std::vector<compiler::Node*>{phi_bb23_6, phi_bb23_8}, &block22, std::vector<compiler::Node*>{phi_bb23_6, phi_bb23_8});
  }

  TNode<IntPtrT> phi_bb21_6;
  TNode<IntPtrT> phi_bb21_8;
  TNode<IntPtrT> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<Union<HeapObject, TaggedIndex>> tmp26;
  TNode<IntPtrT> tmp27;
  TNode<Uint8T> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<Union<HeapObject, TaggedIndex>> tmp31;
  TNode<IntPtrT> tmp32;
  TNode<Uint8T> tmp33;
  TNode<Uint8T> tmp34;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_6, &phi_bb21_8);
    tmp24 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb21_8});
    tmp25 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp24});
    std::tie(tmp26, tmp27) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp25}).Flatten();
    tmp28 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp26, tmp27});
    tmp29 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb21_8});
    tmp30 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp29});
    std::tie(tmp31, tmp32) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp30}).Flatten();
    tmp33 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp31, tmp32});
    compiler::CodeAssemblerLabel label35(&ca_);
    tmp34 = IntlAsciiCollationWeightL1_0(state_, TNode<Uint8T>{tmp28}, &label35);
    ca_.Goto(&block26, phi_bb21_6, phi_bb21_8);
    if (label35.is_used()) {
      ca_.Bind(&label35);
      ca_.Goto(&block27, phi_bb21_6, phi_bb21_8);
    }
  }

  TNode<IntPtrT> phi_bb27_6;
  TNode<IntPtrT> phi_bb27_8;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_6, &phi_bb27_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb26_6;
  TNode<IntPtrT> phi_bb26_8;
  TNode<Uint32T> tmp36;
  TNode<BoolT> tmp37;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_6, &phi_bb26_8);
    tmp36 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp37 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp36});
    ca_.Branch(tmp37, &block28, std::vector<compiler::Node*>{phi_bb26_6, phi_bb26_8}, &block29, std::vector<compiler::Node*>{phi_bb26_6, phi_bb26_8});
  }

  TNode<IntPtrT> phi_bb28_6;
  TNode<IntPtrT> phi_bb28_8;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_6, &phi_bb28_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb29_6;
  TNode<IntPtrT> phi_bb29_8;
  TNode<Uint8T> tmp38;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_6, &phi_bb29_8);
    compiler::CodeAssemblerLabel label39(&ca_);
    tmp38 = IntlAsciiCollationWeightL1_0(state_, TNode<Uint8T>{tmp33}, &label39);
    ca_.Goto(&block30, phi_bb29_6, phi_bb29_8);
    if (label39.is_used()) {
      ca_.Bind(&label39);
      ca_.Goto(&block31, phi_bb29_6, phi_bb29_8);
    }
  }

  TNode<IntPtrT> phi_bb31_6;
  TNode<IntPtrT> phi_bb31_8;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_6, &phi_bb31_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb30_6;
  TNode<IntPtrT> phi_bb30_8;
  TNode<Uint32T> tmp40;
  TNode<BoolT> tmp41;
  if (block30.is_used()) {
    ca_.Bind(&block30, &phi_bb30_6, &phi_bb30_8);
    tmp40 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp41 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp38}, TNode<Uint32T>{tmp40});
    ca_.Branch(tmp41, &block32, std::vector<compiler::Node*>{phi_bb30_6, phi_bb30_8}, &block33, std::vector<compiler::Node*>{phi_bb30_6, phi_bb30_8});
  }

  TNode<IntPtrT> phi_bb32_6;
  TNode<IntPtrT> phi_bb32_8;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_6, &phi_bb32_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb33_6;
  TNode<IntPtrT> phi_bb33_8;
  TNode<BoolT> tmp42;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_6, &phi_bb33_8);
    tmp42 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp38});
    ca_.Branch(tmp42, &block34, std::vector<compiler::Node*>{phi_bb33_6, phi_bb33_8}, &block35, std::vector<compiler::Node*>{phi_bb33_6, phi_bb33_8});
  }

  TNode<IntPtrT> phi_bb34_6;
  TNode<IntPtrT> phi_bb34_8;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_6, &phi_bb34_8);
    compiler::CodeAssemblerLabel label43(&ca_);
    CheckNextIs1Byte_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{p_left.length}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb34_8}, &label43);
    ca_.Goto(&block36, phi_bb34_6, phi_bb34_8, phi_bb34_8);
    if (label43.is_used()) {
      ca_.Bind(&label43);
      ca_.Goto(&block37, phi_bb34_6, phi_bb34_8, phi_bb34_8);
    }
  }

  TNode<IntPtrT> phi_bb37_6;
  TNode<IntPtrT> phi_bb37_8;
  TNode<IntPtrT> phi_bb37_16;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_6, &phi_bb37_8, &phi_bb37_16);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb36_6;
  TNode<IntPtrT> phi_bb36_8;
  TNode<IntPtrT> phi_bb36_16;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_6, &phi_bb36_8, &phi_bb36_16);
    compiler::CodeAssemblerLabel label44(&ca_);
    CheckNextIs1Byte_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{p_right.length}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb36_8}, &label44);
    ca_.Goto(&block38, phi_bb36_6, phi_bb36_8, phi_bb36_8);
    if (label44.is_used()) {
      ca_.Bind(&label44);
      ca_.Goto(&block39, phi_bb36_6, phi_bb36_8, phi_bb36_8);
    }
  }

  TNode<IntPtrT> phi_bb39_6;
  TNode<IntPtrT> phi_bb39_8;
  TNode<IntPtrT> phi_bb39_16;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_6, &phi_bb39_8, &phi_bb39_16);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb38_6;
  TNode<IntPtrT> phi_bb38_8;
  TNode<IntPtrT> phi_bb38_16;
  TNode<BoolT> tmp45;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_6, &phi_bb38_8, &phi_bb38_16);
    tmp45 = CodeStubAssembler(state_).Uint32LessThan(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp38});
    ca_.Branch(tmp45, &block40, std::vector<compiler::Node*>{phi_bb38_6, phi_bb38_8}, &block41, std::vector<compiler::Node*>{phi_bb38_6, phi_bb38_8});
  }

  TNode<IntPtrT> phi_bb40_6;
  TNode<IntPtrT> phi_bb40_8;
  TNode<Number> tmp46;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_6, &phi_bb40_8);
    tmp46 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp46);
  }

  TNode<IntPtrT> phi_bb41_6;
  TNode<IntPtrT> phi_bb41_8;
  TNode<Number> tmp47;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_6, &phi_bb41_8);
    tmp47 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp47);
  }

  TNode<IntPtrT> phi_bb35_6;
  TNode<IntPtrT> phi_bb35_8;
  TNode<IntPtrT> tmp48;
  TNode<IntPtrT> tmp49;
  if (block35.is_used()) {
    ca_.Bind(&block35, &phi_bb35_6, &phi_bb35_8);
    tmp48 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp49 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb35_8}, TNode<IntPtrT>{tmp48});
    ca_.Goto(&block23, phi_bb35_6, tmp49);
  }

  TNode<IntPtrT> phi_bb22_6;
  TNode<IntPtrT> phi_bb22_8;
  TNode<BoolT> tmp50;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_6, &phi_bb22_8);
    tmp50 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb22_8}, TNode<IntPtrT>{p_left.length});
    ca_.Branch(tmp50, &block42, std::vector<compiler::Node*>{phi_bb22_6, phi_bb22_8}, &block43, std::vector<compiler::Node*>{phi_bb22_6, phi_bb22_8});
  }

  TNode<IntPtrT> phi_bb42_6;
  TNode<IntPtrT> phi_bb42_8;
  TNode<IntPtrT> tmp51;
  TNode<IntPtrT> tmp52;
  TNode<Union<HeapObject, TaggedIndex>> tmp53;
  TNode<IntPtrT> tmp54;
  TNode<Uint8T> tmp55;
  TNode<Uint8T> tmp56;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_6, &phi_bb42_8);
    tmp51 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb42_8});
    tmp52 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp51});
    std::tie(tmp53, tmp54) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp52}).Flatten();
    tmp55 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp53, tmp54});
    compiler::CodeAssemblerLabel label57(&ca_);
    tmp56 = IntlAsciiCollationWeightL1_0(state_, TNode<Uint8T>{tmp55}, &label57);
    ca_.Goto(&block46, phi_bb42_6, phi_bb42_8);
    if (label57.is_used()) {
      ca_.Bind(&label57);
      ca_.Goto(&block47, phi_bb42_6, phi_bb42_8);
    }
  }

  TNode<IntPtrT> phi_bb47_6;
  TNode<IntPtrT> phi_bb47_8;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_6, &phi_bb47_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb46_6;
  TNode<IntPtrT> phi_bb46_8;
  TNode<Uint32T> tmp58;
  TNode<BoolT> tmp59;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_6, &phi_bb46_8);
    tmp58 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp59 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp56}, TNode<Uint32T>{tmp58});
    ca_.Branch(tmp59, &block48, std::vector<compiler::Node*>{phi_bb46_6, phi_bb46_8}, &block49, std::vector<compiler::Node*>{phi_bb46_6, phi_bb46_8});
  }

  TNode<IntPtrT> phi_bb48_6;
  TNode<IntPtrT> phi_bb48_8;
  if (block48.is_used()) {
    ca_.Bind(&block48, &phi_bb48_6, &phi_bb48_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb49_6;
  TNode<IntPtrT> phi_bb49_8;
  TNode<Number> tmp60;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_6, &phi_bb49_8);
    tmp60 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp60);
  }

  TNode<IntPtrT> phi_bb43_6;
  TNode<IntPtrT> phi_bb43_8;
  TNode<BoolT> tmp61;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_6, &phi_bb43_8);
    tmp61 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb43_8}, TNode<IntPtrT>{p_right.length});
    ca_.Branch(tmp61, &block50, std::vector<compiler::Node*>{phi_bb43_6, phi_bb43_8}, &block51, std::vector<compiler::Node*>{phi_bb43_6, phi_bb43_8});
  }

  TNode<IntPtrT> phi_bb50_6;
  TNode<IntPtrT> phi_bb50_8;
  TNode<IntPtrT> tmp62;
  TNode<IntPtrT> tmp63;
  TNode<Union<HeapObject, TaggedIndex>> tmp64;
  TNode<IntPtrT> tmp65;
  TNode<Uint8T> tmp66;
  TNode<Uint8T> tmp67;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_6, &phi_bb50_8);
    tmp62 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb50_8});
    tmp63 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp62});
    std::tie(tmp64, tmp65) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp63}).Flatten();
    tmp66 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp64, tmp65});
    compiler::CodeAssemblerLabel label68(&ca_);
    tmp67 = IntlAsciiCollationWeightL1_0(state_, TNode<Uint8T>{tmp66}, &label68);
    ca_.Goto(&block53, phi_bb50_6, phi_bb50_8);
    if (label68.is_used()) {
      ca_.Bind(&label68);
      ca_.Goto(&block54, phi_bb50_6, phi_bb50_8);
    }
  }

  TNode<IntPtrT> phi_bb54_6;
  TNode<IntPtrT> phi_bb54_8;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_6, &phi_bb54_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb53_6;
  TNode<IntPtrT> phi_bb53_8;
  TNode<Uint32T> tmp69;
  TNode<BoolT> tmp70;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_6, &phi_bb53_8);
    tmp69 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp70 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp67}, TNode<Uint32T>{tmp69});
    ca_.Branch(tmp70, &block55, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_8}, &block56, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_8});
  }

  TNode<IntPtrT> phi_bb55_6;
  TNode<IntPtrT> phi_bb55_8;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_6, &phi_bb55_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb56_6;
  TNode<IntPtrT> phi_bb56_8;
  TNode<Number> tmp71;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_6, &phi_bb56_8);
    tmp71 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp71);
  }

  TNode<IntPtrT> phi_bb51_6;
  TNode<IntPtrT> phi_bb51_8;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_6, &phi_bb51_8);
    ca_.Goto(&block59, phi_bb51_6, phi_bb51_6);
  }

  TNode<IntPtrT> phi_bb59_6;
  TNode<IntPtrT> phi_bb59_8;
  TNode<BoolT> tmp72;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_6, &phi_bb59_8);
    tmp72 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb59_8}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp72, &block57, std::vector<compiler::Node*>{phi_bb59_6, phi_bb59_8}, &block58, std::vector<compiler::Node*>{phi_bb59_6, phi_bb59_8});
  }

  TNode<IntPtrT> phi_bb57_6;
  TNode<IntPtrT> phi_bb57_8;
  TNode<IntPtrT> tmp73;
  TNode<IntPtrT> tmp74;
  TNode<Union<HeapObject, TaggedIndex>> tmp75;
  TNode<IntPtrT> tmp76;
  TNode<Uint8T> tmp77;
  TNode<IntPtrT> tmp78;
  TNode<IntPtrT> tmp79;
  TNode<Union<HeapObject, TaggedIndex>> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<Uint8T> tmp82;
  TNode<Uint8T> tmp83;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_6, &phi_bb57_8);
    tmp73 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb57_8});
    tmp74 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp73});
    std::tie(tmp75, tmp76) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp74}).Flatten();
    tmp77 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp75, tmp76});
    tmp78 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb57_8});
    tmp79 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp78});
    std::tie(tmp80, tmp81) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp79}).Flatten();
    tmp82 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp80, tmp81});
    compiler::CodeAssemblerLabel label84(&ca_);
    tmp83 = IntlAsciiCollationWeightL3_0(state_, TNode<Uint8T>{tmp77}, &label84);
    ca_.Goto(&block64, phi_bb57_6, phi_bb57_8);
    if (label84.is_used()) {
      ca_.Bind(&label84);
      ca_.Goto(&block65, phi_bb57_6, phi_bb57_8);
    }
  }

  TNode<IntPtrT> phi_bb65_6;
  TNode<IntPtrT> phi_bb65_8;
  if (block65.is_used()) {
    ca_.Bind(&block65, &phi_bb65_6, &phi_bb65_8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb64_6;
  TNode<IntPtrT> phi_bb64_8;
  TNode<Uint8T> tmp85;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_6, &phi_bb64_8);
    compiler::CodeAssemblerLabel label86(&ca_);
    tmp85 = IntlAsciiCollationWeightL3_0(state_, TNode<Uint8T>{tmp82}, &label86);
    ca_.Goto(&block68, phi_bb64_6, phi_bb64_8);
    if (label86.is_used()) {
      ca_.Bind(&label86);
      ca_.Goto(&block69, phi_bb64_6, phi_bb64_8);
    }
  }

  TNode<IntPtrT> phi_bb69_6;
  TNode<IntPtrT> phi_bb69_8;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_6, &phi_bb69_8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb68_6;
  TNode<IntPtrT> phi_bb68_8;
  TNode<BoolT> tmp87;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_6, &phi_bb68_8);
    tmp87 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp83}, TNode<Uint32T>{tmp85});
    ca_.Branch(tmp87, &block77, std::vector<compiler::Node*>{phi_bb68_6, phi_bb68_8}, &block78, std::vector<compiler::Node*>{phi_bb68_6, phi_bb68_8});
  }

  TNode<IntPtrT> phi_bb77_6;
  TNode<IntPtrT> phi_bb77_8;
  TNode<BoolT> tmp88;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_6, &phi_bb77_8);
    tmp88 = CodeStubAssembler(state_).Uint32LessThan(TNode<Uint32T>{tmp83}, TNode<Uint32T>{tmp85});
    ca_.Branch(tmp88, &block79, std::vector<compiler::Node*>{phi_bb77_6, phi_bb77_8}, &block80, std::vector<compiler::Node*>{phi_bb77_6, phi_bb77_8});
  }

  TNode<IntPtrT> phi_bb79_6;
  TNode<IntPtrT> phi_bb79_8;
  TNode<Number> tmp89;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_6, &phi_bb79_8);
    tmp89 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp89);
  }

  TNode<IntPtrT> phi_bb80_6;
  TNode<IntPtrT> phi_bb80_8;
  TNode<Number> tmp90;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_6, &phi_bb80_8);
    tmp90 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp90);
  }

  TNode<IntPtrT> phi_bb78_6;
  TNode<IntPtrT> phi_bb78_8;
  TNode<IntPtrT> tmp91;
  TNode<IntPtrT> tmp92;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_6, &phi_bb78_8);
    tmp91 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp92 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb78_8}, TNode<IntPtrT>{tmp91});
    ca_.Goto(&block59, phi_bb78_6, tmp92);
  }

  TNode<IntPtrT> phi_bb58_6;
  TNode<IntPtrT> phi_bb58_8;
  TNode<Number> tmp93;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_6, &phi_bb58_8);
    tmp93 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2, tmp93);
  }

  TNode<Number> phi_bb2_6;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_6);
    ca_.Goto(&block81, phi_bb2_6);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(label_Bailout);
  }

  TNode<Number> phi_bb81_6;
    ca_.Bind(&block81, &phi_bb81_6);
  return TNode<Number>{phi_bb81_6};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=164&c=14
TNode<Number> LocaleCompareFastPath_char8_char16_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_char8_ConstReference_char8_0 p_left, TorqueStructSlice_char16_ConstReference_char16_0 p_right, compiler::CodeAssemblerLabel* label_Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BoolT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block42(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block78(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block81(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{p_left.length}, TNode<IntPtrT>{p_right.length});
    ca_.Branch(tmp1, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block5, p_left.length);
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    ca_.Goto(&block5, p_right.length);
  }

  TNode<IntPtrT> phi_bb5_7;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_7);
    ca_.Goto(&block9, tmp0);
  }

  TNode<IntPtrT> phi_bb9_6;
  TNode<BoolT> tmp2;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_6);
    tmp2 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb9_6}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp2, &block7, std::vector<compiler::Node*>{phi_bb9_6}, &block8, std::vector<compiler::Node*>{phi_bb9_6});
  }

  TNode<IntPtrT> phi_bb7_6;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Union<HeapObject, TaggedIndex>> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<Uint8T> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<Union<HeapObject, TaggedIndex>> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<Uint16T> tmp12;
  TNode<BoolT> tmp13;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_6);
    tmp3 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb7_6});
    tmp4 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp3});
    std::tie(tmp5, tmp6) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp4}).Flatten();
    tmp7 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp5, tmp6});
    tmp8 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb7_6});
    tmp9 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp8});
    std::tie(tmp10, tmp11) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp9}).Flatten();
    tmp12 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp10, tmp11});
    tmp13 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp7}, TNode<Uint32T>{tmp12});
    ca_.Branch(tmp13, &block12, std::vector<compiler::Node*>{phi_bb7_6}, &block13, std::vector<compiler::Node*>{phi_bb7_6});
  }

  TNode<IntPtrT> phi_bb12_6;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_6);
    ca_.Goto(&block8, phi_bb12_6);
  }

  TNode<IntPtrT> phi_bb13_6;
  TNode<Uint32T> tmp14;
  TNode<Uint32T> tmp15;
  TNode<BoolT> tmp16;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_6);
    tmp14 = Convert_uint32_char8_0(state_, TNode<Uint8T>{tmp7});
    tmp15 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x80ull));
    tmp16 = CodeStubAssembler(state_).Uint32GreaterThanOrEqual(TNode<Uint32T>{tmp14}, TNode<Uint32T>{tmp15});
    ca_.Branch(tmp16, &block14, std::vector<compiler::Node*>{phi_bb13_6}, &block15, std::vector<compiler::Node*>{phi_bb13_6});
  }

  TNode<IntPtrT> phi_bb14_6;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_6);
    ca_.Goto(&block8, phi_bb14_6);
  }

  TNode<IntPtrT> phi_bb15_6;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_6);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb15_6}, TNode<IntPtrT>{tmp17});
    ca_.Goto(&block9, tmp18);
  }

  TNode<IntPtrT> phi_bb8_6;
  TNode<BoolT> tmp19;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_6);
    tmp19 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb8_6}, TNode<IntPtrT>{p_left.length});
    ca_.Branch(tmp19, &block18, std::vector<compiler::Node*>{phi_bb8_6}, &block19, std::vector<compiler::Node*>{phi_bb8_6});
  }

  TNode<IntPtrT> phi_bb18_6;
  TNode<BoolT> tmp20;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_6);
    tmp20 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb18_6}, TNode<IntPtrT>{p_right.length});
    ca_.Goto(&block20, phi_bb18_6, tmp20);
  }

  TNode<IntPtrT> phi_bb19_6;
  TNode<BoolT> tmp21;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_6);
    tmp21 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block20, phi_bb19_6, tmp21);
  }

  TNode<IntPtrT> phi_bb20_6;
  TNode<BoolT> phi_bb20_9;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_6, &phi_bb20_9);
    ca_.Branch(phi_bb20_9, &block16, std::vector<compiler::Node*>{phi_bb20_6}, &block17, std::vector<compiler::Node*>{phi_bb20_6});
  }

  TNode<IntPtrT> phi_bb16_6;
  TNode<Number> tmp22;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_6);
    tmp22 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2, tmp22);
  }

  TNode<IntPtrT> phi_bb17_6;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_6);
    ca_.Goto(&block23, phi_bb17_6, phi_bb17_6);
  }

  TNode<IntPtrT> phi_bb23_6;
  TNode<IntPtrT> phi_bb23_8;
  TNode<BoolT> tmp23;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_6, &phi_bb23_8);
    tmp23 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb23_8}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp23, &block21, std::vector<compiler::Node*>{phi_bb23_6, phi_bb23_8}, &block22, std::vector<compiler::Node*>{phi_bb23_6, phi_bb23_8});
  }

  TNode<IntPtrT> phi_bb21_6;
  TNode<IntPtrT> phi_bb21_8;
  TNode<IntPtrT> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<Union<HeapObject, TaggedIndex>> tmp26;
  TNode<IntPtrT> tmp27;
  TNode<Uint8T> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<Union<HeapObject, TaggedIndex>> tmp31;
  TNode<IntPtrT> tmp32;
  TNode<Uint16T> tmp33;
  TNode<Uint8T> tmp34;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_6, &phi_bb21_8);
    tmp24 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb21_8});
    tmp25 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp24});
    std::tie(tmp26, tmp27) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp25}).Flatten();
    tmp28 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp26, tmp27});
    tmp29 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb21_8});
    tmp30 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp29});
    std::tie(tmp31, tmp32) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp30}).Flatten();
    tmp33 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp31, tmp32});
    compiler::CodeAssemblerLabel label35(&ca_);
    tmp34 = IntlAsciiCollationWeightL1_0(state_, TNode<Uint8T>{tmp28}, &label35);
    ca_.Goto(&block26, phi_bb21_6, phi_bb21_8);
    if (label35.is_used()) {
      ca_.Bind(&label35);
      ca_.Goto(&block27, phi_bb21_6, phi_bb21_8);
    }
  }

  TNode<IntPtrT> phi_bb27_6;
  TNode<IntPtrT> phi_bb27_8;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_6, &phi_bb27_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb26_6;
  TNode<IntPtrT> phi_bb26_8;
  TNode<Uint32T> tmp36;
  TNode<BoolT> tmp37;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_6, &phi_bb26_8);
    tmp36 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp37 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp36});
    ca_.Branch(tmp37, &block28, std::vector<compiler::Node*>{phi_bb26_6, phi_bb26_8}, &block29, std::vector<compiler::Node*>{phi_bb26_6, phi_bb26_8});
  }

  TNode<IntPtrT> phi_bb28_6;
  TNode<IntPtrT> phi_bb28_8;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_6, &phi_bb28_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb29_6;
  TNode<IntPtrT> phi_bb29_8;
  TNode<Uint8T> tmp38;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_6, &phi_bb29_8);
    compiler::CodeAssemblerLabel label39(&ca_);
    tmp38 = IntlAsciiCollationWeightL1_1(state_, TNode<Uint16T>{tmp33}, &label39);
    ca_.Goto(&block30, phi_bb29_6, phi_bb29_8);
    if (label39.is_used()) {
      ca_.Bind(&label39);
      ca_.Goto(&block31, phi_bb29_6, phi_bb29_8);
    }
  }

  TNode<IntPtrT> phi_bb31_6;
  TNode<IntPtrT> phi_bb31_8;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_6, &phi_bb31_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb30_6;
  TNode<IntPtrT> phi_bb30_8;
  TNode<Uint32T> tmp40;
  TNode<BoolT> tmp41;
  if (block30.is_used()) {
    ca_.Bind(&block30, &phi_bb30_6, &phi_bb30_8);
    tmp40 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp41 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp38}, TNode<Uint32T>{tmp40});
    ca_.Branch(tmp41, &block32, std::vector<compiler::Node*>{phi_bb30_6, phi_bb30_8}, &block33, std::vector<compiler::Node*>{phi_bb30_6, phi_bb30_8});
  }

  TNode<IntPtrT> phi_bb32_6;
  TNode<IntPtrT> phi_bb32_8;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_6, &phi_bb32_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb33_6;
  TNode<IntPtrT> phi_bb33_8;
  TNode<BoolT> tmp42;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_6, &phi_bb33_8);
    tmp42 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp38});
    ca_.Branch(tmp42, &block34, std::vector<compiler::Node*>{phi_bb33_6, phi_bb33_8}, &block35, std::vector<compiler::Node*>{phi_bb33_6, phi_bb33_8});
  }

  TNode<IntPtrT> phi_bb34_6;
  TNode<IntPtrT> phi_bb34_8;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_6, &phi_bb34_8);
    compiler::CodeAssemblerLabel label43(&ca_);
    CheckNextIs1Byte_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{p_left.length}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb34_8}, &label43);
    ca_.Goto(&block36, phi_bb34_6, phi_bb34_8, phi_bb34_8);
    if (label43.is_used()) {
      ca_.Bind(&label43);
      ca_.Goto(&block37, phi_bb34_6, phi_bb34_8, phi_bb34_8);
    }
  }

  TNode<IntPtrT> phi_bb37_6;
  TNode<IntPtrT> phi_bb37_8;
  TNode<IntPtrT> phi_bb37_16;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_6, &phi_bb37_8, &phi_bb37_16);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb36_6;
  TNode<IntPtrT> phi_bb36_8;
  TNode<IntPtrT> phi_bb36_16;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_6, &phi_bb36_8, &phi_bb36_16);
    compiler::CodeAssemblerLabel label44(&ca_);
    CheckNextIs1Byte_1(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{p_right.length}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb36_8}, &label44);
    ca_.Goto(&block38, phi_bb36_6, phi_bb36_8, phi_bb36_8);
    if (label44.is_used()) {
      ca_.Bind(&label44);
      ca_.Goto(&block39, phi_bb36_6, phi_bb36_8, phi_bb36_8);
    }
  }

  TNode<IntPtrT> phi_bb39_6;
  TNode<IntPtrT> phi_bb39_8;
  TNode<IntPtrT> phi_bb39_16;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_6, &phi_bb39_8, &phi_bb39_16);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb38_6;
  TNode<IntPtrT> phi_bb38_8;
  TNode<IntPtrT> phi_bb38_16;
  TNode<BoolT> tmp45;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_6, &phi_bb38_8, &phi_bb38_16);
    tmp45 = CodeStubAssembler(state_).Uint32LessThan(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp38});
    ca_.Branch(tmp45, &block40, std::vector<compiler::Node*>{phi_bb38_6, phi_bb38_8}, &block41, std::vector<compiler::Node*>{phi_bb38_6, phi_bb38_8});
  }

  TNode<IntPtrT> phi_bb40_6;
  TNode<IntPtrT> phi_bb40_8;
  TNode<Number> tmp46;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_6, &phi_bb40_8);
    tmp46 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp46);
  }

  TNode<IntPtrT> phi_bb41_6;
  TNode<IntPtrT> phi_bb41_8;
  TNode<Number> tmp47;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_6, &phi_bb41_8);
    tmp47 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp47);
  }

  TNode<IntPtrT> phi_bb35_6;
  TNode<IntPtrT> phi_bb35_8;
  TNode<IntPtrT> tmp48;
  TNode<IntPtrT> tmp49;
  if (block35.is_used()) {
    ca_.Bind(&block35, &phi_bb35_6, &phi_bb35_8);
    tmp48 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp49 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb35_8}, TNode<IntPtrT>{tmp48});
    ca_.Goto(&block23, phi_bb35_6, tmp49);
  }

  TNode<IntPtrT> phi_bb22_6;
  TNode<IntPtrT> phi_bb22_8;
  TNode<BoolT> tmp50;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_6, &phi_bb22_8);
    tmp50 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb22_8}, TNode<IntPtrT>{p_left.length});
    ca_.Branch(tmp50, &block42, std::vector<compiler::Node*>{phi_bb22_6, phi_bb22_8}, &block43, std::vector<compiler::Node*>{phi_bb22_6, phi_bb22_8});
  }

  TNode<IntPtrT> phi_bb42_6;
  TNode<IntPtrT> phi_bb42_8;
  TNode<IntPtrT> tmp51;
  TNode<IntPtrT> tmp52;
  TNode<Union<HeapObject, TaggedIndex>> tmp53;
  TNode<IntPtrT> tmp54;
  TNode<Uint8T> tmp55;
  TNode<Uint8T> tmp56;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_6, &phi_bb42_8);
    tmp51 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb42_8});
    tmp52 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp51});
    std::tie(tmp53, tmp54) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp52}).Flatten();
    tmp55 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp53, tmp54});
    compiler::CodeAssemblerLabel label57(&ca_);
    tmp56 = IntlAsciiCollationWeightL1_0(state_, TNode<Uint8T>{tmp55}, &label57);
    ca_.Goto(&block46, phi_bb42_6, phi_bb42_8);
    if (label57.is_used()) {
      ca_.Bind(&label57);
      ca_.Goto(&block47, phi_bb42_6, phi_bb42_8);
    }
  }

  TNode<IntPtrT> phi_bb47_6;
  TNode<IntPtrT> phi_bb47_8;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_6, &phi_bb47_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb46_6;
  TNode<IntPtrT> phi_bb46_8;
  TNode<Uint32T> tmp58;
  TNode<BoolT> tmp59;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_6, &phi_bb46_8);
    tmp58 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp59 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp56}, TNode<Uint32T>{tmp58});
    ca_.Branch(tmp59, &block48, std::vector<compiler::Node*>{phi_bb46_6, phi_bb46_8}, &block49, std::vector<compiler::Node*>{phi_bb46_6, phi_bb46_8});
  }

  TNode<IntPtrT> phi_bb48_6;
  TNode<IntPtrT> phi_bb48_8;
  if (block48.is_used()) {
    ca_.Bind(&block48, &phi_bb48_6, &phi_bb48_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb49_6;
  TNode<IntPtrT> phi_bb49_8;
  TNode<Number> tmp60;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_6, &phi_bb49_8);
    tmp60 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp60);
  }

  TNode<IntPtrT> phi_bb43_6;
  TNode<IntPtrT> phi_bb43_8;
  TNode<BoolT> tmp61;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_6, &phi_bb43_8);
    tmp61 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb43_8}, TNode<IntPtrT>{p_right.length});
    ca_.Branch(tmp61, &block50, std::vector<compiler::Node*>{phi_bb43_6, phi_bb43_8}, &block51, std::vector<compiler::Node*>{phi_bb43_6, phi_bb43_8});
  }

  TNode<IntPtrT> phi_bb50_6;
  TNode<IntPtrT> phi_bb50_8;
  TNode<IntPtrT> tmp62;
  TNode<IntPtrT> tmp63;
  TNode<Union<HeapObject, TaggedIndex>> tmp64;
  TNode<IntPtrT> tmp65;
  TNode<Uint16T> tmp66;
  TNode<Uint8T> tmp67;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_6, &phi_bb50_8);
    tmp62 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb50_8});
    tmp63 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp62});
    std::tie(tmp64, tmp65) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp63}).Flatten();
    tmp66 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp64, tmp65});
    compiler::CodeAssemblerLabel label68(&ca_);
    tmp67 = IntlAsciiCollationWeightL1_1(state_, TNode<Uint16T>{tmp66}, &label68);
    ca_.Goto(&block53, phi_bb50_6, phi_bb50_8);
    if (label68.is_used()) {
      ca_.Bind(&label68);
      ca_.Goto(&block54, phi_bb50_6, phi_bb50_8);
    }
  }

  TNode<IntPtrT> phi_bb54_6;
  TNode<IntPtrT> phi_bb54_8;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_6, &phi_bb54_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb53_6;
  TNode<IntPtrT> phi_bb53_8;
  TNode<Uint32T> tmp69;
  TNode<BoolT> tmp70;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_6, &phi_bb53_8);
    tmp69 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp70 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp67}, TNode<Uint32T>{tmp69});
    ca_.Branch(tmp70, &block55, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_8}, &block56, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_8});
  }

  TNode<IntPtrT> phi_bb55_6;
  TNode<IntPtrT> phi_bb55_8;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_6, &phi_bb55_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb56_6;
  TNode<IntPtrT> phi_bb56_8;
  TNode<Number> tmp71;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_6, &phi_bb56_8);
    tmp71 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp71);
  }

  TNode<IntPtrT> phi_bb51_6;
  TNode<IntPtrT> phi_bb51_8;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_6, &phi_bb51_8);
    ca_.Goto(&block59, phi_bb51_6, phi_bb51_6);
  }

  TNode<IntPtrT> phi_bb59_6;
  TNode<IntPtrT> phi_bb59_8;
  TNode<BoolT> tmp72;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_6, &phi_bb59_8);
    tmp72 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb59_8}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp72, &block57, std::vector<compiler::Node*>{phi_bb59_6, phi_bb59_8}, &block58, std::vector<compiler::Node*>{phi_bb59_6, phi_bb59_8});
  }

  TNode<IntPtrT> phi_bb57_6;
  TNode<IntPtrT> phi_bb57_8;
  TNode<IntPtrT> tmp73;
  TNode<IntPtrT> tmp74;
  TNode<Union<HeapObject, TaggedIndex>> tmp75;
  TNode<IntPtrT> tmp76;
  TNode<Uint8T> tmp77;
  TNode<IntPtrT> tmp78;
  TNode<IntPtrT> tmp79;
  TNode<Union<HeapObject, TaggedIndex>> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<Uint16T> tmp82;
  TNode<Uint8T> tmp83;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_6, &phi_bb57_8);
    tmp73 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb57_8});
    tmp74 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp73});
    std::tie(tmp75, tmp76) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp74}).Flatten();
    tmp77 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp75, tmp76});
    tmp78 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb57_8});
    tmp79 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp78});
    std::tie(tmp80, tmp81) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp79}).Flatten();
    tmp82 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp80, tmp81});
    compiler::CodeAssemblerLabel label84(&ca_);
    tmp83 = IntlAsciiCollationWeightL3_0(state_, TNode<Uint8T>{tmp77}, &label84);
    ca_.Goto(&block64, phi_bb57_6, phi_bb57_8);
    if (label84.is_used()) {
      ca_.Bind(&label84);
      ca_.Goto(&block65, phi_bb57_6, phi_bb57_8);
    }
  }

  TNode<IntPtrT> phi_bb65_6;
  TNode<IntPtrT> phi_bb65_8;
  if (block65.is_used()) {
    ca_.Bind(&block65, &phi_bb65_6, &phi_bb65_8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb64_6;
  TNode<IntPtrT> phi_bb64_8;
  TNode<Uint8T> tmp85;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_6, &phi_bb64_8);
    compiler::CodeAssemblerLabel label86(&ca_);
    tmp85 = IntlAsciiCollationWeightL3_1(state_, TNode<Uint16T>{tmp82}, &label86);
    ca_.Goto(&block68, phi_bb64_6, phi_bb64_8);
    if (label86.is_used()) {
      ca_.Bind(&label86);
      ca_.Goto(&block69, phi_bb64_6, phi_bb64_8);
    }
  }

  TNode<IntPtrT> phi_bb69_6;
  TNode<IntPtrT> phi_bb69_8;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_6, &phi_bb69_8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb68_6;
  TNode<IntPtrT> phi_bb68_8;
  TNode<BoolT> tmp87;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_6, &phi_bb68_8);
    tmp87 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp83}, TNode<Uint32T>{tmp85});
    ca_.Branch(tmp87, &block77, std::vector<compiler::Node*>{phi_bb68_6, phi_bb68_8}, &block78, std::vector<compiler::Node*>{phi_bb68_6, phi_bb68_8});
  }

  TNode<IntPtrT> phi_bb77_6;
  TNode<IntPtrT> phi_bb77_8;
  TNode<BoolT> tmp88;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_6, &phi_bb77_8);
    tmp88 = CodeStubAssembler(state_).Uint32LessThan(TNode<Uint32T>{tmp83}, TNode<Uint32T>{tmp85});
    ca_.Branch(tmp88, &block79, std::vector<compiler::Node*>{phi_bb77_6, phi_bb77_8}, &block80, std::vector<compiler::Node*>{phi_bb77_6, phi_bb77_8});
  }

  TNode<IntPtrT> phi_bb79_6;
  TNode<IntPtrT> phi_bb79_8;
  TNode<Number> tmp89;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_6, &phi_bb79_8);
    tmp89 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp89);
  }

  TNode<IntPtrT> phi_bb80_6;
  TNode<IntPtrT> phi_bb80_8;
  TNode<Number> tmp90;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_6, &phi_bb80_8);
    tmp90 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp90);
  }

  TNode<IntPtrT> phi_bb78_6;
  TNode<IntPtrT> phi_bb78_8;
  TNode<IntPtrT> tmp91;
  TNode<IntPtrT> tmp92;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_6, &phi_bb78_8);
    tmp91 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp92 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb78_8}, TNode<IntPtrT>{tmp91});
    ca_.Goto(&block59, phi_bb78_6, tmp92);
  }

  TNode<IntPtrT> phi_bb58_6;
  TNode<IntPtrT> phi_bb58_8;
  TNode<Number> tmp93;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_6, &phi_bb58_8);
    tmp93 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2, tmp93);
  }

  TNode<Number> phi_bb2_6;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_6);
    ca_.Goto(&block81, phi_bb2_6);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(label_Bailout);
  }

  TNode<Number> phi_bb81_6;
    ca_.Bind(&block81, &phi_bb81_6);
  return TNode<Number>{phi_bb81_6};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=171&c=14
TNode<Number> LocaleCompareFastPath_char16_char8_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_char16_ConstReference_char16_0 p_left, TorqueStructSlice_char8_ConstReference_char8_0 p_right, compiler::CodeAssemblerLabel* label_Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BoolT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block42(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block78(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block81(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{p_left.length}, TNode<IntPtrT>{p_right.length});
    ca_.Branch(tmp1, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block5, p_left.length);
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    ca_.Goto(&block5, p_right.length);
  }

  TNode<IntPtrT> phi_bb5_7;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_7);
    ca_.Goto(&block9, tmp0);
  }

  TNode<IntPtrT> phi_bb9_6;
  TNode<BoolT> tmp2;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_6);
    tmp2 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb9_6}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp2, &block7, std::vector<compiler::Node*>{phi_bb9_6}, &block8, std::vector<compiler::Node*>{phi_bb9_6});
  }

  TNode<IntPtrT> phi_bb7_6;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Union<HeapObject, TaggedIndex>> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<Uint16T> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<Union<HeapObject, TaggedIndex>> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<Uint8T> tmp12;
  TNode<BoolT> tmp13;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_6);
    tmp3 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb7_6});
    tmp4 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp3});
    std::tie(tmp5, tmp6) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp4}).Flatten();
    tmp7 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp5, tmp6});
    tmp8 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb7_6});
    tmp9 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp8});
    std::tie(tmp10, tmp11) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp9}).Flatten();
    tmp12 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp10, tmp11});
    tmp13 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp7}, TNode<Uint32T>{tmp12});
    ca_.Branch(tmp13, &block12, std::vector<compiler::Node*>{phi_bb7_6}, &block13, std::vector<compiler::Node*>{phi_bb7_6});
  }

  TNode<IntPtrT> phi_bb12_6;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_6);
    ca_.Goto(&block8, phi_bb12_6);
  }

  TNode<IntPtrT> phi_bb13_6;
  TNode<Uint32T> tmp14;
  TNode<Uint32T> tmp15;
  TNode<BoolT> tmp16;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_6);
    tmp14 = Convert_uint32_char16_0(state_, TNode<Uint16T>{tmp7});
    tmp15 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x80ull));
    tmp16 = CodeStubAssembler(state_).Uint32GreaterThanOrEqual(TNode<Uint32T>{tmp14}, TNode<Uint32T>{tmp15});
    ca_.Branch(tmp16, &block14, std::vector<compiler::Node*>{phi_bb13_6}, &block15, std::vector<compiler::Node*>{phi_bb13_6});
  }

  TNode<IntPtrT> phi_bb14_6;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_6);
    ca_.Goto(&block8, phi_bb14_6);
  }

  TNode<IntPtrT> phi_bb15_6;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_6);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb15_6}, TNode<IntPtrT>{tmp17});
    ca_.Goto(&block9, tmp18);
  }

  TNode<IntPtrT> phi_bb8_6;
  TNode<BoolT> tmp19;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_6);
    tmp19 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb8_6}, TNode<IntPtrT>{p_left.length});
    ca_.Branch(tmp19, &block18, std::vector<compiler::Node*>{phi_bb8_6}, &block19, std::vector<compiler::Node*>{phi_bb8_6});
  }

  TNode<IntPtrT> phi_bb18_6;
  TNode<BoolT> tmp20;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_6);
    tmp20 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb18_6}, TNode<IntPtrT>{p_right.length});
    ca_.Goto(&block20, phi_bb18_6, tmp20);
  }

  TNode<IntPtrT> phi_bb19_6;
  TNode<BoolT> tmp21;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_6);
    tmp21 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block20, phi_bb19_6, tmp21);
  }

  TNode<IntPtrT> phi_bb20_6;
  TNode<BoolT> phi_bb20_9;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_6, &phi_bb20_9);
    ca_.Branch(phi_bb20_9, &block16, std::vector<compiler::Node*>{phi_bb20_6}, &block17, std::vector<compiler::Node*>{phi_bb20_6});
  }

  TNode<IntPtrT> phi_bb16_6;
  TNode<Number> tmp22;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_6);
    tmp22 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2, tmp22);
  }

  TNode<IntPtrT> phi_bb17_6;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_6);
    ca_.Goto(&block23, phi_bb17_6, phi_bb17_6);
  }

  TNode<IntPtrT> phi_bb23_6;
  TNode<IntPtrT> phi_bb23_8;
  TNode<BoolT> tmp23;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_6, &phi_bb23_8);
    tmp23 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb23_8}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp23, &block21, std::vector<compiler::Node*>{phi_bb23_6, phi_bb23_8}, &block22, std::vector<compiler::Node*>{phi_bb23_6, phi_bb23_8});
  }

  TNode<IntPtrT> phi_bb21_6;
  TNode<IntPtrT> phi_bb21_8;
  TNode<IntPtrT> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<Union<HeapObject, TaggedIndex>> tmp26;
  TNode<IntPtrT> tmp27;
  TNode<Uint16T> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<Union<HeapObject, TaggedIndex>> tmp31;
  TNode<IntPtrT> tmp32;
  TNode<Uint8T> tmp33;
  TNode<Uint8T> tmp34;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_6, &phi_bb21_8);
    tmp24 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb21_8});
    tmp25 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp24});
    std::tie(tmp26, tmp27) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp25}).Flatten();
    tmp28 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp26, tmp27});
    tmp29 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb21_8});
    tmp30 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp29});
    std::tie(tmp31, tmp32) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp30}).Flatten();
    tmp33 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp31, tmp32});
    compiler::CodeAssemblerLabel label35(&ca_);
    tmp34 = IntlAsciiCollationWeightL1_1(state_, TNode<Uint16T>{tmp28}, &label35);
    ca_.Goto(&block26, phi_bb21_6, phi_bb21_8);
    if (label35.is_used()) {
      ca_.Bind(&label35);
      ca_.Goto(&block27, phi_bb21_6, phi_bb21_8);
    }
  }

  TNode<IntPtrT> phi_bb27_6;
  TNode<IntPtrT> phi_bb27_8;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_6, &phi_bb27_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb26_6;
  TNode<IntPtrT> phi_bb26_8;
  TNode<Uint32T> tmp36;
  TNode<BoolT> tmp37;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_6, &phi_bb26_8);
    tmp36 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp37 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp36});
    ca_.Branch(tmp37, &block28, std::vector<compiler::Node*>{phi_bb26_6, phi_bb26_8}, &block29, std::vector<compiler::Node*>{phi_bb26_6, phi_bb26_8});
  }

  TNode<IntPtrT> phi_bb28_6;
  TNode<IntPtrT> phi_bb28_8;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_6, &phi_bb28_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb29_6;
  TNode<IntPtrT> phi_bb29_8;
  TNode<Uint8T> tmp38;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_6, &phi_bb29_8);
    compiler::CodeAssemblerLabel label39(&ca_);
    tmp38 = IntlAsciiCollationWeightL1_0(state_, TNode<Uint8T>{tmp33}, &label39);
    ca_.Goto(&block30, phi_bb29_6, phi_bb29_8);
    if (label39.is_used()) {
      ca_.Bind(&label39);
      ca_.Goto(&block31, phi_bb29_6, phi_bb29_8);
    }
  }

  TNode<IntPtrT> phi_bb31_6;
  TNode<IntPtrT> phi_bb31_8;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_6, &phi_bb31_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb30_6;
  TNode<IntPtrT> phi_bb30_8;
  TNode<Uint32T> tmp40;
  TNode<BoolT> tmp41;
  if (block30.is_used()) {
    ca_.Bind(&block30, &phi_bb30_6, &phi_bb30_8);
    tmp40 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp41 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp38}, TNode<Uint32T>{tmp40});
    ca_.Branch(tmp41, &block32, std::vector<compiler::Node*>{phi_bb30_6, phi_bb30_8}, &block33, std::vector<compiler::Node*>{phi_bb30_6, phi_bb30_8});
  }

  TNode<IntPtrT> phi_bb32_6;
  TNode<IntPtrT> phi_bb32_8;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_6, &phi_bb32_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb33_6;
  TNode<IntPtrT> phi_bb33_8;
  TNode<BoolT> tmp42;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_6, &phi_bb33_8);
    tmp42 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp38});
    ca_.Branch(tmp42, &block34, std::vector<compiler::Node*>{phi_bb33_6, phi_bb33_8}, &block35, std::vector<compiler::Node*>{phi_bb33_6, phi_bb33_8});
  }

  TNode<IntPtrT> phi_bb34_6;
  TNode<IntPtrT> phi_bb34_8;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_6, &phi_bb34_8);
    compiler::CodeAssemblerLabel label43(&ca_);
    CheckNextIs1Byte_1(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{p_left.length}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb34_8}, &label43);
    ca_.Goto(&block36, phi_bb34_6, phi_bb34_8, phi_bb34_8);
    if (label43.is_used()) {
      ca_.Bind(&label43);
      ca_.Goto(&block37, phi_bb34_6, phi_bb34_8, phi_bb34_8);
    }
  }

  TNode<IntPtrT> phi_bb37_6;
  TNode<IntPtrT> phi_bb37_8;
  TNode<IntPtrT> phi_bb37_16;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_6, &phi_bb37_8, &phi_bb37_16);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb36_6;
  TNode<IntPtrT> phi_bb36_8;
  TNode<IntPtrT> phi_bb36_16;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_6, &phi_bb36_8, &phi_bb36_16);
    compiler::CodeAssemblerLabel label44(&ca_);
    CheckNextIs1Byte_0(state_, TorqueStructSlice_char8_ConstReference_char8_0{TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{p_right.length}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb36_8}, &label44);
    ca_.Goto(&block38, phi_bb36_6, phi_bb36_8, phi_bb36_8);
    if (label44.is_used()) {
      ca_.Bind(&label44);
      ca_.Goto(&block39, phi_bb36_6, phi_bb36_8, phi_bb36_8);
    }
  }

  TNode<IntPtrT> phi_bb39_6;
  TNode<IntPtrT> phi_bb39_8;
  TNode<IntPtrT> phi_bb39_16;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_6, &phi_bb39_8, &phi_bb39_16);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb38_6;
  TNode<IntPtrT> phi_bb38_8;
  TNode<IntPtrT> phi_bb38_16;
  TNode<BoolT> tmp45;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_6, &phi_bb38_8, &phi_bb38_16);
    tmp45 = CodeStubAssembler(state_).Uint32LessThan(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp38});
    ca_.Branch(tmp45, &block40, std::vector<compiler::Node*>{phi_bb38_6, phi_bb38_8}, &block41, std::vector<compiler::Node*>{phi_bb38_6, phi_bb38_8});
  }

  TNode<IntPtrT> phi_bb40_6;
  TNode<IntPtrT> phi_bb40_8;
  TNode<Number> tmp46;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_6, &phi_bb40_8);
    tmp46 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp46);
  }

  TNode<IntPtrT> phi_bb41_6;
  TNode<IntPtrT> phi_bb41_8;
  TNode<Number> tmp47;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_6, &phi_bb41_8);
    tmp47 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp47);
  }

  TNode<IntPtrT> phi_bb35_6;
  TNode<IntPtrT> phi_bb35_8;
  TNode<IntPtrT> tmp48;
  TNode<IntPtrT> tmp49;
  if (block35.is_used()) {
    ca_.Bind(&block35, &phi_bb35_6, &phi_bb35_8);
    tmp48 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp49 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb35_8}, TNode<IntPtrT>{tmp48});
    ca_.Goto(&block23, phi_bb35_6, tmp49);
  }

  TNode<IntPtrT> phi_bb22_6;
  TNode<IntPtrT> phi_bb22_8;
  TNode<BoolT> tmp50;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_6, &phi_bb22_8);
    tmp50 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb22_8}, TNode<IntPtrT>{p_left.length});
    ca_.Branch(tmp50, &block42, std::vector<compiler::Node*>{phi_bb22_6, phi_bb22_8}, &block43, std::vector<compiler::Node*>{phi_bb22_6, phi_bb22_8});
  }

  TNode<IntPtrT> phi_bb42_6;
  TNode<IntPtrT> phi_bb42_8;
  TNode<IntPtrT> tmp51;
  TNode<IntPtrT> tmp52;
  TNode<Union<HeapObject, TaggedIndex>> tmp53;
  TNode<IntPtrT> tmp54;
  TNode<Uint16T> tmp55;
  TNode<Uint8T> tmp56;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_6, &phi_bb42_8);
    tmp51 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb42_8});
    tmp52 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp51});
    std::tie(tmp53, tmp54) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp52}).Flatten();
    tmp55 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp53, tmp54});
    compiler::CodeAssemblerLabel label57(&ca_);
    tmp56 = IntlAsciiCollationWeightL1_1(state_, TNode<Uint16T>{tmp55}, &label57);
    ca_.Goto(&block46, phi_bb42_6, phi_bb42_8);
    if (label57.is_used()) {
      ca_.Bind(&label57);
      ca_.Goto(&block47, phi_bb42_6, phi_bb42_8);
    }
  }

  TNode<IntPtrT> phi_bb47_6;
  TNode<IntPtrT> phi_bb47_8;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_6, &phi_bb47_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb46_6;
  TNode<IntPtrT> phi_bb46_8;
  TNode<Uint32T> tmp58;
  TNode<BoolT> tmp59;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_6, &phi_bb46_8);
    tmp58 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp59 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp56}, TNode<Uint32T>{tmp58});
    ca_.Branch(tmp59, &block48, std::vector<compiler::Node*>{phi_bb46_6, phi_bb46_8}, &block49, std::vector<compiler::Node*>{phi_bb46_6, phi_bb46_8});
  }

  TNode<IntPtrT> phi_bb48_6;
  TNode<IntPtrT> phi_bb48_8;
  if (block48.is_used()) {
    ca_.Bind(&block48, &phi_bb48_6, &phi_bb48_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb49_6;
  TNode<IntPtrT> phi_bb49_8;
  TNode<Number> tmp60;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_6, &phi_bb49_8);
    tmp60 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp60);
  }

  TNode<IntPtrT> phi_bb43_6;
  TNode<IntPtrT> phi_bb43_8;
  TNode<BoolT> tmp61;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_6, &phi_bb43_8);
    tmp61 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb43_8}, TNode<IntPtrT>{p_right.length});
    ca_.Branch(tmp61, &block50, std::vector<compiler::Node*>{phi_bb43_6, phi_bb43_8}, &block51, std::vector<compiler::Node*>{phi_bb43_6, phi_bb43_8});
  }

  TNode<IntPtrT> phi_bb50_6;
  TNode<IntPtrT> phi_bb50_8;
  TNode<IntPtrT> tmp62;
  TNode<IntPtrT> tmp63;
  TNode<Union<HeapObject, TaggedIndex>> tmp64;
  TNode<IntPtrT> tmp65;
  TNode<Uint8T> tmp66;
  TNode<Uint8T> tmp67;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_6, &phi_bb50_8);
    tmp62 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb50_8});
    tmp63 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp62});
    std::tie(tmp64, tmp65) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp63}).Flatten();
    tmp66 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp64, tmp65});
    compiler::CodeAssemblerLabel label68(&ca_);
    tmp67 = IntlAsciiCollationWeightL1_0(state_, TNode<Uint8T>{tmp66}, &label68);
    ca_.Goto(&block53, phi_bb50_6, phi_bb50_8);
    if (label68.is_used()) {
      ca_.Bind(&label68);
      ca_.Goto(&block54, phi_bb50_6, phi_bb50_8);
    }
  }

  TNode<IntPtrT> phi_bb54_6;
  TNode<IntPtrT> phi_bb54_8;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_6, &phi_bb54_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb53_6;
  TNode<IntPtrT> phi_bb53_8;
  TNode<Uint32T> tmp69;
  TNode<BoolT> tmp70;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_6, &phi_bb53_8);
    tmp69 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp70 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp67}, TNode<Uint32T>{tmp69});
    ca_.Branch(tmp70, &block55, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_8}, &block56, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_8});
  }

  TNode<IntPtrT> phi_bb55_6;
  TNode<IntPtrT> phi_bb55_8;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_6, &phi_bb55_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb56_6;
  TNode<IntPtrT> phi_bb56_8;
  TNode<Number> tmp71;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_6, &phi_bb56_8);
    tmp71 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp71);
  }

  TNode<IntPtrT> phi_bb51_6;
  TNode<IntPtrT> phi_bb51_8;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_6, &phi_bb51_8);
    ca_.Goto(&block59, phi_bb51_6, phi_bb51_6);
  }

  TNode<IntPtrT> phi_bb59_6;
  TNode<IntPtrT> phi_bb59_8;
  TNode<BoolT> tmp72;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_6, &phi_bb59_8);
    tmp72 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb59_8}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp72, &block57, std::vector<compiler::Node*>{phi_bb59_6, phi_bb59_8}, &block58, std::vector<compiler::Node*>{phi_bb59_6, phi_bb59_8});
  }

  TNode<IntPtrT> phi_bb57_6;
  TNode<IntPtrT> phi_bb57_8;
  TNode<IntPtrT> tmp73;
  TNode<IntPtrT> tmp74;
  TNode<Union<HeapObject, TaggedIndex>> tmp75;
  TNode<IntPtrT> tmp76;
  TNode<Uint16T> tmp77;
  TNode<IntPtrT> tmp78;
  TNode<IntPtrT> tmp79;
  TNode<Union<HeapObject, TaggedIndex>> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<Uint8T> tmp82;
  TNode<Uint8T> tmp83;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_6, &phi_bb57_8);
    tmp73 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb57_8});
    tmp74 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp73});
    std::tie(tmp75, tmp76) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp74}).Flatten();
    tmp77 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp75, tmp76});
    tmp78 = TimesSizeOf_char8_0(state_, TNode<IntPtrT>{phi_bb57_8});
    tmp79 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp78});
    std::tie(tmp80, tmp81) = NewReference_char8_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp79}).Flatten();
    tmp82 = CodeStubAssembler(state_).LoadReference<Uint8T>(CodeStubAssembler::Reference{tmp80, tmp81});
    compiler::CodeAssemblerLabel label84(&ca_);
    tmp83 = IntlAsciiCollationWeightL3_1(state_, TNode<Uint16T>{tmp77}, &label84);
    ca_.Goto(&block64, phi_bb57_6, phi_bb57_8);
    if (label84.is_used()) {
      ca_.Bind(&label84);
      ca_.Goto(&block65, phi_bb57_6, phi_bb57_8);
    }
  }

  TNode<IntPtrT> phi_bb65_6;
  TNode<IntPtrT> phi_bb65_8;
  if (block65.is_used()) {
    ca_.Bind(&block65, &phi_bb65_6, &phi_bb65_8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb64_6;
  TNode<IntPtrT> phi_bb64_8;
  TNode<Uint8T> tmp85;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_6, &phi_bb64_8);
    compiler::CodeAssemblerLabel label86(&ca_);
    tmp85 = IntlAsciiCollationWeightL3_0(state_, TNode<Uint8T>{tmp82}, &label86);
    ca_.Goto(&block68, phi_bb64_6, phi_bb64_8);
    if (label86.is_used()) {
      ca_.Bind(&label86);
      ca_.Goto(&block69, phi_bb64_6, phi_bb64_8);
    }
  }

  TNode<IntPtrT> phi_bb69_6;
  TNode<IntPtrT> phi_bb69_8;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_6, &phi_bb69_8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb68_6;
  TNode<IntPtrT> phi_bb68_8;
  TNode<BoolT> tmp87;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_6, &phi_bb68_8);
    tmp87 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp83}, TNode<Uint32T>{tmp85});
    ca_.Branch(tmp87, &block77, std::vector<compiler::Node*>{phi_bb68_6, phi_bb68_8}, &block78, std::vector<compiler::Node*>{phi_bb68_6, phi_bb68_8});
  }

  TNode<IntPtrT> phi_bb77_6;
  TNode<IntPtrT> phi_bb77_8;
  TNode<BoolT> tmp88;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_6, &phi_bb77_8);
    tmp88 = CodeStubAssembler(state_).Uint32LessThan(TNode<Uint32T>{tmp83}, TNode<Uint32T>{tmp85});
    ca_.Branch(tmp88, &block79, std::vector<compiler::Node*>{phi_bb77_6, phi_bb77_8}, &block80, std::vector<compiler::Node*>{phi_bb77_6, phi_bb77_8});
  }

  TNode<IntPtrT> phi_bb79_6;
  TNode<IntPtrT> phi_bb79_8;
  TNode<Number> tmp89;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_6, &phi_bb79_8);
    tmp89 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp89);
  }

  TNode<IntPtrT> phi_bb80_6;
  TNode<IntPtrT> phi_bb80_8;
  TNode<Number> tmp90;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_6, &phi_bb80_8);
    tmp90 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp90);
  }

  TNode<IntPtrT> phi_bb78_6;
  TNode<IntPtrT> phi_bb78_8;
  TNode<IntPtrT> tmp91;
  TNode<IntPtrT> tmp92;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_6, &phi_bb78_8);
    tmp91 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp92 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb78_8}, TNode<IntPtrT>{tmp91});
    ca_.Goto(&block59, phi_bb78_6, tmp92);
  }

  TNode<IntPtrT> phi_bb58_6;
  TNode<IntPtrT> phi_bb58_8;
  TNode<Number> tmp93;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_6, &phi_bb58_8);
    tmp93 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2, tmp93);
  }

  TNode<Number> phi_bb2_6;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_6);
    ca_.Goto(&block81, phi_bb2_6);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(label_Bailout);
  }

  TNode<Number> phi_bb81_6;
    ca_.Bind(&block81, &phi_bb81_6);
  return TNode<Number>{phi_bb81_6};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=173&c=14
TNode<Number> LocaleCompareFastPath_char16_char16_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_char16_ConstReference_char16_0 p_left, TorqueStructSlice_char16_ConstReference_char16_0 p_right, compiler::CodeAssemblerLabel* label_Bailout) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BoolT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block42(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block78(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block81(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{p_left.length}, TNode<IntPtrT>{p_right.length});
    ca_.Branch(tmp1, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block5, p_left.length);
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    ca_.Goto(&block5, p_right.length);
  }

  TNode<IntPtrT> phi_bb5_7;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_7);
    ca_.Goto(&block9, tmp0);
  }

  TNode<IntPtrT> phi_bb9_6;
  TNode<BoolT> tmp2;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_6);
    tmp2 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb9_6}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp2, &block7, std::vector<compiler::Node*>{phi_bb9_6}, &block8, std::vector<compiler::Node*>{phi_bb9_6});
  }

  TNode<IntPtrT> phi_bb7_6;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Union<HeapObject, TaggedIndex>> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<Uint16T> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<Union<HeapObject, TaggedIndex>> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<Uint16T> tmp12;
  TNode<BoolT> tmp13;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_6);
    tmp3 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb7_6});
    tmp4 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp3});
    std::tie(tmp5, tmp6) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp4}).Flatten();
    tmp7 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp5, tmp6});
    tmp8 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb7_6});
    tmp9 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp8});
    std::tie(tmp10, tmp11) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp9}).Flatten();
    tmp12 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp10, tmp11});
    tmp13 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp7}, TNode<Uint32T>{tmp12});
    ca_.Branch(tmp13, &block12, std::vector<compiler::Node*>{phi_bb7_6}, &block13, std::vector<compiler::Node*>{phi_bb7_6});
  }

  TNode<IntPtrT> phi_bb12_6;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_6);
    ca_.Goto(&block8, phi_bb12_6);
  }

  TNode<IntPtrT> phi_bb13_6;
  TNode<Uint32T> tmp14;
  TNode<Uint32T> tmp15;
  TNode<BoolT> tmp16;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_6);
    tmp14 = Convert_uint32_char16_0(state_, TNode<Uint16T>{tmp7});
    tmp15 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x80ull));
    tmp16 = CodeStubAssembler(state_).Uint32GreaterThanOrEqual(TNode<Uint32T>{tmp14}, TNode<Uint32T>{tmp15});
    ca_.Branch(tmp16, &block14, std::vector<compiler::Node*>{phi_bb13_6}, &block15, std::vector<compiler::Node*>{phi_bb13_6});
  }

  TNode<IntPtrT> phi_bb14_6;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_6);
    ca_.Goto(&block8, phi_bb14_6);
  }

  TNode<IntPtrT> phi_bb15_6;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_6);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb15_6}, TNode<IntPtrT>{tmp17});
    ca_.Goto(&block9, tmp18);
  }

  TNode<IntPtrT> phi_bb8_6;
  TNode<BoolT> tmp19;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_6);
    tmp19 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb8_6}, TNode<IntPtrT>{p_left.length});
    ca_.Branch(tmp19, &block18, std::vector<compiler::Node*>{phi_bb8_6}, &block19, std::vector<compiler::Node*>{phi_bb8_6});
  }

  TNode<IntPtrT> phi_bb18_6;
  TNode<BoolT> tmp20;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_6);
    tmp20 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb18_6}, TNode<IntPtrT>{p_right.length});
    ca_.Goto(&block20, phi_bb18_6, tmp20);
  }

  TNode<IntPtrT> phi_bb19_6;
  TNode<BoolT> tmp21;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_6);
    tmp21 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block20, phi_bb19_6, tmp21);
  }

  TNode<IntPtrT> phi_bb20_6;
  TNode<BoolT> phi_bb20_9;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_6, &phi_bb20_9);
    ca_.Branch(phi_bb20_9, &block16, std::vector<compiler::Node*>{phi_bb20_6}, &block17, std::vector<compiler::Node*>{phi_bb20_6});
  }

  TNode<IntPtrT> phi_bb16_6;
  TNode<Number> tmp22;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_6);
    tmp22 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2, tmp22);
  }

  TNode<IntPtrT> phi_bb17_6;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_6);
    ca_.Goto(&block23, phi_bb17_6, phi_bb17_6);
  }

  TNode<IntPtrT> phi_bb23_6;
  TNode<IntPtrT> phi_bb23_8;
  TNode<BoolT> tmp23;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_6, &phi_bb23_8);
    tmp23 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb23_8}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp23, &block21, std::vector<compiler::Node*>{phi_bb23_6, phi_bb23_8}, &block22, std::vector<compiler::Node*>{phi_bb23_6, phi_bb23_8});
  }

  TNode<IntPtrT> phi_bb21_6;
  TNode<IntPtrT> phi_bb21_8;
  TNode<IntPtrT> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<Union<HeapObject, TaggedIndex>> tmp26;
  TNode<IntPtrT> tmp27;
  TNode<Uint16T> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<Union<HeapObject, TaggedIndex>> tmp31;
  TNode<IntPtrT> tmp32;
  TNode<Uint16T> tmp33;
  TNode<Uint8T> tmp34;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_6, &phi_bb21_8);
    tmp24 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb21_8});
    tmp25 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp24});
    std::tie(tmp26, tmp27) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp25}).Flatten();
    tmp28 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp26, tmp27});
    tmp29 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb21_8});
    tmp30 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp29});
    std::tie(tmp31, tmp32) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp30}).Flatten();
    tmp33 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp31, tmp32});
    compiler::CodeAssemblerLabel label35(&ca_);
    tmp34 = IntlAsciiCollationWeightL1_1(state_, TNode<Uint16T>{tmp28}, &label35);
    ca_.Goto(&block26, phi_bb21_6, phi_bb21_8);
    if (label35.is_used()) {
      ca_.Bind(&label35);
      ca_.Goto(&block27, phi_bb21_6, phi_bb21_8);
    }
  }

  TNode<IntPtrT> phi_bb27_6;
  TNode<IntPtrT> phi_bb27_8;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_6, &phi_bb27_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb26_6;
  TNode<IntPtrT> phi_bb26_8;
  TNode<Uint32T> tmp36;
  TNode<BoolT> tmp37;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_6, &phi_bb26_8);
    tmp36 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp37 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp36});
    ca_.Branch(tmp37, &block28, std::vector<compiler::Node*>{phi_bb26_6, phi_bb26_8}, &block29, std::vector<compiler::Node*>{phi_bb26_6, phi_bb26_8});
  }

  TNode<IntPtrT> phi_bb28_6;
  TNode<IntPtrT> phi_bb28_8;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_6, &phi_bb28_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb29_6;
  TNode<IntPtrT> phi_bb29_8;
  TNode<Uint8T> tmp38;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_6, &phi_bb29_8);
    compiler::CodeAssemblerLabel label39(&ca_);
    tmp38 = IntlAsciiCollationWeightL1_1(state_, TNode<Uint16T>{tmp33}, &label39);
    ca_.Goto(&block30, phi_bb29_6, phi_bb29_8);
    if (label39.is_used()) {
      ca_.Bind(&label39);
      ca_.Goto(&block31, phi_bb29_6, phi_bb29_8);
    }
  }

  TNode<IntPtrT> phi_bb31_6;
  TNode<IntPtrT> phi_bb31_8;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_6, &phi_bb31_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb30_6;
  TNode<IntPtrT> phi_bb30_8;
  TNode<Uint32T> tmp40;
  TNode<BoolT> tmp41;
  if (block30.is_used()) {
    ca_.Bind(&block30, &phi_bb30_6, &phi_bb30_8);
    tmp40 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp41 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp38}, TNode<Uint32T>{tmp40});
    ca_.Branch(tmp41, &block32, std::vector<compiler::Node*>{phi_bb30_6, phi_bb30_8}, &block33, std::vector<compiler::Node*>{phi_bb30_6, phi_bb30_8});
  }

  TNode<IntPtrT> phi_bb32_6;
  TNode<IntPtrT> phi_bb32_8;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_6, &phi_bb32_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb33_6;
  TNode<IntPtrT> phi_bb33_8;
  TNode<BoolT> tmp42;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_6, &phi_bb33_8);
    tmp42 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp38});
    ca_.Branch(tmp42, &block34, std::vector<compiler::Node*>{phi_bb33_6, phi_bb33_8}, &block35, std::vector<compiler::Node*>{phi_bb33_6, phi_bb33_8});
  }

  TNode<IntPtrT> phi_bb34_6;
  TNode<IntPtrT> phi_bb34_8;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_6, &phi_bb34_8);
    compiler::CodeAssemblerLabel label43(&ca_);
    CheckNextIs1Byte_1(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{p_left.length}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb34_8}, &label43);
    ca_.Goto(&block36, phi_bb34_6, phi_bb34_8, phi_bb34_8);
    if (label43.is_used()) {
      ca_.Bind(&label43);
      ca_.Goto(&block37, phi_bb34_6, phi_bb34_8, phi_bb34_8);
    }
  }

  TNode<IntPtrT> phi_bb37_6;
  TNode<IntPtrT> phi_bb37_8;
  TNode<IntPtrT> phi_bb37_16;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_6, &phi_bb37_8, &phi_bb37_16);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb36_6;
  TNode<IntPtrT> phi_bb36_8;
  TNode<IntPtrT> phi_bb36_16;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_6, &phi_bb36_8, &phi_bb36_16);
    compiler::CodeAssemblerLabel label44(&ca_);
    CheckNextIs1Byte_1(state_, TorqueStructSlice_char16_ConstReference_char16_0{TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{p_right.length}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{phi_bb36_8}, &label44);
    ca_.Goto(&block38, phi_bb36_6, phi_bb36_8, phi_bb36_8);
    if (label44.is_used()) {
      ca_.Bind(&label44);
      ca_.Goto(&block39, phi_bb36_6, phi_bb36_8, phi_bb36_8);
    }
  }

  TNode<IntPtrT> phi_bb39_6;
  TNode<IntPtrT> phi_bb39_8;
  TNode<IntPtrT> phi_bb39_16;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_6, &phi_bb39_8, &phi_bb39_16);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb38_6;
  TNode<IntPtrT> phi_bb38_8;
  TNode<IntPtrT> phi_bb38_16;
  TNode<BoolT> tmp45;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_6, &phi_bb38_8, &phi_bb38_16);
    tmp45 = CodeStubAssembler(state_).Uint32LessThan(TNode<Uint32T>{tmp34}, TNode<Uint32T>{tmp38});
    ca_.Branch(tmp45, &block40, std::vector<compiler::Node*>{phi_bb38_6, phi_bb38_8}, &block41, std::vector<compiler::Node*>{phi_bb38_6, phi_bb38_8});
  }

  TNode<IntPtrT> phi_bb40_6;
  TNode<IntPtrT> phi_bb40_8;
  TNode<Number> tmp46;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_6, &phi_bb40_8);
    tmp46 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp46);
  }

  TNode<IntPtrT> phi_bb41_6;
  TNode<IntPtrT> phi_bb41_8;
  TNode<Number> tmp47;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_6, &phi_bb41_8);
    tmp47 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp47);
  }

  TNode<IntPtrT> phi_bb35_6;
  TNode<IntPtrT> phi_bb35_8;
  TNode<IntPtrT> tmp48;
  TNode<IntPtrT> tmp49;
  if (block35.is_used()) {
    ca_.Bind(&block35, &phi_bb35_6, &phi_bb35_8);
    tmp48 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp49 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb35_8}, TNode<IntPtrT>{tmp48});
    ca_.Goto(&block23, phi_bb35_6, tmp49);
  }

  TNode<IntPtrT> phi_bb22_6;
  TNode<IntPtrT> phi_bb22_8;
  TNode<BoolT> tmp50;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_6, &phi_bb22_8);
    tmp50 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb22_8}, TNode<IntPtrT>{p_left.length});
    ca_.Branch(tmp50, &block42, std::vector<compiler::Node*>{phi_bb22_6, phi_bb22_8}, &block43, std::vector<compiler::Node*>{phi_bb22_6, phi_bb22_8});
  }

  TNode<IntPtrT> phi_bb42_6;
  TNode<IntPtrT> phi_bb42_8;
  TNode<IntPtrT> tmp51;
  TNode<IntPtrT> tmp52;
  TNode<Union<HeapObject, TaggedIndex>> tmp53;
  TNode<IntPtrT> tmp54;
  TNode<Uint16T> tmp55;
  TNode<Uint8T> tmp56;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_6, &phi_bb42_8);
    tmp51 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb42_8});
    tmp52 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp51});
    std::tie(tmp53, tmp54) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp52}).Flatten();
    tmp55 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp53, tmp54});
    compiler::CodeAssemblerLabel label57(&ca_);
    tmp56 = IntlAsciiCollationWeightL1_1(state_, TNode<Uint16T>{tmp55}, &label57);
    ca_.Goto(&block46, phi_bb42_6, phi_bb42_8);
    if (label57.is_used()) {
      ca_.Bind(&label57);
      ca_.Goto(&block47, phi_bb42_6, phi_bb42_8);
    }
  }

  TNode<IntPtrT> phi_bb47_6;
  TNode<IntPtrT> phi_bb47_8;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_6, &phi_bb47_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb46_6;
  TNode<IntPtrT> phi_bb46_8;
  TNode<Uint32T> tmp58;
  TNode<BoolT> tmp59;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_6, &phi_bb46_8);
    tmp58 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp59 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp56}, TNode<Uint32T>{tmp58});
    ca_.Branch(tmp59, &block48, std::vector<compiler::Node*>{phi_bb46_6, phi_bb46_8}, &block49, std::vector<compiler::Node*>{phi_bb46_6, phi_bb46_8});
  }

  TNode<IntPtrT> phi_bb48_6;
  TNode<IntPtrT> phi_bb48_8;
  if (block48.is_used()) {
    ca_.Bind(&block48, &phi_bb48_6, &phi_bb48_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb49_6;
  TNode<IntPtrT> phi_bb49_8;
  TNode<Number> tmp60;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_6, &phi_bb49_8);
    tmp60 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp60);
  }

  TNode<IntPtrT> phi_bb43_6;
  TNode<IntPtrT> phi_bb43_8;
  TNode<BoolT> tmp61;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_6, &phi_bb43_8);
    tmp61 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb43_8}, TNode<IntPtrT>{p_right.length});
    ca_.Branch(tmp61, &block50, std::vector<compiler::Node*>{phi_bb43_6, phi_bb43_8}, &block51, std::vector<compiler::Node*>{phi_bb43_6, phi_bb43_8});
  }

  TNode<IntPtrT> phi_bb50_6;
  TNode<IntPtrT> phi_bb50_8;
  TNode<IntPtrT> tmp62;
  TNode<IntPtrT> tmp63;
  TNode<Union<HeapObject, TaggedIndex>> tmp64;
  TNode<IntPtrT> tmp65;
  TNode<Uint16T> tmp66;
  TNode<Uint8T> tmp67;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_6, &phi_bb50_8);
    tmp62 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb50_8});
    tmp63 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp62});
    std::tie(tmp64, tmp65) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp63}).Flatten();
    tmp66 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp64, tmp65});
    compiler::CodeAssemblerLabel label68(&ca_);
    tmp67 = IntlAsciiCollationWeightL1_1(state_, TNode<Uint16T>{tmp66}, &label68);
    ca_.Goto(&block53, phi_bb50_6, phi_bb50_8);
    if (label68.is_used()) {
      ca_.Bind(&label68);
      ca_.Goto(&block54, phi_bb50_6, phi_bb50_8);
    }
  }

  TNode<IntPtrT> phi_bb54_6;
  TNode<IntPtrT> phi_bb54_8;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_6, &phi_bb54_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb53_6;
  TNode<IntPtrT> phi_bb53_8;
  TNode<Uint32T> tmp69;
  TNode<BoolT> tmp70;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_6, &phi_bb53_8);
    tmp69 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp70 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp67}, TNode<Uint32T>{tmp69});
    ca_.Branch(tmp70, &block55, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_8}, &block56, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_8});
  }

  TNode<IntPtrT> phi_bb55_6;
  TNode<IntPtrT> phi_bb55_8;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_6, &phi_bb55_8);
    ca_.Goto(&block1);
  }

  TNode<IntPtrT> phi_bb56_6;
  TNode<IntPtrT> phi_bb56_8;
  TNode<Number> tmp71;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_6, &phi_bb56_8);
    tmp71 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp71);
  }

  TNode<IntPtrT> phi_bb51_6;
  TNode<IntPtrT> phi_bb51_8;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_6, &phi_bb51_8);
    ca_.Goto(&block59, phi_bb51_6, phi_bb51_6);
  }

  TNode<IntPtrT> phi_bb59_6;
  TNode<IntPtrT> phi_bb59_8;
  TNode<BoolT> tmp72;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_6, &phi_bb59_8);
    tmp72 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb59_8}, TNode<IntPtrT>{phi_bb5_7});
    ca_.Branch(tmp72, &block57, std::vector<compiler::Node*>{phi_bb59_6, phi_bb59_8}, &block58, std::vector<compiler::Node*>{phi_bb59_6, phi_bb59_8});
  }

  TNode<IntPtrT> phi_bb57_6;
  TNode<IntPtrT> phi_bb57_8;
  TNode<IntPtrT> tmp73;
  TNode<IntPtrT> tmp74;
  TNode<Union<HeapObject, TaggedIndex>> tmp75;
  TNode<IntPtrT> tmp76;
  TNode<Uint16T> tmp77;
  TNode<IntPtrT> tmp78;
  TNode<IntPtrT> tmp79;
  TNode<Union<HeapObject, TaggedIndex>> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<Uint16T> tmp82;
  TNode<Uint8T> tmp83;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_6, &phi_bb57_8);
    tmp73 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb57_8});
    tmp74 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_left.offset}, TNode<IntPtrT>{tmp73});
    std::tie(tmp75, tmp76) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_left.object}, TNode<IntPtrT>{tmp74}).Flatten();
    tmp77 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp75, tmp76});
    tmp78 = TimesSizeOf_char16_0(state_, TNode<IntPtrT>{phi_bb57_8});
    tmp79 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_right.offset}, TNode<IntPtrT>{tmp78});
    std::tie(tmp80, tmp81) = NewReference_char16_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_right.object}, TNode<IntPtrT>{tmp79}).Flatten();
    tmp82 = CodeStubAssembler(state_).LoadReference<Uint16T>(CodeStubAssembler::Reference{tmp80, tmp81});
    compiler::CodeAssemblerLabel label84(&ca_);
    tmp83 = IntlAsciiCollationWeightL3_1(state_, TNode<Uint16T>{tmp77}, &label84);
    ca_.Goto(&block64, phi_bb57_6, phi_bb57_8);
    if (label84.is_used()) {
      ca_.Bind(&label84);
      ca_.Goto(&block65, phi_bb57_6, phi_bb57_8);
    }
  }

  TNode<IntPtrT> phi_bb65_6;
  TNode<IntPtrT> phi_bb65_8;
  if (block65.is_used()) {
    ca_.Bind(&block65, &phi_bb65_6, &phi_bb65_8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb64_6;
  TNode<IntPtrT> phi_bb64_8;
  TNode<Uint8T> tmp85;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_6, &phi_bb64_8);
    compiler::CodeAssemblerLabel label86(&ca_);
    tmp85 = IntlAsciiCollationWeightL3_1(state_, TNode<Uint16T>{tmp82}, &label86);
    ca_.Goto(&block68, phi_bb64_6, phi_bb64_8);
    if (label86.is_used()) {
      ca_.Bind(&label86);
      ca_.Goto(&block69, phi_bb64_6, phi_bb64_8);
    }
  }

  TNode<IntPtrT> phi_bb69_6;
  TNode<IntPtrT> phi_bb69_8;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_6, &phi_bb69_8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb68_6;
  TNode<IntPtrT> phi_bb68_8;
  TNode<BoolT> tmp87;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_6, &phi_bb68_8);
    tmp87 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp83}, TNode<Uint32T>{tmp85});
    ca_.Branch(tmp87, &block77, std::vector<compiler::Node*>{phi_bb68_6, phi_bb68_8}, &block78, std::vector<compiler::Node*>{phi_bb68_6, phi_bb68_8});
  }

  TNode<IntPtrT> phi_bb77_6;
  TNode<IntPtrT> phi_bb77_8;
  TNode<BoolT> tmp88;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_6, &phi_bb77_8);
    tmp88 = CodeStubAssembler(state_).Uint32LessThan(TNode<Uint32T>{tmp83}, TNode<Uint32T>{tmp85});
    ca_.Branch(tmp88, &block79, std::vector<compiler::Node*>{phi_bb77_6, phi_bb77_8}, &block80, std::vector<compiler::Node*>{phi_bb77_6, phi_bb77_8});
  }

  TNode<IntPtrT> phi_bb79_6;
  TNode<IntPtrT> phi_bb79_8;
  TNode<Number> tmp89;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_6, &phi_bb79_8);
    tmp89 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    ca_.Goto(&block2, tmp89);
  }

  TNode<IntPtrT> phi_bb80_6;
  TNode<IntPtrT> phi_bb80_8;
  TNode<Number> tmp90;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_6, &phi_bb80_8);
    tmp90 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block2, tmp90);
  }

  TNode<IntPtrT> phi_bb78_6;
  TNode<IntPtrT> phi_bb78_8;
  TNode<IntPtrT> tmp91;
  TNode<IntPtrT> tmp92;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_6, &phi_bb78_8);
    tmp91 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp92 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb78_8}, TNode<IntPtrT>{tmp91});
    ca_.Goto(&block59, phi_bb78_6, tmp92);
  }

  TNode<IntPtrT> phi_bb58_6;
  TNode<IntPtrT> phi_bb58_8;
  TNode<Number> tmp93;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_6, &phi_bb58_8);
    tmp93 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block2, tmp93);
  }

  TNode<Number> phi_bb2_6;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_6);
    ca_.Goto(&block81, phi_bb2_6);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(label_Bailout);
  }

  TNode<Number> phi_bb81_6;
    ca_.Bind(&block81, &phi_bb81_6);
  return TNode<Number>{phi_bb81_6};
}

// https://crsrc.org/c/v8/src/objects/intl-objects.tq?l=63&c=9
TNode<Uint32T> Convert_uint32_char8_0(compiler::CodeAssemblerState* state_, TNode<Uint8T> p_i) {
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
  return TNode<Uint32T>{p_i};
}

} // namespace internal
} // namespace v8
