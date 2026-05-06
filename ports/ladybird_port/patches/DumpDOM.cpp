/*
 * Bat_OS — Ladybird HTMLParser → real DOM tree demo (iter 24).
 *
 * Iter 23 crashed in Document::create_for_fragment_parsing because the
 * realm had no PrincipalHostDefined (no Page). Iter 24 adds a
 * HeadlessPageClient + proper Page bootstrap via
 * TraversableNavigable::create_a_new_top_level_traversable, which
 * sets up Window + ESO + PrincipalHostDefined on the realm.
 *
 * Bootstrap chain:
 *   1. initialize_main_thread_vm  → JS::VM + heap
 *   2. HeadlessPageClient + Page::create
 *   3. TraversableNavigable::create_a_new_top_level_traversable
 *      (creates BrowsingContext → Window → ESO → PrincipalHostDefined)
 *   4. Document::create_for_fragment_parsing on the navigable's realm
 *   5. HTMLParser::create + run
 *   6. Walk Document tree, dump Element + Text nodes
 *
 * The `--tokens` flag falls back to the iter 21 flat-token output
 * (still useful when bootstrap fails).
 */

#include <AK/ByteString.h>
#include <AK/Format.h>
#include <AK/Queue.h>
#include <AK/StringView.h>
#include <LibCore/AnonymousBuffer.h>
#include <LibCore/EventLoop.h>
#include <LibGfx/Palette.h>
#include <LibGfx/SystemTheme.h>
#include <LibMain/Main.h>
#include <LibWeb/Bindings/MainThreadVM.h>
#include <LibWeb/CSS/PreferredColorScheme.h>
#include <LibWeb/CSS/PreferredContrast.h>
#include <LibWeb/CSS/PreferredMotion.h>
#include <LibWeb/DOM/Document.h>
#include <LibWeb/DOM/Element.h>
#include <LibWeb/DOM/Node.h>
#include <LibWeb/DOM/Text.h>
#include <LibWeb/HTML/HTMLBodyElement.h>
#include <LibWeb/HTML/HTMLDocument.h>
#include <LibWeb/HTML/Parser/HTMLParser.h>
#include <LibWeb/HTML/Parser/HTMLToken.h>
#include <LibWeb/HTML/Parser/HTMLTokenizer.h>
#include <LibWeb/HTML/TraversableNavigable.h>
#include <LibWeb/Loader/FileRequest.h>
#include <LibWeb/Page/Page.h>
#include <LibWeb/Page/InputEvent.h>
#include <LibWeb/Platform/EventLoopPlugin.h>
#include <LibWeb/Platform/FontPlugin.h>
#include <string.h>

ErrorOr<int> ladybird_main(Main::Arguments arguments);

// ─── HeadlessPageClient ──────────────────────────────────────────
// Minimal concrete PageClient for headless DOM work. Follows the
// SVGDecodedImageData::SVGPageClient pattern from Ladybird's own
// source, but standalone (no host page to delegate to).

class HeadlessPageClient final : public Web::PageClient {
    GC_CELL(HeadlessPageClient, Web::PageClient);
    GC_DECLARE_ALLOCATOR(HeadlessPageClient);

public:
    static GC::Ref<HeadlessPageClient> create(JS::VM& vm)
    {
        return vm.heap().allocate<HeadlessPageClient>();
    }

    void set_page(GC::Ref<Web::Page> page) { m_page = page; }

    virtual u64 id() const override { return 0; }
    virtual Web::Page& page() override { return *m_page; }
    virtual Web::Page const& page() const override { return *m_page; }
    virtual bool is_connection_open() const override { return false; }

    virtual Gfx::Palette palette() const override { return m_palette; }
    virtual Web::DevicePixelRect screen_rect() const override { return { 0, 0, 800, 600 }; }
    virtual double zoom_level() const override { return 1.0; }
    virtual double device_pixel_ratio() const override { return 1.0; }
    virtual double device_pixels_per_css_pixel() const override { return 1.0; }
    virtual Web::CSS::PreferredColorScheme preferred_color_scheme() const override { return Web::CSS::PreferredColorScheme::Light; }
    virtual Web::CSS::PreferredContrast preferred_contrast() const override { return Web::CSS::PreferredContrast::NoPreference; }
    virtual Web::CSS::PreferredMotion preferred_motion() const override { return Web::CSS::PreferredMotion::NoPreference; }
    virtual size_t screen_count() const override { return 1; }

    virtual Queue<Web::QueuedInputEvent>& input_event_queue() override { return m_input_queue; }
    virtual void report_finished_handling_input_event(u64, Web::EventResult) override { }
    virtual void request_frame() override { }
    virtual void request_file(Web::FileRequest) override { }

    virtual Web::DisplayListPlayerType display_list_player_type() const override { return Web::DisplayListPlayerType::SkiaCPU; }
    virtual bool is_headless() const override { return true; }

private:
    HeadlessPageClient()
        : m_palette(Gfx::PaletteImpl::create_with_anonymous_buffer(
              MUST(Core::AnonymousBuffer::create_with_size(sizeof(Gfx::SystemTheme)))))
    {
    }

    virtual void visit_edges(Visitor& visitor) override
    {
        Base::visit_edges(visitor);
        visitor.visit(m_page);
    }

    GC::Ptr<Web::Page> m_page;
    Gfx::Palette m_palette;
    Queue<Web::QueuedInputEvent> m_input_queue;
};

GC_DEFINE_ALLOCATOR(HeadlessPageClient);

// ─── Tree dump helpers ───────────────────────────────────────────

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

// ─── Token-only fallback (iter 21) ──────────────────────────────

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

// ─── Main ────────────────────────────────────────────────────────

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
    outln("[1/5] initialize_main_thread_vm...");
    Core::EventLoop loop;
    Web::Bindings::initialize_main_thread_vm(Web::Bindings::AgentType::SimilarOriginWindow);
    auto& vm = Web::Bindings::main_thread_vm();
    outln("       VM ready, heap @ {:p}", &vm.heap());

    // Step 2: install platform plugins + create HeadlessPageClient + Page.
    outln("[2/5] HeadlessPageClient + Page::create...");
    Web::Platform::EventLoopPlugin::install(*new Web::Platform::EventLoopPlugin);
    Web::Platform::FontPlugin::install(*new Web::Platform::FontPlugin(false));
    auto page_client = HeadlessPageClient::create(vm);
    auto page = Web::Page::create(vm, *page_client);
    page_client->set_page(*page);
    page->set_is_scripting_enabled(false);
    outln("       page @ {:p}, client @ {:p}", page.ptr(), page_client.ptr());

    // Step 3: create a top-level traversable navigable. This is the
    // magic step that builds BrowsingContext → Window →
    // WindowEnvironmentSettingsObject → PrincipalHostDefined on a
    // proper realm. Without this, Document's ctor crashes reading
    // principal_host_defined_page(realm).
    outln("[3/5] TraversableNavigable::create_a_new_top_level_traversable...");
    page->set_top_level_traversable(
        Web::HTML::TraversableNavigable::create_a_new_top_level_traversable(*page, nullptr, {}));
    auto navigable = page->top_level_traversable();
    auto& realm = navigable->active_document()->realm();
    outln("       traversable ready, realm @ {:p}", &realm);

    // Step 4: create a temporary Document for fragment parsing.
    // The realm now has PrincipalHostDefined with our Page, so
    // Document's constructor can read principal_host_defined_page.
    outln("[4/5] Document::create_for_fragment_parsing...");
    auto document = Web::DOM::Document::create_for_fragment_parsing(realm);
    document->set_document_type(Web::DOM::Document::Type::HTML);
    outln("       document ready @ {:p}", document.ptr());

    // Step 5: parse + dump.
    outln("[5/5] HTMLParser::create + run...");
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
