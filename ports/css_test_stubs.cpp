// Bat_OS — CSS Test Stubs
// Provides definitions for Blink symbols referenced by libblink.a's CSS
// objects but never implemented (because they live in DOM/HTML files we
// haven't compiled). Linked into the css_tokenizer_test binary.

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <optional>
#include <ostream>
#include <memory>
#include <string>
#include <vector>
#include <map>
#include <deque>
#include <algorithm>
#include <functional>
#include <tuple>
#include <utility>

#include "base/compiler_specific.h"
#include "build/build_config.h"
#include "build/buildflag.h"
#include "base/check.h"
#include "base/logging.h"
#include "base/containers/span.h"
#include "base/memory/ref_counted.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "base/memory/stack_allocated.h"
#include "base/strings/string_piece.h"

#include "third_party/blink/renderer/platform/wtf/text/wtf_uchar.h"
#include "third_party/blink/renderer/platform/wtf/type_traits.h"
#include "third_party/blink/renderer/platform/wtf/text/string_impl.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_string.h"
#include "third_party/blink/renderer/platform/wtf/text/atomic_string.h"
#include "third_party/blink/renderer/platform/wtf/text/string_view.h"
#include "third_party/blink/renderer/platform/wtf/vector.h"
#include "third_party/blink/renderer/platform/wtf/allocator/allocator.h"

#include "third_party/blink/renderer/platform/heap/garbage_collected.h"
#include "third_party/blink/renderer/core/dom/qualified_name.h"

namespace blink {

// QualifiedName is referenced by global initializers in css_tokenizer.o
// but its definitions live in dom/qualified_name.cc which we haven't built.
QualifiedName::QualifiedName(const WTF::AtomicString&) {}
QualifiedName::~QualifiedName() = default;

// Serialize* helpers live in css_markup.cc which we haven't built.
void SerializeIdentifier(const WTF::String&, WTF::StringBuilder&, bool) {}
void SerializeString(const WTF::String&, WTF::StringBuilder&) {}

}  // namespace blink

// QualifiedNameImpl destructor — referenced via the RefCounted Release path.
namespace blink {
QualifiedName::QualifiedNameImpl::~QualifiedNameImpl() = default;
}  // namespace blink

// sincos isn't in our minimal libc. Forward to extern-C sin/cos symbols
// provided by bat_libc_kernel.o. Use distinct local aliases to avoid
// the C++ linkage conflict with math.h's declarations.
extern "C" double __bat_sin(double) __asm__("sin");
extern "C" double __bat_cos(double) __asm__("cos");
extern "C" float  __bat_sinf(float) __asm__("sinf");
extern "C" float  __bat_cosf(float) __asm__("cosf");

extern "C" void sincos(double x, double* s, double* c) {
    *s = __bat_sin(x);
    *c = __bat_cos(x);
}
extern "C" void sincosf(float x, float* s, float* c) {
    *s = __bat_sinf(x);
    *c = __bat_cosf(x);
}
