/*
 * Bat_OS — paint shim. DisplayListPlayerSkia has hidden visibility
 * in liblagom-web.so, so we can't link against it from a standalone
 * utility. This file re-includes the header (which is fine inside
 * the binary's translation units) and exposes a plain free function
 * that DumpDOM.cpp calls. Both files compile into dump-html-tokens.
 */

#include <LibWeb/Painting/DisplayList.h>
#include <LibWeb/Painting/DisplayListPlayerSkia.h>
#include <LibWeb/Painting/ScrollFrame.h>
#include <LibGfx/PaintingSurface.h>

void batos_paint_into_surface(
    Web::Painting::DisplayList& display_list,
    Web::Painting::ScrollStateSnapshot const& scroll_state,
    RefPtr<Gfx::PaintingSurface> surface);

void batos_paint_into_surface(
    Web::Painting::DisplayList& display_list,
    Web::Painting::ScrollStateSnapshot const& scroll_state,
    RefPtr<Gfx::PaintingSurface> surface)
{
    Web::Painting::DisplayListPlayerSkia player;
    player.execute(display_list, scroll_state, surface);
}
