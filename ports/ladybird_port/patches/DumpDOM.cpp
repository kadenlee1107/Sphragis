/*
 * Bat_OS — Ladybird HTMLParser → real DOM tree demo (iter 23).
 *
 * Iter 21/22 used HTMLTokenizer alone — we got tokens and built a
 * pretend-tree from indentation. This binary runs the actual full
 * HTMLParser against a real Web::DOM::Document and walks the
 * resulting node tree.
 *
 * Bootstrap chain (each step is a known iter 14-22-style risk surface):
 *   1. Web::Bindings::initialize_main_thread_vm  → JS::VM + heap
 *   2. create_a_new_javascript_realm             → JS::Realm
 *   3. DOM::Document::create_for_fragment_parsing → temp Document
 *   4. Make a body Element on it as parse context
 *   5. HTMLParser::parse_html_fragment(body, html)
 *   6. Walk returned nodes recursively, printing tag names + text
 *
 * The `--tokens` flag falls back to the iter 21 flat-token output
 * (still useful when bootstrap fails).
 */

#include <LibCore/EventLoop.h>
#include <LibMain/Main.h>
#include <LibWeb/Bindings/MainThreadVM.h>
#include <LibWeb/DOM/Document.h>
#include <LibWeb/DOM/Element.h>
#include <LibWeb/DOM/Node.h>
#include <LibWeb/DOM/Text.h>
#include <LibWeb/HTML/HTMLBodyElement.h>
#include <LibWeb/HTML/HTMLDocument.h>
#include <LibWeb/HTML/Parser/HTMLParser.h>
#include <LibWeb/HTML/Parser/HTMLToken.h>
#include <LibWeb/HTML/Parser/HTMLTokenizer.h>
#include <AK/ByteString.h>
#include <AK/Format.h>
#include <AK/StringView.h>
#include <string.h>

ErrorOr<int> ladybird_main(Main::Arguments arguments);

static void emit_indent(int depth)
{
    for (int i = 0; i < depth; ++i)
        out("  ");
}

static void dump_node_tree(Web::DOM::Node const& node, int depth)
{
    using namespace Web::DOM;
    if (is<Element>(node)) {
        auto const& el = static_cast<Element const&>(node);
        emit_indent(depth);
        outln("<{}>", el.local_name());
        node.for_each_child([&](Node const& child) {
            dump_node_tree(child, depth + 1);
            return IterationDecision::Continue;
        });
        emit_indent(depth);
        outln("</{}>", el.local_name());
    } else if (is<Text>(node)) {
        auto const& text = static_cast<Text const&>(node);
        auto data = text.data();
        bool has_non_ws = false;
        for (size_t i = 0; i < data.length_in_code_units(); ++i) {
            auto cp = data.code_unit_at(i);
            if (cp != ' ' && cp != '\t' && cp != '\n' && cp != '\r') {
                has_non_ws = true; break;
            }
        }
        if (has_non_ws) {
            emit_indent(depth);
            outln("\"{}\"", data);
        }
    } else {
        emit_indent(depth);
        outln("({})", node.node_name());
    }
}

static int run_tokens_only(StringView html)
{
    outln("=== Bat_OS · HTMLTokenizer fallback (iter 21) ===");
    ByteString encoding = "UTF-8";
    Web::HTML::HTMLTokenizer tokenizer(html, encoding);
    int n = 0;
    while (true) {
        auto token = tokenizer.next_token();
        if (!token.has_value()) break;
        switch (token->type()) {
        case Web::HTML::HTMLToken::Type::DOCTYPE:
            outln("[{}] DOCTYPE", n++); break;
        case Web::HTML::HTMLToken::Type::StartTag:
            outln("[{}] StartTag <{}>", n++, token->tag_name()); break;
        case Web::HTML::HTMLToken::Type::EndTag:
            outln("[{}] EndTag </{}>", n++, token->tag_name()); break;
        case Web::HTML::HTMLToken::Type::Character:
            outln("[{}] Char '{}'", n++, AK::String::from_code_point(token->code_point())); break;
        case Web::HTML::HTMLToken::Type::EndOfFile:
            outln("[{}] EOF", n++); return 0;
        default: break;
        }
    }
    return 0;
}

ErrorOr<int> ladybird_main(Main::Arguments arguments)
{
    StringView html = "<!doctype html>"
                      "<html>"
                      "<head><title>Bat_OS · Ladybird real DOM</title></head>"
                      "<body>"
                      "<h1>Hello from Bat_OS</h1>"
                      "<p>Parsed by Ladybird's <em>real</em> HTMLParser.</p>"
                      "<ul><li>Engine: LibWeb</li><li>Kernel: bare-metal Rust</li></ul>"
                      "</body>"
                      "</html>"sv;

    bool tokens_mode = false;
    for (int i = 1; i < arguments.argc; ++i) {
        StringView a(arguments.argv[i], strlen(arguments.argv[i]));
        if (a == "--tokens"sv) tokens_mode = true;
        else if (!a.starts_with('-')) html = StringView(arguments.argv[i], strlen(arguments.argv[i]));
    }

    if (tokens_mode)
        return run_tokens_only(html);

    outln("=== Bat_OS · Ladybird HTMLParser → DOM ===");
    outln("input: {} bytes", html.length());
    outln("---");

    // Step 1: bring up Ladybird's main-thread JS VM.
    outln("[1/4] initialize_main_thread_vm...");
    Core::EventLoop loop;
    Web::Bindings::initialize_main_thread_vm(Web::Bindings::AgentType::SimilarOriginWindow);
    auto& vm = Web::Bindings::main_thread_vm();
    outln("       VM ready, heap @ {:p}", &vm.heap());

    // Step 2: create a fresh JS realm. The "create_a_new_javascript_realm"
    // helper in MainThreadVM expects callbacks for the global / globalThis
    // objects; for fragment parsing the simplest workable globals are
    // plain JS::Object instances.
    outln("[2/4] create_a_new_javascript_realm...");
    auto execution_context = Web::Bindings::create_a_new_javascript_realm(
        vm,
        [](JS::Realm& realm) -> JS::Object* {
            return JS::Object::create(realm, realm.intrinsics().object_prototype()).ptr();
        },
        [](JS::Realm& realm) -> JS::Object* {
            return JS::Object::create(realm, realm.intrinsics().object_prototype()).ptr();
        });
    auto& realm = *execution_context->realm;
    outln("       realm ready @ {:p}", &realm);

    // Step 3: create a temporary Document. This is the lightweight
    // path used internally by HTMLParser::parse_html_fragment — no
    // BrowsingContext, no Page, no Window required.
    outln("[3/4] Document::create_for_fragment_parsing...");
    auto document = Web::DOM::Document::create_for_fragment_parsing(realm);
    document->set_document_type(Web::DOM::Document::Type::HTML);
    outln("       document ready @ {:p}", document.ptr());

    // Step 4: parse + dump.
    outln("[4/4] HTMLParser::create + run...");
    auto parser = Web::HTML::HTMLParser::create(*document, html,
        Web::HTML::ParserScriptingMode::Disabled, "UTF-8"sv);
    parser->run(document->url());
    outln("       parse complete");
    outln("---");

    if (auto* root = document->document_element()) {
        dump_node_tree(*root, 0);
    } else {
        outln("(document has no root element — parse may have failed)");
    }
    outln("---");
    outln("(parsed via Web::HTML::HTMLParser on Bat_OS)");
    return 0;
}
