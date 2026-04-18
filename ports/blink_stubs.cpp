// Bat_OS — Blink stub implementations
// Provides missing symbols that the tokenizer references but doesn't need
// for basic tokenization.

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <optional>
#include <ostream>
#include <memory>
#include <string>
#include <vector>
#include <map>
#include "base/containers/span.h"
#include "base/check.h"
#include "base/memory/ref_counted.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_uchar.h"
#include "third_party/blink/renderer/platform/wtf/text/string_impl.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_string.h"
#include "third_party/blink/renderer/platform/wtf/text/atomic_string.h"
#include "third_party/blink/renderer/platform/wtf/text/string_view.h"
#include "third_party/blink/renderer/platform/wtf/vector.h"

// Include the real Blink headers to match symbols
#include "third_party/blink/renderer/core/html/parser/html_entity_table.h"
#include "third_party/blink/renderer/core/html/parser/html_parser_options.h"
#include "third_party/blink/renderer/core/html/parser/html_entity_search.h"
#include "third_party/blink/renderer/platform/wtf/text/text_encoding.h"

namespace blink {

// HTMLEntityTable — stub implementations
base::span<const HTMLEntityTableEntry> HTMLEntityTable::AllEntries() {
    return base::span<const HTMLEntityTableEntry>(nullptr, static_cast<size_t>(0));
}
base::span<const HTMLEntityTableEntry> HTMLEntityTable::EntriesStartingWith(UChar) {
    return base::span<const HTMLEntityTableEntry>(nullptr, static_cast<size_t>(0));
}
base::span<const LChar> HTMLEntityTable::EntityString(const HTMLEntityTableEntry&) {
    return base::span<const LChar>(nullptr, static_cast<size_t>(0));
}

LChar HTMLEntityTableEntry::LastCharacter() const { return 0; }

// HTMLParserOptions constructor
HTMLParserOptions::HTMLParserOptions(Document*) {}

// TextEncoding
TextEncoding::TextEncoding(WTF::StringView) {}

}  // namespace blink

// WTF::String deferred implementations
namespace WTF {
String::String(StringImpl* impl) : is_null_(impl == nullptr) {
    if (impl) data_ = impl->StdString();
}
String& String::operator=(const scoped_refptr<StringImpl>& impl) {
    if (impl) { data_ = impl->StdString(); is_null_ = false; }
    else { data_.clear(); is_null_ = true; }
    return *this;
}
}
