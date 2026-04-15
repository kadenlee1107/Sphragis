#include "src/objects/js-function-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-function.tq?l=5&c=1
bool IsJSFunctionOrBoundFunctionOrWrappedFunction_NonInline(Tagged<HeapObject> o) {
  return IsJSFunctionOrBoundFunctionOrWrappedFunction(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSFunctionOrBoundFunctionOrWrappedFunction<JSFunctionOrBoundFunctionOrWrappedFunction, JSObject>::JSFunctionOrBoundFunctionOrWrappedFunctionVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSFunctionOrBoundFunctionOrWrappedFunctionVerify(TrustedCast<JSFunctionOrBoundFunctionOrWrappedFunction>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-function.tq?l=32&c=1
bool IsJSFunction_NonInline(Tagged<HeapObject> o) {
  return IsJSFunction(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSFunction<JSFunction, JSFunctionOrBoundFunctionOrWrappedFunction>::JSFunctionVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSFunctionVerify(TrustedCast<JSFunction>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-function.tq?l=55&c=1
bool IsJSFunctionWithPrototype_NonInline(Tagged<HeapObject> o) {
  return IsJSFunctionWithPrototype(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSFunctionWithPrototype<JSFunctionWithPrototype, JSFunction>::JSFunctionWithPrototypeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSFunctionWithPrototypeVerify(TrustedCast<JSFunctionWithPrototype>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-function.tq?l=8&c=1
bool IsJSBoundFunction_NonInline(Tagged<HeapObject> o) {
  return IsJSBoundFunction(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSBoundFunction<JSBoundFunction, JSFunctionOrBoundFunctionOrWrappedFunction>::JSBoundFunctionVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSBoundFunctionVerify(TrustedCast<JSBoundFunction>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-function.tq?l=20&c=1
bool IsJSWrappedFunction_NonInline(Tagged<HeapObject> o) {
  return IsJSWrappedFunction(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSWrappedFunction<JSWrappedFunction, JSFunctionOrBoundFunctionOrWrappedFunction>::JSWrappedFunctionVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSWrappedFunctionVerify(TrustedCast<JSWrappedFunction>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-function.tq?l=52&c=1
bool IsJSFunctionWithoutPrototype_NonInline(Tagged<HeapObject> o) {
  return IsJSFunctionWithoutPrototype(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSFunctionWithoutPrototype<JSFunctionWithoutPrototype, JSFunction>::JSFunctionWithoutPrototypeVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSFunctionWithoutPrototypeVerify(TrustedCast<JSFunctionWithoutPrototype>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
