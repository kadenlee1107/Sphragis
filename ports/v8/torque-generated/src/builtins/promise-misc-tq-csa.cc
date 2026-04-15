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
#include "torque-generated/src/builtins/promise-misc-tq-csa.h"
#include "torque-generated/src/objects/arguments-tq-csa.h"
#include "torque-generated/src/objects/js-promise-tq-csa.h"
#include "torque-generated/src/objects/js-function-tq-csa.h"
#include "torque-generated/src/objects/contexts-tq-csa.h"
#include "torque-generated/src/builtins/promise-resolve-tq-csa.h"
#include "torque-generated/src/builtins/promise-constructor-tq-csa.h"
#include "torque-generated/src/builtins/promise-finally-tq-csa.h"
#include "torque-generated/src/builtins/convert-tq-csa.h"
#include "torque-generated/src/builtins/conversion-tq-csa.h"
#include "torque-generated/src/builtins/ic-callable-tq-csa.h"
#include "torque-generated/src/builtins/torque-internal-tq-csa.h"
#include "torque-generated/src/builtins/cast-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"
#include "torque-generated/src/builtins/promise-all-tq-csa.h"
#include "torque-generated/src/builtins/promise-misc-tq-csa.h"
#include "torque-generated/src/builtins/promise-abstract-operations-tq-csa.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=42&c=1
TNode<BoolT> PromiseHasHandler_0(compiler::CodeAssemblerState* state_, TNode<JSPromise> p_promise) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Method_JSPromise_HasHandler_0(state_, TNode<JSPromise>{p_promise});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<BoolT>{tmp0};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=47&c=1
void PromiseInit_0(compiler::CodeAssemblerState* state_, TNode<JSPromise> p_promise) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Smi> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Int32T> tmp3;
  TNode<Uint32T> tmp4;
  TNode<Uint32T> tmp5;
  TNode<Int32T> tmp6;
  TNode<Uint32T> tmp7;
  TNode<BoolT> tmp8;
  TNode<Uint32T> tmp9;
  TNode<BoolT> tmp10;
  TNode<Uint32T> tmp11;
  TNode<Uint32T> tmp12;
  TNode<Uint32T> tmp13;
  TNode<Smi> tmp14;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp1 = kZero_0(state_);
    CodeStubAssembler(state_).StoreReference<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, PromiseReaction, Smi, String, Symbol, Undefined>>(CodeStubAssembler::Reference{p_promise, tmp0}, tmp1);
    tmp2 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp3 = FromConstexpr_int32_constexpr_int32_0(state_, 0);
    tmp4 = CodeStubAssembler(state_).Unsigned(TNode<Int32T>{tmp3});
    tmp5 = (TNode<Uint32T>{tmp4});
    tmp6 = FromConstexpr_PromiseState_constexpr_kPending_0(state_, Promise::PromiseState::kPending);
    tmp7 = ca_.UncheckedCast<Uint32T>(CodeStubAssembler(state_).UpdateWord32<base::BitField<Promise::PromiseState, 0, 2, uint32_t>>(ca_.UncheckedCast<Word32T>(tmp5), ca_.UncheckedCast<Uint32T>(tmp6), true));
    tmp8 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp9 = ca_.UncheckedCast<Uint32T>(CodeStubAssembler(state_).UpdateWord32<base::BitField<bool, 2, 1, uint32_t>>(ca_.UncheckedCast<Word32T>(tmp7), ca_.UncheckedCast<Uint32T>(tmp8), true));
    tmp10 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp11 = ca_.UncheckedCast<Uint32T>(CodeStubAssembler(state_).UpdateWord32<base::BitField<bool, 3, 1, uint32_t>>(ca_.UncheckedCast<Word32T>(tmp9), ca_.UncheckedCast<Uint32T>(tmp10), true));
    tmp12 = FromConstexpr_uint32_constexpr_uint32_0(state_, JSPromise::kInvalidAsyncTaskId);
    tmp13 = ca_.UncheckedCast<Uint32T>(CodeStubAssembler(state_).UpdateWord32<base::BitField<uint32_t, 4, 27, uint32_t>>(ca_.UncheckedCast<Word32T>(tmp11), ca_.UncheckedCast<Uint32T>(tmp12), true));
    tmp14 = SmiTag_JSPromiseFlags_0(state_, TNode<Uint32T>{tmp13});
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{p_promise, tmp2}, tmp14);
    PromiseBuiltinsAssembler(state_).ZeroOutEmbedderOffsets(TNode<JSPromise>{p_promise});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=59&c=1
TNode<JSPromise> InnerNewJSPromise_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<JSFunction> tmp3;
  TNode<Union<JSReceiver, Map, TheHole>> tmp4;
  TNode<Map> tmp5;
  TNode<HeapObject> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<Union<HeapObject, TaggedIndex>> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<JSPromise> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<FixedArray> tmp12;
  TNode<IntPtrT> tmp13;
  TNode<FixedArray> tmp14;
  TNode<IntPtrT> tmp15;
  TNode<Smi> tmp16;
  TNode<IntPtrT> tmp17;
  TNode<Int32T> tmp18;
  TNode<Uint32T> tmp19;
  TNode<Uint32T> tmp20;
  TNode<Int32T> tmp21;
  TNode<Uint32T> tmp22;
  TNode<BoolT> tmp23;
  TNode<Uint32T> tmp24;
  TNode<BoolT> tmp25;
  TNode<Uint32T> tmp26;
  TNode<Uint32T> tmp27;
  TNode<Uint32T> tmp28;
  TNode<Smi> tmp29;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = PROMISE_FUNCTION_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_JSFunction_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<JSFunction>(CodeStubAssembler::Reference{tmp1, tmp2});
    tmp4 = CodeStubAssembler(state_).LoadJSFunctionPrototypeOrInitialMap(TNode<JSFunction>{tmp3});
    tmp5 = UnsafeCast_Map_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp4});
    tmp6 = PromiseBuiltinsAssembler(state_).AllocateJSPromise(TNode<Context>{p_context});
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    std::tie(tmp8, tmp9) = UnsafeConstCast_Map_0(state_, TorqueStructReference_Map_0{TNode<Union<HeapObject, TaggedIndex>>{tmp6}, TNode<IntPtrT>{tmp7}, TorqueStructUnsafe_0{}}).Flatten();
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp8, tmp9}, tmp5);
    tmp10 = UnsafeCast_JSPromise_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp6});
    tmp11 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    tmp12 = kEmptyFixedArray_0(state_);
    CodeStubAssembler(state_).StoreReference<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>>(CodeStubAssembler::Reference{tmp10, tmp11}, tmp12);
    tmp13 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    tmp14 = kEmptyFixedArray_0(state_);
    CodeStubAssembler(state_).StoreReference<FixedArrayBase>(CodeStubAssembler::Reference{tmp10, tmp13}, tmp14);
    tmp15 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp16 = kZero_0(state_);
    CodeStubAssembler(state_).StoreReference<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, PromiseReaction, Smi, String, Symbol, Undefined>>(CodeStubAssembler::Reference{tmp10, tmp15}, tmp16);
    tmp17 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp18 = FromConstexpr_int32_constexpr_int32_0(state_, 0);
    tmp19 = CodeStubAssembler(state_).Unsigned(TNode<Int32T>{tmp18});
    tmp20 = (TNode<Uint32T>{tmp19});
    tmp21 = FromConstexpr_PromiseState_constexpr_kPending_0(state_, Promise::PromiseState::kPending);
    tmp22 = ca_.UncheckedCast<Uint32T>(CodeStubAssembler(state_).UpdateWord32<base::BitField<Promise::PromiseState, 0, 2, uint32_t>>(ca_.UncheckedCast<Word32T>(tmp20), ca_.UncheckedCast<Uint32T>(tmp21), true));
    tmp23 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp24 = ca_.UncheckedCast<Uint32T>(CodeStubAssembler(state_).UpdateWord32<base::BitField<bool, 2, 1, uint32_t>>(ca_.UncheckedCast<Word32T>(tmp22), ca_.UncheckedCast<Uint32T>(tmp23), true));
    tmp25 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp26 = ca_.UncheckedCast<Uint32T>(CodeStubAssembler(state_).UpdateWord32<base::BitField<bool, 3, 1, uint32_t>>(ca_.UncheckedCast<Word32T>(tmp24), ca_.UncheckedCast<Uint32T>(tmp25), true));
    tmp27 = FromConstexpr_uint32_constexpr_uint32_0(state_, JSPromise::kInvalidAsyncTaskId);
    tmp28 = ca_.UncheckedCast<Uint32T>(CodeStubAssembler(state_).UpdateWord32<base::BitField<uint32_t, 4, 27, uint32_t>>(ca_.UncheckedCast<Word32T>(tmp26), ca_.UncheckedCast<Uint32T>(tmp27), true));
    tmp29 = SmiTag_JSPromiseFlags_0(state_, TNode<Uint32T>{tmp28});
    CodeStubAssembler(state_).StoreReference<Smi>(CodeStubAssembler::Reference{tmp10, tmp17}, tmp29);
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<JSPromise>{tmp10};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=78&c=1
TNode<PromiseFulfillReactionJobTask> NewPromiseFulfillReactionJobTask_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Context> p_handlerContext, TNode<Object> p_argument, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_handler, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Map> tmp0;
  TNode<BoolT> tmp1;
  TNode<BoolT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<HeapObject> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<PromiseFulfillReactionJobTask> tmp10;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).PromiseFulfillReactionJobTaskMapConstant();
    tmp1 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp2 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp4 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp3}, TNode<Map>{tmp0}, TNode<BoolT>{tmp1}, TNode<BoolT>{tmp2});
    tmp5 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp4, tmp5}, tmp0);
    tmp6 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp4, tmp6}, p_argument);
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<Context>(CodeStubAssembler::Reference{tmp4, tmp7}, p_handlerContext);
    tmp8 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>>(CodeStubAssembler::Reference{tmp4, tmp8}, p_handler);
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<Union<JSPromise, PromiseCapability, Undefined>>(CodeStubAssembler::Reference{tmp4, tmp9}, p_promiseOrCapability);
    tmp10 = TORQUE_CAST(TNode<HeapObject>{tmp4});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<PromiseFulfillReactionJobTask>{tmp10};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=106&c=1
TNode<PromiseRejectReactionJobTask> NewPromiseRejectReactionJobTask_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Context> p_handlerContext, TNode<Object> p_argument, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_handler, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Map> tmp0;
  TNode<BoolT> tmp1;
  TNode<BoolT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<HeapObject> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<PromiseRejectReactionJobTask> tmp10;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).PromiseRejectReactionJobTaskMapConstant();
    tmp1 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp2 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp4 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp3}, TNode<Map>{tmp0}, TNode<BoolT>{tmp1}, TNode<BoolT>{tmp2});
    tmp5 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp4, tmp5}, tmp0);
    tmp6 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp4, tmp6}, p_argument);
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<Context>(CodeStubAssembler::Reference{tmp4, tmp7}, p_handlerContext);
    tmp8 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>>(CodeStubAssembler::Reference{tmp4, tmp8}, p_handler);
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<Union<JSPromise, PromiseCapability, Undefined>>(CodeStubAssembler::Reference{tmp4, tmp9}, p_promiseOrCapability);
    tmp10 = TORQUE_CAST(TNode<HeapObject>{tmp4});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<PromiseRejectReactionJobTask>{tmp10};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=134&c=1
void RunContextPromiseHookInit_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise, TNode<Object> p_parent) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<Union<JSPromise, Undefined>> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block16(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block17(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Union<HeapObject, TaggedIndex>> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> tmp3;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp4;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = PROMISE_HOOK_INIT_FUNCTION_INDEX_0(state_);
    std::tie(tmp1, tmp2) = NativeContextSlot_Context_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}).Flatten();
    tmp3 = CodeStubAssembler(state_).LoadReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>>(CodeStubAssembler::Reference{tmp1, tmp2});
    compiler::CodeAssemblerLabel label5(&ca_);
    tmp4 = Cast_Callable_0(state_, TNode<HeapObject>{tmp3}, &label5);
    ca_.Goto(&block4);
    if (label5.is_used()) {
      ca_.Bind(&label5);
      ca_.Goto(&block5);
    }
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block1);
  }

  TNode<BoolT> tmp6;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp6 = Is_JSPromise_Object_0(state_, TNode<Context>{p_context}, TNode<Object>{p_parent});
    ca_.Branch(tmp6, &block6, std::vector<compiler::Node*>{}, &block7, std::vector<compiler::Node*>{});
  }

  TNode<JSPromise> tmp7;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    compiler::CodeAssemblerLabel label8(&ca_);
    tmp7 = Cast_JSPromise_1(state_, TNode<Context>{p_context}, TNode<Object>{p_parent}, &label8);
    ca_.Goto(&block12);
    if (label8.is_used()) {
      ca_.Bind(&label8);
      ca_.Goto(&block13);
    }
  }

  if (block13.is_used()) {
    ca_.Bind(&block13);
    CodeStubAssembler(state_).Unreachable();
  }

  if (block12.is_used()) {
    ca_.Bind(&block12);
    ca_.Goto(&block8, tmp7);
  }

  TNode<Undefined> tmp9;
  if (block7.is_used()) {
    ca_.Bind(&block7);
    tmp9 = Undefined_0(state_);
    ca_.Goto(&block8, tmp9);
  }

  TNode<Union<JSPromise, Undefined>> phi_bb8_5;
  TNode<Undefined> tmp10;
  TNode<JSAny> tmp11;
      TNode<JSAny> tmp13;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_5);
    tmp10 = Undefined_0(state_);
    compiler::CodeAssemblerExceptionHandlerLabel catch12__label(&ca_, compiler::CodeAssemblerLabel::kDeferred);
    { compiler::ScopedExceptionHandler s(&ca_, &catch12__label);
    tmp11 = CodeStubAssembler(state_).Call(TNode<Context>{p_context}, TNode<JSAny>{tmp4}, TNode<JSAny>{tmp10}, TNode<JSAny>{p_promise}, TNode<JSAny>{phi_bb8_5});
    }
    if (catch12__label.is_used()) {
      compiler::CodeAssemblerLabel catch12_skip(&ca_);
      ca_.Goto(&catch12_skip);
      ca_.Bind(&catch12__label, &tmp13);
      ca_.Goto(&block16);
      ca_.Bind(&catch12_skip);
    }
    ca_.Goto(&block14);
  }

  TNode<Union<JSMessageObject, TheHole>> tmp14;
  TNode<JSAny> tmp15;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp14 = GetAndResetPendingMessage_0(state_);
    tmp15 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kReportMessageFromMicrotask, p_context, tmp13)); 
    ca_.Goto(&block14);
  }

  if (block14.is_used()) {
    ca_.Bind(&block14);
    ca_.Goto(&block1);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(&block17);
  }

    ca_.Bind(&block17);
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=150&c=1
void RunContextPromiseHookResolve_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise) {
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
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=162&c=1
void RunContextPromiseHookResolve_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise, TNode<Uint32T> p_flags) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = PROMISE_HOOK_RESOLVE_FUNCTION_INDEX_0(state_);
    RunContextPromiseHook_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}, TNode<Union<JSPromise, PromiseCapability, Undefined>>{p_promise}, TNode<Uint32T>{p_flags});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=169&c=1
void RunContextPromiseHookBefore_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability) {
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
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=182&c=1
void RunContextPromiseHookBefore_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability, TNode<Uint32T> p_flags) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = PROMISE_HOOK_BEFORE_FUNCTION_INDEX_0(state_);
    RunContextPromiseHook_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}, TNode<Union<JSPromise, PromiseCapability, Undefined>>{p_promiseOrCapability}, TNode<Uint32T>{p_flags});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=192&c=1
void RunContextPromiseHookAfter_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability) {
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
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=205&c=1
void RunContextPromiseHookAfter_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability, TNode<Uint32T> p_flags) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = PROMISE_HOOK_AFTER_FUNCTION_INDEX_0(state_);
    RunContextPromiseHook_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{tmp0}, TNode<Union<JSPromise, PromiseCapability, Undefined>>{p_promiseOrCapability}, TNode<Uint32T>{p_flags});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=215&c=1
void RunContextPromiseHook_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_slot, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability, TNode<Uint32T> p_flags) {
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
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=249&c=1
void RunAnyPromiseHookInit_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promise, TNode<Object> p_parent) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Uint32T> tmp0;
  TNode<Uint32T> tmp1;
  TNode<BoolT> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = PromiseBuiltinsAssembler(state_).PromiseHookFlags();
    tmp1 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp2 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp0}, TNode<Uint32T>{tmp1});
    ca_.Branch(tmp2, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  if (block2.is_used()) {
    ca_.Bind(&block2);
    ca_.Goto(&block1);
  }

  TNode<BoolT> tmp3;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp3 = PromiseBuiltinsAssembler(state_).IsIsolatePromiseHookEnabled(TNode<Uint32T>{tmp0});
    ca_.Branch(tmp3, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<JSAny> tmp4;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp4 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kPromiseHookInit, p_context, p_promise, p_parent)); 
    ca_.Goto(&block5);
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block1);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=269&c=1
TNode<JSPromise> NewJSPromise_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_parent) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSPromise> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = InnerNewJSPromise_0(state_, TNode<Context>{p_context});
    PromiseInit_0(state_, TNode<JSPromise>{tmp0});
    RunAnyPromiseHookInit_0(state_, TNode<Context>{p_context}, TNode<JSPromise>{tmp0}, TNode<Object>{p_parent});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSPromise>{tmp0};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=280&c=1
TNode<JSPromise> NewJSPromise_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Undefined> tmp0;
  TNode<JSPromise> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Undefined_0(state_);
    tmp1 = NewJSPromise_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp0});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSPromise>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=287&c=1
TNode<JSPromise> NewJSPromise_2(compiler::CodeAssemblerState* state_, TNode<Context> p_context, Promise::PromiseState p_status, TNode<JSAny> p_result) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSPromise> tmp0;
  TNode<IntPtrT> tmp1;
  TNode<Undefined> tmp2;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = InnerNewJSPromise_0(state_, TNode<Context>{p_context});
    tmp1 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, PromiseReaction, Smi, String, Symbol, Undefined>>(CodeStubAssembler::Reference{tmp0, tmp1}, p_result);
    Method_JSPromise_SetStatus_0(state_, TNode<JSPromise>{tmp0}, p_status);
    PromiseBuiltinsAssembler(state_).ZeroOutEmbedderOffsets(TNode<JSPromise>{tmp0});
    tmp2 = Undefined_0(state_);
    RunAnyPromiseHookInit_0(state_, TNode<Context>{p_context}, TNode<JSPromise>{tmp0}, TNode<Object>{tmp2});
    ca_.Goto(&block6);
  }

    ca_.Bind(&block6);
  return TNode<JSPromise>{tmp0};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=301&c=1
TNode<PromiseReaction> NewPromiseReaction_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<PromiseReaction, Smi>> p_next, TNode<Union<JSPromise, PromiseCapability, Undefined>> p_promiseOrCapability, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_fulfillHandler, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>> p_rejectHandler) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Map> tmp0;
  TNode<BoolT> tmp1;
  TNode<BoolT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<HeapObject> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<PromiseReaction> tmp10;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).PromiseReactionMapConstant();
    tmp1 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp2 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp4 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp3}, TNode<Map>{tmp0}, TNode<BoolT>{tmp1}, TNode<BoolT>{tmp2});
    tmp5 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp4, tmp5}, tmp0);
    tmp6 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Union<PromiseReaction, Smi>>(CodeStubAssembler::Reference{tmp4, tmp6}, p_next);
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>>(CodeStubAssembler::Reference{tmp4, tmp7}, p_rejectHandler);
    tmp8 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>>(CodeStubAssembler::Reference{tmp4, tmp8}, p_fulfillHandler);
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<Union<JSPromise, PromiseCapability, Undefined>>(CodeStubAssembler::Reference{tmp4, tmp9}, p_promiseOrCapability);
    tmp10 = TORQUE_CAST(TNode<HeapObject>{tmp4});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<PromiseReaction>{tmp10};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=332&c=1
TNode<PromiseResolveThenableJobTask> NewPromiseResolveThenableJobTask_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSPromise> p_promiseToResolve, TNode<JSReceiver> p_thenable, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_then) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Context> tmp0;
  TNode<NativeContext> tmp1;
  TNode<Map> tmp2;
  TNode<BoolT> tmp3;
  TNode<BoolT> tmp4;
  TNode<IntPtrT> tmp5;
  TNode<HeapObject> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<IntPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<PromiseResolveThenableJobTask> tmp12;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = ExtractHandlerContext_0(state_, TNode<Context>{p_context}, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>>{p_then});
    tmp1 = CodeStubAssembler(state_).LoadNativeContext(TNode<Context>{tmp0});
    tmp2 = CodeStubAssembler(state_).PromiseResolveThenableJobTaskMapConstant();
    tmp3 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp4 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    tmp5 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp6 = AllocateFromNew_0(state_, TNode<IntPtrT>{tmp5}, TNode<Map>{tmp2}, TNode<BoolT>{tmp3}, TNode<BoolT>{tmp4});
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    CodeStubAssembler(state_).StoreReference<Map>(CodeStubAssembler::Reference{tmp6, tmp7}, tmp2);
    tmp8 = FromConstexpr_intptr_constexpr_int31_0(state_, 4);
    CodeStubAssembler(state_).StoreReference<Context>(CodeStubAssembler::Reference{tmp6, tmp8}, tmp1);
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 8);
    CodeStubAssembler(state_).StoreReference<JSPromise>(CodeStubAssembler::Reference{tmp6, tmp9}, p_promiseToResolve);
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp6, tmp10}, p_thenable);
    tmp11 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    CodeStubAssembler(state_).StoreReference<JSReceiver>(CodeStubAssembler::Reference{tmp6, tmp11}, p_then);
    tmp12 = TORQUE_CAST(TNode<HeapObject>{tmp6});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<PromiseResolveThenableJobTask>{tmp12};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=409&c=1
TNode<JSAny> InvokeThen_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_receiver, TNode<JSAny> p_arg) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Undefined> tmp0;
  TNode<JSAny> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Undefined_0(state_);
    tmp1 = InvokeThen_InvokeThenOneArgFunctor_0(state_, TNode<Context>{p_context}, TNode<NativeContext>{p_nativeContext}, TNode<JSAny>{p_receiver}, TNode<JSAny>{p_arg}, TNode<JSAny>{tmp0}, TorqueStructInvokeThenOneArgFunctor_0{});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSAny>{tmp1};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=416&c=1
TNode<JSAny> InvokeThen_1(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_receiver, TNode<JSAny> p_arg1, TNode<JSAny> p_arg2) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSAny> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = InvokeThen_InvokeThenTwoArgFunctor_0(state_, TNode<Context>{p_context}, TNode<NativeContext>{p_nativeContext}, TNode<JSAny>{p_receiver}, TNode<JSAny>{p_arg1}, TNode<JSAny>{p_arg2}, TorqueStructInvokeThenTwoArgFunctor_0{});
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TNode<JSAny>{tmp0};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=423&c=1
void BranchIfAccessCheckFailed_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_promiseConstructor, TNode<JSAny> p_executor, compiler::CodeAssemblerLabel* label_IfNoAccess) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block9(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block7(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, JSAny, JSAny> block13(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, JSAny, JSAny> block12(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, JSAny> block14(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, JSAny> block15(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, JSAny> block20(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny, JSAny> block19(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block8(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block21(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block22(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block23(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  if (block0.is_used()) {
    ca_.Bind(&block0);
    ca_.Goto(&block9, p_executor);
  }

  TNode<JSAny> phi_bb9_4;
  TNode<BoolT> tmp0;
  if (block9.is_used()) {
    ca_.Bind(&block9, &phi_bb9_4);
    tmp0 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Branch(tmp0, &block7, std::vector<compiler::Node*>{phi_bb9_4}, &block8, std::vector<compiler::Node*>{phi_bb9_4});
  }

  TNode<JSAny> phi_bb7_4;
  TNode<JSFunction> tmp1;
  if (block7.is_used()) {
    ca_.Bind(&block7, &phi_bb7_4);
    compiler::CodeAssemblerLabel label2(&ca_);
    tmp1 = Cast_JSFunction_1(state_, TNode<Context>{p_context}, TNode<Object>{phi_bb7_4}, &label2);
    ca_.Goto(&block12, phi_bb7_4, phi_bb7_4, phi_bb7_4);
    if (label2.is_used()) {
      ca_.Bind(&label2);
      ca_.Goto(&block13, phi_bb7_4, phi_bb7_4, phi_bb7_4);
    }
  }

  TNode<JSAny> phi_bb13_4;
  TNode<JSAny> phi_bb13_5;
  TNode<JSAny> phi_bb13_6;
  TNode<JSBoundFunction> tmp3;
  if (block13.is_used()) {
    ca_.Bind(&block13, &phi_bb13_4, &phi_bb13_5, &phi_bb13_6);
    compiler::CodeAssemblerLabel label4(&ca_);
    tmp3 = Cast_JSBoundFunction_1(state_, TNode<Context>{p_context}, TNode<Object>{ca_.UncheckedCast<JSAny>(phi_bb13_5)}, &label4);
    ca_.Goto(&block19, phi_bb13_4, phi_bb13_5);
    if (label4.is_used()) {
      ca_.Bind(&label4);
      ca_.Goto(&block20, phi_bb13_4, phi_bb13_5);
    }
  }

  TNode<JSAny> phi_bb12_4;
  TNode<JSAny> phi_bb12_5;
  TNode<JSAny> phi_bb12_6;
  TNode<IntPtrT> tmp5;
  TNode<Context> tmp6;
  TNode<NativeContext> tmp7;
  TNode<BoolT> tmp8;
  if (block12.is_used()) {
    ca_.Bind(&block12, &phi_bb12_4, &phi_bb12_5, &phi_bb12_6);
    tmp5 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp6 = CodeStubAssembler(state_).LoadReference<Context>(CodeStubAssembler::Reference{tmp1, tmp5});
    tmp7 = CodeStubAssembler(state_).LoadNativeContext(TNode<Context>{tmp6});
    tmp8 = CodeStubAssembler(state_).TaggedEqual(TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WeakCell>>{p_nativeContext}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WeakCell>>{tmp7});
    ca_.Branch(tmp8, &block14, std::vector<compiler::Node*>{phi_bb12_4, phi_bb12_5}, &block15, std::vector<compiler::Node*>{phi_bb12_4, phi_bb12_5});
  }

  TNode<JSAny> phi_bb14_4;
  TNode<JSAny> phi_bb14_5;
  if (block14.is_used()) {
    ca_.Bind(&block14, &phi_bb14_4, &phi_bb14_5);
    ca_.Goto(&block3);
  }

  TNode<JSAny> phi_bb15_4;
  TNode<JSAny> phi_bb15_5;
  if (block15.is_used()) {
    ca_.Bind(&block15, &phi_bb15_4, &phi_bb15_5);
    ca_.Goto(&block6);
  }

  TNode<JSAny> phi_bb20_4;
  TNode<JSAny> phi_bb20_5;
  if (block20.is_used()) {
    ca_.Bind(&block20, &phi_bb20_4, &phi_bb20_5);
    ca_.Goto(&block6);
  }

  TNode<JSAny> phi_bb19_4;
  TNode<JSAny> phi_bb19_5;
  TNode<IntPtrT> tmp9;
  TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> tmp10;
  if (block19.is_used()) {
    ca_.Bind(&block19, &phi_bb19_4, &phi_bb19_5);
    tmp9 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp10 = CodeStubAssembler(state_).LoadReference<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>>(CodeStubAssembler::Reference{tmp3, tmp9});
    ca_.Goto(&block9, tmp10);
  }

  TNode<JSAny> phi_bb8_4;
  if (block8.is_used()) {
    ca_.Bind(&block8, &phi_bb8_4);
    ca_.Goto(&block5);
  }

  TNode<JSAny> tmp11;
  TNode<True> tmp12;
  TNode<BoolT> tmp13;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    tmp11 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kAllowDynamicFunction, p_context, p_promiseConstructor)); 
    tmp12 = True_0(state_);
    tmp13 = CodeStubAssembler(state_).TaggedNotEqual(TNode<Object>{tmp11}, TNode<Union<Context, FixedArrayBase, FunctionTemplateInfo, Hole, JSReceiver, Map, Oddball, String, Symbol, WeakCell>>{tmp12});
    ca_.Branch(tmp13, &block21, std::vector<compiler::Node*>{}, &block22, std::vector<compiler::Node*>{});
  }

  if (block21.is_used()) {
    ca_.Bind(&block21);
    ca_.Goto(label_IfNoAccess);
  }

  if (block22.is_used()) {
    ca_.Bind(&block22);
    ca_.Goto(&block5);
  }

  if (block5.is_used()) {
    ca_.Bind(&block5);
    ca_.Goto(&block3);
  }

  if (block3.is_used()) {
    ca_.Bind(&block3);
    ca_.Goto(&block23);
  }

    ca_.Bind(&block23);
}

TF_BUILTIN(AsyncAwaitNonThenableFastPath, CodeStubAssembler) {
  compiler::CodeAssemblerState* state_ = state();  compiler::CodeAssembler ca_(state());
  TNode<Context> parameter0 = UncheckedParameter<Context>(Descriptor::kContext);
  USE(parameter0);
  TNode<JSAny> parameter1 = UncheckedParameter<JSAny>(Descriptor::kValue);
  USE(parameter1);
  TNode<JSFunction> parameter2 = UncheckedParameter<JSFunction>(Descriptor::kOnResolve);
  USE(parameter2);
  TNode<Union<JSPromise, Undefined>> parameter3 = UncheckedParameter<Union<JSPromise, Undefined>>(Descriptor::kThrowaway);
  USE(parameter3);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<IntPtrT> tmp0;
  TNode<Context> tmp1;
  TNode<PromiseFulfillReactionJobTask> tmp2;
  TNode<Undefined> tmp3;
  TNode<Undefined> tmp4;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp1 = CodeStubAssembler(state_).LoadReference<Context>(CodeStubAssembler::Reference{parameter2, tmp0});
    tmp2 = NewPromiseFulfillReactionJobTask_0(state_, TNode<Context>{parameter0}, TNode<Context>{tmp1}, TNode<Object>{parameter1}, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction, Undefined>>{parameter2}, TNode<Union<JSPromise, PromiseCapability, Undefined>>{parameter3});
    tmp3 = ca_.CallBuiltin<Undefined>(Builtin::kEnqueueMicrotask, tmp1, tmp2);
    tmp4 = Undefined_0(state_);
    CodeStubAssembler(state_).Return(tmp4);
  }
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=50&c=19
TNode<Smi> SmiTag_JSPromiseFlags_0(compiler::CodeAssemblerState* state_, TNode<Uint32T> p_value) {
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

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=61&c=10
TNode<BoolT> Is_JSFunctionWithPrototype_JSFunction_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSFunctionWithPrototype> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSFunctionWithPrototype_0(state_, TNode<HeapObject>{p_o}, &label1);
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

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=64&c=4
TorqueStructReference_Map_0 UnsafeConstCast_Map_0(compiler::CodeAssemblerState* state_, TorqueStructReference_Map_0 p_r) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = (TorqueStructReference_Map_0{TNode<Union<HeapObject, TaggedIndex>>{p_r.object}, TNode<IntPtrT>{p_r.offset}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_Map_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=137&c=22
TorqueStructReference_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0 NativeContextSlot_Context_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<IntPtrT> p_index) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = NativeContextSlot_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0(state_, TNode<Context>{p_context}, TNode<IntPtrT>{p_index}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_JSFunction_OR_Undefined_OR_JSBoundFunction_OR_JSWrappedFunction_OR_CallableJSProxy_OR_CallableApiObject_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=140&c=24
TNode<BoolT> Is_JSPromise_Object_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Object> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<JSPromise> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_JSPromise_1(state_, TNode<Context>{p_context}, TNode<Object>{p_o}, &label1);
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

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=412&c=10
TNode<JSAny> InvokeThen_InvokeThenOneArgFunctor_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_receiver, TNode<JSAny> p_arg1, TNode<JSAny> p_arg2, TorqueStructInvokeThenOneArgFunctor_0 p_callFunctor) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Is_Smi_JSAny_0(state_, TNode<Context>{p_context}, TNode<JSAny>{p_receiver});
    tmp1 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp0});
    ca_.Branch(tmp1, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<HeapObject> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<Map> tmp4;
  TNode<BoolT> tmp5;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp2 = UnsafeCast_HeapObject_0(state_, TNode<Context>{p_context}, TNode<Object>{p_receiver});
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp4 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp2, tmp3});
    tmp5 = IsPromiseThenLookupChainIntact_0(state_, TNode<Context>{p_context}, TNode<NativeContext>{p_nativeContext}, TNode<Map>{tmp4});
    ca_.Goto(&block6, tmp5);
  }

  TNode<BoolT> tmp6;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp6 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block6, tmp6);
  }

  TNode<BoolT> phi_bb6_6;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_6);
    ca_.Branch(phi_bb6_6, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp7;
  TNode<Union<HeapObject, TaggedIndex>> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<JSFunction> tmp10;
  TNode<JSAny> tmp11;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp7 = PROMISE_THEN_INDEX_0(state_);
    std::tie(tmp8, tmp9) = NativeContextSlot_JSFunction_0(state_, TNode<NativeContext>{p_nativeContext}, TNode<IntPtrT>{tmp7}).Flatten();
    tmp10 = CodeStubAssembler(state_).LoadReference<JSFunction>(CodeStubAssembler::Reference{tmp8, tmp9});
    tmp11 = CodeStubAssembler(state_).Call(TNode<Context>{p_nativeContext}, TNode<JSAny>{tmp10}, TNode<JSAny>{p_receiver}, TNode<JSAny>{p_arg1});
    ca_.Goto(&block1, tmp11);
  }

  TNode<String> tmp12;
  TNode<JSAny> tmp13;
  TNode<JSAny> tmp14;
  TNode<JSAny> tmp15;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp12 = kThenString_0(state_);
    tmp13 = CodeStubAssembler(state_).GetProperty(TNode<Context>{p_context}, TNode<JSAny>{p_receiver}, TNode<JSAny>{tmp12});
    tmp14 = UnsafeCast_JSAny_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp13});
    tmp15 = CodeStubAssembler(state_).Call(TNode<Context>{p_nativeContext}, TNode<JSAny>{tmp14}, TNode<JSAny>{p_receiver}, TNode<JSAny>{p_arg1});
    ca_.Goto(&block1, tmp15);
  }

  TNode<JSAny> phi_bb1_5;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_5);
    ca_.Goto(&block10);
  }

    ca_.Bind(&block10);
  return TNode<JSAny>{phi_bb1_5};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=419&c=10
TNode<JSAny> InvokeThen_InvokeThenTwoArgFunctor_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<NativeContext> p_nativeContext, TNode<JSAny> p_receiver, TNode<JSAny> p_arg1, TNode<JSAny> p_arg2, TorqueStructInvokeThenTwoArgFunctor_0 p_callFunctor) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block3(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<JSAny> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block10(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<BoolT> tmp0;
  TNode<BoolT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = Is_Smi_JSAny_0(state_, TNode<Context>{p_context}, TNode<JSAny>{p_receiver});
    tmp1 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp0});
    ca_.Branch(tmp1, &block4, std::vector<compiler::Node*>{}, &block5, std::vector<compiler::Node*>{});
  }

  TNode<HeapObject> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<Map> tmp4;
  TNode<BoolT> tmp5;
  if (block4.is_used()) {
    ca_.Bind(&block4);
    tmp2 = UnsafeCast_HeapObject_0(state_, TNode<Context>{p_context}, TNode<Object>{p_receiver});
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, 0);
    tmp4 = CodeStubAssembler(state_).LoadReference<Map>(CodeStubAssembler::Reference{tmp2, tmp3});
    tmp5 = IsPromiseThenLookupChainIntact_0(state_, TNode<Context>{p_context}, TNode<NativeContext>{p_nativeContext}, TNode<Map>{tmp4});
    ca_.Goto(&block6, tmp5);
  }

  TNode<BoolT> tmp6;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    tmp6 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block6, tmp6);
  }

  TNode<BoolT> phi_bb6_6;
  if (block6.is_used()) {
    ca_.Bind(&block6, &phi_bb6_6);
    ca_.Branch(phi_bb6_6, &block2, std::vector<compiler::Node*>{}, &block3, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp7;
  TNode<Union<HeapObject, TaggedIndex>> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<JSFunction> tmp10;
  TNode<JSAny> tmp11;
  if (block2.is_used()) {
    ca_.Bind(&block2);
    tmp7 = PROMISE_THEN_INDEX_0(state_);
    std::tie(tmp8, tmp9) = NativeContextSlot_JSFunction_0(state_, TNode<NativeContext>{p_nativeContext}, TNode<IntPtrT>{tmp7}).Flatten();
    tmp10 = CodeStubAssembler(state_).LoadReference<JSFunction>(CodeStubAssembler::Reference{tmp8, tmp9});
    tmp11 = CodeStubAssembler(state_).Call(TNode<Context>{p_nativeContext}, TNode<JSAny>{tmp10}, TNode<JSAny>{p_receiver}, TNode<JSAny>{p_arg1}, TNode<JSAny>{p_arg2});
    ca_.Goto(&block1, tmp11);
  }

  TNode<String> tmp12;
  TNode<JSAny> tmp13;
  TNode<JSAny> tmp14;
  TNode<JSAny> tmp15;
  if (block3.is_used()) {
    ca_.Bind(&block3);
    tmp12 = kThenString_0(state_);
    tmp13 = CodeStubAssembler(state_).GetProperty(TNode<Context>{p_context}, TNode<JSAny>{p_receiver}, TNode<JSAny>{tmp12});
    tmp14 = UnsafeCast_JSAny_0(state_, TNode<Context>{p_context}, TNode<Object>{tmp13});
    tmp15 = CodeStubAssembler(state_).Call(TNode<Context>{p_nativeContext}, TNode<JSAny>{tmp14}, TNode<JSAny>{p_receiver}, TNode<JSAny>{p_arg1}, TNode<JSAny>{p_arg2});
    ca_.Goto(&block1, tmp15);
  }

  TNode<JSAny> phi_bb1_5;
  if (block1.is_used()) {
    ca_.Bind(&block1, &phi_bb1_5);
    ca_.Goto(&block10);
  }

    ca_.Bind(&block10);
  return TNode<JSAny>{phi_bb1_5};
}

// https://crsrc.org/c/v8/src/builtins/promise-misc.tq?l=396&c=8
TNode<BoolT> Is_Smi_JSAny_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSAny> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Smi> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_Smi_0(state_, TNode<Object>{p_o}, &label1);
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

} // namespace internal
} // namespace v8
