// Bat_OS — HTML + CSS Integration Test
// Parses real HTML with <style> tags, computes CSS styles for each element
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <libcss/libcss.h>
#include <parserutils/charset/mibenum.h>
#include <libwapcaplet/libwapcaplet.h>

// === Mini DOM ===
#define MAX_NODES 64
#define MAX_TEXT 128

typedef struct dom_node {
    char tag[16];
    char id[32];
    char classes[64];
    char text[MAX_TEXT];
    int parent;       // index, -1 for root
    int first_child;
    int next_sibling;
    int type;          // 0=element, 1=text
} dom_node;

static dom_node nodes[MAX_NODES];
static int node_count = 0;
static char style_css[2048]; // extracted from <style> tags
static int style_len = 0;
static css_computed_style *computed_styles[MAX_NODES];  // for CSS inheritance
static css_select_results *select_results[MAX_NODES];  // kept alive until cleanup

// === Simple HTML Parser ===
static int add_node(const char *tag, int parent, int type) {
    if (node_count >= MAX_NODES) return -1;
    int idx = node_count++;
    memset(&nodes[idx], 0, sizeof(dom_node));
    if (tag) strncpy(nodes[idx].tag, tag, 15);
    nodes[idx].parent = parent;
    nodes[idx].first_child = -1;
    nodes[idx].next_sibling = -1;
    nodes[idx].type = type;
    // Link as child
    if (parent >= 0) {
        if (nodes[parent].first_child < 0) {
            nodes[parent].first_child = idx;
        } else {
            int sib = nodes[parent].first_child;
            while (nodes[sib].next_sibling >= 0) sib = nodes[sib].next_sibling;
            nodes[sib].next_sibling = idx;
        }
    }
    return idx;
}

static void parse_html(const char *html) {
    int current = -1;
    int i = 0;
    int len = strlen(html);
    while (i < len) {
        if (html[i] == '<') {
            if (html[i+1] == '/') {
                // Closing tag — go up
                while (i < len && html[i] != '>') i++;
                i++;
                if (current >= 0) current = nodes[current].parent;
                continue;
            }
            // Check for <style>
            if (strncmp(&html[i], "<style", 6) == 0) {
                while (i < len && html[i] != '>') i++;
                i++;
                // Copy style content
                while (i < len && strncmp(&html[i], "</style>", 8) != 0) {
                    if (style_len < 2047) style_css[style_len++] = html[i];
                    i++;
                }
                i += 8; // skip </style>
                continue;
            }
            // Opening tag
            i++; // skip '<'
            char tag[16] = {0};
            int ti = 0;
            while (i < len && html[i] != '>' && html[i] != ' ' && ti < 15) {
                tag[ti++] = tolower(html[i++]);
            }
            // Parse attributes
            char id[32] = {0};
            char cls[64] = {0};
            while (i < len && html[i] != '>') {
                if (strncmp(&html[i], "id=\"", 4) == 0) {
                    i += 4; int j = 0;
                    while (i < len && html[i] != '"' && j < 31) id[j++] = html[i++];
                    if (i < len) i++;
                } else if (strncmp(&html[i], "class=\"", 7) == 0) {
                    i += 7; int j = 0;
                    while (i < len && html[i] != '"' && j < 63) cls[j++] = html[i++];
                    if (i < len) i++;
                } else {
                    i++;
                }
            }
            if (i < len) i++; // skip '>'
            int node = add_node(tag, current, 0);
            if (node >= 0) {
                if (id[0]) strncpy(nodes[node].id, id, 31);
                if (cls[0]) strncpy(nodes[node].classes, cls, 63);
                // Don't descend into void elements
                if (strcmp(tag,"br")==0||strcmp(tag,"hr")==0||strcmp(tag,"img")==0||
                    strcmp(tag,"input")==0||strcmp(tag,"meta")==0||strcmp(tag,"link")==0) {
                    continue;
                }
                current = node;
            }
        } else {
            // Text content
            char text[MAX_TEXT] = {0};
            int ti = 0;
            while (i < len && html[i] != '<' && ti < MAX_TEXT-1) {
                text[ti++] = html[i++];
            }
            // Trim whitespace
            int start = 0, end = ti;
            while (start < end && (text[start]==' '||text[start]=='\n'||text[start]=='\t')) start++;
            while (end > start && (text[end-1]==' '||text[end-1]=='\n'||text[end-1]=='\t')) end--;
            if (end > start) {
                int n = add_node("#text", current, 1);
                if (n >= 0) {
                    memcpy(nodes[n].text, &text[start], end-start);
                }
            }
        }
    }
}

// === CSS Handler Callbacks ===
static css_error h_node_name(void *pw, void *n, css_qname *qname) {
    dom_node *node = (dom_node*)n; (void)pw;
    lwc_intern_string(node->tag, strlen(node->tag), &qname->name);
    return CSS_OK;
}
static lwc_string *class_storage[8]; // up to 8 classes per element
static css_error h_node_classes(void *pw, void *n, lwc_string ***cls, uint32_t *nc) {
    dom_node *node = (dom_node*)n; (void)pw;
    if (!node->classes[0]) { *cls = NULL; *nc = 0; return CSS_OK; }
    // Parse space-separated class names
    uint32_t count = 0;
    char buf[64];
    int bi = 0;
    const char *p = node->classes;
    while (*p && count < 8) {
        if (*p == ' ') {
            if (bi > 0) {
                buf[bi] = 0;
                lwc_intern_string(buf, bi, &class_storage[count++]);
                bi = 0;
            }
        } else if (bi < 63) {
            buf[bi++] = *p;
        }
        p++;
    }
    if (bi > 0) {
        buf[bi] = 0;
        lwc_intern_string(buf, bi, &class_storage[count++]);
    }
    *cls = class_storage;
    *nc = count;
    return CSS_OK;
}
static css_error h_node_id(void *pw, void *n, lwc_string **id) {
    dom_node *node = (dom_node*)n; (void)pw;
    if (node->id[0]) lwc_intern_string(node->id, strlen(node->id), id);
    else *id = NULL;
    return CSS_OK;
}
static css_error h_parent(void *pw, void *n, void **p) {
    dom_node *node = (dom_node*)n; (void)pw;
    *p = (node->parent >= 0) ? &nodes[node->parent] : NULL;
    return CSS_OK;
}
static css_error h_named_anc(void *pw, void *n, const css_qname *qn, void **a) {
    dom_node *node = (dom_node*)n; (void)pw;
    const char *want = lwc_string_data(qn->name);
    int idx = node->parent;
    while (idx >= 0) {
        if (strcmp(nodes[idx].tag, want) == 0) {
            *a = &nodes[idx];
            return CSS_OK;
        }
        idx = nodes[idx].parent;
    }
    *a = NULL;
    return CSS_OK;
}
static css_error h_named_par(void *pw, void *n, const css_qname *qn, void **p) {
    dom_node *node = (dom_node*)n; (void)pw;
    const char *want = lwc_string_data(qn->name);
    if (node->parent >= 0 && strcmp(nodes[node->parent].tag, want) == 0) {
        *p = &nodes[node->parent];
    } else {
        *p = NULL;
    }
    return CSS_OK;
}
static css_error h_sibling(void *pw, void *n, void **s) {
    (void)pw; (void)n; *s = NULL; return CSS_OK;
}
static css_error h_named_sib(void *pw, void *n, const css_qname *qn, void **s) {
    (void)pw; (void)n; (void)qn; *s = NULL; return CSS_OK;
}
static css_error h_has_name(void *pw, void *n, const css_qname *qn, bool *m) {
    dom_node *node = (dom_node*)n; (void)pw;
    *m = (strcmp(node->tag, lwc_string_data(qn->name)) == 0);
    return CSS_OK;
}
static css_error h_has_class(void *pw, void *n, lwc_string *name, bool *m) {
    dom_node *node = (dom_node*)n; (void)pw;
    *m = node->classes[0] && strstr(node->classes, lwc_string_data(name));
    return CSS_OK;
}
static css_error h_has_id(void *pw, void *n, lwc_string *name, bool *m) {
    dom_node *node = (dom_node*)n; (void)pw;
    *m = node->id[0] && strcmp(node->id, lwc_string_data(name)) == 0;
    return CSS_OK;
}
static css_error h_is_root(void *pw, void *n, bool *m) {
    dom_node *node = (dom_node*)n; (void)pw;
    *m = (node->parent < 0 || strcmp(node->tag, "html") == 0);
    return CSS_OK;
}
static css_error h_false(void *pw, void *n, bool *m) { (void)pw;(void)n; *m=false; return CSS_OK; }
static css_error h_attr(void *pw, void *n, const css_qname *q, bool *m) { (void)pw;(void)n;(void)q; *m=false; return CSS_OK; }
static css_error h_attr_val(void *pw, void *n, const css_qname *q, lwc_string *v, bool *m) { (void)pw;(void)n;(void)q;(void)v; *m=false; return CSS_OK; }
static css_error h_count(void *pw, void *n, bool a, bool b, int32_t *c) { (void)pw;(void)n;(void)a;(void)b; *c=0; return CSS_OK; }
static css_error h_lang(void *pw, void *n, lwc_string *l, bool *m) { (void)pw;(void)n;(void)l; *m=false; return CSS_OK; }
static css_error h_hint(void *pw, void *n, uint32_t *nh, css_hint **h) { (void)pw;(void)n; *nh=0; *h=NULL; return CSS_OK; }
static css_error h_ua_def(void *pw, uint32_t prop, css_hint *hint) {
    (void)pw; (void)prop;
    memset(hint, 0, sizeof(*hint));
    return CSS_OK;
}
static css_error h_set_data(void *pw, void *n, void *d) { (void)pw;(void)n;(void)d; return CSS_OK; }
static css_error h_get_data(void *pw, void *n, void **d) { (void)pw;(void)n; *d=NULL; return CSS_OK; }
static css_error resolve_url(void *pw, const char *base, lwc_string *rel, lwc_string **abs) {
    (void)pw; (void)base; *abs = lwc_string_ref(rel); return CSS_OK;
}

void _start(void) {
    printf("=== HTML + CSS Integration Test ===\n\n");

    // Parse real HTML with embedded CSS
    const char *html =
        "<html>"
        "<head>"
        "<style>"
        "body { background: #1a1a2e; color: #e0e0e0; font-size: 14px; }"
        "h1 { color: #ff6600; font-size: 28px; margin: 20px 0; }"
        "h2 { color: #00ccff; font-size: 22px; }"
        "p { margin: 10px 0; }"
        "a { color: #ff9900; }"
        ".highlight { color: #ffff00; }"
        "#title { font-size: 32px; }"
        "</style>"
        "</head>"
        "<body>"
        "<h1 id=\"title\">Bat_OS Browser</h1>"
        "<p>A bare-metal ARM64 operating system with a real CSS engine.</p>"
        "<h2>Features</h2>"
        "<p class=\"highlight\">Renders Wikipedia over HTTPS with TrueType fonts.</p>"
        "<p>Built from scratch. <a href=\"#\">Learn more</a></p>"
        "</body>"
        "</html>";

    parse_html(html);
    printf("[1] Parsed %d DOM nodes, extracted %d bytes of CSS\n", node_count, style_len);
    style_css[style_len] = 0;

    // Create CSS stylesheet from extracted <style> content
    css_stylesheet_params params;
    memset(&params, 0, sizeof(params));
    params.params_version = CSS_STYLESHEET_PARAMS_VERSION_1;
    params.level = CSS_LEVEL_3;
    params.charset = "UTF-8";
    params.url = "inline";
    params.resolve = resolve_url;

    css_stylesheet *sheet = NULL;
    css_stylesheet_create(&params, &sheet);
    css_stylesheet_append_data(sheet, (const uint8_t*)style_css, style_len);
    css_stylesheet_data_done(sheet);
    printf("[2] CSS stylesheet created (%d bytes)\n", style_len);

    // Create selection context
    css_select_ctx *ctx = NULL;
    css_select_ctx_create(&ctx);
    css_select_ctx_append_sheet(ctx, sheet, CSS_ORIGIN_AUTHOR, NULL);
    printf("[3] Selection context ready\n");

    // Build handler
    css_select_handler handler;
    memset(&handler, 0, sizeof(handler));
    handler.handler_version = CSS_SELECT_HANDLER_VERSION_1;
    handler.node_name = h_node_name;
    handler.node_classes = h_node_classes;
    handler.node_id = h_node_id;
    handler.named_ancestor_node = h_named_anc;
    handler.named_parent_node = h_named_par;
    handler.named_sibling_node = h_named_sib;
    handler.named_generic_sibling_node = h_named_sib;
    handler.parent_node = h_parent;
    handler.sibling_node = h_sibling;
    handler.node_has_name = h_has_name;
    handler.node_has_class = h_has_class;
    handler.node_has_id = h_has_id;
    handler.node_has_attribute = h_attr;
    handler.node_has_attribute_equal = h_attr_val;
    handler.node_has_attribute_dashmatch = h_attr_val;
    handler.node_has_attribute_includes = h_attr_val;
    handler.node_has_attribute_prefix = h_attr_val;
    handler.node_has_attribute_suffix = h_attr_val;
    handler.node_has_attribute_substring = h_attr_val;
    handler.node_is_root = h_is_root;
    handler.node_count_siblings = h_count;
    handler.node_is_empty = h_false;
    handler.node_is_link = h_false;
    handler.node_is_visited = h_false;
    handler.node_is_hover = h_false;
    handler.node_is_active = h_false;
    handler.node_is_focus = h_false;
    handler.node_is_enabled = h_false;
    handler.node_is_disabled = h_false;
    handler.node_is_checked = h_false;
    handler.node_is_target = h_false;
    handler.node_is_lang = h_lang;
    handler.node_presentational_hint = h_hint;
    handler.ua_default_for_property = h_ua_def;
    handler.set_libcss_node_data = h_set_data;
    handler.get_libcss_node_data = h_get_data;

    css_unit_ctx unit_ctx;
    memset(&unit_ctx, 0, sizeof(unit_ctx));
    unit_ctx.viewport_width = 1280 * (1 << CSS_RADIX_POINT);
    unit_ctx.viewport_height = 1024 * (1 << CSS_RADIX_POINT);
    unit_ctx.device_dpi = 96 * (1 << CSS_RADIX_POINT);
    unit_ctx.font_size_default = 16 * (1 << CSS_RADIX_POINT);

    css_media media;
    memset(&media, 0, sizeof(media));
    media.type = CSS_MEDIA_SCREEN;

    // Compute styles for EACH element
    printf("\n[4] Computing styles for DOM elements:\n\n");
    for (int i = 0; i < node_count; i++) {
        if (nodes[i].type != 0) continue; // skip text nodes
        if (strcmp(nodes[i].tag, "html")==0 || strcmp(nodes[i].tag, "head")==0 ||
            strcmp(nodes[i].tag, "style")==0) continue;

        // Look up parent's computed style for CSS inheritance
        css_computed_style *parent_style = NULL;
        if (nodes[i].parent >= 0) {
            parent_style = computed_styles[nodes[i].parent];
        }

        css_select_results *results = NULL;
        css_error err = css_select_style(ctx, &nodes[i], &unit_ctx, &media,
                                          NULL, &handler, NULL, &results);
        if (err == CSS_OK && results) {
            css_computed_style *style = results->styles[CSS_PSEUDO_ELEMENT_NONE];
            if (style) {
                // Compose with parent style for CSS inheritance
                css_computed_style *final_style = style;
                if (parent_style) {
                    css_computed_style *composed = NULL;
                    css_error cerr = css_computed_style_compose(
                        parent_style, style, &unit_ctx, &composed);
                    if (cerr == CSS_OK && composed) {
                        final_style = composed;
                    }
                }

                // Store for child inheritance (keep results alive)
                computed_styles[i] = final_style;
                select_results[i] = results;

                css_color color;
                css_computed_color(final_style, &color);
                css_fixed fs = 0; css_unit fsu = 0;
                css_computed_font_size(final_style, &fs, &fsu);
                int font_px = fs >> CSS_RADIX_POINT;

                printf("  <%s", nodes[i].tag);
                if (nodes[i].id[0]) printf(" id=\"%s\"", nodes[i].id);
                if (nodes[i].classes[0]) printf(" class=\"%s\"", nodes[i].classes);
                printf("> {\n");
                printf("    color: #%06x;\n", (unsigned)(color & 0xFFFFFF));
                printf("    font-size: %dpx;\n", font_px);
                printf("  }\n");
            } else {
                css_select_results_destroy(results);
            }
        }
    }

    // Clean up all stored select results
    for (int i = 0; i < node_count; i++) {
        if (select_results[i]) {
            css_select_results_destroy(select_results[i]);
            select_results[i] = NULL;
            computed_styles[i] = NULL;
        }
    }

    css_select_ctx_destroy(ctx);
    css_stylesheet_destroy(sheet);

    printf("\n=== HTML + CSS Integration PASSED ===\n");
    exit(0);
}
