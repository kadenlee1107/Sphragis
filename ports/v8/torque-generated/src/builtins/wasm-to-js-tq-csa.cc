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
#include "torque-generated/src/builtins/wasm-to-js-tq-csa.h"
#include "torque-generated/src/wasm/wasm-objects-tq-csa.h"
#include "torque-generated/src/objects/arguments-tq-csa.h"
#include "torque-generated/src/objects/cell-tq-csa.h"
#include "torque-generated/src/objects/fixed-array-tq-csa.h"
#include "torque-generated/src/objects/contexts-tq-csa.h"
#include "torque-generated/src/builtins/convert-tq-csa.h"
#include "torque-generated/src/builtins/regexp-replace-tq-csa.h"
#include "torque-generated/src/builtins/torque-internal-tq-csa.h"
#include "torque-generated/src/builtins/cast-tq-csa.h"
#include "torque-generated/src/builtins/base-tq-csa.h"
#include "torque-generated/src/builtins/wasm-to-js-tq-csa.h"
#include "torque-generated/src/builtins/js-to-wasm-tq-csa.h"
#include "torque-generated/src/builtins/js-to-js-tq-csa.h"
#include "torque-generated/src/builtins/wasm-tq-csa.h"
#include "torque-generated/src/builtins/frames-tq-csa.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=28&c=1
void HandleF32Returns_0(compiler::CodeAssemblerState* state_, TNode<NativeContext> p_context, TorqueStructLocationAllocator_0 p_locationAllocator, TorqueStructReference_intptr_0 p_toRef, TNode<JSAny> p_retVal) {
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
  TNode<Float64T> tmp4;
  TNode<Float64T> tmp5;
  if (block5.is_used()) {
    ca_.Bind(&block5);
    std::tie(tmp2, tmp3) = RefCast_float64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{p_toRef.object}, TNode<IntPtrT>{p_toRef.offset}, TorqueStructUnsafe_0{}}).Flatten();
    tmp4 = CodeStubAssembler(state_).ChangeTaggedToFloat64(TNode<Context>{p_context}, TNode<JSAny>{p_retVal});
    tmp5 = CodeStubAssembler(state_).Float64SilenceNaN(TNode<Float64T>{tmp4});
    CodeStubAssembler(state_).StoreReference<Float64T>(CodeStubAssembler::Reference{tmp2, tmp3}, tmp5);
    ca_.Goto(&block8);
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<Float32T> tmp8;
  if (block6.is_used()) {
    ca_.Bind(&block6);
    std::tie(tmp6, tmp7) = RefCast_float32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{p_toRef.object}, TNode<IntPtrT>{p_toRef.offset}, TorqueStructUnsafe_0{}}).Flatten();
    tmp8 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_retVal);
    CodeStubAssembler(state_).StoreReference<Float32T>(CodeStubAssembler::Reference{tmp6, tmp7}, tmp8);
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

  TNode<Float32T> tmp9;
  TNode<Uint32T> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<IntPtrT> tmp13;
  if (block9.is_used()) {
    ca_.Bind(&block9);
    tmp9 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_retVal);
    tmp10 = Bitcast_uint32_float32_0(state_, TNode<Float32T>{tmp9});
    tmp11 = Convert_intptr_uint32_0(state_, TNode<Uint32T>{tmp10});
    tmp12 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x20ull));
    tmp13 = CodeStubAssembler(state_).WordShl(TNode<IntPtrT>{tmp11}, TNode<IntPtrT>{tmp12});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{p_toRef.object, p_toRef.offset}, tmp13);
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

  TNode<IntPtrT> tmp14;
  TNode<BoolT> tmp15;
  if (block12.is_used()) {
    ca_.Bind(&block12);
    tmp14 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp15 = CodeStubAssembler(state_).IntPtrGreaterThanOrEqual(TNode<IntPtrT>{p_locationAllocator.remainingFPRegs}, TNode<IntPtrT>{tmp14});
    ca_.Branch(tmp15, &block15, std::vector<compiler::Node*>{}, &block16, std::vector<compiler::Node*>{});
  }

  TNode<Float32T> tmp16;
  TNode<Uint32T> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<IntPtrT> tmp19;
  TNode<IntPtrT> tmp20;
  if (block15.is_used()) {
    ca_.Bind(&block15);
    tmp16 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_retVal);
    tmp17 = Bitcast_uint32_float32_0(state_, TNode<Float32T>{tmp16});
    tmp18 = Convert_intptr_uint32_0(state_, TNode<Uint32T>{tmp17});
    tmp19 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x20ull));
    tmp20 = CodeStubAssembler(state_).WordShl(TNode<IntPtrT>{tmp18}, TNode<IntPtrT>{tmp19});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{p_toRef.object, p_toRef.offset}, tmp20);
    ca_.Goto(&block18);
  }

  TNode<Float32T> tmp21;
  TNode<Uint32T> tmp22;
  TNode<IntPtrT> tmp23;
  if (block16.is_used()) {
    ca_.Bind(&block16);
    tmp21 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, p_context, p_retVal);
    tmp22 = Bitcast_uint32_float32_0(state_, TNode<Float32T>{tmp21});
    tmp23 = Convert_intptr_uint32_0(state_, TNode<Uint32T>{tmp22});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{p_toRef.object, p_toRef.offset}, tmp23);
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

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=50&c=1
TorqueStructWasmToJSResult WasmToJSWrapper_0(compiler::CodeAssemblerState* state_, TNode<WasmImportData> p_data) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block25(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block26(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block30(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block29(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block34(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block33(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block39(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block40(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block46(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block44(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block55(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block59(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block60(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block62(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block63(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block65(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block66(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block61(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block58(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block67(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block68(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Int32T> block69(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block74(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block75(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block56(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block78(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block82(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block83(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block85(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block86(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block88(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block89(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block84(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block81(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block90(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block93(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block94(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Float32T> block96(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block91(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block97(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block100(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block101(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Float32T> block103(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block98(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Float32T> block99(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Float32T> block92(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block108(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block109(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block79(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block112(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block115(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block119(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block120(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block122(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block123(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block125(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block126(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block121(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block118(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block131(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block132(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block116(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block136(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block137(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block139(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block140(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block142(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block143(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block138(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block135(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block145(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block146(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block148(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block149(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block151(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block152(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block147(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block144(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block157(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block158(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block117(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block113(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block161(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block165(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block166(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block167(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block171(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block172(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block174(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block175(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block170(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block168(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block164(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block180(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block181(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block162(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block185(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block184(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block163(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block114(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block80(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block57(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block45(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block186(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block189(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block190(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block194(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block192(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block203(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block206(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block207(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block209(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block210(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block212(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block213(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block208(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block205(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block218(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block219(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, BoolT> block204(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block193(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT> block187(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT> block222(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT> block223(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, FixedArray> block224(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT> block226(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT> block227(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT> block228(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT> block229(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block233(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block231(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block235(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block236(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block242(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block243(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT, JSAny> block237(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block253(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block257(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block258(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block260(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block261(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block263(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block264(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block259(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block256(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT, JSAny, JSAny> block268(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT, JSAny, JSAny> block267(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT, JSAny> block265(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block254(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block269(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block273(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block274(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block276(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block277(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block279(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block280(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block275(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block272(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block281(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block282(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block283(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block270(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block284(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block288(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block289(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block290(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block294(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block295(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block297(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block298(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block293(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block291(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block287(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block285(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block299(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block302(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block306(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block307(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block309(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block310(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block312(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block313(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block308(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block305(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block303(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block315(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block316(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block318(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block319(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block321(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block322(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block317(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block314(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block324(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block325(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block327(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block328(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block330(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block331(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block326(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT, Union<HeapObject, TaggedIndex>, IntPtrT> block323(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block304(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block300(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block333(&ca_, compiler::CodeAssemblerLabel::kDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block332(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block334(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block338(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block339(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block341(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block342(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block344(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block345(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block340(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny, Union<HeapObject, TaggedIndex>, IntPtrT> block337(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block335(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, JSAny, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block350(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, JSAny, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block351(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block336(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block301(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block286(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block271(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, JSAny> block255(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block232(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block354(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block357(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block358(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block362(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block360(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT> block371(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT> block374(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT> block375(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT> block377(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT> block378(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT> block380(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT> block381(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block376(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT> block373(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block386(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT, Union<HeapObject, TaggedIndex>, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT> block387(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, BoolT> block372(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, BoolT> block361(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT, IntPtrT, IntPtrT, BoolT> block355(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block390(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<RawPtrT> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<RawPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Union<HeapObject, TaggedIndex>> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  TNode<RawPtrT> tmp8;
  TNode<IntPtrT> tmp9;
  TNode<IntPtrT> tmp10;
  TNode<IntPtrT> tmp11;
  TNode<IntPtrT> tmp12;
  TNode<Cell> tmp13;
  TNode<Object> tmp14;
  TNode<Smi> tmp15;
  TNode<Smi> tmp16;
  TNode<Smi> tmp17;
  TNode<IntPtrT> tmp18;
  TNode<Cell> tmp19;
  TNode<Smi> tmp20;
  TNode<BoolT> tmp21;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = CodeStubAssembler(state_).SwitchToTheCentralStackIfNeeded();
    tmp1 = CodeStubAssembler(state_).LoadFramePointer();
    tmp2 = FromConstexpr_intptr_constexpr_intptr_0(state_, WasmToJSWrapperConstants::kSignatureOffset);
    tmp3 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{tmp1}, TNode<IntPtrT>{tmp2});
    tmp4 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp5, tmp6) = GetRefAt_RawPtr_RawPtr_0(state_, TNode<RawPtrT>{tmp3}, TNode<IntPtrT>{tmp4}).Flatten();
    tmp7 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    tmp8 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{p_data, tmp7});
    CodeStubAssembler(state_).StoreReference<RawPtrT>(CodeStubAssembler::Reference{tmp5, tmp6}, tmp8);
    tmp9 = CodeStubAssembler(state_).StackAlignmentInBytes();
    tmp10 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp11 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp9}, TNode<IntPtrT>{tmp10});
    tmp12 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp13 = CodeStubAssembler(state_).LoadReference<Cell>(CodeStubAssembler::Reference{p_data, tmp12});
    tmp14 = LoadCellValue_0(state_, TNode<Cell>{tmp13});
    tmp15 = TORQUE_CAST(TNode<Object>{tmp14});
    tmp16 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp17 = CodeStubAssembler(state_).SmiSub(TNode<Smi>{tmp15}, TNode<Smi>{tmp16});
    tmp18 = FromConstexpr_intptr_constexpr_int31_0(state_, 20);
    tmp19 = CodeStubAssembler(state_).LoadReference<Cell>(CodeStubAssembler::Reference{p_data, tmp18});
    StoreCellValue_0(state_, TNode<Cell>{tmp19}, TNode<Object>{tmp17});
    tmp20 = FromConstexpr_Smi_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp21 = CodeStubAssembler(state_).SmiEqual(TNode<Smi>{tmp17}, TNode<Smi>{tmp20});
    ca_.Branch(tmp21, &block25, std::vector<compiler::Node*>{}, &block26, std::vector<compiler::Node*>{});
  }

  TNode<Smi> tmp22;
  TNode<JSAny> tmp23;
  if (block25.is_used()) {
    ca_.Bind(&block25);
    tmp22 = kNoContext_0(state_);
    tmp23 = TORQUE_CAST(CodeStubAssembler(state_).CallRuntime(Runtime::kTierUpWasmToJSWrapper, tmp22, p_data)); 
    ca_.Goto(&block26);
  }

  TNode<IntPtrT> tmp24;
  TNode<RawPtrT> tmp25;
  TNode<IntPtrT> tmp26;
  TNode<RawPtrT> tmp27;
  TNode<RawPtrT> tmp28;
  TNode<Union<HeapObject, TaggedIndex>> tmp29;
  TNode<IntPtrT> tmp30;
  TNode<IntPtrT> tmp31;
  TNode<IntPtrT> tmp32;
  TNode<RawPtrT> tmp33;
  TNode<IntPtrT> tmp34;
  TNode<RawPtrT> tmp35;
  TNode<RawPtrT> tmp36;
  TNode<Union<HeapObject, TaggedIndex>> tmp37;
  TNode<IntPtrT> tmp38;
  TNode<IntPtrT> tmp39;
  TNode<IntPtrT> tmp40;
  TNode<RawPtrT> tmp41;
  TNode<IntPtrT> tmp42;
  TNode<RawPtrT> tmp43;
  TNode<RawPtrT> tmp44;
  TNode<Union<HeapObject, TaggedIndex>> tmp45;
  TNode<IntPtrT> tmp46;
  TNode<RawPtrT> tmp47;
  TNode<IntPtrT> tmp48;
  TNode<Union<HeapObject, TaggedIndex>> tmp49;
  TNode<IntPtrT> tmp50;
  TNode<IntPtrT> tmp51;
  TNode<IntPtrT> tmp52;
  TNode<Union<HeapObject, TaggedIndex>> tmp53;
  TNode<IntPtrT> tmp54;
  TNode<IntPtrT> tmp55;
  if (block26.is_used()) {
    ca_.Bind(&block26);
    tmp24 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    tmp25 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{p_data, tmp24});
    tmp26 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp27 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{tmp25}, TNode<IntPtrT>{tmp26});
    tmp28 = (TNode<RawPtrT>{tmp27});
    std::tie(tmp29, tmp30) = NewOffHeapReference_intptr_0(state_, TNode<RawPtrT>{tmp28}).Flatten();
    tmp31 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp29, tmp30});
    tmp32 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    tmp33 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{p_data, tmp32});
    tmp34 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp35 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{tmp33}, TNode<IntPtrT>{tmp34});
    tmp36 = (TNode<RawPtrT>{tmp35});
    std::tie(tmp37, tmp38) = NewOffHeapReference_intptr_0(state_, TNode<RawPtrT>{tmp36}).Flatten();
    tmp39 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp37, tmp38});
    tmp40 = FromConstexpr_intptr_constexpr_int31_0(state_, 24);
    tmp41 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{p_data, tmp40});
    tmp42 = FromConstexpr_intptr_constexpr_int31_0(state_, (CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull))), (SizeOf_intptr_0(state_)))));
    tmp43 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{tmp41}, TNode<IntPtrT>{tmp42});
    tmp44 = (TNode<RawPtrT>{tmp43});
    std::tie(tmp45, tmp46) = NewOffHeapReference_RawPtr_uint32_0(state_, TNode<RawPtrT>{tmp44}).Flatten();
    tmp47 = CodeStubAssembler(state_).LoadReference<RawPtrT>(CodeStubAssembler::Reference{tmp45, tmp46});
    tmp48 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp39}, TNode<IntPtrT>{tmp31});
    std::tie(tmp49, tmp50, tmp51) = NewOffHeapConstSlice_uint32_0(state_, TNode<RawPtrT>{tmp47}, TNode<IntPtrT>{tmp48}).Flatten();
    tmp52 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    compiler::CodeAssemblerLabel label56(&ca_);
    std::tie(tmp53, tmp54, tmp55) = Subslice_uint32_0(state_, TorqueStructSlice_uint32_ConstReference_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp49}, TNode<IntPtrT>{tmp50}, TNode<IntPtrT>{tmp51}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{tmp52}, TNode<IntPtrT>{tmp31}, &label56).Flatten();
    ca_.Goto(&block29);
    if (label56.is_used()) {
      ca_.Bind(&label56);
      ca_.Goto(&block30);
    }
  }

  if (block30.is_used()) {
    ca_.Bind(&block30);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<Union<HeapObject, TaggedIndex>> tmp57;
  TNode<IntPtrT> tmp58;
  TNode<IntPtrT> tmp59;
  if (block29.is_used()) {
    ca_.Bind(&block29);
    compiler::CodeAssemblerLabel label60(&ca_);
    std::tie(tmp57, tmp58, tmp59) = Subslice_uint32_0(state_, TorqueStructSlice_uint32_ConstReference_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp49}, TNode<IntPtrT>{tmp50}, TNode<IntPtrT>{tmp51}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{tmp31}, TNode<IntPtrT>{tmp39}, &label60).Flatten();
    ca_.Goto(&block33);
    if (label60.is_used()) {
      ca_.Bind(&label60);
      ca_.Goto(&block34);
    }
  }

  if (block34.is_used()) {
    ca_.Bind(&block34);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> tmp61;
  TNode<IntPtrT> tmp62;
  TNode<FixedArray> tmp63;
  TNode<IntPtrT> tmp64;
  TNode<Union<HeapObject, TaggedIndex>> tmp65;
  TNode<IntPtrT> tmp66;
  TNode<IntPtrT> tmp67;
  TNode<IntPtrT> tmp68;
  TNode<IntPtrT> tmp69;
  TNode<UintPtrT> tmp70;
  TNode<UintPtrT> tmp71;
  TNode<BoolT> tmp72;
  if (block33.is_used()) {
    ca_.Bind(&block33);
    tmp61 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp62 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp39}, TNode<IntPtrT>{tmp61});
    tmp63 = ca_.CallBuiltin<FixedArray>(Builtin::kWasmAllocateZeroedFixedArray, TNode<Object>(), tmp62);
    tmp64 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp65, tmp66, tmp67) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp63}).Flatten();
    tmp68 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp69 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp64}, TNode<IntPtrT>{tmp68});
    tmp70 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp64});
    tmp71 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp67});
    tmp72 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp70}, TNode<UintPtrT>{tmp71});
    ca_.Branch(tmp72, &block39, std::vector<compiler::Node*>{}, &block40, std::vector<compiler::Node*>{});
  }

  TNode<IntPtrT> tmp73;
  TNode<IntPtrT> tmp74;
  TNode<Union<HeapObject, TaggedIndex>> tmp75;
  TNode<IntPtrT> tmp76;
  TNode<Undefined> tmp77;
  TNode<RawPtrT> tmp78;
  TNode<IntPtrT> tmp79;
  TNode<IntPtrT> tmp80;
  TNode<IntPtrT> tmp81;
  TNode<IntPtrT> tmp82;
  TNode<RawPtrT> tmp83;
  TNode<RawPtrT> tmp84;
  TNode<Union<HeapObject, TaggedIndex>> tmp85;
  TNode<IntPtrT> tmp86;
  TNode<IntPtrT> tmp87;
  TNode<Union<HeapObject, TaggedIndex>> tmp88;
  TNode<IntPtrT> tmp89;
  TNode<IntPtrT> tmp90;
  TNode<IntPtrT> tmp91;
  TNode<IntPtrT> tmp92;
  TNode<IntPtrT> tmp93;
  TNode<IntPtrT> tmp94;
  TNode<IntPtrT> tmp95;
  TNode<IntPtrT> tmp96;
  TNode<BoolT> tmp97;
  TNode<IntPtrT> tmp98;
  TNode<IntPtrT> tmp99;
  TNode<BoolT> tmp100;
  if (block39.is_used()) {
    ca_.Bind(&block39);
    tmp73 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{tmp64});
    tmp74 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp66}, TNode<IntPtrT>{tmp73});
    std::tie(tmp75, tmp76) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp65}, TNode<IntPtrT>{tmp74}).Flatten();
    tmp77 = Undefined_0(state_);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp75, tmp76}, tmp77);
    tmp78 = CodeStubAssembler(state_).LoadFramePointer();
    tmp79 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull));
    tmp80 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp79}, TNode<IntPtrT>{tmp11});
    tmp81 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp82 = CodeStubAssembler(state_).IntPtrMul(TNode<IntPtrT>{tmp80}, TNode<IntPtrT>{tmp81});
    tmp83 = CodeStubAssembler(state_).RawPtrAdd(TNode<RawPtrT>{tmp78}, TNode<IntPtrT>{tmp82});
    tmp84 = (TNode<RawPtrT>{tmp83});
    std::tie(tmp85, tmp86) = NewOffHeapReference_intptr_0(state_, TNode<RawPtrT>{tmp84}).Flatten();
    tmp87 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp88, tmp89, tmp90, tmp91, tmp92, tmp93, tmp94, tmp95, tmp96, tmp97) = LocationAllocatorForParams_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp85}, TNode<IntPtrT>{tmp86}, TorqueStructUnsafe_0{}}, TNode<IntPtrT>{tmp87}).Flatten();
    tmp98 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{tmp59});
    tmp99 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp58}, TNode<IntPtrT>{tmp98});
    tmp100 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block46, tmp69, tmp89, tmp90, tmp91, tmp92, tmp93, tmp96, tmp97, tmp58, tmp100);
  }

  if (block40.is_used()) {
    ca_.Bind(&block40);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb46_20;
  TNode<IntPtrT> phi_bb46_25;
  TNode<IntPtrT> phi_bb46_26;
  TNode<IntPtrT> phi_bb46_27;
  TNode<IntPtrT> phi_bb46_28;
  TNode<IntPtrT> phi_bb46_29;
  TNode<IntPtrT> phi_bb46_32;
  TNode<BoolT> phi_bb46_33;
  TNode<IntPtrT> phi_bb46_35;
  TNode<BoolT> phi_bb46_37;
  TNode<BoolT> tmp101;
  TNode<BoolT> tmp102;
  if (block46.is_used()) {
    ca_.Bind(&block46, &phi_bb46_20, &phi_bb46_25, &phi_bb46_26, &phi_bb46_27, &phi_bb46_28, &phi_bb46_29, &phi_bb46_32, &phi_bb46_33, &phi_bb46_35, &phi_bb46_37);
    tmp101 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb46_35}, TNode<IntPtrT>{tmp99});
    tmp102 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp101});
    ca_.Branch(tmp102, &block44, std::vector<compiler::Node*>{phi_bb46_20, phi_bb46_25, phi_bb46_26, phi_bb46_27, phi_bb46_28, phi_bb46_29, phi_bb46_32, phi_bb46_33, phi_bb46_35, phi_bb46_37}, &block45, std::vector<compiler::Node*>{phi_bb46_20, phi_bb46_25, phi_bb46_26, phi_bb46_27, phi_bb46_28, phi_bb46_29, phi_bb46_32, phi_bb46_33, phi_bb46_35, phi_bb46_37});
  }

  TNode<IntPtrT> phi_bb44_20;
  TNode<IntPtrT> phi_bb44_25;
  TNode<IntPtrT> phi_bb44_26;
  TNode<IntPtrT> phi_bb44_27;
  TNode<IntPtrT> phi_bb44_28;
  TNode<IntPtrT> phi_bb44_29;
  TNode<IntPtrT> phi_bb44_32;
  TNode<BoolT> phi_bb44_33;
  TNode<IntPtrT> phi_bb44_35;
  TNode<BoolT> phi_bb44_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp103;
  TNode<IntPtrT> tmp104;
  TNode<IntPtrT> tmp105;
  TNode<IntPtrT> tmp106;
  TNode<Uint32T> tmp107;
  TNode<Uint32T> tmp108;
  TNode<BoolT> tmp109;
  if (block44.is_used()) {
    ca_.Bind(&block44, &phi_bb44_20, &phi_bb44_25, &phi_bb44_26, &phi_bb44_27, &phi_bb44_28, &phi_bb44_29, &phi_bb44_32, &phi_bb44_33, &phi_bb44_35, &phi_bb44_37);
    std::tie(tmp103, tmp104) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp57}, TNode<IntPtrT>{phi_bb44_35}).Flatten();
    tmp105 = FromConstexpr_intptr_constexpr_int31_0(state_, kInt32Size);
    tmp106 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb44_35}, TNode<IntPtrT>{tmp105});
    tmp107 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp103, tmp104});
    tmp108 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI32.raw_bit_field());
    tmp109 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp107}, TNode<Uint32T>{tmp108});
    ca_.Branch(tmp109, &block55, std::vector<compiler::Node*>{phi_bb44_20, phi_bb44_25, phi_bb44_26, phi_bb44_27, phi_bb44_28, phi_bb44_29, phi_bb44_32, phi_bb44_33, phi_bb44_37}, &block56, std::vector<compiler::Node*>{phi_bb44_20, phi_bb44_25, phi_bb44_26, phi_bb44_27, phi_bb44_28, phi_bb44_29, phi_bb44_32, phi_bb44_33, phi_bb44_37});
  }

  TNode<IntPtrT> phi_bb55_20;
  TNode<IntPtrT> phi_bb55_25;
  TNode<IntPtrT> phi_bb55_26;
  TNode<IntPtrT> phi_bb55_27;
  TNode<IntPtrT> phi_bb55_28;
  TNode<IntPtrT> phi_bb55_29;
  TNode<IntPtrT> phi_bb55_32;
  TNode<BoolT> phi_bb55_33;
  TNode<BoolT> phi_bb55_37;
  TNode<IntPtrT> tmp110;
  TNode<IntPtrT> tmp111;
  TNode<IntPtrT> tmp112;
  TNode<BoolT> tmp113;
  if (block55.is_used()) {
    ca_.Bind(&block55, &phi_bb55_20, &phi_bb55_25, &phi_bb55_26, &phi_bb55_27, &phi_bb55_28, &phi_bb55_29, &phi_bb55_32, &phi_bb55_33, &phi_bb55_37);
    tmp110 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp111 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb55_25}, TNode<IntPtrT>{tmp110});
    tmp112 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp113 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb55_25}, TNode<IntPtrT>{tmp112});
    ca_.Branch(tmp113, &block59, std::vector<compiler::Node*>{phi_bb55_20, phi_bb55_26, phi_bb55_27, phi_bb55_28, phi_bb55_29, phi_bb55_32, phi_bb55_33, phi_bb55_37}, &block60, std::vector<compiler::Node*>{phi_bb55_20, phi_bb55_26, phi_bb55_27, phi_bb55_28, phi_bb55_29, phi_bb55_32, phi_bb55_33, phi_bb55_37});
  }

  TNode<IntPtrT> phi_bb59_20;
  TNode<IntPtrT> phi_bb59_26;
  TNode<IntPtrT> phi_bb59_27;
  TNode<IntPtrT> phi_bb59_28;
  TNode<IntPtrT> phi_bb59_29;
  TNode<IntPtrT> phi_bb59_32;
  TNode<BoolT> phi_bb59_33;
  TNode<BoolT> phi_bb59_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp114;
  TNode<IntPtrT> tmp115;
  TNode<IntPtrT> tmp116;
  TNode<IntPtrT> tmp117;
  if (block59.is_used()) {
    ca_.Bind(&block59, &phi_bb59_20, &phi_bb59_26, &phi_bb59_27, &phi_bb59_28, &phi_bb59_29, &phi_bb59_32, &phi_bb59_33, &phi_bb59_37);
    std::tie(tmp114, tmp115) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb59_27}).Flatten();
    tmp116 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp117 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb59_27}, TNode<IntPtrT>{tmp116});
    ca_.Goto(&block58, phi_bb59_20, phi_bb59_26, tmp117, phi_bb59_28, phi_bb59_29, phi_bb59_32, phi_bb59_33, phi_bb59_37, tmp114, tmp115);
  }

  TNode<IntPtrT> phi_bb60_20;
  TNode<IntPtrT> phi_bb60_26;
  TNode<IntPtrT> phi_bb60_27;
  TNode<IntPtrT> phi_bb60_28;
  TNode<IntPtrT> phi_bb60_29;
  TNode<IntPtrT> phi_bb60_32;
  TNode<BoolT> phi_bb60_33;
  TNode<BoolT> phi_bb60_37;
  if (block60.is_used()) {
    ca_.Bind(&block60, &phi_bb60_20, &phi_bb60_26, &phi_bb60_27, &phi_bb60_28, &phi_bb60_29, &phi_bb60_32, &phi_bb60_33, &phi_bb60_37);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block62, phi_bb60_20, phi_bb60_26, phi_bb60_27, phi_bb60_28, phi_bb60_29, phi_bb60_32, phi_bb60_33, phi_bb60_37);
    } else {
      ca_.Goto(&block63, phi_bb60_20, phi_bb60_26, phi_bb60_27, phi_bb60_28, phi_bb60_29, phi_bb60_32, phi_bb60_33, phi_bb60_37);
    }
  }

  TNode<IntPtrT> phi_bb62_20;
  TNode<IntPtrT> phi_bb62_26;
  TNode<IntPtrT> phi_bb62_27;
  TNode<IntPtrT> phi_bb62_28;
  TNode<IntPtrT> phi_bb62_29;
  TNode<IntPtrT> phi_bb62_32;
  TNode<BoolT> phi_bb62_33;
  TNode<BoolT> phi_bb62_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp118;
  TNode<IntPtrT> tmp119;
  TNode<IntPtrT> tmp120;
  TNode<IntPtrT> tmp121;
  if (block62.is_used()) {
    ca_.Bind(&block62, &phi_bb62_20, &phi_bb62_26, &phi_bb62_27, &phi_bb62_28, &phi_bb62_29, &phi_bb62_32, &phi_bb62_33, &phi_bb62_37);
    std::tie(tmp118, tmp119) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb62_29}).Flatten();
    tmp120 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp121 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb62_29}, TNode<IntPtrT>{tmp120});
    ca_.Goto(&block61, phi_bb62_20, phi_bb62_26, phi_bb62_27, phi_bb62_28, tmp121, phi_bb62_32, phi_bb62_33, phi_bb62_37, tmp118, tmp119);
  }

  TNode<IntPtrT> phi_bb63_20;
  TNode<IntPtrT> phi_bb63_26;
  TNode<IntPtrT> phi_bb63_27;
  TNode<IntPtrT> phi_bb63_28;
  TNode<IntPtrT> phi_bb63_29;
  TNode<IntPtrT> phi_bb63_32;
  TNode<BoolT> phi_bb63_33;
  TNode<BoolT> phi_bb63_37;
  TNode<IntPtrT> tmp122;
  TNode<BoolT> tmp123;
  if (block63.is_used()) {
    ca_.Bind(&block63, &phi_bb63_20, &phi_bb63_26, &phi_bb63_27, &phi_bb63_28, &phi_bb63_29, &phi_bb63_32, &phi_bb63_33, &phi_bb63_37);
    tmp122 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp123 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb63_32}, TNode<IntPtrT>{tmp122});
    ca_.Branch(tmp123, &block65, std::vector<compiler::Node*>{phi_bb63_20, phi_bb63_26, phi_bb63_27, phi_bb63_28, phi_bb63_29, phi_bb63_32, phi_bb63_33, phi_bb63_37}, &block66, std::vector<compiler::Node*>{phi_bb63_20, phi_bb63_26, phi_bb63_27, phi_bb63_28, phi_bb63_29, phi_bb63_32, phi_bb63_33, phi_bb63_37});
  }

  TNode<IntPtrT> phi_bb65_20;
  TNode<IntPtrT> phi_bb65_26;
  TNode<IntPtrT> phi_bb65_27;
  TNode<IntPtrT> phi_bb65_28;
  TNode<IntPtrT> phi_bb65_29;
  TNode<IntPtrT> phi_bb65_32;
  TNode<BoolT> phi_bb65_33;
  TNode<BoolT> phi_bb65_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp124;
  TNode<IntPtrT> tmp125;
  TNode<IntPtrT> tmp126;
  TNode<BoolT> tmp127;
  if (block65.is_used()) {
    ca_.Bind(&block65, &phi_bb65_20, &phi_bb65_26, &phi_bb65_27, &phi_bb65_28, &phi_bb65_29, &phi_bb65_32, &phi_bb65_33, &phi_bb65_37);
    std::tie(tmp124, tmp125) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb65_32}).Flatten();
    tmp126 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp127 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block61, phi_bb65_20, phi_bb65_26, phi_bb65_27, phi_bb65_28, phi_bb65_29, tmp126, tmp127, phi_bb65_37, tmp124, tmp125);
  }

  TNode<IntPtrT> phi_bb66_20;
  TNode<IntPtrT> phi_bb66_26;
  TNode<IntPtrT> phi_bb66_27;
  TNode<IntPtrT> phi_bb66_28;
  TNode<IntPtrT> phi_bb66_29;
  TNode<IntPtrT> phi_bb66_32;
  TNode<BoolT> phi_bb66_33;
  TNode<BoolT> phi_bb66_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp128;
  TNode<IntPtrT> tmp129;
  TNode<IntPtrT> tmp130;
  TNode<IntPtrT> tmp131;
  TNode<IntPtrT> tmp132;
  TNode<IntPtrT> tmp133;
  TNode<BoolT> tmp134;
  if (block66.is_used()) {
    ca_.Bind(&block66, &phi_bb66_20, &phi_bb66_26, &phi_bb66_27, &phi_bb66_28, &phi_bb66_29, &phi_bb66_32, &phi_bb66_33, &phi_bb66_37);
    std::tie(tmp128, tmp129) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb66_29}).Flatten();
    tmp130 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp131 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb66_29}, TNode<IntPtrT>{tmp130});
    tmp132 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp133 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp131}, TNode<IntPtrT>{tmp132});
    tmp134 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block61, phi_bb66_20, phi_bb66_26, phi_bb66_27, phi_bb66_28, tmp133, tmp131, tmp134, phi_bb66_37, tmp128, tmp129);
  }

  TNode<IntPtrT> phi_bb61_20;
  TNode<IntPtrT> phi_bb61_26;
  TNode<IntPtrT> phi_bb61_27;
  TNode<IntPtrT> phi_bb61_28;
  TNode<IntPtrT> phi_bb61_29;
  TNode<IntPtrT> phi_bb61_32;
  TNode<BoolT> phi_bb61_33;
  TNode<BoolT> phi_bb61_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb61_39;
  TNode<IntPtrT> phi_bb61_40;
  if (block61.is_used()) {
    ca_.Bind(&block61, &phi_bb61_20, &phi_bb61_26, &phi_bb61_27, &phi_bb61_28, &phi_bb61_29, &phi_bb61_32, &phi_bb61_33, &phi_bb61_37, &phi_bb61_39, &phi_bb61_40);
    ca_.Goto(&block58, phi_bb61_20, phi_bb61_26, phi_bb61_27, phi_bb61_28, phi_bb61_29, phi_bb61_32, phi_bb61_33, phi_bb61_37, phi_bb61_39, phi_bb61_40);
  }

  TNode<IntPtrT> phi_bb58_20;
  TNode<IntPtrT> phi_bb58_26;
  TNode<IntPtrT> phi_bb58_27;
  TNode<IntPtrT> phi_bb58_28;
  TNode<IntPtrT> phi_bb58_29;
  TNode<IntPtrT> phi_bb58_32;
  TNode<BoolT> phi_bb58_33;
  TNode<BoolT> phi_bb58_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb58_39;
  TNode<IntPtrT> phi_bb58_40;
  if (block58.is_used()) {
    ca_.Bind(&block58, &phi_bb58_20, &phi_bb58_26, &phi_bb58_27, &phi_bb58_28, &phi_bb58_29, &phi_bb58_32, &phi_bb58_33, &phi_bb58_37, &phi_bb58_39, &phi_bb58_40);
    if ((wasm::kIsBigEndian)) {
      ca_.Goto(&block67, phi_bb58_20, phi_bb58_26, phi_bb58_27, phi_bb58_28, phi_bb58_29, phi_bb58_32, phi_bb58_33, phi_bb58_37, phi_bb58_39, phi_bb58_40);
    } else {
      ca_.Goto(&block68, phi_bb58_20, phi_bb58_26, phi_bb58_27, phi_bb58_28, phi_bb58_29, phi_bb58_32, phi_bb58_33, phi_bb58_37, phi_bb58_39, phi_bb58_40);
    }
  }

  TNode<IntPtrT> phi_bb67_20;
  TNode<IntPtrT> phi_bb67_26;
  TNode<IntPtrT> phi_bb67_27;
  TNode<IntPtrT> phi_bb67_28;
  TNode<IntPtrT> phi_bb67_29;
  TNode<IntPtrT> phi_bb67_32;
  TNode<BoolT> phi_bb67_33;
  TNode<BoolT> phi_bb67_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb67_39;
  TNode<IntPtrT> phi_bb67_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp135;
  TNode<IntPtrT> tmp136;
  TNode<Int64T> tmp137;
  TNode<Int32T> tmp138;
  if (block67.is_used()) {
    ca_.Bind(&block67, &phi_bb67_20, &phi_bb67_26, &phi_bb67_27, &phi_bb67_28, &phi_bb67_29, &phi_bb67_32, &phi_bb67_33, &phi_bb67_37, &phi_bb67_39, &phi_bb67_40);
    std::tie(tmp135, tmp136) = RefCast_int64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb67_39}, TNode<IntPtrT>{phi_bb67_40}, TorqueStructUnsafe_0{}}).Flatten();
    tmp137 = CodeStubAssembler(state_).LoadReference<Int64T>(CodeStubAssembler::Reference{tmp135, tmp136});
    tmp138 = CodeStubAssembler(state_).TruncateInt64ToInt32(TNode<Int64T>{tmp137});
    ca_.Goto(&block69, phi_bb67_20, phi_bb67_26, phi_bb67_27, phi_bb67_28, phi_bb67_29, phi_bb67_32, phi_bb67_33, phi_bb67_37, phi_bb67_39, phi_bb67_40, tmp138);
  }

  TNode<IntPtrT> phi_bb68_20;
  TNode<IntPtrT> phi_bb68_26;
  TNode<IntPtrT> phi_bb68_27;
  TNode<IntPtrT> phi_bb68_28;
  TNode<IntPtrT> phi_bb68_29;
  TNode<IntPtrT> phi_bb68_32;
  TNode<BoolT> phi_bb68_33;
  TNode<BoolT> phi_bb68_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb68_39;
  TNode<IntPtrT> phi_bb68_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp139;
  TNode<IntPtrT> tmp140;
  TNode<Int32T> tmp141;
  if (block68.is_used()) {
    ca_.Bind(&block68, &phi_bb68_20, &phi_bb68_26, &phi_bb68_27, &phi_bb68_28, &phi_bb68_29, &phi_bb68_32, &phi_bb68_33, &phi_bb68_37, &phi_bb68_39, &phi_bb68_40);
    std::tie(tmp139, tmp140) = RefCast_int32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb68_39}, TNode<IntPtrT>{phi_bb68_40}, TorqueStructUnsafe_0{}}).Flatten();
    tmp141 = CodeStubAssembler(state_).LoadReference<Int32T>(CodeStubAssembler::Reference{tmp139, tmp140});
    ca_.Goto(&block69, phi_bb68_20, phi_bb68_26, phi_bb68_27, phi_bb68_28, phi_bb68_29, phi_bb68_32, phi_bb68_33, phi_bb68_37, phi_bb68_39, phi_bb68_40, tmp141);
  }

  TNode<IntPtrT> phi_bb69_20;
  TNode<IntPtrT> phi_bb69_26;
  TNode<IntPtrT> phi_bb69_27;
  TNode<IntPtrT> phi_bb69_28;
  TNode<IntPtrT> phi_bb69_29;
  TNode<IntPtrT> phi_bb69_32;
  TNode<BoolT> phi_bb69_33;
  TNode<BoolT> phi_bb69_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb69_39;
  TNode<IntPtrT> phi_bb69_40;
  TNode<Int32T> phi_bb69_41;
  TNode<Union<HeapObject, TaggedIndex>> tmp142;
  TNode<IntPtrT> tmp143;
  TNode<IntPtrT> tmp144;
  TNode<IntPtrT> tmp145;
  TNode<IntPtrT> tmp146;
  TNode<UintPtrT> tmp147;
  TNode<UintPtrT> tmp148;
  TNode<BoolT> tmp149;
  if (block69.is_used()) {
    ca_.Bind(&block69, &phi_bb69_20, &phi_bb69_26, &phi_bb69_27, &phi_bb69_28, &phi_bb69_29, &phi_bb69_32, &phi_bb69_33, &phi_bb69_37, &phi_bb69_39, &phi_bb69_40, &phi_bb69_41);
    std::tie(tmp142, tmp143, tmp144) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp63}).Flatten();
    tmp145 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp146 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb69_20}, TNode<IntPtrT>{tmp145});
    tmp147 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb69_20});
    tmp148 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp144});
    tmp149 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp147}, TNode<UintPtrT>{tmp148});
    ca_.Branch(tmp149, &block74, std::vector<compiler::Node*>{phi_bb69_26, phi_bb69_27, phi_bb69_28, phi_bb69_29, phi_bb69_32, phi_bb69_33, phi_bb69_37, phi_bb69_39, phi_bb69_40, phi_bb69_20, phi_bb69_20, phi_bb69_20, phi_bb69_20}, &block75, std::vector<compiler::Node*>{phi_bb69_26, phi_bb69_27, phi_bb69_28, phi_bb69_29, phi_bb69_32, phi_bb69_33, phi_bb69_37, phi_bb69_39, phi_bb69_40, phi_bb69_20, phi_bb69_20, phi_bb69_20, phi_bb69_20});
  }

  TNode<IntPtrT> phi_bb74_26;
  TNode<IntPtrT> phi_bb74_27;
  TNode<IntPtrT> phi_bb74_28;
  TNode<IntPtrT> phi_bb74_29;
  TNode<IntPtrT> phi_bb74_32;
  TNode<BoolT> phi_bb74_33;
  TNode<BoolT> phi_bb74_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb74_39;
  TNode<IntPtrT> phi_bb74_40;
  TNode<IntPtrT> phi_bb74_46;
  TNode<IntPtrT> phi_bb74_47;
  TNode<IntPtrT> phi_bb74_51;
  TNode<IntPtrT> phi_bb74_52;
  TNode<IntPtrT> tmp150;
  TNode<IntPtrT> tmp151;
  TNode<Union<HeapObject, TaggedIndex>> tmp152;
  TNode<IntPtrT> tmp153;
  TNode<Number> tmp154;
  if (block74.is_used()) {
    ca_.Bind(&block74, &phi_bb74_26, &phi_bb74_27, &phi_bb74_28, &phi_bb74_29, &phi_bb74_32, &phi_bb74_33, &phi_bb74_37, &phi_bb74_39, &phi_bb74_40, &phi_bb74_46, &phi_bb74_47, &phi_bb74_51, &phi_bb74_52);
    tmp150 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb74_52});
    tmp151 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp143}, TNode<IntPtrT>{tmp150});
    std::tie(tmp152, tmp153) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp142}, TNode<IntPtrT>{tmp151}).Flatten();
    tmp154 = Convert_Number_int32_0(state_, TNode<Int32T>{phi_bb69_41});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp152, tmp153}, tmp154);
    ca_.Goto(&block57, tmp146, tmp111, phi_bb74_26, phi_bb74_27, phi_bb74_28, phi_bb74_29, phi_bb74_32, phi_bb74_33, phi_bb74_37);
  }

  TNode<IntPtrT> phi_bb75_26;
  TNode<IntPtrT> phi_bb75_27;
  TNode<IntPtrT> phi_bb75_28;
  TNode<IntPtrT> phi_bb75_29;
  TNode<IntPtrT> phi_bb75_32;
  TNode<BoolT> phi_bb75_33;
  TNode<BoolT> phi_bb75_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb75_39;
  TNode<IntPtrT> phi_bb75_40;
  TNode<IntPtrT> phi_bb75_46;
  TNode<IntPtrT> phi_bb75_47;
  TNode<IntPtrT> phi_bb75_51;
  TNode<IntPtrT> phi_bb75_52;
  if (block75.is_used()) {
    ca_.Bind(&block75, &phi_bb75_26, &phi_bb75_27, &phi_bb75_28, &phi_bb75_29, &phi_bb75_32, &phi_bb75_33, &phi_bb75_37, &phi_bb75_39, &phi_bb75_40, &phi_bb75_46, &phi_bb75_47, &phi_bb75_51, &phi_bb75_52);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb56_20;
  TNode<IntPtrT> phi_bb56_25;
  TNode<IntPtrT> phi_bb56_26;
  TNode<IntPtrT> phi_bb56_27;
  TNode<IntPtrT> phi_bb56_28;
  TNode<IntPtrT> phi_bb56_29;
  TNode<IntPtrT> phi_bb56_32;
  TNode<BoolT> phi_bb56_33;
  TNode<BoolT> phi_bb56_37;
  TNode<Uint32T> tmp155;
  TNode<BoolT> tmp156;
  if (block56.is_used()) {
    ca_.Bind(&block56, &phi_bb56_20, &phi_bb56_25, &phi_bb56_26, &phi_bb56_27, &phi_bb56_28, &phi_bb56_29, &phi_bb56_32, &phi_bb56_33, &phi_bb56_37);
    tmp155 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF32.raw_bit_field());
    tmp156 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp107}, TNode<Uint32T>{tmp155});
    ca_.Branch(tmp156, &block78, std::vector<compiler::Node*>{phi_bb56_20, phi_bb56_25, phi_bb56_26, phi_bb56_27, phi_bb56_28, phi_bb56_29, phi_bb56_32, phi_bb56_33, phi_bb56_37}, &block79, std::vector<compiler::Node*>{phi_bb56_20, phi_bb56_25, phi_bb56_26, phi_bb56_27, phi_bb56_28, phi_bb56_29, phi_bb56_32, phi_bb56_33, phi_bb56_37});
  }

  TNode<IntPtrT> phi_bb78_20;
  TNode<IntPtrT> phi_bb78_25;
  TNode<IntPtrT> phi_bb78_26;
  TNode<IntPtrT> phi_bb78_27;
  TNode<IntPtrT> phi_bb78_28;
  TNode<IntPtrT> phi_bb78_29;
  TNode<IntPtrT> phi_bb78_32;
  TNode<BoolT> phi_bb78_33;
  TNode<BoolT> phi_bb78_37;
  TNode<IntPtrT> tmp157;
  TNode<IntPtrT> tmp158;
  TNode<IntPtrT> tmp159;
  TNode<BoolT> tmp160;
  if (block78.is_used()) {
    ca_.Bind(&block78, &phi_bb78_20, &phi_bb78_25, &phi_bb78_26, &phi_bb78_27, &phi_bb78_28, &phi_bb78_29, &phi_bb78_32, &phi_bb78_33, &phi_bb78_37);
    tmp157 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp158 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb78_26}, TNode<IntPtrT>{tmp157});
    tmp159 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp160 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb78_26}, TNode<IntPtrT>{tmp159});
    ca_.Branch(tmp160, &block82, std::vector<compiler::Node*>{phi_bb78_20, phi_bb78_25, phi_bb78_27, phi_bb78_28, phi_bb78_29, phi_bb78_32, phi_bb78_33, phi_bb78_37}, &block83, std::vector<compiler::Node*>{phi_bb78_20, phi_bb78_25, phi_bb78_27, phi_bb78_28, phi_bb78_29, phi_bb78_32, phi_bb78_33, phi_bb78_37});
  }

  TNode<IntPtrT> phi_bb82_20;
  TNode<IntPtrT> phi_bb82_25;
  TNode<IntPtrT> phi_bb82_27;
  TNode<IntPtrT> phi_bb82_28;
  TNode<IntPtrT> phi_bb82_29;
  TNode<IntPtrT> phi_bb82_32;
  TNode<BoolT> phi_bb82_33;
  TNode<BoolT> phi_bb82_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp161;
  TNode<IntPtrT> tmp162;
  TNode<IntPtrT> tmp163;
  TNode<IntPtrT> tmp164;
  if (block82.is_used()) {
    ca_.Bind(&block82, &phi_bb82_20, &phi_bb82_25, &phi_bb82_27, &phi_bb82_28, &phi_bb82_29, &phi_bb82_32, &phi_bb82_33, &phi_bb82_37);
    std::tie(tmp161, tmp162) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb82_28}).Flatten();
    tmp163 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp164 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb82_28}, TNode<IntPtrT>{tmp163});
    ca_.Goto(&block81, phi_bb82_20, phi_bb82_25, phi_bb82_27, tmp164, phi_bb82_29, phi_bb82_32, phi_bb82_33, phi_bb82_37, tmp161, tmp162);
  }

  TNode<IntPtrT> phi_bb83_20;
  TNode<IntPtrT> phi_bb83_25;
  TNode<IntPtrT> phi_bb83_27;
  TNode<IntPtrT> phi_bb83_28;
  TNode<IntPtrT> phi_bb83_29;
  TNode<IntPtrT> phi_bb83_32;
  TNode<BoolT> phi_bb83_33;
  TNode<BoolT> phi_bb83_37;
  if (block83.is_used()) {
    ca_.Bind(&block83, &phi_bb83_20, &phi_bb83_25, &phi_bb83_27, &phi_bb83_28, &phi_bb83_29, &phi_bb83_32, &phi_bb83_33, &phi_bb83_37);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block85, phi_bb83_20, phi_bb83_25, phi_bb83_27, phi_bb83_28, phi_bb83_29, phi_bb83_32, phi_bb83_33, phi_bb83_37);
    } else {
      ca_.Goto(&block86, phi_bb83_20, phi_bb83_25, phi_bb83_27, phi_bb83_28, phi_bb83_29, phi_bb83_32, phi_bb83_33, phi_bb83_37);
    }
  }

  TNode<IntPtrT> phi_bb85_20;
  TNode<IntPtrT> phi_bb85_25;
  TNode<IntPtrT> phi_bb85_27;
  TNode<IntPtrT> phi_bb85_28;
  TNode<IntPtrT> phi_bb85_29;
  TNode<IntPtrT> phi_bb85_32;
  TNode<BoolT> phi_bb85_33;
  TNode<BoolT> phi_bb85_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp165;
  TNode<IntPtrT> tmp166;
  TNode<IntPtrT> tmp167;
  TNode<IntPtrT> tmp168;
  if (block85.is_used()) {
    ca_.Bind(&block85, &phi_bb85_20, &phi_bb85_25, &phi_bb85_27, &phi_bb85_28, &phi_bb85_29, &phi_bb85_32, &phi_bb85_33, &phi_bb85_37);
    std::tie(tmp165, tmp166) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb85_29}).Flatten();
    tmp167 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp168 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb85_29}, TNode<IntPtrT>{tmp167});
    ca_.Goto(&block84, phi_bb85_20, phi_bb85_25, phi_bb85_27, phi_bb85_28, tmp168, phi_bb85_32, phi_bb85_33, phi_bb85_37, tmp165, tmp166);
  }

  TNode<IntPtrT> phi_bb86_20;
  TNode<IntPtrT> phi_bb86_25;
  TNode<IntPtrT> phi_bb86_27;
  TNode<IntPtrT> phi_bb86_28;
  TNode<IntPtrT> phi_bb86_29;
  TNode<IntPtrT> phi_bb86_32;
  TNode<BoolT> phi_bb86_33;
  TNode<BoolT> phi_bb86_37;
  TNode<IntPtrT> tmp169;
  TNode<BoolT> tmp170;
  if (block86.is_used()) {
    ca_.Bind(&block86, &phi_bb86_20, &phi_bb86_25, &phi_bb86_27, &phi_bb86_28, &phi_bb86_29, &phi_bb86_32, &phi_bb86_33, &phi_bb86_37);
    tmp169 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp170 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb86_32}, TNode<IntPtrT>{tmp169});
    ca_.Branch(tmp170, &block88, std::vector<compiler::Node*>{phi_bb86_20, phi_bb86_25, phi_bb86_27, phi_bb86_28, phi_bb86_29, phi_bb86_32, phi_bb86_33, phi_bb86_37}, &block89, std::vector<compiler::Node*>{phi_bb86_20, phi_bb86_25, phi_bb86_27, phi_bb86_28, phi_bb86_29, phi_bb86_32, phi_bb86_33, phi_bb86_37});
  }

  TNode<IntPtrT> phi_bb88_20;
  TNode<IntPtrT> phi_bb88_25;
  TNode<IntPtrT> phi_bb88_27;
  TNode<IntPtrT> phi_bb88_28;
  TNode<IntPtrT> phi_bb88_29;
  TNode<IntPtrT> phi_bb88_32;
  TNode<BoolT> phi_bb88_33;
  TNode<BoolT> phi_bb88_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp171;
  TNode<IntPtrT> tmp172;
  TNode<IntPtrT> tmp173;
  TNode<BoolT> tmp174;
  if (block88.is_used()) {
    ca_.Bind(&block88, &phi_bb88_20, &phi_bb88_25, &phi_bb88_27, &phi_bb88_28, &phi_bb88_29, &phi_bb88_32, &phi_bb88_33, &phi_bb88_37);
    std::tie(tmp171, tmp172) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb88_32}).Flatten();
    tmp173 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp174 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block84, phi_bb88_20, phi_bb88_25, phi_bb88_27, phi_bb88_28, phi_bb88_29, tmp173, tmp174, phi_bb88_37, tmp171, tmp172);
  }

  TNode<IntPtrT> phi_bb89_20;
  TNode<IntPtrT> phi_bb89_25;
  TNode<IntPtrT> phi_bb89_27;
  TNode<IntPtrT> phi_bb89_28;
  TNode<IntPtrT> phi_bb89_29;
  TNode<IntPtrT> phi_bb89_32;
  TNode<BoolT> phi_bb89_33;
  TNode<BoolT> phi_bb89_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp175;
  TNode<IntPtrT> tmp176;
  TNode<IntPtrT> tmp177;
  TNode<IntPtrT> tmp178;
  TNode<IntPtrT> tmp179;
  TNode<IntPtrT> tmp180;
  TNode<BoolT> tmp181;
  if (block89.is_used()) {
    ca_.Bind(&block89, &phi_bb89_20, &phi_bb89_25, &phi_bb89_27, &phi_bb89_28, &phi_bb89_29, &phi_bb89_32, &phi_bb89_33, &phi_bb89_37);
    std::tie(tmp175, tmp176) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb89_29}).Flatten();
    tmp177 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp178 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb89_29}, TNode<IntPtrT>{tmp177});
    tmp179 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp180 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp178}, TNode<IntPtrT>{tmp179});
    tmp181 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block84, phi_bb89_20, phi_bb89_25, phi_bb89_27, phi_bb89_28, tmp180, tmp178, tmp181, phi_bb89_37, tmp175, tmp176);
  }

  TNode<IntPtrT> phi_bb84_20;
  TNode<IntPtrT> phi_bb84_25;
  TNode<IntPtrT> phi_bb84_27;
  TNode<IntPtrT> phi_bb84_28;
  TNode<IntPtrT> phi_bb84_29;
  TNode<IntPtrT> phi_bb84_32;
  TNode<BoolT> phi_bb84_33;
  TNode<BoolT> phi_bb84_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb84_39;
  TNode<IntPtrT> phi_bb84_40;
  if (block84.is_used()) {
    ca_.Bind(&block84, &phi_bb84_20, &phi_bb84_25, &phi_bb84_27, &phi_bb84_28, &phi_bb84_29, &phi_bb84_32, &phi_bb84_33, &phi_bb84_37, &phi_bb84_39, &phi_bb84_40);
    ca_.Goto(&block81, phi_bb84_20, phi_bb84_25, phi_bb84_27, phi_bb84_28, phi_bb84_29, phi_bb84_32, phi_bb84_33, phi_bb84_37, phi_bb84_39, phi_bb84_40);
  }

  TNode<IntPtrT> phi_bb81_20;
  TNode<IntPtrT> phi_bb81_25;
  TNode<IntPtrT> phi_bb81_27;
  TNode<IntPtrT> phi_bb81_28;
  TNode<IntPtrT> phi_bb81_29;
  TNode<IntPtrT> phi_bb81_32;
  TNode<BoolT> phi_bb81_33;
  TNode<BoolT> phi_bb81_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb81_39;
  TNode<IntPtrT> phi_bb81_40;
  if (block81.is_used()) {
    ca_.Bind(&block81, &phi_bb81_20, &phi_bb81_25, &phi_bb81_27, &phi_bb81_28, &phi_bb81_29, &phi_bb81_32, &phi_bb81_33, &phi_bb81_37, &phi_bb81_39, &phi_bb81_40);
    if ((wasm::kIsFpAlwaysDouble)) {
      ca_.Goto(&block90, phi_bb81_20, phi_bb81_25, phi_bb81_27, phi_bb81_28, phi_bb81_29, phi_bb81_32, phi_bb81_33, phi_bb81_37, phi_bb81_39, phi_bb81_40);
    } else {
      ca_.Goto(&block91, phi_bb81_20, phi_bb81_25, phi_bb81_27, phi_bb81_28, phi_bb81_29, phi_bb81_32, phi_bb81_33, phi_bb81_37, phi_bb81_39, phi_bb81_40);
    }
  }

  TNode<IntPtrT> phi_bb90_20;
  TNode<IntPtrT> phi_bb90_25;
  TNode<IntPtrT> phi_bb90_27;
  TNode<IntPtrT> phi_bb90_28;
  TNode<IntPtrT> phi_bb90_29;
  TNode<IntPtrT> phi_bb90_32;
  TNode<BoolT> phi_bb90_33;
  TNode<BoolT> phi_bb90_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb90_39;
  TNode<IntPtrT> phi_bb90_40;
  TNode<IntPtrT> tmp182;
  TNode<BoolT> tmp183;
  if (block90.is_used()) {
    ca_.Bind(&block90, &phi_bb90_20, &phi_bb90_25, &phi_bb90_27, &phi_bb90_28, &phi_bb90_29, &phi_bb90_32, &phi_bb90_33, &phi_bb90_37, &phi_bb90_39, &phi_bb90_40);
    tmp182 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp183 = CodeStubAssembler(state_).IntPtrGreaterThanOrEqual(TNode<IntPtrT>{tmp158}, TNode<IntPtrT>{tmp182});
    ca_.Branch(tmp183, &block93, std::vector<compiler::Node*>{phi_bb90_20, phi_bb90_25, phi_bb90_27, phi_bb90_28, phi_bb90_29, phi_bb90_32, phi_bb90_33, phi_bb90_37, phi_bb90_39, phi_bb90_40}, &block94, std::vector<compiler::Node*>{phi_bb90_20, phi_bb90_25, phi_bb90_27, phi_bb90_28, phi_bb90_29, phi_bb90_32, phi_bb90_33, phi_bb90_37, phi_bb90_39, phi_bb90_40});
  }

  TNode<IntPtrT> phi_bb93_20;
  TNode<IntPtrT> phi_bb93_25;
  TNode<IntPtrT> phi_bb93_27;
  TNode<IntPtrT> phi_bb93_28;
  TNode<IntPtrT> phi_bb93_29;
  TNode<IntPtrT> phi_bb93_32;
  TNode<BoolT> phi_bb93_33;
  TNode<BoolT> phi_bb93_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb93_39;
  TNode<IntPtrT> phi_bb93_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp184;
  TNode<IntPtrT> tmp185;
  TNode<Float64T> tmp186;
  TNode<Float32T> tmp187;
  if (block93.is_used()) {
    ca_.Bind(&block93, &phi_bb93_20, &phi_bb93_25, &phi_bb93_27, &phi_bb93_28, &phi_bb93_29, &phi_bb93_32, &phi_bb93_33, &phi_bb93_37, &phi_bb93_39, &phi_bb93_40);
    std::tie(tmp184, tmp185) = RefCast_float64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb93_39}, TNode<IntPtrT>{phi_bb93_40}, TorqueStructUnsafe_0{}}).Flatten();
    tmp186 = CodeStubAssembler(state_).LoadReference<Float64T>(CodeStubAssembler::Reference{tmp184, tmp185});
    tmp187 = CodeStubAssembler(state_).TruncateFloat64ToFloat32(TNode<Float64T>{tmp186});
    ca_.Goto(&block96, phi_bb93_20, phi_bb93_25, phi_bb93_27, phi_bb93_28, phi_bb93_29, phi_bb93_32, phi_bb93_33, phi_bb93_37, phi_bb93_39, phi_bb93_40, tmp187);
  }

  TNode<IntPtrT> phi_bb94_20;
  TNode<IntPtrT> phi_bb94_25;
  TNode<IntPtrT> phi_bb94_27;
  TNode<IntPtrT> phi_bb94_28;
  TNode<IntPtrT> phi_bb94_29;
  TNode<IntPtrT> phi_bb94_32;
  TNode<BoolT> phi_bb94_33;
  TNode<BoolT> phi_bb94_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb94_39;
  TNode<IntPtrT> phi_bb94_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp188;
  TNode<IntPtrT> tmp189;
  TNode<Float32T> tmp190;
  if (block94.is_used()) {
    ca_.Bind(&block94, &phi_bb94_20, &phi_bb94_25, &phi_bb94_27, &phi_bb94_28, &phi_bb94_29, &phi_bb94_32, &phi_bb94_33, &phi_bb94_37, &phi_bb94_39, &phi_bb94_40);
    std::tie(tmp188, tmp189) = RefCast_float32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb94_39}, TNode<IntPtrT>{phi_bb94_40}, TorqueStructUnsafe_0{}}).Flatten();
    tmp190 = CodeStubAssembler(state_).LoadReference<Float32T>(CodeStubAssembler::Reference{tmp188, tmp189});
    ca_.Goto(&block96, phi_bb94_20, phi_bb94_25, phi_bb94_27, phi_bb94_28, phi_bb94_29, phi_bb94_32, phi_bb94_33, phi_bb94_37, phi_bb94_39, phi_bb94_40, tmp190);
  }

  TNode<IntPtrT> phi_bb96_20;
  TNode<IntPtrT> phi_bb96_25;
  TNode<IntPtrT> phi_bb96_27;
  TNode<IntPtrT> phi_bb96_28;
  TNode<IntPtrT> phi_bb96_29;
  TNode<IntPtrT> phi_bb96_32;
  TNode<BoolT> phi_bb96_33;
  TNode<BoolT> phi_bb96_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb96_39;
  TNode<IntPtrT> phi_bb96_40;
  TNode<Float32T> phi_bb96_41;
  if (block96.is_used()) {
    ca_.Bind(&block96, &phi_bb96_20, &phi_bb96_25, &phi_bb96_27, &phi_bb96_28, &phi_bb96_29, &phi_bb96_32, &phi_bb96_33, &phi_bb96_37, &phi_bb96_39, &phi_bb96_40, &phi_bb96_41);
    ca_.Goto(&block92, phi_bb96_20, phi_bb96_25, phi_bb96_27, phi_bb96_28, phi_bb96_29, phi_bb96_32, phi_bb96_33, phi_bb96_37, phi_bb96_39, phi_bb96_40, phi_bb96_41);
  }

  TNode<IntPtrT> phi_bb91_20;
  TNode<IntPtrT> phi_bb91_25;
  TNode<IntPtrT> phi_bb91_27;
  TNode<IntPtrT> phi_bb91_28;
  TNode<IntPtrT> phi_bb91_29;
  TNode<IntPtrT> phi_bb91_32;
  TNode<BoolT> phi_bb91_33;
  TNode<BoolT> phi_bb91_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb91_39;
  TNode<IntPtrT> phi_bb91_40;
  if (block91.is_used()) {
    ca_.Bind(&block91, &phi_bb91_20, &phi_bb91_25, &phi_bb91_27, &phi_bb91_28, &phi_bb91_29, &phi_bb91_32, &phi_bb91_33, &phi_bb91_37, &phi_bb91_39, &phi_bb91_40);
    if ((wasm::kIsBigEndianOnSim)) {
      ca_.Goto(&block97, phi_bb91_20, phi_bb91_25, phi_bb91_27, phi_bb91_28, phi_bb91_29, phi_bb91_32, phi_bb91_33, phi_bb91_37, phi_bb91_39, phi_bb91_40);
    } else {
      ca_.Goto(&block98, phi_bb91_20, phi_bb91_25, phi_bb91_27, phi_bb91_28, phi_bb91_29, phi_bb91_32, phi_bb91_33, phi_bb91_37, phi_bb91_39, phi_bb91_40);
    }
  }

  TNode<IntPtrT> phi_bb97_20;
  TNode<IntPtrT> phi_bb97_25;
  TNode<IntPtrT> phi_bb97_27;
  TNode<IntPtrT> phi_bb97_28;
  TNode<IntPtrT> phi_bb97_29;
  TNode<IntPtrT> phi_bb97_32;
  TNode<BoolT> phi_bb97_33;
  TNode<BoolT> phi_bb97_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb97_39;
  TNode<IntPtrT> phi_bb97_40;
  TNode<IntPtrT> tmp191;
  TNode<BoolT> tmp192;
  if (block97.is_used()) {
    ca_.Bind(&block97, &phi_bb97_20, &phi_bb97_25, &phi_bb97_27, &phi_bb97_28, &phi_bb97_29, &phi_bb97_32, &phi_bb97_33, &phi_bb97_37, &phi_bb97_39, &phi_bb97_40);
    tmp191 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp192 = CodeStubAssembler(state_).IntPtrGreaterThanOrEqual(TNode<IntPtrT>{tmp158}, TNode<IntPtrT>{tmp191});
    ca_.Branch(tmp192, &block100, std::vector<compiler::Node*>{phi_bb97_20, phi_bb97_25, phi_bb97_27, phi_bb97_28, phi_bb97_29, phi_bb97_32, phi_bb97_33, phi_bb97_37, phi_bb97_39, phi_bb97_40}, &block101, std::vector<compiler::Node*>{phi_bb97_20, phi_bb97_25, phi_bb97_27, phi_bb97_28, phi_bb97_29, phi_bb97_32, phi_bb97_33, phi_bb97_37, phi_bb97_39, phi_bb97_40});
  }

  TNode<IntPtrT> phi_bb100_20;
  TNode<IntPtrT> phi_bb100_25;
  TNode<IntPtrT> phi_bb100_27;
  TNode<IntPtrT> phi_bb100_28;
  TNode<IntPtrT> phi_bb100_29;
  TNode<IntPtrT> phi_bb100_32;
  TNode<BoolT> phi_bb100_33;
  TNode<BoolT> phi_bb100_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb100_39;
  TNode<IntPtrT> phi_bb100_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp193;
  TNode<IntPtrT> tmp194;
  TNode<Int64T> tmp195;
  TNode<Int64T> tmp196;
  TNode<Int64T> tmp197;
  TNode<Int32T> tmp198;
  TNode<Float32T> tmp199;
  if (block100.is_used()) {
    ca_.Bind(&block100, &phi_bb100_20, &phi_bb100_25, &phi_bb100_27, &phi_bb100_28, &phi_bb100_29, &phi_bb100_32, &phi_bb100_33, &phi_bb100_37, &phi_bb100_39, &phi_bb100_40);
    std::tie(tmp193, tmp194) = RefCast_int64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb100_39}, TNode<IntPtrT>{phi_bb100_40}, TorqueStructUnsafe_0{}}).Flatten();
    tmp195 = CodeStubAssembler(state_).LoadReference<Int64T>(CodeStubAssembler::Reference{tmp193, tmp194});
    tmp196 = FromConstexpr_int64_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x20ull));
    tmp197 = CodeStubAssembler(state_).Word64Sar(TNode<Int64T>{tmp195}, TNode<Int64T>{tmp196});
    tmp198 = CodeStubAssembler(state_).TruncateInt64ToInt32(TNode<Int64T>{tmp197});
    tmp199 = CodeStubAssembler(state_).BitcastInt32ToFloat32(TNode<Int32T>{tmp198});
    ca_.Goto(&block103, phi_bb100_20, phi_bb100_25, phi_bb100_27, phi_bb100_28, phi_bb100_29, phi_bb100_32, phi_bb100_33, phi_bb100_37, phi_bb100_39, phi_bb100_40, tmp199);
  }

  TNode<IntPtrT> phi_bb101_20;
  TNode<IntPtrT> phi_bb101_25;
  TNode<IntPtrT> phi_bb101_27;
  TNode<IntPtrT> phi_bb101_28;
  TNode<IntPtrT> phi_bb101_29;
  TNode<IntPtrT> phi_bb101_32;
  TNode<BoolT> phi_bb101_33;
  TNode<BoolT> phi_bb101_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb101_39;
  TNode<IntPtrT> phi_bb101_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp200;
  TNode<IntPtrT> tmp201;
  TNode<Float32T> tmp202;
  if (block101.is_used()) {
    ca_.Bind(&block101, &phi_bb101_20, &phi_bb101_25, &phi_bb101_27, &phi_bb101_28, &phi_bb101_29, &phi_bb101_32, &phi_bb101_33, &phi_bb101_37, &phi_bb101_39, &phi_bb101_40);
    std::tie(tmp200, tmp201) = RefCast_float32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb101_39}, TNode<IntPtrT>{phi_bb101_40}, TorqueStructUnsafe_0{}}).Flatten();
    tmp202 = CodeStubAssembler(state_).LoadReference<Float32T>(CodeStubAssembler::Reference{tmp200, tmp201});
    ca_.Goto(&block103, phi_bb101_20, phi_bb101_25, phi_bb101_27, phi_bb101_28, phi_bb101_29, phi_bb101_32, phi_bb101_33, phi_bb101_37, phi_bb101_39, phi_bb101_40, tmp202);
  }

  TNode<IntPtrT> phi_bb103_20;
  TNode<IntPtrT> phi_bb103_25;
  TNode<IntPtrT> phi_bb103_27;
  TNode<IntPtrT> phi_bb103_28;
  TNode<IntPtrT> phi_bb103_29;
  TNode<IntPtrT> phi_bb103_32;
  TNode<BoolT> phi_bb103_33;
  TNode<BoolT> phi_bb103_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb103_39;
  TNode<IntPtrT> phi_bb103_40;
  TNode<Float32T> phi_bb103_41;
  if (block103.is_used()) {
    ca_.Bind(&block103, &phi_bb103_20, &phi_bb103_25, &phi_bb103_27, &phi_bb103_28, &phi_bb103_29, &phi_bb103_32, &phi_bb103_33, &phi_bb103_37, &phi_bb103_39, &phi_bb103_40, &phi_bb103_41);
    ca_.Goto(&block99, phi_bb103_20, phi_bb103_25, phi_bb103_27, phi_bb103_28, phi_bb103_29, phi_bb103_32, phi_bb103_33, phi_bb103_37, phi_bb103_39, phi_bb103_40, phi_bb103_41);
  }

  TNode<IntPtrT> phi_bb98_20;
  TNode<IntPtrT> phi_bb98_25;
  TNode<IntPtrT> phi_bb98_27;
  TNode<IntPtrT> phi_bb98_28;
  TNode<IntPtrT> phi_bb98_29;
  TNode<IntPtrT> phi_bb98_32;
  TNode<BoolT> phi_bb98_33;
  TNode<BoolT> phi_bb98_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb98_39;
  TNode<IntPtrT> phi_bb98_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp203;
  TNode<IntPtrT> tmp204;
  TNode<Float32T> tmp205;
  if (block98.is_used()) {
    ca_.Bind(&block98, &phi_bb98_20, &phi_bb98_25, &phi_bb98_27, &phi_bb98_28, &phi_bb98_29, &phi_bb98_32, &phi_bb98_33, &phi_bb98_37, &phi_bb98_39, &phi_bb98_40);
    std::tie(tmp203, tmp204) = RefCast_float32_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb98_39}, TNode<IntPtrT>{phi_bb98_40}, TorqueStructUnsafe_0{}}).Flatten();
    tmp205 = CodeStubAssembler(state_).LoadReference<Float32T>(CodeStubAssembler::Reference{tmp203, tmp204});
    ca_.Goto(&block99, phi_bb98_20, phi_bb98_25, phi_bb98_27, phi_bb98_28, phi_bb98_29, phi_bb98_32, phi_bb98_33, phi_bb98_37, phi_bb98_39, phi_bb98_40, tmp205);
  }

  TNode<IntPtrT> phi_bb99_20;
  TNode<IntPtrT> phi_bb99_25;
  TNode<IntPtrT> phi_bb99_27;
  TNode<IntPtrT> phi_bb99_28;
  TNode<IntPtrT> phi_bb99_29;
  TNode<IntPtrT> phi_bb99_32;
  TNode<BoolT> phi_bb99_33;
  TNode<BoolT> phi_bb99_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb99_39;
  TNode<IntPtrT> phi_bb99_40;
  TNode<Float32T> phi_bb99_41;
  if (block99.is_used()) {
    ca_.Bind(&block99, &phi_bb99_20, &phi_bb99_25, &phi_bb99_27, &phi_bb99_28, &phi_bb99_29, &phi_bb99_32, &phi_bb99_33, &phi_bb99_37, &phi_bb99_39, &phi_bb99_40, &phi_bb99_41);
    ca_.Goto(&block92, phi_bb99_20, phi_bb99_25, phi_bb99_27, phi_bb99_28, phi_bb99_29, phi_bb99_32, phi_bb99_33, phi_bb99_37, phi_bb99_39, phi_bb99_40, phi_bb99_41);
  }

  TNode<IntPtrT> phi_bb92_20;
  TNode<IntPtrT> phi_bb92_25;
  TNode<IntPtrT> phi_bb92_27;
  TNode<IntPtrT> phi_bb92_28;
  TNode<IntPtrT> phi_bb92_29;
  TNode<IntPtrT> phi_bb92_32;
  TNode<BoolT> phi_bb92_33;
  TNode<BoolT> phi_bb92_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb92_39;
  TNode<IntPtrT> phi_bb92_40;
  TNode<Float32T> phi_bb92_41;
  TNode<Union<HeapObject, TaggedIndex>> tmp206;
  TNode<IntPtrT> tmp207;
  TNode<IntPtrT> tmp208;
  TNode<IntPtrT> tmp209;
  TNode<IntPtrT> tmp210;
  TNode<UintPtrT> tmp211;
  TNode<UintPtrT> tmp212;
  TNode<BoolT> tmp213;
  if (block92.is_used()) {
    ca_.Bind(&block92, &phi_bb92_20, &phi_bb92_25, &phi_bb92_27, &phi_bb92_28, &phi_bb92_29, &phi_bb92_32, &phi_bb92_33, &phi_bb92_37, &phi_bb92_39, &phi_bb92_40, &phi_bb92_41);
    std::tie(tmp206, tmp207, tmp208) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp63}).Flatten();
    tmp209 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp210 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb92_20}, TNode<IntPtrT>{tmp209});
    tmp211 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb92_20});
    tmp212 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp208});
    tmp213 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp211}, TNode<UintPtrT>{tmp212});
    ca_.Branch(tmp213, &block108, std::vector<compiler::Node*>{phi_bb92_25, phi_bb92_27, phi_bb92_28, phi_bb92_29, phi_bb92_32, phi_bb92_33, phi_bb92_37, phi_bb92_39, phi_bb92_40, phi_bb92_20, phi_bb92_20, phi_bb92_20, phi_bb92_20}, &block109, std::vector<compiler::Node*>{phi_bb92_25, phi_bb92_27, phi_bb92_28, phi_bb92_29, phi_bb92_32, phi_bb92_33, phi_bb92_37, phi_bb92_39, phi_bb92_40, phi_bb92_20, phi_bb92_20, phi_bb92_20, phi_bb92_20});
  }

  TNode<IntPtrT> phi_bb108_25;
  TNode<IntPtrT> phi_bb108_27;
  TNode<IntPtrT> phi_bb108_28;
  TNode<IntPtrT> phi_bb108_29;
  TNode<IntPtrT> phi_bb108_32;
  TNode<BoolT> phi_bb108_33;
  TNode<BoolT> phi_bb108_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb108_39;
  TNode<IntPtrT> phi_bb108_40;
  TNode<IntPtrT> phi_bb108_46;
  TNode<IntPtrT> phi_bb108_47;
  TNode<IntPtrT> phi_bb108_51;
  TNode<IntPtrT> phi_bb108_52;
  TNode<IntPtrT> tmp214;
  TNode<IntPtrT> tmp215;
  TNode<Union<HeapObject, TaggedIndex>> tmp216;
  TNode<IntPtrT> tmp217;
  TNode<Number> tmp218;
  if (block108.is_used()) {
    ca_.Bind(&block108, &phi_bb108_25, &phi_bb108_27, &phi_bb108_28, &phi_bb108_29, &phi_bb108_32, &phi_bb108_33, &phi_bb108_37, &phi_bb108_39, &phi_bb108_40, &phi_bb108_46, &phi_bb108_47, &phi_bb108_51, &phi_bb108_52);
    tmp214 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb108_52});
    tmp215 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp207}, TNode<IntPtrT>{tmp214});
    std::tie(tmp216, tmp217) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp206}, TNode<IntPtrT>{tmp215}).Flatten();
    tmp218 = Convert_Number_float32_0(state_, TNode<Float32T>{phi_bb92_41});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp216, tmp217}, tmp218);
    ca_.Goto(&block80, tmp210, phi_bb108_25, tmp158, phi_bb108_27, phi_bb108_28, phi_bb108_29, phi_bb108_32, phi_bb108_33, phi_bb108_37);
  }

  TNode<IntPtrT> phi_bb109_25;
  TNode<IntPtrT> phi_bb109_27;
  TNode<IntPtrT> phi_bb109_28;
  TNode<IntPtrT> phi_bb109_29;
  TNode<IntPtrT> phi_bb109_32;
  TNode<BoolT> phi_bb109_33;
  TNode<BoolT> phi_bb109_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb109_39;
  TNode<IntPtrT> phi_bb109_40;
  TNode<IntPtrT> phi_bb109_46;
  TNode<IntPtrT> phi_bb109_47;
  TNode<IntPtrT> phi_bb109_51;
  TNode<IntPtrT> phi_bb109_52;
  if (block109.is_used()) {
    ca_.Bind(&block109, &phi_bb109_25, &phi_bb109_27, &phi_bb109_28, &phi_bb109_29, &phi_bb109_32, &phi_bb109_33, &phi_bb109_37, &phi_bb109_39, &phi_bb109_40, &phi_bb109_46, &phi_bb109_47, &phi_bb109_51, &phi_bb109_52);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb79_20;
  TNode<IntPtrT> phi_bb79_25;
  TNode<IntPtrT> phi_bb79_26;
  TNode<IntPtrT> phi_bb79_27;
  TNode<IntPtrT> phi_bb79_28;
  TNode<IntPtrT> phi_bb79_29;
  TNode<IntPtrT> phi_bb79_32;
  TNode<BoolT> phi_bb79_33;
  TNode<BoolT> phi_bb79_37;
  TNode<Uint32T> tmp219;
  TNode<BoolT> tmp220;
  if (block79.is_used()) {
    ca_.Bind(&block79, &phi_bb79_20, &phi_bb79_25, &phi_bb79_26, &phi_bb79_27, &phi_bb79_28, &phi_bb79_29, &phi_bb79_32, &phi_bb79_33, &phi_bb79_37);
    tmp219 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI64.raw_bit_field());
    tmp220 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp107}, TNode<Uint32T>{tmp219});
    ca_.Branch(tmp220, &block112, std::vector<compiler::Node*>{phi_bb79_20, phi_bb79_25, phi_bb79_26, phi_bb79_27, phi_bb79_28, phi_bb79_29, phi_bb79_32, phi_bb79_33, phi_bb79_37}, &block113, std::vector<compiler::Node*>{phi_bb79_20, phi_bb79_25, phi_bb79_26, phi_bb79_27, phi_bb79_28, phi_bb79_29, phi_bb79_32, phi_bb79_33, phi_bb79_37});
  }

  TNode<IntPtrT> phi_bb112_20;
  TNode<IntPtrT> phi_bb112_25;
  TNode<IntPtrT> phi_bb112_26;
  TNode<IntPtrT> phi_bb112_27;
  TNode<IntPtrT> phi_bb112_28;
  TNode<IntPtrT> phi_bb112_29;
  TNode<IntPtrT> phi_bb112_32;
  TNode<BoolT> phi_bb112_33;
  TNode<BoolT> phi_bb112_37;
  if (block112.is_used()) {
    ca_.Bind(&block112, &phi_bb112_20, &phi_bb112_25, &phi_bb112_26, &phi_bb112_27, &phi_bb112_28, &phi_bb112_29, &phi_bb112_32, &phi_bb112_33, &phi_bb112_37);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block115, phi_bb112_20, phi_bb112_25, phi_bb112_26, phi_bb112_27, phi_bb112_28, phi_bb112_29, phi_bb112_32, phi_bb112_33, phi_bb112_37);
    } else {
      ca_.Goto(&block116, phi_bb112_20, phi_bb112_25, phi_bb112_26, phi_bb112_27, phi_bb112_28, phi_bb112_29, phi_bb112_32, phi_bb112_33, phi_bb112_37);
    }
  }

  TNode<IntPtrT> phi_bb115_20;
  TNode<IntPtrT> phi_bb115_25;
  TNode<IntPtrT> phi_bb115_26;
  TNode<IntPtrT> phi_bb115_27;
  TNode<IntPtrT> phi_bb115_28;
  TNode<IntPtrT> phi_bb115_29;
  TNode<IntPtrT> phi_bb115_32;
  TNode<BoolT> phi_bb115_33;
  TNode<BoolT> phi_bb115_37;
  TNode<IntPtrT> tmp221;
  TNode<IntPtrT> tmp222;
  TNode<IntPtrT> tmp223;
  TNode<BoolT> tmp224;
  if (block115.is_used()) {
    ca_.Bind(&block115, &phi_bb115_20, &phi_bb115_25, &phi_bb115_26, &phi_bb115_27, &phi_bb115_28, &phi_bb115_29, &phi_bb115_32, &phi_bb115_33, &phi_bb115_37);
    tmp221 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp222 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb115_25}, TNode<IntPtrT>{tmp221});
    tmp223 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp224 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb115_25}, TNode<IntPtrT>{tmp223});
    ca_.Branch(tmp224, &block119, std::vector<compiler::Node*>{phi_bb115_20, phi_bb115_26, phi_bb115_27, phi_bb115_28, phi_bb115_29, phi_bb115_32, phi_bb115_33, phi_bb115_37}, &block120, std::vector<compiler::Node*>{phi_bb115_20, phi_bb115_26, phi_bb115_27, phi_bb115_28, phi_bb115_29, phi_bb115_32, phi_bb115_33, phi_bb115_37});
  }

  TNode<IntPtrT> phi_bb119_20;
  TNode<IntPtrT> phi_bb119_26;
  TNode<IntPtrT> phi_bb119_27;
  TNode<IntPtrT> phi_bb119_28;
  TNode<IntPtrT> phi_bb119_29;
  TNode<IntPtrT> phi_bb119_32;
  TNode<BoolT> phi_bb119_33;
  TNode<BoolT> phi_bb119_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp225;
  TNode<IntPtrT> tmp226;
  TNode<IntPtrT> tmp227;
  TNode<IntPtrT> tmp228;
  if (block119.is_used()) {
    ca_.Bind(&block119, &phi_bb119_20, &phi_bb119_26, &phi_bb119_27, &phi_bb119_28, &phi_bb119_29, &phi_bb119_32, &phi_bb119_33, &phi_bb119_37);
    std::tie(tmp225, tmp226) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb119_27}).Flatten();
    tmp227 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp228 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb119_27}, TNode<IntPtrT>{tmp227});
    ca_.Goto(&block118, phi_bb119_20, phi_bb119_26, tmp228, phi_bb119_28, phi_bb119_29, phi_bb119_32, phi_bb119_33, phi_bb119_37, tmp225, tmp226);
  }

  TNode<IntPtrT> phi_bb120_20;
  TNode<IntPtrT> phi_bb120_26;
  TNode<IntPtrT> phi_bb120_27;
  TNode<IntPtrT> phi_bb120_28;
  TNode<IntPtrT> phi_bb120_29;
  TNode<IntPtrT> phi_bb120_32;
  TNode<BoolT> phi_bb120_33;
  TNode<BoolT> phi_bb120_37;
  if (block120.is_used()) {
    ca_.Bind(&block120, &phi_bb120_20, &phi_bb120_26, &phi_bb120_27, &phi_bb120_28, &phi_bb120_29, &phi_bb120_32, &phi_bb120_33, &phi_bb120_37);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block122, phi_bb120_20, phi_bb120_26, phi_bb120_27, phi_bb120_28, phi_bb120_29, phi_bb120_32, phi_bb120_33, phi_bb120_37);
    } else {
      ca_.Goto(&block123, phi_bb120_20, phi_bb120_26, phi_bb120_27, phi_bb120_28, phi_bb120_29, phi_bb120_32, phi_bb120_33, phi_bb120_37);
    }
  }

  TNode<IntPtrT> phi_bb122_20;
  TNode<IntPtrT> phi_bb122_26;
  TNode<IntPtrT> phi_bb122_27;
  TNode<IntPtrT> phi_bb122_28;
  TNode<IntPtrT> phi_bb122_29;
  TNode<IntPtrT> phi_bb122_32;
  TNode<BoolT> phi_bb122_33;
  TNode<BoolT> phi_bb122_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp229;
  TNode<IntPtrT> tmp230;
  TNode<IntPtrT> tmp231;
  TNode<IntPtrT> tmp232;
  if (block122.is_used()) {
    ca_.Bind(&block122, &phi_bb122_20, &phi_bb122_26, &phi_bb122_27, &phi_bb122_28, &phi_bb122_29, &phi_bb122_32, &phi_bb122_33, &phi_bb122_37);
    std::tie(tmp229, tmp230) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb122_29}).Flatten();
    tmp231 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp232 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb122_29}, TNode<IntPtrT>{tmp231});
    ca_.Goto(&block121, phi_bb122_20, phi_bb122_26, phi_bb122_27, phi_bb122_28, tmp232, phi_bb122_32, phi_bb122_33, phi_bb122_37, tmp229, tmp230);
  }

  TNode<IntPtrT> phi_bb123_20;
  TNode<IntPtrT> phi_bb123_26;
  TNode<IntPtrT> phi_bb123_27;
  TNode<IntPtrT> phi_bb123_28;
  TNode<IntPtrT> phi_bb123_29;
  TNode<IntPtrT> phi_bb123_32;
  TNode<BoolT> phi_bb123_33;
  TNode<BoolT> phi_bb123_37;
  TNode<IntPtrT> tmp233;
  TNode<BoolT> tmp234;
  if (block123.is_used()) {
    ca_.Bind(&block123, &phi_bb123_20, &phi_bb123_26, &phi_bb123_27, &phi_bb123_28, &phi_bb123_29, &phi_bb123_32, &phi_bb123_33, &phi_bb123_37);
    tmp233 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp234 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb123_32}, TNode<IntPtrT>{tmp233});
    ca_.Branch(tmp234, &block125, std::vector<compiler::Node*>{phi_bb123_20, phi_bb123_26, phi_bb123_27, phi_bb123_28, phi_bb123_29, phi_bb123_32, phi_bb123_33, phi_bb123_37}, &block126, std::vector<compiler::Node*>{phi_bb123_20, phi_bb123_26, phi_bb123_27, phi_bb123_28, phi_bb123_29, phi_bb123_32, phi_bb123_33, phi_bb123_37});
  }

  TNode<IntPtrT> phi_bb125_20;
  TNode<IntPtrT> phi_bb125_26;
  TNode<IntPtrT> phi_bb125_27;
  TNode<IntPtrT> phi_bb125_28;
  TNode<IntPtrT> phi_bb125_29;
  TNode<IntPtrT> phi_bb125_32;
  TNode<BoolT> phi_bb125_33;
  TNode<BoolT> phi_bb125_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp235;
  TNode<IntPtrT> tmp236;
  TNode<IntPtrT> tmp237;
  TNode<BoolT> tmp238;
  if (block125.is_used()) {
    ca_.Bind(&block125, &phi_bb125_20, &phi_bb125_26, &phi_bb125_27, &phi_bb125_28, &phi_bb125_29, &phi_bb125_32, &phi_bb125_33, &phi_bb125_37);
    std::tie(tmp235, tmp236) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb125_32}).Flatten();
    tmp237 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp238 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block121, phi_bb125_20, phi_bb125_26, phi_bb125_27, phi_bb125_28, phi_bb125_29, tmp237, tmp238, phi_bb125_37, tmp235, tmp236);
  }

  TNode<IntPtrT> phi_bb126_20;
  TNode<IntPtrT> phi_bb126_26;
  TNode<IntPtrT> phi_bb126_27;
  TNode<IntPtrT> phi_bb126_28;
  TNode<IntPtrT> phi_bb126_29;
  TNode<IntPtrT> phi_bb126_32;
  TNode<BoolT> phi_bb126_33;
  TNode<BoolT> phi_bb126_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp239;
  TNode<IntPtrT> tmp240;
  TNode<IntPtrT> tmp241;
  TNode<IntPtrT> tmp242;
  TNode<IntPtrT> tmp243;
  TNode<IntPtrT> tmp244;
  TNode<BoolT> tmp245;
  if (block126.is_used()) {
    ca_.Bind(&block126, &phi_bb126_20, &phi_bb126_26, &phi_bb126_27, &phi_bb126_28, &phi_bb126_29, &phi_bb126_32, &phi_bb126_33, &phi_bb126_37);
    std::tie(tmp239, tmp240) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb126_29}).Flatten();
    tmp241 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp242 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb126_29}, TNode<IntPtrT>{tmp241});
    tmp243 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp244 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp242}, TNode<IntPtrT>{tmp243});
    tmp245 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block121, phi_bb126_20, phi_bb126_26, phi_bb126_27, phi_bb126_28, tmp244, tmp242, tmp245, phi_bb126_37, tmp239, tmp240);
  }

  TNode<IntPtrT> phi_bb121_20;
  TNode<IntPtrT> phi_bb121_26;
  TNode<IntPtrT> phi_bb121_27;
  TNode<IntPtrT> phi_bb121_28;
  TNode<IntPtrT> phi_bb121_29;
  TNode<IntPtrT> phi_bb121_32;
  TNode<BoolT> phi_bb121_33;
  TNode<BoolT> phi_bb121_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb121_39;
  TNode<IntPtrT> phi_bb121_40;
  if (block121.is_used()) {
    ca_.Bind(&block121, &phi_bb121_20, &phi_bb121_26, &phi_bb121_27, &phi_bb121_28, &phi_bb121_29, &phi_bb121_32, &phi_bb121_33, &phi_bb121_37, &phi_bb121_39, &phi_bb121_40);
    ca_.Goto(&block118, phi_bb121_20, phi_bb121_26, phi_bb121_27, phi_bb121_28, phi_bb121_29, phi_bb121_32, phi_bb121_33, phi_bb121_37, phi_bb121_39, phi_bb121_40);
  }

  TNode<IntPtrT> phi_bb118_20;
  TNode<IntPtrT> phi_bb118_26;
  TNode<IntPtrT> phi_bb118_27;
  TNode<IntPtrT> phi_bb118_28;
  TNode<IntPtrT> phi_bb118_29;
  TNode<IntPtrT> phi_bb118_32;
  TNode<BoolT> phi_bb118_33;
  TNode<BoolT> phi_bb118_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb118_39;
  TNode<IntPtrT> phi_bb118_40;
  TNode<IntPtrT> tmp246;
  TNode<Union<HeapObject, TaggedIndex>> tmp247;
  TNode<IntPtrT> tmp248;
  TNode<IntPtrT> tmp249;
  TNode<IntPtrT> tmp250;
  TNode<IntPtrT> tmp251;
  TNode<UintPtrT> tmp252;
  TNode<UintPtrT> tmp253;
  TNode<BoolT> tmp254;
  if (block118.is_used()) {
    ca_.Bind(&block118, &phi_bb118_20, &phi_bb118_26, &phi_bb118_27, &phi_bb118_28, &phi_bb118_29, &phi_bb118_32, &phi_bb118_33, &phi_bb118_37, &phi_bb118_39, &phi_bb118_40);
    tmp246 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb118_39, phi_bb118_40});
    std::tie(tmp247, tmp248, tmp249) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp63}).Flatten();
    tmp250 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp251 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb118_20}, TNode<IntPtrT>{tmp250});
    tmp252 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb118_20});
    tmp253 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp249});
    tmp254 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp252}, TNode<UintPtrT>{tmp253});
    ca_.Branch(tmp254, &block131, std::vector<compiler::Node*>{phi_bb118_26, phi_bb118_27, phi_bb118_28, phi_bb118_29, phi_bb118_32, phi_bb118_33, phi_bb118_37, phi_bb118_39, phi_bb118_40, phi_bb118_20, phi_bb118_20, phi_bb118_20, phi_bb118_20}, &block132, std::vector<compiler::Node*>{phi_bb118_26, phi_bb118_27, phi_bb118_28, phi_bb118_29, phi_bb118_32, phi_bb118_33, phi_bb118_37, phi_bb118_39, phi_bb118_40, phi_bb118_20, phi_bb118_20, phi_bb118_20, phi_bb118_20});
  }

  TNode<IntPtrT> phi_bb131_26;
  TNode<IntPtrT> phi_bb131_27;
  TNode<IntPtrT> phi_bb131_28;
  TNode<IntPtrT> phi_bb131_29;
  TNode<IntPtrT> phi_bb131_32;
  TNode<BoolT> phi_bb131_33;
  TNode<BoolT> phi_bb131_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb131_39;
  TNode<IntPtrT> phi_bb131_40;
  TNode<IntPtrT> phi_bb131_46;
  TNode<IntPtrT> phi_bb131_47;
  TNode<IntPtrT> phi_bb131_51;
  TNode<IntPtrT> phi_bb131_52;
  TNode<IntPtrT> tmp255;
  TNode<IntPtrT> tmp256;
  TNode<Union<HeapObject, TaggedIndex>> tmp257;
  TNode<IntPtrT> tmp258;
  TNode<BigInt> tmp259;
  if (block131.is_used()) {
    ca_.Bind(&block131, &phi_bb131_26, &phi_bb131_27, &phi_bb131_28, &phi_bb131_29, &phi_bb131_32, &phi_bb131_33, &phi_bb131_37, &phi_bb131_39, &phi_bb131_40, &phi_bb131_46, &phi_bb131_47, &phi_bb131_51, &phi_bb131_52);
    tmp255 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb131_52});
    tmp256 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp248}, TNode<IntPtrT>{tmp255});
    std::tie(tmp257, tmp258) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp247}, TNode<IntPtrT>{tmp256}).Flatten();
    tmp259 = ca_.CallBuiltin<BigInt>(Builtin::kI64ToBigInt, TNode<Object>(), tmp246);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp257, tmp258}, tmp259);
    ca_.Goto(&block117, tmp251, tmp222, phi_bb131_26, phi_bb131_27, phi_bb131_28, phi_bb131_29, phi_bb131_32, phi_bb131_33, phi_bb131_37);
  }

  TNode<IntPtrT> phi_bb132_26;
  TNode<IntPtrT> phi_bb132_27;
  TNode<IntPtrT> phi_bb132_28;
  TNode<IntPtrT> phi_bb132_29;
  TNode<IntPtrT> phi_bb132_32;
  TNode<BoolT> phi_bb132_33;
  TNode<BoolT> phi_bb132_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb132_39;
  TNode<IntPtrT> phi_bb132_40;
  TNode<IntPtrT> phi_bb132_46;
  TNode<IntPtrT> phi_bb132_47;
  TNode<IntPtrT> phi_bb132_51;
  TNode<IntPtrT> phi_bb132_52;
  if (block132.is_used()) {
    ca_.Bind(&block132, &phi_bb132_26, &phi_bb132_27, &phi_bb132_28, &phi_bb132_29, &phi_bb132_32, &phi_bb132_33, &phi_bb132_37, &phi_bb132_39, &phi_bb132_40, &phi_bb132_46, &phi_bb132_47, &phi_bb132_51, &phi_bb132_52);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb116_20;
  TNode<IntPtrT> phi_bb116_25;
  TNode<IntPtrT> phi_bb116_26;
  TNode<IntPtrT> phi_bb116_27;
  TNode<IntPtrT> phi_bb116_28;
  TNode<IntPtrT> phi_bb116_29;
  TNode<IntPtrT> phi_bb116_32;
  TNode<BoolT> phi_bb116_33;
  TNode<BoolT> phi_bb116_37;
  TNode<IntPtrT> tmp260;
  TNode<IntPtrT> tmp261;
  TNode<IntPtrT> tmp262;
  TNode<BoolT> tmp263;
  if (block116.is_used()) {
    ca_.Bind(&block116, &phi_bb116_20, &phi_bb116_25, &phi_bb116_26, &phi_bb116_27, &phi_bb116_28, &phi_bb116_29, &phi_bb116_32, &phi_bb116_33, &phi_bb116_37);
    tmp260 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp261 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb116_25}, TNode<IntPtrT>{tmp260});
    tmp262 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp263 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb116_25}, TNode<IntPtrT>{tmp262});
    ca_.Branch(tmp263, &block136, std::vector<compiler::Node*>{phi_bb116_20, phi_bb116_26, phi_bb116_27, phi_bb116_28, phi_bb116_29, phi_bb116_32, phi_bb116_33, phi_bb116_37}, &block137, std::vector<compiler::Node*>{phi_bb116_20, phi_bb116_26, phi_bb116_27, phi_bb116_28, phi_bb116_29, phi_bb116_32, phi_bb116_33, phi_bb116_37});
  }

  TNode<IntPtrT> phi_bb136_20;
  TNode<IntPtrT> phi_bb136_26;
  TNode<IntPtrT> phi_bb136_27;
  TNode<IntPtrT> phi_bb136_28;
  TNode<IntPtrT> phi_bb136_29;
  TNode<IntPtrT> phi_bb136_32;
  TNode<BoolT> phi_bb136_33;
  TNode<BoolT> phi_bb136_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp264;
  TNode<IntPtrT> tmp265;
  TNode<IntPtrT> tmp266;
  TNode<IntPtrT> tmp267;
  if (block136.is_used()) {
    ca_.Bind(&block136, &phi_bb136_20, &phi_bb136_26, &phi_bb136_27, &phi_bb136_28, &phi_bb136_29, &phi_bb136_32, &phi_bb136_33, &phi_bb136_37);
    std::tie(tmp264, tmp265) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb136_27}).Flatten();
    tmp266 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp267 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb136_27}, TNode<IntPtrT>{tmp266});
    ca_.Goto(&block135, phi_bb136_20, phi_bb136_26, tmp267, phi_bb136_28, phi_bb136_29, phi_bb136_32, phi_bb136_33, phi_bb136_37, tmp264, tmp265);
  }

  TNode<IntPtrT> phi_bb137_20;
  TNode<IntPtrT> phi_bb137_26;
  TNode<IntPtrT> phi_bb137_27;
  TNode<IntPtrT> phi_bb137_28;
  TNode<IntPtrT> phi_bb137_29;
  TNode<IntPtrT> phi_bb137_32;
  TNode<BoolT> phi_bb137_33;
  TNode<BoolT> phi_bb137_37;
  if (block137.is_used()) {
    ca_.Bind(&block137, &phi_bb137_20, &phi_bb137_26, &phi_bb137_27, &phi_bb137_28, &phi_bb137_29, &phi_bb137_32, &phi_bb137_33, &phi_bb137_37);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block139, phi_bb137_20, phi_bb137_26, phi_bb137_27, phi_bb137_28, phi_bb137_29, phi_bb137_32, phi_bb137_33, phi_bb137_37);
    } else {
      ca_.Goto(&block140, phi_bb137_20, phi_bb137_26, phi_bb137_27, phi_bb137_28, phi_bb137_29, phi_bb137_32, phi_bb137_33, phi_bb137_37);
    }
  }

  TNode<IntPtrT> phi_bb139_20;
  TNode<IntPtrT> phi_bb139_26;
  TNode<IntPtrT> phi_bb139_27;
  TNode<IntPtrT> phi_bb139_28;
  TNode<IntPtrT> phi_bb139_29;
  TNode<IntPtrT> phi_bb139_32;
  TNode<BoolT> phi_bb139_33;
  TNode<BoolT> phi_bb139_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp268;
  TNode<IntPtrT> tmp269;
  TNode<IntPtrT> tmp270;
  TNode<IntPtrT> tmp271;
  if (block139.is_used()) {
    ca_.Bind(&block139, &phi_bb139_20, &phi_bb139_26, &phi_bb139_27, &phi_bb139_28, &phi_bb139_29, &phi_bb139_32, &phi_bb139_33, &phi_bb139_37);
    std::tie(tmp268, tmp269) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb139_29}).Flatten();
    tmp270 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp271 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb139_29}, TNode<IntPtrT>{tmp270});
    ca_.Goto(&block138, phi_bb139_20, phi_bb139_26, phi_bb139_27, phi_bb139_28, tmp271, phi_bb139_32, phi_bb139_33, phi_bb139_37, tmp268, tmp269);
  }

  TNode<IntPtrT> phi_bb140_20;
  TNode<IntPtrT> phi_bb140_26;
  TNode<IntPtrT> phi_bb140_27;
  TNode<IntPtrT> phi_bb140_28;
  TNode<IntPtrT> phi_bb140_29;
  TNode<IntPtrT> phi_bb140_32;
  TNode<BoolT> phi_bb140_33;
  TNode<BoolT> phi_bb140_37;
  TNode<IntPtrT> tmp272;
  TNode<BoolT> tmp273;
  if (block140.is_used()) {
    ca_.Bind(&block140, &phi_bb140_20, &phi_bb140_26, &phi_bb140_27, &phi_bb140_28, &phi_bb140_29, &phi_bb140_32, &phi_bb140_33, &phi_bb140_37);
    tmp272 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp273 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb140_32}, TNode<IntPtrT>{tmp272});
    ca_.Branch(tmp273, &block142, std::vector<compiler::Node*>{phi_bb140_20, phi_bb140_26, phi_bb140_27, phi_bb140_28, phi_bb140_29, phi_bb140_32, phi_bb140_33, phi_bb140_37}, &block143, std::vector<compiler::Node*>{phi_bb140_20, phi_bb140_26, phi_bb140_27, phi_bb140_28, phi_bb140_29, phi_bb140_32, phi_bb140_33, phi_bb140_37});
  }

  TNode<IntPtrT> phi_bb142_20;
  TNode<IntPtrT> phi_bb142_26;
  TNode<IntPtrT> phi_bb142_27;
  TNode<IntPtrT> phi_bb142_28;
  TNode<IntPtrT> phi_bb142_29;
  TNode<IntPtrT> phi_bb142_32;
  TNode<BoolT> phi_bb142_33;
  TNode<BoolT> phi_bb142_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp274;
  TNode<IntPtrT> tmp275;
  TNode<IntPtrT> tmp276;
  TNode<BoolT> tmp277;
  if (block142.is_used()) {
    ca_.Bind(&block142, &phi_bb142_20, &phi_bb142_26, &phi_bb142_27, &phi_bb142_28, &phi_bb142_29, &phi_bb142_32, &phi_bb142_33, &phi_bb142_37);
    std::tie(tmp274, tmp275) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb142_32}).Flatten();
    tmp276 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp277 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block138, phi_bb142_20, phi_bb142_26, phi_bb142_27, phi_bb142_28, phi_bb142_29, tmp276, tmp277, phi_bb142_37, tmp274, tmp275);
  }

  TNode<IntPtrT> phi_bb143_20;
  TNode<IntPtrT> phi_bb143_26;
  TNode<IntPtrT> phi_bb143_27;
  TNode<IntPtrT> phi_bb143_28;
  TNode<IntPtrT> phi_bb143_29;
  TNode<IntPtrT> phi_bb143_32;
  TNode<BoolT> phi_bb143_33;
  TNode<BoolT> phi_bb143_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp278;
  TNode<IntPtrT> tmp279;
  TNode<IntPtrT> tmp280;
  TNode<IntPtrT> tmp281;
  TNode<IntPtrT> tmp282;
  TNode<IntPtrT> tmp283;
  TNode<BoolT> tmp284;
  if (block143.is_used()) {
    ca_.Bind(&block143, &phi_bb143_20, &phi_bb143_26, &phi_bb143_27, &phi_bb143_28, &phi_bb143_29, &phi_bb143_32, &phi_bb143_33, &phi_bb143_37);
    std::tie(tmp278, tmp279) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb143_29}).Flatten();
    tmp280 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp281 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb143_29}, TNode<IntPtrT>{tmp280});
    tmp282 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp283 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp281}, TNode<IntPtrT>{tmp282});
    tmp284 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block138, phi_bb143_20, phi_bb143_26, phi_bb143_27, phi_bb143_28, tmp283, tmp281, tmp284, phi_bb143_37, tmp278, tmp279);
  }

  TNode<IntPtrT> phi_bb138_20;
  TNode<IntPtrT> phi_bb138_26;
  TNode<IntPtrT> phi_bb138_27;
  TNode<IntPtrT> phi_bb138_28;
  TNode<IntPtrT> phi_bb138_29;
  TNode<IntPtrT> phi_bb138_32;
  TNode<BoolT> phi_bb138_33;
  TNode<BoolT> phi_bb138_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb138_39;
  TNode<IntPtrT> phi_bb138_40;
  if (block138.is_used()) {
    ca_.Bind(&block138, &phi_bb138_20, &phi_bb138_26, &phi_bb138_27, &phi_bb138_28, &phi_bb138_29, &phi_bb138_32, &phi_bb138_33, &phi_bb138_37, &phi_bb138_39, &phi_bb138_40);
    ca_.Goto(&block135, phi_bb138_20, phi_bb138_26, phi_bb138_27, phi_bb138_28, phi_bb138_29, phi_bb138_32, phi_bb138_33, phi_bb138_37, phi_bb138_39, phi_bb138_40);
  }

  TNode<IntPtrT> phi_bb135_20;
  TNode<IntPtrT> phi_bb135_26;
  TNode<IntPtrT> phi_bb135_27;
  TNode<IntPtrT> phi_bb135_28;
  TNode<IntPtrT> phi_bb135_29;
  TNode<IntPtrT> phi_bb135_32;
  TNode<BoolT> phi_bb135_33;
  TNode<BoolT> phi_bb135_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb135_39;
  TNode<IntPtrT> phi_bb135_40;
  TNode<IntPtrT> tmp285;
  TNode<IntPtrT> tmp286;
  TNode<IntPtrT> tmp287;
  TNode<BoolT> tmp288;
  if (block135.is_used()) {
    ca_.Bind(&block135, &phi_bb135_20, &phi_bb135_26, &phi_bb135_27, &phi_bb135_28, &phi_bb135_29, &phi_bb135_32, &phi_bb135_33, &phi_bb135_37, &phi_bb135_39, &phi_bb135_40);
    tmp285 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp286 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp261}, TNode<IntPtrT>{tmp285});
    tmp287 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp288 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp261}, TNode<IntPtrT>{tmp287});
    ca_.Branch(tmp288, &block145, std::vector<compiler::Node*>{phi_bb135_20, phi_bb135_26, phi_bb135_27, phi_bb135_28, phi_bb135_29, phi_bb135_32, phi_bb135_33, phi_bb135_37, phi_bb135_39, phi_bb135_40}, &block146, std::vector<compiler::Node*>{phi_bb135_20, phi_bb135_26, phi_bb135_27, phi_bb135_28, phi_bb135_29, phi_bb135_32, phi_bb135_33, phi_bb135_37, phi_bb135_39, phi_bb135_40});
  }

  TNode<IntPtrT> phi_bb145_20;
  TNode<IntPtrT> phi_bb145_26;
  TNode<IntPtrT> phi_bb145_27;
  TNode<IntPtrT> phi_bb145_28;
  TNode<IntPtrT> phi_bb145_29;
  TNode<IntPtrT> phi_bb145_32;
  TNode<BoolT> phi_bb145_33;
  TNode<BoolT> phi_bb145_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb145_39;
  TNode<IntPtrT> phi_bb145_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp289;
  TNode<IntPtrT> tmp290;
  TNode<IntPtrT> tmp291;
  TNode<IntPtrT> tmp292;
  if (block145.is_used()) {
    ca_.Bind(&block145, &phi_bb145_20, &phi_bb145_26, &phi_bb145_27, &phi_bb145_28, &phi_bb145_29, &phi_bb145_32, &phi_bb145_33, &phi_bb145_37, &phi_bb145_39, &phi_bb145_40);
    std::tie(tmp289, tmp290) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb145_27}).Flatten();
    tmp291 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp292 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb145_27}, TNode<IntPtrT>{tmp291});
    ca_.Goto(&block144, phi_bb145_20, phi_bb145_26, tmp292, phi_bb145_28, phi_bb145_29, phi_bb145_32, phi_bb145_33, phi_bb145_37, phi_bb145_39, phi_bb145_40, tmp289, tmp290);
  }

  TNode<IntPtrT> phi_bb146_20;
  TNode<IntPtrT> phi_bb146_26;
  TNode<IntPtrT> phi_bb146_27;
  TNode<IntPtrT> phi_bb146_28;
  TNode<IntPtrT> phi_bb146_29;
  TNode<IntPtrT> phi_bb146_32;
  TNode<BoolT> phi_bb146_33;
  TNode<BoolT> phi_bb146_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb146_39;
  TNode<IntPtrT> phi_bb146_40;
  if (block146.is_used()) {
    ca_.Bind(&block146, &phi_bb146_20, &phi_bb146_26, &phi_bb146_27, &phi_bb146_28, &phi_bb146_29, &phi_bb146_32, &phi_bb146_33, &phi_bb146_37, &phi_bb146_39, &phi_bb146_40);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block148, phi_bb146_20, phi_bb146_26, phi_bb146_27, phi_bb146_28, phi_bb146_29, phi_bb146_32, phi_bb146_33, phi_bb146_37, phi_bb146_39, phi_bb146_40);
    } else {
      ca_.Goto(&block149, phi_bb146_20, phi_bb146_26, phi_bb146_27, phi_bb146_28, phi_bb146_29, phi_bb146_32, phi_bb146_33, phi_bb146_37, phi_bb146_39, phi_bb146_40);
    }
  }

  TNode<IntPtrT> phi_bb148_20;
  TNode<IntPtrT> phi_bb148_26;
  TNode<IntPtrT> phi_bb148_27;
  TNode<IntPtrT> phi_bb148_28;
  TNode<IntPtrT> phi_bb148_29;
  TNode<IntPtrT> phi_bb148_32;
  TNode<BoolT> phi_bb148_33;
  TNode<BoolT> phi_bb148_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb148_39;
  TNode<IntPtrT> phi_bb148_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp293;
  TNode<IntPtrT> tmp294;
  TNode<IntPtrT> tmp295;
  TNode<IntPtrT> tmp296;
  if (block148.is_used()) {
    ca_.Bind(&block148, &phi_bb148_20, &phi_bb148_26, &phi_bb148_27, &phi_bb148_28, &phi_bb148_29, &phi_bb148_32, &phi_bb148_33, &phi_bb148_37, &phi_bb148_39, &phi_bb148_40);
    std::tie(tmp293, tmp294) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb148_29}).Flatten();
    tmp295 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp296 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb148_29}, TNode<IntPtrT>{tmp295});
    ca_.Goto(&block147, phi_bb148_20, phi_bb148_26, phi_bb148_27, phi_bb148_28, tmp296, phi_bb148_32, phi_bb148_33, phi_bb148_37, phi_bb148_39, phi_bb148_40, tmp293, tmp294);
  }

  TNode<IntPtrT> phi_bb149_20;
  TNode<IntPtrT> phi_bb149_26;
  TNode<IntPtrT> phi_bb149_27;
  TNode<IntPtrT> phi_bb149_28;
  TNode<IntPtrT> phi_bb149_29;
  TNode<IntPtrT> phi_bb149_32;
  TNode<BoolT> phi_bb149_33;
  TNode<BoolT> phi_bb149_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb149_39;
  TNode<IntPtrT> phi_bb149_40;
  TNode<IntPtrT> tmp297;
  TNode<BoolT> tmp298;
  if (block149.is_used()) {
    ca_.Bind(&block149, &phi_bb149_20, &phi_bb149_26, &phi_bb149_27, &phi_bb149_28, &phi_bb149_29, &phi_bb149_32, &phi_bb149_33, &phi_bb149_37, &phi_bb149_39, &phi_bb149_40);
    tmp297 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp298 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb149_32}, TNode<IntPtrT>{tmp297});
    ca_.Branch(tmp298, &block151, std::vector<compiler::Node*>{phi_bb149_20, phi_bb149_26, phi_bb149_27, phi_bb149_28, phi_bb149_29, phi_bb149_32, phi_bb149_33, phi_bb149_37, phi_bb149_39, phi_bb149_40}, &block152, std::vector<compiler::Node*>{phi_bb149_20, phi_bb149_26, phi_bb149_27, phi_bb149_28, phi_bb149_29, phi_bb149_32, phi_bb149_33, phi_bb149_37, phi_bb149_39, phi_bb149_40});
  }

  TNode<IntPtrT> phi_bb151_20;
  TNode<IntPtrT> phi_bb151_26;
  TNode<IntPtrT> phi_bb151_27;
  TNode<IntPtrT> phi_bb151_28;
  TNode<IntPtrT> phi_bb151_29;
  TNode<IntPtrT> phi_bb151_32;
  TNode<BoolT> phi_bb151_33;
  TNode<BoolT> phi_bb151_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb151_39;
  TNode<IntPtrT> phi_bb151_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp299;
  TNode<IntPtrT> tmp300;
  TNode<IntPtrT> tmp301;
  TNode<BoolT> tmp302;
  if (block151.is_used()) {
    ca_.Bind(&block151, &phi_bb151_20, &phi_bb151_26, &phi_bb151_27, &phi_bb151_28, &phi_bb151_29, &phi_bb151_32, &phi_bb151_33, &phi_bb151_37, &phi_bb151_39, &phi_bb151_40);
    std::tie(tmp299, tmp300) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb151_32}).Flatten();
    tmp301 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp302 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block147, phi_bb151_20, phi_bb151_26, phi_bb151_27, phi_bb151_28, phi_bb151_29, tmp301, tmp302, phi_bb151_37, phi_bb151_39, phi_bb151_40, tmp299, tmp300);
  }

  TNode<IntPtrT> phi_bb152_20;
  TNode<IntPtrT> phi_bb152_26;
  TNode<IntPtrT> phi_bb152_27;
  TNode<IntPtrT> phi_bb152_28;
  TNode<IntPtrT> phi_bb152_29;
  TNode<IntPtrT> phi_bb152_32;
  TNode<BoolT> phi_bb152_33;
  TNode<BoolT> phi_bb152_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb152_39;
  TNode<IntPtrT> phi_bb152_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp303;
  TNode<IntPtrT> tmp304;
  TNode<IntPtrT> tmp305;
  TNode<IntPtrT> tmp306;
  TNode<IntPtrT> tmp307;
  TNode<IntPtrT> tmp308;
  TNode<BoolT> tmp309;
  if (block152.is_used()) {
    ca_.Bind(&block152, &phi_bb152_20, &phi_bb152_26, &phi_bb152_27, &phi_bb152_28, &phi_bb152_29, &phi_bb152_32, &phi_bb152_33, &phi_bb152_37, &phi_bb152_39, &phi_bb152_40);
    std::tie(tmp303, tmp304) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb152_29}).Flatten();
    tmp305 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp306 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb152_29}, TNode<IntPtrT>{tmp305});
    tmp307 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp308 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp306}, TNode<IntPtrT>{tmp307});
    tmp309 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block147, phi_bb152_20, phi_bb152_26, phi_bb152_27, phi_bb152_28, tmp308, tmp306, tmp309, phi_bb152_37, phi_bb152_39, phi_bb152_40, tmp303, tmp304);
  }

  TNode<IntPtrT> phi_bb147_20;
  TNode<IntPtrT> phi_bb147_26;
  TNode<IntPtrT> phi_bb147_27;
  TNode<IntPtrT> phi_bb147_28;
  TNode<IntPtrT> phi_bb147_29;
  TNode<IntPtrT> phi_bb147_32;
  TNode<BoolT> phi_bb147_33;
  TNode<BoolT> phi_bb147_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb147_39;
  TNode<IntPtrT> phi_bb147_40;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb147_41;
  TNode<IntPtrT> phi_bb147_42;
  if (block147.is_used()) {
    ca_.Bind(&block147, &phi_bb147_20, &phi_bb147_26, &phi_bb147_27, &phi_bb147_28, &phi_bb147_29, &phi_bb147_32, &phi_bb147_33, &phi_bb147_37, &phi_bb147_39, &phi_bb147_40, &phi_bb147_41, &phi_bb147_42);
    ca_.Goto(&block144, phi_bb147_20, phi_bb147_26, phi_bb147_27, phi_bb147_28, phi_bb147_29, phi_bb147_32, phi_bb147_33, phi_bb147_37, phi_bb147_39, phi_bb147_40, phi_bb147_41, phi_bb147_42);
  }

  TNode<IntPtrT> phi_bb144_20;
  TNode<IntPtrT> phi_bb144_26;
  TNode<IntPtrT> phi_bb144_27;
  TNode<IntPtrT> phi_bb144_28;
  TNode<IntPtrT> phi_bb144_29;
  TNode<IntPtrT> phi_bb144_32;
  TNode<BoolT> phi_bb144_33;
  TNode<BoolT> phi_bb144_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb144_39;
  TNode<IntPtrT> phi_bb144_40;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb144_41;
  TNode<IntPtrT> phi_bb144_42;
  TNode<IntPtrT> tmp310;
  TNode<IntPtrT> tmp311;
  TNode<Union<HeapObject, TaggedIndex>> tmp312;
  TNode<IntPtrT> tmp313;
  TNode<IntPtrT> tmp314;
  TNode<IntPtrT> tmp315;
  TNode<IntPtrT> tmp316;
  TNode<UintPtrT> tmp317;
  TNode<UintPtrT> tmp318;
  TNode<BoolT> tmp319;
  if (block144.is_used()) {
    ca_.Bind(&block144, &phi_bb144_20, &phi_bb144_26, &phi_bb144_27, &phi_bb144_28, &phi_bb144_29, &phi_bb144_32, &phi_bb144_33, &phi_bb144_37, &phi_bb144_39, &phi_bb144_40, &phi_bb144_41, &phi_bb144_42);
    tmp310 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb144_39, phi_bb144_40});
    tmp311 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb144_41, phi_bb144_42});
    std::tie(tmp312, tmp313, tmp314) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp63}).Flatten();
    tmp315 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp316 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb144_20}, TNode<IntPtrT>{tmp315});
    tmp317 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb144_20});
    tmp318 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp314});
    tmp319 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp317}, TNode<UintPtrT>{tmp318});
    ca_.Branch(tmp319, &block157, std::vector<compiler::Node*>{phi_bb144_26, phi_bb144_27, phi_bb144_28, phi_bb144_29, phi_bb144_32, phi_bb144_33, phi_bb144_37, phi_bb144_39, phi_bb144_40, phi_bb144_41, phi_bb144_42, phi_bb144_20, phi_bb144_20, phi_bb144_20, phi_bb144_20}, &block158, std::vector<compiler::Node*>{phi_bb144_26, phi_bb144_27, phi_bb144_28, phi_bb144_29, phi_bb144_32, phi_bb144_33, phi_bb144_37, phi_bb144_39, phi_bb144_40, phi_bb144_41, phi_bb144_42, phi_bb144_20, phi_bb144_20, phi_bb144_20, phi_bb144_20});
  }

  TNode<IntPtrT> phi_bb157_26;
  TNode<IntPtrT> phi_bb157_27;
  TNode<IntPtrT> phi_bb157_28;
  TNode<IntPtrT> phi_bb157_29;
  TNode<IntPtrT> phi_bb157_32;
  TNode<BoolT> phi_bb157_33;
  TNode<BoolT> phi_bb157_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb157_39;
  TNode<IntPtrT> phi_bb157_40;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb157_41;
  TNode<IntPtrT> phi_bb157_42;
  TNode<IntPtrT> phi_bb157_49;
  TNode<IntPtrT> phi_bb157_50;
  TNode<IntPtrT> phi_bb157_54;
  TNode<IntPtrT> phi_bb157_55;
  TNode<IntPtrT> tmp320;
  TNode<IntPtrT> tmp321;
  TNode<Union<HeapObject, TaggedIndex>> tmp322;
  TNode<IntPtrT> tmp323;
  TNode<BigInt> tmp324;
  if (block157.is_used()) {
    ca_.Bind(&block157, &phi_bb157_26, &phi_bb157_27, &phi_bb157_28, &phi_bb157_29, &phi_bb157_32, &phi_bb157_33, &phi_bb157_37, &phi_bb157_39, &phi_bb157_40, &phi_bb157_41, &phi_bb157_42, &phi_bb157_49, &phi_bb157_50, &phi_bb157_54, &phi_bb157_55);
    tmp320 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb157_55});
    tmp321 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp313}, TNode<IntPtrT>{tmp320});
    std::tie(tmp322, tmp323) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp312}, TNode<IntPtrT>{tmp321}).Flatten();
    tmp324 = ca_.CallBuiltin<BigInt>(Builtin::kI32PairToBigInt, TNode<Object>(), tmp310, tmp311);
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp322, tmp323}, tmp324);
    ca_.Goto(&block117, tmp316, tmp286, phi_bb157_26, phi_bb157_27, phi_bb157_28, phi_bb157_29, phi_bb157_32, phi_bb157_33, phi_bb157_37);
  }

  TNode<IntPtrT> phi_bb158_26;
  TNode<IntPtrT> phi_bb158_27;
  TNode<IntPtrT> phi_bb158_28;
  TNode<IntPtrT> phi_bb158_29;
  TNode<IntPtrT> phi_bb158_32;
  TNode<BoolT> phi_bb158_33;
  TNode<BoolT> phi_bb158_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb158_39;
  TNode<IntPtrT> phi_bb158_40;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb158_41;
  TNode<IntPtrT> phi_bb158_42;
  TNode<IntPtrT> phi_bb158_49;
  TNode<IntPtrT> phi_bb158_50;
  TNode<IntPtrT> phi_bb158_54;
  TNode<IntPtrT> phi_bb158_55;
  if (block158.is_used()) {
    ca_.Bind(&block158, &phi_bb158_26, &phi_bb158_27, &phi_bb158_28, &phi_bb158_29, &phi_bb158_32, &phi_bb158_33, &phi_bb158_37, &phi_bb158_39, &phi_bb158_40, &phi_bb158_41, &phi_bb158_42, &phi_bb158_49, &phi_bb158_50, &phi_bb158_54, &phi_bb158_55);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb117_20;
  TNode<IntPtrT> phi_bb117_25;
  TNode<IntPtrT> phi_bb117_26;
  TNode<IntPtrT> phi_bb117_27;
  TNode<IntPtrT> phi_bb117_28;
  TNode<IntPtrT> phi_bb117_29;
  TNode<IntPtrT> phi_bb117_32;
  TNode<BoolT> phi_bb117_33;
  TNode<BoolT> phi_bb117_37;
  if (block117.is_used()) {
    ca_.Bind(&block117, &phi_bb117_20, &phi_bb117_25, &phi_bb117_26, &phi_bb117_27, &phi_bb117_28, &phi_bb117_29, &phi_bb117_32, &phi_bb117_33, &phi_bb117_37);
    ca_.Goto(&block114, phi_bb117_20, phi_bb117_25, phi_bb117_26, phi_bb117_27, phi_bb117_28, phi_bb117_29, phi_bb117_32, phi_bb117_33, phi_bb117_37);
  }

  TNode<IntPtrT> phi_bb113_20;
  TNode<IntPtrT> phi_bb113_25;
  TNode<IntPtrT> phi_bb113_26;
  TNode<IntPtrT> phi_bb113_27;
  TNode<IntPtrT> phi_bb113_28;
  TNode<IntPtrT> phi_bb113_29;
  TNode<IntPtrT> phi_bb113_32;
  TNode<BoolT> phi_bb113_33;
  TNode<BoolT> phi_bb113_37;
  TNode<Uint32T> tmp325;
  TNode<BoolT> tmp326;
  if (block113.is_used()) {
    ca_.Bind(&block113, &phi_bb113_20, &phi_bb113_25, &phi_bb113_26, &phi_bb113_27, &phi_bb113_28, &phi_bb113_29, &phi_bb113_32, &phi_bb113_33, &phi_bb113_37);
    tmp325 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF64.raw_bit_field());
    tmp326 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp107}, TNode<Uint32T>{tmp325});
    ca_.Branch(tmp326, &block161, std::vector<compiler::Node*>{phi_bb113_20, phi_bb113_25, phi_bb113_26, phi_bb113_27, phi_bb113_28, phi_bb113_29, phi_bb113_32, phi_bb113_33, phi_bb113_37}, &block162, std::vector<compiler::Node*>{phi_bb113_20, phi_bb113_25, phi_bb113_26, phi_bb113_27, phi_bb113_28, phi_bb113_29, phi_bb113_32, phi_bb113_33, phi_bb113_37});
  }

  TNode<IntPtrT> phi_bb161_20;
  TNode<IntPtrT> phi_bb161_25;
  TNode<IntPtrT> phi_bb161_26;
  TNode<IntPtrT> phi_bb161_27;
  TNode<IntPtrT> phi_bb161_28;
  TNode<IntPtrT> phi_bb161_29;
  TNode<IntPtrT> phi_bb161_32;
  TNode<BoolT> phi_bb161_33;
  TNode<BoolT> phi_bb161_37;
  TNode<IntPtrT> tmp327;
  TNode<IntPtrT> tmp328;
  TNode<IntPtrT> tmp329;
  TNode<BoolT> tmp330;
  if (block161.is_used()) {
    ca_.Bind(&block161, &phi_bb161_20, &phi_bb161_25, &phi_bb161_26, &phi_bb161_27, &phi_bb161_28, &phi_bb161_29, &phi_bb161_32, &phi_bb161_33, &phi_bb161_37);
    tmp327 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp328 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb161_26}, TNode<IntPtrT>{tmp327});
    tmp329 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp330 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb161_26}, TNode<IntPtrT>{tmp329});
    ca_.Branch(tmp330, &block165, std::vector<compiler::Node*>{phi_bb161_20, phi_bb161_25, phi_bb161_27, phi_bb161_28, phi_bb161_29, phi_bb161_32, phi_bb161_33, phi_bb161_37}, &block166, std::vector<compiler::Node*>{phi_bb161_20, phi_bb161_25, phi_bb161_27, phi_bb161_28, phi_bb161_29, phi_bb161_32, phi_bb161_33, phi_bb161_37});
  }

  TNode<IntPtrT> phi_bb165_20;
  TNode<IntPtrT> phi_bb165_25;
  TNode<IntPtrT> phi_bb165_27;
  TNode<IntPtrT> phi_bb165_28;
  TNode<IntPtrT> phi_bb165_29;
  TNode<IntPtrT> phi_bb165_32;
  TNode<BoolT> phi_bb165_33;
  TNode<BoolT> phi_bb165_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp331;
  TNode<IntPtrT> tmp332;
  TNode<IntPtrT> tmp333;
  TNode<IntPtrT> tmp334;
  if (block165.is_used()) {
    ca_.Bind(&block165, &phi_bb165_20, &phi_bb165_25, &phi_bb165_27, &phi_bb165_28, &phi_bb165_29, &phi_bb165_32, &phi_bb165_33, &phi_bb165_37);
    std::tie(tmp331, tmp332) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb165_28}).Flatten();
    tmp333 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp334 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb165_28}, TNode<IntPtrT>{tmp333});
    ca_.Goto(&block164, phi_bb165_20, phi_bb165_25, phi_bb165_27, tmp334, phi_bb165_29, phi_bb165_32, phi_bb165_33, phi_bb165_37, tmp331, tmp332);
  }

  TNode<IntPtrT> phi_bb166_20;
  TNode<IntPtrT> phi_bb166_25;
  TNode<IntPtrT> phi_bb166_27;
  TNode<IntPtrT> phi_bb166_28;
  TNode<IntPtrT> phi_bb166_29;
  TNode<IntPtrT> phi_bb166_32;
  TNode<BoolT> phi_bb166_33;
  TNode<BoolT> phi_bb166_37;
  if (block166.is_used()) {
    ca_.Bind(&block166, &phi_bb166_20, &phi_bb166_25, &phi_bb166_27, &phi_bb166_28, &phi_bb166_29, &phi_bb166_32, &phi_bb166_33, &phi_bb166_37);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block167, phi_bb166_20, phi_bb166_25, phi_bb166_27, phi_bb166_28, phi_bb166_29, phi_bb166_32, phi_bb166_33, phi_bb166_37);
    } else {
      ca_.Goto(&block168, phi_bb166_20, phi_bb166_25, phi_bb166_27, phi_bb166_28, phi_bb166_29, phi_bb166_32, phi_bb166_33, phi_bb166_37);
    }
  }

  TNode<IntPtrT> phi_bb167_20;
  TNode<IntPtrT> phi_bb167_25;
  TNode<IntPtrT> phi_bb167_27;
  TNode<IntPtrT> phi_bb167_28;
  TNode<IntPtrT> phi_bb167_29;
  TNode<IntPtrT> phi_bb167_32;
  TNode<BoolT> phi_bb167_33;
  TNode<BoolT> phi_bb167_37;
  if (block167.is_used()) {
    ca_.Bind(&block167, &phi_bb167_20, &phi_bb167_25, &phi_bb167_27, &phi_bb167_28, &phi_bb167_29, &phi_bb167_32, &phi_bb167_33, &phi_bb167_37);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block171, phi_bb167_20, phi_bb167_25, phi_bb167_27, phi_bb167_28, phi_bb167_29, phi_bb167_32, phi_bb167_33, phi_bb167_37);
    } else {
      ca_.Goto(&block172, phi_bb167_20, phi_bb167_25, phi_bb167_27, phi_bb167_28, phi_bb167_29, phi_bb167_32, phi_bb167_33, phi_bb167_37);
    }
  }

  TNode<IntPtrT> phi_bb171_20;
  TNode<IntPtrT> phi_bb171_25;
  TNode<IntPtrT> phi_bb171_27;
  TNode<IntPtrT> phi_bb171_28;
  TNode<IntPtrT> phi_bb171_29;
  TNode<IntPtrT> phi_bb171_32;
  TNode<BoolT> phi_bb171_33;
  TNode<BoolT> phi_bb171_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp335;
  TNode<IntPtrT> tmp336;
  TNode<IntPtrT> tmp337;
  TNode<IntPtrT> tmp338;
  if (block171.is_used()) {
    ca_.Bind(&block171, &phi_bb171_20, &phi_bb171_25, &phi_bb171_27, &phi_bb171_28, &phi_bb171_29, &phi_bb171_32, &phi_bb171_33, &phi_bb171_37);
    std::tie(tmp335, tmp336) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb171_29}).Flatten();
    tmp337 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp338 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb171_29}, TNode<IntPtrT>{tmp337});
    ca_.Goto(&block170, phi_bb171_20, phi_bb171_25, phi_bb171_27, phi_bb171_28, tmp338, phi_bb171_32, phi_bb171_33, phi_bb171_37, tmp335, tmp336);
  }

  TNode<IntPtrT> phi_bb172_20;
  TNode<IntPtrT> phi_bb172_25;
  TNode<IntPtrT> phi_bb172_27;
  TNode<IntPtrT> phi_bb172_28;
  TNode<IntPtrT> phi_bb172_29;
  TNode<IntPtrT> phi_bb172_32;
  TNode<BoolT> phi_bb172_33;
  TNode<BoolT> phi_bb172_37;
  TNode<IntPtrT> tmp339;
  TNode<BoolT> tmp340;
  if (block172.is_used()) {
    ca_.Bind(&block172, &phi_bb172_20, &phi_bb172_25, &phi_bb172_27, &phi_bb172_28, &phi_bb172_29, &phi_bb172_32, &phi_bb172_33, &phi_bb172_37);
    tmp339 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp340 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb172_32}, TNode<IntPtrT>{tmp339});
    ca_.Branch(tmp340, &block174, std::vector<compiler::Node*>{phi_bb172_20, phi_bb172_25, phi_bb172_27, phi_bb172_28, phi_bb172_29, phi_bb172_32, phi_bb172_33, phi_bb172_37}, &block175, std::vector<compiler::Node*>{phi_bb172_20, phi_bb172_25, phi_bb172_27, phi_bb172_28, phi_bb172_29, phi_bb172_32, phi_bb172_33, phi_bb172_37});
  }

  TNode<IntPtrT> phi_bb174_20;
  TNode<IntPtrT> phi_bb174_25;
  TNode<IntPtrT> phi_bb174_27;
  TNode<IntPtrT> phi_bb174_28;
  TNode<IntPtrT> phi_bb174_29;
  TNode<IntPtrT> phi_bb174_32;
  TNode<BoolT> phi_bb174_33;
  TNode<BoolT> phi_bb174_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp341;
  TNode<IntPtrT> tmp342;
  TNode<IntPtrT> tmp343;
  TNode<BoolT> tmp344;
  if (block174.is_used()) {
    ca_.Bind(&block174, &phi_bb174_20, &phi_bb174_25, &phi_bb174_27, &phi_bb174_28, &phi_bb174_29, &phi_bb174_32, &phi_bb174_33, &phi_bb174_37);
    std::tie(tmp341, tmp342) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb174_32}).Flatten();
    tmp343 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp344 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block170, phi_bb174_20, phi_bb174_25, phi_bb174_27, phi_bb174_28, phi_bb174_29, tmp343, tmp344, phi_bb174_37, tmp341, tmp342);
  }

  TNode<IntPtrT> phi_bb175_20;
  TNode<IntPtrT> phi_bb175_25;
  TNode<IntPtrT> phi_bb175_27;
  TNode<IntPtrT> phi_bb175_28;
  TNode<IntPtrT> phi_bb175_29;
  TNode<IntPtrT> phi_bb175_32;
  TNode<BoolT> phi_bb175_33;
  TNode<BoolT> phi_bb175_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp345;
  TNode<IntPtrT> tmp346;
  TNode<IntPtrT> tmp347;
  TNode<IntPtrT> tmp348;
  TNode<IntPtrT> tmp349;
  TNode<IntPtrT> tmp350;
  TNode<BoolT> tmp351;
  if (block175.is_used()) {
    ca_.Bind(&block175, &phi_bb175_20, &phi_bb175_25, &phi_bb175_27, &phi_bb175_28, &phi_bb175_29, &phi_bb175_32, &phi_bb175_33, &phi_bb175_37);
    std::tie(tmp345, tmp346) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb175_29}).Flatten();
    tmp347 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp348 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb175_29}, TNode<IntPtrT>{tmp347});
    tmp349 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp350 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp348}, TNode<IntPtrT>{tmp349});
    tmp351 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block170, phi_bb175_20, phi_bb175_25, phi_bb175_27, phi_bb175_28, tmp350, tmp348, tmp351, phi_bb175_37, tmp345, tmp346);
  }

  TNode<IntPtrT> phi_bb170_20;
  TNode<IntPtrT> phi_bb170_25;
  TNode<IntPtrT> phi_bb170_27;
  TNode<IntPtrT> phi_bb170_28;
  TNode<IntPtrT> phi_bb170_29;
  TNode<IntPtrT> phi_bb170_32;
  TNode<BoolT> phi_bb170_33;
  TNode<BoolT> phi_bb170_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb170_39;
  TNode<IntPtrT> phi_bb170_40;
  if (block170.is_used()) {
    ca_.Bind(&block170, &phi_bb170_20, &phi_bb170_25, &phi_bb170_27, &phi_bb170_28, &phi_bb170_29, &phi_bb170_32, &phi_bb170_33, &phi_bb170_37, &phi_bb170_39, &phi_bb170_40);
    ca_.Goto(&block164, phi_bb170_20, phi_bb170_25, phi_bb170_27, phi_bb170_28, phi_bb170_29, phi_bb170_32, phi_bb170_33, phi_bb170_37, phi_bb170_39, phi_bb170_40);
  }

  TNode<IntPtrT> phi_bb168_20;
  TNode<IntPtrT> phi_bb168_25;
  TNode<IntPtrT> phi_bb168_27;
  TNode<IntPtrT> phi_bb168_28;
  TNode<IntPtrT> phi_bb168_29;
  TNode<IntPtrT> phi_bb168_32;
  TNode<BoolT> phi_bb168_33;
  TNode<BoolT> phi_bb168_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp352;
  TNode<IntPtrT> tmp353;
  TNode<IntPtrT> tmp354;
  TNode<IntPtrT> tmp355;
  TNode<BoolT> tmp356;
  if (block168.is_used()) {
    ca_.Bind(&block168, &phi_bb168_20, &phi_bb168_25, &phi_bb168_27, &phi_bb168_28, &phi_bb168_29, &phi_bb168_32, &phi_bb168_33, &phi_bb168_37);
    std::tie(tmp352, tmp353) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb168_29}).Flatten();
    tmp354 = FromConstexpr_intptr_constexpr_int31_0(state_, (CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull))), (SizeOf_intptr_0(state_)))));
    tmp355 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb168_29}, TNode<IntPtrT>{tmp354});
    tmp356 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block164, phi_bb168_20, phi_bb168_25, phi_bb168_27, phi_bb168_28, tmp355, phi_bb168_32, tmp356, phi_bb168_37, tmp352, tmp353);
  }

  TNode<IntPtrT> phi_bb164_20;
  TNode<IntPtrT> phi_bb164_25;
  TNode<IntPtrT> phi_bb164_27;
  TNode<IntPtrT> phi_bb164_28;
  TNode<IntPtrT> phi_bb164_29;
  TNode<IntPtrT> phi_bb164_32;
  TNode<BoolT> phi_bb164_33;
  TNode<BoolT> phi_bb164_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb164_39;
  TNode<IntPtrT> phi_bb164_40;
  TNode<Union<HeapObject, TaggedIndex>> tmp357;
  TNode<IntPtrT> tmp358;
  TNode<Float64T> tmp359;
  TNode<Union<HeapObject, TaggedIndex>> tmp360;
  TNode<IntPtrT> tmp361;
  TNode<IntPtrT> tmp362;
  TNode<IntPtrT> tmp363;
  TNode<IntPtrT> tmp364;
  TNode<UintPtrT> tmp365;
  TNode<UintPtrT> tmp366;
  TNode<BoolT> tmp367;
  if (block164.is_used()) {
    ca_.Bind(&block164, &phi_bb164_20, &phi_bb164_25, &phi_bb164_27, &phi_bb164_28, &phi_bb164_29, &phi_bb164_32, &phi_bb164_33, &phi_bb164_37, &phi_bb164_39, &phi_bb164_40);
    std::tie(tmp357, tmp358) = RefCast_float64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb164_39}, TNode<IntPtrT>{phi_bb164_40}, TorqueStructUnsafe_0{}}).Flatten();
    tmp359 = CodeStubAssembler(state_).LoadReference<Float64T>(CodeStubAssembler::Reference{tmp357, tmp358});
    std::tie(tmp360, tmp361, tmp362) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp63}).Flatten();
    tmp363 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp364 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb164_20}, TNode<IntPtrT>{tmp363});
    tmp365 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb164_20});
    tmp366 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp362});
    tmp367 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp365}, TNode<UintPtrT>{tmp366});
    ca_.Branch(tmp367, &block180, std::vector<compiler::Node*>{phi_bb164_25, phi_bb164_27, phi_bb164_28, phi_bb164_29, phi_bb164_32, phi_bb164_33, phi_bb164_37, phi_bb164_39, phi_bb164_40, phi_bb164_20, phi_bb164_20, phi_bb164_20, phi_bb164_20}, &block181, std::vector<compiler::Node*>{phi_bb164_25, phi_bb164_27, phi_bb164_28, phi_bb164_29, phi_bb164_32, phi_bb164_33, phi_bb164_37, phi_bb164_39, phi_bb164_40, phi_bb164_20, phi_bb164_20, phi_bb164_20, phi_bb164_20});
  }

  TNode<IntPtrT> phi_bb180_25;
  TNode<IntPtrT> phi_bb180_27;
  TNode<IntPtrT> phi_bb180_28;
  TNode<IntPtrT> phi_bb180_29;
  TNode<IntPtrT> phi_bb180_32;
  TNode<BoolT> phi_bb180_33;
  TNode<BoolT> phi_bb180_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb180_39;
  TNode<IntPtrT> phi_bb180_40;
  TNode<IntPtrT> phi_bb180_46;
  TNode<IntPtrT> phi_bb180_47;
  TNode<IntPtrT> phi_bb180_51;
  TNode<IntPtrT> phi_bb180_52;
  TNode<IntPtrT> tmp368;
  TNode<IntPtrT> tmp369;
  TNode<Union<HeapObject, TaggedIndex>> tmp370;
  TNode<IntPtrT> tmp371;
  TNode<Number> tmp372;
  if (block180.is_used()) {
    ca_.Bind(&block180, &phi_bb180_25, &phi_bb180_27, &phi_bb180_28, &phi_bb180_29, &phi_bb180_32, &phi_bb180_33, &phi_bb180_37, &phi_bb180_39, &phi_bb180_40, &phi_bb180_46, &phi_bb180_47, &phi_bb180_51, &phi_bb180_52);
    tmp368 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb180_52});
    tmp369 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp361}, TNode<IntPtrT>{tmp368});
    std::tie(tmp370, tmp371) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp360}, TNode<IntPtrT>{tmp369}).Flatten();
    tmp372 = Convert_Number_float64_0(state_, TNode<Float64T>{tmp359});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp370, tmp371}, tmp372);
    ca_.Goto(&block163, tmp364, phi_bb180_25, tmp328, phi_bb180_27, phi_bb180_28, phi_bb180_29, phi_bb180_32, phi_bb180_33, phi_bb180_37);
  }

  TNode<IntPtrT> phi_bb181_25;
  TNode<IntPtrT> phi_bb181_27;
  TNode<IntPtrT> phi_bb181_28;
  TNode<IntPtrT> phi_bb181_29;
  TNode<IntPtrT> phi_bb181_32;
  TNode<BoolT> phi_bb181_33;
  TNode<BoolT> phi_bb181_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb181_39;
  TNode<IntPtrT> phi_bb181_40;
  TNode<IntPtrT> phi_bb181_46;
  TNode<IntPtrT> phi_bb181_47;
  TNode<IntPtrT> phi_bb181_51;
  TNode<IntPtrT> phi_bb181_52;
  if (block181.is_used()) {
    ca_.Bind(&block181, &phi_bb181_25, &phi_bb181_27, &phi_bb181_28, &phi_bb181_29, &phi_bb181_32, &phi_bb181_33, &phi_bb181_37, &phi_bb181_39, &phi_bb181_40, &phi_bb181_46, &phi_bb181_47, &phi_bb181_51, &phi_bb181_52);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb162_20;
  TNode<IntPtrT> phi_bb162_25;
  TNode<IntPtrT> phi_bb162_26;
  TNode<IntPtrT> phi_bb162_27;
  TNode<IntPtrT> phi_bb162_28;
  TNode<IntPtrT> phi_bb162_29;
  TNode<IntPtrT> phi_bb162_32;
  TNode<BoolT> phi_bb162_33;
  TNode<BoolT> phi_bb162_37;
  TNode<Uint32T> tmp373;
  TNode<Uint32T> tmp374;
  TNode<Uint32T> tmp375;
  TNode<BoolT> tmp376;
  if (block162.is_used()) {
    ca_.Bind(&block162, &phi_bb162_20, &phi_bb162_25, &phi_bb162_26, &phi_bb162_27, &phi_bb162_28, &phi_bb162_29, &phi_bb162_32, &phi_bb162_33, &phi_bb162_37);
    tmp373 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp374 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp107}, TNode<Uint32T>{tmp373});
    tmp375 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp376 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp374}, TNode<Uint32T>{tmp375});
    ca_.Branch(tmp376, &block184, std::vector<compiler::Node*>{phi_bb162_20, phi_bb162_25, phi_bb162_26, phi_bb162_27, phi_bb162_28, phi_bb162_29, phi_bb162_32, phi_bb162_33, phi_bb162_37}, &block185, std::vector<compiler::Node*>{phi_bb162_20, phi_bb162_25, phi_bb162_26, phi_bb162_27, phi_bb162_28, phi_bb162_29, phi_bb162_32, phi_bb162_33, phi_bb162_37});
  }

  TNode<IntPtrT> phi_bb185_20;
  TNode<IntPtrT> phi_bb185_25;
  TNode<IntPtrT> phi_bb185_26;
  TNode<IntPtrT> phi_bb185_27;
  TNode<IntPtrT> phi_bb185_28;
  TNode<IntPtrT> phi_bb185_29;
  TNode<IntPtrT> phi_bb185_32;
  TNode<BoolT> phi_bb185_33;
  TNode<BoolT> phi_bb185_37;
  if (block185.is_used()) {
    ca_.Bind(&block185, &phi_bb185_20, &phi_bb185_25, &phi_bb185_26, &phi_bb185_27, &phi_bb185_28, &phi_bb185_29, &phi_bb185_32, &phi_bb185_33, &phi_bb185_37);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/wasm-to-js.tq", 158});
      CodeStubAssembler(state_).FailAssert("Torque assert '(paramType & kValueTypeIsRefBit) != 0' failed", pos_stack);
    }
  }

  TNode<IntPtrT> phi_bb184_20;
  TNode<IntPtrT> phi_bb184_25;
  TNode<IntPtrT> phi_bb184_26;
  TNode<IntPtrT> phi_bb184_27;
  TNode<IntPtrT> phi_bb184_28;
  TNode<IntPtrT> phi_bb184_29;
  TNode<IntPtrT> phi_bb184_32;
  TNode<BoolT> phi_bb184_33;
  TNode<BoolT> phi_bb184_37;
  TNode<IntPtrT> tmp377;
  TNode<IntPtrT> tmp378;
  TNode<BoolT> tmp379;
  if (block184.is_used()) {
    ca_.Bind(&block184, &phi_bb184_20, &phi_bb184_25, &phi_bb184_26, &phi_bb184_27, &phi_bb184_28, &phi_bb184_29, &phi_bb184_32, &phi_bb184_33, &phi_bb184_37);
    tmp377 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp378 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb184_20}, TNode<IntPtrT>{tmp377});
    tmp379 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block163, tmp378, phi_bb184_25, phi_bb184_26, phi_bb184_27, phi_bb184_28, phi_bb184_29, phi_bb184_32, phi_bb184_33, tmp379);
  }

  TNode<IntPtrT> phi_bb163_20;
  TNode<IntPtrT> phi_bb163_25;
  TNode<IntPtrT> phi_bb163_26;
  TNode<IntPtrT> phi_bb163_27;
  TNode<IntPtrT> phi_bb163_28;
  TNode<IntPtrT> phi_bb163_29;
  TNode<IntPtrT> phi_bb163_32;
  TNode<BoolT> phi_bb163_33;
  TNode<BoolT> phi_bb163_37;
  if (block163.is_used()) {
    ca_.Bind(&block163, &phi_bb163_20, &phi_bb163_25, &phi_bb163_26, &phi_bb163_27, &phi_bb163_28, &phi_bb163_29, &phi_bb163_32, &phi_bb163_33, &phi_bb163_37);
    ca_.Goto(&block114, phi_bb163_20, phi_bb163_25, phi_bb163_26, phi_bb163_27, phi_bb163_28, phi_bb163_29, phi_bb163_32, phi_bb163_33, phi_bb163_37);
  }

  TNode<IntPtrT> phi_bb114_20;
  TNode<IntPtrT> phi_bb114_25;
  TNode<IntPtrT> phi_bb114_26;
  TNode<IntPtrT> phi_bb114_27;
  TNode<IntPtrT> phi_bb114_28;
  TNode<IntPtrT> phi_bb114_29;
  TNode<IntPtrT> phi_bb114_32;
  TNode<BoolT> phi_bb114_33;
  TNode<BoolT> phi_bb114_37;
  if (block114.is_used()) {
    ca_.Bind(&block114, &phi_bb114_20, &phi_bb114_25, &phi_bb114_26, &phi_bb114_27, &phi_bb114_28, &phi_bb114_29, &phi_bb114_32, &phi_bb114_33, &phi_bb114_37);
    ca_.Goto(&block80, phi_bb114_20, phi_bb114_25, phi_bb114_26, phi_bb114_27, phi_bb114_28, phi_bb114_29, phi_bb114_32, phi_bb114_33, phi_bb114_37);
  }

  TNode<IntPtrT> phi_bb80_20;
  TNode<IntPtrT> phi_bb80_25;
  TNode<IntPtrT> phi_bb80_26;
  TNode<IntPtrT> phi_bb80_27;
  TNode<IntPtrT> phi_bb80_28;
  TNode<IntPtrT> phi_bb80_29;
  TNode<IntPtrT> phi_bb80_32;
  TNode<BoolT> phi_bb80_33;
  TNode<BoolT> phi_bb80_37;
  if (block80.is_used()) {
    ca_.Bind(&block80, &phi_bb80_20, &phi_bb80_25, &phi_bb80_26, &phi_bb80_27, &phi_bb80_28, &phi_bb80_29, &phi_bb80_32, &phi_bb80_33, &phi_bb80_37);
    ca_.Goto(&block57, phi_bb80_20, phi_bb80_25, phi_bb80_26, phi_bb80_27, phi_bb80_28, phi_bb80_29, phi_bb80_32, phi_bb80_33, phi_bb80_37);
  }

  TNode<IntPtrT> phi_bb57_20;
  TNode<IntPtrT> phi_bb57_25;
  TNode<IntPtrT> phi_bb57_26;
  TNode<IntPtrT> phi_bb57_27;
  TNode<IntPtrT> phi_bb57_28;
  TNode<IntPtrT> phi_bb57_29;
  TNode<IntPtrT> phi_bb57_32;
  TNode<BoolT> phi_bb57_33;
  TNode<BoolT> phi_bb57_37;
  if (block57.is_used()) {
    ca_.Bind(&block57, &phi_bb57_20, &phi_bb57_25, &phi_bb57_26, &phi_bb57_27, &phi_bb57_28, &phi_bb57_29, &phi_bb57_32, &phi_bb57_33, &phi_bb57_37);
    ca_.Goto(&block46, phi_bb57_20, phi_bb57_25, phi_bb57_26, phi_bb57_27, phi_bb57_28, phi_bb57_29, phi_bb57_32, phi_bb57_33, tmp106, phi_bb57_37);
  }

  TNode<IntPtrT> phi_bb45_20;
  TNode<IntPtrT> phi_bb45_25;
  TNode<IntPtrT> phi_bb45_26;
  TNode<IntPtrT> phi_bb45_27;
  TNode<IntPtrT> phi_bb45_28;
  TNode<IntPtrT> phi_bb45_29;
  TNode<IntPtrT> phi_bb45_32;
  TNode<BoolT> phi_bb45_33;
  TNode<IntPtrT> phi_bb45_35;
  TNode<BoolT> phi_bb45_37;
  if (block45.is_used()) {
    ca_.Bind(&block45, &phi_bb45_20, &phi_bb45_25, &phi_bb45_26, &phi_bb45_27, &phi_bb45_28, &phi_bb45_29, &phi_bb45_32, &phi_bb45_33, &phi_bb45_35, &phi_bb45_37);
    ca_.Branch(phi_bb45_37, &block186, std::vector<compiler::Node*>{phi_bb45_20, phi_bb45_25, phi_bb45_26, phi_bb45_27, phi_bb45_28, phi_bb45_29, phi_bb45_32, phi_bb45_33, phi_bb45_35, phi_bb45_37}, &block187, std::vector<compiler::Node*>{phi_bb45_20, phi_bb45_25, phi_bb45_26, phi_bb45_27, phi_bb45_28, phi_bb45_29, phi_bb45_32, phi_bb45_33, phi_bb45_35, tmp99, phi_bb45_37});
  }

  TNode<IntPtrT> phi_bb186_20;
  TNode<IntPtrT> phi_bb186_25;
  TNode<IntPtrT> phi_bb186_26;
  TNode<IntPtrT> phi_bb186_27;
  TNode<IntPtrT> phi_bb186_28;
  TNode<IntPtrT> phi_bb186_29;
  TNode<IntPtrT> phi_bb186_32;
  TNode<BoolT> phi_bb186_33;
  TNode<IntPtrT> phi_bb186_35;
  TNode<BoolT> phi_bb186_37;
  TNode<BoolT> tmp380;
  if (block186.is_used()) {
    ca_.Bind(&block186, &phi_bb186_20, &phi_bb186_25, &phi_bb186_26, &phi_bb186_27, &phi_bb186_28, &phi_bb186_29, &phi_bb186_32, &phi_bb186_33, &phi_bb186_35, &phi_bb186_37);
    tmp380 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{phi_bb186_33});
    ca_.Branch(tmp380, &block189, std::vector<compiler::Node*>{phi_bb186_20, phi_bb186_25, phi_bb186_26, phi_bb186_27, phi_bb186_28, phi_bb186_29, phi_bb186_32, phi_bb186_33, phi_bb186_35, phi_bb186_37}, &block190, std::vector<compiler::Node*>{phi_bb186_20, phi_bb186_25, phi_bb186_26, phi_bb186_27, phi_bb186_28, phi_bb186_29, phi_bb186_32, phi_bb186_33, phi_bb186_35, phi_bb186_37});
  }

  TNode<IntPtrT> phi_bb189_20;
  TNode<IntPtrT> phi_bb189_25;
  TNode<IntPtrT> phi_bb189_26;
  TNode<IntPtrT> phi_bb189_27;
  TNode<IntPtrT> phi_bb189_28;
  TNode<IntPtrT> phi_bb189_29;
  TNode<IntPtrT> phi_bb189_32;
  TNode<BoolT> phi_bb189_33;
  TNode<IntPtrT> phi_bb189_35;
  TNode<BoolT> phi_bb189_37;
  TNode<IntPtrT> tmp381;
  if (block189.is_used()) {
    ca_.Bind(&block189, &phi_bb189_20, &phi_bb189_25, &phi_bb189_26, &phi_bb189_27, &phi_bb189_28, &phi_bb189_29, &phi_bb189_32, &phi_bb189_33, &phi_bb189_35, &phi_bb189_37);
    tmp381 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block190, phi_bb189_20, phi_bb189_25, phi_bb189_26, phi_bb189_27, phi_bb189_28, phi_bb189_29, tmp381, phi_bb189_33, phi_bb189_35, phi_bb189_37);
  }

  TNode<IntPtrT> phi_bb190_20;
  TNode<IntPtrT> phi_bb190_25;
  TNode<IntPtrT> phi_bb190_26;
  TNode<IntPtrT> phi_bb190_27;
  TNode<IntPtrT> phi_bb190_28;
  TNode<IntPtrT> phi_bb190_29;
  TNode<IntPtrT> phi_bb190_32;
  TNode<BoolT> phi_bb190_33;
  TNode<IntPtrT> phi_bb190_35;
  TNode<BoolT> phi_bb190_37;
  TNode<IntPtrT> tmp382;
  TNode<IntPtrT> tmp383;
  TNode<IntPtrT> tmp384;
  if (block190.is_used()) {
    ca_.Bind(&block190, &phi_bb190_20, &phi_bb190_25, &phi_bb190_26, &phi_bb190_27, &phi_bb190_28, &phi_bb190_29, &phi_bb190_32, &phi_bb190_33, &phi_bb190_35, &phi_bb190_37);
    tmp382 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp383 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{tmp59});
    tmp384 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp58}, TNode<IntPtrT>{tmp383});
    ca_.Goto(&block194, tmp382, phi_bb190_25, phi_bb190_26, phi_bb190_27, phi_bb190_28, phi_bb190_29, phi_bb190_32, phi_bb190_33, tmp58, phi_bb190_37);
  }

  TNode<IntPtrT> phi_bb194_20;
  TNode<IntPtrT> phi_bb194_25;
  TNode<IntPtrT> phi_bb194_26;
  TNode<IntPtrT> phi_bb194_27;
  TNode<IntPtrT> phi_bb194_28;
  TNode<IntPtrT> phi_bb194_29;
  TNode<IntPtrT> phi_bb194_32;
  TNode<BoolT> phi_bb194_33;
  TNode<IntPtrT> phi_bb194_35;
  TNode<BoolT> phi_bb194_37;
  TNode<BoolT> tmp385;
  TNode<BoolT> tmp386;
  if (block194.is_used()) {
    ca_.Bind(&block194, &phi_bb194_20, &phi_bb194_25, &phi_bb194_26, &phi_bb194_27, &phi_bb194_28, &phi_bb194_29, &phi_bb194_32, &phi_bb194_33, &phi_bb194_35, &phi_bb194_37);
    tmp385 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb194_35}, TNode<IntPtrT>{tmp384});
    tmp386 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp385});
    ca_.Branch(tmp386, &block192, std::vector<compiler::Node*>{phi_bb194_20, phi_bb194_25, phi_bb194_26, phi_bb194_27, phi_bb194_28, phi_bb194_29, phi_bb194_32, phi_bb194_33, phi_bb194_35, phi_bb194_37}, &block193, std::vector<compiler::Node*>{phi_bb194_20, phi_bb194_25, phi_bb194_26, phi_bb194_27, phi_bb194_28, phi_bb194_29, phi_bb194_32, phi_bb194_33, phi_bb194_35, phi_bb194_37});
  }

  TNode<IntPtrT> phi_bb192_20;
  TNode<IntPtrT> phi_bb192_25;
  TNode<IntPtrT> phi_bb192_26;
  TNode<IntPtrT> phi_bb192_27;
  TNode<IntPtrT> phi_bb192_28;
  TNode<IntPtrT> phi_bb192_29;
  TNode<IntPtrT> phi_bb192_32;
  TNode<BoolT> phi_bb192_33;
  TNode<IntPtrT> phi_bb192_35;
  TNode<BoolT> phi_bb192_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp387;
  TNode<IntPtrT> tmp388;
  TNode<IntPtrT> tmp389;
  TNode<IntPtrT> tmp390;
  TNode<Uint32T> tmp391;
  TNode<Uint32T> tmp392;
  TNode<Uint32T> tmp393;
  TNode<Uint32T> tmp394;
  TNode<BoolT> tmp395;
  if (block192.is_used()) {
    ca_.Bind(&block192, &phi_bb192_20, &phi_bb192_25, &phi_bb192_26, &phi_bb192_27, &phi_bb192_28, &phi_bb192_29, &phi_bb192_32, &phi_bb192_33, &phi_bb192_35, &phi_bb192_37);
    std::tie(tmp387, tmp388) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp57}, TNode<IntPtrT>{phi_bb192_35}).Flatten();
    tmp389 = FromConstexpr_intptr_constexpr_int31_0(state_, kInt32Size);
    tmp390 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb192_35}, TNode<IntPtrT>{tmp389});
    tmp391 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp387, tmp388});
    tmp392 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp393 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp391}, TNode<Uint32T>{tmp392});
    tmp394 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp395 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp393}, TNode<Uint32T>{tmp394});
    ca_.Branch(tmp395, &block203, std::vector<compiler::Node*>{phi_bb192_20, phi_bb192_25, phi_bb192_26, phi_bb192_27, phi_bb192_28, phi_bb192_29, phi_bb192_32, phi_bb192_33, phi_bb192_37}, &block204, std::vector<compiler::Node*>{phi_bb192_20, phi_bb192_25, phi_bb192_26, phi_bb192_27, phi_bb192_28, phi_bb192_29, phi_bb192_32, phi_bb192_33, phi_bb192_37});
  }

  TNode<IntPtrT> phi_bb203_20;
  TNode<IntPtrT> phi_bb203_25;
  TNode<IntPtrT> phi_bb203_26;
  TNode<IntPtrT> phi_bb203_27;
  TNode<IntPtrT> phi_bb203_28;
  TNode<IntPtrT> phi_bb203_29;
  TNode<IntPtrT> phi_bb203_32;
  TNode<BoolT> phi_bb203_33;
  TNode<BoolT> phi_bb203_37;
  TNode<IntPtrT> tmp396;
  TNode<IntPtrT> tmp397;
  TNode<IntPtrT> tmp398;
  TNode<BoolT> tmp399;
  if (block203.is_used()) {
    ca_.Bind(&block203, &phi_bb203_20, &phi_bb203_25, &phi_bb203_26, &phi_bb203_27, &phi_bb203_28, &phi_bb203_29, &phi_bb203_32, &phi_bb203_33, &phi_bb203_37);
    tmp396 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp397 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb203_25}, TNode<IntPtrT>{tmp396});
    tmp398 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp399 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb203_25}, TNode<IntPtrT>{tmp398});
    ca_.Branch(tmp399, &block206, std::vector<compiler::Node*>{phi_bb203_20, phi_bb203_26, phi_bb203_27, phi_bb203_28, phi_bb203_29, phi_bb203_32, phi_bb203_33, phi_bb203_37}, &block207, std::vector<compiler::Node*>{phi_bb203_20, phi_bb203_26, phi_bb203_27, phi_bb203_28, phi_bb203_29, phi_bb203_32, phi_bb203_33, phi_bb203_37});
  }

  TNode<IntPtrT> phi_bb206_20;
  TNode<IntPtrT> phi_bb206_26;
  TNode<IntPtrT> phi_bb206_27;
  TNode<IntPtrT> phi_bb206_28;
  TNode<IntPtrT> phi_bb206_29;
  TNode<IntPtrT> phi_bb206_32;
  TNode<BoolT> phi_bb206_33;
  TNode<BoolT> phi_bb206_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp400;
  TNode<IntPtrT> tmp401;
  TNode<IntPtrT> tmp402;
  TNode<IntPtrT> tmp403;
  if (block206.is_used()) {
    ca_.Bind(&block206, &phi_bb206_20, &phi_bb206_26, &phi_bb206_27, &phi_bb206_28, &phi_bb206_29, &phi_bb206_32, &phi_bb206_33, &phi_bb206_37);
    std::tie(tmp400, tmp401) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb206_27}).Flatten();
    tmp402 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp403 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb206_27}, TNode<IntPtrT>{tmp402});
    ca_.Goto(&block205, phi_bb206_20, phi_bb206_26, tmp403, phi_bb206_28, phi_bb206_29, phi_bb206_32, phi_bb206_33, phi_bb206_37, tmp400, tmp401);
  }

  TNode<IntPtrT> phi_bb207_20;
  TNode<IntPtrT> phi_bb207_26;
  TNode<IntPtrT> phi_bb207_27;
  TNode<IntPtrT> phi_bb207_28;
  TNode<IntPtrT> phi_bb207_29;
  TNode<IntPtrT> phi_bb207_32;
  TNode<BoolT> phi_bb207_33;
  TNode<BoolT> phi_bb207_37;
  if (block207.is_used()) {
    ca_.Bind(&block207, &phi_bb207_20, &phi_bb207_26, &phi_bb207_27, &phi_bb207_28, &phi_bb207_29, &phi_bb207_32, &phi_bb207_33, &phi_bb207_37);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block209, phi_bb207_20, phi_bb207_26, phi_bb207_27, phi_bb207_28, phi_bb207_29, phi_bb207_32, phi_bb207_33, phi_bb207_37);
    } else {
      ca_.Goto(&block210, phi_bb207_20, phi_bb207_26, phi_bb207_27, phi_bb207_28, phi_bb207_29, phi_bb207_32, phi_bb207_33, phi_bb207_37);
    }
  }

  TNode<IntPtrT> phi_bb209_20;
  TNode<IntPtrT> phi_bb209_26;
  TNode<IntPtrT> phi_bb209_27;
  TNode<IntPtrT> phi_bb209_28;
  TNode<IntPtrT> phi_bb209_29;
  TNode<IntPtrT> phi_bb209_32;
  TNode<BoolT> phi_bb209_33;
  TNode<BoolT> phi_bb209_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp404;
  TNode<IntPtrT> tmp405;
  TNode<IntPtrT> tmp406;
  TNode<IntPtrT> tmp407;
  if (block209.is_used()) {
    ca_.Bind(&block209, &phi_bb209_20, &phi_bb209_26, &phi_bb209_27, &phi_bb209_28, &phi_bb209_29, &phi_bb209_32, &phi_bb209_33, &phi_bb209_37);
    std::tie(tmp404, tmp405) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb209_29}).Flatten();
    tmp406 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp407 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb209_29}, TNode<IntPtrT>{tmp406});
    ca_.Goto(&block208, phi_bb209_20, phi_bb209_26, phi_bb209_27, phi_bb209_28, tmp407, phi_bb209_32, phi_bb209_33, phi_bb209_37, tmp404, tmp405);
  }

  TNode<IntPtrT> phi_bb210_20;
  TNode<IntPtrT> phi_bb210_26;
  TNode<IntPtrT> phi_bb210_27;
  TNode<IntPtrT> phi_bb210_28;
  TNode<IntPtrT> phi_bb210_29;
  TNode<IntPtrT> phi_bb210_32;
  TNode<BoolT> phi_bb210_33;
  TNode<BoolT> phi_bb210_37;
  TNode<IntPtrT> tmp408;
  TNode<BoolT> tmp409;
  if (block210.is_used()) {
    ca_.Bind(&block210, &phi_bb210_20, &phi_bb210_26, &phi_bb210_27, &phi_bb210_28, &phi_bb210_29, &phi_bb210_32, &phi_bb210_33, &phi_bb210_37);
    tmp408 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp409 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb210_32}, TNode<IntPtrT>{tmp408});
    ca_.Branch(tmp409, &block212, std::vector<compiler::Node*>{phi_bb210_20, phi_bb210_26, phi_bb210_27, phi_bb210_28, phi_bb210_29, phi_bb210_32, phi_bb210_33, phi_bb210_37}, &block213, std::vector<compiler::Node*>{phi_bb210_20, phi_bb210_26, phi_bb210_27, phi_bb210_28, phi_bb210_29, phi_bb210_32, phi_bb210_33, phi_bb210_37});
  }

  TNode<IntPtrT> phi_bb212_20;
  TNode<IntPtrT> phi_bb212_26;
  TNode<IntPtrT> phi_bb212_27;
  TNode<IntPtrT> phi_bb212_28;
  TNode<IntPtrT> phi_bb212_29;
  TNode<IntPtrT> phi_bb212_32;
  TNode<BoolT> phi_bb212_33;
  TNode<BoolT> phi_bb212_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp410;
  TNode<IntPtrT> tmp411;
  TNode<IntPtrT> tmp412;
  TNode<BoolT> tmp413;
  if (block212.is_used()) {
    ca_.Bind(&block212, &phi_bb212_20, &phi_bb212_26, &phi_bb212_27, &phi_bb212_28, &phi_bb212_29, &phi_bb212_32, &phi_bb212_33, &phi_bb212_37);
    std::tie(tmp410, tmp411) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb212_32}).Flatten();
    tmp412 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp413 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block208, phi_bb212_20, phi_bb212_26, phi_bb212_27, phi_bb212_28, phi_bb212_29, tmp412, tmp413, phi_bb212_37, tmp410, tmp411);
  }

  TNode<IntPtrT> phi_bb213_20;
  TNode<IntPtrT> phi_bb213_26;
  TNode<IntPtrT> phi_bb213_27;
  TNode<IntPtrT> phi_bb213_28;
  TNode<IntPtrT> phi_bb213_29;
  TNode<IntPtrT> phi_bb213_32;
  TNode<BoolT> phi_bb213_33;
  TNode<BoolT> phi_bb213_37;
  TNode<Union<HeapObject, TaggedIndex>> tmp414;
  TNode<IntPtrT> tmp415;
  TNode<IntPtrT> tmp416;
  TNode<IntPtrT> tmp417;
  TNode<IntPtrT> tmp418;
  TNode<IntPtrT> tmp419;
  TNode<BoolT> tmp420;
  if (block213.is_used()) {
    ca_.Bind(&block213, &phi_bb213_20, &phi_bb213_26, &phi_bb213_27, &phi_bb213_28, &phi_bb213_29, &phi_bb213_32, &phi_bb213_33, &phi_bb213_37);
    std::tie(tmp414, tmp415) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb213_29}).Flatten();
    tmp416 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp417 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb213_29}, TNode<IntPtrT>{tmp416});
    tmp418 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp419 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp417}, TNode<IntPtrT>{tmp418});
    tmp420 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block208, phi_bb213_20, phi_bb213_26, phi_bb213_27, phi_bb213_28, tmp419, tmp417, tmp420, phi_bb213_37, tmp414, tmp415);
  }

  TNode<IntPtrT> phi_bb208_20;
  TNode<IntPtrT> phi_bb208_26;
  TNode<IntPtrT> phi_bb208_27;
  TNode<IntPtrT> phi_bb208_28;
  TNode<IntPtrT> phi_bb208_29;
  TNode<IntPtrT> phi_bb208_32;
  TNode<BoolT> phi_bb208_33;
  TNode<BoolT> phi_bb208_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb208_39;
  TNode<IntPtrT> phi_bb208_40;
  if (block208.is_used()) {
    ca_.Bind(&block208, &phi_bb208_20, &phi_bb208_26, &phi_bb208_27, &phi_bb208_28, &phi_bb208_29, &phi_bb208_32, &phi_bb208_33, &phi_bb208_37, &phi_bb208_39, &phi_bb208_40);
    ca_.Goto(&block205, phi_bb208_20, phi_bb208_26, phi_bb208_27, phi_bb208_28, phi_bb208_29, phi_bb208_32, phi_bb208_33, phi_bb208_37, phi_bb208_39, phi_bb208_40);
  }

  TNode<IntPtrT> phi_bb205_20;
  TNode<IntPtrT> phi_bb205_26;
  TNode<IntPtrT> phi_bb205_27;
  TNode<IntPtrT> phi_bb205_28;
  TNode<IntPtrT> phi_bb205_29;
  TNode<IntPtrT> phi_bb205_32;
  TNode<BoolT> phi_bb205_33;
  TNode<BoolT> phi_bb205_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb205_39;
  TNode<IntPtrT> phi_bb205_40;
  TNode<IntPtrT> tmp421;
  TNode<Object> tmp422;
  TNode<Union<HeapObject, TaggedIndex>> tmp423;
  TNode<IntPtrT> tmp424;
  TNode<IntPtrT> tmp425;
  TNode<UintPtrT> tmp426;
  TNode<UintPtrT> tmp427;
  TNode<BoolT> tmp428;
  if (block205.is_used()) {
    ca_.Bind(&block205, &phi_bb205_20, &phi_bb205_26, &phi_bb205_27, &phi_bb205_28, &phi_bb205_29, &phi_bb205_32, &phi_bb205_33, &phi_bb205_37, &phi_bb205_39, &phi_bb205_40);
    tmp421 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb205_39, phi_bb205_40});
    tmp422 = CodeStubAssembler(state_).BitcastWordToTagged(TNode<IntPtrT>{tmp421});
    std::tie(tmp423, tmp424, tmp425) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{tmp63}).Flatten();
    tmp426 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb205_20});
    tmp427 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp425});
    tmp428 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp426}, TNode<UintPtrT>{tmp427});
    ca_.Branch(tmp428, &block218, std::vector<compiler::Node*>{phi_bb205_20, phi_bb205_26, phi_bb205_27, phi_bb205_28, phi_bb205_29, phi_bb205_32, phi_bb205_33, phi_bb205_37, phi_bb205_39, phi_bb205_40, phi_bb205_20, phi_bb205_20, phi_bb205_20, phi_bb205_20}, &block219, std::vector<compiler::Node*>{phi_bb205_20, phi_bb205_26, phi_bb205_27, phi_bb205_28, phi_bb205_29, phi_bb205_32, phi_bb205_33, phi_bb205_37, phi_bb205_39, phi_bb205_40, phi_bb205_20, phi_bb205_20, phi_bb205_20, phi_bb205_20});
  }

  TNode<IntPtrT> phi_bb218_20;
  TNode<IntPtrT> phi_bb218_26;
  TNode<IntPtrT> phi_bb218_27;
  TNode<IntPtrT> phi_bb218_28;
  TNode<IntPtrT> phi_bb218_29;
  TNode<IntPtrT> phi_bb218_32;
  TNode<BoolT> phi_bb218_33;
  TNode<BoolT> phi_bb218_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb218_39;
  TNode<IntPtrT> phi_bb218_40;
  TNode<IntPtrT> phi_bb218_47;
  TNode<IntPtrT> phi_bb218_48;
  TNode<IntPtrT> phi_bb218_52;
  TNode<IntPtrT> phi_bb218_53;
  TNode<IntPtrT> tmp429;
  TNode<IntPtrT> tmp430;
  TNode<Union<HeapObject, TaggedIndex>> tmp431;
  TNode<IntPtrT> tmp432;
  TNode<IntPtrT> tmp433;
  TNode<NativeContext> tmp434;
  TNode<JSAny> tmp435;
  if (block218.is_used()) {
    ca_.Bind(&block218, &phi_bb218_20, &phi_bb218_26, &phi_bb218_27, &phi_bb218_28, &phi_bb218_29, &phi_bb218_32, &phi_bb218_33, &phi_bb218_37, &phi_bb218_39, &phi_bb218_40, &phi_bb218_47, &phi_bb218_48, &phi_bb218_52, &phi_bb218_53);
    tmp429 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb218_53});
    tmp430 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp424}, TNode<IntPtrT>{tmp429});
    std::tie(tmp431, tmp432) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp423}, TNode<IntPtrT>{tmp430}).Flatten();
    tmp433 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp434 = CodeStubAssembler(state_).LoadReference<NativeContext>(CodeStubAssembler::Reference{p_data, tmp433});
    tmp435 = WasmToJSObject_0(state_, TNode<NativeContext>{tmp434}, TNode<Object>{tmp422});
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp431, tmp432}, tmp435);
    ca_.Goto(&block204, phi_bb218_20, tmp397, phi_bb218_26, phi_bb218_27, phi_bb218_28, phi_bb218_29, phi_bb218_32, phi_bb218_33, phi_bb218_37);
  }

  TNode<IntPtrT> phi_bb219_20;
  TNode<IntPtrT> phi_bb219_26;
  TNode<IntPtrT> phi_bb219_27;
  TNode<IntPtrT> phi_bb219_28;
  TNode<IntPtrT> phi_bb219_29;
  TNode<IntPtrT> phi_bb219_32;
  TNode<BoolT> phi_bb219_33;
  TNode<BoolT> phi_bb219_37;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb219_39;
  TNode<IntPtrT> phi_bb219_40;
  TNode<IntPtrT> phi_bb219_47;
  TNode<IntPtrT> phi_bb219_48;
  TNode<IntPtrT> phi_bb219_52;
  TNode<IntPtrT> phi_bb219_53;
  if (block219.is_used()) {
    ca_.Bind(&block219, &phi_bb219_20, &phi_bb219_26, &phi_bb219_27, &phi_bb219_28, &phi_bb219_29, &phi_bb219_32, &phi_bb219_33, &phi_bb219_37, &phi_bb219_39, &phi_bb219_40, &phi_bb219_47, &phi_bb219_48, &phi_bb219_52, &phi_bb219_53);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb204_20;
  TNode<IntPtrT> phi_bb204_25;
  TNode<IntPtrT> phi_bb204_26;
  TNode<IntPtrT> phi_bb204_27;
  TNode<IntPtrT> phi_bb204_28;
  TNode<IntPtrT> phi_bb204_29;
  TNode<IntPtrT> phi_bb204_32;
  TNode<BoolT> phi_bb204_33;
  TNode<BoolT> phi_bb204_37;
  TNode<IntPtrT> tmp436;
  TNode<IntPtrT> tmp437;
  if (block204.is_used()) {
    ca_.Bind(&block204, &phi_bb204_20, &phi_bb204_25, &phi_bb204_26, &phi_bb204_27, &phi_bb204_28, &phi_bb204_29, &phi_bb204_32, &phi_bb204_33, &phi_bb204_37);
    tmp436 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp437 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb204_20}, TNode<IntPtrT>{tmp436});
    ca_.Goto(&block194, tmp437, phi_bb204_25, phi_bb204_26, phi_bb204_27, phi_bb204_28, phi_bb204_29, phi_bb204_32, phi_bb204_33, tmp390, phi_bb204_37);
  }

  TNode<IntPtrT> phi_bb193_20;
  TNode<IntPtrT> phi_bb193_25;
  TNode<IntPtrT> phi_bb193_26;
  TNode<IntPtrT> phi_bb193_27;
  TNode<IntPtrT> phi_bb193_28;
  TNode<IntPtrT> phi_bb193_29;
  TNode<IntPtrT> phi_bb193_32;
  TNode<BoolT> phi_bb193_33;
  TNode<IntPtrT> phi_bb193_35;
  TNode<BoolT> phi_bb193_37;
  if (block193.is_used()) {
    ca_.Bind(&block193, &phi_bb193_20, &phi_bb193_25, &phi_bb193_26, &phi_bb193_27, &phi_bb193_28, &phi_bb193_29, &phi_bb193_32, &phi_bb193_33, &phi_bb193_35, &phi_bb193_37);
    ca_.Goto(&block187, phi_bb193_20, phi_bb193_25, phi_bb193_26, phi_bb193_27, phi_bb193_28, phi_bb193_29, phi_bb193_32, phi_bb193_33, phi_bb193_35, tmp384, phi_bb193_37);
  }

  TNode<IntPtrT> phi_bb187_20;
  TNode<IntPtrT> phi_bb187_25;
  TNode<IntPtrT> phi_bb187_26;
  TNode<IntPtrT> phi_bb187_27;
  TNode<IntPtrT> phi_bb187_28;
  TNode<IntPtrT> phi_bb187_29;
  TNode<IntPtrT> phi_bb187_32;
  TNode<BoolT> phi_bb187_33;
  TNode<IntPtrT> phi_bb187_35;
  TNode<IntPtrT> phi_bb187_36;
  TNode<BoolT> phi_bb187_37;
  TNode<IntPtrT> tmp438;
  TNode<Union<JSReceiver, Undefined>> tmp439;
  TNode<IntPtrT> tmp440;
  TNode<NativeContext> tmp441;
  TNode<IntPtrT> tmp442;
  TNode<Union<HeapObject, TaggedIndex>> tmp443;
  TNode<IntPtrT> tmp444;
  TNode<IntPtrT> tmp445;
  TNode<Int32T> tmp446;
  TNode<Int32T> tmp447;
  TNode<JSAny> tmp448;
  TNode<IntPtrT> tmp449;
  TNode<Union<HeapObject, TaggedIndex>> tmp450;
  TNode<IntPtrT> tmp451;
  TNode<IntPtrT> tmp452;
  TNode<IntPtrT> tmp453;
  TNode<BoolT> tmp454;
  if (block187.is_used()) {
    ca_.Bind(&block187, &phi_bb187_20, &phi_bb187_25, &phi_bb187_26, &phi_bb187_27, &phi_bb187_28, &phi_bb187_29, &phi_bb187_32, &phi_bb187_33, &phi_bb187_35, &phi_bb187_36, &phi_bb187_37);
    tmp438 = FromConstexpr_intptr_constexpr_int31_0(state_, 16);
    tmp439 = CodeStubAssembler(state_).LoadReference<Union<JSReceiver, Undefined>>(CodeStubAssembler::Reference{p_data, tmp438});
    tmp440 = FromConstexpr_intptr_constexpr_int31_0(state_, 12);
    tmp441 = CodeStubAssembler(state_).LoadReference<NativeContext>(CodeStubAssembler::Reference{p_data, tmp440});
    tmp442 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp443, tmp444) = GetRefAt_intptr_RawPtr_0(state_, TNode<RawPtrT>{tmp3}, TNode<IntPtrT>{tmp442}).Flatten();
    tmp445 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{tmp443, tmp444}, tmp445);
    tmp446 = Convert_int32_intptr_0(state_, TNode<IntPtrT>{tmp62});
    tmp447 = FromConstexpr_int32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp448 = ca_.CallBuiltin<JSAny>(Builtin::kCallVarargs, tmp441, tmp439, tmp447, tmp446, tmp63);
    tmp449 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp450, tmp451) = GetRefAt_intptr_RawPtr_0(state_, TNode<RawPtrT>{tmp3}, TNode<IntPtrT>{tmp449}).Flatten();
    tmp452 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(true, 0x1ull));
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{tmp450, tmp451}, tmp452);
    tmp453 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp454 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp31}, TNode<IntPtrT>{tmp453});
    ca_.Branch(tmp454, &block222, std::vector<compiler::Node*>{phi_bb187_20, phi_bb187_25, phi_bb187_26, phi_bb187_27, phi_bb187_28, phi_bb187_29, phi_bb187_32, phi_bb187_33, phi_bb187_35, phi_bb187_36, phi_bb187_37}, &block223, std::vector<compiler::Node*>{phi_bb187_20, phi_bb187_25, phi_bb187_26, phi_bb187_27, phi_bb187_28, phi_bb187_29, phi_bb187_32, phi_bb187_33, phi_bb187_35, phi_bb187_36, phi_bb187_37});
  }

  TNode<IntPtrT> phi_bb222_20;
  TNode<IntPtrT> phi_bb222_25;
  TNode<IntPtrT> phi_bb222_26;
  TNode<IntPtrT> phi_bb222_27;
  TNode<IntPtrT> phi_bb222_28;
  TNode<IntPtrT> phi_bb222_29;
  TNode<IntPtrT> phi_bb222_32;
  TNode<BoolT> phi_bb222_33;
  TNode<IntPtrT> phi_bb222_35;
  TNode<IntPtrT> phi_bb222_36;
  TNode<BoolT> phi_bb222_37;
  TNode<Smi> tmp455;
  TNode<FixedArray> tmp456;
  if (block222.is_used()) {
    ca_.Bind(&block222, &phi_bb222_20, &phi_bb222_25, &phi_bb222_26, &phi_bb222_27, &phi_bb222_28, &phi_bb222_29, &phi_bb222_32, &phi_bb222_33, &phi_bb222_35, &phi_bb222_36, &phi_bb222_37);
    tmp455 = Convert_Smi_intptr_0(state_, TNode<IntPtrT>{tmp31});
    tmp456 = ca_.CallBuiltin<FixedArray>(Builtin::kIterableToFixedArrayForWasm, tmp441, tmp448, tmp455);
    ca_.Goto(&block224, phi_bb222_20, phi_bb222_25, phi_bb222_26, phi_bb222_27, phi_bb222_28, phi_bb222_29, phi_bb222_32, phi_bb222_33, phi_bb222_35, phi_bb222_36, phi_bb222_37, tmp456);
  }

  TNode<IntPtrT> phi_bb223_20;
  TNode<IntPtrT> phi_bb223_25;
  TNode<IntPtrT> phi_bb223_26;
  TNode<IntPtrT> phi_bb223_27;
  TNode<IntPtrT> phi_bb223_28;
  TNode<IntPtrT> phi_bb223_29;
  TNode<IntPtrT> phi_bb223_32;
  TNode<BoolT> phi_bb223_33;
  TNode<IntPtrT> phi_bb223_35;
  TNode<IntPtrT> phi_bb223_36;
  TNode<BoolT> phi_bb223_37;
  TNode<FixedArray> tmp457;
  if (block223.is_used()) {
    ca_.Bind(&block223, &phi_bb223_20, &phi_bb223_25, &phi_bb223_26, &phi_bb223_27, &phi_bb223_28, &phi_bb223_29, &phi_bb223_32, &phi_bb223_33, &phi_bb223_35, &phi_bb223_36, &phi_bb223_37);
    tmp457 = kEmptyFixedArray_0(state_);
    ca_.Goto(&block224, phi_bb223_20, phi_bb223_25, phi_bb223_26, phi_bb223_27, phi_bb223_28, phi_bb223_29, phi_bb223_32, phi_bb223_33, phi_bb223_35, phi_bb223_36, phi_bb223_37, tmp457);
  }

  TNode<IntPtrT> phi_bb224_20;
  TNode<IntPtrT> phi_bb224_25;
  TNode<IntPtrT> phi_bb224_26;
  TNode<IntPtrT> phi_bb224_27;
  TNode<IntPtrT> phi_bb224_28;
  TNode<IntPtrT> phi_bb224_29;
  TNode<IntPtrT> phi_bb224_32;
  TNode<BoolT> phi_bb224_33;
  TNode<IntPtrT> phi_bb224_35;
  TNode<IntPtrT> phi_bb224_36;
  TNode<BoolT> phi_bb224_37;
  TNode<FixedArray> phi_bb224_41;
  TNode<RawPtrT> tmp458;
  TNode<RawPtrT> tmp459;
  TNode<RawPtrT> tmp460;
  TNode<RawPtrT> tmp461;
  TNode<IntPtrT> tmp462;
  if (block224.is_used()) {
    ca_.Bind(&block224, &phi_bb224_20, &phi_bb224_25, &phi_bb224_26, &phi_bb224_27, &phi_bb224_28, &phi_bb224_29, &phi_bb224_32, &phi_bb224_33, &phi_bb224_35, &phi_bb224_36, &phi_bb224_37, &phi_bb224_41);
    tmp458 = CodeStubAssembler(state_).StackSlotPtr(CastIfEnumClass<int32_t>((CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull))), (SizeOf_intptr_0(state_))))), CastIfEnumClass<int32_t>((SizeOf_intptr_0(state_))));
    tmp459 = (TNode<RawPtrT>{tmp458});
    tmp460 = CodeStubAssembler(state_).StackSlotPtr(CastIfEnumClass<int32_t>((CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull))), (SizeOf_float64_0(state_))))), CastIfEnumClass<int32_t>((SizeOf_float64_0(state_))));
    tmp461 = (TNode<RawPtrT>{tmp460});
    tmp462 = CodeStubAssembler(state_).StackAlignmentInBytes();
    ca_.Branch(phi_bb224_33, &block226, std::vector<compiler::Node*>{phi_bb224_20, phi_bb224_25, phi_bb224_26, phi_bb224_27, phi_bb224_28, phi_bb224_29, phi_bb224_32, phi_bb224_33, phi_bb224_35, phi_bb224_36, phi_bb224_37, phi_bb224_29}, &block227, std::vector<compiler::Node*>{phi_bb224_20, phi_bb224_25, phi_bb224_26, phi_bb224_27, phi_bb224_28, phi_bb224_29, phi_bb224_32, phi_bb224_33, phi_bb224_35, phi_bb224_36, phi_bb224_37, phi_bb224_29});
  }

  TNode<IntPtrT> phi_bb226_20;
  TNode<IntPtrT> phi_bb226_25;
  TNode<IntPtrT> phi_bb226_26;
  TNode<IntPtrT> phi_bb226_27;
  TNode<IntPtrT> phi_bb226_28;
  TNode<IntPtrT> phi_bb226_29;
  TNode<IntPtrT> phi_bb226_32;
  TNode<BoolT> phi_bb226_33;
  TNode<IntPtrT> phi_bb226_35;
  TNode<IntPtrT> phi_bb226_36;
  TNode<BoolT> phi_bb226_37;
  TNode<IntPtrT> phi_bb226_46;
  TNode<IntPtrT> tmp463;
  TNode<IntPtrT> tmp464;
  if (block226.is_used()) {
    ca_.Bind(&block226, &phi_bb226_20, &phi_bb226_25, &phi_bb226_26, &phi_bb226_27, &phi_bb226_28, &phi_bb226_29, &phi_bb226_32, &phi_bb226_33, &phi_bb226_35, &phi_bb226_36, &phi_bb226_37, &phi_bb226_46);
    tmp463 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp464 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb226_46}, TNode<IntPtrT>{tmp463});
    ca_.Goto(&block227, phi_bb226_20, phi_bb226_25, phi_bb226_26, phi_bb226_27, phi_bb226_28, phi_bb226_29, phi_bb226_32, phi_bb226_33, phi_bb226_35, phi_bb226_36, phi_bb226_37, tmp464);
  }

  TNode<IntPtrT> phi_bb227_20;
  TNode<IntPtrT> phi_bb227_25;
  TNode<IntPtrT> phi_bb227_26;
  TNode<IntPtrT> phi_bb227_27;
  TNode<IntPtrT> phi_bb227_28;
  TNode<IntPtrT> phi_bb227_29;
  TNode<IntPtrT> phi_bb227_32;
  TNode<BoolT> phi_bb227_33;
  TNode<IntPtrT> phi_bb227_35;
  TNode<IntPtrT> phi_bb227_36;
  TNode<BoolT> phi_bb227_37;
  TNode<IntPtrT> phi_bb227_46;
  TNode<IntPtrT> tmp465;
  TNode<IntPtrT> tmp466;
  TNode<IntPtrT> tmp467;
  TNode<BoolT> tmp468;
  if (block227.is_used()) {
    ca_.Bind(&block227, &phi_bb227_20, &phi_bb227_25, &phi_bb227_26, &phi_bb227_27, &phi_bb227_28, &phi_bb227_29, &phi_bb227_32, &phi_bb227_33, &phi_bb227_35, &phi_bb227_36, &phi_bb227_37, &phi_bb227_46);
    tmp465 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb227_46}, TNode<IntPtrT>{tmp94});
    tmp466 = CodeStubAssembler(state_).IntPtrMod(TNode<IntPtrT>{tmp465}, TNode<IntPtrT>{tmp462});
    tmp467 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp468 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{tmp466}, TNode<IntPtrT>{tmp467});
    ca_.Branch(tmp468, &block228, std::vector<compiler::Node*>{phi_bb227_20, phi_bb227_25, phi_bb227_26, phi_bb227_27, phi_bb227_28, phi_bb227_29, phi_bb227_32, phi_bb227_33, phi_bb227_35, phi_bb227_36, phi_bb227_37}, &block229, std::vector<compiler::Node*>{phi_bb227_20, phi_bb227_25, phi_bb227_26, phi_bb227_27, phi_bb227_28, phi_bb227_29, phi_bb227_32, phi_bb227_33, phi_bb227_35, phi_bb227_36, phi_bb227_37, phi_bb227_46});
  }

  TNode<IntPtrT> phi_bb228_20;
  TNode<IntPtrT> phi_bb228_25;
  TNode<IntPtrT> phi_bb228_26;
  TNode<IntPtrT> phi_bb228_27;
  TNode<IntPtrT> phi_bb228_28;
  TNode<IntPtrT> phi_bb228_29;
  TNode<IntPtrT> phi_bb228_32;
  TNode<BoolT> phi_bb228_33;
  TNode<IntPtrT> phi_bb228_35;
  TNode<IntPtrT> phi_bb228_36;
  TNode<BoolT> phi_bb228_37;
  TNode<IntPtrT> tmp469;
  TNode<IntPtrT> tmp470;
  TNode<IntPtrT> tmp471;
  if (block228.is_used()) {
    ca_.Bind(&block228, &phi_bb228_20, &phi_bb228_25, &phi_bb228_26, &phi_bb228_27, &phi_bb228_28, &phi_bb228_29, &phi_bb228_32, &phi_bb228_33, &phi_bb228_35, &phi_bb228_36, &phi_bb228_37);
    tmp469 = CodeStubAssembler(state_).IntPtrMod(TNode<IntPtrT>{tmp465}, TNode<IntPtrT>{tmp462});
    tmp470 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp462}, TNode<IntPtrT>{tmp469});
    tmp471 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb227_46}, TNode<IntPtrT>{tmp470});
    ca_.Goto(&block229, phi_bb228_20, phi_bb228_25, phi_bb228_26, phi_bb228_27, phi_bb228_28, phi_bb228_29, phi_bb228_32, phi_bb228_33, phi_bb228_35, phi_bb228_36, phi_bb228_37, tmp471);
  }

  TNode<IntPtrT> phi_bb229_20;
  TNode<IntPtrT> phi_bb229_25;
  TNode<IntPtrT> phi_bb229_26;
  TNode<IntPtrT> phi_bb229_27;
  TNode<IntPtrT> phi_bb229_28;
  TNode<IntPtrT> phi_bb229_29;
  TNode<IntPtrT> phi_bb229_32;
  TNode<BoolT> phi_bb229_33;
  TNode<IntPtrT> phi_bb229_35;
  TNode<IntPtrT> phi_bb229_36;
  TNode<BoolT> phi_bb229_37;
  TNode<IntPtrT> phi_bb229_46;
  TNode<RawPtrT> tmp472;
  TNode<Union<HeapObject, TaggedIndex>> tmp473;
  TNode<IntPtrT> tmp474;
  TNode<IntPtrT> tmp475;
  TNode<IntPtrT> tmp476;
  TNode<IntPtrT> tmp477;
  TNode<IntPtrT> tmp478;
  TNode<IntPtrT> tmp479;
  TNode<IntPtrT> tmp480;
  TNode<IntPtrT> tmp481;
  TNode<BoolT> tmp482;
  TNode<IntPtrT> tmp483;
  TNode<IntPtrT> tmp484;
  TNode<IntPtrT> tmp485;
  TNode<BoolT> tmp486;
  if (block229.is_used()) {
    ca_.Bind(&block229, &phi_bb229_20, &phi_bb229_25, &phi_bb229_26, &phi_bb229_27, &phi_bb229_28, &phi_bb229_29, &phi_bb229_32, &phi_bb229_33, &phi_bb229_35, &phi_bb229_36, &phi_bb229_37, &phi_bb229_46);
    tmp472 = CodeStubAssembler(state_).GCUnsafeReferenceToRawPtr(TNode<Union<HeapObject, TaggedIndex>>{tmp88}, TNode<IntPtrT>{phi_bb229_46});
    std::tie(tmp473, tmp474, tmp475, tmp476, tmp477, tmp478, tmp479, tmp480, tmp481, tmp482) = LocationAllocatorForReturns_0(state_, TNode<RawPtrT>{tmp459}, TNode<RawPtrT>{tmp461}, TNode<RawPtrT>{tmp472}).Flatten();
    tmp483 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{tmp55});
    tmp484 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp54}, TNode<IntPtrT>{tmp483});
    tmp485 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp486 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block233, tmp485, tmp474, tmp475, tmp476, tmp477, tmp478, tmp481, tmp482, phi_bb229_35, phi_bb229_36, phi_bb229_37, tmp54, tmp486);
  }

  TNode<IntPtrT> phi_bb233_20;
  TNode<IntPtrT> phi_bb233_25;
  TNode<IntPtrT> phi_bb233_26;
  TNode<IntPtrT> phi_bb233_27;
  TNode<IntPtrT> phi_bb233_28;
  TNode<IntPtrT> phi_bb233_29;
  TNode<IntPtrT> phi_bb233_32;
  TNode<BoolT> phi_bb233_33;
  TNode<IntPtrT> phi_bb233_35;
  TNode<IntPtrT> phi_bb233_36;
  TNode<BoolT> phi_bb233_37;
  TNode<IntPtrT> phi_bb233_46;
  TNode<BoolT> phi_bb233_48;
  TNode<BoolT> tmp487;
  TNode<BoolT> tmp488;
  if (block233.is_used()) {
    ca_.Bind(&block233, &phi_bb233_20, &phi_bb233_25, &phi_bb233_26, &phi_bb233_27, &phi_bb233_28, &phi_bb233_29, &phi_bb233_32, &phi_bb233_33, &phi_bb233_35, &phi_bb233_36, &phi_bb233_37, &phi_bb233_46, &phi_bb233_48);
    tmp487 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb233_46}, TNode<IntPtrT>{tmp484});
    tmp488 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp487});
    ca_.Branch(tmp488, &block231, std::vector<compiler::Node*>{phi_bb233_20, phi_bb233_25, phi_bb233_26, phi_bb233_27, phi_bb233_28, phi_bb233_29, phi_bb233_32, phi_bb233_33, phi_bb233_35, phi_bb233_36, phi_bb233_37, phi_bb233_46, phi_bb233_48}, &block232, std::vector<compiler::Node*>{phi_bb233_20, phi_bb233_25, phi_bb233_26, phi_bb233_27, phi_bb233_28, phi_bb233_29, phi_bb233_32, phi_bb233_33, phi_bb233_35, phi_bb233_36, phi_bb233_37, phi_bb233_46, phi_bb233_48});
  }

  TNode<IntPtrT> phi_bb231_20;
  TNode<IntPtrT> phi_bb231_25;
  TNode<IntPtrT> phi_bb231_26;
  TNode<IntPtrT> phi_bb231_27;
  TNode<IntPtrT> phi_bb231_28;
  TNode<IntPtrT> phi_bb231_29;
  TNode<IntPtrT> phi_bb231_32;
  TNode<BoolT> phi_bb231_33;
  TNode<IntPtrT> phi_bb231_35;
  TNode<IntPtrT> phi_bb231_36;
  TNode<BoolT> phi_bb231_37;
  TNode<IntPtrT> phi_bb231_46;
  TNode<BoolT> phi_bb231_48;
  TNode<IntPtrT> tmp489;
  TNode<BoolT> tmp490;
  if (block231.is_used()) {
    ca_.Bind(&block231, &phi_bb231_20, &phi_bb231_25, &phi_bb231_26, &phi_bb231_27, &phi_bb231_28, &phi_bb231_29, &phi_bb231_32, &phi_bb231_33, &phi_bb231_35, &phi_bb231_36, &phi_bb231_37, &phi_bb231_46, &phi_bb231_48);
    tmp489 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp490 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp31}, TNode<IntPtrT>{tmp489});
    ca_.Branch(tmp490, &block235, std::vector<compiler::Node*>{phi_bb231_20, phi_bb231_25, phi_bb231_26, phi_bb231_27, phi_bb231_28, phi_bb231_29, phi_bb231_32, phi_bb231_33, phi_bb231_35, phi_bb231_36, phi_bb231_37, phi_bb231_46, phi_bb231_48}, &block236, std::vector<compiler::Node*>{phi_bb231_20, phi_bb231_25, phi_bb231_26, phi_bb231_27, phi_bb231_28, phi_bb231_29, phi_bb231_32, phi_bb231_33, phi_bb231_35, phi_bb231_36, phi_bb231_37, phi_bb231_46, phi_bb231_48});
  }

  TNode<IntPtrT> phi_bb235_20;
  TNode<IntPtrT> phi_bb235_25;
  TNode<IntPtrT> phi_bb235_26;
  TNode<IntPtrT> phi_bb235_27;
  TNode<IntPtrT> phi_bb235_28;
  TNode<IntPtrT> phi_bb235_29;
  TNode<IntPtrT> phi_bb235_32;
  TNode<BoolT> phi_bb235_33;
  TNode<IntPtrT> phi_bb235_35;
  TNode<IntPtrT> phi_bb235_36;
  TNode<BoolT> phi_bb235_37;
  TNode<IntPtrT> phi_bb235_46;
  TNode<BoolT> phi_bb235_48;
  if (block235.is_used()) {
    ca_.Bind(&block235, &phi_bb235_20, &phi_bb235_25, &phi_bb235_26, &phi_bb235_27, &phi_bb235_28, &phi_bb235_29, &phi_bb235_32, &phi_bb235_33, &phi_bb235_35, &phi_bb235_36, &phi_bb235_37, &phi_bb235_46, &phi_bb235_48);
    ca_.Goto(&block237, phi_bb235_20, phi_bb235_25, phi_bb235_26, phi_bb235_27, phi_bb235_28, phi_bb235_29, phi_bb235_32, phi_bb235_33, phi_bb235_35, phi_bb235_36, phi_bb235_37, phi_bb235_46, phi_bb235_48, tmp448);
  }

  TNode<IntPtrT> phi_bb236_20;
  TNode<IntPtrT> phi_bb236_25;
  TNode<IntPtrT> phi_bb236_26;
  TNode<IntPtrT> phi_bb236_27;
  TNode<IntPtrT> phi_bb236_28;
  TNode<IntPtrT> phi_bb236_29;
  TNode<IntPtrT> phi_bb236_32;
  TNode<BoolT> phi_bb236_33;
  TNode<IntPtrT> phi_bb236_35;
  TNode<IntPtrT> phi_bb236_36;
  TNode<BoolT> phi_bb236_37;
  TNode<IntPtrT> phi_bb236_46;
  TNode<BoolT> phi_bb236_48;
  TNode<Union<HeapObject, TaggedIndex>> tmp491;
  TNode<IntPtrT> tmp492;
  TNode<IntPtrT> tmp493;
  TNode<UintPtrT> tmp494;
  TNode<UintPtrT> tmp495;
  TNode<BoolT> tmp496;
  if (block236.is_used()) {
    ca_.Bind(&block236, &phi_bb236_20, &phi_bb236_25, &phi_bb236_26, &phi_bb236_27, &phi_bb236_28, &phi_bb236_29, &phi_bb236_32, &phi_bb236_33, &phi_bb236_35, &phi_bb236_36, &phi_bb236_37, &phi_bb236_46, &phi_bb236_48);
    std::tie(tmp491, tmp492, tmp493) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb224_41}).Flatten();
    tmp494 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb236_20});
    tmp495 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp493});
    tmp496 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp494}, TNode<UintPtrT>{tmp495});
    ca_.Branch(tmp496, &block242, std::vector<compiler::Node*>{phi_bb236_20, phi_bb236_25, phi_bb236_26, phi_bb236_27, phi_bb236_28, phi_bb236_29, phi_bb236_32, phi_bb236_33, phi_bb236_35, phi_bb236_36, phi_bb236_37, phi_bb236_46, phi_bb236_48, phi_bb236_20, phi_bb236_20, phi_bb236_20, phi_bb236_20}, &block243, std::vector<compiler::Node*>{phi_bb236_20, phi_bb236_25, phi_bb236_26, phi_bb236_27, phi_bb236_28, phi_bb236_29, phi_bb236_32, phi_bb236_33, phi_bb236_35, phi_bb236_36, phi_bb236_37, phi_bb236_46, phi_bb236_48, phi_bb236_20, phi_bb236_20, phi_bb236_20, phi_bb236_20});
  }

  TNode<IntPtrT> phi_bb242_20;
  TNode<IntPtrT> phi_bb242_25;
  TNode<IntPtrT> phi_bb242_26;
  TNode<IntPtrT> phi_bb242_27;
  TNode<IntPtrT> phi_bb242_28;
  TNode<IntPtrT> phi_bb242_29;
  TNode<IntPtrT> phi_bb242_32;
  TNode<BoolT> phi_bb242_33;
  TNode<IntPtrT> phi_bb242_35;
  TNode<IntPtrT> phi_bb242_36;
  TNode<BoolT> phi_bb242_37;
  TNode<IntPtrT> phi_bb242_46;
  TNode<BoolT> phi_bb242_48;
  TNode<IntPtrT> phi_bb242_54;
  TNode<IntPtrT> phi_bb242_55;
  TNode<IntPtrT> phi_bb242_59;
  TNode<IntPtrT> phi_bb242_60;
  TNode<IntPtrT> tmp497;
  TNode<IntPtrT> tmp498;
  TNode<Union<HeapObject, TaggedIndex>> tmp499;
  TNode<IntPtrT> tmp500;
  TNode<Object> tmp501;
  TNode<JSAny> tmp502;
  if (block242.is_used()) {
    ca_.Bind(&block242, &phi_bb242_20, &phi_bb242_25, &phi_bb242_26, &phi_bb242_27, &phi_bb242_28, &phi_bb242_29, &phi_bb242_32, &phi_bb242_33, &phi_bb242_35, &phi_bb242_36, &phi_bb242_37, &phi_bb242_46, &phi_bb242_48, &phi_bb242_54, &phi_bb242_55, &phi_bb242_59, &phi_bb242_60);
    tmp497 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb242_60});
    tmp498 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp492}, TNode<IntPtrT>{tmp497});
    std::tie(tmp499, tmp500) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp491}, TNode<IntPtrT>{tmp498}).Flatten();
    tmp501 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp499, tmp500});
    tmp502 = UnsafeCast_JSAny_0(state_, TNode<Context>{tmp441}, TNode<Object>{tmp501});
    ca_.Goto(&block237, phi_bb242_20, phi_bb242_25, phi_bb242_26, phi_bb242_27, phi_bb242_28, phi_bb242_29, phi_bb242_32, phi_bb242_33, phi_bb242_35, phi_bb242_36, phi_bb242_37, phi_bb242_46, phi_bb242_48, tmp502);
  }

  TNode<IntPtrT> phi_bb243_20;
  TNode<IntPtrT> phi_bb243_25;
  TNode<IntPtrT> phi_bb243_26;
  TNode<IntPtrT> phi_bb243_27;
  TNode<IntPtrT> phi_bb243_28;
  TNode<IntPtrT> phi_bb243_29;
  TNode<IntPtrT> phi_bb243_32;
  TNode<BoolT> phi_bb243_33;
  TNode<IntPtrT> phi_bb243_35;
  TNode<IntPtrT> phi_bb243_36;
  TNode<BoolT> phi_bb243_37;
  TNode<IntPtrT> phi_bb243_46;
  TNode<BoolT> phi_bb243_48;
  TNode<IntPtrT> phi_bb243_54;
  TNode<IntPtrT> phi_bb243_55;
  TNode<IntPtrT> phi_bb243_59;
  TNode<IntPtrT> phi_bb243_60;
  if (block243.is_used()) {
    ca_.Bind(&block243, &phi_bb243_20, &phi_bb243_25, &phi_bb243_26, &phi_bb243_27, &phi_bb243_28, &phi_bb243_29, &phi_bb243_32, &phi_bb243_33, &phi_bb243_35, &phi_bb243_36, &phi_bb243_37, &phi_bb243_46, &phi_bb243_48, &phi_bb243_54, &phi_bb243_55, &phi_bb243_59, &phi_bb243_60);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb237_20;
  TNode<IntPtrT> phi_bb237_25;
  TNode<IntPtrT> phi_bb237_26;
  TNode<IntPtrT> phi_bb237_27;
  TNode<IntPtrT> phi_bb237_28;
  TNode<IntPtrT> phi_bb237_29;
  TNode<IntPtrT> phi_bb237_32;
  TNode<BoolT> phi_bb237_33;
  TNode<IntPtrT> phi_bb237_35;
  TNode<IntPtrT> phi_bb237_36;
  TNode<BoolT> phi_bb237_37;
  TNode<IntPtrT> phi_bb237_46;
  TNode<BoolT> phi_bb237_48;
  TNode<JSAny> phi_bb237_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp503;
  TNode<IntPtrT> tmp504;
  TNode<IntPtrT> tmp505;
  TNode<IntPtrT> tmp506;
  TNode<Uint32T> tmp507;
  TNode<Uint32T> tmp508;
  TNode<BoolT> tmp509;
  if (block237.is_used()) {
    ca_.Bind(&block237, &phi_bb237_20, &phi_bb237_25, &phi_bb237_26, &phi_bb237_27, &phi_bb237_28, &phi_bb237_29, &phi_bb237_32, &phi_bb237_33, &phi_bb237_35, &phi_bb237_36, &phi_bb237_37, &phi_bb237_46, &phi_bb237_48, &phi_bb237_49);
    std::tie(tmp503, tmp504) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp53}, TNode<IntPtrT>{phi_bb237_46}).Flatten();
    tmp505 = FromConstexpr_intptr_constexpr_int31_0(state_, kInt32Size);
    tmp506 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb237_46}, TNode<IntPtrT>{tmp505});
    tmp507 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp503, tmp504});
    tmp508 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI32.raw_bit_field());
    tmp509 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp507}, TNode<Uint32T>{tmp508});
    ca_.Branch(tmp509, &block253, std::vector<compiler::Node*>{phi_bb237_20, phi_bb237_25, phi_bb237_26, phi_bb237_27, phi_bb237_28, phi_bb237_29, phi_bb237_32, phi_bb237_33, phi_bb237_35, phi_bb237_36, phi_bb237_37, phi_bb237_48, phi_bb237_49}, &block254, std::vector<compiler::Node*>{phi_bb237_20, phi_bb237_25, phi_bb237_26, phi_bb237_27, phi_bb237_28, phi_bb237_29, phi_bb237_32, phi_bb237_33, phi_bb237_35, phi_bb237_36, phi_bb237_37, phi_bb237_48, phi_bb237_49});
  }

  TNode<IntPtrT> phi_bb253_20;
  TNode<IntPtrT> phi_bb253_25;
  TNode<IntPtrT> phi_bb253_26;
  TNode<IntPtrT> phi_bb253_27;
  TNode<IntPtrT> phi_bb253_28;
  TNode<IntPtrT> phi_bb253_29;
  TNode<IntPtrT> phi_bb253_32;
  TNode<BoolT> phi_bb253_33;
  TNode<IntPtrT> phi_bb253_35;
  TNode<IntPtrT> phi_bb253_36;
  TNode<BoolT> phi_bb253_37;
  TNode<BoolT> phi_bb253_48;
  TNode<JSAny> phi_bb253_49;
  TNode<IntPtrT> tmp510;
  TNode<IntPtrT> tmp511;
  TNode<IntPtrT> tmp512;
  TNode<BoolT> tmp513;
  if (block253.is_used()) {
    ca_.Bind(&block253, &phi_bb253_20, &phi_bb253_25, &phi_bb253_26, &phi_bb253_27, &phi_bb253_28, &phi_bb253_29, &phi_bb253_32, &phi_bb253_33, &phi_bb253_35, &phi_bb253_36, &phi_bb253_37, &phi_bb253_48, &phi_bb253_49);
    tmp510 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp511 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb253_25}, TNode<IntPtrT>{tmp510});
    tmp512 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp513 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb253_25}, TNode<IntPtrT>{tmp512});
    ca_.Branch(tmp513, &block257, std::vector<compiler::Node*>{phi_bb253_20, phi_bb253_26, phi_bb253_27, phi_bb253_28, phi_bb253_29, phi_bb253_32, phi_bb253_33, phi_bb253_35, phi_bb253_36, phi_bb253_37, phi_bb253_48, phi_bb253_49}, &block258, std::vector<compiler::Node*>{phi_bb253_20, phi_bb253_26, phi_bb253_27, phi_bb253_28, phi_bb253_29, phi_bb253_32, phi_bb253_33, phi_bb253_35, phi_bb253_36, phi_bb253_37, phi_bb253_48, phi_bb253_49});
  }

  TNode<IntPtrT> phi_bb257_20;
  TNode<IntPtrT> phi_bb257_26;
  TNode<IntPtrT> phi_bb257_27;
  TNode<IntPtrT> phi_bb257_28;
  TNode<IntPtrT> phi_bb257_29;
  TNode<IntPtrT> phi_bb257_32;
  TNode<BoolT> phi_bb257_33;
  TNode<IntPtrT> phi_bb257_35;
  TNode<IntPtrT> phi_bb257_36;
  TNode<BoolT> phi_bb257_37;
  TNode<BoolT> phi_bb257_48;
  TNode<JSAny> phi_bb257_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp514;
  TNode<IntPtrT> tmp515;
  TNode<IntPtrT> tmp516;
  TNode<IntPtrT> tmp517;
  if (block257.is_used()) {
    ca_.Bind(&block257, &phi_bb257_20, &phi_bb257_26, &phi_bb257_27, &phi_bb257_28, &phi_bb257_29, &phi_bb257_32, &phi_bb257_33, &phi_bb257_35, &phi_bb257_36, &phi_bb257_37, &phi_bb257_48, &phi_bb257_49);
    std::tie(tmp514, tmp515) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb257_27}).Flatten();
    tmp516 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp517 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb257_27}, TNode<IntPtrT>{tmp516});
    ca_.Goto(&block256, phi_bb257_20, phi_bb257_26, tmp517, phi_bb257_28, phi_bb257_29, phi_bb257_32, phi_bb257_33, phi_bb257_35, phi_bb257_36, phi_bb257_37, phi_bb257_48, phi_bb257_49, tmp514, tmp515);
  }

  TNode<IntPtrT> phi_bb258_20;
  TNode<IntPtrT> phi_bb258_26;
  TNode<IntPtrT> phi_bb258_27;
  TNode<IntPtrT> phi_bb258_28;
  TNode<IntPtrT> phi_bb258_29;
  TNode<IntPtrT> phi_bb258_32;
  TNode<BoolT> phi_bb258_33;
  TNode<IntPtrT> phi_bb258_35;
  TNode<IntPtrT> phi_bb258_36;
  TNode<BoolT> phi_bb258_37;
  TNode<BoolT> phi_bb258_48;
  TNode<JSAny> phi_bb258_49;
  if (block258.is_used()) {
    ca_.Bind(&block258, &phi_bb258_20, &phi_bb258_26, &phi_bb258_27, &phi_bb258_28, &phi_bb258_29, &phi_bb258_32, &phi_bb258_33, &phi_bb258_35, &phi_bb258_36, &phi_bb258_37, &phi_bb258_48, &phi_bb258_49);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block260, phi_bb258_20, phi_bb258_26, phi_bb258_27, phi_bb258_28, phi_bb258_29, phi_bb258_32, phi_bb258_33, phi_bb258_35, phi_bb258_36, phi_bb258_37, phi_bb258_48, phi_bb258_49);
    } else {
      ca_.Goto(&block261, phi_bb258_20, phi_bb258_26, phi_bb258_27, phi_bb258_28, phi_bb258_29, phi_bb258_32, phi_bb258_33, phi_bb258_35, phi_bb258_36, phi_bb258_37, phi_bb258_48, phi_bb258_49);
    }
  }

  TNode<IntPtrT> phi_bb260_20;
  TNode<IntPtrT> phi_bb260_26;
  TNode<IntPtrT> phi_bb260_27;
  TNode<IntPtrT> phi_bb260_28;
  TNode<IntPtrT> phi_bb260_29;
  TNode<IntPtrT> phi_bb260_32;
  TNode<BoolT> phi_bb260_33;
  TNode<IntPtrT> phi_bb260_35;
  TNode<IntPtrT> phi_bb260_36;
  TNode<BoolT> phi_bb260_37;
  TNode<BoolT> phi_bb260_48;
  TNode<JSAny> phi_bb260_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp518;
  TNode<IntPtrT> tmp519;
  TNode<IntPtrT> tmp520;
  TNode<IntPtrT> tmp521;
  if (block260.is_used()) {
    ca_.Bind(&block260, &phi_bb260_20, &phi_bb260_26, &phi_bb260_27, &phi_bb260_28, &phi_bb260_29, &phi_bb260_32, &phi_bb260_33, &phi_bb260_35, &phi_bb260_36, &phi_bb260_37, &phi_bb260_48, &phi_bb260_49);
    std::tie(tmp518, tmp519) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb260_29}).Flatten();
    tmp520 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp521 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb260_29}, TNode<IntPtrT>{tmp520});
    ca_.Goto(&block259, phi_bb260_20, phi_bb260_26, phi_bb260_27, phi_bb260_28, tmp521, phi_bb260_32, phi_bb260_33, phi_bb260_35, phi_bb260_36, phi_bb260_37, phi_bb260_48, phi_bb260_49, tmp518, tmp519);
  }

  TNode<IntPtrT> phi_bb261_20;
  TNode<IntPtrT> phi_bb261_26;
  TNode<IntPtrT> phi_bb261_27;
  TNode<IntPtrT> phi_bb261_28;
  TNode<IntPtrT> phi_bb261_29;
  TNode<IntPtrT> phi_bb261_32;
  TNode<BoolT> phi_bb261_33;
  TNode<IntPtrT> phi_bb261_35;
  TNode<IntPtrT> phi_bb261_36;
  TNode<BoolT> phi_bb261_37;
  TNode<BoolT> phi_bb261_48;
  TNode<JSAny> phi_bb261_49;
  TNode<IntPtrT> tmp522;
  TNode<BoolT> tmp523;
  if (block261.is_used()) {
    ca_.Bind(&block261, &phi_bb261_20, &phi_bb261_26, &phi_bb261_27, &phi_bb261_28, &phi_bb261_29, &phi_bb261_32, &phi_bb261_33, &phi_bb261_35, &phi_bb261_36, &phi_bb261_37, &phi_bb261_48, &phi_bb261_49);
    tmp522 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp523 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb261_32}, TNode<IntPtrT>{tmp522});
    ca_.Branch(tmp523, &block263, std::vector<compiler::Node*>{phi_bb261_20, phi_bb261_26, phi_bb261_27, phi_bb261_28, phi_bb261_29, phi_bb261_32, phi_bb261_33, phi_bb261_35, phi_bb261_36, phi_bb261_37, phi_bb261_48, phi_bb261_49}, &block264, std::vector<compiler::Node*>{phi_bb261_20, phi_bb261_26, phi_bb261_27, phi_bb261_28, phi_bb261_29, phi_bb261_32, phi_bb261_33, phi_bb261_35, phi_bb261_36, phi_bb261_37, phi_bb261_48, phi_bb261_49});
  }

  TNode<IntPtrT> phi_bb263_20;
  TNode<IntPtrT> phi_bb263_26;
  TNode<IntPtrT> phi_bb263_27;
  TNode<IntPtrT> phi_bb263_28;
  TNode<IntPtrT> phi_bb263_29;
  TNode<IntPtrT> phi_bb263_32;
  TNode<BoolT> phi_bb263_33;
  TNode<IntPtrT> phi_bb263_35;
  TNode<IntPtrT> phi_bb263_36;
  TNode<BoolT> phi_bb263_37;
  TNode<BoolT> phi_bb263_48;
  TNode<JSAny> phi_bb263_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp524;
  TNode<IntPtrT> tmp525;
  TNode<IntPtrT> tmp526;
  TNode<BoolT> tmp527;
  if (block263.is_used()) {
    ca_.Bind(&block263, &phi_bb263_20, &phi_bb263_26, &phi_bb263_27, &phi_bb263_28, &phi_bb263_29, &phi_bb263_32, &phi_bb263_33, &phi_bb263_35, &phi_bb263_36, &phi_bb263_37, &phi_bb263_48, &phi_bb263_49);
    std::tie(tmp524, tmp525) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb263_32}).Flatten();
    tmp526 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp527 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block259, phi_bb263_20, phi_bb263_26, phi_bb263_27, phi_bb263_28, phi_bb263_29, tmp526, tmp527, phi_bb263_35, phi_bb263_36, phi_bb263_37, phi_bb263_48, phi_bb263_49, tmp524, tmp525);
  }

  TNode<IntPtrT> phi_bb264_20;
  TNode<IntPtrT> phi_bb264_26;
  TNode<IntPtrT> phi_bb264_27;
  TNode<IntPtrT> phi_bb264_28;
  TNode<IntPtrT> phi_bb264_29;
  TNode<IntPtrT> phi_bb264_32;
  TNode<BoolT> phi_bb264_33;
  TNode<IntPtrT> phi_bb264_35;
  TNode<IntPtrT> phi_bb264_36;
  TNode<BoolT> phi_bb264_37;
  TNode<BoolT> phi_bb264_48;
  TNode<JSAny> phi_bb264_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp528;
  TNode<IntPtrT> tmp529;
  TNode<IntPtrT> tmp530;
  TNode<IntPtrT> tmp531;
  TNode<IntPtrT> tmp532;
  TNode<IntPtrT> tmp533;
  TNode<BoolT> tmp534;
  if (block264.is_used()) {
    ca_.Bind(&block264, &phi_bb264_20, &phi_bb264_26, &phi_bb264_27, &phi_bb264_28, &phi_bb264_29, &phi_bb264_32, &phi_bb264_33, &phi_bb264_35, &phi_bb264_36, &phi_bb264_37, &phi_bb264_48, &phi_bb264_49);
    std::tie(tmp528, tmp529) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb264_29}).Flatten();
    tmp530 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp531 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb264_29}, TNode<IntPtrT>{tmp530});
    tmp532 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp533 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp531}, TNode<IntPtrT>{tmp532});
    tmp534 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block259, phi_bb264_20, phi_bb264_26, phi_bb264_27, phi_bb264_28, tmp533, tmp531, tmp534, phi_bb264_35, phi_bb264_36, phi_bb264_37, phi_bb264_48, phi_bb264_49, tmp528, tmp529);
  }

  TNode<IntPtrT> phi_bb259_20;
  TNode<IntPtrT> phi_bb259_26;
  TNode<IntPtrT> phi_bb259_27;
  TNode<IntPtrT> phi_bb259_28;
  TNode<IntPtrT> phi_bb259_29;
  TNode<IntPtrT> phi_bb259_32;
  TNode<BoolT> phi_bb259_33;
  TNode<IntPtrT> phi_bb259_35;
  TNode<IntPtrT> phi_bb259_36;
  TNode<BoolT> phi_bb259_37;
  TNode<BoolT> phi_bb259_48;
  TNode<JSAny> phi_bb259_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb259_51;
  TNode<IntPtrT> phi_bb259_52;
  if (block259.is_used()) {
    ca_.Bind(&block259, &phi_bb259_20, &phi_bb259_26, &phi_bb259_27, &phi_bb259_28, &phi_bb259_29, &phi_bb259_32, &phi_bb259_33, &phi_bb259_35, &phi_bb259_36, &phi_bb259_37, &phi_bb259_48, &phi_bb259_49, &phi_bb259_51, &phi_bb259_52);
    ca_.Goto(&block256, phi_bb259_20, phi_bb259_26, phi_bb259_27, phi_bb259_28, phi_bb259_29, phi_bb259_32, phi_bb259_33, phi_bb259_35, phi_bb259_36, phi_bb259_37, phi_bb259_48, phi_bb259_49, phi_bb259_51, phi_bb259_52);
  }

  TNode<IntPtrT> phi_bb256_20;
  TNode<IntPtrT> phi_bb256_26;
  TNode<IntPtrT> phi_bb256_27;
  TNode<IntPtrT> phi_bb256_28;
  TNode<IntPtrT> phi_bb256_29;
  TNode<IntPtrT> phi_bb256_32;
  TNode<BoolT> phi_bb256_33;
  TNode<IntPtrT> phi_bb256_35;
  TNode<IntPtrT> phi_bb256_36;
  TNode<BoolT> phi_bb256_37;
  TNode<BoolT> phi_bb256_48;
  TNode<JSAny> phi_bb256_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb256_51;
  TNode<IntPtrT> phi_bb256_52;
  TNode<Smi> tmp535;
  if (block256.is_used()) {
    ca_.Bind(&block256, &phi_bb256_20, &phi_bb256_26, &phi_bb256_27, &phi_bb256_28, &phi_bb256_29, &phi_bb256_32, &phi_bb256_33, &phi_bb256_35, &phi_bb256_36, &phi_bb256_37, &phi_bb256_48, &phi_bb256_49, &phi_bb256_51, &phi_bb256_52);
    compiler::CodeAssemblerLabel label536(&ca_);
    tmp535 = Cast_Smi_0(state_, TNode<Object>{phi_bb256_49}, &label536);
    ca_.Goto(&block267, phi_bb256_20, phi_bb256_26, phi_bb256_27, phi_bb256_28, phi_bb256_29, phi_bb256_32, phi_bb256_33, phi_bb256_35, phi_bb256_36, phi_bb256_37, phi_bb256_48, phi_bb256_49, phi_bb256_51, phi_bb256_52, phi_bb256_49, phi_bb256_49);
    if (label536.is_used()) {
      ca_.Bind(&label536);
      ca_.Goto(&block268, phi_bb256_20, phi_bb256_26, phi_bb256_27, phi_bb256_28, phi_bb256_29, phi_bb256_32, phi_bb256_33, phi_bb256_35, phi_bb256_36, phi_bb256_37, phi_bb256_48, phi_bb256_49, phi_bb256_51, phi_bb256_52, phi_bb256_49, phi_bb256_49);
    }
  }

  TNode<IntPtrT> phi_bb268_20;
  TNode<IntPtrT> phi_bb268_26;
  TNode<IntPtrT> phi_bb268_27;
  TNode<IntPtrT> phi_bb268_28;
  TNode<IntPtrT> phi_bb268_29;
  TNode<IntPtrT> phi_bb268_32;
  TNode<BoolT> phi_bb268_33;
  TNode<IntPtrT> phi_bb268_35;
  TNode<IntPtrT> phi_bb268_36;
  TNode<BoolT> phi_bb268_37;
  TNode<BoolT> phi_bb268_48;
  TNode<JSAny> phi_bb268_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb268_51;
  TNode<IntPtrT> phi_bb268_52;
  TNode<JSAny> phi_bb268_53;
  TNode<JSAny> phi_bb268_54;
  TNode<Int32T> tmp537;
  TNode<IntPtrT> tmp538;
  if (block268.is_used()) {
    ca_.Bind(&block268, &phi_bb268_20, &phi_bb268_26, &phi_bb268_27, &phi_bb268_28, &phi_bb268_29, &phi_bb268_32, &phi_bb268_33, &phi_bb268_35, &phi_bb268_36, &phi_bb268_37, &phi_bb268_48, &phi_bb268_49, &phi_bb268_51, &phi_bb268_52, &phi_bb268_53, &phi_bb268_54);
    tmp537 = ca_.CallBuiltin<Int32T>(Builtin::kWasmTaggedNonSmiToInt32, tmp441, ca_.UncheckedCast<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, String, Symbol, Undefined>>(phi_bb268_53));
    tmp538 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp537});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb268_51, phi_bb268_52}, tmp538);
    ca_.Goto(&block265, phi_bb268_20, phi_bb268_26, phi_bb268_27, phi_bb268_28, phi_bb268_29, phi_bb268_32, phi_bb268_33, phi_bb268_35, phi_bb268_36, phi_bb268_37, phi_bb268_48, phi_bb268_49, phi_bb268_51, phi_bb268_52, phi_bb268_53);
  }

  TNode<IntPtrT> phi_bb267_20;
  TNode<IntPtrT> phi_bb267_26;
  TNode<IntPtrT> phi_bb267_27;
  TNode<IntPtrT> phi_bb267_28;
  TNode<IntPtrT> phi_bb267_29;
  TNode<IntPtrT> phi_bb267_32;
  TNode<BoolT> phi_bb267_33;
  TNode<IntPtrT> phi_bb267_35;
  TNode<IntPtrT> phi_bb267_36;
  TNode<BoolT> phi_bb267_37;
  TNode<BoolT> phi_bb267_48;
  TNode<JSAny> phi_bb267_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb267_51;
  TNode<IntPtrT> phi_bb267_52;
  TNode<JSAny> phi_bb267_53;
  TNode<JSAny> phi_bb267_54;
  TNode<Int32T> tmp539;
  TNode<IntPtrT> tmp540;
  if (block267.is_used()) {
    ca_.Bind(&block267, &phi_bb267_20, &phi_bb267_26, &phi_bb267_27, &phi_bb267_28, &phi_bb267_29, &phi_bb267_32, &phi_bb267_33, &phi_bb267_35, &phi_bb267_36, &phi_bb267_37, &phi_bb267_48, &phi_bb267_49, &phi_bb267_51, &phi_bb267_52, &phi_bb267_53, &phi_bb267_54);
    tmp539 = CodeStubAssembler(state_).SmiToInt32(TNode<Smi>{tmp535});
    tmp540 = Convert_intptr_int32_0(state_, TNode<Int32T>{tmp539});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb267_51, phi_bb267_52}, tmp540);
    ca_.Goto(&block265, phi_bb267_20, phi_bb267_26, phi_bb267_27, phi_bb267_28, phi_bb267_29, phi_bb267_32, phi_bb267_33, phi_bb267_35, phi_bb267_36, phi_bb267_37, phi_bb267_48, phi_bb267_49, phi_bb267_51, phi_bb267_52, phi_bb267_53);
  }

  TNode<IntPtrT> phi_bb265_20;
  TNode<IntPtrT> phi_bb265_26;
  TNode<IntPtrT> phi_bb265_27;
  TNode<IntPtrT> phi_bb265_28;
  TNode<IntPtrT> phi_bb265_29;
  TNode<IntPtrT> phi_bb265_32;
  TNode<BoolT> phi_bb265_33;
  TNode<IntPtrT> phi_bb265_35;
  TNode<IntPtrT> phi_bb265_36;
  TNode<BoolT> phi_bb265_37;
  TNode<BoolT> phi_bb265_48;
  TNode<JSAny> phi_bb265_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb265_51;
  TNode<IntPtrT> phi_bb265_52;
  TNode<JSAny> phi_bb265_53;
  if (block265.is_used()) {
    ca_.Bind(&block265, &phi_bb265_20, &phi_bb265_26, &phi_bb265_27, &phi_bb265_28, &phi_bb265_29, &phi_bb265_32, &phi_bb265_33, &phi_bb265_35, &phi_bb265_36, &phi_bb265_37, &phi_bb265_48, &phi_bb265_49, &phi_bb265_51, &phi_bb265_52, &phi_bb265_53);
    ca_.Goto(&block255, phi_bb265_20, tmp511, phi_bb265_26, phi_bb265_27, phi_bb265_28, phi_bb265_29, phi_bb265_32, phi_bb265_33, phi_bb265_35, phi_bb265_36, phi_bb265_37, phi_bb265_48, phi_bb265_49);
  }

  TNode<IntPtrT> phi_bb254_20;
  TNode<IntPtrT> phi_bb254_25;
  TNode<IntPtrT> phi_bb254_26;
  TNode<IntPtrT> phi_bb254_27;
  TNode<IntPtrT> phi_bb254_28;
  TNode<IntPtrT> phi_bb254_29;
  TNode<IntPtrT> phi_bb254_32;
  TNode<BoolT> phi_bb254_33;
  TNode<IntPtrT> phi_bb254_35;
  TNode<IntPtrT> phi_bb254_36;
  TNode<BoolT> phi_bb254_37;
  TNode<BoolT> phi_bb254_48;
  TNode<JSAny> phi_bb254_49;
  TNode<Uint32T> tmp541;
  TNode<BoolT> tmp542;
  if (block254.is_used()) {
    ca_.Bind(&block254, &phi_bb254_20, &phi_bb254_25, &phi_bb254_26, &phi_bb254_27, &phi_bb254_28, &phi_bb254_29, &phi_bb254_32, &phi_bb254_33, &phi_bb254_35, &phi_bb254_36, &phi_bb254_37, &phi_bb254_48, &phi_bb254_49);
    tmp541 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF32.raw_bit_field());
    tmp542 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp507}, TNode<Uint32T>{tmp541});
    ca_.Branch(tmp542, &block269, std::vector<compiler::Node*>{phi_bb254_20, phi_bb254_25, phi_bb254_26, phi_bb254_27, phi_bb254_28, phi_bb254_29, phi_bb254_32, phi_bb254_33, phi_bb254_35, phi_bb254_36, phi_bb254_37, phi_bb254_48, phi_bb254_49}, &block270, std::vector<compiler::Node*>{phi_bb254_20, phi_bb254_25, phi_bb254_26, phi_bb254_27, phi_bb254_28, phi_bb254_29, phi_bb254_32, phi_bb254_33, phi_bb254_35, phi_bb254_36, phi_bb254_37, phi_bb254_48, phi_bb254_49});
  }

  TNode<IntPtrT> phi_bb269_20;
  TNode<IntPtrT> phi_bb269_25;
  TNode<IntPtrT> phi_bb269_26;
  TNode<IntPtrT> phi_bb269_27;
  TNode<IntPtrT> phi_bb269_28;
  TNode<IntPtrT> phi_bb269_29;
  TNode<IntPtrT> phi_bb269_32;
  TNode<BoolT> phi_bb269_33;
  TNode<IntPtrT> phi_bb269_35;
  TNode<IntPtrT> phi_bb269_36;
  TNode<BoolT> phi_bb269_37;
  TNode<BoolT> phi_bb269_48;
  TNode<JSAny> phi_bb269_49;
  TNode<IntPtrT> tmp543;
  TNode<IntPtrT> tmp544;
  TNode<IntPtrT> tmp545;
  TNode<BoolT> tmp546;
  if (block269.is_used()) {
    ca_.Bind(&block269, &phi_bb269_20, &phi_bb269_25, &phi_bb269_26, &phi_bb269_27, &phi_bb269_28, &phi_bb269_29, &phi_bb269_32, &phi_bb269_33, &phi_bb269_35, &phi_bb269_36, &phi_bb269_37, &phi_bb269_48, &phi_bb269_49);
    tmp543 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp544 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb269_26}, TNode<IntPtrT>{tmp543});
    tmp545 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp546 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb269_26}, TNode<IntPtrT>{tmp545});
    ca_.Branch(tmp546, &block273, std::vector<compiler::Node*>{phi_bb269_20, phi_bb269_25, phi_bb269_27, phi_bb269_28, phi_bb269_29, phi_bb269_32, phi_bb269_33, phi_bb269_35, phi_bb269_36, phi_bb269_37, phi_bb269_48, phi_bb269_49}, &block274, std::vector<compiler::Node*>{phi_bb269_20, phi_bb269_25, phi_bb269_27, phi_bb269_28, phi_bb269_29, phi_bb269_32, phi_bb269_33, phi_bb269_35, phi_bb269_36, phi_bb269_37, phi_bb269_48, phi_bb269_49});
  }

  TNode<IntPtrT> phi_bb273_20;
  TNode<IntPtrT> phi_bb273_25;
  TNode<IntPtrT> phi_bb273_27;
  TNode<IntPtrT> phi_bb273_28;
  TNode<IntPtrT> phi_bb273_29;
  TNode<IntPtrT> phi_bb273_32;
  TNode<BoolT> phi_bb273_33;
  TNode<IntPtrT> phi_bb273_35;
  TNode<IntPtrT> phi_bb273_36;
  TNode<BoolT> phi_bb273_37;
  TNode<BoolT> phi_bb273_48;
  TNode<JSAny> phi_bb273_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp547;
  TNode<IntPtrT> tmp548;
  TNode<IntPtrT> tmp549;
  TNode<IntPtrT> tmp550;
  if (block273.is_used()) {
    ca_.Bind(&block273, &phi_bb273_20, &phi_bb273_25, &phi_bb273_27, &phi_bb273_28, &phi_bb273_29, &phi_bb273_32, &phi_bb273_33, &phi_bb273_35, &phi_bb273_36, &phi_bb273_37, &phi_bb273_48, &phi_bb273_49);
    std::tie(tmp547, tmp548) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb273_28}).Flatten();
    tmp549 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp550 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb273_28}, TNode<IntPtrT>{tmp549});
    ca_.Goto(&block272, phi_bb273_20, phi_bb273_25, phi_bb273_27, tmp550, phi_bb273_29, phi_bb273_32, phi_bb273_33, phi_bb273_35, phi_bb273_36, phi_bb273_37, phi_bb273_48, phi_bb273_49, tmp547, tmp548);
  }

  TNode<IntPtrT> phi_bb274_20;
  TNode<IntPtrT> phi_bb274_25;
  TNode<IntPtrT> phi_bb274_27;
  TNode<IntPtrT> phi_bb274_28;
  TNode<IntPtrT> phi_bb274_29;
  TNode<IntPtrT> phi_bb274_32;
  TNode<BoolT> phi_bb274_33;
  TNode<IntPtrT> phi_bb274_35;
  TNode<IntPtrT> phi_bb274_36;
  TNode<BoolT> phi_bb274_37;
  TNode<BoolT> phi_bb274_48;
  TNode<JSAny> phi_bb274_49;
  if (block274.is_used()) {
    ca_.Bind(&block274, &phi_bb274_20, &phi_bb274_25, &phi_bb274_27, &phi_bb274_28, &phi_bb274_29, &phi_bb274_32, &phi_bb274_33, &phi_bb274_35, &phi_bb274_36, &phi_bb274_37, &phi_bb274_48, &phi_bb274_49);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block276, phi_bb274_20, phi_bb274_25, phi_bb274_27, phi_bb274_28, phi_bb274_29, phi_bb274_32, phi_bb274_33, phi_bb274_35, phi_bb274_36, phi_bb274_37, phi_bb274_48, phi_bb274_49);
    } else {
      ca_.Goto(&block277, phi_bb274_20, phi_bb274_25, phi_bb274_27, phi_bb274_28, phi_bb274_29, phi_bb274_32, phi_bb274_33, phi_bb274_35, phi_bb274_36, phi_bb274_37, phi_bb274_48, phi_bb274_49);
    }
  }

  TNode<IntPtrT> phi_bb276_20;
  TNode<IntPtrT> phi_bb276_25;
  TNode<IntPtrT> phi_bb276_27;
  TNode<IntPtrT> phi_bb276_28;
  TNode<IntPtrT> phi_bb276_29;
  TNode<IntPtrT> phi_bb276_32;
  TNode<BoolT> phi_bb276_33;
  TNode<IntPtrT> phi_bb276_35;
  TNode<IntPtrT> phi_bb276_36;
  TNode<BoolT> phi_bb276_37;
  TNode<BoolT> phi_bb276_48;
  TNode<JSAny> phi_bb276_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp551;
  TNode<IntPtrT> tmp552;
  TNode<IntPtrT> tmp553;
  TNode<IntPtrT> tmp554;
  if (block276.is_used()) {
    ca_.Bind(&block276, &phi_bb276_20, &phi_bb276_25, &phi_bb276_27, &phi_bb276_28, &phi_bb276_29, &phi_bb276_32, &phi_bb276_33, &phi_bb276_35, &phi_bb276_36, &phi_bb276_37, &phi_bb276_48, &phi_bb276_49);
    std::tie(tmp551, tmp552) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb276_29}).Flatten();
    tmp553 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp554 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb276_29}, TNode<IntPtrT>{tmp553});
    ca_.Goto(&block275, phi_bb276_20, phi_bb276_25, phi_bb276_27, phi_bb276_28, tmp554, phi_bb276_32, phi_bb276_33, phi_bb276_35, phi_bb276_36, phi_bb276_37, phi_bb276_48, phi_bb276_49, tmp551, tmp552);
  }

  TNode<IntPtrT> phi_bb277_20;
  TNode<IntPtrT> phi_bb277_25;
  TNode<IntPtrT> phi_bb277_27;
  TNode<IntPtrT> phi_bb277_28;
  TNode<IntPtrT> phi_bb277_29;
  TNode<IntPtrT> phi_bb277_32;
  TNode<BoolT> phi_bb277_33;
  TNode<IntPtrT> phi_bb277_35;
  TNode<IntPtrT> phi_bb277_36;
  TNode<BoolT> phi_bb277_37;
  TNode<BoolT> phi_bb277_48;
  TNode<JSAny> phi_bb277_49;
  TNode<IntPtrT> tmp555;
  TNode<BoolT> tmp556;
  if (block277.is_used()) {
    ca_.Bind(&block277, &phi_bb277_20, &phi_bb277_25, &phi_bb277_27, &phi_bb277_28, &phi_bb277_29, &phi_bb277_32, &phi_bb277_33, &phi_bb277_35, &phi_bb277_36, &phi_bb277_37, &phi_bb277_48, &phi_bb277_49);
    tmp555 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp556 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb277_32}, TNode<IntPtrT>{tmp555});
    ca_.Branch(tmp556, &block279, std::vector<compiler::Node*>{phi_bb277_20, phi_bb277_25, phi_bb277_27, phi_bb277_28, phi_bb277_29, phi_bb277_32, phi_bb277_33, phi_bb277_35, phi_bb277_36, phi_bb277_37, phi_bb277_48, phi_bb277_49}, &block280, std::vector<compiler::Node*>{phi_bb277_20, phi_bb277_25, phi_bb277_27, phi_bb277_28, phi_bb277_29, phi_bb277_32, phi_bb277_33, phi_bb277_35, phi_bb277_36, phi_bb277_37, phi_bb277_48, phi_bb277_49});
  }

  TNode<IntPtrT> phi_bb279_20;
  TNode<IntPtrT> phi_bb279_25;
  TNode<IntPtrT> phi_bb279_27;
  TNode<IntPtrT> phi_bb279_28;
  TNode<IntPtrT> phi_bb279_29;
  TNode<IntPtrT> phi_bb279_32;
  TNode<BoolT> phi_bb279_33;
  TNode<IntPtrT> phi_bb279_35;
  TNode<IntPtrT> phi_bb279_36;
  TNode<BoolT> phi_bb279_37;
  TNode<BoolT> phi_bb279_48;
  TNode<JSAny> phi_bb279_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp557;
  TNode<IntPtrT> tmp558;
  TNode<IntPtrT> tmp559;
  TNode<BoolT> tmp560;
  if (block279.is_used()) {
    ca_.Bind(&block279, &phi_bb279_20, &phi_bb279_25, &phi_bb279_27, &phi_bb279_28, &phi_bb279_29, &phi_bb279_32, &phi_bb279_33, &phi_bb279_35, &phi_bb279_36, &phi_bb279_37, &phi_bb279_48, &phi_bb279_49);
    std::tie(tmp557, tmp558) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb279_32}).Flatten();
    tmp559 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp560 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block275, phi_bb279_20, phi_bb279_25, phi_bb279_27, phi_bb279_28, phi_bb279_29, tmp559, tmp560, phi_bb279_35, phi_bb279_36, phi_bb279_37, phi_bb279_48, phi_bb279_49, tmp557, tmp558);
  }

  TNode<IntPtrT> phi_bb280_20;
  TNode<IntPtrT> phi_bb280_25;
  TNode<IntPtrT> phi_bb280_27;
  TNode<IntPtrT> phi_bb280_28;
  TNode<IntPtrT> phi_bb280_29;
  TNode<IntPtrT> phi_bb280_32;
  TNode<BoolT> phi_bb280_33;
  TNode<IntPtrT> phi_bb280_35;
  TNode<IntPtrT> phi_bb280_36;
  TNode<BoolT> phi_bb280_37;
  TNode<BoolT> phi_bb280_48;
  TNode<JSAny> phi_bb280_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp561;
  TNode<IntPtrT> tmp562;
  TNode<IntPtrT> tmp563;
  TNode<IntPtrT> tmp564;
  TNode<IntPtrT> tmp565;
  TNode<IntPtrT> tmp566;
  TNode<BoolT> tmp567;
  if (block280.is_used()) {
    ca_.Bind(&block280, &phi_bb280_20, &phi_bb280_25, &phi_bb280_27, &phi_bb280_28, &phi_bb280_29, &phi_bb280_32, &phi_bb280_33, &phi_bb280_35, &phi_bb280_36, &phi_bb280_37, &phi_bb280_48, &phi_bb280_49);
    std::tie(tmp561, tmp562) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb280_29}).Flatten();
    tmp563 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp564 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb280_29}, TNode<IntPtrT>{tmp563});
    tmp565 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp566 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp564}, TNode<IntPtrT>{tmp565});
    tmp567 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block275, phi_bb280_20, phi_bb280_25, phi_bb280_27, phi_bb280_28, tmp566, tmp564, tmp567, phi_bb280_35, phi_bb280_36, phi_bb280_37, phi_bb280_48, phi_bb280_49, tmp561, tmp562);
  }

  TNode<IntPtrT> phi_bb275_20;
  TNode<IntPtrT> phi_bb275_25;
  TNode<IntPtrT> phi_bb275_27;
  TNode<IntPtrT> phi_bb275_28;
  TNode<IntPtrT> phi_bb275_29;
  TNode<IntPtrT> phi_bb275_32;
  TNode<BoolT> phi_bb275_33;
  TNode<IntPtrT> phi_bb275_35;
  TNode<IntPtrT> phi_bb275_36;
  TNode<BoolT> phi_bb275_37;
  TNode<BoolT> phi_bb275_48;
  TNode<JSAny> phi_bb275_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb275_51;
  TNode<IntPtrT> phi_bb275_52;
  if (block275.is_used()) {
    ca_.Bind(&block275, &phi_bb275_20, &phi_bb275_25, &phi_bb275_27, &phi_bb275_28, &phi_bb275_29, &phi_bb275_32, &phi_bb275_33, &phi_bb275_35, &phi_bb275_36, &phi_bb275_37, &phi_bb275_48, &phi_bb275_49, &phi_bb275_51, &phi_bb275_52);
    ca_.Goto(&block272, phi_bb275_20, phi_bb275_25, phi_bb275_27, phi_bb275_28, phi_bb275_29, phi_bb275_32, phi_bb275_33, phi_bb275_35, phi_bb275_36, phi_bb275_37, phi_bb275_48, phi_bb275_49, phi_bb275_51, phi_bb275_52);
  }

  TNode<IntPtrT> phi_bb272_20;
  TNode<IntPtrT> phi_bb272_25;
  TNode<IntPtrT> phi_bb272_27;
  TNode<IntPtrT> phi_bb272_28;
  TNode<IntPtrT> phi_bb272_29;
  TNode<IntPtrT> phi_bb272_32;
  TNode<BoolT> phi_bb272_33;
  TNode<IntPtrT> phi_bb272_35;
  TNode<IntPtrT> phi_bb272_36;
  TNode<BoolT> phi_bb272_37;
  TNode<BoolT> phi_bb272_48;
  TNode<JSAny> phi_bb272_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb272_51;
  TNode<IntPtrT> phi_bb272_52;
  if (block272.is_used()) {
    ca_.Bind(&block272, &phi_bb272_20, &phi_bb272_25, &phi_bb272_27, &phi_bb272_28, &phi_bb272_29, &phi_bb272_32, &phi_bb272_33, &phi_bb272_35, &phi_bb272_36, &phi_bb272_37, &phi_bb272_48, &phi_bb272_49, &phi_bb272_51, &phi_bb272_52);
    if ((((wasm::kIsFpAlwaysDouble || wasm::kIsBigEndian) || wasm::kIsBigEndianOnSim))) {
      ca_.Goto(&block281, phi_bb272_20, phi_bb272_25, phi_bb272_27, phi_bb272_28, phi_bb272_29, phi_bb272_32, phi_bb272_33, phi_bb272_35, phi_bb272_36, phi_bb272_37, phi_bb272_48, phi_bb272_49, phi_bb272_51, phi_bb272_52);
    } else {
      ca_.Goto(&block282, phi_bb272_20, phi_bb272_25, phi_bb272_27, phi_bb272_28, phi_bb272_29, phi_bb272_32, phi_bb272_33, phi_bb272_35, phi_bb272_36, phi_bb272_37, phi_bb272_48, phi_bb272_49, phi_bb272_51, phi_bb272_52);
    }
  }

  TNode<IntPtrT> phi_bb281_20;
  TNode<IntPtrT> phi_bb281_25;
  TNode<IntPtrT> phi_bb281_27;
  TNode<IntPtrT> phi_bb281_28;
  TNode<IntPtrT> phi_bb281_29;
  TNode<IntPtrT> phi_bb281_32;
  TNode<BoolT> phi_bb281_33;
  TNode<IntPtrT> phi_bb281_35;
  TNode<IntPtrT> phi_bb281_36;
  TNode<BoolT> phi_bb281_37;
  TNode<BoolT> phi_bb281_48;
  TNode<JSAny> phi_bb281_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb281_51;
  TNode<IntPtrT> phi_bb281_52;
  if (block281.is_used()) {
    ca_.Bind(&block281, &phi_bb281_20, &phi_bb281_25, &phi_bb281_27, &phi_bb281_28, &phi_bb281_29, &phi_bb281_32, &phi_bb281_33, &phi_bb281_35, &phi_bb281_36, &phi_bb281_37, &phi_bb281_48, &phi_bb281_49, &phi_bb281_51, &phi_bb281_52);
    HandleF32Returns_0(state_, TNode<NativeContext>{tmp441}, TorqueStructLocationAllocator_0{TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb281_25}, TNode<IntPtrT>{tmp544}, TNode<IntPtrT>{phi_bb281_27}, TNode<IntPtrT>{phi_bb281_28}, TNode<IntPtrT>{phi_bb281_29}, TNode<IntPtrT>{tmp479}, TNode<IntPtrT>{tmp480}, TNode<IntPtrT>{phi_bb281_32}, TNode<BoolT>{phi_bb281_33}}, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb281_51}, TNode<IntPtrT>{phi_bb281_52}, TorqueStructUnsafe_0{}}, TNode<JSAny>{phi_bb281_49});
    ca_.Goto(&block283, phi_bb281_20, phi_bb281_25, phi_bb281_27, phi_bb281_28, phi_bb281_29, phi_bb281_32, phi_bb281_33, phi_bb281_35, phi_bb281_36, phi_bb281_37, phi_bb281_48, phi_bb281_49, phi_bb281_51, phi_bb281_52);
  }

  TNode<IntPtrT> phi_bb282_20;
  TNode<IntPtrT> phi_bb282_25;
  TNode<IntPtrT> phi_bb282_27;
  TNode<IntPtrT> phi_bb282_28;
  TNode<IntPtrT> phi_bb282_29;
  TNode<IntPtrT> phi_bb282_32;
  TNode<BoolT> phi_bb282_33;
  TNode<IntPtrT> phi_bb282_35;
  TNode<IntPtrT> phi_bb282_36;
  TNode<BoolT> phi_bb282_37;
  TNode<BoolT> phi_bb282_48;
  TNode<JSAny> phi_bb282_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb282_51;
  TNode<IntPtrT> phi_bb282_52;
  TNode<Float32T> tmp568;
  TNode<Uint32T> tmp569;
  TNode<IntPtrT> tmp570;
  if (block282.is_used()) {
    ca_.Bind(&block282, &phi_bb282_20, &phi_bb282_25, &phi_bb282_27, &phi_bb282_28, &phi_bb282_29, &phi_bb282_32, &phi_bb282_33, &phi_bb282_35, &phi_bb282_36, &phi_bb282_37, &phi_bb282_48, &phi_bb282_49, &phi_bb282_51, &phi_bb282_52);
    tmp568 = ca_.CallBuiltin<Float32T>(Builtin::kWasmTaggedToFloat32, tmp441, phi_bb282_49);
    tmp569 = Bitcast_uint32_float32_0(state_, TNode<Float32T>{tmp568});
    tmp570 = Convert_intptr_uint32_0(state_, TNode<Uint32T>{tmp569});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb282_51, phi_bb282_52}, tmp570);
    ca_.Goto(&block283, phi_bb282_20, phi_bb282_25, phi_bb282_27, phi_bb282_28, phi_bb282_29, phi_bb282_32, phi_bb282_33, phi_bb282_35, phi_bb282_36, phi_bb282_37, phi_bb282_48, phi_bb282_49, phi_bb282_51, phi_bb282_52);
  }

  TNode<IntPtrT> phi_bb283_20;
  TNode<IntPtrT> phi_bb283_25;
  TNode<IntPtrT> phi_bb283_27;
  TNode<IntPtrT> phi_bb283_28;
  TNode<IntPtrT> phi_bb283_29;
  TNode<IntPtrT> phi_bb283_32;
  TNode<BoolT> phi_bb283_33;
  TNode<IntPtrT> phi_bb283_35;
  TNode<IntPtrT> phi_bb283_36;
  TNode<BoolT> phi_bb283_37;
  TNode<BoolT> phi_bb283_48;
  TNode<JSAny> phi_bb283_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb283_51;
  TNode<IntPtrT> phi_bb283_52;
  if (block283.is_used()) {
    ca_.Bind(&block283, &phi_bb283_20, &phi_bb283_25, &phi_bb283_27, &phi_bb283_28, &phi_bb283_29, &phi_bb283_32, &phi_bb283_33, &phi_bb283_35, &phi_bb283_36, &phi_bb283_37, &phi_bb283_48, &phi_bb283_49, &phi_bb283_51, &phi_bb283_52);
    ca_.Goto(&block271, phi_bb283_20, phi_bb283_25, tmp544, phi_bb283_27, phi_bb283_28, phi_bb283_29, phi_bb283_32, phi_bb283_33, phi_bb283_35, phi_bb283_36, phi_bb283_37, phi_bb283_48, phi_bb283_49);
  }

  TNode<IntPtrT> phi_bb270_20;
  TNode<IntPtrT> phi_bb270_25;
  TNode<IntPtrT> phi_bb270_26;
  TNode<IntPtrT> phi_bb270_27;
  TNode<IntPtrT> phi_bb270_28;
  TNode<IntPtrT> phi_bb270_29;
  TNode<IntPtrT> phi_bb270_32;
  TNode<BoolT> phi_bb270_33;
  TNode<IntPtrT> phi_bb270_35;
  TNode<IntPtrT> phi_bb270_36;
  TNode<BoolT> phi_bb270_37;
  TNode<BoolT> phi_bb270_48;
  TNode<JSAny> phi_bb270_49;
  TNode<Uint32T> tmp571;
  TNode<BoolT> tmp572;
  if (block270.is_used()) {
    ca_.Bind(&block270, &phi_bb270_20, &phi_bb270_25, &phi_bb270_26, &phi_bb270_27, &phi_bb270_28, &phi_bb270_29, &phi_bb270_32, &phi_bb270_33, &phi_bb270_35, &phi_bb270_36, &phi_bb270_37, &phi_bb270_48, &phi_bb270_49);
    tmp571 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmF64.raw_bit_field());
    tmp572 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp507}, TNode<Uint32T>{tmp571});
    ca_.Branch(tmp572, &block284, std::vector<compiler::Node*>{phi_bb270_20, phi_bb270_25, phi_bb270_26, phi_bb270_27, phi_bb270_28, phi_bb270_29, phi_bb270_32, phi_bb270_33, phi_bb270_35, phi_bb270_36, phi_bb270_37, phi_bb270_48, phi_bb270_49}, &block285, std::vector<compiler::Node*>{phi_bb270_20, phi_bb270_25, phi_bb270_26, phi_bb270_27, phi_bb270_28, phi_bb270_29, phi_bb270_32, phi_bb270_33, phi_bb270_35, phi_bb270_36, phi_bb270_37, phi_bb270_48, phi_bb270_49});
  }

  TNode<IntPtrT> phi_bb284_20;
  TNode<IntPtrT> phi_bb284_25;
  TNode<IntPtrT> phi_bb284_26;
  TNode<IntPtrT> phi_bb284_27;
  TNode<IntPtrT> phi_bb284_28;
  TNode<IntPtrT> phi_bb284_29;
  TNode<IntPtrT> phi_bb284_32;
  TNode<BoolT> phi_bb284_33;
  TNode<IntPtrT> phi_bb284_35;
  TNode<IntPtrT> phi_bb284_36;
  TNode<BoolT> phi_bb284_37;
  TNode<BoolT> phi_bb284_48;
  TNode<JSAny> phi_bb284_49;
  TNode<IntPtrT> tmp573;
  TNode<IntPtrT> tmp574;
  TNode<IntPtrT> tmp575;
  TNode<BoolT> tmp576;
  if (block284.is_used()) {
    ca_.Bind(&block284, &phi_bb284_20, &phi_bb284_25, &phi_bb284_26, &phi_bb284_27, &phi_bb284_28, &phi_bb284_29, &phi_bb284_32, &phi_bb284_33, &phi_bb284_35, &phi_bb284_36, &phi_bb284_37, &phi_bb284_48, &phi_bb284_49);
    tmp573 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp574 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb284_26}, TNode<IntPtrT>{tmp573});
    tmp575 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp576 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb284_26}, TNode<IntPtrT>{tmp575});
    ca_.Branch(tmp576, &block288, std::vector<compiler::Node*>{phi_bb284_20, phi_bb284_25, phi_bb284_27, phi_bb284_28, phi_bb284_29, phi_bb284_32, phi_bb284_33, phi_bb284_35, phi_bb284_36, phi_bb284_37, phi_bb284_48, phi_bb284_49}, &block289, std::vector<compiler::Node*>{phi_bb284_20, phi_bb284_25, phi_bb284_27, phi_bb284_28, phi_bb284_29, phi_bb284_32, phi_bb284_33, phi_bb284_35, phi_bb284_36, phi_bb284_37, phi_bb284_48, phi_bb284_49});
  }

  TNode<IntPtrT> phi_bb288_20;
  TNode<IntPtrT> phi_bb288_25;
  TNode<IntPtrT> phi_bb288_27;
  TNode<IntPtrT> phi_bb288_28;
  TNode<IntPtrT> phi_bb288_29;
  TNode<IntPtrT> phi_bb288_32;
  TNode<BoolT> phi_bb288_33;
  TNode<IntPtrT> phi_bb288_35;
  TNode<IntPtrT> phi_bb288_36;
  TNode<BoolT> phi_bb288_37;
  TNode<BoolT> phi_bb288_48;
  TNode<JSAny> phi_bb288_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp577;
  TNode<IntPtrT> tmp578;
  TNode<IntPtrT> tmp579;
  TNode<IntPtrT> tmp580;
  if (block288.is_used()) {
    ca_.Bind(&block288, &phi_bb288_20, &phi_bb288_25, &phi_bb288_27, &phi_bb288_28, &phi_bb288_29, &phi_bb288_32, &phi_bb288_33, &phi_bb288_35, &phi_bb288_36, &phi_bb288_37, &phi_bb288_48, &phi_bb288_49);
    std::tie(tmp577, tmp578) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb288_28}).Flatten();
    tmp579 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    tmp580 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb288_28}, TNode<IntPtrT>{tmp579});
    ca_.Goto(&block287, phi_bb288_20, phi_bb288_25, phi_bb288_27, tmp580, phi_bb288_29, phi_bb288_32, phi_bb288_33, phi_bb288_35, phi_bb288_36, phi_bb288_37, phi_bb288_48, phi_bb288_49, tmp577, tmp578);
  }

  TNode<IntPtrT> phi_bb289_20;
  TNode<IntPtrT> phi_bb289_25;
  TNode<IntPtrT> phi_bb289_27;
  TNode<IntPtrT> phi_bb289_28;
  TNode<IntPtrT> phi_bb289_29;
  TNode<IntPtrT> phi_bb289_32;
  TNode<BoolT> phi_bb289_33;
  TNode<IntPtrT> phi_bb289_35;
  TNode<IntPtrT> phi_bb289_36;
  TNode<BoolT> phi_bb289_37;
  TNode<BoolT> phi_bb289_48;
  TNode<JSAny> phi_bb289_49;
  if (block289.is_used()) {
    ca_.Bind(&block289, &phi_bb289_20, &phi_bb289_25, &phi_bb289_27, &phi_bb289_28, &phi_bb289_29, &phi_bb289_32, &phi_bb289_33, &phi_bb289_35, &phi_bb289_36, &phi_bb289_37, &phi_bb289_48, &phi_bb289_49);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block290, phi_bb289_20, phi_bb289_25, phi_bb289_27, phi_bb289_28, phi_bb289_29, phi_bb289_32, phi_bb289_33, phi_bb289_35, phi_bb289_36, phi_bb289_37, phi_bb289_48, phi_bb289_49);
    } else {
      ca_.Goto(&block291, phi_bb289_20, phi_bb289_25, phi_bb289_27, phi_bb289_28, phi_bb289_29, phi_bb289_32, phi_bb289_33, phi_bb289_35, phi_bb289_36, phi_bb289_37, phi_bb289_48, phi_bb289_49);
    }
  }

  TNode<IntPtrT> phi_bb290_20;
  TNode<IntPtrT> phi_bb290_25;
  TNode<IntPtrT> phi_bb290_27;
  TNode<IntPtrT> phi_bb290_28;
  TNode<IntPtrT> phi_bb290_29;
  TNode<IntPtrT> phi_bb290_32;
  TNode<BoolT> phi_bb290_33;
  TNode<IntPtrT> phi_bb290_35;
  TNode<IntPtrT> phi_bb290_36;
  TNode<BoolT> phi_bb290_37;
  TNode<BoolT> phi_bb290_48;
  TNode<JSAny> phi_bb290_49;
  if (block290.is_used()) {
    ca_.Bind(&block290, &phi_bb290_20, &phi_bb290_25, &phi_bb290_27, &phi_bb290_28, &phi_bb290_29, &phi_bb290_32, &phi_bb290_33, &phi_bb290_35, &phi_bb290_36, &phi_bb290_37, &phi_bb290_48, &phi_bb290_49);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block294, phi_bb290_20, phi_bb290_25, phi_bb290_27, phi_bb290_28, phi_bb290_29, phi_bb290_32, phi_bb290_33, phi_bb290_35, phi_bb290_36, phi_bb290_37, phi_bb290_48, phi_bb290_49);
    } else {
      ca_.Goto(&block295, phi_bb290_20, phi_bb290_25, phi_bb290_27, phi_bb290_28, phi_bb290_29, phi_bb290_32, phi_bb290_33, phi_bb290_35, phi_bb290_36, phi_bb290_37, phi_bb290_48, phi_bb290_49);
    }
  }

  TNode<IntPtrT> phi_bb294_20;
  TNode<IntPtrT> phi_bb294_25;
  TNode<IntPtrT> phi_bb294_27;
  TNode<IntPtrT> phi_bb294_28;
  TNode<IntPtrT> phi_bb294_29;
  TNode<IntPtrT> phi_bb294_32;
  TNode<BoolT> phi_bb294_33;
  TNode<IntPtrT> phi_bb294_35;
  TNode<IntPtrT> phi_bb294_36;
  TNode<BoolT> phi_bb294_37;
  TNode<BoolT> phi_bb294_48;
  TNode<JSAny> phi_bb294_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp581;
  TNode<IntPtrT> tmp582;
  TNode<IntPtrT> tmp583;
  TNode<IntPtrT> tmp584;
  if (block294.is_used()) {
    ca_.Bind(&block294, &phi_bb294_20, &phi_bb294_25, &phi_bb294_27, &phi_bb294_28, &phi_bb294_29, &phi_bb294_32, &phi_bb294_33, &phi_bb294_35, &phi_bb294_36, &phi_bb294_37, &phi_bb294_48, &phi_bb294_49);
    std::tie(tmp581, tmp582) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb294_29}).Flatten();
    tmp583 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp584 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb294_29}, TNode<IntPtrT>{tmp583});
    ca_.Goto(&block293, phi_bb294_20, phi_bb294_25, phi_bb294_27, phi_bb294_28, tmp584, phi_bb294_32, phi_bb294_33, phi_bb294_35, phi_bb294_36, phi_bb294_37, phi_bb294_48, phi_bb294_49, tmp581, tmp582);
  }

  TNode<IntPtrT> phi_bb295_20;
  TNode<IntPtrT> phi_bb295_25;
  TNode<IntPtrT> phi_bb295_27;
  TNode<IntPtrT> phi_bb295_28;
  TNode<IntPtrT> phi_bb295_29;
  TNode<IntPtrT> phi_bb295_32;
  TNode<BoolT> phi_bb295_33;
  TNode<IntPtrT> phi_bb295_35;
  TNode<IntPtrT> phi_bb295_36;
  TNode<BoolT> phi_bb295_37;
  TNode<BoolT> phi_bb295_48;
  TNode<JSAny> phi_bb295_49;
  TNode<IntPtrT> tmp585;
  TNode<BoolT> tmp586;
  if (block295.is_used()) {
    ca_.Bind(&block295, &phi_bb295_20, &phi_bb295_25, &phi_bb295_27, &phi_bb295_28, &phi_bb295_29, &phi_bb295_32, &phi_bb295_33, &phi_bb295_35, &phi_bb295_36, &phi_bb295_37, &phi_bb295_48, &phi_bb295_49);
    tmp585 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp586 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb295_32}, TNode<IntPtrT>{tmp585});
    ca_.Branch(tmp586, &block297, std::vector<compiler::Node*>{phi_bb295_20, phi_bb295_25, phi_bb295_27, phi_bb295_28, phi_bb295_29, phi_bb295_32, phi_bb295_33, phi_bb295_35, phi_bb295_36, phi_bb295_37, phi_bb295_48, phi_bb295_49}, &block298, std::vector<compiler::Node*>{phi_bb295_20, phi_bb295_25, phi_bb295_27, phi_bb295_28, phi_bb295_29, phi_bb295_32, phi_bb295_33, phi_bb295_35, phi_bb295_36, phi_bb295_37, phi_bb295_48, phi_bb295_49});
  }

  TNode<IntPtrT> phi_bb297_20;
  TNode<IntPtrT> phi_bb297_25;
  TNode<IntPtrT> phi_bb297_27;
  TNode<IntPtrT> phi_bb297_28;
  TNode<IntPtrT> phi_bb297_29;
  TNode<IntPtrT> phi_bb297_32;
  TNode<BoolT> phi_bb297_33;
  TNode<IntPtrT> phi_bb297_35;
  TNode<IntPtrT> phi_bb297_36;
  TNode<BoolT> phi_bb297_37;
  TNode<BoolT> phi_bb297_48;
  TNode<JSAny> phi_bb297_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp587;
  TNode<IntPtrT> tmp588;
  TNode<IntPtrT> tmp589;
  TNode<BoolT> tmp590;
  if (block297.is_used()) {
    ca_.Bind(&block297, &phi_bb297_20, &phi_bb297_25, &phi_bb297_27, &phi_bb297_28, &phi_bb297_29, &phi_bb297_32, &phi_bb297_33, &phi_bb297_35, &phi_bb297_36, &phi_bb297_37, &phi_bb297_48, &phi_bb297_49);
    std::tie(tmp587, tmp588) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb297_32}).Flatten();
    tmp589 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp590 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block293, phi_bb297_20, phi_bb297_25, phi_bb297_27, phi_bb297_28, phi_bb297_29, tmp589, tmp590, phi_bb297_35, phi_bb297_36, phi_bb297_37, phi_bb297_48, phi_bb297_49, tmp587, tmp588);
  }

  TNode<IntPtrT> phi_bb298_20;
  TNode<IntPtrT> phi_bb298_25;
  TNode<IntPtrT> phi_bb298_27;
  TNode<IntPtrT> phi_bb298_28;
  TNode<IntPtrT> phi_bb298_29;
  TNode<IntPtrT> phi_bb298_32;
  TNode<BoolT> phi_bb298_33;
  TNode<IntPtrT> phi_bb298_35;
  TNode<IntPtrT> phi_bb298_36;
  TNode<BoolT> phi_bb298_37;
  TNode<BoolT> phi_bb298_48;
  TNode<JSAny> phi_bb298_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp591;
  TNode<IntPtrT> tmp592;
  TNode<IntPtrT> tmp593;
  TNode<IntPtrT> tmp594;
  TNode<IntPtrT> tmp595;
  TNode<IntPtrT> tmp596;
  TNode<BoolT> tmp597;
  if (block298.is_used()) {
    ca_.Bind(&block298, &phi_bb298_20, &phi_bb298_25, &phi_bb298_27, &phi_bb298_28, &phi_bb298_29, &phi_bb298_32, &phi_bb298_33, &phi_bb298_35, &phi_bb298_36, &phi_bb298_37, &phi_bb298_48, &phi_bb298_49);
    std::tie(tmp591, tmp592) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb298_29}).Flatten();
    tmp593 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp594 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb298_29}, TNode<IntPtrT>{tmp593});
    tmp595 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp596 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp594}, TNode<IntPtrT>{tmp595});
    tmp597 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block293, phi_bb298_20, phi_bb298_25, phi_bb298_27, phi_bb298_28, tmp596, tmp594, tmp597, phi_bb298_35, phi_bb298_36, phi_bb298_37, phi_bb298_48, phi_bb298_49, tmp591, tmp592);
  }

  TNode<IntPtrT> phi_bb293_20;
  TNode<IntPtrT> phi_bb293_25;
  TNode<IntPtrT> phi_bb293_27;
  TNode<IntPtrT> phi_bb293_28;
  TNode<IntPtrT> phi_bb293_29;
  TNode<IntPtrT> phi_bb293_32;
  TNode<BoolT> phi_bb293_33;
  TNode<IntPtrT> phi_bb293_35;
  TNode<IntPtrT> phi_bb293_36;
  TNode<BoolT> phi_bb293_37;
  TNode<BoolT> phi_bb293_48;
  TNode<JSAny> phi_bb293_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb293_51;
  TNode<IntPtrT> phi_bb293_52;
  if (block293.is_used()) {
    ca_.Bind(&block293, &phi_bb293_20, &phi_bb293_25, &phi_bb293_27, &phi_bb293_28, &phi_bb293_29, &phi_bb293_32, &phi_bb293_33, &phi_bb293_35, &phi_bb293_36, &phi_bb293_37, &phi_bb293_48, &phi_bb293_49, &phi_bb293_51, &phi_bb293_52);
    ca_.Goto(&block287, phi_bb293_20, phi_bb293_25, phi_bb293_27, phi_bb293_28, phi_bb293_29, phi_bb293_32, phi_bb293_33, phi_bb293_35, phi_bb293_36, phi_bb293_37, phi_bb293_48, phi_bb293_49, phi_bb293_51, phi_bb293_52);
  }

  TNode<IntPtrT> phi_bb291_20;
  TNode<IntPtrT> phi_bb291_25;
  TNode<IntPtrT> phi_bb291_27;
  TNode<IntPtrT> phi_bb291_28;
  TNode<IntPtrT> phi_bb291_29;
  TNode<IntPtrT> phi_bb291_32;
  TNode<BoolT> phi_bb291_33;
  TNode<IntPtrT> phi_bb291_35;
  TNode<IntPtrT> phi_bb291_36;
  TNode<BoolT> phi_bb291_37;
  TNode<BoolT> phi_bb291_48;
  TNode<JSAny> phi_bb291_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp598;
  TNode<IntPtrT> tmp599;
  TNode<IntPtrT> tmp600;
  TNode<IntPtrT> tmp601;
  TNode<BoolT> tmp602;
  if (block291.is_used()) {
    ca_.Bind(&block291, &phi_bb291_20, &phi_bb291_25, &phi_bb291_27, &phi_bb291_28, &phi_bb291_29, &phi_bb291_32, &phi_bb291_33, &phi_bb291_35, &phi_bb291_36, &phi_bb291_37, &phi_bb291_48, &phi_bb291_49);
    std::tie(tmp598, tmp599) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb291_29}).Flatten();
    tmp600 = FromConstexpr_intptr_constexpr_int31_0(state_, (CodeStubAssembler(state_).ConstexprInt31Mul((FromConstexpr_constexpr_int31_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x2ull))), (SizeOf_intptr_0(state_)))));
    tmp601 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb291_29}, TNode<IntPtrT>{tmp600});
    tmp602 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block287, phi_bb291_20, phi_bb291_25, phi_bb291_27, phi_bb291_28, tmp601, phi_bb291_32, tmp602, phi_bb291_35, phi_bb291_36, phi_bb291_37, phi_bb291_48, phi_bb291_49, tmp598, tmp599);
  }

  TNode<IntPtrT> phi_bb287_20;
  TNode<IntPtrT> phi_bb287_25;
  TNode<IntPtrT> phi_bb287_27;
  TNode<IntPtrT> phi_bb287_28;
  TNode<IntPtrT> phi_bb287_29;
  TNode<IntPtrT> phi_bb287_32;
  TNode<BoolT> phi_bb287_33;
  TNode<IntPtrT> phi_bb287_35;
  TNode<IntPtrT> phi_bb287_36;
  TNode<BoolT> phi_bb287_37;
  TNode<BoolT> phi_bb287_48;
  TNode<JSAny> phi_bb287_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb287_51;
  TNode<IntPtrT> phi_bb287_52;
  TNode<Union<HeapObject, TaggedIndex>> tmp603;
  TNode<IntPtrT> tmp604;
  TNode<Float64T> tmp605;
  TNode<Float64T> tmp606;
  if (block287.is_used()) {
    ca_.Bind(&block287, &phi_bb287_20, &phi_bb287_25, &phi_bb287_27, &phi_bb287_28, &phi_bb287_29, &phi_bb287_32, &phi_bb287_33, &phi_bb287_35, &phi_bb287_36, &phi_bb287_37, &phi_bb287_48, &phi_bb287_49, &phi_bb287_51, &phi_bb287_52);
    std::tie(tmp603, tmp604) = RefCast_float64_0(state_, TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{phi_bb287_51}, TNode<IntPtrT>{phi_bb287_52}, TorqueStructUnsafe_0{}}).Flatten();
    tmp605 = CodeStubAssembler(state_).ChangeTaggedToFloat64(TNode<Context>{tmp441}, TNode<JSAny>{phi_bb287_49});
    tmp606 = CodeStubAssembler(state_).Float64SilenceNaN(TNode<Float64T>{tmp605});
    CodeStubAssembler(state_).StoreReference<Float64T>(CodeStubAssembler::Reference{tmp603, tmp604}, tmp606);
    ca_.Goto(&block286, phi_bb287_20, phi_bb287_25, tmp574, phi_bb287_27, phi_bb287_28, phi_bb287_29, phi_bb287_32, phi_bb287_33, phi_bb287_35, phi_bb287_36, phi_bb287_37, phi_bb287_48, phi_bb287_49);
  }

  TNode<IntPtrT> phi_bb285_20;
  TNode<IntPtrT> phi_bb285_25;
  TNode<IntPtrT> phi_bb285_26;
  TNode<IntPtrT> phi_bb285_27;
  TNode<IntPtrT> phi_bb285_28;
  TNode<IntPtrT> phi_bb285_29;
  TNode<IntPtrT> phi_bb285_32;
  TNode<BoolT> phi_bb285_33;
  TNode<IntPtrT> phi_bb285_35;
  TNode<IntPtrT> phi_bb285_36;
  TNode<BoolT> phi_bb285_37;
  TNode<BoolT> phi_bb285_48;
  TNode<JSAny> phi_bb285_49;
  TNode<Uint32T> tmp607;
  TNode<BoolT> tmp608;
  if (block285.is_used()) {
    ca_.Bind(&block285, &phi_bb285_20, &phi_bb285_25, &phi_bb285_26, &phi_bb285_27, &phi_bb285_28, &phi_bb285_29, &phi_bb285_32, &phi_bb285_33, &phi_bb285_35, &phi_bb285_36, &phi_bb285_37, &phi_bb285_48, &phi_bb285_49);
    tmp607 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::kWasmI64.raw_bit_field());
    tmp608 = CodeStubAssembler(state_).Word32Equal(TNode<Uint32T>{tmp507}, TNode<Uint32T>{tmp607});
    ca_.Branch(tmp608, &block299, std::vector<compiler::Node*>{phi_bb285_20, phi_bb285_25, phi_bb285_26, phi_bb285_27, phi_bb285_28, phi_bb285_29, phi_bb285_32, phi_bb285_33, phi_bb285_35, phi_bb285_36, phi_bb285_37, phi_bb285_48, phi_bb285_49}, &block300, std::vector<compiler::Node*>{phi_bb285_20, phi_bb285_25, phi_bb285_26, phi_bb285_27, phi_bb285_28, phi_bb285_29, phi_bb285_32, phi_bb285_33, phi_bb285_35, phi_bb285_36, phi_bb285_37, phi_bb285_48, phi_bb285_49});
  }

  TNode<IntPtrT> phi_bb299_20;
  TNode<IntPtrT> phi_bb299_25;
  TNode<IntPtrT> phi_bb299_26;
  TNode<IntPtrT> phi_bb299_27;
  TNode<IntPtrT> phi_bb299_28;
  TNode<IntPtrT> phi_bb299_29;
  TNode<IntPtrT> phi_bb299_32;
  TNode<BoolT> phi_bb299_33;
  TNode<IntPtrT> phi_bb299_35;
  TNode<IntPtrT> phi_bb299_36;
  TNode<BoolT> phi_bb299_37;
  TNode<BoolT> phi_bb299_48;
  TNode<JSAny> phi_bb299_49;
  if (block299.is_used()) {
    ca_.Bind(&block299, &phi_bb299_20, &phi_bb299_25, &phi_bb299_26, &phi_bb299_27, &phi_bb299_28, &phi_bb299_29, &phi_bb299_32, &phi_bb299_33, &phi_bb299_35, &phi_bb299_36, &phi_bb299_37, &phi_bb299_48, &phi_bb299_49);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block302, phi_bb299_20, phi_bb299_25, phi_bb299_26, phi_bb299_27, phi_bb299_28, phi_bb299_29, phi_bb299_32, phi_bb299_33, phi_bb299_35, phi_bb299_36, phi_bb299_37, phi_bb299_48, phi_bb299_49);
    } else {
      ca_.Goto(&block303, phi_bb299_20, phi_bb299_25, phi_bb299_26, phi_bb299_27, phi_bb299_28, phi_bb299_29, phi_bb299_32, phi_bb299_33, phi_bb299_35, phi_bb299_36, phi_bb299_37, phi_bb299_48, phi_bb299_49);
    }
  }

  TNode<IntPtrT> phi_bb302_20;
  TNode<IntPtrT> phi_bb302_25;
  TNode<IntPtrT> phi_bb302_26;
  TNode<IntPtrT> phi_bb302_27;
  TNode<IntPtrT> phi_bb302_28;
  TNode<IntPtrT> phi_bb302_29;
  TNode<IntPtrT> phi_bb302_32;
  TNode<BoolT> phi_bb302_33;
  TNode<IntPtrT> phi_bb302_35;
  TNode<IntPtrT> phi_bb302_36;
  TNode<BoolT> phi_bb302_37;
  TNode<BoolT> phi_bb302_48;
  TNode<JSAny> phi_bb302_49;
  TNode<IntPtrT> tmp609;
  TNode<IntPtrT> tmp610;
  TNode<IntPtrT> tmp611;
  TNode<BoolT> tmp612;
  if (block302.is_used()) {
    ca_.Bind(&block302, &phi_bb302_20, &phi_bb302_25, &phi_bb302_26, &phi_bb302_27, &phi_bb302_28, &phi_bb302_29, &phi_bb302_32, &phi_bb302_33, &phi_bb302_35, &phi_bb302_36, &phi_bb302_37, &phi_bb302_48, &phi_bb302_49);
    tmp609 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp610 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb302_25}, TNode<IntPtrT>{tmp609});
    tmp611 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp612 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb302_25}, TNode<IntPtrT>{tmp611});
    ca_.Branch(tmp612, &block306, std::vector<compiler::Node*>{phi_bb302_20, phi_bb302_26, phi_bb302_27, phi_bb302_28, phi_bb302_29, phi_bb302_32, phi_bb302_33, phi_bb302_35, phi_bb302_36, phi_bb302_37, phi_bb302_48, phi_bb302_49}, &block307, std::vector<compiler::Node*>{phi_bb302_20, phi_bb302_26, phi_bb302_27, phi_bb302_28, phi_bb302_29, phi_bb302_32, phi_bb302_33, phi_bb302_35, phi_bb302_36, phi_bb302_37, phi_bb302_48, phi_bb302_49});
  }

  TNode<IntPtrT> phi_bb306_20;
  TNode<IntPtrT> phi_bb306_26;
  TNode<IntPtrT> phi_bb306_27;
  TNode<IntPtrT> phi_bb306_28;
  TNode<IntPtrT> phi_bb306_29;
  TNode<IntPtrT> phi_bb306_32;
  TNode<BoolT> phi_bb306_33;
  TNode<IntPtrT> phi_bb306_35;
  TNode<IntPtrT> phi_bb306_36;
  TNode<BoolT> phi_bb306_37;
  TNode<BoolT> phi_bb306_48;
  TNode<JSAny> phi_bb306_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp613;
  TNode<IntPtrT> tmp614;
  TNode<IntPtrT> tmp615;
  TNode<IntPtrT> tmp616;
  if (block306.is_used()) {
    ca_.Bind(&block306, &phi_bb306_20, &phi_bb306_26, &phi_bb306_27, &phi_bb306_28, &phi_bb306_29, &phi_bb306_32, &phi_bb306_33, &phi_bb306_35, &phi_bb306_36, &phi_bb306_37, &phi_bb306_48, &phi_bb306_49);
    std::tie(tmp613, tmp614) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb306_27}).Flatten();
    tmp615 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp616 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb306_27}, TNode<IntPtrT>{tmp615});
    ca_.Goto(&block305, phi_bb306_20, phi_bb306_26, tmp616, phi_bb306_28, phi_bb306_29, phi_bb306_32, phi_bb306_33, phi_bb306_35, phi_bb306_36, phi_bb306_37, phi_bb306_48, phi_bb306_49, tmp613, tmp614);
  }

  TNode<IntPtrT> phi_bb307_20;
  TNode<IntPtrT> phi_bb307_26;
  TNode<IntPtrT> phi_bb307_27;
  TNode<IntPtrT> phi_bb307_28;
  TNode<IntPtrT> phi_bb307_29;
  TNode<IntPtrT> phi_bb307_32;
  TNode<BoolT> phi_bb307_33;
  TNode<IntPtrT> phi_bb307_35;
  TNode<IntPtrT> phi_bb307_36;
  TNode<BoolT> phi_bb307_37;
  TNode<BoolT> phi_bb307_48;
  TNode<JSAny> phi_bb307_49;
  if (block307.is_used()) {
    ca_.Bind(&block307, &phi_bb307_20, &phi_bb307_26, &phi_bb307_27, &phi_bb307_28, &phi_bb307_29, &phi_bb307_32, &phi_bb307_33, &phi_bb307_35, &phi_bb307_36, &phi_bb307_37, &phi_bb307_48, &phi_bb307_49);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block309, phi_bb307_20, phi_bb307_26, phi_bb307_27, phi_bb307_28, phi_bb307_29, phi_bb307_32, phi_bb307_33, phi_bb307_35, phi_bb307_36, phi_bb307_37, phi_bb307_48, phi_bb307_49);
    } else {
      ca_.Goto(&block310, phi_bb307_20, phi_bb307_26, phi_bb307_27, phi_bb307_28, phi_bb307_29, phi_bb307_32, phi_bb307_33, phi_bb307_35, phi_bb307_36, phi_bb307_37, phi_bb307_48, phi_bb307_49);
    }
  }

  TNode<IntPtrT> phi_bb309_20;
  TNode<IntPtrT> phi_bb309_26;
  TNode<IntPtrT> phi_bb309_27;
  TNode<IntPtrT> phi_bb309_28;
  TNode<IntPtrT> phi_bb309_29;
  TNode<IntPtrT> phi_bb309_32;
  TNode<BoolT> phi_bb309_33;
  TNode<IntPtrT> phi_bb309_35;
  TNode<IntPtrT> phi_bb309_36;
  TNode<BoolT> phi_bb309_37;
  TNode<BoolT> phi_bb309_48;
  TNode<JSAny> phi_bb309_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp617;
  TNode<IntPtrT> tmp618;
  TNode<IntPtrT> tmp619;
  TNode<IntPtrT> tmp620;
  if (block309.is_used()) {
    ca_.Bind(&block309, &phi_bb309_20, &phi_bb309_26, &phi_bb309_27, &phi_bb309_28, &phi_bb309_29, &phi_bb309_32, &phi_bb309_33, &phi_bb309_35, &phi_bb309_36, &phi_bb309_37, &phi_bb309_48, &phi_bb309_49);
    std::tie(tmp617, tmp618) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb309_29}).Flatten();
    tmp619 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp620 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb309_29}, TNode<IntPtrT>{tmp619});
    ca_.Goto(&block308, phi_bb309_20, phi_bb309_26, phi_bb309_27, phi_bb309_28, tmp620, phi_bb309_32, phi_bb309_33, phi_bb309_35, phi_bb309_36, phi_bb309_37, phi_bb309_48, phi_bb309_49, tmp617, tmp618);
  }

  TNode<IntPtrT> phi_bb310_20;
  TNode<IntPtrT> phi_bb310_26;
  TNode<IntPtrT> phi_bb310_27;
  TNode<IntPtrT> phi_bb310_28;
  TNode<IntPtrT> phi_bb310_29;
  TNode<IntPtrT> phi_bb310_32;
  TNode<BoolT> phi_bb310_33;
  TNode<IntPtrT> phi_bb310_35;
  TNode<IntPtrT> phi_bb310_36;
  TNode<BoolT> phi_bb310_37;
  TNode<BoolT> phi_bb310_48;
  TNode<JSAny> phi_bb310_49;
  TNode<IntPtrT> tmp621;
  TNode<BoolT> tmp622;
  if (block310.is_used()) {
    ca_.Bind(&block310, &phi_bb310_20, &phi_bb310_26, &phi_bb310_27, &phi_bb310_28, &phi_bb310_29, &phi_bb310_32, &phi_bb310_33, &phi_bb310_35, &phi_bb310_36, &phi_bb310_37, &phi_bb310_48, &phi_bb310_49);
    tmp621 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp622 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb310_32}, TNode<IntPtrT>{tmp621});
    ca_.Branch(tmp622, &block312, std::vector<compiler::Node*>{phi_bb310_20, phi_bb310_26, phi_bb310_27, phi_bb310_28, phi_bb310_29, phi_bb310_32, phi_bb310_33, phi_bb310_35, phi_bb310_36, phi_bb310_37, phi_bb310_48, phi_bb310_49}, &block313, std::vector<compiler::Node*>{phi_bb310_20, phi_bb310_26, phi_bb310_27, phi_bb310_28, phi_bb310_29, phi_bb310_32, phi_bb310_33, phi_bb310_35, phi_bb310_36, phi_bb310_37, phi_bb310_48, phi_bb310_49});
  }

  TNode<IntPtrT> phi_bb312_20;
  TNode<IntPtrT> phi_bb312_26;
  TNode<IntPtrT> phi_bb312_27;
  TNode<IntPtrT> phi_bb312_28;
  TNode<IntPtrT> phi_bb312_29;
  TNode<IntPtrT> phi_bb312_32;
  TNode<BoolT> phi_bb312_33;
  TNode<IntPtrT> phi_bb312_35;
  TNode<IntPtrT> phi_bb312_36;
  TNode<BoolT> phi_bb312_37;
  TNode<BoolT> phi_bb312_48;
  TNode<JSAny> phi_bb312_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp623;
  TNode<IntPtrT> tmp624;
  TNode<IntPtrT> tmp625;
  TNode<BoolT> tmp626;
  if (block312.is_used()) {
    ca_.Bind(&block312, &phi_bb312_20, &phi_bb312_26, &phi_bb312_27, &phi_bb312_28, &phi_bb312_29, &phi_bb312_32, &phi_bb312_33, &phi_bb312_35, &phi_bb312_36, &phi_bb312_37, &phi_bb312_48, &phi_bb312_49);
    std::tie(tmp623, tmp624) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb312_32}).Flatten();
    tmp625 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp626 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block308, phi_bb312_20, phi_bb312_26, phi_bb312_27, phi_bb312_28, phi_bb312_29, tmp625, tmp626, phi_bb312_35, phi_bb312_36, phi_bb312_37, phi_bb312_48, phi_bb312_49, tmp623, tmp624);
  }

  TNode<IntPtrT> phi_bb313_20;
  TNode<IntPtrT> phi_bb313_26;
  TNode<IntPtrT> phi_bb313_27;
  TNode<IntPtrT> phi_bb313_28;
  TNode<IntPtrT> phi_bb313_29;
  TNode<IntPtrT> phi_bb313_32;
  TNode<BoolT> phi_bb313_33;
  TNode<IntPtrT> phi_bb313_35;
  TNode<IntPtrT> phi_bb313_36;
  TNode<BoolT> phi_bb313_37;
  TNode<BoolT> phi_bb313_48;
  TNode<JSAny> phi_bb313_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp627;
  TNode<IntPtrT> tmp628;
  TNode<IntPtrT> tmp629;
  TNode<IntPtrT> tmp630;
  TNode<IntPtrT> tmp631;
  TNode<IntPtrT> tmp632;
  TNode<BoolT> tmp633;
  if (block313.is_used()) {
    ca_.Bind(&block313, &phi_bb313_20, &phi_bb313_26, &phi_bb313_27, &phi_bb313_28, &phi_bb313_29, &phi_bb313_32, &phi_bb313_33, &phi_bb313_35, &phi_bb313_36, &phi_bb313_37, &phi_bb313_48, &phi_bb313_49);
    std::tie(tmp627, tmp628) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb313_29}).Flatten();
    tmp629 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp630 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb313_29}, TNode<IntPtrT>{tmp629});
    tmp631 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp632 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp630}, TNode<IntPtrT>{tmp631});
    tmp633 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block308, phi_bb313_20, phi_bb313_26, phi_bb313_27, phi_bb313_28, tmp632, tmp630, tmp633, phi_bb313_35, phi_bb313_36, phi_bb313_37, phi_bb313_48, phi_bb313_49, tmp627, tmp628);
  }

  TNode<IntPtrT> phi_bb308_20;
  TNode<IntPtrT> phi_bb308_26;
  TNode<IntPtrT> phi_bb308_27;
  TNode<IntPtrT> phi_bb308_28;
  TNode<IntPtrT> phi_bb308_29;
  TNode<IntPtrT> phi_bb308_32;
  TNode<BoolT> phi_bb308_33;
  TNode<IntPtrT> phi_bb308_35;
  TNode<IntPtrT> phi_bb308_36;
  TNode<BoolT> phi_bb308_37;
  TNode<BoolT> phi_bb308_48;
  TNode<JSAny> phi_bb308_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb308_51;
  TNode<IntPtrT> phi_bb308_52;
  if (block308.is_used()) {
    ca_.Bind(&block308, &phi_bb308_20, &phi_bb308_26, &phi_bb308_27, &phi_bb308_28, &phi_bb308_29, &phi_bb308_32, &phi_bb308_33, &phi_bb308_35, &phi_bb308_36, &phi_bb308_37, &phi_bb308_48, &phi_bb308_49, &phi_bb308_51, &phi_bb308_52);
    ca_.Goto(&block305, phi_bb308_20, phi_bb308_26, phi_bb308_27, phi_bb308_28, phi_bb308_29, phi_bb308_32, phi_bb308_33, phi_bb308_35, phi_bb308_36, phi_bb308_37, phi_bb308_48, phi_bb308_49, phi_bb308_51, phi_bb308_52);
  }

  TNode<IntPtrT> phi_bb305_20;
  TNode<IntPtrT> phi_bb305_26;
  TNode<IntPtrT> phi_bb305_27;
  TNode<IntPtrT> phi_bb305_28;
  TNode<IntPtrT> phi_bb305_29;
  TNode<IntPtrT> phi_bb305_32;
  TNode<BoolT> phi_bb305_33;
  TNode<IntPtrT> phi_bb305_35;
  TNode<IntPtrT> phi_bb305_36;
  TNode<BoolT> phi_bb305_37;
  TNode<BoolT> phi_bb305_48;
  TNode<JSAny> phi_bb305_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb305_51;
  TNode<IntPtrT> phi_bb305_52;
  TNode<IntPtrT> tmp634;
  if (block305.is_used()) {
    ca_.Bind(&block305, &phi_bb305_20, &phi_bb305_26, &phi_bb305_27, &phi_bb305_28, &phi_bb305_29, &phi_bb305_32, &phi_bb305_33, &phi_bb305_35, &phi_bb305_36, &phi_bb305_37, &phi_bb305_48, &phi_bb305_49, &phi_bb305_51, &phi_bb305_52);
    tmp634 = TruncateBigIntToI64_0(state_, TNode<Context>{tmp441}, TNode<JSAny>{phi_bb305_49});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb305_51, phi_bb305_52}, tmp634);
    ca_.Goto(&block304, phi_bb305_20, tmp610, phi_bb305_26, phi_bb305_27, phi_bb305_28, phi_bb305_29, phi_bb305_32, phi_bb305_33, phi_bb305_35, phi_bb305_36, phi_bb305_37, phi_bb305_48, phi_bb305_49);
  }

  TNode<IntPtrT> phi_bb303_20;
  TNode<IntPtrT> phi_bb303_25;
  TNode<IntPtrT> phi_bb303_26;
  TNode<IntPtrT> phi_bb303_27;
  TNode<IntPtrT> phi_bb303_28;
  TNode<IntPtrT> phi_bb303_29;
  TNode<IntPtrT> phi_bb303_32;
  TNode<BoolT> phi_bb303_33;
  TNode<IntPtrT> phi_bb303_35;
  TNode<IntPtrT> phi_bb303_36;
  TNode<BoolT> phi_bb303_37;
  TNode<BoolT> phi_bb303_48;
  TNode<JSAny> phi_bb303_49;
  TNode<IntPtrT> tmp635;
  TNode<IntPtrT> tmp636;
  TNode<IntPtrT> tmp637;
  TNode<BoolT> tmp638;
  if (block303.is_used()) {
    ca_.Bind(&block303, &phi_bb303_20, &phi_bb303_25, &phi_bb303_26, &phi_bb303_27, &phi_bb303_28, &phi_bb303_29, &phi_bb303_32, &phi_bb303_33, &phi_bb303_35, &phi_bb303_36, &phi_bb303_37, &phi_bb303_48, &phi_bb303_49);
    tmp635 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp636 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb303_25}, TNode<IntPtrT>{tmp635});
    tmp637 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp638 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb303_25}, TNode<IntPtrT>{tmp637});
    ca_.Branch(tmp638, &block315, std::vector<compiler::Node*>{phi_bb303_20, phi_bb303_26, phi_bb303_27, phi_bb303_28, phi_bb303_29, phi_bb303_32, phi_bb303_33, phi_bb303_35, phi_bb303_36, phi_bb303_37, phi_bb303_48, phi_bb303_49}, &block316, std::vector<compiler::Node*>{phi_bb303_20, phi_bb303_26, phi_bb303_27, phi_bb303_28, phi_bb303_29, phi_bb303_32, phi_bb303_33, phi_bb303_35, phi_bb303_36, phi_bb303_37, phi_bb303_48, phi_bb303_49});
  }

  TNode<IntPtrT> phi_bb315_20;
  TNode<IntPtrT> phi_bb315_26;
  TNode<IntPtrT> phi_bb315_27;
  TNode<IntPtrT> phi_bb315_28;
  TNode<IntPtrT> phi_bb315_29;
  TNode<IntPtrT> phi_bb315_32;
  TNode<BoolT> phi_bb315_33;
  TNode<IntPtrT> phi_bb315_35;
  TNode<IntPtrT> phi_bb315_36;
  TNode<BoolT> phi_bb315_37;
  TNode<BoolT> phi_bb315_48;
  TNode<JSAny> phi_bb315_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp639;
  TNode<IntPtrT> tmp640;
  TNode<IntPtrT> tmp641;
  TNode<IntPtrT> tmp642;
  if (block315.is_used()) {
    ca_.Bind(&block315, &phi_bb315_20, &phi_bb315_26, &phi_bb315_27, &phi_bb315_28, &phi_bb315_29, &phi_bb315_32, &phi_bb315_33, &phi_bb315_35, &phi_bb315_36, &phi_bb315_37, &phi_bb315_48, &phi_bb315_49);
    std::tie(tmp639, tmp640) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb315_27}).Flatten();
    tmp641 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp642 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb315_27}, TNode<IntPtrT>{tmp641});
    ca_.Goto(&block314, phi_bb315_20, phi_bb315_26, tmp642, phi_bb315_28, phi_bb315_29, phi_bb315_32, phi_bb315_33, phi_bb315_35, phi_bb315_36, phi_bb315_37, phi_bb315_48, phi_bb315_49, tmp639, tmp640);
  }

  TNode<IntPtrT> phi_bb316_20;
  TNode<IntPtrT> phi_bb316_26;
  TNode<IntPtrT> phi_bb316_27;
  TNode<IntPtrT> phi_bb316_28;
  TNode<IntPtrT> phi_bb316_29;
  TNode<IntPtrT> phi_bb316_32;
  TNode<BoolT> phi_bb316_33;
  TNode<IntPtrT> phi_bb316_35;
  TNode<IntPtrT> phi_bb316_36;
  TNode<BoolT> phi_bb316_37;
  TNode<BoolT> phi_bb316_48;
  TNode<JSAny> phi_bb316_49;
  if (block316.is_used()) {
    ca_.Bind(&block316, &phi_bb316_20, &phi_bb316_26, &phi_bb316_27, &phi_bb316_28, &phi_bb316_29, &phi_bb316_32, &phi_bb316_33, &phi_bb316_35, &phi_bb316_36, &phi_bb316_37, &phi_bb316_48, &phi_bb316_49);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block318, phi_bb316_20, phi_bb316_26, phi_bb316_27, phi_bb316_28, phi_bb316_29, phi_bb316_32, phi_bb316_33, phi_bb316_35, phi_bb316_36, phi_bb316_37, phi_bb316_48, phi_bb316_49);
    } else {
      ca_.Goto(&block319, phi_bb316_20, phi_bb316_26, phi_bb316_27, phi_bb316_28, phi_bb316_29, phi_bb316_32, phi_bb316_33, phi_bb316_35, phi_bb316_36, phi_bb316_37, phi_bb316_48, phi_bb316_49);
    }
  }

  TNode<IntPtrT> phi_bb318_20;
  TNode<IntPtrT> phi_bb318_26;
  TNode<IntPtrT> phi_bb318_27;
  TNode<IntPtrT> phi_bb318_28;
  TNode<IntPtrT> phi_bb318_29;
  TNode<IntPtrT> phi_bb318_32;
  TNode<BoolT> phi_bb318_33;
  TNode<IntPtrT> phi_bb318_35;
  TNode<IntPtrT> phi_bb318_36;
  TNode<BoolT> phi_bb318_37;
  TNode<BoolT> phi_bb318_48;
  TNode<JSAny> phi_bb318_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp643;
  TNode<IntPtrT> tmp644;
  TNode<IntPtrT> tmp645;
  TNode<IntPtrT> tmp646;
  if (block318.is_used()) {
    ca_.Bind(&block318, &phi_bb318_20, &phi_bb318_26, &phi_bb318_27, &phi_bb318_28, &phi_bb318_29, &phi_bb318_32, &phi_bb318_33, &phi_bb318_35, &phi_bb318_36, &phi_bb318_37, &phi_bb318_48, &phi_bb318_49);
    std::tie(tmp643, tmp644) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb318_29}).Flatten();
    tmp645 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp646 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb318_29}, TNode<IntPtrT>{tmp645});
    ca_.Goto(&block317, phi_bb318_20, phi_bb318_26, phi_bb318_27, phi_bb318_28, tmp646, phi_bb318_32, phi_bb318_33, phi_bb318_35, phi_bb318_36, phi_bb318_37, phi_bb318_48, phi_bb318_49, tmp643, tmp644);
  }

  TNode<IntPtrT> phi_bb319_20;
  TNode<IntPtrT> phi_bb319_26;
  TNode<IntPtrT> phi_bb319_27;
  TNode<IntPtrT> phi_bb319_28;
  TNode<IntPtrT> phi_bb319_29;
  TNode<IntPtrT> phi_bb319_32;
  TNode<BoolT> phi_bb319_33;
  TNode<IntPtrT> phi_bb319_35;
  TNode<IntPtrT> phi_bb319_36;
  TNode<BoolT> phi_bb319_37;
  TNode<BoolT> phi_bb319_48;
  TNode<JSAny> phi_bb319_49;
  TNode<IntPtrT> tmp647;
  TNode<BoolT> tmp648;
  if (block319.is_used()) {
    ca_.Bind(&block319, &phi_bb319_20, &phi_bb319_26, &phi_bb319_27, &phi_bb319_28, &phi_bb319_29, &phi_bb319_32, &phi_bb319_33, &phi_bb319_35, &phi_bb319_36, &phi_bb319_37, &phi_bb319_48, &phi_bb319_49);
    tmp647 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp648 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb319_32}, TNode<IntPtrT>{tmp647});
    ca_.Branch(tmp648, &block321, std::vector<compiler::Node*>{phi_bb319_20, phi_bb319_26, phi_bb319_27, phi_bb319_28, phi_bb319_29, phi_bb319_32, phi_bb319_33, phi_bb319_35, phi_bb319_36, phi_bb319_37, phi_bb319_48, phi_bb319_49}, &block322, std::vector<compiler::Node*>{phi_bb319_20, phi_bb319_26, phi_bb319_27, phi_bb319_28, phi_bb319_29, phi_bb319_32, phi_bb319_33, phi_bb319_35, phi_bb319_36, phi_bb319_37, phi_bb319_48, phi_bb319_49});
  }

  TNode<IntPtrT> phi_bb321_20;
  TNode<IntPtrT> phi_bb321_26;
  TNode<IntPtrT> phi_bb321_27;
  TNode<IntPtrT> phi_bb321_28;
  TNode<IntPtrT> phi_bb321_29;
  TNode<IntPtrT> phi_bb321_32;
  TNode<BoolT> phi_bb321_33;
  TNode<IntPtrT> phi_bb321_35;
  TNode<IntPtrT> phi_bb321_36;
  TNode<BoolT> phi_bb321_37;
  TNode<BoolT> phi_bb321_48;
  TNode<JSAny> phi_bb321_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp649;
  TNode<IntPtrT> tmp650;
  TNode<IntPtrT> tmp651;
  TNode<BoolT> tmp652;
  if (block321.is_used()) {
    ca_.Bind(&block321, &phi_bb321_20, &phi_bb321_26, &phi_bb321_27, &phi_bb321_28, &phi_bb321_29, &phi_bb321_32, &phi_bb321_33, &phi_bb321_35, &phi_bb321_36, &phi_bb321_37, &phi_bb321_48, &phi_bb321_49);
    std::tie(tmp649, tmp650) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb321_32}).Flatten();
    tmp651 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp652 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block317, phi_bb321_20, phi_bb321_26, phi_bb321_27, phi_bb321_28, phi_bb321_29, tmp651, tmp652, phi_bb321_35, phi_bb321_36, phi_bb321_37, phi_bb321_48, phi_bb321_49, tmp649, tmp650);
  }

  TNode<IntPtrT> phi_bb322_20;
  TNode<IntPtrT> phi_bb322_26;
  TNode<IntPtrT> phi_bb322_27;
  TNode<IntPtrT> phi_bb322_28;
  TNode<IntPtrT> phi_bb322_29;
  TNode<IntPtrT> phi_bb322_32;
  TNode<BoolT> phi_bb322_33;
  TNode<IntPtrT> phi_bb322_35;
  TNode<IntPtrT> phi_bb322_36;
  TNode<BoolT> phi_bb322_37;
  TNode<BoolT> phi_bb322_48;
  TNode<JSAny> phi_bb322_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp653;
  TNode<IntPtrT> tmp654;
  TNode<IntPtrT> tmp655;
  TNode<IntPtrT> tmp656;
  TNode<IntPtrT> tmp657;
  TNode<IntPtrT> tmp658;
  TNode<BoolT> tmp659;
  if (block322.is_used()) {
    ca_.Bind(&block322, &phi_bb322_20, &phi_bb322_26, &phi_bb322_27, &phi_bb322_28, &phi_bb322_29, &phi_bb322_32, &phi_bb322_33, &phi_bb322_35, &phi_bb322_36, &phi_bb322_37, &phi_bb322_48, &phi_bb322_49);
    std::tie(tmp653, tmp654) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb322_29}).Flatten();
    tmp655 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp656 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb322_29}, TNode<IntPtrT>{tmp655});
    tmp657 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp658 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp656}, TNode<IntPtrT>{tmp657});
    tmp659 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block317, phi_bb322_20, phi_bb322_26, phi_bb322_27, phi_bb322_28, tmp658, tmp656, tmp659, phi_bb322_35, phi_bb322_36, phi_bb322_37, phi_bb322_48, phi_bb322_49, tmp653, tmp654);
  }

  TNode<IntPtrT> phi_bb317_20;
  TNode<IntPtrT> phi_bb317_26;
  TNode<IntPtrT> phi_bb317_27;
  TNode<IntPtrT> phi_bb317_28;
  TNode<IntPtrT> phi_bb317_29;
  TNode<IntPtrT> phi_bb317_32;
  TNode<BoolT> phi_bb317_33;
  TNode<IntPtrT> phi_bb317_35;
  TNode<IntPtrT> phi_bb317_36;
  TNode<BoolT> phi_bb317_37;
  TNode<BoolT> phi_bb317_48;
  TNode<JSAny> phi_bb317_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb317_51;
  TNode<IntPtrT> phi_bb317_52;
  if (block317.is_used()) {
    ca_.Bind(&block317, &phi_bb317_20, &phi_bb317_26, &phi_bb317_27, &phi_bb317_28, &phi_bb317_29, &phi_bb317_32, &phi_bb317_33, &phi_bb317_35, &phi_bb317_36, &phi_bb317_37, &phi_bb317_48, &phi_bb317_49, &phi_bb317_51, &phi_bb317_52);
    ca_.Goto(&block314, phi_bb317_20, phi_bb317_26, phi_bb317_27, phi_bb317_28, phi_bb317_29, phi_bb317_32, phi_bb317_33, phi_bb317_35, phi_bb317_36, phi_bb317_37, phi_bb317_48, phi_bb317_49, phi_bb317_51, phi_bb317_52);
  }

  TNode<IntPtrT> phi_bb314_20;
  TNode<IntPtrT> phi_bb314_26;
  TNode<IntPtrT> phi_bb314_27;
  TNode<IntPtrT> phi_bb314_28;
  TNode<IntPtrT> phi_bb314_29;
  TNode<IntPtrT> phi_bb314_32;
  TNode<BoolT> phi_bb314_33;
  TNode<IntPtrT> phi_bb314_35;
  TNode<IntPtrT> phi_bb314_36;
  TNode<BoolT> phi_bb314_37;
  TNode<BoolT> phi_bb314_48;
  TNode<JSAny> phi_bb314_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb314_51;
  TNode<IntPtrT> phi_bb314_52;
  TNode<IntPtrT> tmp660;
  TNode<IntPtrT> tmp661;
  TNode<IntPtrT> tmp662;
  TNode<BoolT> tmp663;
  if (block314.is_used()) {
    ca_.Bind(&block314, &phi_bb314_20, &phi_bb314_26, &phi_bb314_27, &phi_bb314_28, &phi_bb314_29, &phi_bb314_32, &phi_bb314_33, &phi_bb314_35, &phi_bb314_36, &phi_bb314_37, &phi_bb314_48, &phi_bb314_49, &phi_bb314_51, &phi_bb314_52);
    tmp660 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp661 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp636}, TNode<IntPtrT>{tmp660});
    tmp662 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp663 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{tmp636}, TNode<IntPtrT>{tmp662});
    ca_.Branch(tmp663, &block324, std::vector<compiler::Node*>{phi_bb314_20, phi_bb314_26, phi_bb314_27, phi_bb314_28, phi_bb314_29, phi_bb314_32, phi_bb314_33, phi_bb314_35, phi_bb314_36, phi_bb314_37, phi_bb314_48, phi_bb314_49, phi_bb314_51, phi_bb314_52}, &block325, std::vector<compiler::Node*>{phi_bb314_20, phi_bb314_26, phi_bb314_27, phi_bb314_28, phi_bb314_29, phi_bb314_32, phi_bb314_33, phi_bb314_35, phi_bb314_36, phi_bb314_37, phi_bb314_48, phi_bb314_49, phi_bb314_51, phi_bb314_52});
  }

  TNode<IntPtrT> phi_bb324_20;
  TNode<IntPtrT> phi_bb324_26;
  TNode<IntPtrT> phi_bb324_27;
  TNode<IntPtrT> phi_bb324_28;
  TNode<IntPtrT> phi_bb324_29;
  TNode<IntPtrT> phi_bb324_32;
  TNode<BoolT> phi_bb324_33;
  TNode<IntPtrT> phi_bb324_35;
  TNode<IntPtrT> phi_bb324_36;
  TNode<BoolT> phi_bb324_37;
  TNode<BoolT> phi_bb324_48;
  TNode<JSAny> phi_bb324_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb324_51;
  TNode<IntPtrT> phi_bb324_52;
  TNode<Union<HeapObject, TaggedIndex>> tmp664;
  TNode<IntPtrT> tmp665;
  TNode<IntPtrT> tmp666;
  TNode<IntPtrT> tmp667;
  if (block324.is_used()) {
    ca_.Bind(&block324, &phi_bb324_20, &phi_bb324_26, &phi_bb324_27, &phi_bb324_28, &phi_bb324_29, &phi_bb324_32, &phi_bb324_33, &phi_bb324_35, &phi_bb324_36, &phi_bb324_37, &phi_bb324_48, &phi_bb324_49, &phi_bb324_51, &phi_bb324_52);
    std::tie(tmp664, tmp665) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb324_27}).Flatten();
    tmp666 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp667 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb324_27}, TNode<IntPtrT>{tmp666});
    ca_.Goto(&block323, phi_bb324_20, phi_bb324_26, tmp667, phi_bb324_28, phi_bb324_29, phi_bb324_32, phi_bb324_33, phi_bb324_35, phi_bb324_36, phi_bb324_37, phi_bb324_48, phi_bb324_49, phi_bb324_51, phi_bb324_52, tmp664, tmp665);
  }

  TNode<IntPtrT> phi_bb325_20;
  TNode<IntPtrT> phi_bb325_26;
  TNode<IntPtrT> phi_bb325_27;
  TNode<IntPtrT> phi_bb325_28;
  TNode<IntPtrT> phi_bb325_29;
  TNode<IntPtrT> phi_bb325_32;
  TNode<BoolT> phi_bb325_33;
  TNode<IntPtrT> phi_bb325_35;
  TNode<IntPtrT> phi_bb325_36;
  TNode<BoolT> phi_bb325_37;
  TNode<BoolT> phi_bb325_48;
  TNode<JSAny> phi_bb325_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb325_51;
  TNode<IntPtrT> phi_bb325_52;
  if (block325.is_used()) {
    ca_.Bind(&block325, &phi_bb325_20, &phi_bb325_26, &phi_bb325_27, &phi_bb325_28, &phi_bb325_29, &phi_bb325_32, &phi_bb325_33, &phi_bb325_35, &phi_bb325_36, &phi_bb325_37, &phi_bb325_48, &phi_bb325_49, &phi_bb325_51, &phi_bb325_52);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block327, phi_bb325_20, phi_bb325_26, phi_bb325_27, phi_bb325_28, phi_bb325_29, phi_bb325_32, phi_bb325_33, phi_bb325_35, phi_bb325_36, phi_bb325_37, phi_bb325_48, phi_bb325_49, phi_bb325_51, phi_bb325_52);
    } else {
      ca_.Goto(&block328, phi_bb325_20, phi_bb325_26, phi_bb325_27, phi_bb325_28, phi_bb325_29, phi_bb325_32, phi_bb325_33, phi_bb325_35, phi_bb325_36, phi_bb325_37, phi_bb325_48, phi_bb325_49, phi_bb325_51, phi_bb325_52);
    }
  }

  TNode<IntPtrT> phi_bb327_20;
  TNode<IntPtrT> phi_bb327_26;
  TNode<IntPtrT> phi_bb327_27;
  TNode<IntPtrT> phi_bb327_28;
  TNode<IntPtrT> phi_bb327_29;
  TNode<IntPtrT> phi_bb327_32;
  TNode<BoolT> phi_bb327_33;
  TNode<IntPtrT> phi_bb327_35;
  TNode<IntPtrT> phi_bb327_36;
  TNode<BoolT> phi_bb327_37;
  TNode<BoolT> phi_bb327_48;
  TNode<JSAny> phi_bb327_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb327_51;
  TNode<IntPtrT> phi_bb327_52;
  TNode<Union<HeapObject, TaggedIndex>> tmp668;
  TNode<IntPtrT> tmp669;
  TNode<IntPtrT> tmp670;
  TNode<IntPtrT> tmp671;
  if (block327.is_used()) {
    ca_.Bind(&block327, &phi_bb327_20, &phi_bb327_26, &phi_bb327_27, &phi_bb327_28, &phi_bb327_29, &phi_bb327_32, &phi_bb327_33, &phi_bb327_35, &phi_bb327_36, &phi_bb327_37, &phi_bb327_48, &phi_bb327_49, &phi_bb327_51, &phi_bb327_52);
    std::tie(tmp668, tmp669) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb327_29}).Flatten();
    tmp670 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp671 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb327_29}, TNode<IntPtrT>{tmp670});
    ca_.Goto(&block326, phi_bb327_20, phi_bb327_26, phi_bb327_27, phi_bb327_28, tmp671, phi_bb327_32, phi_bb327_33, phi_bb327_35, phi_bb327_36, phi_bb327_37, phi_bb327_48, phi_bb327_49, phi_bb327_51, phi_bb327_52, tmp668, tmp669);
  }

  TNode<IntPtrT> phi_bb328_20;
  TNode<IntPtrT> phi_bb328_26;
  TNode<IntPtrT> phi_bb328_27;
  TNode<IntPtrT> phi_bb328_28;
  TNode<IntPtrT> phi_bb328_29;
  TNode<IntPtrT> phi_bb328_32;
  TNode<BoolT> phi_bb328_33;
  TNode<IntPtrT> phi_bb328_35;
  TNode<IntPtrT> phi_bb328_36;
  TNode<BoolT> phi_bb328_37;
  TNode<BoolT> phi_bb328_48;
  TNode<JSAny> phi_bb328_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb328_51;
  TNode<IntPtrT> phi_bb328_52;
  TNode<IntPtrT> tmp672;
  TNode<BoolT> tmp673;
  if (block328.is_used()) {
    ca_.Bind(&block328, &phi_bb328_20, &phi_bb328_26, &phi_bb328_27, &phi_bb328_28, &phi_bb328_29, &phi_bb328_32, &phi_bb328_33, &phi_bb328_35, &phi_bb328_36, &phi_bb328_37, &phi_bb328_48, &phi_bb328_49, &phi_bb328_51, &phi_bb328_52);
    tmp672 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp673 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb328_32}, TNode<IntPtrT>{tmp672});
    ca_.Branch(tmp673, &block330, std::vector<compiler::Node*>{phi_bb328_20, phi_bb328_26, phi_bb328_27, phi_bb328_28, phi_bb328_29, phi_bb328_32, phi_bb328_33, phi_bb328_35, phi_bb328_36, phi_bb328_37, phi_bb328_48, phi_bb328_49, phi_bb328_51, phi_bb328_52}, &block331, std::vector<compiler::Node*>{phi_bb328_20, phi_bb328_26, phi_bb328_27, phi_bb328_28, phi_bb328_29, phi_bb328_32, phi_bb328_33, phi_bb328_35, phi_bb328_36, phi_bb328_37, phi_bb328_48, phi_bb328_49, phi_bb328_51, phi_bb328_52});
  }

  TNode<IntPtrT> phi_bb330_20;
  TNode<IntPtrT> phi_bb330_26;
  TNode<IntPtrT> phi_bb330_27;
  TNode<IntPtrT> phi_bb330_28;
  TNode<IntPtrT> phi_bb330_29;
  TNode<IntPtrT> phi_bb330_32;
  TNode<BoolT> phi_bb330_33;
  TNode<IntPtrT> phi_bb330_35;
  TNode<IntPtrT> phi_bb330_36;
  TNode<BoolT> phi_bb330_37;
  TNode<BoolT> phi_bb330_48;
  TNode<JSAny> phi_bb330_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb330_51;
  TNode<IntPtrT> phi_bb330_52;
  TNode<Union<HeapObject, TaggedIndex>> tmp674;
  TNode<IntPtrT> tmp675;
  TNode<IntPtrT> tmp676;
  TNode<BoolT> tmp677;
  if (block330.is_used()) {
    ca_.Bind(&block330, &phi_bb330_20, &phi_bb330_26, &phi_bb330_27, &phi_bb330_28, &phi_bb330_29, &phi_bb330_32, &phi_bb330_33, &phi_bb330_35, &phi_bb330_36, &phi_bb330_37, &phi_bb330_48, &phi_bb330_49, &phi_bb330_51, &phi_bb330_52);
    std::tie(tmp674, tmp675) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb330_32}).Flatten();
    tmp676 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp677 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block326, phi_bb330_20, phi_bb330_26, phi_bb330_27, phi_bb330_28, phi_bb330_29, tmp676, tmp677, phi_bb330_35, phi_bb330_36, phi_bb330_37, phi_bb330_48, phi_bb330_49, phi_bb330_51, phi_bb330_52, tmp674, tmp675);
  }

  TNode<IntPtrT> phi_bb331_20;
  TNode<IntPtrT> phi_bb331_26;
  TNode<IntPtrT> phi_bb331_27;
  TNode<IntPtrT> phi_bb331_28;
  TNode<IntPtrT> phi_bb331_29;
  TNode<IntPtrT> phi_bb331_32;
  TNode<BoolT> phi_bb331_33;
  TNode<IntPtrT> phi_bb331_35;
  TNode<IntPtrT> phi_bb331_36;
  TNode<BoolT> phi_bb331_37;
  TNode<BoolT> phi_bb331_48;
  TNode<JSAny> phi_bb331_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb331_51;
  TNode<IntPtrT> phi_bb331_52;
  TNode<Union<HeapObject, TaggedIndex>> tmp678;
  TNode<IntPtrT> tmp679;
  TNode<IntPtrT> tmp680;
  TNode<IntPtrT> tmp681;
  TNode<IntPtrT> tmp682;
  TNode<IntPtrT> tmp683;
  TNode<BoolT> tmp684;
  if (block331.is_used()) {
    ca_.Bind(&block331, &phi_bb331_20, &phi_bb331_26, &phi_bb331_27, &phi_bb331_28, &phi_bb331_29, &phi_bb331_32, &phi_bb331_33, &phi_bb331_35, &phi_bb331_36, &phi_bb331_37, &phi_bb331_48, &phi_bb331_49, &phi_bb331_51, &phi_bb331_52);
    std::tie(tmp678, tmp679) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb331_29}).Flatten();
    tmp680 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp681 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb331_29}, TNode<IntPtrT>{tmp680});
    tmp682 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp683 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp681}, TNode<IntPtrT>{tmp682});
    tmp684 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block326, phi_bb331_20, phi_bb331_26, phi_bb331_27, phi_bb331_28, tmp683, tmp681, tmp684, phi_bb331_35, phi_bb331_36, phi_bb331_37, phi_bb331_48, phi_bb331_49, phi_bb331_51, phi_bb331_52, tmp678, tmp679);
  }

  TNode<IntPtrT> phi_bb326_20;
  TNode<IntPtrT> phi_bb326_26;
  TNode<IntPtrT> phi_bb326_27;
  TNode<IntPtrT> phi_bb326_28;
  TNode<IntPtrT> phi_bb326_29;
  TNode<IntPtrT> phi_bb326_32;
  TNode<BoolT> phi_bb326_33;
  TNode<IntPtrT> phi_bb326_35;
  TNode<IntPtrT> phi_bb326_36;
  TNode<BoolT> phi_bb326_37;
  TNode<BoolT> phi_bb326_48;
  TNode<JSAny> phi_bb326_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb326_51;
  TNode<IntPtrT> phi_bb326_52;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb326_53;
  TNode<IntPtrT> phi_bb326_54;
  if (block326.is_used()) {
    ca_.Bind(&block326, &phi_bb326_20, &phi_bb326_26, &phi_bb326_27, &phi_bb326_28, &phi_bb326_29, &phi_bb326_32, &phi_bb326_33, &phi_bb326_35, &phi_bb326_36, &phi_bb326_37, &phi_bb326_48, &phi_bb326_49, &phi_bb326_51, &phi_bb326_52, &phi_bb326_53, &phi_bb326_54);
    ca_.Goto(&block323, phi_bb326_20, phi_bb326_26, phi_bb326_27, phi_bb326_28, phi_bb326_29, phi_bb326_32, phi_bb326_33, phi_bb326_35, phi_bb326_36, phi_bb326_37, phi_bb326_48, phi_bb326_49, phi_bb326_51, phi_bb326_52, phi_bb326_53, phi_bb326_54);
  }

  TNode<IntPtrT> phi_bb323_20;
  TNode<IntPtrT> phi_bb323_26;
  TNode<IntPtrT> phi_bb323_27;
  TNode<IntPtrT> phi_bb323_28;
  TNode<IntPtrT> phi_bb323_29;
  TNode<IntPtrT> phi_bb323_32;
  TNode<BoolT> phi_bb323_33;
  TNode<IntPtrT> phi_bb323_35;
  TNode<IntPtrT> phi_bb323_36;
  TNode<BoolT> phi_bb323_37;
  TNode<BoolT> phi_bb323_48;
  TNode<JSAny> phi_bb323_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb323_51;
  TNode<IntPtrT> phi_bb323_52;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb323_53;
  TNode<IntPtrT> phi_bb323_54;
  TNode<BigInt> tmp685;
  TNode<UintPtrT> tmp686;
  TNode<UintPtrT> tmp687;
  TNode<IntPtrT> tmp688;
  TNode<IntPtrT> tmp689;
  if (block323.is_used()) {
    ca_.Bind(&block323, &phi_bb323_20, &phi_bb323_26, &phi_bb323_27, &phi_bb323_28, &phi_bb323_29, &phi_bb323_32, &phi_bb323_33, &phi_bb323_35, &phi_bb323_36, &phi_bb323_37, &phi_bb323_48, &phi_bb323_49, &phi_bb323_51, &phi_bb323_52, &phi_bb323_53, &phi_bb323_54);
    tmp685 = CodeStubAssembler(state_).ToBigInt(TNode<Context>{tmp441}, TNode<JSAny>{phi_bb323_49});
    std::tie(tmp686, tmp687) = CodeStubAssembler(state_).BigIntToRawBytes(TNode<BigInt>{tmp685}).Flatten();
    tmp688 = CodeStubAssembler(state_).Signed(TNode<UintPtrT>{tmp686});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb323_51, phi_bb323_52}, tmp688);
    tmp689 = CodeStubAssembler(state_).Signed(TNode<UintPtrT>{tmp687});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb323_53, phi_bb323_54}, tmp689);
    ca_.Goto(&block304, phi_bb323_20, tmp661, phi_bb323_26, phi_bb323_27, phi_bb323_28, phi_bb323_29, phi_bb323_32, phi_bb323_33, phi_bb323_35, phi_bb323_36, phi_bb323_37, phi_bb323_48, phi_bb323_49);
  }

  TNode<IntPtrT> phi_bb304_20;
  TNode<IntPtrT> phi_bb304_25;
  TNode<IntPtrT> phi_bb304_26;
  TNode<IntPtrT> phi_bb304_27;
  TNode<IntPtrT> phi_bb304_28;
  TNode<IntPtrT> phi_bb304_29;
  TNode<IntPtrT> phi_bb304_32;
  TNode<BoolT> phi_bb304_33;
  TNode<IntPtrT> phi_bb304_35;
  TNode<IntPtrT> phi_bb304_36;
  TNode<BoolT> phi_bb304_37;
  TNode<BoolT> phi_bb304_48;
  TNode<JSAny> phi_bb304_49;
  if (block304.is_used()) {
    ca_.Bind(&block304, &phi_bb304_20, &phi_bb304_25, &phi_bb304_26, &phi_bb304_27, &phi_bb304_28, &phi_bb304_29, &phi_bb304_32, &phi_bb304_33, &phi_bb304_35, &phi_bb304_36, &phi_bb304_37, &phi_bb304_48, &phi_bb304_49);
    ca_.Goto(&block301, phi_bb304_20, phi_bb304_25, phi_bb304_26, phi_bb304_27, phi_bb304_28, phi_bb304_29, phi_bb304_32, phi_bb304_33, phi_bb304_35, phi_bb304_36, phi_bb304_37, phi_bb304_48, phi_bb304_49);
  }

  TNode<IntPtrT> phi_bb300_20;
  TNode<IntPtrT> phi_bb300_25;
  TNode<IntPtrT> phi_bb300_26;
  TNode<IntPtrT> phi_bb300_27;
  TNode<IntPtrT> phi_bb300_28;
  TNode<IntPtrT> phi_bb300_29;
  TNode<IntPtrT> phi_bb300_32;
  TNode<BoolT> phi_bb300_33;
  TNode<IntPtrT> phi_bb300_35;
  TNode<IntPtrT> phi_bb300_36;
  TNode<BoolT> phi_bb300_37;
  TNode<BoolT> phi_bb300_48;
  TNode<JSAny> phi_bb300_49;
  TNode<Uint32T> tmp690;
  TNode<Uint32T> tmp691;
  TNode<Uint32T> tmp692;
  TNode<BoolT> tmp693;
  if (block300.is_used()) {
    ca_.Bind(&block300, &phi_bb300_20, &phi_bb300_25, &phi_bb300_26, &phi_bb300_27, &phi_bb300_28, &phi_bb300_29, &phi_bb300_32, &phi_bb300_33, &phi_bb300_35, &phi_bb300_36, &phi_bb300_37, &phi_bb300_48, &phi_bb300_49);
    tmp690 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp691 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp507}, TNode<Uint32T>{tmp690});
    tmp692 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp693 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp691}, TNode<Uint32T>{tmp692});
    ca_.Branch(tmp693, &block332, std::vector<compiler::Node*>{phi_bb300_20, phi_bb300_25, phi_bb300_26, phi_bb300_27, phi_bb300_28, phi_bb300_29, phi_bb300_32, phi_bb300_33, phi_bb300_35, phi_bb300_36, phi_bb300_37, phi_bb300_48, phi_bb300_49}, &block333, std::vector<compiler::Node*>{phi_bb300_20, phi_bb300_25, phi_bb300_26, phi_bb300_27, phi_bb300_28, phi_bb300_29, phi_bb300_32, phi_bb300_33, phi_bb300_35, phi_bb300_36, phi_bb300_37, phi_bb300_48, phi_bb300_49});
  }

  TNode<IntPtrT> phi_bb333_20;
  TNode<IntPtrT> phi_bb333_25;
  TNode<IntPtrT> phi_bb333_26;
  TNode<IntPtrT> phi_bb333_27;
  TNode<IntPtrT> phi_bb333_28;
  TNode<IntPtrT> phi_bb333_29;
  TNode<IntPtrT> phi_bb333_32;
  TNode<BoolT> phi_bb333_33;
  TNode<IntPtrT> phi_bb333_35;
  TNode<IntPtrT> phi_bb333_36;
  TNode<BoolT> phi_bb333_37;
  TNode<BoolT> phi_bb333_48;
  TNode<JSAny> phi_bb333_49;
  if (block333.is_used()) {
    ca_.Bind(&block333, &phi_bb333_20, &phi_bb333_25, &phi_bb333_26, &phi_bb333_27, &phi_bb333_28, &phi_bb333_29, &phi_bb333_32, &phi_bb333_33, &phi_bb333_35, &phi_bb333_36, &phi_bb333_37, &phi_bb333_48, &phi_bb333_49);
    {
      auto pos_stack = ca_.GetMacroSourcePositionStack();
      pos_stack.push_back({"src/builtins/wasm-to-js.tq", 261});
      CodeStubAssembler(state_).FailAssert("Torque assert '(retType & kValueTypeIsRefBit) != 0' failed", pos_stack);
    }
  }

  TNode<IntPtrT> phi_bb332_20;
  TNode<IntPtrT> phi_bb332_25;
  TNode<IntPtrT> phi_bb332_26;
  TNode<IntPtrT> phi_bb332_27;
  TNode<IntPtrT> phi_bb332_28;
  TNode<IntPtrT> phi_bb332_29;
  TNode<IntPtrT> phi_bb332_32;
  TNode<BoolT> phi_bb332_33;
  TNode<IntPtrT> phi_bb332_35;
  TNode<IntPtrT> phi_bb332_36;
  TNode<BoolT> phi_bb332_37;
  TNode<BoolT> phi_bb332_48;
  TNode<JSAny> phi_bb332_49;
  TNode<Object> tmp694;
  TNode<IntPtrT> tmp695;
  TNode<BoolT> tmp696;
  if (block332.is_used()) {
    ca_.Bind(&block332, &phi_bb332_20, &phi_bb332_25, &phi_bb332_26, &phi_bb332_27, &phi_bb332_28, &phi_bb332_29, &phi_bb332_32, &phi_bb332_33, &phi_bb332_35, &phi_bb332_36, &phi_bb332_37, &phi_bb332_48, &phi_bb332_49);
    tmp694 = JSToWasmObject_0(state_, TNode<NativeContext>{tmp441}, TNode<Uint32T>{tmp507}, TNode<JSAny>{phi_bb332_49});
    tmp695 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x1ull));
    tmp696 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{tmp31}, TNode<IntPtrT>{tmp695});
    ca_.Branch(tmp696, &block334, std::vector<compiler::Node*>{phi_bb332_20, phi_bb332_25, phi_bb332_26, phi_bb332_27, phi_bb332_28, phi_bb332_29, phi_bb332_32, phi_bb332_33, phi_bb332_35, phi_bb332_36, phi_bb332_37, phi_bb332_48, phi_bb332_49}, &block335, std::vector<compiler::Node*>{phi_bb332_20, phi_bb332_25, phi_bb332_26, phi_bb332_27, phi_bb332_28, phi_bb332_29, phi_bb332_32, phi_bb332_33, phi_bb332_35, phi_bb332_36, phi_bb332_37, phi_bb332_48, phi_bb332_49});
  }

  TNode<IntPtrT> phi_bb334_20;
  TNode<IntPtrT> phi_bb334_25;
  TNode<IntPtrT> phi_bb334_26;
  TNode<IntPtrT> phi_bb334_27;
  TNode<IntPtrT> phi_bb334_28;
  TNode<IntPtrT> phi_bb334_29;
  TNode<IntPtrT> phi_bb334_32;
  TNode<BoolT> phi_bb334_33;
  TNode<IntPtrT> phi_bb334_35;
  TNode<IntPtrT> phi_bb334_36;
  TNode<BoolT> phi_bb334_37;
  TNode<BoolT> phi_bb334_48;
  TNode<JSAny> phi_bb334_49;
  TNode<IntPtrT> tmp697;
  TNode<IntPtrT> tmp698;
  TNode<IntPtrT> tmp699;
  TNode<BoolT> tmp700;
  if (block334.is_used()) {
    ca_.Bind(&block334, &phi_bb334_20, &phi_bb334_25, &phi_bb334_26, &phi_bb334_27, &phi_bb334_28, &phi_bb334_29, &phi_bb334_32, &phi_bb334_33, &phi_bb334_35, &phi_bb334_36, &phi_bb334_37, &phi_bb334_48, &phi_bb334_49);
    tmp697 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp698 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb334_25}, TNode<IntPtrT>{tmp697});
    tmp699 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp700 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb334_25}, TNode<IntPtrT>{tmp699});
    ca_.Branch(tmp700, &block338, std::vector<compiler::Node*>{phi_bb334_20, phi_bb334_26, phi_bb334_27, phi_bb334_28, phi_bb334_29, phi_bb334_32, phi_bb334_33, phi_bb334_35, phi_bb334_36, phi_bb334_37, phi_bb334_48, phi_bb334_49}, &block339, std::vector<compiler::Node*>{phi_bb334_20, phi_bb334_26, phi_bb334_27, phi_bb334_28, phi_bb334_29, phi_bb334_32, phi_bb334_33, phi_bb334_35, phi_bb334_36, phi_bb334_37, phi_bb334_48, phi_bb334_49});
  }

  TNode<IntPtrT> phi_bb338_20;
  TNode<IntPtrT> phi_bb338_26;
  TNode<IntPtrT> phi_bb338_27;
  TNode<IntPtrT> phi_bb338_28;
  TNode<IntPtrT> phi_bb338_29;
  TNode<IntPtrT> phi_bb338_32;
  TNode<BoolT> phi_bb338_33;
  TNode<IntPtrT> phi_bb338_35;
  TNode<IntPtrT> phi_bb338_36;
  TNode<BoolT> phi_bb338_37;
  TNode<BoolT> phi_bb338_48;
  TNode<JSAny> phi_bb338_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp701;
  TNode<IntPtrT> tmp702;
  TNode<IntPtrT> tmp703;
  TNode<IntPtrT> tmp704;
  if (block338.is_used()) {
    ca_.Bind(&block338, &phi_bb338_20, &phi_bb338_26, &phi_bb338_27, &phi_bb338_28, &phi_bb338_29, &phi_bb338_32, &phi_bb338_33, &phi_bb338_35, &phi_bb338_36, &phi_bb338_37, &phi_bb338_48, &phi_bb338_49);
    std::tie(tmp701, tmp702) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb338_27}).Flatten();
    tmp703 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp704 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb338_27}, TNode<IntPtrT>{tmp703});
    ca_.Goto(&block337, phi_bb338_20, phi_bb338_26, tmp704, phi_bb338_28, phi_bb338_29, phi_bb338_32, phi_bb338_33, phi_bb338_35, phi_bb338_36, phi_bb338_37, phi_bb338_48, phi_bb338_49, tmp701, tmp702);
  }

  TNode<IntPtrT> phi_bb339_20;
  TNode<IntPtrT> phi_bb339_26;
  TNode<IntPtrT> phi_bb339_27;
  TNode<IntPtrT> phi_bb339_28;
  TNode<IntPtrT> phi_bb339_29;
  TNode<IntPtrT> phi_bb339_32;
  TNode<BoolT> phi_bb339_33;
  TNode<IntPtrT> phi_bb339_35;
  TNode<IntPtrT> phi_bb339_36;
  TNode<BoolT> phi_bb339_37;
  TNode<BoolT> phi_bb339_48;
  TNode<JSAny> phi_bb339_49;
  if (block339.is_used()) {
    ca_.Bind(&block339, &phi_bb339_20, &phi_bb339_26, &phi_bb339_27, &phi_bb339_28, &phi_bb339_29, &phi_bb339_32, &phi_bb339_33, &phi_bb339_35, &phi_bb339_36, &phi_bb339_37, &phi_bb339_48, &phi_bb339_49);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block341, phi_bb339_20, phi_bb339_26, phi_bb339_27, phi_bb339_28, phi_bb339_29, phi_bb339_32, phi_bb339_33, phi_bb339_35, phi_bb339_36, phi_bb339_37, phi_bb339_48, phi_bb339_49);
    } else {
      ca_.Goto(&block342, phi_bb339_20, phi_bb339_26, phi_bb339_27, phi_bb339_28, phi_bb339_29, phi_bb339_32, phi_bb339_33, phi_bb339_35, phi_bb339_36, phi_bb339_37, phi_bb339_48, phi_bb339_49);
    }
  }

  TNode<IntPtrT> phi_bb341_20;
  TNode<IntPtrT> phi_bb341_26;
  TNode<IntPtrT> phi_bb341_27;
  TNode<IntPtrT> phi_bb341_28;
  TNode<IntPtrT> phi_bb341_29;
  TNode<IntPtrT> phi_bb341_32;
  TNode<BoolT> phi_bb341_33;
  TNode<IntPtrT> phi_bb341_35;
  TNode<IntPtrT> phi_bb341_36;
  TNode<BoolT> phi_bb341_37;
  TNode<BoolT> phi_bb341_48;
  TNode<JSAny> phi_bb341_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp705;
  TNode<IntPtrT> tmp706;
  TNode<IntPtrT> tmp707;
  TNode<IntPtrT> tmp708;
  if (block341.is_used()) {
    ca_.Bind(&block341, &phi_bb341_20, &phi_bb341_26, &phi_bb341_27, &phi_bb341_28, &phi_bb341_29, &phi_bb341_32, &phi_bb341_33, &phi_bb341_35, &phi_bb341_36, &phi_bb341_37, &phi_bb341_48, &phi_bb341_49);
    std::tie(tmp705, tmp706) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb341_29}).Flatten();
    tmp707 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp708 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb341_29}, TNode<IntPtrT>{tmp707});
    ca_.Goto(&block340, phi_bb341_20, phi_bb341_26, phi_bb341_27, phi_bb341_28, tmp708, phi_bb341_32, phi_bb341_33, phi_bb341_35, phi_bb341_36, phi_bb341_37, phi_bb341_48, phi_bb341_49, tmp705, tmp706);
  }

  TNode<IntPtrT> phi_bb342_20;
  TNode<IntPtrT> phi_bb342_26;
  TNode<IntPtrT> phi_bb342_27;
  TNode<IntPtrT> phi_bb342_28;
  TNode<IntPtrT> phi_bb342_29;
  TNode<IntPtrT> phi_bb342_32;
  TNode<BoolT> phi_bb342_33;
  TNode<IntPtrT> phi_bb342_35;
  TNode<IntPtrT> phi_bb342_36;
  TNode<BoolT> phi_bb342_37;
  TNode<BoolT> phi_bb342_48;
  TNode<JSAny> phi_bb342_49;
  TNode<IntPtrT> tmp709;
  TNode<BoolT> tmp710;
  if (block342.is_used()) {
    ca_.Bind(&block342, &phi_bb342_20, &phi_bb342_26, &phi_bb342_27, &phi_bb342_28, &phi_bb342_29, &phi_bb342_32, &phi_bb342_33, &phi_bb342_35, &phi_bb342_36, &phi_bb342_37, &phi_bb342_48, &phi_bb342_49);
    tmp709 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp710 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb342_32}, TNode<IntPtrT>{tmp709});
    ca_.Branch(tmp710, &block344, std::vector<compiler::Node*>{phi_bb342_20, phi_bb342_26, phi_bb342_27, phi_bb342_28, phi_bb342_29, phi_bb342_32, phi_bb342_33, phi_bb342_35, phi_bb342_36, phi_bb342_37, phi_bb342_48, phi_bb342_49}, &block345, std::vector<compiler::Node*>{phi_bb342_20, phi_bb342_26, phi_bb342_27, phi_bb342_28, phi_bb342_29, phi_bb342_32, phi_bb342_33, phi_bb342_35, phi_bb342_36, phi_bb342_37, phi_bb342_48, phi_bb342_49});
  }

  TNode<IntPtrT> phi_bb344_20;
  TNode<IntPtrT> phi_bb344_26;
  TNode<IntPtrT> phi_bb344_27;
  TNode<IntPtrT> phi_bb344_28;
  TNode<IntPtrT> phi_bb344_29;
  TNode<IntPtrT> phi_bb344_32;
  TNode<BoolT> phi_bb344_33;
  TNode<IntPtrT> phi_bb344_35;
  TNode<IntPtrT> phi_bb344_36;
  TNode<BoolT> phi_bb344_37;
  TNode<BoolT> phi_bb344_48;
  TNode<JSAny> phi_bb344_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp711;
  TNode<IntPtrT> tmp712;
  TNode<IntPtrT> tmp713;
  TNode<BoolT> tmp714;
  if (block344.is_used()) {
    ca_.Bind(&block344, &phi_bb344_20, &phi_bb344_26, &phi_bb344_27, &phi_bb344_28, &phi_bb344_29, &phi_bb344_32, &phi_bb344_33, &phi_bb344_35, &phi_bb344_36, &phi_bb344_37, &phi_bb344_48, &phi_bb344_49);
    std::tie(tmp711, tmp712) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb344_32}).Flatten();
    tmp713 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp714 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block340, phi_bb344_20, phi_bb344_26, phi_bb344_27, phi_bb344_28, phi_bb344_29, tmp713, tmp714, phi_bb344_35, phi_bb344_36, phi_bb344_37, phi_bb344_48, phi_bb344_49, tmp711, tmp712);
  }

  TNode<IntPtrT> phi_bb345_20;
  TNode<IntPtrT> phi_bb345_26;
  TNode<IntPtrT> phi_bb345_27;
  TNode<IntPtrT> phi_bb345_28;
  TNode<IntPtrT> phi_bb345_29;
  TNode<IntPtrT> phi_bb345_32;
  TNode<BoolT> phi_bb345_33;
  TNode<IntPtrT> phi_bb345_35;
  TNode<IntPtrT> phi_bb345_36;
  TNode<BoolT> phi_bb345_37;
  TNode<BoolT> phi_bb345_48;
  TNode<JSAny> phi_bb345_49;
  TNode<Union<HeapObject, TaggedIndex>> tmp715;
  TNode<IntPtrT> tmp716;
  TNode<IntPtrT> tmp717;
  TNode<IntPtrT> tmp718;
  TNode<IntPtrT> tmp719;
  TNode<IntPtrT> tmp720;
  TNode<BoolT> tmp721;
  if (block345.is_used()) {
    ca_.Bind(&block345, &phi_bb345_20, &phi_bb345_26, &phi_bb345_27, &phi_bb345_28, &phi_bb345_29, &phi_bb345_32, &phi_bb345_33, &phi_bb345_35, &phi_bb345_36, &phi_bb345_37, &phi_bb345_48, &phi_bb345_49);
    std::tie(tmp715, tmp716) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb345_29}).Flatten();
    tmp717 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp718 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb345_29}, TNode<IntPtrT>{tmp717});
    tmp719 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp720 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp718}, TNode<IntPtrT>{tmp719});
    tmp721 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block340, phi_bb345_20, phi_bb345_26, phi_bb345_27, phi_bb345_28, tmp720, tmp718, tmp721, phi_bb345_35, phi_bb345_36, phi_bb345_37, phi_bb345_48, phi_bb345_49, tmp715, tmp716);
  }

  TNode<IntPtrT> phi_bb340_20;
  TNode<IntPtrT> phi_bb340_26;
  TNode<IntPtrT> phi_bb340_27;
  TNode<IntPtrT> phi_bb340_28;
  TNode<IntPtrT> phi_bb340_29;
  TNode<IntPtrT> phi_bb340_32;
  TNode<BoolT> phi_bb340_33;
  TNode<IntPtrT> phi_bb340_35;
  TNode<IntPtrT> phi_bb340_36;
  TNode<BoolT> phi_bb340_37;
  TNode<BoolT> phi_bb340_48;
  TNode<JSAny> phi_bb340_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb340_52;
  TNode<IntPtrT> phi_bb340_53;
  if (block340.is_used()) {
    ca_.Bind(&block340, &phi_bb340_20, &phi_bb340_26, &phi_bb340_27, &phi_bb340_28, &phi_bb340_29, &phi_bb340_32, &phi_bb340_33, &phi_bb340_35, &phi_bb340_36, &phi_bb340_37, &phi_bb340_48, &phi_bb340_49, &phi_bb340_52, &phi_bb340_53);
    ca_.Goto(&block337, phi_bb340_20, phi_bb340_26, phi_bb340_27, phi_bb340_28, phi_bb340_29, phi_bb340_32, phi_bb340_33, phi_bb340_35, phi_bb340_36, phi_bb340_37, phi_bb340_48, phi_bb340_49, phi_bb340_52, phi_bb340_53);
  }

  TNode<IntPtrT> phi_bb337_20;
  TNode<IntPtrT> phi_bb337_26;
  TNode<IntPtrT> phi_bb337_27;
  TNode<IntPtrT> phi_bb337_28;
  TNode<IntPtrT> phi_bb337_29;
  TNode<IntPtrT> phi_bb337_32;
  TNode<BoolT> phi_bb337_33;
  TNode<IntPtrT> phi_bb337_35;
  TNode<IntPtrT> phi_bb337_36;
  TNode<BoolT> phi_bb337_37;
  TNode<BoolT> phi_bb337_48;
  TNode<JSAny> phi_bb337_49;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb337_52;
  TNode<IntPtrT> phi_bb337_53;
  TNode<IntPtrT> tmp722;
  if (block337.is_used()) {
    ca_.Bind(&block337, &phi_bb337_20, &phi_bb337_26, &phi_bb337_27, &phi_bb337_28, &phi_bb337_29, &phi_bb337_32, &phi_bb337_33, &phi_bb337_35, &phi_bb337_36, &phi_bb337_37, &phi_bb337_48, &phi_bb337_49, &phi_bb337_52, &phi_bb337_53);
    tmp722 = CodeStubAssembler(state_).BitcastTaggedToWord(TNode<Object>{tmp694});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb337_52, phi_bb337_53}, tmp722);
    ca_.Goto(&block336, phi_bb337_20, tmp698, phi_bb337_26, phi_bb337_27, phi_bb337_28, phi_bb337_29, phi_bb337_32, phi_bb337_33, phi_bb337_35, phi_bb337_36, phi_bb337_37, phi_bb337_48, phi_bb337_49);
  }

  TNode<IntPtrT> phi_bb335_20;
  TNode<IntPtrT> phi_bb335_25;
  TNode<IntPtrT> phi_bb335_26;
  TNode<IntPtrT> phi_bb335_27;
  TNode<IntPtrT> phi_bb335_28;
  TNode<IntPtrT> phi_bb335_29;
  TNode<IntPtrT> phi_bb335_32;
  TNode<BoolT> phi_bb335_33;
  TNode<IntPtrT> phi_bb335_35;
  TNode<IntPtrT> phi_bb335_36;
  TNode<BoolT> phi_bb335_37;
  TNode<BoolT> phi_bb335_48;
  TNode<JSAny> phi_bb335_49;
  TNode<BoolT> tmp723;
  TNode<Union<HeapObject, TaggedIndex>> tmp724;
  TNode<IntPtrT> tmp725;
  TNode<IntPtrT> tmp726;
  TNode<UintPtrT> tmp727;
  TNode<UintPtrT> tmp728;
  TNode<BoolT> tmp729;
  if (block335.is_used()) {
    ca_.Bind(&block335, &phi_bb335_20, &phi_bb335_25, &phi_bb335_26, &phi_bb335_27, &phi_bb335_28, &phi_bb335_29, &phi_bb335_32, &phi_bb335_33, &phi_bb335_35, &phi_bb335_36, &phi_bb335_37, &phi_bb335_48, &phi_bb335_49);
    tmp723 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    std::tie(tmp724, tmp725, tmp726) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb224_41}).Flatten();
    tmp727 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb335_20});
    tmp728 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp726});
    tmp729 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp727}, TNode<UintPtrT>{tmp728});
    ca_.Branch(tmp729, &block350, std::vector<compiler::Node*>{phi_bb335_20, phi_bb335_25, phi_bb335_26, phi_bb335_27, phi_bb335_28, phi_bb335_29, phi_bb335_32, phi_bb335_33, phi_bb335_35, phi_bb335_36, phi_bb335_37, phi_bb335_49, phi_bb335_20, phi_bb335_20, phi_bb335_20, phi_bb335_20}, &block351, std::vector<compiler::Node*>{phi_bb335_20, phi_bb335_25, phi_bb335_26, phi_bb335_27, phi_bb335_28, phi_bb335_29, phi_bb335_32, phi_bb335_33, phi_bb335_35, phi_bb335_36, phi_bb335_37, phi_bb335_49, phi_bb335_20, phi_bb335_20, phi_bb335_20, phi_bb335_20});
  }

  TNode<IntPtrT> phi_bb350_20;
  TNode<IntPtrT> phi_bb350_25;
  TNode<IntPtrT> phi_bb350_26;
  TNode<IntPtrT> phi_bb350_27;
  TNode<IntPtrT> phi_bb350_28;
  TNode<IntPtrT> phi_bb350_29;
  TNode<IntPtrT> phi_bb350_32;
  TNode<BoolT> phi_bb350_33;
  TNode<IntPtrT> phi_bb350_35;
  TNode<IntPtrT> phi_bb350_36;
  TNode<BoolT> phi_bb350_37;
  TNode<JSAny> phi_bb350_49;
  TNode<IntPtrT> phi_bb350_56;
  TNode<IntPtrT> phi_bb350_57;
  TNode<IntPtrT> phi_bb350_61;
  TNode<IntPtrT> phi_bb350_62;
  TNode<IntPtrT> tmp730;
  TNode<IntPtrT> tmp731;
  TNode<Union<HeapObject, TaggedIndex>> tmp732;
  TNode<IntPtrT> tmp733;
  if (block350.is_used()) {
    ca_.Bind(&block350, &phi_bb350_20, &phi_bb350_25, &phi_bb350_26, &phi_bb350_27, &phi_bb350_28, &phi_bb350_29, &phi_bb350_32, &phi_bb350_33, &phi_bb350_35, &phi_bb350_36, &phi_bb350_37, &phi_bb350_49, &phi_bb350_56, &phi_bb350_57, &phi_bb350_61, &phi_bb350_62);
    tmp730 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb350_62});
    tmp731 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp725}, TNode<IntPtrT>{tmp730});
    std::tie(tmp732, tmp733) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp724}, TNode<IntPtrT>{tmp731}).Flatten();
    CodeStubAssembler(state_).StoreReference<Object>(CodeStubAssembler::Reference{tmp732, tmp733}, tmp694);
    ca_.Goto(&block336, phi_bb350_20, phi_bb350_25, phi_bb350_26, phi_bb350_27, phi_bb350_28, phi_bb350_29, phi_bb350_32, phi_bb350_33, phi_bb350_35, phi_bb350_36, phi_bb350_37, tmp723, phi_bb350_49);
  }

  TNode<IntPtrT> phi_bb351_20;
  TNode<IntPtrT> phi_bb351_25;
  TNode<IntPtrT> phi_bb351_26;
  TNode<IntPtrT> phi_bb351_27;
  TNode<IntPtrT> phi_bb351_28;
  TNode<IntPtrT> phi_bb351_29;
  TNode<IntPtrT> phi_bb351_32;
  TNode<BoolT> phi_bb351_33;
  TNode<IntPtrT> phi_bb351_35;
  TNode<IntPtrT> phi_bb351_36;
  TNode<BoolT> phi_bb351_37;
  TNode<JSAny> phi_bb351_49;
  TNode<IntPtrT> phi_bb351_56;
  TNode<IntPtrT> phi_bb351_57;
  TNode<IntPtrT> phi_bb351_61;
  TNode<IntPtrT> phi_bb351_62;
  if (block351.is_used()) {
    ca_.Bind(&block351, &phi_bb351_20, &phi_bb351_25, &phi_bb351_26, &phi_bb351_27, &phi_bb351_28, &phi_bb351_29, &phi_bb351_32, &phi_bb351_33, &phi_bb351_35, &phi_bb351_36, &phi_bb351_37, &phi_bb351_49, &phi_bb351_56, &phi_bb351_57, &phi_bb351_61, &phi_bb351_62);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb336_20;
  TNode<IntPtrT> phi_bb336_25;
  TNode<IntPtrT> phi_bb336_26;
  TNode<IntPtrT> phi_bb336_27;
  TNode<IntPtrT> phi_bb336_28;
  TNode<IntPtrT> phi_bb336_29;
  TNode<IntPtrT> phi_bb336_32;
  TNode<BoolT> phi_bb336_33;
  TNode<IntPtrT> phi_bb336_35;
  TNode<IntPtrT> phi_bb336_36;
  TNode<BoolT> phi_bb336_37;
  TNode<BoolT> phi_bb336_48;
  TNode<JSAny> phi_bb336_49;
  if (block336.is_used()) {
    ca_.Bind(&block336, &phi_bb336_20, &phi_bb336_25, &phi_bb336_26, &phi_bb336_27, &phi_bb336_28, &phi_bb336_29, &phi_bb336_32, &phi_bb336_33, &phi_bb336_35, &phi_bb336_36, &phi_bb336_37, &phi_bb336_48, &phi_bb336_49);
    ca_.Goto(&block301, phi_bb336_20, phi_bb336_25, phi_bb336_26, phi_bb336_27, phi_bb336_28, phi_bb336_29, phi_bb336_32, phi_bb336_33, phi_bb336_35, phi_bb336_36, phi_bb336_37, phi_bb336_48, phi_bb336_49);
  }

  TNode<IntPtrT> phi_bb301_20;
  TNode<IntPtrT> phi_bb301_25;
  TNode<IntPtrT> phi_bb301_26;
  TNode<IntPtrT> phi_bb301_27;
  TNode<IntPtrT> phi_bb301_28;
  TNode<IntPtrT> phi_bb301_29;
  TNode<IntPtrT> phi_bb301_32;
  TNode<BoolT> phi_bb301_33;
  TNode<IntPtrT> phi_bb301_35;
  TNode<IntPtrT> phi_bb301_36;
  TNode<BoolT> phi_bb301_37;
  TNode<BoolT> phi_bb301_48;
  TNode<JSAny> phi_bb301_49;
  if (block301.is_used()) {
    ca_.Bind(&block301, &phi_bb301_20, &phi_bb301_25, &phi_bb301_26, &phi_bb301_27, &phi_bb301_28, &phi_bb301_29, &phi_bb301_32, &phi_bb301_33, &phi_bb301_35, &phi_bb301_36, &phi_bb301_37, &phi_bb301_48, &phi_bb301_49);
    ca_.Goto(&block286, phi_bb301_20, phi_bb301_25, phi_bb301_26, phi_bb301_27, phi_bb301_28, phi_bb301_29, phi_bb301_32, phi_bb301_33, phi_bb301_35, phi_bb301_36, phi_bb301_37, phi_bb301_48, phi_bb301_49);
  }

  TNode<IntPtrT> phi_bb286_20;
  TNode<IntPtrT> phi_bb286_25;
  TNode<IntPtrT> phi_bb286_26;
  TNode<IntPtrT> phi_bb286_27;
  TNode<IntPtrT> phi_bb286_28;
  TNode<IntPtrT> phi_bb286_29;
  TNode<IntPtrT> phi_bb286_32;
  TNode<BoolT> phi_bb286_33;
  TNode<IntPtrT> phi_bb286_35;
  TNode<IntPtrT> phi_bb286_36;
  TNode<BoolT> phi_bb286_37;
  TNode<BoolT> phi_bb286_48;
  TNode<JSAny> phi_bb286_49;
  if (block286.is_used()) {
    ca_.Bind(&block286, &phi_bb286_20, &phi_bb286_25, &phi_bb286_26, &phi_bb286_27, &phi_bb286_28, &phi_bb286_29, &phi_bb286_32, &phi_bb286_33, &phi_bb286_35, &phi_bb286_36, &phi_bb286_37, &phi_bb286_48, &phi_bb286_49);
    ca_.Goto(&block271, phi_bb286_20, phi_bb286_25, phi_bb286_26, phi_bb286_27, phi_bb286_28, phi_bb286_29, phi_bb286_32, phi_bb286_33, phi_bb286_35, phi_bb286_36, phi_bb286_37, phi_bb286_48, phi_bb286_49);
  }

  TNode<IntPtrT> phi_bb271_20;
  TNode<IntPtrT> phi_bb271_25;
  TNode<IntPtrT> phi_bb271_26;
  TNode<IntPtrT> phi_bb271_27;
  TNode<IntPtrT> phi_bb271_28;
  TNode<IntPtrT> phi_bb271_29;
  TNode<IntPtrT> phi_bb271_32;
  TNode<BoolT> phi_bb271_33;
  TNode<IntPtrT> phi_bb271_35;
  TNode<IntPtrT> phi_bb271_36;
  TNode<BoolT> phi_bb271_37;
  TNode<BoolT> phi_bb271_48;
  TNode<JSAny> phi_bb271_49;
  if (block271.is_used()) {
    ca_.Bind(&block271, &phi_bb271_20, &phi_bb271_25, &phi_bb271_26, &phi_bb271_27, &phi_bb271_28, &phi_bb271_29, &phi_bb271_32, &phi_bb271_33, &phi_bb271_35, &phi_bb271_36, &phi_bb271_37, &phi_bb271_48, &phi_bb271_49);
    ca_.Goto(&block255, phi_bb271_20, phi_bb271_25, phi_bb271_26, phi_bb271_27, phi_bb271_28, phi_bb271_29, phi_bb271_32, phi_bb271_33, phi_bb271_35, phi_bb271_36, phi_bb271_37, phi_bb271_48, phi_bb271_49);
  }

  TNode<IntPtrT> phi_bb255_20;
  TNode<IntPtrT> phi_bb255_25;
  TNode<IntPtrT> phi_bb255_26;
  TNode<IntPtrT> phi_bb255_27;
  TNode<IntPtrT> phi_bb255_28;
  TNode<IntPtrT> phi_bb255_29;
  TNode<IntPtrT> phi_bb255_32;
  TNode<BoolT> phi_bb255_33;
  TNode<IntPtrT> phi_bb255_35;
  TNode<IntPtrT> phi_bb255_36;
  TNode<BoolT> phi_bb255_37;
  TNode<BoolT> phi_bb255_48;
  TNode<JSAny> phi_bb255_49;
  TNode<IntPtrT> tmp734;
  TNode<IntPtrT> tmp735;
  if (block255.is_used()) {
    ca_.Bind(&block255, &phi_bb255_20, &phi_bb255_25, &phi_bb255_26, &phi_bb255_27, &phi_bb255_28, &phi_bb255_29, &phi_bb255_32, &phi_bb255_33, &phi_bb255_35, &phi_bb255_36, &phi_bb255_37, &phi_bb255_48, &phi_bb255_49);
    tmp734 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp735 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb255_20}, TNode<IntPtrT>{tmp734});
    ca_.Goto(&block233, tmp735, phi_bb255_25, phi_bb255_26, phi_bb255_27, phi_bb255_28, phi_bb255_29, phi_bb255_32, phi_bb255_33, phi_bb255_35, phi_bb255_36, phi_bb255_37, tmp506, phi_bb255_48);
  }

  TNode<IntPtrT> phi_bb232_20;
  TNode<IntPtrT> phi_bb232_25;
  TNode<IntPtrT> phi_bb232_26;
  TNode<IntPtrT> phi_bb232_27;
  TNode<IntPtrT> phi_bb232_28;
  TNode<IntPtrT> phi_bb232_29;
  TNode<IntPtrT> phi_bb232_32;
  TNode<BoolT> phi_bb232_33;
  TNode<IntPtrT> phi_bb232_35;
  TNode<IntPtrT> phi_bb232_36;
  TNode<BoolT> phi_bb232_37;
  TNode<IntPtrT> phi_bb232_46;
  TNode<BoolT> phi_bb232_48;
  if (block232.is_used()) {
    ca_.Bind(&block232, &phi_bb232_20, &phi_bb232_25, &phi_bb232_26, &phi_bb232_27, &phi_bb232_28, &phi_bb232_29, &phi_bb232_32, &phi_bb232_33, &phi_bb232_35, &phi_bb232_36, &phi_bb232_37, &phi_bb232_46, &phi_bb232_48);
    ca_.Branch(phi_bb232_48, &block354, std::vector<compiler::Node*>{phi_bb232_20, phi_bb232_25, phi_bb232_26, phi_bb232_27, phi_bb232_28, phi_bb232_29, phi_bb232_32, phi_bb232_33, phi_bb232_35, phi_bb232_36, phi_bb232_37, phi_bb232_46, phi_bb232_48}, &block355, std::vector<compiler::Node*>{phi_bb232_20, phi_bb232_25, phi_bb232_26, phi_bb232_27, phi_bb232_28, phi_bb232_29, phi_bb232_32, phi_bb232_33, phi_bb232_35, phi_bb232_36, phi_bb232_37, phi_bb232_46, tmp484, phi_bb232_48});
  }

  TNode<IntPtrT> phi_bb354_20;
  TNode<IntPtrT> phi_bb354_25;
  TNode<IntPtrT> phi_bb354_26;
  TNode<IntPtrT> phi_bb354_27;
  TNode<IntPtrT> phi_bb354_28;
  TNode<IntPtrT> phi_bb354_29;
  TNode<IntPtrT> phi_bb354_32;
  TNode<BoolT> phi_bb354_33;
  TNode<IntPtrT> phi_bb354_35;
  TNode<IntPtrT> phi_bb354_36;
  TNode<BoolT> phi_bb354_37;
  TNode<IntPtrT> phi_bb354_46;
  TNode<BoolT> phi_bb354_48;
  TNode<BoolT> tmp736;
  if (block354.is_used()) {
    ca_.Bind(&block354, &phi_bb354_20, &phi_bb354_25, &phi_bb354_26, &phi_bb354_27, &phi_bb354_28, &phi_bb354_29, &phi_bb354_32, &phi_bb354_33, &phi_bb354_35, &phi_bb354_36, &phi_bb354_37, &phi_bb354_46, &phi_bb354_48);
    tmp736 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{phi_bb354_33});
    ca_.Branch(tmp736, &block357, std::vector<compiler::Node*>{phi_bb354_20, phi_bb354_25, phi_bb354_26, phi_bb354_27, phi_bb354_28, phi_bb354_29, phi_bb354_32, phi_bb354_33, phi_bb354_35, phi_bb354_36, phi_bb354_37, phi_bb354_46, phi_bb354_48}, &block358, std::vector<compiler::Node*>{phi_bb354_20, phi_bb354_25, phi_bb354_26, phi_bb354_27, phi_bb354_28, phi_bb354_29, phi_bb354_32, phi_bb354_33, phi_bb354_35, phi_bb354_36, phi_bb354_37, phi_bb354_46, phi_bb354_48});
  }

  TNode<IntPtrT> phi_bb357_20;
  TNode<IntPtrT> phi_bb357_25;
  TNode<IntPtrT> phi_bb357_26;
  TNode<IntPtrT> phi_bb357_27;
  TNode<IntPtrT> phi_bb357_28;
  TNode<IntPtrT> phi_bb357_29;
  TNode<IntPtrT> phi_bb357_32;
  TNode<BoolT> phi_bb357_33;
  TNode<IntPtrT> phi_bb357_35;
  TNode<IntPtrT> phi_bb357_36;
  TNode<BoolT> phi_bb357_37;
  TNode<IntPtrT> phi_bb357_46;
  TNode<BoolT> phi_bb357_48;
  TNode<IntPtrT> tmp737;
  if (block357.is_used()) {
    ca_.Bind(&block357, &phi_bb357_20, &phi_bb357_25, &phi_bb357_26, &phi_bb357_27, &phi_bb357_28, &phi_bb357_29, &phi_bb357_32, &phi_bb357_33, &phi_bb357_35, &phi_bb357_36, &phi_bb357_37, &phi_bb357_46, &phi_bb357_48);
    tmp737 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block358, phi_bb357_20, phi_bb357_25, phi_bb357_26, phi_bb357_27, phi_bb357_28, phi_bb357_29, tmp737, phi_bb357_33, phi_bb357_35, phi_bb357_36, phi_bb357_37, phi_bb357_46, phi_bb357_48);
  }

  TNode<IntPtrT> phi_bb358_20;
  TNode<IntPtrT> phi_bb358_25;
  TNode<IntPtrT> phi_bb358_26;
  TNode<IntPtrT> phi_bb358_27;
  TNode<IntPtrT> phi_bb358_28;
  TNode<IntPtrT> phi_bb358_29;
  TNode<IntPtrT> phi_bb358_32;
  TNode<BoolT> phi_bb358_33;
  TNode<IntPtrT> phi_bb358_35;
  TNode<IntPtrT> phi_bb358_36;
  TNode<BoolT> phi_bb358_37;
  TNode<IntPtrT> phi_bb358_46;
  TNode<BoolT> phi_bb358_48;
  TNode<IntPtrT> tmp738;
  TNode<IntPtrT> tmp739;
  TNode<IntPtrT> tmp740;
  if (block358.is_used()) {
    ca_.Bind(&block358, &phi_bb358_20, &phi_bb358_25, &phi_bb358_26, &phi_bb358_27, &phi_bb358_28, &phi_bb358_29, &phi_bb358_32, &phi_bb358_33, &phi_bb358_35, &phi_bb358_36, &phi_bb358_37, &phi_bb358_46, &phi_bb358_48);
    tmp738 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{tmp55});
    tmp739 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp54}, TNode<IntPtrT>{tmp738});
    tmp740 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    ca_.Goto(&block362, tmp740, phi_bb358_25, phi_bb358_26, phi_bb358_27, phi_bb358_28, phi_bb358_29, phi_bb358_32, phi_bb358_33, phi_bb358_35, phi_bb358_36, phi_bb358_37, tmp54, phi_bb358_48);
  }

  TNode<IntPtrT> phi_bb362_20;
  TNode<IntPtrT> phi_bb362_25;
  TNode<IntPtrT> phi_bb362_26;
  TNode<IntPtrT> phi_bb362_27;
  TNode<IntPtrT> phi_bb362_28;
  TNode<IntPtrT> phi_bb362_29;
  TNode<IntPtrT> phi_bb362_32;
  TNode<BoolT> phi_bb362_33;
  TNode<IntPtrT> phi_bb362_35;
  TNode<IntPtrT> phi_bb362_36;
  TNode<BoolT> phi_bb362_37;
  TNode<IntPtrT> phi_bb362_46;
  TNode<BoolT> phi_bb362_48;
  TNode<BoolT> tmp741;
  TNode<BoolT> tmp742;
  if (block362.is_used()) {
    ca_.Bind(&block362, &phi_bb362_20, &phi_bb362_25, &phi_bb362_26, &phi_bb362_27, &phi_bb362_28, &phi_bb362_29, &phi_bb362_32, &phi_bb362_33, &phi_bb362_35, &phi_bb362_36, &phi_bb362_37, &phi_bb362_46, &phi_bb362_48);
    tmp741 = CodeStubAssembler(state_).WordEqual(TNode<IntPtrT>{phi_bb362_46}, TNode<IntPtrT>{tmp739});
    tmp742 = CodeStubAssembler(state_).Word32BinaryNot(TNode<BoolT>{tmp741});
    ca_.Branch(tmp742, &block360, std::vector<compiler::Node*>{phi_bb362_20, phi_bb362_25, phi_bb362_26, phi_bb362_27, phi_bb362_28, phi_bb362_29, phi_bb362_32, phi_bb362_33, phi_bb362_35, phi_bb362_36, phi_bb362_37, phi_bb362_46, phi_bb362_48}, &block361, std::vector<compiler::Node*>{phi_bb362_20, phi_bb362_25, phi_bb362_26, phi_bb362_27, phi_bb362_28, phi_bb362_29, phi_bb362_32, phi_bb362_33, phi_bb362_35, phi_bb362_36, phi_bb362_37, phi_bb362_46, phi_bb362_48});
  }

  TNode<IntPtrT> phi_bb360_20;
  TNode<IntPtrT> phi_bb360_25;
  TNode<IntPtrT> phi_bb360_26;
  TNode<IntPtrT> phi_bb360_27;
  TNode<IntPtrT> phi_bb360_28;
  TNode<IntPtrT> phi_bb360_29;
  TNode<IntPtrT> phi_bb360_32;
  TNode<BoolT> phi_bb360_33;
  TNode<IntPtrT> phi_bb360_35;
  TNode<IntPtrT> phi_bb360_36;
  TNode<BoolT> phi_bb360_37;
  TNode<IntPtrT> phi_bb360_46;
  TNode<BoolT> phi_bb360_48;
  TNode<Union<HeapObject, TaggedIndex>> tmp743;
  TNode<IntPtrT> tmp744;
  TNode<IntPtrT> tmp745;
  TNode<IntPtrT> tmp746;
  TNode<Uint32T> tmp747;
  TNode<Uint32T> tmp748;
  TNode<Uint32T> tmp749;
  TNode<Uint32T> tmp750;
  TNode<BoolT> tmp751;
  if (block360.is_used()) {
    ca_.Bind(&block360, &phi_bb360_20, &phi_bb360_25, &phi_bb360_26, &phi_bb360_27, &phi_bb360_28, &phi_bb360_29, &phi_bb360_32, &phi_bb360_33, &phi_bb360_35, &phi_bb360_36, &phi_bb360_37, &phi_bb360_46, &phi_bb360_48);
    std::tie(tmp743, tmp744) = NewReference_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp53}, TNode<IntPtrT>{phi_bb360_46}).Flatten();
    tmp745 = FromConstexpr_intptr_constexpr_int31_0(state_, kInt32Size);
    tmp746 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb360_46}, TNode<IntPtrT>{tmp745});
    tmp747 = CodeStubAssembler(state_).LoadReference<Uint32T>(CodeStubAssembler::Reference{tmp743, tmp744});
    tmp748 = FromConstexpr_uint32_constexpr_uint32_0(state_, wasm::ValueType::kIsRefBit);
    tmp749 = CodeStubAssembler(state_).Word32And(TNode<Uint32T>{tmp747}, TNode<Uint32T>{tmp748});
    tmp750 = FromConstexpr_uint32_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp751 = CodeStubAssembler(state_).Word32NotEqual(TNode<Uint32T>{tmp749}, TNode<Uint32T>{tmp750});
    ca_.Branch(tmp751, &block371, std::vector<compiler::Node*>{phi_bb360_20, phi_bb360_25, phi_bb360_26, phi_bb360_27, phi_bb360_28, phi_bb360_29, phi_bb360_32, phi_bb360_33, phi_bb360_35, phi_bb360_36, phi_bb360_37, phi_bb360_48}, &block372, std::vector<compiler::Node*>{phi_bb360_20, phi_bb360_25, phi_bb360_26, phi_bb360_27, phi_bb360_28, phi_bb360_29, phi_bb360_32, phi_bb360_33, phi_bb360_35, phi_bb360_36, phi_bb360_37, phi_bb360_48});
  }

  TNode<IntPtrT> phi_bb371_20;
  TNode<IntPtrT> phi_bb371_25;
  TNode<IntPtrT> phi_bb371_26;
  TNode<IntPtrT> phi_bb371_27;
  TNode<IntPtrT> phi_bb371_28;
  TNode<IntPtrT> phi_bb371_29;
  TNode<IntPtrT> phi_bb371_32;
  TNode<BoolT> phi_bb371_33;
  TNode<IntPtrT> phi_bb371_35;
  TNode<IntPtrT> phi_bb371_36;
  TNode<BoolT> phi_bb371_37;
  TNode<BoolT> phi_bb371_48;
  TNode<IntPtrT> tmp752;
  TNode<IntPtrT> tmp753;
  TNode<IntPtrT> tmp754;
  TNode<BoolT> tmp755;
  if (block371.is_used()) {
    ca_.Bind(&block371, &phi_bb371_20, &phi_bb371_25, &phi_bb371_26, &phi_bb371_27, &phi_bb371_28, &phi_bb371_29, &phi_bb371_32, &phi_bb371_33, &phi_bb371_35, &phi_bb371_36, &phi_bb371_37, &phi_bb371_48);
    tmp752 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp753 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{phi_bb371_25}, TNode<IntPtrT>{tmp752});
    tmp754 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp755 = CodeStubAssembler(state_).IntPtrGreaterThan(TNode<IntPtrT>{phi_bb371_25}, TNode<IntPtrT>{tmp754});
    ca_.Branch(tmp755, &block374, std::vector<compiler::Node*>{phi_bb371_20, phi_bb371_26, phi_bb371_27, phi_bb371_28, phi_bb371_29, phi_bb371_32, phi_bb371_33, phi_bb371_35, phi_bb371_36, phi_bb371_37, phi_bb371_48}, &block375, std::vector<compiler::Node*>{phi_bb371_20, phi_bb371_26, phi_bb371_27, phi_bb371_28, phi_bb371_29, phi_bb371_32, phi_bb371_33, phi_bb371_35, phi_bb371_36, phi_bb371_37, phi_bb371_48});
  }

  TNode<IntPtrT> phi_bb374_20;
  TNode<IntPtrT> phi_bb374_26;
  TNode<IntPtrT> phi_bb374_27;
  TNode<IntPtrT> phi_bb374_28;
  TNode<IntPtrT> phi_bb374_29;
  TNode<IntPtrT> phi_bb374_32;
  TNode<BoolT> phi_bb374_33;
  TNode<IntPtrT> phi_bb374_35;
  TNode<IntPtrT> phi_bb374_36;
  TNode<BoolT> phi_bb374_37;
  TNode<BoolT> phi_bb374_48;
  TNode<Union<HeapObject, TaggedIndex>> tmp756;
  TNode<IntPtrT> tmp757;
  TNode<IntPtrT> tmp758;
  TNode<IntPtrT> tmp759;
  if (block374.is_used()) {
    ca_.Bind(&block374, &phi_bb374_20, &phi_bb374_26, &phi_bb374_27, &phi_bb374_28, &phi_bb374_29, &phi_bb374_32, &phi_bb374_33, &phi_bb374_35, &phi_bb374_36, &phi_bb374_37, &phi_bb374_48);
    std::tie(tmp756, tmp757) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb374_27}).Flatten();
    tmp758 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp759 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb374_27}, TNode<IntPtrT>{tmp758});
    ca_.Goto(&block373, phi_bb374_20, phi_bb374_26, tmp759, phi_bb374_28, phi_bb374_29, phi_bb374_32, phi_bb374_33, phi_bb374_35, phi_bb374_36, phi_bb374_37, phi_bb374_48, tmp756, tmp757);
  }

  TNode<IntPtrT> phi_bb375_20;
  TNode<IntPtrT> phi_bb375_26;
  TNode<IntPtrT> phi_bb375_27;
  TNode<IntPtrT> phi_bb375_28;
  TNode<IntPtrT> phi_bb375_29;
  TNode<IntPtrT> phi_bb375_32;
  TNode<BoolT> phi_bb375_33;
  TNode<IntPtrT> phi_bb375_35;
  TNode<IntPtrT> phi_bb375_36;
  TNode<BoolT> phi_bb375_37;
  TNode<BoolT> phi_bb375_48;
  if (block375.is_used()) {
    ca_.Bind(&block375, &phi_bb375_20, &phi_bb375_26, &phi_bb375_27, &phi_bb375_28, &phi_bb375_29, &phi_bb375_32, &phi_bb375_33, &phi_bb375_35, &phi_bb375_36, &phi_bb375_37, &phi_bb375_48);
    if (((CodeStubAssembler(state_).Is64()))) {
      ca_.Goto(&block377, phi_bb375_20, phi_bb375_26, phi_bb375_27, phi_bb375_28, phi_bb375_29, phi_bb375_32, phi_bb375_33, phi_bb375_35, phi_bb375_36, phi_bb375_37, phi_bb375_48);
    } else {
      ca_.Goto(&block378, phi_bb375_20, phi_bb375_26, phi_bb375_27, phi_bb375_28, phi_bb375_29, phi_bb375_32, phi_bb375_33, phi_bb375_35, phi_bb375_36, phi_bb375_37, phi_bb375_48);
    }
  }

  TNode<IntPtrT> phi_bb377_20;
  TNode<IntPtrT> phi_bb377_26;
  TNode<IntPtrT> phi_bb377_27;
  TNode<IntPtrT> phi_bb377_28;
  TNode<IntPtrT> phi_bb377_29;
  TNode<IntPtrT> phi_bb377_32;
  TNode<BoolT> phi_bb377_33;
  TNode<IntPtrT> phi_bb377_35;
  TNode<IntPtrT> phi_bb377_36;
  TNode<BoolT> phi_bb377_37;
  TNode<BoolT> phi_bb377_48;
  TNode<Union<HeapObject, TaggedIndex>> tmp760;
  TNode<IntPtrT> tmp761;
  TNode<IntPtrT> tmp762;
  TNode<IntPtrT> tmp763;
  if (block377.is_used()) {
    ca_.Bind(&block377, &phi_bb377_20, &phi_bb377_26, &phi_bb377_27, &phi_bb377_28, &phi_bb377_29, &phi_bb377_32, &phi_bb377_33, &phi_bb377_35, &phi_bb377_36, &phi_bb377_37, &phi_bb377_48);
    std::tie(tmp760, tmp761) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb377_29}).Flatten();
    tmp762 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp763 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb377_29}, TNode<IntPtrT>{tmp762});
    ca_.Goto(&block376, phi_bb377_20, phi_bb377_26, phi_bb377_27, phi_bb377_28, tmp763, phi_bb377_32, phi_bb377_33, phi_bb377_35, phi_bb377_36, phi_bb377_37, phi_bb377_48, tmp760, tmp761);
  }

  TNode<IntPtrT> phi_bb378_20;
  TNode<IntPtrT> phi_bb378_26;
  TNode<IntPtrT> phi_bb378_27;
  TNode<IntPtrT> phi_bb378_28;
  TNode<IntPtrT> phi_bb378_29;
  TNode<IntPtrT> phi_bb378_32;
  TNode<BoolT> phi_bb378_33;
  TNode<IntPtrT> phi_bb378_35;
  TNode<IntPtrT> phi_bb378_36;
  TNode<BoolT> phi_bb378_37;
  TNode<BoolT> phi_bb378_48;
  TNode<IntPtrT> tmp764;
  TNode<BoolT> tmp765;
  if (block378.is_used()) {
    ca_.Bind(&block378, &phi_bb378_20, &phi_bb378_26, &phi_bb378_27, &phi_bb378_28, &phi_bb378_29, &phi_bb378_32, &phi_bb378_33, &phi_bb378_35, &phi_bb378_36, &phi_bb378_37, &phi_bb378_48);
    tmp764 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp765 = CodeStubAssembler(state_).WordNotEqual(TNode<IntPtrT>{phi_bb378_32}, TNode<IntPtrT>{tmp764});
    ca_.Branch(tmp765, &block380, std::vector<compiler::Node*>{phi_bb378_20, phi_bb378_26, phi_bb378_27, phi_bb378_28, phi_bb378_29, phi_bb378_32, phi_bb378_33, phi_bb378_35, phi_bb378_36, phi_bb378_37, phi_bb378_48}, &block381, std::vector<compiler::Node*>{phi_bb378_20, phi_bb378_26, phi_bb378_27, phi_bb378_28, phi_bb378_29, phi_bb378_32, phi_bb378_33, phi_bb378_35, phi_bb378_36, phi_bb378_37, phi_bb378_48});
  }

  TNode<IntPtrT> phi_bb380_20;
  TNode<IntPtrT> phi_bb380_26;
  TNode<IntPtrT> phi_bb380_27;
  TNode<IntPtrT> phi_bb380_28;
  TNode<IntPtrT> phi_bb380_29;
  TNode<IntPtrT> phi_bb380_32;
  TNode<BoolT> phi_bb380_33;
  TNode<IntPtrT> phi_bb380_35;
  TNode<IntPtrT> phi_bb380_36;
  TNode<BoolT> phi_bb380_37;
  TNode<BoolT> phi_bb380_48;
  TNode<Union<HeapObject, TaggedIndex>> tmp766;
  TNode<IntPtrT> tmp767;
  TNode<IntPtrT> tmp768;
  TNode<BoolT> tmp769;
  if (block380.is_used()) {
    ca_.Bind(&block380, &phi_bb380_20, &phi_bb380_26, &phi_bb380_27, &phi_bb380_28, &phi_bb380_29, &phi_bb380_32, &phi_bb380_33, &phi_bb380_35, &phi_bb380_36, &phi_bb380_37, &phi_bb380_48);
    std::tie(tmp766, tmp767) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb380_32}).Flatten();
    tmp768 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    tmp769 = FromConstexpr_bool_constexpr_bool_0(state_, false);
    ca_.Goto(&block376, phi_bb380_20, phi_bb380_26, phi_bb380_27, phi_bb380_28, phi_bb380_29, tmp768, tmp769, phi_bb380_35, phi_bb380_36, phi_bb380_37, phi_bb380_48, tmp766, tmp767);
  }

  TNode<IntPtrT> phi_bb381_20;
  TNode<IntPtrT> phi_bb381_26;
  TNode<IntPtrT> phi_bb381_27;
  TNode<IntPtrT> phi_bb381_28;
  TNode<IntPtrT> phi_bb381_29;
  TNode<IntPtrT> phi_bb381_32;
  TNode<BoolT> phi_bb381_33;
  TNode<IntPtrT> phi_bb381_35;
  TNode<IntPtrT> phi_bb381_36;
  TNode<BoolT> phi_bb381_37;
  TNode<BoolT> phi_bb381_48;
  TNode<Union<HeapObject, TaggedIndex>> tmp770;
  TNode<IntPtrT> tmp771;
  TNode<IntPtrT> tmp772;
  TNode<IntPtrT> tmp773;
  TNode<IntPtrT> tmp774;
  TNode<IntPtrT> tmp775;
  TNode<BoolT> tmp776;
  if (block381.is_used()) {
    ca_.Bind(&block381, &phi_bb381_20, &phi_bb381_26, &phi_bb381_27, &phi_bb381_28, &phi_bb381_29, &phi_bb381_32, &phi_bb381_33, &phi_bb381_35, &phi_bb381_36, &phi_bb381_37, &phi_bb381_48);
    std::tie(tmp770, tmp771) = NewReference_intptr_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp473}, TNode<IntPtrT>{phi_bb381_29}).Flatten();
    tmp772 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp773 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb381_29}, TNode<IntPtrT>{tmp772});
    tmp774 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp775 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp773}, TNode<IntPtrT>{tmp774});
    tmp776 = FromConstexpr_bool_constexpr_bool_0(state_, true);
    ca_.Goto(&block376, phi_bb381_20, phi_bb381_26, phi_bb381_27, phi_bb381_28, tmp775, tmp773, tmp776, phi_bb381_35, phi_bb381_36, phi_bb381_37, phi_bb381_48, tmp770, tmp771);
  }

  TNode<IntPtrT> phi_bb376_20;
  TNode<IntPtrT> phi_bb376_26;
  TNode<IntPtrT> phi_bb376_27;
  TNode<IntPtrT> phi_bb376_28;
  TNode<IntPtrT> phi_bb376_29;
  TNode<IntPtrT> phi_bb376_32;
  TNode<BoolT> phi_bb376_33;
  TNode<IntPtrT> phi_bb376_35;
  TNode<IntPtrT> phi_bb376_36;
  TNode<BoolT> phi_bb376_37;
  TNode<BoolT> phi_bb376_48;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb376_50;
  TNode<IntPtrT> phi_bb376_51;
  if (block376.is_used()) {
    ca_.Bind(&block376, &phi_bb376_20, &phi_bb376_26, &phi_bb376_27, &phi_bb376_28, &phi_bb376_29, &phi_bb376_32, &phi_bb376_33, &phi_bb376_35, &phi_bb376_36, &phi_bb376_37, &phi_bb376_48, &phi_bb376_50, &phi_bb376_51);
    ca_.Goto(&block373, phi_bb376_20, phi_bb376_26, phi_bb376_27, phi_bb376_28, phi_bb376_29, phi_bb376_32, phi_bb376_33, phi_bb376_35, phi_bb376_36, phi_bb376_37, phi_bb376_48, phi_bb376_50, phi_bb376_51);
  }

  TNode<IntPtrT> phi_bb373_20;
  TNode<IntPtrT> phi_bb373_26;
  TNode<IntPtrT> phi_bb373_27;
  TNode<IntPtrT> phi_bb373_28;
  TNode<IntPtrT> phi_bb373_29;
  TNode<IntPtrT> phi_bb373_32;
  TNode<BoolT> phi_bb373_33;
  TNode<IntPtrT> phi_bb373_35;
  TNode<IntPtrT> phi_bb373_36;
  TNode<BoolT> phi_bb373_37;
  TNode<BoolT> phi_bb373_48;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb373_50;
  TNode<IntPtrT> phi_bb373_51;
  TNode<Union<HeapObject, TaggedIndex>> tmp777;
  TNode<IntPtrT> tmp778;
  TNode<IntPtrT> tmp779;
  TNode<UintPtrT> tmp780;
  TNode<UintPtrT> tmp781;
  TNode<BoolT> tmp782;
  if (block373.is_used()) {
    ca_.Bind(&block373, &phi_bb373_20, &phi_bb373_26, &phi_bb373_27, &phi_bb373_28, &phi_bb373_29, &phi_bb373_32, &phi_bb373_33, &phi_bb373_35, &phi_bb373_36, &phi_bb373_37, &phi_bb373_48, &phi_bb373_50, &phi_bb373_51);
    std::tie(tmp777, tmp778, tmp779) = FieldSliceFixedArrayObjects_0(state_, TNode<FixedArray>{phi_bb224_41}).Flatten();
    tmp780 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{phi_bb373_20});
    tmp781 = Convert_uintptr_intptr_0(state_, TNode<IntPtrT>{tmp779});
    tmp782 = CodeStubAssembler(state_).UintPtrLessThan(TNode<UintPtrT>{tmp780}, TNode<UintPtrT>{tmp781});
    ca_.Branch(tmp782, &block386, std::vector<compiler::Node*>{phi_bb373_20, phi_bb373_26, phi_bb373_27, phi_bb373_28, phi_bb373_29, phi_bb373_32, phi_bb373_33, phi_bb373_35, phi_bb373_36, phi_bb373_37, phi_bb373_48, phi_bb373_50, phi_bb373_51, phi_bb373_20, phi_bb373_20, phi_bb373_20, phi_bb373_20}, &block387, std::vector<compiler::Node*>{phi_bb373_20, phi_bb373_26, phi_bb373_27, phi_bb373_28, phi_bb373_29, phi_bb373_32, phi_bb373_33, phi_bb373_35, phi_bb373_36, phi_bb373_37, phi_bb373_48, phi_bb373_50, phi_bb373_51, phi_bb373_20, phi_bb373_20, phi_bb373_20, phi_bb373_20});
  }

  TNode<IntPtrT> phi_bb386_20;
  TNode<IntPtrT> phi_bb386_26;
  TNode<IntPtrT> phi_bb386_27;
  TNode<IntPtrT> phi_bb386_28;
  TNode<IntPtrT> phi_bb386_29;
  TNode<IntPtrT> phi_bb386_32;
  TNode<BoolT> phi_bb386_33;
  TNode<IntPtrT> phi_bb386_35;
  TNode<IntPtrT> phi_bb386_36;
  TNode<BoolT> phi_bb386_37;
  TNode<BoolT> phi_bb386_48;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb386_50;
  TNode<IntPtrT> phi_bb386_51;
  TNode<IntPtrT> phi_bb386_56;
  TNode<IntPtrT> phi_bb386_57;
  TNode<IntPtrT> phi_bb386_61;
  TNode<IntPtrT> phi_bb386_62;
  TNode<IntPtrT> tmp783;
  TNode<IntPtrT> tmp784;
  TNode<Union<HeapObject, TaggedIndex>> tmp785;
  TNode<IntPtrT> tmp786;
  TNode<Object> tmp787;
  TNode<IntPtrT> tmp788;
  if (block386.is_used()) {
    ca_.Bind(&block386, &phi_bb386_20, &phi_bb386_26, &phi_bb386_27, &phi_bb386_28, &phi_bb386_29, &phi_bb386_32, &phi_bb386_33, &phi_bb386_35, &phi_bb386_36, &phi_bb386_37, &phi_bb386_48, &phi_bb386_50, &phi_bb386_51, &phi_bb386_56, &phi_bb386_57, &phi_bb386_61, &phi_bb386_62);
    tmp783 = TimesSizeOf_Object_0(state_, TNode<IntPtrT>{phi_bb386_62});
    tmp784 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp778}, TNode<IntPtrT>{tmp783});
    std::tie(tmp785, tmp786) = NewReference_Object_0(state_, TNode<Union<HeapObject, TaggedIndex>>{tmp777}, TNode<IntPtrT>{tmp784}).Flatten();
    tmp787 = CodeStubAssembler(state_).LoadReference<Object>(CodeStubAssembler::Reference{tmp785, tmp786});
    tmp788 = CodeStubAssembler(state_).BitcastTaggedToWord(TNode<Object>{tmp787});
    CodeStubAssembler(state_).StoreReference<IntPtrT>(CodeStubAssembler::Reference{phi_bb386_50, phi_bb386_51}, tmp788);
    ca_.Goto(&block372, phi_bb386_20, tmp753, phi_bb386_26, phi_bb386_27, phi_bb386_28, phi_bb386_29, phi_bb386_32, phi_bb386_33, phi_bb386_35, phi_bb386_36, phi_bb386_37, phi_bb386_48);
  }

  TNode<IntPtrT> phi_bb387_20;
  TNode<IntPtrT> phi_bb387_26;
  TNode<IntPtrT> phi_bb387_27;
  TNode<IntPtrT> phi_bb387_28;
  TNode<IntPtrT> phi_bb387_29;
  TNode<IntPtrT> phi_bb387_32;
  TNode<BoolT> phi_bb387_33;
  TNode<IntPtrT> phi_bb387_35;
  TNode<IntPtrT> phi_bb387_36;
  TNode<BoolT> phi_bb387_37;
  TNode<BoolT> phi_bb387_48;
  TNode<Union<HeapObject, TaggedIndex>> phi_bb387_50;
  TNode<IntPtrT> phi_bb387_51;
  TNode<IntPtrT> phi_bb387_56;
  TNode<IntPtrT> phi_bb387_57;
  TNode<IntPtrT> phi_bb387_61;
  TNode<IntPtrT> phi_bb387_62;
  if (block387.is_used()) {
    ca_.Bind(&block387, &phi_bb387_20, &phi_bb387_26, &phi_bb387_27, &phi_bb387_28, &phi_bb387_29, &phi_bb387_32, &phi_bb387_33, &phi_bb387_35, &phi_bb387_36, &phi_bb387_37, &phi_bb387_48, &phi_bb387_50, &phi_bb387_51, &phi_bb387_56, &phi_bb387_57, &phi_bb387_61, &phi_bb387_62);
    CodeStubAssembler(state_).Unreachable();
  }

  TNode<IntPtrT> phi_bb372_20;
  TNode<IntPtrT> phi_bb372_25;
  TNode<IntPtrT> phi_bb372_26;
  TNode<IntPtrT> phi_bb372_27;
  TNode<IntPtrT> phi_bb372_28;
  TNode<IntPtrT> phi_bb372_29;
  TNode<IntPtrT> phi_bb372_32;
  TNode<BoolT> phi_bb372_33;
  TNode<IntPtrT> phi_bb372_35;
  TNode<IntPtrT> phi_bb372_36;
  TNode<BoolT> phi_bb372_37;
  TNode<BoolT> phi_bb372_48;
  TNode<IntPtrT> tmp789;
  TNode<IntPtrT> tmp790;
  if (block372.is_used()) {
    ca_.Bind(&block372, &phi_bb372_20, &phi_bb372_25, &phi_bb372_26, &phi_bb372_27, &phi_bb372_28, &phi_bb372_29, &phi_bb372_32, &phi_bb372_33, &phi_bb372_35, &phi_bb372_36, &phi_bb372_37, &phi_bb372_48);
    tmp789 = FromConstexpr_intptr_constexpr_int31_0(state_, 1);
    tmp790 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{phi_bb372_20}, TNode<IntPtrT>{tmp789});
    ca_.Goto(&block362, tmp790, phi_bb372_25, phi_bb372_26, phi_bb372_27, phi_bb372_28, phi_bb372_29, phi_bb372_32, phi_bb372_33, phi_bb372_35, phi_bb372_36, phi_bb372_37, tmp746, phi_bb372_48);
  }

  TNode<IntPtrT> phi_bb361_20;
  TNode<IntPtrT> phi_bb361_25;
  TNode<IntPtrT> phi_bb361_26;
  TNode<IntPtrT> phi_bb361_27;
  TNode<IntPtrT> phi_bb361_28;
  TNode<IntPtrT> phi_bb361_29;
  TNode<IntPtrT> phi_bb361_32;
  TNode<BoolT> phi_bb361_33;
  TNode<IntPtrT> phi_bb361_35;
  TNode<IntPtrT> phi_bb361_36;
  TNode<BoolT> phi_bb361_37;
  TNode<IntPtrT> phi_bb361_46;
  TNode<BoolT> phi_bb361_48;
  if (block361.is_used()) {
    ca_.Bind(&block361, &phi_bb361_20, &phi_bb361_25, &phi_bb361_26, &phi_bb361_27, &phi_bb361_28, &phi_bb361_29, &phi_bb361_32, &phi_bb361_33, &phi_bb361_35, &phi_bb361_36, &phi_bb361_37, &phi_bb361_46, &phi_bb361_48);
    ca_.Goto(&block355, phi_bb361_20, phi_bb361_25, phi_bb361_26, phi_bb361_27, phi_bb361_28, phi_bb361_29, phi_bb361_32, phi_bb361_33, phi_bb361_35, phi_bb361_36, phi_bb361_37, phi_bb361_46, tmp739, phi_bb361_48);
  }

  TNode<IntPtrT> phi_bb355_20;
  TNode<IntPtrT> phi_bb355_25;
  TNode<IntPtrT> phi_bb355_26;
  TNode<IntPtrT> phi_bb355_27;
  TNode<IntPtrT> phi_bb355_28;
  TNode<IntPtrT> phi_bb355_29;
  TNode<IntPtrT> phi_bb355_32;
  TNode<BoolT> phi_bb355_33;
  TNode<IntPtrT> phi_bb355_35;
  TNode<IntPtrT> phi_bb355_36;
  TNode<BoolT> phi_bb355_37;
  TNode<IntPtrT> phi_bb355_46;
  TNode<IntPtrT> phi_bb355_47;
  TNode<BoolT> phi_bb355_48;
  TNode<IntPtrT> tmp791;
  TNode<IntPtrT> tmp792;
  TNode<IntPtrT> tmp793;
  TNode<IntPtrT> tmp794;
  TNode<IntPtrT> tmp795;
  TNode<IntPtrT> tmp796;
  TNode<IntPtrT> tmp797;
  TNode<Union<HeapObject, TaggedIndex>> tmp798;
  TNode<IntPtrT> tmp799;
  TNode<IntPtrT> tmp800;
  TNode<IntPtrT> tmp801;
  TNode<Union<HeapObject, TaggedIndex>> tmp802;
  TNode<IntPtrT> tmp803;
  TNode<IntPtrT> tmp804;
  TNode<IntPtrT> tmp805;
  TNode<Union<HeapObject, TaggedIndex>> tmp806;
  TNode<IntPtrT> tmp807;
  TNode<Float64T> tmp808;
  TNode<IntPtrT> tmp809;
  TNode<Union<HeapObject, TaggedIndex>> tmp810;
  TNode<IntPtrT> tmp811;
  TNode<Float64T> tmp812;
  if (block355.is_used()) {
    ca_.Bind(&block355, &phi_bb355_20, &phi_bb355_25, &phi_bb355_26, &phi_bb355_27, &phi_bb355_28, &phi_bb355_29, &phi_bb355_32, &phi_bb355_33, &phi_bb355_35, &phi_bb355_36, &phi_bb355_37, &phi_bb355_46, &phi_bb355_47, &phi_bb355_48);
    tmp791 = Convert_intptr_RawPtr_0(state_, TNode<RawPtrT>{tmp472});
    tmp792 = Convert_intptr_RawPtr_0(state_, TNode<RawPtrT>{tmp83});
    tmp793 = CodeStubAssembler(state_).IntPtrSub(TNode<IntPtrT>{tmp791}, TNode<IntPtrT>{tmp792});
    tmp794 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    tmp795 = CodeStubAssembler(state_).IntPtrDiv(TNode<IntPtrT>{tmp793}, TNode<IntPtrT>{tmp794});
    tmp796 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp795}, TNode<IntPtrT>{tmp11});
    tmp797 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp798, tmp799) = GetRefAt_intptr_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp459}, TNode<IntPtrT>{tmp797}).Flatten();
    tmp800 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp798, tmp799});
    tmp801 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_intptr_0(state_)));
    std::tie(tmp802, tmp803) = GetRefAt_intptr_RawPtr_intptr_0(state_, TNode<RawPtrT>{tmp459}, TNode<IntPtrT>{tmp801}).Flatten();
    tmp804 = CodeStubAssembler(state_).LoadReference<IntPtrT>(CodeStubAssembler::Reference{tmp802, tmp803});
    tmp805 = FromConstexpr_intptr_constexpr_IntegerLiteral_0(state_, IntegerLiteral(false, 0x0ull));
    std::tie(tmp806, tmp807) = GetRefAt_float64_RawPtr_float64_0(state_, TNode<RawPtrT>{tmp461}, TNode<IntPtrT>{tmp805}).Flatten();
    tmp808 = CodeStubAssembler(state_).LoadReference<Float64T>(CodeStubAssembler::Reference{tmp806, tmp807});
    tmp809 = FromConstexpr_intptr_constexpr_int31_0(state_, (SizeOf_float64_0(state_)));
    std::tie(tmp810, tmp811) = GetRefAt_float64_RawPtr_float64_0(state_, TNode<RawPtrT>{tmp461}, TNode<IntPtrT>{tmp809}).Flatten();
    tmp812 = CodeStubAssembler(state_).LoadReference<Float64T>(CodeStubAssembler::Reference{tmp810, tmp811});
    CodeStubAssembler(state_).SwitchFromTheCentralStack(TNode<RawPtrT>{tmp0});
    ca_.Goto(&block390);
  }

    ca_.Bind(&block390);
  return TorqueStructWasmToJSResult{TNode<IntPtrT>{tmp796}, TNode<IntPtrT>{tmp800}, TNode<IntPtrT>{tmp804}, TNode<Float64T>{tmp808}, TNode<Float64T>{tmp812}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=33&c=8
TorqueStructReference_float64_0 RefCast_float64_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_i) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = NewReference_float64_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_i.object}, TNode<IntPtrT>{p_i.offset}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_float64_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=35&c=8
TorqueStructReference_float32_0 RefCast_float32_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_i) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = NewReference_float32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_i.object}, TNode<IntPtrT>{p_i.offset}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_float32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=53&c=10
TNode<BoolT> Is_WasmImportData_WasmImportData_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<WasmImportData> p_o) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block5(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block4(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<BoolT> block1(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block6(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<WasmImportData> tmp0;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    compiler::CodeAssemblerLabel label1(&ca_);
    tmp0 = Cast_WasmImportData_0(state_, TNode<HeapObject>{p_o}, &label1);
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

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=58&c=4
TorqueStructReference_RawPtr_0 GetRefAt_RawPtr_RawPtr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
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

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=60&c=33
int31_t SizeOf_intptr_0(compiler::CodeAssemblerState* state_) {
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
  return kIntptrSize;
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=73&c=24
TorqueStructReference_intptr_0 NewOffHeapReference_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_ptr) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<TaggedIndex> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Union<HeapObject, TaggedIndex>> tmp5;
  TNode<IntPtrT> tmp6;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = kZeroBitPattern_0(state_);
    tmp1 = Convert_RawPtr_RawPtr_intptr_0(state_, TNode<RawPtrT>{p_ptr});
    tmp2 = Convert_intptr_RawPtr_0(state_, TNode<RawPtrT>{tmp1});
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, kHeapObjectTag);
    tmp4 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp2}, TNode<IntPtrT>{tmp3});
    std::tie(tmp5, tmp6) = (TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp4}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_intptr_0{TNode<Union<HeapObject, TaggedIndex>>{tmp5}, TNode<IntPtrT>{tmp6}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=78&c=30
TorqueStructReference_RawPtr_uint32_0 NewOffHeapReference_RawPtr_uint32_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_ptr) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<TaggedIndex> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Union<HeapObject, TaggedIndex>> tmp5;
  TNode<IntPtrT> tmp6;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = kZeroBitPattern_0(state_);
    tmp1 = Convert_RawPtr_RawPtr_RawPtr_uint32_0(state_, TNode<RawPtrT>{p_ptr});
    tmp2 = Convert_intptr_RawPtr_0(state_, TNode<RawPtrT>{tmp1});
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, kHeapObjectTag);
    tmp4 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp2}, TNode<IntPtrT>{tmp3});
    std::tie(tmp5, tmp6) = (TorqueStructReference_RawPtr_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp4}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_RawPtr_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp5}, TNode<IntPtrT>{tmp6}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=82&c=7
TorqueStructSlice_uint32_ConstReference_uint32_0 NewOffHeapConstSlice_uint32_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_startPointer, TNode<IntPtrT> p_length) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<TaggedIndex> tmp0;
  TNode<RawPtrT> tmp1;
  TNode<IntPtrT> tmp2;
  TNode<IntPtrT> tmp3;
  TNode<IntPtrT> tmp4;
  TNode<Union<HeapObject, TaggedIndex>> tmp5;
  TNode<IntPtrT> tmp6;
  TNode<IntPtrT> tmp7;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    tmp0 = kZeroBitPattern_0(state_);
    tmp1 = Convert_RawPtr_RawPtr_uint32_0(state_, TNode<RawPtrT>{p_startPointer});
    tmp2 = Convert_intptr_RawPtr_0(state_, TNode<RawPtrT>{tmp1});
    tmp3 = FromConstexpr_intptr_constexpr_int31_0(state_, kHeapObjectTag);
    tmp4 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{tmp2}, TNode<IntPtrT>{tmp3});
    std::tie(tmp5, tmp6, tmp7) = (TorqueStructSlice_uint32_ConstReference_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp4}, TNode<IntPtrT>{p_length}, TorqueStructUnsafe_0{}}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructSlice_uint32_ConstReference_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp5}, TNode<IntPtrT>{tmp6}, TNode<IntPtrT>{tmp7}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=85&c=7
TorqueStructSlice_uint32_ConstReference_uint32_0 Subslice_uint32_0(compiler::CodeAssemblerState* state_, TorqueStructSlice_uint32_ConstReference_uint32_0 p_slice, TNode<IntPtrT> p_start, TNode<IntPtrT> p_length, compiler::CodeAssemblerLabel* label_OutOfBounds) {
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
    tmp7 = TimesSizeOf_uint32_0(state_, TNode<IntPtrT>{p_start});
    tmp8 = CodeStubAssembler(state_).IntPtrAdd(TNode<IntPtrT>{p_slice.offset}, TNode<IntPtrT>{tmp7});
    std::tie(tmp9, tmp10, tmp11) = NewConstSlice_uint32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_slice.object}, TNode<IntPtrT>{tmp8}, TNode<IntPtrT>{p_length}).Flatten();
    ca_.Goto(&block7);
  }

  if (block1.is_used()) {
    ca_.Bind(&block1);
    ca_.Goto(label_OutOfBounds);
  }

    ca_.Bind(&block7);
  return TorqueStructSlice_uint32_ConstReference_uint32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp9}, TNode<IntPtrT>{tmp10}, TNode<IntPtrT>{tmp11}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=116&c=37
TorqueStructReference_int64_0 RefCast_int64_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_i) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = NewReference_int64_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_i.object}, TNode<IntPtrT>{p_i.offset}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_int64_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=118&c=16
TorqueStructReference_int32_0 RefCast_int32_0(compiler::CodeAssemblerState* state_, TorqueStructReference_intptr_0 p_i) {
  compiler::CodeAssembler ca_(state_);
  compiler::CodeAssembler::SourcePositionScope pos_scope(&ca_);
  compiler::CodeAssemblerParameterizedLabel<> block0(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
  compiler::CodeAssemblerParameterizedLabel<> block2(&ca_, compiler::CodeAssemblerLabel::kNonDeferred);
    ca_.Goto(&block0);

  TNode<Union<HeapObject, TaggedIndex>> tmp0;
  TNode<IntPtrT> tmp1;
  if (block0.is_used()) {
    ca_.Bind(&block0);
    std::tie(tmp0, tmp1) = NewReference_int32_0(state_, TNode<Union<HeapObject, TaggedIndex>>{p_i.object}, TNode<IntPtrT>{p_i.offset}).Flatten();
    ca_.Goto(&block2);
  }

    ca_.Bind(&block2);
  return TorqueStructReference_int32_0{TNode<Union<HeapObject, TaggedIndex>>{tmp0}, TNode<IntPtrT>{tmp1}, TorqueStructUnsafe_0{}};
}

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=186&c=4
TorqueStructReference_intptr_0 GetRefAt_intptr_RawPtr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
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

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=303&c=15
TorqueStructReference_intptr_0 GetRefAt_intptr_RawPtr_intptr_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
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

// https://crsrc.org/c/v8/src/builtins/wasm-to-js.tq?l=305&c=15
TorqueStructReference_float64_0 GetRefAt_float64_RawPtr_float64_0(compiler::CodeAssemblerState* state_, TNode<RawPtrT> p_base, TNode<IntPtrT> p_offset) {
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

} // namespace internal
} // namespace v8
