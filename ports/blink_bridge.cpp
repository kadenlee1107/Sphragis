// Bat_OS — Blink Bridge
// C API wrapper around Chromium's HTML5 tokenizer for Rust FFI.
// This lets the Rust browser call Blink's tokenizer directly.

// System headers in global namespace first
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <optional>
#include <ostream>
#include <memory>
#include <string>
#include <vector>
#include <map>
#include <algorithm>
#include <functional>
#include <tuple>
#include <utility>

#include "base/containers/span.h"
#include "base/check.h"
#include "base/compiler_specific.h"
#include "base/memory/ref_counted.h"
#include "base/memory/raw_ptr.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_uchar.h"
#include "third_party/blink/renderer/platform/wtf/text/string_impl.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_string.h"
#include "third_party/blink/renderer/platform/wtf/text/atomic_string.h"
#include "third_party/blink/renderer/platform/wtf/text/string_view.h"
#include "third_party/blink/renderer/platform/wtf/vector.h"
#include "third_party/blink/renderer/core/html/parser/html_token.h"
#include "third_party/blink/renderer/core/html/parser/html_tokenizer.h"
#include "third_party/blink/renderer/platform/text/segmented_string.h"

// ═══════════════════════════════════════════════════════
// C Bridge API — callable from Rust via FFI
// ═══════════════════════════════════════════════════════

// Token types matching our Rust enum
enum BlinkTokenType {
    BLINK_TOKEN_START_TAG = 1,
    BLINK_TOKEN_END_TAG = 2,
    BLINK_TOKEN_CHARACTER = 3,
    BLINK_TOKEN_COMMENT = 4,
    BLINK_TOKEN_DOCTYPE = 5,
    BLINK_TOKEN_EOF = 6,
};

struct BlinkTokenizerState {
    blink::HTMLTokenizer* tokenizer;
    blink::SegmentedString* input;
};

extern "C" {

void blink_debug_sizes(void);

// Create a new Blink HTML tokenizer
void* blink_tokenizer_create(void) {
    blink_debug_sizes();
    auto* state = static_cast<BlinkTokenizerState*>(malloc(sizeof(BlinkTokenizerState)));
    blink::HTMLParserOptions opts;
    state->tokenizer = static_cast<blink::HTMLTokenizer*>(malloc(sizeof(blink::HTMLTokenizer)));
    new (state->tokenizer) blink::HTMLTokenizer(opts);
    state->input = nullptr;
    return state;
}

// Debug: print struct sizes
void blink_debug_sizes(void) {
    printf("sizeof(WTF::String) = %lu\n", sizeof(WTF::String));
    printf("sizeof(blink::SegmentedString) = %lu\n", sizeof(blink::SegmentedString));
    printf("sizeof(blink::HTMLTokenizer) = %lu\n", sizeof(blink::HTMLTokenizer));
    printf("sizeof(blink::HTMLToken) = %lu\n", sizeof(blink::HTMLToken));
    printf("sizeof(std::string) = %lu\n", sizeof(std::string));
}

// Feed HTML text to the tokenizer
void blink_tokenizer_feed(void* handle, const char* html, int len) {
    auto* state = static_cast<BlinkTokenizerState*>(handle);
    printf("[bridge] feed %d bytes\n", len);

    // Simple approach: construct WTF::String on stack
    WTF::String str(html, static_cast<unsigned>(len));
    printf("[bridge] String len=%u is_null=%d\n", str.length(), str.IsNull());
    printf("[bridge] first char: %c (0x%x)\n", html[0], (unsigned char)html[0]);

    // Verify our string works
    const char* chars = str.Characters8();
    printf("[bridge] Characters8=%p data_ptr=%p\n", (void*)chars, (void*)str.impl().c_str());

    if (state->input) { state->input->~SegmentedString(); free(state->input); }
    state->input = static_cast<blink::SegmentedString*>(malloc(sizeof(blink::SegmentedString)));
    printf("[bridge] seg_string at %p, constructing...\n", (void*)state->input);
    new (state->input) blink::SegmentedString(str);
    printf("[bridge] done! length=%u\n", static_cast<unsigned>(state->input->length()));
}

// Get the next token. Returns token type (0 = no more tokens).
// Fills name_buf with tag name (for start/end tags).
// Fills text_buf with text content (for character tokens).
int blink_tokenizer_next(void* handle,
                         char* name_buf, int* name_len,
                         char* text_buf, int* text_len,
                         int name_cap, int text_cap) {
    auto* state = static_cast<BlinkTokenizerState*>(handle);
    if (!state->input) return 0;

    blink::HTMLToken* tok = state->tokenizer->NextToken(*state->input);
    if (!tok) return 0;

    int type = 0;
    *name_len = 0;
    *text_len = 0;

    switch (tok->GetType()) {
        case blink::HTMLToken::kStartTag: {
            type = BLINK_TOKEN_START_TAG;
            const auto& name = tok->GetName();
            int nl = name.size() < (unsigned)name_cap ? name.size() : name_cap;
            for (int i = 0; i < nl; i++) name_buf[i] = static_cast<char>(name.data()[i]);
            *name_len = nl;
            break;
        }
        case blink::HTMLToken::kEndTag: {
            type = BLINK_TOKEN_END_TAG;
            const auto& name = tok->GetName();
            int nl = name.size() < (unsigned)name_cap ? name.size() : name_cap;
            for (int i = 0; i < nl; i++) name_buf[i] = static_cast<char>(name.data()[i]);
            *name_len = nl;
            break;
        }
        case blink::HTMLToken::kCharacter: {
            type = BLINK_TOKEN_CHARACTER;
            const auto& data = tok->Data();
            int tl = data.size() < (unsigned)text_cap ? data.size() : text_cap;
            for (int i = 0; i < tl; i++) text_buf[i] = static_cast<char>(data.data()[i]);
            *text_len = tl;
            break;
        }
        case blink::HTMLToken::kComment:
            type = BLINK_TOKEN_COMMENT;
            break;
        case blink::HTMLToken::kEndOfFile:
            type = BLINK_TOKEN_EOF;
            break;
        default:
            type = BLINK_TOKEN_DOCTYPE;
            break;
    }

    state->tokenizer->ClearToken();
    return type;
}

// Destroy the tokenizer
void blink_tokenizer_destroy(void* handle) {
    auto* state = static_cast<BlinkTokenizerState*>(handle);
    state->tokenizer->~HTMLTokenizer();
    free(state->tokenizer);
    if (state->input) { state->input->~SegmentedString(); free(state->input); }
    free(state);
}

}  // extern "C"
