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
#include "torque-generated/src/builtins/string-replaceall-tq-csa.h"
#include "torque-generated/src/objects/js-regexp-tq-csa.h"
#include "torque-generated/src/objects/string-tq-csa.h"
#include "torque-generated/src/builtins/array-find-tq-csa.h"
#include "torque-generated/src/builtins/set-is-superset-of-tq-csa.h"
#include "torque-generated/src/builtins/typed-array-from-tq-csa.h"
#include "torque-generated/src/builtins/convert-tq-csa.h"
#include "torque-generated/src/builtins/string-replaceall-tq-csa.h"
#include "torque-generated/src/builtins/cast-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"
#include "torque-generated/src/builtins/builtins-string-tq-csa.h"
#include "torque-generated/src/builtins/regexp-tq-csa.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/string-replaceall.tq?l=12&c=1
void ThrowIfNotGlobal_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_searchValue) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSRegExp> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_FastJSRegExp_1(state_, TNode<Context>{p_context}, TNode<Object>{p_searchValue}, &label1);
    ca_.Goto(&block4);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block5);
    }
  }

  TNode<JSAny> tmp2;
  TNode<JSAny> tmp3;
  TNode<JSAny> tmp4;
  TNode<String> tmp5;
  TNode<String> tmp6;
  TNode<Smi> tmp7;
  TNode<Smi> tmp8;
  TNode<Smi> tmp9;
  TNode<BoolT> tmp10;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp2 = FromConstexpr_JSAny_constexpr_string_0(state_, "flags");
    tmp3 = CodeStubAssembler(state_).GetProperty(TNode<Context>{p_context}, TNode<JSAny>{p_searchValue}, TNode<JSAny>{tmp2});
    tmp4 = RequireObjectCoercible_0(state_, TNode<Context>{p_context}, TNode<JSAny>{tmp3}, "String.prototype.replaceAll");
    tmp5 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{p_context}, TNode<JSAny>{tmp3});
    tmp6 = CodeStubAssembler(state_).StringConstant("g");
    tmp7 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp8 = ca_.CallBuiltin<Smi>(Builtin::kStringIndexOf, TNode<Object>(), tmp5, tmp6, tmp7);
    tmp9 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    tmp10 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp8}, TNode<Smi>{tmp9});
    ca_.Goto(&block2, tmp10);
  }

  TNode<BoolT> tmp11;
  TNode<BoolT> tmp12;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp11 = RegExpBuiltinsAssembler(state_).FastFlagGetterGlobal(TNode<JSRegExp>{tmp0});
    tmp12 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp11});
    ca_.Goto(&block2, tmp12);
  }

  TNode<BoolT> phi_bb2_2;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_2);
    ca_.Branch(phi_bb2_2, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{p_context}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kRegExpGlobalInvokedOnNonGlobal), "String.prototype.replaceAll");
  }

  if (block7.is_used()) {
    ca_.Bind(&block7);
    ca_.Goto(&block8);
  }

    ca_.Bind(&block8);
}

TF_BUILTIN(StringPrototypeReplaceAll, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kSearchValue);
  USE(parameter2);
  TNode<JSAny> parameter3 = UncheckedParameter<JSAny>(Descriptor::kReplaceValue);
  USE(parameter3);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kStringReplaceAll));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Smi, String, Smi> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Smi, String, Smi> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Smi, String, Smi> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Smi, String, Smi> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Smi, String, Smi, String> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Smi, String, Smi> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Smi, String, Smi> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Smi, String, Smi> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = RequireObjectCoercible_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{parameter1}, "String.prototype.replaceAll");
    tmp1 = Is_JSReceiver_JSAny_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{parameter2});
    ca_.Branch(tmp1, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp2;
  if (block1.is_used()) {
    ca_.Bind(&block1);
    tmp2 = IsRegExp_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{parameter2});
    ca_.Branch(tmp2, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ThrowIfNotGlobal_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{parameter2});
    ca_.Goto(&block4);
  }

  TNode<Symbol> tmp3;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp4;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp3 = CodeStubAssembler(state_).ReplaceSymbolConstant();
    compiler::CodeAssemblerLabel label5(&ca_);
    tmp4 = GetMethod_3(state_, TNode<Context>{parameter0}, TNode<JSAny>{parameter2}, TNode<Symbol>{tmp3}, &label5);
    ca_.Goto(&block7);
    if (label5.is_used()) {
      ca_.Bind(&label5);
      ca_.Goto(&block8);
    }
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    ca_.Goto(&block2);
  }

  TNode<JSAny> tmp6;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp6 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp4}, TNode<JSAny>{parameter2}, TNode<JSAny>{parameter1}, TNode<JSAny>{parameter3});
    CodeStubAssembler(state_).Return(tmp6);
  }

  TNode<String> tmp7;
  TNode<String> tmp8;
  TNode<BoolT> tmp9;
  TNode<BoolT> tmp10;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp7 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter1});
    tmp8 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter2});
    tmp9 = Is_Callable_JSAny_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{parameter3});
    tmp10 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp9});
    ca_.Branch(tmp10, &block9, std::vector<compiler::Node*>{}, &block10, std::vector<compiler::Node*>{parameter3});
  }

  TNode<String> tmp11;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp11 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter3});
    ca_.Goto(&block10, tmp11);
  }

  TNode<JSAny> phi_bb10_6;
  TNode<Smi> tmp12;
  TNode<Smi> tmp13;
  TNode<Smi> tmp14;
  TNode<Smi> tmp15;
  TNode<String> tmp16;
  TNode<Smi> tmp17;
  TNode<Smi> tmp18;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_6);
    tmp12 = CodeStubAssembler(state_).LoadStringLengthAsSmi(TNode<String>{tmp8});
    tmp13 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp14 = CodeStubAssembler(state_).SmiMax(TNode<Smi>{tmp13}, TNode<Smi>{tmp12});
    tmp15 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp16 = kEmptyString_0(state_);
    tmp17 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp18 = AbstractStringIndexOf_4(state_, TNode<Context>{parameter0}, TNode<String>{tmp7}, TNode<String>{tmp8}, TNode<Smi>{tmp17});
    ca_.Goto(&block13, tmp15, tmp16, tmp18);
  }

  TNode<Smi> phi_bb13_10;
  TNode<String> phi_bb13_11;
  TNode<Smi> phi_bb13_12;
  TNode<Smi> tmp19;
  TNode<BoolT> tmp20;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_10, &phi_bb13_11, &phi_bb13_12);
    tmp19 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    tmp20 = CodeStubAssembler(state_).SmiNotEqual(TNode<Smi>{phi_bb13_12}, TNode<Smi>{tmp19});
    ca_.Branch(tmp20, &block11, std::vector<compiler::Node*>{phi_bb13_10, phi_bb13_11, phi_bb13_12}, &block12, std::vector<compiler::Node*>{phi_bb13_10, phi_bb13_11, phi_bb13_12});
  }

  TNode<Smi> phi_bb11_10;
  TNode<String> phi_bb11_11;
  TNode<Smi> phi_bb11_12;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_10, &phi_bb11_11, &phi_bb11_12);
    ca_.Branch(tmp9, &block14, std::vector<compiler::Node*>{phi_bb11_10, phi_bb11_11, phi_bb11_12}, &block15, std::vector<compiler::Node*>{phi_bb11_10, phi_bb11_11, phi_bb11_12});
  }

  TNode<Smi> phi_bb14_10;
  TNode<String> phi_bb14_11;
  TNode<Smi> phi_bb14_12;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp21;
  TNode<Undefined> tmp22;
  TNode<JSAny> tmp23;
  TNode<String> tmp24;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_10, &phi_bb14_11, &phi_bb14_12);
    tmp21 = UnsafeCast_Callable_0(state_, TNode<Context>{parameter0}, TNode<Object>{phi_bb10_6});
    tmp22 = Undefined_0(state_);
    tmp23 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp21}, TNode<JSAny>{tmp22}, TNode<JSAny>{tmp8}, TNode<JSAny>{phi_bb14_12}, TNode<JSAny>{tmp7});
    tmp24 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{tmp23});
    ca_.Goto(&block16, phi_bb14_10, phi_bb14_11, phi_bb14_12, tmp24);
  }

  TNode<Smi> phi_bb15_10;
  TNode<String> phi_bb15_11;
  TNode<Smi> phi_bb15_12;
  TNode<String> tmp25;
  TNode<Smi> tmp26;
  TNode<String> tmp27;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_10, &phi_bb15_11, &phi_bb15_12);
    tmp25 = UnsafeCast_String_0(state_, TNode<Context>{parameter0}, TNode<Object>{phi_bb10_6});
    tmp26 = CodeStubAssembler(state_).SmiAdd(TNode<Smi>{phi_bb15_12}, TNode<Smi>{tmp12});
    tmp27 = StringBuiltinsAssembler(state_).GetSubstitution(TNode<Context>{parameter0}, TNode<String>{tmp7}, TNode<Smi>{phi_bb15_12}, TNode<Smi>{tmp26}, TNode<String>{tmp25});
    ca_.Goto(&block16, phi_bb15_10, phi_bb15_11, phi_bb15_12, tmp27);
  }

  TNode<Smi> phi_bb16_10;
  TNode<String> phi_bb16_11;
  TNode<Smi> phi_bb16_12;
  TNode<String> phi_bb16_13;
  TNode<IntPtrT> tmp28;
  TNode<UintPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<UintPtrT> tmp31;
  TNode<String> tmp32;
  TNode<String> tmp33;
  TNode<String> tmp34;
  TNode<Smi> tmp35;
  TNode<Smi> tmp36;
  TNode<Smi> tmp37;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_10, &phi_bb16_11, &phi_bb16_12, &phi_bb16_13);
    tmp28 = CodeStubAssembler(state_).SmiUntag(TNode<Smi>{phi_bb16_10});
    tmp29 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{tmp28});
    tmp30 = CodeStubAssembler(state_).SmiUntag(TNode<Smi>{phi_bb16_12});
    tmp31 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{tmp30});
    tmp32 = StringBuiltinsAssembler(state_).SubString(TNode<String>{tmp7}, TNode<UintPtrT>{tmp29}, TNode<UintPtrT>{tmp31});
    tmp33 = StringAdd_0(state_, TNode<Context>{parameter0}, TNode<String>{phi_bb16_11}, TNode<String>{tmp32});
    tmp34 = StringAdd_0(state_, TNode<Context>{parameter0}, TNode<String>{tmp33}, TNode<String>{phi_bb16_13});
    tmp35 = CodeStubAssembler(state_).SmiAdd(TNode<Smi>{phi_bb16_12}, TNode<Smi>{tmp12});
    tmp36 = CodeStubAssembler(state_).SmiAdd(TNode<Smi>{phi_bb16_12}, TNode<Smi>{tmp14});
    tmp37 = AbstractStringIndexOf_4(state_, TNode<Context>{parameter0}, TNode<String>{tmp7}, TNode<String>{tmp8}, TNode<Smi>{tmp36});
    ca_.Goto(&block13, tmp35, tmp34, tmp37);
  }

  TNode<Smi> phi_bb12_10;
  TNode<String> phi_bb12_11;
  TNode<Smi> phi_bb12_12;
  TNode<Smi> tmp38;
  TNode<BoolT> tmp39;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_10, &phi_bb12_11, &phi_bb12_12);
    tmp38 = CodeStubAssembler(state_).LoadStringLengthAsSmi(TNode<String>{tmp7});
    tmp39 = CodeStubAssembler(state_).SmiLessThan(TNode<Smi>{phi_bb12_10}, TNode<Smi>{tmp38});
    ca_.Branch(tmp39, &block17, std::vector<compiler::Node*>{phi_bb12_10, phi_bb12_11, phi_bb12_12}, &block18, std::vector<compiler::Node*>{phi_bb12_10, phi_bb12_11, phi_bb12_12});
  }

  TNode<Smi> phi_bb17_10;
  TNode<String> phi_bb17_11;
  TNode<Smi> phi_bb17_12;
  TNode<IntPtrT> tmp40;
  TNode<UintPtrT> tmp41;
  TNode<IntPtrT> tmp42;
  TNode<UintPtrT> tmp43;
  TNode<String> tmp44;
  TNode<String> tmp45;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_10, &phi_bb17_11, &phi_bb17_12);
    tmp40 = CodeStubAssembler(state_).SmiUntag(TNode<Smi>{phi_bb17_10});
    tmp41 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{tmp40});
    tmp42 = CodeStubAssembler(state_).LoadStringLengthAsWord(TNode<String>{tmp7});
    tmp43 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{tmp42});
    tmp44 = StringBuiltinsAssembler(state_).SubString(TNode<String>{tmp7}, TNode<UintPtrT>{tmp41}, TNode<UintPtrT>{tmp43});
    tmp45 = StringAdd_0(state_, TNode<Context>{parameter0}, TNode<String>{phi_bb17_11}, TNode<String>{tmp44});
    ca_.Goto(&block18, phi_bb17_10, tmp45, phi_bb17_12);
  }

  TNode<Smi> phi_bb18_10;
  TNode<String> phi_bb18_11;
  TNode<Smi> phi_bb18_12;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_10, &phi_bb18_11, &phi_bb18_12);
    CodeStubAssembler(state_).Return(phi_bb18_11);
  }
}

// https://crsrc.org/c/v8/src/builtins/string-replaceall.tq?l=16&c=5
TNode<JSRegExp> Cast_FastJSRegExp_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError) {
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

  TNode<JSRegExp> tmp2;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_FastJSRegExp_0(state_, TNode<Context>{p_context}, TNode<HeapObject>{tmp0}, &label3);
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
  return TNode<JSRegExp>{tmp2};
}

// https://crsrc.org/c/v8/src/builtins/string-replaceall.tq?l=42&c=7
TNode<BoolT> Is_JSReceiver_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{p_context}, TNode<Object>{p_o}, &label1);
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

// https://crsrc.org/c/v8/src/builtins/string-replaceall.tq?l=115&c=34
TNode<String> UnsafeCast_String_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<String> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = TORQUE_CAST(TNode<Object>{p_o});
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<String>{tmp0};
}

} // namespace internal
} // namespace v8
