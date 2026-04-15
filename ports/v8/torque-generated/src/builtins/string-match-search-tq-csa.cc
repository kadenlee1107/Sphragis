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
#include "torque-generated/src/builtins/string-match-search-tq-csa.h"
#include "torque-generated/src/objects/intl-objects-tq-csa.h"
#include "torque-generated/src/builtins/string-replaceall-tq-csa.h"
#include "torque-generated/src/builtins/regexp-replace-tq-csa.h"
#include "torque-generated/src/builtins/ic-callable-tq-csa.h"
#include "torque-generated/src/builtins/string-match-search-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"
#include "torque-generated/src/builtins/regexp-search-tq-csa.h"
#include "torque-generated/src/builtins/regexp-match-tq-csa.h"
#include "torque-generated/src/builtins/regexp-tq-csa.h"

namespace v8 {
namespace internal {

TF_BUILTIN(StringPrototypeMatch, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kRegexp);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = StringMatchSearch_StringMatchFunctor_0(state_, TNode<NativeContext>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSAny>{parameter2}, TorqueStructStringMatchFunctor_0{}, "String.prototype.match");
    CodeStubAssembler(state_).Return(tmp0);
  }
}

TF_BUILTIN(StringPrototypeSearch, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<NativeContext> parameter0 = UncheckedParameter<NativeContext>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kRegexp);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = StringMatchSearch_StringSearchFunctor_0(state_, TNode<NativeContext>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSAny>{parameter2}, TorqueStructStringSearchFunctor_0{}, "String.prototype.search");
    CodeStubAssembler(state_).Return(tmp0);
  }
}

// https://crsrc.org/c/v8/src/builtins/string-match-search.tq?l=77&c=10
TNode<JSAny> StringMatchSearch_StringMatchFunctor_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<JSAny> p_receiver, TNode<JSAny> p_regexp, TorqueStructStringMatchFunctor_0 p_functor, const char* p_methodName) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  TNode<String> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = RequireObjectCoercible_0(state_, TNode<Context>{p_context}, TNode<JSAny>{p_receiver}, p_methodName);
    compiler::CodeAssemblerLabel label2(&ca_);
    tmp1 = Cast_String_1(state_, TNode<Context>{p_context}, TNode<Object>{p_receiver}, &label2);
    ca_.Goto(&block4);
    if (label2.is_used()) {
      ca_.Bind(&label2);
      ca_.Goto(&block5);
    }
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block3);
  }

  TNode<HeapObject> tmp3;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    compiler::CodeAssemblerLabel label4(&ca_);
    tmp3 = Cast_HeapObject_1(state_, TNode<Context>{p_context}, TNode<Object>{p_regexp}, &label4);
    ca_.Goto(&block6);
    if (label4.is_used()) {
      ca_.Bind(&label4);
      ca_.Goto(&block7);
    }
  }

  if (block7.is_used()) {
    ca_.Bind(&block7);
    ca_.Goto(&block3);
  }

  TNode<BoolT> tmp5;
  TNode<BoolT> tmp6;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp5 = IsFastRegExpForMatch_0(state_, TNode<Context>{p_context}, TNode<HeapObject>{tmp3});
    tmp6 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp5});
    ca_.Branch(tmp6, &block8, std::vector<compiler::Node*>{}, &block9, std::vector<compiler::Node*>{});
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    ca_.Goto(&block3);
  }

  TNode<JSRegExp> tmp7;
  TNode<JSAny> tmp8;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp7 = UnsafeCast_FastJSRegExp_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp3});
    tmp8 = ca_.CallBuiltin<JSAny>(Builtin::kRegExpMatchFast, p_context, tmp7, tmp1);
    ca_.Goto(&block1, tmp8);
  }

  TNode<BoolT> tmp9;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp9 = Is_JSReceiver_JSAny_0(state_, TNode<Context>{p_context}, TNode<JSAny>{p_regexp});
    ca_.Branch(tmp9, &block12, std::vector<compiler::Node*>{}, &block13, std::vector<compiler::Node*>{});
  }

  TNode<Symbol> tmp10;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp11;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp10 = CodeStubAssembler(state_).MatchSymbolConstant();
    compiler::CodeAssemblerLabel label12(&ca_);
    tmp11 = GetMethod_3(state_, TNode<Context>{p_context}, TNode<JSAny>{p_regexp}, TNode<Symbol>{tmp10}, &label12);
    ca_.Goto(&block17);
    if (label12.is_used()) {
      ca_.Bind(&label12);
      ca_.Goto(&block18);
    }
  }

  if (block18.is_used()) {
    ca_.Bind(&block18);
    ca_.Goto(&block13);
  }

  TNode<JSAny> tmp13;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp13 = CodeStubAssembler(state_).Call(TNode<Context>{p_context}, TNode<JSAny>{tmp11}, TNode<JSAny>{p_regexp}, TNode<JSAny>{p_receiver});
    ca_.Goto(&block1, tmp13);
  }

  TNode<String> tmp14;
  TNode<String> tmp15;
  TNode<JSAny> tmp16;
  TNode<Symbol> tmp17;
  TNode<JSAny> tmp18;
  TNode<JSAny> tmp19;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp14 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{p_context}, TNode<JSAny>{p_receiver});
    tmp15 = kEmptyString_0(state_);
    tmp16 = RegExpCreate_0(state_, TNode<Context>{p_context}, TNode<NativeContext>{p_context}, TNode<JSAny>{p_regexp}, TNode<String>{tmp15});
    tmp17 = CodeStubAssembler(state_).MatchSymbolConstant();
    tmp18 = CodeStubAssembler(state_).GetProperty(TNode<Context>{p_context}, TNode<JSAny>{tmp16}, TNode<JSAny>{tmp17});
    tmp19 = CodeStubAssembler(state_).Call(TNode<Context>{p_context}, TNode<JSAny>{tmp18}, TNode<JSAny>{tmp16}, TNode<JSAny>{tmp14});
    ca_.Goto(&block1, tmp19);
  }

  TNode<JSAny> phi_bb1_3;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_3);
    ca_.Goto(&block20);
  }

    ca_.Bind(&block20);
  return TNode<JSAny>{phi_bb1_3};
}

// https://crsrc.org/c/v8/src/builtins/string-match-search.tq?l=85&c=10
TNode<JSAny> StringMatchSearch_StringSearchFunctor_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TNode<JSAny> p_receiver, TNode<JSAny> p_regexp, TorqueStructStringSearchFunctor_0 p_functor, const char* p_methodName) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  TNode<String> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = RequireObjectCoercible_0(state_, TNode<Context>{p_context}, TNode<JSAny>{p_receiver}, p_methodName);
    compiler::CodeAssemblerLabel label2(&ca_);
    tmp1 = Cast_String_1(state_, TNode<Context>{p_context}, TNode<Object>{p_receiver}, &label2);
    ca_.Goto(&block4);
    if (label2.is_used()) {
      ca_.Bind(&label2);
      ca_.Goto(&block5);
    }
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block3);
  }

  TNode<HeapObject> tmp3;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    compiler::CodeAssemblerLabel label4(&ca_);
    tmp3 = Cast_HeapObject_1(state_, TNode<Context>{p_context}, TNode<Object>{p_regexp}, &label4);
    ca_.Goto(&block6);
    if (label4.is_used()) {
      ca_.Bind(&label4);
      ca_.Goto(&block7);
    }
  }

  if (block7.is_used()) {
    ca_.Bind(&block7);
    ca_.Goto(&block3);
  }

  TNode<BoolT> tmp5;
  TNode<BoolT> tmp6;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp5 = IsFastRegExpForSearch_0(state_, TNode<Context>{p_context}, TNode<HeapObject>{tmp3});
    tmp6 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp5});
    ca_.Branch(tmp6, &block8, std::vector<compiler::Node*>{}, &block9, std::vector<compiler::Node*>{});
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    ca_.Goto(&block3);
  }

  TNode<JSRegExp> tmp7;
  TNode<JSAny> tmp8;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp7 = UnsafeCast_FastJSRegExp_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp3});
    tmp8 = ca_.CallBuiltin<JSAny>(Builtin::kRegExpSearchFast, p_context, tmp7, tmp1);
    ca_.Goto(&block1, tmp8);
  }

  TNode<BoolT> tmp9;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp9 = Is_JSReceiver_JSAny_0(state_, TNode<Context>{p_context}, TNode<JSAny>{p_regexp});
    ca_.Branch(tmp9, &block12, std::vector<compiler::Node*>{}, &block13, std::vector<compiler::Node*>{});
  }

  TNode<Symbol> tmp10;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp11;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp10 = CodeStubAssembler(state_).SearchSymbolConstant();
    compiler::CodeAssemblerLabel label12(&ca_);
    tmp11 = GetMethod_3(state_, TNode<Context>{p_context}, TNode<JSAny>{p_regexp}, TNode<Symbol>{tmp10}, &label12);
    ca_.Goto(&block17);
    if (label12.is_used()) {
      ca_.Bind(&label12);
      ca_.Goto(&block18);
    }
  }

  if (block18.is_used()) {
    ca_.Bind(&block18);
    ca_.Goto(&block13);
  }

  TNode<JSAny> tmp13;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp13 = CodeStubAssembler(state_).Call(TNode<Context>{p_context}, TNode<JSAny>{tmp11}, TNode<JSAny>{p_regexp}, TNode<JSAny>{p_receiver});
    ca_.Goto(&block1, tmp13);
  }

  TNode<String> tmp14;
  TNode<String> tmp15;
  TNode<JSAny> tmp16;
  TNode<Symbol> tmp17;
  TNode<JSAny> tmp18;
  TNode<JSAny> tmp19;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp14 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{p_context}, TNode<JSAny>{p_receiver});
    tmp15 = kEmptyString_0(state_);
    tmp16 = RegExpCreate_0(state_, TNode<Context>{p_context}, TNode<NativeContext>{p_context}, TNode<JSAny>{p_regexp}, TNode<String>{tmp15});
    tmp17 = CodeStubAssembler(state_).SearchSymbolConstant();
    tmp18 = CodeStubAssembler(state_).GetProperty(TNode<Context>{p_context}, TNode<JSAny>{tmp16}, TNode<JSAny>{tmp17});
    tmp19 = CodeStubAssembler(state_).Call(TNode<Context>{p_context}, TNode<JSAny>{tmp18}, TNode<JSAny>{tmp16}, TNode<JSAny>{tmp14});
    ca_.Goto(&block1, tmp19);
  }

  TNode<JSAny> phi_bb1_3;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_3);
    ca_.Goto(&block20);
  }

    ca_.Bind(&block20);
  return TNode<JSAny>{phi_bb1_3};
}

} // namespace internal
} // namespace v8
