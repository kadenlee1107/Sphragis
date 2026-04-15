#include "src/objects/js-objects-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=6&c=1
bool IsJSReceiver_NonInline(Tagged<HeapObject> o) {
  return IsJSReceiver(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSReceiver<JSReceiver, HeapObject>::JSReceiverVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSReceiverVerify(TrustedCast<JSReceiver>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=14&c=1
bool IsJSObject_NonInline(Tagged<HeapObject> o) {
  return IsJSObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSObject<JSObject, JSReceiver>::JSObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSObjectVerify(TrustedCast<JSObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=50&c=1
bool IsJSObjectWithEmbedderSlots_NonInline(Tagged<HeapObject> o) {
  return IsJSObjectWithEmbedderSlots(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSObjectWithEmbedderSlots<JSObjectWithEmbedderSlots, JSObject>::JSObjectWithEmbedderSlotsVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSObjectWithEmbedderSlotsVerify(TrustedCast<JSObjectWithEmbedderSlots>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=54&c=1
bool IsJSAPIObjectWithEmbedderSlots_NonInline(Tagged<HeapObject> o) {
  return IsJSAPIObjectWithEmbedderSlots(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSAPIObjectWithEmbedderSlots<JSAPIObjectWithEmbedderSlots, JSObject>::JSAPIObjectWithEmbedderSlotsVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSAPIObjectWithEmbedderSlotsVerify(TrustedCast<JSAPIObjectWithEmbedderSlots>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=59&c=1
bool IsJSCustomElementsObject_NonInline(Tagged<HeapObject> o) {
  return IsJSCustomElementsObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSCustomElementsObject<JSCustomElementsObject, JSObject>::JSCustomElementsObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSCustomElementsObjectVerify(TrustedCast<JSCustomElementsObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=66&c=1
bool IsJSSpecialObject_NonInline(Tagged<HeapObject> o) {
  return IsJSSpecialObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSSpecialObject<JSSpecialObject, JSCustomElementsObject>::JSSpecialObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSSpecialObjectVerify(TrustedCast<JSSpecialObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=43&c=1
bool IsJSExternalObject_NonInline(Tagged<HeapObject> o) {
  return IsJSExternalObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSExternalObject<JSExternalObject, JSObject>::JSExternalObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSExternalObjectVerify(TrustedCast<JSExternalObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=117&c=1
bool IsJSGlobalProxy_NonInline(Tagged<HeapObject> o) {
  return IsJSGlobalProxy(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSGlobalProxy<JSGlobalProxy, JSSpecialObject>::JSGlobalProxyVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSGlobalProxyVerify(TrustedCast<JSGlobalProxy>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=119&c=1
bool IsJSGlobalObject_NonInline(Tagged<HeapObject> o) {
  return IsJSGlobalObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSGlobalObject<JSGlobalObject, JSSpecialObject>::JSGlobalObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSGlobalObjectVerify(TrustedCast<JSGlobalObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=145&c=1
bool IsJSPrimitiveWrapper_NonInline(Tagged<HeapObject> o) {
  return IsJSPrimitiveWrapper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSPrimitiveWrapper<JSPrimitiveWrapper, JSCustomElementsObject>::JSPrimitiveWrapperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSPrimitiveWrapperVerify(TrustedCast<JSPrimitiveWrapper>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=149&c=1
bool IsJSMessageObject_NonInline(Tagged<HeapObject> o) {
  return IsJSMessageObject(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSMessageObject<JSMessageObject, JSObject>::JSMessageObjectVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSMessageObjectVerify(TrustedCast<JSMessageObject>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=168&c=1
bool IsJSDate_NonInline(Tagged<HeapObject> o) {
  return IsJSDate(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSDate<JSDate, JSObject>::JSDateVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSDateVerify(TrustedCast<JSDate>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=188&c=1
bool IsJSAsyncFromSyncIterator_NonInline(Tagged<HeapObject> o) {
  return IsJSAsyncFromSyncIterator(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSAsyncFromSyncIterator<JSAsyncFromSyncIterator, JSObject>::JSAsyncFromSyncIteratorVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSAsyncFromSyncIteratorVerify(TrustedCast<JSAsyncFromSyncIterator>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=195&c=1
bool IsJSStringIterator_NonInline(Tagged<HeapObject> o) {
  return IsJSStringIterator(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSStringIterator<JSStringIterator, JSObject>::JSStringIteratorVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSStringIteratorVerify(TrustedCast<JSStringIterator>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-objects.tq?l=204&c=1
bool IsJSValidIteratorWrapper_NonInline(Tagged<HeapObject> o) {
  return IsJSValidIteratorWrapper(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSValidIteratorWrapper<JSValidIteratorWrapper, JSObject>::JSValidIteratorWrapperVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSValidIteratorWrapperVerify(TrustedCast<JSValidIteratorWrapper>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
