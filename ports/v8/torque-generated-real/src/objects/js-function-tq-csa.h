#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_FUNCTION_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_FUNCTION_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=5&c=1
TNode<JSFunctionOrBoundFunctionOrWrappedFunction> Cast_JSFunctionOrBoundFunctionOrWrappedFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=8&c=1
TNode<JSBoundFunction> Cast_JSBoundFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=20&c=1
TNode<JSWrappedFunction> Cast_JSWrappedFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=32&c=1
TNode<JSFunction> Cast_JSFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=52&c=1
TNode<JSFunctionWithoutPrototype> Cast_JSFunctionWithoutPrototype_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=55&c=1
TNode<JSFunctionWithPrototype> Cast_JSFunctionWithPrototype_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=34&c=3
TNode<Int32T> LoadJSFunctionDispatchHandle_0(compiler::CodeAssemblerState* state_, TNode<JSFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=34&c=3
void StoreJSFunctionDispatchHandle_0(compiler::CodeAssemblerState* state_, TNode<JSFunction> p_o, TNode<Int32T> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=36&c=3
TNode<SharedFunctionInfo> LoadJSFunctionSharedFunctionInfo_0(compiler::CodeAssemblerState* state_, TNode<JSFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=36&c=3
void StoreJSFunctionSharedFunctionInfo_0(compiler::CodeAssemblerState* state_, TNode<JSFunction> p_o, TNode<SharedFunctionInfo> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=37&c=3
TNode<Context> LoadJSFunctionContext_0(compiler::CodeAssemblerState* state_, TNode<JSFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=37&c=3
void StoreJSFunctionContext_0(compiler::CodeAssemblerState* state_, TNode<JSFunction> p_o, TNode<Context> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=38&c=3
TNode<FeedbackCell> LoadJSFunctionFeedbackCell_0(compiler::CodeAssemblerState* state_, TNode<JSFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=38&c=3
void StoreJSFunctionFeedbackCell_0(compiler::CodeAssemblerState* state_, TNode<JSFunction> p_o, TNode<FeedbackCell> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=57&c=3
TNode<Union<JSReceiver, Map, TheHole>> LoadJSFunctionWithPrototypePrototypeOrInitialMap_0(compiler::CodeAssemblerState* state_, TNode<JSFunctionWithPrototype> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=57&c=3
void StoreJSFunctionWithPrototypePrototypeOrInitialMap_0(compiler::CodeAssemblerState* state_, TNode<JSFunctionWithPrototype> p_o, TNode<Union<JSReceiver, Map, TheHole>> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=11&c=3
TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> LoadJSBoundFunctionBoundTargetFunction_0(compiler::CodeAssemblerState* state_, TNode<JSBoundFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=11&c=3
void StoreJSBoundFunctionBoundTargetFunction_0(compiler::CodeAssemblerState* state_, TNode<JSBoundFunction> p_o, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=14&c=3
TNode<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, SourceTextModule, String, Symbol, Undefined>> LoadJSBoundFunctionBoundThis_0(compiler::CodeAssemblerState* state_, TNode<JSBoundFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=14&c=3
void StoreJSBoundFunctionBoundThis_0(compiler::CodeAssemblerState* state_, TNode<JSBoundFunction> p_o, TNode<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, SourceTextModule, String, Symbol, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=17&c=3
TNode<FixedArray> LoadJSBoundFunctionBoundArguments_0(compiler::CodeAssemblerState* state_, TNode<JSBoundFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=17&c=3
void StoreJSBoundFunctionBoundArguments_0(compiler::CodeAssemblerState* state_, TNode<JSBoundFunction> p_o, TNode<FixedArray> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=23&c=3
TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> LoadJSWrappedFunctionWrappedTargetFunction_0(compiler::CodeAssemblerState* state_, TNode<JSWrappedFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=23&c=3
void StoreJSWrappedFunctionWrappedTargetFunction_0(compiler::CodeAssemblerState* state_, TNode<JSWrappedFunction> p_o, TNode<Union<JSBoundFunction, JSFunction, JSObject, JSProxy, JSWrappedFunction>> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=25&c=3
TNode<NativeContext> LoadJSWrappedFunctionContext_0(compiler::CodeAssemblerState* state_, TNode<JSWrappedFunction> p_o);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=25&c=3
void StoreJSWrappedFunctionContext_0(compiler::CodeAssemblerState* state_, TNode<JSWrappedFunction> p_o, TNode<NativeContext> p_v);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=5&c=1
TNode<JSFunctionOrBoundFunctionOrWrappedFunction> DownCastForTorqueClass_JSFunctionOrBoundFunctionOrWrappedFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=8&c=1
TNode<JSBoundFunction> DownCastForTorqueClass_JSBoundFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=20&c=1
TNode<JSWrappedFunction> DownCastForTorqueClass_JSWrappedFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=32&c=1
TNode<JSFunction> DownCastForTorqueClass_JSFunction_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=52&c=1
TNode<JSFunctionWithoutPrototype> DownCastForTorqueClass_JSFunctionWithoutPrototype_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=55&c=1
TNode<JSFunctionWithPrototype> DownCastForTorqueClass_JSFunctionWithPrototype_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_FUNCTION_TQ_CSA_H_
