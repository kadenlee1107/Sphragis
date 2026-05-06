/*
 * Bat_OS — Ladybird HTML demo binary.
 *
 * Links against Ladybird's LibWeb. Reads HTML from argv[1] (or a
 * hardcoded sample), feeds it through the actual HTMLTokenizer
 * Ladybird uses to parse real web pages, then prints the result
 * two ways:
 *
 *  (1) flat token list — tag start/end + every char (--tokens)
 *  (2) tree-style DOM-ish dump — indented start/end tags with
 *      coalesced text content (default)
 *
 * The tree is built by walking the token stream and tracking
 * nesting depth via StartTag/EndTag pairs. It's NOT a full
 * HTMLParser run (that needs a Document, Page, EventLoop, GC
 * heap…), but it's enough to show the parser pipeline working
 * end-to-end on Bat_OS without setting up the entire browser.
 */

#include <LibCore/ArgsParser.h>
#include <LibMain/Main.h>
#include <LibWeb/HTML/Parser/HTMLToken.h>
#include <LibWeb/HTML/Parser/HTMLTokenizer.h>
#include <AK/ByteString.h>
#include <AK/Format.h>
#include <AK/StringBuilder.h>
#include <AK/StringView.h>
#include <string.h>

ErrorOr<int> ladybird_main(Main::Arguments arguments);

static void emit_indent(int depth)
{
    for (int i = 0; i < depth; ++i)
        out("  ");
}

ErrorOr<int> ladybird_main(Main::Arguments arguments)
{
    StringView html = "<!doctype html>"
                      "<html>"
                      "<head><title>Bat_OS · Ladybird first render</title></head>"
                      "<body>"
                      "<h1>Hello from Bat_OS</h1>"
                      "<p>You're reading this through Ladybird's <em>real</em> HTML parser.</p>"
                      "<p>Kernel: bare-metal Rust. Browser engine: <strong>LibWeb</strong>.</p>"
                      "<ul><li>JS engine: LibJS</li><li>Parser: HTMLTokenizer</li><li>OS: Bat_OS</li></ul>"
                      "</body>"
                      "</html>"sv;

    bool tokens_mode = false;
    for (int i = 1; i < arguments.argc; ++i) {
        StringView a(arguments.argv[i], strlen(arguments.argv[i]));
        if (a == "--tokens"sv) {
            tokens_mode = true;
        } else if (!a.starts_with('-')) {
            html = StringView(arguments.argv[i], strlen(arguments.argv[i]));
        }
    }

    outln("=== Bat_OS · Ladybird HTMLTokenizer demo ===");
    outln("input: {} bytes", html.length());
    outln("---");

    ByteString encoding = "UTF-8";
    Web::HTML::HTMLTokenizer tokenizer(html, encoding);

    if (tokens_mode) {
        // Flat per-token output (the original mode).
        int n = 0;
        while (true) {
            auto token = tokenizer.next_token();
            if (!token.has_value())
                break;
            switch (token->type()) {
            case Web::HTML::HTMLToken::Type::DOCTYPE:
                outln("[{}] DOCTYPE", n++);
                break;
            case Web::HTML::HTMLToken::Type::StartTag:
                outln("[{}] StartTag <{}>", n++, token->tag_name());
                break;
            case Web::HTML::HTMLToken::Type::EndTag:
                outln("[{}] EndTag </{}>", n++, token->tag_name());
                break;
            case Web::HTML::HTMLToken::Type::Character:
                outln("[{}] Char    '{}'", n++, AK::String::from_code_point(token->code_point()));
                break;
            case Web::HTML::HTMLToken::Type::Comment:
                outln("[{}] Comment", n++);
                break;
            case Web::HTML::HTMLToken::Type::EndOfFile:
                outln("[{}] EOF", n++);
                return 0;
            default:
                outln("[{}] (other)", n++);
                break;
            }
        }
        return 0;
    }

    // Tree-style: indent on StartTag, dedent on EndTag, coalesce
    // adjacent Character tokens into a single "text" line.
    int depth = 0;
    StringBuilder text_buf;
    auto flush_text = [&] {
        if (text_buf.is_empty()) return;
        // Trim runs of whitespace to one space; skip pure-whitespace text.
        bool has_non_ws = false;
        for (auto ch : text_buf.string_view()) {
            if (ch != ' ' && ch != '\t' && ch != '\n' && ch != '\r') {
                has_non_ws = true; break;
            }
        }
        if (has_non_ws) {
            emit_indent(depth + 1);
            outln("\"{}\"", text_buf.string_view());
        }
        text_buf.clear();
    };

    while (true) {
        auto token = tokenizer.next_token();
        if (!token.has_value())
            break;
        switch (token->type()) {
        case Web::HTML::HTMLToken::Type::DOCTYPE:
            flush_text();
            emit_indent(depth);
            outln("<!doctype html>");
            break;
        case Web::HTML::HTMLToken::Type::StartTag:
            flush_text();
            emit_indent(depth);
            outln("<{}>", token->tag_name());
            ++depth;
            break;
        case Web::HTML::HTMLToken::Type::EndTag:
            flush_text();
            if (depth > 0) --depth;
            emit_indent(depth);
            outln("</{}>", token->tag_name());
            break;
        case Web::HTML::HTMLToken::Type::Character: {
            u32 cp = token->code_point();
            if (cp < 0x80) {
                text_buf.append((char)cp);
            } else {
                // For non-ASCII, append the codepoint's UTF-8 form via
                // String::from_code_point.
                auto s = AK::String::from_code_point(cp);
                text_buf.append(s);
            }
            break;
        }
        case Web::HTML::HTMLToken::Type::Comment:
            flush_text();
            emit_indent(depth);
            outln("<!-- comment -->");
            break;
        case Web::HTML::HTMLToken::Type::EndOfFile:
            flush_text();
            outln("---");
            outln("(parsed via Web::HTML::HTMLTokenizer on Bat_OS)");
            return 0;
        default:
            break;
        }
    }
    return 0;
}
