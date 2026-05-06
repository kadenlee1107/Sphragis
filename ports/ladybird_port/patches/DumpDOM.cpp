/*
 * Bat_OS — minimal standalone HTML tokenize demo.
 *
 * Links against Ladybird's LibWeb (Lagom build of liblagom-web).
 * Reads HTML from argv[1] (or hardcoded string), tokenizes it via
 * the actual same HTMLTokenizer Ladybird uses for real pages,
 * prints each token to stdout. Exits 0 on success.
 *
 * This is a standalone main — not a LibTest TestCase — so the
 * build produces a regular binary we can ship in the cave.
 */

#include <LibCore/ArgsParser.h>
#include <LibCore/EventLoop.h>
#include <LibMain/Main.h>
#include <LibWeb/HTML/Parser/HTMLToken.h>
#include <LibWeb/HTML/Parser/HTMLTokenizer.h>
#include <AK/ByteString.h>
#include <AK/Format.h>
#include <AK/StringView.h>

ErrorOr<int> ladybird_main(Main::Arguments arguments);

ErrorOr<int> ladybird_main(Main::Arguments arguments)
{
    StringView html = "<!doctype html><html><body><h1>Hello, Bat_OS!</h1><p>Ladybird parses this on a custom kernel.</p></body></html>"sv;
    if (arguments.argc > 1)
        html = StringView(arguments.argv[1], strlen(arguments.argv[1]));

    outln("=== Bat_OS dump-html-tokens ===");
    outln("input ({} bytes): {}", html.length(), html);
    outln("---");

    ByteString encoding = "UTF-8";
    Web::HTML::HTMLTokenizer tokenizer(html, encoding);
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
        case Web::HTML::HTMLToken::Type::Character: {
            // Coalesce whitespace-only into a single line for readability.
            outln("[{}] Char    '{}'", n++, AK::String::from_code_point(token->code_point()));
            break;
        }
        case Web::HTML::HTMLToken::Type::Comment:
            outln("[{}] Comment", n++);
            break;
        case Web::HTML::HTMLToken::Type::EndOfFile:
            outln("[{}] EOF", n++);
            return 0;
        default:
            outln("[{}] (other type)", n++);
            break;
        }
    }
    outln("--- done, {} tokens ---", n);
    return 0;
}
