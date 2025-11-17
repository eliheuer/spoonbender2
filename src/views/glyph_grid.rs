use std::sync::Arc;

use xilem::WidgetView;
use xilem::core::one_of::Either;
use xilem::style::Style;
use xilem::view::{button, flex_col, flex_row, label, portal, sized_box};

use masonry::properties::types::AsUnit;

use crate::components::glyph_view;
use crate::data::AppState;
use crate::glyph_renderer;
use crate::theme;
use crate::workspace;

/// Tab 0: Glyph grid view with header
pub fn glyph_grid_tab(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    flex_col((glyph_grid_view(state),)).background_color(theme::app::BACKGROUND)
}

/// Glyph grid showing all glyphs
fn glyph_grid_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let glyph_names = state.glyph_names();

    // Get UPM from workspace for uniform scaling
    let upm = state
        .workspace
        .as_ref()
        .and_then(|w| w.units_per_em)
        .unwrap_or(1000.0);

    // Pre-compute glyph data
    let glyph_data: Vec<(String, Option<Arc<workspace::Glyph>>, Vec<char>, usize)> =
        if let Some(workspace) = &state.workspace {
            glyph_names
                .iter()
                .map(|name| {
                    if let Some(glyph) = workspace.get_glyph(name) {
                        let count = glyph.contours.len();
                        let codepoints = glyph.codepoints.clone();
                        (
                            name.clone(),
                            Some(Arc::new(glyph.clone())),
                            codepoints,
                            count,
                        )
                    } else {
                        (name.clone(), None, Vec::new(), 0)
                    }
                })
                .collect()
        } else {
            glyph_names
                .iter()
                .map(|name| (name.clone(), None, Vec::new(), 0))
                .collect()
        };

    let columns = 8;
    let selected_glyph = state.selected_glyph.clone();

    let rows_of_cells = glyph_data
        .chunks(columns)
        .map(|chunk| {
            let row_items: Vec<_> = chunk
                .iter()
                .map(|(name, glyph_opt, codepoints, contour_count)| {
                    let is_selected = selected_glyph.as_ref() == Some(name);
                    glyph_cell(
                        name.clone(),
                        glyph_opt.clone(),
                        codepoints.clone(),
                        is_selected,
                        upm,
                        *contour_count,
                    )
                })
                .collect();
            flex_row(row_items).gap(6.px())
        })
        .collect::<Vec<_>>();

    flex_col((
        sized_box(label("")).height(6.px()),
        flex_row((
            sized_box(label("")).width(6.px()),
            portal(flex_col(rows_of_cells).gap(6.px())),
            sized_box(label("")).width(6.px()),
        )),
    ))
}

/// Individual glyph cell in the grid
fn glyph_cell(
    glyph_name: String,
    glyph_opt: Option<Arc<workspace::Glyph>>,
    codepoints: Vec<char>,
    is_selected: bool,
    upm: f64,
    contour_count: usize,
) -> impl WidgetView<AppState> + use<> {
    let name_clone = glyph_name.clone();
    let display_name = if glyph_name.len() > 12 {
        format!("{}...", &glyph_name[..9])
    } else {
        glyph_name.clone()
    };

    let unicode_display = if let Some(first_char) = codepoints.first() {
        format!("U+{:04X} {}", *first_char as u32, contour_count)
    } else {
        format!("{}", contour_count)
    };

    let glyph_view_widget = if let Some(glyph) = glyph_opt {
        let path = glyph_renderer::glyph_to_bezpath(&glyph);
        Either::A(
            sized_box(flex_col((
                sized_box(label("")).height(4.px()),
                glyph_view(path, 60.0, 60.0, upm).baseline_offset(0.06),
            )))
            .height(78.px()),
        )
    } else {
        Either::B(
            sized_box(flex_col((
                sized_box(label("")).height(4.px()),
                label("?").text_size(40.0),
            )))
            .height(78.px()),
        )
    };

    let name_label = label(display_name)
        .text_size(11.0)
        .color(theme::text::PRIMARY);

    let unicode_label = label(unicode_display)
        .text_size(11.0)
        .color(theme::text::PRIMARY);

    let label_with_spacing = sized_box(
        flex_col((
            name_label,
            unicode_label,
            sized_box(label("")).height(4.px()),
        ))
        .gap(2.px()),
    )
    .height(32.px());

    let (bg_color, border_color) = if is_selected {
        (
            theme::grid::CELL_SELECTED_BACKGROUND,
            theme::grid::CELL_SELECTED_OUTLINE,
        )
    } else {
        (theme::grid::CELL_BACKGROUND, theme::grid::CELL_OUTLINE)
    };

    sized_box(
        button(
            flex_col((glyph_view_widget, label_with_spacing)),
            move |state: &mut AppState| {
                state.select_glyph(name_clone.clone());
                state.open_editor(name_clone.clone());
            },
        )
        .background_color(bg_color)
        .border_color(border_color),
    )
    .width(120.px())
    .height(120.px())
}
