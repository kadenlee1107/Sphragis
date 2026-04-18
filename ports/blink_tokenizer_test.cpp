// Bat_OS — Blink HTML Tokenizer Test
// Feeds real HTML to Chromium's HTML5 tokenizer and prints the tokens.
// This is the REAL Chromium tokenizer — same code used by Chrome.

// Preamble: include system headers in global namespace
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

// Chromium infrastructure
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

// Blink tokenizer
#include "third_party/blink/renderer/core/html/parser/html_token.h"
#include "third_party/blink/renderer/core/html/parser/html_tokenizer.h"
#include "third_party/blink/renderer/platform/text/segmented_string.h"

using blink::HTMLParserOptions;
using blink::HTMLTokenizer;
using blink::HTMLToken;
using blink::SegmentedString;

extern "C" void _start() {
    printf("\n=== Bat_OS Blink HTML Tokenizer Test ===\n");
    printf("Chromium's real HTML5 tokenizer on bare-metal ARM64\n\n");

    // Create tokenizer with default options
    HTMLParserOptions options;
    HTMLTokenizer tokenizer(options);

    // Feed HTML to tokenizer
    const char* html = "<html><head><title>Bat_OS</title></head>"
                       "<body><h1>Hello from Chromium!</h1>"
                       "<p>This is <b>Blink</b> on bare metal.</p>"
                       "</body></html>";

    printf("Input HTML:\n  %s\n\n", html);

    SegmentedString input{WTF::String(html)};

    int token_count = 0;
    printf("Tokens produced by Chromium's HTML5 tokenizer:\n");

    HTMLToken* tok;
    while ((tok = tokenizer.NextToken(input)) != nullptr) {
        const HTMLToken& token = *tok;
        token_count++;

        const char* type_name = "???";
        switch (token.GetType()) {
            case HTMLToken::kUninitialized: type_name = "Uninitialized"; break;
            case HTMLToken::DOCTYPE: type_name = "DOCTYPE"; break;
            case HTMLToken::kStartTag: type_name = "StartTag"; break;
            case HTMLToken::kEndTag: type_name = "EndTag"; break;
            case HTMLToken::kComment: type_name = "Comment"; break;
            case HTMLToken::kCharacter: type_name = "Character"; break;
            case HTMLToken::kEndOfFile: type_name = "EndOfFile"; break;
        }

        printf("  [%2d] %-10s", token_count, type_name);

        if (token.GetType() == HTMLToken::kStartTag ||
            token.GetType() == HTMLToken::kEndTag) {
            const auto& nameData = token.GetName();
            char nameBuf[64] = {0};
            for (unsigned i = 0; i < nameData.size() && i < 63; i++)
                nameBuf[i] = static_cast<char>(nameData.data()[i]);
            printf(" <%s%s>",
                   token.GetType() == HTMLToken::kEndTag ? "/" : "",
                   nameBuf);
        } else if (token.GetType() == HTMLToken::kCharacter) {
            // Print first few chars of text content
            printf(" \"");
            const auto& data = token.Data();
            for (unsigned i = 0; i < data.size() && i < 30; i++) {
                char c = static_cast<char>(data.data()[i]);
                if (c == '\n') printf("\\n");
                else printf("%c", c);
            }
            printf("\"");
        }
        printf("\n");

        tokenizer.ClearToken();

        if (tok->GetType() == HTMLToken::kEndOfFile) break;
        if (token_count > 50) { printf("  ... (truncated)\n"); break; }
    }

    printf("\n=== %d tokens produced ===\n", token_count);
    if (token_count > 10) {
        printf("=== CHROMIUM HTML5 TOKENIZER WORKS ON BAT_OS ===\n");
    }

    exit(0);
}
