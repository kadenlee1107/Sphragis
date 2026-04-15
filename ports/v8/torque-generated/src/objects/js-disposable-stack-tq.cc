#include "src/objects/js-disposable-stack-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=24&c=1
bool IsJSDisposableStackBase_NonInline(Tagged<HeapObject> o) {
  return IsJSDisposableStackBase(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSDisposableStackBase<JSDisposableStackBase, JSObject>::JSDisposableStackBaseVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSDisposableStackBaseVerify(TrustedCast<JSDisposableStackBase>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=36&c=1
bool IsJSSyncDisposableStack_NonInline(Tagged<HeapObject> o) {
  return IsJSSyncDisposableStack(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSSyncDisposableStack<JSSyncDisposableStack, JSDisposableStackBase>::JSSyncDisposableStackVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSSyncDisposableStackVerify(TrustedCast<JSSyncDisposableStack>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/js-disposable-stack.tq?l=38&c=1
bool IsJSAsyncDisposableStack_NonInline(Tagged<HeapObject> o) {
  return IsJSAsyncDisposableStack(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedJSAsyncDisposableStack<JSAsyncDisposableStack, JSDisposableStackBase>::JSAsyncDisposableStackVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::JSAsyncDisposableStackVerify(TrustedCast<JSAsyncDisposableStack>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
