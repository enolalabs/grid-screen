#include <wayland-client.h>
#include "ext-foreign-toplevel-list-v1-client.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

struct toplevel {
    char *title;
    char *app_id;
    int minimized;
    int maximized;
    int fullscreen;
    struct toplevel *next;
};

struct context {
    struct ext_foreign_toplevel_list_v1 *manager;
    struct toplevel *list;
    struct toplevel *tail;
};

static void toplevel_handle_title(void *data,
    struct ext_foreign_toplevel_handle_v1 *handle, const char *title) {
    struct toplevel *tl = (struct toplevel *)data;
    if (tl->title) free(tl->title);
    tl->title = title ? strdup(title) : strdup("");
}

static void toplevel_handle_app_id(void *data,
    struct ext_foreign_toplevel_handle_v1 *handle, const char *app_id) {
    struct toplevel *tl = (struct toplevel *)data;
    if (tl->app_id) free(tl->app_id);
    tl->app_id = app_id ? strdup(app_id) : strdup("");
}

static void toplevel_handle_done(void *data,
    struct ext_foreign_toplevel_handle_v1 *handle) {}

static void toplevel_handle_closed(void *data,
    struct ext_foreign_toplevel_handle_v1 *handle) {}

static const struct ext_foreign_toplevel_handle_v1_listener toplevel_listener = {
    .title = toplevel_handle_title,
    .app_id = toplevel_handle_app_id,
    .done = toplevel_handle_done,
    .closed = toplevel_handle_closed,
    .identifier = NULL,
};

static void manager_handle_toplevel(void *data,
    struct ext_foreign_toplevel_list_v1 *manager,
    struct ext_foreign_toplevel_handle_v1 *handle) {
    struct context *ctx = (struct context *)data;
    struct toplevel *tl = calloc(1, sizeof(struct toplevel));
    tl->title = strdup("");
    tl->app_id = strdup("");
    if (ctx->tail) {
        ctx->tail->next = tl;
    } else {
        ctx->list = tl;
    }
    ctx->tail = tl;
    ext_foreign_toplevel_handle_v1_add_listener(handle, &toplevel_listener, ctx);
}

static void manager_handle_finished(void *data,
    struct ext_foreign_toplevel_list_v1 *manager) {}

static const struct ext_foreign_toplevel_list_v1_listener manager_listener = {
    .toplevel = manager_handle_toplevel,
    .finished = manager_handle_finished,
};

static void registry_handle_global(void *data, struct wl_registry *registry,
    uint32_t name, const char *interface, uint32_t version) {
    struct context *ctx = (struct context *)data;
    if (strcmp(interface, ext_foreign_toplevel_list_v1_interface.name) == 0) {
        ctx->manager = wl_registry_bind(registry, name,
            &ext_foreign_toplevel_list_v1_interface, 1);
        ext_foreign_toplevel_list_v1_add_listener(ctx->manager, &manager_listener, ctx);
    }
}

static void registry_handle_global_remove(void *data,
    struct wl_registry *registry, uint32_t name) {}

static const struct wl_registry_listener registry_listener = {
    .global = registry_handle_global,
    .global_remove = registry_handle_global_remove,
};

char *enumerate_toplevels_json(void) {
    struct wl_display *display = wl_display_connect(NULL);
    if (!display) return strdup("[]");

    struct wl_registry *registry = wl_display_get_registry(display);
    struct context ctx = {0};

    wl_registry_add_listener(registry, &registry_listener, &ctx);
    wl_display_roundtrip(display); // get globals

    if (!ctx.manager) {
        wl_display_disconnect(display);
        return strdup("[]");
    }

    // Roundtrip to receive toplevels
    wl_display_roundtrip(display);

    // Another roundtrip for toplevel details (titles, app_ids)
    wl_display_roundtrip(display);

    // Build JSON
    char *buf = malloc(65536);
    char *p = buf;
    p += snprintf(p, 65536, "[");
    struct toplevel *tl = ctx.list;
    int first = 1;
    while (tl) {
        if (!first) p += snprintf(p, 65536 - (p - buf), ",");
        first = 0;
        p += snprintf(p, 65536 - (p - buf),
            "{\"title\":\"%s\",\"app_id\":\"%s\",\"minimized\":false,\"maximized\":false,\"fullscreen\":false}",
            tl->title ? tl->title : "",
            tl->app_id ? tl->app_id : "");
        struct toplevel *next = tl->next;
        free(tl->title);
        free(tl->app_id);
        free(tl);
        tl = next;
    }
    p += snprintf(p, 65536 - (p - buf), "]");

    if (ctx.manager) ext_foreign_toplevel_list_v1_destroy(ctx.manager);
    wl_registry_destroy(registry);
    wl_display_disconnect(display);

    return buf;
}
