// Bat_OS — CSS Tokenizer Test
// Feeds real CSS to Chromium's CSS3 tokenizer and prints the tokens.
// This is the REAL Chromium tokenizer — same code used by Chrome.

#include <cstdio>
#include <cstdlib>

extern "C" {
    void* css_tokenizer_create(const char* css, int len);
    int   css_tokenizer_next(void* handle,
                             char* text_buf, int* text_len,
                             int text_cap,
                             double* numeric_value);
    unsigned css_tokenizer_count(void* handle);
    void  css_tokenizer_destroy(void* handle);
}

// Token codes (mirror enum CssTokenCode in css_bridge.cpp)
enum {
    CSS_IDENT       = 1,
    CSS_FUNCTION    = 2,
    CSS_AT_KEYWORD  = 3,
    CSS_HASH        = 4,
    CSS_STRING      = 5,
    CSS_NUMBER      = 6,
    CSS_PERCENTAGE  = 7,
    CSS_DIMENSION   = 8,
    CSS_WHITESPACE  = 9,
    CSS_DELIM       = 10,
    CSS_COLON       = 11,
    CSS_SEMICOLON   = 12,
    CSS_COMMA       = 13,
    CSS_LBRACE      = 14,
    CSS_RBRACE      = 15,
    CSS_LPAREN      = 16,
    CSS_RPAREN      = 17,
    CSS_LBRACKET    = 18,
    CSS_RBRACKET    = 19,
    CSS_URL         = 20,
    CSS_BAD_STRING  = 21,
    CSS_BAD_URL     = 22,
    CSS_COMMENT     = 23,
    CSS_CDO         = 24,
    CSS_CDC         = 25,
    CSS_EOF         = 26,
};

static const char* code_name(int c) {
    switch (c) {
        case CSS_IDENT:       return "Ident";
        case CSS_FUNCTION:    return "Function";
        case CSS_AT_KEYWORD:  return "AtKeyword";
        case CSS_HASH:        return "Hash";
        case CSS_STRING:      return "String";
        case CSS_NUMBER:      return "Number";
        case CSS_PERCENTAGE:  return "Percentage";
        case CSS_DIMENSION:   return "Dimension";
        case CSS_WHITESPACE:  return "Whitespace";
        case CSS_DELIM:       return "Delim";
        case CSS_COLON:       return "Colon";
        case CSS_SEMICOLON:   return "Semicolon";
        case CSS_COMMA:       return "Comma";
        case CSS_LBRACE:      return "LBrace";
        case CSS_RBRACE:      return "RBrace";
        case CSS_LPAREN:      return "LParen";
        case CSS_RPAREN:      return "RParen";
        case CSS_LBRACKET:    return "LBracket";
        case CSS_RBRACKET:    return "RBracket";
        case CSS_URL:         return "Url";
        case CSS_COMMENT:     return "Comment";
        case CSS_CDO:         return "CDO";
        case CSS_CDC:         return "CDC";
        case CSS_EOF:         return "EOF";
        default:              return "Other";
    }
}

extern "C" void _start() {
    const char* css =
        "/* Bat_OS CSS test */\n"
        "body {\n"
        "  background-color: #1a1a1a;\n"
        "  color: rgb(255, 255, 255);\n"
        "  font-size: 14px;\n"
        "  margin: 0;\n"
        "}\n"
        "a:hover {\n"
        "  text-decoration: underline;\n"
        "  opacity: 0.8;\n"
        "}\n"
        ".container { padding: 1.5em 2rem; }\n"
        "@media (max-width: 768px) {\n"
        "  body { font-size: 12px; }\n"
        "}\n";

    int len = 0;
    while (css[len]) len++;

    printf("=== Chromium CSS3 Tokenizer Test on Bat_OS ===\n");
    printf("Input length: %d bytes\n\n", len);
    printf("Source:\n%s\n", css);
    printf("--- Tokens ---\n");

    void* tok = css_tokenizer_create(css, len);
    if (!tok) {
        printf("ERROR: failed to create tokenizer\n");
        exit(1);
    }

    int count = 0;
    char text[128];
    int text_len = 0;
    double nval = 0.0;
    int significant = 0;  // non-whitespace count

    while (count < 1000) {
        int code = css_tokenizer_next(tok, text, &text_len, sizeof(text), &nval);
        count++;
        if (code == CSS_EOF) break;
        if (code == CSS_WHITESPACE) continue;
        significant++;

        text[text_len] = '\0';
        printf("  [%3d] %-12s", significant, code_name(code));
        if (text_len > 0) {
            printf("  \"");
            for (int i = 0; i < text_len; i++) {
                char c = text[i];
                if (c == '\n') printf("\\n");
                else if (c >= 0x20 && c < 0x7f) printf("%c", c);
                else printf("?");
            }
            printf("\"");
        }
        if (code == CSS_NUMBER || code == CSS_PERCENTAGE || code == CSS_DIMENSION) {
            printf("  (value=%g)", nval);
        }
        printf("\n");
    }

    unsigned total = css_tokenizer_count(tok);
    printf("\n=== %d total tokens (%d non-whitespace) ===\n", (int)total, significant);
    if (significant > 30) {
        printf("=== CHROMIUM CSS3 TOKENIZER WORKS ON BAT_OS ===\n");
    }

    css_tokenizer_destroy(tok);
    exit(0);
    // not reached
}
int main() { _start(); return 0; }
