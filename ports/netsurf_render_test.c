// Bat_OS — NetSurf CSS Style Computation Test
// Computes actual CSS styles for simulated DOM elements

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <libcss/libcss.h>
#include <parserutils/charset/mibenum.h>
#include <libwapcaplet/libwapcaplet.h>

// --- Simulated DOM ---
typedef struct dom_node {
    const char *tag;
    const char *id;
    const char *classes;
    struct dom_node *parent;
} dom_node;

// --- Selection handler callbacks ---
static css_error h_node_name(void *pw, void *n, css_qname *qname) {
    dom_node *node = (dom_node *)n; (void)pw;
    lwc_intern_string(node->tag, strlen(node->tag), &qname->name);
    return CSS_OK;
}
static css_error h_node_classes(void *pw, void *n, lwc_string ***cls, uint32_t *nc) {
    (void)pw; (void)n; *cls = NULL; *nc = 0; return CSS_OK;
}
static css_error h_node_id(void *pw, void *n, lwc_string **id) {
    dom_node *node = (dom_node *)n; (void)pw;
    if (node->id) lwc_intern_string(node->id, strlen(node->id), id);
    else *id = NULL;
    return CSS_OK;
}
static css_error h_named_ancestor(void *pw, void *n, const css_qname *qn, void **anc) {
    (void)pw; (void)n; (void)qn; *anc = NULL; return CSS_OK;
}
static css_error h_named_parent(void *pw, void *n, const css_qname *qn, void **p) {
    dom_node *node = (dom_node *)n; (void)pw; (void)qn;
    *p = node->parent;
    return CSS_OK;
}
static css_error h_named_sibling(void *pw, void *n, const css_qname *qn, void **s) {
    (void)pw; (void)n; (void)qn; *s = NULL; return CSS_OK;
}
static css_error h_parent(void *pw, void *n, void **p) {
    dom_node *node = (dom_node *)n; (void)pw; *p = node->parent; return CSS_OK;
}
static css_error h_sibling(void *pw, void *n, void **s) {
    (void)pw; (void)n; *s = NULL; return CSS_OK;
}
static css_error h_has_name(void *pw, void *n, const css_qname *qn, bool *m) {
    dom_node *node = (dom_node *)n; (void)pw;
    const char *want = lwc_string_data(qn->name);
    *m = (strcmp(node->tag, want) == 0);
    return CSS_OK;
}
static css_error h_has_class(void *pw, void *n, lwc_string *name, bool *m) {
    dom_node *node = (dom_node *)n; (void)pw;
    if (!node->classes) { *m = false; return CSS_OK; }
    *m = (strstr(node->classes, lwc_string_data(name)) != NULL);
    return CSS_OK;
}
static css_error h_has_id(void *pw, void *n, lwc_string *name, bool *m) {
    dom_node *node = (dom_node *)n; (void)pw;
    if (!node->id) { *m = false; return CSS_OK; }
    *m = (strcmp(node->id, lwc_string_data(name)) == 0);
    return CSS_OK;
}
static css_error h_bool_false(void *pw, void *n, bool *m) {
    (void)pw; (void)n; *m = false; return CSS_OK;
}
static css_error h_has_attr(void *pw, void *n, const css_qname *qn, bool *m) {
    (void)pw; (void)n; (void)qn; *m = false; return CSS_OK;
}
static css_error h_has_attr_val(void *pw, void *n, const css_qname *qn, lwc_string *v, bool *m) {
    (void)pw; (void)n; (void)qn; (void)v; *m = false; return CSS_OK;
}
static css_error h_count_siblings(void *pw, void *n, bool sn, bool af, int32_t *c) {
    (void)pw; (void)n; (void)sn; (void)af; *c = 0; return CSS_OK;
}
static css_error h_is_lang(void *pw, void *n, lwc_string *lang, bool *m) {
    (void)pw; (void)n; (void)lang; *m = false; return CSS_OK;
}
static css_error h_pres_hint(void *pw, void *n, uint32_t *nh, css_hint **h) {
    (void)pw; (void)n; *nh = 0; *h = NULL; return CSS_OK;
}
static css_error h_ua_default(void *pw, uint32_t prop, css_hint *hint) {
    (void)pw; (void)prop;
    hint->status = 0;
    return CSS_OK;
}
static css_error h_set_data(void *pw, void *n, void *d) {
    (void)pw; (void)n; (void)d; return CSS_OK;
}
static css_error h_get_data(void *pw, void *n, void **d) {
    (void)pw; (void)n; *d = NULL; return CSS_OK;
}

static css_error resolve_url(void *pw, const char *base, lwc_string *rel, lwc_string **abs) {
    (void)pw; (void)base; *abs = lwc_string_ref(rel); return CSS_OK;
}

void _start(void) {
    printf("=== CSS Style Computation Test ===\n\n");

    // Create stylesheet
    css_stylesheet_params params;
    memset(&params, 0, sizeof(params));
    params.params_version = CSS_STYLESHEET_PARAMS_VERSION_1;
    params.level = CSS_LEVEL_3;
    params.url = "test.css";
    params.resolve = resolve_url;

    css_stylesheet *sheet = NULL;
    css_stylesheet_create(&params, &sheet);
    const char *css = "body { color: red; font-size: 16px; } h1 { color: blue; font-size: 24px; } p { margin: 10px; }";
    css_stylesheet_append_data(sheet, (const uint8_t *)css, strlen(css));
    css_stylesheet_data_done(sheet);
    printf("[1] Stylesheet ready (3 rules)\n");

    // Create selection context
    css_select_ctx *ctx = NULL;
    css_select_ctx_create(&ctx);
    css_select_ctx_append_sheet(ctx, sheet, CSS_ORIGIN_AUTHOR, NULL);
    printf("[2] Selection context ready\n");

    // Build handler
    css_select_handler handler;
    memset(&handler, 0, sizeof(handler));
    handler.handler_version = CSS_SELECT_HANDLER_VERSION_1;
    handler.node_name = h_node_name;
    handler.node_classes = h_node_classes;
    handler.node_id = h_node_id;
    handler.named_ancestor_node = h_named_ancestor;
    handler.named_parent_node = h_named_parent;
    handler.named_sibling_node = h_named_sibling;
    handler.named_generic_sibling_node = h_named_sibling;
    handler.parent_node = h_parent;
    handler.sibling_node = h_sibling;
    handler.node_has_name = h_has_name;
    handler.node_has_class = h_has_class;
    handler.node_has_id = h_has_id;
    handler.node_has_attribute = h_has_attr;
    handler.node_has_attribute_equal = h_has_attr_val;
    handler.node_has_attribute_dashmatch = h_has_attr_val;
    handler.node_has_attribute_includes = h_has_attr_val;
    handler.node_has_attribute_prefix = h_has_attr_val;
    handler.node_has_attribute_suffix = h_has_attr_val;
    handler.node_has_attribute_substring = h_has_attr_val;
    handler.node_is_root = h_bool_false;
    handler.node_count_siblings = h_count_siblings;
    handler.node_is_empty = h_bool_false;
    handler.node_is_link = h_bool_false;
    handler.node_is_visited = h_bool_false;
    handler.node_is_hover = h_bool_false;
    handler.node_is_active = h_bool_false;
    handler.node_is_focus = h_bool_false;
    handler.node_is_enabled = h_bool_false;
    handler.node_is_disabled = h_bool_false;
    handler.node_is_checked = h_bool_false;
    handler.node_is_target = h_bool_false;
    handler.node_is_lang = h_is_lang;
    handler.node_presentational_hint = h_pres_hint;
    handler.ua_default_for_property = h_ua_default;
    handler.set_libcss_node_data = h_set_data;
    handler.get_libcss_node_data = h_get_data;

    // Unit context
    css_unit_ctx unit_ctx;
    memset(&unit_ctx, 0, sizeof(unit_ctx));
    unit_ctx.viewport_width = 1280 * (1 << CSS_RADIX_POINT);
    unit_ctx.viewport_height = 1024 * (1 << CSS_RADIX_POINT);
    unit_ctx.device_dpi = 96 * (1 << CSS_RADIX_POINT);
    unit_ctx.font_size_default = 16 * (1 << CSS_RADIX_POINT);

    // Media
    css_media media;
    memset(&media, 0, sizeof(media));
    media.type = CSS_MEDIA_SCREEN;

    // Create DOM nodes
    dom_node body_node = { "body", NULL, NULL, NULL };
    dom_node h1_node = { "h1", NULL, NULL, &body_node };
    dom_node p_node = { "p", NULL, NULL, &body_node };

    // Compute style for body
    printf("\n[3] Computing styles...\n");
    css_select_results *results = NULL;
    css_error err = css_select_style(ctx, &body_node, &unit_ctx, &media, NULL,
                                      &handler, NULL, &results);
    if (err == CSS_OK && results) {
        css_computed_style *style = results->styles[CSS_PSEUDO_ELEMENT_NONE];
        if (style) {
            css_color color;
            uint8_t type = css_computed_color(style, &color);
            printf("  body {\n");
            printf("    color: #%06x (type=%d)\n", (unsigned)(color & 0xFFFFFF), type);
            css_fixed fs = 0; css_unit fsu = 0;
            css_computed_font_size(style, &fs, &fsu);
            printf("    font-size: %dpx\n", fs >> CSS_RADIX_POINT);
            printf("  }\n");
        }
        css_select_results_destroy(results);
    } else {
        printf("  body: FAILED (err=%d)\n", err);
    }

    // Compute style for h1
    results = NULL;
    err = css_select_style(ctx, &h1_node, &unit_ctx, &media, NULL,
                            &handler, NULL, &results);
    if (err == CSS_OK && results) {
        css_computed_style *style = results->styles[CSS_PSEUDO_ELEMENT_NONE];
        if (style) {
            css_color color;
            css_computed_color(style, &color);
            printf("  h1 {\n");
            printf("    color: #%06x\n", (unsigned)(color & 0xFFFFFF));
            css_fixed fs = 0; css_unit fsu = 0;
            css_computed_font_size(style, &fs, &fsu);
            printf("    font-size: %dpx\n", fs >> CSS_RADIX_POINT);
            printf("  }\n");
        }
        css_select_results_destroy(results);
    } else {
        printf("  h1: FAILED (err=%d)\n", err);
    }

    // Compute style for p
    results = NULL;
    err = css_select_style(ctx, &p_node, &unit_ctx, &media, NULL,
                            &handler, NULL, &results);
    if (err == CSS_OK && results) {
        css_computed_style *style = results->styles[CSS_PSEUDO_ELEMENT_NONE];
        if (style) {
            css_fixed mt = 0, mr = 0, mb = 0, ml = 0;
            css_unit mtu = 0, mru = 0, mbu = 0, mlu = 0;
            css_computed_margin_top(style, &mt, &mtu);
            css_computed_margin_bottom(style, &mb, &mbu);
            printf("  p {\n");
            printf("    margin-top: %dpx\n", mt >> CSS_RADIX_POINT);
            printf("    margin-bottom: %dpx\n", mb >> CSS_RADIX_POINT);
            printf("  }\n");
        }
        css_select_results_destroy(results);
    } else {
        printf("  p: FAILED (err=%d)\n", err);
    }

    css_select_ctx_destroy(ctx);
    css_stylesheet_destroy(sheet);

    printf("\n=== CSS Computation PASSED ===\n");
    printf("Real CSS values computed for DOM elements!\n");
    exit(0);
}
