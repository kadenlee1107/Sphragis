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
#include "torque-generated/src/builtins/array-join-tq-csa.h"
#include "torque-generated/src/objects/arguments-tq-csa.h"
#include "torque-generated/src/objects/js-array-tq-csa.h"
#include "torque-generated/src/objects/fixed-array-tq-csa.h"
#include "torque-generated/src/objects/intl-objects-tq-csa.h"
#include "torque-generated/src/objects/js-array-buffer-tq-csa.h"
#include "torque-generated/src/objects/contexts-tq-csa.h"
#include "torque-generated/src/objects/string-tq-csa.h"
#include "torque-generated/src/builtins/array-find-tq-csa.h"
#include "torque-generated/src/builtins/array-to-spliced-tq-csa.h"
#include "torque-generated/src/builtins/convert-tq-csa.h"
#include "torque-generated/src/builtins/frame-arguments-tq-csa.h"
#include "torque-generated/src/builtins/typed-array-tq-csa.h"
#include "torque-generated/src/builtins/regexp-replace-tq-csa.h"
#include "torque-generated/src/builtins/torque-internal-tq-csa.h"
#include "torque-generated/src/builtins/cast-tq-csa.h"
#include "torque-generated/src/builtins/typed-array-to-sorted-tq-csa.h"
#include "torque-generated/src/builtins/internal-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"
#include "torque-generated/src/builtins/array-join-tq-csa.h"
#include "torque-generated/src/builtins/builtins-string-tq-csa.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=12&c=1
int31_t kMaxBufferChunkSize_0(compiler::CodeAssemblerState* state_) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

    ca_.Bind(&block0);
  return FixedArray::kMaxRegularLength;}

TF_BUILTIN(LoadJoinElement_DictionaryElements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSArray> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<FixedArrayBase> tmp2;
  TNode<NumberDictionary> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<JSAny> tmp5;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp2 = CodeStubAssembler(state_).LoadReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp0, tmp1});
    tmp3 = UnsafeCast_NumberDictionary_0(state_, TNode<Context>{parameter0}, TNode<Object>{tmp2});
    tmp4 = CodeStubAssembler(state_).Signed(TNode<UintPtrT>{parameter2});
    compiler::CodeAssemblerLabel label6(&ca_);
    compiler::CodeAssemblerLabel label7(&ca_);
    tmp5 = CodeStubAssembler(state_).BasicLoadNumberDictionaryElement(TNode<NumberDictionary>{tmp3}, TNode<IntPtrT>{tmp4}, &label6, &label7);
    ca_.Goto(&block5);
    if (label6.is_used()) {
      ca_.Bind(&label6);
      ca_.Goto(&block6);
    }
    if (label7.is_used()) {
      ca_.Bind(&label7);
      ca_.Goto(&block7);
    }
  }

  TNode<Number> tmp8;
  TNode<JSAny> tmp9;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp8 = Convert_Number_uintptr_0(state_, TNode<UintPtrT>{parameter2});
    tmp9 = CodeStubAssembler(state_).GetProperty(TNode<Context>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSAny>{tmp8});
    CodeStubAssembler(state_).Return(tmp9);
  }

  TNode<String> tmp10;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp10 = kEmptyString_0(state_);
    CodeStubAssembler(state_).Return(tmp10);
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    CodeStubAssembler(state_).Return(tmp5);
  }
}

TF_BUILTIN(LoadJoinElement_FastSmiOrObjectElements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSArray> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<FixedArrayBase> tmp2;
  TNode<FixedArray> tmp3;
  TNode<Union<HeapObject, TaggedIndex>> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<UintPtrT> tmp8;
  TNode<UintPtrT> tmp9;
  TNode<BoolT> tmp10;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp2 = CodeStubAssembler(state_).LoadReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp0, tmp1});
    tmp3 = UnsafeCast_FixedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{tmp2});
    std::tie(tmp4, tmp5, tmp6) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp3}).Flatten();
    tmp7 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{parameter2});
    tmp8 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp7});
    tmp9 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp6});
    tmp10 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp8}, TNode<UintPtrT>{tmp9});
    ca_.Branch(tmp10, &block5, std::vector<compiler::Node*>{}, &block6, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<Union<HeapObject, TaggedIndex>> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<Object> tmp15;
  TNode<TheHole> tmp16;
  TNode<BoolT> tmp17;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp11 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp7});
    tmp12 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp5}, TNode<IntPtrT>{tmp11});
    std::tie(tmp13, tmp14) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp4}, TNode<IntPtrT>{tmp12}).Flatten();
    tmp15 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp13, tmp14});
    tmp16 = TheHole_0(state_);
    tmp17 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp15}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp16});
    ca_.Branch(tmp17, &block9, std::vector<compiler::Node*>{}, &block10, std::vector<compiler::Node*>{});
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> tmp18;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp18 = kEmptyString_0(state_);
    ca_.Goto(&block11, tmp18);
  }

  TNode<JSAny> tmp19;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp19 = UnsafeCast_JSAny_0(state_, TNode<Context>{parameter0}, TNode<Object>{tmp15});
    ca_.Goto(&block11, tmp19);
  }

  TNode<JSAny> phi_bb11_6;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_6);
    CodeStubAssembler(state_).Return(phi_bb11_6);
  }
}

TF_BUILTIN(LoadJoinElement_FastDoubleElements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSArray> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<FixedArrayBase> tmp2;
  TNode<FixedDoubleArray> tmp3;
  TNode<Union<HeapObject, TaggedIndex>> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<UintPtrT> tmp8;
  TNode<UintPtrT> tmp9;
  TNode<BoolT> tmp10;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp2 = CodeStubAssembler(state_).LoadReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp0, tmp1});
    tmp3 = UnsafeCast_FixedDoubleArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{tmp2});
    std::tie(tmp4, tmp5, tmp6) = FieldSliceFixedDoubleArrayValues_0(state_, TNode<FixedDoubleArray>{tmp3}).Flatten();
    tmp7 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{parameter2});
    tmp8 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp7});
    tmp9 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp6});
    tmp10 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp8}, TNode<UintPtrT>{tmp9});
    ca_.Branch(tmp10, &block9, std::vector<compiler::Node*>{}, &block10, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<Union<HeapObject, TaggedIndex>> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<BoolT> tmp15;
  TNode<Float64T> tmp16;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp11 = TimesSizeOf_float64_or_undefined_or_hole_0(state_, TNode<IntPtrT>{tmp7});
    tmp12 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp5}, TNode<IntPtrT>{tmp11});
    std::tie(tmp13, tmp14) = NewReference_float64_or_undefined_or_hole_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp4}, TNode<IntPtrT>{tmp12}).Flatten();
    std::tie(tmp15, tmp16) = LoadFloat64OrHole_0(state_, TorqueStructReference_float64_or_undefined_or_hole_0{TNode<Union<HeapObject, TaggedIndex>>{tmp13}, TNode<IntPtrT>{tmp14}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Branch(tmp15, &block14, std::vector<compiler::Node*>{}, &block15, std::vector<compiler::Node*>{});
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> tmp17;
  if (block14.is_used()) {
    ca_.Bind(&block14);
    tmp17 = kEmptyString_0(state_);
    CodeStubAssembler(state_).Return(tmp17);
  }

  TNode<HeapNumber> tmp18;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp18 = CodeStubAssembler(state_).AllocateHeapNumberWithValue(TNode<Float64T>{tmp16});
    CodeStubAssembler(state_).Return(tmp18);
  }
}

TF_BUILTIN(ConvertToLocaleString, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kElement);
  USE(parameter1);
  TNode<JSAny> parameter2 = UncheckedParameter<JSAny>(Descriptor::kLocales);
  USE(parameter2);
  TNode<JSAny> parameter3 = UncheckedParameter<JSAny>(Descriptor::kOptions);
  USE(parameter3);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).IsNullOrUndefined(TNode<Object>{parameter1});
    ca_.Branch(tmp0, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp1;
  if (block1.is_used()) {
    ca_.Bind(&block1);
    tmp1 = kEmptyString_0(state_);
    CodeStubAssembler(state_).Return(tmp1);
  }

  TNode<JSAny> tmp2;
  TNode<JSAny> tmp3;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp4;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp2 = FromConstexpr_JSAny_constexpr_string_0(state_, "toLocaleString");
    tmp3 = CodeStubAssembler(state_).GetProperty(TNode<Context>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSAny>{tmp2});
    compiler::CodeAssemblerLabel label5(&ca_);
    tmp4 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp3}, &label5);
    ca_.Goto(&block5);
    if (label5.is_used()) {
      ca_.Bind(&label5);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowCalledNonCallable, parameter0, tmp3);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> tmp6;
  TNode<String> tmp7;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp6 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp4}, TNode<JSAny>{parameter1});
    tmp7 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{tmp6});
    CodeStubAssembler(state_).Return(tmp7);
  }
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=108&c=1
TNode<BoolT> CannotUseSameArrayAccessor_JSArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BuiltinPtr> p_loadFn, TNode<JSReceiver> p_receiver, TNode<Map> p_originalMap, TNode<Number> p_originalLen) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{p_loadFn}, TNode<Smi>{ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinElement_GenericElementsAccessor_0))});
    ca_.Branch(tmp0, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp1;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp1 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block1, tmp1);
  }

  TNode<JSArray> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<Map> tmp4;
  TNode<BoolT> tmp5;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp2 = UnsafeCast_JSArray_0(state_, TNode<Context>{p_context}, TNode<Object>{p_receiver});
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp4 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp2, tmp3});
    tmp5 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{p_originalMap}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp4});
    ca_.Branch(tmp5, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp6;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp6 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp6);
  }

  TNode<IntPtrT> tmp7;
  TNode<Number> tmp8;
  TNode<BoolT> tmp9;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp8 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{tmp2, tmp7});
    tmp9 = IsNumberNotEqual_0(state_, TNode<Number>{p_originalLen}, TNode<Number>{tmp8});
    ca_.Branch(tmp9, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp10;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp10 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp10);
  }

  TNode<BoolT> tmp11;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp11 = CodeStubAssembler(state_).IsNoElementsProtectorCellInvalid();
    ca_.Branch(tmp11, &block8, std::vector<compiler::Node*>{}, &block9, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp12;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    tmp12 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp12);
  }

  TNode<BoolT> tmp13;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp13 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block1, tmp13);
  }

  TNode<BoolT> phi_bb1_5;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_5);
    ca_.Goto(&block10, phi_bb1_5);
  }

  TNode<BoolT> phi_bb10_5;
    ca_.Bind(&block10, &phi_bb10_5);
  return TNode<BoolT>{phi_bb10_5};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=120&c=1
TNode<BoolT> CannotUseSameArrayAccessor_JSTypedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<BuiltinPtr> p__loadFn, TNode<JSReceiver> p_receiver, TNode<Map> p__initialMap, TNode<Number> p__initialLen) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<JSArrayBuffer> tmp2;
  TNode<BoolT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{p_context}, TNode<Object>{p_receiver});
    tmp1 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp2 = CodeStubAssembler(state_).LoadReference<JSArrayBuffer>(CodeStubAssembler::Reference{tmp0, tmp1});
    tmp3 = IsDetachedBuffer_0(state_, TNode<JSArrayBuffer>{tmp2});
    ca_.Branch(tmp3, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp4;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp4 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp4);
  }

  TNode<BoolT> tmp5;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp5 = IsVariableLengthJSArrayBufferView_0(state_, TNode<JSArrayBufferView>{tmp0});
    ca_.Branch(tmp5, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp6;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp6 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp6);
  }

  TNode<BoolT> tmp7;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp7 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block1, tmp7);
  }

  TNode<BoolT> phi_bb1_5;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_5);
    ca_.Goto(&block6, phi_bb1_5);
  }

  TNode<BoolT> phi_bb6_5;
    ca_.Bind(&block6, &phi_bb6_5);
  return TNode<BoolT>{phi_bb6_5};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=142&c=1
TNode<IntPtrT> AddStringLength_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_lenA, TNode<IntPtrT> p_lenB) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = CodeStubAssembler(state_).TryIntPtrAdd(TNode<IntPtrT>{p_lenA}, TNode<IntPtrT>{p_lenB}, &label1);
    ca_.Goto(&block4);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block5);
    }
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block3);
  }

  TNode<IntPtrT> tmp2;
  TNode<BoolT> tmp3;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp2 = FromConstexpr_intptr_constexpr_int31_0(state_, String::kMaxLength);
    tmp3 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp0}, TNode<IntPtrT>{tmp2});
    ca_.Branch(tmp3, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    ca_.Goto(&block3);
  }

  if (block7.is_used()) {
    ca_.Bind(&block7);
    ca_.Goto(&block8);
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowInvalidStringLength, p_context);
    CodeStubAssembler(state_).Unreachable();
  }

    ca_.Bind(&block8);
  return TNode<IntPtrT>{tmp0};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=306&c=1
TorqueStructBuffer_0 NewBuffer_0(compiler::CodeAssemblerState* state_, TNode<UintPtrT> p_len, TNode<String> p_sep) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<UintPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_uintptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp1 = CodeStubAssembler(state_).UintPtrGreaterThanOrEqual(TNode<UintPtrT>{p_len}, TNode<UintPtrT>{tmp0});
    ca_.Branch(tmp1, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp2;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp2 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block4, tmp2);
  }

  TNode<UintPtrT> tmp3;
  TNode<UintPtrT> tmp4;
  TNode<IntPtrT> tmp5;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp3 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp4 = CodeStubAssembler(state_).UintPtrAdd(TNode<UintPtrT>{p_len}, TNode<UintPtrT>{tmp3});
    tmp5 = CodeStubAssembler(state_).Signed(TNode<UintPtrT>{tmp4});
    ca_.Goto(&block4, tmp5);
  }

  TNode<IntPtrT> phi_bb4_2;
  TNode<FixedArray> tmp6;
  TNode<Union<HeapObject, TaggedIndex>> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<UintPtrT> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<UintPtrT> tmp12;
  TNode<UintPtrT> tmp13;
  TNode<BoolT> tmp14;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_2);
    tmp6 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb4_2});
    std::tie(tmp7, tmp8, tmp9) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp6}).Flatten();
    tmp10 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp11 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp10});
    tmp12 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp11});
    tmp13 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp9});
    tmp14 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp12}, TNode<UintPtrT>{tmp13});
    ca_.Branch(tmp14, &block15, std::vector<compiler::Node*>{}, &block16, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<Union<HeapObject, TaggedIndex>> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<Undefined> tmp19;
  TNode<IntPtrT> tmp20;
  TNode<Map> tmp21;
  TNode<BoolT> tmp22;
  TNode<Null> tmp23;
  TNode<IntPtrT> tmp24;
  TNode<IntPtrT> tmp25;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp15 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp11});
    tmp16 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp8}, TNode<IntPtrT>{tmp15});
    std::tie(tmp17, tmp18) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp7}, TNode<IntPtrT>{tmp16}).Flatten();
    tmp19 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp17, tmp18}, tmp19);
    tmp20 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp21 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{p_sep, tmp20});
    tmp22 = CodeStubAssembler(state_).IsOneByteStringMap(TNode<Map>{tmp21});
    tmp23 = Null_0(state_);
    tmp24 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp25 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block19);
  }

  if (block16.is_used()) {
    ca_.Bind(&block16);
    CodeStubAssembler(state_).Unreachable();
  }

    ca_.Bind(&block19);
  return TorqueStructBuffer_0{TNode<FixedArray>{tmp6}, TNode<FixedArray>{tmp6}, TNode<IntPtrT>{tmp24}, TNode<IntPtrT>{tmp25}, TNode<BoolT>{tmp22}, TNode<PrimitiveHeapObject>{tmp23}};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=323&c=1
TNode<String> BufferJoin_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TorqueStructBuffer_0 p_buffer, TNode<String> p_sep) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<SeqOneByteString, SeqTwoByteString, String>> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{p_buffer.totalStringLength}, TNode<IntPtrT>{tmp0});
    ca_.Branch(tmp1, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp2;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp2 = kEmptyString_0(state_);
    ca_.Goto(&block1, tmp2);
  }

  TNode<IntPtrT> tmp3;
  TNode<BoolT> tmp4;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp3 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp4 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{p_buffer.index}, TNode<IntPtrT>{tmp3});
    ca_.Branch(tmp4, &block10, std::vector<compiler::Node*>{}, &block11, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp5;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp5 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{p_buffer.head}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{p_buffer.chunk});
    ca_.Goto(&block12, tmp5);
  }

  TNode<BoolT> tmp6;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp6 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block12, tmp6);
  }

  TNode<BoolT> phi_bb12_9;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_9);
    ca_.Branch(phi_bb12_9, &block8, std::vector<compiler::Node*>{}, &block9, std::vector<compiler::Node*>{});
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<UintPtrT> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<UintPtrT> tmp12;
  TNode<UintPtrT> tmp13;
  TNode<BoolT> tmp14;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    std::tie(tmp7, tmp8, tmp9) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{p_buffer.head}).Flatten();
    tmp10 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp11 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp10});
    tmp12 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp11});
    tmp13 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp9});
    tmp14 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp12}, TNode<UintPtrT>{tmp13});
    ca_.Branch(tmp14, &block18, std::vector<compiler::Node*>{}, &block19, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<Union<HeapObject, TaggedIndex>> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<Object> tmp19;
  TNode<String> tmp20;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp15 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp11});
    tmp16 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp8}, TNode<IntPtrT>{tmp15});
    std::tie(tmp17, tmp18) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp7}, TNode<IntPtrT>{tmp16}).Flatten();
    tmp19 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp17, tmp18});
    compiler::CodeAssemblerLabel label21(&ca_);
    tmp20 = Cast_String_1(state_, TNode<Context>{p_context}, TNode<Object>{tmp19}, &label21);
    ca_.Goto(&block24);
    if (label21.is_used()) {
      ca_.Bind(&label21);
      ca_.Goto(&block25);
    }
  }

  if (block19.is_used()) {
    ca_.Bind(&block19);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Smi> tmp22;
  if (block25.is_used()) {
    ca_.Bind(&block25);
    compiler::CodeAssemblerLabel label23(&ca_);
    tmp22 = Cast_Smi_0(state_, TNode<Object>{ca_.UncheckedCast<Object>(tmp19)}, &label23);
    ca_.Goto(&block28);
    if (label23.is_used()) {
      ca_.Bind(&label23);
      ca_.Goto(&block29);
    }
  }

  if (block24.is_used()) {
    ca_.Bind(&block24);
    ca_.Goto(&block1, tmp20);
  }

  if (block29.is_used()) {
    ca_.Bind(&block29);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> tmp24;
  if (block28.is_used()) {
    ca_.Bind(&block28);
    tmp24 = ca_.CallBuiltin<String>(Builtin::kStringRepeat, p_context, p_sep, tmp22);
    ca_.Goto(&block1, tmp24);
  }

  TNode<UintPtrT> tmp25;
  TNode<Uint32T> tmp26;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp25 = CodeStubAssembler(state_).Unsigned(TNode<IntPtrT>{p_buffer.totalStringLength});
    tmp26 = Convert_uint32_uintptr_0(state_, TNode<UintPtrT>{tmp25});
    ca_.Branch(p_buffer.isOneByte, &block34, std::vector<compiler::Node*>{}, &block35, std::vector<compiler::Node*>{});
  }

  TNode<Union<SeqOneByteString, String>> tmp27;
  if (block34.is_used()) {
    ca_.Bind(&block34);
    tmp27 = AllocateSeqOneByteString_0(state_, TNode<Uint32T>{tmp26});
    ca_.Goto(&block36, tmp27);
  }

  TNode<Union<SeqTwoByteString, String>> tmp28;
  if (block35.is_used()) {
    ca_.Bind(&block35);
    tmp28 = AllocateSeqTwoByteString_0(state_, TNode<Uint32T>{tmp26});
    ca_.Goto(&block36, tmp28);
  }

  TNode<Union<SeqOneByteString, SeqTwoByteString, String>> phi_bb36_9;
  TNode<String> tmp29;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_9);
    tmp29 = ArrayBuiltinsAssembler(state_).CallJSArrayArrayJoinConcatToSequentialString(TNode<FixedArray>{p_buffer.head}, TNode<IntPtrT>{p_buffer.index}, TNode<String>{p_sep}, TNode<String>{phi_bb36_9});
    ca_.Goto(&block1, tmp29);
  }

  TNode<String> phi_bb1_8;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_8);
    ca_.Goto(&block38, phi_bb1_8);
  }

  TNode<String> phi_bb38_8;
    ca_.Bind(&block38, &phi_bb38_8);
  return TNode<String>{phi_bb38_8};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=357&c=1
TNode<String> FastArrayJoin_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, ElementsKind p_kind, TNode<JSArray> p_array, TNode<String> p_sep, TNode<Number> p_lengthNumber) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<UintPtrT, IntPtrT> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, UintPtrT> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, UintPtrT> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, UintPtrT> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, UintPtrT> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, UintPtrT> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, UintPtrT> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, String> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, UintPtrT> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, UintPtrT> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, String> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block70(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, BoolT> block71(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block67(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block72(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block73(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block74(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block85(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block92(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block93(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block86(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block96(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block97(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, IntPtrT> block98(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block109(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block110(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block118(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block119(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block127(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block128(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block87(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block75(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block66(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block131(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block143(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block144(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block151(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block150(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block153(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block152(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block163(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block170(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block171(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block164(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block174(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block175(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, IntPtrT> block176(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block187(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block188(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block196(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block197(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block205(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block206(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block165(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block148(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block212(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block211(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block225(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block226(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block147(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block132(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block238(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block245(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block246(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block239(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block249(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block250(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, IntPtrT> block251(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block262(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, FixedArray> block263(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block271(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block272(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block280(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block281(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block240(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject> block133(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block284(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block286(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block287(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block285(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block291(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block292(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, BoolT> block293(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block289(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block290(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block294(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block295(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block296(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block307(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block314(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block315(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block308(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block318(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block319(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block320(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray> block331(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray> block332(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block340(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block341(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block349(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block350(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block309(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block297(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block288(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block353(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<FixedArrayBase> tmp1;
  TNode<UintPtrT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<FixedArray> tmp5;
  TNode<FixedArray> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<BoolT> tmp9;
  TNode<PrimitiveHeapObject> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<Number> tmp13;
  TNode<UintPtrT> tmp14;
  TNode<BoolT> tmp15;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp1 = CodeStubAssembler(state_).LoadReference<FixedArrayBase>(CodeStubAssembler::Reference{p_array, tmp0});
    tmp2 = Convert_uintptr_Number_0(state_, TNode<Number>{p_lengthNumber});
    tmp3 = CodeStubAssembler(state_).LoadStringLengthAsWord(TNode<String>{p_sep});
    tmp4 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp5, tmp6, tmp7, tmp8, tmp9, tmp10) = NewBuffer_0(state_, TNode<UintPtrT>{tmp2}, TNode<String>{p_sep}).Flatten();
    tmp11 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp12 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp13 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{p_array, tmp12});
    tmp14 = Convert_uintptr_Number_0(state_, TNode<Number>{tmp13});
    tmp15 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp14}, TNode<UintPtrT>{tmp2});
    ca_.Branch(tmp15, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{tmp2, tmp11});
  }

  TNode<UintPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp16 = CodeStubAssembler(state_).UintPtrSub(TNode<UintPtrT>{tmp2}, TNode<UintPtrT>{tmp14});
    tmp17 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp16});
    ca_.Goto(&block3, tmp14, tmp17);
  }

  TNode<UintPtrT> phi_bb3_5;
  TNode<IntPtrT> phi_bb3_14;
  TNode<UintPtrT> tmp18;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_5, &phi_bb3_14);
    tmp18 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block6, tmp4, tmp6, tmp7, tmp8, tmp9, tmp10, tmp18);
  }

  TNode<IntPtrT> phi_bb6_7;
  TNode<FixedArray> phi_bb6_9;
  TNode<IntPtrT> phi_bb6_10;
  TNode<IntPtrT> phi_bb6_11;
  TNode<BoolT> phi_bb6_12;
  TNode<PrimitiveHeapObject> phi_bb6_13;
  TNode<UintPtrT> phi_bb6_15;
  TNode<BoolT> tmp19;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_7, &phi_bb6_9, &phi_bb6_10, &phi_bb6_11, &phi_bb6_12, &phi_bb6_13, &phi_bb6_15);
    tmp19 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{phi_bb6_15}, TNode<UintPtrT>{phi_bb3_5});
    ca_.Branch(tmp19, &block4, std::vector<compiler::Node*>{phi_bb6_7, phi_bb6_9, phi_bb6_10, phi_bb6_11, phi_bb6_12, phi_bb6_13, phi_bb6_15}, &block5, std::vector<compiler::Node*>{phi_bb6_7, phi_bb6_9, phi_bb6_10, phi_bb6_11, phi_bb6_12, phi_bb6_13, phi_bb6_15});
  }

  TNode<IntPtrT> phi_bb4_7;
  TNode<FixedArray> phi_bb4_9;
  TNode<IntPtrT> phi_bb4_10;
  TNode<IntPtrT> phi_bb4_11;
  TNode<BoolT> phi_bb4_12;
  TNode<PrimitiveHeapObject> phi_bb4_13;
  TNode<UintPtrT> phi_bb4_15;
  TNode<UintPtrT> tmp20;
  TNode<BoolT> tmp21;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_7, &phi_bb4_9, &phi_bb4_10, &phi_bb4_11, &phi_bb4_12, &phi_bb4_13, &phi_bb4_15);
    tmp20 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp21 = CodeStubAssembler(state_).UintPtrGreaterThan(TNode<UintPtrT>{phi_bb4_15}, TNode<UintPtrT>{tmp20});
    ca_.Branch(tmp21, &block7, std::vector<compiler::Node*>{phi_bb4_7, phi_bb4_9, phi_bb4_10, phi_bb4_11, phi_bb4_12, phi_bb4_13, phi_bb4_15}, &block8, std::vector<compiler::Node*>{phi_bb4_7, phi_bb4_9, phi_bb4_10, phi_bb4_11, phi_bb4_12, phi_bb4_13, phi_bb4_15});
  }

  TNode<IntPtrT> phi_bb7_7;
  TNode<FixedArray> phi_bb7_9;
  TNode<IntPtrT> phi_bb7_10;
  TNode<IntPtrT> phi_bb7_11;
  TNode<BoolT> phi_bb7_12;
  TNode<PrimitiveHeapObject> phi_bb7_13;
  TNode<UintPtrT> phi_bb7_15;
  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_7, &phi_bb7_9, &phi_bb7_10, &phi_bb7_11, &phi_bb7_12, &phi_bb7_13, &phi_bb7_15);
    tmp22 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp23 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb7_7}, TNode<IntPtrT>{tmp22});
    ca_.Goto(&block8, tmp23, phi_bb7_9, phi_bb7_10, phi_bb7_11, phi_bb7_12, phi_bb7_13, phi_bb7_15);
  }

  TNode<IntPtrT> phi_bb8_7;
  TNode<FixedArray> phi_bb8_9;
  TNode<IntPtrT> phi_bb8_10;
  TNode<IntPtrT> phi_bb8_11;
  TNode<BoolT> phi_bb8_12;
  TNode<PrimitiveHeapObject> phi_bb8_13;
  TNode<UintPtrT> phi_bb8_15;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_7, &phi_bb8_9, &phi_bb8_10, &phi_bb8_11, &phi_bb8_12, &phi_bb8_13, &phi_bb8_15);
    if (((CodeStubAssembler(state_).IsDoubleElementsKind(p_kind)))) {
      ca_.Goto(&block9, phi_bb8_9, phi_bb8_10, phi_bb8_11, phi_bb8_12, phi_bb8_13, phi_bb8_15);
    } else {
      ca_.Goto(&block10, phi_bb8_9, phi_bb8_10, phi_bb8_11, phi_bb8_12, phi_bb8_13, phi_bb8_15);
    }
  }

  TNode<FixedArray> phi_bb9_9;
  TNode<IntPtrT> phi_bb9_10;
  TNode<IntPtrT> phi_bb9_11;
  TNode<BoolT> phi_bb9_12;
  TNode<PrimitiveHeapObject> phi_bb9_13;
  TNode<UintPtrT> phi_bb9_15;
  TNode<FixedDoubleArray> tmp24;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_9, &phi_bb9_10, &phi_bb9_11, &phi_bb9_12, &phi_bb9_13, &phi_bb9_15);
    tmp24 = UnsafeCast_FixedDoubleArray_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp1});
    if (((CodeStubAssembler(state_).IsHoleyElementsKind(p_kind)))) {
      ca_.Goto(&block12, phi_bb9_9, phi_bb9_10, phi_bb9_11, phi_bb9_12, phi_bb9_13, phi_bb9_15);
    } else {
      ca_.Goto(&block13, phi_bb9_9, phi_bb9_10, phi_bb9_11, phi_bb9_12, phi_bb9_13, phi_bb9_15);
    }
  }

  TNode<FixedArray> phi_bb12_9;
  TNode<IntPtrT> phi_bb12_10;
  TNode<IntPtrT> phi_bb12_11;
  TNode<BoolT> phi_bb12_12;
  TNode<PrimitiveHeapObject> phi_bb12_13;
  TNode<UintPtrT> phi_bb12_15;
  TNode<Union<HeapObject, TaggedIndex>> tmp25;
  TNode<IntPtrT> tmp26;
  TNode<IntPtrT> tmp27;
  TNode<UintPtrT> tmp28;
  TNode<UintPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<UintPtrT> tmp31;
  TNode<UintPtrT> tmp32;
  TNode<BoolT> tmp33;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_9, &phi_bb12_10, &phi_bb12_11, &phi_bb12_12, &phi_bb12_13, &phi_bb12_15);
    std::tie(tmp25, tmp26, tmp27) = FieldSliceFixedDoubleArrayValues_0(state_, TNode<FixedDoubleArray>{tmp24}).Flatten();
    tmp28 = FromConstexpr_uintptr_constexpr_int31_0(state_, 1);
    tmp29 = CodeStubAssembler(state_).UintPtrAdd(TNode<UintPtrT>{phi_bb12_15}, TNode<UintPtrT>{tmp28});
    tmp30 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{phi_bb12_15});
    tmp31 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp30});
    tmp32 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp27});
    tmp33 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp31}, TNode<UintPtrT>{tmp32});
    ca_.Branch(tmp33, &block21, std::vector<compiler::Node*>{phi_bb12_9, phi_bb12_10, phi_bb12_11, phi_bb12_12, phi_bb12_13, phi_bb12_15, phi_bb12_15}, &block22, std::vector<compiler::Node*>{phi_bb12_9, phi_bb12_10, phi_bb12_11, phi_bb12_12, phi_bb12_13, phi_bb12_15, phi_bb12_15});
  }

  TNode<FixedArray> phi_bb21_9;
  TNode<IntPtrT> phi_bb21_10;
  TNode<IntPtrT> phi_bb21_11;
  TNode<BoolT> phi_bb21_12;
  TNode<PrimitiveHeapObject> phi_bb21_13;
  TNode<UintPtrT> phi_bb21_22;
  TNode<UintPtrT> phi_bb21_23;
  TNode<IntPtrT> tmp34;
  TNode<IntPtrT> tmp35;
  TNode<Union<HeapObject, TaggedIndex>> tmp36;
  TNode<IntPtrT> tmp37;
  TNode<BoolT> tmp38;
  TNode<Float64T> tmp39;
  if (block21.is_used()) {
    ca_.Bind(&block21, &phi_bb21_9, &phi_bb21_10, &phi_bb21_11, &phi_bb21_12, &phi_bb21_13, &phi_bb21_22, &phi_bb21_23);
    tmp34 = TimesSizeOf_float64_or_undefined_or_hole_0(state_, TNode<IntPtrT>{tmp30});
    tmp35 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp26}, TNode<IntPtrT>{tmp34});
    std::tie(tmp36, tmp37) = NewReference_float64_or_undefined_or_hole_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp25}, TNode<IntPtrT>{tmp35}).Flatten();
    std::tie(tmp38, tmp39) = LoadFloat64OrHole_0(state_, TorqueStructReference_float64_or_undefined_or_hole_0{TNode<Union<HeapObject, TaggedIndex>>{tmp36}, TNode<IntPtrT>{tmp37}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Branch(tmp38, &block26, std::vector<compiler::Node*>{phi_bb21_9, phi_bb21_10, phi_bb21_11, phi_bb21_12, phi_bb21_13, phi_bb21_22, phi_bb21_23}, &block27, std::vector<compiler::Node*>{phi_bb21_9, phi_bb21_10, phi_bb21_11, phi_bb21_12, phi_bb21_13, phi_bb21_22, phi_bb21_23});
  }

  TNode<FixedArray> phi_bb22_9;
  TNode<IntPtrT> phi_bb22_10;
  TNode<IntPtrT> phi_bb22_11;
  TNode<BoolT> phi_bb22_12;
  TNode<PrimitiveHeapObject> phi_bb22_13;
  TNode<UintPtrT> phi_bb22_22;
  TNode<UintPtrT> phi_bb22_23;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_9, &phi_bb22_10, &phi_bb22_11, &phi_bb22_12, &phi_bb22_13, &phi_bb22_22, &phi_bb22_23);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb26_9;
  TNode<IntPtrT> phi_bb26_10;
  TNode<IntPtrT> phi_bb26_11;
  TNode<BoolT> phi_bb26_12;
  TNode<PrimitiveHeapObject> phi_bb26_13;
  TNode<UintPtrT> phi_bb26_22;
  TNode<UintPtrT> phi_bb26_23;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_9, &phi_bb26_10, &phi_bb26_11, &phi_bb26_12, &phi_bb26_13, &phi_bb26_22, &phi_bb26_23);
    ca_.Goto(&block6, phi_bb8_7, phi_bb26_9, phi_bb26_10, phi_bb26_11, phi_bb26_12, phi_bb26_13, tmp29);
  }

  TNode<FixedArray> phi_bb27_9;
  TNode<IntPtrT> phi_bb27_10;
  TNode<IntPtrT> phi_bb27_11;
  TNode<BoolT> phi_bb27_12;
  TNode<PrimitiveHeapObject> phi_bb27_13;
  TNode<UintPtrT> phi_bb27_22;
  TNode<UintPtrT> phi_bb27_23;
  TNode<String> tmp40;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_9, &phi_bb27_10, &phi_bb27_11, &phi_bb27_12, &phi_bb27_13, &phi_bb27_22, &phi_bb27_23);
    tmp40 = CodeStubAssembler(state_).Float64ToString(TNode<Float64T>{tmp39});
    ca_.Goto(&block14, phi_bb27_9, phi_bb27_10, phi_bb27_11, phi_bb27_12, phi_bb27_13, tmp29, tmp40);
  }

  TNode<FixedArray> phi_bb13_9;
  TNode<IntPtrT> phi_bb13_10;
  TNode<IntPtrT> phi_bb13_11;
  TNode<BoolT> phi_bb13_12;
  TNode<PrimitiveHeapObject> phi_bb13_13;
  TNode<UintPtrT> phi_bb13_15;
  TNode<Union<HeapObject, TaggedIndex>> tmp41;
  TNode<IntPtrT> tmp42;
  TNode<IntPtrT> tmp43;
  TNode<UintPtrT> tmp44;
  TNode<UintPtrT> tmp45;
  TNode<IntPtrT> tmp46;
  TNode<UintPtrT> tmp47;
  TNode<UintPtrT> tmp48;
  TNode<BoolT> tmp49;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_9, &phi_bb13_10, &phi_bb13_11, &phi_bb13_12, &phi_bb13_13, &phi_bb13_15);
    std::tie(tmp41, tmp42, tmp43) = FieldSliceFixedDoubleArrayValues_0(state_, TNode<FixedDoubleArray>{tmp24}).Flatten();
    tmp44 = FromConstexpr_uintptr_constexpr_int31_0(state_, 1);
    tmp45 = CodeStubAssembler(state_).UintPtrAdd(TNode<UintPtrT>{phi_bb13_15}, TNode<UintPtrT>{tmp44});
    tmp46 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{phi_bb13_15});
    tmp47 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp46});
    tmp48 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp43});
    tmp49 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp47}, TNode<UintPtrT>{tmp48});
    ca_.Branch(tmp49, &block32, std::vector<compiler::Node*>{phi_bb13_9, phi_bb13_10, phi_bb13_11, phi_bb13_12, phi_bb13_13, phi_bb13_15, phi_bb13_15}, &block33, std::vector<compiler::Node*>{phi_bb13_9, phi_bb13_10, phi_bb13_11, phi_bb13_12, phi_bb13_13, phi_bb13_15, phi_bb13_15});
  }

  TNode<FixedArray> phi_bb32_9;
  TNode<IntPtrT> phi_bb32_10;
  TNode<IntPtrT> phi_bb32_11;
  TNode<BoolT> phi_bb32_12;
  TNode<PrimitiveHeapObject> phi_bb32_13;
  TNode<UintPtrT> phi_bb32_22;
  TNode<UintPtrT> phi_bb32_23;
  TNode<IntPtrT> tmp50;
  TNode<IntPtrT> tmp51;
  TNode<Union<HeapObject, TaggedIndex>> tmp52;
  TNode<IntPtrT> tmp53;
  TNode<BoolT> tmp54;
  TNode<Float64T> tmp55;
  TNode<String> tmp56;
  if (block32.is_used()) {
    ca_.Bind(&block32, &phi_bb32_9, &phi_bb32_10, &phi_bb32_11, &phi_bb32_12, &phi_bb32_13, &phi_bb32_22, &phi_bb32_23);
    tmp50 = TimesSizeOf_float64_or_undefined_or_hole_0(state_, TNode<IntPtrT>{tmp46});
    tmp51 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp42}, TNode<IntPtrT>{tmp50});
    std::tie(tmp52, tmp53) = NewReference_float64_or_undefined_or_hole_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp41}, TNode<IntPtrT>{tmp51}).Flatten();
    std::tie(tmp54, tmp55) = LoadFloat64OrHole_0(state_, TorqueStructReference_float64_or_undefined_or_hole_0{TNode<Union<HeapObject, TaggedIndex>>{tmp52}, TNode<IntPtrT>{tmp53}, TorqueStructUnsafe_0{}}).Flatten();
    tmp56 = CodeStubAssembler(state_).Float64ToString(TNode<Float64T>{tmp55});
    ca_.Goto(&block14, phi_bb32_9, phi_bb32_10, phi_bb32_11, phi_bb32_12, phi_bb32_13, tmp45, tmp56);
  }

  TNode<FixedArray> phi_bb33_9;
  TNode<IntPtrT> phi_bb33_10;
  TNode<IntPtrT> phi_bb33_11;
  TNode<BoolT> phi_bb33_12;
  TNode<PrimitiveHeapObject> phi_bb33_13;
  TNode<UintPtrT> phi_bb33_22;
  TNode<UintPtrT> phi_bb33_23;
  if (block33.is_used()) {
    ca_.Bind(&block33, &phi_bb33_9, &phi_bb33_10, &phi_bb33_11, &phi_bb33_12, &phi_bb33_13, &phi_bb33_22, &phi_bb33_23);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb14_9;
  TNode<IntPtrT> phi_bb14_10;
  TNode<IntPtrT> phi_bb14_11;
  TNode<BoolT> phi_bb14_12;
  TNode<PrimitiveHeapObject> phi_bb14_13;
  TNode<UintPtrT> phi_bb14_15;
  TNode<String> phi_bb14_16;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_9, &phi_bb14_10, &phi_bb14_11, &phi_bb14_12, &phi_bb14_13, &phi_bb14_15, &phi_bb14_16);
    ca_.Goto(&block11, phi_bb14_9, phi_bb14_10, phi_bb14_11, phi_bb14_12, phi_bb14_13, phi_bb14_15, phi_bb14_16);
  }

  TNode<FixedArray> phi_bb10_9;
  TNode<IntPtrT> phi_bb10_10;
  TNode<IntPtrT> phi_bb10_11;
  TNode<BoolT> phi_bb10_12;
  TNode<PrimitiveHeapObject> phi_bb10_13;
  TNode<UintPtrT> phi_bb10_15;
  TNode<FixedArray> tmp57;
  TNode<Union<HeapObject, TaggedIndex>> tmp58;
  TNode<IntPtrT> tmp59;
  TNode<IntPtrT> tmp60;
  TNode<UintPtrT> tmp61;
  TNode<UintPtrT> tmp62;
  TNode<IntPtrT> tmp63;
  TNode<UintPtrT> tmp64;
  TNode<UintPtrT> tmp65;
  TNode<BoolT> tmp66;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_9, &phi_bb10_10, &phi_bb10_11, &phi_bb10_12, &phi_bb10_13, &phi_bb10_15);
    tmp57 = UnsafeCast_FixedArray_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp1});
    std::tie(tmp58, tmp59, tmp60) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp57}).Flatten();
    tmp61 = FromConstexpr_uintptr_constexpr_int31_0(state_, 1);
    tmp62 = CodeStubAssembler(state_).UintPtrAdd(TNode<UintPtrT>{phi_bb10_15}, TNode<UintPtrT>{tmp61});
    tmp63 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{phi_bb10_15});
    tmp64 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp63});
    tmp65 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp60});
    tmp66 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp64}, TNode<UintPtrT>{tmp65});
    ca_.Branch(tmp66, &block49, std::vector<compiler::Node*>{phi_bb10_9, phi_bb10_10, phi_bb10_11, phi_bb10_12, phi_bb10_13, phi_bb10_15, phi_bb10_15}, &block50, std::vector<compiler::Node*>{phi_bb10_9, phi_bb10_10, phi_bb10_11, phi_bb10_12, phi_bb10_13, phi_bb10_15, phi_bb10_15});
  }

  TNode<FixedArray> phi_bb49_9;
  TNode<IntPtrT> phi_bb49_10;
  TNode<IntPtrT> phi_bb49_11;
  TNode<BoolT> phi_bb49_12;
  TNode<PrimitiveHeapObject> phi_bb49_13;
  TNode<UintPtrT> phi_bb49_22;
  TNode<UintPtrT> phi_bb49_23;
  TNode<IntPtrT> tmp67;
  TNode<IntPtrT> tmp68;
  TNode<Union<HeapObject, TaggedIndex>> tmp69;
  TNode<IntPtrT> tmp70;
  TNode<Object> tmp71;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_9, &phi_bb49_10, &phi_bb49_11, &phi_bb49_12, &phi_bb49_13, &phi_bb49_22, &phi_bb49_23);
    tmp67 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp63});
    tmp68 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp59}, TNode<IntPtrT>{tmp67});
    std::tie(tmp69, tmp70) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp58}, TNode<IntPtrT>{tmp68}).Flatten();
    tmp71 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp69, tmp70});
    if (((CodeStubAssembler(state_).IsHoleyElementsKind(p_kind)))) {
      ca_.Goto(&block53, phi_bb49_9, phi_bb49_10, phi_bb49_11, phi_bb49_12, phi_bb49_13);
    } else {
      ca_.Goto(&block54, phi_bb49_9, phi_bb49_10, phi_bb49_11, phi_bb49_12, phi_bb49_13);
    }
  }

  TNode<FixedArray> phi_bb50_9;
  TNode<IntPtrT> phi_bb50_10;
  TNode<IntPtrT> phi_bb50_11;
  TNode<BoolT> phi_bb50_12;
  TNode<PrimitiveHeapObject> phi_bb50_13;
  TNode<UintPtrT> phi_bb50_22;
  TNode<UintPtrT> phi_bb50_23;
  if (block50.is_used()) {
    ca_.Bind(&block50, &phi_bb50_9, &phi_bb50_10, &phi_bb50_11, &phi_bb50_12, &phi_bb50_13, &phi_bb50_22, &phi_bb50_23);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb53_9;
  TNode<IntPtrT> phi_bb53_10;
  TNode<IntPtrT> phi_bb53_11;
  TNode<BoolT> phi_bb53_12;
  TNode<PrimitiveHeapObject> phi_bb53_13;
  TNode<TheHole> tmp72;
  TNode<BoolT> tmp73;
  if (block53.is_used()) {
    ca_.Bind(&block53, &phi_bb53_9, &phi_bb53_10, &phi_bb53_11, &phi_bb53_12, &phi_bb53_13);
    tmp72 = TheHole_0(state_);
    tmp73 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp71}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp72});
    ca_.Branch(tmp73, &block56, std::vector<compiler::Node*>{phi_bb53_9, phi_bb53_10, phi_bb53_11, phi_bb53_12, phi_bb53_13}, &block57, std::vector<compiler::Node*>{phi_bb53_9, phi_bb53_10, phi_bb53_11, phi_bb53_12, phi_bb53_13});
  }

  TNode<FixedArray> phi_bb56_9;
  TNode<IntPtrT> phi_bb56_10;
  TNode<IntPtrT> phi_bb56_11;
  TNode<BoolT> phi_bb56_12;
  TNode<PrimitiveHeapObject> phi_bb56_13;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_9, &phi_bb56_10, &phi_bb56_11, &phi_bb56_12, &phi_bb56_13);
    ca_.Goto(&block6, phi_bb8_7, phi_bb56_9, phi_bb56_10, phi_bb56_11, phi_bb56_12, phi_bb56_13, tmp62);
  }

  TNode<FixedArray> phi_bb57_9;
  TNode<IntPtrT> phi_bb57_10;
  TNode<IntPtrT> phi_bb57_11;
  TNode<BoolT> phi_bb57_12;
  TNode<PrimitiveHeapObject> phi_bb57_13;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_9, &phi_bb57_10, &phi_bb57_11, &phi_bb57_12, &phi_bb57_13);
    ca_.Goto(&block55, phi_bb57_9, phi_bb57_10, phi_bb57_11, phi_bb57_12, phi_bb57_13);
  }

  TNode<FixedArray> phi_bb54_9;
  TNode<IntPtrT> phi_bb54_10;
  TNode<IntPtrT> phi_bb54_11;
  TNode<BoolT> phi_bb54_12;
  TNode<PrimitiveHeapObject> phi_bb54_13;
  if (block54.is_used()) {
    ca_.Bind(&block54, &phi_bb54_9, &phi_bb54_10, &phi_bb54_11, &phi_bb54_12, &phi_bb54_13);
    ca_.Goto(&block55, phi_bb54_9, phi_bb54_10, phi_bb54_11, phi_bb54_12, phi_bb54_13);
  }

  TNode<FixedArray> phi_bb55_9;
  TNode<IntPtrT> phi_bb55_10;
  TNode<IntPtrT> phi_bb55_11;
  TNode<BoolT> phi_bb55_12;
  TNode<PrimitiveHeapObject> phi_bb55_13;
  TNode<Smi> tmp74;
  TNode<String> tmp75;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_9, &phi_bb55_10, &phi_bb55_11, &phi_bb55_12, &phi_bb55_13);
    tmp74 = UnsafeCast_Smi_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp71});
    tmp75 = CodeStubAssembler(state_).SmiToString(TNode<Smi>{tmp74});
    ca_.Goto(&block11, phi_bb55_9, phi_bb55_10, phi_bb55_11, phi_bb55_12, phi_bb55_13, tmp62, tmp75);
  }

  TNode<FixedArray> phi_bb11_9;
  TNode<IntPtrT> phi_bb11_10;
  TNode<IntPtrT> phi_bb11_11;
  TNode<BoolT> phi_bb11_12;
  TNode<PrimitiveHeapObject> phi_bb11_13;
  TNode<UintPtrT> phi_bb11_15;
  TNode<String> phi_bb11_16;
  TNode<IntPtrT> tmp76;
  TNode<BoolT> tmp77;
  TNode<IntPtrT> tmp78;
  TNode<BoolT> tmp79;
  TNode<BoolT> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<BoolT> tmp82;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_9, &phi_bb11_10, &phi_bb11_11, &phi_bb11_12, &phi_bb11_13, &phi_bb11_15, &phi_bb11_16);
    tmp76 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp77 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb11_10}, TNode<IntPtrT>{tmp76});
    tmp78 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp79 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb8_7}, TNode<IntPtrT>{tmp78});
    tmp80 = CodeStubAssembler(state_).Word32Or(TNode<BoolT>{tmp77}, TNode<BoolT>{tmp79});
    tmp81 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp82 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb8_7}, TNode<IntPtrT>{tmp81});
    ca_.Branch(tmp82, &block69, std::vector<compiler::Node*>{phi_bb11_9, phi_bb11_10, phi_bb11_11, phi_bb11_12, phi_bb11_13}, &block70, std::vector<compiler::Node*>{phi_bb11_9, phi_bb11_10, phi_bb11_11, phi_bb11_12, phi_bb11_13});
  }

  TNode<FixedArray> phi_bb69_9;
  TNode<IntPtrT> phi_bb69_10;
  TNode<IntPtrT> phi_bb69_11;
  TNode<BoolT> phi_bb69_12;
  TNode<PrimitiveHeapObject> phi_bb69_13;
  TNode<BoolT> tmp83;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_9, &phi_bb69_10, &phi_bb69_11, &phi_bb69_12, &phi_bb69_13);
    tmp83 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block71, phi_bb69_9, phi_bb69_10, phi_bb69_11, phi_bb69_12, phi_bb69_13, tmp83);
  }

  TNode<FixedArray> phi_bb70_9;
  TNode<IntPtrT> phi_bb70_10;
  TNode<IntPtrT> phi_bb70_11;
  TNode<BoolT> phi_bb70_12;
  TNode<PrimitiveHeapObject> phi_bb70_13;
  TNode<IntPtrT> tmp84;
  TNode<BoolT> tmp85;
  if (block70.is_used()) {
    ca_.Bind(&block70, &phi_bb70_9, &phi_bb70_10, &phi_bb70_11, &phi_bb70_12, &phi_bb70_13);
    tmp84 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp85 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp84});
    ca_.Goto(&block71, phi_bb70_9, phi_bb70_10, phi_bb70_11, phi_bb70_12, phi_bb70_13, tmp85);
  }

  TNode<FixedArray> phi_bb71_9;
  TNode<IntPtrT> phi_bb71_10;
  TNode<IntPtrT> phi_bb71_11;
  TNode<BoolT> phi_bb71_12;
  TNode<PrimitiveHeapObject> phi_bb71_13;
  TNode<BoolT> phi_bb71_35;
  if (block71.is_used()) {
    ca_.Bind(&block71, &phi_bb71_9, &phi_bb71_10, &phi_bb71_11, &phi_bb71_12, &phi_bb71_13, &phi_bb71_35);
    ca_.Branch(phi_bb71_35, &block67, std::vector<compiler::Node*>{phi_bb71_9, phi_bb71_10, phi_bb71_11, phi_bb71_12, phi_bb71_13}, &block68, std::vector<compiler::Node*>{phi_bb71_9, phi_bb71_10, phi_bb71_11, phi_bb71_12, phi_bb71_13});
  }

  TNode<FixedArray> phi_bb67_9;
  TNode<IntPtrT> phi_bb67_10;
  TNode<IntPtrT> phi_bb67_11;
  TNode<BoolT> phi_bb67_12;
  TNode<PrimitiveHeapObject> phi_bb67_13;
  if (block67.is_used()) {
    ca_.Bind(&block67, &phi_bb67_9, &phi_bb67_10, &phi_bb67_11, &phi_bb67_12, &phi_bb67_13);
    ca_.Goto(&block66, phi_bb67_9, phi_bb67_10, phi_bb67_11, phi_bb67_12, phi_bb67_13);
  }

  TNode<FixedArray> phi_bb68_9;
  TNode<IntPtrT> phi_bb68_10;
  TNode<IntPtrT> phi_bb68_11;
  TNode<BoolT> phi_bb68_12;
  TNode<PrimitiveHeapObject> phi_bb68_13;
  TNode<IntPtrT> tmp86;
  TNode<IntPtrT> tmp87;
  TNode<BoolT> tmp88;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_9, &phi_bb68_10, &phi_bb68_11, &phi_bb68_12, &phi_bb68_13);
    tmp86 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{phi_bb8_7});
    tmp87 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp86}, TNode<IntPtrT>{tmp3});
    tmp88 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{tmp87}, TNode<IntPtrT>{phi_bb8_7});
    ca_.Branch(tmp88, &block72, std::vector<compiler::Node*>{phi_bb68_9, phi_bb68_10, phi_bb68_11, phi_bb68_12, phi_bb68_13}, &block73, std::vector<compiler::Node*>{phi_bb68_9, phi_bb68_10, phi_bb68_11, phi_bb68_12, phi_bb68_13});
  }

  TNode<FixedArray> phi_bb72_9;
  TNode<IntPtrT> phi_bb72_10;
  TNode<IntPtrT> phi_bb72_11;
  TNode<BoolT> phi_bb72_12;
  TNode<PrimitiveHeapObject> phi_bb72_13;
  if (block72.is_used()) {
    ca_.Bind(&block72, &phi_bb72_9, &phi_bb72_10, &phi_bb72_11, &phi_bb72_12, &phi_bb72_13);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowInvalidStringLength, p_context);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb73_9;
  TNode<IntPtrT> phi_bb73_10;
  TNode<IntPtrT> phi_bb73_11;
  TNode<BoolT> phi_bb73_12;
  TNode<PrimitiveHeapObject> phi_bb73_13;
  TNode<IntPtrT> tmp89;
  if (block73.is_used()) {
    ca_.Bind(&block73, &phi_bb73_9, &phi_bb73_10, &phi_bb73_11, &phi_bb73_12, &phi_bb73_13);
    tmp89 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb73_11}, TNode<IntPtrT>{tmp86});
    ca_.Branch(tmp80, &block74, std::vector<compiler::Node*>{phi_bb73_9, phi_bb73_10, phi_bb73_12, phi_bb73_13}, &block75, std::vector<compiler::Node*>{phi_bb73_9, phi_bb73_10, phi_bb73_12, phi_bb73_13});
  }

  TNode<FixedArray> phi_bb74_9;
  TNode<IntPtrT> phi_bb74_10;
  TNode<BoolT> phi_bb74_12;
  TNode<PrimitiveHeapObject> phi_bb74_13;
  TNode<Smi> tmp90;
  TNode<IntPtrT> tmp91;
  TNode<BoolT> tmp92;
  if (block74.is_used()) {
    ca_.Bind(&block74, &phi_bb74_9, &phi_bb74_10, &phi_bb74_12, &phi_bb74_13);
    tmp90 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{phi_bb8_7});
    tmp91 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb74_9});
    tmp92 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb74_10}, TNode<IntPtrT>{tmp91});
    ca_.Branch(tmp92, &block85, std::vector<compiler::Node*>{phi_bb74_9, phi_bb74_10, phi_bb74_12, phi_bb74_13}, &block86, std::vector<compiler::Node*>{phi_bb74_9, phi_bb74_10, phi_bb74_12, phi_bb74_13});
  }

  TNode<FixedArray> phi_bb85_9;
  TNode<IntPtrT> phi_bb85_10;
  TNode<BoolT> phi_bb85_12;
  TNode<PrimitiveHeapObject> phi_bb85_13;
  TNode<Union<HeapObject, TaggedIndex>> tmp93;
  TNode<IntPtrT> tmp94;
  TNode<IntPtrT> tmp95;
  TNode<IntPtrT> tmp96;
  TNode<IntPtrT> tmp97;
  TNode<UintPtrT> tmp98;
  TNode<UintPtrT> tmp99;
  TNode<BoolT> tmp100;
  if (block85.is_used()) {
    ca_.Bind(&block85, &phi_bb85_9, &phi_bb85_10, &phi_bb85_12, &phi_bb85_13);
    std::tie(tmp93, tmp94, tmp95) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb85_9}).Flatten();
    tmp96 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp97 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb85_10}, TNode<IntPtrT>{tmp96});
    tmp98 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb85_10});
    tmp99 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp95});
    tmp100 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp98}, TNode<UintPtrT>{tmp99});
    ca_.Branch(tmp100, &block92, std::vector<compiler::Node*>{phi_bb85_9, phi_bb85_12, phi_bb85_13, phi_bb85_9, phi_bb85_10, phi_bb85_10, phi_bb85_10, phi_bb85_10}, &block93, std::vector<compiler::Node*>{phi_bb85_9, phi_bb85_12, phi_bb85_13, phi_bb85_9, phi_bb85_10, phi_bb85_10, phi_bb85_10, phi_bb85_10});
  }

  TNode<FixedArray> phi_bb92_9;
  TNode<BoolT> phi_bb92_12;
  TNode<PrimitiveHeapObject> phi_bb92_13;
  TNode<FixedArray> phi_bb92_39;
  TNode<IntPtrT> phi_bb92_43;
  TNode<IntPtrT> phi_bb92_44;
  TNode<IntPtrT> phi_bb92_48;
  TNode<IntPtrT> phi_bb92_49;
  TNode<IntPtrT> tmp101;
  TNode<IntPtrT> tmp102;
  TNode<Union<HeapObject, TaggedIndex>> tmp103;
  TNode<IntPtrT> tmp104;
  if (block92.is_used()) {
    ca_.Bind(&block92, &phi_bb92_9, &phi_bb92_12, &phi_bb92_13, &phi_bb92_39, &phi_bb92_43, &phi_bb92_44, &phi_bb92_48, &phi_bb92_49);
    tmp101 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb92_49});
    tmp102 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp94}, TNode<IntPtrT>{tmp101});
    std::tie(tmp103, tmp104) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp93}, TNode<IntPtrT>{tmp102}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp103, tmp104}, tmp90);
    ca_.Goto(&block87, phi_bb92_9, tmp97, phi_bb92_12, phi_bb92_13);
  }

  TNode<FixedArray> phi_bb93_9;
  TNode<BoolT> phi_bb93_12;
  TNode<PrimitiveHeapObject> phi_bb93_13;
  TNode<FixedArray> phi_bb93_39;
  TNode<IntPtrT> phi_bb93_43;
  TNode<IntPtrT> phi_bb93_44;
  TNode<IntPtrT> phi_bb93_48;
  TNode<IntPtrT> phi_bb93_49;
  if (block93.is_used()) {
    ca_.Bind(&block93, &phi_bb93_9, &phi_bb93_12, &phi_bb93_13, &phi_bb93_39, &phi_bb93_43, &phi_bb93_44, &phi_bb93_48, &phi_bb93_49);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb86_9;
  TNode<IntPtrT> phi_bb86_10;
  TNode<BoolT> phi_bb86_12;
  TNode<PrimitiveHeapObject> phi_bb86_13;
  TNode<IntPtrT> tmp105;
  TNode<IntPtrT> tmp106;
  TNode<BoolT> tmp107;
  if (block86.is_used()) {
    ca_.Bind(&block86, &phi_bb86_9, &phi_bb86_10, &phi_bb86_12, &phi_bb86_13);
    tmp105 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp91});
    tmp106 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp107 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp105}, TNode<IntPtrT>{tmp106});
    ca_.Branch(tmp107, &block96, std::vector<compiler::Node*>{phi_bb86_9, phi_bb86_10, phi_bb86_12, phi_bb86_13}, &block97, std::vector<compiler::Node*>{phi_bb86_9, phi_bb86_10, phi_bb86_12, phi_bb86_13});
  }

  TNode<FixedArray> phi_bb96_9;
  TNode<IntPtrT> phi_bb96_10;
  TNode<BoolT> phi_bb96_12;
  TNode<PrimitiveHeapObject> phi_bb96_13;
  TNode<IntPtrT> tmp108;
  if (block96.is_used()) {
    ca_.Bind(&block96, &phi_bb96_9, &phi_bb96_10, &phi_bb96_12, &phi_bb96_13);
    tmp108 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block98, phi_bb96_9, phi_bb96_10, phi_bb96_12, phi_bb96_13, tmp108);
  }

  TNode<FixedArray> phi_bb97_9;
  TNode<IntPtrT> phi_bb97_10;
  TNode<BoolT> phi_bb97_12;
  TNode<PrimitiveHeapObject> phi_bb97_13;
  if (block97.is_used()) {
    ca_.Bind(&block97, &phi_bb97_9, &phi_bb97_10, &phi_bb97_12, &phi_bb97_13);
    ca_.Goto(&block98, phi_bb97_9, phi_bb97_10, phi_bb97_12, phi_bb97_13, tmp105);
  }

  TNode<FixedArray> phi_bb98_9;
  TNode<IntPtrT> phi_bb98_10;
  TNode<BoolT> phi_bb98_12;
  TNode<PrimitiveHeapObject> phi_bb98_13;
  TNode<IntPtrT> phi_bb98_40;
  TNode<FixedArray> tmp109;
  TNode<Union<HeapObject, TaggedIndex>> tmp110;
  TNode<IntPtrT> tmp111;
  TNode<IntPtrT> tmp112;
  TNode<UintPtrT> tmp113;
  TNode<IntPtrT> tmp114;
  TNode<UintPtrT> tmp115;
  TNode<UintPtrT> tmp116;
  TNode<BoolT> tmp117;
  if (block98.is_used()) {
    ca_.Bind(&block98, &phi_bb98_9, &phi_bb98_10, &phi_bb98_12, &phi_bb98_13, &phi_bb98_40);
    tmp109 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb98_40});
    std::tie(tmp110, tmp111, tmp112) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb98_9}).Flatten();
    tmp113 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp114 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp113});
    tmp115 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp114});
    tmp116 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp112});
    tmp117 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp115}, TNode<UintPtrT>{tmp116});
    ca_.Branch(tmp117, &block109, std::vector<compiler::Node*>{phi_bb98_9, phi_bb98_10, phi_bb98_12, phi_bb98_13, phi_bb98_9}, &block110, std::vector<compiler::Node*>{phi_bb98_9, phi_bb98_10, phi_bb98_12, phi_bb98_13, phi_bb98_9});
  }

  TNode<FixedArray> phi_bb109_9;
  TNode<IntPtrT> phi_bb109_10;
  TNode<BoolT> phi_bb109_12;
  TNode<PrimitiveHeapObject> phi_bb109_13;
  TNode<FixedArray> phi_bb109_42;
  TNode<IntPtrT> tmp118;
  TNode<IntPtrT> tmp119;
  TNode<Union<HeapObject, TaggedIndex>> tmp120;
  TNode<IntPtrT> tmp121;
  TNode<Union<HeapObject, TaggedIndex>> tmp122;
  TNode<IntPtrT> tmp123;
  TNode<IntPtrT> tmp124;
  TNode<UintPtrT> tmp125;
  TNode<IntPtrT> tmp126;
  TNode<UintPtrT> tmp127;
  TNode<UintPtrT> tmp128;
  TNode<BoolT> tmp129;
  if (block109.is_used()) {
    ca_.Bind(&block109, &phi_bb109_9, &phi_bb109_10, &phi_bb109_12, &phi_bb109_13, &phi_bb109_42);
    tmp118 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp114});
    tmp119 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp111}, TNode<IntPtrT>{tmp118});
    std::tie(tmp120, tmp121) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp110}, TNode<IntPtrT>{tmp119}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp120, tmp121}, tmp109);
    std::tie(tmp122, tmp123, tmp124) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp109}).Flatten();
    tmp125 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp126 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp125});
    tmp127 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp126});
    tmp128 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp124});
    tmp129 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp127}, TNode<UintPtrT>{tmp128});
    ca_.Branch(tmp129, &block118, std::vector<compiler::Node*>{phi_bb109_9, phi_bb109_10, phi_bb109_12, phi_bb109_13}, &block119, std::vector<compiler::Node*>{phi_bb109_9, phi_bb109_10, phi_bb109_12, phi_bb109_13});
  }

  TNode<FixedArray> phi_bb110_9;
  TNode<IntPtrT> phi_bb110_10;
  TNode<BoolT> phi_bb110_12;
  TNode<PrimitiveHeapObject> phi_bb110_13;
  TNode<FixedArray> phi_bb110_42;
  if (block110.is_used()) {
    ca_.Bind(&block110, &phi_bb110_9, &phi_bb110_10, &phi_bb110_12, &phi_bb110_13, &phi_bb110_42);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb118_9;
  TNode<IntPtrT> phi_bb118_10;
  TNode<BoolT> phi_bb118_12;
  TNode<PrimitiveHeapObject> phi_bb118_13;
  TNode<IntPtrT> tmp130;
  TNode<IntPtrT> tmp131;
  TNode<Union<HeapObject, TaggedIndex>> tmp132;
  TNode<IntPtrT> tmp133;
  TNode<Undefined> tmp134;
  TNode<Union<HeapObject, TaggedIndex>> tmp135;
  TNode<IntPtrT> tmp136;
  TNode<IntPtrT> tmp137;
  TNode<UintPtrT> tmp138;
  TNode<IntPtrT> tmp139;
  TNode<UintPtrT> tmp140;
  TNode<UintPtrT> tmp141;
  TNode<BoolT> tmp142;
  if (block118.is_used()) {
    ca_.Bind(&block118, &phi_bb118_9, &phi_bb118_10, &phi_bb118_12, &phi_bb118_13);
    tmp130 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp126});
    tmp131 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp123}, TNode<IntPtrT>{tmp130});
    std::tie(tmp132, tmp133) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp122}, TNode<IntPtrT>{tmp131}).Flatten();
    tmp134 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp132, tmp133}, tmp134);
    std::tie(tmp135, tmp136, tmp137) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp109}).Flatten();
    tmp138 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp139 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp138});
    tmp140 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp139});
    tmp141 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp137});
    tmp142 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp140}, TNode<UintPtrT>{tmp141});
    ca_.Branch(tmp142, &block127, std::vector<compiler::Node*>{phi_bb118_9, phi_bb118_10, phi_bb118_12, phi_bb118_13}, &block128, std::vector<compiler::Node*>{phi_bb118_9, phi_bb118_10, phi_bb118_12, phi_bb118_13});
  }

  TNode<FixedArray> phi_bb119_9;
  TNode<IntPtrT> phi_bb119_10;
  TNode<BoolT> phi_bb119_12;
  TNode<PrimitiveHeapObject> phi_bb119_13;
  if (block119.is_used()) {
    ca_.Bind(&block119, &phi_bb119_9, &phi_bb119_10, &phi_bb119_12, &phi_bb119_13);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb127_9;
  TNode<IntPtrT> phi_bb127_10;
  TNode<BoolT> phi_bb127_12;
  TNode<PrimitiveHeapObject> phi_bb127_13;
  TNode<IntPtrT> tmp143;
  TNode<IntPtrT> tmp144;
  TNode<Union<HeapObject, TaggedIndex>> tmp145;
  TNode<IntPtrT> tmp146;
  TNode<IntPtrT> tmp147;
  if (block127.is_used()) {
    ca_.Bind(&block127, &phi_bb127_9, &phi_bb127_10, &phi_bb127_12, &phi_bb127_13);
    tmp143 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp139});
    tmp144 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp136}, TNode<IntPtrT>{tmp143});
    std::tie(tmp145, tmp146) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp135}, TNode<IntPtrT>{tmp144}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp145, tmp146}, tmp90);
    tmp147 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block87, tmp109, tmp147, phi_bb127_12, phi_bb127_13);
  }

  TNode<FixedArray> phi_bb128_9;
  TNode<IntPtrT> phi_bb128_10;
  TNode<BoolT> phi_bb128_12;
  TNode<PrimitiveHeapObject> phi_bb128_13;
  if (block128.is_used()) {
    ca_.Bind(&block128, &phi_bb128_9, &phi_bb128_10, &phi_bb128_12, &phi_bb128_13);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb87_9;
  TNode<IntPtrT> phi_bb87_10;
  TNode<BoolT> phi_bb87_12;
  TNode<PrimitiveHeapObject> phi_bb87_13;
  TNode<Null> tmp148;
  if (block87.is_used()) {
    ca_.Bind(&block87, &phi_bb87_9, &phi_bb87_10, &phi_bb87_12, &phi_bb87_13);
    tmp148 = Null_0(state_);
    ca_.Goto(&block75, phi_bb87_9, phi_bb87_10, phi_bb87_12, tmp148);
  }

  TNode<FixedArray> phi_bb75_9;
  TNode<IntPtrT> phi_bb75_10;
  TNode<BoolT> phi_bb75_12;
  TNode<PrimitiveHeapObject> phi_bb75_13;
  if (block75.is_used()) {
    ca_.Bind(&block75, &phi_bb75_9, &phi_bb75_10, &phi_bb75_12, &phi_bb75_13);
    ca_.Goto(&block66, phi_bb75_9, phi_bb75_10, tmp89, phi_bb75_12, phi_bb75_13);
  }

  TNode<FixedArray> phi_bb66_9;
  TNode<IntPtrT> phi_bb66_10;
  TNode<IntPtrT> phi_bb66_11;
  TNode<BoolT> phi_bb66_12;
  TNode<PrimitiveHeapObject> phi_bb66_13;
  TNode<IntPtrT> tmp149;
  TNode<IntPtrT> tmp150;
  TNode<BoolT> tmp151;
  if (block66.is_used()) {
    ca_.Bind(&block66, &phi_bb66_9, &phi_bb66_10, &phi_bb66_11, &phi_bb66_12, &phi_bb66_13);
    tmp149 = CodeStubAssembler(state_).LoadStringLengthAsWord(TNode<String>{phi_bb11_16});
    tmp150 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb66_11}, TNode<IntPtrT>{tmp149});
    tmp151 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<HeapObject, Smi, Weak<HeapObject>>>{phi_bb11_16}, TNode<Union<HeapObject, Smi, Weak<HeapObject>>>{phi_bb66_13});
    ca_.Branch(tmp151, &block131, std::vector<compiler::Node*>{phi_bb66_9, phi_bb66_10, phi_bb66_12, phi_bb66_13}, &block132, std::vector<compiler::Node*>{phi_bb66_9, phi_bb66_10, phi_bb66_12, phi_bb66_13});
  }

  TNode<FixedArray> phi_bb131_9;
  TNode<IntPtrT> phi_bb131_10;
  TNode<BoolT> phi_bb131_12;
  TNode<PrimitiveHeapObject> phi_bb131_13;
  TNode<Union<HeapObject, TaggedIndex>> tmp152;
  TNode<IntPtrT> tmp153;
  TNode<IntPtrT> tmp154;
  TNode<IntPtrT> tmp155;
  TNode<IntPtrT> tmp156;
  TNode<UintPtrT> tmp157;
  TNode<UintPtrT> tmp158;
  TNode<BoolT> tmp159;
  if (block131.is_used()) {
    ca_.Bind(&block131, &phi_bb131_9, &phi_bb131_10, &phi_bb131_12, &phi_bb131_13);
    std::tie(tmp152, tmp153, tmp154) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb131_9}).Flatten();
    tmp155 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp156 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb131_10}, TNode<IntPtrT>{tmp155});
    tmp157 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp156});
    tmp158 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp154});
    tmp159 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp157}, TNode<UintPtrT>{tmp158});
    ca_.Branch(tmp159, &block143, std::vector<compiler::Node*>{phi_bb131_9, phi_bb131_10, phi_bb131_12, phi_bb131_13, phi_bb131_9}, &block144, std::vector<compiler::Node*>{phi_bb131_9, phi_bb131_10, phi_bb131_12, phi_bb131_13, phi_bb131_9});
  }

  TNode<FixedArray> phi_bb143_9;
  TNode<IntPtrT> phi_bb143_10;
  TNode<BoolT> phi_bb143_12;
  TNode<PrimitiveHeapObject> phi_bb143_13;
  TNode<FixedArray> phi_bb143_26;
  TNode<IntPtrT> tmp160;
  TNode<IntPtrT> tmp161;
  TNode<Union<HeapObject, TaggedIndex>> tmp162;
  TNode<IntPtrT> tmp163;
  TNode<Object> tmp164;
  TNode<HeapObject> tmp165;
  if (block143.is_used()) {
    ca_.Bind(&block143, &phi_bb143_9, &phi_bb143_10, &phi_bb143_12, &phi_bb143_13, &phi_bb143_26);
    tmp160 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp156});
    tmp161 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp153}, TNode<IntPtrT>{tmp160});
    std::tie(tmp162, tmp163) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp152}, TNode<IntPtrT>{tmp161}).Flatten();
    tmp164 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp162, tmp163});
    compiler::CodeAssemblerLabel label166(&ca_);
    tmp165 = CodeStubAssembler(state_).TaggedToHeapObject(TNode<Object>{tmp164}, &label166);
    ca_.Goto(&block150, phi_bb143_9, phi_bb143_10, phi_bb143_12, phi_bb143_13);
    if (label166.is_used()) {
      ca_.Bind(&label166);
      ca_.Goto(&block151, phi_bb143_9, phi_bb143_10, phi_bb143_12, phi_bb143_13);
    }
  }

  TNode<FixedArray> phi_bb144_9;
  TNode<IntPtrT> phi_bb144_10;
  TNode<BoolT> phi_bb144_12;
  TNode<PrimitiveHeapObject> phi_bb144_13;
  TNode<FixedArray> phi_bb144_26;
  if (block144.is_used()) {
    ca_.Bind(&block144, &phi_bb144_9, &phi_bb144_10, &phi_bb144_12, &phi_bb144_13, &phi_bb144_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb151_9;
  TNode<IntPtrT> phi_bb151_10;
  TNode<BoolT> phi_bb151_12;
  TNode<PrimitiveHeapObject> phi_bb151_13;
  if (block151.is_used()) {
    ca_.Bind(&block151, &phi_bb151_9, &phi_bb151_10, &phi_bb151_12, &phi_bb151_13);
    ca_.Goto(&block148, phi_bb151_9, phi_bb151_10, phi_bb151_12, phi_bb151_13);
  }

  TNode<FixedArray> phi_bb150_9;
  TNode<IntPtrT> phi_bb150_10;
  TNode<BoolT> phi_bb150_12;
  TNode<PrimitiveHeapObject> phi_bb150_13;
  TNode<String> tmp167;
  if (block150.is_used()) {
    ca_.Bind(&block150, &phi_bb150_9, &phi_bb150_10, &phi_bb150_12, &phi_bb150_13);
    compiler::CodeAssemblerLabel label168(&ca_);
    tmp167 = Cast_String_0(state_, TNode<HeapObject>{tmp165}, &label168);
    ca_.Goto(&block152, phi_bb150_9, phi_bb150_10, phi_bb150_12, phi_bb150_13);
    if (label168.is_used()) {
      ca_.Bind(&label168);
      ca_.Goto(&block153, phi_bb150_9, phi_bb150_10, phi_bb150_12, phi_bb150_13);
    }
  }

  TNode<FixedArray> phi_bb153_9;
  TNode<IntPtrT> phi_bb153_10;
  TNode<BoolT> phi_bb153_12;
  TNode<PrimitiveHeapObject> phi_bb153_13;
  if (block153.is_used()) {
    ca_.Bind(&block153, &phi_bb153_9, &phi_bb153_10, &phi_bb153_12, &phi_bb153_13);
    ca_.Goto(&block148, phi_bb153_9, phi_bb153_10, phi_bb153_12, phi_bb153_13);
  }

  TNode<FixedArray> phi_bb152_9;
  TNode<IntPtrT> phi_bb152_10;
  TNode<BoolT> phi_bb152_12;
  TNode<PrimitiveHeapObject> phi_bb152_13;
  TNode<Smi> tmp169;
  TNode<IntPtrT> tmp170;
  TNode<BoolT> tmp171;
  if (block152.is_used()) {
    ca_.Bind(&block152, &phi_bb152_9, &phi_bb152_10, &phi_bb152_12, &phi_bb152_13);
    tmp169 = SmiConstant_0(state_, IntegerLiteral(true, 0x1ull));
    tmp170 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb152_9});
    tmp171 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb152_10}, TNode<IntPtrT>{tmp170});
    ca_.Branch(tmp171, &block163, std::vector<compiler::Node*>{phi_bb152_9, phi_bb152_10, phi_bb152_12, phi_bb152_13}, &block164, std::vector<compiler::Node*>{phi_bb152_9, phi_bb152_10, phi_bb152_12, phi_bb152_13});
  }

  TNode<FixedArray> phi_bb163_9;
  TNode<IntPtrT> phi_bb163_10;
  TNode<BoolT> phi_bb163_12;
  TNode<PrimitiveHeapObject> phi_bb163_13;
  TNode<Union<HeapObject, TaggedIndex>> tmp172;
  TNode<IntPtrT> tmp173;
  TNode<IntPtrT> tmp174;
  TNode<IntPtrT> tmp175;
  TNode<IntPtrT> tmp176;
  TNode<UintPtrT> tmp177;
  TNode<UintPtrT> tmp178;
  TNode<BoolT> tmp179;
  if (block163.is_used()) {
    ca_.Bind(&block163, &phi_bb163_9, &phi_bb163_10, &phi_bb163_12, &phi_bb163_13);
    std::tie(tmp172, tmp173, tmp174) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb163_9}).Flatten();
    tmp175 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp176 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb163_10}, TNode<IntPtrT>{tmp175});
    tmp177 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb163_10});
    tmp178 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp174});
    tmp179 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp177}, TNode<UintPtrT>{tmp178});
    ca_.Branch(tmp179, &block170, std::vector<compiler::Node*>{phi_bb163_9, phi_bb163_12, phi_bb163_13, phi_bb163_9, phi_bb163_10, phi_bb163_10, phi_bb163_10, phi_bb163_10}, &block171, std::vector<compiler::Node*>{phi_bb163_9, phi_bb163_12, phi_bb163_13, phi_bb163_9, phi_bb163_10, phi_bb163_10, phi_bb163_10, phi_bb163_10});
  }

  TNode<FixedArray> phi_bb170_9;
  TNode<BoolT> phi_bb170_12;
  TNode<PrimitiveHeapObject> phi_bb170_13;
  TNode<FixedArray> phi_bb170_31;
  TNode<IntPtrT> phi_bb170_35;
  TNode<IntPtrT> phi_bb170_36;
  TNode<IntPtrT> phi_bb170_40;
  TNode<IntPtrT> phi_bb170_41;
  TNode<IntPtrT> tmp180;
  TNode<IntPtrT> tmp181;
  TNode<Union<HeapObject, TaggedIndex>> tmp182;
  TNode<IntPtrT> tmp183;
  if (block170.is_used()) {
    ca_.Bind(&block170, &phi_bb170_9, &phi_bb170_12, &phi_bb170_13, &phi_bb170_31, &phi_bb170_35, &phi_bb170_36, &phi_bb170_40, &phi_bb170_41);
    tmp180 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb170_41});
    tmp181 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp173}, TNode<IntPtrT>{tmp180});
    std::tie(tmp182, tmp183) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp172}, TNode<IntPtrT>{tmp181}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp182, tmp183}, tmp169);
    ca_.Goto(&block165, phi_bb170_9, tmp176, phi_bb170_12, phi_bb170_13);
  }

  TNode<FixedArray> phi_bb171_9;
  TNode<BoolT> phi_bb171_12;
  TNode<PrimitiveHeapObject> phi_bb171_13;
  TNode<FixedArray> phi_bb171_31;
  TNode<IntPtrT> phi_bb171_35;
  TNode<IntPtrT> phi_bb171_36;
  TNode<IntPtrT> phi_bb171_40;
  TNode<IntPtrT> phi_bb171_41;
  if (block171.is_used()) {
    ca_.Bind(&block171, &phi_bb171_9, &phi_bb171_12, &phi_bb171_13, &phi_bb171_31, &phi_bb171_35, &phi_bb171_36, &phi_bb171_40, &phi_bb171_41);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb164_9;
  TNode<IntPtrT> phi_bb164_10;
  TNode<BoolT> phi_bb164_12;
  TNode<PrimitiveHeapObject> phi_bb164_13;
  TNode<IntPtrT> tmp184;
  TNode<IntPtrT> tmp185;
  TNode<BoolT> tmp186;
  if (block164.is_used()) {
    ca_.Bind(&block164, &phi_bb164_9, &phi_bb164_10, &phi_bb164_12, &phi_bb164_13);
    tmp184 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp170});
    tmp185 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp186 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp184}, TNode<IntPtrT>{tmp185});
    ca_.Branch(tmp186, &block174, std::vector<compiler::Node*>{phi_bb164_9, phi_bb164_10, phi_bb164_12, phi_bb164_13}, &block175, std::vector<compiler::Node*>{phi_bb164_9, phi_bb164_10, phi_bb164_12, phi_bb164_13});
  }

  TNode<FixedArray> phi_bb174_9;
  TNode<IntPtrT> phi_bb174_10;
  TNode<BoolT> phi_bb174_12;
  TNode<PrimitiveHeapObject> phi_bb174_13;
  TNode<IntPtrT> tmp187;
  if (block174.is_used()) {
    ca_.Bind(&block174, &phi_bb174_9, &phi_bb174_10, &phi_bb174_12, &phi_bb174_13);
    tmp187 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block176, phi_bb174_9, phi_bb174_10, phi_bb174_12, phi_bb174_13, tmp187);
  }

  TNode<FixedArray> phi_bb175_9;
  TNode<IntPtrT> phi_bb175_10;
  TNode<BoolT> phi_bb175_12;
  TNode<PrimitiveHeapObject> phi_bb175_13;
  if (block175.is_used()) {
    ca_.Bind(&block175, &phi_bb175_9, &phi_bb175_10, &phi_bb175_12, &phi_bb175_13);
    ca_.Goto(&block176, phi_bb175_9, phi_bb175_10, phi_bb175_12, phi_bb175_13, tmp184);
  }

  TNode<FixedArray> phi_bb176_9;
  TNode<IntPtrT> phi_bb176_10;
  TNode<BoolT> phi_bb176_12;
  TNode<PrimitiveHeapObject> phi_bb176_13;
  TNode<IntPtrT> phi_bb176_32;
  TNode<FixedArray> tmp188;
  TNode<Union<HeapObject, TaggedIndex>> tmp189;
  TNode<IntPtrT> tmp190;
  TNode<IntPtrT> tmp191;
  TNode<UintPtrT> tmp192;
  TNode<IntPtrT> tmp193;
  TNode<UintPtrT> tmp194;
  TNode<UintPtrT> tmp195;
  TNode<BoolT> tmp196;
  if (block176.is_used()) {
    ca_.Bind(&block176, &phi_bb176_9, &phi_bb176_10, &phi_bb176_12, &phi_bb176_13, &phi_bb176_32);
    tmp188 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb176_32});
    std::tie(tmp189, tmp190, tmp191) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb176_9}).Flatten();
    tmp192 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp193 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp192});
    tmp194 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp193});
    tmp195 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp191});
    tmp196 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp194}, TNode<UintPtrT>{tmp195});
    ca_.Branch(tmp196, &block187, std::vector<compiler::Node*>{phi_bb176_9, phi_bb176_10, phi_bb176_12, phi_bb176_13, phi_bb176_9}, &block188, std::vector<compiler::Node*>{phi_bb176_9, phi_bb176_10, phi_bb176_12, phi_bb176_13, phi_bb176_9});
  }

  TNode<FixedArray> phi_bb187_9;
  TNode<IntPtrT> phi_bb187_10;
  TNode<BoolT> phi_bb187_12;
  TNode<PrimitiveHeapObject> phi_bb187_13;
  TNode<FixedArray> phi_bb187_34;
  TNode<IntPtrT> tmp197;
  TNode<IntPtrT> tmp198;
  TNode<Union<HeapObject, TaggedIndex>> tmp199;
  TNode<IntPtrT> tmp200;
  TNode<Union<HeapObject, TaggedIndex>> tmp201;
  TNode<IntPtrT> tmp202;
  TNode<IntPtrT> tmp203;
  TNode<UintPtrT> tmp204;
  TNode<IntPtrT> tmp205;
  TNode<UintPtrT> tmp206;
  TNode<UintPtrT> tmp207;
  TNode<BoolT> tmp208;
  if (block187.is_used()) {
    ca_.Bind(&block187, &phi_bb187_9, &phi_bb187_10, &phi_bb187_12, &phi_bb187_13, &phi_bb187_34);
    tmp197 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp193});
    tmp198 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp190}, TNode<IntPtrT>{tmp197});
    std::tie(tmp199, tmp200) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp189}, TNode<IntPtrT>{tmp198}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp199, tmp200}, tmp188);
    std::tie(tmp201, tmp202, tmp203) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp188}).Flatten();
    tmp204 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp205 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp204});
    tmp206 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp205});
    tmp207 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp203});
    tmp208 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp206}, TNode<UintPtrT>{tmp207});
    ca_.Branch(tmp208, &block196, std::vector<compiler::Node*>{phi_bb187_9, phi_bb187_10, phi_bb187_12, phi_bb187_13}, &block197, std::vector<compiler::Node*>{phi_bb187_9, phi_bb187_10, phi_bb187_12, phi_bb187_13});
  }

  TNode<FixedArray> phi_bb188_9;
  TNode<IntPtrT> phi_bb188_10;
  TNode<BoolT> phi_bb188_12;
  TNode<PrimitiveHeapObject> phi_bb188_13;
  TNode<FixedArray> phi_bb188_34;
  if (block188.is_used()) {
    ca_.Bind(&block188, &phi_bb188_9, &phi_bb188_10, &phi_bb188_12, &phi_bb188_13, &phi_bb188_34);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb196_9;
  TNode<IntPtrT> phi_bb196_10;
  TNode<BoolT> phi_bb196_12;
  TNode<PrimitiveHeapObject> phi_bb196_13;
  TNode<IntPtrT> tmp209;
  TNode<IntPtrT> tmp210;
  TNode<Union<HeapObject, TaggedIndex>> tmp211;
  TNode<IntPtrT> tmp212;
  TNode<Undefined> tmp213;
  TNode<Union<HeapObject, TaggedIndex>> tmp214;
  TNode<IntPtrT> tmp215;
  TNode<IntPtrT> tmp216;
  TNode<UintPtrT> tmp217;
  TNode<IntPtrT> tmp218;
  TNode<UintPtrT> tmp219;
  TNode<UintPtrT> tmp220;
  TNode<BoolT> tmp221;
  if (block196.is_used()) {
    ca_.Bind(&block196, &phi_bb196_9, &phi_bb196_10, &phi_bb196_12, &phi_bb196_13);
    tmp209 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp205});
    tmp210 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp202}, TNode<IntPtrT>{tmp209});
    std::tie(tmp211, tmp212) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp201}, TNode<IntPtrT>{tmp210}).Flatten();
    tmp213 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp211, tmp212}, tmp213);
    std::tie(tmp214, tmp215, tmp216) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp188}).Flatten();
    tmp217 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp218 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp217});
    tmp219 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp218});
    tmp220 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp216});
    tmp221 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp219}, TNode<UintPtrT>{tmp220});
    ca_.Branch(tmp221, &block205, std::vector<compiler::Node*>{phi_bb196_9, phi_bb196_10, phi_bb196_12, phi_bb196_13}, &block206, std::vector<compiler::Node*>{phi_bb196_9, phi_bb196_10, phi_bb196_12, phi_bb196_13});
  }

  TNode<FixedArray> phi_bb197_9;
  TNode<IntPtrT> phi_bb197_10;
  TNode<BoolT> phi_bb197_12;
  TNode<PrimitiveHeapObject> phi_bb197_13;
  if (block197.is_used()) {
    ca_.Bind(&block197, &phi_bb197_9, &phi_bb197_10, &phi_bb197_12, &phi_bb197_13);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb205_9;
  TNode<IntPtrT> phi_bb205_10;
  TNode<BoolT> phi_bb205_12;
  TNode<PrimitiveHeapObject> phi_bb205_13;
  TNode<IntPtrT> tmp222;
  TNode<IntPtrT> tmp223;
  TNode<Union<HeapObject, TaggedIndex>> tmp224;
  TNode<IntPtrT> tmp225;
  TNode<IntPtrT> tmp226;
  if (block205.is_used()) {
    ca_.Bind(&block205, &phi_bb205_9, &phi_bb205_10, &phi_bb205_12, &phi_bb205_13);
    tmp222 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp218});
    tmp223 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp215}, TNode<IntPtrT>{tmp222});
    std::tie(tmp224, tmp225) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp214}, TNode<IntPtrT>{tmp223}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp224, tmp225}, tmp169);
    tmp226 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block165, tmp188, tmp226, phi_bb205_12, phi_bb205_13);
  }

  TNode<FixedArray> phi_bb206_9;
  TNode<IntPtrT> phi_bb206_10;
  TNode<BoolT> phi_bb206_12;
  TNode<PrimitiveHeapObject> phi_bb206_13;
  if (block206.is_used()) {
    ca_.Bind(&block206, &phi_bb206_9, &phi_bb206_10, &phi_bb206_12, &phi_bb206_13);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb165_9;
  TNode<IntPtrT> phi_bb165_10;
  TNode<BoolT> phi_bb165_12;
  TNode<PrimitiveHeapObject> phi_bb165_13;
  if (block165.is_used()) {
    ca_.Bind(&block165, &phi_bb165_9, &phi_bb165_10, &phi_bb165_12, &phi_bb165_13);
    ca_.Goto(&block147, phi_bb165_9, phi_bb165_10, phi_bb165_12, phi_bb165_13);
  }

  TNode<FixedArray> phi_bb148_9;
  TNode<IntPtrT> phi_bb148_10;
  TNode<BoolT> phi_bb148_12;
  TNode<PrimitiveHeapObject> phi_bb148_13;
  TNode<Smi> tmp227;
  if (block148.is_used()) {
    ca_.Bind(&block148, &phi_bb148_9, &phi_bb148_10, &phi_bb148_12, &phi_bb148_13);
    compiler::CodeAssemblerLabel label228(&ca_);
    tmp227 = Cast_Smi_0(state_, TNode<Object>{ca_.UncheckedCast<Object>(tmp164)}, &label228);
    ca_.Goto(&block211, phi_bb148_9, phi_bb148_10, phi_bb148_12, phi_bb148_13);
    if (label228.is_used()) {
      ca_.Bind(&label228);
      ca_.Goto(&block212, phi_bb148_9, phi_bb148_10, phi_bb148_12, phi_bb148_13);
    }
  }

  TNode<FixedArray> phi_bb212_9;
  TNode<IntPtrT> phi_bb212_10;
  TNode<BoolT> phi_bb212_12;
  TNode<PrimitiveHeapObject> phi_bb212_13;
  if (block212.is_used()) {
    ca_.Bind(&block212, &phi_bb212_9, &phi_bb212_10, &phi_bb212_12, &phi_bb212_13);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb211_9;
  TNode<IntPtrT> phi_bb211_10;
  TNode<BoolT> phi_bb211_12;
  TNode<PrimitiveHeapObject> phi_bb211_13;
  TNode<Union<HeapObject, TaggedIndex>> tmp229;
  TNode<IntPtrT> tmp230;
  TNode<IntPtrT> tmp231;
  TNode<IntPtrT> tmp232;
  TNode<IntPtrT> tmp233;
  TNode<UintPtrT> tmp234;
  TNode<UintPtrT> tmp235;
  TNode<BoolT> tmp236;
  if (block211.is_used()) {
    ca_.Bind(&block211, &phi_bb211_9, &phi_bb211_10, &phi_bb211_12, &phi_bb211_13);
    std::tie(tmp229, tmp230, tmp231) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb211_9}).Flatten();
    tmp232 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp233 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb211_10}, TNode<IntPtrT>{tmp232});
    tmp234 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp233});
    tmp235 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp231});
    tmp236 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp234}, TNode<UintPtrT>{tmp235});
    ca_.Branch(tmp236, &block225, std::vector<compiler::Node*>{phi_bb211_9, phi_bb211_10, phi_bb211_12, phi_bb211_13, phi_bb211_9}, &block226, std::vector<compiler::Node*>{phi_bb211_9, phi_bb211_10, phi_bb211_12, phi_bb211_13, phi_bb211_9});
  }

  TNode<FixedArray> phi_bb225_9;
  TNode<IntPtrT> phi_bb225_10;
  TNode<BoolT> phi_bb225_12;
  TNode<PrimitiveHeapObject> phi_bb225_13;
  TNode<FixedArray> phi_bb225_28;
  TNode<IntPtrT> tmp237;
  TNode<IntPtrT> tmp238;
  TNode<Union<HeapObject, TaggedIndex>> tmp239;
  TNode<IntPtrT> tmp240;
  TNode<Smi> tmp241;
  TNode<Smi> tmp242;
  if (block225.is_used()) {
    ca_.Bind(&block225, &phi_bb225_9, &phi_bb225_10, &phi_bb225_12, &phi_bb225_13, &phi_bb225_28);
    tmp237 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp233});
    tmp238 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp230}, TNode<IntPtrT>{tmp237});
    std::tie(tmp239, tmp240) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp229}, TNode<IntPtrT>{tmp238}).Flatten();
    tmp241 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp242 = CodeStubAssembler(state_).SmiSub(TNode<Smi>{tmp227}, TNode<Smi>{tmp241});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp239, tmp240}, tmp242);
    ca_.Goto(&block147, phi_bb225_9, phi_bb225_10, phi_bb225_12, phi_bb225_13);
  }

  TNode<FixedArray> phi_bb226_9;
  TNode<IntPtrT> phi_bb226_10;
  TNode<BoolT> phi_bb226_12;
  TNode<PrimitiveHeapObject> phi_bb226_13;
  TNode<FixedArray> phi_bb226_28;
  if (block226.is_used()) {
    ca_.Bind(&block226, &phi_bb226_9, &phi_bb226_10, &phi_bb226_12, &phi_bb226_13, &phi_bb226_28);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb147_9;
  TNode<IntPtrT> phi_bb147_10;
  TNode<BoolT> phi_bb147_12;
  TNode<PrimitiveHeapObject> phi_bb147_13;
  if (block147.is_used()) {
    ca_.Bind(&block147, &phi_bb147_9, &phi_bb147_10, &phi_bb147_12, &phi_bb147_13);
    ca_.Goto(&block133, phi_bb147_9, phi_bb147_10, phi_bb147_12, phi_bb147_13);
  }

  TNode<FixedArray> phi_bb132_9;
  TNode<IntPtrT> phi_bb132_10;
  TNode<BoolT> phi_bb132_12;
  TNode<PrimitiveHeapObject> phi_bb132_13;
  TNode<IntPtrT> tmp243;
  TNode<BoolT> tmp244;
  if (block132.is_used()) {
    ca_.Bind(&block132, &phi_bb132_9, &phi_bb132_10, &phi_bb132_12, &phi_bb132_13);
    tmp243 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb132_9});
    tmp244 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb132_10}, TNode<IntPtrT>{tmp243});
    ca_.Branch(tmp244, &block238, std::vector<compiler::Node*>{phi_bb132_9, phi_bb132_10, phi_bb132_12, phi_bb132_13}, &block239, std::vector<compiler::Node*>{phi_bb132_9, phi_bb132_10, phi_bb132_12, phi_bb132_13});
  }

  TNode<FixedArray> phi_bb238_9;
  TNode<IntPtrT> phi_bb238_10;
  TNode<BoolT> phi_bb238_12;
  TNode<PrimitiveHeapObject> phi_bb238_13;
  TNode<Union<HeapObject, TaggedIndex>> tmp245;
  TNode<IntPtrT> tmp246;
  TNode<IntPtrT> tmp247;
  TNode<IntPtrT> tmp248;
  TNode<IntPtrT> tmp249;
  TNode<UintPtrT> tmp250;
  TNode<UintPtrT> tmp251;
  TNode<BoolT> tmp252;
  if (block238.is_used()) {
    ca_.Bind(&block238, &phi_bb238_9, &phi_bb238_10, &phi_bb238_12, &phi_bb238_13);
    std::tie(tmp245, tmp246, tmp247) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb238_9}).Flatten();
    tmp248 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp249 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb238_10}, TNode<IntPtrT>{tmp248});
    tmp250 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb238_10});
    tmp251 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp247});
    tmp252 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp250}, TNode<UintPtrT>{tmp251});
    ca_.Branch(tmp252, &block245, std::vector<compiler::Node*>{phi_bb238_9, phi_bb238_12, phi_bb238_13, phi_bb238_9, phi_bb238_10, phi_bb238_10, phi_bb238_10, phi_bb238_10}, &block246, std::vector<compiler::Node*>{phi_bb238_9, phi_bb238_12, phi_bb238_13, phi_bb238_9, phi_bb238_10, phi_bb238_10, phi_bb238_10, phi_bb238_10});
  }

  TNode<FixedArray> phi_bb245_9;
  TNode<BoolT> phi_bb245_12;
  TNode<PrimitiveHeapObject> phi_bb245_13;
  TNode<FixedArray> phi_bb245_29;
  TNode<IntPtrT> phi_bb245_33;
  TNode<IntPtrT> phi_bb245_34;
  TNode<IntPtrT> phi_bb245_38;
  TNode<IntPtrT> phi_bb245_39;
  TNode<IntPtrT> tmp253;
  TNode<IntPtrT> tmp254;
  TNode<Union<HeapObject, TaggedIndex>> tmp255;
  TNode<IntPtrT> tmp256;
  if (block245.is_used()) {
    ca_.Bind(&block245, &phi_bb245_9, &phi_bb245_12, &phi_bb245_13, &phi_bb245_29, &phi_bb245_33, &phi_bb245_34, &phi_bb245_38, &phi_bb245_39);
    tmp253 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb245_39});
    tmp254 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp246}, TNode<IntPtrT>{tmp253});
    std::tie(tmp255, tmp256) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp245}, TNode<IntPtrT>{tmp254}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp255, tmp256}, phi_bb11_16);
    ca_.Goto(&block240, phi_bb245_9, tmp249, phi_bb245_12, phi_bb245_13);
  }

  TNode<FixedArray> phi_bb246_9;
  TNode<BoolT> phi_bb246_12;
  TNode<PrimitiveHeapObject> phi_bb246_13;
  TNode<FixedArray> phi_bb246_29;
  TNode<IntPtrT> phi_bb246_33;
  TNode<IntPtrT> phi_bb246_34;
  TNode<IntPtrT> phi_bb246_38;
  TNode<IntPtrT> phi_bb246_39;
  if (block246.is_used()) {
    ca_.Bind(&block246, &phi_bb246_9, &phi_bb246_12, &phi_bb246_13, &phi_bb246_29, &phi_bb246_33, &phi_bb246_34, &phi_bb246_38, &phi_bb246_39);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb239_9;
  TNode<IntPtrT> phi_bb239_10;
  TNode<BoolT> phi_bb239_12;
  TNode<PrimitiveHeapObject> phi_bb239_13;
  TNode<IntPtrT> tmp257;
  TNode<IntPtrT> tmp258;
  TNode<BoolT> tmp259;
  if (block239.is_used()) {
    ca_.Bind(&block239, &phi_bb239_9, &phi_bb239_10, &phi_bb239_12, &phi_bb239_13);
    tmp257 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp243});
    tmp258 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp259 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp257}, TNode<IntPtrT>{tmp258});
    ca_.Branch(tmp259, &block249, std::vector<compiler::Node*>{phi_bb239_9, phi_bb239_10, phi_bb239_12, phi_bb239_13}, &block250, std::vector<compiler::Node*>{phi_bb239_9, phi_bb239_10, phi_bb239_12, phi_bb239_13});
  }

  TNode<FixedArray> phi_bb249_9;
  TNode<IntPtrT> phi_bb249_10;
  TNode<BoolT> phi_bb249_12;
  TNode<PrimitiveHeapObject> phi_bb249_13;
  TNode<IntPtrT> tmp260;
  if (block249.is_used()) {
    ca_.Bind(&block249, &phi_bb249_9, &phi_bb249_10, &phi_bb249_12, &phi_bb249_13);
    tmp260 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block251, phi_bb249_9, phi_bb249_10, phi_bb249_12, phi_bb249_13, tmp260);
  }

  TNode<FixedArray> phi_bb250_9;
  TNode<IntPtrT> phi_bb250_10;
  TNode<BoolT> phi_bb250_12;
  TNode<PrimitiveHeapObject> phi_bb250_13;
  if (block250.is_used()) {
    ca_.Bind(&block250, &phi_bb250_9, &phi_bb250_10, &phi_bb250_12, &phi_bb250_13);
    ca_.Goto(&block251, phi_bb250_9, phi_bb250_10, phi_bb250_12, phi_bb250_13, tmp257);
  }

  TNode<FixedArray> phi_bb251_9;
  TNode<IntPtrT> phi_bb251_10;
  TNode<BoolT> phi_bb251_12;
  TNode<PrimitiveHeapObject> phi_bb251_13;
  TNode<IntPtrT> phi_bb251_30;
  TNode<FixedArray> tmp261;
  TNode<Union<HeapObject, TaggedIndex>> tmp262;
  TNode<IntPtrT> tmp263;
  TNode<IntPtrT> tmp264;
  TNode<UintPtrT> tmp265;
  TNode<IntPtrT> tmp266;
  TNode<UintPtrT> tmp267;
  TNode<UintPtrT> tmp268;
  TNode<BoolT> tmp269;
  if (block251.is_used()) {
    ca_.Bind(&block251, &phi_bb251_9, &phi_bb251_10, &phi_bb251_12, &phi_bb251_13, &phi_bb251_30);
    tmp261 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb251_30});
    std::tie(tmp262, tmp263, tmp264) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb251_9}).Flatten();
    tmp265 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp266 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp265});
    tmp267 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp266});
    tmp268 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp264});
    tmp269 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp267}, TNode<UintPtrT>{tmp268});
    ca_.Branch(tmp269, &block262, std::vector<compiler::Node*>{phi_bb251_9, phi_bb251_10, phi_bb251_12, phi_bb251_13, phi_bb251_9}, &block263, std::vector<compiler::Node*>{phi_bb251_9, phi_bb251_10, phi_bb251_12, phi_bb251_13, phi_bb251_9});
  }

  TNode<FixedArray> phi_bb262_9;
  TNode<IntPtrT> phi_bb262_10;
  TNode<BoolT> phi_bb262_12;
  TNode<PrimitiveHeapObject> phi_bb262_13;
  TNode<FixedArray> phi_bb262_32;
  TNode<IntPtrT> tmp270;
  TNode<IntPtrT> tmp271;
  TNode<Union<HeapObject, TaggedIndex>> tmp272;
  TNode<IntPtrT> tmp273;
  TNode<Union<HeapObject, TaggedIndex>> tmp274;
  TNode<IntPtrT> tmp275;
  TNode<IntPtrT> tmp276;
  TNode<UintPtrT> tmp277;
  TNode<IntPtrT> tmp278;
  TNode<UintPtrT> tmp279;
  TNode<UintPtrT> tmp280;
  TNode<BoolT> tmp281;
  if (block262.is_used()) {
    ca_.Bind(&block262, &phi_bb262_9, &phi_bb262_10, &phi_bb262_12, &phi_bb262_13, &phi_bb262_32);
    tmp270 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp266});
    tmp271 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp263}, TNode<IntPtrT>{tmp270});
    std::tie(tmp272, tmp273) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp262}, TNode<IntPtrT>{tmp271}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp272, tmp273}, tmp261);
    std::tie(tmp274, tmp275, tmp276) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp261}).Flatten();
    tmp277 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp278 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp277});
    tmp279 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp278});
    tmp280 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp276});
    tmp281 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp279}, TNode<UintPtrT>{tmp280});
    ca_.Branch(tmp281, &block271, std::vector<compiler::Node*>{phi_bb262_9, phi_bb262_10, phi_bb262_12, phi_bb262_13}, &block272, std::vector<compiler::Node*>{phi_bb262_9, phi_bb262_10, phi_bb262_12, phi_bb262_13});
  }

  TNode<FixedArray> phi_bb263_9;
  TNode<IntPtrT> phi_bb263_10;
  TNode<BoolT> phi_bb263_12;
  TNode<PrimitiveHeapObject> phi_bb263_13;
  TNode<FixedArray> phi_bb263_32;
  if (block263.is_used()) {
    ca_.Bind(&block263, &phi_bb263_9, &phi_bb263_10, &phi_bb263_12, &phi_bb263_13, &phi_bb263_32);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb271_9;
  TNode<IntPtrT> phi_bb271_10;
  TNode<BoolT> phi_bb271_12;
  TNode<PrimitiveHeapObject> phi_bb271_13;
  TNode<IntPtrT> tmp282;
  TNode<IntPtrT> tmp283;
  TNode<Union<HeapObject, TaggedIndex>> tmp284;
  TNode<IntPtrT> tmp285;
  TNode<Undefined> tmp286;
  TNode<Union<HeapObject, TaggedIndex>> tmp287;
  TNode<IntPtrT> tmp288;
  TNode<IntPtrT> tmp289;
  TNode<UintPtrT> tmp290;
  TNode<IntPtrT> tmp291;
  TNode<UintPtrT> tmp292;
  TNode<UintPtrT> tmp293;
  TNode<BoolT> tmp294;
  if (block271.is_used()) {
    ca_.Bind(&block271, &phi_bb271_9, &phi_bb271_10, &phi_bb271_12, &phi_bb271_13);
    tmp282 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp278});
    tmp283 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp275}, TNode<IntPtrT>{tmp282});
    std::tie(tmp284, tmp285) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp274}, TNode<IntPtrT>{tmp283}).Flatten();
    tmp286 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp284, tmp285}, tmp286);
    std::tie(tmp287, tmp288, tmp289) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp261}).Flatten();
    tmp290 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp291 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp290});
    tmp292 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp291});
    tmp293 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp289});
    tmp294 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp292}, TNode<UintPtrT>{tmp293});
    ca_.Branch(tmp294, &block280, std::vector<compiler::Node*>{phi_bb271_9, phi_bb271_10, phi_bb271_12, phi_bb271_13}, &block281, std::vector<compiler::Node*>{phi_bb271_9, phi_bb271_10, phi_bb271_12, phi_bb271_13});
  }

  TNode<FixedArray> phi_bb272_9;
  TNode<IntPtrT> phi_bb272_10;
  TNode<BoolT> phi_bb272_12;
  TNode<PrimitiveHeapObject> phi_bb272_13;
  if (block272.is_used()) {
    ca_.Bind(&block272, &phi_bb272_9, &phi_bb272_10, &phi_bb272_12, &phi_bb272_13);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb280_9;
  TNode<IntPtrT> phi_bb280_10;
  TNode<BoolT> phi_bb280_12;
  TNode<PrimitiveHeapObject> phi_bb280_13;
  TNode<IntPtrT> tmp295;
  TNode<IntPtrT> tmp296;
  TNode<Union<HeapObject, TaggedIndex>> tmp297;
  TNode<IntPtrT> tmp298;
  TNode<IntPtrT> tmp299;
  if (block280.is_used()) {
    ca_.Bind(&block280, &phi_bb280_9, &phi_bb280_10, &phi_bb280_12, &phi_bb280_13);
    tmp295 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp291});
    tmp296 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp288}, TNode<IntPtrT>{tmp295});
    std::tie(tmp297, tmp298) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp287}, TNode<IntPtrT>{tmp296}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp297, tmp298}, phi_bb11_16);
    tmp299 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block240, tmp261, tmp299, phi_bb280_12, phi_bb280_13);
  }

  TNode<FixedArray> phi_bb281_9;
  TNode<IntPtrT> phi_bb281_10;
  TNode<BoolT> phi_bb281_12;
  TNode<PrimitiveHeapObject> phi_bb281_13;
  if (block281.is_used()) {
    ca_.Bind(&block281, &phi_bb281_9, &phi_bb281_10, &phi_bb281_12, &phi_bb281_13);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb240_9;
  TNode<IntPtrT> phi_bb240_10;
  TNode<BoolT> phi_bb240_12;
  TNode<PrimitiveHeapObject> phi_bb240_13;
  if (block240.is_used()) {
    ca_.Bind(&block240, &phi_bb240_9, &phi_bb240_10, &phi_bb240_12, &phi_bb240_13);
    ca_.Goto(&block133, phi_bb240_9, phi_bb240_10, phi_bb240_12, phi_bb11_16);
  }

  TNode<FixedArray> phi_bb133_9;
  TNode<IntPtrT> phi_bb133_10;
  TNode<BoolT> phi_bb133_12;
  TNode<PrimitiveHeapObject> phi_bb133_13;
  TNode<IntPtrT> tmp300;
  TNode<Map> tmp301;
  TNode<BoolT> tmp302;
  TNode<BoolT> tmp303;
  TNode<IntPtrT> tmp304;
  if (block133.is_used()) {
    ca_.Bind(&block133, &phi_bb133_9, &phi_bb133_10, &phi_bb133_12, &phi_bb133_13);
    tmp300 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp301 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{phi_bb11_16, tmp300});
    tmp302 = CodeStubAssembler(state_).IsOneByteStringMap(TNode<Map>{tmp301});
    tmp303 = CodeStubAssembler(state_).Word32And(TNode<BoolT>{tmp302}, TNode<BoolT>{phi_bb133_12});
    tmp304 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block6, tmp304, phi_bb133_9, phi_bb133_10, tmp150, tmp303, phi_bb133_13, phi_bb11_15);
  }

  TNode<IntPtrT> phi_bb5_7;
  TNode<FixedArray> phi_bb5_9;
  TNode<IntPtrT> phi_bb5_10;
  TNode<IntPtrT> phi_bb5_11;
  TNode<BoolT> phi_bb5_12;
  TNode<PrimitiveHeapObject> phi_bb5_13;
  TNode<UintPtrT> phi_bb5_15;
  TNode<IntPtrT> tmp305;
  TNode<BoolT> tmp306;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_7, &phi_bb5_9, &phi_bb5_10, &phi_bb5_11, &phi_bb5_12, &phi_bb5_13, &phi_bb5_15);
    tmp305 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp306 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb3_14}, TNode<IntPtrT>{tmp305});
    ca_.Branch(tmp306, &block284, std::vector<compiler::Node*>{phi_bb5_7, phi_bb5_9, phi_bb5_10, phi_bb5_11, phi_bb5_12, phi_bb5_13, phi_bb5_15}, &block285, std::vector<compiler::Node*>{phi_bb5_7, phi_bb5_9, phi_bb5_10, phi_bb5_11, phi_bb5_12, phi_bb5_13, phi_bb5_15});
  }

  TNode<IntPtrT> phi_bb284_7;
  TNode<FixedArray> phi_bb284_9;
  TNode<IntPtrT> phi_bb284_10;
  TNode<IntPtrT> phi_bb284_11;
  TNode<BoolT> phi_bb284_12;
  TNode<PrimitiveHeapObject> phi_bb284_13;
  TNode<UintPtrT> phi_bb284_15;
  TNode<UintPtrT> tmp307;
  TNode<BoolT> tmp308;
  if (block284.is_used()) {
    ca_.Bind(&block284, &phi_bb284_7, &phi_bb284_9, &phi_bb284_10, &phi_bb284_11, &phi_bb284_12, &phi_bb284_13, &phi_bb284_15);
    tmp307 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp308 = CodeStubAssembler(state_).UintPtrGreaterThan(TNode<UintPtrT>{phi_bb284_15}, TNode<UintPtrT>{tmp307});
    ca_.Branch(tmp308, &block286, std::vector<compiler::Node*>{phi_bb284_7, phi_bb284_9, phi_bb284_10, phi_bb284_11, phi_bb284_12, phi_bb284_13, phi_bb284_15}, &block287, std::vector<compiler::Node*>{phi_bb284_7, phi_bb284_9, phi_bb284_10, phi_bb284_11, phi_bb284_12, phi_bb284_13, phi_bb284_15});
  }

  TNode<IntPtrT> phi_bb286_7;
  TNode<FixedArray> phi_bb286_9;
  TNode<IntPtrT> phi_bb286_10;
  TNode<IntPtrT> phi_bb286_11;
  TNode<BoolT> phi_bb286_12;
  TNode<PrimitiveHeapObject> phi_bb286_13;
  TNode<UintPtrT> phi_bb286_15;
  TNode<IntPtrT> tmp309;
  TNode<IntPtrT> tmp310;
  if (block286.is_used()) {
    ca_.Bind(&block286, &phi_bb286_7, &phi_bb286_9, &phi_bb286_10, &phi_bb286_11, &phi_bb286_12, &phi_bb286_13, &phi_bb286_15);
    tmp309 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp310 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb286_7}, TNode<IntPtrT>{tmp309});
    ca_.Goto(&block287, tmp310, phi_bb286_9, phi_bb286_10, phi_bb286_11, phi_bb286_12, phi_bb286_13, phi_bb286_15);
  }

  TNode<IntPtrT> phi_bb287_7;
  TNode<FixedArray> phi_bb287_9;
  TNode<IntPtrT> phi_bb287_10;
  TNode<IntPtrT> phi_bb287_11;
  TNode<BoolT> phi_bb287_12;
  TNode<PrimitiveHeapObject> phi_bb287_13;
  TNode<UintPtrT> phi_bb287_15;
  TNode<IntPtrT> tmp311;
  TNode<IntPtrT> tmp312;
  TNode<IntPtrT> tmp313;
  if (block287.is_used()) {
    ca_.Bind(&block287, &phi_bb287_7, &phi_bb287_9, &phi_bb287_10, &phi_bb287_11, &phi_bb287_12, &phi_bb287_13, &phi_bb287_15);
    tmp311 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp312 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb3_14}, TNode<IntPtrT>{tmp311});
    tmp313 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb287_7}, TNode<IntPtrT>{tmp312});
    ca_.Goto(&block285, tmp313, phi_bb287_9, phi_bb287_10, phi_bb287_11, phi_bb287_12, phi_bb287_13, phi_bb287_15);
  }

  TNode<IntPtrT> phi_bb285_7;
  TNode<FixedArray> phi_bb285_9;
  TNode<IntPtrT> phi_bb285_10;
  TNode<IntPtrT> phi_bb285_11;
  TNode<BoolT> phi_bb285_12;
  TNode<PrimitiveHeapObject> phi_bb285_13;
  TNode<UintPtrT> phi_bb285_15;
  TNode<BoolT> tmp314;
  TNode<IntPtrT> tmp315;
  TNode<BoolT> tmp316;
  if (block285.is_used()) {
    ca_.Bind(&block285, &phi_bb285_7, &phi_bb285_9, &phi_bb285_10, &phi_bb285_11, &phi_bb285_12, &phi_bb285_13, &phi_bb285_15);
    tmp314 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp315 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp316 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb285_7}, TNode<IntPtrT>{tmp315});
    ca_.Branch(tmp316, &block291, std::vector<compiler::Node*>{phi_bb285_7, phi_bb285_9, phi_bb285_10, phi_bb285_11, phi_bb285_12, phi_bb285_13, phi_bb285_15, phi_bb285_7, phi_bb285_7}, &block292, std::vector<compiler::Node*>{phi_bb285_7, phi_bb285_9, phi_bb285_10, phi_bb285_11, phi_bb285_12, phi_bb285_13, phi_bb285_15, phi_bb285_7, phi_bb285_7});
  }

  TNode<IntPtrT> phi_bb291_7;
  TNode<FixedArray> phi_bb291_9;
  TNode<IntPtrT> phi_bb291_10;
  TNode<IntPtrT> phi_bb291_11;
  TNode<BoolT> phi_bb291_12;
  TNode<PrimitiveHeapObject> phi_bb291_13;
  TNode<UintPtrT> phi_bb291_15;
  TNode<IntPtrT> phi_bb291_16;
  TNode<IntPtrT> phi_bb291_20;
  TNode<BoolT> tmp317;
  if (block291.is_used()) {
    ca_.Bind(&block291, &phi_bb291_7, &phi_bb291_9, &phi_bb291_10, &phi_bb291_11, &phi_bb291_12, &phi_bb291_13, &phi_bb291_15, &phi_bb291_16, &phi_bb291_20);
    tmp317 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block293, phi_bb291_7, phi_bb291_9, phi_bb291_10, phi_bb291_11, phi_bb291_12, phi_bb291_13, phi_bb291_15, phi_bb291_16, phi_bb291_20, tmp317);
  }

  TNode<IntPtrT> phi_bb292_7;
  TNode<FixedArray> phi_bb292_9;
  TNode<IntPtrT> phi_bb292_10;
  TNode<IntPtrT> phi_bb292_11;
  TNode<BoolT> phi_bb292_12;
  TNode<PrimitiveHeapObject> phi_bb292_13;
  TNode<UintPtrT> phi_bb292_15;
  TNode<IntPtrT> phi_bb292_16;
  TNode<IntPtrT> phi_bb292_20;
  TNode<IntPtrT> tmp318;
  TNode<BoolT> tmp319;
  if (block292.is_used()) {
    ca_.Bind(&block292, &phi_bb292_7, &phi_bb292_9, &phi_bb292_10, &phi_bb292_11, &phi_bb292_12, &phi_bb292_13, &phi_bb292_15, &phi_bb292_16, &phi_bb292_20);
    tmp318 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp319 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp318});
    ca_.Goto(&block293, phi_bb292_7, phi_bb292_9, phi_bb292_10, phi_bb292_11, phi_bb292_12, phi_bb292_13, phi_bb292_15, phi_bb292_16, phi_bb292_20, tmp319);
  }

  TNode<IntPtrT> phi_bb293_7;
  TNode<FixedArray> phi_bb293_9;
  TNode<IntPtrT> phi_bb293_10;
  TNode<IntPtrT> phi_bb293_11;
  TNode<BoolT> phi_bb293_12;
  TNode<PrimitiveHeapObject> phi_bb293_13;
  TNode<UintPtrT> phi_bb293_15;
  TNode<IntPtrT> phi_bb293_16;
  TNode<IntPtrT> phi_bb293_20;
  TNode<BoolT> phi_bb293_24;
  if (block293.is_used()) {
    ca_.Bind(&block293, &phi_bb293_7, &phi_bb293_9, &phi_bb293_10, &phi_bb293_11, &phi_bb293_12, &phi_bb293_13, &phi_bb293_15, &phi_bb293_16, &phi_bb293_20, &phi_bb293_24);
    ca_.Branch(phi_bb293_24, &block289, std::vector<compiler::Node*>{phi_bb293_7, phi_bb293_9, phi_bb293_10, phi_bb293_11, phi_bb293_12, phi_bb293_13, phi_bb293_15, phi_bb293_16, phi_bb293_20}, &block290, std::vector<compiler::Node*>{phi_bb293_7, phi_bb293_9, phi_bb293_10, phi_bb293_11, phi_bb293_12, phi_bb293_13, phi_bb293_15, phi_bb293_16, phi_bb293_20});
  }

  TNode<IntPtrT> phi_bb289_7;
  TNode<FixedArray> phi_bb289_9;
  TNode<IntPtrT> phi_bb289_10;
  TNode<IntPtrT> phi_bb289_11;
  TNode<BoolT> phi_bb289_12;
  TNode<PrimitiveHeapObject> phi_bb289_13;
  TNode<UintPtrT> phi_bb289_15;
  TNode<IntPtrT> phi_bb289_16;
  TNode<IntPtrT> phi_bb289_20;
  if (block289.is_used()) {
    ca_.Bind(&block289, &phi_bb289_7, &phi_bb289_9, &phi_bb289_10, &phi_bb289_11, &phi_bb289_12, &phi_bb289_13, &phi_bb289_15, &phi_bb289_16, &phi_bb289_20);
    ca_.Goto(&block288, phi_bb289_7, phi_bb289_9, phi_bb289_10, phi_bb289_11, phi_bb289_12, phi_bb289_13, phi_bb289_15, phi_bb289_16, phi_bb289_20);
  }

  TNode<IntPtrT> phi_bb290_7;
  TNode<FixedArray> phi_bb290_9;
  TNode<IntPtrT> phi_bb290_10;
  TNode<IntPtrT> phi_bb290_11;
  TNode<BoolT> phi_bb290_12;
  TNode<PrimitiveHeapObject> phi_bb290_13;
  TNode<UintPtrT> phi_bb290_15;
  TNode<IntPtrT> phi_bb290_16;
  TNode<IntPtrT> phi_bb290_20;
  TNode<IntPtrT> tmp320;
  TNode<IntPtrT> tmp321;
  TNode<BoolT> tmp322;
  if (block290.is_used()) {
    ca_.Bind(&block290, &phi_bb290_7, &phi_bb290_9, &phi_bb290_10, &phi_bb290_11, &phi_bb290_12, &phi_bb290_13, &phi_bb290_15, &phi_bb290_16, &phi_bb290_20);
    tmp320 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{phi_bb290_20});
    tmp321 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp320}, TNode<IntPtrT>{tmp3});
    tmp322 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{tmp321}, TNode<IntPtrT>{phi_bb290_20});
    ca_.Branch(tmp322, &block294, std::vector<compiler::Node*>{phi_bb290_7, phi_bb290_9, phi_bb290_10, phi_bb290_11, phi_bb290_12, phi_bb290_13, phi_bb290_15, phi_bb290_16, phi_bb290_20, phi_bb290_20}, &block295, std::vector<compiler::Node*>{phi_bb290_7, phi_bb290_9, phi_bb290_10, phi_bb290_11, phi_bb290_12, phi_bb290_13, phi_bb290_15, phi_bb290_16, phi_bb290_20, phi_bb290_20});
  }

  TNode<IntPtrT> phi_bb294_7;
  TNode<FixedArray> phi_bb294_9;
  TNode<IntPtrT> phi_bb294_10;
  TNode<IntPtrT> phi_bb294_11;
  TNode<BoolT> phi_bb294_12;
  TNode<PrimitiveHeapObject> phi_bb294_13;
  TNode<UintPtrT> phi_bb294_15;
  TNode<IntPtrT> phi_bb294_16;
  TNode<IntPtrT> phi_bb294_20;
  TNode<IntPtrT> phi_bb294_23;
  if (block294.is_used()) {
    ca_.Bind(&block294, &phi_bb294_7, &phi_bb294_9, &phi_bb294_10, &phi_bb294_11, &phi_bb294_12, &phi_bb294_13, &phi_bb294_15, &phi_bb294_16, &phi_bb294_20, &phi_bb294_23);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowInvalidStringLength, p_context);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb295_7;
  TNode<FixedArray> phi_bb295_9;
  TNode<IntPtrT> phi_bb295_10;
  TNode<IntPtrT> phi_bb295_11;
  TNode<BoolT> phi_bb295_12;
  TNode<PrimitiveHeapObject> phi_bb295_13;
  TNode<UintPtrT> phi_bb295_15;
  TNode<IntPtrT> phi_bb295_16;
  TNode<IntPtrT> phi_bb295_20;
  TNode<IntPtrT> phi_bb295_23;
  TNode<IntPtrT> tmp323;
  if (block295.is_used()) {
    ca_.Bind(&block295, &phi_bb295_7, &phi_bb295_9, &phi_bb295_10, &phi_bb295_11, &phi_bb295_12, &phi_bb295_13, &phi_bb295_15, &phi_bb295_16, &phi_bb295_20, &phi_bb295_23);
    tmp323 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb295_11}, TNode<IntPtrT>{tmp320});
    ca_.Branch(tmp314, &block296, std::vector<compiler::Node*>{phi_bb295_7, phi_bb295_9, phi_bb295_10, phi_bb295_12, phi_bb295_13, phi_bb295_15, phi_bb295_16, phi_bb295_20, phi_bb295_23}, &block297, std::vector<compiler::Node*>{phi_bb295_7, phi_bb295_9, phi_bb295_10, phi_bb295_12, phi_bb295_13, phi_bb295_15, phi_bb295_16, phi_bb295_20, phi_bb295_23});
  }

  TNode<IntPtrT> phi_bb296_7;
  TNode<FixedArray> phi_bb296_9;
  TNode<IntPtrT> phi_bb296_10;
  TNode<BoolT> phi_bb296_12;
  TNode<PrimitiveHeapObject> phi_bb296_13;
  TNode<UintPtrT> phi_bb296_15;
  TNode<IntPtrT> phi_bb296_16;
  TNode<IntPtrT> phi_bb296_20;
  TNode<IntPtrT> phi_bb296_23;
  TNode<Smi> tmp324;
  TNode<IntPtrT> tmp325;
  TNode<BoolT> tmp326;
  if (block296.is_used()) {
    ca_.Bind(&block296, &phi_bb296_7, &phi_bb296_9, &phi_bb296_10, &phi_bb296_12, &phi_bb296_13, &phi_bb296_15, &phi_bb296_16, &phi_bb296_20, &phi_bb296_23);
    tmp324 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{phi_bb296_23});
    tmp325 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb296_9});
    tmp326 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb296_10}, TNode<IntPtrT>{tmp325});
    ca_.Branch(tmp326, &block307, std::vector<compiler::Node*>{phi_bb296_7, phi_bb296_9, phi_bb296_10, phi_bb296_12, phi_bb296_13, phi_bb296_15, phi_bb296_16, phi_bb296_20, phi_bb296_23}, &block308, std::vector<compiler::Node*>{phi_bb296_7, phi_bb296_9, phi_bb296_10, phi_bb296_12, phi_bb296_13, phi_bb296_15, phi_bb296_16, phi_bb296_20, phi_bb296_23});
  }

  TNode<IntPtrT> phi_bb307_7;
  TNode<FixedArray> phi_bb307_9;
  TNode<IntPtrT> phi_bb307_10;
  TNode<BoolT> phi_bb307_12;
  TNode<PrimitiveHeapObject> phi_bb307_13;
  TNode<UintPtrT> phi_bb307_15;
  TNode<IntPtrT> phi_bb307_16;
  TNode<IntPtrT> phi_bb307_20;
  TNode<IntPtrT> phi_bb307_23;
  TNode<Union<HeapObject, TaggedIndex>> tmp327;
  TNode<IntPtrT> tmp328;
  TNode<IntPtrT> tmp329;
  TNode<IntPtrT> tmp330;
  TNode<IntPtrT> tmp331;
  TNode<UintPtrT> tmp332;
  TNode<UintPtrT> tmp333;
  TNode<BoolT> tmp334;
  if (block307.is_used()) {
    ca_.Bind(&block307, &phi_bb307_7, &phi_bb307_9, &phi_bb307_10, &phi_bb307_12, &phi_bb307_13, &phi_bb307_15, &phi_bb307_16, &phi_bb307_20, &phi_bb307_23);
    std::tie(tmp327, tmp328, tmp329) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb307_9}).Flatten();
    tmp330 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp331 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb307_10}, TNode<IntPtrT>{tmp330});
    tmp332 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb307_10});
    tmp333 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp329});
    tmp334 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp332}, TNode<UintPtrT>{tmp333});
    ca_.Branch(tmp334, &block314, std::vector<compiler::Node*>{phi_bb307_7, phi_bb307_9, phi_bb307_12, phi_bb307_13, phi_bb307_15, phi_bb307_16, phi_bb307_20, phi_bb307_23, phi_bb307_9, phi_bb307_10, phi_bb307_10, phi_bb307_10, phi_bb307_10}, &block315, std::vector<compiler::Node*>{phi_bb307_7, phi_bb307_9, phi_bb307_12, phi_bb307_13, phi_bb307_15, phi_bb307_16, phi_bb307_20, phi_bb307_23, phi_bb307_9, phi_bb307_10, phi_bb307_10, phi_bb307_10, phi_bb307_10});
  }

  TNode<IntPtrT> phi_bb314_7;
  TNode<FixedArray> phi_bb314_9;
  TNode<BoolT> phi_bb314_12;
  TNode<PrimitiveHeapObject> phi_bb314_13;
  TNode<UintPtrT> phi_bb314_15;
  TNode<IntPtrT> phi_bb314_16;
  TNode<IntPtrT> phi_bb314_20;
  TNode<IntPtrT> phi_bb314_23;
  TNode<FixedArray> phi_bb314_28;
  TNode<IntPtrT> phi_bb314_32;
  TNode<IntPtrT> phi_bb314_33;
  TNode<IntPtrT> phi_bb314_37;
  TNode<IntPtrT> phi_bb314_38;
  TNode<IntPtrT> tmp335;
  TNode<IntPtrT> tmp336;
  TNode<Union<HeapObject, TaggedIndex>> tmp337;
  TNode<IntPtrT> tmp338;
  if (block314.is_used()) {
    ca_.Bind(&block314, &phi_bb314_7, &phi_bb314_9, &phi_bb314_12, &phi_bb314_13, &phi_bb314_15, &phi_bb314_16, &phi_bb314_20, &phi_bb314_23, &phi_bb314_28, &phi_bb314_32, &phi_bb314_33, &phi_bb314_37, &phi_bb314_38);
    tmp335 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb314_38});
    tmp336 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp328}, TNode<IntPtrT>{tmp335});
    std::tie(tmp337, tmp338) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp327}, TNode<IntPtrT>{tmp336}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp337, tmp338}, tmp324);
    ca_.Goto(&block309, phi_bb314_7, phi_bb314_9, tmp331, phi_bb314_12, phi_bb314_13, phi_bb314_15, phi_bb314_16, phi_bb314_20, phi_bb314_23);
  }

  TNode<IntPtrT> phi_bb315_7;
  TNode<FixedArray> phi_bb315_9;
  TNode<BoolT> phi_bb315_12;
  TNode<PrimitiveHeapObject> phi_bb315_13;
  TNode<UintPtrT> phi_bb315_15;
  TNode<IntPtrT> phi_bb315_16;
  TNode<IntPtrT> phi_bb315_20;
  TNode<IntPtrT> phi_bb315_23;
  TNode<FixedArray> phi_bb315_28;
  TNode<IntPtrT> phi_bb315_32;
  TNode<IntPtrT> phi_bb315_33;
  TNode<IntPtrT> phi_bb315_37;
  TNode<IntPtrT> phi_bb315_38;
  if (block315.is_used()) {
    ca_.Bind(&block315, &phi_bb315_7, &phi_bb315_9, &phi_bb315_12, &phi_bb315_13, &phi_bb315_15, &phi_bb315_16, &phi_bb315_20, &phi_bb315_23, &phi_bb315_28, &phi_bb315_32, &phi_bb315_33, &phi_bb315_37, &phi_bb315_38);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb308_7;
  TNode<FixedArray> phi_bb308_9;
  TNode<IntPtrT> phi_bb308_10;
  TNode<BoolT> phi_bb308_12;
  TNode<PrimitiveHeapObject> phi_bb308_13;
  TNode<UintPtrT> phi_bb308_15;
  TNode<IntPtrT> phi_bb308_16;
  TNode<IntPtrT> phi_bb308_20;
  TNode<IntPtrT> phi_bb308_23;
  TNode<IntPtrT> tmp339;
  TNode<IntPtrT> tmp340;
  TNode<BoolT> tmp341;
  if (block308.is_used()) {
    ca_.Bind(&block308, &phi_bb308_7, &phi_bb308_9, &phi_bb308_10, &phi_bb308_12, &phi_bb308_13, &phi_bb308_15, &phi_bb308_16, &phi_bb308_20, &phi_bb308_23);
    tmp339 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp325});
    tmp340 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp341 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp339}, TNode<IntPtrT>{tmp340});
    ca_.Branch(tmp341, &block318, std::vector<compiler::Node*>{phi_bb308_7, phi_bb308_9, phi_bb308_10, phi_bb308_12, phi_bb308_13, phi_bb308_15, phi_bb308_16, phi_bb308_20, phi_bb308_23}, &block319, std::vector<compiler::Node*>{phi_bb308_7, phi_bb308_9, phi_bb308_10, phi_bb308_12, phi_bb308_13, phi_bb308_15, phi_bb308_16, phi_bb308_20, phi_bb308_23});
  }

  TNode<IntPtrT> phi_bb318_7;
  TNode<FixedArray> phi_bb318_9;
  TNode<IntPtrT> phi_bb318_10;
  TNode<BoolT> phi_bb318_12;
  TNode<PrimitiveHeapObject> phi_bb318_13;
  TNode<UintPtrT> phi_bb318_15;
  TNode<IntPtrT> phi_bb318_16;
  TNode<IntPtrT> phi_bb318_20;
  TNode<IntPtrT> phi_bb318_23;
  TNode<IntPtrT> tmp342;
  if (block318.is_used()) {
    ca_.Bind(&block318, &phi_bb318_7, &phi_bb318_9, &phi_bb318_10, &phi_bb318_12, &phi_bb318_13, &phi_bb318_15, &phi_bb318_16, &phi_bb318_20, &phi_bb318_23);
    tmp342 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block320, phi_bb318_7, phi_bb318_9, phi_bb318_10, phi_bb318_12, phi_bb318_13, phi_bb318_15, phi_bb318_16, phi_bb318_20, phi_bb318_23, tmp342);
  }

  TNode<IntPtrT> phi_bb319_7;
  TNode<FixedArray> phi_bb319_9;
  TNode<IntPtrT> phi_bb319_10;
  TNode<BoolT> phi_bb319_12;
  TNode<PrimitiveHeapObject> phi_bb319_13;
  TNode<UintPtrT> phi_bb319_15;
  TNode<IntPtrT> phi_bb319_16;
  TNode<IntPtrT> phi_bb319_20;
  TNode<IntPtrT> phi_bb319_23;
  if (block319.is_used()) {
    ca_.Bind(&block319, &phi_bb319_7, &phi_bb319_9, &phi_bb319_10, &phi_bb319_12, &phi_bb319_13, &phi_bb319_15, &phi_bb319_16, &phi_bb319_20, &phi_bb319_23);
    ca_.Goto(&block320, phi_bb319_7, phi_bb319_9, phi_bb319_10, phi_bb319_12, phi_bb319_13, phi_bb319_15, phi_bb319_16, phi_bb319_20, phi_bb319_23, tmp339);
  }

  TNode<IntPtrT> phi_bb320_7;
  TNode<FixedArray> phi_bb320_9;
  TNode<IntPtrT> phi_bb320_10;
  TNode<BoolT> phi_bb320_12;
  TNode<PrimitiveHeapObject> phi_bb320_13;
  TNode<UintPtrT> phi_bb320_15;
  TNode<IntPtrT> phi_bb320_16;
  TNode<IntPtrT> phi_bb320_20;
  TNode<IntPtrT> phi_bb320_23;
  TNode<IntPtrT> phi_bb320_29;
  TNode<FixedArray> tmp343;
  TNode<Union<HeapObject, TaggedIndex>> tmp344;
  TNode<IntPtrT> tmp345;
  TNode<IntPtrT> tmp346;
  TNode<UintPtrT> tmp347;
  TNode<IntPtrT> tmp348;
  TNode<UintPtrT> tmp349;
  TNode<UintPtrT> tmp350;
  TNode<BoolT> tmp351;
  if (block320.is_used()) {
    ca_.Bind(&block320, &phi_bb320_7, &phi_bb320_9, &phi_bb320_10, &phi_bb320_12, &phi_bb320_13, &phi_bb320_15, &phi_bb320_16, &phi_bb320_20, &phi_bb320_23, &phi_bb320_29);
    tmp343 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb320_29});
    std::tie(tmp344, tmp345, tmp346) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb320_9}).Flatten();
    tmp347 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp348 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp347});
    tmp349 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp348});
    tmp350 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp346});
    tmp351 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp349}, TNode<UintPtrT>{tmp350});
    ca_.Branch(tmp351, &block331, std::vector<compiler::Node*>{phi_bb320_7, phi_bb320_9, phi_bb320_10, phi_bb320_12, phi_bb320_13, phi_bb320_15, phi_bb320_16, phi_bb320_20, phi_bb320_23, phi_bb320_9}, &block332, std::vector<compiler::Node*>{phi_bb320_7, phi_bb320_9, phi_bb320_10, phi_bb320_12, phi_bb320_13, phi_bb320_15, phi_bb320_16, phi_bb320_20, phi_bb320_23, phi_bb320_9});
  }

  TNode<IntPtrT> phi_bb331_7;
  TNode<FixedArray> phi_bb331_9;
  TNode<IntPtrT> phi_bb331_10;
  TNode<BoolT> phi_bb331_12;
  TNode<PrimitiveHeapObject> phi_bb331_13;
  TNode<UintPtrT> phi_bb331_15;
  TNode<IntPtrT> phi_bb331_16;
  TNode<IntPtrT> phi_bb331_20;
  TNode<IntPtrT> phi_bb331_23;
  TNode<FixedArray> phi_bb331_31;
  TNode<IntPtrT> tmp352;
  TNode<IntPtrT> tmp353;
  TNode<Union<HeapObject, TaggedIndex>> tmp354;
  TNode<IntPtrT> tmp355;
  TNode<Union<HeapObject, TaggedIndex>> tmp356;
  TNode<IntPtrT> tmp357;
  TNode<IntPtrT> tmp358;
  TNode<UintPtrT> tmp359;
  TNode<IntPtrT> tmp360;
  TNode<UintPtrT> tmp361;
  TNode<UintPtrT> tmp362;
  TNode<BoolT> tmp363;
  if (block331.is_used()) {
    ca_.Bind(&block331, &phi_bb331_7, &phi_bb331_9, &phi_bb331_10, &phi_bb331_12, &phi_bb331_13, &phi_bb331_15, &phi_bb331_16, &phi_bb331_20, &phi_bb331_23, &phi_bb331_31);
    tmp352 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp348});
    tmp353 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp345}, TNode<IntPtrT>{tmp352});
    std::tie(tmp354, tmp355) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp344}, TNode<IntPtrT>{tmp353}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp354, tmp355}, tmp343);
    std::tie(tmp356, tmp357, tmp358) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp343}).Flatten();
    tmp359 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp360 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp359});
    tmp361 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp360});
    tmp362 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp358});
    tmp363 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp361}, TNode<UintPtrT>{tmp362});
    ca_.Branch(tmp363, &block340, std::vector<compiler::Node*>{phi_bb331_7, phi_bb331_9, phi_bb331_10, phi_bb331_12, phi_bb331_13, phi_bb331_15, phi_bb331_16, phi_bb331_20, phi_bb331_23}, &block341, std::vector<compiler::Node*>{phi_bb331_7, phi_bb331_9, phi_bb331_10, phi_bb331_12, phi_bb331_13, phi_bb331_15, phi_bb331_16, phi_bb331_20, phi_bb331_23});
  }

  TNode<IntPtrT> phi_bb332_7;
  TNode<FixedArray> phi_bb332_9;
  TNode<IntPtrT> phi_bb332_10;
  TNode<BoolT> phi_bb332_12;
  TNode<PrimitiveHeapObject> phi_bb332_13;
  TNode<UintPtrT> phi_bb332_15;
  TNode<IntPtrT> phi_bb332_16;
  TNode<IntPtrT> phi_bb332_20;
  TNode<IntPtrT> phi_bb332_23;
  TNode<FixedArray> phi_bb332_31;
  if (block332.is_used()) {
    ca_.Bind(&block332, &phi_bb332_7, &phi_bb332_9, &phi_bb332_10, &phi_bb332_12, &phi_bb332_13, &phi_bb332_15, &phi_bb332_16, &phi_bb332_20, &phi_bb332_23, &phi_bb332_31);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb340_7;
  TNode<FixedArray> phi_bb340_9;
  TNode<IntPtrT> phi_bb340_10;
  TNode<BoolT> phi_bb340_12;
  TNode<PrimitiveHeapObject> phi_bb340_13;
  TNode<UintPtrT> phi_bb340_15;
  TNode<IntPtrT> phi_bb340_16;
  TNode<IntPtrT> phi_bb340_20;
  TNode<IntPtrT> phi_bb340_23;
  TNode<IntPtrT> tmp364;
  TNode<IntPtrT> tmp365;
  TNode<Union<HeapObject, TaggedIndex>> tmp366;
  TNode<IntPtrT> tmp367;
  TNode<Undefined> tmp368;
  TNode<Union<HeapObject, TaggedIndex>> tmp369;
  TNode<IntPtrT> tmp370;
  TNode<IntPtrT> tmp371;
  TNode<UintPtrT> tmp372;
  TNode<IntPtrT> tmp373;
  TNode<UintPtrT> tmp374;
  TNode<UintPtrT> tmp375;
  TNode<BoolT> tmp376;
  if (block340.is_used()) {
    ca_.Bind(&block340, &phi_bb340_7, &phi_bb340_9, &phi_bb340_10, &phi_bb340_12, &phi_bb340_13, &phi_bb340_15, &phi_bb340_16, &phi_bb340_20, &phi_bb340_23);
    tmp364 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp360});
    tmp365 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp357}, TNode<IntPtrT>{tmp364});
    std::tie(tmp366, tmp367) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp356}, TNode<IntPtrT>{tmp365}).Flatten();
    tmp368 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp366, tmp367}, tmp368);
    std::tie(tmp369, tmp370, tmp371) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp343}).Flatten();
    tmp372 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp373 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp372});
    tmp374 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp373});
    tmp375 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp371});
    tmp376 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp374}, TNode<UintPtrT>{tmp375});
    ca_.Branch(tmp376, &block349, std::vector<compiler::Node*>{phi_bb340_7, phi_bb340_9, phi_bb340_10, phi_bb340_12, phi_bb340_13, phi_bb340_15, phi_bb340_16, phi_bb340_20, phi_bb340_23}, &block350, std::vector<compiler::Node*>{phi_bb340_7, phi_bb340_9, phi_bb340_10, phi_bb340_12, phi_bb340_13, phi_bb340_15, phi_bb340_16, phi_bb340_20, phi_bb340_23});
  }

  TNode<IntPtrT> phi_bb341_7;
  TNode<FixedArray> phi_bb341_9;
  TNode<IntPtrT> phi_bb341_10;
  TNode<BoolT> phi_bb341_12;
  TNode<PrimitiveHeapObject> phi_bb341_13;
  TNode<UintPtrT> phi_bb341_15;
  TNode<IntPtrT> phi_bb341_16;
  TNode<IntPtrT> phi_bb341_20;
  TNode<IntPtrT> phi_bb341_23;
  if (block341.is_used()) {
    ca_.Bind(&block341, &phi_bb341_7, &phi_bb341_9, &phi_bb341_10, &phi_bb341_12, &phi_bb341_13, &phi_bb341_15, &phi_bb341_16, &phi_bb341_20, &phi_bb341_23);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb349_7;
  TNode<FixedArray> phi_bb349_9;
  TNode<IntPtrT> phi_bb349_10;
  TNode<BoolT> phi_bb349_12;
  TNode<PrimitiveHeapObject> phi_bb349_13;
  TNode<UintPtrT> phi_bb349_15;
  TNode<IntPtrT> phi_bb349_16;
  TNode<IntPtrT> phi_bb349_20;
  TNode<IntPtrT> phi_bb349_23;
  TNode<IntPtrT> tmp377;
  TNode<IntPtrT> tmp378;
  TNode<Union<HeapObject, TaggedIndex>> tmp379;
  TNode<IntPtrT> tmp380;
  TNode<IntPtrT> tmp381;
  if (block349.is_used()) {
    ca_.Bind(&block349, &phi_bb349_7, &phi_bb349_9, &phi_bb349_10, &phi_bb349_12, &phi_bb349_13, &phi_bb349_15, &phi_bb349_16, &phi_bb349_20, &phi_bb349_23);
    tmp377 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp373});
    tmp378 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp370}, TNode<IntPtrT>{tmp377});
    std::tie(tmp379, tmp380) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp369}, TNode<IntPtrT>{tmp378}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp379, tmp380}, tmp324);
    tmp381 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block309, phi_bb349_7, tmp343, tmp381, phi_bb349_12, phi_bb349_13, phi_bb349_15, phi_bb349_16, phi_bb349_20, phi_bb349_23);
  }

  TNode<IntPtrT> phi_bb350_7;
  TNode<FixedArray> phi_bb350_9;
  TNode<IntPtrT> phi_bb350_10;
  TNode<BoolT> phi_bb350_12;
  TNode<PrimitiveHeapObject> phi_bb350_13;
  TNode<UintPtrT> phi_bb350_15;
  TNode<IntPtrT> phi_bb350_16;
  TNode<IntPtrT> phi_bb350_20;
  TNode<IntPtrT> phi_bb350_23;
  if (block350.is_used()) {
    ca_.Bind(&block350, &phi_bb350_7, &phi_bb350_9, &phi_bb350_10, &phi_bb350_12, &phi_bb350_13, &phi_bb350_15, &phi_bb350_16, &phi_bb350_20, &phi_bb350_23);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb309_7;
  TNode<FixedArray> phi_bb309_9;
  TNode<IntPtrT> phi_bb309_10;
  TNode<BoolT> phi_bb309_12;
  TNode<PrimitiveHeapObject> phi_bb309_13;
  TNode<UintPtrT> phi_bb309_15;
  TNode<IntPtrT> phi_bb309_16;
  TNode<IntPtrT> phi_bb309_20;
  TNode<IntPtrT> phi_bb309_23;
  TNode<Null> tmp382;
  if (block309.is_used()) {
    ca_.Bind(&block309, &phi_bb309_7, &phi_bb309_9, &phi_bb309_10, &phi_bb309_12, &phi_bb309_13, &phi_bb309_15, &phi_bb309_16, &phi_bb309_20, &phi_bb309_23);
    tmp382 = Null_0(state_);
    ca_.Goto(&block297, phi_bb309_7, phi_bb309_9, phi_bb309_10, phi_bb309_12, tmp382, phi_bb309_15, phi_bb309_16, phi_bb309_20, phi_bb309_23);
  }

  TNode<IntPtrT> phi_bb297_7;
  TNode<FixedArray> phi_bb297_9;
  TNode<IntPtrT> phi_bb297_10;
  TNode<BoolT> phi_bb297_12;
  TNode<PrimitiveHeapObject> phi_bb297_13;
  TNode<UintPtrT> phi_bb297_15;
  TNode<IntPtrT> phi_bb297_16;
  TNode<IntPtrT> phi_bb297_20;
  TNode<IntPtrT> phi_bb297_23;
  if (block297.is_used()) {
    ca_.Bind(&block297, &phi_bb297_7, &phi_bb297_9, &phi_bb297_10, &phi_bb297_12, &phi_bb297_13, &phi_bb297_15, &phi_bb297_16, &phi_bb297_20, &phi_bb297_23);
    ca_.Goto(&block288, phi_bb297_7, phi_bb297_9, phi_bb297_10, tmp323, phi_bb297_12, phi_bb297_13, phi_bb297_15, phi_bb297_16, phi_bb297_20);
  }

  TNode<IntPtrT> phi_bb288_7;
  TNode<FixedArray> phi_bb288_9;
  TNode<IntPtrT> phi_bb288_10;
  TNode<IntPtrT> phi_bb288_11;
  TNode<BoolT> phi_bb288_12;
  TNode<PrimitiveHeapObject> phi_bb288_13;
  TNode<UintPtrT> phi_bb288_15;
  TNode<IntPtrT> phi_bb288_16;
  TNode<IntPtrT> phi_bb288_20;
  TNode<String> tmp383;
  if (block288.is_used()) {
    ca_.Bind(&block288, &phi_bb288_7, &phi_bb288_9, &phi_bb288_10, &phi_bb288_11, &phi_bb288_12, &phi_bb288_13, &phi_bb288_15, &phi_bb288_16, &phi_bb288_20);
    tmp383 = BufferJoin_0(state_, TNode<Context>{p_context}, TorqueStructBuffer_0{TNode<FixedArray>{tmp5}, TNode<FixedArray>{phi_bb288_9}, TNode<IntPtrT>{phi_bb288_10}, TNode<IntPtrT>{phi_bb288_11}, TNode<BoolT>{phi_bb288_12}, TNode<PrimitiveHeapObject>{phi_bb288_13}}, TNode<String>{p_sep});
    ca_.Goto(&block353);
  }

    ca_.Bind(&block353);
  return TNode<String>{tmp383};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=516&c=1
TNode<JSAny> ArrayJoin_JSArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, bool p_useToLocaleString, TNode<JSReceiver> p_receiver, TNode<String> p_sep, TNode<Number> p_lenNumber, TNode<JSAny> p_locales, TNode<JSAny> p_options) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Map> tmp1;
  TNode<Int32T> tmp2;
  TNode<JSArray> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp1 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{p_receiver, tmp0});
    tmp2 = CodeStubAssembler(state_).LoadMapElementsKind(TNode<Map>{tmp1});
    compiler::CodeAssemblerLabel label4(&ca_);
    tmp3 = Cast_JSArray_0(state_, TNode<HeapObject>{p_receiver}, &label4);
    ca_.Goto(&block4);
    if (label4.is_used()) {
      ca_.Bind(&label4);
      ca_.Goto(&block5);
    }
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block3);
  }

  TNode<IntPtrT> tmp5;
  TNode<Number> tmp6;
  TNode<BoolT> tmp7;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp5 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp6 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{tmp3, tmp5});
    tmp7 = IsNumberNotEqual_0(state_, TNode<Number>{tmp6}, TNode<Number>{p_lenNumber});
    ca_.Branch(tmp7, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    ca_.Goto(&block3);
  }

  TNode<BoolT> tmp8;
  TNode<BoolT> tmp9;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp8 = CodeStubAssembler(state_).IsPrototypeInitialArrayPrototype(TNode<Context>{p_context}, TNode<Map>{tmp1});
    tmp9 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp8});
    ca_.Branch(tmp9, &block8, std::vector<compiler::Node*>{}, &block9, std::vector<compiler::Node*>{});
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    ca_.Goto(&block3);
  }

  TNode<BoolT> tmp10;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp10 = CodeStubAssembler(state_).IsNoElementsProtectorCellInvalid();
    ca_.Branch(tmp10, &block10, std::vector<compiler::Node*>{}, &block11, std::vector<compiler::Node*>{});
  }

  if (block10.is_used()) {
    ca_.Bind(&block10);
    ca_.Goto(&block3);
  }

  TNode<BoolT> tmp11;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp11 = CodeStubAssembler(state_).IsElementsKindLessThanOrEqual(TNode<Int32T>{tmp2}, CastIfEnumClass<ElementsKind>(ElementsKind::HOLEY_ELEMENTS));
    ca_.Branch(tmp11, &block12, std::vector<compiler::Node*>{}, &block13, std::vector<compiler::Node*>{});
  }

  if (block12.is_used()) {
    ca_.Bind(&block12);
    ca_.Goto(&block14, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinElement_FastSmiOrObjectElements_0)));
  }

  TNode<BoolT> tmp12;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp12 = CodeStubAssembler(state_).IsElementsKindLessThanOrEqual(TNode<Int32T>{tmp2}, CastIfEnumClass<ElementsKind>(ElementsKind::HOLEY_DOUBLE_ELEMENTS));
    ca_.Branch(tmp12, &block15, std::vector<compiler::Node*>{}, &block16, std::vector<compiler::Node*>{});
  }

  if (block15.is_used()) {
    ca_.Bind(&block15);
    ca_.Goto(&block17, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinElement_FastDoubleElements_0)));
  }

  TNode<BoolT> tmp13;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp13 = CodeStubAssembler(state_).IsElementsKindLessThanOrEqual(TNode<Int32T>{tmp2}, CastIfEnumClass<ElementsKind>(ElementsKind::LAST_ANY_NONEXTENSIBLE_ELEMENTS_KIND));
    ca_.Branch(tmp13, &block18, std::vector<compiler::Node*>{}, &block19, std::vector<compiler::Node*>{});
  }

  if (block18.is_used()) {
    ca_.Bind(&block18);
    ca_.Goto(&block20, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinElement_FastSmiOrObjectElements_0)));
  }

  TNode<Int32T> tmp14;
  TNode<BoolT> tmp15;
  if (block19.is_used()) {
    ca_.Bind(&block19);
    tmp14 = FromConstexpr_ElementsKind_constexpr_DICTIONARY_ELEMENTS_0(state_, ElementsKind::DICTIONARY_ELEMENTS);
    tmp15 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp14});
    ca_.Branch(tmp15, &block21, std::vector<compiler::Node*>{}, &block22, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp16;
  TNode<FixedArrayBase> tmp17;
  TNode<NumberDictionary> tmp18;
  TNode<Smi> tmp19;
  TNode<Smi> tmp20;
  TNode<BoolT> tmp21;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp16 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp17 = CodeStubAssembler(state_).LoadReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp3, tmp16});
    tmp18 = UnsafeCast_NumberDictionary_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp17});
    tmp19 = CodeStubAssembler(state_).GetNumberDictionaryNumberOfElements(TNode<NumberDictionary>{tmp18});
    tmp20 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp21 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp19}, TNode<Smi>{tmp20});
    ca_.Branch(tmp21, &block24, std::vector<compiler::Node*>{}, &block25, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp22;
  TNode<BoolT> tmp23;
  if (block24.is_used()) {
    ca_.Bind(&block24);
    tmp22 = kEmptyString_0(state_);
    tmp23 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{p_sep}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp22});
    ca_.Branch(tmp23, &block27, std::vector<compiler::Node*>{}, &block28, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp24;
  if (block27.is_used()) {
    ca_.Bind(&block27);
    tmp24 = kEmptyString_0(state_);
    ca_.Goto(&block1, tmp24);
  }

  TNode<Number> tmp25;
  TNode<Number> tmp26;
  TNode<Smi> tmp27;
  if (block28.is_used()) {
    ca_.Bind(&block28);
    tmp25 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp26 = CodeStubAssembler(state_).NumberSub(TNode<Number>{p_lenNumber}, TNode<Number>{tmp25});
    compiler::CodeAssemblerLabel label28(&ca_);
    tmp27 = Cast_Smi_0(state_, TNode<Object>{tmp26}, &label28);
    ca_.Goto(&block31);
    if (label28.is_used()) {
      ca_.Bind(&label28);
      ca_.Goto(&block32);
    }
  }

  if (block32.is_used()) {
    ca_.Bind(&block32);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowInvalidStringLength, p_context);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> tmp29;
  if (block31.is_used()) {
    ca_.Bind(&block31);
    tmp29 = ca_.CallBuiltin<String>(Builtin::kStringRepeat, p_context, p_sep, tmp27);
    ca_.Goto(&block1, tmp29);
  }

  if (block25.is_used()) {
    ca_.Bind(&block25);
    ca_.Goto(&block20, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinElement_DictionaryElements_0)));
  }

  if (block22.is_used()) {
    ca_.Bind(&block22);
    ca_.Goto(&block3);
  }

  TNode<BuiltinPtr> phi_bb20_8;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_8);
    ca_.Goto(&block17, phi_bb20_8);
  }

  TNode<BuiltinPtr> phi_bb17_8;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_8);
    ca_.Goto(&block14, phi_bb17_8);
  }

  TNode<BuiltinPtr> phi_bb14_8;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_8);
    ca_.Goto(&block2, phi_bb14_8);
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block2, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinElement_GenericElementsAccessor_0)));
  }

  TNode<BuiltinPtr> phi_bb2_8;
  TNode<String> tmp30;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_8);
    tmp30 = ArrayJoinImpl_JSArray_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{p_receiver}, TNode<String>{p_sep}, TNode<Number>{p_lenNumber}, p_useToLocaleString, TNode<JSAny>{p_locales}, TNode<JSAny>{p_options}, TNode<BuiltinPtr>{phi_bb2_8});
    ca_.Goto(&block1, tmp30);
  }

  TNode<JSAny> phi_bb1_6;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_6);
    ca_.Goto(&block33, phi_bb1_6);
  }

  TNode<JSAny> phi_bb33_6;
    ca_.Bind(&block33, &phi_bb33_6);
  return TNode<JSAny>{phi_bb33_6};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=566&c=1
TNode<JSAny> ArrayJoin_JSTypedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, bool p_useToLocaleString, TNode<JSReceiver> p_receiver, TNode<String> p_sep, TNode<Number> p_lenNumber, TNode<JSAny> p_locales, TNode<JSAny> p_options) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block32(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block42(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block44(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block45(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block47(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block48(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block50(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block51(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block53(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block54(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block60(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block52(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block49(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block31(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block62(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block63(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block66(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block71(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block72(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block74(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block75(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block73(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block70(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block67(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block64(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BuiltinPtr> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block77(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Map> tmp1;
  TNode<Int32T> tmp2;
  TNode<BoolT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp1 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{p_receiver, tmp0});
    tmp2 = CodeStubAssembler(state_).LoadMapElementsKind(TNode<Map>{tmp1});
    tmp3 = CodeStubAssembler(state_).IsElementsKindGreaterThan(TNode<Int32T>{tmp2}, CastIfEnumClass<ElementsKind>(ElementsKind::UINT32_ELEMENTS));
    ca_.Branch(tmp3, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<Int32T> tmp4;
  TNode<BoolT> tmp5;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp4 = FromConstexpr_ElementsKind_constexpr_INT32_ELEMENTS_0(state_, ElementsKind::INT32_ELEMENTS);
    tmp5 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp4});
    ca_.Branch(tmp5, &block5, std::vector<compiler::Node*>{}, &block6, std::vector<compiler::Node*>{});
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block7, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Int32Elements_0)));
  }

  TNode<Int32T> tmp6;
  TNode<BoolT> tmp7;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp6 = FromConstexpr_ElementsKind_constexpr_FLOAT16_ELEMENTS_0(state_, ElementsKind::FLOAT16_ELEMENTS);
    tmp7 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp6});
    ca_.Branch(tmp7, &block8, std::vector<compiler::Node*>{}, &block9, std::vector<compiler::Node*>{});
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    ca_.Goto(&block10, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Float16Elements_0)));
  }

  TNode<Int32T> tmp8;
  TNode<BoolT> tmp9;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp8 = FromConstexpr_ElementsKind_constexpr_FLOAT32_ELEMENTS_0(state_, ElementsKind::FLOAT32_ELEMENTS);
    tmp9 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp8});
    ca_.Branch(tmp9, &block11, std::vector<compiler::Node*>{}, &block12, std::vector<compiler::Node*>{});
  }

  if (block11.is_used()) {
    ca_.Bind(&block11);
    ca_.Goto(&block13, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Float32Elements_0)));
  }

  TNode<Int32T> tmp10;
  TNode<BoolT> tmp11;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp10 = FromConstexpr_ElementsKind_constexpr_FLOAT64_ELEMENTS_0(state_, ElementsKind::FLOAT64_ELEMENTS);
    tmp11 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp10});
    ca_.Branch(tmp11, &block14, std::vector<compiler::Node*>{}, &block15, std::vector<compiler::Node*>{});
  }

  if (block14.is_used()) {
    ca_.Bind(&block14);
    ca_.Goto(&block16, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Float64Elements_0)));
  }

  TNode<Int32T> tmp12;
  TNode<BoolT> tmp13;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp12 = FromConstexpr_ElementsKind_constexpr_UINT8_CLAMPED_ELEMENTS_0(state_, ElementsKind::UINT8_CLAMPED_ELEMENTS);
    tmp13 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp12});
    ca_.Branch(tmp13, &block17, std::vector<compiler::Node*>{}, &block18, std::vector<compiler::Node*>{});
  }

  if (block17.is_used()) {
    ca_.Bind(&block17);
    ca_.Goto(&block19, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Uint8ClampedElements_0)));
  }

  TNode<Int32T> tmp14;
  TNode<BoolT> tmp15;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp14 = FromConstexpr_ElementsKind_constexpr_BIGUINT64_ELEMENTS_0(state_, ElementsKind::BIGUINT64_ELEMENTS);
    tmp15 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp14});
    ca_.Branch(tmp15, &block20, std::vector<compiler::Node*>{}, &block21, std::vector<compiler::Node*>{});
  }

  if (block20.is_used()) {
    ca_.Bind(&block20);
    ca_.Goto(&block22, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_BigUint64Elements_0)));
  }

  TNode<Int32T> tmp16;
  TNode<BoolT> tmp17;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp16 = FromConstexpr_ElementsKind_constexpr_BIGINT64_ELEMENTS_0(state_, ElementsKind::BIGINT64_ELEMENTS);
    tmp17 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp16});
    ca_.Branch(tmp17, &block23, std::vector<compiler::Node*>{}, &block24, std::vector<compiler::Node*>{});
  }

  if (block23.is_used()) {
    ca_.Bind(&block23);
    ca_.Goto(&block25, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_BigInt64Elements_0)));
  }

  TNode<Int32T> tmp18;
  TNode<BoolT> tmp19;
  if (block24.is_used()) {
    ca_.Bind(&block24);
    tmp18 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_UINT8_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_UINT8_ELEMENTS);
    tmp19 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp18});
    ca_.Branch(tmp19, &block26, std::vector<compiler::Node*>{}, &block27, std::vector<compiler::Node*>{});
  }

  if (block26.is_used()) {
    ca_.Bind(&block26);
    ca_.Goto(&block28, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Uint8Elements_0)));
  }

  TNode<Int32T> tmp20;
  TNode<BoolT> tmp21;
  if (block27.is_used()) {
    ca_.Bind(&block27);
    tmp20 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_INT8_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_INT8_ELEMENTS);
    tmp21 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp20});
    ca_.Branch(tmp21, &block29, std::vector<compiler::Node*>{}, &block30, std::vector<compiler::Node*>{});
  }

  if (block29.is_used()) {
    ca_.Bind(&block29);
    ca_.Goto(&block31, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Int8Elements_0)));
  }

  TNode<Int32T> tmp22;
  TNode<BoolT> tmp23;
  if (block30.is_used()) {
    ca_.Bind(&block30);
    tmp22 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_UINT16_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_UINT16_ELEMENTS);
    tmp23 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp22});
    ca_.Branch(tmp23, &block32, std::vector<compiler::Node*>{}, &block33, std::vector<compiler::Node*>{});
  }

  if (block32.is_used()) {
    ca_.Bind(&block32);
    ca_.Goto(&block34, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Uint16Elements_0)));
  }

  TNode<Int32T> tmp24;
  TNode<BoolT> tmp25;
  if (block33.is_used()) {
    ca_.Bind(&block33);
    tmp24 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_INT16_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_INT16_ELEMENTS);
    tmp25 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp24});
    ca_.Branch(tmp25, &block35, std::vector<compiler::Node*>{}, &block36, std::vector<compiler::Node*>{});
  }

  if (block35.is_used()) {
    ca_.Bind(&block35);
    ca_.Goto(&block37, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Int16Elements_0)));
  }

  TNode<Int32T> tmp26;
  TNode<BoolT> tmp27;
  if (block36.is_used()) {
    ca_.Bind(&block36);
    tmp26 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_UINT32_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_UINT32_ELEMENTS);
    tmp27 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp26});
    ca_.Branch(tmp27, &block38, std::vector<compiler::Node*>{}, &block39, std::vector<compiler::Node*>{});
  }

  if (block38.is_used()) {
    ca_.Bind(&block38);
    ca_.Goto(&block40, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Uint32Elements_0)));
  }

  TNode<Int32T> tmp28;
  TNode<BoolT> tmp29;
  if (block39.is_used()) {
    ca_.Bind(&block39);
    tmp28 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_INT32_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_INT32_ELEMENTS);
    tmp29 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp28});
    ca_.Branch(tmp29, &block41, std::vector<compiler::Node*>{}, &block42, std::vector<compiler::Node*>{});
  }

  if (block41.is_used()) {
    ca_.Bind(&block41);
    ca_.Goto(&block43, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Int32Elements_0)));
  }

  TNode<Int32T> tmp30;
  TNode<BoolT> tmp31;
  if (block42.is_used()) {
    ca_.Bind(&block42);
    tmp30 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_FLOAT16_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_FLOAT16_ELEMENTS);
    tmp31 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp30});
    ca_.Branch(tmp31, &block44, std::vector<compiler::Node*>{}, &block45, std::vector<compiler::Node*>{});
  }

  if (block44.is_used()) {
    ca_.Bind(&block44);
    ca_.Goto(&block46, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Float16Elements_0)));
  }

  TNode<Int32T> tmp32;
  TNode<BoolT> tmp33;
  if (block45.is_used()) {
    ca_.Bind(&block45);
    tmp32 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_FLOAT32_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_FLOAT32_ELEMENTS);
    tmp33 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp32});
    ca_.Branch(tmp33, &block47, std::vector<compiler::Node*>{}, &block48, std::vector<compiler::Node*>{});
  }

  if (block47.is_used()) {
    ca_.Bind(&block47);
    ca_.Goto(&block49, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Float32Elements_0)));
  }

  TNode<Int32T> tmp34;
  TNode<BoolT> tmp35;
  if (block48.is_used()) {
    ca_.Bind(&block48);
    tmp34 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_FLOAT64_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_FLOAT64_ELEMENTS);
    tmp35 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp34});
    ca_.Branch(tmp35, &block50, std::vector<compiler::Node*>{}, &block51, std::vector<compiler::Node*>{});
  }

  if (block50.is_used()) {
    ca_.Bind(&block50);
    ca_.Goto(&block52, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Float64Elements_0)));
  }

  TNode<Int32T> tmp36;
  TNode<BoolT> tmp37;
  if (block51.is_used()) {
    ca_.Bind(&block51);
    tmp36 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_UINT8_CLAMPED_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_UINT8_CLAMPED_ELEMENTS);
    tmp37 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp36});
    ca_.Branch(tmp37, &block53, std::vector<compiler::Node*>{}, &block54, std::vector<compiler::Node*>{});
  }

  if (block53.is_used()) {
    ca_.Bind(&block53);
    ca_.Goto(&block55, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Uint8ClampedElements_0)));
  }

  TNode<Int32T> tmp38;
  TNode<BoolT> tmp39;
  if (block54.is_used()) {
    ca_.Bind(&block54);
    tmp38 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_BIGUINT64_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_BIGUINT64_ELEMENTS);
    tmp39 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp38});
    ca_.Branch(tmp39, &block56, std::vector<compiler::Node*>{}, &block57, std::vector<compiler::Node*>{});
  }

  if (block56.is_used()) {
    ca_.Bind(&block56);
    ca_.Goto(&block58, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_BigUint64Elements_0)));
  }

  TNode<Int32T> tmp40;
  TNode<BoolT> tmp41;
  if (block57.is_used()) {
    ca_.Bind(&block57);
    tmp40 = FromConstexpr_ElementsKind_constexpr_RAB_GSAB_BIGINT64_ELEMENTS_0(state_, ElementsKind::RAB_GSAB_BIGINT64_ELEMENTS);
    tmp41 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp40});
    ca_.Branch(tmp41, &block59, std::vector<compiler::Node*>{}, &block60, std::vector<compiler::Node*>{});
  }

  if (block59.is_used()) {
    ca_.Bind(&block59);
    ca_.Goto(&block58, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_BigInt64Elements_0)));
  }

  if (block60.is_used()) {
    ca_.Bind(&block60);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BuiltinPtr> phi_bb58_8;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_8);
    ca_.Goto(&block55, phi_bb58_8);
  }

  TNode<BuiltinPtr> phi_bb55_8;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_8);
    ca_.Goto(&block52, phi_bb55_8);
  }

  TNode<BuiltinPtr> phi_bb52_8;
  if (block52.is_used()) {
    ca_.Bind(&block52, &phi_bb52_8);
    ca_.Goto(&block49, phi_bb52_8);
  }

  TNode<BuiltinPtr> phi_bb49_8;
  if (block49.is_used()) {
    ca_.Bind(&block49, &phi_bb49_8);
    ca_.Goto(&block46, phi_bb49_8);
  }

  TNode<BuiltinPtr> phi_bb46_8;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_8);
    ca_.Goto(&block43, phi_bb46_8);
  }

  TNode<BuiltinPtr> phi_bb43_8;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_8);
    ca_.Goto(&block40, phi_bb43_8);
  }

  TNode<BuiltinPtr> phi_bb40_8;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_8);
    ca_.Goto(&block37, phi_bb40_8);
  }

  TNode<BuiltinPtr> phi_bb37_8;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_8);
    ca_.Goto(&block34, phi_bb37_8);
  }

  TNode<BuiltinPtr> phi_bb34_8;
  if (block34.is_used()) {
    ca_.Bind(&block34, &phi_bb34_8);
    ca_.Goto(&block31, phi_bb34_8);
  }

  TNode<BuiltinPtr> phi_bb31_8;
  if (block31.is_used()) {
    ca_.Bind(&block31, &phi_bb31_8);
    ca_.Goto(&block28, phi_bb31_8);
  }

  TNode<BuiltinPtr> phi_bb28_8;
  if (block28.is_used()) {
    ca_.Bind(&block28, &phi_bb28_8);
    ca_.Goto(&block25, phi_bb28_8);
  }

  TNode<BuiltinPtr> phi_bb25_8;
  if (block25.is_used()) {
    ca_.Bind(&block25, &phi_bb25_8);
    ca_.Goto(&block22, phi_bb25_8);
  }

  TNode<BuiltinPtr> phi_bb22_8;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_8);
    ca_.Goto(&block19, phi_bb22_8);
  }

  TNode<BuiltinPtr> phi_bb19_8;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_8);
    ca_.Goto(&block16, phi_bb19_8);
  }

  TNode<BuiltinPtr> phi_bb16_8;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_8);
    ca_.Goto(&block13, phi_bb16_8);
  }

  TNode<BuiltinPtr> phi_bb13_8;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_8);
    ca_.Goto(&block10, phi_bb13_8);
  }

  TNode<BuiltinPtr> phi_bb10_8;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_8);
    ca_.Goto(&block7, phi_bb10_8);
  }

  TNode<BuiltinPtr> phi_bb7_8;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_8);
    ca_.Goto(&block4, phi_bb7_8);
  }

  TNode<Int32T> tmp42;
  TNode<BoolT> tmp43;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp42 = FromConstexpr_ElementsKind_constexpr_UINT8_ELEMENTS_0(state_, ElementsKind::UINT8_ELEMENTS);
    tmp43 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp42});
    ca_.Branch(tmp43, &block62, std::vector<compiler::Node*>{}, &block63, std::vector<compiler::Node*>{});
  }

  if (block62.is_used()) {
    ca_.Bind(&block62);
    ca_.Goto(&block64, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Uint8Elements_0)));
  }

  TNode<Int32T> tmp44;
  TNode<BoolT> tmp45;
  if (block63.is_used()) {
    ca_.Bind(&block63);
    tmp44 = FromConstexpr_ElementsKind_constexpr_INT8_ELEMENTS_0(state_, ElementsKind::INT8_ELEMENTS);
    tmp45 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp44});
    ca_.Branch(tmp45, &block65, std::vector<compiler::Node*>{}, &block66, std::vector<compiler::Node*>{});
  }

  if (block65.is_used()) {
    ca_.Bind(&block65);
    ca_.Goto(&block67, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Int8Elements_0)));
  }

  TNode<Int32T> tmp46;
  TNode<BoolT> tmp47;
  if (block66.is_used()) {
    ca_.Bind(&block66);
    tmp46 = FromConstexpr_ElementsKind_constexpr_UINT16_ELEMENTS_0(state_, ElementsKind::UINT16_ELEMENTS);
    tmp47 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp46});
    ca_.Branch(tmp47, &block68, std::vector<compiler::Node*>{}, &block69, std::vector<compiler::Node*>{});
  }

  if (block68.is_used()) {
    ca_.Bind(&block68);
    ca_.Goto(&block70, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Uint16Elements_0)));
  }

  TNode<Int32T> tmp48;
  TNode<BoolT> tmp49;
  if (block69.is_used()) {
    ca_.Bind(&block69);
    tmp48 = FromConstexpr_ElementsKind_constexpr_INT16_ELEMENTS_0(state_, ElementsKind::INT16_ELEMENTS);
    tmp49 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp48});
    ca_.Branch(tmp49, &block71, std::vector<compiler::Node*>{}, &block72, std::vector<compiler::Node*>{});
  }

  if (block71.is_used()) {
    ca_.Bind(&block71);
    ca_.Goto(&block73, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Int16Elements_0)));
  }

  TNode<Int32T> tmp50;
  TNode<BoolT> tmp51;
  if (block72.is_used()) {
    ca_.Bind(&block72);
    tmp50 = FromConstexpr_ElementsKind_constexpr_UINT32_ELEMENTS_0(state_, ElementsKind::UINT32_ELEMENTS);
    tmp51 = CodeStubAssembler(state_).ElementsKindEqual(TNode<Int32T>{tmp2}, TNode<Int32T>{tmp50});
    ca_.Branch(tmp51, &block74, std::vector<compiler::Node*>{}, &block75, std::vector<compiler::Node*>{});
  }

  if (block74.is_used()) {
    ca_.Bind(&block74);
    ca_.Goto(&block73, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinTypedElement_Uint32Elements_0)));
  }

  if (block75.is_used()) {
    ca_.Bind(&block75);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BuiltinPtr> phi_bb73_8;
  if (block73.is_used()) {
    ca_.Bind(&block73, &phi_bb73_8);
    ca_.Goto(&block70, phi_bb73_8);
  }

  TNode<BuiltinPtr> phi_bb70_8;
  if (block70.is_used()) {
    ca_.Bind(&block70, &phi_bb70_8);
    ca_.Goto(&block67, phi_bb70_8);
  }

  TNode<BuiltinPtr> phi_bb67_8;
  if (block67.is_used()) {
    ca_.Bind(&block67, &phi_bb67_8);
    ca_.Goto(&block64, phi_bb67_8);
  }

  TNode<BuiltinPtr> phi_bb64_8;
  if (block64.is_used()) {
    ca_.Bind(&block64, &phi_bb64_8);
    ca_.Goto(&block4, phi_bb64_8);
  }

  TNode<BuiltinPtr> phi_bb4_8;
  TNode<String> tmp52;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_8);
    tmp52 = ArrayJoinImpl_JSTypedArray_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{p_receiver}, TNode<String>{p_sep}, TNode<Number>{p_lenNumber}, p_useToLocaleString, TNode<JSAny>{p_locales}, TNode<JSAny>{p_options}, TNode<BuiltinPtr>{phi_bb4_8});
    ca_.Goto(&block77);
  }

    ca_.Bind(&block77);
  return TNode<JSAny>{tmp52};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=642&c=1
TNode<FixedArray> LoadJoinStack_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, compiler::CodeAssemblerLabel* label_IfUninitialized) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Union<FixedArray, Undefined>> tmp3;
  TNode<Undefined> tmp4;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ARRAY_JOIN_STACK_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Undefined_OR_FixedArray_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Union<FixedArray, Undefined>>(CodeStubAssembler::Reference{tmp1, tmp2});
    compiler::CodeAssemblerLabel label5(&ca_);
    tmp4 = Cast_Undefined_2(state_, TNode<HeapObject>{tmp3}, &label5);
    ca_.Goto(&block5);
    if (label5.is_used()) {
      ca_.Bind(&label5);
      ca_.Goto(&block6);
    }
  }

  if (block6.is_used()) {
    ca_.Bind(&block6);
    ca_.Goto(&block7);
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(label_IfUninitialized);
  }

    ca_.Bind(&block7);
  return TNode<FixedArray>{ca_.UncheckedCast<FixedArray>(tmp3)};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=654&c=1
void SetJoinStack_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<FixedArray> p_stack) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ARRAY_JOIN_STACK_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_Undefined_OR_FixedArray_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    CodeStubAssembler(state_).StoreReference<Union<FixedArray, Undefined>>(CodeStubAssembler::Reference{tmp1, tmp2}, p_stack);
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
}

TF_BUILTIN(JoinStackPush, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<FixedArray> parameter1 = UncheckedParameter<FixedArray>(Descriptor::kStack);
  USE(parameter1);
  TNode<JSReceiver> parameter2 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{parameter1});
    tmp1 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block3, tmp1);
  }

  TNode<IntPtrT> phi_bb3_4;
  TNode<BoolT> tmp2;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_4);
    tmp2 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb3_4}, TNode<IntPtrT>{tmp0});
    ca_.Branch(tmp2, &block1, std::vector<compiler::Node*>{phi_bb3_4}, &block2, std::vector<compiler::Node*>{phi_bb3_4});
  }

  TNode<IntPtrT> phi_bb1_4;
  TNode<Union<HeapObject, TaggedIndex>> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<UintPtrT> tmp6;
  TNode<UintPtrT> tmp7;
  TNode<BoolT> tmp8;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_4);
    std::tie(tmp3, tmp4, tmp5) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{parameter1}).Flatten();
    tmp6 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb1_4});
    tmp7 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp5});
    tmp8 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp6}, TNode<UintPtrT>{tmp7});
    ca_.Branch(tmp8, &block9, std::vector<compiler::Node*>{phi_bb1_4, phi_bb1_4, phi_bb1_4, phi_bb1_4, phi_bb1_4}, &block10, std::vector<compiler::Node*>{phi_bb1_4, phi_bb1_4, phi_bb1_4, phi_bb1_4, phi_bb1_4});
  }

  TNode<IntPtrT> phi_bb9_4;
  TNode<IntPtrT> phi_bb9_9;
  TNode<IntPtrT> phi_bb9_10;
  TNode<IntPtrT> phi_bb9_14;
  TNode<IntPtrT> phi_bb9_15;
  TNode<IntPtrT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<Union<HeapObject, TaggedIndex>> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<Object> tmp13;
  TNode<TheHole> tmp14;
  TNode<BoolT> tmp15;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_4, &phi_bb9_9, &phi_bb9_10, &phi_bb9_14, &phi_bb9_15);
    tmp9 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb9_15});
    tmp10 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp4}, TNode<IntPtrT>{tmp9});
    std::tie(tmp11, tmp12) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp3}, TNode<IntPtrT>{tmp10}).Flatten();
    tmp13 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp11, tmp12});
    tmp14 = TheHole_0(state_);
    tmp15 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp13}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp14});
    ca_.Branch(tmp15, &block13, std::vector<compiler::Node*>{phi_bb9_4}, &block14, std::vector<compiler::Node*>{phi_bb9_4});
  }

  TNode<IntPtrT> phi_bb10_4;
  TNode<IntPtrT> phi_bb10_9;
  TNode<IntPtrT> phi_bb10_10;
  TNode<IntPtrT> phi_bb10_14;
  TNode<IntPtrT> phi_bb10_15;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_4, &phi_bb10_9, &phi_bb10_10, &phi_bb10_14, &phi_bb10_15);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb13_4;
  TNode<Union<HeapObject, TaggedIndex>> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<UintPtrT> tmp19;
  TNode<UintPtrT> tmp20;
  TNode<BoolT> tmp21;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_4);
    std::tie(tmp16, tmp17, tmp18) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{parameter1}).Flatten();
    tmp19 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb13_4});
    tmp20 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp18});
    tmp21 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp19}, TNode<UintPtrT>{tmp20});
    ca_.Branch(tmp21, &block19, std::vector<compiler::Node*>{phi_bb13_4, phi_bb13_4, phi_bb13_4, phi_bb13_4, phi_bb13_4}, &block20, std::vector<compiler::Node*>{phi_bb13_4, phi_bb13_4, phi_bb13_4, phi_bb13_4, phi_bb13_4});
  }

  TNode<IntPtrT> phi_bb19_4;
  TNode<IntPtrT> phi_bb19_10;
  TNode<IntPtrT> phi_bb19_11;
  TNode<IntPtrT> phi_bb19_15;
  TNode<IntPtrT> phi_bb19_16;
  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<Union<HeapObject, TaggedIndex>> tmp24;
  TNode<IntPtrT> tmp25;
  TNode<True> tmp26;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_4, &phi_bb19_10, &phi_bb19_11, &phi_bb19_15, &phi_bb19_16);
    tmp22 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb19_16});
    tmp23 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp17}, TNode<IntPtrT>{tmp22});
    std::tie(tmp24, tmp25) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp16}, TNode<IntPtrT>{tmp23}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp24, tmp25}, parameter2);
    tmp26 = True_0(state_);
    CodeStubAssembler(state_).Return(tmp26);
  }

  TNode<IntPtrT> phi_bb20_4;
  TNode<IntPtrT> phi_bb20_10;
  TNode<IntPtrT> phi_bb20_11;
  TNode<IntPtrT> phi_bb20_15;
  TNode<IntPtrT> phi_bb20_16;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_4, &phi_bb20_10, &phi_bb20_11, &phi_bb20_15, &phi_bb20_16);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb14_4;
  TNode<BoolT> tmp27;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_4);
    tmp27 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{parameter2}, TNode<Object>{tmp13});
    ca_.Branch(tmp27, &block23, std::vector<compiler::Node*>{phi_bb14_4}, &block24, std::vector<compiler::Node*>{phi_bb14_4});
  }

  TNode<IntPtrT> phi_bb23_4;
  TNode<False> tmp28;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_4);
    tmp28 = False_0(state_);
    CodeStubAssembler(state_).Return(tmp28);
  }

  TNode<IntPtrT> phi_bb24_4;
  TNode<IntPtrT> tmp29;
  TNode<IntPtrT> tmp30;
  if (block24.is_used()) {
    ca_.Bind(&block24, &phi_bb24_4);
    tmp29 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp30 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb24_4}, TNode<IntPtrT>{tmp29});
    ca_.Goto(&block3, tmp30);
  }

  TNode<IntPtrT> phi_bb2_4;
  TNode<FixedArray> tmp31;
  TNode<True> tmp32;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_4);
    tmp31 = StoreAndGrowFixedArray_JSReceiver_0(state_, TNode<FixedArray>{parameter1}, TNode<IntPtrT>{tmp0}, TNode<JSReceiver>{parameter2});
    SetJoinStack_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp31});
    tmp32 = True_0(state_);
    CodeStubAssembler(state_).Return(tmp32);
  }
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=687&c=1
TNode<BoolT> JoinStackPushInline_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_receiver) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block28(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block35(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<FixedArray> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = LoadJoinStack_0(state_, TNode<Context>{p_context}, &label1);
    ca_.Goto(&block4);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block5);
    }
  }

  TNode<IntPtrT> tmp2;
  TNode<FixedArray> tmp3;
  TNode<Union<HeapObject, TaggedIndex>> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<UintPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<UintPtrT> tmp9;
  TNode<UintPtrT> tmp10;
  TNode<BoolT> tmp11;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp2 = FromConstexpr_intptr_constexpr_int31_0(state_, JSArray::kMinJoinStackSize);
    tmp3 = CodeStubAssembler(state_).AllocateFixedArrayWithHoles(TNode<IntPtrT>{tmp2}, CastIfEnumClass<CodeStubAssembler::AllocationFlag>(CodeStubAssembler::AllocationFlag::kNone));
    std::tie(tmp4, tmp5, tmp6) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp3}).Flatten();
    tmp7 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp8 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp7});
    tmp9 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp8});
    tmp10 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp6});
    tmp11 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp9}, TNode<UintPtrT>{tmp10});
    ca_.Branch(tmp11, &block34, std::vector<compiler::Node*>{}, &block35, std::vector<compiler::Node*>{});
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<UintPtrT> tmp15;
  TNode<IntPtrT> tmp16;
  TNode<UintPtrT> tmp17;
  TNode<UintPtrT> tmp18;
  TNode<BoolT> tmp19;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    std::tie(tmp12, tmp13, tmp14) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp0}).Flatten();
    tmp15 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp16 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp15});
    tmp17 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp16});
    tmp18 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp14});
    tmp19 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp17}, TNode<UintPtrT>{tmp18});
    ca_.Branch(tmp19, &block13, std::vector<compiler::Node*>{}, &block14, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp20;
  TNode<IntPtrT> tmp21;
  TNode<Union<HeapObject, TaggedIndex>> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<Object> tmp24;
  TNode<TheHole> tmp25;
  TNode<BoolT> tmp26;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp20 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp16});
    tmp21 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp13}, TNode<IntPtrT>{tmp20});
    std::tie(tmp22, tmp23) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp12}, TNode<IntPtrT>{tmp21}).Flatten();
    tmp24 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp22, tmp23});
    tmp25 = TheHole_0(state_);
    tmp26 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp24}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp25});
    ca_.Branch(tmp26, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  if (block14.is_used()) {
    ca_.Bind(&block14);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp27;
  TNode<IntPtrT> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<UintPtrT> tmp30;
  TNode<IntPtrT> tmp31;
  TNode<UintPtrT> tmp32;
  TNode<UintPtrT> tmp33;
  TNode<BoolT> tmp34;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    std::tie(tmp27, tmp28, tmp29) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp0}).Flatten();
    tmp30 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp31 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp30});
    tmp32 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp31});
    tmp33 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp29});
    tmp34 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp32}, TNode<UintPtrT>{tmp33});
    ca_.Branch(tmp34, &block23, std::vector<compiler::Node*>{}, &block24, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp35;
  TNode<IntPtrT> tmp36;
  TNode<Union<HeapObject, TaggedIndex>> tmp37;
  TNode<IntPtrT> tmp38;
  if (block23.is_used()) {
    ca_.Bind(&block23);
    tmp35 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp31});
    tmp36 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp28}, TNode<IntPtrT>{tmp35});
    std::tie(tmp37, tmp38) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp27}, TNode<IntPtrT>{tmp36}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp37, tmp38}, p_receiver);
    ca_.Goto(&block17);
  }

  if (block24.is_used()) {
    ca_.Bind(&block24);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Boolean> tmp39;
  TNode<False> tmp40;
  TNode<BoolT> tmp41;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp39 = ca_.CallBuiltin<Boolean>(Builtin::kJoinStackPush, p_context, tmp0, p_receiver);
    tmp40 = False_0(state_);
    tmp41 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp39}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp40});
    ca_.Branch(tmp41, &block27, std::vector<compiler::Node*>{}, &block28, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp42;
  if (block27.is_used()) {
    ca_.Bind(&block27);
    tmp42 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block1, tmp42);
  }

  if (block28.is_used()) {
    ca_.Bind(&block28);
    ca_.Goto(&block17);
  }

  if (block17.is_used()) {
    ca_.Bind(&block17);
    ca_.Goto(&block2);
  }

  TNode<IntPtrT> tmp43;
  TNode<IntPtrT> tmp44;
  TNode<Union<HeapObject, TaggedIndex>> tmp45;
  TNode<IntPtrT> tmp46;
  if (block34.is_used()) {
    ca_.Bind(&block34);
    tmp43 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp8});
    tmp44 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp5}, TNode<IntPtrT>{tmp43});
    std::tie(tmp45, tmp46) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp4}, TNode<IntPtrT>{tmp44}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp45, tmp46}, p_receiver);
    SetJoinStack_0(state_, TNode<Context>{p_context}, TNode<FixedArray>{tmp3});
    ca_.Goto(&block2);
  }

  if (block35.is_used()) {
    ca_.Bind(&block35);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<BoolT> tmp47;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp47 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block1, tmp47);
  }

  TNode<BoolT> phi_bb1_2;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_2);
    ca_.Goto(&block38, phi_bb1_2);
  }

  TNode<BoolT> phi_bb38_2;
    ca_.Bind(&block38, &phi_bb38_2);
  return TNode<BoolT>{phi_bb38_2};
}

TF_BUILTIN(JoinStackPop, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<FixedArray> parameter1 = UncheckedParameter<FixedArray>(Descriptor::kStack);
  USE(parameter1);
  TNode<JSReceiver> parameter2 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BoolT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block15(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{parameter1});
    tmp1 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block3, tmp1);
  }

  TNode<IntPtrT> phi_bb3_4;
  TNode<BoolT> tmp2;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_4);
    tmp2 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb3_4}, TNode<IntPtrT>{tmp0});
    ca_.Branch(tmp2, &block1, std::vector<compiler::Node*>{phi_bb3_4}, &block2, std::vector<compiler::Node*>{phi_bb3_4});
  }

  TNode<IntPtrT> phi_bb1_4;
  TNode<Union<HeapObject, TaggedIndex>> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<UintPtrT> tmp6;
  TNode<UintPtrT> tmp7;
  TNode<BoolT> tmp8;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_4);
    std::tie(tmp3, tmp4, tmp5) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{parameter1}).Flatten();
    tmp6 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb1_4});
    tmp7 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp5});
    tmp8 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp6}, TNode<UintPtrT>{tmp7});
    ca_.Branch(tmp8, &block11, std::vector<compiler::Node*>{phi_bb1_4, phi_bb1_4, phi_bb1_4, phi_bb1_4, phi_bb1_4}, &block12, std::vector<compiler::Node*>{phi_bb1_4, phi_bb1_4, phi_bb1_4, phi_bb1_4, phi_bb1_4});
  }

  TNode<IntPtrT> phi_bb11_4;
  TNode<IntPtrT> phi_bb11_9;
  TNode<IntPtrT> phi_bb11_10;
  TNode<IntPtrT> phi_bb11_14;
  TNode<IntPtrT> phi_bb11_15;
  TNode<IntPtrT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<Union<HeapObject, TaggedIndex>> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<Object> tmp13;
  TNode<BoolT> tmp14;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_4, &phi_bb11_9, &phi_bb11_10, &phi_bb11_14, &phi_bb11_15);
    tmp9 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb11_15});
    tmp10 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp4}, TNode<IntPtrT>{tmp9});
    std::tie(tmp11, tmp12) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp3}, TNode<IntPtrT>{tmp10}).Flatten();
    tmp13 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp11, tmp12});
    tmp14 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp13}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{parameter2});
    ca_.Branch(tmp14, &block5, std::vector<compiler::Node*>{phi_bb11_4}, &block6, std::vector<compiler::Node*>{phi_bb11_4});
  }

  TNode<IntPtrT> phi_bb12_4;
  TNode<IntPtrT> phi_bb12_9;
  TNode<IntPtrT> phi_bb12_10;
  TNode<IntPtrT> phi_bb12_14;
  TNode<IntPtrT> phi_bb12_15;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_4, &phi_bb12_9, &phi_bb12_10, &phi_bb12_14, &phi_bb12_15);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb5_4;
  TNode<IntPtrT> tmp15;
  TNode<BoolT> tmp16;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_4);
    tmp15 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp16 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb5_4}, TNode<IntPtrT>{tmp15});
    ca_.Branch(tmp16, &block17, std::vector<compiler::Node*>{phi_bb5_4}, &block18, std::vector<compiler::Node*>{phi_bb5_4});
  }

  TNode<IntPtrT> phi_bb17_4;
  TNode<IntPtrT> tmp17;
  TNode<BoolT> tmp18;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_4);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, JSArray::kMinJoinStackSize);
    tmp18 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp0}, TNode<IntPtrT>{tmp17});
    ca_.Goto(&block19, phi_bb17_4, tmp18);
  }

  TNode<IntPtrT> phi_bb18_4;
  TNode<BoolT> tmp19;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_4);
    tmp19 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block19, phi_bb18_4, tmp19);
  }

  TNode<IntPtrT> phi_bb19_4;
  TNode<BoolT> phi_bb19_6;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_4, &phi_bb19_6);
    ca_.Branch(phi_bb19_6, &block15, std::vector<compiler::Node*>{phi_bb19_4}, &block16, std::vector<compiler::Node*>{phi_bb19_4});
  }

  TNode<IntPtrT> phi_bb15_4;
  TNode<IntPtrT> tmp20;
  TNode<FixedArray> tmp21;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_4);
    tmp20 = FromConstexpr_intptr_constexpr_int31_0(state_, JSArray::kMinJoinStackSize);
    tmp21 = CodeStubAssembler(state_).AllocateFixedArrayWithHoles(TNode<IntPtrT>{tmp20}, CastIfEnumClass<CodeStubAssembler::AllocationFlag>(CodeStubAssembler::AllocationFlag::kNone));
    SetJoinStack_0(state_, TNode<Context>{parameter0}, TNode<FixedArray>{tmp21});
    ca_.Goto(&block20, phi_bb15_4);
  }

  TNode<IntPtrT> phi_bb16_4;
  TNode<Union<HeapObject, TaggedIndex>> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<IntPtrT> tmp24;
  TNode<UintPtrT> tmp25;
  TNode<UintPtrT> tmp26;
  TNode<BoolT> tmp27;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_4);
    std::tie(tmp22, tmp23, tmp24) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{parameter1}).Flatten();
    tmp25 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb16_4});
    tmp26 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp24});
    tmp27 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp25}, TNode<UintPtrT>{tmp26});
    ca_.Branch(tmp27, &block25, std::vector<compiler::Node*>{phi_bb16_4, phi_bb16_4, phi_bb16_4, phi_bb16_4, phi_bb16_4}, &block26, std::vector<compiler::Node*>{phi_bb16_4, phi_bb16_4, phi_bb16_4, phi_bb16_4, phi_bb16_4});
  }

  TNode<IntPtrT> phi_bb25_4;
  TNode<IntPtrT> phi_bb25_9;
  TNode<IntPtrT> phi_bb25_10;
  TNode<IntPtrT> phi_bb25_14;
  TNode<IntPtrT> phi_bb25_15;
  TNode<IntPtrT> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<Union<HeapObject, TaggedIndex>> tmp30;
  TNode<IntPtrT> tmp31;
  TNode<TheHole> tmp32;
  if (block25.is_used()) {
    ca_.Bind(&block25, &phi_bb25_4, &phi_bb25_9, &phi_bb25_10, &phi_bb25_14, &phi_bb25_15);
    tmp28 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb25_15});
    tmp29 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp23}, TNode<IntPtrT>{tmp28});
    std::tie(tmp30, tmp31) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp22}, TNode<IntPtrT>{tmp29}).Flatten();
    tmp32 = TheHole_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp30, tmp31}, tmp32);
    ca_.Goto(&block20, phi_bb25_4);
  }

  TNode<IntPtrT> phi_bb26_4;
  TNode<IntPtrT> phi_bb26_9;
  TNode<IntPtrT> phi_bb26_10;
  TNode<IntPtrT> phi_bb26_14;
  TNode<IntPtrT> phi_bb26_15;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_4, &phi_bb26_9, &phi_bb26_10, &phi_bb26_14, &phi_bb26_15);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb20_4;
  TNode<Undefined> tmp33;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_4);
    tmp33 = Undefined_0(state_);
    CodeStubAssembler(state_).Return(tmp33);
  }

  TNode<IntPtrT> phi_bb6_4;
  TNode<IntPtrT> tmp34;
  TNode<IntPtrT> tmp35;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_4);
    tmp34 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp35 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb6_4}, TNode<IntPtrT>{tmp34});
    ca_.Goto(&block3, tmp35);
  }

  TNode<IntPtrT> phi_bb2_4;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_4);
    CodeStubAssembler(state_).Unreachable();
  }
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=732&c=1
void JoinStackPopInline_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_receiver) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<FixedArray> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = LoadJoinStack_0(state_, TNode<Context>{p_context}, &label1);
    ca_.Goto(&block4);
    if (label1.is_used()) {
      ca_.Bind(&label1);
      ca_.Goto(&block5);
    }
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> tmp2;
  TNode<Union<HeapObject, TaggedIndex>> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<UintPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<UintPtrT> tmp8;
  TNode<UintPtrT> tmp9;
  TNode<BoolT> tmp10;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp2 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{tmp0});
    std::tie(tmp3, tmp4, tmp5) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp0}).Flatten();
    tmp6 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp7 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp6});
    tmp8 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp7});
    tmp9 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp5});
    tmp10 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp8}, TNode<UintPtrT>{tmp9});
    ca_.Branch(tmp10, &block13, std::vector<compiler::Node*>{}, &block14, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<Union<HeapObject, TaggedIndex>> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<Object> tmp15;
  TNode<BoolT> tmp16;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp11 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp7});
    tmp12 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp4}, TNode<IntPtrT>{tmp11});
    std::tie(tmp13, tmp14) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp3}, TNode<IntPtrT>{tmp12}).Flatten();
    tmp15 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp13, tmp14});
    tmp16 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp15}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{p_receiver});
    ca_.Branch(tmp16, &block17, std::vector<compiler::Node*>{}, &block18, std::vector<compiler::Node*>{});
  }

  if (block14.is_used()) {
    ca_.Bind(&block14);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> tmp17;
  TNode<BoolT> tmp18;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, JSArray::kMinJoinStackSize);
    tmp18 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp2}, TNode<IntPtrT>{tmp17});
    ca_.Goto(&block19, tmp18);
  }

  TNode<BoolT> tmp19;
  if (block18.is_used()) {
    ca_.Bind(&block18);
    tmp19 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block19, tmp19);
  }

  TNode<BoolT> phi_bb19_5;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_5);
    ca_.Branch(phi_bb19_5, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp20;
  TNode<IntPtrT> tmp21;
  TNode<IntPtrT> tmp22;
  TNode<UintPtrT> tmp23;
  TNode<IntPtrT> tmp24;
  TNode<UintPtrT> tmp25;
  TNode<UintPtrT> tmp26;
  TNode<BoolT> tmp27;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    std::tie(tmp20, tmp21, tmp22) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp0}).Flatten();
    tmp23 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp24 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp23});
    tmp25 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp24});
    tmp26 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp22});
    tmp27 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp25}, TNode<UintPtrT>{tmp26});
    ca_.Branch(tmp27, &block26, std::vector<compiler::Node*>{}, &block27, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp28;
  TNode<IntPtrT> tmp29;
  TNode<Union<HeapObject, TaggedIndex>> tmp30;
  TNode<IntPtrT> tmp31;
  TNode<TheHole> tmp32;
  if (block26.is_used()) {
    ca_.Bind(&block26);
    tmp28 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp24});
    tmp29 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp21}, TNode<IntPtrT>{tmp28});
    std::tie(tmp30, tmp31) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp20}, TNode<IntPtrT>{tmp29}).Flatten();
    tmp32 = TheHole_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp30, tmp31}, tmp32);
    ca_.Goto(&block20);
  }

  if (block27.is_used()) {
    ca_.Bind(&block27);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<JSAny> tmp33;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp33 = ca_.CallBuiltin<JSAny>(Builtin::kJoinStackPop, p_context, tmp0, p_receiver);
    ca_.Goto(&block20);
  }

  if (block20.is_used()) {
    ca_.Bind(&block20);
    ca_.Goto(&block30);
  }

    ca_.Bind(&block30);
}

TF_BUILTIN(ArrayPrototypeJoin, CodeStubAssembler) {
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
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<JSAny> tmp1;
  TNode<JSReceiver> tmp2;
  TNode<Number> tmp3;
  TNode<Number> tmp4;
  TNode<BoolT> tmp5;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp0});
    tmp2 = CodeStubAssembler(state_).ToObject_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter1});
    tmp3 = GetLengthProperty_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{tmp2});
    tmp4 = FromConstexpr_Number_constexpr_uint32_0(state_, JSArray::kMaxArrayLength);
    tmp5 = NumberIsGreaterThan_0(state_, TNode<Number>{tmp3}, TNode<Number>{tmp4});
    ca_.Branch(tmp5, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kInvalidArrayLength));
  }

  TNode<Undefined> tmp6;
  TNode<BoolT> tmp7;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp6 = Undefined_0(state_);
    tmp7 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp1}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp6});
    ca_.Branch(tmp7, &block3, std::vector<compiler::Node*>{}, &block4, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp8;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp8 = FromConstexpr_String_constexpr_string_0(state_, ",");
    ca_.Goto(&block5, tmp8);
  }

  TNode<String> tmp9;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp9 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{tmp1});
    ca_.Goto(&block5, tmp9);
  }

  TNode<String> phi_bb5_9;
  TNode<Number> tmp10;
  TNode<BoolT> tmp11;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_9);
    tmp10 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp11 = IsNumberEqual_0(state_, TNode<Number>{tmp3}, TNode<Number>{tmp10});
    ca_.Branch(tmp11, &block7, std::vector<compiler::Node*>{}, &block8, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp12;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp12 = kEmptyString_0(state_);
    arguments.PopAndReturn(tmp12);
  }

  TNode<JSAny> tmp13;
  if (block8.is_used()) {
    ca_.Bind(&block8);
    tmp13 = ca_.CallBuiltin<JSAny>(Builtin::kArrayPrototypeJoinImpl, parameter0, tmp2, tmp3, phi_bb5_9);
    arguments.PopAndReturn(tmp13);
  }
}

TF_BUILTIN(ArrayPrototypeJoinImpl, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kO);
  USE(parameter1);
  TNode<Number> parameter2 = UncheckedParameter<Number>(Descriptor::kLen);
  USE(parameter2);
  TNode<String> parameter3 = UncheckedParameter<String>(Descriptor::kSeparator);
  USE(parameter3);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSArray> tmp0;
  TNode<Int32T> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label2(&ca_);
    std::tie(tmp0, tmp1) = CastFastJSArrayForRead_0(state_, TNode<Context>{parameter0}, TNode<HeapObject>{parameter1}, &label2).Flatten();
    ca_.Goto(&block11);
    if (label2.is_used()) {
      ca_.Bind(&label2);
      ca_.Goto(&block12);
    }
  }

  if (block12.is_used()) {
    ca_.Bind(&block12);
    ca_.Goto(&block9);
  }

  TNode<BoolT> tmp3;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp3 = CodeStubAssembler(state_).IsFastSmiElementsKind(TNode<Int32T>{tmp1});
    ca_.Branch(tmp3, &block13, std::vector<compiler::Node*>{}, &block14, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp4;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp4 = CodeStubAssembler(state_).IsHoleyFastElementsKind(TNode<Int32T>{tmp1});
    ca_.Branch(tmp4, &block16, std::vector<compiler::Node*>{}, &block17, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp5;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp5 = FastArrayJoin_0(state_, TNode<Context>{parameter0}, CastIfEnumClass<ElementsKind>(ElementsKind::HOLEY_SMI_ELEMENTS), TNode<JSArray>{tmp0}, TNode<String>{parameter3}, TNode<Number>{parameter2});
    CodeStubAssembler(state_).Return(tmp5);
  }

  TNode<String> tmp6;
  if (block17.is_used()) {
    ca_.Bind(&block17);
    tmp6 = FastArrayJoin_0(state_, TNode<Context>{parameter0}, CastIfEnumClass<ElementsKind>(ElementsKind::PACKED_SMI_ELEMENTS), TNode<JSArray>{tmp0}, TNode<String>{parameter3}, TNode<Number>{parameter2});
    CodeStubAssembler(state_).Return(tmp6);
  }

  TNode<BoolT> tmp7;
  if (block14.is_used()) {
    ca_.Bind(&block14);
    tmp7 = CodeStubAssembler(state_).IsDoubleElementsKind(TNode<Int32T>{tmp1});
    ca_.Branch(tmp7, &block19, std::vector<compiler::Node*>{}, &block20, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp8;
  if (block19.is_used()) {
    ca_.Bind(&block19);
    tmp8 = CodeStubAssembler(state_).IsHoleyFastElementsKind(TNode<Int32T>{tmp1});
    ca_.Branch(tmp8, &block21, std::vector<compiler::Node*>{}, &block22, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp9;
  if (block21.is_used()) {
    ca_.Bind(&block21);
    tmp9 = FastArrayJoin_0(state_, TNode<Context>{parameter0}, CastIfEnumClass<ElementsKind>(ElementsKind::HOLEY_DOUBLE_ELEMENTS), TNode<JSArray>{tmp0}, TNode<String>{parameter3}, TNode<Number>{parameter2});
    CodeStubAssembler(state_).Return(tmp9);
  }

  TNode<String> tmp10;
  if (block22.is_used()) {
    ca_.Bind(&block22);
    tmp10 = FastArrayJoin_0(state_, TNode<Context>{parameter0}, CastIfEnumClass<ElementsKind>(ElementsKind::PACKED_DOUBLE_ELEMENTS), TNode<JSArray>{tmp0}, TNode<String>{parameter3}, TNode<Number>{parameter2});
    CodeStubAssembler(state_).Return(tmp10);
  }

  if (block20.is_used()) {
    ca_.Bind(&block20);
    ca_.Goto(&block9);
  }

  TNode<Undefined> tmp11;
  TNode<Undefined> tmp12;
  TNode<JSAny> tmp13;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    CodeStubAssembler(state_).PerformStackCheck(TNode<Context>{parameter0});
    tmp11 = Undefined_0(state_);
    tmp12 = Undefined_0(state_);
    tmp13 = CycleProtectedArrayJoin_JSArray_0(state_, TNode<Context>{parameter0}, false, TNode<JSReceiver>{parameter1}, TNode<Number>{parameter2}, TNode<String>{parameter3}, TNode<JSAny>{tmp11}, TNode<JSAny>{tmp12});
    CodeStubAssembler(state_).Return(tmp13);
  }
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=839&c=1
TNode<String> ArrayPrototypeToString_Inline_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSArray> p_array) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<String> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Number> tmp1;
  TNode<Number> tmp2;
  TNode<BoolT> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp1 = CodeStubAssembler(state_).LoadReference<Number>(CodeStubAssembler::Reference{p_array, tmp0});
    tmp2 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp3 = IsNumberEqual_0(state_, TNode<Number>{tmp1}, TNode<Number>{tmp2});
    ca_.Branch(tmp3, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp4;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp4 = kEmptyString_0(state_);
    ca_.Goto(&block1, tmp4);
  }

  TNode<Number> tmp5;
  TNode<BoolT> tmp6;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp5 = FromConstexpr_Number_constexpr_uint32_0(state_, JSArray::kMaxArrayLength);
    tmp6 = NumberIsGreaterThan_0(state_, TNode<Number>{tmp1}, TNode<Number>{tmp5});
    ca_.Branch(tmp6, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  if (block4.is_used()) {
    ca_.Bind(&block4);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{p_context}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kInvalidArrayLength));
  }

  TNode<String> tmp7;
  TNode<JSAny> tmp8;
  TNode<String> tmp9;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp7 = FromConstexpr_String_constexpr_string_0(state_, ",");
    tmp8 = ca_.CallBuiltin<JSAny>(Builtin::kArrayPrototypeJoinImpl, p_context, p_array, tmp1, tmp7);
    compiler::CodeAssemblerLabel label10(&ca_);
    tmp9 = Cast_String_1(state_, TNode<Context>{p_context}, TNode<Object>{tmp8}, &label10);
    ca_.Goto(&block8);
    if (label10.is_used()) {
      ca_.Bind(&label10);
      ca_.Goto(&block9);
    }
  }

  if (block9.is_used()) {
    ca_.Bind(&block9);
    CodeStubAssembler(state_).Unreachable();
  }

  if (block8.is_used()) {
    ca_.Bind(&block8);
    ca_.Goto(&block1, tmp9);
  }

  TNode<String> phi_bb1_2;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_2);
    ca_.Goto(&block10, phi_bb1_2);
  }

  TNode<String> phi_bb10_2;
    ca_.Bind(&block10, &phi_bb10_2);
  return TNode<String>{phi_bb10_2};
}

TF_BUILTIN(ArrayPrototypeToLocaleString, CodeStubAssembler) {
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
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<JSAny> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<JSAny> tmp3;
  TNode<JSReceiver> tmp4;
  TNode<Number> tmp5;
  TNode<Number> tmp6;
  TNode<BoolT> tmp7;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp0});
    tmp2 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp3 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp2});
    tmp4 = CodeStubAssembler(state_).ToObject_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter1});
    tmp5 = GetLengthProperty_0(state_, TNode<Context>{parameter0}, TNode<JSAny>{tmp4});
    tmp6 = FromConstexpr_Number_constexpr_uint32_0(state_, JSArray::kMaxArrayLength);
    tmp7 = NumberIsGreaterThan_0(state_, TNode<Number>{tmp5}, TNode<Number>{tmp6});
    ca_.Branch(tmp7, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    CodeStubAssembler(state_).ThrowTypeError(TNode<Context>{parameter0}, CastIfEnumClass<MessageTemplate>(MessageTemplate::kInvalidArrayLength));
  }

  TNode<String> tmp8;
  TNode<JSAny> tmp9;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp8 = FromConstexpr_String_constexpr_string_0(state_, ",");
    tmp9 = CycleProtectedArrayJoin_JSArray_0(state_, TNode<Context>{parameter0}, true, TNode<JSReceiver>{tmp4}, TNode<Number>{tmp5}, TNode<String>{tmp8}, TNode<JSAny>{tmp1}, TNode<JSAny>{tmp3});
    arguments.PopAndReturn(tmp9);
  }
}

TF_BUILTIN(ArrayPrototypeToString, CodeStubAssembler) {
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
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSReceiver> tmp0;
  TNode<JSAny> tmp1;
  TNode<JSAny> tmp2;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp3;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).ToObject_Inline(TNode<Context>{parameter0}, TNode<JSAny>{parameter1});
    tmp1 = FromConstexpr_JSAny_constexpr_string_0(state_, "join");
    tmp2 = CodeStubAssembler(state_).GetProperty(TNode<Context>{parameter0}, TNode<JSAny>{tmp0}, TNode<JSAny>{tmp1});
    compiler::CodeAssemblerLabel label4(&ca_);
    tmp3 = Cast_Callable_1(state_, TNode<Context>{parameter0}, TNode<Object>{tmp2}, &label4);
    ca_.Goto(&block3);
    if (label4.is_used()) {
      ca_.Bind(&label4);
      ca_.Goto(&block4);
    }
  }

  TNode<String> tmp5;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp5 = ca_.CallBuiltin<String>(Builtin::kObjectToString, parameter0, tmp0);
    arguments.PopAndReturn(tmp5);
  }

  TNode<JSAny> tmp6;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp6 = CodeStubAssembler(state_).Call(TNode<Context>{parameter0}, TNode<JSAny>{tmp3}, TNode<JSAny>{tmp0});
    arguments.PopAndReturn(tmp6);
  }
}

TF_BUILTIN(TypedArrayPrototypeJoin, CodeStubAssembler) {
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
  compiler::CodeAssemblerParameterizedLabel<String> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<JSAny> tmp1;
  TNode<UintPtrT> tmp2;
  TNode<JSTypedArray> tmp3;
  TNode<Undefined> tmp4;
  TNode<BoolT> tmp5;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp0});
    tmp2 = TypedArrayBuiltinsAssembler(state_).ValidateTypedArrayAndGetLength(TNode<Context>{parameter0}, TNode<JSAny>{parameter1}, "%TypedArray%.prototype.join", CastIfEnumClass<TypedArrayAccessMode>(TypedArrayAccessMode::kRead));
    tmp3 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp4 = Undefined_0(state_);
    tmp5 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp1}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp4});
    ca_.Branch(tmp5, &block1, std::vector<compiler::Node*>{}, &block2, std::vector<compiler::Node*>{});
  }

  TNode<String> tmp6;
  if (block1.is_used()) {
    ca_.Bind(&block1);
    tmp6 = FromConstexpr_String_constexpr_string_0(state_, ",");
    ca_.Goto(&block3, tmp6);
  }

  TNode<String> tmp7;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp7 = CodeStubAssembler(state_).ToString_Inline(TNode<Context>{parameter0}, TNode<JSAny>{tmp1});
    ca_.Goto(&block3, tmp7);
  }

  TNode<String> phi_bb3_9;
  TNode<Number> tmp8;
  TNode<Undefined> tmp9;
  TNode<Undefined> tmp10;
  TNode<JSAny> tmp11;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_9);
    tmp8 = Convert_Number_uintptr_0(state_, TNode<UintPtrT>{tmp2});
    tmp9 = Undefined_0(state_);
    tmp10 = Undefined_0(state_);
    tmp11 = CycleProtectedArrayJoin_JSTypedArray_0(state_, TNode<Context>{parameter0}, false, TNode<JSReceiver>{tmp3}, TNode<Number>{tmp8}, TNode<String>{phi_bb3_9}, TNode<JSAny>{tmp9}, TNode<JSAny>{tmp10});
    arguments.PopAndReturn(tmp11);
  }
}

TF_BUILTIN(TypedArrayPrototypeToLocaleString, CodeStubAssembler) {
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
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<JSAny> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<JSAny> tmp3;
  TNode<UintPtrT> tmp4;
  TNode<JSTypedArray> tmp5;
  TNode<Number> tmp6;
  TNode<String> tmp7;
  TNode<JSAny> tmp8;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp0});
    tmp2 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp3 = CodeStubAssembler(state_).GetArgumentValue(TorqueStructArguments{TNode<RawPtrT>{torque_arguments.frame}, TNode<RawPtrT>{torque_arguments.base}, TNode<IntPtrT>{torque_arguments.length}, TNode<IntPtrT>{torque_arguments.actual_count}}, TNode<IntPtrT>{tmp2});
    tmp4 = TypedArrayBuiltinsAssembler(state_).ValidateTypedArrayAndGetLength(TNode<Context>{parameter0}, TNode<JSAny>{parameter1}, "%TypedArray%.prototype.toLocaleString", CastIfEnumClass<TypedArrayAccessMode>(TypedArrayAccessMode::kRead));
    tmp5 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp6 = Convert_Number_uintptr_0(state_, TNode<UintPtrT>{tmp4});
    tmp7 = FromConstexpr_String_constexpr_string_0(state_, ",");
    tmp8 = CycleProtectedArrayJoin_JSTypedArray_0(state_, TNode<Context>{parameter0}, true, TNode<JSReceiver>{tmp5}, TNode<Number>{tmp6}, TNode<String>{tmp7}, TNode<JSAny>{tmp1}, TNode<JSAny>{tmp3});
    arguments.PopAndReturn(tmp8);
  }
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=28&c=34
TNode<NumberDictionary> UnsafeCast_NumberDictionary_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<NumberDictionary> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = TORQUE_CAST(TNode<Object>{p_o});
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<NumberDictionary>{tmp0};
}

TF_BUILTIN(LoadJoinElement_GenericElementsAccessor_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Number> tmp0;
  TNode<JSAny> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Convert_Number_uintptr_0(state_, TNode<UintPtrT>{parameter2});
    tmp1 = CodeStubAssembler(state_).GetProperty(TNode<Context>{parameter0}, TNode<JSAny>{parameter1}, TNode<JSAny>{tmp0});
    CodeStubAssembler(state_).Return(tmp1);
  }
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=562&c=10
TNode<String> ArrayJoinImpl_JSArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_receiver, TNode<String> p_sep, TNode<Number> p_lengthNumber, bool p_useToLocaleString, TNode<JSAny> p_locales, TNode<JSAny> p_options, TNode<BuiltinPtr> p_initialLoadFn) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, BoolT> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block42(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block44(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block62(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block63(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block56(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block66(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block67(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block88(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block89(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block97(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block98(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block45(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block101(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block113(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block114(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block121(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block120(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block123(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block122(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block133(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block140(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block141(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block134(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block144(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block145(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, IntPtrT> block146(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block157(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block158(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block166(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block167(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block175(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block176(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block135(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block118(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block182(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block181(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block195(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block196(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block117(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block102(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block208(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, String, Object, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block215(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, String, Object, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block216(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block209(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block219(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block220(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object, IntPtrT> block221(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object, FixedArray> block232(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object, FixedArray> block233(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block241(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block242(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block250(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block251(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block210(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block103(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block257(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block258(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, BoolT> block259(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block255(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block256(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block260(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block261(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block262(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block273(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block280(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block281(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block274(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block284(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block285(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block286(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray> block297(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray> block298(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block306(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block307(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block315(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block316(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block275(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block263(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block254(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block319(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Map> tmp1;
  TNode<UintPtrT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<FixedArray> tmp5;
  TNode<FixedArray> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<BoolT> tmp9;
  TNode<PrimitiveHeapObject> tmp10;
  TNode<UintPtrT> tmp11;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp1 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{p_receiver, tmp0});
    tmp2 = Convert_uintptr_Number_0(state_, TNode<Number>{p_lengthNumber});
    tmp3 = CodeStubAssembler(state_).LoadStringLengthAsWord(TNode<String>{p_sep});
    tmp4 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp5, tmp6, tmp7, tmp8, tmp9, tmp10) = NewBuffer_0(state_, TNode<UintPtrT>{tmp2}, TNode<String>{p_sep}).Flatten();
    tmp11 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block4, tmp4, p_initialLoadFn, tmp6, tmp7, tmp8, tmp9, tmp10, tmp11);
  }

  TNode<IntPtrT> phi_bb4_10;
  TNode<BuiltinPtr> phi_bb4_11;
  TNode<FixedArray> phi_bb4_13;
  TNode<IntPtrT> phi_bb4_14;
  TNode<IntPtrT> phi_bb4_15;
  TNode<BoolT> phi_bb4_16;
  TNode<PrimitiveHeapObject> phi_bb4_17;
  TNode<UintPtrT> phi_bb4_18;
  TNode<BoolT> tmp12;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_10, &phi_bb4_11, &phi_bb4_13, &phi_bb4_14, &phi_bb4_15, &phi_bb4_16, &phi_bb4_17, &phi_bb4_18);
    tmp12 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{phi_bb4_18}, TNode<UintPtrT>{tmp2});
    ca_.Branch(tmp12, &block2, std::vector<compiler::Node*>{phi_bb4_10, phi_bb4_11, phi_bb4_13, phi_bb4_14, phi_bb4_15, phi_bb4_16, phi_bb4_17, phi_bb4_18}, &block3, std::vector<compiler::Node*>{phi_bb4_10, phi_bb4_11, phi_bb4_13, phi_bb4_14, phi_bb4_15, phi_bb4_16, phi_bb4_17, phi_bb4_18});
  }

  TNode<IntPtrT> phi_bb2_10;
  TNode<BuiltinPtr> phi_bb2_11;
  TNode<FixedArray> phi_bb2_13;
  TNode<IntPtrT> phi_bb2_14;
  TNode<IntPtrT> phi_bb2_15;
  TNode<BoolT> phi_bb2_16;
  TNode<PrimitiveHeapObject> phi_bb2_17;
  TNode<UintPtrT> phi_bb2_18;
  TNode<BoolT> tmp13;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_10, &phi_bb2_11, &phi_bb2_13, &phi_bb2_14, &phi_bb2_15, &phi_bb2_16, &phi_bb2_17, &phi_bb2_18);
    tmp13 = CannotUseSameArrayAccessor_JSArray_0(state_, TNode<Context>{p_context}, TNode<BuiltinPtr>{phi_bb2_11}, TNode<JSReceiver>{p_receiver}, TNode<Map>{tmp1}, TNode<Number>{p_lengthNumber});
    ca_.Branch(tmp13, &block5, std::vector<compiler::Node*>{phi_bb2_10, phi_bb2_11, phi_bb2_13, phi_bb2_14, phi_bb2_15, phi_bb2_16, phi_bb2_17, phi_bb2_18}, &block6, std::vector<compiler::Node*>{phi_bb2_10, phi_bb2_11, phi_bb2_13, phi_bb2_14, phi_bb2_15, phi_bb2_16, phi_bb2_17, phi_bb2_18});
  }

  TNode<IntPtrT> phi_bb5_10;
  TNode<BuiltinPtr> phi_bb5_11;
  TNode<FixedArray> phi_bb5_13;
  TNode<IntPtrT> phi_bb5_14;
  TNode<IntPtrT> phi_bb5_15;
  TNode<BoolT> phi_bb5_16;
  TNode<PrimitiveHeapObject> phi_bb5_17;
  TNode<UintPtrT> phi_bb5_18;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_10, &phi_bb5_11, &phi_bb5_13, &phi_bb5_14, &phi_bb5_15, &phi_bb5_16, &phi_bb5_17, &phi_bb5_18);
    ca_.Goto(&block6, phi_bb5_10, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinElement_GenericElementsAccessor_0)), phi_bb5_13, phi_bb5_14, phi_bb5_15, phi_bb5_16, phi_bb5_17, phi_bb5_18);
  }

  TNode<IntPtrT> phi_bb6_10;
  TNode<BuiltinPtr> phi_bb6_11;
  TNode<FixedArray> phi_bb6_13;
  TNode<IntPtrT> phi_bb6_14;
  TNode<IntPtrT> phi_bb6_15;
  TNode<BoolT> phi_bb6_16;
  TNode<PrimitiveHeapObject> phi_bb6_17;
  TNode<UintPtrT> phi_bb6_18;
  TNode<UintPtrT> tmp14;
  TNode<BoolT> tmp15;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_10, &phi_bb6_11, &phi_bb6_13, &phi_bb6_14, &phi_bb6_15, &phi_bb6_16, &phi_bb6_17, &phi_bb6_18);
    tmp14 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp15 = CodeStubAssembler(state_).UintPtrGreaterThan(TNode<UintPtrT>{phi_bb6_18}, TNode<UintPtrT>{tmp14});
    ca_.Branch(tmp15, &block7, std::vector<compiler::Node*>{phi_bb6_10, phi_bb6_13, phi_bb6_14, phi_bb6_15, phi_bb6_16, phi_bb6_17, phi_bb6_18}, &block8, std::vector<compiler::Node*>{phi_bb6_10, phi_bb6_13, phi_bb6_14, phi_bb6_15, phi_bb6_16, phi_bb6_17, phi_bb6_18});
  }

  TNode<IntPtrT> phi_bb7_10;
  TNode<FixedArray> phi_bb7_13;
  TNode<IntPtrT> phi_bb7_14;
  TNode<IntPtrT> phi_bb7_15;
  TNode<BoolT> phi_bb7_16;
  TNode<PrimitiveHeapObject> phi_bb7_17;
  TNode<UintPtrT> phi_bb7_18;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_10, &phi_bb7_13, &phi_bb7_14, &phi_bb7_15, &phi_bb7_16, &phi_bb7_17, &phi_bb7_18);
    tmp16 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp17 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb7_10}, TNode<IntPtrT>{tmp16});
    ca_.Goto(&block8, tmp17, phi_bb7_13, phi_bb7_14, phi_bb7_15, phi_bb7_16, phi_bb7_17, phi_bb7_18);
  }

  TNode<IntPtrT> phi_bb8_10;
  TNode<FixedArray> phi_bb8_13;
  TNode<IntPtrT> phi_bb8_14;
  TNode<IntPtrT> phi_bb8_15;
  TNode<BoolT> phi_bb8_16;
  TNode<PrimitiveHeapObject> phi_bb8_17;
  TNode<UintPtrT> phi_bb8_18;
  TNode<UintPtrT> tmp18;
  TNode<UintPtrT> tmp19;
  TNode<JSAny> tmp20;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_10, &phi_bb8_13, &phi_bb8_14, &phi_bb8_15, &phi_bb8_16, &phi_bb8_17, &phi_bb8_18);
    tmp18 = FromConstexpr_uintptr_constexpr_int31_0(state_, 1);
    tmp19 = CodeStubAssembler(state_).UintPtrAdd(TNode<UintPtrT>{phi_bb8_18}, TNode<UintPtrT>{tmp18});
tmp20 = TORQUE_CAST(CodeStubAssembler(state_).CallBuiltinPointer(Builtins::CallInterfaceDescriptorFor(ExampleBuiltinForTorqueFunctionPointerType(3)), phi_bb6_11, p_context, p_receiver, phi_bb8_18));
    if ((p_useToLocaleString)) {
      ca_.Goto(&block9, phi_bb8_13, phi_bb8_14, phi_bb8_15, phi_bb8_16, phi_bb8_17);
    } else {
      ca_.Goto(&block10, phi_bb8_13, phi_bb8_14, phi_bb8_15, phi_bb8_16, phi_bb8_17);
    }
  }

  TNode<FixedArray> phi_bb9_13;
  TNode<IntPtrT> phi_bb9_14;
  TNode<IntPtrT> phi_bb9_15;
  TNode<BoolT> phi_bb9_16;
  TNode<PrimitiveHeapObject> phi_bb9_17;
  TNode<String> tmp21;
  TNode<String> tmp22;
  TNode<BoolT> tmp23;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_13, &phi_bb9_14, &phi_bb9_15, &phi_bb9_16, &phi_bb9_17);
    tmp21 = ca_.CallBuiltin<String>(Builtin::kConvertToLocaleString, p_context, tmp20, p_locales, p_options);
    tmp22 = kEmptyString_0(state_);
    tmp23 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp21}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp22});
    ca_.Branch(tmp23, &block12, std::vector<compiler::Node*>{phi_bb9_13, phi_bb9_14, phi_bb9_15, phi_bb9_16, phi_bb9_17}, &block13, std::vector<compiler::Node*>{phi_bb9_13, phi_bb9_14, phi_bb9_15, phi_bb9_16, phi_bb9_17});
  }

  TNode<FixedArray> phi_bb12_13;
  TNode<IntPtrT> phi_bb12_14;
  TNode<IntPtrT> phi_bb12_15;
  TNode<BoolT> phi_bb12_16;
  TNode<PrimitiveHeapObject> phi_bb12_17;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_13, &phi_bb12_14, &phi_bb12_15, &phi_bb12_16, &phi_bb12_17);
    ca_.Goto(&block4, phi_bb8_10, phi_bb6_11, phi_bb12_13, phi_bb12_14, phi_bb12_15, phi_bb12_16, phi_bb12_17, tmp19);
  }

  TNode<FixedArray> phi_bb13_13;
  TNode<IntPtrT> phi_bb13_14;
  TNode<IntPtrT> phi_bb13_15;
  TNode<BoolT> phi_bb13_16;
  TNode<PrimitiveHeapObject> phi_bb13_17;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_13, &phi_bb13_14, &phi_bb13_15, &phi_bb13_16, &phi_bb13_17);
    ca_.Goto(&block11, phi_bb13_13, phi_bb13_14, phi_bb13_15, phi_bb13_16, phi_bb13_17, tmp21);
  }

  TNode<FixedArray> phi_bb10_13;
  TNode<IntPtrT> phi_bb10_14;
  TNode<IntPtrT> phi_bb10_15;
  TNode<BoolT> phi_bb10_16;
  TNode<PrimitiveHeapObject> phi_bb10_17;
  TNode<String> tmp24;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_13, &phi_bb10_14, &phi_bb10_15, &phi_bb10_16, &phi_bb10_17);
    compiler::CodeAssemblerLabel label25(&ca_);
    tmp24 = Cast_String_1(state_, TNode<Context>{p_context}, TNode<Object>{tmp20}, &label25);
    ca_.Goto(&block16, phi_bb10_13, phi_bb10_14, phi_bb10_15, phi_bb10_16, phi_bb10_17);
    if (label25.is_used()) {
      ca_.Bind(&label25);
      ca_.Goto(&block17, phi_bb10_13, phi_bb10_14, phi_bb10_15, phi_bb10_16, phi_bb10_17);
    }
  }

  TNode<FixedArray> phi_bb17_13;
  TNode<IntPtrT> phi_bb17_14;
  TNode<IntPtrT> phi_bb17_15;
  TNode<BoolT> phi_bb17_16;
  TNode<PrimitiveHeapObject> phi_bb17_17;
  TNode<Number> tmp26;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_13, &phi_bb17_14, &phi_bb17_15, &phi_bb17_16, &phi_bb17_17);
    compiler::CodeAssemblerLabel label27(&ca_);
    tmp26 = Cast_Number_0(state_, TNode<Object>{ca_.UncheckedCast<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, Symbol, Undefined>>(tmp20)}, &label27);
    ca_.Goto(&block22, phi_bb17_13, phi_bb17_14, phi_bb17_15, phi_bb17_16, phi_bb17_17);
    if (label27.is_used()) {
      ca_.Bind(&label27);
      ca_.Goto(&block23, phi_bb17_13, phi_bb17_14, phi_bb17_15, phi_bb17_16, phi_bb17_17);
    }
  }

  TNode<FixedArray> phi_bb16_13;
  TNode<IntPtrT> phi_bb16_14;
  TNode<IntPtrT> phi_bb16_15;
  TNode<BoolT> phi_bb16_16;
  TNode<PrimitiveHeapObject> phi_bb16_17;
  TNode<String> tmp28;
  TNode<BoolT> tmp29;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_13, &phi_bb16_14, &phi_bb16_15, &phi_bb16_16, &phi_bb16_17);
    tmp28 = kEmptyString_0(state_);
    tmp29 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp24}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp28});
    ca_.Branch(tmp29, &block18, std::vector<compiler::Node*>{phi_bb16_13, phi_bb16_14, phi_bb16_15, phi_bb16_16, phi_bb16_17}, &block19, std::vector<compiler::Node*>{phi_bb16_13, phi_bb16_14, phi_bb16_15, phi_bb16_16, phi_bb16_17});
  }

  TNode<FixedArray> phi_bb18_13;
  TNode<IntPtrT> phi_bb18_14;
  TNode<IntPtrT> phi_bb18_15;
  TNode<BoolT> phi_bb18_16;
  TNode<PrimitiveHeapObject> phi_bb18_17;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_13, &phi_bb18_14, &phi_bb18_15, &phi_bb18_16, &phi_bb18_17);
    ca_.Goto(&block4, phi_bb8_10, phi_bb6_11, phi_bb18_13, phi_bb18_14, phi_bb18_15, phi_bb18_16, phi_bb18_17, tmp19);
  }

  TNode<FixedArray> phi_bb19_13;
  TNode<IntPtrT> phi_bb19_14;
  TNode<IntPtrT> phi_bb19_15;
  TNode<BoolT> phi_bb19_16;
  TNode<PrimitiveHeapObject> phi_bb19_17;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_13, &phi_bb19_14, &phi_bb19_15, &phi_bb19_16, &phi_bb19_17);
    ca_.Goto(&block14, phi_bb19_13, phi_bb19_14, phi_bb19_15, phi_bb19_16, phi_bb19_17, tmp24);
  }

  TNode<FixedArray> phi_bb23_13;
  TNode<IntPtrT> phi_bb23_14;
  TNode<IntPtrT> phi_bb23_15;
  TNode<BoolT> phi_bb23_16;
  TNode<PrimitiveHeapObject> phi_bb23_17;
  TNode<BoolT> tmp30;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_13, &phi_bb23_14, &phi_bb23_15, &phi_bb23_16, &phi_bb23_17);
    tmp30 = CodeStubAssembler(state_).IsNullOrUndefined(TNode<Object>{ca_.UncheckedCast<Union<BigInt, Boolean, JSReceiver, Null, Symbol, Undefined>>(tmp20)});
    ca_.Branch(tmp30, &block24, std::vector<compiler::Node*>{phi_bb23_13, phi_bb23_14, phi_bb23_15, phi_bb23_16, phi_bb23_17}, &block25, std::vector<compiler::Node*>{phi_bb23_13, phi_bb23_14, phi_bb23_15, phi_bb23_16, phi_bb23_17});
  }

  TNode<FixedArray> phi_bb22_13;
  TNode<IntPtrT> phi_bb22_14;
  TNode<IntPtrT> phi_bb22_15;
  TNode<BoolT> phi_bb22_16;
  TNode<PrimitiveHeapObject> phi_bb22_17;
  TNode<String> tmp31;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_13, &phi_bb22_14, &phi_bb22_15, &phi_bb22_16, &phi_bb22_17);
    tmp31 = CodeStubAssembler(state_).NumberToString(TNode<Number>{tmp26});
    ca_.Goto(&block20, phi_bb22_13, phi_bb22_14, phi_bb22_15, phi_bb22_16, phi_bb22_17, tmp31);
  }

  TNode<FixedArray> phi_bb24_13;
  TNode<IntPtrT> phi_bb24_14;
  TNode<IntPtrT> phi_bb24_15;
  TNode<BoolT> phi_bb24_16;
  TNode<PrimitiveHeapObject> phi_bb24_17;
  if (block24.is_used()) {
    ca_.Bind(&block24, &phi_bb24_13, &phi_bb24_14, &phi_bb24_15, &phi_bb24_16, &phi_bb24_17);
    ca_.Goto(&block4, phi_bb8_10, phi_bb6_11, phi_bb24_13, phi_bb24_14, phi_bb24_15, phi_bb24_16, phi_bb24_17, tmp19);
  }

  TNode<FixedArray> phi_bb25_13;
  TNode<IntPtrT> phi_bb25_14;
  TNode<IntPtrT> phi_bb25_15;
  TNode<BoolT> phi_bb25_16;
  TNode<PrimitiveHeapObject> phi_bb25_17;
  TNode<String> tmp32;
  TNode<String> tmp33;
  TNode<BoolT> tmp34;
  if (block25.is_used()) {
    ca_.Bind(&block25, &phi_bb25_13, &phi_bb25_14, &phi_bb25_15, &phi_bb25_16, &phi_bb25_17);
    tmp32 = ToString_Inline_0(state_, TNode<Context>{p_context}, TNode<JSAny>{ca_.UncheckedCast<Union<BigInt, Boolean, JSReceiver, Null, Symbol, Undefined>>(tmp20)});
    tmp33 = kEmptyString_0(state_);
    tmp34 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp32}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp33});
    ca_.Branch(tmp34, &block26, std::vector<compiler::Node*>{phi_bb25_13, phi_bb25_14, phi_bb25_15, phi_bb25_16, phi_bb25_17}, &block27, std::vector<compiler::Node*>{phi_bb25_13, phi_bb25_14, phi_bb25_15, phi_bb25_16, phi_bb25_17});
  }

  TNode<FixedArray> phi_bb26_13;
  TNode<IntPtrT> phi_bb26_14;
  TNode<IntPtrT> phi_bb26_15;
  TNode<BoolT> phi_bb26_16;
  TNode<PrimitiveHeapObject> phi_bb26_17;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_13, &phi_bb26_14, &phi_bb26_15, &phi_bb26_16, &phi_bb26_17);
    ca_.Goto(&block4, phi_bb8_10, phi_bb6_11, phi_bb26_13, phi_bb26_14, phi_bb26_15, phi_bb26_16, phi_bb26_17, tmp19);
  }

  TNode<FixedArray> phi_bb27_13;
  TNode<IntPtrT> phi_bb27_14;
  TNode<IntPtrT> phi_bb27_15;
  TNode<BoolT> phi_bb27_16;
  TNode<PrimitiveHeapObject> phi_bb27_17;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_13, &phi_bb27_14, &phi_bb27_15, &phi_bb27_16, &phi_bb27_17);
    ca_.Goto(&block20, phi_bb27_13, phi_bb27_14, phi_bb27_15, phi_bb27_16, phi_bb27_17, tmp32);
  }

  TNode<FixedArray> phi_bb20_13;
  TNode<IntPtrT> phi_bb20_14;
  TNode<IntPtrT> phi_bb20_15;
  TNode<BoolT> phi_bb20_16;
  TNode<PrimitiveHeapObject> phi_bb20_17;
  TNode<String> phi_bb20_20;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_13, &phi_bb20_14, &phi_bb20_15, &phi_bb20_16, &phi_bb20_17, &phi_bb20_20);
    ca_.Goto(&block14, phi_bb20_13, phi_bb20_14, phi_bb20_15, phi_bb20_16, phi_bb20_17, phi_bb20_20);
  }

  TNode<FixedArray> phi_bb14_13;
  TNode<IntPtrT> phi_bb14_14;
  TNode<IntPtrT> phi_bb14_15;
  TNode<BoolT> phi_bb14_16;
  TNode<PrimitiveHeapObject> phi_bb14_17;
  TNode<String> phi_bb14_20;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_13, &phi_bb14_14, &phi_bb14_15, &phi_bb14_16, &phi_bb14_17, &phi_bb14_20);
    ca_.Goto(&block11, phi_bb14_13, phi_bb14_14, phi_bb14_15, phi_bb14_16, phi_bb14_17, phi_bb14_20);
  }

  TNode<FixedArray> phi_bb11_13;
  TNode<IntPtrT> phi_bb11_14;
  TNode<IntPtrT> phi_bb11_15;
  TNode<BoolT> phi_bb11_16;
  TNode<PrimitiveHeapObject> phi_bb11_17;
  TNode<String> phi_bb11_20;
  TNode<IntPtrT> tmp35;
  TNode<BoolT> tmp36;
  TNode<IntPtrT> tmp37;
  TNode<BoolT> tmp38;
  TNode<BoolT> tmp39;
  TNode<IntPtrT> tmp40;
  TNode<BoolT> tmp41;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_13, &phi_bb11_14, &phi_bb11_15, &phi_bb11_16, &phi_bb11_17, &phi_bb11_20);
    tmp35 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp36 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb11_14}, TNode<IntPtrT>{tmp35});
    tmp37 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp38 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb8_10}, TNode<IntPtrT>{tmp37});
    tmp39 = CodeStubAssembler(state_).Word32Or(TNode<BoolT>{tmp36}, TNode<BoolT>{tmp38});
    tmp40 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp41 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb8_10}, TNode<IntPtrT>{tmp40});
    ca_.Branch(tmp41, &block39, std::vector<compiler::Node*>{phi_bb11_13, phi_bb11_14, phi_bb11_15, phi_bb11_16, phi_bb11_17, phi_bb11_20, phi_bb11_20, phi_bb11_20}, &block40, std::vector<compiler::Node*>{phi_bb11_13, phi_bb11_14, phi_bb11_15, phi_bb11_16, phi_bb11_17, phi_bb11_20, phi_bb11_20, phi_bb11_20});
  }

  TNode<FixedArray> phi_bb39_13;
  TNode<IntPtrT> phi_bb39_14;
  TNode<IntPtrT> phi_bb39_15;
  TNode<BoolT> phi_bb39_16;
  TNode<PrimitiveHeapObject> phi_bb39_17;
  TNode<String> phi_bb39_20;
  TNode<String> phi_bb39_21;
  TNode<String> phi_bb39_26;
  TNode<BoolT> tmp42;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_13, &phi_bb39_14, &phi_bb39_15, &phi_bb39_16, &phi_bb39_17, &phi_bb39_20, &phi_bb39_21, &phi_bb39_26);
    tmp42 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block41, phi_bb39_13, phi_bb39_14, phi_bb39_15, phi_bb39_16, phi_bb39_17, phi_bb39_20, phi_bb39_21, phi_bb39_26, tmp42);
  }

  TNode<FixedArray> phi_bb40_13;
  TNode<IntPtrT> phi_bb40_14;
  TNode<IntPtrT> phi_bb40_15;
  TNode<BoolT> phi_bb40_16;
  TNode<PrimitiveHeapObject> phi_bb40_17;
  TNode<String> phi_bb40_20;
  TNode<String> phi_bb40_21;
  TNode<String> phi_bb40_26;
  TNode<IntPtrT> tmp43;
  TNode<BoolT> tmp44;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_13, &phi_bb40_14, &phi_bb40_15, &phi_bb40_16, &phi_bb40_17, &phi_bb40_20, &phi_bb40_21, &phi_bb40_26);
    tmp43 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp44 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp43});
    ca_.Goto(&block41, phi_bb40_13, phi_bb40_14, phi_bb40_15, phi_bb40_16, phi_bb40_17, phi_bb40_20, phi_bb40_21, phi_bb40_26, tmp44);
  }

  TNode<FixedArray> phi_bb41_13;
  TNode<IntPtrT> phi_bb41_14;
  TNode<IntPtrT> phi_bb41_15;
  TNode<BoolT> phi_bb41_16;
  TNode<PrimitiveHeapObject> phi_bb41_17;
  TNode<String> phi_bb41_20;
  TNode<String> phi_bb41_21;
  TNode<String> phi_bb41_26;
  TNode<BoolT> phi_bb41_39;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_13, &phi_bb41_14, &phi_bb41_15, &phi_bb41_16, &phi_bb41_17, &phi_bb41_20, &phi_bb41_21, &phi_bb41_26, &phi_bb41_39);
    ca_.Branch(phi_bb41_39, &block37, std::vector<compiler::Node*>{phi_bb41_13, phi_bb41_14, phi_bb41_15, phi_bb41_16, phi_bb41_17, phi_bb41_20, phi_bb41_21, phi_bb41_26}, &block38, std::vector<compiler::Node*>{phi_bb41_13, phi_bb41_14, phi_bb41_15, phi_bb41_16, phi_bb41_17, phi_bb41_20, phi_bb41_21, phi_bb41_26});
  }

  TNode<FixedArray> phi_bb37_13;
  TNode<IntPtrT> phi_bb37_14;
  TNode<IntPtrT> phi_bb37_15;
  TNode<BoolT> phi_bb37_16;
  TNode<PrimitiveHeapObject> phi_bb37_17;
  TNode<String> phi_bb37_20;
  TNode<String> phi_bb37_21;
  TNode<String> phi_bb37_26;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_13, &phi_bb37_14, &phi_bb37_15, &phi_bb37_16, &phi_bb37_17, &phi_bb37_20, &phi_bb37_21, &phi_bb37_26);
    ca_.Goto(&block36, phi_bb37_13, phi_bb37_14, phi_bb37_15, phi_bb37_16, phi_bb37_17, phi_bb37_20, phi_bb37_21, phi_bb37_26);
  }

  TNode<FixedArray> phi_bb38_13;
  TNode<IntPtrT> phi_bb38_14;
  TNode<IntPtrT> phi_bb38_15;
  TNode<BoolT> phi_bb38_16;
  TNode<PrimitiveHeapObject> phi_bb38_17;
  TNode<String> phi_bb38_20;
  TNode<String> phi_bb38_21;
  TNode<String> phi_bb38_26;
  TNode<IntPtrT> tmp45;
  TNode<IntPtrT> tmp46;
  TNode<BoolT> tmp47;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_13, &phi_bb38_14, &phi_bb38_15, &phi_bb38_16, &phi_bb38_17, &phi_bb38_20, &phi_bb38_21, &phi_bb38_26);
    tmp45 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{phi_bb8_10});
    tmp46 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp45}, TNode<IntPtrT>{tmp3});
    tmp47 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{tmp46}, TNode<IntPtrT>{phi_bb8_10});
    ca_.Branch(tmp47, &block42, std::vector<compiler::Node*>{phi_bb38_13, phi_bb38_14, phi_bb38_15, phi_bb38_16, phi_bb38_17, phi_bb38_20, phi_bb38_21, phi_bb38_26}, &block43, std::vector<compiler::Node*>{phi_bb38_13, phi_bb38_14, phi_bb38_15, phi_bb38_16, phi_bb38_17, phi_bb38_20, phi_bb38_21, phi_bb38_26});
  }

  TNode<FixedArray> phi_bb42_13;
  TNode<IntPtrT> phi_bb42_14;
  TNode<IntPtrT> phi_bb42_15;
  TNode<BoolT> phi_bb42_16;
  TNode<PrimitiveHeapObject> phi_bb42_17;
  TNode<String> phi_bb42_20;
  TNode<String> phi_bb42_21;
  TNode<String> phi_bb42_26;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_13, &phi_bb42_14, &phi_bb42_15, &phi_bb42_16, &phi_bb42_17, &phi_bb42_20, &phi_bb42_21, &phi_bb42_26);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowInvalidStringLength, p_context);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb43_13;
  TNode<IntPtrT> phi_bb43_14;
  TNode<IntPtrT> phi_bb43_15;
  TNode<BoolT> phi_bb43_16;
  TNode<PrimitiveHeapObject> phi_bb43_17;
  TNode<String> phi_bb43_20;
  TNode<String> phi_bb43_21;
  TNode<String> phi_bb43_26;
  TNode<IntPtrT> tmp48;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_13, &phi_bb43_14, &phi_bb43_15, &phi_bb43_16, &phi_bb43_17, &phi_bb43_20, &phi_bb43_21, &phi_bb43_26);
    tmp48 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb43_15}, TNode<IntPtrT>{tmp45});
    ca_.Branch(tmp39, &block44, std::vector<compiler::Node*>{phi_bb43_13, phi_bb43_14, phi_bb43_16, phi_bb43_17, phi_bb43_20, phi_bb43_21, phi_bb43_26}, &block45, std::vector<compiler::Node*>{phi_bb43_13, phi_bb43_14, phi_bb43_16, phi_bb43_17, phi_bb43_20, phi_bb43_21, phi_bb43_26});
  }

  TNode<FixedArray> phi_bb44_13;
  TNode<IntPtrT> phi_bb44_14;
  TNode<BoolT> phi_bb44_16;
  TNode<PrimitiveHeapObject> phi_bb44_17;
  TNode<String> phi_bb44_20;
  TNode<String> phi_bb44_21;
  TNode<String> phi_bb44_26;
  TNode<Smi> tmp49;
  TNode<IntPtrT> tmp50;
  TNode<BoolT> tmp51;
  if (block44.is_used()) {
    ca_.Bind(&block44, &phi_bb44_13, &phi_bb44_14, &phi_bb44_16, &phi_bb44_17, &phi_bb44_20, &phi_bb44_21, &phi_bb44_26);
    tmp49 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{phi_bb8_10});
    tmp50 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb44_13});
    tmp51 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb44_14}, TNode<IntPtrT>{tmp50});
    ca_.Branch(tmp51, &block55, std::vector<compiler::Node*>{phi_bb44_13, phi_bb44_14, phi_bb44_16, phi_bb44_17, phi_bb44_20, phi_bb44_21, phi_bb44_26}, &block56, std::vector<compiler::Node*>{phi_bb44_13, phi_bb44_14, phi_bb44_16, phi_bb44_17, phi_bb44_20, phi_bb44_21, phi_bb44_26});
  }

  TNode<FixedArray> phi_bb55_13;
  TNode<IntPtrT> phi_bb55_14;
  TNode<BoolT> phi_bb55_16;
  TNode<PrimitiveHeapObject> phi_bb55_17;
  TNode<String> phi_bb55_20;
  TNode<String> phi_bb55_21;
  TNode<String> phi_bb55_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp52;
  TNode<IntPtrT> tmp53;
  TNode<IntPtrT> tmp54;
  TNode<IntPtrT> tmp55;
  TNode<IntPtrT> tmp56;
  TNode<UintPtrT> tmp57;
  TNode<UintPtrT> tmp58;
  TNode<BoolT> tmp59;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_13, &phi_bb55_14, &phi_bb55_16, &phi_bb55_17, &phi_bb55_20, &phi_bb55_21, &phi_bb55_26);
    std::tie(tmp52, tmp53, tmp54) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb55_13}).Flatten();
    tmp55 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp56 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb55_14}, TNode<IntPtrT>{tmp55});
    tmp57 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb55_14});
    tmp58 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp54});
    tmp59 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp57}, TNode<UintPtrT>{tmp58});
    ca_.Branch(tmp59, &block62, std::vector<compiler::Node*>{phi_bb55_13, phi_bb55_16, phi_bb55_17, phi_bb55_20, phi_bb55_21, phi_bb55_26, phi_bb55_13, phi_bb55_14, phi_bb55_14, phi_bb55_14, phi_bb55_14}, &block63, std::vector<compiler::Node*>{phi_bb55_13, phi_bb55_16, phi_bb55_17, phi_bb55_20, phi_bb55_21, phi_bb55_26, phi_bb55_13, phi_bb55_14, phi_bb55_14, phi_bb55_14, phi_bb55_14});
  }

  TNode<FixedArray> phi_bb62_13;
  TNode<BoolT> phi_bb62_16;
  TNode<PrimitiveHeapObject> phi_bb62_17;
  TNode<String> phi_bb62_20;
  TNode<String> phi_bb62_21;
  TNode<String> phi_bb62_26;
  TNode<FixedArray> phi_bb62_43;
  TNode<IntPtrT> phi_bb62_47;
  TNode<IntPtrT> phi_bb62_48;
  TNode<IntPtrT> phi_bb62_52;
  TNode<IntPtrT> phi_bb62_53;
  TNode<IntPtrT> tmp60;
  TNode<IntPtrT> tmp61;
  TNode<Union<HeapObject, TaggedIndex>> tmp62;
  TNode<IntPtrT> tmp63;
  if (block62.is_used()) {
    ca_.Bind(&block62, &phi_bb62_13, &phi_bb62_16, &phi_bb62_17, &phi_bb62_20, &phi_bb62_21, &phi_bb62_26, &phi_bb62_43, &phi_bb62_47, &phi_bb62_48, &phi_bb62_52, &phi_bb62_53);
    tmp60 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb62_53});
    tmp61 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp53}, TNode<IntPtrT>{tmp60});
    std::tie(tmp62, tmp63) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp52}, TNode<IntPtrT>{tmp61}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp62, tmp63}, tmp49);
    ca_.Goto(&block57, phi_bb62_13, tmp56, phi_bb62_16, phi_bb62_17, phi_bb62_20, phi_bb62_21, phi_bb62_26);
  }

  TNode<FixedArray> phi_bb63_13;
  TNode<BoolT> phi_bb63_16;
  TNode<PrimitiveHeapObject> phi_bb63_17;
  TNode<String> phi_bb63_20;
  TNode<String> phi_bb63_21;
  TNode<String> phi_bb63_26;
  TNode<FixedArray> phi_bb63_43;
  TNode<IntPtrT> phi_bb63_47;
  TNode<IntPtrT> phi_bb63_48;
  TNode<IntPtrT> phi_bb63_52;
  TNode<IntPtrT> phi_bb63_53;
  if (block63.is_used()) {
    ca_.Bind(&block63, &phi_bb63_13, &phi_bb63_16, &phi_bb63_17, &phi_bb63_20, &phi_bb63_21, &phi_bb63_26, &phi_bb63_43, &phi_bb63_47, &phi_bb63_48, &phi_bb63_52, &phi_bb63_53);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb56_13;
  TNode<IntPtrT> phi_bb56_14;
  TNode<BoolT> phi_bb56_16;
  TNode<PrimitiveHeapObject> phi_bb56_17;
  TNode<String> phi_bb56_20;
  TNode<String> phi_bb56_21;
  TNode<String> phi_bb56_26;
  TNode<IntPtrT> tmp64;
  TNode<IntPtrT> tmp65;
  TNode<BoolT> tmp66;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_13, &phi_bb56_14, &phi_bb56_16, &phi_bb56_17, &phi_bb56_20, &phi_bb56_21, &phi_bb56_26);
    tmp64 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp50});
    tmp65 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp66 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp64}, TNode<IntPtrT>{tmp65});
    ca_.Branch(tmp66, &block66, std::vector<compiler::Node*>{phi_bb56_13, phi_bb56_14, phi_bb56_16, phi_bb56_17, phi_bb56_20, phi_bb56_21, phi_bb56_26}, &block67, std::vector<compiler::Node*>{phi_bb56_13, phi_bb56_14, phi_bb56_16, phi_bb56_17, phi_bb56_20, phi_bb56_21, phi_bb56_26});
  }

  TNode<FixedArray> phi_bb66_13;
  TNode<IntPtrT> phi_bb66_14;
  TNode<BoolT> phi_bb66_16;
  TNode<PrimitiveHeapObject> phi_bb66_17;
  TNode<String> phi_bb66_20;
  TNode<String> phi_bb66_21;
  TNode<String> phi_bb66_26;
  TNode<IntPtrT> tmp67;
  if (block66.is_used()) {
    ca_.Bind(&block66, &phi_bb66_13, &phi_bb66_14, &phi_bb66_16, &phi_bb66_17, &phi_bb66_20, &phi_bb66_21, &phi_bb66_26);
    tmp67 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block68, phi_bb66_13, phi_bb66_14, phi_bb66_16, phi_bb66_17, phi_bb66_20, phi_bb66_21, phi_bb66_26, tmp67);
  }

  TNode<FixedArray> phi_bb67_13;
  TNode<IntPtrT> phi_bb67_14;
  TNode<BoolT> phi_bb67_16;
  TNode<PrimitiveHeapObject> phi_bb67_17;
  TNode<String> phi_bb67_20;
  TNode<String> phi_bb67_21;
  TNode<String> phi_bb67_26;
  if (block67.is_used()) {
    ca_.Bind(&block67, &phi_bb67_13, &phi_bb67_14, &phi_bb67_16, &phi_bb67_17, &phi_bb67_20, &phi_bb67_21, &phi_bb67_26);
    ca_.Goto(&block68, phi_bb67_13, phi_bb67_14, phi_bb67_16, phi_bb67_17, phi_bb67_20, phi_bb67_21, phi_bb67_26, tmp64);
  }

  TNode<FixedArray> phi_bb68_13;
  TNode<IntPtrT> phi_bb68_14;
  TNode<BoolT> phi_bb68_16;
  TNode<PrimitiveHeapObject> phi_bb68_17;
  TNode<String> phi_bb68_20;
  TNode<String> phi_bb68_21;
  TNode<String> phi_bb68_26;
  TNode<IntPtrT> phi_bb68_44;
  TNode<FixedArray> tmp68;
  TNode<Union<HeapObject, TaggedIndex>> tmp69;
  TNode<IntPtrT> tmp70;
  TNode<IntPtrT> tmp71;
  TNode<UintPtrT> tmp72;
  TNode<IntPtrT> tmp73;
  TNode<UintPtrT> tmp74;
  TNode<UintPtrT> tmp75;
  TNode<BoolT> tmp76;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_13, &phi_bb68_14, &phi_bb68_16, &phi_bb68_17, &phi_bb68_20, &phi_bb68_21, &phi_bb68_26, &phi_bb68_44);
    tmp68 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb68_44});
    std::tie(tmp69, tmp70, tmp71) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb68_13}).Flatten();
    tmp72 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp73 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp72});
    tmp74 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp73});
    tmp75 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp71});
    tmp76 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp74}, TNode<UintPtrT>{tmp75});
    ca_.Branch(tmp76, &block79, std::vector<compiler::Node*>{phi_bb68_13, phi_bb68_14, phi_bb68_16, phi_bb68_17, phi_bb68_20, phi_bb68_21, phi_bb68_26, phi_bb68_13}, &block80, std::vector<compiler::Node*>{phi_bb68_13, phi_bb68_14, phi_bb68_16, phi_bb68_17, phi_bb68_20, phi_bb68_21, phi_bb68_26, phi_bb68_13});
  }

  TNode<FixedArray> phi_bb79_13;
  TNode<IntPtrT> phi_bb79_14;
  TNode<BoolT> phi_bb79_16;
  TNode<PrimitiveHeapObject> phi_bb79_17;
  TNode<String> phi_bb79_20;
  TNode<String> phi_bb79_21;
  TNode<String> phi_bb79_26;
  TNode<FixedArray> phi_bb79_46;
  TNode<IntPtrT> tmp77;
  TNode<IntPtrT> tmp78;
  TNode<Union<HeapObject, TaggedIndex>> tmp79;
  TNode<IntPtrT> tmp80;
  TNode<Union<HeapObject, TaggedIndex>> tmp81;
  TNode<IntPtrT> tmp82;
  TNode<IntPtrT> tmp83;
  TNode<UintPtrT> tmp84;
  TNode<IntPtrT> tmp85;
  TNode<UintPtrT> tmp86;
  TNode<UintPtrT> tmp87;
  TNode<BoolT> tmp88;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_13, &phi_bb79_14, &phi_bb79_16, &phi_bb79_17, &phi_bb79_20, &phi_bb79_21, &phi_bb79_26, &phi_bb79_46);
    tmp77 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp73});
    tmp78 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp70}, TNode<IntPtrT>{tmp77});
    std::tie(tmp79, tmp80) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp69}, TNode<IntPtrT>{tmp78}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp79, tmp80}, tmp68);
    std::tie(tmp81, tmp82, tmp83) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp68}).Flatten();
    tmp84 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp85 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp84});
    tmp86 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp85});
    tmp87 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp83});
    tmp88 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp86}, TNode<UintPtrT>{tmp87});
    ca_.Branch(tmp88, &block88, std::vector<compiler::Node*>{phi_bb79_13, phi_bb79_14, phi_bb79_16, phi_bb79_17, phi_bb79_20, phi_bb79_21, phi_bb79_26}, &block89, std::vector<compiler::Node*>{phi_bb79_13, phi_bb79_14, phi_bb79_16, phi_bb79_17, phi_bb79_20, phi_bb79_21, phi_bb79_26});
  }

  TNode<FixedArray> phi_bb80_13;
  TNode<IntPtrT> phi_bb80_14;
  TNode<BoolT> phi_bb80_16;
  TNode<PrimitiveHeapObject> phi_bb80_17;
  TNode<String> phi_bb80_20;
  TNode<String> phi_bb80_21;
  TNode<String> phi_bb80_26;
  TNode<FixedArray> phi_bb80_46;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_13, &phi_bb80_14, &phi_bb80_16, &phi_bb80_17, &phi_bb80_20, &phi_bb80_21, &phi_bb80_26, &phi_bb80_46);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb88_13;
  TNode<IntPtrT> phi_bb88_14;
  TNode<BoolT> phi_bb88_16;
  TNode<PrimitiveHeapObject> phi_bb88_17;
  TNode<String> phi_bb88_20;
  TNode<String> phi_bb88_21;
  TNode<String> phi_bb88_26;
  TNode<IntPtrT> tmp89;
  TNode<IntPtrT> tmp90;
  TNode<Union<HeapObject, TaggedIndex>> tmp91;
  TNode<IntPtrT> tmp92;
  TNode<Undefined> tmp93;
  TNode<Union<HeapObject, TaggedIndex>> tmp94;
  TNode<IntPtrT> tmp95;
  TNode<IntPtrT> tmp96;
  TNode<UintPtrT> tmp97;
  TNode<IntPtrT> tmp98;
  TNode<UintPtrT> tmp99;
  TNode<UintPtrT> tmp100;
  TNode<BoolT> tmp101;
  if (block88.is_used()) {
    ca_.Bind(&block88, &phi_bb88_13, &phi_bb88_14, &phi_bb88_16, &phi_bb88_17, &phi_bb88_20, &phi_bb88_21, &phi_bb88_26);
    tmp89 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp85});
    tmp90 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp82}, TNode<IntPtrT>{tmp89});
    std::tie(tmp91, tmp92) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp81}, TNode<IntPtrT>{tmp90}).Flatten();
    tmp93 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp91, tmp92}, tmp93);
    std::tie(tmp94, tmp95, tmp96) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp68}).Flatten();
    tmp97 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp98 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp97});
    tmp99 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp98});
    tmp100 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp96});
    tmp101 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp99}, TNode<UintPtrT>{tmp100});
    ca_.Branch(tmp101, &block97, std::vector<compiler::Node*>{phi_bb88_13, phi_bb88_14, phi_bb88_16, phi_bb88_17, phi_bb88_20, phi_bb88_21, phi_bb88_26}, &block98, std::vector<compiler::Node*>{phi_bb88_13, phi_bb88_14, phi_bb88_16, phi_bb88_17, phi_bb88_20, phi_bb88_21, phi_bb88_26});
  }

  TNode<FixedArray> phi_bb89_13;
  TNode<IntPtrT> phi_bb89_14;
  TNode<BoolT> phi_bb89_16;
  TNode<PrimitiveHeapObject> phi_bb89_17;
  TNode<String> phi_bb89_20;
  TNode<String> phi_bb89_21;
  TNode<String> phi_bb89_26;
  if (block89.is_used()) {
    ca_.Bind(&block89, &phi_bb89_13, &phi_bb89_14, &phi_bb89_16, &phi_bb89_17, &phi_bb89_20, &phi_bb89_21, &phi_bb89_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb97_13;
  TNode<IntPtrT> phi_bb97_14;
  TNode<BoolT> phi_bb97_16;
  TNode<PrimitiveHeapObject> phi_bb97_17;
  TNode<String> phi_bb97_20;
  TNode<String> phi_bb97_21;
  TNode<String> phi_bb97_26;
  TNode<IntPtrT> tmp102;
  TNode<IntPtrT> tmp103;
  TNode<Union<HeapObject, TaggedIndex>> tmp104;
  TNode<IntPtrT> tmp105;
  TNode<IntPtrT> tmp106;
  if (block97.is_used()) {
    ca_.Bind(&block97, &phi_bb97_13, &phi_bb97_14, &phi_bb97_16, &phi_bb97_17, &phi_bb97_20, &phi_bb97_21, &phi_bb97_26);
    tmp102 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp98});
    tmp103 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp95}, TNode<IntPtrT>{tmp102});
    std::tie(tmp104, tmp105) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp94}, TNode<IntPtrT>{tmp103}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp104, tmp105}, tmp49);
    tmp106 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block57, tmp68, tmp106, phi_bb97_16, phi_bb97_17, phi_bb97_20, phi_bb97_21, phi_bb97_26);
  }

  TNode<FixedArray> phi_bb98_13;
  TNode<IntPtrT> phi_bb98_14;
  TNode<BoolT> phi_bb98_16;
  TNode<PrimitiveHeapObject> phi_bb98_17;
  TNode<String> phi_bb98_20;
  TNode<String> phi_bb98_21;
  TNode<String> phi_bb98_26;
  if (block98.is_used()) {
    ca_.Bind(&block98, &phi_bb98_13, &phi_bb98_14, &phi_bb98_16, &phi_bb98_17, &phi_bb98_20, &phi_bb98_21, &phi_bb98_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb57_13;
  TNode<IntPtrT> phi_bb57_14;
  TNode<BoolT> phi_bb57_16;
  TNode<PrimitiveHeapObject> phi_bb57_17;
  TNode<String> phi_bb57_20;
  TNode<String> phi_bb57_21;
  TNode<String> phi_bb57_26;
  TNode<Null> tmp107;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_13, &phi_bb57_14, &phi_bb57_16, &phi_bb57_17, &phi_bb57_20, &phi_bb57_21, &phi_bb57_26);
    tmp107 = Null_0(state_);
    ca_.Goto(&block45, phi_bb57_13, phi_bb57_14, phi_bb57_16, tmp107, phi_bb57_20, phi_bb57_21, phi_bb57_26);
  }

  TNode<FixedArray> phi_bb45_13;
  TNode<IntPtrT> phi_bb45_14;
  TNode<BoolT> phi_bb45_16;
  TNode<PrimitiveHeapObject> phi_bb45_17;
  TNode<String> phi_bb45_20;
  TNode<String> phi_bb45_21;
  TNode<String> phi_bb45_26;
  if (block45.is_used()) {
    ca_.Bind(&block45, &phi_bb45_13, &phi_bb45_14, &phi_bb45_16, &phi_bb45_17, &phi_bb45_20, &phi_bb45_21, &phi_bb45_26);
    ca_.Goto(&block36, phi_bb45_13, phi_bb45_14, tmp48, phi_bb45_16, phi_bb45_17, phi_bb45_20, phi_bb45_21, phi_bb45_26);
  }

  TNode<FixedArray> phi_bb36_13;
  TNode<IntPtrT> phi_bb36_14;
  TNode<IntPtrT> phi_bb36_15;
  TNode<BoolT> phi_bb36_16;
  TNode<PrimitiveHeapObject> phi_bb36_17;
  TNode<String> phi_bb36_20;
  TNode<String> phi_bb36_21;
  TNode<String> phi_bb36_26;
  TNode<IntPtrT> tmp108;
  TNode<IntPtrT> tmp109;
  TNode<BoolT> tmp110;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_13, &phi_bb36_14, &phi_bb36_15, &phi_bb36_16, &phi_bb36_17, &phi_bb36_20, &phi_bb36_21, &phi_bb36_26);
    tmp108 = CodeStubAssembler(state_).LoadStringLengthAsWord(TNode<String>{phi_bb36_26});
    tmp109 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb36_15}, TNode<IntPtrT>{tmp108});
    tmp110 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<HeapObject, Smi, Weak<HeapObject>>>{phi_bb36_26}, TNode<Union<HeapObject, Smi, Weak<HeapObject>>>{phi_bb36_17});
    ca_.Branch(tmp110, &block101, std::vector<compiler::Node*>{phi_bb36_13, phi_bb36_14, phi_bb36_16, phi_bb36_17, phi_bb36_20, phi_bb36_21, phi_bb36_26}, &block102, std::vector<compiler::Node*>{phi_bb36_13, phi_bb36_14, phi_bb36_16, phi_bb36_17, phi_bb36_20, phi_bb36_21, phi_bb36_26});
  }

  TNode<FixedArray> phi_bb101_13;
  TNode<IntPtrT> phi_bb101_14;
  TNode<BoolT> phi_bb101_16;
  TNode<PrimitiveHeapObject> phi_bb101_17;
  TNode<String> phi_bb101_20;
  TNode<String> phi_bb101_21;
  TNode<String> phi_bb101_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp111;
  TNode<IntPtrT> tmp112;
  TNode<IntPtrT> tmp113;
  TNode<IntPtrT> tmp114;
  TNode<IntPtrT> tmp115;
  TNode<UintPtrT> tmp116;
  TNode<UintPtrT> tmp117;
  TNode<BoolT> tmp118;
  if (block101.is_used()) {
    ca_.Bind(&block101, &phi_bb101_13, &phi_bb101_14, &phi_bb101_16, &phi_bb101_17, &phi_bb101_20, &phi_bb101_21, &phi_bb101_26);
    std::tie(tmp111, tmp112, tmp113) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb101_13}).Flatten();
    tmp114 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp115 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb101_14}, TNode<IntPtrT>{tmp114});
    tmp116 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp115});
    tmp117 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp113});
    tmp118 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp116}, TNode<UintPtrT>{tmp117});
    ca_.Branch(tmp118, &block113, std::vector<compiler::Node*>{phi_bb101_13, phi_bb101_14, phi_bb101_16, phi_bb101_17, phi_bb101_20, phi_bb101_21, phi_bb101_26, phi_bb101_13}, &block114, std::vector<compiler::Node*>{phi_bb101_13, phi_bb101_14, phi_bb101_16, phi_bb101_17, phi_bb101_20, phi_bb101_21, phi_bb101_26, phi_bb101_13});
  }

  TNode<FixedArray> phi_bb113_13;
  TNode<IntPtrT> phi_bb113_14;
  TNode<BoolT> phi_bb113_16;
  TNode<PrimitiveHeapObject> phi_bb113_17;
  TNode<String> phi_bb113_20;
  TNode<String> phi_bb113_21;
  TNode<String> phi_bb113_26;
  TNode<FixedArray> phi_bb113_30;
  TNode<IntPtrT> tmp119;
  TNode<IntPtrT> tmp120;
  TNode<Union<HeapObject, TaggedIndex>> tmp121;
  TNode<IntPtrT> tmp122;
  TNode<Object> tmp123;
  TNode<HeapObject> tmp124;
  if (block113.is_used()) {
    ca_.Bind(&block113, &phi_bb113_13, &phi_bb113_14, &phi_bb113_16, &phi_bb113_17, &phi_bb113_20, &phi_bb113_21, &phi_bb113_26, &phi_bb113_30);
    tmp119 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp115});
    tmp120 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp112}, TNode<IntPtrT>{tmp119});
    std::tie(tmp121, tmp122) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp111}, TNode<IntPtrT>{tmp120}).Flatten();
    tmp123 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp121, tmp122});
    compiler::CodeAssemblerLabel label125(&ca_);
    tmp124 = CodeStubAssembler(state_).TaggedToHeapObject(TNode<Object>{tmp123}, &label125);
    ca_.Goto(&block120, phi_bb113_13, phi_bb113_14, phi_bb113_16, phi_bb113_17, phi_bb113_20, phi_bb113_21, phi_bb113_26);
    if (label125.is_used()) {
      ca_.Bind(&label125);
      ca_.Goto(&block121, phi_bb113_13, phi_bb113_14, phi_bb113_16, phi_bb113_17, phi_bb113_20, phi_bb113_21, phi_bb113_26);
    }
  }

  TNode<FixedArray> phi_bb114_13;
  TNode<IntPtrT> phi_bb114_14;
  TNode<BoolT> phi_bb114_16;
  TNode<PrimitiveHeapObject> phi_bb114_17;
  TNode<String> phi_bb114_20;
  TNode<String> phi_bb114_21;
  TNode<String> phi_bb114_26;
  TNode<FixedArray> phi_bb114_30;
  if (block114.is_used()) {
    ca_.Bind(&block114, &phi_bb114_13, &phi_bb114_14, &phi_bb114_16, &phi_bb114_17, &phi_bb114_20, &phi_bb114_21, &phi_bb114_26, &phi_bb114_30);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb121_13;
  TNode<IntPtrT> phi_bb121_14;
  TNode<BoolT> phi_bb121_16;
  TNode<PrimitiveHeapObject> phi_bb121_17;
  TNode<String> phi_bb121_20;
  TNode<String> phi_bb121_21;
  TNode<String> phi_bb121_26;
  if (block121.is_used()) {
    ca_.Bind(&block121, &phi_bb121_13, &phi_bb121_14, &phi_bb121_16, &phi_bb121_17, &phi_bb121_20, &phi_bb121_21, &phi_bb121_26);
    ca_.Goto(&block118, phi_bb121_13, phi_bb121_14, phi_bb121_16, phi_bb121_17, phi_bb121_20, phi_bb121_21, phi_bb121_26);
  }

  TNode<FixedArray> phi_bb120_13;
  TNode<IntPtrT> phi_bb120_14;
  TNode<BoolT> phi_bb120_16;
  TNode<PrimitiveHeapObject> phi_bb120_17;
  TNode<String> phi_bb120_20;
  TNode<String> phi_bb120_21;
  TNode<String> phi_bb120_26;
  TNode<String> tmp126;
  if (block120.is_used()) {
    ca_.Bind(&block120, &phi_bb120_13, &phi_bb120_14, &phi_bb120_16, &phi_bb120_17, &phi_bb120_20, &phi_bb120_21, &phi_bb120_26);
    compiler::CodeAssemblerLabel label127(&ca_);
    tmp126 = Cast_String_0(state_, TNode<HeapObject>{tmp124}, &label127);
    ca_.Goto(&block122, phi_bb120_13, phi_bb120_14, phi_bb120_16, phi_bb120_17, phi_bb120_20, phi_bb120_21, phi_bb120_26);
    if (label127.is_used()) {
      ca_.Bind(&label127);
      ca_.Goto(&block123, phi_bb120_13, phi_bb120_14, phi_bb120_16, phi_bb120_17, phi_bb120_20, phi_bb120_21, phi_bb120_26);
    }
  }

  TNode<FixedArray> phi_bb123_13;
  TNode<IntPtrT> phi_bb123_14;
  TNode<BoolT> phi_bb123_16;
  TNode<PrimitiveHeapObject> phi_bb123_17;
  TNode<String> phi_bb123_20;
  TNode<String> phi_bb123_21;
  TNode<String> phi_bb123_26;
  if (block123.is_used()) {
    ca_.Bind(&block123, &phi_bb123_13, &phi_bb123_14, &phi_bb123_16, &phi_bb123_17, &phi_bb123_20, &phi_bb123_21, &phi_bb123_26);
    ca_.Goto(&block118, phi_bb123_13, phi_bb123_14, phi_bb123_16, phi_bb123_17, phi_bb123_20, phi_bb123_21, phi_bb123_26);
  }

  TNode<FixedArray> phi_bb122_13;
  TNode<IntPtrT> phi_bb122_14;
  TNode<BoolT> phi_bb122_16;
  TNode<PrimitiveHeapObject> phi_bb122_17;
  TNode<String> phi_bb122_20;
  TNode<String> phi_bb122_21;
  TNode<String> phi_bb122_26;
  TNode<Smi> tmp128;
  TNode<IntPtrT> tmp129;
  TNode<BoolT> tmp130;
  if (block122.is_used()) {
    ca_.Bind(&block122, &phi_bb122_13, &phi_bb122_14, &phi_bb122_16, &phi_bb122_17, &phi_bb122_20, &phi_bb122_21, &phi_bb122_26);
    tmp128 = SmiConstant_0(state_, IntegerLiteral(true, 0x1ull));
    tmp129 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb122_13});
    tmp130 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb122_14}, TNode<IntPtrT>{tmp129});
    ca_.Branch(tmp130, &block133, std::vector<compiler::Node*>{phi_bb122_13, phi_bb122_14, phi_bb122_16, phi_bb122_17, phi_bb122_20, phi_bb122_21, phi_bb122_26}, &block134, std::vector<compiler::Node*>{phi_bb122_13, phi_bb122_14, phi_bb122_16, phi_bb122_17, phi_bb122_20, phi_bb122_21, phi_bb122_26});
  }

  TNode<FixedArray> phi_bb133_13;
  TNode<IntPtrT> phi_bb133_14;
  TNode<BoolT> phi_bb133_16;
  TNode<PrimitiveHeapObject> phi_bb133_17;
  TNode<String> phi_bb133_20;
  TNode<String> phi_bb133_21;
  TNode<String> phi_bb133_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp131;
  TNode<IntPtrT> tmp132;
  TNode<IntPtrT> tmp133;
  TNode<IntPtrT> tmp134;
  TNode<IntPtrT> tmp135;
  TNode<UintPtrT> tmp136;
  TNode<UintPtrT> tmp137;
  TNode<BoolT> tmp138;
  if (block133.is_used()) {
    ca_.Bind(&block133, &phi_bb133_13, &phi_bb133_14, &phi_bb133_16, &phi_bb133_17, &phi_bb133_20, &phi_bb133_21, &phi_bb133_26);
    std::tie(tmp131, tmp132, tmp133) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb133_13}).Flatten();
    tmp134 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp135 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb133_14}, TNode<IntPtrT>{tmp134});
    tmp136 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb133_14});
    tmp137 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp133});
    tmp138 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp136}, TNode<UintPtrT>{tmp137});
    ca_.Branch(tmp138, &block140, std::vector<compiler::Node*>{phi_bb133_13, phi_bb133_16, phi_bb133_17, phi_bb133_20, phi_bb133_21, phi_bb133_26, phi_bb133_13, phi_bb133_14, phi_bb133_14, phi_bb133_14, phi_bb133_14}, &block141, std::vector<compiler::Node*>{phi_bb133_13, phi_bb133_16, phi_bb133_17, phi_bb133_20, phi_bb133_21, phi_bb133_26, phi_bb133_13, phi_bb133_14, phi_bb133_14, phi_bb133_14, phi_bb133_14});
  }

  TNode<FixedArray> phi_bb140_13;
  TNode<BoolT> phi_bb140_16;
  TNode<PrimitiveHeapObject> phi_bb140_17;
  TNode<String> phi_bb140_20;
  TNode<String> phi_bb140_21;
  TNode<String> phi_bb140_26;
  TNode<FixedArray> phi_bb140_35;
  TNode<IntPtrT> phi_bb140_39;
  TNode<IntPtrT> phi_bb140_40;
  TNode<IntPtrT> phi_bb140_44;
  TNode<IntPtrT> phi_bb140_45;
  TNode<IntPtrT> tmp139;
  TNode<IntPtrT> tmp140;
  TNode<Union<HeapObject, TaggedIndex>> tmp141;
  TNode<IntPtrT> tmp142;
  if (block140.is_used()) {
    ca_.Bind(&block140, &phi_bb140_13, &phi_bb140_16, &phi_bb140_17, &phi_bb140_20, &phi_bb140_21, &phi_bb140_26, &phi_bb140_35, &phi_bb140_39, &phi_bb140_40, &phi_bb140_44, &phi_bb140_45);
    tmp139 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb140_45});
    tmp140 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp132}, TNode<IntPtrT>{tmp139});
    std::tie(tmp141, tmp142) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp131}, TNode<IntPtrT>{tmp140}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp141, tmp142}, tmp128);
    ca_.Goto(&block135, phi_bb140_13, tmp135, phi_bb140_16, phi_bb140_17, phi_bb140_20, phi_bb140_21, phi_bb140_26);
  }

  TNode<FixedArray> phi_bb141_13;
  TNode<BoolT> phi_bb141_16;
  TNode<PrimitiveHeapObject> phi_bb141_17;
  TNode<String> phi_bb141_20;
  TNode<String> phi_bb141_21;
  TNode<String> phi_bb141_26;
  TNode<FixedArray> phi_bb141_35;
  TNode<IntPtrT> phi_bb141_39;
  TNode<IntPtrT> phi_bb141_40;
  TNode<IntPtrT> phi_bb141_44;
  TNode<IntPtrT> phi_bb141_45;
  if (block141.is_used()) {
    ca_.Bind(&block141, &phi_bb141_13, &phi_bb141_16, &phi_bb141_17, &phi_bb141_20, &phi_bb141_21, &phi_bb141_26, &phi_bb141_35, &phi_bb141_39, &phi_bb141_40, &phi_bb141_44, &phi_bb141_45);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb134_13;
  TNode<IntPtrT> phi_bb134_14;
  TNode<BoolT> phi_bb134_16;
  TNode<PrimitiveHeapObject> phi_bb134_17;
  TNode<String> phi_bb134_20;
  TNode<String> phi_bb134_21;
  TNode<String> phi_bb134_26;
  TNode<IntPtrT> tmp143;
  TNode<IntPtrT> tmp144;
  TNode<BoolT> tmp145;
  if (block134.is_used()) {
    ca_.Bind(&block134, &phi_bb134_13, &phi_bb134_14, &phi_bb134_16, &phi_bb134_17, &phi_bb134_20, &phi_bb134_21, &phi_bb134_26);
    tmp143 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp129});
    tmp144 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp145 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp143}, TNode<IntPtrT>{tmp144});
    ca_.Branch(tmp145, &block144, std::vector<compiler::Node*>{phi_bb134_13, phi_bb134_14, phi_bb134_16, phi_bb134_17, phi_bb134_20, phi_bb134_21, phi_bb134_26}, &block145, std::vector<compiler::Node*>{phi_bb134_13, phi_bb134_14, phi_bb134_16, phi_bb134_17, phi_bb134_20, phi_bb134_21, phi_bb134_26});
  }

  TNode<FixedArray> phi_bb144_13;
  TNode<IntPtrT> phi_bb144_14;
  TNode<BoolT> phi_bb144_16;
  TNode<PrimitiveHeapObject> phi_bb144_17;
  TNode<String> phi_bb144_20;
  TNode<String> phi_bb144_21;
  TNode<String> phi_bb144_26;
  TNode<IntPtrT> tmp146;
  if (block144.is_used()) {
    ca_.Bind(&block144, &phi_bb144_13, &phi_bb144_14, &phi_bb144_16, &phi_bb144_17, &phi_bb144_20, &phi_bb144_21, &phi_bb144_26);
    tmp146 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block146, phi_bb144_13, phi_bb144_14, phi_bb144_16, phi_bb144_17, phi_bb144_20, phi_bb144_21, phi_bb144_26, tmp146);
  }

  TNode<FixedArray> phi_bb145_13;
  TNode<IntPtrT> phi_bb145_14;
  TNode<BoolT> phi_bb145_16;
  TNode<PrimitiveHeapObject> phi_bb145_17;
  TNode<String> phi_bb145_20;
  TNode<String> phi_bb145_21;
  TNode<String> phi_bb145_26;
  if (block145.is_used()) {
    ca_.Bind(&block145, &phi_bb145_13, &phi_bb145_14, &phi_bb145_16, &phi_bb145_17, &phi_bb145_20, &phi_bb145_21, &phi_bb145_26);
    ca_.Goto(&block146, phi_bb145_13, phi_bb145_14, phi_bb145_16, phi_bb145_17, phi_bb145_20, phi_bb145_21, phi_bb145_26, tmp143);
  }

  TNode<FixedArray> phi_bb146_13;
  TNode<IntPtrT> phi_bb146_14;
  TNode<BoolT> phi_bb146_16;
  TNode<PrimitiveHeapObject> phi_bb146_17;
  TNode<String> phi_bb146_20;
  TNode<String> phi_bb146_21;
  TNode<String> phi_bb146_26;
  TNode<IntPtrT> phi_bb146_36;
  TNode<FixedArray> tmp147;
  TNode<Union<HeapObject, TaggedIndex>> tmp148;
  TNode<IntPtrT> tmp149;
  TNode<IntPtrT> tmp150;
  TNode<UintPtrT> tmp151;
  TNode<IntPtrT> tmp152;
  TNode<UintPtrT> tmp153;
  TNode<UintPtrT> tmp154;
  TNode<BoolT> tmp155;
  if (block146.is_used()) {
    ca_.Bind(&block146, &phi_bb146_13, &phi_bb146_14, &phi_bb146_16, &phi_bb146_17, &phi_bb146_20, &phi_bb146_21, &phi_bb146_26, &phi_bb146_36);
    tmp147 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb146_36});
    std::tie(tmp148, tmp149, tmp150) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb146_13}).Flatten();
    tmp151 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp152 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp151});
    tmp153 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp152});
    tmp154 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp150});
    tmp155 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp153}, TNode<UintPtrT>{tmp154});
    ca_.Branch(tmp155, &block157, std::vector<compiler::Node*>{phi_bb146_13, phi_bb146_14, phi_bb146_16, phi_bb146_17, phi_bb146_20, phi_bb146_21, phi_bb146_26, phi_bb146_13}, &block158, std::vector<compiler::Node*>{phi_bb146_13, phi_bb146_14, phi_bb146_16, phi_bb146_17, phi_bb146_20, phi_bb146_21, phi_bb146_26, phi_bb146_13});
  }

  TNode<FixedArray> phi_bb157_13;
  TNode<IntPtrT> phi_bb157_14;
  TNode<BoolT> phi_bb157_16;
  TNode<PrimitiveHeapObject> phi_bb157_17;
  TNode<String> phi_bb157_20;
  TNode<String> phi_bb157_21;
  TNode<String> phi_bb157_26;
  TNode<FixedArray> phi_bb157_38;
  TNode<IntPtrT> tmp156;
  TNode<IntPtrT> tmp157;
  TNode<Union<HeapObject, TaggedIndex>> tmp158;
  TNode<IntPtrT> tmp159;
  TNode<Union<HeapObject, TaggedIndex>> tmp160;
  TNode<IntPtrT> tmp161;
  TNode<IntPtrT> tmp162;
  TNode<UintPtrT> tmp163;
  TNode<IntPtrT> tmp164;
  TNode<UintPtrT> tmp165;
  TNode<UintPtrT> tmp166;
  TNode<BoolT> tmp167;
  if (block157.is_used()) {
    ca_.Bind(&block157, &phi_bb157_13, &phi_bb157_14, &phi_bb157_16, &phi_bb157_17, &phi_bb157_20, &phi_bb157_21, &phi_bb157_26, &phi_bb157_38);
    tmp156 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp152});
    tmp157 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp149}, TNode<IntPtrT>{tmp156});
    std::tie(tmp158, tmp159) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp148}, TNode<IntPtrT>{tmp157}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp158, tmp159}, tmp147);
    std::tie(tmp160, tmp161, tmp162) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp147}).Flatten();
    tmp163 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp164 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp163});
    tmp165 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp164});
    tmp166 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp162});
    tmp167 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp165}, TNode<UintPtrT>{tmp166});
    ca_.Branch(tmp167, &block166, std::vector<compiler::Node*>{phi_bb157_13, phi_bb157_14, phi_bb157_16, phi_bb157_17, phi_bb157_20, phi_bb157_21, phi_bb157_26}, &block167, std::vector<compiler::Node*>{phi_bb157_13, phi_bb157_14, phi_bb157_16, phi_bb157_17, phi_bb157_20, phi_bb157_21, phi_bb157_26});
  }

  TNode<FixedArray> phi_bb158_13;
  TNode<IntPtrT> phi_bb158_14;
  TNode<BoolT> phi_bb158_16;
  TNode<PrimitiveHeapObject> phi_bb158_17;
  TNode<String> phi_bb158_20;
  TNode<String> phi_bb158_21;
  TNode<String> phi_bb158_26;
  TNode<FixedArray> phi_bb158_38;
  if (block158.is_used()) {
    ca_.Bind(&block158, &phi_bb158_13, &phi_bb158_14, &phi_bb158_16, &phi_bb158_17, &phi_bb158_20, &phi_bb158_21, &phi_bb158_26, &phi_bb158_38);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb166_13;
  TNode<IntPtrT> phi_bb166_14;
  TNode<BoolT> phi_bb166_16;
  TNode<PrimitiveHeapObject> phi_bb166_17;
  TNode<String> phi_bb166_20;
  TNode<String> phi_bb166_21;
  TNode<String> phi_bb166_26;
  TNode<IntPtrT> tmp168;
  TNode<IntPtrT> tmp169;
  TNode<Union<HeapObject, TaggedIndex>> tmp170;
  TNode<IntPtrT> tmp171;
  TNode<Undefined> tmp172;
  TNode<Union<HeapObject, TaggedIndex>> tmp173;
  TNode<IntPtrT> tmp174;
  TNode<IntPtrT> tmp175;
  TNode<UintPtrT> tmp176;
  TNode<IntPtrT> tmp177;
  TNode<UintPtrT> tmp178;
  TNode<UintPtrT> tmp179;
  TNode<BoolT> tmp180;
  if (block166.is_used()) {
    ca_.Bind(&block166, &phi_bb166_13, &phi_bb166_14, &phi_bb166_16, &phi_bb166_17, &phi_bb166_20, &phi_bb166_21, &phi_bb166_26);
    tmp168 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp164});
    tmp169 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp161}, TNode<IntPtrT>{tmp168});
    std::tie(tmp170, tmp171) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp160}, TNode<IntPtrT>{tmp169}).Flatten();
    tmp172 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp170, tmp171}, tmp172);
    std::tie(tmp173, tmp174, tmp175) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp147}).Flatten();
    tmp176 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp177 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp176});
    tmp178 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp177});
    tmp179 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp175});
    tmp180 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp178}, TNode<UintPtrT>{tmp179});
    ca_.Branch(tmp180, &block175, std::vector<compiler::Node*>{phi_bb166_13, phi_bb166_14, phi_bb166_16, phi_bb166_17, phi_bb166_20, phi_bb166_21, phi_bb166_26}, &block176, std::vector<compiler::Node*>{phi_bb166_13, phi_bb166_14, phi_bb166_16, phi_bb166_17, phi_bb166_20, phi_bb166_21, phi_bb166_26});
  }

  TNode<FixedArray> phi_bb167_13;
  TNode<IntPtrT> phi_bb167_14;
  TNode<BoolT> phi_bb167_16;
  TNode<PrimitiveHeapObject> phi_bb167_17;
  TNode<String> phi_bb167_20;
  TNode<String> phi_bb167_21;
  TNode<String> phi_bb167_26;
  if (block167.is_used()) {
    ca_.Bind(&block167, &phi_bb167_13, &phi_bb167_14, &phi_bb167_16, &phi_bb167_17, &phi_bb167_20, &phi_bb167_21, &phi_bb167_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb175_13;
  TNode<IntPtrT> phi_bb175_14;
  TNode<BoolT> phi_bb175_16;
  TNode<PrimitiveHeapObject> phi_bb175_17;
  TNode<String> phi_bb175_20;
  TNode<String> phi_bb175_21;
  TNode<String> phi_bb175_26;
  TNode<IntPtrT> tmp181;
  TNode<IntPtrT> tmp182;
  TNode<Union<HeapObject, TaggedIndex>> tmp183;
  TNode<IntPtrT> tmp184;
  TNode<IntPtrT> tmp185;
  if (block175.is_used()) {
    ca_.Bind(&block175, &phi_bb175_13, &phi_bb175_14, &phi_bb175_16, &phi_bb175_17, &phi_bb175_20, &phi_bb175_21, &phi_bb175_26);
    tmp181 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp177});
    tmp182 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp174}, TNode<IntPtrT>{tmp181});
    std::tie(tmp183, tmp184) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp173}, TNode<IntPtrT>{tmp182}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp183, tmp184}, tmp128);
    tmp185 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block135, tmp147, tmp185, phi_bb175_16, phi_bb175_17, phi_bb175_20, phi_bb175_21, phi_bb175_26);
  }

  TNode<FixedArray> phi_bb176_13;
  TNode<IntPtrT> phi_bb176_14;
  TNode<BoolT> phi_bb176_16;
  TNode<PrimitiveHeapObject> phi_bb176_17;
  TNode<String> phi_bb176_20;
  TNode<String> phi_bb176_21;
  TNode<String> phi_bb176_26;
  if (block176.is_used()) {
    ca_.Bind(&block176, &phi_bb176_13, &phi_bb176_14, &phi_bb176_16, &phi_bb176_17, &phi_bb176_20, &phi_bb176_21, &phi_bb176_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb135_13;
  TNode<IntPtrT> phi_bb135_14;
  TNode<BoolT> phi_bb135_16;
  TNode<PrimitiveHeapObject> phi_bb135_17;
  TNode<String> phi_bb135_20;
  TNode<String> phi_bb135_21;
  TNode<String> phi_bb135_26;
  if (block135.is_used()) {
    ca_.Bind(&block135, &phi_bb135_13, &phi_bb135_14, &phi_bb135_16, &phi_bb135_17, &phi_bb135_20, &phi_bb135_21, &phi_bb135_26);
    ca_.Goto(&block117, phi_bb135_13, phi_bb135_14, phi_bb135_16, phi_bb135_17, phi_bb135_20, phi_bb135_21, phi_bb135_26);
  }

  TNode<FixedArray> phi_bb118_13;
  TNode<IntPtrT> phi_bb118_14;
  TNode<BoolT> phi_bb118_16;
  TNode<PrimitiveHeapObject> phi_bb118_17;
  TNode<String> phi_bb118_20;
  TNode<String> phi_bb118_21;
  TNode<String> phi_bb118_26;
  TNode<Smi> tmp186;
  if (block118.is_used()) {
    ca_.Bind(&block118, &phi_bb118_13, &phi_bb118_14, &phi_bb118_16, &phi_bb118_17, &phi_bb118_20, &phi_bb118_21, &phi_bb118_26);
    compiler::CodeAssemblerLabel label187(&ca_);
    tmp186 = Cast_Smi_0(state_, TNode<Object>{ca_.UncheckedCast<Object>(tmp123)}, &label187);
    ca_.Goto(&block181, phi_bb118_13, phi_bb118_14, phi_bb118_16, phi_bb118_17, phi_bb118_20, phi_bb118_21, phi_bb118_26);
    if (label187.is_used()) {
      ca_.Bind(&label187);
      ca_.Goto(&block182, phi_bb118_13, phi_bb118_14, phi_bb118_16, phi_bb118_17, phi_bb118_20, phi_bb118_21, phi_bb118_26);
    }
  }

  TNode<FixedArray> phi_bb182_13;
  TNode<IntPtrT> phi_bb182_14;
  TNode<BoolT> phi_bb182_16;
  TNode<PrimitiveHeapObject> phi_bb182_17;
  TNode<String> phi_bb182_20;
  TNode<String> phi_bb182_21;
  TNode<String> phi_bb182_26;
  if (block182.is_used()) {
    ca_.Bind(&block182, &phi_bb182_13, &phi_bb182_14, &phi_bb182_16, &phi_bb182_17, &phi_bb182_20, &phi_bb182_21, &phi_bb182_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb181_13;
  TNode<IntPtrT> phi_bb181_14;
  TNode<BoolT> phi_bb181_16;
  TNode<PrimitiveHeapObject> phi_bb181_17;
  TNode<String> phi_bb181_20;
  TNode<String> phi_bb181_21;
  TNode<String> phi_bb181_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp188;
  TNode<IntPtrT> tmp189;
  TNode<IntPtrT> tmp190;
  TNode<IntPtrT> tmp191;
  TNode<IntPtrT> tmp192;
  TNode<UintPtrT> tmp193;
  TNode<UintPtrT> tmp194;
  TNode<BoolT> tmp195;
  if (block181.is_used()) {
    ca_.Bind(&block181, &phi_bb181_13, &phi_bb181_14, &phi_bb181_16, &phi_bb181_17, &phi_bb181_20, &phi_bb181_21, &phi_bb181_26);
    std::tie(tmp188, tmp189, tmp190) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb181_13}).Flatten();
    tmp191 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp192 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb181_14}, TNode<IntPtrT>{tmp191});
    tmp193 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp192});
    tmp194 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp190});
    tmp195 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp193}, TNode<UintPtrT>{tmp194});
    ca_.Branch(tmp195, &block195, std::vector<compiler::Node*>{phi_bb181_13, phi_bb181_14, phi_bb181_16, phi_bb181_17, phi_bb181_20, phi_bb181_21, phi_bb181_26, phi_bb181_13}, &block196, std::vector<compiler::Node*>{phi_bb181_13, phi_bb181_14, phi_bb181_16, phi_bb181_17, phi_bb181_20, phi_bb181_21, phi_bb181_26, phi_bb181_13});
  }

  TNode<FixedArray> phi_bb195_13;
  TNode<IntPtrT> phi_bb195_14;
  TNode<BoolT> phi_bb195_16;
  TNode<PrimitiveHeapObject> phi_bb195_17;
  TNode<String> phi_bb195_20;
  TNode<String> phi_bb195_21;
  TNode<String> phi_bb195_26;
  TNode<FixedArray> phi_bb195_32;
  TNode<IntPtrT> tmp196;
  TNode<IntPtrT> tmp197;
  TNode<Union<HeapObject, TaggedIndex>> tmp198;
  TNode<IntPtrT> tmp199;
  TNode<Smi> tmp200;
  TNode<Smi> tmp201;
  if (block195.is_used()) {
    ca_.Bind(&block195, &phi_bb195_13, &phi_bb195_14, &phi_bb195_16, &phi_bb195_17, &phi_bb195_20, &phi_bb195_21, &phi_bb195_26, &phi_bb195_32);
    tmp196 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp192});
    tmp197 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp189}, TNode<IntPtrT>{tmp196});
    std::tie(tmp198, tmp199) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp188}, TNode<IntPtrT>{tmp197}).Flatten();
    tmp200 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp201 = CodeStubAssembler(state_).SmiSub(TNode<Smi>{tmp186}, TNode<Smi>{tmp200});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp198, tmp199}, tmp201);
    ca_.Goto(&block117, phi_bb195_13, phi_bb195_14, phi_bb195_16, phi_bb195_17, phi_bb195_20, phi_bb195_21, phi_bb195_26);
  }

  TNode<FixedArray> phi_bb196_13;
  TNode<IntPtrT> phi_bb196_14;
  TNode<BoolT> phi_bb196_16;
  TNode<PrimitiveHeapObject> phi_bb196_17;
  TNode<String> phi_bb196_20;
  TNode<String> phi_bb196_21;
  TNode<String> phi_bb196_26;
  TNode<FixedArray> phi_bb196_32;
  if (block196.is_used()) {
    ca_.Bind(&block196, &phi_bb196_13, &phi_bb196_14, &phi_bb196_16, &phi_bb196_17, &phi_bb196_20, &phi_bb196_21, &phi_bb196_26, &phi_bb196_32);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb117_13;
  TNode<IntPtrT> phi_bb117_14;
  TNode<BoolT> phi_bb117_16;
  TNode<PrimitiveHeapObject> phi_bb117_17;
  TNode<String> phi_bb117_20;
  TNode<String> phi_bb117_21;
  TNode<String> phi_bb117_26;
  if (block117.is_used()) {
    ca_.Bind(&block117, &phi_bb117_13, &phi_bb117_14, &phi_bb117_16, &phi_bb117_17, &phi_bb117_20, &phi_bb117_21, &phi_bb117_26);
    ca_.Goto(&block103, phi_bb117_13, phi_bb117_14, phi_bb117_16, phi_bb117_17, phi_bb117_20, phi_bb117_21, phi_bb117_26);
  }

  TNode<FixedArray> phi_bb102_13;
  TNode<IntPtrT> phi_bb102_14;
  TNode<BoolT> phi_bb102_16;
  TNode<PrimitiveHeapObject> phi_bb102_17;
  TNode<String> phi_bb102_20;
  TNode<String> phi_bb102_21;
  TNode<String> phi_bb102_26;
  TNode<IntPtrT> tmp202;
  TNode<BoolT> tmp203;
  if (block102.is_used()) {
    ca_.Bind(&block102, &phi_bb102_13, &phi_bb102_14, &phi_bb102_16, &phi_bb102_17, &phi_bb102_20, &phi_bb102_21, &phi_bb102_26);
    tmp202 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb102_13});
    tmp203 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb102_14}, TNode<IntPtrT>{tmp202});
    ca_.Branch(tmp203, &block208, std::vector<compiler::Node*>{phi_bb102_13, phi_bb102_14, phi_bb102_16, phi_bb102_17, phi_bb102_20, phi_bb102_21, phi_bb102_26, phi_bb102_26, phi_bb102_26}, &block209, std::vector<compiler::Node*>{phi_bb102_13, phi_bb102_14, phi_bb102_16, phi_bb102_17, phi_bb102_20, phi_bb102_21, phi_bb102_26, phi_bb102_26, phi_bb102_26});
  }

  TNode<FixedArray> phi_bb208_13;
  TNode<IntPtrT> phi_bb208_14;
  TNode<BoolT> phi_bb208_16;
  TNode<PrimitiveHeapObject> phi_bb208_17;
  TNode<String> phi_bb208_20;
  TNode<String> phi_bb208_21;
  TNode<String> phi_bb208_26;
  TNode<String> phi_bb208_30;
  TNode<Object> phi_bb208_31;
  TNode<Union<HeapObject, TaggedIndex>> tmp204;
  TNode<IntPtrT> tmp205;
  TNode<IntPtrT> tmp206;
  TNode<IntPtrT> tmp207;
  TNode<IntPtrT> tmp208;
  TNode<UintPtrT> tmp209;
  TNode<UintPtrT> tmp210;
  TNode<BoolT> tmp211;
  if (block208.is_used()) {
    ca_.Bind(&block208, &phi_bb208_13, &phi_bb208_14, &phi_bb208_16, &phi_bb208_17, &phi_bb208_20, &phi_bb208_21, &phi_bb208_26, &phi_bb208_30, &phi_bb208_31);
    std::tie(tmp204, tmp205, tmp206) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb208_13}).Flatten();
    tmp207 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp208 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb208_14}, TNode<IntPtrT>{tmp207});
    tmp209 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb208_14});
    tmp210 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp206});
    tmp211 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp209}, TNode<UintPtrT>{tmp210});
    ca_.Branch(tmp211, &block215, std::vector<compiler::Node*>{phi_bb208_13, phi_bb208_16, phi_bb208_17, phi_bb208_20, phi_bb208_21, phi_bb208_26, phi_bb208_30, phi_bb208_31, phi_bb208_13, phi_bb208_14, phi_bb208_14, phi_bb208_14, phi_bb208_14}, &block216, std::vector<compiler::Node*>{phi_bb208_13, phi_bb208_16, phi_bb208_17, phi_bb208_20, phi_bb208_21, phi_bb208_26, phi_bb208_30, phi_bb208_31, phi_bb208_13, phi_bb208_14, phi_bb208_14, phi_bb208_14, phi_bb208_14});
  }

  TNode<FixedArray> phi_bb215_13;
  TNode<BoolT> phi_bb215_16;
  TNode<PrimitiveHeapObject> phi_bb215_17;
  TNode<String> phi_bb215_20;
  TNode<String> phi_bb215_21;
  TNode<String> phi_bb215_26;
  TNode<String> phi_bb215_30;
  TNode<Object> phi_bb215_31;
  TNode<FixedArray> phi_bb215_33;
  TNode<IntPtrT> phi_bb215_37;
  TNode<IntPtrT> phi_bb215_38;
  TNode<IntPtrT> phi_bb215_42;
  TNode<IntPtrT> phi_bb215_43;
  TNode<IntPtrT> tmp212;
  TNode<IntPtrT> tmp213;
  TNode<Union<HeapObject, TaggedIndex>> tmp214;
  TNode<IntPtrT> tmp215;
  if (block215.is_used()) {
    ca_.Bind(&block215, &phi_bb215_13, &phi_bb215_16, &phi_bb215_17, &phi_bb215_20, &phi_bb215_21, &phi_bb215_26, &phi_bb215_30, &phi_bb215_31, &phi_bb215_33, &phi_bb215_37, &phi_bb215_38, &phi_bb215_42, &phi_bb215_43);
    tmp212 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb215_43});
    tmp213 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp205}, TNode<IntPtrT>{tmp212});
    std::tie(tmp214, tmp215) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp204}, TNode<IntPtrT>{tmp213}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp214, tmp215}, phi_bb215_31);
    ca_.Goto(&block210, phi_bb215_13, tmp208, phi_bb215_16, phi_bb215_17, phi_bb215_20, phi_bb215_21, phi_bb215_26, phi_bb215_30, phi_bb215_31);
  }

  TNode<FixedArray> phi_bb216_13;
  TNode<BoolT> phi_bb216_16;
  TNode<PrimitiveHeapObject> phi_bb216_17;
  TNode<String> phi_bb216_20;
  TNode<String> phi_bb216_21;
  TNode<String> phi_bb216_26;
  TNode<String> phi_bb216_30;
  TNode<Object> phi_bb216_31;
  TNode<FixedArray> phi_bb216_33;
  TNode<IntPtrT> phi_bb216_37;
  TNode<IntPtrT> phi_bb216_38;
  TNode<IntPtrT> phi_bb216_42;
  TNode<IntPtrT> phi_bb216_43;
  if (block216.is_used()) {
    ca_.Bind(&block216, &phi_bb216_13, &phi_bb216_16, &phi_bb216_17, &phi_bb216_20, &phi_bb216_21, &phi_bb216_26, &phi_bb216_30, &phi_bb216_31, &phi_bb216_33, &phi_bb216_37, &phi_bb216_38, &phi_bb216_42, &phi_bb216_43);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb209_13;
  TNode<IntPtrT> phi_bb209_14;
  TNode<BoolT> phi_bb209_16;
  TNode<PrimitiveHeapObject> phi_bb209_17;
  TNode<String> phi_bb209_20;
  TNode<String> phi_bb209_21;
  TNode<String> phi_bb209_26;
  TNode<String> phi_bb209_30;
  TNode<Object> phi_bb209_31;
  TNode<IntPtrT> tmp216;
  TNode<IntPtrT> tmp217;
  TNode<BoolT> tmp218;
  if (block209.is_used()) {
    ca_.Bind(&block209, &phi_bb209_13, &phi_bb209_14, &phi_bb209_16, &phi_bb209_17, &phi_bb209_20, &phi_bb209_21, &phi_bb209_26, &phi_bb209_30, &phi_bb209_31);
    tmp216 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp202});
    tmp217 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp218 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp216}, TNode<IntPtrT>{tmp217});
    ca_.Branch(tmp218, &block219, std::vector<compiler::Node*>{phi_bb209_13, phi_bb209_14, phi_bb209_16, phi_bb209_17, phi_bb209_20, phi_bb209_21, phi_bb209_26, phi_bb209_30, phi_bb209_31}, &block220, std::vector<compiler::Node*>{phi_bb209_13, phi_bb209_14, phi_bb209_16, phi_bb209_17, phi_bb209_20, phi_bb209_21, phi_bb209_26, phi_bb209_30, phi_bb209_31});
  }

  TNode<FixedArray> phi_bb219_13;
  TNode<IntPtrT> phi_bb219_14;
  TNode<BoolT> phi_bb219_16;
  TNode<PrimitiveHeapObject> phi_bb219_17;
  TNode<String> phi_bb219_20;
  TNode<String> phi_bb219_21;
  TNode<String> phi_bb219_26;
  TNode<String> phi_bb219_30;
  TNode<Object> phi_bb219_31;
  TNode<IntPtrT> tmp219;
  if (block219.is_used()) {
    ca_.Bind(&block219, &phi_bb219_13, &phi_bb219_14, &phi_bb219_16, &phi_bb219_17, &phi_bb219_20, &phi_bb219_21, &phi_bb219_26, &phi_bb219_30, &phi_bb219_31);
    tmp219 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block221, phi_bb219_13, phi_bb219_14, phi_bb219_16, phi_bb219_17, phi_bb219_20, phi_bb219_21, phi_bb219_26, phi_bb219_30, phi_bb219_31, tmp219);
  }

  TNode<FixedArray> phi_bb220_13;
  TNode<IntPtrT> phi_bb220_14;
  TNode<BoolT> phi_bb220_16;
  TNode<PrimitiveHeapObject> phi_bb220_17;
  TNode<String> phi_bb220_20;
  TNode<String> phi_bb220_21;
  TNode<String> phi_bb220_26;
  TNode<String> phi_bb220_30;
  TNode<Object> phi_bb220_31;
  if (block220.is_used()) {
    ca_.Bind(&block220, &phi_bb220_13, &phi_bb220_14, &phi_bb220_16, &phi_bb220_17, &phi_bb220_20, &phi_bb220_21, &phi_bb220_26, &phi_bb220_30, &phi_bb220_31);
    ca_.Goto(&block221, phi_bb220_13, phi_bb220_14, phi_bb220_16, phi_bb220_17, phi_bb220_20, phi_bb220_21, phi_bb220_26, phi_bb220_30, phi_bb220_31, tmp216);
  }

  TNode<FixedArray> phi_bb221_13;
  TNode<IntPtrT> phi_bb221_14;
  TNode<BoolT> phi_bb221_16;
  TNode<PrimitiveHeapObject> phi_bb221_17;
  TNode<String> phi_bb221_20;
  TNode<String> phi_bb221_21;
  TNode<String> phi_bb221_26;
  TNode<String> phi_bb221_30;
  TNode<Object> phi_bb221_31;
  TNode<IntPtrT> phi_bb221_34;
  TNode<FixedArray> tmp220;
  TNode<Union<HeapObject, TaggedIndex>> tmp221;
  TNode<IntPtrT> tmp222;
  TNode<IntPtrT> tmp223;
  TNode<UintPtrT> tmp224;
  TNode<IntPtrT> tmp225;
  TNode<UintPtrT> tmp226;
  TNode<UintPtrT> tmp227;
  TNode<BoolT> tmp228;
  if (block221.is_used()) {
    ca_.Bind(&block221, &phi_bb221_13, &phi_bb221_14, &phi_bb221_16, &phi_bb221_17, &phi_bb221_20, &phi_bb221_21, &phi_bb221_26, &phi_bb221_30, &phi_bb221_31, &phi_bb221_34);
    tmp220 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb221_34});
    std::tie(tmp221, tmp222, tmp223) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb221_13}).Flatten();
    tmp224 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp225 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp224});
    tmp226 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp225});
    tmp227 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp223});
    tmp228 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp226}, TNode<UintPtrT>{tmp227});
    ca_.Branch(tmp228, &block232, std::vector<compiler::Node*>{phi_bb221_13, phi_bb221_14, phi_bb221_16, phi_bb221_17, phi_bb221_20, phi_bb221_21, phi_bb221_26, phi_bb221_30, phi_bb221_31, phi_bb221_13}, &block233, std::vector<compiler::Node*>{phi_bb221_13, phi_bb221_14, phi_bb221_16, phi_bb221_17, phi_bb221_20, phi_bb221_21, phi_bb221_26, phi_bb221_30, phi_bb221_31, phi_bb221_13});
  }

  TNode<FixedArray> phi_bb232_13;
  TNode<IntPtrT> phi_bb232_14;
  TNode<BoolT> phi_bb232_16;
  TNode<PrimitiveHeapObject> phi_bb232_17;
  TNode<String> phi_bb232_20;
  TNode<String> phi_bb232_21;
  TNode<String> phi_bb232_26;
  TNode<String> phi_bb232_30;
  TNode<Object> phi_bb232_31;
  TNode<FixedArray> phi_bb232_36;
  TNode<IntPtrT> tmp229;
  TNode<IntPtrT> tmp230;
  TNode<Union<HeapObject, TaggedIndex>> tmp231;
  TNode<IntPtrT> tmp232;
  TNode<Union<HeapObject, TaggedIndex>> tmp233;
  TNode<IntPtrT> tmp234;
  TNode<IntPtrT> tmp235;
  TNode<UintPtrT> tmp236;
  TNode<IntPtrT> tmp237;
  TNode<UintPtrT> tmp238;
  TNode<UintPtrT> tmp239;
  TNode<BoolT> tmp240;
  if (block232.is_used()) {
    ca_.Bind(&block232, &phi_bb232_13, &phi_bb232_14, &phi_bb232_16, &phi_bb232_17, &phi_bb232_20, &phi_bb232_21, &phi_bb232_26, &phi_bb232_30, &phi_bb232_31, &phi_bb232_36);
    tmp229 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp225});
    tmp230 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp222}, TNode<IntPtrT>{tmp229});
    std::tie(tmp231, tmp232) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp221}, TNode<IntPtrT>{tmp230}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp231, tmp232}, tmp220);
    std::tie(tmp233, tmp234, tmp235) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp220}).Flatten();
    tmp236 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp237 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp236});
    tmp238 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp237});
    tmp239 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp235});
    tmp240 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp238}, TNode<UintPtrT>{tmp239});
    ca_.Branch(tmp240, &block241, std::vector<compiler::Node*>{phi_bb232_13, phi_bb232_14, phi_bb232_16, phi_bb232_17, phi_bb232_20, phi_bb232_21, phi_bb232_26, phi_bb232_30, phi_bb232_31}, &block242, std::vector<compiler::Node*>{phi_bb232_13, phi_bb232_14, phi_bb232_16, phi_bb232_17, phi_bb232_20, phi_bb232_21, phi_bb232_26, phi_bb232_30, phi_bb232_31});
  }

  TNode<FixedArray> phi_bb233_13;
  TNode<IntPtrT> phi_bb233_14;
  TNode<BoolT> phi_bb233_16;
  TNode<PrimitiveHeapObject> phi_bb233_17;
  TNode<String> phi_bb233_20;
  TNode<String> phi_bb233_21;
  TNode<String> phi_bb233_26;
  TNode<String> phi_bb233_30;
  TNode<Object> phi_bb233_31;
  TNode<FixedArray> phi_bb233_36;
  if (block233.is_used()) {
    ca_.Bind(&block233, &phi_bb233_13, &phi_bb233_14, &phi_bb233_16, &phi_bb233_17, &phi_bb233_20, &phi_bb233_21, &phi_bb233_26, &phi_bb233_30, &phi_bb233_31, &phi_bb233_36);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb241_13;
  TNode<IntPtrT> phi_bb241_14;
  TNode<BoolT> phi_bb241_16;
  TNode<PrimitiveHeapObject> phi_bb241_17;
  TNode<String> phi_bb241_20;
  TNode<String> phi_bb241_21;
  TNode<String> phi_bb241_26;
  TNode<String> phi_bb241_30;
  TNode<Object> phi_bb241_31;
  TNode<IntPtrT> tmp241;
  TNode<IntPtrT> tmp242;
  TNode<Union<HeapObject, TaggedIndex>> tmp243;
  TNode<IntPtrT> tmp244;
  TNode<Undefined> tmp245;
  TNode<Union<HeapObject, TaggedIndex>> tmp246;
  TNode<IntPtrT> tmp247;
  TNode<IntPtrT> tmp248;
  TNode<UintPtrT> tmp249;
  TNode<IntPtrT> tmp250;
  TNode<UintPtrT> tmp251;
  TNode<UintPtrT> tmp252;
  TNode<BoolT> tmp253;
  if (block241.is_used()) {
    ca_.Bind(&block241, &phi_bb241_13, &phi_bb241_14, &phi_bb241_16, &phi_bb241_17, &phi_bb241_20, &phi_bb241_21, &phi_bb241_26, &phi_bb241_30, &phi_bb241_31);
    tmp241 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp237});
    tmp242 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp234}, TNode<IntPtrT>{tmp241});
    std::tie(tmp243, tmp244) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp233}, TNode<IntPtrT>{tmp242}).Flatten();
    tmp245 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp243, tmp244}, tmp245);
    std::tie(tmp246, tmp247, tmp248) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp220}).Flatten();
    tmp249 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp250 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp249});
    tmp251 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp250});
    tmp252 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp248});
    tmp253 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp251}, TNode<UintPtrT>{tmp252});
    ca_.Branch(tmp253, &block250, std::vector<compiler::Node*>{phi_bb241_13, phi_bb241_14, phi_bb241_16, phi_bb241_17, phi_bb241_20, phi_bb241_21, phi_bb241_26, phi_bb241_30, phi_bb241_31}, &block251, std::vector<compiler::Node*>{phi_bb241_13, phi_bb241_14, phi_bb241_16, phi_bb241_17, phi_bb241_20, phi_bb241_21, phi_bb241_26, phi_bb241_30, phi_bb241_31});
  }

  TNode<FixedArray> phi_bb242_13;
  TNode<IntPtrT> phi_bb242_14;
  TNode<BoolT> phi_bb242_16;
  TNode<PrimitiveHeapObject> phi_bb242_17;
  TNode<String> phi_bb242_20;
  TNode<String> phi_bb242_21;
  TNode<String> phi_bb242_26;
  TNode<String> phi_bb242_30;
  TNode<Object> phi_bb242_31;
  if (block242.is_used()) {
    ca_.Bind(&block242, &phi_bb242_13, &phi_bb242_14, &phi_bb242_16, &phi_bb242_17, &phi_bb242_20, &phi_bb242_21, &phi_bb242_26, &phi_bb242_30, &phi_bb242_31);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb250_13;
  TNode<IntPtrT> phi_bb250_14;
  TNode<BoolT> phi_bb250_16;
  TNode<PrimitiveHeapObject> phi_bb250_17;
  TNode<String> phi_bb250_20;
  TNode<String> phi_bb250_21;
  TNode<String> phi_bb250_26;
  TNode<String> phi_bb250_30;
  TNode<Object> phi_bb250_31;
  TNode<IntPtrT> tmp254;
  TNode<IntPtrT> tmp255;
  TNode<Union<HeapObject, TaggedIndex>> tmp256;
  TNode<IntPtrT> tmp257;
  TNode<IntPtrT> tmp258;
  if (block250.is_used()) {
    ca_.Bind(&block250, &phi_bb250_13, &phi_bb250_14, &phi_bb250_16, &phi_bb250_17, &phi_bb250_20, &phi_bb250_21, &phi_bb250_26, &phi_bb250_30, &phi_bb250_31);
    tmp254 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp250});
    tmp255 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp247}, TNode<IntPtrT>{tmp254});
    std::tie(tmp256, tmp257) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp246}, TNode<IntPtrT>{tmp255}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp256, tmp257}, phi_bb250_31);
    tmp258 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block210, tmp220, tmp258, phi_bb250_16, phi_bb250_17, phi_bb250_20, phi_bb250_21, phi_bb250_26, phi_bb250_30, phi_bb250_31);
  }

  TNode<FixedArray> phi_bb251_13;
  TNode<IntPtrT> phi_bb251_14;
  TNode<BoolT> phi_bb251_16;
  TNode<PrimitiveHeapObject> phi_bb251_17;
  TNode<String> phi_bb251_20;
  TNode<String> phi_bb251_21;
  TNode<String> phi_bb251_26;
  TNode<String> phi_bb251_30;
  TNode<Object> phi_bb251_31;
  if (block251.is_used()) {
    ca_.Bind(&block251, &phi_bb251_13, &phi_bb251_14, &phi_bb251_16, &phi_bb251_17, &phi_bb251_20, &phi_bb251_21, &phi_bb251_26, &phi_bb251_30, &phi_bb251_31);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb210_13;
  TNode<IntPtrT> phi_bb210_14;
  TNode<BoolT> phi_bb210_16;
  TNode<PrimitiveHeapObject> phi_bb210_17;
  TNode<String> phi_bb210_20;
  TNode<String> phi_bb210_21;
  TNode<String> phi_bb210_26;
  TNode<String> phi_bb210_30;
  TNode<Object> phi_bb210_31;
  if (block210.is_used()) {
    ca_.Bind(&block210, &phi_bb210_13, &phi_bb210_14, &phi_bb210_16, &phi_bb210_17, &phi_bb210_20, &phi_bb210_21, &phi_bb210_26, &phi_bb210_30, &phi_bb210_31);
    ca_.Goto(&block103, phi_bb210_13, phi_bb210_14, phi_bb210_16, phi_bb210_26, phi_bb210_20, phi_bb210_21, phi_bb210_26);
  }

  TNode<FixedArray> phi_bb103_13;
  TNode<IntPtrT> phi_bb103_14;
  TNode<BoolT> phi_bb103_16;
  TNode<PrimitiveHeapObject> phi_bb103_17;
  TNode<String> phi_bb103_20;
  TNode<String> phi_bb103_21;
  TNode<String> phi_bb103_26;
  TNode<IntPtrT> tmp259;
  TNode<Map> tmp260;
  TNode<BoolT> tmp261;
  TNode<BoolT> tmp262;
  TNode<IntPtrT> tmp263;
  if (block103.is_used()) {
    ca_.Bind(&block103, &phi_bb103_13, &phi_bb103_14, &phi_bb103_16, &phi_bb103_17, &phi_bb103_20, &phi_bb103_21, &phi_bb103_26);
    tmp259 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp260 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{phi_bb103_26, tmp259});
    tmp261 = CodeStubAssembler(state_).IsOneByteStringMap(TNode<Map>{tmp260});
    tmp262 = CodeStubAssembler(state_).Word32And(TNode<BoolT>{tmp261}, TNode<BoolT>{phi_bb103_16});
    tmp263 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block4, tmp263, phi_bb6_11, phi_bb103_13, phi_bb103_14, tmp109, tmp262, phi_bb103_17, tmp19);
  }

  TNode<IntPtrT> phi_bb3_10;
  TNode<BuiltinPtr> phi_bb3_11;
  TNode<FixedArray> phi_bb3_13;
  TNode<IntPtrT> phi_bb3_14;
  TNode<IntPtrT> phi_bb3_15;
  TNode<BoolT> phi_bb3_16;
  TNode<PrimitiveHeapObject> phi_bb3_17;
  TNode<UintPtrT> phi_bb3_18;
  TNode<BoolT> tmp264;
  TNode<IntPtrT> tmp265;
  TNode<BoolT> tmp266;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_10, &phi_bb3_11, &phi_bb3_13, &phi_bb3_14, &phi_bb3_15, &phi_bb3_16, &phi_bb3_17, &phi_bb3_18);
    tmp264 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp265 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp266 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb3_10}, TNode<IntPtrT>{tmp265});
    ca_.Branch(tmp266, &block257, std::vector<compiler::Node*>{phi_bb3_10, phi_bb3_11, phi_bb3_13, phi_bb3_14, phi_bb3_15, phi_bb3_16, phi_bb3_17, phi_bb3_18, phi_bb3_10, phi_bb3_10}, &block258, std::vector<compiler::Node*>{phi_bb3_10, phi_bb3_11, phi_bb3_13, phi_bb3_14, phi_bb3_15, phi_bb3_16, phi_bb3_17, phi_bb3_18, phi_bb3_10, phi_bb3_10});
  }

  TNode<IntPtrT> phi_bb257_10;
  TNode<BuiltinPtr> phi_bb257_11;
  TNode<FixedArray> phi_bb257_13;
  TNode<IntPtrT> phi_bb257_14;
  TNode<IntPtrT> phi_bb257_15;
  TNode<BoolT> phi_bb257_16;
  TNode<PrimitiveHeapObject> phi_bb257_17;
  TNode<UintPtrT> phi_bb257_18;
  TNode<IntPtrT> phi_bb257_19;
  TNode<IntPtrT> phi_bb257_23;
  TNode<BoolT> tmp267;
  if (block257.is_used()) {
    ca_.Bind(&block257, &phi_bb257_10, &phi_bb257_11, &phi_bb257_13, &phi_bb257_14, &phi_bb257_15, &phi_bb257_16, &phi_bb257_17, &phi_bb257_18, &phi_bb257_19, &phi_bb257_23);
    tmp267 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block259, phi_bb257_10, phi_bb257_11, phi_bb257_13, phi_bb257_14, phi_bb257_15, phi_bb257_16, phi_bb257_17, phi_bb257_18, phi_bb257_19, phi_bb257_23, tmp267);
  }

  TNode<IntPtrT> phi_bb258_10;
  TNode<BuiltinPtr> phi_bb258_11;
  TNode<FixedArray> phi_bb258_13;
  TNode<IntPtrT> phi_bb258_14;
  TNode<IntPtrT> phi_bb258_15;
  TNode<BoolT> phi_bb258_16;
  TNode<PrimitiveHeapObject> phi_bb258_17;
  TNode<UintPtrT> phi_bb258_18;
  TNode<IntPtrT> phi_bb258_19;
  TNode<IntPtrT> phi_bb258_23;
  TNode<IntPtrT> tmp268;
  TNode<BoolT> tmp269;
  if (block258.is_used()) {
    ca_.Bind(&block258, &phi_bb258_10, &phi_bb258_11, &phi_bb258_13, &phi_bb258_14, &phi_bb258_15, &phi_bb258_16, &phi_bb258_17, &phi_bb258_18, &phi_bb258_19, &phi_bb258_23);
    tmp268 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp269 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp268});
    ca_.Goto(&block259, phi_bb258_10, phi_bb258_11, phi_bb258_13, phi_bb258_14, phi_bb258_15, phi_bb258_16, phi_bb258_17, phi_bb258_18, phi_bb258_19, phi_bb258_23, tmp269);
  }

  TNode<IntPtrT> phi_bb259_10;
  TNode<BuiltinPtr> phi_bb259_11;
  TNode<FixedArray> phi_bb259_13;
  TNode<IntPtrT> phi_bb259_14;
  TNode<IntPtrT> phi_bb259_15;
  TNode<BoolT> phi_bb259_16;
  TNode<PrimitiveHeapObject> phi_bb259_17;
  TNode<UintPtrT> phi_bb259_18;
  TNode<IntPtrT> phi_bb259_19;
  TNode<IntPtrT> phi_bb259_23;
  TNode<BoolT> phi_bb259_27;
  if (block259.is_used()) {
    ca_.Bind(&block259, &phi_bb259_10, &phi_bb259_11, &phi_bb259_13, &phi_bb259_14, &phi_bb259_15, &phi_bb259_16, &phi_bb259_17, &phi_bb259_18, &phi_bb259_19, &phi_bb259_23, &phi_bb259_27);
    ca_.Branch(phi_bb259_27, &block255, std::vector<compiler::Node*>{phi_bb259_10, phi_bb259_11, phi_bb259_13, phi_bb259_14, phi_bb259_15, phi_bb259_16, phi_bb259_17, phi_bb259_18, phi_bb259_19, phi_bb259_23}, &block256, std::vector<compiler::Node*>{phi_bb259_10, phi_bb259_11, phi_bb259_13, phi_bb259_14, phi_bb259_15, phi_bb259_16, phi_bb259_17, phi_bb259_18, phi_bb259_19, phi_bb259_23});
  }

  TNode<IntPtrT> phi_bb255_10;
  TNode<BuiltinPtr> phi_bb255_11;
  TNode<FixedArray> phi_bb255_13;
  TNode<IntPtrT> phi_bb255_14;
  TNode<IntPtrT> phi_bb255_15;
  TNode<BoolT> phi_bb255_16;
  TNode<PrimitiveHeapObject> phi_bb255_17;
  TNode<UintPtrT> phi_bb255_18;
  TNode<IntPtrT> phi_bb255_19;
  TNode<IntPtrT> phi_bb255_23;
  if (block255.is_used()) {
    ca_.Bind(&block255, &phi_bb255_10, &phi_bb255_11, &phi_bb255_13, &phi_bb255_14, &phi_bb255_15, &phi_bb255_16, &phi_bb255_17, &phi_bb255_18, &phi_bb255_19, &phi_bb255_23);
    ca_.Goto(&block254, phi_bb255_10, phi_bb255_11, phi_bb255_13, phi_bb255_14, phi_bb255_15, phi_bb255_16, phi_bb255_17, phi_bb255_18, phi_bb255_19, phi_bb255_23);
  }

  TNode<IntPtrT> phi_bb256_10;
  TNode<BuiltinPtr> phi_bb256_11;
  TNode<FixedArray> phi_bb256_13;
  TNode<IntPtrT> phi_bb256_14;
  TNode<IntPtrT> phi_bb256_15;
  TNode<BoolT> phi_bb256_16;
  TNode<PrimitiveHeapObject> phi_bb256_17;
  TNode<UintPtrT> phi_bb256_18;
  TNode<IntPtrT> phi_bb256_19;
  TNode<IntPtrT> phi_bb256_23;
  TNode<IntPtrT> tmp270;
  TNode<IntPtrT> tmp271;
  TNode<BoolT> tmp272;
  if (block256.is_used()) {
    ca_.Bind(&block256, &phi_bb256_10, &phi_bb256_11, &phi_bb256_13, &phi_bb256_14, &phi_bb256_15, &phi_bb256_16, &phi_bb256_17, &phi_bb256_18, &phi_bb256_19, &phi_bb256_23);
    tmp270 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{phi_bb256_23});
    tmp271 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp270}, TNode<IntPtrT>{tmp3});
    tmp272 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{tmp271}, TNode<IntPtrT>{phi_bb256_23});
    ca_.Branch(tmp272, &block260, std::vector<compiler::Node*>{phi_bb256_10, phi_bb256_11, phi_bb256_13, phi_bb256_14, phi_bb256_15, phi_bb256_16, phi_bb256_17, phi_bb256_18, phi_bb256_19, phi_bb256_23, phi_bb256_23}, &block261, std::vector<compiler::Node*>{phi_bb256_10, phi_bb256_11, phi_bb256_13, phi_bb256_14, phi_bb256_15, phi_bb256_16, phi_bb256_17, phi_bb256_18, phi_bb256_19, phi_bb256_23, phi_bb256_23});
  }

  TNode<IntPtrT> phi_bb260_10;
  TNode<BuiltinPtr> phi_bb260_11;
  TNode<FixedArray> phi_bb260_13;
  TNode<IntPtrT> phi_bb260_14;
  TNode<IntPtrT> phi_bb260_15;
  TNode<BoolT> phi_bb260_16;
  TNode<PrimitiveHeapObject> phi_bb260_17;
  TNode<UintPtrT> phi_bb260_18;
  TNode<IntPtrT> phi_bb260_19;
  TNode<IntPtrT> phi_bb260_23;
  TNode<IntPtrT> phi_bb260_26;
  if (block260.is_used()) {
    ca_.Bind(&block260, &phi_bb260_10, &phi_bb260_11, &phi_bb260_13, &phi_bb260_14, &phi_bb260_15, &phi_bb260_16, &phi_bb260_17, &phi_bb260_18, &phi_bb260_19, &phi_bb260_23, &phi_bb260_26);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowInvalidStringLength, p_context);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb261_10;
  TNode<BuiltinPtr> phi_bb261_11;
  TNode<FixedArray> phi_bb261_13;
  TNode<IntPtrT> phi_bb261_14;
  TNode<IntPtrT> phi_bb261_15;
  TNode<BoolT> phi_bb261_16;
  TNode<PrimitiveHeapObject> phi_bb261_17;
  TNode<UintPtrT> phi_bb261_18;
  TNode<IntPtrT> phi_bb261_19;
  TNode<IntPtrT> phi_bb261_23;
  TNode<IntPtrT> phi_bb261_26;
  TNode<IntPtrT> tmp273;
  if (block261.is_used()) {
    ca_.Bind(&block261, &phi_bb261_10, &phi_bb261_11, &phi_bb261_13, &phi_bb261_14, &phi_bb261_15, &phi_bb261_16, &phi_bb261_17, &phi_bb261_18, &phi_bb261_19, &phi_bb261_23, &phi_bb261_26);
    tmp273 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb261_15}, TNode<IntPtrT>{tmp270});
    ca_.Branch(tmp264, &block262, std::vector<compiler::Node*>{phi_bb261_10, phi_bb261_11, phi_bb261_13, phi_bb261_14, phi_bb261_16, phi_bb261_17, phi_bb261_18, phi_bb261_19, phi_bb261_23, phi_bb261_26}, &block263, std::vector<compiler::Node*>{phi_bb261_10, phi_bb261_11, phi_bb261_13, phi_bb261_14, phi_bb261_16, phi_bb261_17, phi_bb261_18, phi_bb261_19, phi_bb261_23, phi_bb261_26});
  }

  TNode<IntPtrT> phi_bb262_10;
  TNode<BuiltinPtr> phi_bb262_11;
  TNode<FixedArray> phi_bb262_13;
  TNode<IntPtrT> phi_bb262_14;
  TNode<BoolT> phi_bb262_16;
  TNode<PrimitiveHeapObject> phi_bb262_17;
  TNode<UintPtrT> phi_bb262_18;
  TNode<IntPtrT> phi_bb262_19;
  TNode<IntPtrT> phi_bb262_23;
  TNode<IntPtrT> phi_bb262_26;
  TNode<Smi> tmp274;
  TNode<IntPtrT> tmp275;
  TNode<BoolT> tmp276;
  if (block262.is_used()) {
    ca_.Bind(&block262, &phi_bb262_10, &phi_bb262_11, &phi_bb262_13, &phi_bb262_14, &phi_bb262_16, &phi_bb262_17, &phi_bb262_18, &phi_bb262_19, &phi_bb262_23, &phi_bb262_26);
    tmp274 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{phi_bb262_26});
    tmp275 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb262_13});
    tmp276 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb262_14}, TNode<IntPtrT>{tmp275});
    ca_.Branch(tmp276, &block273, std::vector<compiler::Node*>{phi_bb262_10, phi_bb262_11, phi_bb262_13, phi_bb262_14, phi_bb262_16, phi_bb262_17, phi_bb262_18, phi_bb262_19, phi_bb262_23, phi_bb262_26}, &block274, std::vector<compiler::Node*>{phi_bb262_10, phi_bb262_11, phi_bb262_13, phi_bb262_14, phi_bb262_16, phi_bb262_17, phi_bb262_18, phi_bb262_19, phi_bb262_23, phi_bb262_26});
  }

  TNode<IntPtrT> phi_bb273_10;
  TNode<BuiltinPtr> phi_bb273_11;
  TNode<FixedArray> phi_bb273_13;
  TNode<IntPtrT> phi_bb273_14;
  TNode<BoolT> phi_bb273_16;
  TNode<PrimitiveHeapObject> phi_bb273_17;
  TNode<UintPtrT> phi_bb273_18;
  TNode<IntPtrT> phi_bb273_19;
  TNode<IntPtrT> phi_bb273_23;
  TNode<IntPtrT> phi_bb273_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp277;
  TNode<IntPtrT> tmp278;
  TNode<IntPtrT> tmp279;
  TNode<IntPtrT> tmp280;
  TNode<IntPtrT> tmp281;
  TNode<UintPtrT> tmp282;
  TNode<UintPtrT> tmp283;
  TNode<BoolT> tmp284;
  if (block273.is_used()) {
    ca_.Bind(&block273, &phi_bb273_10, &phi_bb273_11, &phi_bb273_13, &phi_bb273_14, &phi_bb273_16, &phi_bb273_17, &phi_bb273_18, &phi_bb273_19, &phi_bb273_23, &phi_bb273_26);
    std::tie(tmp277, tmp278, tmp279) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb273_13}).Flatten();
    tmp280 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp281 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb273_14}, TNode<IntPtrT>{tmp280});
    tmp282 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb273_14});
    tmp283 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp279});
    tmp284 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp282}, TNode<UintPtrT>{tmp283});
    ca_.Branch(tmp284, &block280, std::vector<compiler::Node*>{phi_bb273_10, phi_bb273_11, phi_bb273_13, phi_bb273_16, phi_bb273_17, phi_bb273_18, phi_bb273_19, phi_bb273_23, phi_bb273_26, phi_bb273_13, phi_bb273_14, phi_bb273_14, phi_bb273_14, phi_bb273_14}, &block281, std::vector<compiler::Node*>{phi_bb273_10, phi_bb273_11, phi_bb273_13, phi_bb273_16, phi_bb273_17, phi_bb273_18, phi_bb273_19, phi_bb273_23, phi_bb273_26, phi_bb273_13, phi_bb273_14, phi_bb273_14, phi_bb273_14, phi_bb273_14});
  }

  TNode<IntPtrT> phi_bb280_10;
  TNode<BuiltinPtr> phi_bb280_11;
  TNode<FixedArray> phi_bb280_13;
  TNode<BoolT> phi_bb280_16;
  TNode<PrimitiveHeapObject> phi_bb280_17;
  TNode<UintPtrT> phi_bb280_18;
  TNode<IntPtrT> phi_bb280_19;
  TNode<IntPtrT> phi_bb280_23;
  TNode<IntPtrT> phi_bb280_26;
  TNode<FixedArray> phi_bb280_31;
  TNode<IntPtrT> phi_bb280_35;
  TNode<IntPtrT> phi_bb280_36;
  TNode<IntPtrT> phi_bb280_40;
  TNode<IntPtrT> phi_bb280_41;
  TNode<IntPtrT> tmp285;
  TNode<IntPtrT> tmp286;
  TNode<Union<HeapObject, TaggedIndex>> tmp287;
  TNode<IntPtrT> tmp288;
  if (block280.is_used()) {
    ca_.Bind(&block280, &phi_bb280_10, &phi_bb280_11, &phi_bb280_13, &phi_bb280_16, &phi_bb280_17, &phi_bb280_18, &phi_bb280_19, &phi_bb280_23, &phi_bb280_26, &phi_bb280_31, &phi_bb280_35, &phi_bb280_36, &phi_bb280_40, &phi_bb280_41);
    tmp285 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb280_41});
    tmp286 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp278}, TNode<IntPtrT>{tmp285});
    std::tie(tmp287, tmp288) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp277}, TNode<IntPtrT>{tmp286}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp287, tmp288}, tmp274);
    ca_.Goto(&block275, phi_bb280_10, phi_bb280_11, phi_bb280_13, tmp281, phi_bb280_16, phi_bb280_17, phi_bb280_18, phi_bb280_19, phi_bb280_23, phi_bb280_26);
  }

  TNode<IntPtrT> phi_bb281_10;
  TNode<BuiltinPtr> phi_bb281_11;
  TNode<FixedArray> phi_bb281_13;
  TNode<BoolT> phi_bb281_16;
  TNode<PrimitiveHeapObject> phi_bb281_17;
  TNode<UintPtrT> phi_bb281_18;
  TNode<IntPtrT> phi_bb281_19;
  TNode<IntPtrT> phi_bb281_23;
  TNode<IntPtrT> phi_bb281_26;
  TNode<FixedArray> phi_bb281_31;
  TNode<IntPtrT> phi_bb281_35;
  TNode<IntPtrT> phi_bb281_36;
  TNode<IntPtrT> phi_bb281_40;
  TNode<IntPtrT> phi_bb281_41;
  if (block281.is_used()) {
    ca_.Bind(&block281, &phi_bb281_10, &phi_bb281_11, &phi_bb281_13, &phi_bb281_16, &phi_bb281_17, &phi_bb281_18, &phi_bb281_19, &phi_bb281_23, &phi_bb281_26, &phi_bb281_31, &phi_bb281_35, &phi_bb281_36, &phi_bb281_40, &phi_bb281_41);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb274_10;
  TNode<BuiltinPtr> phi_bb274_11;
  TNode<FixedArray> phi_bb274_13;
  TNode<IntPtrT> phi_bb274_14;
  TNode<BoolT> phi_bb274_16;
  TNode<PrimitiveHeapObject> phi_bb274_17;
  TNode<UintPtrT> phi_bb274_18;
  TNode<IntPtrT> phi_bb274_19;
  TNode<IntPtrT> phi_bb274_23;
  TNode<IntPtrT> phi_bb274_26;
  TNode<IntPtrT> tmp289;
  TNode<IntPtrT> tmp290;
  TNode<BoolT> tmp291;
  if (block274.is_used()) {
    ca_.Bind(&block274, &phi_bb274_10, &phi_bb274_11, &phi_bb274_13, &phi_bb274_14, &phi_bb274_16, &phi_bb274_17, &phi_bb274_18, &phi_bb274_19, &phi_bb274_23, &phi_bb274_26);
    tmp289 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp275});
    tmp290 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp291 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp289}, TNode<IntPtrT>{tmp290});
    ca_.Branch(tmp291, &block284, std::vector<compiler::Node*>{phi_bb274_10, phi_bb274_11, phi_bb274_13, phi_bb274_14, phi_bb274_16, phi_bb274_17, phi_bb274_18, phi_bb274_19, phi_bb274_23, phi_bb274_26}, &block285, std::vector<compiler::Node*>{phi_bb274_10, phi_bb274_11, phi_bb274_13, phi_bb274_14, phi_bb274_16, phi_bb274_17, phi_bb274_18, phi_bb274_19, phi_bb274_23, phi_bb274_26});
  }

  TNode<IntPtrT> phi_bb284_10;
  TNode<BuiltinPtr> phi_bb284_11;
  TNode<FixedArray> phi_bb284_13;
  TNode<IntPtrT> phi_bb284_14;
  TNode<BoolT> phi_bb284_16;
  TNode<PrimitiveHeapObject> phi_bb284_17;
  TNode<UintPtrT> phi_bb284_18;
  TNode<IntPtrT> phi_bb284_19;
  TNode<IntPtrT> phi_bb284_23;
  TNode<IntPtrT> phi_bb284_26;
  TNode<IntPtrT> tmp292;
  if (block284.is_used()) {
    ca_.Bind(&block284, &phi_bb284_10, &phi_bb284_11, &phi_bb284_13, &phi_bb284_14, &phi_bb284_16, &phi_bb284_17, &phi_bb284_18, &phi_bb284_19, &phi_bb284_23, &phi_bb284_26);
    tmp292 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block286, phi_bb284_10, phi_bb284_11, phi_bb284_13, phi_bb284_14, phi_bb284_16, phi_bb284_17, phi_bb284_18, phi_bb284_19, phi_bb284_23, phi_bb284_26, tmp292);
  }

  TNode<IntPtrT> phi_bb285_10;
  TNode<BuiltinPtr> phi_bb285_11;
  TNode<FixedArray> phi_bb285_13;
  TNode<IntPtrT> phi_bb285_14;
  TNode<BoolT> phi_bb285_16;
  TNode<PrimitiveHeapObject> phi_bb285_17;
  TNode<UintPtrT> phi_bb285_18;
  TNode<IntPtrT> phi_bb285_19;
  TNode<IntPtrT> phi_bb285_23;
  TNode<IntPtrT> phi_bb285_26;
  if (block285.is_used()) {
    ca_.Bind(&block285, &phi_bb285_10, &phi_bb285_11, &phi_bb285_13, &phi_bb285_14, &phi_bb285_16, &phi_bb285_17, &phi_bb285_18, &phi_bb285_19, &phi_bb285_23, &phi_bb285_26);
    ca_.Goto(&block286, phi_bb285_10, phi_bb285_11, phi_bb285_13, phi_bb285_14, phi_bb285_16, phi_bb285_17, phi_bb285_18, phi_bb285_19, phi_bb285_23, phi_bb285_26, tmp289);
  }

  TNode<IntPtrT> phi_bb286_10;
  TNode<BuiltinPtr> phi_bb286_11;
  TNode<FixedArray> phi_bb286_13;
  TNode<IntPtrT> phi_bb286_14;
  TNode<BoolT> phi_bb286_16;
  TNode<PrimitiveHeapObject> phi_bb286_17;
  TNode<UintPtrT> phi_bb286_18;
  TNode<IntPtrT> phi_bb286_19;
  TNode<IntPtrT> phi_bb286_23;
  TNode<IntPtrT> phi_bb286_26;
  TNode<IntPtrT> phi_bb286_32;
  TNode<FixedArray> tmp293;
  TNode<Union<HeapObject, TaggedIndex>> tmp294;
  TNode<IntPtrT> tmp295;
  TNode<IntPtrT> tmp296;
  TNode<UintPtrT> tmp297;
  TNode<IntPtrT> tmp298;
  TNode<UintPtrT> tmp299;
  TNode<UintPtrT> tmp300;
  TNode<BoolT> tmp301;
  if (block286.is_used()) {
    ca_.Bind(&block286, &phi_bb286_10, &phi_bb286_11, &phi_bb286_13, &phi_bb286_14, &phi_bb286_16, &phi_bb286_17, &phi_bb286_18, &phi_bb286_19, &phi_bb286_23, &phi_bb286_26, &phi_bb286_32);
    tmp293 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb286_32});
    std::tie(tmp294, tmp295, tmp296) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb286_13}).Flatten();
    tmp297 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp298 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp297});
    tmp299 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp298});
    tmp300 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp296});
    tmp301 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp299}, TNode<UintPtrT>{tmp300});
    ca_.Branch(tmp301, &block297, std::vector<compiler::Node*>{phi_bb286_10, phi_bb286_11, phi_bb286_13, phi_bb286_14, phi_bb286_16, phi_bb286_17, phi_bb286_18, phi_bb286_19, phi_bb286_23, phi_bb286_26, phi_bb286_13}, &block298, std::vector<compiler::Node*>{phi_bb286_10, phi_bb286_11, phi_bb286_13, phi_bb286_14, phi_bb286_16, phi_bb286_17, phi_bb286_18, phi_bb286_19, phi_bb286_23, phi_bb286_26, phi_bb286_13});
  }

  TNode<IntPtrT> phi_bb297_10;
  TNode<BuiltinPtr> phi_bb297_11;
  TNode<FixedArray> phi_bb297_13;
  TNode<IntPtrT> phi_bb297_14;
  TNode<BoolT> phi_bb297_16;
  TNode<PrimitiveHeapObject> phi_bb297_17;
  TNode<UintPtrT> phi_bb297_18;
  TNode<IntPtrT> phi_bb297_19;
  TNode<IntPtrT> phi_bb297_23;
  TNode<IntPtrT> phi_bb297_26;
  TNode<FixedArray> phi_bb297_34;
  TNode<IntPtrT> tmp302;
  TNode<IntPtrT> tmp303;
  TNode<Union<HeapObject, TaggedIndex>> tmp304;
  TNode<IntPtrT> tmp305;
  TNode<Union<HeapObject, TaggedIndex>> tmp306;
  TNode<IntPtrT> tmp307;
  TNode<IntPtrT> tmp308;
  TNode<UintPtrT> tmp309;
  TNode<IntPtrT> tmp310;
  TNode<UintPtrT> tmp311;
  TNode<UintPtrT> tmp312;
  TNode<BoolT> tmp313;
  if (block297.is_used()) {
    ca_.Bind(&block297, &phi_bb297_10, &phi_bb297_11, &phi_bb297_13, &phi_bb297_14, &phi_bb297_16, &phi_bb297_17, &phi_bb297_18, &phi_bb297_19, &phi_bb297_23, &phi_bb297_26, &phi_bb297_34);
    tmp302 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp298});
    tmp303 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp295}, TNode<IntPtrT>{tmp302});
    std::tie(tmp304, tmp305) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp294}, TNode<IntPtrT>{tmp303}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp304, tmp305}, tmp293);
    std::tie(tmp306, tmp307, tmp308) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp293}).Flatten();
    tmp309 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp310 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp309});
    tmp311 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp310});
    tmp312 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp308});
    tmp313 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp311}, TNode<UintPtrT>{tmp312});
    ca_.Branch(tmp313, &block306, std::vector<compiler::Node*>{phi_bb297_10, phi_bb297_11, phi_bb297_13, phi_bb297_14, phi_bb297_16, phi_bb297_17, phi_bb297_18, phi_bb297_19, phi_bb297_23, phi_bb297_26}, &block307, std::vector<compiler::Node*>{phi_bb297_10, phi_bb297_11, phi_bb297_13, phi_bb297_14, phi_bb297_16, phi_bb297_17, phi_bb297_18, phi_bb297_19, phi_bb297_23, phi_bb297_26});
  }

  TNode<IntPtrT> phi_bb298_10;
  TNode<BuiltinPtr> phi_bb298_11;
  TNode<FixedArray> phi_bb298_13;
  TNode<IntPtrT> phi_bb298_14;
  TNode<BoolT> phi_bb298_16;
  TNode<PrimitiveHeapObject> phi_bb298_17;
  TNode<UintPtrT> phi_bb298_18;
  TNode<IntPtrT> phi_bb298_19;
  TNode<IntPtrT> phi_bb298_23;
  TNode<IntPtrT> phi_bb298_26;
  TNode<FixedArray> phi_bb298_34;
  if (block298.is_used()) {
    ca_.Bind(&block298, &phi_bb298_10, &phi_bb298_11, &phi_bb298_13, &phi_bb298_14, &phi_bb298_16, &phi_bb298_17, &phi_bb298_18, &phi_bb298_19, &phi_bb298_23, &phi_bb298_26, &phi_bb298_34);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb306_10;
  TNode<BuiltinPtr> phi_bb306_11;
  TNode<FixedArray> phi_bb306_13;
  TNode<IntPtrT> phi_bb306_14;
  TNode<BoolT> phi_bb306_16;
  TNode<PrimitiveHeapObject> phi_bb306_17;
  TNode<UintPtrT> phi_bb306_18;
  TNode<IntPtrT> phi_bb306_19;
  TNode<IntPtrT> phi_bb306_23;
  TNode<IntPtrT> phi_bb306_26;
  TNode<IntPtrT> tmp314;
  TNode<IntPtrT> tmp315;
  TNode<Union<HeapObject, TaggedIndex>> tmp316;
  TNode<IntPtrT> tmp317;
  TNode<Undefined> tmp318;
  TNode<Union<HeapObject, TaggedIndex>> tmp319;
  TNode<IntPtrT> tmp320;
  TNode<IntPtrT> tmp321;
  TNode<UintPtrT> tmp322;
  TNode<IntPtrT> tmp323;
  TNode<UintPtrT> tmp324;
  TNode<UintPtrT> tmp325;
  TNode<BoolT> tmp326;
  if (block306.is_used()) {
    ca_.Bind(&block306, &phi_bb306_10, &phi_bb306_11, &phi_bb306_13, &phi_bb306_14, &phi_bb306_16, &phi_bb306_17, &phi_bb306_18, &phi_bb306_19, &phi_bb306_23, &phi_bb306_26);
    tmp314 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp310});
    tmp315 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp307}, TNode<IntPtrT>{tmp314});
    std::tie(tmp316, tmp317) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp306}, TNode<IntPtrT>{tmp315}).Flatten();
    tmp318 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp316, tmp317}, tmp318);
    std::tie(tmp319, tmp320, tmp321) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp293}).Flatten();
    tmp322 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp323 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp322});
    tmp324 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp323});
    tmp325 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp321});
    tmp326 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp324}, TNode<UintPtrT>{tmp325});
    ca_.Branch(tmp326, &block315, std::vector<compiler::Node*>{phi_bb306_10, phi_bb306_11, phi_bb306_13, phi_bb306_14, phi_bb306_16, phi_bb306_17, phi_bb306_18, phi_bb306_19, phi_bb306_23, phi_bb306_26}, &block316, std::vector<compiler::Node*>{phi_bb306_10, phi_bb306_11, phi_bb306_13, phi_bb306_14, phi_bb306_16, phi_bb306_17, phi_bb306_18, phi_bb306_19, phi_bb306_23, phi_bb306_26});
  }

  TNode<IntPtrT> phi_bb307_10;
  TNode<BuiltinPtr> phi_bb307_11;
  TNode<FixedArray> phi_bb307_13;
  TNode<IntPtrT> phi_bb307_14;
  TNode<BoolT> phi_bb307_16;
  TNode<PrimitiveHeapObject> phi_bb307_17;
  TNode<UintPtrT> phi_bb307_18;
  TNode<IntPtrT> phi_bb307_19;
  TNode<IntPtrT> phi_bb307_23;
  TNode<IntPtrT> phi_bb307_26;
  if (block307.is_used()) {
    ca_.Bind(&block307, &phi_bb307_10, &phi_bb307_11, &phi_bb307_13, &phi_bb307_14, &phi_bb307_16, &phi_bb307_17, &phi_bb307_18, &phi_bb307_19, &phi_bb307_23, &phi_bb307_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb315_10;
  TNode<BuiltinPtr> phi_bb315_11;
  TNode<FixedArray> phi_bb315_13;
  TNode<IntPtrT> phi_bb315_14;
  TNode<BoolT> phi_bb315_16;
  TNode<PrimitiveHeapObject> phi_bb315_17;
  TNode<UintPtrT> phi_bb315_18;
  TNode<IntPtrT> phi_bb315_19;
  TNode<IntPtrT> phi_bb315_23;
  TNode<IntPtrT> phi_bb315_26;
  TNode<IntPtrT> tmp327;
  TNode<IntPtrT> tmp328;
  TNode<Union<HeapObject, TaggedIndex>> tmp329;
  TNode<IntPtrT> tmp330;
  TNode<IntPtrT> tmp331;
  if (block315.is_used()) {
    ca_.Bind(&block315, &phi_bb315_10, &phi_bb315_11, &phi_bb315_13, &phi_bb315_14, &phi_bb315_16, &phi_bb315_17, &phi_bb315_18, &phi_bb315_19, &phi_bb315_23, &phi_bb315_26);
    tmp327 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp323});
    tmp328 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp320}, TNode<IntPtrT>{tmp327});
    std::tie(tmp329, tmp330) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp319}, TNode<IntPtrT>{tmp328}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp329, tmp330}, tmp274);
    tmp331 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block275, phi_bb315_10, phi_bb315_11, tmp293, tmp331, phi_bb315_16, phi_bb315_17, phi_bb315_18, phi_bb315_19, phi_bb315_23, phi_bb315_26);
  }

  TNode<IntPtrT> phi_bb316_10;
  TNode<BuiltinPtr> phi_bb316_11;
  TNode<FixedArray> phi_bb316_13;
  TNode<IntPtrT> phi_bb316_14;
  TNode<BoolT> phi_bb316_16;
  TNode<PrimitiveHeapObject> phi_bb316_17;
  TNode<UintPtrT> phi_bb316_18;
  TNode<IntPtrT> phi_bb316_19;
  TNode<IntPtrT> phi_bb316_23;
  TNode<IntPtrT> phi_bb316_26;
  if (block316.is_used()) {
    ca_.Bind(&block316, &phi_bb316_10, &phi_bb316_11, &phi_bb316_13, &phi_bb316_14, &phi_bb316_16, &phi_bb316_17, &phi_bb316_18, &phi_bb316_19, &phi_bb316_23, &phi_bb316_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb275_10;
  TNode<BuiltinPtr> phi_bb275_11;
  TNode<FixedArray> phi_bb275_13;
  TNode<IntPtrT> phi_bb275_14;
  TNode<BoolT> phi_bb275_16;
  TNode<PrimitiveHeapObject> phi_bb275_17;
  TNode<UintPtrT> phi_bb275_18;
  TNode<IntPtrT> phi_bb275_19;
  TNode<IntPtrT> phi_bb275_23;
  TNode<IntPtrT> phi_bb275_26;
  TNode<Null> tmp332;
  if (block275.is_used()) {
    ca_.Bind(&block275, &phi_bb275_10, &phi_bb275_11, &phi_bb275_13, &phi_bb275_14, &phi_bb275_16, &phi_bb275_17, &phi_bb275_18, &phi_bb275_19, &phi_bb275_23, &phi_bb275_26);
    tmp332 = Null_0(state_);
    ca_.Goto(&block263, phi_bb275_10, phi_bb275_11, phi_bb275_13, phi_bb275_14, phi_bb275_16, tmp332, phi_bb275_18, phi_bb275_19, phi_bb275_23, phi_bb275_26);
  }

  TNode<IntPtrT> phi_bb263_10;
  TNode<BuiltinPtr> phi_bb263_11;
  TNode<FixedArray> phi_bb263_13;
  TNode<IntPtrT> phi_bb263_14;
  TNode<BoolT> phi_bb263_16;
  TNode<PrimitiveHeapObject> phi_bb263_17;
  TNode<UintPtrT> phi_bb263_18;
  TNode<IntPtrT> phi_bb263_19;
  TNode<IntPtrT> phi_bb263_23;
  TNode<IntPtrT> phi_bb263_26;
  if (block263.is_used()) {
    ca_.Bind(&block263, &phi_bb263_10, &phi_bb263_11, &phi_bb263_13, &phi_bb263_14, &phi_bb263_16, &phi_bb263_17, &phi_bb263_18, &phi_bb263_19, &phi_bb263_23, &phi_bb263_26);
    ca_.Goto(&block254, phi_bb263_10, phi_bb263_11, phi_bb263_13, phi_bb263_14, tmp273, phi_bb263_16, phi_bb263_17, phi_bb263_18, phi_bb263_19, phi_bb263_23);
  }

  TNode<IntPtrT> phi_bb254_10;
  TNode<BuiltinPtr> phi_bb254_11;
  TNode<FixedArray> phi_bb254_13;
  TNode<IntPtrT> phi_bb254_14;
  TNode<IntPtrT> phi_bb254_15;
  TNode<BoolT> phi_bb254_16;
  TNode<PrimitiveHeapObject> phi_bb254_17;
  TNode<UintPtrT> phi_bb254_18;
  TNode<IntPtrT> phi_bb254_19;
  TNode<IntPtrT> phi_bb254_23;
  TNode<String> tmp333;
  if (block254.is_used()) {
    ca_.Bind(&block254, &phi_bb254_10, &phi_bb254_11, &phi_bb254_13, &phi_bb254_14, &phi_bb254_15, &phi_bb254_16, &phi_bb254_17, &phi_bb254_18, &phi_bb254_19, &phi_bb254_23);
    tmp333 = BufferJoin_0(state_, TNode<Context>{p_context}, TorqueStructBuffer_0{TNode<FixedArray>{tmp5}, TNode<FixedArray>{phi_bb254_13}, TNode<IntPtrT>{phi_bb254_14}, TNode<IntPtrT>{phi_bb254_15}, TNode<BoolT>{phi_bb254_16}, TNode<PrimitiveHeapObject>{phi_bb254_17}}, TNode<String>{p_sep});
    ca_.Goto(&block319);
  }

    ca_.Bind(&block319);
  return TNode<String>{tmp333};
}

TF_BUILTIN(LoadJoinTypedElement_Int32Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Int32Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Float16Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Float16Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Float32Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Float32Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Float64Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Float64Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Uint8ClampedElements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Uint8ClampedElements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_BigUint64Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_BigUint64Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_BigInt64Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_BigInt64Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Uint8Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Uint8Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Int8Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Int8Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Uint16Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Uint16Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Int16Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Int16Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

TF_BUILTIN(LoadJoinTypedElement_Uint32Elements_0, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSReceiver> parameter1 = UncheckedParameter<JSReceiver>(Descriptor::kReceiver);
  USE(parameter1);
  TNode<UintPtrT> parameter2 = UncheckedParameter<UintPtrT>(Descriptor::kK);
  USE(parameter2);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSTypedArray> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<Numeric> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = UnsafeCast_JSTypedArray_0(state_, TNode<Context>{parameter0}, TNode<Object>{parameter1});
    tmp1 = CodeStubAssembler(state_).LoadJSTypedArrayDataPtr(TNode<JSTypedArray>{tmp0});
    tmp2 = CodeStubAssembler(state_).LoadFixedTypedArrayElementAsTagged(TNode<RawPtrT>{tmp1}, TNode<UintPtrT>{parameter2}, (KindForArrayType_Uint32Elements_0(state_)));
    CodeStubAssembler(state_).Return(tmp2);
  }
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=631&c=10
TNode<String> ArrayJoinImpl_JSTypedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSReceiver> p_receiver, TNode<String> p_sep, TNode<Number> p_lengthNumber, bool p_useToLocaleString, TNode<JSAny> p_locales, TNode<JSAny> p_options, TNode<BuiltinPtr> p_initialLoadFn) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block5(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block16(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block18(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block24(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject> block27(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String> block11(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, BoolT> block41(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block37(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block38(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block42(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block43(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block44(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block62(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block63(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block56(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block66(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block67(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block88(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block89(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block97(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block98(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block45(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block36(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block101(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block113(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block114(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block121(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block120(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block123(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block122(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block133(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block140(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block141(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block134(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block144(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block145(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, IntPtrT> block146(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block157(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block158(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block166(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block167(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block175(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block176(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block135(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block118(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block182(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block181(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block195(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, FixedArray> block196(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block117(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block102(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block208(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, String, Object, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block215(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, BoolT, PrimitiveHeapObject, String, String, String, String, Object, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block216(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block209(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block219(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block220(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object, IntPtrT> block221(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object, FixedArray> block232(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object, FixedArray> block233(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block241(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block242(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block250(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block251(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String, String, Object> block210(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, String, String, String> block103(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block257(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block258(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, BoolT> block259(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block255(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block256(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block260(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block261(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block262(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block273(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block280(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block281(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block274(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block284(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block285(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block286(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray> block297(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT, FixedArray> block298(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block306(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block307(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block315(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block316(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block275(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT, IntPtrT> block263(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, BuiltinPtr, FixedArray, IntPtrT, IntPtrT, BoolT, PrimitiveHeapObject, UintPtrT, IntPtrT, IntPtrT> block254(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block319(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Map> tmp1;
  TNode<UintPtrT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<FixedArray> tmp5;
  TNode<FixedArray> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<BoolT> tmp9;
  TNode<PrimitiveHeapObject> tmp10;
  TNode<UintPtrT> tmp11;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp1 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{p_receiver, tmp0});
    tmp2 = Convert_uintptr_Number_0(state_, TNode<Number>{p_lengthNumber});
    tmp3 = CodeStubAssembler(state_).LoadStringLengthAsWord(TNode<String>{p_sep});
    tmp4 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp5, tmp6, tmp7, tmp8, tmp9, tmp10) = NewBuffer_0(state_, TNode<UintPtrT>{tmp2}, TNode<String>{p_sep}).Flatten();
    tmp11 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block4, tmp4, p_initialLoadFn, tmp6, tmp7, tmp8, tmp9, tmp10, tmp11);
  }

  TNode<IntPtrT> phi_bb4_10;
  TNode<BuiltinPtr> phi_bb4_11;
  TNode<FixedArray> phi_bb4_13;
  TNode<IntPtrT> phi_bb4_14;
  TNode<IntPtrT> phi_bb4_15;
  TNode<BoolT> phi_bb4_16;
  TNode<PrimitiveHeapObject> phi_bb4_17;
  TNode<UintPtrT> phi_bb4_18;
  TNode<BoolT> tmp12;
  if (block4.is_used()) {
    ca_.Bind(&block4, &phi_bb4_10, &phi_bb4_11, &phi_bb4_13, &phi_bb4_14, &phi_bb4_15, &phi_bb4_16, &phi_bb4_17, &phi_bb4_18);
    tmp12 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{phi_bb4_18}, TNode<UintPtrT>{tmp2});
    ca_.Branch(tmp12, &block2, std::vector<compiler::Node*>{phi_bb4_10, phi_bb4_11, phi_bb4_13, phi_bb4_14, phi_bb4_15, phi_bb4_16, phi_bb4_17, phi_bb4_18}, &block3, std::vector<compiler::Node*>{phi_bb4_10, phi_bb4_11, phi_bb4_13, phi_bb4_14, phi_bb4_15, phi_bb4_16, phi_bb4_17, phi_bb4_18});
  }

  TNode<IntPtrT> phi_bb2_10;
  TNode<BuiltinPtr> phi_bb2_11;
  TNode<FixedArray> phi_bb2_13;
  TNode<IntPtrT> phi_bb2_14;
  TNode<IntPtrT> phi_bb2_15;
  TNode<BoolT> phi_bb2_16;
  TNode<PrimitiveHeapObject> phi_bb2_17;
  TNode<UintPtrT> phi_bb2_18;
  TNode<BoolT> tmp13;
  if (block2.is_used()) {
    ca_.Bind(&block2, &phi_bb2_10, &phi_bb2_11, &phi_bb2_13, &phi_bb2_14, &phi_bb2_15, &phi_bb2_16, &phi_bb2_17, &phi_bb2_18);
    tmp13 = CannotUseSameArrayAccessor_JSTypedArray_0(state_, TNode<Context>{p_context}, TNode<BuiltinPtr>{phi_bb2_11}, TNode<JSReceiver>{p_receiver}, TNode<Map>{tmp1}, TNode<Number>{p_lengthNumber});
    ca_.Branch(tmp13, &block5, std::vector<compiler::Node*>{phi_bb2_10, phi_bb2_11, phi_bb2_13, phi_bb2_14, phi_bb2_15, phi_bb2_16, phi_bb2_17, phi_bb2_18}, &block6, std::vector<compiler::Node*>{phi_bb2_10, phi_bb2_11, phi_bb2_13, phi_bb2_14, phi_bb2_15, phi_bb2_16, phi_bb2_17, phi_bb2_18});
  }

  TNode<IntPtrT> phi_bb5_10;
  TNode<BuiltinPtr> phi_bb5_11;
  TNode<FixedArray> phi_bb5_13;
  TNode<IntPtrT> phi_bb5_14;
  TNode<IntPtrT> phi_bb5_15;
  TNode<BoolT> phi_bb5_16;
  TNode<PrimitiveHeapObject> phi_bb5_17;
  TNode<UintPtrT> phi_bb5_18;
  if (block5.is_used()) {
    ca_.Bind(&block5, &phi_bb5_10, &phi_bb5_11, &phi_bb5_13, &phi_bb5_14, &phi_bb5_15, &phi_bb5_16, &phi_bb5_17, &phi_bb5_18);
    ca_.Goto(&block6, phi_bb5_10, ca_.UncheckedCast<BuiltinPtr>(ca_.SmiConstant(Builtin::kLoadJoinElement_GenericElementsAccessor_0)), phi_bb5_13, phi_bb5_14, phi_bb5_15, phi_bb5_16, phi_bb5_17, phi_bb5_18);
  }

  TNode<IntPtrT> phi_bb6_10;
  TNode<BuiltinPtr> phi_bb6_11;
  TNode<FixedArray> phi_bb6_13;
  TNode<IntPtrT> phi_bb6_14;
  TNode<IntPtrT> phi_bb6_15;
  TNode<BoolT> phi_bb6_16;
  TNode<PrimitiveHeapObject> phi_bb6_17;
  TNode<UintPtrT> phi_bb6_18;
  TNode<UintPtrT> tmp14;
  TNode<BoolT> tmp15;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_10, &phi_bb6_11, &phi_bb6_13, &phi_bb6_14, &phi_bb6_15, &phi_bb6_16, &phi_bb6_17, &phi_bb6_18);
    tmp14 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp15 = CodeStubAssembler(state_).UintPtrGreaterThan(TNode<UintPtrT>{phi_bb6_18}, TNode<UintPtrT>{tmp14});
    ca_.Branch(tmp15, &block7, std::vector<compiler::Node*>{phi_bb6_10, phi_bb6_13, phi_bb6_14, phi_bb6_15, phi_bb6_16, phi_bb6_17, phi_bb6_18}, &block8, std::vector<compiler::Node*>{phi_bb6_10, phi_bb6_13, phi_bb6_14, phi_bb6_15, phi_bb6_16, phi_bb6_17, phi_bb6_18});
  }

  TNode<IntPtrT> phi_bb7_10;
  TNode<FixedArray> phi_bb7_13;
  TNode<IntPtrT> phi_bb7_14;
  TNode<IntPtrT> phi_bb7_15;
  TNode<BoolT> phi_bb7_16;
  TNode<PrimitiveHeapObject> phi_bb7_17;
  TNode<UintPtrT> phi_bb7_18;
  TNode<IntPtrT> tmp16;
  TNode<IntPtrT> tmp17;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_10, &phi_bb7_13, &phi_bb7_14, &phi_bb7_15, &phi_bb7_16, &phi_bb7_17, &phi_bb7_18);
    tmp16 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp17 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb7_10}, TNode<IntPtrT>{tmp16});
    ca_.Goto(&block8, tmp17, phi_bb7_13, phi_bb7_14, phi_bb7_15, phi_bb7_16, phi_bb7_17, phi_bb7_18);
  }

  TNode<IntPtrT> phi_bb8_10;
  TNode<FixedArray> phi_bb8_13;
  TNode<IntPtrT> phi_bb8_14;
  TNode<IntPtrT> phi_bb8_15;
  TNode<BoolT> phi_bb8_16;
  TNode<PrimitiveHeapObject> phi_bb8_17;
  TNode<UintPtrT> phi_bb8_18;
  TNode<UintPtrT> tmp18;
  TNode<UintPtrT> tmp19;
  TNode<JSAny> tmp20;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_10, &phi_bb8_13, &phi_bb8_14, &phi_bb8_15, &phi_bb8_16, &phi_bb8_17, &phi_bb8_18);
    tmp18 = FromConstexpr_uintptr_constexpr_int31_0(state_, 1);
    tmp19 = CodeStubAssembler(state_).UintPtrAdd(TNode<UintPtrT>{phi_bb8_18}, TNode<UintPtrT>{tmp18});
tmp20 = TORQUE_CAST(CodeStubAssembler(state_).CallBuiltinPointer(Builtins::CallInterfaceDescriptorFor(ExampleBuiltinForTorqueFunctionPointerType(3)), phi_bb6_11, p_context, p_receiver, phi_bb8_18));
    if ((p_useToLocaleString)) {
      ca_.Goto(&block9, phi_bb8_13, phi_bb8_14, phi_bb8_15, phi_bb8_16, phi_bb8_17);
    } else {
      ca_.Goto(&block10, phi_bb8_13, phi_bb8_14, phi_bb8_15, phi_bb8_16, phi_bb8_17);
    }
  }

  TNode<FixedArray> phi_bb9_13;
  TNode<IntPtrT> phi_bb9_14;
  TNode<IntPtrT> phi_bb9_15;
  TNode<BoolT> phi_bb9_16;
  TNode<PrimitiveHeapObject> phi_bb9_17;
  TNode<String> tmp21;
  TNode<String> tmp22;
  TNode<BoolT> tmp23;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_13, &phi_bb9_14, &phi_bb9_15, &phi_bb9_16, &phi_bb9_17);
    tmp21 = ca_.CallBuiltin<String>(Builtin::kConvertToLocaleString, p_context, tmp20, p_locales, p_options);
    tmp22 = kEmptyString_0(state_);
    tmp23 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp21}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp22});
    ca_.Branch(tmp23, &block12, std::vector<compiler::Node*>{phi_bb9_13, phi_bb9_14, phi_bb9_15, phi_bb9_16, phi_bb9_17}, &block13, std::vector<compiler::Node*>{phi_bb9_13, phi_bb9_14, phi_bb9_15, phi_bb9_16, phi_bb9_17});
  }

  TNode<FixedArray> phi_bb12_13;
  TNode<IntPtrT> phi_bb12_14;
  TNode<IntPtrT> phi_bb12_15;
  TNode<BoolT> phi_bb12_16;
  TNode<PrimitiveHeapObject> phi_bb12_17;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_13, &phi_bb12_14, &phi_bb12_15, &phi_bb12_16, &phi_bb12_17);
    ca_.Goto(&block4, phi_bb8_10, phi_bb6_11, phi_bb12_13, phi_bb12_14, phi_bb12_15, phi_bb12_16, phi_bb12_17, tmp19);
  }

  TNode<FixedArray> phi_bb13_13;
  TNode<IntPtrT> phi_bb13_14;
  TNode<IntPtrT> phi_bb13_15;
  TNode<BoolT> phi_bb13_16;
  TNode<PrimitiveHeapObject> phi_bb13_17;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_13, &phi_bb13_14, &phi_bb13_15, &phi_bb13_16, &phi_bb13_17);
    ca_.Goto(&block11, phi_bb13_13, phi_bb13_14, phi_bb13_15, phi_bb13_16, phi_bb13_17, tmp21);
  }

  TNode<FixedArray> phi_bb10_13;
  TNode<IntPtrT> phi_bb10_14;
  TNode<IntPtrT> phi_bb10_15;
  TNode<BoolT> phi_bb10_16;
  TNode<PrimitiveHeapObject> phi_bb10_17;
  TNode<String> tmp24;
  if (block10.is_used()) {
    ca_.Bind(&block10, &phi_bb10_13, &phi_bb10_14, &phi_bb10_15, &phi_bb10_16, &phi_bb10_17);
    compiler::CodeAssemblerLabel label25(&ca_);
    tmp24 = Cast_String_1(state_, TNode<Context>{p_context}, TNode<Object>{tmp20}, &label25);
    ca_.Goto(&block16, phi_bb10_13, phi_bb10_14, phi_bb10_15, phi_bb10_16, phi_bb10_17);
    if (label25.is_used()) {
      ca_.Bind(&label25);
      ca_.Goto(&block17, phi_bb10_13, phi_bb10_14, phi_bb10_15, phi_bb10_16, phi_bb10_17);
    }
  }

  TNode<FixedArray> phi_bb17_13;
  TNode<IntPtrT> phi_bb17_14;
  TNode<IntPtrT> phi_bb17_15;
  TNode<BoolT> phi_bb17_16;
  TNode<PrimitiveHeapObject> phi_bb17_17;
  TNode<Number> tmp26;
  if (block17.is_used()) {
    ca_.Bind(&block17, &phi_bb17_13, &phi_bb17_14, &phi_bb17_15, &phi_bb17_16, &phi_bb17_17);
    compiler::CodeAssemblerLabel label27(&ca_);
    tmp26 = Cast_Number_0(state_, TNode<Object>{ca_.UncheckedCast<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, Symbol, Undefined>>(tmp20)}, &label27);
    ca_.Goto(&block22, phi_bb17_13, phi_bb17_14, phi_bb17_15, phi_bb17_16, phi_bb17_17);
    if (label27.is_used()) {
      ca_.Bind(&label27);
      ca_.Goto(&block23, phi_bb17_13, phi_bb17_14, phi_bb17_15, phi_bb17_16, phi_bb17_17);
    }
  }

  TNode<FixedArray> phi_bb16_13;
  TNode<IntPtrT> phi_bb16_14;
  TNode<IntPtrT> phi_bb16_15;
  TNode<BoolT> phi_bb16_16;
  TNode<PrimitiveHeapObject> phi_bb16_17;
  TNode<String> tmp28;
  TNode<BoolT> tmp29;
  if (block16.is_used()) {
    ca_.Bind(&block16, &phi_bb16_13, &phi_bb16_14, &phi_bb16_15, &phi_bb16_16, &phi_bb16_17);
    tmp28 = kEmptyString_0(state_);
    tmp29 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp24}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp28});
    ca_.Branch(tmp29, &block18, std::vector<compiler::Node*>{phi_bb16_13, phi_bb16_14, phi_bb16_15, phi_bb16_16, phi_bb16_17}, &block19, std::vector<compiler::Node*>{phi_bb16_13, phi_bb16_14, phi_bb16_15, phi_bb16_16, phi_bb16_17});
  }

  TNode<FixedArray> phi_bb18_13;
  TNode<IntPtrT> phi_bb18_14;
  TNode<IntPtrT> phi_bb18_15;
  TNode<BoolT> phi_bb18_16;
  TNode<PrimitiveHeapObject> phi_bb18_17;
  if (block18.is_used()) {
    ca_.Bind(&block18, &phi_bb18_13, &phi_bb18_14, &phi_bb18_15, &phi_bb18_16, &phi_bb18_17);
    ca_.Goto(&block4, phi_bb8_10, phi_bb6_11, phi_bb18_13, phi_bb18_14, phi_bb18_15, phi_bb18_16, phi_bb18_17, tmp19);
  }

  TNode<FixedArray> phi_bb19_13;
  TNode<IntPtrT> phi_bb19_14;
  TNode<IntPtrT> phi_bb19_15;
  TNode<BoolT> phi_bb19_16;
  TNode<PrimitiveHeapObject> phi_bb19_17;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_13, &phi_bb19_14, &phi_bb19_15, &phi_bb19_16, &phi_bb19_17);
    ca_.Goto(&block14, phi_bb19_13, phi_bb19_14, phi_bb19_15, phi_bb19_16, phi_bb19_17, tmp24);
  }

  TNode<FixedArray> phi_bb23_13;
  TNode<IntPtrT> phi_bb23_14;
  TNode<IntPtrT> phi_bb23_15;
  TNode<BoolT> phi_bb23_16;
  TNode<PrimitiveHeapObject> phi_bb23_17;
  TNode<BoolT> tmp30;
  if (block23.is_used()) {
    ca_.Bind(&block23, &phi_bb23_13, &phi_bb23_14, &phi_bb23_15, &phi_bb23_16, &phi_bb23_17);
    tmp30 = CodeStubAssembler(state_).IsNullOrUndefined(TNode<Object>{ca_.UncheckedCast<Union<BigInt, Boolean, JSReceiver, Null, Symbol, Undefined>>(tmp20)});
    ca_.Branch(tmp30, &block24, std::vector<compiler::Node*>{phi_bb23_13, phi_bb23_14, phi_bb23_15, phi_bb23_16, phi_bb23_17}, &block25, std::vector<compiler::Node*>{phi_bb23_13, phi_bb23_14, phi_bb23_15, phi_bb23_16, phi_bb23_17});
  }

  TNode<FixedArray> phi_bb22_13;
  TNode<IntPtrT> phi_bb22_14;
  TNode<IntPtrT> phi_bb22_15;
  TNode<BoolT> phi_bb22_16;
  TNode<PrimitiveHeapObject> phi_bb22_17;
  TNode<String> tmp31;
  if (block22.is_used()) {
    ca_.Bind(&block22, &phi_bb22_13, &phi_bb22_14, &phi_bb22_15, &phi_bb22_16, &phi_bb22_17);
    tmp31 = CodeStubAssembler(state_).NumberToString(TNode<Number>{tmp26});
    ca_.Goto(&block20, phi_bb22_13, phi_bb22_14, phi_bb22_15, phi_bb22_16, phi_bb22_17, tmp31);
  }

  TNode<FixedArray> phi_bb24_13;
  TNode<IntPtrT> phi_bb24_14;
  TNode<IntPtrT> phi_bb24_15;
  TNode<BoolT> phi_bb24_16;
  TNode<PrimitiveHeapObject> phi_bb24_17;
  if (block24.is_used()) {
    ca_.Bind(&block24, &phi_bb24_13, &phi_bb24_14, &phi_bb24_15, &phi_bb24_16, &phi_bb24_17);
    ca_.Goto(&block4, phi_bb8_10, phi_bb6_11, phi_bb24_13, phi_bb24_14, phi_bb24_15, phi_bb24_16, phi_bb24_17, tmp19);
  }

  TNode<FixedArray> phi_bb25_13;
  TNode<IntPtrT> phi_bb25_14;
  TNode<IntPtrT> phi_bb25_15;
  TNode<BoolT> phi_bb25_16;
  TNode<PrimitiveHeapObject> phi_bb25_17;
  TNode<String> tmp32;
  TNode<String> tmp33;
  TNode<BoolT> tmp34;
  if (block25.is_used()) {
    ca_.Bind(&block25, &phi_bb25_13, &phi_bb25_14, &phi_bb25_15, &phi_bb25_16, &phi_bb25_17);
    tmp32 = ToString_Inline_0(state_, TNode<Context>{p_context}, TNode<JSAny>{ca_.UncheckedCast<Union<BigInt, Boolean, JSReceiver, Null, Symbol, Undefined>>(tmp20)});
    tmp33 = kEmptyString_0(state_);
    tmp34 = CodeStubAssembler(state_).TaggedEqual(TNode<Object>{tmp32}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WasmFuncRef, WasmNull, WeakCell>>{tmp33});
    ca_.Branch(tmp34, &block26, std::vector<compiler::Node*>{phi_bb25_13, phi_bb25_14, phi_bb25_15, phi_bb25_16, phi_bb25_17}, &block27, std::vector<compiler::Node*>{phi_bb25_13, phi_bb25_14, phi_bb25_15, phi_bb25_16, phi_bb25_17});
  }

  TNode<FixedArray> phi_bb26_13;
  TNode<IntPtrT> phi_bb26_14;
  TNode<IntPtrT> phi_bb26_15;
  TNode<BoolT> phi_bb26_16;
  TNode<PrimitiveHeapObject> phi_bb26_17;
  if (block26.is_used()) {
    ca_.Bind(&block26, &phi_bb26_13, &phi_bb26_14, &phi_bb26_15, &phi_bb26_16, &phi_bb26_17);
    ca_.Goto(&block4, phi_bb8_10, phi_bb6_11, phi_bb26_13, phi_bb26_14, phi_bb26_15, phi_bb26_16, phi_bb26_17, tmp19);
  }

  TNode<FixedArray> phi_bb27_13;
  TNode<IntPtrT> phi_bb27_14;
  TNode<IntPtrT> phi_bb27_15;
  TNode<BoolT> phi_bb27_16;
  TNode<PrimitiveHeapObject> phi_bb27_17;
  if (block27.is_used()) {
    ca_.Bind(&block27, &phi_bb27_13, &phi_bb27_14, &phi_bb27_15, &phi_bb27_16, &phi_bb27_17);
    ca_.Goto(&block20, phi_bb27_13, phi_bb27_14, phi_bb27_15, phi_bb27_16, phi_bb27_17, tmp32);
  }

  TNode<FixedArray> phi_bb20_13;
  TNode<IntPtrT> phi_bb20_14;
  TNode<IntPtrT> phi_bb20_15;
  TNode<BoolT> phi_bb20_16;
  TNode<PrimitiveHeapObject> phi_bb20_17;
  TNode<String> phi_bb20_20;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_13, &phi_bb20_14, &phi_bb20_15, &phi_bb20_16, &phi_bb20_17, &phi_bb20_20);
    ca_.Goto(&block14, phi_bb20_13, phi_bb20_14, phi_bb20_15, phi_bb20_16, phi_bb20_17, phi_bb20_20);
  }

  TNode<FixedArray> phi_bb14_13;
  TNode<IntPtrT> phi_bb14_14;
  TNode<IntPtrT> phi_bb14_15;
  TNode<BoolT> phi_bb14_16;
  TNode<PrimitiveHeapObject> phi_bb14_17;
  TNode<String> phi_bb14_20;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_13, &phi_bb14_14, &phi_bb14_15, &phi_bb14_16, &phi_bb14_17, &phi_bb14_20);
    ca_.Goto(&block11, phi_bb14_13, phi_bb14_14, phi_bb14_15, phi_bb14_16, phi_bb14_17, phi_bb14_20);
  }

  TNode<FixedArray> phi_bb11_13;
  TNode<IntPtrT> phi_bb11_14;
  TNode<IntPtrT> phi_bb11_15;
  TNode<BoolT> phi_bb11_16;
  TNode<PrimitiveHeapObject> phi_bb11_17;
  TNode<String> phi_bb11_20;
  TNode<IntPtrT> tmp35;
  TNode<BoolT> tmp36;
  TNode<IntPtrT> tmp37;
  TNode<BoolT> tmp38;
  TNode<BoolT> tmp39;
  TNode<IntPtrT> tmp40;
  TNode<BoolT> tmp41;
  if (block11.is_used()) {
    ca_.Bind(&block11, &phi_bb11_13, &phi_bb11_14, &phi_bb11_15, &phi_bb11_16, &phi_bb11_17, &phi_bb11_20);
    tmp35 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp36 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb11_14}, TNode<IntPtrT>{tmp35});
    tmp37 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp38 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb8_10}, TNode<IntPtrT>{tmp37});
    tmp39 = CodeStubAssembler(state_).Word32Or(TNode<BoolT>{tmp36}, TNode<BoolT>{tmp38});
    tmp40 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp41 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb8_10}, TNode<IntPtrT>{tmp40});
    ca_.Branch(tmp41, &block39, std::vector<compiler::Node*>{phi_bb11_13, phi_bb11_14, phi_bb11_15, phi_bb11_16, phi_bb11_17, phi_bb11_20, phi_bb11_20, phi_bb11_20}, &block40, std::vector<compiler::Node*>{phi_bb11_13, phi_bb11_14, phi_bb11_15, phi_bb11_16, phi_bb11_17, phi_bb11_20, phi_bb11_20, phi_bb11_20});
  }

  TNode<FixedArray> phi_bb39_13;
  TNode<IntPtrT> phi_bb39_14;
  TNode<IntPtrT> phi_bb39_15;
  TNode<BoolT> phi_bb39_16;
  TNode<PrimitiveHeapObject> phi_bb39_17;
  TNode<String> phi_bb39_20;
  TNode<String> phi_bb39_21;
  TNode<String> phi_bb39_26;
  TNode<BoolT> tmp42;
  if (block39.is_used()) {
    ca_.Bind(&block39, &phi_bb39_13, &phi_bb39_14, &phi_bb39_15, &phi_bb39_16, &phi_bb39_17, &phi_bb39_20, &phi_bb39_21, &phi_bb39_26);
    tmp42 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block41, phi_bb39_13, phi_bb39_14, phi_bb39_15, phi_bb39_16, phi_bb39_17, phi_bb39_20, phi_bb39_21, phi_bb39_26, tmp42);
  }

  TNode<FixedArray> phi_bb40_13;
  TNode<IntPtrT> phi_bb40_14;
  TNode<IntPtrT> phi_bb40_15;
  TNode<BoolT> phi_bb40_16;
  TNode<PrimitiveHeapObject> phi_bb40_17;
  TNode<String> phi_bb40_20;
  TNode<String> phi_bb40_21;
  TNode<String> phi_bb40_26;
  TNode<IntPtrT> tmp43;
  TNode<BoolT> tmp44;
  if (block40.is_used()) {
    ca_.Bind(&block40, &phi_bb40_13, &phi_bb40_14, &phi_bb40_15, &phi_bb40_16, &phi_bb40_17, &phi_bb40_20, &phi_bb40_21, &phi_bb40_26);
    tmp43 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp44 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp43});
    ca_.Goto(&block41, phi_bb40_13, phi_bb40_14, phi_bb40_15, phi_bb40_16, phi_bb40_17, phi_bb40_20, phi_bb40_21, phi_bb40_26, tmp44);
  }

  TNode<FixedArray> phi_bb41_13;
  TNode<IntPtrT> phi_bb41_14;
  TNode<IntPtrT> phi_bb41_15;
  TNode<BoolT> phi_bb41_16;
  TNode<PrimitiveHeapObject> phi_bb41_17;
  TNode<String> phi_bb41_20;
  TNode<String> phi_bb41_21;
  TNode<String> phi_bb41_26;
  TNode<BoolT> phi_bb41_39;
  if (block41.is_used()) {
    ca_.Bind(&block41, &phi_bb41_13, &phi_bb41_14, &phi_bb41_15, &phi_bb41_16, &phi_bb41_17, &phi_bb41_20, &phi_bb41_21, &phi_bb41_26, &phi_bb41_39);
    ca_.Branch(phi_bb41_39, &block37, std::vector<compiler::Node*>{phi_bb41_13, phi_bb41_14, phi_bb41_15, phi_bb41_16, phi_bb41_17, phi_bb41_20, phi_bb41_21, phi_bb41_26}, &block38, std::vector<compiler::Node*>{phi_bb41_13, phi_bb41_14, phi_bb41_15, phi_bb41_16, phi_bb41_17, phi_bb41_20, phi_bb41_21, phi_bb41_26});
  }

  TNode<FixedArray> phi_bb37_13;
  TNode<IntPtrT> phi_bb37_14;
  TNode<IntPtrT> phi_bb37_15;
  TNode<BoolT> phi_bb37_16;
  TNode<PrimitiveHeapObject> phi_bb37_17;
  TNode<String> phi_bb37_20;
  TNode<String> phi_bb37_21;
  TNode<String> phi_bb37_26;
  if (block37.is_used()) {
    ca_.Bind(&block37, &phi_bb37_13, &phi_bb37_14, &phi_bb37_15, &phi_bb37_16, &phi_bb37_17, &phi_bb37_20, &phi_bb37_21, &phi_bb37_26);
    ca_.Goto(&block36, phi_bb37_13, phi_bb37_14, phi_bb37_15, phi_bb37_16, phi_bb37_17, phi_bb37_20, phi_bb37_21, phi_bb37_26);
  }

  TNode<FixedArray> phi_bb38_13;
  TNode<IntPtrT> phi_bb38_14;
  TNode<IntPtrT> phi_bb38_15;
  TNode<BoolT> phi_bb38_16;
  TNode<PrimitiveHeapObject> phi_bb38_17;
  TNode<String> phi_bb38_20;
  TNode<String> phi_bb38_21;
  TNode<String> phi_bb38_26;
  TNode<IntPtrT> tmp45;
  TNode<IntPtrT> tmp46;
  TNode<BoolT> tmp47;
  if (block38.is_used()) {
    ca_.Bind(&block38, &phi_bb38_13, &phi_bb38_14, &phi_bb38_15, &phi_bb38_16, &phi_bb38_17, &phi_bb38_20, &phi_bb38_21, &phi_bb38_26);
    tmp45 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{phi_bb8_10});
    tmp46 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp45}, TNode<IntPtrT>{tmp3});
    tmp47 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{tmp46}, TNode<IntPtrT>{phi_bb8_10});
    ca_.Branch(tmp47, &block42, std::vector<compiler::Node*>{phi_bb38_13, phi_bb38_14, phi_bb38_15, phi_bb38_16, phi_bb38_17, phi_bb38_20, phi_bb38_21, phi_bb38_26}, &block43, std::vector<compiler::Node*>{phi_bb38_13, phi_bb38_14, phi_bb38_15, phi_bb38_16, phi_bb38_17, phi_bb38_20, phi_bb38_21, phi_bb38_26});
  }

  TNode<FixedArray> phi_bb42_13;
  TNode<IntPtrT> phi_bb42_14;
  TNode<IntPtrT> phi_bb42_15;
  TNode<BoolT> phi_bb42_16;
  TNode<PrimitiveHeapObject> phi_bb42_17;
  TNode<String> phi_bb42_20;
  TNode<String> phi_bb42_21;
  TNode<String> phi_bb42_26;
  if (block42.is_used()) {
    ca_.Bind(&block42, &phi_bb42_13, &phi_bb42_14, &phi_bb42_15, &phi_bb42_16, &phi_bb42_17, &phi_bb42_20, &phi_bb42_21, &phi_bb42_26);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowInvalidStringLength, p_context);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb43_13;
  TNode<IntPtrT> phi_bb43_14;
  TNode<IntPtrT> phi_bb43_15;
  TNode<BoolT> phi_bb43_16;
  TNode<PrimitiveHeapObject> phi_bb43_17;
  TNode<String> phi_bb43_20;
  TNode<String> phi_bb43_21;
  TNode<String> phi_bb43_26;
  TNode<IntPtrT> tmp48;
  if (block43.is_used()) {
    ca_.Bind(&block43, &phi_bb43_13, &phi_bb43_14, &phi_bb43_15, &phi_bb43_16, &phi_bb43_17, &phi_bb43_20, &phi_bb43_21, &phi_bb43_26);
    tmp48 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb43_15}, TNode<IntPtrT>{tmp45});
    ca_.Branch(tmp39, &block44, std::vector<compiler::Node*>{phi_bb43_13, phi_bb43_14, phi_bb43_16, phi_bb43_17, phi_bb43_20, phi_bb43_21, phi_bb43_26}, &block45, std::vector<compiler::Node*>{phi_bb43_13, phi_bb43_14, phi_bb43_16, phi_bb43_17, phi_bb43_20, phi_bb43_21, phi_bb43_26});
  }

  TNode<FixedArray> phi_bb44_13;
  TNode<IntPtrT> phi_bb44_14;
  TNode<BoolT> phi_bb44_16;
  TNode<PrimitiveHeapObject> phi_bb44_17;
  TNode<String> phi_bb44_20;
  TNode<String> phi_bb44_21;
  TNode<String> phi_bb44_26;
  TNode<Smi> tmp49;
  TNode<IntPtrT> tmp50;
  TNode<BoolT> tmp51;
  if (block44.is_used()) {
    ca_.Bind(&block44, &phi_bb44_13, &phi_bb44_14, &phi_bb44_16, &phi_bb44_17, &phi_bb44_20, &phi_bb44_21, &phi_bb44_26);
    tmp49 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{phi_bb8_10});
    tmp50 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb44_13});
    tmp51 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb44_14}, TNode<IntPtrT>{tmp50});
    ca_.Branch(tmp51, &block55, std::vector<compiler::Node*>{phi_bb44_13, phi_bb44_14, phi_bb44_16, phi_bb44_17, phi_bb44_20, phi_bb44_21, phi_bb44_26}, &block56, std::vector<compiler::Node*>{phi_bb44_13, phi_bb44_14, phi_bb44_16, phi_bb44_17, phi_bb44_20, phi_bb44_21, phi_bb44_26});
  }

  TNode<FixedArray> phi_bb55_13;
  TNode<IntPtrT> phi_bb55_14;
  TNode<BoolT> phi_bb55_16;
  TNode<PrimitiveHeapObject> phi_bb55_17;
  TNode<String> phi_bb55_20;
  TNode<String> phi_bb55_21;
  TNode<String> phi_bb55_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp52;
  TNode<IntPtrT> tmp53;
  TNode<IntPtrT> tmp54;
  TNode<IntPtrT> tmp55;
  TNode<IntPtrT> tmp56;
  TNode<UintPtrT> tmp57;
  TNode<UintPtrT> tmp58;
  TNode<BoolT> tmp59;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_13, &phi_bb55_14, &phi_bb55_16, &phi_bb55_17, &phi_bb55_20, &phi_bb55_21, &phi_bb55_26);
    std::tie(tmp52, tmp53, tmp54) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb55_13}).Flatten();
    tmp55 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp56 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb55_14}, TNode<IntPtrT>{tmp55});
    tmp57 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb55_14});
    tmp58 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp54});
    tmp59 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp57}, TNode<UintPtrT>{tmp58});
    ca_.Branch(tmp59, &block62, std::vector<compiler::Node*>{phi_bb55_13, phi_bb55_16, phi_bb55_17, phi_bb55_20, phi_bb55_21, phi_bb55_26, phi_bb55_13, phi_bb55_14, phi_bb55_14, phi_bb55_14, phi_bb55_14}, &block63, std::vector<compiler::Node*>{phi_bb55_13, phi_bb55_16, phi_bb55_17, phi_bb55_20, phi_bb55_21, phi_bb55_26, phi_bb55_13, phi_bb55_14, phi_bb55_14, phi_bb55_14, phi_bb55_14});
  }

  TNode<FixedArray> phi_bb62_13;
  TNode<BoolT> phi_bb62_16;
  TNode<PrimitiveHeapObject> phi_bb62_17;
  TNode<String> phi_bb62_20;
  TNode<String> phi_bb62_21;
  TNode<String> phi_bb62_26;
  TNode<FixedArray> phi_bb62_43;
  TNode<IntPtrT> phi_bb62_47;
  TNode<IntPtrT> phi_bb62_48;
  TNode<IntPtrT> phi_bb62_52;
  TNode<IntPtrT> phi_bb62_53;
  TNode<IntPtrT> tmp60;
  TNode<IntPtrT> tmp61;
  TNode<Union<HeapObject, TaggedIndex>> tmp62;
  TNode<IntPtrT> tmp63;
  if (block62.is_used()) {
    ca_.Bind(&block62, &phi_bb62_13, &phi_bb62_16, &phi_bb62_17, &phi_bb62_20, &phi_bb62_21, &phi_bb62_26, &phi_bb62_43, &phi_bb62_47, &phi_bb62_48, &phi_bb62_52, &phi_bb62_53);
    tmp60 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb62_53});
    tmp61 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp53}, TNode<IntPtrT>{tmp60});
    std::tie(tmp62, tmp63) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp52}, TNode<IntPtrT>{tmp61}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp62, tmp63}, tmp49);
    ca_.Goto(&block57, phi_bb62_13, tmp56, phi_bb62_16, phi_bb62_17, phi_bb62_20, phi_bb62_21, phi_bb62_26);
  }

  TNode<FixedArray> phi_bb63_13;
  TNode<BoolT> phi_bb63_16;
  TNode<PrimitiveHeapObject> phi_bb63_17;
  TNode<String> phi_bb63_20;
  TNode<String> phi_bb63_21;
  TNode<String> phi_bb63_26;
  TNode<FixedArray> phi_bb63_43;
  TNode<IntPtrT> phi_bb63_47;
  TNode<IntPtrT> phi_bb63_48;
  TNode<IntPtrT> phi_bb63_52;
  TNode<IntPtrT> phi_bb63_53;
  if (block63.is_used()) {
    ca_.Bind(&block63, &phi_bb63_13, &phi_bb63_16, &phi_bb63_17, &phi_bb63_20, &phi_bb63_21, &phi_bb63_26, &phi_bb63_43, &phi_bb63_47, &phi_bb63_48, &phi_bb63_52, &phi_bb63_53);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb56_13;
  TNode<IntPtrT> phi_bb56_14;
  TNode<BoolT> phi_bb56_16;
  TNode<PrimitiveHeapObject> phi_bb56_17;
  TNode<String> phi_bb56_20;
  TNode<String> phi_bb56_21;
  TNode<String> phi_bb56_26;
  TNode<IntPtrT> tmp64;
  TNode<IntPtrT> tmp65;
  TNode<BoolT> tmp66;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_13, &phi_bb56_14, &phi_bb56_16, &phi_bb56_17, &phi_bb56_20, &phi_bb56_21, &phi_bb56_26);
    tmp64 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp50});
    tmp65 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp66 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp64}, TNode<IntPtrT>{tmp65});
    ca_.Branch(tmp66, &block66, std::vector<compiler::Node*>{phi_bb56_13, phi_bb56_14, phi_bb56_16, phi_bb56_17, phi_bb56_20, phi_bb56_21, phi_bb56_26}, &block67, std::vector<compiler::Node*>{phi_bb56_13, phi_bb56_14, phi_bb56_16, phi_bb56_17, phi_bb56_20, phi_bb56_21, phi_bb56_26});
  }

  TNode<FixedArray> phi_bb66_13;
  TNode<IntPtrT> phi_bb66_14;
  TNode<BoolT> phi_bb66_16;
  TNode<PrimitiveHeapObject> phi_bb66_17;
  TNode<String> phi_bb66_20;
  TNode<String> phi_bb66_21;
  TNode<String> phi_bb66_26;
  TNode<IntPtrT> tmp67;
  if (block66.is_used()) {
    ca_.Bind(&block66, &phi_bb66_13, &phi_bb66_14, &phi_bb66_16, &phi_bb66_17, &phi_bb66_20, &phi_bb66_21, &phi_bb66_26);
    tmp67 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block68, phi_bb66_13, phi_bb66_14, phi_bb66_16, phi_bb66_17, phi_bb66_20, phi_bb66_21, phi_bb66_26, tmp67);
  }

  TNode<FixedArray> phi_bb67_13;
  TNode<IntPtrT> phi_bb67_14;
  TNode<BoolT> phi_bb67_16;
  TNode<PrimitiveHeapObject> phi_bb67_17;
  TNode<String> phi_bb67_20;
  TNode<String> phi_bb67_21;
  TNode<String> phi_bb67_26;
  if (block67.is_used()) {
    ca_.Bind(&block67, &phi_bb67_13, &phi_bb67_14, &phi_bb67_16, &phi_bb67_17, &phi_bb67_20, &phi_bb67_21, &phi_bb67_26);
    ca_.Goto(&block68, phi_bb67_13, phi_bb67_14, phi_bb67_16, phi_bb67_17, phi_bb67_20, phi_bb67_21, phi_bb67_26, tmp64);
  }

  TNode<FixedArray> phi_bb68_13;
  TNode<IntPtrT> phi_bb68_14;
  TNode<BoolT> phi_bb68_16;
  TNode<PrimitiveHeapObject> phi_bb68_17;
  TNode<String> phi_bb68_20;
  TNode<String> phi_bb68_21;
  TNode<String> phi_bb68_26;
  TNode<IntPtrT> phi_bb68_44;
  TNode<FixedArray> tmp68;
  TNode<Union<HeapObject, TaggedIndex>> tmp69;
  TNode<IntPtrT> tmp70;
  TNode<IntPtrT> tmp71;
  TNode<UintPtrT> tmp72;
  TNode<IntPtrT> tmp73;
  TNode<UintPtrT> tmp74;
  TNode<UintPtrT> tmp75;
  TNode<BoolT> tmp76;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_13, &phi_bb68_14, &phi_bb68_16, &phi_bb68_17, &phi_bb68_20, &phi_bb68_21, &phi_bb68_26, &phi_bb68_44);
    tmp68 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb68_44});
    std::tie(tmp69, tmp70, tmp71) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb68_13}).Flatten();
    tmp72 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp73 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp72});
    tmp74 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp73});
    tmp75 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp71});
    tmp76 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp74}, TNode<UintPtrT>{tmp75});
    ca_.Branch(tmp76, &block79, std::vector<compiler::Node*>{phi_bb68_13, phi_bb68_14, phi_bb68_16, phi_bb68_17, phi_bb68_20, phi_bb68_21, phi_bb68_26, phi_bb68_13}, &block80, std::vector<compiler::Node*>{phi_bb68_13, phi_bb68_14, phi_bb68_16, phi_bb68_17, phi_bb68_20, phi_bb68_21, phi_bb68_26, phi_bb68_13});
  }

  TNode<FixedArray> phi_bb79_13;
  TNode<IntPtrT> phi_bb79_14;
  TNode<BoolT> phi_bb79_16;
  TNode<PrimitiveHeapObject> phi_bb79_17;
  TNode<String> phi_bb79_20;
  TNode<String> phi_bb79_21;
  TNode<String> phi_bb79_26;
  TNode<FixedArray> phi_bb79_46;
  TNode<IntPtrT> tmp77;
  TNode<IntPtrT> tmp78;
  TNode<Union<HeapObject, TaggedIndex>> tmp79;
  TNode<IntPtrT> tmp80;
  TNode<Union<HeapObject, TaggedIndex>> tmp81;
  TNode<IntPtrT> tmp82;
  TNode<IntPtrT> tmp83;
  TNode<UintPtrT> tmp84;
  TNode<IntPtrT> tmp85;
  TNode<UintPtrT> tmp86;
  TNode<UintPtrT> tmp87;
  TNode<BoolT> tmp88;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_13, &phi_bb79_14, &phi_bb79_16, &phi_bb79_17, &phi_bb79_20, &phi_bb79_21, &phi_bb79_26, &phi_bb79_46);
    tmp77 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp73});
    tmp78 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp70}, TNode<IntPtrT>{tmp77});
    std::tie(tmp79, tmp80) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp69}, TNode<IntPtrT>{tmp78}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp79, tmp80}, tmp68);
    std::tie(tmp81, tmp82, tmp83) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp68}).Flatten();
    tmp84 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp85 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp84});
    tmp86 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp85});
    tmp87 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp83});
    tmp88 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp86}, TNode<UintPtrT>{tmp87});
    ca_.Branch(tmp88, &block88, std::vector<compiler::Node*>{phi_bb79_13, phi_bb79_14, phi_bb79_16, phi_bb79_17, phi_bb79_20, phi_bb79_21, phi_bb79_26}, &block89, std::vector<compiler::Node*>{phi_bb79_13, phi_bb79_14, phi_bb79_16, phi_bb79_17, phi_bb79_20, phi_bb79_21, phi_bb79_26});
  }

  TNode<FixedArray> phi_bb80_13;
  TNode<IntPtrT> phi_bb80_14;
  TNode<BoolT> phi_bb80_16;
  TNode<PrimitiveHeapObject> phi_bb80_17;
  TNode<String> phi_bb80_20;
  TNode<String> phi_bb80_21;
  TNode<String> phi_bb80_26;
  TNode<FixedArray> phi_bb80_46;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_13, &phi_bb80_14, &phi_bb80_16, &phi_bb80_17, &phi_bb80_20, &phi_bb80_21, &phi_bb80_26, &phi_bb80_46);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb88_13;
  TNode<IntPtrT> phi_bb88_14;
  TNode<BoolT> phi_bb88_16;
  TNode<PrimitiveHeapObject> phi_bb88_17;
  TNode<String> phi_bb88_20;
  TNode<String> phi_bb88_21;
  TNode<String> phi_bb88_26;
  TNode<IntPtrT> tmp89;
  TNode<IntPtrT> tmp90;
  TNode<Union<HeapObject, TaggedIndex>> tmp91;
  TNode<IntPtrT> tmp92;
  TNode<Undefined> tmp93;
  TNode<Union<HeapObject, TaggedIndex>> tmp94;
  TNode<IntPtrT> tmp95;
  TNode<IntPtrT> tmp96;
  TNode<UintPtrT> tmp97;
  TNode<IntPtrT> tmp98;
  TNode<UintPtrT> tmp99;
  TNode<UintPtrT> tmp100;
  TNode<BoolT> tmp101;
  if (block88.is_used()) {
    ca_.Bind(&block88, &phi_bb88_13, &phi_bb88_14, &phi_bb88_16, &phi_bb88_17, &phi_bb88_20, &phi_bb88_21, &phi_bb88_26);
    tmp89 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp85});
    tmp90 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp82}, TNode<IntPtrT>{tmp89});
    std::tie(tmp91, tmp92) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp81}, TNode<IntPtrT>{tmp90}).Flatten();
    tmp93 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp91, tmp92}, tmp93);
    std::tie(tmp94, tmp95, tmp96) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp68}).Flatten();
    tmp97 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp98 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp97});
    tmp99 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp98});
    tmp100 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp96});
    tmp101 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp99}, TNode<UintPtrT>{tmp100});
    ca_.Branch(tmp101, &block97, std::vector<compiler::Node*>{phi_bb88_13, phi_bb88_14, phi_bb88_16, phi_bb88_17, phi_bb88_20, phi_bb88_21, phi_bb88_26}, &block98, std::vector<compiler::Node*>{phi_bb88_13, phi_bb88_14, phi_bb88_16, phi_bb88_17, phi_bb88_20, phi_bb88_21, phi_bb88_26});
  }

  TNode<FixedArray> phi_bb89_13;
  TNode<IntPtrT> phi_bb89_14;
  TNode<BoolT> phi_bb89_16;
  TNode<PrimitiveHeapObject> phi_bb89_17;
  TNode<String> phi_bb89_20;
  TNode<String> phi_bb89_21;
  TNode<String> phi_bb89_26;
  if (block89.is_used()) {
    ca_.Bind(&block89, &phi_bb89_13, &phi_bb89_14, &phi_bb89_16, &phi_bb89_17, &phi_bb89_20, &phi_bb89_21, &phi_bb89_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb97_13;
  TNode<IntPtrT> phi_bb97_14;
  TNode<BoolT> phi_bb97_16;
  TNode<PrimitiveHeapObject> phi_bb97_17;
  TNode<String> phi_bb97_20;
  TNode<String> phi_bb97_21;
  TNode<String> phi_bb97_26;
  TNode<IntPtrT> tmp102;
  TNode<IntPtrT> tmp103;
  TNode<Union<HeapObject, TaggedIndex>> tmp104;
  TNode<IntPtrT> tmp105;
  TNode<IntPtrT> tmp106;
  if (block97.is_used()) {
    ca_.Bind(&block97, &phi_bb97_13, &phi_bb97_14, &phi_bb97_16, &phi_bb97_17, &phi_bb97_20, &phi_bb97_21, &phi_bb97_26);
    tmp102 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp98});
    tmp103 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp95}, TNode<IntPtrT>{tmp102});
    std::tie(tmp104, tmp105) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp94}, TNode<IntPtrT>{tmp103}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp104, tmp105}, tmp49);
    tmp106 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block57, tmp68, tmp106, phi_bb97_16, phi_bb97_17, phi_bb97_20, phi_bb97_21, phi_bb97_26);
  }

  TNode<FixedArray> phi_bb98_13;
  TNode<IntPtrT> phi_bb98_14;
  TNode<BoolT> phi_bb98_16;
  TNode<PrimitiveHeapObject> phi_bb98_17;
  TNode<String> phi_bb98_20;
  TNode<String> phi_bb98_21;
  TNode<String> phi_bb98_26;
  if (block98.is_used()) {
    ca_.Bind(&block98, &phi_bb98_13, &phi_bb98_14, &phi_bb98_16, &phi_bb98_17, &phi_bb98_20, &phi_bb98_21, &phi_bb98_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb57_13;
  TNode<IntPtrT> phi_bb57_14;
  TNode<BoolT> phi_bb57_16;
  TNode<PrimitiveHeapObject> phi_bb57_17;
  TNode<String> phi_bb57_20;
  TNode<String> phi_bb57_21;
  TNode<String> phi_bb57_26;
  TNode<Null> tmp107;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_13, &phi_bb57_14, &phi_bb57_16, &phi_bb57_17, &phi_bb57_20, &phi_bb57_21, &phi_bb57_26);
    tmp107 = Null_0(state_);
    ca_.Goto(&block45, phi_bb57_13, phi_bb57_14, phi_bb57_16, tmp107, phi_bb57_20, phi_bb57_21, phi_bb57_26);
  }

  TNode<FixedArray> phi_bb45_13;
  TNode<IntPtrT> phi_bb45_14;
  TNode<BoolT> phi_bb45_16;
  TNode<PrimitiveHeapObject> phi_bb45_17;
  TNode<String> phi_bb45_20;
  TNode<String> phi_bb45_21;
  TNode<String> phi_bb45_26;
  if (block45.is_used()) {
    ca_.Bind(&block45, &phi_bb45_13, &phi_bb45_14, &phi_bb45_16, &phi_bb45_17, &phi_bb45_20, &phi_bb45_21, &phi_bb45_26);
    ca_.Goto(&block36, phi_bb45_13, phi_bb45_14, tmp48, phi_bb45_16, phi_bb45_17, phi_bb45_20, phi_bb45_21, phi_bb45_26);
  }

  TNode<FixedArray> phi_bb36_13;
  TNode<IntPtrT> phi_bb36_14;
  TNode<IntPtrT> phi_bb36_15;
  TNode<BoolT> phi_bb36_16;
  TNode<PrimitiveHeapObject> phi_bb36_17;
  TNode<String> phi_bb36_20;
  TNode<String> phi_bb36_21;
  TNode<String> phi_bb36_26;
  TNode<IntPtrT> tmp108;
  TNode<IntPtrT> tmp109;
  TNode<BoolT> tmp110;
  if (block36.is_used()) {
    ca_.Bind(&block36, &phi_bb36_13, &phi_bb36_14, &phi_bb36_15, &phi_bb36_16, &phi_bb36_17, &phi_bb36_20, &phi_bb36_21, &phi_bb36_26);
    tmp108 = CodeStubAssembler(state_).LoadStringLengthAsWord(TNode<String>{phi_bb36_26});
    tmp109 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb36_15}, TNode<IntPtrT>{tmp108});
    tmp110 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<HeapObject, Smi, Weak<HeapObject>>>{phi_bb36_26}, TNode<Union<HeapObject, Smi, Weak<HeapObject>>>{phi_bb36_17});
    ca_.Branch(tmp110, &block101, std::vector<compiler::Node*>{phi_bb36_13, phi_bb36_14, phi_bb36_16, phi_bb36_17, phi_bb36_20, phi_bb36_21, phi_bb36_26}, &block102, std::vector<compiler::Node*>{phi_bb36_13, phi_bb36_14, phi_bb36_16, phi_bb36_17, phi_bb36_20, phi_bb36_21, phi_bb36_26});
  }

  TNode<FixedArray> phi_bb101_13;
  TNode<IntPtrT> phi_bb101_14;
  TNode<BoolT> phi_bb101_16;
  TNode<PrimitiveHeapObject> phi_bb101_17;
  TNode<String> phi_bb101_20;
  TNode<String> phi_bb101_21;
  TNode<String> phi_bb101_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp111;
  TNode<IntPtrT> tmp112;
  TNode<IntPtrT> tmp113;
  TNode<IntPtrT> tmp114;
  TNode<IntPtrT> tmp115;
  TNode<UintPtrT> tmp116;
  TNode<UintPtrT> tmp117;
  TNode<BoolT> tmp118;
  if (block101.is_used()) {
    ca_.Bind(&block101, &phi_bb101_13, &phi_bb101_14, &phi_bb101_16, &phi_bb101_17, &phi_bb101_20, &phi_bb101_21, &phi_bb101_26);
    std::tie(tmp111, tmp112, tmp113) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb101_13}).Flatten();
    tmp114 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp115 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb101_14}, TNode<IntPtrT>{tmp114});
    tmp116 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp115});
    tmp117 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp113});
    tmp118 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp116}, TNode<UintPtrT>{tmp117});
    ca_.Branch(tmp118, &block113, std::vector<compiler::Node*>{phi_bb101_13, phi_bb101_14, phi_bb101_16, phi_bb101_17, phi_bb101_20, phi_bb101_21, phi_bb101_26, phi_bb101_13}, &block114, std::vector<compiler::Node*>{phi_bb101_13, phi_bb101_14, phi_bb101_16, phi_bb101_17, phi_bb101_20, phi_bb101_21, phi_bb101_26, phi_bb101_13});
  }

  TNode<FixedArray> phi_bb113_13;
  TNode<IntPtrT> phi_bb113_14;
  TNode<BoolT> phi_bb113_16;
  TNode<PrimitiveHeapObject> phi_bb113_17;
  TNode<String> phi_bb113_20;
  TNode<String> phi_bb113_21;
  TNode<String> phi_bb113_26;
  TNode<FixedArray> phi_bb113_30;
  TNode<IntPtrT> tmp119;
  TNode<IntPtrT> tmp120;
  TNode<Union<HeapObject, TaggedIndex>> tmp121;
  TNode<IntPtrT> tmp122;
  TNode<Object> tmp123;
  TNode<HeapObject> tmp124;
  if (block113.is_used()) {
    ca_.Bind(&block113, &phi_bb113_13, &phi_bb113_14, &phi_bb113_16, &phi_bb113_17, &phi_bb113_20, &phi_bb113_21, &phi_bb113_26, &phi_bb113_30);
    tmp119 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp115});
    tmp120 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp112}, TNode<IntPtrT>{tmp119});
    std::tie(tmp121, tmp122) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp111}, TNode<IntPtrT>{tmp120}).Flatten();
    tmp123 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp121, tmp122});
    compiler::CodeAssemblerLabel label125(&ca_);
    tmp124 = CodeStubAssembler(state_).TaggedToHeapObject(TNode<Object>{tmp123}, &label125);
    ca_.Goto(&block120, phi_bb113_13, phi_bb113_14, phi_bb113_16, phi_bb113_17, phi_bb113_20, phi_bb113_21, phi_bb113_26);
    if (label125.is_used()) {
      ca_.Bind(&label125);
      ca_.Goto(&block121, phi_bb113_13, phi_bb113_14, phi_bb113_16, phi_bb113_17, phi_bb113_20, phi_bb113_21, phi_bb113_26);
    }
  }

  TNode<FixedArray> phi_bb114_13;
  TNode<IntPtrT> phi_bb114_14;
  TNode<BoolT> phi_bb114_16;
  TNode<PrimitiveHeapObject> phi_bb114_17;
  TNode<String> phi_bb114_20;
  TNode<String> phi_bb114_21;
  TNode<String> phi_bb114_26;
  TNode<FixedArray> phi_bb114_30;
  if (block114.is_used()) {
    ca_.Bind(&block114, &phi_bb114_13, &phi_bb114_14, &phi_bb114_16, &phi_bb114_17, &phi_bb114_20, &phi_bb114_21, &phi_bb114_26, &phi_bb114_30);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb121_13;
  TNode<IntPtrT> phi_bb121_14;
  TNode<BoolT> phi_bb121_16;
  TNode<PrimitiveHeapObject> phi_bb121_17;
  TNode<String> phi_bb121_20;
  TNode<String> phi_bb121_21;
  TNode<String> phi_bb121_26;
  if (block121.is_used()) {
    ca_.Bind(&block121, &phi_bb121_13, &phi_bb121_14, &phi_bb121_16, &phi_bb121_17, &phi_bb121_20, &phi_bb121_21, &phi_bb121_26);
    ca_.Goto(&block118, phi_bb121_13, phi_bb121_14, phi_bb121_16, phi_bb121_17, phi_bb121_20, phi_bb121_21, phi_bb121_26);
  }

  TNode<FixedArray> phi_bb120_13;
  TNode<IntPtrT> phi_bb120_14;
  TNode<BoolT> phi_bb120_16;
  TNode<PrimitiveHeapObject> phi_bb120_17;
  TNode<String> phi_bb120_20;
  TNode<String> phi_bb120_21;
  TNode<String> phi_bb120_26;
  TNode<String> tmp126;
  if (block120.is_used()) {
    ca_.Bind(&block120, &phi_bb120_13, &phi_bb120_14, &phi_bb120_16, &phi_bb120_17, &phi_bb120_20, &phi_bb120_21, &phi_bb120_26);
    compiler::CodeAssemblerLabel label127(&ca_);
    tmp126 = Cast_String_0(state_, TNode<HeapObject>{tmp124}, &label127);
    ca_.Goto(&block122, phi_bb120_13, phi_bb120_14, phi_bb120_16, phi_bb120_17, phi_bb120_20, phi_bb120_21, phi_bb120_26);
    if (label127.is_used()) {
      ca_.Bind(&label127);
      ca_.Goto(&block123, phi_bb120_13, phi_bb120_14, phi_bb120_16, phi_bb120_17, phi_bb120_20, phi_bb120_21, phi_bb120_26);
    }
  }

  TNode<FixedArray> phi_bb123_13;
  TNode<IntPtrT> phi_bb123_14;
  TNode<BoolT> phi_bb123_16;
  TNode<PrimitiveHeapObject> phi_bb123_17;
  TNode<String> phi_bb123_20;
  TNode<String> phi_bb123_21;
  TNode<String> phi_bb123_26;
  if (block123.is_used()) {
    ca_.Bind(&block123, &phi_bb123_13, &phi_bb123_14, &phi_bb123_16, &phi_bb123_17, &phi_bb123_20, &phi_bb123_21, &phi_bb123_26);
    ca_.Goto(&block118, phi_bb123_13, phi_bb123_14, phi_bb123_16, phi_bb123_17, phi_bb123_20, phi_bb123_21, phi_bb123_26);
  }

  TNode<FixedArray> phi_bb122_13;
  TNode<IntPtrT> phi_bb122_14;
  TNode<BoolT> phi_bb122_16;
  TNode<PrimitiveHeapObject> phi_bb122_17;
  TNode<String> phi_bb122_20;
  TNode<String> phi_bb122_21;
  TNode<String> phi_bb122_26;
  TNode<Smi> tmp128;
  TNode<IntPtrT> tmp129;
  TNode<BoolT> tmp130;
  if (block122.is_used()) {
    ca_.Bind(&block122, &phi_bb122_13, &phi_bb122_14, &phi_bb122_16, &phi_bb122_17, &phi_bb122_20, &phi_bb122_21, &phi_bb122_26);
    tmp128 = SmiConstant_0(state_, IntegerLiteral(true, 0x1ull));
    tmp129 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb122_13});
    tmp130 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb122_14}, TNode<IntPtrT>{tmp129});
    ca_.Branch(tmp130, &block133, std::vector<compiler::Node*>{phi_bb122_13, phi_bb122_14, phi_bb122_16, phi_bb122_17, phi_bb122_20, phi_bb122_21, phi_bb122_26}, &block134, std::vector<compiler::Node*>{phi_bb122_13, phi_bb122_14, phi_bb122_16, phi_bb122_17, phi_bb122_20, phi_bb122_21, phi_bb122_26});
  }

  TNode<FixedArray> phi_bb133_13;
  TNode<IntPtrT> phi_bb133_14;
  TNode<BoolT> phi_bb133_16;
  TNode<PrimitiveHeapObject> phi_bb133_17;
  TNode<String> phi_bb133_20;
  TNode<String> phi_bb133_21;
  TNode<String> phi_bb133_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp131;
  TNode<IntPtrT> tmp132;
  TNode<IntPtrT> tmp133;
  TNode<IntPtrT> tmp134;
  TNode<IntPtrT> tmp135;
  TNode<UintPtrT> tmp136;
  TNode<UintPtrT> tmp137;
  TNode<BoolT> tmp138;
  if (block133.is_used()) {
    ca_.Bind(&block133, &phi_bb133_13, &phi_bb133_14, &phi_bb133_16, &phi_bb133_17, &phi_bb133_20, &phi_bb133_21, &phi_bb133_26);
    std::tie(tmp131, tmp132, tmp133) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb133_13}).Flatten();
    tmp134 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp135 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb133_14}, TNode<IntPtrT>{tmp134});
    tmp136 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb133_14});
    tmp137 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp133});
    tmp138 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp136}, TNode<UintPtrT>{tmp137});
    ca_.Branch(tmp138, &block140, std::vector<compiler::Node*>{phi_bb133_13, phi_bb133_16, phi_bb133_17, phi_bb133_20, phi_bb133_21, phi_bb133_26, phi_bb133_13, phi_bb133_14, phi_bb133_14, phi_bb133_14, phi_bb133_14}, &block141, std::vector<compiler::Node*>{phi_bb133_13, phi_bb133_16, phi_bb133_17, phi_bb133_20, phi_bb133_21, phi_bb133_26, phi_bb133_13, phi_bb133_14, phi_bb133_14, phi_bb133_14, phi_bb133_14});
  }

  TNode<FixedArray> phi_bb140_13;
  TNode<BoolT> phi_bb140_16;
  TNode<PrimitiveHeapObject> phi_bb140_17;
  TNode<String> phi_bb140_20;
  TNode<String> phi_bb140_21;
  TNode<String> phi_bb140_26;
  TNode<FixedArray> phi_bb140_35;
  TNode<IntPtrT> phi_bb140_39;
  TNode<IntPtrT> phi_bb140_40;
  TNode<IntPtrT> phi_bb140_44;
  TNode<IntPtrT> phi_bb140_45;
  TNode<IntPtrT> tmp139;
  TNode<IntPtrT> tmp140;
  TNode<Union<HeapObject, TaggedIndex>> tmp141;
  TNode<IntPtrT> tmp142;
  if (block140.is_used()) {
    ca_.Bind(&block140, &phi_bb140_13, &phi_bb140_16, &phi_bb140_17, &phi_bb140_20, &phi_bb140_21, &phi_bb140_26, &phi_bb140_35, &phi_bb140_39, &phi_bb140_40, &phi_bb140_44, &phi_bb140_45);
    tmp139 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb140_45});
    tmp140 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp132}, TNode<IntPtrT>{tmp139});
    std::tie(tmp141, tmp142) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp131}, TNode<IntPtrT>{tmp140}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp141, tmp142}, tmp128);
    ca_.Goto(&block135, phi_bb140_13, tmp135, phi_bb140_16, phi_bb140_17, phi_bb140_20, phi_bb140_21, phi_bb140_26);
  }

  TNode<FixedArray> phi_bb141_13;
  TNode<BoolT> phi_bb141_16;
  TNode<PrimitiveHeapObject> phi_bb141_17;
  TNode<String> phi_bb141_20;
  TNode<String> phi_bb141_21;
  TNode<String> phi_bb141_26;
  TNode<FixedArray> phi_bb141_35;
  TNode<IntPtrT> phi_bb141_39;
  TNode<IntPtrT> phi_bb141_40;
  TNode<IntPtrT> phi_bb141_44;
  TNode<IntPtrT> phi_bb141_45;
  if (block141.is_used()) {
    ca_.Bind(&block141, &phi_bb141_13, &phi_bb141_16, &phi_bb141_17, &phi_bb141_20, &phi_bb141_21, &phi_bb141_26, &phi_bb141_35, &phi_bb141_39, &phi_bb141_40, &phi_bb141_44, &phi_bb141_45);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb134_13;
  TNode<IntPtrT> phi_bb134_14;
  TNode<BoolT> phi_bb134_16;
  TNode<PrimitiveHeapObject> phi_bb134_17;
  TNode<String> phi_bb134_20;
  TNode<String> phi_bb134_21;
  TNode<String> phi_bb134_26;
  TNode<IntPtrT> tmp143;
  TNode<IntPtrT> tmp144;
  TNode<BoolT> tmp145;
  if (block134.is_used()) {
    ca_.Bind(&block134, &phi_bb134_13, &phi_bb134_14, &phi_bb134_16, &phi_bb134_17, &phi_bb134_20, &phi_bb134_21, &phi_bb134_26);
    tmp143 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp129});
    tmp144 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp145 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp143}, TNode<IntPtrT>{tmp144});
    ca_.Branch(tmp145, &block144, std::vector<compiler::Node*>{phi_bb134_13, phi_bb134_14, phi_bb134_16, phi_bb134_17, phi_bb134_20, phi_bb134_21, phi_bb134_26}, &block145, std::vector<compiler::Node*>{phi_bb134_13, phi_bb134_14, phi_bb134_16, phi_bb134_17, phi_bb134_20, phi_bb134_21, phi_bb134_26});
  }

  TNode<FixedArray> phi_bb144_13;
  TNode<IntPtrT> phi_bb144_14;
  TNode<BoolT> phi_bb144_16;
  TNode<PrimitiveHeapObject> phi_bb144_17;
  TNode<String> phi_bb144_20;
  TNode<String> phi_bb144_21;
  TNode<String> phi_bb144_26;
  TNode<IntPtrT> tmp146;
  if (block144.is_used()) {
    ca_.Bind(&block144, &phi_bb144_13, &phi_bb144_14, &phi_bb144_16, &phi_bb144_17, &phi_bb144_20, &phi_bb144_21, &phi_bb144_26);
    tmp146 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block146, phi_bb144_13, phi_bb144_14, phi_bb144_16, phi_bb144_17, phi_bb144_20, phi_bb144_21, phi_bb144_26, tmp146);
  }

  TNode<FixedArray> phi_bb145_13;
  TNode<IntPtrT> phi_bb145_14;
  TNode<BoolT> phi_bb145_16;
  TNode<PrimitiveHeapObject> phi_bb145_17;
  TNode<String> phi_bb145_20;
  TNode<String> phi_bb145_21;
  TNode<String> phi_bb145_26;
  if (block145.is_used()) {
    ca_.Bind(&block145, &phi_bb145_13, &phi_bb145_14, &phi_bb145_16, &phi_bb145_17, &phi_bb145_20, &phi_bb145_21, &phi_bb145_26);
    ca_.Goto(&block146, phi_bb145_13, phi_bb145_14, phi_bb145_16, phi_bb145_17, phi_bb145_20, phi_bb145_21, phi_bb145_26, tmp143);
  }

  TNode<FixedArray> phi_bb146_13;
  TNode<IntPtrT> phi_bb146_14;
  TNode<BoolT> phi_bb146_16;
  TNode<PrimitiveHeapObject> phi_bb146_17;
  TNode<String> phi_bb146_20;
  TNode<String> phi_bb146_21;
  TNode<String> phi_bb146_26;
  TNode<IntPtrT> phi_bb146_36;
  TNode<FixedArray> tmp147;
  TNode<Union<HeapObject, TaggedIndex>> tmp148;
  TNode<IntPtrT> tmp149;
  TNode<IntPtrT> tmp150;
  TNode<UintPtrT> tmp151;
  TNode<IntPtrT> tmp152;
  TNode<UintPtrT> tmp153;
  TNode<UintPtrT> tmp154;
  TNode<BoolT> tmp155;
  if (block146.is_used()) {
    ca_.Bind(&block146, &phi_bb146_13, &phi_bb146_14, &phi_bb146_16, &phi_bb146_17, &phi_bb146_20, &phi_bb146_21, &phi_bb146_26, &phi_bb146_36);
    tmp147 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb146_36});
    std::tie(tmp148, tmp149, tmp150) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb146_13}).Flatten();
    tmp151 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp152 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp151});
    tmp153 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp152});
    tmp154 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp150});
    tmp155 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp153}, TNode<UintPtrT>{tmp154});
    ca_.Branch(tmp155, &block157, std::vector<compiler::Node*>{phi_bb146_13, phi_bb146_14, phi_bb146_16, phi_bb146_17, phi_bb146_20, phi_bb146_21, phi_bb146_26, phi_bb146_13}, &block158, std::vector<compiler::Node*>{phi_bb146_13, phi_bb146_14, phi_bb146_16, phi_bb146_17, phi_bb146_20, phi_bb146_21, phi_bb146_26, phi_bb146_13});
  }

  TNode<FixedArray> phi_bb157_13;
  TNode<IntPtrT> phi_bb157_14;
  TNode<BoolT> phi_bb157_16;
  TNode<PrimitiveHeapObject> phi_bb157_17;
  TNode<String> phi_bb157_20;
  TNode<String> phi_bb157_21;
  TNode<String> phi_bb157_26;
  TNode<FixedArray> phi_bb157_38;
  TNode<IntPtrT> tmp156;
  TNode<IntPtrT> tmp157;
  TNode<Union<HeapObject, TaggedIndex>> tmp158;
  TNode<IntPtrT> tmp159;
  TNode<Union<HeapObject, TaggedIndex>> tmp160;
  TNode<IntPtrT> tmp161;
  TNode<IntPtrT> tmp162;
  TNode<UintPtrT> tmp163;
  TNode<IntPtrT> tmp164;
  TNode<UintPtrT> tmp165;
  TNode<UintPtrT> tmp166;
  TNode<BoolT> tmp167;
  if (block157.is_used()) {
    ca_.Bind(&block157, &phi_bb157_13, &phi_bb157_14, &phi_bb157_16, &phi_bb157_17, &phi_bb157_20, &phi_bb157_21, &phi_bb157_26, &phi_bb157_38);
    tmp156 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp152});
    tmp157 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp149}, TNode<IntPtrT>{tmp156});
    std::tie(tmp158, tmp159) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp148}, TNode<IntPtrT>{tmp157}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp158, tmp159}, tmp147);
    std::tie(tmp160, tmp161, tmp162) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp147}).Flatten();
    tmp163 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp164 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp163});
    tmp165 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp164});
    tmp166 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp162});
    tmp167 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp165}, TNode<UintPtrT>{tmp166});
    ca_.Branch(tmp167, &block166, std::vector<compiler::Node*>{phi_bb157_13, phi_bb157_14, phi_bb157_16, phi_bb157_17, phi_bb157_20, phi_bb157_21, phi_bb157_26}, &block167, std::vector<compiler::Node*>{phi_bb157_13, phi_bb157_14, phi_bb157_16, phi_bb157_17, phi_bb157_20, phi_bb157_21, phi_bb157_26});
  }

  TNode<FixedArray> phi_bb158_13;
  TNode<IntPtrT> phi_bb158_14;
  TNode<BoolT> phi_bb158_16;
  TNode<PrimitiveHeapObject> phi_bb158_17;
  TNode<String> phi_bb158_20;
  TNode<String> phi_bb158_21;
  TNode<String> phi_bb158_26;
  TNode<FixedArray> phi_bb158_38;
  if (block158.is_used()) {
    ca_.Bind(&block158, &phi_bb158_13, &phi_bb158_14, &phi_bb158_16, &phi_bb158_17, &phi_bb158_20, &phi_bb158_21, &phi_bb158_26, &phi_bb158_38);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb166_13;
  TNode<IntPtrT> phi_bb166_14;
  TNode<BoolT> phi_bb166_16;
  TNode<PrimitiveHeapObject> phi_bb166_17;
  TNode<String> phi_bb166_20;
  TNode<String> phi_bb166_21;
  TNode<String> phi_bb166_26;
  TNode<IntPtrT> tmp168;
  TNode<IntPtrT> tmp169;
  TNode<Union<HeapObject, TaggedIndex>> tmp170;
  TNode<IntPtrT> tmp171;
  TNode<Undefined> tmp172;
  TNode<Union<HeapObject, TaggedIndex>> tmp173;
  TNode<IntPtrT> tmp174;
  TNode<IntPtrT> tmp175;
  TNode<UintPtrT> tmp176;
  TNode<IntPtrT> tmp177;
  TNode<UintPtrT> tmp178;
  TNode<UintPtrT> tmp179;
  TNode<BoolT> tmp180;
  if (block166.is_used()) {
    ca_.Bind(&block166, &phi_bb166_13, &phi_bb166_14, &phi_bb166_16, &phi_bb166_17, &phi_bb166_20, &phi_bb166_21, &phi_bb166_26);
    tmp168 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp164});
    tmp169 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp161}, TNode<IntPtrT>{tmp168});
    std::tie(tmp170, tmp171) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp160}, TNode<IntPtrT>{tmp169}).Flatten();
    tmp172 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp170, tmp171}, tmp172);
    std::tie(tmp173, tmp174, tmp175) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp147}).Flatten();
    tmp176 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp177 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp176});
    tmp178 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp177});
    tmp179 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp175});
    tmp180 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp178}, TNode<UintPtrT>{tmp179});
    ca_.Branch(tmp180, &block175, std::vector<compiler::Node*>{phi_bb166_13, phi_bb166_14, phi_bb166_16, phi_bb166_17, phi_bb166_20, phi_bb166_21, phi_bb166_26}, &block176, std::vector<compiler::Node*>{phi_bb166_13, phi_bb166_14, phi_bb166_16, phi_bb166_17, phi_bb166_20, phi_bb166_21, phi_bb166_26});
  }

  TNode<FixedArray> phi_bb167_13;
  TNode<IntPtrT> phi_bb167_14;
  TNode<BoolT> phi_bb167_16;
  TNode<PrimitiveHeapObject> phi_bb167_17;
  TNode<String> phi_bb167_20;
  TNode<String> phi_bb167_21;
  TNode<String> phi_bb167_26;
  if (block167.is_used()) {
    ca_.Bind(&block167, &phi_bb167_13, &phi_bb167_14, &phi_bb167_16, &phi_bb167_17, &phi_bb167_20, &phi_bb167_21, &phi_bb167_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb175_13;
  TNode<IntPtrT> phi_bb175_14;
  TNode<BoolT> phi_bb175_16;
  TNode<PrimitiveHeapObject> phi_bb175_17;
  TNode<String> phi_bb175_20;
  TNode<String> phi_bb175_21;
  TNode<String> phi_bb175_26;
  TNode<IntPtrT> tmp181;
  TNode<IntPtrT> tmp182;
  TNode<Union<HeapObject, TaggedIndex>> tmp183;
  TNode<IntPtrT> tmp184;
  TNode<IntPtrT> tmp185;
  if (block175.is_used()) {
    ca_.Bind(&block175, &phi_bb175_13, &phi_bb175_14, &phi_bb175_16, &phi_bb175_17, &phi_bb175_20, &phi_bb175_21, &phi_bb175_26);
    tmp181 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp177});
    tmp182 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp174}, TNode<IntPtrT>{tmp181});
    std::tie(tmp183, tmp184) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp173}, TNode<IntPtrT>{tmp182}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp183, tmp184}, tmp128);
    tmp185 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block135, tmp147, tmp185, phi_bb175_16, phi_bb175_17, phi_bb175_20, phi_bb175_21, phi_bb175_26);
  }

  TNode<FixedArray> phi_bb176_13;
  TNode<IntPtrT> phi_bb176_14;
  TNode<BoolT> phi_bb176_16;
  TNode<PrimitiveHeapObject> phi_bb176_17;
  TNode<String> phi_bb176_20;
  TNode<String> phi_bb176_21;
  TNode<String> phi_bb176_26;
  if (block176.is_used()) {
    ca_.Bind(&block176, &phi_bb176_13, &phi_bb176_14, &phi_bb176_16, &phi_bb176_17, &phi_bb176_20, &phi_bb176_21, &phi_bb176_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb135_13;
  TNode<IntPtrT> phi_bb135_14;
  TNode<BoolT> phi_bb135_16;
  TNode<PrimitiveHeapObject> phi_bb135_17;
  TNode<String> phi_bb135_20;
  TNode<String> phi_bb135_21;
  TNode<String> phi_bb135_26;
  if (block135.is_used()) {
    ca_.Bind(&block135, &phi_bb135_13, &phi_bb135_14, &phi_bb135_16, &phi_bb135_17, &phi_bb135_20, &phi_bb135_21, &phi_bb135_26);
    ca_.Goto(&block117, phi_bb135_13, phi_bb135_14, phi_bb135_16, phi_bb135_17, phi_bb135_20, phi_bb135_21, phi_bb135_26);
  }

  TNode<FixedArray> phi_bb118_13;
  TNode<IntPtrT> phi_bb118_14;
  TNode<BoolT> phi_bb118_16;
  TNode<PrimitiveHeapObject> phi_bb118_17;
  TNode<String> phi_bb118_20;
  TNode<String> phi_bb118_21;
  TNode<String> phi_bb118_26;
  TNode<Smi> tmp186;
  if (block118.is_used()) {
    ca_.Bind(&block118, &phi_bb118_13, &phi_bb118_14, &phi_bb118_16, &phi_bb118_17, &phi_bb118_20, &phi_bb118_21, &phi_bb118_26);
    compiler::CodeAssemblerLabel label187(&ca_);
    tmp186 = Cast_Smi_0(state_, TNode<Object>{ca_.UncheckedCast<Object>(tmp123)}, &label187);
    ca_.Goto(&block181, phi_bb118_13, phi_bb118_14, phi_bb118_16, phi_bb118_17, phi_bb118_20, phi_bb118_21, phi_bb118_26);
    if (label187.is_used()) {
      ca_.Bind(&label187);
      ca_.Goto(&block182, phi_bb118_13, phi_bb118_14, phi_bb118_16, phi_bb118_17, phi_bb118_20, phi_bb118_21, phi_bb118_26);
    }
  }

  TNode<FixedArray> phi_bb182_13;
  TNode<IntPtrT> phi_bb182_14;
  TNode<BoolT> phi_bb182_16;
  TNode<PrimitiveHeapObject> phi_bb182_17;
  TNode<String> phi_bb182_20;
  TNode<String> phi_bb182_21;
  TNode<String> phi_bb182_26;
  if (block182.is_used()) {
    ca_.Bind(&block182, &phi_bb182_13, &phi_bb182_14, &phi_bb182_16, &phi_bb182_17, &phi_bb182_20, &phi_bb182_21, &phi_bb182_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb181_13;
  TNode<IntPtrT> phi_bb181_14;
  TNode<BoolT> phi_bb181_16;
  TNode<PrimitiveHeapObject> phi_bb181_17;
  TNode<String> phi_bb181_20;
  TNode<String> phi_bb181_21;
  TNode<String> phi_bb181_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp188;
  TNode<IntPtrT> tmp189;
  TNode<IntPtrT> tmp190;
  TNode<IntPtrT> tmp191;
  TNode<IntPtrT> tmp192;
  TNode<UintPtrT> tmp193;
  TNode<UintPtrT> tmp194;
  TNode<BoolT> tmp195;
  if (block181.is_used()) {
    ca_.Bind(&block181, &phi_bb181_13, &phi_bb181_14, &phi_bb181_16, &phi_bb181_17, &phi_bb181_20, &phi_bb181_21, &phi_bb181_26);
    std::tie(tmp188, tmp189, tmp190) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb181_13}).Flatten();
    tmp191 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp192 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb181_14}, TNode<IntPtrT>{tmp191});
    tmp193 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp192});
    tmp194 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp190});
    tmp195 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp193}, TNode<UintPtrT>{tmp194});
    ca_.Branch(tmp195, &block195, std::vector<compiler::Node*>{phi_bb181_13, phi_bb181_14, phi_bb181_16, phi_bb181_17, phi_bb181_20, phi_bb181_21, phi_bb181_26, phi_bb181_13}, &block196, std::vector<compiler::Node*>{phi_bb181_13, phi_bb181_14, phi_bb181_16, phi_bb181_17, phi_bb181_20, phi_bb181_21, phi_bb181_26, phi_bb181_13});
  }

  TNode<FixedArray> phi_bb195_13;
  TNode<IntPtrT> phi_bb195_14;
  TNode<BoolT> phi_bb195_16;
  TNode<PrimitiveHeapObject> phi_bb195_17;
  TNode<String> phi_bb195_20;
  TNode<String> phi_bb195_21;
  TNode<String> phi_bb195_26;
  TNode<FixedArray> phi_bb195_32;
  TNode<IntPtrT> tmp196;
  TNode<IntPtrT> tmp197;
  TNode<Union<HeapObject, TaggedIndex>> tmp198;
  TNode<IntPtrT> tmp199;
  TNode<Smi> tmp200;
  TNode<Smi> tmp201;
  if (block195.is_used()) {
    ca_.Bind(&block195, &phi_bb195_13, &phi_bb195_14, &phi_bb195_16, &phi_bb195_17, &phi_bb195_20, &phi_bb195_21, &phi_bb195_26, &phi_bb195_32);
    tmp196 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp192});
    tmp197 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp189}, TNode<IntPtrT>{tmp196});
    std::tie(tmp198, tmp199) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp188}, TNode<IntPtrT>{tmp197}).Flatten();
    tmp200 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp201 = CodeStubAssembler(state_).SmiSub(TNode<Smi>{tmp186}, TNode<Smi>{tmp200});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp198, tmp199}, tmp201);
    ca_.Goto(&block117, phi_bb195_13, phi_bb195_14, phi_bb195_16, phi_bb195_17, phi_bb195_20, phi_bb195_21, phi_bb195_26);
  }

  TNode<FixedArray> phi_bb196_13;
  TNode<IntPtrT> phi_bb196_14;
  TNode<BoolT> phi_bb196_16;
  TNode<PrimitiveHeapObject> phi_bb196_17;
  TNode<String> phi_bb196_20;
  TNode<String> phi_bb196_21;
  TNode<String> phi_bb196_26;
  TNode<FixedArray> phi_bb196_32;
  if (block196.is_used()) {
    ca_.Bind(&block196, &phi_bb196_13, &phi_bb196_14, &phi_bb196_16, &phi_bb196_17, &phi_bb196_20, &phi_bb196_21, &phi_bb196_26, &phi_bb196_32);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb117_13;
  TNode<IntPtrT> phi_bb117_14;
  TNode<BoolT> phi_bb117_16;
  TNode<PrimitiveHeapObject> phi_bb117_17;
  TNode<String> phi_bb117_20;
  TNode<String> phi_bb117_21;
  TNode<String> phi_bb117_26;
  if (block117.is_used()) {
    ca_.Bind(&block117, &phi_bb117_13, &phi_bb117_14, &phi_bb117_16, &phi_bb117_17, &phi_bb117_20, &phi_bb117_21, &phi_bb117_26);
    ca_.Goto(&block103, phi_bb117_13, phi_bb117_14, phi_bb117_16, phi_bb117_17, phi_bb117_20, phi_bb117_21, phi_bb117_26);
  }

  TNode<FixedArray> phi_bb102_13;
  TNode<IntPtrT> phi_bb102_14;
  TNode<BoolT> phi_bb102_16;
  TNode<PrimitiveHeapObject> phi_bb102_17;
  TNode<String> phi_bb102_20;
  TNode<String> phi_bb102_21;
  TNode<String> phi_bb102_26;
  TNode<IntPtrT> tmp202;
  TNode<BoolT> tmp203;
  if (block102.is_used()) {
    ca_.Bind(&block102, &phi_bb102_13, &phi_bb102_14, &phi_bb102_16, &phi_bb102_17, &phi_bb102_20, &phi_bb102_21, &phi_bb102_26);
    tmp202 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb102_13});
    tmp203 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb102_14}, TNode<IntPtrT>{tmp202});
    ca_.Branch(tmp203, &block208, std::vector<compiler::Node*>{phi_bb102_13, phi_bb102_14, phi_bb102_16, phi_bb102_17, phi_bb102_20, phi_bb102_21, phi_bb102_26, phi_bb102_26, phi_bb102_26}, &block209, std::vector<compiler::Node*>{phi_bb102_13, phi_bb102_14, phi_bb102_16, phi_bb102_17, phi_bb102_20, phi_bb102_21, phi_bb102_26, phi_bb102_26, phi_bb102_26});
  }

  TNode<FixedArray> phi_bb208_13;
  TNode<IntPtrT> phi_bb208_14;
  TNode<BoolT> phi_bb208_16;
  TNode<PrimitiveHeapObject> phi_bb208_17;
  TNode<String> phi_bb208_20;
  TNode<String> phi_bb208_21;
  TNode<String> phi_bb208_26;
  TNode<String> phi_bb208_30;
  TNode<Object> phi_bb208_31;
  TNode<Union<HeapObject, TaggedIndex>> tmp204;
  TNode<IntPtrT> tmp205;
  TNode<IntPtrT> tmp206;
  TNode<IntPtrT> tmp207;
  TNode<IntPtrT> tmp208;
  TNode<UintPtrT> tmp209;
  TNode<UintPtrT> tmp210;
  TNode<BoolT> tmp211;
  if (block208.is_used()) {
    ca_.Bind(&block208, &phi_bb208_13, &phi_bb208_14, &phi_bb208_16, &phi_bb208_17, &phi_bb208_20, &phi_bb208_21, &phi_bb208_26, &phi_bb208_30, &phi_bb208_31);
    std::tie(tmp204, tmp205, tmp206) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb208_13}).Flatten();
    tmp207 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp208 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb208_14}, TNode<IntPtrT>{tmp207});
    tmp209 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb208_14});
    tmp210 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp206});
    tmp211 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp209}, TNode<UintPtrT>{tmp210});
    ca_.Branch(tmp211, &block215, std::vector<compiler::Node*>{phi_bb208_13, phi_bb208_16, phi_bb208_17, phi_bb208_20, phi_bb208_21, phi_bb208_26, phi_bb208_30, phi_bb208_31, phi_bb208_13, phi_bb208_14, phi_bb208_14, phi_bb208_14, phi_bb208_14}, &block216, std::vector<compiler::Node*>{phi_bb208_13, phi_bb208_16, phi_bb208_17, phi_bb208_20, phi_bb208_21, phi_bb208_26, phi_bb208_30, phi_bb208_31, phi_bb208_13, phi_bb208_14, phi_bb208_14, phi_bb208_14, phi_bb208_14});
  }

  TNode<FixedArray> phi_bb215_13;
  TNode<BoolT> phi_bb215_16;
  TNode<PrimitiveHeapObject> phi_bb215_17;
  TNode<String> phi_bb215_20;
  TNode<String> phi_bb215_21;
  TNode<String> phi_bb215_26;
  TNode<String> phi_bb215_30;
  TNode<Object> phi_bb215_31;
  TNode<FixedArray> phi_bb215_33;
  TNode<IntPtrT> phi_bb215_37;
  TNode<IntPtrT> phi_bb215_38;
  TNode<IntPtrT> phi_bb215_42;
  TNode<IntPtrT> phi_bb215_43;
  TNode<IntPtrT> tmp212;
  TNode<IntPtrT> tmp213;
  TNode<Union<HeapObject, TaggedIndex>> tmp214;
  TNode<IntPtrT> tmp215;
  if (block215.is_used()) {
    ca_.Bind(&block215, &phi_bb215_13, &phi_bb215_16, &phi_bb215_17, &phi_bb215_20, &phi_bb215_21, &phi_bb215_26, &phi_bb215_30, &phi_bb215_31, &phi_bb215_33, &phi_bb215_37, &phi_bb215_38, &phi_bb215_42, &phi_bb215_43);
    tmp212 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb215_43});
    tmp213 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp205}, TNode<IntPtrT>{tmp212});
    std::tie(tmp214, tmp215) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp204}, TNode<IntPtrT>{tmp213}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp214, tmp215}, phi_bb215_31);
    ca_.Goto(&block210, phi_bb215_13, tmp208, phi_bb215_16, phi_bb215_17, phi_bb215_20, phi_bb215_21, phi_bb215_26, phi_bb215_30, phi_bb215_31);
  }

  TNode<FixedArray> phi_bb216_13;
  TNode<BoolT> phi_bb216_16;
  TNode<PrimitiveHeapObject> phi_bb216_17;
  TNode<String> phi_bb216_20;
  TNode<String> phi_bb216_21;
  TNode<String> phi_bb216_26;
  TNode<String> phi_bb216_30;
  TNode<Object> phi_bb216_31;
  TNode<FixedArray> phi_bb216_33;
  TNode<IntPtrT> phi_bb216_37;
  TNode<IntPtrT> phi_bb216_38;
  TNode<IntPtrT> phi_bb216_42;
  TNode<IntPtrT> phi_bb216_43;
  if (block216.is_used()) {
    ca_.Bind(&block216, &phi_bb216_13, &phi_bb216_16, &phi_bb216_17, &phi_bb216_20, &phi_bb216_21, &phi_bb216_26, &phi_bb216_30, &phi_bb216_31, &phi_bb216_33, &phi_bb216_37, &phi_bb216_38, &phi_bb216_42, &phi_bb216_43);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb209_13;
  TNode<IntPtrT> phi_bb209_14;
  TNode<BoolT> phi_bb209_16;
  TNode<PrimitiveHeapObject> phi_bb209_17;
  TNode<String> phi_bb209_20;
  TNode<String> phi_bb209_21;
  TNode<String> phi_bb209_26;
  TNode<String> phi_bb209_30;
  TNode<Object> phi_bb209_31;
  TNode<IntPtrT> tmp216;
  TNode<IntPtrT> tmp217;
  TNode<BoolT> tmp218;
  if (block209.is_used()) {
    ca_.Bind(&block209, &phi_bb209_13, &phi_bb209_14, &phi_bb209_16, &phi_bb209_17, &phi_bb209_20, &phi_bb209_21, &phi_bb209_26, &phi_bb209_30, &phi_bb209_31);
    tmp216 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp202});
    tmp217 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp218 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp216}, TNode<IntPtrT>{tmp217});
    ca_.Branch(tmp218, &block219, std::vector<compiler::Node*>{phi_bb209_13, phi_bb209_14, phi_bb209_16, phi_bb209_17, phi_bb209_20, phi_bb209_21, phi_bb209_26, phi_bb209_30, phi_bb209_31}, &block220, std::vector<compiler::Node*>{phi_bb209_13, phi_bb209_14, phi_bb209_16, phi_bb209_17, phi_bb209_20, phi_bb209_21, phi_bb209_26, phi_bb209_30, phi_bb209_31});
  }

  TNode<FixedArray> phi_bb219_13;
  TNode<IntPtrT> phi_bb219_14;
  TNode<BoolT> phi_bb219_16;
  TNode<PrimitiveHeapObject> phi_bb219_17;
  TNode<String> phi_bb219_20;
  TNode<String> phi_bb219_21;
  TNode<String> phi_bb219_26;
  TNode<String> phi_bb219_30;
  TNode<Object> phi_bb219_31;
  TNode<IntPtrT> tmp219;
  if (block219.is_used()) {
    ca_.Bind(&block219, &phi_bb219_13, &phi_bb219_14, &phi_bb219_16, &phi_bb219_17, &phi_bb219_20, &phi_bb219_21, &phi_bb219_26, &phi_bb219_30, &phi_bb219_31);
    tmp219 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block221, phi_bb219_13, phi_bb219_14, phi_bb219_16, phi_bb219_17, phi_bb219_20, phi_bb219_21, phi_bb219_26, phi_bb219_30, phi_bb219_31, tmp219);
  }

  TNode<FixedArray> phi_bb220_13;
  TNode<IntPtrT> phi_bb220_14;
  TNode<BoolT> phi_bb220_16;
  TNode<PrimitiveHeapObject> phi_bb220_17;
  TNode<String> phi_bb220_20;
  TNode<String> phi_bb220_21;
  TNode<String> phi_bb220_26;
  TNode<String> phi_bb220_30;
  TNode<Object> phi_bb220_31;
  if (block220.is_used()) {
    ca_.Bind(&block220, &phi_bb220_13, &phi_bb220_14, &phi_bb220_16, &phi_bb220_17, &phi_bb220_20, &phi_bb220_21, &phi_bb220_26, &phi_bb220_30, &phi_bb220_31);
    ca_.Goto(&block221, phi_bb220_13, phi_bb220_14, phi_bb220_16, phi_bb220_17, phi_bb220_20, phi_bb220_21, phi_bb220_26, phi_bb220_30, phi_bb220_31, tmp216);
  }

  TNode<FixedArray> phi_bb221_13;
  TNode<IntPtrT> phi_bb221_14;
  TNode<BoolT> phi_bb221_16;
  TNode<PrimitiveHeapObject> phi_bb221_17;
  TNode<String> phi_bb221_20;
  TNode<String> phi_bb221_21;
  TNode<String> phi_bb221_26;
  TNode<String> phi_bb221_30;
  TNode<Object> phi_bb221_31;
  TNode<IntPtrT> phi_bb221_34;
  TNode<FixedArray> tmp220;
  TNode<Union<HeapObject, TaggedIndex>> tmp221;
  TNode<IntPtrT> tmp222;
  TNode<IntPtrT> tmp223;
  TNode<UintPtrT> tmp224;
  TNode<IntPtrT> tmp225;
  TNode<UintPtrT> tmp226;
  TNode<UintPtrT> tmp227;
  TNode<BoolT> tmp228;
  if (block221.is_used()) {
    ca_.Bind(&block221, &phi_bb221_13, &phi_bb221_14, &phi_bb221_16, &phi_bb221_17, &phi_bb221_20, &phi_bb221_21, &phi_bb221_26, &phi_bb221_30, &phi_bb221_31, &phi_bb221_34);
    tmp220 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb221_34});
    std::tie(tmp221, tmp222, tmp223) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb221_13}).Flatten();
    tmp224 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp225 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp224});
    tmp226 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp225});
    tmp227 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp223});
    tmp228 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp226}, TNode<UintPtrT>{tmp227});
    ca_.Branch(tmp228, &block232, std::vector<compiler::Node*>{phi_bb221_13, phi_bb221_14, phi_bb221_16, phi_bb221_17, phi_bb221_20, phi_bb221_21, phi_bb221_26, phi_bb221_30, phi_bb221_31, phi_bb221_13}, &block233, std::vector<compiler::Node*>{phi_bb221_13, phi_bb221_14, phi_bb221_16, phi_bb221_17, phi_bb221_20, phi_bb221_21, phi_bb221_26, phi_bb221_30, phi_bb221_31, phi_bb221_13});
  }

  TNode<FixedArray> phi_bb232_13;
  TNode<IntPtrT> phi_bb232_14;
  TNode<BoolT> phi_bb232_16;
  TNode<PrimitiveHeapObject> phi_bb232_17;
  TNode<String> phi_bb232_20;
  TNode<String> phi_bb232_21;
  TNode<String> phi_bb232_26;
  TNode<String> phi_bb232_30;
  TNode<Object> phi_bb232_31;
  TNode<FixedArray> phi_bb232_36;
  TNode<IntPtrT> tmp229;
  TNode<IntPtrT> tmp230;
  TNode<Union<HeapObject, TaggedIndex>> tmp231;
  TNode<IntPtrT> tmp232;
  TNode<Union<HeapObject, TaggedIndex>> tmp233;
  TNode<IntPtrT> tmp234;
  TNode<IntPtrT> tmp235;
  TNode<UintPtrT> tmp236;
  TNode<IntPtrT> tmp237;
  TNode<UintPtrT> tmp238;
  TNode<UintPtrT> tmp239;
  TNode<BoolT> tmp240;
  if (block232.is_used()) {
    ca_.Bind(&block232, &phi_bb232_13, &phi_bb232_14, &phi_bb232_16, &phi_bb232_17, &phi_bb232_20, &phi_bb232_21, &phi_bb232_26, &phi_bb232_30, &phi_bb232_31, &phi_bb232_36);
    tmp229 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp225});
    tmp230 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp222}, TNode<IntPtrT>{tmp229});
    std::tie(tmp231, tmp232) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp221}, TNode<IntPtrT>{tmp230}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp231, tmp232}, tmp220);
    std::tie(tmp233, tmp234, tmp235) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp220}).Flatten();
    tmp236 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp237 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp236});
    tmp238 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp237});
    tmp239 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp235});
    tmp240 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp238}, TNode<UintPtrT>{tmp239});
    ca_.Branch(tmp240, &block241, std::vector<compiler::Node*>{phi_bb232_13, phi_bb232_14, phi_bb232_16, phi_bb232_17, phi_bb232_20, phi_bb232_21, phi_bb232_26, phi_bb232_30, phi_bb232_31}, &block242, std::vector<compiler::Node*>{phi_bb232_13, phi_bb232_14, phi_bb232_16, phi_bb232_17, phi_bb232_20, phi_bb232_21, phi_bb232_26, phi_bb232_30, phi_bb232_31});
  }

  TNode<FixedArray> phi_bb233_13;
  TNode<IntPtrT> phi_bb233_14;
  TNode<BoolT> phi_bb233_16;
  TNode<PrimitiveHeapObject> phi_bb233_17;
  TNode<String> phi_bb233_20;
  TNode<String> phi_bb233_21;
  TNode<String> phi_bb233_26;
  TNode<String> phi_bb233_30;
  TNode<Object> phi_bb233_31;
  TNode<FixedArray> phi_bb233_36;
  if (block233.is_used()) {
    ca_.Bind(&block233, &phi_bb233_13, &phi_bb233_14, &phi_bb233_16, &phi_bb233_17, &phi_bb233_20, &phi_bb233_21, &phi_bb233_26, &phi_bb233_30, &phi_bb233_31, &phi_bb233_36);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb241_13;
  TNode<IntPtrT> phi_bb241_14;
  TNode<BoolT> phi_bb241_16;
  TNode<PrimitiveHeapObject> phi_bb241_17;
  TNode<String> phi_bb241_20;
  TNode<String> phi_bb241_21;
  TNode<String> phi_bb241_26;
  TNode<String> phi_bb241_30;
  TNode<Object> phi_bb241_31;
  TNode<IntPtrT> tmp241;
  TNode<IntPtrT> tmp242;
  TNode<Union<HeapObject, TaggedIndex>> tmp243;
  TNode<IntPtrT> tmp244;
  TNode<Undefined> tmp245;
  TNode<Union<HeapObject, TaggedIndex>> tmp246;
  TNode<IntPtrT> tmp247;
  TNode<IntPtrT> tmp248;
  TNode<UintPtrT> tmp249;
  TNode<IntPtrT> tmp250;
  TNode<UintPtrT> tmp251;
  TNode<UintPtrT> tmp252;
  TNode<BoolT> tmp253;
  if (block241.is_used()) {
    ca_.Bind(&block241, &phi_bb241_13, &phi_bb241_14, &phi_bb241_16, &phi_bb241_17, &phi_bb241_20, &phi_bb241_21, &phi_bb241_26, &phi_bb241_30, &phi_bb241_31);
    tmp241 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp237});
    tmp242 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp234}, TNode<IntPtrT>{tmp241});
    std::tie(tmp243, tmp244) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp233}, TNode<IntPtrT>{tmp242}).Flatten();
    tmp245 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp243, tmp244}, tmp245);
    std::tie(tmp246, tmp247, tmp248) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp220}).Flatten();
    tmp249 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp250 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp249});
    tmp251 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp250});
    tmp252 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp248});
    tmp253 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp251}, TNode<UintPtrT>{tmp252});
    ca_.Branch(tmp253, &block250, std::vector<compiler::Node*>{phi_bb241_13, phi_bb241_14, phi_bb241_16, phi_bb241_17, phi_bb241_20, phi_bb241_21, phi_bb241_26, phi_bb241_30, phi_bb241_31}, &block251, std::vector<compiler::Node*>{phi_bb241_13, phi_bb241_14, phi_bb241_16, phi_bb241_17, phi_bb241_20, phi_bb241_21, phi_bb241_26, phi_bb241_30, phi_bb241_31});
  }

  TNode<FixedArray> phi_bb242_13;
  TNode<IntPtrT> phi_bb242_14;
  TNode<BoolT> phi_bb242_16;
  TNode<PrimitiveHeapObject> phi_bb242_17;
  TNode<String> phi_bb242_20;
  TNode<String> phi_bb242_21;
  TNode<String> phi_bb242_26;
  TNode<String> phi_bb242_30;
  TNode<Object> phi_bb242_31;
  if (block242.is_used()) {
    ca_.Bind(&block242, &phi_bb242_13, &phi_bb242_14, &phi_bb242_16, &phi_bb242_17, &phi_bb242_20, &phi_bb242_21, &phi_bb242_26, &phi_bb242_30, &phi_bb242_31);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb250_13;
  TNode<IntPtrT> phi_bb250_14;
  TNode<BoolT> phi_bb250_16;
  TNode<PrimitiveHeapObject> phi_bb250_17;
  TNode<String> phi_bb250_20;
  TNode<String> phi_bb250_21;
  TNode<String> phi_bb250_26;
  TNode<String> phi_bb250_30;
  TNode<Object> phi_bb250_31;
  TNode<IntPtrT> tmp254;
  TNode<IntPtrT> tmp255;
  TNode<Union<HeapObject, TaggedIndex>> tmp256;
  TNode<IntPtrT> tmp257;
  TNode<IntPtrT> tmp258;
  if (block250.is_used()) {
    ca_.Bind(&block250, &phi_bb250_13, &phi_bb250_14, &phi_bb250_16, &phi_bb250_17, &phi_bb250_20, &phi_bb250_21, &phi_bb250_26, &phi_bb250_30, &phi_bb250_31);
    tmp254 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp250});
    tmp255 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp247}, TNode<IntPtrT>{tmp254});
    std::tie(tmp256, tmp257) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp246}, TNode<IntPtrT>{tmp255}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp256, tmp257}, phi_bb250_31);
    tmp258 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block210, tmp220, tmp258, phi_bb250_16, phi_bb250_17, phi_bb250_20, phi_bb250_21, phi_bb250_26, phi_bb250_30, phi_bb250_31);
  }

  TNode<FixedArray> phi_bb251_13;
  TNode<IntPtrT> phi_bb251_14;
  TNode<BoolT> phi_bb251_16;
  TNode<PrimitiveHeapObject> phi_bb251_17;
  TNode<String> phi_bb251_20;
  TNode<String> phi_bb251_21;
  TNode<String> phi_bb251_26;
  TNode<String> phi_bb251_30;
  TNode<Object> phi_bb251_31;
  if (block251.is_used()) {
    ca_.Bind(&block251, &phi_bb251_13, &phi_bb251_14, &phi_bb251_16, &phi_bb251_17, &phi_bb251_20, &phi_bb251_21, &phi_bb251_26, &phi_bb251_30, &phi_bb251_31);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb210_13;
  TNode<IntPtrT> phi_bb210_14;
  TNode<BoolT> phi_bb210_16;
  TNode<PrimitiveHeapObject> phi_bb210_17;
  TNode<String> phi_bb210_20;
  TNode<String> phi_bb210_21;
  TNode<String> phi_bb210_26;
  TNode<String> phi_bb210_30;
  TNode<Object> phi_bb210_31;
  if (block210.is_used()) {
    ca_.Bind(&block210, &phi_bb210_13, &phi_bb210_14, &phi_bb210_16, &phi_bb210_17, &phi_bb210_20, &phi_bb210_21, &phi_bb210_26, &phi_bb210_30, &phi_bb210_31);
    ca_.Goto(&block103, phi_bb210_13, phi_bb210_14, phi_bb210_16, phi_bb210_26, phi_bb210_20, phi_bb210_21, phi_bb210_26);
  }

  TNode<FixedArray> phi_bb103_13;
  TNode<IntPtrT> phi_bb103_14;
  TNode<BoolT> phi_bb103_16;
  TNode<PrimitiveHeapObject> phi_bb103_17;
  TNode<String> phi_bb103_20;
  TNode<String> phi_bb103_21;
  TNode<String> phi_bb103_26;
  TNode<IntPtrT> tmp259;
  TNode<Map> tmp260;
  TNode<BoolT> tmp261;
  TNode<BoolT> tmp262;
  TNode<IntPtrT> tmp263;
  if (block103.is_used()) {
    ca_.Bind(&block103, &phi_bb103_13, &phi_bb103_14, &phi_bb103_16, &phi_bb103_17, &phi_bb103_20, &phi_bb103_21, &phi_bb103_26);
    tmp259 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp260 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{phi_bb103_26, tmp259});
    tmp261 = CodeStubAssembler(state_).IsOneByteStringMap(TNode<Map>{tmp260});
    tmp262 = CodeStubAssembler(state_).Word32And(TNode<BoolT>{tmp261}, TNode<BoolT>{phi_bb103_16});
    tmp263 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block4, tmp263, phi_bb6_11, phi_bb103_13, phi_bb103_14, tmp109, tmp262, phi_bb103_17, tmp19);
  }

  TNode<IntPtrT> phi_bb3_10;
  TNode<BuiltinPtr> phi_bb3_11;
  TNode<FixedArray> phi_bb3_13;
  TNode<IntPtrT> phi_bb3_14;
  TNode<IntPtrT> phi_bb3_15;
  TNode<BoolT> phi_bb3_16;
  TNode<PrimitiveHeapObject> phi_bb3_17;
  TNode<UintPtrT> phi_bb3_18;
  TNode<BoolT> tmp264;
  TNode<IntPtrT> tmp265;
  TNode<BoolT> tmp266;
  if (block3.is_used()) {
    ca_.Bind(&block3, &phi_bb3_10, &phi_bb3_11, &phi_bb3_13, &phi_bb3_14, &phi_bb3_15, &phi_bb3_16, &phi_bb3_17, &phi_bb3_18);
    tmp264 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    tmp265 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp266 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb3_10}, TNode<IntPtrT>{tmp265});
    ca_.Branch(tmp266, &block257, std::vector<compiler::Node*>{phi_bb3_10, phi_bb3_11, phi_bb3_13, phi_bb3_14, phi_bb3_15, phi_bb3_16, phi_bb3_17, phi_bb3_18, phi_bb3_10, phi_bb3_10}, &block258, std::vector<compiler::Node*>{phi_bb3_10, phi_bb3_11, phi_bb3_13, phi_bb3_14, phi_bb3_15, phi_bb3_16, phi_bb3_17, phi_bb3_18, phi_bb3_10, phi_bb3_10});
  }

  TNode<IntPtrT> phi_bb257_10;
  TNode<BuiltinPtr> phi_bb257_11;
  TNode<FixedArray> phi_bb257_13;
  TNode<IntPtrT> phi_bb257_14;
  TNode<IntPtrT> phi_bb257_15;
  TNode<BoolT> phi_bb257_16;
  TNode<PrimitiveHeapObject> phi_bb257_17;
  TNode<UintPtrT> phi_bb257_18;
  TNode<IntPtrT> phi_bb257_19;
  TNode<IntPtrT> phi_bb257_23;
  TNode<BoolT> tmp267;
  if (block257.is_used()) {
    ca_.Bind(&block257, &phi_bb257_10, &phi_bb257_11, &phi_bb257_13, &phi_bb257_14, &phi_bb257_15, &phi_bb257_16, &phi_bb257_17, &phi_bb257_18, &phi_bb257_19, &phi_bb257_23);
    tmp267 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block259, phi_bb257_10, phi_bb257_11, phi_bb257_13, phi_bb257_14, phi_bb257_15, phi_bb257_16, phi_bb257_17, phi_bb257_18, phi_bb257_19, phi_bb257_23, tmp267);
  }

  TNode<IntPtrT> phi_bb258_10;
  TNode<BuiltinPtr> phi_bb258_11;
  TNode<FixedArray> phi_bb258_13;
  TNode<IntPtrT> phi_bb258_14;
  TNode<IntPtrT> phi_bb258_15;
  TNode<BoolT> phi_bb258_16;
  TNode<PrimitiveHeapObject> phi_bb258_17;
  TNode<UintPtrT> phi_bb258_18;
  TNode<IntPtrT> phi_bb258_19;
  TNode<IntPtrT> phi_bb258_23;
  TNode<IntPtrT> tmp268;
  TNode<BoolT> tmp269;
  if (block258.is_used()) {
    ca_.Bind(&block258, &phi_bb258_10, &phi_bb258_11, &phi_bb258_13, &phi_bb258_14, &phi_bb258_15, &phi_bb258_16, &phi_bb258_17, &phi_bb258_18, &phi_bb258_19, &phi_bb258_23);
    tmp268 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp269 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp268});
    ca_.Goto(&block259, phi_bb258_10, phi_bb258_11, phi_bb258_13, phi_bb258_14, phi_bb258_15, phi_bb258_16, phi_bb258_17, phi_bb258_18, phi_bb258_19, phi_bb258_23, tmp269);
  }

  TNode<IntPtrT> phi_bb259_10;
  TNode<BuiltinPtr> phi_bb259_11;
  TNode<FixedArray> phi_bb259_13;
  TNode<IntPtrT> phi_bb259_14;
  TNode<IntPtrT> phi_bb259_15;
  TNode<BoolT> phi_bb259_16;
  TNode<PrimitiveHeapObject> phi_bb259_17;
  TNode<UintPtrT> phi_bb259_18;
  TNode<IntPtrT> phi_bb259_19;
  TNode<IntPtrT> phi_bb259_23;
  TNode<BoolT> phi_bb259_27;
  if (block259.is_used()) {
    ca_.Bind(&block259, &phi_bb259_10, &phi_bb259_11, &phi_bb259_13, &phi_bb259_14, &phi_bb259_15, &phi_bb259_16, &phi_bb259_17, &phi_bb259_18, &phi_bb259_19, &phi_bb259_23, &phi_bb259_27);
    ca_.Branch(phi_bb259_27, &block255, std::vector<compiler::Node*>{phi_bb259_10, phi_bb259_11, phi_bb259_13, phi_bb259_14, phi_bb259_15, phi_bb259_16, phi_bb259_17, phi_bb259_18, phi_bb259_19, phi_bb259_23}, &block256, std::vector<compiler::Node*>{phi_bb259_10, phi_bb259_11, phi_bb259_13, phi_bb259_14, phi_bb259_15, phi_bb259_16, phi_bb259_17, phi_bb259_18, phi_bb259_19, phi_bb259_23});
  }

  TNode<IntPtrT> phi_bb255_10;
  TNode<BuiltinPtr> phi_bb255_11;
  TNode<FixedArray> phi_bb255_13;
  TNode<IntPtrT> phi_bb255_14;
  TNode<IntPtrT> phi_bb255_15;
  TNode<BoolT> phi_bb255_16;
  TNode<PrimitiveHeapObject> phi_bb255_17;
  TNode<UintPtrT> phi_bb255_18;
  TNode<IntPtrT> phi_bb255_19;
  TNode<IntPtrT> phi_bb255_23;
  if (block255.is_used()) {
    ca_.Bind(&block255, &phi_bb255_10, &phi_bb255_11, &phi_bb255_13, &phi_bb255_14, &phi_bb255_15, &phi_bb255_16, &phi_bb255_17, &phi_bb255_18, &phi_bb255_19, &phi_bb255_23);
    ca_.Goto(&block254, phi_bb255_10, phi_bb255_11, phi_bb255_13, phi_bb255_14, phi_bb255_15, phi_bb255_16, phi_bb255_17, phi_bb255_18, phi_bb255_19, phi_bb255_23);
  }

  TNode<IntPtrT> phi_bb256_10;
  TNode<BuiltinPtr> phi_bb256_11;
  TNode<FixedArray> phi_bb256_13;
  TNode<IntPtrT> phi_bb256_14;
  TNode<IntPtrT> phi_bb256_15;
  TNode<BoolT> phi_bb256_16;
  TNode<PrimitiveHeapObject> phi_bb256_17;
  TNode<UintPtrT> phi_bb256_18;
  TNode<IntPtrT> phi_bb256_19;
  TNode<IntPtrT> phi_bb256_23;
  TNode<IntPtrT> tmp270;
  TNode<IntPtrT> tmp271;
  TNode<BoolT> tmp272;
  if (block256.is_used()) {
    ca_.Bind(&block256, &phi_bb256_10, &phi_bb256_11, &phi_bb256_13, &phi_bb256_14, &phi_bb256_15, &phi_bb256_16, &phi_bb256_17, &phi_bb256_18, &phi_bb256_19, &phi_bb256_23);
    tmp270 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{phi_bb256_23});
    tmp271 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp270}, TNode<IntPtrT>{tmp3});
    tmp272 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{tmp271}, TNode<IntPtrT>{phi_bb256_23});
    ca_.Branch(tmp272, &block260, std::vector<compiler::Node*>{phi_bb256_10, phi_bb256_11, phi_bb256_13, phi_bb256_14, phi_bb256_15, phi_bb256_16, phi_bb256_17, phi_bb256_18, phi_bb256_19, phi_bb256_23, phi_bb256_23}, &block261, std::vector<compiler::Node*>{phi_bb256_10, phi_bb256_11, phi_bb256_13, phi_bb256_14, phi_bb256_15, phi_bb256_16, phi_bb256_17, phi_bb256_18, phi_bb256_19, phi_bb256_23, phi_bb256_23});
  }

  TNode<IntPtrT> phi_bb260_10;
  TNode<BuiltinPtr> phi_bb260_11;
  TNode<FixedArray> phi_bb260_13;
  TNode<IntPtrT> phi_bb260_14;
  TNode<IntPtrT> phi_bb260_15;
  TNode<BoolT> phi_bb260_16;
  TNode<PrimitiveHeapObject> phi_bb260_17;
  TNode<UintPtrT> phi_bb260_18;
  TNode<IntPtrT> phi_bb260_19;
  TNode<IntPtrT> phi_bb260_23;
  TNode<IntPtrT> phi_bb260_26;
  if (block260.is_used()) {
    ca_.Bind(&block260, &phi_bb260_10, &phi_bb260_11, &phi_bb260_13, &phi_bb260_14, &phi_bb260_15, &phi_bb260_16, &phi_bb260_17, &phi_bb260_18, &phi_bb260_19, &phi_bb260_23, &phi_bb260_26);
    CodeStubAssembler(state_).CallRuntime(Runtime::kThrowInvalidStringLength, p_context);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb261_10;
  TNode<BuiltinPtr> phi_bb261_11;
  TNode<FixedArray> phi_bb261_13;
  TNode<IntPtrT> phi_bb261_14;
  TNode<IntPtrT> phi_bb261_15;
  TNode<BoolT> phi_bb261_16;
  TNode<PrimitiveHeapObject> phi_bb261_17;
  TNode<UintPtrT> phi_bb261_18;
  TNode<IntPtrT> phi_bb261_19;
  TNode<IntPtrT> phi_bb261_23;
  TNode<IntPtrT> phi_bb261_26;
  TNode<IntPtrT> tmp273;
  if (block261.is_used()) {
    ca_.Bind(&block261, &phi_bb261_10, &phi_bb261_11, &phi_bb261_13, &phi_bb261_14, &phi_bb261_15, &phi_bb261_16, &phi_bb261_17, &phi_bb261_18, &phi_bb261_19, &phi_bb261_23, &phi_bb261_26);
    tmp273 = AddStringLength_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{phi_bb261_15}, TNode<IntPtrT>{tmp270});
    ca_.Branch(tmp264, &block262, std::vector<compiler::Node*>{phi_bb261_10, phi_bb261_11, phi_bb261_13, phi_bb261_14, phi_bb261_16, phi_bb261_17, phi_bb261_18, phi_bb261_19, phi_bb261_23, phi_bb261_26}, &block263, std::vector<compiler::Node*>{phi_bb261_10, phi_bb261_11, phi_bb261_13, phi_bb261_14, phi_bb261_16, phi_bb261_17, phi_bb261_18, phi_bb261_19, phi_bb261_23, phi_bb261_26});
  }

  TNode<IntPtrT> phi_bb262_10;
  TNode<BuiltinPtr> phi_bb262_11;
  TNode<FixedArray> phi_bb262_13;
  TNode<IntPtrT> phi_bb262_14;
  TNode<BoolT> phi_bb262_16;
  TNode<PrimitiveHeapObject> phi_bb262_17;
  TNode<UintPtrT> phi_bb262_18;
  TNode<IntPtrT> phi_bb262_19;
  TNode<IntPtrT> phi_bb262_23;
  TNode<IntPtrT> phi_bb262_26;
  TNode<Smi> tmp274;
  TNode<IntPtrT> tmp275;
  TNode<BoolT> tmp276;
  if (block262.is_used()) {
    ca_.Bind(&block262, &phi_bb262_10, &phi_bb262_11, &phi_bb262_13, &phi_bb262_14, &phi_bb262_16, &phi_bb262_17, &phi_bb262_18, &phi_bb262_19, &phi_bb262_23, &phi_bb262_26);
    tmp274 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{phi_bb262_26});
    tmp275 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{phi_bb262_13});
    tmp276 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{phi_bb262_14}, TNode<IntPtrT>{tmp275});
    ca_.Branch(tmp276, &block273, std::vector<compiler::Node*>{phi_bb262_10, phi_bb262_11, phi_bb262_13, phi_bb262_14, phi_bb262_16, phi_bb262_17, phi_bb262_18, phi_bb262_19, phi_bb262_23, phi_bb262_26}, &block274, std::vector<compiler::Node*>{phi_bb262_10, phi_bb262_11, phi_bb262_13, phi_bb262_14, phi_bb262_16, phi_bb262_17, phi_bb262_18, phi_bb262_19, phi_bb262_23, phi_bb262_26});
  }

  TNode<IntPtrT> phi_bb273_10;
  TNode<BuiltinPtr> phi_bb273_11;
  TNode<FixedArray> phi_bb273_13;
  TNode<IntPtrT> phi_bb273_14;
  TNode<BoolT> phi_bb273_16;
  TNode<PrimitiveHeapObject> phi_bb273_17;
  TNode<UintPtrT> phi_bb273_18;
  TNode<IntPtrT> phi_bb273_19;
  TNode<IntPtrT> phi_bb273_23;
  TNode<IntPtrT> phi_bb273_26;
  TNode<Union<HeapObject, TaggedIndex>> tmp277;
  TNode<IntPtrT> tmp278;
  TNode<IntPtrT> tmp279;
  TNode<IntPtrT> tmp280;
  TNode<IntPtrT> tmp281;
  TNode<UintPtrT> tmp282;
  TNode<UintPtrT> tmp283;
  TNode<BoolT> tmp284;
  if (block273.is_used()) {
    ca_.Bind(&block273, &phi_bb273_10, &phi_bb273_11, &phi_bb273_13, &phi_bb273_14, &phi_bb273_16, &phi_bb273_17, &phi_bb273_18, &phi_bb273_19, &phi_bb273_23, &phi_bb273_26);
    std::tie(tmp277, tmp278, tmp279) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb273_13}).Flatten();
    tmp280 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp281 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb273_14}, TNode<IntPtrT>{tmp280});
    tmp282 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb273_14});
    tmp283 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp279});
    tmp284 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp282}, TNode<UintPtrT>{tmp283});
    ca_.Branch(tmp284, &block280, std::vector<compiler::Node*>{phi_bb273_10, phi_bb273_11, phi_bb273_13, phi_bb273_16, phi_bb273_17, phi_bb273_18, phi_bb273_19, phi_bb273_23, phi_bb273_26, phi_bb273_13, phi_bb273_14, phi_bb273_14, phi_bb273_14, phi_bb273_14}, &block281, std::vector<compiler::Node*>{phi_bb273_10, phi_bb273_11, phi_bb273_13, phi_bb273_16, phi_bb273_17, phi_bb273_18, phi_bb273_19, phi_bb273_23, phi_bb273_26, phi_bb273_13, phi_bb273_14, phi_bb273_14, phi_bb273_14, phi_bb273_14});
  }

  TNode<IntPtrT> phi_bb280_10;
  TNode<BuiltinPtr> phi_bb280_11;
  TNode<FixedArray> phi_bb280_13;
  TNode<BoolT> phi_bb280_16;
  TNode<PrimitiveHeapObject> phi_bb280_17;
  TNode<UintPtrT> phi_bb280_18;
  TNode<IntPtrT> phi_bb280_19;
  TNode<IntPtrT> phi_bb280_23;
  TNode<IntPtrT> phi_bb280_26;
  TNode<FixedArray> phi_bb280_31;
  TNode<IntPtrT> phi_bb280_35;
  TNode<IntPtrT> phi_bb280_36;
  TNode<IntPtrT> phi_bb280_40;
  TNode<IntPtrT> phi_bb280_41;
  TNode<IntPtrT> tmp285;
  TNode<IntPtrT> tmp286;
  TNode<Union<HeapObject, TaggedIndex>> tmp287;
  TNode<IntPtrT> tmp288;
  if (block280.is_used()) {
    ca_.Bind(&block280, &phi_bb280_10, &phi_bb280_11, &phi_bb280_13, &phi_bb280_16, &phi_bb280_17, &phi_bb280_18, &phi_bb280_19, &phi_bb280_23, &phi_bb280_26, &phi_bb280_31, &phi_bb280_35, &phi_bb280_36, &phi_bb280_40, &phi_bb280_41);
    tmp285 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb280_41});
    tmp286 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp278}, TNode<IntPtrT>{tmp285});
    std::tie(tmp287, tmp288) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp277}, TNode<IntPtrT>{tmp286}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp287, tmp288}, tmp274);
    ca_.Goto(&block275, phi_bb280_10, phi_bb280_11, phi_bb280_13, tmp281, phi_bb280_16, phi_bb280_17, phi_bb280_18, phi_bb280_19, phi_bb280_23, phi_bb280_26);
  }

  TNode<IntPtrT> phi_bb281_10;
  TNode<BuiltinPtr> phi_bb281_11;
  TNode<FixedArray> phi_bb281_13;
  TNode<BoolT> phi_bb281_16;
  TNode<PrimitiveHeapObject> phi_bb281_17;
  TNode<UintPtrT> phi_bb281_18;
  TNode<IntPtrT> phi_bb281_19;
  TNode<IntPtrT> phi_bb281_23;
  TNode<IntPtrT> phi_bb281_26;
  TNode<FixedArray> phi_bb281_31;
  TNode<IntPtrT> phi_bb281_35;
  TNode<IntPtrT> phi_bb281_36;
  TNode<IntPtrT> phi_bb281_40;
  TNode<IntPtrT> phi_bb281_41;
  if (block281.is_used()) {
    ca_.Bind(&block281, &phi_bb281_10, &phi_bb281_11, &phi_bb281_13, &phi_bb281_16, &phi_bb281_17, &phi_bb281_18, &phi_bb281_19, &phi_bb281_23, &phi_bb281_26, &phi_bb281_31, &phi_bb281_35, &phi_bb281_36, &phi_bb281_40, &phi_bb281_41);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb274_10;
  TNode<BuiltinPtr> phi_bb274_11;
  TNode<FixedArray> phi_bb274_13;
  TNode<IntPtrT> phi_bb274_14;
  TNode<BoolT> phi_bb274_16;
  TNode<PrimitiveHeapObject> phi_bb274_17;
  TNode<UintPtrT> phi_bb274_18;
  TNode<IntPtrT> phi_bb274_19;
  TNode<IntPtrT> phi_bb274_23;
  TNode<IntPtrT> phi_bb274_26;
  TNode<IntPtrT> tmp289;
  TNode<IntPtrT> tmp290;
  TNode<BoolT> tmp291;
  if (block274.is_used()) {
    ca_.Bind(&block274, &phi_bb274_10, &phi_bb274_11, &phi_bb274_13, &phi_bb274_14, &phi_bb274_16, &phi_bb274_17, &phi_bb274_18, &phi_bb274_19, &phi_bb274_23, &phi_bb274_26);
    tmp289 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp275});
    tmp290 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    tmp291 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp289}, TNode<IntPtrT>{tmp290});
    ca_.Branch(tmp291, &block284, std::vector<compiler::Node*>{phi_bb274_10, phi_bb274_11, phi_bb274_13, phi_bb274_14, phi_bb274_16, phi_bb274_17, phi_bb274_18, phi_bb274_19, phi_bb274_23, phi_bb274_26}, &block285, std::vector<compiler::Node*>{phi_bb274_10, phi_bb274_11, phi_bb274_13, phi_bb274_14, phi_bb274_16, phi_bb274_17, phi_bb274_18, phi_bb274_19, phi_bb274_23, phi_bb274_26});
  }

  TNode<IntPtrT> phi_bb284_10;
  TNode<BuiltinPtr> phi_bb284_11;
  TNode<FixedArray> phi_bb284_13;
  TNode<IntPtrT> phi_bb284_14;
  TNode<BoolT> phi_bb284_16;
  TNode<PrimitiveHeapObject> phi_bb284_17;
  TNode<UintPtrT> phi_bb284_18;
  TNode<IntPtrT> phi_bb284_19;
  TNode<IntPtrT> phi_bb284_23;
  TNode<IntPtrT> phi_bb284_26;
  TNode<IntPtrT> tmp292;
  if (block284.is_used()) {
    ca_.Bind(&block284, &phi_bb284_10, &phi_bb284_11, &phi_bb284_13, &phi_bb284_14, &phi_bb284_16, &phi_bb284_17, &phi_bb284_18, &phi_bb284_19, &phi_bb284_23, &phi_bb284_26);
    tmp292 = FromConstexpr_intptr_constexpr_int31_0(state_, kMaxBufferChunkSize_0(state_));
    ca_.Goto(&block286, phi_bb284_10, phi_bb284_11, phi_bb284_13, phi_bb284_14, phi_bb284_16, phi_bb284_17, phi_bb284_18, phi_bb284_19, phi_bb284_23, phi_bb284_26, tmp292);
  }

  TNode<IntPtrT> phi_bb285_10;
  TNode<BuiltinPtr> phi_bb285_11;
  TNode<FixedArray> phi_bb285_13;
  TNode<IntPtrT> phi_bb285_14;
  TNode<BoolT> phi_bb285_16;
  TNode<PrimitiveHeapObject> phi_bb285_17;
  TNode<UintPtrT> phi_bb285_18;
  TNode<IntPtrT> phi_bb285_19;
  TNode<IntPtrT> phi_bb285_23;
  TNode<IntPtrT> phi_bb285_26;
  if (block285.is_used()) {
    ca_.Bind(&block285, &phi_bb285_10, &phi_bb285_11, &phi_bb285_13, &phi_bb285_14, &phi_bb285_16, &phi_bb285_17, &phi_bb285_18, &phi_bb285_19, &phi_bb285_23, &phi_bb285_26);
    ca_.Goto(&block286, phi_bb285_10, phi_bb285_11, phi_bb285_13, phi_bb285_14, phi_bb285_16, phi_bb285_17, phi_bb285_18, phi_bb285_19, phi_bb285_23, phi_bb285_26, tmp289);
  }

  TNode<IntPtrT> phi_bb286_10;
  TNode<BuiltinPtr> phi_bb286_11;
  TNode<FixedArray> phi_bb286_13;
  TNode<IntPtrT> phi_bb286_14;
  TNode<BoolT> phi_bb286_16;
  TNode<PrimitiveHeapObject> phi_bb286_17;
  TNode<UintPtrT> phi_bb286_18;
  TNode<IntPtrT> phi_bb286_19;
  TNode<IntPtrT> phi_bb286_23;
  TNode<IntPtrT> phi_bb286_26;
  TNode<IntPtrT> phi_bb286_32;
  TNode<FixedArray> tmp293;
  TNode<Union<HeapObject, TaggedIndex>> tmp294;
  TNode<IntPtrT> tmp295;
  TNode<IntPtrT> tmp296;
  TNode<UintPtrT> tmp297;
  TNode<IntPtrT> tmp298;
  TNode<UintPtrT> tmp299;
  TNode<UintPtrT> tmp300;
  TNode<BoolT> tmp301;
  if (block286.is_used()) {
    ca_.Bind(&block286, &phi_bb286_10, &phi_bb286_11, &phi_bb286_13, &phi_bb286_14, &phi_bb286_16, &phi_bb286_17, &phi_bb286_18, &phi_bb286_19, &phi_bb286_23, &phi_bb286_26, &phi_bb286_32);
    tmp293 = CodeStubAssembler(state_).AllocateZeroedFixedArray(TNode<IntPtrT>{phi_bb286_32});
    std::tie(tmp294, tmp295, tmp296) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb286_13}).Flatten();
    tmp297 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp298 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp297});
    tmp299 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp298});
    tmp300 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp296});
    tmp301 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp299}, TNode<UintPtrT>{tmp300});
    ca_.Branch(tmp301, &block297, std::vector<compiler::Node*>{phi_bb286_10, phi_bb286_11, phi_bb286_13, phi_bb286_14, phi_bb286_16, phi_bb286_17, phi_bb286_18, phi_bb286_19, phi_bb286_23, phi_bb286_26, phi_bb286_13}, &block298, std::vector<compiler::Node*>{phi_bb286_10, phi_bb286_11, phi_bb286_13, phi_bb286_14, phi_bb286_16, phi_bb286_17, phi_bb286_18, phi_bb286_19, phi_bb286_23, phi_bb286_26, phi_bb286_13});
  }

  TNode<IntPtrT> phi_bb297_10;
  TNode<BuiltinPtr> phi_bb297_11;
  TNode<FixedArray> phi_bb297_13;
  TNode<IntPtrT> phi_bb297_14;
  TNode<BoolT> phi_bb297_16;
  TNode<PrimitiveHeapObject> phi_bb297_17;
  TNode<UintPtrT> phi_bb297_18;
  TNode<IntPtrT> phi_bb297_19;
  TNode<IntPtrT> phi_bb297_23;
  TNode<IntPtrT> phi_bb297_26;
  TNode<FixedArray> phi_bb297_34;
  TNode<IntPtrT> tmp302;
  TNode<IntPtrT> tmp303;
  TNode<Union<HeapObject, TaggedIndex>> tmp304;
  TNode<IntPtrT> tmp305;
  TNode<Union<HeapObject, TaggedIndex>> tmp306;
  TNode<IntPtrT> tmp307;
  TNode<IntPtrT> tmp308;
  TNode<UintPtrT> tmp309;
  TNode<IntPtrT> tmp310;
  TNode<UintPtrT> tmp311;
  TNode<UintPtrT> tmp312;
  TNode<BoolT> tmp313;
  if (block297.is_used()) {
    ca_.Bind(&block297, &phi_bb297_10, &phi_bb297_11, &phi_bb297_13, &phi_bb297_14, &phi_bb297_16, &phi_bb297_17, &phi_bb297_18, &phi_bb297_19, &phi_bb297_23, &phi_bb297_26, &phi_bb297_34);
    tmp302 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp298});
    tmp303 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp295}, TNode<IntPtrT>{tmp302});
    std::tie(tmp304, tmp305) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp294}, TNode<IntPtrT>{tmp303}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp304, tmp305}, tmp293);
    std::tie(tmp306, tmp307, tmp308) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp293}).Flatten();
    tmp309 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp310 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp309});
    tmp311 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp310});
    tmp312 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp308});
    tmp313 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp311}, TNode<UintPtrT>{tmp312});
    ca_.Branch(tmp313, &block306, std::vector<compiler::Node*>{phi_bb297_10, phi_bb297_11, phi_bb297_13, phi_bb297_14, phi_bb297_16, phi_bb297_17, phi_bb297_18, phi_bb297_19, phi_bb297_23, phi_bb297_26}, &block307, std::vector<compiler::Node*>{phi_bb297_10, phi_bb297_11, phi_bb297_13, phi_bb297_14, phi_bb297_16, phi_bb297_17, phi_bb297_18, phi_bb297_19, phi_bb297_23, phi_bb297_26});
  }

  TNode<IntPtrT> phi_bb298_10;
  TNode<BuiltinPtr> phi_bb298_11;
  TNode<FixedArray> phi_bb298_13;
  TNode<IntPtrT> phi_bb298_14;
  TNode<BoolT> phi_bb298_16;
  TNode<PrimitiveHeapObject> phi_bb298_17;
  TNode<UintPtrT> phi_bb298_18;
  TNode<IntPtrT> phi_bb298_19;
  TNode<IntPtrT> phi_bb298_23;
  TNode<IntPtrT> phi_bb298_26;
  TNode<FixedArray> phi_bb298_34;
  if (block298.is_used()) {
    ca_.Bind(&block298, &phi_bb298_10, &phi_bb298_11, &phi_bb298_13, &phi_bb298_14, &phi_bb298_16, &phi_bb298_17, &phi_bb298_18, &phi_bb298_19, &phi_bb298_23, &phi_bb298_26, &phi_bb298_34);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb306_10;
  TNode<BuiltinPtr> phi_bb306_11;
  TNode<FixedArray> phi_bb306_13;
  TNode<IntPtrT> phi_bb306_14;
  TNode<BoolT> phi_bb306_16;
  TNode<PrimitiveHeapObject> phi_bb306_17;
  TNode<UintPtrT> phi_bb306_18;
  TNode<IntPtrT> phi_bb306_19;
  TNode<IntPtrT> phi_bb306_23;
  TNode<IntPtrT> phi_bb306_26;
  TNode<IntPtrT> tmp314;
  TNode<IntPtrT> tmp315;
  TNode<Union<HeapObject, TaggedIndex>> tmp316;
  TNode<IntPtrT> tmp317;
  TNode<Undefined> tmp318;
  TNode<Union<HeapObject, TaggedIndex>> tmp319;
  TNode<IntPtrT> tmp320;
  TNode<IntPtrT> tmp321;
  TNode<UintPtrT> tmp322;
  TNode<IntPtrT> tmp323;
  TNode<UintPtrT> tmp324;
  TNode<UintPtrT> tmp325;
  TNode<BoolT> tmp326;
  if (block306.is_used()) {
    ca_.Bind(&block306, &phi_bb306_10, &phi_bb306_11, &phi_bb306_13, &phi_bb306_14, &phi_bb306_16, &phi_bb306_17, &phi_bb306_18, &phi_bb306_19, &phi_bb306_23, &phi_bb306_26);
    tmp314 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp310});
    tmp315 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp307}, TNode<IntPtrT>{tmp314});
    std::tie(tmp316, tmp317) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp306}, TNode<IntPtrT>{tmp315}).Flatten();
    tmp318 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp316, tmp317}, tmp318);
    std::tie(tmp319, tmp320, tmp321) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp293}).Flatten();
    tmp322 = FromConstexpr_uintptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp323 = Convert_intptr_uintptr_0(state_, TNode<UintPtrT>{tmp322});
    tmp324 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp323});
    tmp325 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp321});
    tmp326 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp324}, TNode<UintPtrT>{tmp325});
    ca_.Branch(tmp326, &block315, std::vector<compiler::Node*>{phi_bb306_10, phi_bb306_11, phi_bb306_13, phi_bb306_14, phi_bb306_16, phi_bb306_17, phi_bb306_18, phi_bb306_19, phi_bb306_23, phi_bb306_26}, &block316, std::vector<compiler::Node*>{phi_bb306_10, phi_bb306_11, phi_bb306_13, phi_bb306_14, phi_bb306_16, phi_bb306_17, phi_bb306_18, phi_bb306_19, phi_bb306_23, phi_bb306_26});
  }

  TNode<IntPtrT> phi_bb307_10;
  TNode<BuiltinPtr> phi_bb307_11;
  TNode<FixedArray> phi_bb307_13;
  TNode<IntPtrT> phi_bb307_14;
  TNode<BoolT> phi_bb307_16;
  TNode<PrimitiveHeapObject> phi_bb307_17;
  TNode<UintPtrT> phi_bb307_18;
  TNode<IntPtrT> phi_bb307_19;
  TNode<IntPtrT> phi_bb307_23;
  TNode<IntPtrT> phi_bb307_26;
  if (block307.is_used()) {
    ca_.Bind(&block307, &phi_bb307_10, &phi_bb307_11, &phi_bb307_13, &phi_bb307_14, &phi_bb307_16, &phi_bb307_17, &phi_bb307_18, &phi_bb307_19, &phi_bb307_23, &phi_bb307_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb315_10;
  TNode<BuiltinPtr> phi_bb315_11;
  TNode<FixedArray> phi_bb315_13;
  TNode<IntPtrT> phi_bb315_14;
  TNode<BoolT> phi_bb315_16;
  TNode<PrimitiveHeapObject> phi_bb315_17;
  TNode<UintPtrT> phi_bb315_18;
  TNode<IntPtrT> phi_bb315_19;
  TNode<IntPtrT> phi_bb315_23;
  TNode<IntPtrT> phi_bb315_26;
  TNode<IntPtrT> tmp327;
  TNode<IntPtrT> tmp328;
  TNode<Union<HeapObject, TaggedIndex>> tmp329;
  TNode<IntPtrT> tmp330;
  TNode<IntPtrT> tmp331;
  if (block315.is_used()) {
    ca_.Bind(&block315, &phi_bb315_10, &phi_bb315_11, &phi_bb315_13, &phi_bb315_14, &phi_bb315_16, &phi_bb315_17, &phi_bb315_18, &phi_bb315_19, &phi_bb315_23, &phi_bb315_26);
    tmp327 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp323});
    tmp328 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp320}, TNode<IntPtrT>{tmp327});
    std::tie(tmp329, tmp330) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp319}, TNode<IntPtrT>{tmp328}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp329, tmp330}, tmp274);
    tmp331 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    ca_.Goto(&block275, phi_bb315_10, phi_bb315_11, tmp293, tmp331, phi_bb315_16, phi_bb315_17, phi_bb315_18, phi_bb315_19, phi_bb315_23, phi_bb315_26);
  }

  TNode<IntPtrT> phi_bb316_10;
  TNode<BuiltinPtr> phi_bb316_11;
  TNode<FixedArray> phi_bb316_13;
  TNode<IntPtrT> phi_bb316_14;
  TNode<BoolT> phi_bb316_16;
  TNode<PrimitiveHeapObject> phi_bb316_17;
  TNode<UintPtrT> phi_bb316_18;
  TNode<IntPtrT> phi_bb316_19;
  TNode<IntPtrT> phi_bb316_23;
  TNode<IntPtrT> phi_bb316_26;
  if (block316.is_used()) {
    ca_.Bind(&block316, &phi_bb316_10, &phi_bb316_11, &phi_bb316_13, &phi_bb316_14, &phi_bb316_16, &phi_bb316_17, &phi_bb316_18, &phi_bb316_19, &phi_bb316_23, &phi_bb316_26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb275_10;
  TNode<BuiltinPtr> phi_bb275_11;
  TNode<FixedArray> phi_bb275_13;
  TNode<IntPtrT> phi_bb275_14;
  TNode<BoolT> phi_bb275_16;
  TNode<PrimitiveHeapObject> phi_bb275_17;
  TNode<UintPtrT> phi_bb275_18;
  TNode<IntPtrT> phi_bb275_19;
  TNode<IntPtrT> phi_bb275_23;
  TNode<IntPtrT> phi_bb275_26;
  TNode<Null> tmp332;
  if (block275.is_used()) {
    ca_.Bind(&block275, &phi_bb275_10, &phi_bb275_11, &phi_bb275_13, &phi_bb275_14, &phi_bb275_16, &phi_bb275_17, &phi_bb275_18, &phi_bb275_19, &phi_bb275_23, &phi_bb275_26);
    tmp332 = Null_0(state_);
    ca_.Goto(&block263, phi_bb275_10, phi_bb275_11, phi_bb275_13, phi_bb275_14, phi_bb275_16, tmp332, phi_bb275_18, phi_bb275_19, phi_bb275_23, phi_bb275_26);
  }

  TNode<IntPtrT> phi_bb263_10;
  TNode<BuiltinPtr> phi_bb263_11;
  TNode<FixedArray> phi_bb263_13;
  TNode<IntPtrT> phi_bb263_14;
  TNode<BoolT> phi_bb263_16;
  TNode<PrimitiveHeapObject> phi_bb263_17;
  TNode<UintPtrT> phi_bb263_18;
  TNode<IntPtrT> phi_bb263_19;
  TNode<IntPtrT> phi_bb263_23;
  TNode<IntPtrT> phi_bb263_26;
  if (block263.is_used()) {
    ca_.Bind(&block263, &phi_bb263_10, &phi_bb263_11, &phi_bb263_13, &phi_bb263_14, &phi_bb263_16, &phi_bb263_17, &phi_bb263_18, &phi_bb263_19, &phi_bb263_23, &phi_bb263_26);
    ca_.Goto(&block254, phi_bb263_10, phi_bb263_11, phi_bb263_13, phi_bb263_14, tmp273, phi_bb263_16, phi_bb263_17, phi_bb263_18, phi_bb263_19, phi_bb263_23);
  }

  TNode<IntPtrT> phi_bb254_10;
  TNode<BuiltinPtr> phi_bb254_11;
  TNode<FixedArray> phi_bb254_13;
  TNode<IntPtrT> phi_bb254_14;
  TNode<IntPtrT> phi_bb254_15;
  TNode<BoolT> phi_bb254_16;
  TNode<PrimitiveHeapObject> phi_bb254_17;
  TNode<UintPtrT> phi_bb254_18;
  TNode<IntPtrT> phi_bb254_19;
  TNode<IntPtrT> phi_bb254_23;
  TNode<String> tmp333;
  if (block254.is_used()) {
    ca_.Bind(&block254, &phi_bb254_10, &phi_bb254_11, &phi_bb254_13, &phi_bb254_14, &phi_bb254_15, &phi_bb254_16, &phi_bb254_17, &phi_bb254_18, &phi_bb254_19, &phi_bb254_23);
    tmp333 = BufferJoin_0(state_, TNode<Context>{p_context}, TorqueStructBuffer_0{TNode<FixedArray>{tmp5}, TNode<FixedArray>{phi_bb254_13}, TNode<IntPtrT>{phi_bb254_14}, TNode<IntPtrT>{phi_bb254_15}, TNode<BoolT>{phi_bb254_16}, TNode<PrimitiveHeapObject>{phi_bb254_17}}, TNode<String>{p_sep});
    ca_.Goto(&block319);
  }

    ca_.Bind(&block319);
  return TNode<String>{tmp333};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=644&c=16
TorqueStructReference_Undefined_OR_FixedArray_0 NativeContextSlot_Context_Undefined_OR_FixedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_index) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = NativeContextSlot_Undefined_OR_FixedArray_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{p_index}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_Undefined_OR_FixedArray_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=680&c=7
TNode<FixedArray> StoreAndGrowFixedArray_JSReceiver_0(compiler::CodeAssemblerState* state_, TNode<FixedArray> p_fixedArray, TNode<IntPtrT> p_index, TNode<JSReceiver> p_element) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<FixedArray> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).LoadAndUntagFixedArrayBaseLength(TNode<FixedArrayBase>{p_fixedArray});
    tmp1 = CodeStubAssembler(state_).IntPtrLessThan(TNode<IntPtrT>{p_index}, TNode<IntPtrT>{tmp0});
    ca_.Branch(tmp1, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<UintPtrT> tmp5;
  TNode<UintPtrT> tmp6;
  TNode<BoolT> tmp7;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    std::tie(tmp2, tmp3, tmp4) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{p_fixedArray}).Flatten();
    tmp5 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{p_index});
    tmp6 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp4});
    tmp7 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp5}, TNode<UintPtrT>{tmp6});
    ca_.Branch(tmp7, &block13, std::vector<compiler::Node*>{}, &block14, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<Union<HeapObject, TaggedIndex>> tmp10;
  TNode<IntPtrT> tmp11;
  if (block13.is_used()) {
    ca_.Bind(&block13);
    tmp8 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{p_index});
    tmp9 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp3}, TNode<IntPtrT>{tmp8});
    std::tie(tmp10, tmp11) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp2}, TNode<IntPtrT>{tmp9}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp10, tmp11}, p_element);
    ca_.Goto(&block1, p_fixedArray);
  }

  if (block14.is_used()) {
    ca_.Bind(&block14);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> tmp12;
  TNode<TheHole> tmp13;
  TNode<IntPtrT> tmp14;
  TNode<FixedArray> tmp15;
  TNode<Union<HeapObject, TaggedIndex>> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<UintPtrT> tmp19;
  TNode<UintPtrT> tmp20;
  TNode<BoolT> tmp21;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp12 = CodeStubAssembler(state_).CalculateNewElementsCapacity(TNode<IntPtrT>{tmp0});
    tmp13 = TheHole_0(state_);
    tmp14 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp15 = ExtractFixedArray_0(state_, TNode<FixedArray>{p_fixedArray}, TNode<IntPtrT>{tmp14}, TNode<IntPtrT>{tmp0}, TNode<IntPtrT>{tmp12}, TNode<Hole>{tmp13});
    std::tie(tmp16, tmp17, tmp18) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp15}).Flatten();
    tmp19 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{p_index});
    tmp20 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp18});
    tmp21 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp19}, TNode<UintPtrT>{tmp20});
    ca_.Branch(tmp21, &block25, std::vector<compiler::Node*>{}, &block26, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp22;
  TNode<IntPtrT> tmp23;
  TNode<Union<HeapObject, TaggedIndex>> tmp24;
  TNode<IntPtrT> tmp25;
  if (block25.is_used()) {
    ca_.Bind(&block25);
    tmp22 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{p_index});
    tmp23 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp17}, TNode<IntPtrT>{tmp22});
    std::tie(tmp24, tmp25) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp16}, TNode<IntPtrT>{tmp23}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp24, tmp25}, p_element);
    ca_.Goto(&block1, tmp15);
  }

  if (block26.is_used()) {
    ca_.Bind(&block26);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<FixedArray> phi_bb1_3;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_3);
    ca_.Goto(&block29);
  }

    ca_.Bind(&block29);
  return TNode<FixedArray>{phi_bb1_3};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=832&c=10
TNode<JSAny> CycleProtectedArrayJoin_JSArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, bool p_useToLocaleString, TNode<JSReceiver> p_o, TNode<Number> p_len, TNode<String> p_sep, TNode<JSAny> p_locales, TNode<JSAny> p_options) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block9(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Number> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = NumberIsGreaterThan_0(state_, TNode<Number>{p_len}, TNode<Number>{tmp0});
    ca_.Branch(tmp1, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp2;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp2 = JoinStackPushInline_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{p_o});
    ca_.Goto(&block6, tmp2);
  }

  TNode<BoolT> tmp3;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp3 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block6, tmp3);
  }

  TNode<BoolT> phi_bb6_7;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_7);
    ca_.Branch(phi_bb6_7, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<JSAny> tmp4;
      TNode<JSAny> tmp6;
      TNode<JSAny> tmp8;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    compiler::CodeAssemblerExceptionHandlerLabel catch5__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch5__label);
    tmp4 = ArrayJoin_JSArray_0(state_, TNode<Context>{p_context}, p_useToLocaleString, TNode<JSReceiver>{p_o}, TNode<String>{p_sep}, TNode<Number>{p_len}, TNode<JSAny>{p_locales}, TNode<JSAny>{p_options});
    }
    if (catch5__label.is_used()) {
      compiler::CodeAssemblerLabel catch5_skip(&ca_);
      ca_.Goto(&catch5_skip);
      ca_.Bind(&catch5__label, &tmp6);
      ca_.Goto(&block10);
      ca_.Bind(&catch5_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch7__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch7__label);
    JoinStackPopInline_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{p_o});
    }
    if (catch7__label.is_used()) {
      compiler::CodeAssemblerLabel catch7_skip(&ca_);
      ca_.Goto(&catch7_skip);
      ca_.Bind(&catch7__label, &tmp8);
      ca_.Goto(&block11);
      ca_.Bind(&catch7_skip);
    }
    ca_.Goto(&block1, tmp4);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp9;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp9 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block9, tmp6, tmp9);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp10;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp10 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block9, tmp8, tmp10);
  }

  TNode<JSAny> phi_bb9_6;
  TNode<Union<JSMessageObject, TheHole>> phi_bb9_7;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_6, &phi_bb9_7);
    JoinStackPopInline_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{p_o});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, p_context, phi_bb9_6, phi_bb9_7);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> tmp11;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp11 = kEmptyString_0(state_);
    ca_.Goto(&block1, tmp11);
  }

  TNode<JSAny> phi_bb1_6;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_6);
    ca_.Goto(&block12);
  }

    ca_.Bind(&block12);
  return TNode<JSAny>{phi_bb1_6};
}

// https://crsrc.org/c/v8/src/builtins/array-join.tq?l=919&c=10
TNode<JSAny> CycleProtectedArrayJoin_JSTypedArray_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, bool p_useToLocaleString, TNode<JSReceiver> p_o, TNode<Number> p_len, TNode<String> p_sep, TNode<JSAny> p_locales, TNode<JSAny> p_options) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block11(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, Union<JSMessageObject, TheHole>> block9(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Number> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_Number_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp1 = NumberIsGreaterThan_0(state_, TNode<Number>{p_len}, TNode<Number>{tmp0});
    ca_.Branch(tmp1, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<BoolT> tmp2;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp2 = JoinStackPushInline_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{p_o});
    ca_.Goto(&block6, tmp2);
  }

  TNode<BoolT> tmp3;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp3 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block6, tmp3);
  }

  TNode<BoolT> phi_bb6_7;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_7);
    ca_.Branch(phi_bb6_7, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<JSAny> tmp4;
      TNode<JSAny> tmp6;
      TNode<JSAny> tmp8;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    compiler::CodeAssemblerExceptionHandlerLabel catch5__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch5__label);
    tmp4 = ArrayJoin_JSTypedArray_0(state_, TNode<Context>{p_context}, p_useToLocaleString, TNode<JSReceiver>{p_o}, TNode<String>{p_sep}, TNode<Number>{p_len}, TNode<JSAny>{p_locales}, TNode<JSAny>{p_options});
    }
    if (catch5__label.is_used()) {
      compiler::CodeAssemblerLabel catch5_skip(&ca_);
      ca_.Goto(&catch5_skip);
      ca_.Bind(&catch5__label, &tmp6);
      ca_.Goto(&block10);
      ca_.Bind(&catch5_skip);
    }
    compiler::CodeAssemblerExceptionHandlerLabel catch7__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch7__label);
    JoinStackPopInline_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{p_o});
    }
    if (catch7__label.is_used()) {
      compiler::CodeAssemblerLabel catch7_skip(&ca_);
      ca_.Goto(&catch7_skip);
      ca_.Bind(&catch7__label, &tmp8);
      ca_.Goto(&block11);
      ca_.Bind(&catch7_skip);
    }
    ca_.Goto(&block1, tmp4);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp9;
  if (block10.is_used()) {
    ca_.Bind(&block10);
    tmp9 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block9, tmp6, tmp9);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp10;
  if (block11.is_used()) {
    ca_.Bind(&block11);
    tmp10 = GetAndResetPendingMessage_0(state_);
    ca_.Goto(&block9, tmp8, tmp10);
  }

  TNode<JSAny> phi_bb9_6;
  TNode<Union<JSMessageObject, TheHole>> phi_bb9_7;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_6, &phi_bb9_7);
    JoinStackPopInline_0(state_, TNode<Context>{p_context}, TNode<JSReceiver>{p_o});
    CodeStubAssembler(state_).CallRuntime(Runtime::kReThrowWithMessage, p_context, phi_bb9_6, phi_bb9_7);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<String> tmp11;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp11 = kEmptyString_0(state_);
    ca_.Goto(&block1, tmp11);
  }

  TNode<JSAny> phi_bb1_6;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_6);
    ca_.Goto(&block12);
  }

    ca_.Bind(&block12);
  return TNode<JSAny>{phi_bb1_6};
}

} // namespace internal
} // namespace v8
