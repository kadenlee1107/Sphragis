#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_OBJECTS_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_OBJECTS_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=6&c=1
TNode<JSReceiver> Cast_JSReceiver_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=14&c=1
TNode<JSObject> Cast_JSObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=36&c=1
TNode<JSObject> NewJSObject_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=43&c=1
TNode<JSExternalObject> Cast_JSExternalObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=50&c=1
TNode<JSObjectWithEmbedderSlots> Cast_JSObjectWithEmbedderSlots_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=54&c=1
TNode<JSAPIObjectWithEmbedderSlots> Cast_JSAPIObjectWithEmbedderSlots_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=59&c=1
TNode<JSCustomElementsObject> Cast_JSCustomElementsObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=66&c=1
TNode<JSSpecialObject> Cast_JSSpecialObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=73&c=1
TNode<Map> GetDerivedMap_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p_target, TNode<JSReceiver> p_newTarget);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=93&c=1
TNode<Map> GetDerivedRabGsabTypedArrayMap_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<JSFunction> p_target, TNode<JSReceiver> p_newTarget);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=99&c=1
TNode<JSObject> AllocateFastOrSlowJSObjectFromMap_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Map> p_map);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=117&c=1
TNode<JSGlobalProxy> Cast_JSGlobalProxy_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=119&c=1
TNode<JSGlobalObject> Cast_JSGlobalObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=145&c=1
TNode<JSPrimitiveWrapper> Cast_JSPrimitiveWrapper_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=149&c=1
TNode<JSMessageObject> Cast_JSMessageObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=168&c=1
TNode<JSDate> Cast_JSDate_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=188&c=1
TNode<JSAsyncFromSyncIterator> Cast_JSAsyncFromSyncIterator_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=195&c=1
TNode<JSStringIterator> Cast_JSStringIterator_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=204&c=1
TNode<JSValidIteratorWrapper> Cast_JSValidIteratorWrapper_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=9&c=3
TNode<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>> LoadJSReceiverPropertiesOrHash_0(compiler::CodeAssemblerState* state_, TNode<JSReceiver> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=9&c=3
void StoreJSReceiverPropertiesOrHash_0(compiler::CodeAssemblerState* state_, TNode<JSReceiver> p_o, TNode<Union<FixedArrayBase, PropertyArray, Smi, SwissNameDictionary>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=33&c=3
TNode<FixedArrayBase> LoadJSObjectElements_0(compiler::CodeAssemblerState* state_, TNode<JSObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=33&c=3
void StoreJSObjectElements_0(compiler::CodeAssemblerState* state_, TNode<JSObject> p_o, TNode<FixedArrayBase> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=56&c=3
TNode<CppHeapPointerT> LoadJSAPIObjectWithEmbedderSlotsCppHeapWrappable_0(compiler::CodeAssemblerState* state_, TNode<JSAPIObjectWithEmbedderSlots> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=56&c=3
void StoreJSAPIObjectWithEmbedderSlotsCppHeapWrappable_0(compiler::CodeAssemblerState* state_, TNode<JSAPIObjectWithEmbedderSlots> p_o, TNode<CppHeapPointerT> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=70&c=3
TNode<CppHeapPointerT> LoadJSSpecialObjectCppHeapWrappable_0(compiler::CodeAssemblerState* state_, TNode<JSSpecialObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=70&c=3
void StoreJSSpecialObjectCppHeapWrappable_0(compiler::CodeAssemblerState* state_, TNode<JSSpecialObject> p_o, TNode<CppHeapPointerT> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=44&c=3
TNode<ExternalPointerT> LoadJSExternalObjectValue_0(compiler::CodeAssemblerState* state_, TNode<JSExternalObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=44&c=3
void StoreJSExternalObjectValue_0(compiler::CodeAssemblerState* state_, TNode<JSExternalObject> p_o, TNode<ExternalPointerT> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=128&c=3
TNode<JSGlobalProxy> LoadJSGlobalObjectGlobalProxy_0(compiler::CodeAssemblerState* state_, TNode<JSGlobalObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=128&c=3
void StoreJSGlobalObjectGlobalProxy_0(compiler::CodeAssemblerState* state_, TNode<JSGlobalObject> p_o, TNode<JSGlobalProxy> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=142&c=3
TNode<JSGlobalProxy> LoadJSGlobalObjectGlobalProxyForApi_0(compiler::CodeAssemblerState* state_, TNode<JSGlobalObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=142&c=3
void StoreJSGlobalObjectGlobalProxyForApi_0(compiler::CodeAssemblerState* state_, TNode<JSGlobalObject> p_o, TNode<JSGlobalProxy> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=146&c=3
TNode<JSAny> LoadJSPrimitiveWrapperValue_0(compiler::CodeAssemblerState* state_, TNode<JSPrimitiveWrapper> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=146&c=3
void StoreJSPrimitiveWrapperValue_0(compiler::CodeAssemblerState* state_, TNode<JSPrimitiveWrapper> p_o, TNode<JSAny> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=151&c=3
TNode<Smi> LoadJSMessageObjectMessageType_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=151&c=3
void StoreJSMessageObjectMessageType_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=153&c=3
TNode<Object> LoadJSMessageObjectArgument_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=153&c=3
void StoreJSMessageObjectArgument_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Object> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=155&c=3
TNode<Script> LoadJSMessageObjectScript_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=155&c=3
void StoreJSMessageObjectScript_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Script> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=157&c=3
TNode<Union<StackTraceInfo, TheHole>> LoadJSMessageObjectStackTrace_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=157&c=3
void StoreJSMessageObjectStackTrace_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Union<StackTraceInfo, TheHole>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=158&c=3
TNode<Union<SharedFunctionInfo, Smi>> LoadJSMessageObjectSharedInfo_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=158&c=3
void StoreJSMessageObjectSharedInfo_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Union<SharedFunctionInfo, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=162&c=3
TNode<Smi> LoadJSMessageObjectBytecodeOffset_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=162&c=3
void StoreJSMessageObjectBytecodeOffset_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=163&c=3
TNode<Smi> LoadJSMessageObjectStartPosition_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=163&c=3
void StoreJSMessageObjectStartPosition_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=164&c=3
TNode<Smi> LoadJSMessageObjectEndPosition_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=164&c=3
void StoreJSMessageObjectEndPosition_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=165&c=3
TNode<Smi> LoadJSMessageObjectErrorLevel_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=165&c=3
void StoreJSMessageObjectErrorLevel_0(compiler::CodeAssemblerState* state_, TNode<JSMessageObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=172&c=3
TNode<Float64T> LoadJSDateValue_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=172&c=3
void StoreJSDateValue_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Float64T> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=175&c=3
TNode<Union<HeapNumber, Smi>> LoadJSDateYear_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=175&c=3
void StoreJSDateYear_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Union<HeapNumber, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=176&c=3
TNode<Union<HeapNumber, Smi>> LoadJSDateMonth_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=176&c=3
void StoreJSDateMonth_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Union<HeapNumber, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=177&c=3
TNode<Union<HeapNumber, Smi>> LoadJSDateDay_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=177&c=3
void StoreJSDateDay_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Union<HeapNumber, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=178&c=3
TNode<Union<HeapNumber, Smi>> LoadJSDateWeekday_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=178&c=3
void StoreJSDateWeekday_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Union<HeapNumber, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=179&c=3
TNode<Union<HeapNumber, Smi>> LoadJSDateHour_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=179&c=3
void StoreJSDateHour_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Union<HeapNumber, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=180&c=3
TNode<Union<HeapNumber, Smi>> LoadJSDateMin_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=180&c=3
void StoreJSDateMin_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Union<HeapNumber, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=181&c=3
TNode<Union<HeapNumber, Smi>> LoadJSDateSec_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=181&c=3
void StoreJSDateSec_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Union<HeapNumber, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=185&c=3
TNode<Union<HeapNumber, Smi>> LoadJSDateCacheStamp_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=185&c=3
void StoreJSDateCacheStamp_0(compiler::CodeAssemblerState* state_, TNode<JSDate> p_o, TNode<Union<HeapNumber, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=189&c=3
TNode<JSReceiver> LoadJSAsyncFromSyncIteratorSyncIterator_0(compiler::CodeAssemblerState* state_, TNode<JSAsyncFromSyncIterator> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=189&c=3
void StoreJSAsyncFromSyncIteratorSyncIterator_0(compiler::CodeAssemblerState* state_, TNode<JSAsyncFromSyncIterator> p_o, TNode<JSReceiver> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=192&c=3
TNode<Object> LoadJSAsyncFromSyncIteratorNext_0(compiler::CodeAssemblerState* state_, TNode<JSAsyncFromSyncIterator> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=192&c=3
void StoreJSAsyncFromSyncIteratorNext_0(compiler::CodeAssemblerState* state_, TNode<JSAsyncFromSyncIterator> p_o, TNode<Object> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=197&c=3
TNode<String> LoadJSStringIteratorString_0(compiler::CodeAssemblerState* state_, TNode<JSStringIterator> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=197&c=3
void StoreJSStringIteratorString_0(compiler::CodeAssemblerState* state_, TNode<JSStringIterator> p_o, TNode<String> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=199&c=3
TNode<Smi> LoadJSStringIteratorIndex_0(compiler::CodeAssemblerState* state_, TNode<JSStringIterator> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=199&c=3
void StoreJSStringIteratorIndex_0(compiler::CodeAssemblerState* state_, TNode<JSStringIterator> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=206&c=3
TorqueStructIteratorRecord LoadJSValidIteratorWrapperUnderlying_0(compiler::CodeAssemblerState* state_, TNode<JSValidIteratorWrapper> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=206&c=3
void StoreJSValidIteratorWrapperUnderlying_0(compiler::CodeAssemblerState* state_, TNode<JSValidIteratorWrapper> p_o, TorqueStructIteratorRecord p_v);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=6&c=1
TNode<JSReceiver> DownCastForTorqueClass_JSReceiver_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=14&c=1
TNode<JSObject> DownCastForTorqueClass_JSObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=43&c=1
TNode<JSExternalObject> DownCastForTorqueClass_JSExternalObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=50&c=1
TNode<JSObjectWithEmbedderSlots> DownCastForTorqueClass_JSObjectWithEmbedderSlots_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=54&c=1
TNode<JSAPIObjectWithEmbedderSlots> DownCastForTorqueClass_JSAPIObjectWithEmbedderSlots_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=59&c=1
TNode<JSCustomElementsObject> DownCastForTorqueClass_JSCustomElementsObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=66&c=1
TNode<JSSpecialObject> DownCastForTorqueClass_JSSpecialObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=80&c=9
TNode<BoolT> Is_TheHole_JSReceiver_OR_Map_OR_TheHole_0(compiler::CodeAssemblerState* state_, TNode<Context> p_context, TNode<Union<JSReceiver, Map, TheHole>> p_o);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=117&c=1
TNode<JSGlobalProxy> DownCastForTorqueClass_JSGlobalProxy_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=119&c=1
TNode<JSGlobalObject> DownCastForTorqueClass_JSGlobalObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=145&c=1
TNode<JSPrimitiveWrapper> DownCastForTorqueClass_JSPrimitiveWrapper_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=149&c=1
TNode<JSMessageObject> DownCastForTorqueClass_JSMessageObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=168&c=1
TNode<JSDate> DownCastForTorqueClass_JSDate_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=188&c=1
TNode<JSAsyncFromSyncIterator> DownCastForTorqueClass_JSAsyncFromSyncIterator_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=195&c=1
TNode<JSStringIterator> DownCastForTorqueClass_JSStringIterator_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=204&c=1
TNode<JSValidIteratorWrapper> DownCastForTorqueClass_JSValidIteratorWrapper_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_JS_OBJECTS_TQ_CSA_H_
