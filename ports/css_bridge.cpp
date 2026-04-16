// Bat_OS — CSS Tokenizer Bridge
// C API wrapper around Chromium's real CSS3 tokenizer for Rust FFI.
// Tokenizes the entire CSS source up-front into a flat array (since CSSTokenizer
// is STACK_ALLOCATED in Chromium and cannot be placement-newed onto the heap).

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
#include <deque>
#include <algorithm>
#include <functional>
#include <tuple>
#include <utility>

// Base infrastructure — MUST be in global namespace
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

// WTF types — in global namespace
#include "third_party/blink/renderer/platform/wtf/text/wtf_uchar.h"
#include "third_party/blink/renderer/platform/wtf/type_traits.h"
#include "third_party/blink/renderer/platform/wtf/text/string_impl.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_string.h"
#include "third_party/blink/renderer/platform/wtf/text/atomic_string.h"
#include "third_party/blink/renderer/platform/wtf/text/string_view.h"
#include "third_party/blink/renderer/platform/wtf/vector.h"
#include "third_party/blink/renderer/platform/wtf/allocator/allocator.h"

// GC infrastructure — in global namespace
#include "third_party/blink/renderer/platform/heap/garbage_collected.h"

// CSS headers — must come last
#include "third_party/blink/renderer/core/css/parser/css_parser_token.h"
#include "third_party/blink/renderer/core/css/parser/css_tokenizer.h"

// ═══════════════════════════════════════════════════════
// CSS Token type codes (FFI-stable)
// ═══════════════════════════════════════════════════════
enum CssTokenCode {
    CSS_TOKEN_IDENT       = 1,
    CSS_TOKEN_FUNCTION    = 2,
    CSS_TOKEN_AT_KEYWORD  = 3,
    CSS_TOKEN_HASH        = 4,
    CSS_TOKEN_STRING      = 5,
    CSS_TOKEN_NUMBER      = 6,
    CSS_TOKEN_PERCENTAGE  = 7,
    CSS_TOKEN_DIMENSION   = 8,
    CSS_TOKEN_WHITESPACE  = 9,
    CSS_TOKEN_DELIM       = 10,
    CSS_TOKEN_COLON       = 11,
    CSS_TOKEN_SEMICOLON   = 12,
    CSS_TOKEN_COMMA       = 13,
    CSS_TOKEN_LBRACE      = 14,
    CSS_TOKEN_RBRACE      = 15,
    CSS_TOKEN_LPAREN      = 16,
    CSS_TOKEN_RPAREN      = 17,
    CSS_TOKEN_LBRACKET    = 18,
    CSS_TOKEN_RBRACKET    = 19,
    CSS_TOKEN_URL         = 20,
    CSS_TOKEN_BAD_STRING  = 21,
    CSS_TOKEN_BAD_URL     = 22,
    CSS_TOKEN_COMMENT     = 23,
    CSS_TOKEN_CDO         = 24,
    CSS_TOKEN_CDC         = 25,
    CSS_TOKEN_EOF         = 26,
    CSS_TOKEN_OTHER       = 99,
};

// Flat token record stored in the result buffer.
struct CssTokenRecord {
    int    code;            // CssTokenCode
    int    text_offset;     // offset into the lexeme arena
    int    text_len;
    double numeric_value;
};

struct CssBridgeResult {
    CssTokenRecord* tokens;
    int             token_count;
    int             cursor;       // next token to return via _next()
    char*           lexeme_arena; // packed UTF-8 lexemes
    int             arena_len;
};

static int map_css_type(blink::CSSParserTokenType t) {
    using T = blink::CSSParserTokenType;
    switch (t) {
        case T::kIdentToken:        return CSS_TOKEN_IDENT;
        case T::kFunctionToken:     return CSS_TOKEN_FUNCTION;
        case T::kAtKeywordToken:    return CSS_TOKEN_AT_KEYWORD;
        case T::kHashToken:         return CSS_TOKEN_HASH;
        case T::kStringToken:       return CSS_TOKEN_STRING;
        case T::kNumberToken:       return CSS_TOKEN_NUMBER;
        case T::kPercentageToken:   return CSS_TOKEN_PERCENTAGE;
        case T::kDimensionToken:    return CSS_TOKEN_DIMENSION;
        case T::kWhitespaceToken:   return CSS_TOKEN_WHITESPACE;
        case T::kDelimiterToken:    return CSS_TOKEN_DELIM;
        case T::kColonToken:        return CSS_TOKEN_COLON;
        case T::kSemicolonToken:    return CSS_TOKEN_SEMICOLON;
        case T::kCommaToken:        return CSS_TOKEN_COMMA;
        case T::kLeftBraceToken:    return CSS_TOKEN_LBRACE;
        case T::kRightBraceToken:   return CSS_TOKEN_RBRACE;
        case T::kLeftParenthesisToken:  return CSS_TOKEN_LPAREN;
        case T::kRightParenthesisToken: return CSS_TOKEN_RPAREN;
        case T::kLeftBracketToken:  return CSS_TOKEN_LBRACKET;
        case T::kRightBracketToken: return CSS_TOKEN_RBRACKET;
        case T::kUrlToken:          return CSS_TOKEN_URL;
        case T::kBadStringToken:    return CSS_TOKEN_BAD_STRING;
        case T::kBadUrlToken:       return CSS_TOKEN_BAD_URL;
        case T::kCommentToken:      return CSS_TOKEN_COMMENT;
        case T::kCDOToken:          return CSS_TOKEN_CDO;
        case T::kCDCToken:          return CSS_TOKEN_CDC;
        case T::kEOFToken:          return CSS_TOKEN_EOF;
        default:                    return CSS_TOKEN_OTHER;
    }
}

extern "C" {

// Tokenize the entire CSS source up-front. Returns an opaque handle.
// The caller iterates with css_tokenizer_next() until CSS_TOKEN_EOF.
void* css_tokenizer_create(const char* css, int len) {
    if (!css || len <= 0) return nullptr;

    // Build a stable WTF::String — must outlive the tokenizer.
    WTF::String source(css, static_cast<unsigned>(len));

    // CSSTokenizer is STACK_ALLOCATED — construct it on our stack.
    // Use brace-initialization to avoid the most-vexing-parse.
    WTF::StringView source_view{source};
    blink::CSSTokenizer tokenizer{source_view};

    // Pre-size buffers conservatively.
    int cap_tokens = len + 16;
    int cap_arena  = len + 16;
    CssTokenRecord* tokens = static_cast<CssTokenRecord*>(
        malloc(sizeof(CssTokenRecord) * static_cast<size_t>(cap_tokens)));
    char* arena = static_cast<char*>(malloc(static_cast<size_t>(cap_arena)));
    int token_count = 0;
    int arena_used = 0;

    while (token_count < cap_tokens - 1) {
        blink::CSSParserToken tok = tokenizer.TokenizeSingle();
        int code = map_css_type(tok.GetType());

        WTF::StringView val = tok.Value();
        int vlen = static_cast<int>(val.length());
        if (arena_used + vlen > cap_arena) {
            cap_arena = (arena_used + vlen) * 2 + 64;
            arena = static_cast<char*>(realloc(arena, static_cast<size_t>(cap_arena)));
        }
        for (int i = 0; i < vlen; i++) {
            arena[arena_used + i] = static_cast<char>(val.Characters8()[i]);
        }

        CssTokenRecord& rec = tokens[token_count];
        rec.code = code;
        rec.text_offset = arena_used;
        rec.text_len = vlen;
        rec.numeric_value = 0.0;
        if (code == CSS_TOKEN_NUMBER || code == CSS_TOKEN_PERCENTAGE
            || code == CSS_TOKEN_DIMENSION) {
            rec.numeric_value = tok.NumericValue();
        }

        arena_used += vlen;
        token_count++;

        if (code == CSS_TOKEN_EOF) break;
    }

    auto* result = static_cast<CssBridgeResult*>(malloc(sizeof(CssBridgeResult)));
    result->tokens = tokens;
    result->token_count = token_count;
    result->cursor = 0;
    result->lexeme_arena = arena;
    result->arena_len = arena_used;
    return result;
}

// Pull the next token. Returns CSS_TOKEN_EOF when exhausted.
int css_tokenizer_next(void* handle,
                       char* text_buf, int* text_len,
                       int text_cap,
                       double* numeric_value) {
    auto* r = static_cast<CssBridgeResult*>(handle);
    if (!r || r->cursor >= r->token_count) {
        if (text_len) *text_len = 0;
        if (numeric_value) *numeric_value = 0.0;
        return CSS_TOKEN_EOF;
    }

    const CssTokenRecord& rec = r->tokens[r->cursor++];
    int copy = rec.text_len < text_cap ? rec.text_len : text_cap;
    for (int i = 0; i < copy; i++) {
        text_buf[i] = r->lexeme_arena[rec.text_offset + i];
    }
    if (text_len) *text_len = copy;
    if (numeric_value) *numeric_value = rec.numeric_value;
    return rec.code;
}

// Total tokens generated.
unsigned css_tokenizer_count(void* handle) {
    auto* r = static_cast<CssBridgeResult*>(handle);
    return r ? static_cast<unsigned>(r->token_count) : 0;
}

// Reset cursor so the caller can iterate again from the beginning.
void css_tokenizer_reset(void* handle) {
    auto* r = static_cast<CssBridgeResult*>(handle);
    if (r) r->cursor = 0;
}

// Free everything.
void css_tokenizer_destroy(void* handle) {
    auto* r = static_cast<CssBridgeResult*>(handle);
    if (!r) return;
    free(r->tokens);
    free(r->lexeme_arena);
    free(r);
}

}  // extern "C"
