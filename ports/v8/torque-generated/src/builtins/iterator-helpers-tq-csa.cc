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
#include "torque-generated/src/builtins/iterator-helpers-tq-csa.h"
#include "torque-generated/src/objects/js-array-tq-csa.h"
#include "torque-generated/src/objects/fixed-array-tq-csa.h"
#include "torque-generated/src/objects/intl-objects-tq-csa.h"
#include "torque-generated/src/objects/js-iterator-helpers-tq-csa.h"
#include "torque-generated/src/objects/contexts-tq-csa.h"
#include "torque-generated/src/builtins/array-find-tq-csa.h"
#include "torque-generated/src/builtins/convert-tq-csa.h"
#include "torque-generated/src/builtins/frame-arguments-tq-csa.h"
#include "torque-generated/src/builtins/string-replaceall-tq-csa.h"
#include "torque-generated/src/builtins/object-tq-csa.h"
#include "torque-generated/src/builtins/torque-internal-tq-csa.h"
#include "torque-generated/src/builtins/cast-tq-csa.h"
#include "torque-generated/src/builtins/iterator-helpers-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"
#include "torque-generated/src/builtins/growable-fixed-array-tq-csa.h"
#include "torque-generated/src/builtins/iterator-from-tq-csa.h"
#include "torque-generated/src/builtins/iterator-tq-csa.h"
#include "torque-generated/test/torque/test-torque-tq-csa.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=9&c=1
TNode<BoolT> IsIteratorHelperExhausted_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Smi> tmp1;
  TNode<Uint32T> tmp2;
  TNode<Smi> tmp3;
  TNode<BoolT> tmp4;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp1 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{p_helper, tmp0});
    tmp2 = FromConstexpr_JSIteratorHelperState_constexpr_kCompleted_0(state_, JSIteratorHelperState::kCompleted);
    tmp3 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp2});
    tmp4 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp1}, TNode<Smi>{tmp3});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<BoolT>{tmp4};
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=14&c=1
void MarkIteratorHelperAsExhausted_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Uint32T> tmp1;
  TNode<Smi> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp1 = FromConstexpr_JSIteratorHelperState_constexpr_kCompleted_0(state_, JSIteratorHelperState::kCompleted);
    tmp2 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp1});
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{p_helper, tmp0}, tmp2);
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=19&c=1
TNode<BoolT> IsIteratorHelperExecuting_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Smi> tmp1;
  TNode<Uint32T> tmp2;
  TNode<Smi> tmp3;
  TNode<BoolT> tmp4;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp1 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{p_helper, tmp0});
    tmp2 = FromConstexpr_JSIteratorHelperState_constexpr_kExecuting_0(state_, JSIteratorHelperState::kExecuting);
    tmp3 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp2});
    tmp4 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp1}, TNode<Smi>{tmp3});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<BoolT>{tmp4};
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=27&c=1
void ThrowIfIteratorHelperExecuting_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSIteratorHelper> p_helper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = IsIteratorHelperExecuting_0(state_, TNode<JSIteratorHelper>{p_helper});
    ca_.Branch(tmp0, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  if (block2.is_used()) {
    ca_.Bind(&block2);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{p_context}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kGeneratorRunning));
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block4);
  }

    ca_.Bind(&block4);
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=34&c=1
void MarkIteratorHelperAsExecuting_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Uint32T> tmp1;
  TNode<Smi> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp1 = FromConstexpr_JSIteratorHelperState_constexpr_kExecuting_0(state_, JSIteratorHelperState::kExecuting);
    tmp2 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp1});
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{p_helper, tmp0}, tmp2);
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=40&c=1
TNode<BoolT> IteratorHelperIsSuspendedStart_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Smi> tmp1;
  TNode<Uint32T> tmp2;
  TNode<Smi> tmp3;
  TNode<BoolT> tmp4;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp1 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{p_helper, tmp0});
    tmp2 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedStart_0(state_, JSIteratorHelperState::kSuspendedStart);
    tmp3 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp2});
    tmp4 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp1}, TNode<Smi>{tmp3});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<BoolT>{tmp4};
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=45&c=1
void MarkIteratorHelperAsFinishedExecuting_0(compiler::CodeAssemblerState* state_, TNode<JSIteratorHelper> p_helper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Uint32T> tmp1;
  TNode<Smi> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp1 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedYield_0(state_, JSIteratorHelperState::kSuspendedYield);
    tmp2 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp1});
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{p_helper, tmp0}, tmp2);
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=52&c=1
TorqueStructIteratorRecord GetIteratorDirect_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_obj) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<String> tmp0;
  TNode<JSAny> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = kNextString_0(state_);
    tmp1 = CodeStubAssembler(state_).GetProperty(TNode<Context>{p_context}, TNode<JSAny>{p_obj}, TNode<JSAny>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructIteratorRecord{TNode<JSReceiver>{p_obj}, TNode<JSAny>{tmp1}};
}

TF_BUILTIN(IteratorHelperPrototypeNext, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSIteratorHelper> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSIteratorHelper_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block3);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block4);
    }
  }

  TNode<Object> tmp2;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp2 = FromConstexpr_Object_constexpr_string_0(state_, "Iterator Helper.prototype.next");
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kIncompatibleMethodReceiver), TNode<Object>{tmp2}, TNode<Object>{parameter1});
  }

  TNode<BoolT> tmp3;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    ThrowIfIteratorHelperExecuting_0(state_, TNode<Context>{parameter0}, TNode<JSIteratorHelper>{tmp0});
    tmp3 = IsIteratorHelperExhausted_0(state_, TNode<JSIteratorHelper>{tmp0});
    ca_.Branch(tmp3, &block5, std::vector<compiler::Node*>{}, &block6, std::vector<compiler::Node*>{});
  }

  TNode<Undefined> tmp4;
  TNode<True> tmp5;
  TNode<JSObject> tmp6;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp4 = Undefined_0(state_);
    tmp5 = True_0(state_);
    tmp6 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp4}, TNode<Boolean>{tmp5});
    CodeStubAssembler(state_).Return(tmp6);
  }

  TNode<JSIteratorMapHelper> tmp7;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    compiler::CodeAssemblerLabel label8(&ca_);
    tmp7 = Cast_JSIteratorMapHelper_0(state_, TNode<HeapObject>{tmp0}, &label8);
    ca_.Goto(&block9);
    if (label8.is_used()) {
      ca_.Bind(&label8);
      ca_.Goto(&block10);
    }
  }

  TNode<JSIteratorFilterHelper> tmp9;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    compiler::CodeAssemblerLabel label10(&ca_);
    tmp9 = Cast_JSIteratorFilterHelper_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label10);
    ca_.Goto(&block13);
    if (label10.is_used()) {
      ca_.Bind(&label10);
      ca_.Goto(&block14);
    }
  }

  TNode<JSAny> tmp11;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp11 = ca_.CallBuiltin<JSAny>(Builtin::kIteratorMapHelperNext, parameter0, tmp7);
    CodeStubAssembler(state_).Return(tmp11);
  }

  TNode<JSIteratorTakeHelper> tmp12;
  if (block14.is_used()) {
    ca_.Bind(&block14);
    compiler::CodeAssemblerLabel label13(&ca_);
    tmp12 = Cast_JSIteratorTakeHelper_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label13);
    ca_.Goto(&block17);
    if (label13.is_used()) {
      ca_.Bind(&label13);
      ca_.Goto(&block18);
    }
  }

  TNode<JSAny> tmp14;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp14 = ca_.CallBuiltin<JSAny>(Builtin::kIteratorFilterHelperNext, parameter0, tmp9);
    CodeStubAssembler(state_).Return(tmp14);
  }

  TNode<JSIteratorDropHelper> tmp15;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    compiler::CodeAssemblerLabel label16(&ca_);
    tmp15 = Cast_JSIteratorDropHelper_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label16);
    ca_.Goto(&block21);
    if (label16.is_used()) {
      ca_.Bind(&label16);
      ca_.Goto(&block22);
    }
  }

  TNode<JSAny> tmp17;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp17 = ca_.CallBuiltin<JSAny>(Builtin::kIteratorTakeHelperNext, parameter0, tmp12);
    CodeStubAssembler(state_).Return(tmp17);
  }

  TNode<JSIteratorFlatMapHelper> tmp18;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    compiler::CodeAssemblerLabel label19(&ca_);
    tmp18 = Cast_JSIteratorFlatMapHelper_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label19);
    ca_.Goto(&block25);
    if (label19.is_used()) {
      ca_.Bind(&label19);
      ca_.Goto(&block26);
    }
  }

  TNode<JSAny> tmp20;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp20 = ca_.CallBuiltin<JSAny>(Builtin::kIteratorDropHelperNext, parameter0, tmp15);
    CodeStubAssembler(state_).Return(tmp20);
  }

  TNode<JSIteratorConcatHelper> tmp21;
  if (block26.is_used()) {
    ca_.Bind(&block26);
    compiler::CodeAssemblerLabel label22(&ca_);
    tmp21 = Cast_JSIteratorConcatHelper_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label22);
    ca_.Goto(&block29);
    if (label22.is_used()) {
      ca_.Bind(&label22);
      ca_.Goto(&block30);
    }
  }

  TNode<JSAny> tmp23;
  if (block25.is_used()) {
    ca_.Bind(&block25);
    tmp23 = ca_.CallBuiltin<JSAny>(Builtin::kIteratorFlatMapHelperNext, parameter0, tmp18);
    CodeStubAssembler(state_).Return(tmp23);
  }

  TNode<JSIteratorZipHelper> tmp24;
  if (block30.is_used()) {
    ca_.Bind(&block30);
    compiler::CodeAssemblerLabel label25(&ca_);
    tmp24 = Cast_JSIteratorZipHelper_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label25);
    ca_.Goto(&block33);
    if (label25.is_used()) {
      ca_.Bind(&label25);
      ca_.Goto(&block34);
    }
  }

  TNode<JSAny> tmp26;
  if (block29.is_used()) {
    ca_.Bind(&block29);
    tmp26 = ca_.CallBuiltin<JSAny>(Builtin::kIteratorConcatHelperNext, parameter0, tmp21);
    CodeStubAssembler(state_).Return(tmp26);
  }

  if (block34.is_used()) {
    ca_.Bind(&block34);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> tmp27;
  if (block33.is_used()) {
    ca_.Bind(&block33);
    tmp27 = ca_.CallBuiltin<JSAny>(Builtin::kIteratorZipHelperNext, parameter0, tmp24);
    CodeStubAssembler(state_).Return(tmp27);
  }
}

TF_BUILTIN(IteratorHelperPrototypeReturn, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block33(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block36(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block37(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block38(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block39(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block40(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block41(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block42(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block43(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block44(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block45(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block46(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block47(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block52(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block53(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block54(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block55(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block56(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block61(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block60(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block62(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block63(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSIteratorHelper> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSIteratorHelper_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block3);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block4);
    }
  }

  TNode<Object> tmp2;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp2 = FromConstexpr_Object_constexpr_string_0(state_, "Iterator Helper.prototype.return");
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kIncompatibleMethodReceiver), TNode<Object>{tmp2}, TNode<Object>{parameter1});
  }

  TNode<BoolT> tmp3;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    ThrowIfIteratorHelperExecuting_0(state_, TNode<Context>{parameter0}, TNode<JSIteratorHelper>{tmp0});
    tmp3 = IsIteratorHelperExhausted_0(state_, TNode<JSIteratorHelper>{tmp0});
    ca_.Branch(tmp3, &block5, std::vector<compiler::Node*>{}, &block6, std::vector<compiler::Node*>{});
  }

  TNode<Undefined> tmp4;
  TNode<True> tmp5;
  TNode<JSObject> tmp6;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp4 = Undefined_0(state_);
    tmp5 = True_0(state_);
    tmp6 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp4}, TNode<Boolean>{tmp5});
    CodeStubAssembler(state_).Return(tmp6);
  }

  TNode<BoolT> tmp7;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp7 = IteratorHelperIsSuspendedStart_0(state_, TNode<JSIteratorHelper>{tmp0});
    ca_.Branch(tmp7, &block7, std::vector<compiler::Node*>{}, &block8, std::vector<compiler::Node*>{});
  }

  TNode<JSIteratorConcatHelper> tmp8;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{tmp0});
    compiler::CodeAssemblerLabel label9(&ca_);
    tmp8 = Cast_JSIteratorConcatHelper_0(state_, TNode<HeapObject>{tmp0}, &label9);
    ca_.Goto(&block11);
    if (label9.is_used()) {
      ca_.Bind(&label9);
      ca_.Goto(&block12);
    }
  }

  TNode<JSIteratorHelperSimple> tmp10;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    compiler::CodeAssemblerLabel label11(&ca_);
    tmp10 = Cast_JSIteratorHelperSimple_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label11);
    ca_.Goto(&block15);
    if (label11.is_used()) {
      ca_.Bind(&label11);
      ca_.Goto(&block16);
    }
  }

  if (block11.is_used()) {
    ca_.Bind(&block11);
    ca_.Goto(&block9);
  }

  TNode<JSIteratorZipHelper> tmp12;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    compiler::CodeAssemblerLabel label13(&ca_);
    tmp12 = Cast_JSIteratorZipHelper_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label13);
    ca_.Goto(&block25);
    if (label13.is_used()) {
      ca_.Bind(&label13);
      ca_.Goto(&block26);
    }
  }

  TNode<IntPtrT> tmp14;
      TNode<JSAny> tmp16;
  TNode<JSReceiver> tmp17;
  TNode<IntPtrT> tmp18;
      TNode<JSAny> tmp20;
  TNode<IntPtrT> tmp21;
      TNode<JSAny> tmp23;
  TNode<JSAny> tmp24;
      TNode<JSAny> tmp26;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    compiler::CodeAssemblerExceptionHandlerLabel catch15__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch15__label);
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch15__label.is_used()) {
      compiler::CodeAssemblerLabel catch15_skip(&ca_);
      ca_.Goto(&catch15_skip);
      ca_.Bind(&catch15__label, &tmp16);
      ca_.Goto(&block19);
      ca_.Bind(&catch15_skip);
    }
    tmp17 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{tmp10, tmp14});
    compiler::CodeAssemblerExceptionHandlerLabel catch19__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch19__label);
    tmp18 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch19__label.is_used()) {
      compiler::CodeAssemblerLabel catch19_skip(&ca_);
      ca_.Goto(&catch19_skip);
      ca_.Bind(&catch19__label, &tmp20);
      ca_.Goto(&block20);
      ca_.Bind(&catch19_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch22__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch22__label);
    tmp21 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp14}, TNode<IntPtrT>{tmp18});
    }
    if (catch22__label.is_used()) {
      compiler::CodeAssemblerLabel catch22_skip(&ca_);
      ca_.Goto(&catch22_skip);
      ca_.Bind(&catch22__label, &tmp23);
      ca_.Goto(&block21);
      ca_.Bind(&catch22_skip);
    }
    tmp24 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{tmp10, tmp21});
    compiler::CodeAssemblerExceptionHandlerLabel catch25__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch25__label);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp17}, TNode<JSAny>{tmp24}});
    }
    if (catch25__label.is_used()) {
      compiler::CodeAssemblerLabel catch25_skip(&ca_);
      ca_.Goto(&catch25_skip);
      ca_.Bind(&catch25__label, &tmp26);
      ca_.Goto(&block22);
      ca_.Bind(&catch25_skip);
    }
    ca_.Goto(&block13);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp27;
  if (block19.is_used()) {
    ca_.Bind(&block19);
    tmp27 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block18, tmp16, tmp27);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp28;
  if (block20.is_used()) {
    ca_.Bind(&block20);
    tmp28 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block18, tmp20, tmp28);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp29;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp29 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block18, tmp23, tmp29);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp30;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    tmp30 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block18, tmp26, tmp30);
  }

  TNode<JSAny> phi_bb18_5;
  TNode<Union<JSMessageObject, TheHole>> phi_bb18_6;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_5, &phi_bb18_6);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb18_5, phi_bb18_6);
    CodeStubAssembler(state_).Unreachable();
  }

  if (block26.is_used()) {
    ca_.Bind(&block26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> tmp31;
  TNode<FixedArray> tmp32;
  if (block25.is_used()) {
    ca_.Bind(&block25);
    tmp31 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp32 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{tmp12, tmp31});
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp32}, true);
    ca_.Goto(&block13);
  }

  if (block13.is_used()) {
    ca_.Bind(&block13);
    ca_.Goto(&block9);
  }

  TNode<Undefined> tmp33;
  TNode<True> tmp34;
  TNode<JSObject> tmp35;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp33 = Undefined_0(state_);
    tmp34 = True_0(state_);
    tmp35 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp33}, TNode<Boolean>{tmp34});
    CodeStubAssembler(state_).Return(tmp35);
  }

  TNode<JSIteratorFlatMapHelper> tmp36;
      TNode<JSAny> tmp39;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    MarkIteratorHelperAsExecuting_0(state_, TNode<JSIteratorHelper>{tmp0});
    compiler::CodeAssemblerLabel label37(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch38__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch38__label);
    tmp36 = Cast_JSIteratorFlatMapHelper_0(state_, TNode<HeapObject>{tmp0}, &label37);
    }
    if (catch38__label.is_used()) {
      compiler::CodeAssemblerLabel catch38_skip(&ca_);
      ca_.Goto(&catch38_skip);
      ca_.Bind(&catch38__label, &tmp39);
      ca_.Goto(&block33);
      ca_.Bind(&catch38_skip);
    }
    ca_.Goto(&block31);
    if (label37.is_used()) {
      ca_.Bind(&label37);
      ca_.Goto(&block32);
    }
  }

  TNode<Union<JSMessageObject, TheHole>> tmp40;
  if (block33.is_used()) {
    ca_.Bind(&block33);
    tmp40 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp39, tmp40);
  }

  TNode<JSIteratorHelperSimple> tmp41;
      TNode<JSAny> tmp44;
  if (block32.is_used()) {
    ca_.Bind(&block32);
    compiler::CodeAssemblerLabel label42(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch43__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch43__label);
    tmp41 = Cast_JSIteratorHelperSimple_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label42);
    }
    if (catch43__label.is_used()) {
      compiler::CodeAssemblerLabel catch43_skip(&ca_);
      ca_.Goto(&catch43_skip);
      ca_.Bind(&catch43__label, &tmp44);
      ca_.Goto(&block52);
      ca_.Bind(&catch43_skip);
    }
    ca_.Goto(&block50);
    if (label42.is_used()) {
      ca_.Bind(&label42);
      ca_.Goto(&block51);
    }
  }

  TNode<IntPtrT> tmp45;
      TNode<JSAny> tmp47;
  TNode<JSReceiver> tmp48;
  TNode<IntPtrT> tmp49;
      TNode<JSAny> tmp51;
  TNode<IntPtrT> tmp52;
      TNode<JSAny> tmp54;
  TNode<JSAny> tmp55;
      TNode<JSAny> tmp57;
  TNode<IntPtrT> tmp58;
      TNode<JSAny> tmp60;
  TNode<JSReceiver> tmp61;
  TNode<IntPtrT> tmp62;
      TNode<JSAny> tmp64;
  TNode<IntPtrT> tmp65;
      TNode<JSAny> tmp67;
  TNode<JSAny> tmp68;
      TNode<JSAny> tmp70;
  if (block31.is_used()) {
    ca_.Bind(&block31);
    compiler::CodeAssemblerExceptionHandlerLabel catch46__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch46__label);
    tmp45 = FromConstexpr_intptr_constexpr_int31_0(state_, 32);
    }
    if (catch46__label.is_used()) {
      compiler::CodeAssemblerLabel catch46_skip(&ca_);
      ca_.Goto(&catch46_skip);
      ca_.Bind(&catch46__label, &tmp47);
      ca_.Goto(&block36);
      ca_.Bind(&catch46_skip);
    }
    tmp48 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{tmp36, tmp45});
    compiler::CodeAssemblerExceptionHandlerLabel catch50__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch50__label);
    tmp49 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch50__label.is_used()) {
      compiler::CodeAssemblerLabel catch50_skip(&ca_);
      ca_.Goto(&catch50_skip);
      ca_.Bind(&catch50__label, &tmp51);
      ca_.Goto(&block37);
      ca_.Bind(&catch50_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch53__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch53__label);
    tmp52 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp45}, TNode<IntPtrT>{tmp49});
    }
    if (catch53__label.is_used()) {
      compiler::CodeAssemblerLabel catch53_skip(&ca_);
      ca_.Goto(&catch53_skip);
      ca_.Bind(&catch53__label, &tmp54);
      ca_.Goto(&block38);
      ca_.Bind(&catch53_skip);
    }
    tmp55 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{tmp36, tmp52});
    compiler::CodeAssemblerExceptionHandlerLabel catch56__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch56__label);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp48}, TNode<JSAny>{tmp55}});
    }
    if (catch56__label.is_used()) {
      compiler::CodeAssemblerLabel catch56_skip(&ca_);
      ca_.Goto(&catch56_skip);
      ca_.Bind(&catch56__label, &tmp57);
      ca_.Goto(&block39);
      ca_.Bind(&catch56_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch59__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch59__label);
    tmp58 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch59__label.is_used()) {
      compiler::CodeAssemblerLabel catch59_skip(&ca_);
      ca_.Goto(&catch59_skip);
      ca_.Bind(&catch59__label, &tmp60);
      ca_.Goto(&block44);
      ca_.Bind(&catch59_skip);
    }
    tmp61 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{tmp36, tmp58});
    compiler::CodeAssemblerExceptionHandlerLabel catch63__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch63__label);
    tmp62 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch63__label.is_used()) {
      compiler::CodeAssemblerLabel catch63_skip(&ca_);
      ca_.Goto(&catch63_skip);
      ca_.Bind(&catch63__label, &tmp64);
      ca_.Goto(&block45);
      ca_.Bind(&catch63_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch66__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch66__label);
    tmp65 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp58}, TNode<IntPtrT>{tmp62});
    }
    if (catch66__label.is_used()) {
      compiler::CodeAssemblerLabel catch66_skip(&ca_);
      ca_.Goto(&catch66_skip);
      ca_.Bind(&catch66__label, &tmp67);
      ca_.Goto(&block46);
      ca_.Bind(&catch66_skip);
    }
    tmp68 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{tmp36, tmp65});
    compiler::CodeAssemblerExceptionHandlerLabel catch69__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch69__label);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp61}, TNode<JSAny>{tmp68}});
    }
    if (catch69__label.is_used()) {
      compiler::CodeAssemblerLabel catch69_skip(&ca_);
      ca_.Goto(&catch69_skip);
      ca_.Bind(&catch69__label, &tmp70);
      ca_.Goto(&block47);
      ca_.Bind(&catch69_skip);
    }
    ca_.Goto(&block29);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp71;
  if (block36.is_used()) {
    ca_.Bind(&block36);
    tmp71 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block35, tmp47, tmp71);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp72;
  if (block37.is_used()) {
    ca_.Bind(&block37);
    tmp72 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block35, tmp51, tmp72);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp73;
  if (block38.is_used()) {
    ca_.Bind(&block38);
    tmp73 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block35, tmp54, tmp73);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp74;
  if (block39.is_used()) {
    ca_.Bind(&block39);
    tmp74 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block35, tmp57, tmp74);
  }

  TNode<JSAny> phi_bb35_5;
  TNode<Union<JSMessageObject, TheHole>> phi_bb35_6;
  TNode<IntPtrT> tmp75;
      TNode<JSAny> tmp77;
  TNode<JSReceiver> tmp78;
      TNode<JSAny> tmp80;
      TNode<JSAny> tmp82;
      TNode<JSAny> tmp84;
  if (block35.is_used()) {
    ca_.Bind(&block35, &phi_bb35_5, &phi_bb35_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch76__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch76__label);
    tmp75 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch76__label.is_used()) {
      compiler::CodeAssemblerLabel catch76_skip(&ca_);
      ca_.Goto(&catch76_skip);
      ca_.Bind(&catch76__label, &tmp77);
      ca_.Goto(&block40);
      ca_.Bind(&catch76_skip);
    }
    tmp78 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{tmp36, tmp75});
    compiler::CodeAssemblerExceptionHandlerLabel catch79__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch79__label);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp78});
    }
    if (catch79__label.is_used()) {
      compiler::CodeAssemblerLabel catch79_skip(&ca_);
      ca_.Goto(&catch79_skip);
      ca_.Bind(&catch79__label, &tmp80);
      ca_.Goto(&block41);
      ca_.Bind(&catch79_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch81__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch81__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{tmp36});
    }
    if (catch81__label.is_used()) {
      compiler::CodeAssemblerLabel catch81_skip(&ca_);
      ca_.Goto(&catch81_skip);
      ca_.Bind(&catch81__label, &tmp82);
      ca_.Goto(&block42);
      ca_.Bind(&catch81_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch83__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch83__label);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb35_5, phi_bb35_6);
    CodeStubAssembler(state_).Unreachable();
    }
    if (catch83__label.is_used()) {
      compiler::CodeAssemblerLabel catch83_skip(&ca_);
      ca_.Bind(&catch83__label, &tmp84);
      ca_.Goto(&block43);
    }
  }

  TNode<Union<JSMessageObject, TheHole>> tmp85;
  if (block40.is_used()) {
    ca_.Bind(&block40);
    tmp85 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp77, tmp85);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp86;
  if (block41.is_used()) {
    ca_.Bind(&block41);
    tmp86 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp80, tmp86);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp87;
  if (block42.is_used()) {
    ca_.Bind(&block42);
    tmp87 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp82, tmp87);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp88;
  if (block43.is_used()) {
    ca_.Bind(&block43);
    tmp88 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp84, tmp88);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp89;
  if (block44.is_used()) {
    ca_.Bind(&block44);
    tmp89 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp60, tmp89);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp90;
  if (block45.is_used()) {
    ca_.Bind(&block45);
    tmp90 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp64, tmp90);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp91;
  if (block46.is_used()) {
    ca_.Bind(&block46);
    tmp91 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp67, tmp91);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp92;
  if (block47.is_used()) {
    ca_.Bind(&block47);
    tmp92 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp70, tmp92);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp93;
  if (block52.is_used()) {
    ca_.Bind(&block52);
    tmp93 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp44, tmp93);
  }

  TNode<JSIteratorZipHelper> tmp94;
      TNode<JSAny> tmp97;
  if (block51.is_used()) {
    ca_.Bind(&block51);
    compiler::CodeAssemblerLabel label95(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch96__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch96__label);
    tmp94 = Cast_JSIteratorZipHelper_0(state_, TNode<HeapObject>{ca_.UncheckedCast<JSIteratorHelper>(tmp0)}, &label95);
    }
    if (catch96__label.is_used()) {
      compiler::CodeAssemblerLabel catch96_skip(&ca_);
      ca_.Goto(&catch96_skip);
      ca_.Bind(&catch96__label, &tmp97);
      ca_.Goto(&block61);
      ca_.Bind(&catch96_skip);
    }
    ca_.Goto(&block59);
    if (label95.is_used()) {
      ca_.Bind(&label95);
      ca_.Goto(&block60);
    }
  }

  TNode<IntPtrT> tmp98;
      TNode<JSAny> tmp100;
  TNode<JSReceiver> tmp101;
  TNode<IntPtrT> tmp102;
      TNode<JSAny> tmp104;
  TNode<IntPtrT> tmp105;
      TNode<JSAny> tmp107;
  TNode<JSAny> tmp108;
      TNode<JSAny> tmp110;
  if (block50.is_used()) {
    ca_.Bind(&block50);
    compiler::CodeAssemblerExceptionHandlerLabel catch99__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch99__label);
    tmp98 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch99__label.is_used()) {
      compiler::CodeAssemblerLabel catch99_skip(&ca_);
      ca_.Goto(&catch99_skip);
      ca_.Bind(&catch99__label, &tmp100);
      ca_.Goto(&block53);
      ca_.Bind(&catch99_skip);
    }
    tmp101 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{tmp41, tmp98});
    compiler::CodeAssemblerExceptionHandlerLabel catch103__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch103__label);
    tmp102 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch103__label.is_used()) {
      compiler::CodeAssemblerLabel catch103_skip(&ca_);
      ca_.Goto(&catch103_skip);
      ca_.Bind(&catch103__label, &tmp104);
      ca_.Goto(&block54);
      ca_.Bind(&catch103_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch106__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch106__label);
    tmp105 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp98}, TNode<IntPtrT>{tmp102});
    }
    if (catch106__label.is_used()) {
      compiler::CodeAssemblerLabel catch106_skip(&ca_);
      ca_.Goto(&catch106_skip);
      ca_.Bind(&catch106__label, &tmp107);
      ca_.Goto(&block55);
      ca_.Bind(&catch106_skip);
    }
    tmp108 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{tmp41, tmp105});
    compiler::CodeAssemblerExceptionHandlerLabel catch109__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch109__label);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp101}, TNode<JSAny>{tmp108}});
    }
    if (catch109__label.is_used()) {
      compiler::CodeAssemblerLabel catch109_skip(&ca_);
      ca_.Goto(&catch109_skip);
      ca_.Bind(&catch109__label, &tmp110);
      ca_.Goto(&block56);
      ca_.Bind(&catch109_skip);
    }
    ca_.Goto(&block48);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp111;
  if (block53.is_used()) {
    ca_.Bind(&block53);
    tmp111 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp100, tmp111);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp112;
  if (block54.is_used()) {
    ca_.Bind(&block54);
    tmp112 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp104, tmp112);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp113;
  if (block55.is_used()) {
    ca_.Bind(&block55);
    tmp113 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp107, tmp113);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp114;
  if (block56.is_used()) {
    ca_.Bind(&block56);
    tmp114 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp110, tmp114);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp115;
  if (block61.is_used()) {
    ca_.Bind(&block61);
    tmp115 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp97, tmp115);
  }

  if (block60.is_used()) {
    ca_.Bind(&block60);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> tmp116;
      TNode<JSAny> tmp118;
  TNode<FixedArray> tmp119;
      TNode<JSAny> tmp121;
  if (block59.is_used()) {
    ca_.Bind(&block59);
    compiler::CodeAssemblerExceptionHandlerLabel catch117__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch117__label);
    tmp116 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch117__label.is_used()) {
      compiler::CodeAssemblerLabel catch117_skip(&ca_);
      ca_.Goto(&catch117_skip);
      ca_.Bind(&catch117__label, &tmp118);
      ca_.Goto(&block62);
      ca_.Bind(&catch117_skip);
    }
    tmp119 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{tmp94, tmp116});
    compiler::CodeAssemblerExceptionHandlerLabel catch120__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch120__label);
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp119}, true);
    }
    if (catch120__label.is_used()) {
      compiler::CodeAssemblerLabel catch120_skip(&ca_);
      ca_.Goto(&catch120_skip);
      ca_.Bind(&catch120__label, &tmp121);
      ca_.Goto(&block63);
      ca_.Bind(&catch120_skip);
    }
    ca_.Goto(&block48);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp122;
  if (block62.is_used()) {
    ca_.Bind(&block62);
    tmp122 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp118, tmp122);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp123;
  if (block63.is_used()) {
    ca_.Bind(&block63);
    tmp123 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block28, tmp121, tmp123);
  }

  if (block48.is_used()) {
    ca_.Bind(&block48);
    ca_.Goto(&block29);
  }

  TNode<Undefined> tmp124;
  TNode<True> tmp125;
  TNode<JSObject> tmp126;
  if (block29.is_used()) {
    ca_.Bind(&block29);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{tmp0});
    tmp124 = Undefined_0(state_);
    tmp125 = True_0(state_);
    tmp126 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp124}, TNode<Boolean>{tmp125});
    CodeStubAssembler(state_).Return(tmp126);
  }

  TNode<JSAny> phi_bb28_3;
  TNode<Union<JSMessageObject, TheHole>> phi_bb28_4;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_3, &phi_bb28_4);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb28_3, phi_bb28_4);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=207&c=1
TNode<JSIteratorMapHelper> NewJSIteratorMapHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_mapper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Map> tmp3;
  TNode<FixedArray> tmp4;
  TNode<FixedArray> tmp5;
  TNode<Uint32T> tmp6;
  TNode<Smi> tmp7;
  TNode<BoolT> tmp8;
  TNode<BoolT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<HeapObject> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<IntPtrT> tmp20;
  TNode<Number> tmp21;
  TNode<JSIteratorMapHelper> tmp22;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ITERATOR_MAP_HELPER_MAP_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Map_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = kEmptyFixedArray_0(state_);
    tmp5 = kEmptyFixedArray_0(state_);
    tmp6 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedStart_0(state_, JSIteratorHelperState::kSuspendedStart);
    tmp7 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp6});
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp9 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, 32);
    tmp11 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp10}, TNode<Map>{tmp3}, TNode<BoolT>{tmp8}, TNode<BoolT>{tmp9});
    tmp12 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp11, tmp12}, tmp3);
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>>(CodeStubAssembler::Reference{tmp11, tmp13}, tmp4);
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp11, tmp14}, tmp5);
    tmp15 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp11, tmp15}, tmp7);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp11, tmp16}, p_underlying.object);
    tmp17 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp16}, TNode<IntPtrT>{tmp17});
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{tmp11, tmp18}, p_underlying.next);
    tmp19 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    CodeStubAssembler(state_).StoreReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>(CodeStubAssembler::Reference{tmp11, tmp19}, p_mapper);
    tmp20 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    tmp21 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{tmp11, tmp20}, tmp21);
    tmp22 = TORQUE_CAST(TNode<HeapObject>{tmp11});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSIteratorMapHelper>{tmp22};
}

TF_BUILTIN(IteratorPrototypeMap, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kMapper);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.map");
  }

  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter2}, &label3);
    ca_.Goto(&block9);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowCalledNonCallable, parameter0, parameter2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSReceiver> tmp4;
  TNode<JSAny> tmp5;
  TNode<JSIteratorMapHelper> tmp6;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    std::tie(tmp4, tmp5) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp6 = NewJSIteratorMapHelper_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}}, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>{tmp2});
    CodeStubAssembler(state_).Return(tmp6);
  }
}

TF_BUILTIN(IteratorMapHelperNext, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSIteratorMapHelper> parameter1 = UncheckedParameter<JSIteratorMapHelper>(Descriptor::kHelper);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Map> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<Number> tmp2;
  TNode<IntPtrT> tmp3;
      TNode<JSAny> tmp5;
  TNode<JSReceiver> tmp6;
  TNode<IntPtrT> tmp7;
      TNode<JSAny> tmp9;
  TNode<IntPtrT> tmp10;
      TNode<JSAny> tmp12;
  TNode<JSAny> tmp13;
  TNode<JSReceiver> tmp14;
      TNode<JSAny> tmp17;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    MarkIteratorHelperAsExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    tmp1 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    tmp2 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{parameter1, tmp1});
    compiler::CodeAssemblerExceptionHandlerLabel catch4__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch4__label);
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch4__label.is_used()) {
      compiler::CodeAssemblerLabel catch4_skip(&ca_);
      ca_.Goto(&catch4_skip);
      ca_.Bind(&catch4__label, &tmp5);
      ca_.Goto(&block5);
      ca_.Bind(&catch4_skip);
    }
    tmp6 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp3});
    compiler::CodeAssemblerExceptionHandlerLabel catch8__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch8__label);
    tmp7 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch8__label.is_used()) {
      compiler::CodeAssemblerLabel catch8_skip(&ca_);
      ca_.Goto(&catch8_skip);
      ca_.Bind(&catch8__label, &tmp9);
      ca_.Goto(&block6);
      ca_.Bind(&catch8_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch11__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch11__label);
    tmp10 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp7});
    }
    if (catch11__label.is_used()) {
      compiler::CodeAssemblerLabel catch11_skip(&ca_);
      ca_.Goto(&catch11_skip);
      ca_.Bind(&catch11__label, &tmp12);
      ca_.Goto(&block7);
      ca_.Bind(&catch11_skip);
    }
    tmp13 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp10});
    compiler::CodeAssemblerLabel label15(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch16__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch16__label);
    tmp14 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp6}, TNode<JSAny>{tmp13}}, TNode<Map>{tmp0}, &label15);
    }
    if (catch16__label.is_used()) {
      compiler::CodeAssemblerLabel catch16_skip(&ca_);
      ca_.Goto(&catch16_skip);
      ca_.Bind(&catch16__label, &tmp17);
      ca_.Goto(&block10);
      ca_.Bind(&catch16_skip);
    }
    ca_.Goto(&block8);
    if (label15.is_used()) {
      ca_.Bind(&label15);
      ca_.Goto(&block9);
    }
  }

  TNode<Union<JSMessageObject, TheHole>> tmp18;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp18 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp5, tmp18);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp19;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp19 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp9, tmp19);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp20;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp20 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp12, tmp20);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp21;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp21 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp17, tmp21);
  }

      TNode<JSAny> tmp23;
  TNode<Undefined> tmp24;
  TNode<True> tmp25;
  TNode<JSObject> tmp26;
      TNode<JSAny> tmp28;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    compiler::CodeAssemblerExceptionHandlerLabel catch22__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch22__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch22__label.is_used()) {
      compiler::CodeAssemblerLabel catch22_skip(&ca_);
      ca_.Goto(&catch22_skip);
      ca_.Bind(&catch22__label, &tmp23);
      ca_.Goto(&block11);
      ca_.Bind(&catch22_skip);
    }
    tmp24 = Undefined_0(state_);
    tmp25 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch27__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch27__label);
    tmp26 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp24}, TNode<Boolean>{tmp25});
    }
    if (catch27__label.is_used()) {
      compiler::CodeAssemblerLabel catch27_skip(&ca_);
      ca_.Goto(&catch27_skip);
      ca_.Bind(&catch27__label, &tmp28);
      ca_.Goto(&block12);
      ca_.Bind(&catch27_skip);
    }
    CodeStubAssembler(state_).Return(tmp26);
  }

  TNode<JSAny> tmp29;
      TNode<JSAny> tmp31;
  TNode<IntPtrT> tmp32;
      TNode<JSAny> tmp34;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp35;
  TNode<Undefined> tmp36;
  TNode<JSAny> tmp37;
      TNode<JSAny> tmp39;
  TNode<IntPtrT> tmp40;
      TNode<JSAny> tmp42;
  TNode<Number> tmp43;
      TNode<JSAny> tmp45;
  TNode<Number> tmp46;
      TNode<JSAny> tmp48;
      TNode<JSAny> tmp50;
  TNode<False> tmp51;
  TNode<JSObject> tmp52;
      TNode<JSAny> tmp54;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    compiler::CodeAssemblerExceptionHandlerLabel catch30__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch30__label);
    tmp29 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp14}, TNode<Map>{tmp0});
    }
    if (catch30__label.is_used()) {
      compiler::CodeAssemblerLabel catch30_skip(&ca_);
      ca_.Goto(&catch30_skip);
      ca_.Bind(&catch30__label, &tmp31);
      ca_.Goto(&block13);
      ca_.Bind(&catch30_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch33__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch33__label);
    tmp32 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch33__label.is_used()) {
      compiler::CodeAssemblerLabel catch33_skip(&ca_);
      ca_.Goto(&catch33_skip);
      ca_.Bind(&catch33__label, &tmp34);
      ca_.Goto(&block16);
      ca_.Bind(&catch33_skip);
    }
    tmp35 = CodeStubAssembler(state_).LoadReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>(CodeStubAssembler::Reference{parameter1, tmp32});
    tmp36 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch38__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch38__label);
    tmp37 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp35}, TNode<JSAny>{tmp36}, TNode<JSAny>{tmp29}, TNode<JSAny>{tmp2});
    }
    if (catch38__label.is_used()) {
      compiler::CodeAssemblerLabel catch38_skip(&ca_);
      ca_.Goto(&catch38_skip);
      ca_.Bind(&catch38__label, &tmp39);
      ca_.Goto(&block17);
      ca_.Bind(&catch38_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch41__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch41__label);
    tmp40 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch41__label.is_used()) {
      compiler::CodeAssemblerLabel catch41_skip(&ca_);
      ca_.Goto(&catch41_skip);
      ca_.Bind(&catch41__label, &tmp42);
      ca_.Goto(&block18);
      ca_.Bind(&catch41_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch44__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch44__label);
    tmp43 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch44__label.is_used()) {
      compiler::CodeAssemblerLabel catch44_skip(&ca_);
      ca_.Goto(&catch44_skip);
      ca_.Bind(&catch44__label, &tmp45);
      ca_.Goto(&block19);
      ca_.Bind(&catch44_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch47__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch47__label);
    tmp46 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{tmp2}, TNode<Number>{tmp43});
    }
    if (catch47__label.is_used()) {
      compiler::CodeAssemblerLabel catch47_skip(&ca_);
      ca_.Goto(&catch47_skip);
      ca_.Bind(&catch47__label, &tmp48);
      ca_.Goto(&block20);
      ca_.Bind(&catch47_skip);
    }
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{parameter1, tmp40}, tmp46);
    compiler::CodeAssemblerExceptionHandlerLabel catch49__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch49__label);
    MarkIteratorHelperAsFinishedExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch49__label.is_used()) {
      compiler::CodeAssemblerLabel catch49_skip(&ca_);
      ca_.Goto(&catch49_skip);
      ca_.Bind(&catch49__label, &tmp50);
      ca_.Goto(&block21);
      ca_.Bind(&catch49_skip);
    }
    tmp51 = False_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch53__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch53__label);
    tmp52 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp37}, TNode<Boolean>{tmp51});
    }
    if (catch53__label.is_used()) {
      compiler::CodeAssemblerLabel catch53_skip(&ca_);
      ca_.Goto(&catch53_skip);
      ca_.Bind(&catch53__label, &tmp54);
      ca_.Goto(&block22);
      ca_.Bind(&catch53_skip);
    }
    CodeStubAssembler(state_).Return(tmp52);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp55;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp55 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp23, tmp55);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp56;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp56 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp28, tmp56);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp57;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp57 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp31, tmp57);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp58;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp58 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block15, tmp34, tmp58);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp59;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp59 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block15, tmp39, tmp59);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp60;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp60 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block15, tmp42, tmp60);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp61;
  if (block19.is_used()) {
    ca_.Bind(&block19);
    tmp61 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block15, tmp45, tmp61);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp62;
  if (block20.is_used()) {
    ca_.Bind(&block20);
    tmp62 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block15, tmp48, tmp62);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp63;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp63 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block15, tmp50, tmp63);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp64;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    tmp64 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block15, tmp54, tmp64);
  }

  TNode<JSAny> phi_bb15_6;
  TNode<Union<JSMessageObject, TheHole>> phi_bb15_7;
  TNode<IntPtrT> tmp65;
      TNode<JSAny> tmp67;
  TNode<JSReceiver> tmp68;
      TNode<JSAny> tmp70;
      TNode<JSAny> tmp72;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_6, &phi_bb15_7);
    compiler::CodeAssemblerExceptionHandlerLabel catch66__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch66__label);
    tmp65 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch66__label.is_used()) {
      compiler::CodeAssemblerLabel catch66_skip(&ca_);
      ca_.Goto(&catch66_skip);
      ca_.Bind(&catch66__label, &tmp67);
      ca_.Goto(&block23);
      ca_.Bind(&catch66_skip);
    }
    tmp68 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp65});
    compiler::CodeAssemblerExceptionHandlerLabel catch69__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch69__label);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp68});
    }
    if (catch69__label.is_used()) {
      compiler::CodeAssemblerLabel catch69_skip(&ca_);
      ca_.Goto(&catch69_skip);
      ca_.Bind(&catch69__label, &tmp70);
      ca_.Goto(&block24);
      ca_.Bind(&catch69_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch71__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch71__label);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb15_6, phi_bb15_7);
    CodeStubAssembler(state_).Unreachable();
    }
    if (catch71__label.is_used()) {
      compiler::CodeAssemblerLabel catch71_skip(&ca_);
      ca_.Bind(&catch71__label, &tmp72);
      ca_.Goto(&block25);
    }
  }

  TNode<Union<JSMessageObject, TheHole>> tmp73;
  if (block23.is_used()) {
    ca_.Bind(&block23);
    tmp73 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp67, tmp73);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp74;
  if (block24.is_used()) {
    ca_.Bind(&block24);
    tmp74 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp70, tmp74);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp75;
  if (block25.is_used()) {
    ca_.Bind(&block25);
    tmp75 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp72, tmp75);
  }

  TNode<JSAny> phi_bb2_4;
  TNode<Union<JSMessageObject, TheHole>> phi_bb2_5;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_4, &phi_bb2_5);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb2_4, phi_bb2_5);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=313&c=1
TNode<JSIteratorFilterHelper> NewJSIteratorFilterHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_predicate) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Map> tmp3;
  TNode<FixedArray> tmp4;
  TNode<FixedArray> tmp5;
  TNode<Uint32T> tmp6;
  TNode<Smi> tmp7;
  TNode<BoolT> tmp8;
  TNode<BoolT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<HeapObject> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<IntPtrT> tmp20;
  TNode<Number> tmp21;
  TNode<JSIteratorFilterHelper> tmp22;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ITERATOR_FILTER_HELPER_MAP_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Map_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = kEmptyFixedArray_0(state_);
    tmp5 = kEmptyFixedArray_0(state_);
    tmp6 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedStart_0(state_, JSIteratorHelperState::kSuspendedStart);
    tmp7 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp6});
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp9 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, 32);
    tmp11 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp10}, TNode<Map>{tmp3}, TNode<BoolT>{tmp8}, TNode<BoolT>{tmp9});
    tmp12 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp11, tmp12}, tmp3);
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>>(CodeStubAssembler::Reference{tmp11, tmp13}, tmp4);
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp11, tmp14}, tmp5);
    tmp15 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp11, tmp15}, tmp7);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp11, tmp16}, p_underlying.object);
    tmp17 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp16}, TNode<IntPtrT>{tmp17});
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{tmp11, tmp18}, p_underlying.next);
    tmp19 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    CodeStubAssembler(state_).StoreReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>(CodeStubAssembler::Reference{tmp11, tmp19}, p_predicate);
    tmp20 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    tmp21 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{tmp11, tmp20}, tmp21);
    tmp22 = TORQUE_CAST(TNode<HeapObject>{tmp11});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSIteratorFilterHelper>{tmp22};
}

TF_BUILTIN(IteratorPrototypeFilter, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kPredicate);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.filter");
  }

  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter2}, &label3);
    ca_.Goto(&block9);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowCalledNonCallable, parameter0, parameter2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSReceiver> tmp4;
  TNode<JSAny> tmp5;
  TNode<JSIteratorFilterHelper> tmp6;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    std::tie(tmp4, tmp5) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp6 = NewJSIteratorFilterHelper_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}}, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>{tmp2});
    CodeStubAssembler(state_).Return(tmp6);
  }
}

TF_BUILTIN(IteratorFilterHelperNext, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSIteratorFilterHelper> parameter1 = UncheckedParameter<JSIteratorFilterHelper>(Descriptor::kHelper);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block28(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block31(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block32(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block33(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Map> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    MarkIteratorHelperAsExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    ca_.Goto(&block5);
  }

  TNode<BoolT> tmp1;
      TNode<JSAny> tmp3;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerExceptionHandlerLabel catch2__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch2__label);
    tmp1 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    }
    if (catch2__label.is_used()) {
      compiler::CodeAssemblerLabel catch2_skip(&ca_);
      ca_.Goto(&catch2_skip);
      ca_.Bind(&catch2__label, &tmp3);
      ca_.Goto(&block6);
      ca_.Bind(&catch2_skip);
    }
    ca_.Branch(tmp1, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  TNode<Union<JSMessageObject, TheHole>> tmp4;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp4 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp3, tmp4);
  }

  TNode<IntPtrT> tmp5;
      TNode<JSAny> tmp7;
  TNode<Number> tmp8;
  TNode<IntPtrT> tmp9;
      TNode<JSAny> tmp11;
  TNode<JSReceiver> tmp12;
  TNode<IntPtrT> tmp13;
      TNode<JSAny> tmp15;
  TNode<IntPtrT> tmp16;
      TNode<JSAny> tmp18;
  TNode<JSAny> tmp19;
  TNode<JSReceiver> tmp20;
      TNode<JSAny> tmp23;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    compiler::CodeAssemblerExceptionHandlerLabel catch6__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch6__label);
    tmp5 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch6__label.is_used()) {
      compiler::CodeAssemblerLabel catch6_skip(&ca_);
      ca_.Goto(&catch6_skip);
      ca_.Bind(&catch6__label, &tmp7);
      ca_.Goto(&block7);
      ca_.Bind(&catch6_skip);
    }
    tmp8 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{parameter1, tmp5});
    compiler::CodeAssemblerExceptionHandlerLabel catch10__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch10__label);
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch10__label.is_used()) {
      compiler::CodeAssemblerLabel catch10_skip(&ca_);
      ca_.Goto(&catch10_skip);
      ca_.Bind(&catch10__label, &tmp11);
      ca_.Goto(&block10);
      ca_.Bind(&catch10_skip);
    }
    tmp12 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp9});
    compiler::CodeAssemblerExceptionHandlerLabel catch14__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch14__label);
    tmp13 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch14__label.is_used()) {
      compiler::CodeAssemblerLabel catch14_skip(&ca_);
      ca_.Goto(&catch14_skip);
      ca_.Bind(&catch14__label, &tmp15);
      ca_.Goto(&block11);
      ca_.Bind(&catch14_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch17__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch17__label);
    tmp16 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp9}, TNode<IntPtrT>{tmp13});
    }
    if (catch17__label.is_used()) {
      compiler::CodeAssemblerLabel catch17_skip(&ca_);
      ca_.Goto(&catch17_skip);
      ca_.Bind(&catch17__label, &tmp18);
      ca_.Goto(&block12);
      ca_.Bind(&catch17_skip);
    }
    tmp19 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp16});
    compiler::CodeAssemblerLabel label21(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch22__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch22__label);
    tmp20 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp12}, TNode<JSAny>{tmp19}}, TNode<Map>{tmp0}, &label21);
    }
    if (catch22__label.is_used()) {
      compiler::CodeAssemblerLabel catch22_skip(&ca_);
      ca_.Goto(&catch22_skip);
      ca_.Bind(&catch22__label, &tmp23);
      ca_.Goto(&block15);
      ca_.Bind(&catch22_skip);
    }
    ca_.Goto(&block13);
    if (label21.is_used()) {
      ca_.Bind(&label21);
      ca_.Goto(&block14);
    }
  }

  TNode<Union<JSMessageObject, TheHole>> tmp24;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp24 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp7, tmp24);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp25;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp25 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp11, tmp25);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp26;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp26 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp15, tmp26);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp27;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp27 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp18, tmp27);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp28;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp28 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp23, tmp28);
  }

      TNode<JSAny> tmp30;
  TNode<Undefined> tmp31;
  TNode<True> tmp32;
  TNode<JSObject> tmp33;
      TNode<JSAny> tmp35;
  if (block14.is_used()) {
    ca_.Bind(&block14);
    compiler::CodeAssemblerExceptionHandlerLabel catch29__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch29__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch29__label.is_used()) {
      compiler::CodeAssemblerLabel catch29_skip(&ca_);
      ca_.Goto(&catch29_skip);
      ca_.Bind(&catch29__label, &tmp30);
      ca_.Goto(&block16);
      ca_.Bind(&catch29_skip);
    }
    tmp31 = Undefined_0(state_);
    tmp32 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch34__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch34__label);
    tmp33 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp31}, TNode<Boolean>{tmp32});
    }
    if (catch34__label.is_used()) {
      compiler::CodeAssemblerLabel catch34_skip(&ca_);
      ca_.Goto(&catch34_skip);
      ca_.Bind(&catch34__label, &tmp35);
      ca_.Goto(&block17);
      ca_.Bind(&catch34_skip);
    }
    CodeStubAssembler(state_).Return(tmp33);
  }

  TNode<JSAny> tmp36;
      TNode<JSAny> tmp38;
  TNode<IntPtrT> tmp39;
      TNode<JSAny> tmp41;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp42;
  TNode<Undefined> tmp43;
  TNode<JSAny> tmp44;
      TNode<JSAny> tmp46;
  TNode<IntPtrT> tmp47;
      TNode<JSAny> tmp49;
  TNode<Number> tmp50;
      TNode<JSAny> tmp52;
  TNode<Number> tmp53;
      TNode<JSAny> tmp55;
  TNode<BoolT> tmp56;
      TNode<JSAny> tmp58;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    compiler::CodeAssemblerExceptionHandlerLabel catch37__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch37__label);
    tmp36 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp20}, TNode<Map>{tmp0});
    }
    if (catch37__label.is_used()) {
      compiler::CodeAssemblerLabel catch37_skip(&ca_);
      ca_.Goto(&catch37_skip);
      ca_.Bind(&catch37__label, &tmp38);
      ca_.Goto(&block18);
      ca_.Bind(&catch37_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch40__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch40__label);
    tmp39 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch40__label.is_used()) {
      compiler::CodeAssemblerLabel catch40_skip(&ca_);
      ca_.Goto(&catch40_skip);
      ca_.Bind(&catch40__label, &tmp41);
      ca_.Goto(&block21);
      ca_.Bind(&catch40_skip);
    }
    tmp42 = CodeStubAssembler(state_).LoadReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>(CodeStubAssembler::Reference{parameter1, tmp39});
    tmp43 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch45__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch45__label);
    tmp44 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp42}, TNode<JSAny>{tmp43}, TNode<JSAny>{tmp36}, TNode<JSAny>{tmp8});
    }
    if (catch45__label.is_used()) {
      compiler::CodeAssemblerLabel catch45_skip(&ca_);
      ca_.Goto(&catch45_skip);
      ca_.Bind(&catch45__label, &tmp46);
      ca_.Goto(&block22);
      ca_.Bind(&catch45_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch48__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch48__label);
    tmp47 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch48__label.is_used()) {
      compiler::CodeAssemblerLabel catch48_skip(&ca_);
      ca_.Goto(&catch48_skip);
      ca_.Bind(&catch48__label, &tmp49);
      ca_.Goto(&block23);
      ca_.Bind(&catch48_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch51__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch51__label);
    tmp50 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch51__label.is_used()) {
      compiler::CodeAssemblerLabel catch51_skip(&ca_);
      ca_.Goto(&catch51_skip);
      ca_.Bind(&catch51__label, &tmp52);
      ca_.Goto(&block24);
      ca_.Bind(&catch51_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch54__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch54__label);
    tmp53 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{tmp8}, TNode<Number>{tmp50});
    }
    if (catch54__label.is_used()) {
      compiler::CodeAssemblerLabel catch54_skip(&ca_);
      ca_.Goto(&catch54_skip);
      ca_.Bind(&catch54__label, &tmp55);
      ca_.Goto(&block25);
      ca_.Bind(&catch54_skip);
    }
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{parameter1, tmp47}, tmp53);
    compiler::CodeAssemblerExceptionHandlerLabel catch57__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch57__label);
    tmp56 = ToBoolean_0(state_, TNode<JSAny>{tmp44});
    }
    if (catch57__label.is_used()) {
      compiler::CodeAssemblerLabel catch57_skip(&ca_);
      ca_.Goto(&catch57_skip);
      ca_.Bind(&catch57__label, &tmp58);
      ca_.Goto(&block28);
      ca_.Bind(&catch57_skip);
    }
    ca_.Branch(tmp56, &block26, std::vector<compiler::Node*>{}, &block27, std::vector<compiler::Node*>{});
  }

  TNode<Union<JSMessageObject, TheHole>> tmp59;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp59 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp30, tmp59);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp60;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp60 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp35, tmp60);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp61;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp61 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp38, tmp61);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp62;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp62 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block20, tmp41, tmp62);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp63;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    tmp63 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block20, tmp46, tmp63);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp64;
  if (block23.is_used()) {
    ca_.Bind(&block23);
    tmp64 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block20, tmp49, tmp64);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp65;
  if (block24.is_used()) {
    ca_.Bind(&block24);
    tmp65 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block20, tmp52, tmp65);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp66;
  if (block25.is_used()) {
    ca_.Bind(&block25);
    tmp66 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block20, tmp55, tmp66);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp67;
  if (block28.is_used()) {
    ca_.Bind(&block28);
    tmp67 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block20, tmp58, tmp67);
  }

      TNode<JSAny> tmp69;
  TNode<False> tmp70;
  TNode<JSObject> tmp71;
      TNode<JSAny> tmp73;
  if (block26.is_used()) {
    ca_.Bind(&block26);
    compiler::CodeAssemblerExceptionHandlerLabel catch68__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch68__label);
    MarkIteratorHelperAsFinishedExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch68__label.is_used()) {
      compiler::CodeAssemblerLabel catch68_skip(&ca_);
      ca_.Goto(&catch68_skip);
      ca_.Bind(&catch68__label, &tmp69);
      ca_.Goto(&block29);
      ca_.Bind(&catch68_skip);
    }
    tmp70 = False_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch72__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch72__label);
    tmp71 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp36}, TNode<Boolean>{tmp70});
    }
    if (catch72__label.is_used()) {
      compiler::CodeAssemblerLabel catch72_skip(&ca_);
      ca_.Goto(&catch72_skip);
      ca_.Bind(&catch72__label, &tmp73);
      ca_.Goto(&block30);
      ca_.Bind(&catch72_skip);
    }
    CodeStubAssembler(state_).Return(tmp71);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp74;
  if (block29.is_used()) {
    ca_.Bind(&block29);
    tmp74 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block20, tmp69, tmp74);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp75;
  if (block30.is_used()) {
    ca_.Bind(&block30);
    tmp75 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block20, tmp73, tmp75);
  }

  if (block27.is_used()) {
    ca_.Bind(&block27);
    ca_.Goto(&block5);
  }

  TNode<JSAny> phi_bb20_6;
  TNode<Union<JSMessageObject, TheHole>> phi_bb20_7;
  TNode<IntPtrT> tmp76;
      TNode<JSAny> tmp78;
  TNode<JSReceiver> tmp79;
      TNode<JSAny> tmp81;
      TNode<JSAny> tmp83;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_6, &phi_bb20_7);
    compiler::CodeAssemblerExceptionHandlerLabel catch77__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch77__label);
    tmp76 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch77__label.is_used()) {
      compiler::CodeAssemblerLabel catch77_skip(&ca_);
      ca_.Goto(&catch77_skip);
      ca_.Bind(&catch77__label, &tmp78);
      ca_.Goto(&block31);
      ca_.Bind(&catch77_skip);
    }
    tmp79 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp76});
    compiler::CodeAssemblerExceptionHandlerLabel catch80__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch80__label);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp79});
    }
    if (catch80__label.is_used()) {
      compiler::CodeAssemblerLabel catch80_skip(&ca_);
      ca_.Goto(&catch80_skip);
      ca_.Bind(&catch80__label, &tmp81);
      ca_.Goto(&block32);
      ca_.Bind(&catch80_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch82__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch82__label);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb20_6, phi_bb20_7);
    CodeStubAssembler(state_).Unreachable();
    }
    if (catch82__label.is_used()) {
      compiler::CodeAssemblerLabel catch82_skip(&ca_);
      ca_.Bind(&catch82__label, &tmp83);
      ca_.Goto(&block33);
    }
  }

  TNode<Union<JSMessageObject, TheHole>> tmp84;
  if (block31.is_used()) {
    ca_.Bind(&block31);
    tmp84 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp78, tmp84);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp85;
  if (block32.is_used()) {
    ca_.Bind(&block32);
    tmp85 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp81, tmp85);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp86;
  if (block33.is_used()) {
    ca_.Bind(&block33);
    tmp86 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp83, tmp86);
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb2_3;
  TNode<Union<JSMessageObject, TheHole>> phi_bb2_4;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_3, &phi_bb2_4);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb2_3, phi_bb2_4);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=427&c=1
TNode<JSIteratorTakeHelper> NewJSIteratorTakeHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Number> p_limit) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Map> tmp3;
  TNode<FixedArray> tmp4;
  TNode<FixedArray> tmp5;
  TNode<Uint32T> tmp6;
  TNode<Smi> tmp7;
  TNode<BoolT> tmp8;
  TNode<BoolT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<HeapObject> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<JSIteratorTakeHelper> tmp20;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ITERATOR_TAKE_HELPER_MAP_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Map_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = kEmptyFixedArray_0(state_);
    tmp5 = kEmptyFixedArray_0(state_);
    tmp6 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedStart_0(state_, JSIteratorHelperState::kSuspendedStart);
    tmp7 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp6});
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp9 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    tmp11 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp10}, TNode<Map>{tmp3}, TNode<BoolT>{tmp8}, TNode<BoolT>{tmp9});
    tmp12 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp11, tmp12}, tmp3);
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>>(CodeStubAssembler::Reference{tmp11, tmp13}, tmp4);
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp11, tmp14}, tmp5);
    tmp15 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp11, tmp15}, tmp7);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp11, tmp16}, p_underlying.object);
    tmp17 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp16}, TNode<IntPtrT>{tmp17});
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{tmp11, tmp18}, p_underlying.next);
    tmp19 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{tmp11, tmp19}, p_limit);
    tmp20 = TORQUE_CAST(TNode<HeapObject>{tmp11});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSIteratorTakeHelper>{tmp20};
}

TF_BUILTIN(IteratorPrototypeTake, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kLimit);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.take");
  }

  TNode<Number> tmp2;
      TNode<JSAny> tmp4;
  TNode<BoolT> tmp5;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerExceptionHandlerLabel catch3__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch3__label);
    tmp2 = CodeStubAssembler(state_).ToNumber_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter2});
    }
    if (catch3__label.is_used()) {
      compiler::CodeAssemblerLabel catch3_skip(&ca_);
      ca_.Goto(&catch3_skip);
      ca_.Bind(&catch3__label, &tmp4);
      ca_.Goto(&block9);
      ca_.Bind(&catch3_skip);
    }
    tmp5 = NumberIsNaN_0(state_, TNode<Number>{tmp2});
    ca_.Branch(tmp5, &block10, std::vector<compiler::Node*>{}, &block11, std::vector<compiler::Node*>{});
  }

  TNode<Union<JSMessageObject, TheHole>> tmp6;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp6 = GetAndResetPendingMessage_0(state_);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp4, tmp6);
    CodeStubAssembler(state_).Unreachable();
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    ca_.Goto(&block2);
  }

  TNode<Number> tmp7;
  TNode<Number> tmp8;
  TNode<BoolT> tmp9;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp7 = ToInteger_Inline_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{tmp2});
    tmp8 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp9 = NumberIsLessThan_0(state_, TNode<Number>{tmp7}, TNode<Number>{tmp8});
    ca_.Branch(tmp9, &block12, std::vector<compiler::Node*>{}, &block13, std::vector<compiler::Node*>{});
  }

  if (block12.is_used()) {
    ca_.Bind(&block12);
    ca_.Goto(&block2);
  }

  TNode<JSReceiver> tmp10;
  TNode<JSAny> tmp11;
  TNode<JSIteratorTakeHelper> tmp12;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    std::tie(tmp10, tmp11) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp12 = NewJSIteratorTakeHelper_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp10}, TNode<JSAny>{tmp11}}, TNode<Number>{tmp7});
    CodeStubAssembler(state_).Return(tmp12);
  }

  if (block2.is_used()) {
    ca_.Bind(&block2);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).ThrowRangeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kMustBePositive), TNode<Object>{parameter2});
  }
}

TF_BUILTIN(IteratorTakeHelperNext, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSIteratorTakeHelper> parameter1 = UncheckedParameter<JSIteratorTakeHelper>(Descriptor::kHelper);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block28(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block31(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block32(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Map> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<Number> tmp2;
  TNode<Number> tmp3;
      TNode<JSAny> tmp5;
  TNode<BoolT> tmp6;
      TNode<JSAny> tmp8;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    MarkIteratorHelperAsExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    tmp1 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    tmp2 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{parameter1, tmp1});
    compiler::CodeAssemblerExceptionHandlerLabel catch4__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch4__label);
    tmp3 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    }
    if (catch4__label.is_used()) {
      compiler::CodeAssemblerLabel catch4_skip(&ca_);
      ca_.Goto(&catch4_skip);
      ca_.Bind(&catch4__label, &tmp5);
      ca_.Goto(&block5);
      ca_.Bind(&catch4_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch7__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch7__label);
    tmp6 = IsNumberEqual_0(state_, TNode<Number>{tmp2}, TNode<Number>{tmp3});
    }
    if (catch7__label.is_used()) {
      compiler::CodeAssemblerLabel catch7_skip(&ca_);
      ca_.Goto(&catch7_skip);
      ca_.Bind(&catch7__label, &tmp8);
      ca_.Goto(&block6);
      ca_.Bind(&catch7_skip);
    }
    ca_.Branch(tmp6, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  TNode<Union<JSMessageObject, TheHole>> tmp9;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp9 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp5, tmp9);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp10;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp10 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp8, tmp10);
  }

      TNode<JSAny> tmp12;
  TNode<IntPtrT> tmp13;
      TNode<JSAny> tmp15;
  TNode<JSReceiver> tmp16;
  TNode<IntPtrT> tmp17;
      TNode<JSAny> tmp19;
  TNode<IntPtrT> tmp20;
      TNode<JSAny> tmp22;
  TNode<JSAny> tmp23;
      TNode<JSAny> tmp25;
  TNode<Undefined> tmp26;
  TNode<True> tmp27;
  TNode<JSObject> tmp28;
      TNode<JSAny> tmp30;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    compiler::CodeAssemblerExceptionHandlerLabel catch11__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch11__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch11__label.is_used()) {
      compiler::CodeAssemblerLabel catch11_skip(&ca_);
      ca_.Goto(&catch11_skip);
      ca_.Bind(&catch11__label, &tmp12);
      ca_.Goto(&block7);
      ca_.Bind(&catch11_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch14__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch14__label);
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch14__label.is_used()) {
      compiler::CodeAssemblerLabel catch14_skip(&ca_);
      ca_.Goto(&catch14_skip);
      ca_.Bind(&catch14__label, &tmp15);
      ca_.Goto(&block8);
      ca_.Bind(&catch14_skip);
    }
    tmp16 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp13});
    compiler::CodeAssemblerExceptionHandlerLabel catch18__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch18__label);
    tmp17 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch18__label.is_used()) {
      compiler::CodeAssemblerLabel catch18_skip(&ca_);
      ca_.Goto(&catch18_skip);
      ca_.Bind(&catch18__label, &tmp19);
      ca_.Goto(&block9);
      ca_.Bind(&catch18_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch21__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch21__label);
    tmp20 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp13}, TNode<IntPtrT>{tmp17});
    }
    if (catch21__label.is_used()) {
      compiler::CodeAssemblerLabel catch21_skip(&ca_);
      ca_.Goto(&catch21_skip);
      ca_.Bind(&catch21__label, &tmp22);
      ca_.Goto(&block10);
      ca_.Bind(&catch21_skip);
    }
    tmp23 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp20});
    compiler::CodeAssemblerExceptionHandlerLabel catch24__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch24__label);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp16}, TNode<JSAny>{tmp23}});
    }
    if (catch24__label.is_used()) {
      compiler::CodeAssemblerLabel catch24_skip(&ca_);
      ca_.Goto(&catch24_skip);
      ca_.Bind(&catch24__label, &tmp25);
      ca_.Goto(&block11);
      ca_.Bind(&catch24_skip);
    }
    tmp26 = Undefined_0(state_);
    tmp27 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch29__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch29__label);
    tmp28 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp26}, TNode<Boolean>{tmp27});
    }
    if (catch29__label.is_used()) {
      compiler::CodeAssemblerLabel catch29_skip(&ca_);
      ca_.Goto(&catch29_skip);
      ca_.Bind(&catch29__label, &tmp30);
      ca_.Goto(&block12);
      ca_.Bind(&catch29_skip);
    }
    CodeStubAssembler(state_).Return(tmp28);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp31;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp31 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp12, tmp31);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp32;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    tmp32 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp15, tmp32);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp33;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp33 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp19, tmp33);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp34;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp34 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp22, tmp34);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp35;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp35 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp25, tmp35);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp36;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp36 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp30, tmp36);
  }

  TNode<BoolT> tmp37;
      TNode<JSAny> tmp39;
  TNode<BoolT> tmp40;
      TNode<JSAny> tmp42;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    compiler::CodeAssemblerExceptionHandlerLabel catch38__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch38__label);
    tmp37 = NumberIsSomeInfinity_0(state_, TNode<Number>{tmp2});
    }
    if (catch38__label.is_used()) {
      compiler::CodeAssemblerLabel catch38_skip(&ca_);
      ca_.Goto(&catch38_skip);
      ca_.Bind(&catch38__label, &tmp39);
      ca_.Goto(&block15);
      ca_.Bind(&catch38_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch41__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch41__label);
    tmp40 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp37});
    }
    if (catch41__label.is_used()) {
      compiler::CodeAssemblerLabel catch41_skip(&ca_);
      ca_.Goto(&catch41_skip);
      ca_.Bind(&catch41__label, &tmp42);
      ca_.Goto(&block16);
      ca_.Bind(&catch41_skip);
    }
    ca_.Branch(tmp40, &block13, std::vector<compiler::Node*>{}, &block14, std::vector<compiler::Node*>{});
  }

  TNode<Union<JSMessageObject, TheHole>> tmp43;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp43 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp39, tmp43);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp44;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp44 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp42, tmp44);
  }

  TNode<IntPtrT> tmp45;
      TNode<JSAny> tmp47;
  TNode<Number> tmp48;
      TNode<JSAny> tmp50;
  TNode<Number> tmp51;
      TNode<JSAny> tmp53;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    compiler::CodeAssemblerExceptionHandlerLabel catch46__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch46__label);
    tmp45 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch46__label.is_used()) {
      compiler::CodeAssemblerLabel catch46_skip(&ca_);
      ca_.Goto(&catch46_skip);
      ca_.Bind(&catch46__label, &tmp47);
      ca_.Goto(&block17);
      ca_.Bind(&catch46_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch49__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch49__label);
    tmp48 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch49__label.is_used()) {
      compiler::CodeAssemblerLabel catch49_skip(&ca_);
      ca_.Goto(&catch49_skip);
      ca_.Bind(&catch49__label, &tmp50);
      ca_.Goto(&block18);
      ca_.Bind(&catch49_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch52__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch52__label);
    tmp51 = CodeStubAssembler(state_).NumberSub(TNode<Number>{tmp2}, TNode<Number>{tmp48});
    }
    if (catch52__label.is_used()) {
      compiler::CodeAssemblerLabel catch52_skip(&ca_);
      ca_.Goto(&catch52_skip);
      ca_.Bind(&catch52__label, &tmp53);
      ca_.Goto(&block19);
      ca_.Bind(&catch52_skip);
    }
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{parameter1, tmp45}, tmp51);
    ca_.Goto(&block14);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp54;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp54 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp47, tmp54);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp55;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp55 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp50, tmp55);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp56;
  if (block19.is_used()) {
    ca_.Bind(&block19);
    tmp56 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp53, tmp56);
  }

  TNode<IntPtrT> tmp57;
      TNode<JSAny> tmp59;
  TNode<JSReceiver> tmp60;
  TNode<IntPtrT> tmp61;
      TNode<JSAny> tmp63;
  TNode<IntPtrT> tmp64;
      TNode<JSAny> tmp66;
  TNode<JSAny> tmp67;
  TNode<JSReceiver> tmp68;
      TNode<JSAny> tmp71;
  if (block14.is_used()) {
    ca_.Bind(&block14);
    compiler::CodeAssemblerExceptionHandlerLabel catch58__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch58__label);
    tmp57 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch58__label.is_used()) {
      compiler::CodeAssemblerLabel catch58_skip(&ca_);
      ca_.Goto(&catch58_skip);
      ca_.Bind(&catch58__label, &tmp59);
      ca_.Goto(&block22);
      ca_.Bind(&catch58_skip);
    }
    tmp60 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp57});
    compiler::CodeAssemblerExceptionHandlerLabel catch62__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch62__label);
    tmp61 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch62__label.is_used()) {
      compiler::CodeAssemblerLabel catch62_skip(&ca_);
      ca_.Goto(&catch62_skip);
      ca_.Bind(&catch62__label, &tmp63);
      ca_.Goto(&block23);
      ca_.Bind(&catch62_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch65__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch65__label);
    tmp64 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp57}, TNode<IntPtrT>{tmp61});
    }
    if (catch65__label.is_used()) {
      compiler::CodeAssemblerLabel catch65_skip(&ca_);
      ca_.Goto(&catch65_skip);
      ca_.Bind(&catch65__label, &tmp66);
      ca_.Goto(&block24);
      ca_.Bind(&catch65_skip);
    }
    tmp67 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp64});
    compiler::CodeAssemblerLabel label69(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch70__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch70__label);
    tmp68 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp60}, TNode<JSAny>{tmp67}}, TNode<Map>{tmp0}, &label69);
    }
    if (catch70__label.is_used()) {
      compiler::CodeAssemblerLabel catch70_skip(&ca_);
      ca_.Goto(&catch70_skip);
      ca_.Bind(&catch70__label, &tmp71);
      ca_.Goto(&block27);
      ca_.Bind(&catch70_skip);
    }
    ca_.Goto(&block25);
    if (label69.is_used()) {
      ca_.Bind(&label69);
      ca_.Goto(&block26);
    }
  }

  TNode<Union<JSMessageObject, TheHole>> tmp72;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    tmp72 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp59, tmp72);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp73;
  if (block23.is_used()) {
    ca_.Bind(&block23);
    tmp73 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp63, tmp73);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp74;
  if (block24.is_used()) {
    ca_.Bind(&block24);
    tmp74 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp66, tmp74);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp75;
  if (block27.is_used()) {
    ca_.Bind(&block27);
    tmp75 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp71, tmp75);
  }

      TNode<JSAny> tmp77;
  TNode<Undefined> tmp78;
  TNode<True> tmp79;
  TNode<JSObject> tmp80;
      TNode<JSAny> tmp82;
  if (block26.is_used()) {
    ca_.Bind(&block26);
    compiler::CodeAssemblerExceptionHandlerLabel catch76__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch76__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch76__label.is_used()) {
      compiler::CodeAssemblerLabel catch76_skip(&ca_);
      ca_.Goto(&catch76_skip);
      ca_.Bind(&catch76__label, &tmp77);
      ca_.Goto(&block28);
      ca_.Bind(&catch76_skip);
    }
    tmp78 = Undefined_0(state_);
    tmp79 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch81__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch81__label);
    tmp80 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp78}, TNode<Boolean>{tmp79});
    }
    if (catch81__label.is_used()) {
      compiler::CodeAssemblerLabel catch81_skip(&ca_);
      ca_.Goto(&catch81_skip);
      ca_.Bind(&catch81__label, &tmp82);
      ca_.Goto(&block29);
      ca_.Bind(&catch81_skip);
    }
    CodeStubAssembler(state_).Return(tmp80);
  }

  TNode<JSAny> tmp83;
      TNode<JSAny> tmp85;
      TNode<JSAny> tmp87;
  TNode<False> tmp88;
  TNode<JSObject> tmp89;
      TNode<JSAny> tmp91;
  if (block25.is_used()) {
    ca_.Bind(&block25);
    compiler::CodeAssemblerExceptionHandlerLabel catch84__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch84__label);
    tmp83 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp68}, TNode<Map>{tmp0});
    }
    if (catch84__label.is_used()) {
      compiler::CodeAssemblerLabel catch84_skip(&ca_);
      ca_.Goto(&catch84_skip);
      ca_.Bind(&catch84__label, &tmp85);
      ca_.Goto(&block30);
      ca_.Bind(&catch84_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch86__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch86__label);
    MarkIteratorHelperAsFinishedExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch86__label.is_used()) {
      compiler::CodeAssemblerLabel catch86_skip(&ca_);
      ca_.Goto(&catch86_skip);
      ca_.Bind(&catch86__label, &tmp87);
      ca_.Goto(&block31);
      ca_.Bind(&catch86_skip);
    }
    tmp88 = False_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch90__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch90__label);
    tmp89 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp83}, TNode<Boolean>{tmp88});
    }
    if (catch90__label.is_used()) {
      compiler::CodeAssemblerLabel catch90_skip(&ca_);
      ca_.Goto(&catch90_skip);
      ca_.Bind(&catch90__label, &tmp91);
      ca_.Goto(&block32);
      ca_.Bind(&catch90_skip);
    }
    CodeStubAssembler(state_).Return(tmp89);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp92;
  if (block28.is_used()) {
    ca_.Bind(&block28);
    tmp92 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp77, tmp92);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp93;
  if (block29.is_used()) {
    ca_.Bind(&block29);
    tmp93 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp82, tmp93);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp94;
  if (block30.is_used()) {
    ca_.Bind(&block30);
    tmp94 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp85, tmp94);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp95;
  if (block31.is_used()) {
    ca_.Bind(&block31);
    tmp95 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp87, tmp95);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp96;
  if (block32.is_used()) {
    ca_.Bind(&block32);
    tmp96 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp91, tmp96);
  }

  TNode<JSAny> phi_bb2_4;
  TNode<Union<JSMessageObject, TheHole>> phi_bb2_5;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_4, &phi_bb2_5);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb2_4, phi_bb2_5);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=548&c=1
TNode<JSIteratorDropHelper> NewJSIteratorDropHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Number> p_limit) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Map> tmp3;
  TNode<FixedArray> tmp4;
  TNode<FixedArray> tmp5;
  TNode<Uint32T> tmp6;
  TNode<Smi> tmp7;
  TNode<BoolT> tmp8;
  TNode<BoolT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<HeapObject> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<JSIteratorDropHelper> tmp20;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ITERATOR_DROP_HELPER_MAP_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Map_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = kEmptyFixedArray_0(state_);
    tmp5 = kEmptyFixedArray_0(state_);
    tmp6 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedStart_0(state_, JSIteratorHelperState::kSuspendedStart);
    tmp7 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp6});
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp9 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    tmp11 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp10}, TNode<Map>{tmp3}, TNode<BoolT>{tmp8}, TNode<BoolT>{tmp9});
    tmp12 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp11, tmp12}, tmp3);
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>>(CodeStubAssembler::Reference{tmp11, tmp13}, tmp4);
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp11, tmp14}, tmp5);
    tmp15 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp11, tmp15}, tmp7);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp11, tmp16}, p_underlying.object);
    tmp17 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp16}, TNode<IntPtrT>{tmp17});
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{tmp11, tmp18}, p_underlying.next);
    tmp19 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{tmp11, tmp19}, p_limit);
    tmp20 = TORQUE_CAST(TNode<HeapObject>{tmp11});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSIteratorDropHelper>{tmp20};
}

TF_BUILTIN(IteratorPrototypeDrop, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kLimit);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.drop");
  }

  TNode<Number> tmp2;
      TNode<JSAny> tmp4;
  TNode<BoolT> tmp5;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerExceptionHandlerLabel catch3__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch3__label);
    tmp2 = CodeStubAssembler(state_).ToNumber_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter2});
    }
    if (catch3__label.is_used()) {
      compiler::CodeAssemblerLabel catch3_skip(&ca_);
      ca_.Goto(&catch3_skip);
      ca_.Bind(&catch3__label, &tmp4);
      ca_.Goto(&block9);
      ca_.Bind(&catch3_skip);
    }
    tmp5 = NumberIsNaN_0(state_, TNode<Number>{tmp2});
    ca_.Branch(tmp5, &block10, std::vector<compiler::Node*>{}, &block11, std::vector<compiler::Node*>{});
  }

  TNode<Union<JSMessageObject, TheHole>> tmp6;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp6 = GetAndResetPendingMessage_0(state_);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp4, tmp6);
    CodeStubAssembler(state_).Unreachable();
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    ca_.Goto(&block2);
  }

  TNode<Number> tmp7;
  TNode<Number> tmp8;
  TNode<BoolT> tmp9;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp7 = ToInteger_Inline_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{tmp2});
    tmp8 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp9 = NumberIsLessThan_0(state_, TNode<Number>{tmp7}, TNode<Number>{tmp8});
    ca_.Branch(tmp9, &block12, std::vector<compiler::Node*>{}, &block13, std::vector<compiler::Node*>{});
  }

  if (block12.is_used()) {
    ca_.Bind(&block12);
    ca_.Goto(&block2);
  }

  TNode<JSReceiver> tmp10;
  TNode<JSAny> tmp11;
  TNode<JSIteratorDropHelper> tmp12;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    std::tie(tmp10, tmp11) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp12 = NewJSIteratorDropHelper_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp10}, TNode<JSAny>{tmp11}}, TNode<Number>{tmp7});
    CodeStubAssembler(state_).Return(tmp12);
  }

  if (block2.is_used()) {
    ca_.Bind(&block2);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).ThrowRangeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kMustBePositive), TNode<Object>{parameter2});
  }
}

TF_BUILTIN(IteratorDropHelperNext, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSIteratorDropHelper> parameter1 = UncheckedParameter<JSIteratorDropHelper>(Descriptor::kHelper);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number, Number> block8(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block9(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block12(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block13(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number, Number> block14(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block15(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block23(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block24(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block25(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block28(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block31(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block32(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block33(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, JSAny, Union<JSMessageObject, TheHole>> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Map> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<Number> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    MarkIteratorHelperAsExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    tmp1 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    tmp2 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{parameter1, tmp1});
    ca_.Goto(&block7, tmp2);
  }

  TNode<Number> phi_bb7_3;
  TNode<Number> tmp3;
      TNode<JSAny> tmp5;
  TNode<BoolT> tmp6;
      TNode<JSAny> tmp8;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch4__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch4__label);
    tmp3 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    }
    if (catch4__label.is_used()) {
      compiler::CodeAssemblerLabel catch4_skip(&ca_);
      ca_.Goto(&catch4_skip);
      ca_.Bind(&catch4__label, &tmp5);
      ca_.Goto(&block8, phi_bb7_3, phi_bb7_3, phi_bb7_3);
      ca_.Bind(&catch4_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch7__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch7__label);
    tmp6 = NumberIsGreaterThan_0(state_, TNode<Number>{phi_bb7_3}, TNode<Number>{tmp3});
    }
    if (catch7__label.is_used()) {
      compiler::CodeAssemblerLabel catch7_skip(&ca_);
      ca_.Goto(&catch7_skip);
      ca_.Bind(&catch7__label, &tmp8);
      ca_.Goto(&block9, phi_bb7_3, phi_bb7_3);
      ca_.Bind(&catch7_skip);
    }
    ca_.Branch(tmp6, &block5, std::vector<compiler::Node*>{phi_bb7_3}, &block6, std::vector<compiler::Node*>{phi_bb7_3});
  }

  TNode<Number> phi_bb8_3;
  TNode<Number> phi_bb8_5;
  TNode<Number> phi_bb8_6;
  TNode<Union<JSMessageObject, TheHole>> tmp9;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_3, &phi_bb8_5, &phi_bb8_6);
    tmp9 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb8_3, tmp5, tmp9);
  }

  TNode<Number> phi_bb9_3;
  TNode<Number> phi_bb9_5;
  TNode<Union<JSMessageObject, TheHole>> tmp10;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_3, &phi_bb9_5);
    tmp10 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb9_3, tmp8, tmp10);
  }

  TNode<Number> phi_bb5_3;
  TNode<BoolT> tmp11;
      TNode<JSAny> tmp13;
  TNode<BoolT> tmp14;
      TNode<JSAny> tmp16;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch12__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch12__label);
    tmp11 = NumberIsSomeInfinity_0(state_, TNode<Number>{phi_bb5_3});
    }
    if (catch12__label.is_used()) {
      compiler::CodeAssemblerLabel catch12_skip(&ca_);
      ca_.Goto(&catch12_skip);
      ca_.Bind(&catch12__label, &tmp13);
      ca_.Goto(&block12, phi_bb5_3, phi_bb5_3);
      ca_.Bind(&catch12_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch15__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch15__label);
    tmp14 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp11});
    }
    if (catch15__label.is_used()) {
      compiler::CodeAssemblerLabel catch15_skip(&ca_);
      ca_.Goto(&catch15_skip);
      ca_.Bind(&catch15__label, &tmp16);
      ca_.Goto(&block13, phi_bb5_3);
      ca_.Bind(&catch15_skip);
    }
    ca_.Branch(tmp14, &block10, std::vector<compiler::Node*>{phi_bb5_3}, &block11, std::vector<compiler::Node*>{phi_bb5_3});
  }

  TNode<Number> phi_bb12_3;
  TNode<Number> phi_bb12_5;
  TNode<Union<JSMessageObject, TheHole>> tmp17;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_3, &phi_bb12_5);
    tmp17 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb12_3, tmp13, tmp17);
  }

  TNode<Number> phi_bb13_3;
  TNode<Union<JSMessageObject, TheHole>> tmp18;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_3);
    tmp18 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb13_3, tmp16, tmp18);
  }

  TNode<Number> phi_bb10_3;
  TNode<Number> tmp19;
      TNode<JSAny> tmp21;
  TNode<Number> tmp22;
      TNode<JSAny> tmp24;
  TNode<IntPtrT> tmp25;
      TNode<JSAny> tmp27;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch20__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch20__label);
    tmp19 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch20__label.is_used()) {
      compiler::CodeAssemblerLabel catch20_skip(&ca_);
      ca_.Goto(&catch20_skip);
      ca_.Bind(&catch20__label, &tmp21);
      ca_.Goto(&block14, phi_bb10_3, phi_bb10_3, phi_bb10_3);
      ca_.Bind(&catch20_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch23__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch23__label);
    tmp22 = CodeStubAssembler(state_).NumberSub(TNode<Number>{phi_bb10_3}, TNode<Number>{tmp19});
    }
    if (catch23__label.is_used()) {
      compiler::CodeAssemblerLabel catch23_skip(&ca_);
      ca_.Goto(&catch23_skip);
      ca_.Bind(&catch23__label, &tmp24);
      ca_.Goto(&block15, phi_bb10_3, phi_bb10_3);
      ca_.Bind(&catch23_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch26__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch26__label);
    tmp25 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch26__label.is_used()) {
      compiler::CodeAssemblerLabel catch26_skip(&ca_);
      ca_.Goto(&catch26_skip);
      ca_.Bind(&catch26__label, &tmp27);
      ca_.Goto(&block16);
      ca_.Bind(&catch26_skip);
    }
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{parameter1, tmp25}, tmp22);
    ca_.Goto(&block11, tmp22);
  }

  TNode<Number> phi_bb14_3;
  TNode<Number> phi_bb14_5;
  TNode<Number> phi_bb14_6;
  TNode<Union<JSMessageObject, TheHole>> tmp28;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_3, &phi_bb14_5, &phi_bb14_6);
    tmp28 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb14_3, tmp21, tmp28);
  }

  TNode<Number> phi_bb15_3;
  TNode<Number> phi_bb15_5;
  TNode<Union<JSMessageObject, TheHole>> tmp29;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_3, &phi_bb15_5);
    tmp29 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb15_3, tmp24, tmp29);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp30;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp30 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp22, tmp27, tmp30);
  }

  TNode<Number> phi_bb11_3;
  TNode<IntPtrT> tmp31;
      TNode<JSAny> tmp33;
  TNode<JSReceiver> tmp34;
  TNode<IntPtrT> tmp35;
      TNode<JSAny> tmp37;
  TNode<IntPtrT> tmp38;
      TNode<JSAny> tmp40;
  TNode<JSAny> tmp41;
  TNode<JSReceiver> tmp42;
      TNode<JSAny> tmp45;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch32__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch32__label);
    tmp31 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch32__label.is_used()) {
      compiler::CodeAssemblerLabel catch32_skip(&ca_);
      ca_.Goto(&catch32_skip);
      ca_.Bind(&catch32__label, &tmp33);
      ca_.Goto(&block17);
      ca_.Bind(&catch32_skip);
    }
    tmp34 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp31});
    compiler::CodeAssemblerExceptionHandlerLabel catch36__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch36__label);
    tmp35 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch36__label.is_used()) {
      compiler::CodeAssemblerLabel catch36_skip(&ca_);
      ca_.Goto(&catch36_skip);
      ca_.Bind(&catch36__label, &tmp37);
      ca_.Goto(&block18);
      ca_.Bind(&catch36_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch39__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch39__label);
    tmp38 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp31}, TNode<IntPtrT>{tmp35});
    }
    if (catch39__label.is_used()) {
      compiler::CodeAssemblerLabel catch39_skip(&ca_);
      ca_.Goto(&catch39_skip);
      ca_.Bind(&catch39__label, &tmp40);
      ca_.Goto(&block19);
      ca_.Bind(&catch39_skip);
    }
    tmp41 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp38});
    compiler::CodeAssemblerLabel label43(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch44__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch44__label);
    tmp42 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp34}, TNode<JSAny>{tmp41}}, TNode<Map>{tmp0}, &label43);
    }
    if (catch44__label.is_used()) {
      compiler::CodeAssemblerLabel catch44_skip(&ca_);
      ca_.Goto(&catch44_skip);
      ca_.Bind(&catch44__label, &tmp45);
      ca_.Goto(&block22);
      ca_.Bind(&catch44_skip);
    }
    ca_.Goto(&block20);
    if (label43.is_used()) {
      ca_.Bind(&label43);
      ca_.Goto(&block21);
    }
  }

  TNode<Union<JSMessageObject, TheHole>> tmp46;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp46 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb11_3, tmp33, tmp46);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp47;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp47 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb11_3, tmp37, tmp47);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp48;
  if (block19.is_used()) {
    ca_.Bind(&block19);
    tmp48 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb11_3, tmp40, tmp48);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp49;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    tmp49 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb11_3, tmp45, tmp49);
  }

  if (block21.is_used()) {
    ca_.Bind(&block21);
    ca_.Goto(&block4, phi_bb11_3);
  }

  if (block20.is_used()) {
    ca_.Bind(&block20);
    ca_.Goto(&block7, phi_bb11_3);
  }

  TNode<Number> phi_bb6_3;
  TNode<IntPtrT> tmp50;
      TNode<JSAny> tmp52;
  TNode<JSReceiver> tmp53;
  TNode<IntPtrT> tmp54;
      TNode<JSAny> tmp56;
  TNode<IntPtrT> tmp57;
      TNode<JSAny> tmp59;
  TNode<JSAny> tmp60;
  TNode<JSReceiver> tmp61;
      TNode<JSAny> tmp64;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch51__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch51__label);
    tmp50 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch51__label.is_used()) {
      compiler::CodeAssemblerLabel catch51_skip(&ca_);
      ca_.Goto(&catch51_skip);
      ca_.Bind(&catch51__label, &tmp52);
      ca_.Goto(&block23, phi_bb6_3);
      ca_.Bind(&catch51_skip);
    }
    tmp53 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp50});
    compiler::CodeAssemblerExceptionHandlerLabel catch55__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch55__label);
    tmp54 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch55__label.is_used()) {
      compiler::CodeAssemblerLabel catch55_skip(&ca_);
      ca_.Goto(&catch55_skip);
      ca_.Bind(&catch55__label, &tmp56);
      ca_.Goto(&block24, phi_bb6_3);
      ca_.Bind(&catch55_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch58__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch58__label);
    tmp57 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp50}, TNode<IntPtrT>{tmp54});
    }
    if (catch58__label.is_used()) {
      compiler::CodeAssemblerLabel catch58_skip(&ca_);
      ca_.Goto(&catch58_skip);
      ca_.Bind(&catch58__label, &tmp59);
      ca_.Goto(&block25, phi_bb6_3);
      ca_.Bind(&catch58_skip);
    }
    tmp60 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp57});
    compiler::CodeAssemblerLabel label62(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch63__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch63__label);
    tmp61 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp53}, TNode<JSAny>{tmp60}}, TNode<Map>{tmp0}, &label62);
    }
    if (catch63__label.is_used()) {
      compiler::CodeAssemblerLabel catch63_skip(&ca_);
      ca_.Goto(&catch63_skip);
      ca_.Bind(&catch63__label, &tmp64);
      ca_.Goto(&block28, phi_bb6_3);
      ca_.Bind(&catch63_skip);
    }
    ca_.Goto(&block26, phi_bb6_3);
    if (label62.is_used()) {
      ca_.Bind(&label62);
      ca_.Goto(&block27, phi_bb6_3);
    }
  }

  TNode<Number> phi_bb23_3;
  TNode<Union<JSMessageObject, TheHole>> tmp65;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_3);
    tmp65 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb23_3, tmp52, tmp65);
  }

  TNode<Number> phi_bb24_3;
  TNode<Union<JSMessageObject, TheHole>> tmp66;
  if (block24.is_used()) {
    ca_.Bind(&block24, &phi_bb24_3);
    tmp66 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb24_3, tmp56, tmp66);
  }

  TNode<Number> phi_bb25_3;
  TNode<Union<JSMessageObject, TheHole>> tmp67;
  if (block25.is_used()) {
    ca_.Bind(&block25, &phi_bb25_3);
    tmp67 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb25_3, tmp59, tmp67);
  }

  TNode<Number> phi_bb28_3;
  TNode<Union<JSMessageObject, TheHole>> tmp68;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_3);
    tmp68 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb28_3, tmp64, tmp68);
  }

  TNode<Number> phi_bb27_3;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_3);
    ca_.Goto(&block4, phi_bb27_3);
  }

  TNode<Number> phi_bb26_3;
  TNode<JSAny> tmp69;
      TNode<JSAny> tmp71;
      TNode<JSAny> tmp73;
  TNode<False> tmp74;
  TNode<JSObject> tmp75;
      TNode<JSAny> tmp77;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch70__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch70__label);
    tmp69 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp61}, TNode<Map>{tmp0});
    }
    if (catch70__label.is_used()) {
      compiler::CodeAssemblerLabel catch70_skip(&ca_);
      ca_.Goto(&catch70_skip);
      ca_.Bind(&catch70__label, &tmp71);
      ca_.Goto(&block31, phi_bb26_3);
      ca_.Bind(&catch70_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch72__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch72__label);
    MarkIteratorHelperAsFinishedExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch72__label.is_used()) {
      compiler::CodeAssemblerLabel catch72_skip(&ca_);
      ca_.Goto(&catch72_skip);
      ca_.Bind(&catch72__label, &tmp73);
      ca_.Goto(&block32, phi_bb26_3);
      ca_.Bind(&catch72_skip);
    }
    tmp74 = False_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch76__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch76__label);
    tmp75 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp69}, TNode<Boolean>{tmp74});
    }
    if (catch76__label.is_used()) {
      compiler::CodeAssemblerLabel catch76_skip(&ca_);
      ca_.Goto(&catch76_skip);
      ca_.Bind(&catch76__label, &tmp77);
      ca_.Goto(&block33, phi_bb26_3);
      ca_.Bind(&catch76_skip);
    }
    CodeStubAssembler(state_).Return(tmp75);
  }

  TNode<Number> phi_bb4_3;
      TNode<JSAny> tmp79;
  TNode<Undefined> tmp80;
  TNode<True> tmp81;
  TNode<JSObject> tmp82;
      TNode<JSAny> tmp84;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch78__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch78__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch78__label.is_used()) {
      compiler::CodeAssemblerLabel catch78_skip(&ca_);
      ca_.Goto(&catch78_skip);
      ca_.Bind(&catch78__label, &tmp79);
      ca_.Goto(&block29);
      ca_.Bind(&catch78_skip);
    }
    tmp80 = Undefined_0(state_);
    tmp81 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch83__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch83__label);
    tmp82 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp80}, TNode<Boolean>{tmp81});
    }
    if (catch83__label.is_used()) {
      compiler::CodeAssemblerLabel catch83_skip(&ca_);
      ca_.Goto(&catch83_skip);
      ca_.Bind(&catch83__label, &tmp84);
      ca_.Goto(&block30);
      ca_.Bind(&catch83_skip);
    }
    CodeStubAssembler(state_).Return(tmp82);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp85;
  if (block29.is_used()) {
    ca_.Bind(&block29);
    tmp85 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb4_3, tmp79, tmp85);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp86;
  if (block30.is_used()) {
    ca_.Bind(&block30);
    tmp86 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb4_3, tmp84, tmp86);
  }

  TNode<Number> phi_bb31_3;
  TNode<Union<JSMessageObject, TheHole>> tmp87;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_3);
    tmp87 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb31_3, tmp71, tmp87);
  }

  TNode<Number> phi_bb32_3;
  TNode<Union<JSMessageObject, TheHole>> tmp88;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_3);
    tmp88 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb32_3, tmp73, tmp88);
  }

  TNode<Number> phi_bb33_3;
  TNode<Union<JSMessageObject, TheHole>> tmp89;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_3);
    tmp89 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb33_3, tmp77, tmp89);
  }

  TNode<Number> phi_bb2_3;
  TNode<JSAny> phi_bb2_4;
  TNode<Union<JSMessageObject, TheHole>> phi_bb2_5;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_3, &phi_bb2_4, &phi_bb2_5);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb2_4, phi_bb2_5);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=669&c=1
const char* kFlatMapMethodName_0(compiler::CodeAssemblerState* state_) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

    ca_.Bind(&block0);
  return "Iterator.prototype.flatMap";}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=671&c=1
TNode<JSIteratorFlatMapHelper> NewJSIteratorFlatMapHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructIteratorRecord p_underlying, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_mapper) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Map> tmp3;
  TNode<FixedArray> tmp4;
  TNode<FixedArray> tmp5;
  TNode<Uint32T> tmp6;
  TNode<Smi> tmp7;
  TNode<BoolT> tmp8;
  TNode<BoolT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<HeapObject> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<IntPtrT> tmp20;
  TNode<Number> tmp21;
  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<IntPtrT> tmp24;
  TNode<JSIteratorFlatMapHelper> tmp25;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ITERATOR_FLAT_MAP_HELPER_MAP_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Map_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = kEmptyFixedArray_0(state_);
    tmp5 = kEmptyFixedArray_0(state_);
    tmp6 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedStart_0(state_, JSIteratorHelperState::kSuspendedStart);
    tmp7 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp6});
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp9 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, 40);
    tmp11 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp10}, TNode<Map>{tmp3}, TNode<BoolT>{tmp8}, TNode<BoolT>{tmp9});
    tmp12 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp11, tmp12}, tmp3);
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>>(CodeStubAssembler::Reference{tmp11, tmp13}, tmp4);
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp11, tmp14}, tmp5);
    tmp15 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp11, tmp15}, tmp7);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp11, tmp16}, p_underlying.object);
    tmp17 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    tmp18 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp16}, TNode<IntPtrT>{tmp17});
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{tmp11, tmp18}, p_underlying.next);
    tmp19 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    CodeStubAssembler(state_).StoreReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>(CodeStubAssembler::Reference{tmp11, tmp19}, p_mapper);
    tmp20 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    tmp21 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{tmp11, tmp20}, tmp21);
    tmp22 = FromConstexpr_intptr_constexpr_int31_0(state_, 32);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp11, tmp22}, p_underlying.object);
    tmp23 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    tmp24 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp22}, TNode<IntPtrT>{tmp23});
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{tmp11, tmp24}, p_underlying.next);
    tmp25 = TORQUE_CAST(TNode<HeapObject>{tmp11});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSIteratorFlatMapHelper>{tmp25};
}

TF_BUILTIN(IteratorPrototypeFlatMap, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kMapper);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), kFlatMapMethodName_0(state_));
  }

  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter2}, &label3);
    ca_.Goto(&block9);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledNonCallable), kFlatMapMethodName_0(state_));
  }

  TNode<JSReceiver> tmp4;
  TNode<JSAny> tmp5;
  TNode<JSIteratorFlatMapHelper> tmp6;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    std::tie(tmp4, tmp5) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp6 = NewJSIteratorFlatMapHelper_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}}, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>{tmp2});
    CodeStubAssembler(state_).Return(tmp6);
  }
}

TF_BUILTIN(IteratorFlatMapHelperNext, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSIteratorFlatMapHelper> parameter1 = UncheckedParameter<JSIteratorFlatMapHelper>(Descriptor::kHelper);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block6(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block7(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, BoolT> block10(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block13(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block14(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block15(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block18(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block19(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block21(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block24(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block25(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block30(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block31(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block32(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block33(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block34(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block35(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block36(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, JSAny, Union<JSMessageObject, TheHole>> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block37(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block38(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block39(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block40(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block41(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block42(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block47(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block48(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block49(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block52(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block53(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block54(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block55(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, JSAny, Union<JSMessageObject, TheHole>> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block56(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block57(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block58(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block59(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, JSAny, Union<JSMessageObject, TheHole>> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Map> tmp0;
  TNode<BoolT> tmp1;
  TNode<BoolT> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    tmp1 = IteratorHelperIsSuspendedStart_0(state_, TNode<JSIteratorHelper>{parameter1});
    tmp2 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp1});
    MarkIteratorHelperAsExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    ca_.Goto(&block5, tmp2);
  }

  TNode<BoolT> phi_bb5_3;
  TNode<BoolT> tmp3;
      TNode<JSAny> tmp5;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch4__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch4__label);
    tmp3 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    }
    if (catch4__label.is_used()) {
      compiler::CodeAssemblerLabel catch4_skip(&ca_);
      ca_.Goto(&catch4_skip);
      ca_.Bind(&catch4__label, &tmp5);
      ca_.Goto(&block6, phi_bb5_3);
      ca_.Bind(&catch4_skip);
    }
    ca_.Branch(tmp3, &block3, std::vector<compiler::Node*>{phi_bb5_3}, &block4, std::vector<compiler::Node*>{phi_bb5_3});
  }

  TNode<BoolT> phi_bb6_3;
  TNode<Union<JSMessageObject, TheHole>> tmp6;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_3);
    tmp6 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb6_3, tmp5, tmp6);
  }

  TNode<BoolT> phi_bb3_3;
  TNode<IntPtrT> tmp7;
      TNode<JSAny> tmp9;
  TNode<Number> tmp10;
  TNode<BoolT> tmp11;
      TNode<JSAny> tmp13;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch8__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch8__label);
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch8__label.is_used()) {
      compiler::CodeAssemblerLabel catch8_skip(&ca_);
      ca_.Goto(&catch8_skip);
      ca_.Bind(&catch8__label, &tmp9);
      ca_.Goto(&block7, phi_bb3_3);
      ca_.Bind(&catch8_skip);
    }
    tmp10 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{parameter1, tmp7});
    compiler::CodeAssemblerExceptionHandlerLabel catch12__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch12__label);
    tmp11 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{phi_bb3_3});
    }
    if (catch12__label.is_used()) {
      compiler::CodeAssemblerLabel catch12_skip(&ca_);
      ca_.Goto(&catch12_skip);
      ca_.Bind(&catch12__label, &tmp13);
      ca_.Goto(&block10, phi_bb3_3, phi_bb3_3);
      ca_.Bind(&catch12_skip);
    }
    ca_.Branch(tmp11, &block8, std::vector<compiler::Node*>{phi_bb3_3}, &block9, std::vector<compiler::Node*>{phi_bb3_3});
  }

  TNode<BoolT> phi_bb7_3;
  TNode<Union<JSMessageObject, TheHole>> tmp14;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_3);
    tmp14 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb7_3, tmp9, tmp14);
  }

  TNode<BoolT> phi_bb10_3;
  TNode<BoolT> phi_bb10_5;
  TNode<Union<JSMessageObject, TheHole>> tmp15;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_3, &phi_bb10_5);
    tmp15 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb10_3, tmp13, tmp15);
  }

  TNode<BoolT> phi_bb8_3;
  TNode<IntPtrT> tmp16;
      TNode<JSAny> tmp18;
  TNode<JSReceiver> tmp19;
  TNode<IntPtrT> tmp20;
      TNode<JSAny> tmp22;
  TNode<IntPtrT> tmp23;
      TNode<JSAny> tmp25;
  TNode<JSAny> tmp26;
  TNode<JSReceiver> tmp27;
      TNode<JSAny> tmp30;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch17__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch17__label);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch17__label.is_used()) {
      compiler::CodeAssemblerLabel catch17_skip(&ca_);
      ca_.Goto(&catch17_skip);
      ca_.Bind(&catch17__label, &tmp18);
      ca_.Goto(&block13, phi_bb8_3);
      ca_.Bind(&catch17_skip);
    }
    tmp19 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp16});
    compiler::CodeAssemblerExceptionHandlerLabel catch21__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch21__label);
    tmp20 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch21__label.is_used()) {
      compiler::CodeAssemblerLabel catch21_skip(&ca_);
      ca_.Goto(&catch21_skip);
      ca_.Bind(&catch21__label, &tmp22);
      ca_.Goto(&block14, phi_bb8_3);
      ca_.Bind(&catch21_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch24__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch24__label);
    tmp23 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp16}, TNode<IntPtrT>{tmp20});
    }
    if (catch24__label.is_used()) {
      compiler::CodeAssemblerLabel catch24_skip(&ca_);
      ca_.Goto(&catch24_skip);
      ca_.Bind(&catch24__label, &tmp25);
      ca_.Goto(&block15, phi_bb8_3);
      ca_.Bind(&catch24_skip);
    }
    tmp26 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp23});
    compiler::CodeAssemblerLabel label28(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch29__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch29__label);
    tmp27 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp19}, TNode<JSAny>{tmp26}}, TNode<Map>{tmp0}, &label28);
    }
    if (catch29__label.is_used()) {
      compiler::CodeAssemblerLabel catch29_skip(&ca_);
      ca_.Goto(&catch29_skip);
      ca_.Bind(&catch29__label, &tmp30);
      ca_.Goto(&block18, phi_bb8_3);
      ca_.Bind(&catch29_skip);
    }
    ca_.Goto(&block16, phi_bb8_3);
    if (label28.is_used()) {
      ca_.Bind(&label28);
      ca_.Goto(&block17, phi_bb8_3);
    }
  }

  TNode<BoolT> phi_bb13_3;
  TNode<Union<JSMessageObject, TheHole>> tmp31;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_3);
    tmp31 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb13_3, tmp18, tmp31);
  }

  TNode<BoolT> phi_bb14_3;
  TNode<Union<JSMessageObject, TheHole>> tmp32;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_3);
    tmp32 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb14_3, tmp22, tmp32);
  }

  TNode<BoolT> phi_bb15_3;
  TNode<Union<JSMessageObject, TheHole>> tmp33;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_3);
    tmp33 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb15_3, tmp25, tmp33);
  }

  TNode<BoolT> phi_bb18_3;
  TNode<Union<JSMessageObject, TheHole>> tmp34;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_3);
    tmp34 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb18_3, tmp30, tmp34);
  }

  TNode<BoolT> phi_bb17_3;
      TNode<JSAny> tmp36;
  TNode<Undefined> tmp37;
  TNode<True> tmp38;
  TNode<JSObject> tmp39;
      TNode<JSAny> tmp41;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch35__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch35__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch35__label.is_used()) {
      compiler::CodeAssemblerLabel catch35_skip(&ca_);
      ca_.Goto(&catch35_skip);
      ca_.Bind(&catch35__label, &tmp36);
      ca_.Goto(&block19, phi_bb17_3);
      ca_.Bind(&catch35_skip);
    }
    tmp37 = Undefined_0(state_);
    tmp38 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch40__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch40__label);
    tmp39 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp37}, TNode<Boolean>{tmp38});
    }
    if (catch40__label.is_used()) {
      compiler::CodeAssemblerLabel catch40_skip(&ca_);
      ca_.Goto(&catch40_skip);
      ca_.Bind(&catch40__label, &tmp41);
      ca_.Goto(&block20, phi_bb17_3);
      ca_.Bind(&catch40_skip);
    }
    CodeStubAssembler(state_).Return(tmp39);
  }

  TNode<BoolT> phi_bb16_3;
  TNode<JSAny> tmp42;
      TNode<JSAny> tmp44;
  TNode<IntPtrT> tmp45;
      TNode<JSAny> tmp47;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp48;
  TNode<Undefined> tmp49;
  TNode<JSAny> tmp50;
      TNode<JSAny> tmp52;
  TNode<JSReceiver> tmp53;
      TNode<JSAny> tmp56;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch43__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch43__label);
    tmp42 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp27}, TNode<Map>{tmp0});
    }
    if (catch43__label.is_used()) {
      compiler::CodeAssemblerLabel catch43_skip(&ca_);
      ca_.Goto(&catch43_skip);
      ca_.Bind(&catch43__label, &tmp44);
      ca_.Goto(&block21, phi_bb16_3);
      ca_.Bind(&catch43_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch46__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch46__label);
    tmp45 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch46__label.is_used()) {
      compiler::CodeAssemblerLabel catch46_skip(&ca_);
      ca_.Goto(&catch46_skip);
      ca_.Bind(&catch46__label, &tmp47);
      ca_.Goto(&block24, phi_bb16_3);
      ca_.Bind(&catch46_skip);
    }
    tmp48 = CodeStubAssembler(state_).LoadReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>(CodeStubAssembler::Reference{parameter1, tmp45});
    tmp49 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch51__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch51__label);
    tmp50 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp48}, TNode<JSAny>{tmp49}, TNode<JSAny>{tmp42}, TNode<JSAny>{tmp10});
    }
    if (catch51__label.is_used()) {
      compiler::CodeAssemblerLabel catch51_skip(&ca_);
      ca_.Goto(&catch51_skip);
      ca_.Bind(&catch51__label, &tmp52);
      ca_.Goto(&block25, phi_bb16_3);
      ca_.Bind(&catch51_skip);
    }
    compiler::CodeAssemblerLabel label54(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch55__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch55__label);
    tmp53 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp50}, &label54);
    }
    if (catch55__label.is_used()) {
      compiler::CodeAssemblerLabel catch55_skip(&ca_);
      ca_.Goto(&catch55_skip);
      ca_.Bind(&catch55__label, &tmp56);
      ca_.Goto(&block30, phi_bb16_3);
      ca_.Bind(&catch55_skip);
    }
    ca_.Goto(&block28, phi_bb16_3);
    if (label54.is_used()) {
      ca_.Bind(&label54);
      ca_.Goto(&block29, phi_bb16_3);
    }
  }

  TNode<BoolT> phi_bb19_3;
  TNode<Union<JSMessageObject, TheHole>> tmp57;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_3);
    tmp57 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb19_3, tmp36, tmp57);
  }

  TNode<BoolT> phi_bb20_3;
  TNode<Union<JSMessageObject, TheHole>> tmp58;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_3);
    tmp58 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb20_3, tmp41, tmp58);
  }

  TNode<BoolT> phi_bb21_3;
  TNode<Union<JSMessageObject, TheHole>> tmp59;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_3);
    tmp59 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb21_3, tmp44, tmp59);
  }

  TNode<BoolT> phi_bb24_3;
  TNode<Union<JSMessageObject, TheHole>> tmp60;
  if (block24.is_used()) {
    ca_.Bind(&block24, &phi_bb24_3);
    tmp60 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb24_3, tmp47, tmp60);
  }

  TNode<BoolT> phi_bb25_3;
  TNode<Union<JSMessageObject, TheHole>> tmp61;
  if (block25.is_used()) {
    ca_.Bind(&block25, &phi_bb25_3);
    tmp61 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb25_3, tmp52, tmp61);
  }

  TNode<BoolT> phi_bb30_3;
  TNode<Union<JSMessageObject, TheHole>> tmp62;
  if (block30.is_used()) {
    ca_.Bind(&block30, &phi_bb30_3);
    tmp62 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb30_3, tmp56, tmp62);
  }

  TNode<BoolT> phi_bb29_3;
      TNode<JSAny> tmp64;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch63__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch63__label);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), kFlatMapMethodName_0(state_));
    }
    if (catch63__label.is_used()) {
      compiler::CodeAssemblerLabel catch63_skip(&ca_);
      ca_.Bind(&catch63__label, &tmp64);
      ca_.Goto(&block31, phi_bb29_3);
    }
  }

  TNode<BoolT> phi_bb28_3;
  TNode<IntPtrT> tmp65;
      TNode<JSAny> tmp67;
  TNode<JSReceiver> tmp68;
  TNode<JSAny> tmp69;
      TNode<JSAny> tmp71;
  TNode<IntPtrT> tmp72;
      TNode<JSAny> tmp74;
  TNode<IntPtrT> tmp75;
      TNode<JSAny> tmp77;
  TNode<BoolT> tmp78;
      TNode<JSAny> tmp80;
  TNode<IntPtrT> tmp81;
      TNode<JSAny> tmp83;
  TNode<Number> tmp84;
      TNode<JSAny> tmp86;
  TNode<Number> tmp87;
      TNode<JSAny> tmp89;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch66__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch66__label);
    tmp65 = FromConstexpr_intptr_constexpr_int31_0(state_, 32);
    }
    if (catch66__label.is_used()) {
      compiler::CodeAssemblerLabel catch66_skip(&ca_);
      ca_.Goto(&catch66_skip);
      ca_.Bind(&catch66__label, &tmp67);
      ca_.Goto(&block32, phi_bb28_3);
      ca_.Bind(&catch66_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch70__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch70__label);
    std::tie(tmp68, tmp69) = GetIteratorFlattenable_0(state_, TNode<Context>{parameter0}, TNode<Union<JSReceiver, String>>{tmp53}).Flatten();
    }
    if (catch70__label.is_used()) {
      compiler::CodeAssemblerLabel catch70_skip(&ca_);
      ca_.Goto(&catch70_skip);
      ca_.Bind(&catch70__label, &tmp71);
      ca_.Goto(&block33, phi_bb28_3);
      ca_.Bind(&catch70_skip);
    }
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp65}, tmp68);
    compiler::CodeAssemblerExceptionHandlerLabel catch73__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch73__label);
    tmp72 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch73__label.is_used()) {
      compiler::CodeAssemblerLabel catch73_skip(&ca_);
      ca_.Goto(&catch73_skip);
      ca_.Bind(&catch73__label, &tmp74);
      ca_.Goto(&block34, phi_bb28_3);
      ca_.Bind(&catch73_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch76__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch76__label);
    tmp75 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp65}, TNode<IntPtrT>{tmp72});
    }
    if (catch76__label.is_used()) {
      compiler::CodeAssemblerLabel catch76_skip(&ca_);
      ca_.Goto(&catch76_skip);
      ca_.Bind(&catch76__label, &tmp77);
      ca_.Goto(&block35, phi_bb28_3);
      ca_.Bind(&catch76_skip);
    }
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp75}, tmp69);
    compiler::CodeAssemblerExceptionHandlerLabel catch79__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch79__label);
    tmp78 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    }
    if (catch79__label.is_used()) {
      compiler::CodeAssemblerLabel catch79_skip(&ca_);
      ca_.Goto(&catch79_skip);
      ca_.Bind(&catch79__label, &tmp80);
      ca_.Goto(&block36, phi_bb28_3);
      ca_.Bind(&catch79_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch82__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch82__label);
    tmp81 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch82__label.is_used()) {
      compiler::CodeAssemblerLabel catch82_skip(&ca_);
      ca_.Goto(&catch82_skip);
      ca_.Bind(&catch82__label, &tmp83);
      ca_.Goto(&block40);
      ca_.Bind(&catch82_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch85__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch85__label);
    tmp84 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch85__label.is_used()) {
      compiler::CodeAssemblerLabel catch85_skip(&ca_);
      ca_.Goto(&catch85_skip);
      ca_.Bind(&catch85__label, &tmp86);
      ca_.Goto(&block41);
      ca_.Bind(&catch85_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch88__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch88__label);
    tmp87 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{tmp10}, TNode<Number>{tmp84});
    }
    if (catch88__label.is_used()) {
      compiler::CodeAssemblerLabel catch88_skip(&ca_);
      ca_.Goto(&catch88_skip);
      ca_.Bind(&catch88__label, &tmp89);
      ca_.Goto(&block42);
      ca_.Bind(&catch88_skip);
    }
    CodeStubAssembler(state_).StoreReference<Number>(CodeStubAssembler::Reference{parameter1, tmp81}, tmp87);
    ca_.Goto(&block9, tmp78);
  }

  TNode<BoolT> phi_bb31_3;
  TNode<Union<JSMessageObject, TheHole>> tmp90;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_3);
    tmp90 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb31_3, tmp64, tmp90);
  }

  TNode<BoolT> phi_bb32_3;
  TNode<Union<JSMessageObject, TheHole>> tmp91;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_3);
    tmp91 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb32_3, tmp67, tmp91);
  }

  TNode<BoolT> phi_bb33_3;
  TNode<Union<JSMessageObject, TheHole>> tmp92;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_3);
    tmp92 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb33_3, tmp71, tmp92);
  }

  TNode<BoolT> phi_bb34_3;
  TNode<Union<JSMessageObject, TheHole>> tmp93;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_3);
    tmp93 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb34_3, tmp74, tmp93);
  }

  TNode<BoolT> phi_bb35_3;
  TNode<Union<JSMessageObject, TheHole>> tmp94;
  if (block35.is_used()) {
    ca_.Bind(&block35, &phi_bb35_3);
    tmp94 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb35_3, tmp77, tmp94);
  }

  TNode<BoolT> phi_bb36_3;
  TNode<Union<JSMessageObject, TheHole>> tmp95;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_3);
    tmp95 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block23, phi_bb36_3, tmp80, tmp95);
  }

  TNode<BoolT> phi_bb23_3;
  TNode<JSAny> phi_bb23_7;
  TNode<Union<JSMessageObject, TheHole>> phi_bb23_8;
  TNode<IntPtrT> tmp96;
      TNode<JSAny> tmp98;
  TNode<JSReceiver> tmp99;
      TNode<JSAny> tmp101;
      TNode<JSAny> tmp103;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_3, &phi_bb23_7, &phi_bb23_8);
    compiler::CodeAssemblerExceptionHandlerLabel catch97__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch97__label);
    tmp96 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch97__label.is_used()) {
      compiler::CodeAssemblerLabel catch97_skip(&ca_);
      ca_.Goto(&catch97_skip);
      ca_.Bind(&catch97__label, &tmp98);
      ca_.Goto(&block37, phi_bb23_3);
      ca_.Bind(&catch97_skip);
    }
    tmp99 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp96});
    compiler::CodeAssemblerExceptionHandlerLabel catch100__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch100__label);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp99});
    }
    if (catch100__label.is_used()) {
      compiler::CodeAssemblerLabel catch100_skip(&ca_);
      ca_.Goto(&catch100_skip);
      ca_.Bind(&catch100__label, &tmp101);
      ca_.Goto(&block38, phi_bb23_3);
      ca_.Bind(&catch100_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch102__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch102__label);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb23_7, phi_bb23_8);
    CodeStubAssembler(state_).Unreachable();
    }
    if (catch102__label.is_used()) {
      compiler::CodeAssemblerLabel catch102_skip(&ca_);
      ca_.Bind(&catch102__label, &tmp103);
      ca_.Goto(&block39, phi_bb23_3);
    }
  }

  TNode<BoolT> phi_bb37_3;
  TNode<Union<JSMessageObject, TheHole>> tmp104;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_3);
    tmp104 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb37_3, tmp98, tmp104);
  }

  TNode<BoolT> phi_bb38_3;
  TNode<Union<JSMessageObject, TheHole>> tmp105;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_3);
    tmp105 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb38_3, tmp101, tmp105);
  }

  TNode<BoolT> phi_bb39_3;
  TNode<Union<JSMessageObject, TheHole>> tmp106;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_3);
    tmp106 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb39_3, tmp103, tmp106);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp107;
  if (block40.is_used()) {
    ca_.Bind(&block40);
    tmp107 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp78, tmp83, tmp107);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp108;
  if (block41.is_used()) {
    ca_.Bind(&block41);
    tmp108 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp78, tmp86, tmp108);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp109;
  if (block42.is_used()) {
    ca_.Bind(&block42);
    tmp109 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp78, tmp89, tmp109);
  }

  TNode<BoolT> phi_bb9_3;
  TNode<IntPtrT> tmp110;
      TNode<JSAny> tmp112;
  TNode<JSReceiver> tmp113;
  TNode<IntPtrT> tmp114;
      TNode<JSAny> tmp116;
  TNode<IntPtrT> tmp117;
      TNode<JSAny> tmp119;
  TNode<JSAny> tmp120;
  TNode<JSReceiver> tmp121;
      TNode<JSAny> tmp124;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch111__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch111__label);
    tmp110 = FromConstexpr_intptr_constexpr_int31_0(state_, 32);
    }
    if (catch111__label.is_used()) {
      compiler::CodeAssemblerLabel catch111_skip(&ca_);
      ca_.Goto(&catch111_skip);
      ca_.Bind(&catch111__label, &tmp112);
      ca_.Goto(&block47, phi_bb9_3);
      ca_.Bind(&catch111_skip);
    }
    tmp113 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp110});
    compiler::CodeAssemblerExceptionHandlerLabel catch115__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch115__label);
    tmp114 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch115__label.is_used()) {
      compiler::CodeAssemblerLabel catch115_skip(&ca_);
      ca_.Goto(&catch115_skip);
      ca_.Bind(&catch115__label, &tmp116);
      ca_.Goto(&block48, phi_bb9_3);
      ca_.Bind(&catch115_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch118__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch118__label);
    tmp117 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp110}, TNode<IntPtrT>{tmp114});
    }
    if (catch118__label.is_used()) {
      compiler::CodeAssemblerLabel catch118_skip(&ca_);
      ca_.Goto(&catch118_skip);
      ca_.Bind(&catch118__label, &tmp119);
      ca_.Goto(&block49, phi_bb9_3);
      ca_.Bind(&catch118_skip);
    }
    tmp120 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp117});
    compiler::CodeAssemblerLabel label122(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch123__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch123__label);
    tmp121 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp113}, TNode<JSAny>{tmp120}}, TNode<Map>{tmp0}, &label122);
    }
    if (catch123__label.is_used()) {
      compiler::CodeAssemblerLabel catch123_skip(&ca_);
      ca_.Goto(&catch123_skip);
      ca_.Bind(&catch123__label, &tmp124);
      ca_.Goto(&block52, phi_bb9_3);
      ca_.Bind(&catch123_skip);
    }
    ca_.Goto(&block50, phi_bb9_3);
    if (label122.is_used()) {
      ca_.Bind(&label122);
      ca_.Goto(&block51, phi_bb9_3);
    }
  }

  TNode<BoolT> phi_bb47_3;
  TNode<Union<JSMessageObject, TheHole>> tmp125;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_3);
    tmp125 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block46, phi_bb47_3, tmp112, tmp125);
  }

  TNode<BoolT> phi_bb48_3;
  TNode<Union<JSMessageObject, TheHole>> tmp126;
  if (block48.is_used()) {
    ca_.Bind(&block48, &phi_bb48_3);
    tmp126 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block46, phi_bb48_3, tmp116, tmp126);
  }

  TNode<BoolT> phi_bb49_3;
  TNode<Union<JSMessageObject, TheHole>> tmp127;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_3);
    tmp127 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block46, phi_bb49_3, tmp119, tmp127);
  }

  TNode<BoolT> phi_bb52_3;
  TNode<Union<JSMessageObject, TheHole>> tmp128;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_3);
    tmp128 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block46, phi_bb52_3, tmp124, tmp128);
  }

  TNode<BoolT> phi_bb51_3;
  TNode<BoolT> tmp129;
      TNode<JSAny> tmp131;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch130__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch130__label);
    tmp129 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    }
    if (catch130__label.is_used()) {
      compiler::CodeAssemblerLabel catch130_skip(&ca_);
      ca_.Goto(&catch130_skip);
      ca_.Bind(&catch130__label, &tmp131);
      ca_.Goto(&block59, phi_bb51_3);
      ca_.Bind(&catch130_skip);
    }
    ca_.Goto(&block5, tmp129);
  }

  TNode<BoolT> phi_bb50_3;
  TNode<JSAny> tmp132;
      TNode<JSAny> tmp134;
      TNode<JSAny> tmp136;
  TNode<False> tmp137;
  TNode<JSObject> tmp138;
      TNode<JSAny> tmp140;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_3);
    compiler::CodeAssemblerExceptionHandlerLabel catch133__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch133__label);
    tmp132 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp121}, TNode<Map>{tmp0});
    }
    if (catch133__label.is_used()) {
      compiler::CodeAssemblerLabel catch133_skip(&ca_);
      ca_.Goto(&catch133_skip);
      ca_.Bind(&catch133__label, &tmp134);
      ca_.Goto(&block53, phi_bb50_3);
      ca_.Bind(&catch133_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch135__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch135__label);
    MarkIteratorHelperAsFinishedExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch135__label.is_used()) {
      compiler::CodeAssemblerLabel catch135_skip(&ca_);
      ca_.Goto(&catch135_skip);
      ca_.Bind(&catch135__label, &tmp136);
      ca_.Goto(&block54, phi_bb50_3);
      ca_.Bind(&catch135_skip);
    }
    tmp137 = False_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch139__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch139__label);
    tmp138 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp132}, TNode<Boolean>{tmp137});
    }
    if (catch139__label.is_used()) {
      compiler::CodeAssemblerLabel catch139_skip(&ca_);
      ca_.Goto(&catch139_skip);
      ca_.Bind(&catch139__label, &tmp140);
      ca_.Goto(&block55, phi_bb50_3);
      ca_.Bind(&catch139_skip);
    }
    CodeStubAssembler(state_).Return(tmp138);
  }

  TNode<BoolT> phi_bb53_3;
  TNode<Union<JSMessageObject, TheHole>> tmp141;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_3);
    tmp141 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block46, phi_bb53_3, tmp134, tmp141);
  }

  TNode<BoolT> phi_bb54_3;
  TNode<Union<JSMessageObject, TheHole>> tmp142;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_3);
    tmp142 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block46, phi_bb54_3, tmp136, tmp142);
  }

  TNode<BoolT> phi_bb55_3;
  TNode<Union<JSMessageObject, TheHole>> tmp143;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_3);
    tmp143 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block46, phi_bb55_3, tmp140, tmp143);
  }

  TNode<BoolT> phi_bb46_3;
  TNode<JSAny> phi_bb46_5;
  TNode<Union<JSMessageObject, TheHole>> phi_bb46_6;
  TNode<IntPtrT> tmp144;
      TNode<JSAny> tmp146;
  TNode<JSReceiver> tmp147;
      TNode<JSAny> tmp149;
      TNode<JSAny> tmp151;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_3, &phi_bb46_5, &phi_bb46_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch145__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch145__label);
    tmp144 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch145__label.is_used()) {
      compiler::CodeAssemblerLabel catch145_skip(&ca_);
      ca_.Goto(&catch145_skip);
      ca_.Bind(&catch145__label, &tmp146);
      ca_.Goto(&block56, phi_bb46_3);
      ca_.Bind(&catch145_skip);
    }
    tmp147 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp144});
    compiler::CodeAssemblerExceptionHandlerLabel catch148__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch148__label);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp147});
    }
    if (catch148__label.is_used()) {
      compiler::CodeAssemblerLabel catch148_skip(&ca_);
      ca_.Goto(&catch148_skip);
      ca_.Bind(&catch148__label, &tmp149);
      ca_.Goto(&block57, phi_bb46_3);
      ca_.Bind(&catch148_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch150__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch150__label);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb46_5, phi_bb46_6);
    CodeStubAssembler(state_).Unreachable();
    }
    if (catch150__label.is_used()) {
      compiler::CodeAssemblerLabel catch150_skip(&ca_);
      ca_.Bind(&catch150__label, &tmp151);
      ca_.Goto(&block58, phi_bb46_3);
    }
  }

  TNode<BoolT> phi_bb56_3;
  TNode<Union<JSMessageObject, TheHole>> tmp152;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_3);
    tmp152 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb56_3, tmp146, tmp152);
  }

  TNode<BoolT> phi_bb57_3;
  TNode<Union<JSMessageObject, TheHole>> tmp153;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_3);
    tmp153 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb57_3, tmp149, tmp153);
  }

  TNode<BoolT> phi_bb58_3;
  TNode<Union<JSMessageObject, TheHole>> tmp154;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_3);
    tmp154 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb58_3, tmp151, tmp154);
  }

  TNode<BoolT> phi_bb59_3;
  TNode<Union<JSMessageObject, TheHole>> tmp155;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_3);
    tmp155 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb59_3, tmp131, tmp155);
  }

  TNode<BoolT> phi_bb4_3;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_3);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BoolT> phi_bb2_3;
  TNode<JSAny> phi_bb2_4;
  TNode<Union<JSMessageObject, TheHole>> phi_bb2_5;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_3, &phi_bb2_4, &phi_bb2_5);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb2_4, phi_bb2_5);
    CodeStubAssembler(state_).Unreachable();
  }
}

TF_BUILTIN(IteratorPrototypeReduce, CodeStubAssembler) {
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
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSReceiver, JSAny, Number> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSReceiver, JSAny, Number> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSReceiver, JSAny, Number> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSReceiver, JSAny, Number> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSReceiver, JSAny, Number> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Number, JSAny, Number> block27(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number, Number> block28(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block29(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Number, JSAny, Union<JSMessageObject, TheHole>> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSReceiver, JSAny, Number> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.reduce");
  }

  TNode<IntPtrT> tmp2;
  TNode<JSAny> tmp3;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp4;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp2 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp3 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp2});
    compiler::CodeAssemblerLabel label5(&ca_);
    tmp4 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp3}, &label5);
    ca_.Goto(&block9);
    if (label5.is_used()) {
      ca_.Bind(&label5);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowCalledNonCallable, parameter0, tmp3);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSReceiver> tmp6;
  TNode<JSAny> tmp7;
  TNode<Map> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<BoolT> tmp10;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    std::tie(tmp6, tmp7) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp8 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    tmp9 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp10 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{tmp9});
    ca_.Branch(tmp10, &block11, std::vector<compiler::Node*>{}, &block12, std::vector<compiler::Node*>{});
  }

  TNode<JSReceiver> tmp11;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    compiler::CodeAssemblerLabel label12(&ca_);
    tmp11 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp6}, TNode<JSAny>{tmp7}}, TNode<Map>{tmp8}, &label12);
    ca_.Goto(&block16);
    if (label12.is_used()) {
      ca_.Bind(&label12);
      ca_.Goto(&block17);
    }
  }

  if (block17.is_used()) {
    ca_.Bind(&block17);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kIteratorReduceNoInitial), "Iterator.prototype.reduce");
  }

  TNode<JSAny> tmp13;
  TNode<Number> tmp14;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp13 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp11}, TNode<Map>{tmp8});
    tmp14 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    ca_.Goto(&block13, tmp11, tmp13, tmp14);
  }

  TNode<IntPtrT> tmp15;
  TNode<JSAny> tmp16;
  TNode<Number> tmp17;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp15 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp16 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp15});
    tmp17 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block13, ca_.Uninitialized<JSReceiver>(), tmp16, tmp17);
  }

  TNode<JSReceiver> phi_bb13_11;
  TNode<JSAny> phi_bb13_12;
  TNode<Number> phi_bb13_13;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_11, &phi_bb13_12, &phi_bb13_13);
    ca_.Goto(&block20, phi_bb13_11, phi_bb13_12, phi_bb13_13);
  }

  TNode<JSReceiver> phi_bb20_11;
  TNode<JSAny> phi_bb20_12;
  TNode<Number> phi_bb20_13;
  TNode<BoolT> tmp18;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_11, &phi_bb20_12, &phi_bb20_13);
    tmp18 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp18, &block18, std::vector<compiler::Node*>{phi_bb20_11, phi_bb20_12, phi_bb20_13}, &block19, std::vector<compiler::Node*>{phi_bb20_11, phi_bb20_12, phi_bb20_13});
  }

  TNode<JSReceiver> phi_bb18_11;
  TNode<JSAny> phi_bb18_12;
  TNode<Number> phi_bb18_13;
  TNode<JSReceiver> tmp19;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_11, &phi_bb18_12, &phi_bb18_13);
    compiler::CodeAssemblerLabel label20(&ca_);
    tmp19 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp6}, TNode<JSAny>{tmp7}}, TNode<Map>{tmp8}, &label20);
    ca_.Goto(&block23, phi_bb18_11, phi_bb18_12, phi_bb18_13);
    if (label20.is_used()) {
      ca_.Bind(&label20);
      ca_.Goto(&block24, phi_bb18_11, phi_bb18_12, phi_bb18_13);
    }
  }

  TNode<JSReceiver> phi_bb24_11;
  TNode<JSAny> phi_bb24_12;
  TNode<Number> phi_bb24_13;
  if (block24.is_used()) {
    ca_.Bind(&block24, &phi_bb24_11, &phi_bb24_12, &phi_bb24_13);
    arguments.PopAndReturn(phi_bb24_12);
  }

  TNode<JSReceiver> phi_bb23_11;
  TNode<JSAny> phi_bb23_12;
  TNode<Number> phi_bb23_13;
  TNode<JSAny> tmp21;
  TNode<Undefined> tmp22;
  TNode<JSAny> tmp23;
      TNode<JSAny> tmp25;
  TNode<Number> tmp26;
      TNode<JSAny> tmp28;
  TNode<Number> tmp29;
      TNode<JSAny> tmp31;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_11, &phi_bb23_12, &phi_bb23_13);
    tmp21 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp19}, TNode<Map>{tmp8});
    tmp22 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch24__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch24__label);
    tmp23 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp4}, TNode<JSAny>{tmp22}, TNode<JSAny>{phi_bb23_12}, TNode<JSAny>{tmp21}, TNode<JSAny>{phi_bb23_13});
    }
    if (catch24__label.is_used()) {
      compiler::CodeAssemblerLabel catch24_skip(&ca_);
      ca_.Goto(&catch24_skip);
      ca_.Bind(&catch24__label, &tmp25);
      ca_.Goto(&block27, phi_bb23_12, phi_bb23_13, phi_bb23_12, phi_bb23_13);
      ca_.Bind(&catch24_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch27__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch27__label);
    tmp26 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch27__label.is_used()) {
      compiler::CodeAssemblerLabel catch27_skip(&ca_);
      ca_.Goto(&catch27_skip);
      ca_.Bind(&catch27__label, &tmp28);
      ca_.Goto(&block28, phi_bb23_13, phi_bb23_13, phi_bb23_13);
      ca_.Bind(&catch27_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch30__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch30__label);
    tmp29 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{phi_bb23_13}, TNode<Number>{tmp26});
    }
    if (catch30__label.is_used()) {
      compiler::CodeAssemblerLabel catch30_skip(&ca_);
      ca_.Goto(&catch30_skip);
      ca_.Bind(&catch30__label, &tmp31);
      ca_.Goto(&block29, phi_bb23_13, phi_bb23_13);
      ca_.Bind(&catch30_skip);
    }
    ca_.Goto(&block20, tmp19, tmp23, tmp29);
  }

  TNode<JSAny> phi_bb27_12;
  TNode<Number> phi_bb27_13;
  TNode<JSAny> phi_bb27_18;
  TNode<Number> phi_bb27_20;
  TNode<Union<JSMessageObject, TheHole>> tmp32;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_12, &phi_bb27_13, &phi_bb27_18, &phi_bb27_20);
    tmp32 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block26, phi_bb27_12, phi_bb27_13, tmp25, tmp32);
  }

  TNode<Number> phi_bb28_13;
  TNode<Number> phi_bb28_16;
  TNode<Number> phi_bb28_17;
  TNode<Union<JSMessageObject, TheHole>> tmp33;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_13, &phi_bb28_16, &phi_bb28_17);
    tmp33 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block26, tmp23, phi_bb28_13, tmp28, tmp33);
  }

  TNode<Number> phi_bb29_13;
  TNode<Number> phi_bb29_16;
  TNode<Union<JSMessageObject, TheHole>> tmp34;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_13, &phi_bb29_16);
    tmp34 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block26, tmp23, phi_bb29_13, tmp31, tmp34);
  }

  TNode<JSAny> phi_bb26_12;
  TNode<Number> phi_bb26_13;
  TNode<JSAny> phi_bb26_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb26_16;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_12, &phi_bb26_13, &phi_bb26_15, &phi_bb26_16);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb26_15, phi_bb26_16);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSReceiver> phi_bb19_11;
  TNode<JSAny> phi_bb19_12;
  TNode<Number> phi_bb19_13;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_11, &phi_bb19_12, &phi_bb19_13);
    CodeStubAssembler(state_).Unreachable();
  }
}

TF_BUILTIN(IteratorPrototypeToArray, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, JSReceiver> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, JSReceiver> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, JSReceiver> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, JSReceiver> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT> block52(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, JSReceiver> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block3);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block4);
    }
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.toArray");
  }

  TNode<JSReceiver> tmp2;
  TNode<JSAny> tmp3;
  TNode<FixedArray> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<Map> tmp7;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    std::tie(tmp2, tmp3) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    std::tie(tmp4, tmp5, tmp6) = NewGrowableFixedArray_0(state_).Flatten();
    tmp7 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    ca_.Goto(&block7, tmp4, tmp5, tmp6, ca_.Uninitialized<JSReceiver>());
  }

  TNode<FixedArray> phi_bb7_5;
  TNode<IntPtrT> phi_bb7_6;
  TNode<IntPtrT> phi_bb7_7;
  TNode<JSReceiver> phi_bb7_9;
  TNode<BoolT> tmp8;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_5, &phi_bb7_6, &phi_bb7_7, &phi_bb7_9);
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp8, &block5, std::vector<compiler::Node*>{phi_bb7_5, phi_bb7_6, phi_bb7_7, phi_bb7_9}, &block6, std::vector<compiler::Node*>{phi_bb7_5, phi_bb7_6, phi_bb7_7, phi_bb7_9});
  }

  TNode<FixedArray> phi_bb5_5;
  TNode<IntPtrT> phi_bb5_6;
  TNode<IntPtrT> phi_bb5_7;
  TNode<JSReceiver> phi_bb5_9;
  TNode<JSReceiver> tmp9;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_5, &phi_bb5_6, &phi_bb5_7, &phi_bb5_9);
    compiler::CodeAssemblerLabel label10(&ca_);
    tmp9 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp2}, TNode<JSAny>{tmp3}}, TNode<Map>{tmp7}, &label10);
    ca_.Goto(&block10, phi_bb5_5, phi_bb5_6, phi_bb5_7, phi_bb5_9);
    if (label10.is_used()) {
      ca_.Bind(&label10);
      ca_.Goto(&block11, phi_bb5_5, phi_bb5_6, phi_bb5_7, phi_bb5_9);
    }
  }

  TNode<FixedArray> phi_bb11_5;
  TNode<IntPtrT> phi_bb11_6;
  TNode<IntPtrT> phi_bb11_7;
  TNode<JSReceiver> phi_bb11_9;
  TNode<NativeContext> tmp11;
  TNode<Map> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<TheHole> tmp14;
  TNode<FixedArray> tmp15;
  TNode<Smi> tmp16;
  TNode<JSArray> tmp17;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_5, &phi_bb11_6, &phi_bb11_7, &phi_bb11_9);
    tmp11 = CodeStubAssembler(state_).LoadNativeContext(TNode<Context>{parameter0});
    tmp12 = CodeStubAssembler(state_).LoadJSArrayElementsMap(CastIfEnumClass<ElementsKind>(ElementsKind::PACKED_ELEMENTS), TNode<NativeContext>{tmp11});
    tmp13 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp14 = TheHole_0(state_);
    tmp15 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb11_5}, TNode<IntPtrT>{tmp13}, TNode<IntPtrT>{phi_bb11_7}, TNode<IntPtrT>{phi_bb11_7}, TNode<Hole>{tmp14});
    tmp16 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{phi_bb11_7});
    tmp17 = CodeStubAssembler(state_).AllocateJSArray(TNode<Map>{tmp12}, TNode<FixedArrayBase>{tmp15}, TNode<Smi>{tmp16});
    CodeStubAssembler(state_).Return(tmp17);
  }

  TNode<FixedArray> phi_bb10_5;
  TNode<IntPtrT> phi_bb10_6;
  TNode<IntPtrT> phi_bb10_7;
  TNode<JSReceiver> phi_bb10_9;
  TNode<JSAny> tmp18;
  TNode<BoolT> tmp19;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_5, &phi_bb10_6, &phi_bb10_7, &phi_bb10_9);
    tmp18 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp9}, TNode<Map>{tmp7});
    tmp19 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb10_6}, TNode<IntPtrT>{phi_bb10_7});
    ca_.Branch(tmp19, &block32, std::vector<compiler::Node*>{phi_bb10_5, phi_bb10_6, phi_bb10_7}, &block33, std::vector<compiler::Node*>{phi_bb10_5, phi_bb10_6, phi_bb10_7});
  }

  TNode<FixedArray> phi_bb32_5;
  TNode<IntPtrT> phi_bb32_6;
  TNode<IntPtrT> phi_bb32_7;
  TNode<IntPtrT> tmp20;
  TNode<IntPtrT> tmp21;
  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<IntPtrT> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<TheHole> tmp26;
  TNode<FixedArray> tmp27;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_5, &phi_bb32_6, &phi_bb32_7);
    tmp20 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp21 = CodeStubAssembler(state_).WordSar(TNode<IntPtrT>{phi_bb32_6}, TNode<IntPtrT>{tmp20});
    tmp22 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb32_6}, TNode<IntPtrT>{tmp21});
    tmp23 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x10ull));
    tmp24 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp22}, TNode<IntPtrT>{tmp23});
    tmp25 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp26 = TheHole_0(state_);
    tmp27 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb32_5}, TNode<IntPtrT>{tmp25}, TNode<IntPtrT>{phi_bb32_7}, TNode<IntPtrT>{tmp24}, TNode<Hole>{tmp26});
    ca_.Goto(&block33, tmp27, tmp24, phi_bb32_7);
  }

  TNode<FixedArray> phi_bb33_5;
  TNode<IntPtrT> phi_bb33_6;
  TNode<IntPtrT> phi_bb33_7;
  TNode<Union<HeapObject, TaggedIndex>> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<IntPtrT> tmp31;
  TNode<IntPtrT> tmp32;
  TNode<UintPtrT> tmp33;
  TNode<UintPtrT> tmp34;
  TNode<BoolT> tmp35;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_5, &phi_bb33_6, &phi_bb33_7);
    std::tie(tmp28, tmp29, tmp30) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb33_5}).Flatten();
    tmp31 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp32 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb33_7}, TNode<IntPtrT>{tmp31});
    tmp33 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb33_7});
    tmp34 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp30});
    tmp35 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp33}, TNode<UintPtrT>{tmp34});
    ca_.Branch(tmp35, &block51, std::vector<compiler::Node*>{phi_bb33_7, phi_bb33_7, phi_bb33_7, phi_bb33_7}, &block52, std::vector<compiler::Node*>{phi_bb33_7, phi_bb33_7, phi_bb33_7, phi_bb33_7});
  }

  TNode<IntPtrT> phi_bb51_17;
  TNode<IntPtrT> phi_bb51_18;
  TNode<IntPtrT> phi_bb51_22;
  TNode<IntPtrT> phi_bb51_23;
  TNode<IntPtrT> tmp36;
  TNode<IntPtrT> tmp37;
  TNode<Union<HeapObject, TaggedIndex>> tmp38;
  TNode<IntPtrT> tmp39;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_17, &phi_bb51_18, &phi_bb51_22, &phi_bb51_23);
    tmp36 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb51_23});
    tmp37 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp29}, TNode<IntPtrT>{tmp36});
    std::tie(tmp38, tmp39) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp28}, TNode<IntPtrT>{tmp37}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp38, tmp39}, tmp18);
    ca_.Goto(&block7, phi_bb33_5, phi_bb33_6, tmp32, tmp9);
  }

  TNode<IntPtrT> phi_bb52_17;
  TNode<IntPtrT> phi_bb52_18;
  TNode<IntPtrT> phi_bb52_22;
  TNode<IntPtrT> phi_bb52_23;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_17, &phi_bb52_18, &phi_bb52_22, &phi_bb52_23);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb6_5;
  TNode<IntPtrT> phi_bb6_6;
  TNode<IntPtrT> phi_bb6_7;
  TNode<JSReceiver> phi_bb6_9;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_5, &phi_bb6_6, &phi_bb6_7, &phi_bb6_9);
    CodeStubAssembler(state_).Unreachable();
  }
}

TF_BUILTIN(IteratorPrototypeForEach, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kFn);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number, Number> block21(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block22(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, JSAny, Union<JSMessageObject, TheHole>> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.forEach");
  }

  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter2}, &label3);
    ca_.Goto(&block9);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowCalledNonCallable, parameter0, parameter2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSReceiver> tmp4;
  TNode<JSAny> tmp5;
  TNode<Number> tmp6;
  TNode<Map> tmp7;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    std::tie(tmp4, tmp5) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp6 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp7 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    ca_.Goto(&block13, tmp6);
  }

  TNode<Number> phi_bb13_7;
  TNode<BoolT> tmp8;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_7);
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp8, &block11, std::vector<compiler::Node*>{phi_bb13_7}, &block12, std::vector<compiler::Node*>{phi_bb13_7});
  }

  TNode<Number> phi_bb11_7;
  TNode<JSReceiver> tmp9;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_7);
    compiler::CodeAssemblerLabel label10(&ca_);
    tmp9 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}}, TNode<Map>{tmp7}, &label10);
    ca_.Goto(&block16, phi_bb11_7);
    if (label10.is_used()) {
      ca_.Bind(&label10);
      ca_.Goto(&block17, phi_bb11_7);
    }
  }

  TNode<Number> phi_bb17_7;
  TNode<Undefined> tmp11;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_7);
    tmp11 = Undefined_0(state_);
    CodeStubAssembler(state_).Return(tmp11);
  }

  TNode<Number> phi_bb16_7;
  TNode<JSAny> tmp12;
  TNode<Undefined> tmp13;
  TNode<JSAny> tmp14;
      TNode<JSAny> tmp16;
  TNode<Number> tmp17;
      TNode<JSAny> tmp19;
  TNode<Number> tmp20;
      TNode<JSAny> tmp22;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_7);
    tmp12 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp9}, TNode<Map>{tmp7});
    tmp13 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch15__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch15__label);
    tmp14 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp2}, TNode<JSAny>{tmp13}, TNode<JSAny>{tmp12}, TNode<JSAny>{phi_bb16_7});
    }
    if (catch15__label.is_used()) {
      compiler::CodeAssemblerLabel catch15_skip(&ca_);
      ca_.Goto(&catch15_skip);
      ca_.Bind(&catch15__label, &tmp16);
      ca_.Goto(&block20, phi_bb16_7, phi_bb16_7);
      ca_.Bind(&catch15_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch18__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch18__label);
    tmp17 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch18__label.is_used()) {
      compiler::CodeAssemblerLabel catch18_skip(&ca_);
      ca_.Goto(&catch18_skip);
      ca_.Bind(&catch18__label, &tmp19);
      ca_.Goto(&block21, phi_bb16_7, phi_bb16_7, phi_bb16_7);
      ca_.Bind(&catch18_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch21__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch21__label);
    tmp20 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{phi_bb16_7}, TNode<Number>{tmp17});
    }
    if (catch21__label.is_used()) {
      compiler::CodeAssemblerLabel catch21_skip(&ca_);
      ca_.Goto(&catch21_skip);
      ca_.Bind(&catch21__label, &tmp22);
      ca_.Goto(&block22, phi_bb16_7, phi_bb16_7);
      ca_.Bind(&catch21_skip);
    }
    ca_.Goto(&block13, tmp20);
  }

  TNode<Number> phi_bb20_7;
  TNode<Number> phi_bb20_15;
  TNode<Union<JSMessageObject, TheHole>> tmp23;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_7, &phi_bb20_15);
    tmp23 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block19, phi_bb20_7, tmp16, tmp23);
  }

  TNode<Number> phi_bb21_7;
  TNode<Number> phi_bb21_11;
  TNode<Number> phi_bb21_12;
  TNode<Union<JSMessageObject, TheHole>> tmp24;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_7, &phi_bb21_11, &phi_bb21_12);
    tmp24 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block19, phi_bb21_7, tmp19, tmp24);
  }

  TNode<Number> phi_bb22_7;
  TNode<Number> phi_bb22_11;
  TNode<Union<JSMessageObject, TheHole>> tmp25;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_7, &phi_bb22_11);
    tmp25 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block19, phi_bb22_7, tmp22, tmp25);
  }

  TNode<Number> phi_bb19_7;
  TNode<JSAny> phi_bb19_11;
  TNode<Union<JSMessageObject, TheHole>> phi_bb19_12;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_7, &phi_bb19_11, &phi_bb19_12);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb19_11, phi_bb19_12);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Number> phi_bb12_7;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_7);
    CodeStubAssembler(state_).Unreachable();
  }
}

TF_BUILTIN(IteratorPrototypeSome, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kPredicate);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.some");
  }

  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter2}, &label3);
    ca_.Goto(&block9);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowCalledNonCallable, parameter0, parameter2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSReceiver> tmp4;
  TNode<JSAny> tmp5;
  TNode<Number> tmp6;
  TNode<Map> tmp7;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    std::tie(tmp4, tmp5) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp6 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp7 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    ca_.Goto(&block13, tmp6);
  }

  TNode<Number> phi_bb13_7;
  TNode<BoolT> tmp8;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_7);
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp8, &block11, std::vector<compiler::Node*>{phi_bb13_7}, &block12, std::vector<compiler::Node*>{phi_bb13_7});
  }

  TNode<Number> phi_bb11_7;
  TNode<JSReceiver> tmp9;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_7);
    compiler::CodeAssemblerLabel label10(&ca_);
    tmp9 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}}, TNode<Map>{tmp7}, &label10);
    ca_.Goto(&block16, phi_bb11_7);
    if (label10.is_used()) {
      ca_.Bind(&label10);
      ca_.Goto(&block17, phi_bb11_7);
    }
  }

  TNode<Number> phi_bb17_7;
  TNode<False> tmp11;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_7);
    tmp11 = False_0(state_);
    CodeStubAssembler(state_).Return(tmp11);
  }

  TNode<Number> phi_bb16_7;
  TNode<JSAny> tmp12;
  TNode<Undefined> tmp13;
  TNode<JSAny> tmp14;
      TNode<JSAny> tmp16;
  TNode<BoolT> tmp17;
  TNode<BoolT> tmp18;
  TNode<BoolT> tmp19;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_7);
    tmp12 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp9}, TNode<Map>{tmp7});
    tmp13 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch15__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch15__label);
    tmp14 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp2}, TNode<JSAny>{tmp13}, TNode<JSAny>{tmp12}, TNode<JSAny>{phi_bb16_7});
    }
    if (catch15__label.is_used()) {
      compiler::CodeAssemblerLabel catch15_skip(&ca_);
      ca_.Goto(&catch15_skip);
      ca_.Bind(&catch15__label, &tmp16);
      ca_.Goto(&block20, phi_bb16_7, phi_bb16_7);
      ca_.Bind(&catch15_skip);
    }
    tmp17 = ToBoolean_0(state_, TNode<JSAny>{tmp14});
    tmp18 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp19 = CodeStubAssembler(state_).Word32Equal(TNode<BoolT>{tmp17}, TNode<BoolT>{tmp18});
    ca_.Branch(tmp19, &block21, std::vector<compiler::Node*>{phi_bb16_7}, &block22, std::vector<compiler::Node*>{phi_bb16_7});
  }

  TNode<Number> phi_bb20_7;
  TNode<Number> phi_bb20_16;
  TNode<Union<JSMessageObject, TheHole>> tmp20;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_7, &phi_bb20_16);
    tmp20 = GetAndResetPendingMessage_0(state_);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp16, tmp20);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Number> phi_bb21_7;
  TNode<True> tmp21;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_7);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}});
    tmp21 = True_0(state_);
    CodeStubAssembler(state_).Return(tmp21);
  }

  TNode<Number> phi_bb22_7;
  TNode<Number> tmp22;
  TNode<Number> tmp23;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_7);
    tmp22 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp23 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{phi_bb22_7}, TNode<Number>{tmp22});
    ca_.Goto(&block13, tmp23);
  }

  TNode<Number> phi_bb12_7;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_7);
    CodeStubAssembler(state_).Unreachable();
  }
}

TF_BUILTIN(IteratorPrototypeEvery, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kPredicate);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.every");
  }

  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter2}, &label3);
    ca_.Goto(&block9);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowCalledNonCallable, parameter0, parameter2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSReceiver> tmp4;
  TNode<JSAny> tmp5;
  TNode<Number> tmp6;
  TNode<Map> tmp7;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    std::tie(tmp4, tmp5) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp6 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp7 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    ca_.Goto(&block13, tmp6);
  }

  TNode<Number> phi_bb13_7;
  TNode<BoolT> tmp8;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_7);
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp8, &block11, std::vector<compiler::Node*>{phi_bb13_7}, &block12, std::vector<compiler::Node*>{phi_bb13_7});
  }

  TNode<Number> phi_bb11_7;
  TNode<JSReceiver> tmp9;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_7);
    compiler::CodeAssemblerLabel label10(&ca_);
    tmp9 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}}, TNode<Map>{tmp7}, &label10);
    ca_.Goto(&block16, phi_bb11_7);
    if (label10.is_used()) {
      ca_.Bind(&label10);
      ca_.Goto(&block17, phi_bb11_7);
    }
  }

  TNode<Number> phi_bb17_7;
  TNode<True> tmp11;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_7);
    tmp11 = True_0(state_);
    CodeStubAssembler(state_).Return(tmp11);
  }

  TNode<Number> phi_bb16_7;
  TNode<JSAny> tmp12;
  TNode<Undefined> tmp13;
  TNode<JSAny> tmp14;
      TNode<JSAny> tmp16;
  TNode<BoolT> tmp17;
  TNode<BoolT> tmp18;
  TNode<BoolT> tmp19;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_7);
    tmp12 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp9}, TNode<Map>{tmp7});
    tmp13 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch15__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch15__label);
    tmp14 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp2}, TNode<JSAny>{tmp13}, TNode<JSAny>{tmp12}, TNode<JSAny>{phi_bb16_7});
    }
    if (catch15__label.is_used()) {
      compiler::CodeAssemblerLabel catch15_skip(&ca_);
      ca_.Goto(&catch15_skip);
      ca_.Bind(&catch15__label, &tmp16);
      ca_.Goto(&block20, phi_bb16_7, phi_bb16_7);
      ca_.Bind(&catch15_skip);
    }
    tmp17 = ToBoolean_0(state_, TNode<JSAny>{tmp14});
    tmp18 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp19 = CodeStubAssembler(state_).Word32Equal(TNode<BoolT>{tmp17}, TNode<BoolT>{tmp18});
    ca_.Branch(tmp19, &block21, std::vector<compiler::Node*>{phi_bb16_7}, &block22, std::vector<compiler::Node*>{phi_bb16_7});
  }

  TNode<Number> phi_bb20_7;
  TNode<Number> phi_bb20_16;
  TNode<Union<JSMessageObject, TheHole>> tmp20;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_7, &phi_bb20_16);
    tmp20 = GetAndResetPendingMessage_0(state_);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp16, tmp20);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Number> phi_bb21_7;
  TNode<False> tmp21;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_7);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}});
    tmp21 = False_0(state_);
    CodeStubAssembler(state_).Return(tmp21);
  }

  TNode<Number> phi_bb22_7;
  TNode<Number> tmp22;
  TNode<Number> tmp23;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_7);
    tmp22 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp23 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{phi_bb22_7}, TNode<Number>{tmp22});
    ca_.Goto(&block13, tmp23);
  }

  TNode<Number> phi_bb12_7;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_7);
    CodeStubAssembler(state_).Unreachable();
  }
}

TF_BUILTIN(IteratorPrototypeFind, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kPredicate);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number, Number> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Number> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block5);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.find");
  }

  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp2;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter2}, &label3);
    ca_.Goto(&block9);
    if (label3.is_used()) {
      ca_.Bind(&label3);
      ca_.Goto(&block10);
    }
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowCalledNonCallable, parameter0, parameter2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSReceiver> tmp4;
  TNode<JSAny> tmp5;
  TNode<Number> tmp6;
  TNode<Map> tmp7;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    std::tie(tmp4, tmp5) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp6 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp7 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    ca_.Goto(&block13, tmp6);
  }

  TNode<Number> phi_bb13_7;
  TNode<BoolT> tmp8;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_7);
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp8, &block11, std::vector<compiler::Node*>{phi_bb13_7}, &block12, std::vector<compiler::Node*>{phi_bb13_7});
  }

  TNode<Number> phi_bb11_7;
  TNode<JSReceiver> tmp9;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_7);
    compiler::CodeAssemblerLabel label10(&ca_);
    tmp9 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}}, TNode<Map>{tmp7}, &label10);
    ca_.Goto(&block16, phi_bb11_7);
    if (label10.is_used()) {
      ca_.Bind(&label10);
      ca_.Goto(&block17, phi_bb11_7);
    }
  }

  TNode<Number> phi_bb17_7;
  TNode<Undefined> tmp11;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_7);
    tmp11 = Undefined_0(state_);
    CodeStubAssembler(state_).Return(tmp11);
  }

  TNode<Number> phi_bb16_7;
  TNode<JSAny> tmp12;
  TNode<Undefined> tmp13;
  TNode<JSAny> tmp14;
      TNode<JSAny> tmp16;
  TNode<BoolT> tmp17;
  TNode<BoolT> tmp18;
  TNode<BoolT> tmp19;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_7);
    tmp12 = IteratorBuiltinsAssembler(state_).IteratorValue(TNode<Context>{parameter0}, TNode<JSReceiver>{tmp9}, TNode<Map>{tmp7});
    tmp13 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch15__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch15__label);
    tmp14 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp2}, TNode<JSAny>{tmp13}, TNode<JSAny>{tmp12}, TNode<JSAny>{phi_bb16_7});
    }
    if (catch15__label.is_used()) {
      compiler::CodeAssemblerLabel catch15_skip(&ca_);
      ca_.Goto(&catch15_skip);
      ca_.Bind(&catch15__label, &tmp16);
      ca_.Goto(&block20, phi_bb16_7, phi_bb16_7);
      ca_.Bind(&catch15_skip);
    }
    tmp17 = ToBoolean_0(state_, TNode<JSAny>{tmp14});
    tmp18 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp19 = CodeStubAssembler(state_).Word32Equal(TNode<BoolT>{tmp17}, TNode<BoolT>{tmp18});
    ca_.Branch(tmp19, &block21, std::vector<compiler::Node*>{phi_bb16_7}, &block22, std::vector<compiler::Node*>{phi_bb16_7});
  }

  TNode<Number> phi_bb20_7;
  TNode<Number> phi_bb20_16;
  TNode<Union<JSMessageObject, TheHole>> tmp20;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_7, &phi_bb20_16);
    tmp20 = GetAndResetPendingMessage_0(state_);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp16, tmp20);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Number> phi_bb21_7;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_7);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp4}, TNode<JSAny>{tmp5}});
    CodeStubAssembler(state_).Return(tmp12);
  }

  TNode<Number> phi_bb22_7;
  TNode<Number> tmp21;
  TNode<Number> tmp22;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_7);
    tmp21 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp22 = CodeStubAssembler(state_).NumberAdd(TNode<Number>{phi_bb22_7}, TNode<Number>{tmp21});
    ca_.Goto(&block13, tmp22);
  }

  TNode<Number> phi_bb12_7;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_7);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1238&c=1
const char* kConcatMethodName_0(compiler::CodeAssemblerState* state_) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

    ca_.Bind(&block0);
  return "Iterator.concat";}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1240&c=1
TNode<JSIteratorConcatHelper> NewJSIteratorConcatHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArray> p_iterables) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Map> tmp3;
  TNode<FixedArray> tmp4;
  TNode<FixedArray> tmp5;
  TNode<Uint32T> tmp6;
  TNode<Smi> tmp7;
  TNode<JSObject> tmp8;
  TNode<Undefined> tmp9;
  TNode<Smi> tmp10;
  TNode<BoolT> tmp11;
  TNode<BoolT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<HeapObject> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<IntPtrT> tmp20;
  TNode<IntPtrT> tmp21;
  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<JSIteratorConcatHelper> tmp24;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ITERATOR_CONCAT_HELPER_MAP_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Map_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = CodeStubAssembler(state_).EmptyFixedArrayConstant();
    tmp5 = CodeStubAssembler(state_).EmptyFixedArrayConstant();
    tmp6 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedStart_0(state_, JSIteratorHelperState::kSuspendedStart);
    tmp7 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp6});
    tmp8 = GetIteratorPrototype_0(state_, TNode<Context>{p_context});
    tmp9 = Undefined_0(state_);
    tmp10 = SmiConstant_0(state_, IntegerLiteral(false, 0x0ull));
    tmp11 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp12 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 32);
    tmp14 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp13}, TNode<Map>{tmp3}, TNode<BoolT>{tmp11}, TNode<BoolT>{tmp12});
    tmp15 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp14, tmp15}, tmp3);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>>(CodeStubAssembler::Reference{tmp14, tmp16}, tmp4);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp14, tmp17}, tmp5);
    tmp18 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp14, tmp18}, tmp7);
    tmp19 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp14, tmp19}, tmp8);
    tmp20 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    tmp21 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp19}, TNode<IntPtrT>{tmp20});
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{tmp14, tmp21}, tmp9);
    tmp22 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    CodeStubAssembler(state_).StoreReference<FixedArray>(CodeStubAssembler::Reference{tmp14, tmp22}, p_iterables);
    tmp23 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp14, tmp23}, tmp10);
    tmp24 = TORQUE_CAST(TNode<HeapObject>{tmp14});
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<JSIteratorConcatHelper>{tmp24};
}

TF_BUILTIN(IteratorConcat, CodeStubAssembler) {
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
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{tmp0});
    ca_.Branch(tmp1, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  TNode<FixedArray> tmp2;
  if (block1.is_used()) {
    ca_.Bind(&block1);
    tmp2 = kEmptyFixedArray_0(state_);
    ca_.Goto(&block3, tmp2);
  }

  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<FixedArray> tmp5;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    tmp4 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{torque_arguments.length});
    tmp5 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{tmp4});
    ca_.Goto(&block3, tmp5);
  }

  TNode<FixedArray> phi_bb3_6;
  TNode<IntPtrT> tmp6;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_6);
    tmp6 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block7, tmp6);
  }

  TNode<IntPtrT> phi_bb7_7;
  TNode<BoolT> tmp7;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_7);
    tmp7 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb7_7}, TNode<IntPtrT>{torque_arguments.length});
    ca_.Branch(tmp7, &block5, std::vector<compiler::Node*>{phi_bb7_7}, &block6, std::vector<compiler::Node*>{phi_bb7_7});
  }

  TNode<IntPtrT> phi_bb5_7;
  TNode<JSAny> tmp8;
  TNode<BoolT> tmp9;
  TNode<BoolT> tmp10;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_7);
    tmp8 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{phi_bb5_7});
    tmp9 = Is_JSReceiver_JSAny_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{tmp8});
    tmp10 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp9});
    ca_.Branch(tmp10, &block9, std::vector<compiler::Node*>{phi_bb5_7}, &block10, std::vector<compiler::Node*>{phi_bb5_7});
  }

  TNode<IntPtrT> phi_bb9_7;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_7);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), kConcatMethodName_0(state_));
  }

  TNode<IntPtrT> phi_bb10_7;
  TNode<Symbol> tmp11;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp12;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_7);
    tmp11 = CodeStubAssembler(state_).IteratorSymbolConstant();
    compiler::CodeAssemblerLabel label13(&ca_);
    tmp12 = GetMethod_3(state_, TNode<Context>{parameter0}, TNode<JSAny>{tmp8}, TNode<Symbol>{tmp11}, &label13);
    ca_.Goto(&block13, phi_bb10_7);
    if (label13.is_used()) {
      ca_.Bind(&label13);
      ca_.Goto(&block14, phi_bb10_7);
    }
  }

  TNode<IntPtrT> phi_bb14_7;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_7);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kNotIterable), TNode<Object>{tmp8});
  }

  TNode<IntPtrT> phi_bb13_7;
  TNode<Union<HeapObject, TaggedIndex>> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<UintPtrT> tmp19;
  TNode<UintPtrT> tmp20;
  TNode<BoolT> tmp21;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_7);
    std::tie(tmp14, tmp15, tmp16) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb3_6}).Flatten();
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    tmp18 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp17}, TNode<IntPtrT>{phi_bb13_7});
    tmp19 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp18});
    tmp20 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp16});
    tmp21 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp19}, TNode<UintPtrT>{tmp20});
    ca_.Branch(tmp21, &block19, std::vector<compiler::Node*>{phi_bb13_7}, &block20, std::vector<compiler::Node*>{phi_bb13_7});
  }

  TNode<IntPtrT> phi_bb19_7;
  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<Union<HeapObject, TaggedIndex>> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<Union<HeapObject, TaggedIndex>> tmp26;
  TNode<IntPtrT> tmp27;
  TNode<IntPtrT> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<IntPtrT> tmp31;
  TNode<IntPtrT> tmp32;
  TNode<UintPtrT> tmp33;
  TNode<UintPtrT> tmp34;
  TNode<BoolT> tmp35;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_7);
    tmp22 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp18});
    tmp23 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp15}, TNode<IntPtrT>{tmp22});
    std::tie(tmp24, tmp25) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp14}, TNode<IntPtrT>{tmp23}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp24, tmp25}, tmp12);
    std::tie(tmp26, tmp27, tmp28) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb3_6}).Flatten();
    tmp29 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    tmp30 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp29}, TNode<IntPtrT>{phi_bb19_7});
    tmp31 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp32 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp30}, TNode<IntPtrT>{tmp31});
    tmp33 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp32});
    tmp34 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp28});
    tmp35 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp33}, TNode<UintPtrT>{tmp34});
    ca_.Branch(tmp35, &block27, std::vector<compiler::Node*>{phi_bb19_7}, &block28, std::vector<compiler::Node*>{phi_bb19_7});
  }

  TNode<IntPtrT> phi_bb20_7;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_7);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb27_7;
  TNode<IntPtrT> tmp36;
  TNode<IntPtrT> tmp37;
  TNode<Union<HeapObject, TaggedIndex>> tmp38;
  TNode<IntPtrT> tmp39;
  TNode<IntPtrT> tmp40;
  TNode<IntPtrT> tmp41;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_7);
    tmp36 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp32});
    tmp37 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp27}, TNode<IntPtrT>{tmp36});
    std::tie(tmp38, tmp39) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp26}, TNode<IntPtrT>{tmp37}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp38, tmp39}, tmp8);
    tmp40 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp41 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb27_7}, TNode<IntPtrT>{tmp40});
    ca_.Goto(&block7, tmp41);
  }

  TNode<IntPtrT> phi_bb28_7;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_7);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb6_7;
  TNode<JSIteratorConcatHelper> tmp42;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_7);
    tmp42 = NewJSIteratorConcatHelper_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{phi_bb3_6});
    arguments.PopAndReturn(tmp42);
  }
}

TF_BUILTIN(IteratorConcatHelperNext, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSIteratorConcatHelper> parameter1 = UncheckedParameter<JSIteratorConcatHelper>(Descriptor::kHelper);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block6(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, BoolT> block9(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block12(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block13(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block14(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block15(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block16(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block17(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block21(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block22(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block33(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block36(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block37(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block38(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block39(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block40(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block45(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block51(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block52(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block57(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block58(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block59(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block60(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block61(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block62(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block63(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block64(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block69(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block70(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block71(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block75(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block73(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block74(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block72(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block76(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block77(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block78(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block79(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block80(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block81(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block82(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block83(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block84(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block85(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block86(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT, JSAny, Union<JSMessageObject, TheHole>> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = IteratorHelperIsSuspendedStart_0(state_, TNode<JSIteratorHelper>{parameter1});
    tmp1 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp0});
    MarkIteratorHelperAsExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    ca_.Goto(&block5, tmp1);
  }

  TNode<BoolT> phi_bb5_2;
  TNode<BoolT> tmp2;
      TNode<JSAny> tmp4;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch3__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch3__label);
    tmp2 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    }
    if (catch3__label.is_used()) {
      compiler::CodeAssemblerLabel catch3_skip(&ca_);
      ca_.Goto(&catch3_skip);
      ca_.Bind(&catch3__label, &tmp4);
      ca_.Goto(&block6, phi_bb5_2);
      ca_.Bind(&catch3_skip);
    }
    ca_.Branch(tmp2, &block3, std::vector<compiler::Node*>{phi_bb5_2}, &block4, std::vector<compiler::Node*>{phi_bb5_2});
  }

  TNode<BoolT> phi_bb6_2;
  TNode<Union<JSMessageObject, TheHole>> tmp5;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_2);
    tmp5 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb6_2, tmp4, tmp5);
  }

  TNode<BoolT> phi_bb3_2;
  TNode<BoolT> tmp6;
      TNode<JSAny> tmp8;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch7__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch7__label);
    tmp6 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{phi_bb3_2});
    }
    if (catch7__label.is_used()) {
      compiler::CodeAssemblerLabel catch7_skip(&ca_);
      ca_.Goto(&catch7_skip);
      ca_.Bind(&catch7__label, &tmp8);
      ca_.Goto(&block9, phi_bb3_2, phi_bb3_2);
      ca_.Bind(&catch7_skip);
    }
    ca_.Branch(tmp6, &block7, std::vector<compiler::Node*>{phi_bb3_2}, &block8, std::vector<compiler::Node*>{phi_bb3_2});
  }

  TNode<BoolT> phi_bb9_2;
  TNode<BoolT> phi_bb9_3;
  TNode<Union<JSMessageObject, TheHole>> tmp9;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_2, &phi_bb9_3);
    tmp9 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb9_2, tmp8, tmp9);
  }

  TNode<BoolT> phi_bb7_2;
  TNode<IntPtrT> tmp10;
      TNode<JSAny> tmp12;
  TNode<Smi> tmp13;
  TNode<IntPtrT> tmp14;
      TNode<JSAny> tmp16;
  TNode<FixedArray> tmp17;
  TNode<IntPtrT> tmp18;
      TNode<JSAny> tmp20;
  TNode<Smi> tmp21;
  TNode<BoolT> tmp22;
      TNode<JSAny> tmp24;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch11__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch11__label);
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch11__label.is_used()) {
      compiler::CodeAssemblerLabel catch11_skip(&ca_);
      ca_.Goto(&catch11_skip);
      ca_.Bind(&catch11__label, &tmp12);
      ca_.Goto(&block12, phi_bb7_2);
      ca_.Bind(&catch11_skip);
    }
    tmp13 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp10});
    compiler::CodeAssemblerExceptionHandlerLabel catch15__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch15__label);
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch15__label.is_used()) {
      compiler::CodeAssemblerLabel catch15_skip(&ca_);
      ca_.Goto(&catch15_skip);
      ca_.Bind(&catch15__label, &tmp16);
      ca_.Goto(&block13, phi_bb7_2);
      ca_.Bind(&catch15_skip);
    }
    tmp17 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{parameter1, tmp14});
    compiler::CodeAssemblerExceptionHandlerLabel catch19__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch19__label);
    tmp18 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    }
    if (catch19__label.is_used()) {
      compiler::CodeAssemblerLabel catch19_skip(&ca_);
      ca_.Goto(&catch19_skip);
      ca_.Bind(&catch19__label, &tmp20);
      ca_.Goto(&block14, phi_bb7_2);
      ca_.Bind(&catch19_skip);
    }
    tmp21 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{tmp17, tmp18});
    compiler::CodeAssemblerExceptionHandlerLabel catch23__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch23__label);
    tmp22 = CodeStubAssembler(state_).SmiGreaterThanOrEqual(TNode<Smi>{tmp13}, TNode<Smi>{tmp21});
    }
    if (catch23__label.is_used()) {
      compiler::CodeAssemblerLabel catch23_skip(&ca_);
      ca_.Goto(&catch23_skip);
      ca_.Bind(&catch23__label, &tmp24);
      ca_.Goto(&block15, phi_bb7_2);
      ca_.Bind(&catch23_skip);
    }
    ca_.Branch(tmp22, &block10, std::vector<compiler::Node*>{phi_bb7_2}, &block11, std::vector<compiler::Node*>{phi_bb7_2});
  }

  TNode<BoolT> phi_bb12_2;
  TNode<Union<JSMessageObject, TheHole>> tmp25;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_2);
    tmp25 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb12_2, tmp12, tmp25);
  }

  TNode<BoolT> phi_bb13_2;
  TNode<Union<JSMessageObject, TheHole>> tmp26;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_2);
    tmp26 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb13_2, tmp16, tmp26);
  }

  TNode<BoolT> phi_bb14_2;
  TNode<Union<JSMessageObject, TheHole>> tmp27;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_2);
    tmp27 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb14_2, tmp20, tmp27);
  }

  TNode<BoolT> phi_bb15_2;
  TNode<Union<JSMessageObject, TheHole>> tmp28;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_2);
    tmp28 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb15_2, tmp24, tmp28);
  }

  TNode<BoolT> phi_bb10_2;
      TNode<JSAny> tmp30;
  TNode<Undefined> tmp31;
  TNode<True> tmp32;
  TNode<JSObject> tmp33;
      TNode<JSAny> tmp35;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch29__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch29__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch29__label.is_used()) {
      compiler::CodeAssemblerLabel catch29_skip(&ca_);
      ca_.Goto(&catch29_skip);
      ca_.Bind(&catch29__label, &tmp30);
      ca_.Goto(&block16, phi_bb10_2);
      ca_.Bind(&catch29_skip);
    }
    tmp31 = Undefined_0(state_);
    tmp32 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch34__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch34__label);
    tmp33 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp31}, TNode<Boolean>{tmp32});
    }
    if (catch34__label.is_used()) {
      compiler::CodeAssemblerLabel catch34_skip(&ca_);
      ca_.Goto(&catch34_skip);
      ca_.Bind(&catch34__label, &tmp35);
      ca_.Goto(&block17, phi_bb10_2);
      ca_.Bind(&catch34_skip);
    }
    CodeStubAssembler(state_).Return(tmp33);
  }

  TNode<BoolT> phi_bb16_2;
  TNode<Union<JSMessageObject, TheHole>> tmp36;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_2);
    tmp36 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb16_2, tmp30, tmp36);
  }

  TNode<BoolT> phi_bb17_2;
  TNode<Union<JSMessageObject, TheHole>> tmp37;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_2);
    tmp37 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb17_2, tmp35, tmp37);
  }

  TNode<BoolT> phi_bb11_2;
  TNode<IntPtrT> tmp38;
      TNode<JSAny> tmp40;
  TNode<FixedArray> tmp41;
  TNode<Union<HeapObject, TaggedIndex>> tmp42;
  TNode<IntPtrT> tmp43;
  TNode<IntPtrT> tmp44;
      TNode<JSAny> tmp46;
  TNode<IntPtrT> tmp47;
      TNode<JSAny> tmp49;
  TNode<Smi> tmp50;
  TNode<IntPtrT> tmp51;
  TNode<UintPtrT> tmp52;
  TNode<UintPtrT> tmp53;
  TNode<BoolT> tmp54;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch39__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch39__label);
    tmp38 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch39__label.is_used()) {
      compiler::CodeAssemblerLabel catch39_skip(&ca_);
      ca_.Goto(&catch39_skip);
      ca_.Bind(&catch39__label, &tmp40);
      ca_.Goto(&block20, phi_bb11_2);
      ca_.Bind(&catch39_skip);
    }
    tmp41 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{parameter1, tmp38});
    compiler::CodeAssemblerExceptionHandlerLabel catch45__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch45__label);
    std::tie(tmp42, tmp43, tmp44) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp41}).Flatten();
    }
    if (catch45__label.is_used()) {
      compiler::CodeAssemblerLabel catch45_skip(&ca_);
      ca_.Goto(&catch45_skip);
      ca_.Bind(&catch45__label, &tmp46);
      ca_.Goto(&block21, phi_bb11_2);
      ca_.Bind(&catch45_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch48__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch48__label);
    tmp47 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch48__label.is_used()) {
      compiler::CodeAssemblerLabel catch48_skip(&ca_);
      ca_.Goto(&catch48_skip);
      ca_.Bind(&catch48__label, &tmp49);
      ca_.Goto(&block22, phi_bb11_2);
      ca_.Bind(&catch48_skip);
    }
    tmp50 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp47});
    tmp51 = Convert_intptr_Smi_0(state_, TNode<Smi>{tmp50});
    tmp52 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp51});
    tmp53 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp44});
    tmp54 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp52}, TNode<UintPtrT>{tmp53});
    ca_.Branch(tmp54, &block27, std::vector<compiler::Node*>{phi_bb11_2}, &block28, std::vector<compiler::Node*>{phi_bb11_2});
  }

  TNode<BoolT> phi_bb20_2;
  TNode<Union<JSMessageObject, TheHole>> tmp55;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_2);
    tmp55 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb20_2, tmp40, tmp55);
  }

  TNode<BoolT> phi_bb21_2;
  TNode<Union<JSMessageObject, TheHole>> tmp56;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_2);
    tmp56 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb21_2, tmp46, tmp56);
  }

  TNode<BoolT> phi_bb22_2;
  TNode<Union<JSMessageObject, TheHole>> tmp57;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_2);
    tmp57 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb22_2, tmp49, tmp57);
  }

  TNode<BoolT> phi_bb27_2;
  TNode<IntPtrT> tmp58;
  TNode<IntPtrT> tmp59;
  TNode<Union<HeapObject, TaggedIndex>> tmp60;
  TNode<IntPtrT> tmp61;
  TNode<Object> tmp62;
  TNode<JSAny> tmp63;
      TNode<JSAny> tmp66;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_2);
    tmp58 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp51});
    tmp59 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp43}, TNode<IntPtrT>{tmp58});
    std::tie(tmp60, tmp61) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp42}, TNode<IntPtrT>{tmp59}).Flatten();
    tmp62 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp60, tmp61});
    compiler::CodeAssemblerLabel label64(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch65__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch65__label);
    tmp63 = Cast_JSAny_0(state_, TNode<Object>{tmp62}, &label64);
    }
    if (catch65__label.is_used()) {
      compiler::CodeAssemblerLabel catch65_skip(&ca_);
      ca_.Goto(&catch65_skip);
      ca_.Bind(&catch65__label, &tmp66);
      ca_.Goto(&block33, phi_bb27_2);
      ca_.Bind(&catch65_skip);
    }
    ca_.Goto(&block31, phi_bb27_2);
    if (label64.is_used()) {
      ca_.Bind(&label64);
      ca_.Goto(&block32, phi_bb27_2);
    }
  }

  TNode<BoolT> phi_bb28_2;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BoolT> phi_bb33_2;
  TNode<Union<JSMessageObject, TheHole>> tmp67;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_2);
    tmp67 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb33_2, tmp66, tmp67);
  }

  TNode<BoolT> phi_bb32_2;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BoolT> phi_bb31_2;
  TNode<IntPtrT> tmp68;
      TNode<JSAny> tmp70;
  TNode<FixedArray> tmp71;
  TNode<Union<HeapObject, TaggedIndex>> tmp72;
  TNode<IntPtrT> tmp73;
  TNode<IntPtrT> tmp74;
      TNode<JSAny> tmp76;
  TNode<IntPtrT> tmp77;
      TNode<JSAny> tmp79;
  TNode<Smi> tmp80;
  TNode<Smi> tmp81;
      TNode<JSAny> tmp83;
  TNode<Smi> tmp84;
      TNode<JSAny> tmp86;
  TNode<IntPtrT> tmp87;
  TNode<UintPtrT> tmp88;
  TNode<UintPtrT> tmp89;
  TNode<BoolT> tmp90;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch69__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch69__label);
    tmp68 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch69__label.is_used()) {
      compiler::CodeAssemblerLabel catch69_skip(&ca_);
      ca_.Goto(&catch69_skip);
      ca_.Bind(&catch69__label, &tmp70);
      ca_.Goto(&block36, phi_bb31_2);
      ca_.Bind(&catch69_skip);
    }
    tmp71 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{parameter1, tmp68});
    compiler::CodeAssemblerExceptionHandlerLabel catch75__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch75__label);
    std::tie(tmp72, tmp73, tmp74) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp71}).Flatten();
    }
    if (catch75__label.is_used()) {
      compiler::CodeAssemblerLabel catch75_skip(&ca_);
      ca_.Goto(&catch75_skip);
      ca_.Bind(&catch75__label, &tmp76);
      ca_.Goto(&block37, phi_bb31_2);
      ca_.Bind(&catch75_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch78__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch78__label);
    tmp77 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch78__label.is_used()) {
      compiler::CodeAssemblerLabel catch78_skip(&ca_);
      ca_.Goto(&catch78_skip);
      ca_.Bind(&catch78__label, &tmp79);
      ca_.Goto(&block38, phi_bb31_2);
      ca_.Bind(&catch78_skip);
    }
    tmp80 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp77});
    compiler::CodeAssemblerExceptionHandlerLabel catch82__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch82__label);
    tmp81 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch82__label.is_used()) {
      compiler::CodeAssemblerLabel catch82_skip(&ca_);
      ca_.Goto(&catch82_skip);
      ca_.Bind(&catch82__label, &tmp83);
      ca_.Goto(&block39, phi_bb31_2);
      ca_.Bind(&catch82_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch85__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch85__label);
    tmp84 = CodeStubAssembler(state_).SmiAdd(TNode<Smi>{tmp80}, TNode<Smi>{tmp81});
    }
    if (catch85__label.is_used()) {
      compiler::CodeAssemblerLabel catch85_skip(&ca_);
      ca_.Goto(&catch85_skip);
      ca_.Bind(&catch85__label, &tmp86);
      ca_.Goto(&block40, phi_bb31_2);
      ca_.Bind(&catch85_skip);
    }
    tmp87 = Convert_intptr_Smi_0(state_, TNode<Smi>{tmp84});
    tmp88 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp87});
    tmp89 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp74});
    tmp90 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp88}, TNode<UintPtrT>{tmp89});
    ca_.Branch(tmp90, &block45, std::vector<compiler::Node*>{phi_bb31_2}, &block46, std::vector<compiler::Node*>{phi_bb31_2});
  }

  TNode<BoolT> phi_bb36_2;
  TNode<Union<JSMessageObject, TheHole>> tmp91;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_2);
    tmp91 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb36_2, tmp70, tmp91);
  }

  TNode<BoolT> phi_bb37_2;
  TNode<Union<JSMessageObject, TheHole>> tmp92;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_2);
    tmp92 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb37_2, tmp76, tmp92);
  }

  TNode<BoolT> phi_bb38_2;
  TNode<Union<JSMessageObject, TheHole>> tmp93;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_2);
    tmp93 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb38_2, tmp79, tmp93);
  }

  TNode<BoolT> phi_bb39_2;
  TNode<Union<JSMessageObject, TheHole>> tmp94;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_2);
    tmp94 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb39_2, tmp83, tmp94);
  }

  TNode<BoolT> phi_bb40_2;
  TNode<Union<JSMessageObject, TheHole>> tmp95;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_2);
    tmp95 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb40_2, tmp86, tmp95);
  }

  TNode<BoolT> phi_bb45_2;
  TNode<IntPtrT> tmp96;
  TNode<IntPtrT> tmp97;
  TNode<Union<HeapObject, TaggedIndex>> tmp98;
  TNode<IntPtrT> tmp99;
  TNode<Object> tmp100;
  TNode<JSReceiver> tmp101;
      TNode<JSAny> tmp104;
  if (block45.is_used()) {
    ca_.Bind(&block45, &phi_bb45_2);
    tmp96 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp87});
    tmp97 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp73}, TNode<IntPtrT>{tmp96});
    std::tie(tmp98, tmp99) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp72}, TNode<IntPtrT>{tmp97}).Flatten();
    tmp100 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp98, tmp99});
    compiler::CodeAssemblerLabel label102(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch103__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch103__label);
    tmp101 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp100}, &label102);
    }
    if (catch103__label.is_used()) {
      compiler::CodeAssemblerLabel catch103_skip(&ca_);
      ca_.Goto(&catch103_skip);
      ca_.Bind(&catch103__label, &tmp104);
      ca_.Goto(&block51, phi_bb45_2);
      ca_.Bind(&catch103_skip);
    }
    ca_.Goto(&block49, phi_bb45_2);
    if (label102.is_used()) {
      ca_.Bind(&label102);
      ca_.Goto(&block50, phi_bb45_2);
    }
  }

  TNode<BoolT> phi_bb46_2;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BoolT> phi_bb51_2;
  TNode<Union<JSMessageObject, TheHole>> tmp105;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_2);
    tmp105 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb51_2, tmp104, tmp105);
  }

  TNode<BoolT> phi_bb50_2;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BoolT> phi_bb49_2;
  TNode<JSAny> tmp106;
      TNode<JSAny> tmp108;
  TNode<JSReceiver> tmp109;
      TNode<JSAny> tmp112;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch107__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch107__label);
    tmp106 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp63}, TNode<JSAny>{tmp101});
    }
    if (catch107__label.is_used()) {
      compiler::CodeAssemblerLabel catch107_skip(&ca_);
      ca_.Goto(&catch107_skip);
      ca_.Bind(&catch107__label, &tmp108);
      ca_.Goto(&block52, phi_bb49_2);
      ca_.Bind(&catch107_skip);
    }
    compiler::CodeAssemblerLabel label110(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch111__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch111__label);
    tmp109 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp106}, &label110);
    }
    if (catch111__label.is_used()) {
      compiler::CodeAssemblerLabel catch111_skip(&ca_);
      ca_.Goto(&catch111_skip);
      ca_.Bind(&catch111__label, &tmp112);
      ca_.Goto(&block57, phi_bb49_2);
      ca_.Bind(&catch111_skip);
    }
    ca_.Goto(&block55, phi_bb49_2);
    if (label110.is_used()) {
      ca_.Bind(&label110);
      ca_.Goto(&block56, phi_bb49_2);
    }
  }

  TNode<BoolT> phi_bb52_2;
  TNode<Union<JSMessageObject, TheHole>> tmp113;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_2);
    tmp113 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb52_2, tmp108, tmp113);
  }

  TNode<BoolT> phi_bb57_2;
  TNode<Union<JSMessageObject, TheHole>> tmp114;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_2);
    tmp114 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb57_2, tmp112, tmp114);
  }

  TNode<BoolT> phi_bb56_2;
      TNode<JSAny> tmp116;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch115__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch115__label);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kNotIterable), TNode<Object>{tmp106});
    }
    if (catch115__label.is_used()) {
      compiler::CodeAssemblerLabel catch115_skip(&ca_);
      ca_.Bind(&catch115__label, &tmp116);
      ca_.Goto(&block58, phi_bb56_2);
    }
  }

  TNode<BoolT> phi_bb55_2;
  TNode<JSReceiver> tmp117;
  TNode<JSAny> tmp118;
      TNode<JSAny> tmp120;
  TNode<IntPtrT> tmp121;
      TNode<JSAny> tmp123;
  TNode<IntPtrT> tmp124;
      TNode<JSAny> tmp126;
  TNode<IntPtrT> tmp127;
      TNode<JSAny> tmp129;
  TNode<BoolT> tmp130;
      TNode<JSAny> tmp132;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch119__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch119__label);
    std::tie(tmp117, tmp118) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp109}).Flatten();
    }
    if (catch119__label.is_used()) {
      compiler::CodeAssemblerLabel catch119_skip(&ca_);
      ca_.Goto(&catch119_skip);
      ca_.Bind(&catch119__label, &tmp120);
      ca_.Goto(&block59, phi_bb55_2);
      ca_.Bind(&catch119_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch122__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch122__label);
    tmp121 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch122__label.is_used()) {
      compiler::CodeAssemblerLabel catch122_skip(&ca_);
      ca_.Goto(&catch122_skip);
      ca_.Bind(&catch122__label, &tmp123);
      ca_.Goto(&block60, phi_bb55_2);
      ca_.Bind(&catch122_skip);
    }
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp121}, tmp117);
    compiler::CodeAssemblerExceptionHandlerLabel catch125__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch125__label);
    tmp124 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch125__label.is_used()) {
      compiler::CodeAssemblerLabel catch125_skip(&ca_);
      ca_.Goto(&catch125_skip);
      ca_.Bind(&catch125__label, &tmp126);
      ca_.Goto(&block61, phi_bb55_2);
      ca_.Bind(&catch125_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch128__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch128__label);
    tmp127 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp121}, TNode<IntPtrT>{tmp124});
    }
    if (catch128__label.is_used()) {
      compiler::CodeAssemblerLabel catch128_skip(&ca_);
      ca_.Goto(&catch128_skip);
      ca_.Bind(&catch128__label, &tmp129);
      ca_.Goto(&block62, phi_bb55_2);
      ca_.Bind(&catch128_skip);
    }
    CodeStubAssembler(state_).StoreReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp127}, tmp118);
    compiler::CodeAssemblerExceptionHandlerLabel catch131__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch131__label);
    tmp130 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    }
    if (catch131__label.is_used()) {
      compiler::CodeAssemblerLabel catch131_skip(&ca_);
      ca_.Goto(&catch131_skip);
      ca_.Bind(&catch131__label, &tmp132);
      ca_.Goto(&block63, phi_bb55_2);
      ca_.Bind(&catch131_skip);
    }
    ca_.Goto(&block8, tmp130);
  }

  TNode<BoolT> phi_bb58_2;
  TNode<Union<JSMessageObject, TheHole>> tmp133;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_2);
    tmp133 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb58_2, tmp116, tmp133);
  }

  TNode<BoolT> phi_bb59_2;
  TNode<Union<JSMessageObject, TheHole>> tmp134;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_2);
    tmp134 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb59_2, tmp120, tmp134);
  }

  TNode<BoolT> phi_bb60_2;
  TNode<Union<JSMessageObject, TheHole>> tmp135;
  if (block60.is_used()) {
    ca_.Bind(&block60, &phi_bb60_2);
    tmp135 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb60_2, tmp123, tmp135);
  }

  TNode<BoolT> phi_bb61_2;
  TNode<Union<JSMessageObject, TheHole>> tmp136;
  if (block61.is_used()) {
    ca_.Bind(&block61, &phi_bb61_2);
    tmp136 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb61_2, tmp126, tmp136);
  }

  TNode<BoolT> phi_bb62_2;
  TNode<Union<JSMessageObject, TheHole>> tmp137;
  if (block62.is_used()) {
    ca_.Bind(&block62, &phi_bb62_2);
    tmp137 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb62_2, tmp129, tmp137);
  }

  TNode<BoolT> phi_bb63_2;
  TNode<Union<JSMessageObject, TheHole>> tmp138;
  if (block63.is_used()) {
    ca_.Bind(&block63, &phi_bb63_2);
    tmp138 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb63_2, tmp132, tmp138);
  }

  TNode<BoolT> phi_bb8_2;
  TNode<Map> tmp139;
      TNode<JSAny> tmp141;
  TNode<IntPtrT> tmp142;
      TNode<JSAny> tmp144;
  TNode<JSReceiver> tmp145;
  TNode<IntPtrT> tmp146;
      TNode<JSAny> tmp148;
  TNode<IntPtrT> tmp149;
      TNode<JSAny> tmp151;
  TNode<JSAny> tmp152;
  TNode<JSAny> tmp153;
    compiler::TypedCodeAssemblerVariable<JSAny> tmp156(&ca_);
    compiler::TypedCodeAssemblerVariable<Union<JSMessageObject, TheHole>> tmp157(&ca_);
      TNode<JSAny> tmp159;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch140__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch140__label);
    tmp139 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    }
    if (catch140__label.is_used()) {
      compiler::CodeAssemblerLabel catch140_skip(&ca_);
      ca_.Goto(&catch140_skip);
      ca_.Bind(&catch140__label, &tmp141);
      ca_.Goto(&block64, phi_bb8_2);
      ca_.Bind(&catch140_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch143__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch143__label);
    tmp142 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch143__label.is_used()) {
      compiler::CodeAssemblerLabel catch143_skip(&ca_);
      ca_.Goto(&catch143_skip);
      ca_.Bind(&catch143__label, &tmp144);
      ca_.Goto(&block69, phi_bb8_2);
      ca_.Bind(&catch143_skip);
    }
    tmp145 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp142});
    compiler::CodeAssemblerExceptionHandlerLabel catch147__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch147__label);
    tmp146 = FromConstexpr_intptr_constexpr_intptr_0(state_, 4);
    }
    if (catch147__label.is_used()) {
      compiler::CodeAssemblerLabel catch147_skip(&ca_);
      ca_.Goto(&catch147_skip);
      ca_.Bind(&catch147__label, &tmp148);
      ca_.Goto(&block70, phi_bb8_2);
      ca_.Bind(&catch147_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch150__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch150__label);
    tmp149 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp142}, TNode<IntPtrT>{tmp146});
    }
    if (catch150__label.is_used()) {
      compiler::CodeAssemblerLabel catch150_skip(&ca_);
      ca_.Goto(&catch150_skip);
      ca_.Bind(&catch150__label, &tmp151);
      ca_.Goto(&block71, phi_bb8_2);
      ca_.Bind(&catch150_skip);
    }
    tmp152 = CodeStubAssembler(state_).LoadReference<JSAny>(CodeStubAssembler::Reference{parameter1, tmp149});
    compiler::CodeAssemblerLabel label154(&ca_);
    compiler::CodeAssemblerLabel label155(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch158__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch158__label);
    tmp153 = IteratorStepValue_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp145}, TNode<JSAny>{tmp152}}, TNode<Map>{tmp139}, &label154, &label155, &tmp156, &tmp157);
    }
    if (catch158__label.is_used()) {
      compiler::CodeAssemblerLabel catch158_skip(&ca_);
      ca_.Goto(&catch158_skip);
      ca_.Bind(&catch158__label, &tmp159);
      ca_.Goto(&block75, phi_bb8_2);
      ca_.Bind(&catch158_skip);
    }
    ca_.Goto(&block72, phi_bb8_2);
    if (label154.is_used()) {
      ca_.Bind(&label154);
      ca_.Goto(&block73, phi_bb8_2);
    }
    if (label155.is_used()) {
      ca_.Bind(&label155);
      ca_.Goto(&block74, phi_bb8_2);
    }
  }

  TNode<BoolT> phi_bb64_2;
  TNode<Union<JSMessageObject, TheHole>> tmp160;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_2);
    tmp160 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb64_2, tmp141, tmp160);
  }

  TNode<BoolT> phi_bb69_2;
  TNode<Union<JSMessageObject, TheHole>> tmp161;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_2);
    tmp161 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb69_2, tmp144, tmp161);
  }

  TNode<BoolT> phi_bb70_2;
  TNode<Union<JSMessageObject, TheHole>> tmp162;
  if (block70.is_used()) {
    ca_.Bind(&block70, &phi_bb70_2);
    tmp162 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb70_2, tmp148, tmp162);
  }

  TNode<BoolT> phi_bb71_2;
  TNode<Union<JSMessageObject, TheHole>> tmp163;
  if (block71.is_used()) {
    ca_.Bind(&block71, &phi_bb71_2);
    tmp163 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb71_2, tmp151, tmp163);
  }

  TNode<BoolT> phi_bb75_2;
  TNode<Union<JSMessageObject, TheHole>> tmp164;
  if (block75.is_used()) {
    ca_.Bind(&block75, &phi_bb75_2);
    tmp164 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb75_2, tmp159, tmp164);
  }

  TNode<BoolT> phi_bb73_2;
  TNode<BoolT> tmp165;
      TNode<JSAny> tmp167;
  TNode<IntPtrT> tmp168;
      TNode<JSAny> tmp170;
  TNode<IntPtrT> tmp171;
      TNode<JSAny> tmp173;
  TNode<Smi> tmp174;
  TNode<Smi> tmp175;
      TNode<JSAny> tmp177;
  TNode<Smi> tmp178;
      TNode<JSAny> tmp180;
  if (block73.is_used()) {
    ca_.Bind(&block73, &phi_bb73_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch166__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch166__label);
    tmp165 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    }
    if (catch166__label.is_used()) {
      compiler::CodeAssemblerLabel catch166_skip(&ca_);
      ca_.Goto(&catch166_skip);
      ca_.Bind(&catch166__label, &tmp167);
      ca_.Goto(&block82, phi_bb73_2);
      ca_.Bind(&catch166_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch169__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch169__label);
    tmp168 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch169__label.is_used()) {
      compiler::CodeAssemblerLabel catch169_skip(&ca_);
      ca_.Goto(&catch169_skip);
      ca_.Bind(&catch169__label, &tmp170);
      ca_.Goto(&block83);
      ca_.Bind(&catch169_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch172__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch172__label);
    tmp171 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch172__label.is_used()) {
      compiler::CodeAssemblerLabel catch172_skip(&ca_);
      ca_.Goto(&catch172_skip);
      ca_.Bind(&catch172__label, &tmp173);
      ca_.Goto(&block84);
      ca_.Bind(&catch172_skip);
    }
    tmp174 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp171});
    compiler::CodeAssemblerExceptionHandlerLabel catch176__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch176__label);
    tmp175 = FromConstexpr_Smi_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch176__label.is_used()) {
      compiler::CodeAssemblerLabel catch176_skip(&ca_);
      ca_.Goto(&catch176_skip);
      ca_.Bind(&catch176__label, &tmp177);
      ca_.Goto(&block85);
      ca_.Bind(&catch176_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch179__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch179__label);
    tmp178 = CodeStubAssembler(state_).SmiAdd(TNode<Smi>{tmp174}, TNode<Smi>{tmp175});
    }
    if (catch179__label.is_used()) {
      compiler::CodeAssemblerLabel catch179_skip(&ca_);
      ca_.Goto(&catch179_skip);
      ca_.Bind(&catch179__label, &tmp180);
      ca_.Goto(&block86);
      ca_.Bind(&catch179_skip);
    }
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp168}, tmp178);
    ca_.Goto(&block5, tmp165);
  }

  TNode<BoolT> phi_bb74_2;
  TNode<IntPtrT> tmp181;
      TNode<JSAny> tmp183;
  TNode<JSReceiver> tmp184;
      TNode<JSAny> tmp186;
      TNode<JSAny> tmp188;
      TNode<JSAny> tmp190;
  if (block74.is_used()) {
    ca_.Bind(&block74, &phi_bb74_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch182__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch182__label);
    tmp181 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch182__label.is_used()) {
      compiler::CodeAssemblerLabel catch182_skip(&ca_);
      ca_.Goto(&catch182_skip);
      ca_.Bind(&catch182__label, &tmp183);
      ca_.Goto(&block78, phi_bb74_2);
      ca_.Bind(&catch182_skip);
    }
    tmp184 = CodeStubAssembler(state_).LoadReference<JSReceiver>(CodeStubAssembler::Reference{parameter1, tmp181});
    compiler::CodeAssemblerExceptionHandlerLabel catch185__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch185__label);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp184});
    }
    if (catch185__label.is_used()) {
      compiler::CodeAssemblerLabel catch185_skip(&ca_);
      ca_.Goto(&catch185_skip);
      ca_.Bind(&catch185__label, &tmp186);
      ca_.Goto(&block79, phi_bb74_2);
      ca_.Bind(&catch185_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch187__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch187__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch187__label.is_used()) {
      compiler::CodeAssemblerLabel catch187_skip(&ca_);
      ca_.Goto(&catch187_skip);
      ca_.Bind(&catch187__label, &tmp188);
      ca_.Goto(&block80, phi_bb74_2);
      ca_.Bind(&catch187_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch189__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch189__label);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp156.value(), tmp157.value());
    CodeStubAssembler(state_).Unreachable();
    }
    if (catch189__label.is_used()) {
      compiler::CodeAssemblerLabel catch189_skip(&ca_);
      ca_.Bind(&catch189__label, &tmp190);
      ca_.Goto(&block81, phi_bb74_2);
    }
  }

  TNode<BoolT> phi_bb72_2;
      TNode<JSAny> tmp192;
  TNode<False> tmp193;
  TNode<JSObject> tmp194;
      TNode<JSAny> tmp196;
  if (block72.is_used()) {
    ca_.Bind(&block72, &phi_bb72_2);
    compiler::CodeAssemblerExceptionHandlerLabel catch191__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch191__label);
    MarkIteratorHelperAsFinishedExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch191__label.is_used()) {
      compiler::CodeAssemblerLabel catch191_skip(&ca_);
      ca_.Goto(&catch191_skip);
      ca_.Bind(&catch191__label, &tmp192);
      ca_.Goto(&block76, phi_bb72_2);
      ca_.Bind(&catch191_skip);
    }
    tmp193 = False_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch195__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch195__label);
    tmp194 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp153}, TNode<Boolean>{tmp193});
    }
    if (catch195__label.is_used()) {
      compiler::CodeAssemblerLabel catch195_skip(&ca_);
      ca_.Goto(&catch195_skip);
      ca_.Bind(&catch195__label, &tmp196);
      ca_.Goto(&block77, phi_bb72_2);
      ca_.Bind(&catch195_skip);
    }
    CodeStubAssembler(state_).Return(tmp194);
  }

  TNode<BoolT> phi_bb76_2;
  TNode<Union<JSMessageObject, TheHole>> tmp197;
  if (block76.is_used()) {
    ca_.Bind(&block76, &phi_bb76_2);
    tmp197 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb76_2, tmp192, tmp197);
  }

  TNode<BoolT> phi_bb77_2;
  TNode<Union<JSMessageObject, TheHole>> tmp198;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_2);
    tmp198 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb77_2, tmp196, tmp198);
  }

  TNode<BoolT> phi_bb78_2;
  TNode<Union<JSMessageObject, TheHole>> tmp199;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_2);
    tmp199 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb78_2, tmp183, tmp199);
  }

  TNode<BoolT> phi_bb79_2;
  TNode<Union<JSMessageObject, TheHole>> tmp200;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_2);
    tmp200 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb79_2, tmp186, tmp200);
  }

  TNode<BoolT> phi_bb80_2;
  TNode<Union<JSMessageObject, TheHole>> tmp201;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_2);
    tmp201 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb80_2, tmp188, tmp201);
  }

  TNode<BoolT> phi_bb81_2;
  TNode<Union<JSMessageObject, TheHole>> tmp202;
  if (block81.is_used()) {
    ca_.Bind(&block81, &phi_bb81_2);
    tmp202 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb81_2, tmp190, tmp202);
  }

  TNode<BoolT> phi_bb82_2;
  TNode<Union<JSMessageObject, TheHole>> tmp203;
  if (block82.is_used()) {
    ca_.Bind(&block82, &phi_bb82_2);
    tmp203 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, phi_bb82_2, tmp167, tmp203);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp204;
  if (block83.is_used()) {
    ca_.Bind(&block83);
    tmp204 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp165, tmp170, tmp204);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp205;
  if (block84.is_used()) {
    ca_.Bind(&block84);
    tmp205 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp165, tmp173, tmp205);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp206;
  if (block85.is_used()) {
    ca_.Bind(&block85);
    tmp206 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp165, tmp177, tmp206);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp207;
  if (block86.is_used()) {
    ca_.Bind(&block86);
    tmp207 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block2, tmp165, tmp180, tmp207);
  }

  TNode<BoolT> phi_bb4_2;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_2);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BoolT> phi_bb2_2;
  TNode<JSAny> phi_bb2_3;
  TNode<Union<JSMessageObject, TheHole>> phi_bb2_4;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_2, &phi_bb2_3, &phi_bb2_4);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb2_3, phi_bb2_4);
    CodeStubAssembler(state_).Unreachable();
  }
}

TF_BUILTIN(IteratorPrototypeJoin, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kSeparator);
  USE(parameter2);
  CodeStubAssembler(state_).CallRuntime(Runtime::kIncrementUseCounter, parameter0, CodeStubAssembler(state_).SmiConstant(v8::Isolate::kIteratorMethods));
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<String> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block31(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<String> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String, BoolT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1}, &label1);
    ca_.Goto(&block3);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block4);
    }
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.prototype.join");
  }

  TNode<Undefined> tmp2;
  TNode<BoolT> tmp3;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp2 = Undefined_0(state_);
    tmp3 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{parameter2}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp2});
    ca_.Branch(tmp3, &block5, std::vector<compiler::Node*>{}, &block6, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp4;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp4 = FromConstexpr_String_constexpr_string_0(state_, ",");
    ca_.Goto(&block7, tmp4);
  }

  TNode<String> tmp5;
      TNode<JSAny> tmp7;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    compiler::CodeAssemblerExceptionHandlerLabel catch6__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch6__label);
    tmp5 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter2});
    }
    if (catch6__label.is_used()) {
      compiler::CodeAssemblerLabel catch6_skip(&ca_);
      ca_.Goto(&catch6_skip);
      ca_.Bind(&catch6__label, &tmp7);
      ca_.Goto(&block10);
      ca_.Bind(&catch6_skip);
    }
    ca_.Goto(&block7, tmp5);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp8;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp8 = GetAndResetPendingMessage_0(state_);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp7, tmp8);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> phi_bb7_4;
  TNode<JSReceiver> tmp9;
  TNode<JSAny> tmp10;
  TNode<String> tmp11;
  TNode<BoolT> tmp12;
  TNode<Map> tmp13;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_4);
    std::tie(tmp9, tmp10) = GetIteratorDirect_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0}).Flatten();
    tmp11 = kEmptyString_0(state_);
    tmp12 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp13 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    ca_.Goto(&block13, tmp11, tmp12);
  }

  TNode<String> phi_bb13_7;
  TNode<BoolT> phi_bb13_8;
  TNode<BoolT> tmp14;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_7, &phi_bb13_8);
    tmp14 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp14, &block11, std::vector<compiler::Node*>{phi_bb13_7, phi_bb13_8}, &block12, std::vector<compiler::Node*>{phi_bb13_7, phi_bb13_8});
  }

  TNode<String> phi_bb11_7;
  TNode<BoolT> phi_bb11_8;
  TNode<JSAny> tmp15;
    compiler::TypedCodeAssemblerVariable<JSAny> tmp18(&ca_);
    compiler::TypedCodeAssemblerVariable<Union<JSMessageObject, TheHole>> tmp19(&ca_);
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_7, &phi_bb11_8);
    compiler::CodeAssemblerLabel label16(&ca_);
    compiler::CodeAssemblerLabel label17(&ca_);
    tmp15 = IteratorStepValue_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp9}, TNode<JSAny>{tmp10}}, TNode<Map>{tmp13}, &label16, &label17, &tmp18, &tmp19);
    ca_.Goto(&block18, phi_bb11_7, phi_bb11_8);
    if (label16.is_used()) {
      ca_.Bind(&label16);
      ca_.Goto(&block19, phi_bb11_7, phi_bb11_8);
    }
    if (label17.is_used()) {
      ca_.Bind(&label17);
      ca_.Goto(&block20, phi_bb11_7, phi_bb11_8);
    }
  }

  TNode<String> phi_bb19_7;
  TNode<BoolT> phi_bb19_8;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_7, &phi_bb19_8);
    CodeStubAssembler(state_).Return(phi_bb19_7);
  }

  TNode<String> phi_bb20_7;
  TNode<BoolT> phi_bb20_8;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_7, &phi_bb20_8);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp18.value(), tmp19.value());
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> phi_bb18_7;
  TNode<BoolT> phi_bb18_8;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_7, &phi_bb18_8);
    ca_.Branch(phi_bb18_8, &block21, std::vector<compiler::Node*>{phi_bb18_7, phi_bb18_8}, &block22, std::vector<compiler::Node*>{phi_bb18_7, phi_bb18_8});
  }

  TNode<String> phi_bb21_7;
  TNode<BoolT> phi_bb21_8;
  TNode<BoolT> tmp20;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_7, &phi_bb21_8);
    tmp20 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block23, phi_bb21_7, tmp20);
  }

  TNode<String> phi_bb22_7;
  TNode<BoolT> phi_bb22_8;
  TNode<String> tmp21;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_7, &phi_bb22_8);
    tmp21 = StringAdd_0(state_, TNode<Context>{parameter0}, TNode<String>{phi_bb22_7}, TNode<String>{phi_bb7_4});
    ca_.Goto(&block23, tmp21, phi_bb22_8);
  }

  TNode<String> phi_bb23_7;
  TNode<BoolT> phi_bb23_8;
  TNode<Undefined> tmp22;
  TNode<BoolT> tmp23;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_7, &phi_bb23_8);
    tmp22 = Undefined_0(state_);
    tmp23 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Object>{tmp15}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp22});
    ca_.Branch(tmp23, &block26, std::vector<compiler::Node*>{}, &block27, std::vector<compiler::Node*>{});
  }

  TNode<Null> tmp24;
  TNode<BoolT> tmp25;
  if (block26.is_used()) {
    ca_.Bind(&block26);
    tmp24 = Null_0(state_);
    tmp25 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Object>{tmp15}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp24});
    ca_.Goto(&block28, tmp25);
  }

  TNode<BoolT> tmp26;
  if (block27.is_used()) {
    ca_.Bind(&block27);
    tmp26 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block28, tmp26);
  }

  TNode<BoolT> phi_bb28_12;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_12);
    ca_.Branch(phi_bb28_12, &block24, std::vector<compiler::Node*>{}, &block25, std::vector<compiler::Node*>{phi_bb23_7});
  }

  TNode<String> tmp27;
      TNode<JSAny> tmp29;
  TNode<String> tmp30;
  if (block24.is_used()) {
    ca_.Bind(&block24);
    compiler::CodeAssemblerExceptionHandlerLabel catch28__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch28__label);
    tmp27 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{tmp15});
    }
    if (catch28__label.is_used()) {
      compiler::CodeAssemblerLabel catch28_skip(&ca_);
      ca_.Goto(&catch28_skip);
      ca_.Bind(&catch28__label, &tmp29);
      ca_.Goto(&block31);
      ca_.Bind(&catch28_skip);
    }
    tmp30 = StringAdd_0(state_, TNode<Context>{parameter0}, TNode<String>{phi_bb23_7}, TNode<String>{tmp27});
    ca_.Goto(&block25, tmp30);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp31;
  if (block31.is_used()) {
    ca_.Bind(&block31);
    tmp31 = GetAndResetPendingMessage_0(state_);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp0});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp29, tmp31);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> phi_bb25_7;
  if (block25.is_used()) {
    ca_.Bind(&block25, &phi_bb25_7);
    ca_.Goto(&block13, phi_bb25_7, phi_bb23_8);
  }

  TNode<String> phi_bb12_7;
  TNode<BoolT> phi_bb12_8;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_7, &phi_bb12_8);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1456&c=1
void IteratorZipCloseAll_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArray> p_iterators, bool p_propagate) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block39(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT, IntPtrT, IntPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT, IntPtrT> block41(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block52(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block53(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT, JSAny, Union<JSMessageObject, TheHole>> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block60(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block61(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Undefined> tmp0;
  TNode<TheHole> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Smi> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Undefined_0(state_);
    tmp1 = TheHole_0(state_);
    tmp2 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    tmp3 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{p_iterators, tmp2});
    tmp4 = CodeStubAssembler(state_).SmiUntag(TNode<Smi>{tmp3});
    tmp5 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    tmp6 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp4}, TNode<IntPtrT>{tmp5});
    ca_.Goto(&block8, tmp0, tmp1, tmp6);
  }

  TNode<JSAny> phi_bb8_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb8_3;
  TNode<IntPtrT> phi_bb8_4;
  TNode<IntPtrT> tmp7;
  TNode<BoolT> tmp8;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_2, &phi_bb8_3, &phi_bb8_4);
    tmp7 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp8 = CodeStubAssembler(state_).IntPtrGreaterThanOrEqual(TNode<IntPtrT>{phi_bb8_4}, TNode<IntPtrT>{tmp7});
    ca_.Branch(tmp8, &block6, std::vector<compiler::Node*>{phi_bb8_2, phi_bb8_3, phi_bb8_4}, &block7, std::vector<compiler::Node*>{phi_bb8_2, phi_bb8_3, phi_bb8_4});
  }

  TNode<JSAny> phi_bb6_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb6_3;
  TNode<IntPtrT> phi_bb6_4;
  TNode<Union<HeapObject, TaggedIndex>> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<UintPtrT> tmp12;
  TNode<UintPtrT> tmp13;
  TNode<BoolT> tmp14;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_2, &phi_bb6_3, &phi_bb6_4);
    std::tie(tmp9, tmp10, tmp11) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{p_iterators}).Flatten();
    tmp12 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb6_4});
    tmp13 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp11});
    tmp14 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp12}, TNode<UintPtrT>{tmp13});
    ca_.Branch(tmp14, &block14, std::vector<compiler::Node*>{phi_bb6_2, phi_bb6_3, phi_bb6_4, phi_bb6_4, phi_bb6_4, phi_bb6_4, phi_bb6_4}, &block15, std::vector<compiler::Node*>{phi_bb6_2, phi_bb6_3, phi_bb6_4, phi_bb6_4, phi_bb6_4, phi_bb6_4, phi_bb6_4});
  }

  TNode<JSAny> phi_bb14_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb14_3;
  TNode<IntPtrT> phi_bb14_4;
  TNode<IntPtrT> phi_bb14_9;
  TNode<IntPtrT> phi_bb14_10;
  TNode<IntPtrT> phi_bb14_14;
  TNode<IntPtrT> phi_bb14_15;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<Union<HeapObject, TaggedIndex>> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<Object> tmp19;
  TNode<TheHole> tmp20;
  TNode<BoolT> tmp21;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_2, &phi_bb14_3, &phi_bb14_4, &phi_bb14_9, &phi_bb14_10, &phi_bb14_14, &phi_bb14_15);
    tmp15 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb14_15});
    tmp16 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp10}, TNode<IntPtrT>{tmp15});
    std::tie(tmp17, tmp18) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp9}, TNode<IntPtrT>{tmp16}).Flatten();
    tmp19 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp17, tmp18});
    tmp20 = TheHole_0(state_);
    tmp21 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Object>{tmp19}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp20});
    ca_.Branch(tmp21, &block18, std::vector<compiler::Node*>{phi_bb14_2, phi_bb14_3, phi_bb14_4}, &block19, std::vector<compiler::Node*>{phi_bb14_2, phi_bb14_3, phi_bb14_4});
  }

  TNode<JSAny> phi_bb15_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb15_3;
  TNode<IntPtrT> phi_bb15_4;
  TNode<IntPtrT> phi_bb15_9;
  TNode<IntPtrT> phi_bb15_10;
  TNode<IntPtrT> phi_bb15_14;
  TNode<IntPtrT> phi_bb15_15;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_2, &phi_bb15_3, &phi_bb15_4, &phi_bb15_9, &phi_bb15_10, &phi_bb15_14, &phi_bb15_15);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb18_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb18_3;
  TNode<IntPtrT> phi_bb18_4;
  TNode<JSReceiver> tmp22;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_2, &phi_bb18_3, &phi_bb18_4);
    compiler::CodeAssemblerLabel label23(&ca_);
    tmp22 = Cast_JSReceiver_1(state_, TNode<Context>{p_context}, TNode<Object>{tmp19}, &label23);
    ca_.Goto(&block22, phi_bb18_2, phi_bb18_3, phi_bb18_4);
    if (label23.is_used()) {
      ca_.Bind(&label23);
      ca_.Goto(&block23, phi_bb18_2, phi_bb18_3, phi_bb18_4);
    }
  }

  TNode<JSAny> phi_bb23_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb23_3;
  TNode<IntPtrT> phi_bb23_4;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_2, &phi_bb23_3, &phi_bb23_4);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb22_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb22_3;
  TNode<IntPtrT> phi_bb22_4;
  TNode<Union<HeapObject, TaggedIndex>> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<IntPtrT> tmp26;
  TNode<UintPtrT> tmp27;
  TNode<UintPtrT> tmp28;
  TNode<BoolT> tmp29;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_2, &phi_bb22_3, &phi_bb22_4);
    std::tie(tmp24, tmp25, tmp26) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{p_iterators}).Flatten();
    tmp27 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb22_4});
    tmp28 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp26});
    tmp29 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp27}, TNode<UintPtrT>{tmp28});
    ca_.Branch(tmp29, &block28, std::vector<compiler::Node*>{phi_bb22_2, phi_bb22_3, phi_bb22_4, phi_bb22_4, phi_bb22_4, phi_bb22_4, phi_bb22_4}, &block29, std::vector<compiler::Node*>{phi_bb22_2, phi_bb22_3, phi_bb22_4, phi_bb22_4, phi_bb22_4, phi_bb22_4, phi_bb22_4});
  }

  TNode<JSAny> phi_bb28_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb28_3;
  TNode<IntPtrT> phi_bb28_4;
  TNode<IntPtrT> phi_bb28_11;
  TNode<IntPtrT> phi_bb28_12;
  TNode<IntPtrT> phi_bb28_16;
  TNode<IntPtrT> phi_bb28_17;
  TNode<IntPtrT> tmp30;
  TNode<IntPtrT> tmp31;
  TNode<Union<HeapObject, TaggedIndex>> tmp32;
  TNode<IntPtrT> tmp33;
  TNode<TheHole> tmp34;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_2, &phi_bb28_3, &phi_bb28_4, &phi_bb28_11, &phi_bb28_12, &phi_bb28_16, &phi_bb28_17);
    tmp30 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb28_17});
    tmp31 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp25}, TNode<IntPtrT>{tmp30});
    std::tie(tmp32, tmp33) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp24}, TNode<IntPtrT>{tmp31}).Flatten();
    tmp34 = TheHole_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp32, tmp33}, tmp34);
    if ((p_propagate)) {
      ca_.Goto(&block32, phi_bb28_2, phi_bb28_3, phi_bb28_4);
    } else {
      ca_.Goto(&block33, phi_bb28_2, phi_bb28_3, phi_bb28_4);
    }
  }

  TNode<JSAny> phi_bb29_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb29_3;
  TNode<IntPtrT> phi_bb29_4;
  TNode<IntPtrT> phi_bb29_11;
  TNode<IntPtrT> phi_bb29_12;
  TNode<IntPtrT> phi_bb29_16;
  TNode<IntPtrT> phi_bb29_17;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_2, &phi_bb29_3, &phi_bb29_4, &phi_bb29_11, &phi_bb29_12, &phi_bb29_16, &phi_bb29_17);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb32_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb32_3;
  TNode<IntPtrT> phi_bb32_4;
  TNode<Union<HeapObject, TaggedIndex>> tmp35;
  TNode<IntPtrT> tmp36;
  TNode<IntPtrT> tmp37;
      TNode<JSAny> tmp39;
  TNode<IntPtrT> tmp40;
      TNode<JSAny> tmp42;
  TNode<IntPtrT> tmp43;
      TNode<JSAny> tmp45;
  TNode<UintPtrT> tmp46;
  TNode<UintPtrT> tmp47;
  TNode<BoolT> tmp48;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_2, &phi_bb32_3, &phi_bb32_4);
    compiler::CodeAssemblerExceptionHandlerLabel catch38__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch38__label);
    std::tie(tmp35, tmp36, tmp37) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{p_iterators}).Flatten();
    }
    if (catch38__label.is_used()) {
      compiler::CodeAssemblerLabel catch38_skip(&ca_);
      ca_.Goto(&catch38_skip);
      ca_.Bind(&catch38__label, &tmp39);
      ca_.Goto(&block39, phi_bb32_2, phi_bb32_3, phi_bb32_4);
      ca_.Bind(&catch38_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch41__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch41__label);
    tmp40 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch41__label.is_used()) {
      compiler::CodeAssemblerLabel catch41_skip(&ca_);
      ca_.Goto(&catch41_skip);
      ca_.Bind(&catch41__label, &tmp42);
      ca_.Goto(&block40, phi_bb32_2, phi_bb32_3, phi_bb32_4, phi_bb32_4, phi_bb32_4);
      ca_.Bind(&catch41_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch44__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch44__label);
    tmp43 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb32_4}, TNode<IntPtrT>{tmp40});
    }
    if (catch44__label.is_used()) {
      compiler::CodeAssemblerLabel catch44_skip(&ca_);
      ca_.Goto(&catch44_skip);
      ca_.Bind(&catch44__label, &tmp45);
      ca_.Goto(&block41, phi_bb32_2, phi_bb32_3, phi_bb32_4, phi_bb32_4);
      ca_.Bind(&catch44_skip);
    }
    tmp46 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp43});
    tmp47 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp37});
    tmp48 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp46}, TNode<UintPtrT>{tmp47});
    ca_.Branch(tmp48, &block46, std::vector<compiler::Node*>{phi_bb32_2, phi_bb32_3, phi_bb32_4}, &block47, std::vector<compiler::Node*>{phi_bb32_2, phi_bb32_3, phi_bb32_4});
  }

  TNode<JSAny> phi_bb39_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb39_3;
  TNode<IntPtrT> phi_bb39_4;
  TNode<Union<JSMessageObject, TheHole>> tmp49;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_2, &phi_bb39_3, &phi_bb39_4);
    tmp49 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block36, phi_bb39_2, phi_bb39_3, phi_bb39_4, tmp39, tmp49);
  }

  TNode<JSAny> phi_bb40_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb40_3;
  TNode<IntPtrT> phi_bb40_4;
  TNode<IntPtrT> phi_bb40_11;
  TNode<IntPtrT> phi_bb40_12;
  TNode<Union<JSMessageObject, TheHole>> tmp50;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_2, &phi_bb40_3, &phi_bb40_4, &phi_bb40_11, &phi_bb40_12);
    tmp50 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block36, phi_bb40_2, phi_bb40_3, phi_bb40_4, tmp42, tmp50);
  }

  TNode<JSAny> phi_bb41_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb41_3;
  TNode<IntPtrT> phi_bb41_4;
  TNode<IntPtrT> phi_bb41_11;
  TNode<Union<JSMessageObject, TheHole>> tmp51;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_2, &phi_bb41_3, &phi_bb41_4, &phi_bb41_11);
    tmp51 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block36, phi_bb41_2, phi_bb41_3, phi_bb41_4, tmp45, tmp51);
  }

  TNode<JSAny> phi_bb46_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb46_3;
  TNode<IntPtrT> phi_bb46_4;
  TNode<IntPtrT> tmp52;
  TNode<IntPtrT> tmp53;
  TNode<Union<HeapObject, TaggedIndex>> tmp54;
  TNode<IntPtrT> tmp55;
  TNode<Object> tmp56;
  TNode<JSAny> tmp57;
      TNode<JSAny> tmp60;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_2, &phi_bb46_3, &phi_bb46_4);
    tmp52 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp43});
    tmp53 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp36}, TNode<IntPtrT>{tmp52});
    std::tie(tmp54, tmp55) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp35}, TNode<IntPtrT>{tmp53}).Flatten();
    tmp56 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp54, tmp55});
    compiler::CodeAssemblerLabel label58(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch59__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch59__label);
    tmp57 = Cast_JSAny_0(state_, TNode<Object>{tmp56}, &label58);
    }
    if (catch59__label.is_used()) {
      compiler::CodeAssemblerLabel catch59_skip(&ca_);
      ca_.Goto(&catch59_skip);
      ca_.Bind(&catch59__label, &tmp60);
      ca_.Goto(&block52, phi_bb46_2, phi_bb46_3, phi_bb46_4);
      ca_.Bind(&catch59_skip);
    }
    ca_.Goto(&block50, phi_bb46_2, phi_bb46_3, phi_bb46_4);
    if (label58.is_used()) {
      ca_.Bind(&label58);
      ca_.Goto(&block51, phi_bb46_2, phi_bb46_3, phi_bb46_4);
    }
  }

  TNode<JSAny> phi_bb47_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb47_3;
  TNode<IntPtrT> phi_bb47_4;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_2, &phi_bb47_3, &phi_bb47_4);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb52_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb52_3;
  TNode<IntPtrT> phi_bb52_4;
  TNode<Union<JSMessageObject, TheHole>> tmp61;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_2, &phi_bb52_3, &phi_bb52_4);
    tmp61 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block36, phi_bb52_2, phi_bb52_3, phi_bb52_4, tmp60, tmp61);
  }

  TNode<JSAny> phi_bb51_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb51_3;
  TNode<IntPtrT> phi_bb51_4;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_2, &phi_bb51_3, &phi_bb51_4);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb50_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb50_3;
  TNode<IntPtrT> phi_bb50_4;
      TNode<JSAny> tmp63;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_2, &phi_bb50_3, &phi_bb50_4);
    compiler::CodeAssemblerExceptionHandlerLabel catch62__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch62__label);
    IteratorClose_0(state_, TNode<Context>{p_context}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp22}, TNode<JSAny>{tmp57}});
    }
    if (catch62__label.is_used()) {
      compiler::CodeAssemblerLabel catch62_skip(&ca_);
      ca_.Goto(&catch62_skip);
      ca_.Bind(&catch62__label, &tmp63);
      ca_.Goto(&block53, phi_bb50_2, phi_bb50_3, phi_bb50_4);
      ca_.Bind(&catch62_skip);
    }
    ca_.Goto(&block35, phi_bb50_2, phi_bb50_3, phi_bb50_4);
  }

  TNode<JSAny> phi_bb53_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb53_3;
  TNode<IntPtrT> phi_bb53_4;
  TNode<Union<JSMessageObject, TheHole>> tmp64;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_2, &phi_bb53_3, &phi_bb53_4);
    tmp64 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block36, phi_bb53_2, phi_bb53_3, phi_bb53_4, tmp63, tmp64);
  }

  TNode<JSAny> phi_bb36_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb36_3;
  TNode<IntPtrT> phi_bb36_4;
  TNode<JSAny> phi_bb36_7;
  TNode<Union<JSMessageObject, TheHole>> phi_bb36_8;
  TNode<Undefined> tmp65;
  TNode<BoolT> tmp66;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_2, &phi_bb36_3, &phi_bb36_4, &phi_bb36_7, &phi_bb36_8);
    tmp65 = Undefined_0(state_);
    tmp66 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{phi_bb36_2}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp65});
    ca_.Branch(tmp66, &block54, std::vector<compiler::Node*>{phi_bb36_2, phi_bb36_3, phi_bb36_4}, &block55, std::vector<compiler::Node*>{phi_bb36_2, phi_bb36_3, phi_bb36_4});
  }

  TNode<JSAny> phi_bb54_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb54_3;
  TNode<IntPtrT> phi_bb54_4;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_2, &phi_bb54_3, &phi_bb54_4);
    ca_.Goto(&block55, phi_bb36_7, phi_bb36_8, phi_bb54_4);
  }

  TNode<JSAny> phi_bb55_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb55_3;
  TNode<IntPtrT> phi_bb55_4;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_2, &phi_bb55_3, &phi_bb55_4);
    ca_.Goto(&block35, phi_bb55_2, phi_bb55_3, phi_bb55_4);
  }

  TNode<JSAny> phi_bb35_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb35_3;
  TNode<IntPtrT> phi_bb35_4;
  if (block35.is_used()) {
    ca_.Bind(&block35, &phi_bb35_2, &phi_bb35_3, &phi_bb35_4);
    ca_.Goto(&block34, phi_bb35_2, phi_bb35_3, phi_bb35_4);
  }

  TNode<JSAny> phi_bb33_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb33_3;
  TNode<IntPtrT> phi_bb33_4;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_2, &phi_bb33_3, &phi_bb33_4);
    IteratorCloseOnException_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{tmp22});
    ca_.Goto(&block34, phi_bb33_2, phi_bb33_3, phi_bb33_4);
  }

  TNode<JSAny> phi_bb34_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb34_3;
  TNode<IntPtrT> phi_bb34_4;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_2, &phi_bb34_3, &phi_bb34_4);
    ca_.Goto(&block19, phi_bb34_2, phi_bb34_3, phi_bb34_4);
  }

  TNode<JSAny> phi_bb19_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb19_3;
  TNode<IntPtrT> phi_bb19_4;
  TNode<IntPtrT> tmp67;
  TNode<IntPtrT> tmp68;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_2, &phi_bb19_3, &phi_bb19_4);
    tmp67 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    tmp68 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb19_4}, TNode<IntPtrT>{tmp67});
    ca_.Goto(&block8, phi_bb19_2, phi_bb19_3, tmp68);
  }

  TNode<JSAny> phi_bb7_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb7_3;
  TNode<IntPtrT> phi_bb7_4;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_2, &phi_bb7_3, &phi_bb7_4);
    if ((p_propagate)) {
      ca_.Goto(&block56, phi_bb7_2, phi_bb7_3);
    } else {
      ca_.Goto(&block57, phi_bb7_2, phi_bb7_3);
    }
  }

  TNode<JSAny> phi_bb56_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb56_3;
  TNode<Undefined> tmp69;
  TNode<BoolT> tmp70;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_2, &phi_bb56_3);
    tmp69 = Undefined_0(state_);
    tmp70 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Object>{phi_bb56_2}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp69});
    ca_.Branch(tmp70, &block59, std::vector<compiler::Node*>{phi_bb56_2, phi_bb56_3}, &block60, std::vector<compiler::Node*>{phi_bb56_2, phi_bb56_3});
  }

  TNode<JSAny> phi_bb59_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb59_3;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_2, &phi_bb59_3);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, p_context, phi_bb59_2, phi_bb59_3);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> phi_bb60_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb60_3;
  if (block60.is_used()) {
    ca_.Bind(&block60, &phi_bb60_2, &phi_bb60_3);
    ca_.Goto(&block58, phi_bb60_2, phi_bb60_3);
  }

  TNode<JSAny> phi_bb57_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb57_3;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_2, &phi_bb57_3);
    ca_.Goto(&block58, phi_bb57_2, phi_bb57_3);
  }

  TNode<JSAny> phi_bb58_2;
  TNode<Union<JSMessageObject, TheHole>> phi_bb58_3;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_2, &phi_bb58_3);
    ca_.Goto(&block61);
  }

    ca_.Bind(&block61);
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1491&c=1
TNode<JSIteratorZipHelper> NewJSIteratorZipHelper_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArray> p_iterators, TNode<FixedArray> p_padding, TNode<Uint32T> p_zipMode) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Map> tmp3;
  TNode<FixedArray> tmp4;
  TNode<FixedArray> tmp5;
  TNode<Uint32T> tmp6;
  TNode<Smi> tmp7;
  TNode<Smi> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<Smi> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<Smi> tmp14;
  TNode<BoolT> tmp15;
  TNode<BoolT> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<HeapObject> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<IntPtrT> tmp20;
  TNode<IntPtrT> tmp21;
  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<IntPtrT> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<IntPtrT> tmp26;
  TNode<JSIteratorZipHelper> tmp27;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ITERATOR_ZIP_HELPER_MAP_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Map_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = kEmptyFixedArray_0(state_);
    tmp5 = kEmptyFixedArray_0(state_);
    tmp6 = FromConstexpr_JSIteratorHelperState_constexpr_kSuspendedStart_0(state_, JSIteratorHelperState::kSuspendedStart);
    tmp7 = SmiTag_JSIteratorHelperState_0(state_, TNode<Uint32T>{tmp6});
    tmp8 = SmiTag_JSIteratorZipHelperMode_0(state_, TNode<Uint32T>{p_zipMode});
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    tmp10 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{p_iterators, tmp9});
    tmp11 = CodeStubAssembler(state_).SmiUntag(TNode<Smi>{tmp10});
    tmp12 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    tmp13 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp11}, TNode<IntPtrT>{tmp12});
    tmp14 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{tmp13});
    tmp15 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp16 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, 32);
    tmp18 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp17}, TNode<Map>{tmp3}, TNode<BoolT>{tmp15}, TNode<BoolT>{tmp16});
    tmp19 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp18, tmp19}, tmp3);
    tmp20 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>>(CodeStubAssembler::Reference{tmp18, tmp20}, tmp4);
    tmp21 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp18, tmp21}, tmp5);
    tmp22 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp18, tmp22}, tmp7);
    tmp23 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<FixedArray>(CodeStubAssembler::Reference{tmp18, tmp23}, p_iterators);
    tmp24 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp18, tmp24}, tmp8);
    tmp25 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp18, tmp25}, tmp14);
    tmp26 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    CodeStubAssembler(state_).StoreReference<FixedArray>(CodeStubAssembler::Reference{tmp18, tmp26}, p_padding);
    tmp27 = TORQUE_CAST(TNode<HeapObject>{tmp18});
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<JSIteratorZipHelper>{tmp27};
}

TF_BUILTIN(IteratorZip, CodeStubAssembler) {
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
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, BoolT> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block41(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block42(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block43(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block63(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block82(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block83(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny> block92(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT> block93(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny> block111(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny> block112(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block128(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block130(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, JSAny> block134(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT> block150(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT> block148(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT> block152(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block162(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block163(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT> block167(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT> block168(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT> block166(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT> block154(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT> block153(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT, IntPtrT> block149(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT> block182(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT> block186(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, BoolT> block183(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT> block131(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Uint32T, JSAny, FixedArray, IntPtrT, IntPtrT, FixedArray> block129(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<JSAny> tmp1;
  TNode<BoolT> tmp2;
  TNode<BoolT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp0});
    tmp2 = Is_JSReceiver_JSAny_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{tmp1});
    tmp3 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp2});
    ca_.Branch(tmp3, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "Iterator.zip");
  }

  TNode<IntPtrT> tmp4;
  TNode<JSAny> tmp5;
  TNode<JSReceiver> tmp6;
  TNode<Uint32T> tmp7;
  TNode<JSAny> tmp8;
  TNode<JSAny> tmp9;
  TNode<Undefined> tmp10;
  TNode<BoolT> tmp11;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp4 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp5 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp4});
    tmp6 = GetOptionsObject_0(state_, TNode<NativeContext>{parameter0}, TNode<JSAny>{tmp5});
    tmp7 = FromConstexpr_JSIteratorZipHelperMode_constexpr_kShortest_0(state_, JSIteratorZipHelperMode::kShortest);
    tmp8 = FromConstexpr_JSAny_constexpr_string_0(state_, "mode");
    tmp9 = CodeStubAssembler(state_).GetProperty(TNode<Context>{parameter0}, TNode<JSAny>{tmp6}, TNode<JSAny>{tmp8});
    tmp10 = Undefined_0(state_);
    tmp11 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Object>{tmp9}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp10});
    ca_.Branch(tmp11, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{tmp7});
  }

  TNode<String> tmp12;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    compiler::CodeAssemblerLabel label13(&ca_);
    tmp12 = Cast_String_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp9}, &label13);
    ca_.Goto(&block7);
    if (label13.is_used()) {
      ca_.Bind(&label13);
      ca_.Goto(&block8);
    }
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kInvalidIteratorZipMode), "Iterator.zip");
  }

  TNode<String> tmp14;
  TNode<Oddball> tmp15;
  TNode<True> tmp16;
  TNode<BoolT> tmp17;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp14 = CodeStubAssembler(state_).StringConstant("shortest");
    tmp15 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kStringEqual, parameter0, tmp12, tmp14)); 
    tmp16 = True_0(state_);
    tmp17 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp15}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp16});
    ca_.Branch(tmp17, &block9, std::vector<compiler::Node*>{}, &block10, std::vector<compiler::Node*>{});
  }

  TNode<Uint32T> tmp18;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp18 = FromConstexpr_JSIteratorZipHelperMode_constexpr_kShortest_0(state_, JSIteratorZipHelperMode::kShortest);
    ca_.Goto(&block11, tmp18);
  }

  TNode<String> tmp19;
  TNode<Oddball> tmp20;
  TNode<True> tmp21;
  TNode<BoolT> tmp22;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp19 = CodeStubAssembler(state_).StringConstant("longest");
    tmp20 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kStringEqual, parameter0, tmp12, tmp19)); 
    tmp21 = True_0(state_);
    tmp22 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp20}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp21});
    ca_.Branch(tmp22, &block12, std::vector<compiler::Node*>{}, &block13, std::vector<compiler::Node*>{});
  }

  TNode<Uint32T> tmp23;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp23 = FromConstexpr_JSIteratorZipHelperMode_constexpr_kLongest_0(state_, JSIteratorZipHelperMode::kLongest);
    ca_.Goto(&block14, tmp23);
  }

  TNode<String> tmp24;
  TNode<Oddball> tmp25;
  TNode<True> tmp26;
  TNode<BoolT> tmp27;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp24 = CodeStubAssembler(state_).StringConstant("strict");
    tmp25 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kStringEqual, parameter0, tmp12, tmp24)); 
    tmp26 = True_0(state_);
    tmp27 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp25}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp26});
    ca_.Branch(tmp27, &block15, std::vector<compiler::Node*>{}, &block16, std::vector<compiler::Node*>{});
  }

  TNode<Uint32T> tmp28;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp28 = FromConstexpr_JSIteratorZipHelperMode_constexpr_kStrict_0(state_, JSIteratorZipHelperMode::kStrict);
    ca_.Goto(&block14, tmp28);
  }

  if (block16.is_used()) {
    ca_.Bind(&block16);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kInvalidIteratorZipMode), "Iterator.zip");
  }

  TNode<Uint32T> phi_bb14_8;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_8);
    ca_.Goto(&block11, phi_bb14_8);
  }

  TNode<Uint32T> phi_bb11_8;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_8);
    ca_.Goto(&block4, phi_bb11_8);
  }

  TNode<Uint32T> phi_bb4_8;
  TNode<Undefined> tmp29;
  TNode<Uint32T> tmp30;
  TNode<BoolT> tmp31;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_8);
    tmp29 = Undefined_0(state_);
    tmp30 = FromConstexpr_uint32_constexpr_uint32_0(state_, CastIfEnumClass<uint32_t>(JSIteratorZipHelperMode::kLongest));
    tmp31 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{phi_bb4_8}, TNode<Uint32T>{tmp30});
    ca_.Branch(tmp31, &block18, std::vector<compiler::Node*>{phi_bb4_8}, &block19, std::vector<compiler::Node*>{phi_bb4_8, tmp29});
  }

  TNode<Uint32T> phi_bb18_8;
  TNode<JSAny> tmp32;
  TNode<JSAny> tmp33;
  TNode<Undefined> tmp34;
  TNode<BoolT> tmp35;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_8);
    tmp32 = FromConstexpr_JSAny_constexpr_string_0(state_, "padding");
    tmp33 = CodeStubAssembler(state_).GetProperty(TNode<Context>{parameter0}, TNode<JSAny>{tmp6}, TNode<JSAny>{tmp32});
    tmp34 = Undefined_0(state_);
    tmp35 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Object>{tmp33}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp34});
    ca_.Branch(tmp35, &block22, std::vector<compiler::Node*>{phi_bb18_8}, &block23, std::vector<compiler::Node*>{phi_bb18_8});
  }

  TNode<Uint32T> phi_bb22_8;
  TNode<BoolT> tmp36;
  TNode<BoolT> tmp37;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_8);
    tmp36 = Is_JSReceiver_JSAny_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{tmp33});
    tmp37 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp36});
    ca_.Goto(&block24, phi_bb22_8, tmp37);
  }

  TNode<Uint32T> phi_bb23_8;
  TNode<BoolT> tmp38;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_8);
    tmp38 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block24, phi_bb23_8, tmp38);
  }

  TNode<Uint32T> phi_bb24_8;
  TNode<BoolT> phi_bb24_12;
  if (block24.is_used()) {
    ca_.Bind(&block24, &phi_bb24_8, &phi_bb24_12);
    ca_.Branch(phi_bb24_12, &block20, std::vector<compiler::Node*>{phi_bb24_8}, &block21, std::vector<compiler::Node*>{phi_bb24_8});
  }

  TNode<Uint32T> phi_bb20_8;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_8);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kCalledOnNonObject), "padding");
  }

  TNode<Uint32T> phi_bb21_8;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_8);
    ca_.Goto(&block19, phi_bb21_8, tmp33);
  }

  TNode<Uint32T> phi_bb19_8;
  TNode<JSAny> phi_bb19_10;
  TNode<FixedArray> tmp39;
  TNode<IntPtrT> tmp40;
  TNode<IntPtrT> tmp41;
  TNode<FixedArray> tmp42;
  TNode<Map> tmp43;
  TNode<JSReceiver> tmp44;
  TNode<JSAny> tmp45;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_8, &phi_bb19_10);
    std::tie(tmp39, tmp40, tmp41) = NewGrowableFixedArray_0(state_).Flatten();
    tmp42 = kEmptyFixedArray_0(state_);
    tmp43 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    std::tie(tmp44, tmp45) = IteratorBuiltinsAssembler(state_).GetIterator(TNode<Context>{parameter0}, TNode<JSAny>{tmp1}).Flatten();
    ca_.Goto(&block31, phi_bb19_8, phi_bb19_10, tmp39, tmp40, tmp41);
  }

  TNode<Uint32T> phi_bb31_8;
  TNode<JSAny> phi_bb31_10;
  TNode<FixedArray> phi_bb31_11;
  TNode<IntPtrT> phi_bb31_12;
  TNode<IntPtrT> phi_bb31_13;
  TNode<BoolT> tmp46;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_8, &phi_bb31_10, &phi_bb31_11, &phi_bb31_12, &phi_bb31_13);
    tmp46 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp46, &block29, std::vector<compiler::Node*>{phi_bb31_8, phi_bb31_10, phi_bb31_11, phi_bb31_12, phi_bb31_13}, &block30, std::vector<compiler::Node*>{phi_bb31_8, phi_bb31_10, phi_bb31_11, phi_bb31_12, phi_bb31_13});
  }

  TNode<Uint32T> phi_bb29_8;
  TNode<JSAny> phi_bb29_10;
  TNode<FixedArray> phi_bb29_11;
  TNode<IntPtrT> phi_bb29_12;
  TNode<IntPtrT> phi_bb29_13;
  TNode<JSAny> tmp47;
    compiler::TypedCodeAssemblerVariable<JSAny> tmp50(&ca_);
    compiler::TypedCodeAssemblerVariable<Union<JSMessageObject, TheHole>> tmp51(&ca_);
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_8, &phi_bb29_10, &phi_bb29_11, &phi_bb29_12, &phi_bb29_13);
    compiler::CodeAssemblerLabel label48(&ca_);
    compiler::CodeAssemblerLabel label49(&ca_);
    tmp47 = IteratorStepValue_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp44}, TNode<JSAny>{tmp45}}, TNode<Map>{tmp43}, &label48, &label49, &tmp50, &tmp51);
    ca_.Goto(&block32, phi_bb29_8, phi_bb29_10, phi_bb29_11, phi_bb29_12, phi_bb29_13);
    if (label48.is_used()) {
      ca_.Bind(&label48);
      ca_.Goto(&block33, phi_bb29_8, phi_bb29_10, phi_bb29_11, phi_bb29_12, phi_bb29_13);
    }
    if (label49.is_used()) {
      ca_.Bind(&label49);
      ca_.Goto(&block34, phi_bb29_8, phi_bb29_10, phi_bb29_11, phi_bb29_12, phi_bb29_13);
    }
  }

  TNode<Uint32T> phi_bb33_8;
  TNode<JSAny> phi_bb33_10;
  TNode<FixedArray> phi_bb33_11;
  TNode<IntPtrT> phi_bb33_12;
  TNode<IntPtrT> phi_bb33_13;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_8, &phi_bb33_10, &phi_bb33_11, &phi_bb33_12, &phi_bb33_13);
    ca_.Goto(&block25, phi_bb33_8, phi_bb33_10, phi_bb33_11, phi_bb33_12, phi_bb33_13);
  }

  TNode<Uint32T> phi_bb34_8;
  TNode<JSAny> phi_bb34_10;
  TNode<FixedArray> phi_bb34_11;
  TNode<IntPtrT> phi_bb34_12;
  TNode<IntPtrT> phi_bb34_13;
  TNode<IntPtrT> tmp52;
  TNode<TheHole> tmp53;
  TNode<FixedArray> tmp54;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_8, &phi_bb34_10, &phi_bb34_11, &phi_bb34_12, &phi_bb34_13);
    tmp52 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp53 = TheHole_0(state_);
    tmp54 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb34_11}, TNode<IntPtrT>{tmp52}, TNode<IntPtrT>{phi_bb34_13}, TNode<IntPtrT>{phi_bb34_13}, TNode<Hole>{tmp53});
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp54}, false);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp50.value(), tmp51.value());
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Uint32T> phi_bb32_8;
  TNode<JSAny> phi_bb32_10;
  TNode<FixedArray> phi_bb32_11;
  TNode<IntPtrT> phi_bb32_12;
  TNode<IntPtrT> phi_bb32_13;
  TNode<JSReceiver> tmp55;
      TNode<JSAny> tmp58;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_8, &phi_bb32_10, &phi_bb32_11, &phi_bb32_12, &phi_bb32_13);
    compiler::CodeAssemblerLabel label56(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch57__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch57__label);
    tmp55 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp47}, &label56);
    }
    if (catch57__label.is_used()) {
      compiler::CodeAssemblerLabel catch57_skip(&ca_);
      ca_.Goto(&catch57_skip);
      ca_.Bind(&catch57__label, &tmp58);
      ca_.Goto(&block41, phi_bb32_8, phi_bb32_10, phi_bb32_11, phi_bb32_12, phi_bb32_13);
      ca_.Bind(&catch57_skip);
    }
    ca_.Goto(&block39, phi_bb32_8, phi_bb32_10, phi_bb32_11, phi_bb32_12, phi_bb32_13);
    if (label56.is_used()) {
      ca_.Bind(&label56);
      ca_.Goto(&block40, phi_bb32_8, phi_bb32_10, phi_bb32_11, phi_bb32_12, phi_bb32_13);
    }
  }

  TNode<Uint32T> phi_bb41_8;
  TNode<JSAny> phi_bb41_10;
  TNode<FixedArray> phi_bb41_11;
  TNode<IntPtrT> phi_bb41_12;
  TNode<IntPtrT> phi_bb41_13;
  TNode<Union<JSMessageObject, TheHole>> tmp59;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_8, &phi_bb41_10, &phi_bb41_11, &phi_bb41_12, &phi_bb41_13);
    tmp59 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block36, phi_bb41_8, phi_bb41_10, phi_bb41_11, phi_bb41_12, phi_bb41_13, tmp58, tmp59);
  }

  TNode<Uint32T> phi_bb40_8;
  TNode<JSAny> phi_bb40_10;
  TNode<FixedArray> phi_bb40_11;
  TNode<IntPtrT> phi_bb40_12;
  TNode<IntPtrT> phi_bb40_13;
      TNode<JSAny> tmp61;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_8, &phi_bb40_10, &phi_bb40_11, &phi_bb40_12, &phi_bb40_13);
    compiler::CodeAssemblerExceptionHandlerLabel catch60__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch60__label);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kNotIterable), TNode<Object>{tmp47});
    }
    if (catch60__label.is_used()) {
      compiler::CodeAssemblerLabel catch60_skip(&ca_);
      ca_.Bind(&catch60__label, &tmp61);
      ca_.Goto(&block42, phi_bb40_8, phi_bb40_10, phi_bb40_11, phi_bb40_12, phi_bb40_13);
    }
  }

  TNode<Uint32T> phi_bb39_8;
  TNode<JSAny> phi_bb39_10;
  TNode<FixedArray> phi_bb39_11;
  TNode<IntPtrT> phi_bb39_12;
  TNode<IntPtrT> phi_bb39_13;
  TNode<JSReceiver> tmp62;
  TNode<JSAny> tmp63;
      TNode<JSAny> tmp65;
  TNode<BoolT> tmp66;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_8, &phi_bb39_10, &phi_bb39_11, &phi_bb39_12, &phi_bb39_13);
    compiler::CodeAssemblerExceptionHandlerLabel catch64__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch64__label);
    std::tie(tmp62, tmp63) = GetIteratorFlattenable_0(state_, TNode<Context>{parameter0}, TNode<Union<JSReceiver, String>>{tmp55}).Flatten();
    }
    if (catch64__label.is_used()) {
      compiler::CodeAssemblerLabel catch64_skip(&ca_);
      ca_.Goto(&catch64_skip);
      ca_.Bind(&catch64__label, &tmp65);
      ca_.Goto(&block43, phi_bb39_8, phi_bb39_10, phi_bb39_11, phi_bb39_12, phi_bb39_13);
      ca_.Bind(&catch64_skip);
    }
    tmp66 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb39_12}, TNode<IntPtrT>{phi_bb39_13});
    ca_.Branch(tmp66, &block63, std::vector<compiler::Node*>{phi_bb39_8, phi_bb39_10, phi_bb39_11, phi_bb39_12, phi_bb39_13}, &block64, std::vector<compiler::Node*>{phi_bb39_8, phi_bb39_10, phi_bb39_11, phi_bb39_12, phi_bb39_13});
  }

  TNode<Uint32T> phi_bb42_8;
  TNode<JSAny> phi_bb42_10;
  TNode<FixedArray> phi_bb42_11;
  TNode<IntPtrT> phi_bb42_12;
  TNode<IntPtrT> phi_bb42_13;
  TNode<Union<JSMessageObject, TheHole>> tmp67;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_8, &phi_bb42_10, &phi_bb42_11, &phi_bb42_12, &phi_bb42_13);
    tmp67 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block36, phi_bb42_8, phi_bb42_10, phi_bb42_11, phi_bb42_12, phi_bb42_13, tmp61, tmp67);
  }

  TNode<Uint32T> phi_bb43_8;
  TNode<JSAny> phi_bb43_10;
  TNode<FixedArray> phi_bb43_11;
  TNode<IntPtrT> phi_bb43_12;
  TNode<IntPtrT> phi_bb43_13;
  TNode<Union<JSMessageObject, TheHole>> tmp68;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_8, &phi_bb43_10, &phi_bb43_11, &phi_bb43_12, &phi_bb43_13);
    tmp68 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block36, phi_bb43_8, phi_bb43_10, phi_bb43_11, phi_bb43_12, phi_bb43_13, tmp65, tmp68);
  }

  TNode<Uint32T> phi_bb36_8;
  TNode<JSAny> phi_bb36_10;
  TNode<FixedArray> phi_bb36_11;
  TNode<IntPtrT> phi_bb36_12;
  TNode<IntPtrT> phi_bb36_13;
  TNode<JSAny> phi_bb36_21;
  TNode<Union<JSMessageObject, TheHole>> phi_bb36_22;
  TNode<IntPtrT> tmp69;
  TNode<TheHole> tmp70;
  TNode<FixedArray> tmp71;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_8, &phi_bb36_10, &phi_bb36_11, &phi_bb36_12, &phi_bb36_13, &phi_bb36_21, &phi_bb36_22);
    tmp69 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp70 = TheHole_0(state_);
    tmp71 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb36_11}, TNode<IntPtrT>{tmp69}, TNode<IntPtrT>{phi_bb36_13}, TNode<IntPtrT>{phi_bb36_13}, TNode<Hole>{tmp70});
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp71}, false);
    IteratorCloseOnException_0(state_, TNode<Context>{parameter0}, TNode<JSReceiver>{tmp44});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb36_21, phi_bb36_22);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Uint32T> phi_bb63_8;
  TNode<JSAny> phi_bb63_10;
  TNode<FixedArray> phi_bb63_11;
  TNode<IntPtrT> phi_bb63_12;
  TNode<IntPtrT> phi_bb63_13;
  TNode<IntPtrT> tmp72;
  TNode<IntPtrT> tmp73;
  TNode<IntPtrT> tmp74;
  TNode<IntPtrT> tmp75;
  TNode<IntPtrT> tmp76;
  TNode<IntPtrT> tmp77;
  TNode<TheHole> tmp78;
  TNode<FixedArray> tmp79;
  if (block63.is_used()) {
    ca_.Bind(&block63, &phi_bb63_8, &phi_bb63_10, &phi_bb63_11, &phi_bb63_12, &phi_bb63_13);
    tmp72 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp73 = CodeStubAssembler(state_).WordSar(TNode<IntPtrT>{phi_bb63_12}, TNode<IntPtrT>{tmp72});
    tmp74 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb63_12}, TNode<IntPtrT>{tmp73});
    tmp75 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x10ull));
    tmp76 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp74}, TNode<IntPtrT>{tmp75});
    tmp77 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp78 = TheHole_0(state_);
    tmp79 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb63_11}, TNode<IntPtrT>{tmp77}, TNode<IntPtrT>{phi_bb63_13}, TNode<IntPtrT>{tmp76}, TNode<Hole>{tmp78});
    ca_.Goto(&block64, phi_bb63_8, phi_bb63_10, tmp79, tmp76, phi_bb63_13);
  }

  TNode<Uint32T> phi_bb64_8;
  TNode<JSAny> phi_bb64_10;
  TNode<FixedArray> phi_bb64_11;
  TNode<IntPtrT> phi_bb64_12;
  TNode<IntPtrT> phi_bb64_13;
  TNode<Union<HeapObject, TaggedIndex>> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<IntPtrT> tmp82;
  TNode<IntPtrT> tmp83;
  TNode<IntPtrT> tmp84;
  TNode<UintPtrT> tmp85;
  TNode<UintPtrT> tmp86;
  TNode<BoolT> tmp87;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_8, &phi_bb64_10, &phi_bb64_11, &phi_bb64_12, &phi_bb64_13);
    std::tie(tmp80, tmp81, tmp82) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb64_11}).Flatten();
    tmp83 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp84 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb64_13}, TNode<IntPtrT>{tmp83});
    tmp85 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb64_13});
    tmp86 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp82});
    tmp87 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp85}, TNode<UintPtrT>{tmp86});
    ca_.Branch(tmp87, &block82, std::vector<compiler::Node*>{phi_bb64_8, phi_bb64_10, phi_bb64_13, phi_bb64_13, phi_bb64_13, phi_bb64_13}, &block83, std::vector<compiler::Node*>{phi_bb64_8, phi_bb64_10, phi_bb64_13, phi_bb64_13, phi_bb64_13, phi_bb64_13});
  }

  TNode<Uint32T> phi_bb82_8;
  TNode<JSAny> phi_bb82_10;
  TNode<IntPtrT> phi_bb82_27;
  TNode<IntPtrT> phi_bb82_28;
  TNode<IntPtrT> phi_bb82_32;
  TNode<IntPtrT> phi_bb82_33;
  TNode<IntPtrT> tmp88;
  TNode<IntPtrT> tmp89;
  TNode<Union<HeapObject, TaggedIndex>> tmp90;
  TNode<IntPtrT> tmp91;
  TNode<BoolT> tmp92;
  if (block82.is_used()) {
    ca_.Bind(&block82, &phi_bb82_8, &phi_bb82_10, &phi_bb82_27, &phi_bb82_28, &phi_bb82_32, &phi_bb82_33);
    tmp88 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb82_33});
    tmp89 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp81}, TNode<IntPtrT>{tmp88});
    std::tie(tmp90, tmp91) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp80}, TNode<IntPtrT>{tmp89}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp90, tmp91}, tmp62);
    tmp92 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb64_12}, TNode<IntPtrT>{tmp84});
    ca_.Branch(tmp92, &block92, std::vector<compiler::Node*>{phi_bb82_8, phi_bb82_10}, &block93, std::vector<compiler::Node*>{phi_bb82_8, phi_bb82_10, phi_bb64_11, phi_bb64_12});
  }

  TNode<Uint32T> phi_bb83_8;
  TNode<JSAny> phi_bb83_10;
  TNode<IntPtrT> phi_bb83_27;
  TNode<IntPtrT> phi_bb83_28;
  TNode<IntPtrT> phi_bb83_32;
  TNode<IntPtrT> phi_bb83_33;
  if (block83.is_used()) {
    ca_.Bind(&block83, &phi_bb83_8, &phi_bb83_10, &phi_bb83_27, &phi_bb83_28, &phi_bb83_32, &phi_bb83_33);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Uint32T> phi_bb92_8;
  TNode<JSAny> phi_bb92_10;
  TNode<IntPtrT> tmp93;
  TNode<IntPtrT> tmp94;
  TNode<IntPtrT> tmp95;
  TNode<IntPtrT> tmp96;
  TNode<IntPtrT> tmp97;
  TNode<IntPtrT> tmp98;
  TNode<TheHole> tmp99;
  TNode<FixedArray> tmp100;
  if (block92.is_used()) {
    ca_.Bind(&block92, &phi_bb92_8, &phi_bb92_10);
    tmp93 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp94 = CodeStubAssembler(state_).WordSar(TNode<IntPtrT>{phi_bb64_12}, TNode<IntPtrT>{tmp93});
    tmp95 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb64_12}, TNode<IntPtrT>{tmp94});
    tmp96 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x10ull));
    tmp97 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp95}, TNode<IntPtrT>{tmp96});
    tmp98 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp99 = TheHole_0(state_);
    tmp100 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb64_11}, TNode<IntPtrT>{tmp98}, TNode<IntPtrT>{tmp84}, TNode<IntPtrT>{tmp97}, TNode<Hole>{tmp99});
    ca_.Goto(&block93, phi_bb92_8, phi_bb92_10, tmp100, tmp97);
  }

  TNode<Uint32T> phi_bb93_8;
  TNode<JSAny> phi_bb93_10;
  TNode<FixedArray> phi_bb93_11;
  TNode<IntPtrT> phi_bb93_12;
  TNode<Union<HeapObject, TaggedIndex>> tmp101;
  TNode<IntPtrT> tmp102;
  TNode<IntPtrT> tmp103;
  TNode<IntPtrT> tmp104;
  TNode<IntPtrT> tmp105;
  TNode<UintPtrT> tmp106;
  TNode<UintPtrT> tmp107;
  TNode<BoolT> tmp108;
  if (block93.is_used()) {
    ca_.Bind(&block93, &phi_bb93_8, &phi_bb93_10, &phi_bb93_11, &phi_bb93_12);
    std::tie(tmp101, tmp102, tmp103) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb93_11}).Flatten();
    tmp104 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp105 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp84}, TNode<IntPtrT>{tmp104});
    tmp106 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp84});
    tmp107 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp103});
    tmp108 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp106}, TNode<UintPtrT>{tmp107});
    ca_.Branch(tmp108, &block111, std::vector<compiler::Node*>{phi_bb93_8, phi_bb93_10}, &block112, std::vector<compiler::Node*>{phi_bb93_8, phi_bb93_10});
  }

  TNode<Uint32T> phi_bb111_8;
  TNode<JSAny> phi_bb111_10;
  TNode<IntPtrT> tmp109;
  TNode<IntPtrT> tmp110;
  TNode<Union<HeapObject, TaggedIndex>> tmp111;
  TNode<IntPtrT> tmp112;
  if (block111.is_used()) {
    ca_.Bind(&block111, &phi_bb111_8, &phi_bb111_10);
    tmp109 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp84});
    tmp110 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp102}, TNode<IntPtrT>{tmp109});
    std::tie(tmp111, tmp112) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp101}, TNode<IntPtrT>{tmp110}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp111, tmp112}, tmp63);
    ca_.Goto(&block31, phi_bb111_8, phi_bb111_10, phi_bb93_11, phi_bb93_12, tmp105);
  }

  TNode<Uint32T> phi_bb112_8;
  TNode<JSAny> phi_bb112_10;
  if (block112.is_used()) {
    ca_.Bind(&block112, &phi_bb112_8, &phi_bb112_10);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Uint32T> phi_bb30_8;
  TNode<JSAny> phi_bb30_10;
  TNode<FixedArray> phi_bb30_11;
  TNode<IntPtrT> phi_bb30_12;
  TNode<IntPtrT> phi_bb30_13;
  if (block30.is_used()) {
    ca_.Bind(&block30, &phi_bb30_8, &phi_bb30_10, &phi_bb30_11, &phi_bb30_12, &phi_bb30_13);
    ca_.Goto(&block25, phi_bb30_8, phi_bb30_10, phi_bb30_11, phi_bb30_12, phi_bb30_13);
  }

  TNode<Uint32T> phi_bb25_8;
  TNode<JSAny> phi_bb25_10;
  TNode<FixedArray> phi_bb25_11;
  TNode<IntPtrT> phi_bb25_12;
  TNode<IntPtrT> phi_bb25_13;
  TNode<IntPtrT> tmp113;
  TNode<IntPtrT> tmp114;
  TNode<Uint32T> tmp115;
  TNode<BoolT> tmp116;
  if (block25.is_used()) {
    ca_.Bind(&block25, &phi_bb25_8, &phi_bb25_10, &phi_bb25_11, &phi_bb25_12, &phi_bb25_13);
    tmp113 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    tmp114 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{phi_bb25_13}, TNode<IntPtrT>{tmp113});
    tmp115 = FromConstexpr_uint32_constexpr_uint32_0(state_, CastIfEnumClass<uint32_t>(JSIteratorZipHelperMode::kLongest));
    tmp116 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{phi_bb25_8}, TNode<Uint32T>{tmp115});
    ca_.Branch(tmp116, &block128, std::vector<compiler::Node*>{phi_bb25_8, phi_bb25_10, phi_bb25_11, phi_bb25_12, phi_bb25_13}, &block129, std::vector<compiler::Node*>{phi_bb25_8, phi_bb25_10, phi_bb25_11, phi_bb25_12, phi_bb25_13, tmp42});
  }

  TNode<Uint32T> phi_bb128_8;
  TNode<JSAny> phi_bb128_10;
  TNode<FixedArray> phi_bb128_11;
  TNode<IntPtrT> phi_bb128_12;
  TNode<IntPtrT> phi_bb128_13;
  TNode<Undefined> tmp117;
  TNode<Undefined> tmp118;
  TNode<FixedArray> tmp119;
  TNode<Undefined> tmp120;
  TNode<BoolT> tmp121;
  if (block128.is_used()) {
    ca_.Bind(&block128, &phi_bb128_8, &phi_bb128_10, &phi_bb128_11, &phi_bb128_12, &phi_bb128_13);
    tmp117 = Undefined_0(state_);
    std::tie(tmp118) = ConstantIterator_Undefined_0(state_, TNode<Undefined>{tmp117}).Flatten();
    tmp119 = NewFixedArray_ConstantIterator_Undefined_0(state_, TNode<IntPtrT>{tmp114}, TorqueStructConstantIterator_Undefined_0{TNode<Undefined>{tmp118}});
    tmp120 = Undefined_0(state_);
    tmp121 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Object>{phi_bb128_10}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp120});
    ca_.Branch(tmp121, &block130, std::vector<compiler::Node*>{phi_bb128_8, phi_bb128_10, phi_bb128_11, phi_bb128_12, phi_bb128_13}, &block131, std::vector<compiler::Node*>{phi_bb128_8, phi_bb128_10, phi_bb128_11, phi_bb128_12, phi_bb128_13});
  }

  TNode<Uint32T> phi_bb130_8;
  TNode<JSAny> phi_bb130_10;
  TNode<FixedArray> phi_bb130_11;
  TNode<IntPtrT> phi_bb130_12;
  TNode<IntPtrT> phi_bb130_13;
  TNode<JSReceiver> tmp122;
  TNode<JSAny> tmp123;
      TNode<JSAny> tmp125;
  TNode<BoolT> tmp126;
  TNode<IntPtrT> tmp127;
  if (block130.is_used()) {
    ca_.Bind(&block130, &phi_bb130_8, &phi_bb130_10, &phi_bb130_11, &phi_bb130_12, &phi_bb130_13);
    compiler::CodeAssemblerExceptionHandlerLabel catch124__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch124__label);
    std::tie(tmp122, tmp123) = IteratorBuiltinsAssembler(state_).GetIterator(TNode<Context>{parameter0}, TNode<JSAny>{phi_bb130_10}).Flatten();
    }
    if (catch124__label.is_used()) {
      compiler::CodeAssemblerLabel catch124_skip(&ca_);
      ca_.Goto(&catch124_skip);
      ca_.Bind(&catch124__label, &tmp125);
      ca_.Goto(&block134, phi_bb130_8, phi_bb130_10, phi_bb130_11, phi_bb130_12, phi_bb130_13, phi_bb130_10);
      ca_.Bind(&catch124_skip);
    }
    tmp126 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp127 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block150, phi_bb130_8, phi_bb130_10, phi_bb130_11, phi_bb130_12, phi_bb130_13, tmp126, tmp127);
  }

  TNode<Uint32T> phi_bb134_8;
  TNode<JSAny> phi_bb134_10;
  TNode<FixedArray> phi_bb134_11;
  TNode<IntPtrT> phi_bb134_12;
  TNode<IntPtrT> phi_bb134_13;
  TNode<JSAny> phi_bb134_19;
  TNode<Union<JSMessageObject, TheHole>> tmp128;
  TNode<IntPtrT> tmp129;
  TNode<TheHole> tmp130;
  TNode<FixedArray> tmp131;
  if (block134.is_used()) {
    ca_.Bind(&block134, &phi_bb134_8, &phi_bb134_10, &phi_bb134_11, &phi_bb134_12, &phi_bb134_13, &phi_bb134_19);
    tmp128 = GetAndResetPendingMessage_0(state_);
    tmp129 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp130 = TheHole_0(state_);
    tmp131 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb134_11}, TNode<IntPtrT>{tmp129}, TNode<IntPtrT>{phi_bb134_13}, TNode<IntPtrT>{phi_bb134_13}, TNode<Hole>{tmp130});
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp131}, false);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp125, tmp128);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Uint32T> phi_bb150_8;
  TNode<JSAny> phi_bb150_10;
  TNode<FixedArray> phi_bb150_11;
  TNode<IntPtrT> phi_bb150_12;
  TNode<IntPtrT> phi_bb150_13;
  TNode<BoolT> phi_bb150_19;
  TNode<IntPtrT> phi_bb150_20;
  TNode<BoolT> tmp132;
  if (block150.is_used()) {
    ca_.Bind(&block150, &phi_bb150_8, &phi_bb150_10, &phi_bb150_11, &phi_bb150_12, &phi_bb150_13, &phi_bb150_19, &phi_bb150_20);
    tmp132 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb150_20}, TNode<IntPtrT>{tmp114});
    ca_.Branch(tmp132, &block148, std::vector<compiler::Node*>{phi_bb150_8, phi_bb150_10, phi_bb150_11, phi_bb150_12, phi_bb150_13, phi_bb150_19, phi_bb150_20}, &block149, std::vector<compiler::Node*>{phi_bb150_8, phi_bb150_10, phi_bb150_11, phi_bb150_12, phi_bb150_13, phi_bb150_19, phi_bb150_20});
  }

  TNode<Uint32T> phi_bb148_8;
  TNode<JSAny> phi_bb148_10;
  TNode<FixedArray> phi_bb148_11;
  TNode<IntPtrT> phi_bb148_12;
  TNode<IntPtrT> phi_bb148_13;
  TNode<BoolT> phi_bb148_19;
  TNode<IntPtrT> phi_bb148_20;
  if (block148.is_used()) {
    ca_.Bind(&block148, &phi_bb148_8, &phi_bb148_10, &phi_bb148_11, &phi_bb148_12, &phi_bb148_13, &phi_bb148_19, &phi_bb148_20);
    ca_.Branch(phi_bb148_19, &block152, std::vector<compiler::Node*>{phi_bb148_8, phi_bb148_10, phi_bb148_11, phi_bb148_12, phi_bb148_13, phi_bb148_19, phi_bb148_20}, &block153, std::vector<compiler::Node*>{phi_bb148_8, phi_bb148_10, phi_bb148_11, phi_bb148_12, phi_bb148_13, phi_bb148_19, phi_bb148_20});
  }

  TNode<Uint32T> phi_bb152_8;
  TNode<JSAny> phi_bb152_10;
  TNode<FixedArray> phi_bb152_11;
  TNode<IntPtrT> phi_bb152_12;
  TNode<IntPtrT> phi_bb152_13;
  TNode<BoolT> phi_bb152_19;
  TNode<IntPtrT> phi_bb152_20;
  TNode<Union<HeapObject, TaggedIndex>> tmp133;
  TNode<IntPtrT> tmp134;
  TNode<IntPtrT> tmp135;
  TNode<UintPtrT> tmp136;
  TNode<UintPtrT> tmp137;
  TNode<BoolT> tmp138;
  if (block152.is_used()) {
    ca_.Bind(&block152, &phi_bb152_8, &phi_bb152_10, &phi_bb152_11, &phi_bb152_12, &phi_bb152_13, &phi_bb152_19, &phi_bb152_20);
    std::tie(tmp133, tmp134, tmp135) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp119}).Flatten();
    tmp136 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb152_20});
    tmp137 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp135});
    tmp138 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp136}, TNode<UintPtrT>{tmp137});
    ca_.Branch(tmp138, &block162, std::vector<compiler::Node*>{phi_bb152_8, phi_bb152_10, phi_bb152_11, phi_bb152_12, phi_bb152_13, phi_bb152_19, phi_bb152_20, phi_bb152_20, phi_bb152_20, phi_bb152_20, phi_bb152_20}, &block163, std::vector<compiler::Node*>{phi_bb152_8, phi_bb152_10, phi_bb152_11, phi_bb152_12, phi_bb152_13, phi_bb152_19, phi_bb152_20, phi_bb152_20, phi_bb152_20, phi_bb152_20, phi_bb152_20});
  }

  TNode<Uint32T> phi_bb162_8;
  TNode<JSAny> phi_bb162_10;
  TNode<FixedArray> phi_bb162_11;
  TNode<IntPtrT> phi_bb162_12;
  TNode<IntPtrT> phi_bb162_13;
  TNode<BoolT> phi_bb162_19;
  TNode<IntPtrT> phi_bb162_20;
  TNode<IntPtrT> phi_bb162_25;
  TNode<IntPtrT> phi_bb162_26;
  TNode<IntPtrT> phi_bb162_30;
  TNode<IntPtrT> phi_bb162_31;
  TNode<IntPtrT> tmp139;
  TNode<IntPtrT> tmp140;
  TNode<Union<HeapObject, TaggedIndex>> tmp141;
  TNode<IntPtrT> tmp142;
  TNode<JSAny> tmp143;
    compiler::TypedCodeAssemblerVariable<JSAny> tmp146(&ca_);
    compiler::TypedCodeAssemblerVariable<Union<JSMessageObject, TheHole>> tmp147(&ca_);
  if (block162.is_used()) {
    ca_.Bind(&block162, &phi_bb162_8, &phi_bb162_10, &phi_bb162_11, &phi_bb162_12, &phi_bb162_13, &phi_bb162_19, &phi_bb162_20, &phi_bb162_25, &phi_bb162_26, &phi_bb162_30, &phi_bb162_31);
    tmp139 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb162_31});
    tmp140 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp134}, TNode<IntPtrT>{tmp139});
    std::tie(tmp141, tmp142) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp133}, TNode<IntPtrT>{tmp140}).Flatten();
    compiler::CodeAssemblerLabel label144(&ca_);
    compiler::CodeAssemblerLabel label145(&ca_);
    tmp143 = IteratorStepValue_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp122}, TNode<JSAny>{tmp123}}, TNode<Map>{tmp43}, &label144, &label145, &tmp146, &tmp147);
    ca_.Goto(&block166, phi_bb162_8, phi_bb162_10, phi_bb162_11, phi_bb162_12, phi_bb162_13, phi_bb162_19, phi_bb162_20, phi_bb162_25, phi_bb162_26);
    if (label144.is_used()) {
      ca_.Bind(&label144);
      ca_.Goto(&block167, phi_bb162_8, phi_bb162_10, phi_bb162_11, phi_bb162_12, phi_bb162_13, phi_bb162_19, phi_bb162_20, phi_bb162_25, phi_bb162_26);
    }
    if (label145.is_used()) {
      ca_.Bind(&label145);
      ca_.Goto(&block168, phi_bb162_8, phi_bb162_10, phi_bb162_11, phi_bb162_12, phi_bb162_13, phi_bb162_19, phi_bb162_20, phi_bb162_25, phi_bb162_26);
    }
  }

  TNode<Uint32T> phi_bb163_8;
  TNode<JSAny> phi_bb163_10;
  TNode<FixedArray> phi_bb163_11;
  TNode<IntPtrT> phi_bb163_12;
  TNode<IntPtrT> phi_bb163_13;
  TNode<BoolT> phi_bb163_19;
  TNode<IntPtrT> phi_bb163_20;
  TNode<IntPtrT> phi_bb163_25;
  TNode<IntPtrT> phi_bb163_26;
  TNode<IntPtrT> phi_bb163_30;
  TNode<IntPtrT> phi_bb163_31;
  if (block163.is_used()) {
    ca_.Bind(&block163, &phi_bb163_8, &phi_bb163_10, &phi_bb163_11, &phi_bb163_12, &phi_bb163_13, &phi_bb163_19, &phi_bb163_20, &phi_bb163_25, &phi_bb163_26, &phi_bb163_30, &phi_bb163_31);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Uint32T> phi_bb167_8;
  TNode<JSAny> phi_bb167_10;
  TNode<FixedArray> phi_bb167_11;
  TNode<IntPtrT> phi_bb167_12;
  TNode<IntPtrT> phi_bb167_13;
  TNode<BoolT> phi_bb167_19;
  TNode<IntPtrT> phi_bb167_20;
  TNode<IntPtrT> phi_bb167_25;
  TNode<IntPtrT> phi_bb167_26;
  TNode<BoolT> tmp148;
  if (block167.is_used()) {
    ca_.Bind(&block167, &phi_bb167_8, &phi_bb167_10, &phi_bb167_11, &phi_bb167_12, &phi_bb167_13, &phi_bb167_19, &phi_bb167_20, &phi_bb167_25, &phi_bb167_26);
    tmp148 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block154, phi_bb167_8, phi_bb167_10, phi_bb167_11, phi_bb167_12, phi_bb167_13, tmp148, phi_bb167_20);
  }

  TNode<Uint32T> phi_bb168_8;
  TNode<JSAny> phi_bb168_10;
  TNode<FixedArray> phi_bb168_11;
  TNode<IntPtrT> phi_bb168_12;
  TNode<IntPtrT> phi_bb168_13;
  TNode<BoolT> phi_bb168_19;
  TNode<IntPtrT> phi_bb168_20;
  TNode<IntPtrT> phi_bb168_25;
  TNode<IntPtrT> phi_bb168_26;
  TNode<IntPtrT> tmp149;
  TNode<TheHole> tmp150;
  TNode<FixedArray> tmp151;
  if (block168.is_used()) {
    ca_.Bind(&block168, &phi_bb168_8, &phi_bb168_10, &phi_bb168_11, &phi_bb168_12, &phi_bb168_13, &phi_bb168_19, &phi_bb168_20, &phi_bb168_25, &phi_bb168_26);
    tmp149 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp150 = TheHole_0(state_);
    tmp151 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb168_11}, TNode<IntPtrT>{tmp149}, TNode<IntPtrT>{phi_bb168_13}, TNode<IntPtrT>{phi_bb168_13}, TNode<Hole>{tmp150});
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp151}, false);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp146.value(), tmp147.value());
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Uint32T> phi_bb166_8;
  TNode<JSAny> phi_bb166_10;
  TNode<FixedArray> phi_bb166_11;
  TNode<IntPtrT> phi_bb166_12;
  TNode<IntPtrT> phi_bb166_13;
  TNode<BoolT> phi_bb166_19;
  TNode<IntPtrT> phi_bb166_20;
  TNode<IntPtrT> phi_bb166_25;
  TNode<IntPtrT> phi_bb166_26;
  if (block166.is_used()) {
    ca_.Bind(&block166, &phi_bb166_8, &phi_bb166_10, &phi_bb166_11, &phi_bb166_12, &phi_bb166_13, &phi_bb166_19, &phi_bb166_20, &phi_bb166_25, &phi_bb166_26);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp141, tmp142}, tmp143);
    ca_.Goto(&block154, phi_bb166_8, phi_bb166_10, phi_bb166_11, phi_bb166_12, phi_bb166_13, phi_bb166_19, phi_bb166_20);
  }

  TNode<Uint32T> phi_bb154_8;
  TNode<JSAny> phi_bb154_10;
  TNode<FixedArray> phi_bb154_11;
  TNode<IntPtrT> phi_bb154_12;
  TNode<IntPtrT> phi_bb154_13;
  TNode<BoolT> phi_bb154_19;
  TNode<IntPtrT> phi_bb154_20;
  if (block154.is_used()) {
    ca_.Bind(&block154, &phi_bb154_8, &phi_bb154_10, &phi_bb154_11, &phi_bb154_12, &phi_bb154_13, &phi_bb154_19, &phi_bb154_20);
    ca_.Goto(&block153, phi_bb154_8, phi_bb154_10, phi_bb154_11, phi_bb154_12, phi_bb154_13, phi_bb154_19, phi_bb154_20);
  }

  TNode<Uint32T> phi_bb153_8;
  TNode<JSAny> phi_bb153_10;
  TNode<FixedArray> phi_bb153_11;
  TNode<IntPtrT> phi_bb153_12;
  TNode<IntPtrT> phi_bb153_13;
  TNode<BoolT> phi_bb153_19;
  TNode<IntPtrT> phi_bb153_20;
  TNode<IntPtrT> tmp152;
  TNode<IntPtrT> tmp153;
  if (block153.is_used()) {
    ca_.Bind(&block153, &phi_bb153_8, &phi_bb153_10, &phi_bb153_11, &phi_bb153_12, &phi_bb153_13, &phi_bb153_19, &phi_bb153_20);
    tmp152 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp153 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb153_20}, TNode<IntPtrT>{tmp152});
    ca_.Goto(&block150, phi_bb153_8, phi_bb153_10, phi_bb153_11, phi_bb153_12, phi_bb153_13, phi_bb153_19, tmp153);
  }

  TNode<Uint32T> phi_bb149_8;
  TNode<JSAny> phi_bb149_10;
  TNode<FixedArray> phi_bb149_11;
  TNode<IntPtrT> phi_bb149_12;
  TNode<IntPtrT> phi_bb149_13;
  TNode<BoolT> phi_bb149_19;
  TNode<IntPtrT> phi_bb149_20;
  if (block149.is_used()) {
    ca_.Bind(&block149, &phi_bb149_8, &phi_bb149_10, &phi_bb149_11, &phi_bb149_12, &phi_bb149_13, &phi_bb149_19, &phi_bb149_20);
    ca_.Branch(phi_bb149_19, &block182, std::vector<compiler::Node*>{phi_bb149_8, phi_bb149_10, phi_bb149_11, phi_bb149_12, phi_bb149_13, phi_bb149_19}, &block183, std::vector<compiler::Node*>{phi_bb149_8, phi_bb149_10, phi_bb149_11, phi_bb149_12, phi_bb149_13, phi_bb149_19});
  }

  TNode<Uint32T> phi_bb182_8;
  TNode<JSAny> phi_bb182_10;
  TNode<FixedArray> phi_bb182_11;
  TNode<IntPtrT> phi_bb182_12;
  TNode<IntPtrT> phi_bb182_13;
  TNode<BoolT> phi_bb182_19;
      TNode<JSAny> tmp155;
  if (block182.is_used()) {
    ca_.Bind(&block182, &phi_bb182_8, &phi_bb182_10, &phi_bb182_11, &phi_bb182_12, &phi_bb182_13, &phi_bb182_19);
    compiler::CodeAssemblerExceptionHandlerLabel catch154__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch154__label);
    IteratorClose_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp122}, TNode<JSAny>{tmp123}});
    }
    if (catch154__label.is_used()) {
      compiler::CodeAssemblerLabel catch154_skip(&ca_);
      ca_.Goto(&catch154_skip);
      ca_.Bind(&catch154__label, &tmp155);
      ca_.Goto(&block186, phi_bb182_8, phi_bb182_10, phi_bb182_11, phi_bb182_12, phi_bb182_13, phi_bb182_19);
      ca_.Bind(&catch154_skip);
    }
    ca_.Goto(&block183, phi_bb182_8, phi_bb182_10, phi_bb182_11, phi_bb182_12, phi_bb182_13, phi_bb182_19);
  }

  TNode<Uint32T> phi_bb186_8;
  TNode<JSAny> phi_bb186_10;
  TNode<FixedArray> phi_bb186_11;
  TNode<IntPtrT> phi_bb186_12;
  TNode<IntPtrT> phi_bb186_13;
  TNode<BoolT> phi_bb186_19;
  TNode<Union<JSMessageObject, TheHole>> tmp156;
  TNode<IntPtrT> tmp157;
  TNode<TheHole> tmp158;
  TNode<FixedArray> tmp159;
  if (block186.is_used()) {
    ca_.Bind(&block186, &phi_bb186_8, &phi_bb186_10, &phi_bb186_11, &phi_bb186_12, &phi_bb186_13, &phi_bb186_19);
    tmp156 = GetAndResetPendingMessage_0(state_);
    tmp157 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp158 = TheHole_0(state_);
    tmp159 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb186_11}, TNode<IntPtrT>{tmp157}, TNode<IntPtrT>{phi_bb186_13}, TNode<IntPtrT>{phi_bb186_13}, TNode<Hole>{tmp158});
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp159}, false);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp155, tmp156);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Uint32T> phi_bb183_8;
  TNode<JSAny> phi_bb183_10;
  TNode<FixedArray> phi_bb183_11;
  TNode<IntPtrT> phi_bb183_12;
  TNode<IntPtrT> phi_bb183_13;
  TNode<BoolT> phi_bb183_19;
  if (block183.is_used()) {
    ca_.Bind(&block183, &phi_bb183_8, &phi_bb183_10, &phi_bb183_11, &phi_bb183_12, &phi_bb183_13, &phi_bb183_19);
    ca_.Goto(&block131, phi_bb183_8, phi_bb183_10, phi_bb183_11, phi_bb183_12, phi_bb183_13);
  }

  TNode<Uint32T> phi_bb131_8;
  TNode<JSAny> phi_bb131_10;
  TNode<FixedArray> phi_bb131_11;
  TNode<IntPtrT> phi_bb131_12;
  TNode<IntPtrT> phi_bb131_13;
  if (block131.is_used()) {
    ca_.Bind(&block131, &phi_bb131_8, &phi_bb131_10, &phi_bb131_11, &phi_bb131_12, &phi_bb131_13);
    ca_.Goto(&block129, phi_bb131_8, phi_bb131_10, phi_bb131_11, phi_bb131_12, phi_bb131_13, tmp119);
  }

  TNode<Uint32T> phi_bb129_8;
  TNode<JSAny> phi_bb129_10;
  TNode<FixedArray> phi_bb129_11;
  TNode<IntPtrT> phi_bb129_12;
  TNode<IntPtrT> phi_bb129_13;
  TNode<FixedArray> phi_bb129_14;
  TNode<IntPtrT> tmp160;
  TNode<TheHole> tmp161;
  TNode<FixedArray> tmp162;
  TNode<JSIteratorZipHelper> tmp163;
  if (block129.is_used()) {
    ca_.Bind(&block129, &phi_bb129_8, &phi_bb129_10, &phi_bb129_11, &phi_bb129_12, &phi_bb129_13, &phi_bb129_14);
    tmp160 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp161 = TheHole_0(state_);
    tmp162 = ExtractFixedArray_0(state_, TNode<FixedArray>{phi_bb129_11}, TNode<IntPtrT>{tmp160}, TNode<IntPtrT>{phi_bb129_13}, TNode<IntPtrT>{phi_bb129_13}, TNode<Hole>{tmp161});
    tmp163 = NewJSIteratorZipHelper_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp162}, TNode<FixedArray>{phi_bb129_14}, TNode<Uint32T>{phi_bb129_8});
    arguments.PopAndReturn(tmp163);
  }
}

TF_BUILTIN(IteratorZipHelperNext, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSIteratorZipHelper> parameter1 = UncheckedParameter<JSIteratorZipHelper>(Descriptor::kHelper);
  USE(parameter1);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block20(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block21(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block22(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block23(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block34(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block35(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block36(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block37(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block38(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block44(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block49(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block52(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block62(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block63(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block74(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block73(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block72(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block79(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block78(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block87(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block85(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block86(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block84(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block88(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block89(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block90(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block95(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block96(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block99(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block100(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block101(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block102(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block103(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block108(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block109(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block112(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block113(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block114(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block115(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block118(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block119(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block116(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block121(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block122(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block123(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block117(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block126(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block127(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block124(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block131(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block132(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block129(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block133(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block134(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block135(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block130(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block136(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block139(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block141(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block137(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block144(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block145(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block146(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block151(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block152(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block157(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block156(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block155(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block165(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block166(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block167(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block168(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block169(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block174(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block175(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block180(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block179(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block178(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block187(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block186(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block185(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block188(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block189(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block190(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>> block184(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>> block191(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block192(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>, IntPtrT> block193(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>> block198(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>> block199(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>> block202(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, JSAny, Union<JSMessageObject, TheHole>, JSAny, Union<JSMessageObject, TheHole>> block203(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block204(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block205(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block206(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block211(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block212(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT> block215(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block216(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block138(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block217(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block218(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block125(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block227(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block228(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block229(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block225(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block230(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block231(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block226(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block234(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block235(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block240(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block241(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block246(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block245(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block244(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, JSAny> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, JSAny> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, JSAny> block247(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, JSAny, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block252(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, JSAny, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block253(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT> block256(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT> block257(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block258(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block259(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block260(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block261(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<FixedArray> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Smi> tmp3;
  TNode<Smi> tmp4;
  TNode<BoolT> tmp5;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp1 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{parameter1, tmp0});
    tmp2 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    tmp3 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp5 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp3}, TNode<Smi>{tmp4});
    ca_.Branch(tmp5, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  TNode<Undefined> tmp6;
  TNode<True> tmp7;
  TNode<JSObject> tmp8;
  if (block1.is_used()) {
    ca_.Bind(&block1);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    tmp6 = Undefined_0(state_);
    tmp7 = True_0(state_);
    tmp8 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp6}, TNode<Boolean>{tmp7});
    CodeStubAssembler(state_).Return(tmp8);
  }

  TNode<IntPtrT> tmp9;
  TNode<FixedArray> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<Smi> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<Smi> tmp17;
  TNode<Uint32T> tmp18;
  TNode<Map> tmp19;
  TNode<FixedArray> tmp20;
      TNode<JSAny> tmp22;
  TNode<IntPtrT> tmp23;
      TNode<JSAny> tmp25;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp10 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{parameter1, tmp9});
    tmp11 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    tmp12 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{tmp10, tmp11});
    tmp13 = CodeStubAssembler(state_).SmiUntag(TNode<Smi>{tmp12});
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    tmp15 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp13}, TNode<IntPtrT>{tmp14});
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp17 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp16});
    tmp18 = SmiUntag_JSIteratorZipHelperMode_0(state_, TNode<Smi>{tmp17});
    MarkIteratorHelperAsExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    tmp19 = GetIteratorResultMap_0(state_, TNode<Context>{parameter0});
    compiler::CodeAssemblerExceptionHandlerLabel catch21__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch21__label);
    tmp20 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{tmp15});
    }
    if (catch21__label.is_used()) {
      compiler::CodeAssemblerLabel catch21_skip(&ca_);
      ca_.Goto(&catch21_skip);
      ca_.Bind(&catch21__label, &tmp22);
      ca_.Goto(&block5);
      ca_.Bind(&catch21_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch24__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch24__label);
    tmp23 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    }
    if (catch24__label.is_used()) {
      compiler::CodeAssemblerLabel catch24_skip(&ca_);
      ca_.Goto(&catch24_skip);
      ca_.Bind(&catch24__label, &tmp25);
      ca_.Goto(&block14);
      ca_.Bind(&catch24_skip);
    }
    ca_.Goto(&block17, tmp23);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp26;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp26 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp22, tmp26);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp27;
  if (block14.is_used()) {
    ca_.Bind(&block14);
    tmp27 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp25, tmp27);
  }

  TNode<IntPtrT> phi_bb17_6;
  TNode<BoolT> tmp28;
      TNode<JSAny> tmp30;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch29__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch29__label);
    tmp28 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb17_6}, TNode<IntPtrT>{tmp15});
    }
    if (catch29__label.is_used()) {
      compiler::CodeAssemblerLabel catch29_skip(&ca_);
      ca_.Goto(&catch29_skip);
      ca_.Bind(&catch29__label, &tmp30);
      ca_.Goto(&block19, phi_bb17_6, phi_bb17_6);
      ca_.Bind(&catch29_skip);
    }
    ca_.Branch(tmp28, &block15, std::vector<compiler::Node*>{phi_bb17_6}, &block16, std::vector<compiler::Node*>{phi_bb17_6});
  }

  TNode<IntPtrT> phi_bb19_6;
  TNode<IntPtrT> phi_bb19_7;
  TNode<Union<JSMessageObject, TheHole>> tmp31;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_6, &phi_bb19_7);
    tmp31 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp30, tmp31);
  }

  TNode<IntPtrT> phi_bb15_6;
  TNode<Undefined> tmp32;
  TNode<IntPtrT> tmp33;
      TNode<JSAny> tmp35;
  TNode<FixedArray> tmp36;
  TNode<Union<HeapObject, TaggedIndex>> tmp37;
  TNode<IntPtrT> tmp38;
  TNode<IntPtrT> tmp39;
      TNode<JSAny> tmp41;
  TNode<IntPtrT> tmp42;
      TNode<JSAny> tmp44;
  TNode<IntPtrT> tmp45;
      TNode<JSAny> tmp47;
  TNode<UintPtrT> tmp48;
  TNode<UintPtrT> tmp49;
  TNode<BoolT> tmp50;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_6);
    tmp32 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch34__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch34__label);
    tmp33 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    }
    if (catch34__label.is_used()) {
      compiler::CodeAssemblerLabel catch34_skip(&ca_);
      ca_.Goto(&catch34_skip);
      ca_.Bind(&catch34__label, &tmp35);
      ca_.Goto(&block20, phi_bb15_6);
      ca_.Bind(&catch34_skip);
    }
    tmp36 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{parameter1, tmp33});
    compiler::CodeAssemblerExceptionHandlerLabel catch40__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch40__label);
    std::tie(tmp37, tmp38, tmp39) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp36}).Flatten();
    }
    if (catch40__label.is_used()) {
      compiler::CodeAssemblerLabel catch40_skip(&ca_);
      ca_.Goto(&catch40_skip);
      ca_.Bind(&catch40__label, &tmp41);
      ca_.Goto(&block21, phi_bb15_6);
      ca_.Bind(&catch40_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch43__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch43__label);
    tmp42 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch43__label.is_used()) {
      compiler::CodeAssemblerLabel catch43_skip(&ca_);
      ca_.Goto(&catch43_skip);
      ca_.Bind(&catch43__label, &tmp44);
      ca_.Goto(&block22, phi_bb15_6, phi_bb15_6);
      ca_.Bind(&catch43_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch46__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch46__label);
    tmp45 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp42}, TNode<IntPtrT>{phi_bb15_6});
    }
    if (catch46__label.is_used()) {
      compiler::CodeAssemblerLabel catch46_skip(&ca_);
      ca_.Goto(&catch46_skip);
      ca_.Bind(&catch46__label, &tmp47);
      ca_.Goto(&block23, phi_bb15_6, phi_bb15_6);
      ca_.Bind(&catch46_skip);
    }
    tmp48 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp45});
    tmp49 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp39});
    tmp50 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp48}, TNode<UintPtrT>{tmp49});
    ca_.Branch(tmp50, &block28, std::vector<compiler::Node*>{phi_bb15_6}, &block29, std::vector<compiler::Node*>{phi_bb15_6});
  }

  TNode<IntPtrT> phi_bb20_6;
  TNode<Union<JSMessageObject, TheHole>> tmp51;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_6);
    tmp51 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp35, tmp51);
  }

  TNode<IntPtrT> phi_bb21_6;
  TNode<Union<JSMessageObject, TheHole>> tmp52;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_6);
    tmp52 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp41, tmp52);
  }

  TNode<IntPtrT> phi_bb22_6;
  TNode<IntPtrT> phi_bb22_13;
  TNode<Union<JSMessageObject, TheHole>> tmp53;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_6, &phi_bb22_13);
    tmp53 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp44, tmp53);
  }

  TNode<IntPtrT> phi_bb23_6;
  TNode<IntPtrT> phi_bb23_13;
  TNode<Union<JSMessageObject, TheHole>> tmp54;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_6, &phi_bb23_13);
    tmp54 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp47, tmp54);
  }

  TNode<IntPtrT> phi_bb28_6;
  TNode<IntPtrT> tmp55;
  TNode<IntPtrT> tmp56;
  TNode<Union<HeapObject, TaggedIndex>> tmp57;
  TNode<IntPtrT> tmp58;
  TNode<Object> tmp59;
  TNode<Union<HeapObject, TaggedIndex>> tmp60;
  TNode<IntPtrT> tmp61;
  TNode<IntPtrT> tmp62;
      TNode<JSAny> tmp64;
  TNode<IntPtrT> tmp65;
      TNode<JSAny> tmp67;
  TNode<IntPtrT> tmp68;
      TNode<JSAny> tmp70;
  TNode<IntPtrT> tmp71;
      TNode<JSAny> tmp73;
  TNode<IntPtrT> tmp74;
      TNode<JSAny> tmp76;
  TNode<UintPtrT> tmp77;
  TNode<UintPtrT> tmp78;
  TNode<BoolT> tmp79;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_6);
    tmp55 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp45});
    tmp56 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp38}, TNode<IntPtrT>{tmp55});
    std::tie(tmp57, tmp58) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp37}, TNode<IntPtrT>{tmp56}).Flatten();
    tmp59 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp57, tmp58});
    compiler::CodeAssemblerExceptionHandlerLabel catch63__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch63__label);
    std::tie(tmp60, tmp61, tmp62) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp36}).Flatten();
    }
    if (catch63__label.is_used()) {
      compiler::CodeAssemblerLabel catch63_skip(&ca_);
      ca_.Goto(&catch63_skip);
      ca_.Bind(&catch63__label, &tmp64);
      ca_.Goto(&block34, phi_bb28_6);
      ca_.Bind(&catch63_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch66__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch66__label);
    tmp65 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch66__label.is_used()) {
      compiler::CodeAssemblerLabel catch66_skip(&ca_);
      ca_.Goto(&catch66_skip);
      ca_.Bind(&catch66__label, &tmp67);
      ca_.Goto(&block35, phi_bb28_6, phi_bb28_6);
      ca_.Bind(&catch66_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch69__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch69__label);
    tmp68 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp65}, TNode<IntPtrT>{phi_bb28_6});
    }
    if (catch69__label.is_used()) {
      compiler::CodeAssemblerLabel catch69_skip(&ca_);
      ca_.Goto(&catch69_skip);
      ca_.Bind(&catch69__label, &tmp70);
      ca_.Goto(&block36, phi_bb28_6, phi_bb28_6);
      ca_.Bind(&catch69_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch72__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch72__label);
    tmp71 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch72__label.is_used()) {
      compiler::CodeAssemblerLabel catch72_skip(&ca_);
      ca_.Goto(&catch72_skip);
      ca_.Bind(&catch72__label, &tmp73);
      ca_.Goto(&block37, phi_bb28_6);
      ca_.Bind(&catch72_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch75__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch75__label);
    tmp74 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp68}, TNode<IntPtrT>{tmp71});
    }
    if (catch75__label.is_used()) {
      compiler::CodeAssemblerLabel catch75_skip(&ca_);
      ca_.Goto(&catch75_skip);
      ca_.Bind(&catch75__label, &tmp76);
      ca_.Goto(&block38, phi_bb28_6);
      ca_.Bind(&catch75_skip);
    }
    tmp77 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp74});
    tmp78 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp62});
    tmp79 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp77}, TNode<UintPtrT>{tmp78});
    ca_.Branch(tmp79, &block43, std::vector<compiler::Node*>{phi_bb28_6}, &block44, std::vector<compiler::Node*>{phi_bb28_6});
  }

  TNode<IntPtrT> phi_bb29_6;
  if (block29.is_used()) {
    ca_.Bind(&block29, &phi_bb29_6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb34_6;
  TNode<Union<JSMessageObject, TheHole>> tmp80;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_6);
    tmp80 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp64, tmp80);
  }

  TNode<IntPtrT> phi_bb35_6;
  TNode<IntPtrT> phi_bb35_14;
  TNode<Union<JSMessageObject, TheHole>> tmp81;
  if (block35.is_used()) {
    ca_.Bind(&block35, &phi_bb35_6, &phi_bb35_14);
    tmp81 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp67, tmp81);
  }

  TNode<IntPtrT> phi_bb36_6;
  TNode<IntPtrT> phi_bb36_14;
  TNode<Union<JSMessageObject, TheHole>> tmp82;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_6, &phi_bb36_14);
    tmp82 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp70, tmp82);
  }

  TNode<IntPtrT> phi_bb37_6;
  TNode<Union<JSMessageObject, TheHole>> tmp83;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_6);
    tmp83 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp73, tmp83);
  }

  TNode<IntPtrT> phi_bb38_6;
  TNode<Union<JSMessageObject, TheHole>> tmp84;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_6);
    tmp84 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp76, tmp84);
  }

  TNode<IntPtrT> phi_bb43_6;
  TNode<IntPtrT> tmp85;
  TNode<IntPtrT> tmp86;
  TNode<Union<HeapObject, TaggedIndex>> tmp87;
  TNode<IntPtrT> tmp88;
  TNode<Object> tmp89;
  TNode<JSAny> tmp90;
      TNode<JSAny> tmp93;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_6);
    tmp85 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp74});
    tmp86 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp61}, TNode<IntPtrT>{tmp85});
    std::tie(tmp87, tmp88) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp60}, TNode<IntPtrT>{tmp86}).Flatten();
    tmp89 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp87, tmp88});
    compiler::CodeAssemblerLabel label91(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch92__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch92__label);
    tmp90 = Cast_JSAny_0(state_, TNode<Object>{tmp89}, &label91);
    }
    if (catch92__label.is_used()) {
      compiler::CodeAssemblerLabel catch92_skip(&ca_);
      ca_.Goto(&catch92_skip);
      ca_.Bind(&catch92__label, &tmp93);
      ca_.Goto(&block49, phi_bb43_6);
      ca_.Bind(&catch92_skip);
    }
    ca_.Goto(&block47, phi_bb43_6);
    if (label91.is_used()) {
      ca_.Bind(&label91);
      ca_.Goto(&block48, phi_bb43_6);
    }
  }

  TNode<IntPtrT> phi_bb44_6;
  if (block44.is_used()) {
    ca_.Bind(&block44, &phi_bb44_6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb49_6;
  TNode<Union<JSMessageObject, TheHole>> tmp94;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_6);
    tmp94 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp93, tmp94);
  }

  TNode<IntPtrT> phi_bb48_6;
  if (block48.is_used()) {
    ca_.Bind(&block48, &phi_bb48_6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb47_6;
  TNode<TheHole> tmp95;
  TNode<BoolT> tmp96;
      TNode<JSAny> tmp98;
  if (block47.is_used()) {
    ca_.Bind(&block47, &phi_bb47_6);
    tmp95 = TheHole_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch97__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch97__label);
    tmp96 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp59}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp95});
    }
    if (catch97__label.is_used()) {
      compiler::CodeAssemblerLabel catch97_skip(&ca_);
      ca_.Goto(&catch97_skip);
      ca_.Bind(&catch97__label, &tmp98);
      ca_.Goto(&block52, phi_bb47_6);
      ca_.Bind(&catch97_skip);
    }
    ca_.Branch(tmp96, &block50, std::vector<compiler::Node*>{phi_bb47_6}, &block51, std::vector<compiler::Node*>{phi_bb47_6});
  }

  TNode<IntPtrT> phi_bb52_6;
  TNode<Union<JSMessageObject, TheHole>> tmp99;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_6);
    tmp99 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp98, tmp99);
  }

  TNode<IntPtrT> phi_bb50_6;
  TNode<IntPtrT> tmp100;
      TNode<JSAny> tmp102;
  TNode<FixedArray> tmp103;
  TNode<Union<HeapObject, TaggedIndex>> tmp104;
  TNode<IntPtrT> tmp105;
  TNode<IntPtrT> tmp106;
      TNode<JSAny> tmp108;
  TNode<UintPtrT> tmp109;
  TNode<UintPtrT> tmp110;
  TNode<BoolT> tmp111;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch101__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch101__label);
    tmp100 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch101__label.is_used()) {
      compiler::CodeAssemblerLabel catch101_skip(&ca_);
      ca_.Goto(&catch101_skip);
      ca_.Bind(&catch101__label, &tmp102);
      ca_.Goto(&block62, phi_bb50_6);
      ca_.Bind(&catch101_skip);
    }
    tmp103 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{parameter1, tmp100});
    compiler::CodeAssemblerExceptionHandlerLabel catch107__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch107__label);
    std::tie(tmp104, tmp105, tmp106) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp103}).Flatten();
    }
    if (catch107__label.is_used()) {
      compiler::CodeAssemblerLabel catch107_skip(&ca_);
      ca_.Goto(&catch107_skip);
      ca_.Bind(&catch107__label, &tmp108);
      ca_.Goto(&block63, phi_bb50_6);
      ca_.Bind(&catch107_skip);
    }
    tmp109 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb50_6});
    tmp110 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp106});
    tmp111 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp109}, TNode<UintPtrT>{tmp110});
    ca_.Branch(tmp111, &block68, std::vector<compiler::Node*>{phi_bb50_6, phi_bb50_6, phi_bb50_6, phi_bb50_6, phi_bb50_6}, &block69, std::vector<compiler::Node*>{phi_bb50_6, phi_bb50_6, phi_bb50_6, phi_bb50_6, phi_bb50_6});
  }

  TNode<IntPtrT> phi_bb62_6;
  TNode<Union<JSMessageObject, TheHole>> tmp112;
  if (block62.is_used()) {
    ca_.Bind(&block62, &phi_bb62_6);
    tmp112 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp102, tmp112);
  }

  TNode<IntPtrT> phi_bb63_6;
  TNode<Union<JSMessageObject, TheHole>> tmp113;
  if (block63.is_used()) {
    ca_.Bind(&block63, &phi_bb63_6);
    tmp113 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp108, tmp113);
  }

  TNode<IntPtrT> phi_bb68_6;
  TNode<IntPtrT> phi_bb68_18;
  TNode<IntPtrT> phi_bb68_19;
  TNode<IntPtrT> phi_bb68_23;
  TNode<IntPtrT> phi_bb68_24;
  TNode<IntPtrT> tmp114;
  TNode<IntPtrT> tmp115;
  TNode<Union<HeapObject, TaggedIndex>> tmp116;
  TNode<IntPtrT> tmp117;
  TNode<Object> tmp118;
  TNode<JSAny> tmp119;
      TNode<JSAny> tmp122;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_6, &phi_bb68_18, &phi_bb68_19, &phi_bb68_23, &phi_bb68_24);
    tmp114 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb68_24});
    tmp115 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp105}, TNode<IntPtrT>{tmp114});
    std::tie(tmp116, tmp117) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp104}, TNode<IntPtrT>{tmp115}).Flatten();
    tmp118 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp116, tmp117});
    compiler::CodeAssemblerLabel label120(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch121__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch121__label);
    tmp119 = Cast_JSAny_0(state_, TNode<Object>{tmp118}, &label120);
    }
    if (catch121__label.is_used()) {
      compiler::CodeAssemblerLabel catch121_skip(&ca_);
      ca_.Goto(&catch121_skip);
      ca_.Bind(&catch121__label, &tmp122);
      ca_.Goto(&block74, phi_bb68_6);
      ca_.Bind(&catch121_skip);
    }
    ca_.Goto(&block72, phi_bb68_6);
    if (label120.is_used()) {
      ca_.Bind(&label120);
      ca_.Goto(&block73, phi_bb68_6);
    }
  }

  TNode<IntPtrT> phi_bb69_6;
  TNode<IntPtrT> phi_bb69_18;
  TNode<IntPtrT> phi_bb69_19;
  TNode<IntPtrT> phi_bb69_23;
  TNode<IntPtrT> phi_bb69_24;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_6, &phi_bb69_18, &phi_bb69_19, &phi_bb69_23, &phi_bb69_24);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb74_6;
  TNode<Union<JSMessageObject, TheHole>> tmp123;
  if (block74.is_used()) {
    ca_.Bind(&block74, &phi_bb74_6);
    tmp123 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp122, tmp123);
  }

  TNode<IntPtrT> phi_bb73_6;
  if (block73.is_used()) {
    ca_.Bind(&block73, &phi_bb73_6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb72_6;
  if (block72.is_used()) {
    ca_.Bind(&block72, &phi_bb72_6);
    ca_.Goto(&block53, phi_bb72_6, tmp119);
  }

  TNode<IntPtrT> phi_bb51_6;
  TNode<JSReceiver> tmp124;
      TNode<JSAny> tmp127;
  if (block51.is_used()) {
    ca_.Bind(&block51, &phi_bb51_6);
    compiler::CodeAssemblerLabel label125(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch126__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch126__label);
    tmp124 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp59}, &label125);
    }
    if (catch126__label.is_used()) {
      compiler::CodeAssemblerLabel catch126_skip(&ca_);
      ca_.Goto(&catch126_skip);
      ca_.Bind(&catch126__label, &tmp127);
      ca_.Goto(&block79, phi_bb51_6);
      ca_.Bind(&catch126_skip);
    }
    ca_.Goto(&block77, phi_bb51_6);
    if (label125.is_used()) {
      ca_.Bind(&label125);
      ca_.Goto(&block78, phi_bb51_6);
    }
  }

  TNode<IntPtrT> phi_bb79_6;
  TNode<Union<JSMessageObject, TheHole>> tmp128;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_6);
    tmp128 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp127, tmp128);
  }

  TNode<IntPtrT> phi_bb78_6;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb77_6;
  TNode<JSAny> tmp129;
    compiler::TypedCodeAssemblerVariable<JSAny> tmp132(&ca_);
    compiler::TypedCodeAssemblerVariable<Union<JSMessageObject, TheHole>> tmp133(&ca_);
      TNode<JSAny> tmp135;
  if (block77.is_used()) {
    ca_.Bind(&block77, &phi_bb77_6);
    compiler::CodeAssemblerLabel label130(&ca_);
    compiler::CodeAssemblerLabel label131(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch134__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch134__label);
    tmp129 = IteratorStepValue_0(state_, TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp124}, TNode<JSAny>{tmp90}}, TNode<Map>{tmp19}, &label130, &label131, &tmp132, &tmp133);
    }
    if (catch134__label.is_used()) {
      compiler::CodeAssemblerLabel catch134_skip(&ca_);
      ca_.Goto(&catch134_skip);
      ca_.Bind(&catch134__label, &tmp135);
      ca_.Goto(&block87, phi_bb77_6);
      ca_.Bind(&catch134_skip);
    }
    ca_.Goto(&block84, phi_bb77_6);
    if (label130.is_used()) {
      ca_.Bind(&label130);
      ca_.Goto(&block85, phi_bb77_6);
    }
    if (label131.is_used()) {
      ca_.Bind(&label131);
      ca_.Goto(&block86, phi_bb77_6);
    }
  }

  TNode<IntPtrT> phi_bb87_6;
  TNode<Union<JSMessageObject, TheHole>> tmp136;
  if (block87.is_used()) {
    ca_.Bind(&block87, &phi_bb87_6);
    tmp136 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp135, tmp136);
  }

  TNode<IntPtrT> phi_bb85_6;
  TNode<Union<HeapObject, TaggedIndex>> tmp137;
  TNode<IntPtrT> tmp138;
  TNode<IntPtrT> tmp139;
      TNode<JSAny> tmp141;
  TNode<IntPtrT> tmp142;
      TNode<JSAny> tmp144;
  TNode<IntPtrT> tmp145;
      TNode<JSAny> tmp147;
  TNode<UintPtrT> tmp148;
  TNode<UintPtrT> tmp149;
  TNode<BoolT> tmp150;
  if (block85.is_used()) {
    ca_.Bind(&block85, &phi_bb85_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch140__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch140__label);
    std::tie(tmp137, tmp138, tmp139) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp36}).Flatten();
    }
    if (catch140__label.is_used()) {
      compiler::CodeAssemblerLabel catch140_skip(&ca_);
      ca_.Goto(&catch140_skip);
      ca_.Bind(&catch140__label, &tmp141);
      ca_.Goto(&block101, phi_bb85_6);
      ca_.Bind(&catch140_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch143__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch143__label);
    tmp142 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch143__label.is_used()) {
      compiler::CodeAssemblerLabel catch143_skip(&ca_);
      ca_.Goto(&catch143_skip);
      ca_.Bind(&catch143__label, &tmp144);
      ca_.Goto(&block102, phi_bb85_6, phi_bb85_6);
      ca_.Bind(&catch143_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch146__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch146__label);
    tmp145 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp142}, TNode<IntPtrT>{phi_bb85_6});
    }
    if (catch146__label.is_used()) {
      compiler::CodeAssemblerLabel catch146_skip(&ca_);
      ca_.Goto(&catch146_skip);
      ca_.Bind(&catch146__label, &tmp147);
      ca_.Goto(&block103, phi_bb85_6, phi_bb85_6);
      ca_.Bind(&catch146_skip);
    }
    tmp148 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp145});
    tmp149 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp139});
    tmp150 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp148}, TNode<UintPtrT>{tmp149});
    ca_.Branch(tmp150, &block108, std::vector<compiler::Node*>{phi_bb85_6}, &block109, std::vector<compiler::Node*>{phi_bb85_6});
  }

  TNode<IntPtrT> phi_bb86_6;
  TNode<Union<HeapObject, TaggedIndex>> tmp151;
  TNode<IntPtrT> tmp152;
  TNode<IntPtrT> tmp153;
      TNode<JSAny> tmp155;
  TNode<IntPtrT> tmp156;
      TNode<JSAny> tmp158;
  TNode<IntPtrT> tmp159;
      TNode<JSAny> tmp161;
  TNode<UintPtrT> tmp162;
  TNode<UintPtrT> tmp163;
  TNode<BoolT> tmp164;
  if (block86.is_used()) {
    ca_.Bind(&block86, &phi_bb86_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch154__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch154__label);
    std::tie(tmp151, tmp152, tmp153) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp36}).Flatten();
    }
    if (catch154__label.is_used()) {
      compiler::CodeAssemblerLabel catch154_skip(&ca_);
      ca_.Goto(&catch154_skip);
      ca_.Bind(&catch154__label, &tmp155);
      ca_.Goto(&block88, phi_bb86_6);
      ca_.Bind(&catch154_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch157__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch157__label);
    tmp156 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch157__label.is_used()) {
      compiler::CodeAssemblerLabel catch157_skip(&ca_);
      ca_.Goto(&catch157_skip);
      ca_.Bind(&catch157__label, &tmp158);
      ca_.Goto(&block89, phi_bb86_6, phi_bb86_6);
      ca_.Bind(&catch157_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch160__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch160__label);
    tmp159 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp156}, TNode<IntPtrT>{phi_bb86_6});
    }
    if (catch160__label.is_used()) {
      compiler::CodeAssemblerLabel catch160_skip(&ca_);
      ca_.Goto(&catch160_skip);
      ca_.Bind(&catch160__label, &tmp161);
      ca_.Goto(&block90, phi_bb86_6, phi_bb86_6);
      ca_.Bind(&catch160_skip);
    }
    tmp162 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp159});
    tmp163 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp153});
    tmp164 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp162}, TNode<UintPtrT>{tmp163});
    ca_.Branch(tmp164, &block95, std::vector<compiler::Node*>{phi_bb86_6}, &block96, std::vector<compiler::Node*>{phi_bb86_6});
  }

  TNode<IntPtrT> phi_bb84_6;
  if (block84.is_used()) {
    ca_.Bind(&block84, &phi_bb84_6);
    ca_.Goto(&block80, phi_bb84_6, tmp129);
  }

  TNode<IntPtrT> phi_bb88_6;
  TNode<Union<JSMessageObject, TheHole>> tmp165;
  if (block88.is_used()) {
    ca_.Bind(&block88, &phi_bb88_6);
    tmp165 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp155, tmp165);
  }

  TNode<IntPtrT> phi_bb89_6;
  TNode<IntPtrT> phi_bb89_18;
  TNode<Union<JSMessageObject, TheHole>> tmp166;
  if (block89.is_used()) {
    ca_.Bind(&block89, &phi_bb89_6, &phi_bb89_18);
    tmp166 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp158, tmp166);
  }

  TNode<IntPtrT> phi_bb90_6;
  TNode<IntPtrT> phi_bb90_18;
  TNode<Union<JSMessageObject, TheHole>> tmp167;
  if (block90.is_used()) {
    ca_.Bind(&block90, &phi_bb90_6, &phi_bb90_18);
    tmp167 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp161, tmp167);
  }

  TNode<IntPtrT> phi_bb95_6;
  TNode<IntPtrT> tmp168;
  TNode<IntPtrT> tmp169;
  TNode<Union<HeapObject, TaggedIndex>> tmp170;
  TNode<IntPtrT> tmp171;
  TNode<TheHole> tmp172;
      TNode<JSAny> tmp174;
      TNode<JSAny> tmp176;
  if (block95.is_used()) {
    ca_.Bind(&block95, &phi_bb95_6);
    tmp168 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp159});
    tmp169 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp152}, TNode<IntPtrT>{tmp168});
    std::tie(tmp170, tmp171) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp151}, TNode<IntPtrT>{tmp169}).Flatten();
    tmp172 = TheHole_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp170, tmp171}, tmp172);
    compiler::CodeAssemblerExceptionHandlerLabel catch173__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch173__label);
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp36}, false);
    }
    if (catch173__label.is_used()) {
      compiler::CodeAssemblerLabel catch173_skip(&ca_);
      ca_.Goto(&catch173_skip);
      ca_.Bind(&catch173__label, &tmp174);
      ca_.Goto(&block99, phi_bb95_6);
      ca_.Bind(&catch173_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch175__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch175__label);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, tmp132.value(), tmp133.value());
    CodeStubAssembler(state_).Unreachable();
    }
    if (catch175__label.is_used()) {
      compiler::CodeAssemblerLabel catch175_skip(&ca_);
      ca_.Bind(&catch175__label, &tmp176);
      ca_.Goto(&block100, phi_bb95_6);
    }
  }

  TNode<IntPtrT> phi_bb96_6;
  if (block96.is_used()) {
    ca_.Bind(&block96, &phi_bb96_6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb99_6;
  TNode<Union<JSMessageObject, TheHole>> tmp177;
  if (block99.is_used()) {
    ca_.Bind(&block99, &phi_bb99_6);
    tmp177 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp174, tmp177);
  }

  TNode<IntPtrT> phi_bb100_6;
  TNode<Union<JSMessageObject, TheHole>> tmp178;
  if (block100.is_used()) {
    ca_.Bind(&block100, &phi_bb100_6);
    tmp178 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp176, tmp178);
  }

  TNode<IntPtrT> phi_bb101_6;
  TNode<Union<JSMessageObject, TheHole>> tmp179;
  if (block101.is_used()) {
    ca_.Bind(&block101, &phi_bb101_6);
    tmp179 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp141, tmp179);
  }

  TNode<IntPtrT> phi_bb102_6;
  TNode<IntPtrT> phi_bb102_16;
  TNode<Union<JSMessageObject, TheHole>> tmp180;
  if (block102.is_used()) {
    ca_.Bind(&block102, &phi_bb102_6, &phi_bb102_16);
    tmp180 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp144, tmp180);
  }

  TNode<IntPtrT> phi_bb103_6;
  TNode<IntPtrT> phi_bb103_16;
  TNode<Union<JSMessageObject, TheHole>> tmp181;
  if (block103.is_used()) {
    ca_.Bind(&block103, &phi_bb103_6, &phi_bb103_16);
    tmp181 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp147, tmp181);
  }

  TNode<IntPtrT> phi_bb108_6;
  TNode<IntPtrT> tmp182;
  TNode<IntPtrT> tmp183;
  TNode<Union<HeapObject, TaggedIndex>> tmp184;
  TNode<IntPtrT> tmp185;
  TNode<TheHole> tmp186;
  TNode<IntPtrT> tmp187;
      TNode<JSAny> tmp189;
  TNode<IntPtrT> tmp190;
      TNode<JSAny> tmp192;
  TNode<Smi> tmp193;
  TNode<Smi> tmp194;
      TNode<JSAny> tmp196;
  TNode<Smi> tmp197;
      TNode<JSAny> tmp199;
  TNode<Uint32T> tmp200;
      TNode<JSAny> tmp202;
  TNode<BoolT> tmp203;
      TNode<JSAny> tmp205;
  if (block108.is_used()) {
    ca_.Bind(&block108, &phi_bb108_6);
    tmp182 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp145});
    tmp183 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp138}, TNode<IntPtrT>{tmp182});
    std::tie(tmp184, tmp185) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp137}, TNode<IntPtrT>{tmp183}).Flatten();
    tmp186 = TheHole_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp184, tmp185}, tmp186);
    compiler::CodeAssemblerExceptionHandlerLabel catch188__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch188__label);
    tmp187 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch188__label.is_used()) {
      compiler::CodeAssemblerLabel catch188_skip(&ca_);
      ca_.Goto(&catch188_skip);
      ca_.Bind(&catch188__label, &tmp189);
      ca_.Goto(&block112, phi_bb108_6);
      ca_.Bind(&catch188_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch191__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch191__label);
    tmp190 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch191__label.is_used()) {
      compiler::CodeAssemblerLabel catch191_skip(&ca_);
      ca_.Goto(&catch191_skip);
      ca_.Bind(&catch191__label, &tmp192);
      ca_.Goto(&block113, phi_bb108_6);
      ca_.Bind(&catch191_skip);
    }
    tmp193 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp190});
    compiler::CodeAssemblerExceptionHandlerLabel catch195__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch195__label);
    tmp194 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch195__label.is_used()) {
      compiler::CodeAssemblerLabel catch195_skip(&ca_);
      ca_.Goto(&catch195_skip);
      ca_.Bind(&catch195__label, &tmp196);
      ca_.Goto(&block114, phi_bb108_6);
      ca_.Bind(&catch195_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch198__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch198__label);
    tmp197 = CodeStubAssembler(state_).SmiSub(TNode<Smi>{tmp193}, TNode<Smi>{tmp194});
    }
    if (catch198__label.is_used()) {
      compiler::CodeAssemblerLabel catch198_skip(&ca_);
      ca_.Goto(&catch198_skip);
      ca_.Bind(&catch198__label, &tmp199);
      ca_.Goto(&block115, phi_bb108_6);
      ca_.Bind(&catch198_skip);
    }
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp187}, tmp197);
    compiler::CodeAssemblerExceptionHandlerLabel catch201__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch201__label);
    tmp200 = FromConstexpr_uint32_constexpr_uint32_0(state_, CastIfEnumClass<uint32_t>(JSIteratorZipHelperMode::kShortest));
    }
    if (catch201__label.is_used()) {
      compiler::CodeAssemblerLabel catch201_skip(&ca_);
      ca_.Goto(&catch201_skip);
      ca_.Bind(&catch201__label, &tmp202);
      ca_.Goto(&block118, phi_bb108_6);
      ca_.Bind(&catch201_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch204__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch204__label);
    tmp203 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp18}, TNode<Uint32T>{tmp200});
    }
    if (catch204__label.is_used()) {
      compiler::CodeAssemblerLabel catch204_skip(&ca_);
      ca_.Goto(&catch204_skip);
      ca_.Bind(&catch204__label, &tmp205);
      ca_.Goto(&block119, phi_bb108_6);
      ca_.Bind(&catch204_skip);
    }
    ca_.Branch(tmp203, &block116, std::vector<compiler::Node*>{phi_bb108_6}, &block117, std::vector<compiler::Node*>{phi_bb108_6});
  }

  TNode<IntPtrT> phi_bb109_6;
  if (block109.is_used()) {
    ca_.Bind(&block109, &phi_bb109_6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb112_6;
  TNode<Union<JSMessageObject, TheHole>> tmp206;
  if (block112.is_used()) {
    ca_.Bind(&block112, &phi_bb112_6);
    tmp206 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp189, tmp206);
  }

  TNode<IntPtrT> phi_bb113_6;
  TNode<Union<JSMessageObject, TheHole>> tmp207;
  if (block113.is_used()) {
    ca_.Bind(&block113, &phi_bb113_6);
    tmp207 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp192, tmp207);
  }

  TNode<IntPtrT> phi_bb114_6;
  TNode<Union<JSMessageObject, TheHole>> tmp208;
  if (block114.is_used()) {
    ca_.Bind(&block114, &phi_bb114_6);
    tmp208 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp196, tmp208);
  }

  TNode<IntPtrT> phi_bb115_6;
  TNode<Union<JSMessageObject, TheHole>> tmp209;
  if (block115.is_used()) {
    ca_.Bind(&block115, &phi_bb115_6);
    tmp209 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp199, tmp209);
  }

  TNode<IntPtrT> phi_bb118_6;
  TNode<Union<JSMessageObject, TheHole>> tmp210;
  if (block118.is_used()) {
    ca_.Bind(&block118, &phi_bb118_6);
    tmp210 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp202, tmp210);
  }

  TNode<IntPtrT> phi_bb119_6;
  TNode<Union<JSMessageObject, TheHole>> tmp211;
  if (block119.is_used()) {
    ca_.Bind(&block119, &phi_bb119_6);
    tmp211 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp205, tmp211);
  }

  TNode<IntPtrT> phi_bb116_6;
      TNode<JSAny> tmp213;
      TNode<JSAny> tmp215;
  TNode<Undefined> tmp216;
  TNode<True> tmp217;
  TNode<JSObject> tmp218;
      TNode<JSAny> tmp220;
  if (block116.is_used()) {
    ca_.Bind(&block116, &phi_bb116_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch212__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch212__label);
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp36}, true);
    }
    if (catch212__label.is_used()) {
      compiler::CodeAssemblerLabel catch212_skip(&ca_);
      ca_.Goto(&catch212_skip);
      ca_.Bind(&catch212__label, &tmp213);
      ca_.Goto(&block121, phi_bb116_6);
      ca_.Bind(&catch212_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch214__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch214__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch214__label.is_used()) {
      compiler::CodeAssemblerLabel catch214_skip(&ca_);
      ca_.Goto(&catch214_skip);
      ca_.Bind(&catch214__label, &tmp215);
      ca_.Goto(&block122, phi_bb116_6);
      ca_.Bind(&catch214_skip);
    }
    tmp216 = Undefined_0(state_);
    tmp217 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch219__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch219__label);
    tmp218 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp216}, TNode<Boolean>{tmp217});
    }
    if (catch219__label.is_used()) {
      compiler::CodeAssemblerLabel catch219_skip(&ca_);
      ca_.Goto(&catch219_skip);
      ca_.Bind(&catch219__label, &tmp220);
      ca_.Goto(&block123, phi_bb116_6);
      ca_.Bind(&catch219_skip);
    }
    CodeStubAssembler(state_).Return(tmp218);
  }

  TNode<IntPtrT> phi_bb121_6;
  TNode<Union<JSMessageObject, TheHole>> tmp221;
  if (block121.is_used()) {
    ca_.Bind(&block121, &phi_bb121_6);
    tmp221 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp213, tmp221);
  }

  TNode<IntPtrT> phi_bb122_6;
  TNode<Union<JSMessageObject, TheHole>> tmp222;
  if (block122.is_used()) {
    ca_.Bind(&block122, &phi_bb122_6);
    tmp222 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp215, tmp222);
  }

  TNode<IntPtrT> phi_bb123_6;
  TNode<Union<JSMessageObject, TheHole>> tmp223;
  if (block123.is_used()) {
    ca_.Bind(&block123, &phi_bb123_6);
    tmp223 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp220, tmp223);
  }

  TNode<IntPtrT> phi_bb117_6;
  TNode<Uint32T> tmp224;
      TNode<JSAny> tmp226;
  TNode<BoolT> tmp227;
      TNode<JSAny> tmp229;
  if (block117.is_used()) {
    ca_.Bind(&block117, &phi_bb117_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch225__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch225__label);
    tmp224 = FromConstexpr_uint32_constexpr_uint32_0(state_, CastIfEnumClass<uint32_t>(JSIteratorZipHelperMode::kStrict));
    }
    if (catch225__label.is_used()) {
      compiler::CodeAssemblerLabel catch225_skip(&ca_);
      ca_.Goto(&catch225_skip);
      ca_.Bind(&catch225__label, &tmp226);
      ca_.Goto(&block126, phi_bb117_6);
      ca_.Bind(&catch225_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch228__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch228__label);
    tmp227 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp18}, TNode<Uint32T>{tmp224});
    }
    if (catch228__label.is_used()) {
      compiler::CodeAssemblerLabel catch228_skip(&ca_);
      ca_.Goto(&catch228_skip);
      ca_.Bind(&catch228__label, &tmp229);
      ca_.Goto(&block127, phi_bb117_6);
      ca_.Bind(&catch228_skip);
    }
    ca_.Branch(tmp227, &block124, std::vector<compiler::Node*>{phi_bb117_6}, &block125, std::vector<compiler::Node*>{phi_bb117_6});
  }

  TNode<IntPtrT> phi_bb126_6;
  TNode<Union<JSMessageObject, TheHole>> tmp230;
  if (block126.is_used()) {
    ca_.Bind(&block126, &phi_bb126_6);
    tmp230 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp226, tmp230);
  }

  TNode<IntPtrT> phi_bb127_6;
  TNode<Union<JSMessageObject, TheHole>> tmp231;
  if (block127.is_used()) {
    ca_.Bind(&block127, &phi_bb127_6);
    tmp231 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp229, tmp231);
  }

  TNode<IntPtrT> phi_bb124_6;
  TNode<IntPtrT> tmp232;
      TNode<JSAny> tmp234;
  TNode<BoolT> tmp235;
      TNode<JSAny> tmp237;
  if (block124.is_used()) {
    ca_.Bind(&block124, &phi_bb124_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch233__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch233__label);
    tmp232 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    }
    if (catch233__label.is_used()) {
      compiler::CodeAssemblerLabel catch233_skip(&ca_);
      ca_.Goto(&catch233_skip);
      ca_.Bind(&catch233__label, &tmp234);
      ca_.Goto(&block131, phi_bb124_6, phi_bb124_6, phi_bb124_6);
      ca_.Bind(&catch233_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch236__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch236__label);
    tmp235 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb124_6}, TNode<IntPtrT>{tmp232});
    }
    if (catch236__label.is_used()) {
      compiler::CodeAssemblerLabel catch236_skip(&ca_);
      ca_.Goto(&catch236_skip);
      ca_.Bind(&catch236__label, &tmp237);
      ca_.Goto(&block132, phi_bb124_6, phi_bb124_6);
      ca_.Bind(&catch236_skip);
    }
    ca_.Branch(tmp235, &block129, std::vector<compiler::Node*>{phi_bb124_6}, &block130, std::vector<compiler::Node*>{phi_bb124_6});
  }

  TNode<IntPtrT> phi_bb131_6;
  TNode<IntPtrT> phi_bb131_12;
  TNode<IntPtrT> phi_bb131_13;
  TNode<Union<JSMessageObject, TheHole>> tmp238;
  if (block131.is_used()) {
    ca_.Bind(&block131, &phi_bb131_6, &phi_bb131_12, &phi_bb131_13);
    tmp238 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp234, tmp238);
  }

  TNode<IntPtrT> phi_bb132_6;
  TNode<IntPtrT> phi_bb132_12;
  TNode<Union<JSMessageObject, TheHole>> tmp239;
  if (block132.is_used()) {
    ca_.Bind(&block132, &phi_bb132_6, &phi_bb132_12);
    tmp239 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp237, tmp239);
  }

  TNode<IntPtrT> phi_bb129_6;
      TNode<JSAny> tmp241;
      TNode<JSAny> tmp243;
      TNode<JSAny> tmp245;
  if (block129.is_used()) {
    ca_.Bind(&block129, &phi_bb129_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch240__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch240__label);
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp36}, false);
    }
    if (catch240__label.is_used()) {
      compiler::CodeAssemblerLabel catch240_skip(&ca_);
      ca_.Goto(&catch240_skip);
      ca_.Bind(&catch240__label, &tmp241);
      ca_.Goto(&block133, phi_bb129_6);
      ca_.Bind(&catch240_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch242__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch242__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch242__label.is_used()) {
      compiler::CodeAssemblerLabel catch242_skip(&ca_);
      ca_.Goto(&catch242_skip);
      ca_.Bind(&catch242__label, &tmp243);
      ca_.Goto(&block134, phi_bb129_6);
      ca_.Bind(&catch242_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch244__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch244__label);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kIteratorZipStrictMismatch), "Iterator.zip");
    }
    if (catch244__label.is_used()) {
      compiler::CodeAssemblerLabel catch244_skip(&ca_);
      ca_.Bind(&catch244__label, &tmp245);
      ca_.Goto(&block135, phi_bb129_6);
    }
  }

  TNode<IntPtrT> phi_bb133_6;
  TNode<Union<JSMessageObject, TheHole>> tmp246;
  if (block133.is_used()) {
    ca_.Bind(&block133, &phi_bb133_6);
    tmp246 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp241, tmp246);
  }

  TNode<IntPtrT> phi_bb134_6;
  TNode<Union<JSMessageObject, TheHole>> tmp247;
  if (block134.is_used()) {
    ca_.Bind(&block134, &phi_bb134_6);
    tmp247 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp243, tmp247);
  }

  TNode<IntPtrT> phi_bb135_6;
  TNode<Union<JSMessageObject, TheHole>> tmp248;
  if (block135.is_used()) {
    ca_.Bind(&block135, &phi_bb135_6);
    tmp248 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp245, tmp248);
  }

  TNode<IntPtrT> phi_bb130_6;
  TNode<IntPtrT> tmp249;
      TNode<JSAny> tmp251;
  if (block130.is_used()) {
    ca_.Bind(&block130, &phi_bb130_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch250__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch250__label);
    tmp249 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch250__label.is_used()) {
      compiler::CodeAssemblerLabel catch250_skip(&ca_);
      ca_.Goto(&catch250_skip);
      ca_.Bind(&catch250__label, &tmp251);
      ca_.Goto(&block136, phi_bb130_6);
      ca_.Bind(&catch250_skip);
    }
    ca_.Goto(&block139, phi_bb130_6, tmp249);
  }

  TNode<IntPtrT> phi_bb136_6;
  TNode<Union<JSMessageObject, TheHole>> tmp252;
  if (block136.is_used()) {
    ca_.Bind(&block136, &phi_bb136_6);
    tmp252 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp251, tmp252);
  }

  TNode<IntPtrT> phi_bb139_6;
  TNode<IntPtrT> phi_bb139_12;
  TNode<BoolT> tmp253;
      TNode<JSAny> tmp255;
  if (block139.is_used()) {
    ca_.Bind(&block139, &phi_bb139_6, &phi_bb139_12);
    compiler::CodeAssemblerExceptionHandlerLabel catch254__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch254__label);
    tmp253 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb139_12}, TNode<IntPtrT>{tmp15});
    }
    if (catch254__label.is_used()) {
      compiler::CodeAssemblerLabel catch254_skip(&ca_);
      ca_.Goto(&catch254_skip);
      ca_.Bind(&catch254__label, &tmp255);
      ca_.Goto(&block141, phi_bb139_6, phi_bb139_12, phi_bb139_12);
      ca_.Bind(&catch254_skip);
    }
    ca_.Branch(tmp253, &block137, std::vector<compiler::Node*>{phi_bb139_6, phi_bb139_12}, &block138, std::vector<compiler::Node*>{phi_bb139_6, phi_bb139_12});
  }

  TNode<IntPtrT> phi_bb141_6;
  TNode<IntPtrT> phi_bb141_12;
  TNode<IntPtrT> phi_bb141_13;
  TNode<Union<JSMessageObject, TheHole>> tmp256;
  if (block141.is_used()) {
    ca_.Bind(&block141, &phi_bb141_6, &phi_bb141_12, &phi_bb141_13);
    tmp256 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp255, tmp256);
  }

  TNode<IntPtrT> phi_bb137_6;
  TNode<IntPtrT> phi_bb137_12;
  TNode<Union<HeapObject, TaggedIndex>> tmp257;
  TNode<IntPtrT> tmp258;
  TNode<IntPtrT> tmp259;
      TNode<JSAny> tmp261;
  TNode<IntPtrT> tmp262;
      TNode<JSAny> tmp264;
  TNode<IntPtrT> tmp265;
      TNode<JSAny> tmp267;
  TNode<UintPtrT> tmp268;
  TNode<UintPtrT> tmp269;
  TNode<BoolT> tmp270;
  if (block137.is_used()) {
    ca_.Bind(&block137, &phi_bb137_6, &phi_bb137_12);
    compiler::CodeAssemblerExceptionHandlerLabel catch260__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch260__label);
    std::tie(tmp257, tmp258, tmp259) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp36}).Flatten();
    }
    if (catch260__label.is_used()) {
      compiler::CodeAssemblerLabel catch260_skip(&ca_);
      ca_.Goto(&catch260_skip);
      ca_.Bind(&catch260__label, &tmp261);
      ca_.Goto(&block144, phi_bb137_6, phi_bb137_12);
      ca_.Bind(&catch260_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch263__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch263__label);
    tmp262 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch263__label.is_used()) {
      compiler::CodeAssemblerLabel catch263_skip(&ca_);
      ca_.Goto(&catch263_skip);
      ca_.Bind(&catch263__label, &tmp264);
      ca_.Goto(&block145, phi_bb137_6, phi_bb137_12, phi_bb137_12);
      ca_.Bind(&catch263_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch266__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch266__label);
    tmp265 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp262}, TNode<IntPtrT>{phi_bb137_12});
    }
    if (catch266__label.is_used()) {
      compiler::CodeAssemblerLabel catch266_skip(&ca_);
      ca_.Goto(&catch266_skip);
      ca_.Bind(&catch266__label, &tmp267);
      ca_.Goto(&block146, phi_bb137_6, phi_bb137_12, phi_bb137_12);
      ca_.Bind(&catch266_skip);
    }
    tmp268 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp265});
    tmp269 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp259});
    tmp270 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp268}, TNode<UintPtrT>{tmp269});
    ca_.Branch(tmp270, &block151, std::vector<compiler::Node*>{phi_bb137_6, phi_bb137_12}, &block152, std::vector<compiler::Node*>{phi_bb137_6, phi_bb137_12});
  }

  TNode<IntPtrT> phi_bb144_6;
  TNode<IntPtrT> phi_bb144_12;
  TNode<Union<JSMessageObject, TheHole>> tmp271;
  if (block144.is_used()) {
    ca_.Bind(&block144, &phi_bb144_6, &phi_bb144_12);
    tmp271 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp261, tmp271);
  }

  TNode<IntPtrT> phi_bb145_6;
  TNode<IntPtrT> phi_bb145_12;
  TNode<IntPtrT> phi_bb145_17;
  TNode<Union<JSMessageObject, TheHole>> tmp272;
  if (block145.is_used()) {
    ca_.Bind(&block145, &phi_bb145_6, &phi_bb145_12, &phi_bb145_17);
    tmp272 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp264, tmp272);
  }

  TNode<IntPtrT> phi_bb146_6;
  TNode<IntPtrT> phi_bb146_12;
  TNode<IntPtrT> phi_bb146_17;
  TNode<Union<JSMessageObject, TheHole>> tmp273;
  if (block146.is_used()) {
    ca_.Bind(&block146, &phi_bb146_6, &phi_bb146_12, &phi_bb146_17);
    tmp273 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp267, tmp273);
  }

  TNode<IntPtrT> phi_bb151_6;
  TNode<IntPtrT> phi_bb151_12;
  TNode<IntPtrT> tmp274;
  TNode<IntPtrT> tmp275;
  TNode<Union<HeapObject, TaggedIndex>> tmp276;
  TNode<IntPtrT> tmp277;
  TNode<Object> tmp278;
  TNode<JSReceiver> tmp279;
      TNode<JSAny> tmp282;
  if (block151.is_used()) {
    ca_.Bind(&block151, &phi_bb151_6, &phi_bb151_12);
    tmp274 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp265});
    tmp275 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp258}, TNode<IntPtrT>{tmp274});
    std::tie(tmp276, tmp277) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp257}, TNode<IntPtrT>{tmp275}).Flatten();
    tmp278 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp276, tmp277});
    compiler::CodeAssemblerLabel label280(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch281__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch281__label);
    tmp279 = Cast_JSReceiver_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp278}, &label280);
    }
    if (catch281__label.is_used()) {
      compiler::CodeAssemblerLabel catch281_skip(&ca_);
      ca_.Goto(&catch281_skip);
      ca_.Bind(&catch281__label, &tmp282);
      ca_.Goto(&block157, phi_bb151_6, phi_bb151_12);
      ca_.Bind(&catch281_skip);
    }
    ca_.Goto(&block155, phi_bb151_6, phi_bb151_12);
    if (label280.is_used()) {
      ca_.Bind(&label280);
      ca_.Goto(&block156, phi_bb151_6, phi_bb151_12);
    }
  }

  TNode<IntPtrT> phi_bb152_6;
  TNode<IntPtrT> phi_bb152_12;
  if (block152.is_used()) {
    ca_.Bind(&block152, &phi_bb152_6, &phi_bb152_12);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb157_6;
  TNode<IntPtrT> phi_bb157_12;
  TNode<Union<JSMessageObject, TheHole>> tmp283;
  if (block157.is_used()) {
    ca_.Bind(&block157, &phi_bb157_6, &phi_bb157_12);
    tmp283 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp282, tmp283);
  }

  TNode<IntPtrT> phi_bb156_6;
  TNode<IntPtrT> phi_bb156_12;
  if (block156.is_used()) {
    ca_.Bind(&block156, &phi_bb156_6, &phi_bb156_12);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb155_6;
  TNode<IntPtrT> phi_bb155_12;
  TNode<Union<HeapObject, TaggedIndex>> tmp284;
  TNode<IntPtrT> tmp285;
  TNode<IntPtrT> tmp286;
      TNode<JSAny> tmp288;
  TNode<IntPtrT> tmp289;
      TNode<JSAny> tmp291;
  TNode<IntPtrT> tmp292;
      TNode<JSAny> tmp294;
  TNode<IntPtrT> tmp295;
      TNode<JSAny> tmp297;
  TNode<IntPtrT> tmp298;
      TNode<JSAny> tmp300;
  TNode<UintPtrT> tmp301;
  TNode<UintPtrT> tmp302;
  TNode<BoolT> tmp303;
  if (block155.is_used()) {
    ca_.Bind(&block155, &phi_bb155_6, &phi_bb155_12);
    compiler::CodeAssemblerExceptionHandlerLabel catch287__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch287__label);
    std::tie(tmp284, tmp285, tmp286) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp36}).Flatten();
    }
    if (catch287__label.is_used()) {
      compiler::CodeAssemblerLabel catch287_skip(&ca_);
      ca_.Goto(&catch287_skip);
      ca_.Bind(&catch287__label, &tmp288);
      ca_.Goto(&block165, phi_bb155_6, phi_bb155_12);
      ca_.Bind(&catch287_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch290__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch290__label);
    tmp289 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch290__label.is_used()) {
      compiler::CodeAssemblerLabel catch290_skip(&ca_);
      ca_.Goto(&catch290_skip);
      ca_.Bind(&catch290__label, &tmp291);
      ca_.Goto(&block166, phi_bb155_6, phi_bb155_12, phi_bb155_12);
      ca_.Bind(&catch290_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch293__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch293__label);
    tmp292 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp289}, TNode<IntPtrT>{phi_bb155_12});
    }
    if (catch293__label.is_used()) {
      compiler::CodeAssemblerLabel catch293_skip(&ca_);
      ca_.Goto(&catch293_skip);
      ca_.Bind(&catch293__label, &tmp294);
      ca_.Goto(&block167, phi_bb155_6, phi_bb155_12, phi_bb155_12);
      ca_.Bind(&catch293_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch296__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch296__label);
    tmp295 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    }
    if (catch296__label.is_used()) {
      compiler::CodeAssemblerLabel catch296_skip(&ca_);
      ca_.Goto(&catch296_skip);
      ca_.Bind(&catch296__label, &tmp297);
      ca_.Goto(&block168, phi_bb155_6, phi_bb155_12);
      ca_.Bind(&catch296_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch299__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch299__label);
    tmp298 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp292}, TNode<IntPtrT>{tmp295});
    }
    if (catch299__label.is_used()) {
      compiler::CodeAssemblerLabel catch299_skip(&ca_);
      ca_.Goto(&catch299_skip);
      ca_.Bind(&catch299__label, &tmp300);
      ca_.Goto(&block169, phi_bb155_6, phi_bb155_12);
      ca_.Bind(&catch299_skip);
    }
    tmp301 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp298});
    tmp302 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp286});
    tmp303 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp301}, TNode<UintPtrT>{tmp302});
    ca_.Branch(tmp303, &block174, std::vector<compiler::Node*>{phi_bb155_6, phi_bb155_12}, &block175, std::vector<compiler::Node*>{phi_bb155_6, phi_bb155_12});
  }

  TNode<IntPtrT> phi_bb165_6;
  TNode<IntPtrT> phi_bb165_12;
  TNode<Union<JSMessageObject, TheHole>> tmp304;
  if (block165.is_used()) {
    ca_.Bind(&block165, &phi_bb165_6, &phi_bb165_12);
    tmp304 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp288, tmp304);
  }

  TNode<IntPtrT> phi_bb166_6;
  TNode<IntPtrT> phi_bb166_12;
  TNode<IntPtrT> phi_bb166_18;
  TNode<Union<JSMessageObject, TheHole>> tmp305;
  if (block166.is_used()) {
    ca_.Bind(&block166, &phi_bb166_6, &phi_bb166_12, &phi_bb166_18);
    tmp305 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp291, tmp305);
  }

  TNode<IntPtrT> phi_bb167_6;
  TNode<IntPtrT> phi_bb167_12;
  TNode<IntPtrT> phi_bb167_18;
  TNode<Union<JSMessageObject, TheHole>> tmp306;
  if (block167.is_used()) {
    ca_.Bind(&block167, &phi_bb167_6, &phi_bb167_12, &phi_bb167_18);
    tmp306 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp294, tmp306);
  }

  TNode<IntPtrT> phi_bb168_6;
  TNode<IntPtrT> phi_bb168_12;
  TNode<Union<JSMessageObject, TheHole>> tmp307;
  if (block168.is_used()) {
    ca_.Bind(&block168, &phi_bb168_6, &phi_bb168_12);
    tmp307 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp297, tmp307);
  }

  TNode<IntPtrT> phi_bb169_6;
  TNode<IntPtrT> phi_bb169_12;
  TNode<Union<JSMessageObject, TheHole>> tmp308;
  if (block169.is_used()) {
    ca_.Bind(&block169, &phi_bb169_6, &phi_bb169_12);
    tmp308 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp300, tmp308);
  }

  TNode<IntPtrT> phi_bb174_6;
  TNode<IntPtrT> phi_bb174_12;
  TNode<IntPtrT> tmp309;
  TNode<IntPtrT> tmp310;
  TNode<Union<HeapObject, TaggedIndex>> tmp311;
  TNode<IntPtrT> tmp312;
  TNode<Object> tmp313;
  TNode<JSAny> tmp314;
      TNode<JSAny> tmp317;
  if (block174.is_used()) {
    ca_.Bind(&block174, &phi_bb174_6, &phi_bb174_12);
    tmp309 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp298});
    tmp310 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp285}, TNode<IntPtrT>{tmp309});
    std::tie(tmp311, tmp312) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp284}, TNode<IntPtrT>{tmp310}).Flatten();
    tmp313 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp311, tmp312});
    compiler::CodeAssemblerLabel label315(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch316__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch316__label);
    tmp314 = Cast_JSAny_0(state_, TNode<Object>{tmp313}, &label315);
    }
    if (catch316__label.is_used()) {
      compiler::CodeAssemblerLabel catch316_skip(&ca_);
      ca_.Goto(&catch316_skip);
      ca_.Bind(&catch316__label, &tmp317);
      ca_.Goto(&block180, phi_bb174_6, phi_bb174_12);
      ca_.Bind(&catch316_skip);
    }
    ca_.Goto(&block178, phi_bb174_6, phi_bb174_12);
    if (label315.is_used()) {
      ca_.Bind(&label315);
      ca_.Goto(&block179, phi_bb174_6, phi_bb174_12);
    }
  }

  TNode<IntPtrT> phi_bb175_6;
  TNode<IntPtrT> phi_bb175_12;
  if (block175.is_used()) {
    ca_.Bind(&block175, &phi_bb175_6, &phi_bb175_12);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb180_6;
  TNode<IntPtrT> phi_bb180_12;
  TNode<Union<JSMessageObject, TheHole>> tmp318;
  if (block180.is_used()) {
    ca_.Bind(&block180, &phi_bb180_6, &phi_bb180_12);
    tmp318 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp317, tmp318);
  }

  TNode<IntPtrT> phi_bb179_6;
  TNode<IntPtrT> phi_bb179_12;
  if (block179.is_used()) {
    ca_.Bind(&block179, &phi_bb179_6, &phi_bb179_12);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb178_6;
  TNode<IntPtrT> phi_bb178_12;
  TNode<JSReceiver> tmp319;
      TNode<JSAny> tmp322;
  if (block178.is_used()) {
    ca_.Bind(&block178, &phi_bb178_6, &phi_bb178_12);
    compiler::CodeAssemblerLabel label320(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch321__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch321__label);
    tmp319 = IteratorBuiltinsAssembler(state_).IteratorStep(TNode<Context>{parameter0}, TorqueStructIteratorRecord{TNode<JSReceiver>{tmp279}, TNode<JSAny>{tmp314}}, TNode<Map>{tmp19}, &label320);
    }
    if (catch321__label.is_used()) {
      compiler::CodeAssemblerLabel catch321_skip(&ca_);
      ca_.Goto(&catch321_skip);
      ca_.Bind(&catch321__label, &tmp322);
      ca_.Goto(&block187, phi_bb178_6, phi_bb178_12);
      ca_.Bind(&catch321_skip);
    }
    ca_.Goto(&block185, phi_bb178_6, phi_bb178_12);
    if (label320.is_used()) {
      ca_.Bind(&label320);
      ca_.Goto(&block186, phi_bb178_6, phi_bb178_12);
    }
  }

  TNode<IntPtrT> phi_bb187_6;
  TNode<IntPtrT> phi_bb187_12;
  TNode<Union<JSMessageObject, TheHole>> tmp323;
  if (block187.is_used()) {
    ca_.Bind(&block187, &phi_bb187_6, &phi_bb187_12);
    tmp323 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block184, phi_bb187_6, phi_bb187_12, tmp322, tmp323);
  }

  TNode<IntPtrT> phi_bb186_6;
  TNode<IntPtrT> phi_bb186_12;
  TNode<Union<HeapObject, TaggedIndex>> tmp324;
  TNode<IntPtrT> tmp325;
  TNode<IntPtrT> tmp326;
      TNode<JSAny> tmp328;
  TNode<IntPtrT> tmp329;
      TNode<JSAny> tmp331;
  TNode<IntPtrT> tmp332;
      TNode<JSAny> tmp334;
  TNode<UintPtrT> tmp335;
  TNode<UintPtrT> tmp336;
  TNode<BoolT> tmp337;
  if (block186.is_used()) {
    ca_.Bind(&block186, &phi_bb186_6, &phi_bb186_12);
    compiler::CodeAssemblerExceptionHandlerLabel catch327__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch327__label);
    std::tie(tmp324, tmp325, tmp326) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp36}).Flatten();
    }
    if (catch327__label.is_used()) {
      compiler::CodeAssemblerLabel catch327_skip(&ca_);
      ca_.Goto(&catch327_skip);
      ca_.Bind(&catch327__label, &tmp328);
      ca_.Goto(&block204, phi_bb186_6, phi_bb186_12);
      ca_.Bind(&catch327_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch330__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch330__label);
    tmp329 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch330__label.is_used()) {
      compiler::CodeAssemblerLabel catch330_skip(&ca_);
      ca_.Goto(&catch330_skip);
      ca_.Bind(&catch330__label, &tmp331);
      ca_.Goto(&block205, phi_bb186_6, phi_bb186_12, phi_bb186_12);
      ca_.Bind(&catch330_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch333__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch333__label);
    tmp332 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp329}, TNode<IntPtrT>{phi_bb186_12});
    }
    if (catch333__label.is_used()) {
      compiler::CodeAssemblerLabel catch333_skip(&ca_);
      ca_.Goto(&catch333_skip);
      ca_.Bind(&catch333__label, &tmp334);
      ca_.Goto(&block206, phi_bb186_6, phi_bb186_12, phi_bb186_12);
      ca_.Bind(&catch333_skip);
    }
    tmp335 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp332});
    tmp336 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp326});
    tmp337 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp335}, TNode<UintPtrT>{tmp336});
    ca_.Branch(tmp337, &block211, std::vector<compiler::Node*>{phi_bb186_6, phi_bb186_12}, &block212, std::vector<compiler::Node*>{phi_bb186_6, phi_bb186_12});
  }

  TNode<IntPtrT> phi_bb185_6;
  TNode<IntPtrT> phi_bb185_12;
      TNode<JSAny> tmp339;
      TNode<JSAny> tmp341;
      TNode<JSAny> tmp343;
  if (block185.is_used()) {
    ca_.Bind(&block185, &phi_bb185_6, &phi_bb185_12);
    compiler::CodeAssemblerExceptionHandlerLabel catch338__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch338__label);
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp36}, false);
    }
    if (catch338__label.is_used()) {
      compiler::CodeAssemblerLabel catch338_skip(&ca_);
      ca_.Goto(&catch338_skip);
      ca_.Bind(&catch338__label, &tmp339);
      ca_.Goto(&block188, phi_bb185_6, phi_bb185_12);
      ca_.Bind(&catch338_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch340__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch340__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch340__label.is_used()) {
      compiler::CodeAssemblerLabel catch340_skip(&ca_);
      ca_.Goto(&catch340_skip);
      ca_.Bind(&catch340__label, &tmp341);
      ca_.Goto(&block189, phi_bb185_6, phi_bb185_12);
      ca_.Bind(&catch340_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch342__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch342__label);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kIteratorZipStrictMismatch), "Iterator.zip");
    }
    if (catch342__label.is_used()) {
      compiler::CodeAssemblerLabel catch342_skip(&ca_);
      ca_.Bind(&catch342__label, &tmp343);
      ca_.Goto(&block190, phi_bb185_6, phi_bb185_12);
    }
  }

  TNode<IntPtrT> phi_bb188_6;
  TNode<IntPtrT> phi_bb188_12;
  TNode<Union<JSMessageObject, TheHole>> tmp344;
  if (block188.is_used()) {
    ca_.Bind(&block188, &phi_bb188_6, &phi_bb188_12);
    tmp344 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block184, phi_bb188_6, phi_bb188_12, tmp339, tmp344);
  }

  TNode<IntPtrT> phi_bb189_6;
  TNode<IntPtrT> phi_bb189_12;
  TNode<Union<JSMessageObject, TheHole>> tmp345;
  if (block189.is_used()) {
    ca_.Bind(&block189, &phi_bb189_6, &phi_bb189_12);
    tmp345 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block184, phi_bb189_6, phi_bb189_12, tmp341, tmp345);
  }

  TNode<IntPtrT> phi_bb190_6;
  TNode<IntPtrT> phi_bb190_12;
  TNode<Union<JSMessageObject, TheHole>> tmp346;
  if (block190.is_used()) {
    ca_.Bind(&block190, &phi_bb190_6, &phi_bb190_12);
    tmp346 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block184, phi_bb190_6, phi_bb190_12, tmp343, tmp346);
  }

  TNode<IntPtrT> phi_bb184_6;
  TNode<IntPtrT> phi_bb184_12;
  TNode<JSAny> phi_bb184_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb184_16;
  TNode<Union<HeapObject, TaggedIndex>> tmp347;
  TNode<IntPtrT> tmp348;
  TNode<IntPtrT> tmp349;
      TNode<JSAny> tmp351;
  TNode<IntPtrT> tmp352;
      TNode<JSAny> tmp354;
  TNode<IntPtrT> tmp355;
      TNode<JSAny> tmp357;
  TNode<UintPtrT> tmp358;
  TNode<UintPtrT> tmp359;
  TNode<BoolT> tmp360;
  if (block184.is_used()) {
    ca_.Bind(&block184, &phi_bb184_6, &phi_bb184_12, &phi_bb184_15, &phi_bb184_16);
    compiler::CodeAssemblerExceptionHandlerLabel catch350__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch350__label);
    std::tie(tmp347, tmp348, tmp349) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp36}).Flatten();
    }
    if (catch350__label.is_used()) {
      compiler::CodeAssemblerLabel catch350_skip(&ca_);
      ca_.Goto(&catch350_skip);
      ca_.Bind(&catch350__label, &tmp351);
      ca_.Goto(&block191, phi_bb184_6, phi_bb184_12, phi_bb184_15, phi_bb184_16);
      ca_.Bind(&catch350_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch353__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch353__label);
    tmp352 = FromConstexpr_intptr_constexpr_int31_0(state_, kIteratorRecordFieldCount_0(state_));
    }
    if (catch353__label.is_used()) {
      compiler::CodeAssemblerLabel catch353_skip(&ca_);
      ca_.Goto(&catch353_skip);
      ca_.Bind(&catch353__label, &tmp354);
      ca_.Goto(&block192, phi_bb184_6, phi_bb184_12, phi_bb184_15, phi_bb184_16, phi_bb184_12);
      ca_.Bind(&catch353_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch356__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch356__label);
    tmp355 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp352}, TNode<IntPtrT>{phi_bb184_12});
    }
    if (catch356__label.is_used()) {
      compiler::CodeAssemblerLabel catch356_skip(&ca_);
      ca_.Goto(&catch356_skip);
      ca_.Bind(&catch356__label, &tmp357);
      ca_.Goto(&block193, phi_bb184_6, phi_bb184_12, phi_bb184_15, phi_bb184_16, phi_bb184_12);
      ca_.Bind(&catch356_skip);
    }
    tmp358 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp355});
    tmp359 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp349});
    tmp360 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp358}, TNode<UintPtrT>{tmp359});
    ca_.Branch(tmp360, &block198, std::vector<compiler::Node*>{phi_bb184_6, phi_bb184_12, phi_bb184_15, phi_bb184_16}, &block199, std::vector<compiler::Node*>{phi_bb184_6, phi_bb184_12, phi_bb184_15, phi_bb184_16});
  }

  TNode<IntPtrT> phi_bb191_6;
  TNode<IntPtrT> phi_bb191_12;
  TNode<JSAny> phi_bb191_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb191_16;
  TNode<Union<JSMessageObject, TheHole>> tmp361;
  if (block191.is_used()) {
    ca_.Bind(&block191, &phi_bb191_6, &phi_bb191_12, &phi_bb191_15, &phi_bb191_16);
    tmp361 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp351, tmp361);
  }

  TNode<IntPtrT> phi_bb192_6;
  TNode<IntPtrT> phi_bb192_12;
  TNode<JSAny> phi_bb192_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb192_16;
  TNode<IntPtrT> phi_bb192_21;
  TNode<Union<JSMessageObject, TheHole>> tmp362;
  if (block192.is_used()) {
    ca_.Bind(&block192, &phi_bb192_6, &phi_bb192_12, &phi_bb192_15, &phi_bb192_16, &phi_bb192_21);
    tmp362 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp354, tmp362);
  }

  TNode<IntPtrT> phi_bb193_6;
  TNode<IntPtrT> phi_bb193_12;
  TNode<JSAny> phi_bb193_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb193_16;
  TNode<IntPtrT> phi_bb193_21;
  TNode<Union<JSMessageObject, TheHole>> tmp363;
  if (block193.is_used()) {
    ca_.Bind(&block193, &phi_bb193_6, &phi_bb193_12, &phi_bb193_15, &phi_bb193_16, &phi_bb193_21);
    tmp363 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp357, tmp363);
  }

  TNode<IntPtrT> phi_bb198_6;
  TNode<IntPtrT> phi_bb198_12;
  TNode<JSAny> phi_bb198_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb198_16;
  TNode<IntPtrT> tmp364;
  TNode<IntPtrT> tmp365;
  TNode<Union<HeapObject, TaggedIndex>> tmp366;
  TNode<IntPtrT> tmp367;
  TNode<TheHole> tmp368;
      TNode<JSAny> tmp370;
      TNode<JSAny> tmp372;
  if (block198.is_used()) {
    ca_.Bind(&block198, &phi_bb198_6, &phi_bb198_12, &phi_bb198_15, &phi_bb198_16);
    tmp364 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp355});
    tmp365 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp348}, TNode<IntPtrT>{tmp364});
    std::tie(tmp366, tmp367) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp347}, TNode<IntPtrT>{tmp365}).Flatten();
    tmp368 = TheHole_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp366, tmp367}, tmp368);
    compiler::CodeAssemblerExceptionHandlerLabel catch369__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch369__label);
    IteratorZipCloseAll_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp36}, false);
    }
    if (catch369__label.is_used()) {
      compiler::CodeAssemblerLabel catch369_skip(&ca_);
      ca_.Goto(&catch369_skip);
      ca_.Bind(&catch369__label, &tmp370);
      ca_.Goto(&block202, phi_bb198_6, phi_bb198_12, phi_bb198_15, phi_bb198_16);
      ca_.Bind(&catch369_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch371__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch371__label);
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb198_15, phi_bb198_16);
    CodeStubAssembler(state_).Unreachable();
    }
    if (catch371__label.is_used()) {
      compiler::CodeAssemblerLabel catch371_skip(&ca_);
      ca_.Bind(&catch371__label, &tmp372);
      ca_.Goto(&block203, phi_bb198_6, phi_bb198_12, phi_bb198_15, phi_bb198_16, phi_bb198_15, phi_bb198_16);
    }
  }

  TNode<IntPtrT> phi_bb199_6;
  TNode<IntPtrT> phi_bb199_12;
  TNode<JSAny> phi_bb199_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb199_16;
  if (block199.is_used()) {
    ca_.Bind(&block199, &phi_bb199_6, &phi_bb199_12, &phi_bb199_15, &phi_bb199_16);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb202_6;
  TNode<IntPtrT> phi_bb202_12;
  TNode<JSAny> phi_bb202_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb202_16;
  TNode<Union<JSMessageObject, TheHole>> tmp373;
  if (block202.is_used()) {
    ca_.Bind(&block202, &phi_bb202_6, &phi_bb202_12, &phi_bb202_15, &phi_bb202_16);
    tmp373 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp370, tmp373);
  }

  TNode<IntPtrT> phi_bb203_6;
  TNode<IntPtrT> phi_bb203_12;
  TNode<JSAny> phi_bb203_15;
  TNode<Union<JSMessageObject, TheHole>> phi_bb203_16;
  TNode<JSAny> phi_bb203_18;
  TNode<Union<JSMessageObject, TheHole>> phi_bb203_19;
  TNode<Union<JSMessageObject, TheHole>> tmp374;
  if (block203.is_used()) {
    ca_.Bind(&block203, &phi_bb203_6, &phi_bb203_12, &phi_bb203_15, &phi_bb203_16, &phi_bb203_18, &phi_bb203_19);
    tmp374 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp372, tmp374);
  }

  TNode<IntPtrT> phi_bb204_6;
  TNode<IntPtrT> phi_bb204_12;
  TNode<Union<JSMessageObject, TheHole>> tmp375;
  if (block204.is_used()) {
    ca_.Bind(&block204, &phi_bb204_6, &phi_bb204_12);
    tmp375 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp328, tmp375);
  }

  TNode<IntPtrT> phi_bb205_6;
  TNode<IntPtrT> phi_bb205_12;
  TNode<IntPtrT> phi_bb205_19;
  TNode<Union<JSMessageObject, TheHole>> tmp376;
  if (block205.is_used()) {
    ca_.Bind(&block205, &phi_bb205_6, &phi_bb205_12, &phi_bb205_19);
    tmp376 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp331, tmp376);
  }

  TNode<IntPtrT> phi_bb206_6;
  TNode<IntPtrT> phi_bb206_12;
  TNode<IntPtrT> phi_bb206_19;
  TNode<Union<JSMessageObject, TheHole>> tmp377;
  if (block206.is_used()) {
    ca_.Bind(&block206, &phi_bb206_6, &phi_bb206_12, &phi_bb206_19);
    tmp377 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp334, tmp377);
  }

  TNode<IntPtrT> phi_bb211_6;
  TNode<IntPtrT> phi_bb211_12;
  TNode<IntPtrT> tmp378;
  TNode<IntPtrT> tmp379;
  TNode<Union<HeapObject, TaggedIndex>> tmp380;
  TNode<IntPtrT> tmp381;
  TNode<TheHole> tmp382;
  TNode<IntPtrT> tmp383;
      TNode<JSAny> tmp385;
  TNode<IntPtrT> tmp386;
      TNode<JSAny> tmp388;
  if (block211.is_used()) {
    ca_.Bind(&block211, &phi_bb211_6, &phi_bb211_12);
    tmp378 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp332});
    tmp379 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp325}, TNode<IntPtrT>{tmp378});
    std::tie(tmp380, tmp381) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp324}, TNode<IntPtrT>{tmp379}).Flatten();
    tmp382 = TheHole_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp380, tmp381}, tmp382);
    compiler::CodeAssemblerExceptionHandlerLabel catch384__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch384__label);
    tmp383 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    }
    if (catch384__label.is_used()) {
      compiler::CodeAssemblerLabel catch384_skip(&ca_);
      ca_.Goto(&catch384_skip);
      ca_.Bind(&catch384__label, &tmp385);
      ca_.Goto(&block215, phi_bb211_6, phi_bb211_12, phi_bb211_12, phi_bb211_12);
      ca_.Bind(&catch384_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch387__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch387__label);
    tmp386 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb211_12}, TNode<IntPtrT>{tmp383});
    }
    if (catch387__label.is_used()) {
      compiler::CodeAssemblerLabel catch387_skip(&ca_);
      ca_.Goto(&catch387_skip);
      ca_.Bind(&catch387__label, &tmp388);
      ca_.Goto(&block216, phi_bb211_6, phi_bb211_12, phi_bb211_12);
      ca_.Bind(&catch387_skip);
    }
    ca_.Goto(&block139, phi_bb211_6, tmp386);
  }

  TNode<IntPtrT> phi_bb212_6;
  TNode<IntPtrT> phi_bb212_12;
  if (block212.is_used()) {
    ca_.Bind(&block212, &phi_bb212_6, &phi_bb212_12);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb215_6;
  TNode<IntPtrT> phi_bb215_12;
  TNode<IntPtrT> phi_bb215_13;
  TNode<IntPtrT> phi_bb215_14;
  TNode<Union<JSMessageObject, TheHole>> tmp389;
  if (block215.is_used()) {
    ca_.Bind(&block215, &phi_bb215_6, &phi_bb215_12, &phi_bb215_13, &phi_bb215_14);
    tmp389 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp385, tmp389);
  }

  TNode<IntPtrT> phi_bb216_6;
  TNode<IntPtrT> phi_bb216_12;
  TNode<IntPtrT> phi_bb216_13;
  TNode<Union<JSMessageObject, TheHole>> tmp390;
  if (block216.is_used()) {
    ca_.Bind(&block216, &phi_bb216_6, &phi_bb216_12, &phi_bb216_13);
    tmp390 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp388, tmp390);
  }

  TNode<IntPtrT> phi_bb138_6;
  TNode<IntPtrT> phi_bb138_12;
      TNode<JSAny> tmp392;
  TNode<Undefined> tmp393;
  TNode<True> tmp394;
  TNode<JSObject> tmp395;
      TNode<JSAny> tmp397;
  if (block138.is_used()) {
    ca_.Bind(&block138, &phi_bb138_6, &phi_bb138_12);
    compiler::CodeAssemblerExceptionHandlerLabel catch391__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch391__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch391__label.is_used()) {
      compiler::CodeAssemblerLabel catch391_skip(&ca_);
      ca_.Goto(&catch391_skip);
      ca_.Bind(&catch391__label, &tmp392);
      ca_.Goto(&block217, phi_bb138_6);
      ca_.Bind(&catch391_skip);
    }
    tmp393 = Undefined_0(state_);
    tmp394 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch396__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch396__label);
    tmp395 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp393}, TNode<Boolean>{tmp394});
    }
    if (catch396__label.is_used()) {
      compiler::CodeAssemblerLabel catch396_skip(&ca_);
      ca_.Goto(&catch396_skip);
      ca_.Bind(&catch396__label, &tmp397);
      ca_.Goto(&block218, phi_bb138_6);
      ca_.Bind(&catch396_skip);
    }
    CodeStubAssembler(state_).Return(tmp395);
  }

  TNode<IntPtrT> phi_bb217_6;
  TNode<Union<JSMessageObject, TheHole>> tmp398;
  if (block217.is_used()) {
    ca_.Bind(&block217, &phi_bb217_6);
    tmp398 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp392, tmp398);
  }

  TNode<IntPtrT> phi_bb218_6;
  TNode<Union<JSMessageObject, TheHole>> tmp399;
  if (block218.is_used()) {
    ca_.Bind(&block218, &phi_bb218_6);
    tmp399 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp397, tmp399);
  }

  TNode<IntPtrT> phi_bb125_6;
  TNode<IntPtrT> tmp400;
      TNode<JSAny> tmp402;
  TNode<Smi> tmp403;
  TNode<Smi> tmp404;
      TNode<JSAny> tmp406;
  TNode<BoolT> tmp407;
      TNode<JSAny> tmp409;
  if (block125.is_used()) {
    ca_.Bind(&block125, &phi_bb125_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch401__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch401__label);
    tmp400 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    }
    if (catch401__label.is_used()) {
      compiler::CodeAssemblerLabel catch401_skip(&ca_);
      ca_.Goto(&catch401_skip);
      ca_.Bind(&catch401__label, &tmp402);
      ca_.Goto(&block227, phi_bb125_6);
      ca_.Bind(&catch401_skip);
    }
    tmp403 = CodeStubAssembler(state_).LoadReference<Smi>(CodeStubAssembler::Reference{parameter1, tmp400});
    compiler::CodeAssemblerExceptionHandlerLabel catch405__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch405__label);
    tmp404 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    }
    if (catch405__label.is_used()) {
      compiler::CodeAssemblerLabel catch405_skip(&ca_);
      ca_.Goto(&catch405_skip);
      ca_.Bind(&catch405__label, &tmp406);
      ca_.Goto(&block228, phi_bb125_6);
      ca_.Bind(&catch405_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch408__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch408__label);
    tmp407 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp403}, TNode<Smi>{tmp404});
    }
    if (catch408__label.is_used()) {
      compiler::CodeAssemblerLabel catch408_skip(&ca_);
      ca_.Goto(&catch408_skip);
      ca_.Bind(&catch408__label, &tmp409);
      ca_.Goto(&block229, phi_bb125_6);
      ca_.Bind(&catch408_skip);
    }
    ca_.Branch(tmp407, &block225, std::vector<compiler::Node*>{phi_bb125_6}, &block226, std::vector<compiler::Node*>{phi_bb125_6});
  }

  TNode<IntPtrT> phi_bb227_6;
  TNode<Union<JSMessageObject, TheHole>> tmp410;
  if (block227.is_used()) {
    ca_.Bind(&block227, &phi_bb227_6);
    tmp410 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp402, tmp410);
  }

  TNode<IntPtrT> phi_bb228_6;
  TNode<Union<JSMessageObject, TheHole>> tmp411;
  if (block228.is_used()) {
    ca_.Bind(&block228, &phi_bb228_6);
    tmp411 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp406, tmp411);
  }

  TNode<IntPtrT> phi_bb229_6;
  TNode<Union<JSMessageObject, TheHole>> tmp412;
  if (block229.is_used()) {
    ca_.Bind(&block229, &phi_bb229_6);
    tmp412 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp409, tmp412);
  }

  TNode<IntPtrT> phi_bb225_6;
      TNode<JSAny> tmp414;
  TNode<Undefined> tmp415;
  TNode<True> tmp416;
  TNode<JSObject> tmp417;
      TNode<JSAny> tmp419;
  if (block225.is_used()) {
    ca_.Bind(&block225, &phi_bb225_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch413__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch413__label);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch413__label.is_used()) {
      compiler::CodeAssemblerLabel catch413_skip(&ca_);
      ca_.Goto(&catch413_skip);
      ca_.Bind(&catch413__label, &tmp414);
      ca_.Goto(&block230, phi_bb225_6);
      ca_.Bind(&catch413_skip);
    }
    tmp415 = Undefined_0(state_);
    tmp416 = True_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch418__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch418__label);
    tmp417 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp415}, TNode<Boolean>{tmp416});
    }
    if (catch418__label.is_used()) {
      compiler::CodeAssemblerLabel catch418_skip(&ca_);
      ca_.Goto(&catch418_skip);
      ca_.Bind(&catch418__label, &tmp419);
      ca_.Goto(&block231, phi_bb225_6);
      ca_.Bind(&catch418_skip);
    }
    CodeStubAssembler(state_).Return(tmp417);
  }

  TNode<IntPtrT> phi_bb230_6;
  TNode<Union<JSMessageObject, TheHole>> tmp420;
  if (block230.is_used()) {
    ca_.Bind(&block230, &phi_bb230_6);
    tmp420 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp414, tmp420);
  }

  TNode<IntPtrT> phi_bb231_6;
  TNode<Union<JSMessageObject, TheHole>> tmp421;
  if (block231.is_used()) {
    ca_.Bind(&block231, &phi_bb231_6);
    tmp421 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp419, tmp421);
  }

  TNode<IntPtrT> phi_bb226_6;
  TNode<IntPtrT> tmp422;
      TNode<JSAny> tmp424;
  TNode<FixedArray> tmp425;
  TNode<Union<HeapObject, TaggedIndex>> tmp426;
  TNode<IntPtrT> tmp427;
  TNode<IntPtrT> tmp428;
      TNode<JSAny> tmp430;
  TNode<UintPtrT> tmp431;
  TNode<UintPtrT> tmp432;
  TNode<BoolT> tmp433;
  if (block226.is_used()) {
    ca_.Bind(&block226, &phi_bb226_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch423__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch423__label);
    tmp422 = FromConstexpr_intptr_constexpr_int31_0(state_, 28);
    }
    if (catch423__label.is_used()) {
      compiler::CodeAssemblerLabel catch423_skip(&ca_);
      ca_.Goto(&catch423_skip);
      ca_.Bind(&catch423__label, &tmp424);
      ca_.Goto(&block234, phi_bb226_6);
      ca_.Bind(&catch423_skip);
    }
    tmp425 = CodeStubAssembler(state_).LoadReference<FixedArray>(CodeStubAssembler::Reference{parameter1, tmp422});
    compiler::CodeAssemblerExceptionHandlerLabel catch429__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch429__label);
    std::tie(tmp426, tmp427, tmp428) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp425}).Flatten();
    }
    if (catch429__label.is_used()) {
      compiler::CodeAssemblerLabel catch429_skip(&ca_);
      ca_.Goto(&catch429_skip);
      ca_.Bind(&catch429__label, &tmp430);
      ca_.Goto(&block235, phi_bb226_6);
      ca_.Bind(&catch429_skip);
    }
    tmp431 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb226_6});
    tmp432 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp428});
    tmp433 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp431}, TNode<UintPtrT>{tmp432});
    ca_.Branch(tmp433, &block240, std::vector<compiler::Node*>{phi_bb226_6, phi_bb226_6, phi_bb226_6, phi_bb226_6, phi_bb226_6}, &block241, std::vector<compiler::Node*>{phi_bb226_6, phi_bb226_6, phi_bb226_6, phi_bb226_6, phi_bb226_6});
  }

  TNode<IntPtrT> phi_bb234_6;
  TNode<Union<JSMessageObject, TheHole>> tmp434;
  if (block234.is_used()) {
    ca_.Bind(&block234, &phi_bb234_6);
    tmp434 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp424, tmp434);
  }

  TNode<IntPtrT> phi_bb235_6;
  TNode<Union<JSMessageObject, TheHole>> tmp435;
  if (block235.is_used()) {
    ca_.Bind(&block235, &phi_bb235_6);
    tmp435 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp430, tmp435);
  }

  TNode<IntPtrT> phi_bb240_6;
  TNode<IntPtrT> phi_bb240_19;
  TNode<IntPtrT> phi_bb240_20;
  TNode<IntPtrT> phi_bb240_24;
  TNode<IntPtrT> phi_bb240_25;
  TNode<IntPtrT> tmp436;
  TNode<IntPtrT> tmp437;
  TNode<Union<HeapObject, TaggedIndex>> tmp438;
  TNode<IntPtrT> tmp439;
  TNode<Object> tmp440;
  TNode<JSAny> tmp441;
      TNode<JSAny> tmp444;
  if (block240.is_used()) {
    ca_.Bind(&block240, &phi_bb240_6, &phi_bb240_19, &phi_bb240_20, &phi_bb240_24, &phi_bb240_25);
    tmp436 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb240_25});
    tmp437 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp427}, TNode<IntPtrT>{tmp436});
    std::tie(tmp438, tmp439) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp426}, TNode<IntPtrT>{tmp437}).Flatten();
    tmp440 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp438, tmp439});
    compiler::CodeAssemblerLabel label442(&ca_);
    compiler::CodeAssemblerExceptionHandlerLabel catch443__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch443__label);
    tmp441 = Cast_JSAny_0(state_, TNode<Object>{tmp440}, &label442);
    }
    if (catch443__label.is_used()) {
      compiler::CodeAssemblerLabel catch443_skip(&ca_);
      ca_.Goto(&catch443_skip);
      ca_.Bind(&catch443__label, &tmp444);
      ca_.Goto(&block246, phi_bb240_6);
      ca_.Bind(&catch443_skip);
    }
    ca_.Goto(&block244, phi_bb240_6);
    if (label442.is_used()) {
      ca_.Bind(&label442);
      ca_.Goto(&block245, phi_bb240_6);
    }
  }

  TNode<IntPtrT> phi_bb241_6;
  TNode<IntPtrT> phi_bb241_19;
  TNode<IntPtrT> phi_bb241_20;
  TNode<IntPtrT> phi_bb241_24;
  TNode<IntPtrT> phi_bb241_25;
  if (block241.is_used()) {
    ca_.Bind(&block241, &phi_bb241_6, &phi_bb241_19, &phi_bb241_20, &phi_bb241_24, &phi_bb241_25);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb246_6;
  TNode<Union<JSMessageObject, TheHole>> tmp445;
  if (block246.is_used()) {
    ca_.Bind(&block246, &phi_bb246_6);
    tmp445 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp444, tmp445);
  }

  TNode<IntPtrT> phi_bb245_6;
  if (block245.is_used()) {
    ca_.Bind(&block245, &phi_bb245_6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb244_6;
  if (block244.is_used()) {
    ca_.Bind(&block244, &phi_bb244_6);
    ca_.Goto(&block80, phi_bb244_6, tmp441);
  }

  TNode<IntPtrT> phi_bb80_6;
  TNode<JSAny> phi_bb80_7;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_6, &phi_bb80_7);
    ca_.Goto(&block53, phi_bb80_6, phi_bb80_7);
  }

  TNode<IntPtrT> phi_bb53_6;
  TNode<JSAny> phi_bb53_7;
  TNode<Union<HeapObject, TaggedIndex>> tmp446;
  TNode<IntPtrT> tmp447;
  TNode<IntPtrT> tmp448;
      TNode<JSAny> tmp450;
  TNode<UintPtrT> tmp451;
  TNode<UintPtrT> tmp452;
  TNode<BoolT> tmp453;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_6, &phi_bb53_7);
    compiler::CodeAssemblerExceptionHandlerLabel catch449__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch449__label);
    std::tie(tmp446, tmp447, tmp448) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp20}).Flatten();
    }
    if (catch449__label.is_used()) {
      compiler::CodeAssemblerLabel catch449_skip(&ca_);
      ca_.Goto(&catch449_skip);
      ca_.Bind(&catch449__label, &tmp450);
      ca_.Goto(&block247, phi_bb53_6, phi_bb53_7);
      ca_.Bind(&catch449_skip);
    }
    tmp451 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb53_6});
    tmp452 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp448});
    tmp453 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp451}, TNode<UintPtrT>{tmp452});
    ca_.Branch(tmp453, &block252, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_7, phi_bb53_6, phi_bb53_6, phi_bb53_6, phi_bb53_6}, &block253, std::vector<compiler::Node*>{phi_bb53_6, phi_bb53_7, phi_bb53_6, phi_bb53_6, phi_bb53_6, phi_bb53_6});
  }

  TNode<IntPtrT> phi_bb247_6;
  TNode<JSAny> phi_bb247_7;
  TNode<Union<JSMessageObject, TheHole>> tmp454;
  if (block247.is_used()) {
    ca_.Bind(&block247, &phi_bb247_6, &phi_bb247_7);
    tmp454 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp450, tmp454);
  }

  TNode<IntPtrT> phi_bb252_6;
  TNode<JSAny> phi_bb252_7;
  TNode<IntPtrT> phi_bb252_15;
  TNode<IntPtrT> phi_bb252_16;
  TNode<IntPtrT> phi_bb252_20;
  TNode<IntPtrT> phi_bb252_21;
  TNode<IntPtrT> tmp455;
  TNode<IntPtrT> tmp456;
  TNode<Union<HeapObject, TaggedIndex>> tmp457;
  TNode<IntPtrT> tmp458;
  TNode<IntPtrT> tmp459;
      TNode<JSAny> tmp461;
  TNode<IntPtrT> tmp462;
      TNode<JSAny> tmp464;
  if (block252.is_used()) {
    ca_.Bind(&block252, &phi_bb252_6, &phi_bb252_7, &phi_bb252_15, &phi_bb252_16, &phi_bb252_20, &phi_bb252_21);
    tmp455 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb252_21});
    tmp456 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp447}, TNode<IntPtrT>{tmp455});
    std::tie(tmp457, tmp458) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp446}, TNode<IntPtrT>{tmp456}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp457, tmp458}, phi_bb252_7);
    compiler::CodeAssemblerExceptionHandlerLabel catch460__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch460__label);
    tmp459 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    }
    if (catch460__label.is_used()) {
      compiler::CodeAssemblerLabel catch460_skip(&ca_);
      ca_.Goto(&catch460_skip);
      ca_.Bind(&catch460__label, &tmp461);
      ca_.Goto(&block256, phi_bb252_6, phi_bb252_6, phi_bb252_6);
      ca_.Bind(&catch460_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch463__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch463__label);
    tmp462 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb252_6}, TNode<IntPtrT>{tmp459});
    }
    if (catch463__label.is_used()) {
      compiler::CodeAssemblerLabel catch463_skip(&ca_);
      ca_.Goto(&catch463_skip);
      ca_.Bind(&catch463__label, &tmp464);
      ca_.Goto(&block257, phi_bb252_6, phi_bb252_6);
      ca_.Bind(&catch463_skip);
    }
    ca_.Goto(&block17, tmp462);
  }

  TNode<IntPtrT> phi_bb253_6;
  TNode<JSAny> phi_bb253_7;
  TNode<IntPtrT> phi_bb253_15;
  TNode<IntPtrT> phi_bb253_16;
  TNode<IntPtrT> phi_bb253_20;
  TNode<IntPtrT> phi_bb253_21;
  if (block253.is_used()) {
    ca_.Bind(&block253, &phi_bb253_6, &phi_bb253_7, &phi_bb253_15, &phi_bb253_16, &phi_bb253_20, &phi_bb253_21);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb256_6;
  TNode<IntPtrT> phi_bb256_7;
  TNode<IntPtrT> phi_bb256_8;
  TNode<Union<JSMessageObject, TheHole>> tmp465;
  if (block256.is_used()) {
    ca_.Bind(&block256, &phi_bb256_6, &phi_bb256_7, &phi_bb256_8);
    tmp465 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp461, tmp465);
  }

  TNode<IntPtrT> phi_bb257_6;
  TNode<IntPtrT> phi_bb257_7;
  TNode<Union<JSMessageObject, TheHole>> tmp466;
  if (block257.is_used()) {
    ca_.Bind(&block257, &phi_bb257_6, &phi_bb257_7);
    tmp466 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp464, tmp466);
  }

  TNode<IntPtrT> phi_bb16_6;
  TNode<Map> tmp467;
      TNode<JSAny> tmp469;
  TNode<JSArray> tmp470;
      TNode<JSAny> tmp472;
      TNode<JSAny> tmp474;
  TNode<False> tmp475;
  TNode<JSObject> tmp476;
      TNode<JSAny> tmp478;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_6);
    compiler::CodeAssemblerExceptionHandlerLabel catch468__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch468__label);
    tmp467 = GetFastPackedElementsJSArrayMap_0(state_, TNode<Context>{parameter0});
    }
    if (catch468__label.is_used()) {
      compiler::CodeAssemblerLabel catch468_skip(&ca_);
      ca_.Goto(&catch468_skip);
      ca_.Bind(&catch468__label, &tmp469);
      ca_.Goto(&block258);
      ca_.Bind(&catch468_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch471__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch471__label);
    tmp470 = NewJSArray_0(state_, TNode<Context>{parameter0}, TNode<Map>{tmp467}, TNode<FixedArrayBase>{tmp20});
    }
    if (catch471__label.is_used()) {
      compiler::CodeAssemblerLabel catch471_skip(&ca_);
      ca_.Goto(&catch471_skip);
      ca_.Bind(&catch471__label, &tmp472);
      ca_.Goto(&block259);
      ca_.Bind(&catch471_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch473__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch473__label);
    MarkIteratorHelperAsFinishedExecuting_0(state_, TNode<JSIteratorHelper>{parameter1});
    }
    if (catch473__label.is_used()) {
      compiler::CodeAssemblerLabel catch473_skip(&ca_);
      ca_.Goto(&catch473_skip);
      ca_.Bind(&catch473__label, &tmp474);
      ca_.Goto(&block260);
      ca_.Bind(&catch473_skip);
    }
    tmp475 = False_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch477__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch477__label);
    tmp476 = CodeStubAssembler(state_).AllocateJSIteratorResult(TNode<Context>{parameter0}, TNode<JSAny>{tmp470}, TNode<Boolean>{tmp475});
    }
    if (catch477__label.is_used()) {
      compiler::CodeAssemblerLabel catch477_skip(&ca_);
      ca_.Goto(&catch477_skip);
      ca_.Bind(&catch477__label, &tmp478);
      ca_.Goto(&block261);
      ca_.Bind(&catch477_skip);
    }
    CodeStubAssembler(state_).Return(tmp476);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp479;
  if (block258.is_used()) {
    ca_.Bind(&block258);
    tmp479 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp469, tmp479);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp480;
  if (block259.is_used()) {
    ca_.Bind(&block259);
    tmp480 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp472, tmp480);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp481;
  if (block260.is_used()) {
    ca_.Bind(&block260);
    tmp481 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp474, tmp481);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp482;
  if (block261.is_used()) {
    ca_.Bind(&block261);
    tmp482 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block4, tmp478, tmp482);
  }

  TNode<JSAny> phi_bb4_5;
  TNode<Union<JSMessageObject, TheHole>> phi_bb4_6;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_5, &phi_bb4_6);
    MarkIteratorHelperAsExhausted_0(state_, TNode<JSIteratorHelper>{parameter1});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, parameter0, phi_bb4_5, phi_bb4_6);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=11&c=7
TNode<Smi> SmiTag_JSIteratorHelperState_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_value) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Smi> tmp0;
  TNode<Smi> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).SmiFromUint32(TNode<Uint32T>{p_value});
    tmp1 = (TNode<Smi>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Smi>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=72&c=18
TNode<JSIteratorHelper> Cast_JSIteratorHelper_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o, compiler::CodeAssemblerLabel* label_CastError) {
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

  TNode<JSIteratorHelper> tmp2;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    compiler::CodeAssemblerLabel label3(&ca_);
    tmp2 = Cast_JSIteratorHelper_0(state_, TNode<HeapObject>{tmp0}, &label3);
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
  return TNode<JSIteratorHelper>{tmp2};
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1502&c=11
TNode<Smi> SmiTag_JSIteratorZipHelperMode_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_value) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Smi> tmp0;
  TNode<Smi> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).SmiFromUint32(TNode<Uint32T>{p_value});
    tmp1 = (TNode<Smi>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Smi>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1612&c=40
TorqueStructConstantIterator_Undefined_0 ConstantIterator_Undefined_0(compiler::CodeAssemblerState* state_, TNode<Undefined> p_value) {
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
  return TorqueStructConstantIterator_Undefined_0{TNode<Undefined>{p_value}};
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1612&c=15
TNode<FixedArray> NewFixedArray_ConstantIterator_Undefined_0(compiler::CodeAssemblerState* state_, TNode<IntPtrT> p_length, TorqueStructConstantIterator_Undefined_0 p_it) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{p_length}, TNode<IntPtrT>{tmp0});
    ca_.Branch(tmp1, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<FixedArray> tmp2;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp2 = kEmptyFixedArray_0(state_);
    ca_.Goto(&block1, tmp2);
  }

  TNode<IntPtrT> tmp3;
  TNode<BoolT> tmp4;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, FixedArray::kMaxLength);
    tmp4 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{p_length}, TNode<IntPtrT>{tmp3});
    ca_.Branch(tmp4, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<Smi> tmp5;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp5 = kNoContext_0(state_);
    CodeStubAssembler(state_).CallRuntime(Runtime::kFatalProcessOutOfMemoryInvalidArrayLength, tmp5);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Map> tmp6;
  TNode<Smi> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<BoolT> tmp11;
  TNode<BoolT> tmp12;
  TNode<HeapObject> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<FixedArray> tmp17;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp6 = kFixedArrayMap_0(state_);
    tmp7 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{p_length});
    tmp8 = Convert_intptr_Smi_0(state_, TNode<Smi>{tmp7});
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp10 = AddIndexedFieldSizeToObjectSize_0(state_, TNode<IntPtrT>{tmp9}, TNode<IntPtrT>{tmp8}, CastIfEnumClass<int32_t>(kTaggedSize));
    tmp11 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp12 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp13 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp10}, TNode<Map>{tmp6}, TNode<BoolT>{tmp11}, TNode<BoolT>{tmp12});
    tmp14 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp13, tmp14}, tmp6);
    tmp15 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp13, tmp15}, tmp7);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    InitializeFieldsFromIterator_Object_ConstantIterator_Undefined_0(state_, TorqueStructSlice_Object_MutableReference_Object_0{TNode<Union<HeapObject, TaggedIndex>>{tmp13}, TNode<IntPtrT>{tmp16}, TNode<IntPtrT>{tmp8}, TorqueStructUnsafe_0{}}, TorqueStructConstantIterator_Undefined_0{TNode<Undefined>{p_it.value}});
    tmp17 = TORQUE_CAST(TNode<HeapObject>{tmp13});
    ca_.Goto(&block1, tmp17);
  }

  TNode<FixedArray> phi_bb1_2;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_2);
    ca_.Goto(&block6, phi_bb1_2);
  }

  TNode<FixedArray> phi_bb6_2;
    ca_.Bind(&block6, &phi_bb6_2);
  return TNode<FixedArray>{phi_bb6_2};
}

// https://crsrc.org/c/v8/src/builtins/iterator-helpers.tq?l=1695&c=16
TNode<Uint32T> SmiUntag_JSIteratorZipHelperMode_0(compiler::CodeAssemblerState* state_, TNode<Smi> p_value) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Smi> tmp0;
  TNode<Int32T> tmp1;
  TNode<Uint32T> tmp2;
  TNode<Uint32T> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Convert_Smi_SmiTagged_JSIteratorZipHelperMode_0(state_, TNode<Smi>{p_value});
    tmp1 = CodeStubAssembler(state_).SmiToInt32(TNode<Smi>{tmp0});
    tmp2 = CodeStubAssembler(state_).Unsigned(TNode<Int32T>{tmp1});
    tmp3 = (TNode<Uint32T>{tmp2});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<Uint32T>{tmp3};
}

} // namespace internal
} // namespace v8
