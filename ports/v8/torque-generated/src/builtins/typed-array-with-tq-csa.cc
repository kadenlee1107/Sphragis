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
#include "torque-generated/src/builtins/typed-array-with-tq-csa.h"
#include "torque-generated/src/objects/js-array-buffer-tq-csa.h"
#include "torque-generated/src/builtins/typed-array-foreach-tq-csa.h"
#include "torque-generated/src/builtins/convert-tq-csa.h"
#include "torque-generated/src/builtins/typed-array-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"
#include "torque-generated/src/builtins/typed-array-createtypedarray-tq-csa.h"
#include "torque-generated/src/builtins/typed-array-with-tq-csa.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/typed-array-with.tq?l=6&c=1
const char* kBuiltinNameWith_0(compiler::CodeAssemblerState* state_) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

    ca_.Bind(&block0);
  return "%TypedArray%.prototype.with";}

TF_BUILTIN(TypedArrayPrototypeWith, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kIndex);
  USE(parameter2);
  TNode<JSAny> parameter3 = UncheckedParameter<JSAny>(Descriptor::kValueArg);
  USE(parameter3);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block37(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<UintPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<UintPtrT> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<UintPtrT, UintPtrT, UintPtrT> block43(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<UintPtrT, UintPtrT, UintPtrT> block42(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<UintPtrT> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSTypedArray_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block7);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block8);
    }
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kNotTypedArray), kBuiltinNameWith_0(state_));
  }

  TNode<JSTypedArray> tmp2;
  TNode<UintPtrT> tmp3;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    compiler::CodeAssemblerLabel label4(&ca_);
    std::tie(tmp2, tmp3) = EnsureValidAndReadLength_0(state_, TNode<JSTypedArray>{tmp0}, CastIfEnumClass<TypedArrayAccessMode>(TypedArrayAccessMode::kRead), &label4).Flatten();
    ca_.Goto(&block9);
    if (label4.is_used()) {
      ca_.Bind(&label4);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kTypedArrayValidateErrorOperation), kBuiltinNameWith_0(state_));
  }

  TNode<Number> tmp5;
  TNode<Number> tmp6;
  TNode<BoolT> tmp7;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp5 = ToInteger_Inline_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{parameter2});
    tmp6 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp7 = NumberIsGreaterThanOrEqual_0(state_, TNode<Number>{tmp5}, TNode<Number>{tmp6});
    ca_.Branch(tmp7, &block11, std::vector<compiler::Node*>{}, &block12, std::vector<compiler::Node*>{});
  }

  if (block11.is_used()) {
    ca_.Bind(&block11);
    ca_.Goto(&block13, tmp5);
  }

  TNode<Number> tmp8;
  TNode<Number> tmp9;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp8 = Convert_Number_uintptr_0(state_, TNode<UintPtrT>{tmp3});
    tmp9 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{tmp8}, TNode<Number>{tmp5});
    ca_.Goto(&block13, tmp9);
  }

  TNode<Number> phi_bb13_9;
  TNode<Int32T> tmp10;
  TNode<BoolT> tmp11;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_9);
    tmp10 = CodeStubAssembler(state_).LoadElementsKind(TNode<JSTypedArray>{tmp0});
    tmp11 = TypedArrayBuiltinsAssembler(state_).IsBigInt64ElementsKind(TNode<Int32T>{tmp10});
    ca_.Branch(tmp11, &block15, std::vector<compiler::Node*>{}, &block16, std::vector<compiler::Node*>{});
  }

  TNode<BigInt> tmp12;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp12 = CodeStubAssembler(state_).ToBigInt(TNode<Context>{parameter0}, TNode<JSAny>{parameter3});
    ca_.Goto(&block17, tmp12);
  }

  TNode<Number> tmp13;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp13 = CodeStubAssembler(state_).ToNumber_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter3});
    ca_.Goto(&block17, tmp13);
  }

  TNode<JSAny> phi_bb17_10;
  TNode<JSTypedArray> tmp14;
  TNode<UintPtrT> tmp15;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_10);
    compiler::CodeAssemblerLabel label16(&ca_);
    std::tie(tmp14, tmp15) = EnsureValidAndReadLength_0(state_, TNode<JSTypedArray>{tmp0}, CastIfEnumClass<TypedArrayAccessMode>(TypedArrayAccessMode::kRead), &label16).Flatten();
    ca_.Goto(&block18);
    if (label16.is_used()) {
      ca_.Bind(&label16);
      ca_.Goto(&block19);
    }
  }

  if (block19.is_used()) {
    ca_.Bind(&block19);
    ca_.Goto(&block6);
  }

  TNode<Number> tmp17;
  TNode<BoolT> tmp18;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp17 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp18 = NumberIsLessThan_0(state_, TNode<Number>{phi_bb13_9}, TNode<Number>{tmp17});
    ca_.Branch(tmp18, &block22, std::vector<compiler::Node*>{}, &block23, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp19;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    tmp19 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block24, tmp19);
  }

  TNode<Number> tmp20;
  TNode<BoolT> tmp21;
  if (block23.is_used()) {
    ca_.Bind(&block23);
    tmp20 = Convert_Number_uintptr_0(state_, TNode<UintPtrT>{tmp15});
    tmp21 = NumberIsGreaterThanOrEqual_0(state_, TNode<Number>{phi_bb13_9}, TNode<Number>{tmp20});
    ca_.Goto(&block24, tmp21);
  }

  TNode<BoolT> phi_bb24_12;
  if (block24.is_used()) {
    ca_.Bind(&block24, &phi_bb24_12);
    ca_.Branch(phi_bb24_12, &block20, std::vector<compiler::Node*>{}, &block21, std::vector<compiler::Node*>{});
  }

  if (block20.is_used()) {
    ca_.Bind(&block20);
    ca_.Goto(&block6);
  }

  TNode<JSTypedArray> tmp22;
  TNode<UintPtrT> tmp23;
  TNode<UintPtrT> tmp24;
  TNode<Int32T> tmp25;
  TNode<UintPtrT> tmp26;
  TNode<UintPtrT> tmp27;
  TNode<BoolT> tmp28;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp22 = TypedArrayCreateSameType_0(state_, TNode<Context>{parameter0}, TNode<JSTypedArray>{tmp0}, TNode<UintPtrT>{tmp3});
    tmp23 = CodeStubAssembler(state_).UintPtrMin(TNode<UintPtrT>{tmp3}, TNode<UintPtrT>{tmp15});
    std::tie(tmp24, tmp25) = TypedArrayBuiltinsAssembler(state_).GetTypedArrayElementsInfo(TNode<JSTypedArray>{tmp22}).Flatten();
    tmp26 = CodeStubAssembler(state_).ArrayBufferMaxByteLength();
    tmp27 = CodeStubAssembler(state_).WordShr(TNode<UintPtrT>{tmp26}, TNode<UintPtrT>{tmp24});
    tmp28 = CodeStubAssembler(state_).UintPtrGreaterThan(TNode<UintPtrT>{tmp23}, TNode<UintPtrT>{tmp27});
    ca_.Branch(tmp28, &block28, std::vector<compiler::Node*>{}, &block29, std::vector<compiler::Node*>{});
  }

  if (block28.is_used()) {
    ca_.Bind(&block28);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<UintPtrT> tmp29;
  TNode<RawPtrT> tmp30;
  TNode<IntPtrT> tmp31;
  TNode<JSArrayBuffer> tmp32;
  TNode<BoolT> tmp33;
  if (block29.is_used()) {
    ca_.Bind(&block29);
    tmp29 = CodeStubAssembler(state_).WordShl(TNode<UintPtrT>{tmp23}, TNode<UintPtrT>{tmp24});
    tmp30 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp31 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp32 = CodeStubAssembler(state_).LoadReference<JSArrayBuffer>(CodeStubAssembler::Reference{tmp0, tmp31});
    tmp33 = IsSharedArrayBuffer_0(state_, TNode<JSArrayBuffer>{tmp32});
    ca_.Branch(tmp33, &block30, std::vector<compiler::Node*>{}, &block31, std::vector<compiler::Node*>{});
  }

  TNode<RawPtrT> tmp34;
  if (block30.is_used()) {
    ca_.Bind(&block30);
    tmp34 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp22});
    TypedArrayBuiltinsAssembler(state_).CallCRelaxedMemmove(TNode<RawPtrT>{tmp34}, TNode<RawPtrT>{tmp30}, TNode<UintPtrT>{tmp29});
    ca_.Goto(&block32);
  }

  TNode<RawPtrT> tmp35;
  if (block31.is_used()) {
    ca_.Bind(&block31);
    tmp35 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp22});
    TypedArrayBuiltinsAssembler(state_).CallCMemmove(TNode<RawPtrT>{tmp35}, TNode<RawPtrT>{tmp30}, TNode<UintPtrT>{tmp29});
    ca_.Goto(&block32);
  }

  TNode<Int32T> tmp36;
  TNode<BuiltinPtr> tmp37;
  TNode<BuiltinPtr> tmp38;
  TNode<BuiltinPtr> tmp39;
  TNode<UintPtrT> tmp40;
  TNode<BoolT> tmp41;
  if (block32.is_used()) {
    ca_.Bind(&block32);
    tmp36 = CodeStubAssembler(state_).LoadElementsKind(TNode<JSTypedArray>{tmp22});
    std::tie(tmp37, tmp38, tmp39) = GetTypedArrayAccessor_0(state_, TNode<Int32T>{tmp36}).Flatten();
    tmp40 = Convert_uintptr_Number_0(state_, TNode<Number>{phi_bb13_9});
    tmp41 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp40}, TNode<UintPtrT>{tmp3});
    ca_.Branch(tmp41, &block33, std::vector<compiler::Node*>{}, &block34, std::vector<compiler::Node*>{});
  }

  TNode<Smi> tmp42;
  TNode<Smi> tmp43;
  TNode<BoolT> tmp44;
  if (block33.is_used()) {
    ca_.Bind(&block33);
tmp42 = TORQUE_CAST(CodeStubAssembler(state_).CallBuiltinPointer(Builtins::CallInterfaceDescriptorFor(ExampleBuiltinForTorqueFunctionPointerType(2)), tmp39, parameter0, tmp22, tmp40, phi_bb17_10));
    tmp43 = kStoreSucceded_0(state_);
    tmp44 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp42}, TNode<Smi>{tmp43});
    ca_.Branch(tmp44, &block36, std::vector<compiler::Node*>{}, &block37, std::vector<compiler::Node*>{});
  }

  if (block37.is_used()) {
    ca_.Bind(&block37);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/typed-array.tq", 115});
      CodeStubAssembler(state_).FailAssert("Torque assert 'result == kStoreSucceded' failed", pos_stack);
    }
  }

  if (block36.is_used()) {
    ca_.Bind(&block36);
    ca_.Goto(&block34);
  }

  if (block34.is_used()) {
    ca_.Bind(&block34);
    ca_.Goto(&block40, tmp23);
  }

  TNode<UintPtrT> phi_bb40_21;
  TNode<UintPtrT> tmp45;
  TNode<BoolT> tmp46;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_21);
    tmp45 = CodeStubAssembler(state_).LoadJSTypedArrayLength(TNode<JSTypedArray>{tmp22});
    tmp46 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{phi_bb40_21}, TNode<UintPtrT>{tmp45});
    ca_.Branch(tmp46, &block38, std::vector<compiler::Node*>{phi_bb40_21}, &block39, std::vector<compiler::Node*>{phi_bb40_21});
  }

  TNode<UintPtrT> phi_bb38_21;
  TNode<Undefined> tmp47;
  TNode<Smi> tmp48;
  TNode<Smi> tmp49;
  TNode<BoolT> tmp50;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_21);
    tmp47 = Undefined_0(state_);
tmp48 = TORQUE_CAST(CodeStubAssembler(state_).CallBuiltinPointer(Builtins::CallInterfaceDescriptorFor(ExampleBuiltinForTorqueFunctionPointerType(2)), tmp39, parameter0, tmp22, phi_bb38_21, tmp47));
    tmp49 = kStoreSucceded_0(state_);
    tmp50 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp48}, TNode<Smi>{tmp49});
    ca_.Branch(tmp50, &block42, std::vector<compiler::Node*>{phi_bb38_21, phi_bb38_21, phi_bb38_21}, &block43, std::vector<compiler::Node*>{phi_bb38_21, phi_bb38_21, phi_bb38_21});
  }

  TNode<UintPtrT> phi_bb43_21;
  TNode<UintPtrT> phi_bb43_27;
  TNode<UintPtrT> phi_bb43_31;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_21, &phi_bb43_27, &phi_bb43_31);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/typed-array.tq", 115});
      CodeStubAssembler(state_).FailAssert("Torque assert 'result == kStoreSucceded' failed", pos_stack);
    }
  }

  TNode<UintPtrT> phi_bb42_21;
  TNode<UintPtrT> phi_bb42_27;
  TNode<UintPtrT> phi_bb42_31;
  TNode<UintPtrT> tmp51;
  TNode<UintPtrT> tmp52;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_21, &phi_bb42_27, &phi_bb42_31);
    tmp51 = FromConstexpr_uintptr_constexpr_int31_0(state_, 1);
    tmp52 = CodeStubAssembler(state_).UintPtrAdd(TNode<UintPtrT>{phi_bb42_21}, TNode<UintPtrT>{tmp51});
    ca_.Goto(&block40, tmp52);
  }

  TNode<UintPtrT> phi_bb39_21;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_21);
    CodeStubAssembler(state_).Return(tmp22);
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowRangeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kInvalidTypedArrayIndex));
  }
}

} // namespace internal
} // namespace v8
