use std::sync::Arc;

use kurbo::BezPath;
use xilem::WidgetView;
use xilem::core::one_of::Either;
use xilem::style::Style;
use xilem::view::{ChildAlignment, ZStackExt, flex_col, label, sized_box, transformed, zstack};

use masonry::properties::types::AsUnit;
use masonry::properties::types::UnitPoint;

use crate::components::grid_toolbar::GridToolbarButton;
use crate::components::{
    coordinate_panel, editor_view, glyph_view, grid_toolbar_view, toolbar_view,
};
use crate::data::AppState;
use crate::theme;

/// Tab 1: Editor view with toolbar floating over canvas
pub fn editor_tab(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    if let Some(session) = &state.editor_session {
        let current_tool = session.current_tool.id();
        let glyph_name = session.glyph_name.clone();
        let session_arc = Arc::new(session.clone());

        const MARGIN: f64 = 16.0; // Fixed 16px margin for all panels

        // Use zstack to layer UI elements over the canvas
        Either::A(zstack((
            // Background: the editor canvas (full screen)
            editor_view(
                session_arc.clone(),
                |state: &mut AppState, updated_session| {
                    state.update_editor_session(updated_session);
                },
            ),
            // Foreground: floating toolbar positioned in top-left with fixed margin
            transformed(toolbar_view(
                current_tool,
                |state: &mut AppState, tool_id| {
                    state.set_editor_tool(tool_id);
                },
            ))
            .translate((MARGIN, MARGIN))
            .alignment(ChildAlignment::SelfAligned(UnitPoint::TOP_LEFT)),
            // Bottom-left: glyph preview pane with fixed margin
            transformed(glyph_preview_pane(session_arc.clone(), glyph_name.clone()))
                .translate((MARGIN, -MARGIN))
                .alignment(ChildAlignment::SelfAligned(UnitPoint::BOTTOM_LEFT)),
            // Bottom-right: coordinate panel with fixed margin
            transformed(coordinate_panel_from_session(&session_arc))
                .translate((-MARGIN, -MARGIN))
                .alignment(ChildAlignment::SelfAligned(UnitPoint::BOTTOM_RIGHT)),
            // Top-right: Grid toolbar for navigation
            transformed(grid_toolbar_view(
                |state: &mut AppState, button| match button {
                    GridToolbarButton::Grid => state.close_editor(),
                },
            ))
            .translate((-MARGIN, MARGIN))
            .alignment(ChildAlignment::SelfAligned(UnitPoint::TOP_RIGHT)),
        )))
    } else {
        // No session - show empty view (shouldn't happen)
        Either::B(flex_col((label("No editor session"),)))
    }
}

/// Helper to create coordinate panel from session data
fn coordinate_panel_from_session(
    session: &Arc<crate::edit_session::EditSession>,
) -> impl WidgetView<AppState> + use<> {
    tracing::debug!(
        "[coordinate_panel_from_session] Building view with quadrant={:?}",
        session.coord_selection.quadrant
    );
    coordinate_panel(
        Arc::clone(session),
        |state: &mut AppState, updated_session| {
            tracing::debug!(
                "[coordinate_panel callback] Session updated, new quadrant={:?}",
                updated_session.coord_selection.quadrant
            );
            state.editor_session = Some(updated_session);
        },
    )
}

/// Glyph preview pane showing the rendered glyph
fn glyph_preview_pane(
    session: Arc<crate::edit_session::EditSession>,
    glyph_name: String,
) -> impl WidgetView<AppState> + use<> {
    // Get the glyph outline path from the session
    let mut glyph_path = BezPath::new();
    for path in session.paths.iter() {
        glyph_path.extend(path.to_bezpath());
    }

    // Make the preview larger to fill more space
    let preview_size = 150.0;
    let upm = session.ascender - session.descender;

    // Format Unicode codepoint (use first codepoint if available)
    let unicode_display = if let Some(first_char) = session.glyph.codepoints.first() {
        format!("U+{:04X}", *first_char as u32)
    } else {
        String::new()
    };

    sized_box(flex_col((
        // Add 4px spacer above glyph preview
        sized_box(label("")).height(4.px()),
        // Glyph preview - use theme color with custom baseline offset
        if !glyph_path.is_empty() {
            Either::A(
                glyph_view(glyph_path.clone(), preview_size, preview_size, upm)
                    .color(theme::panel::GLYPH_PREVIEW)
                    .baseline_offset(0.15),
            )
        } else {
            Either::B(label(""))
        },
        // Glyph name and unicode labels - use primary UI text color
        sized_box(
            flex_col((
                label(glyph_name)
                    .text_size(18.0)
                    .color(theme::text::PRIMARY),
                label(unicode_display)
                    .text_size(18.0)
                    .color(theme::text::PRIMARY),
                sized_box(label("")).height(4.px()),
            ))
            .gap(2.px()),
        )
        .height(32.px()),
    )))
    .width(160.px())
    .height(180.px())
    .background_color(theme::panel::BACKGROUND)
    .border_color(theme::panel::OUTLINE)
    .border_width(1.5)
    .corner_radius(8.0)
}
