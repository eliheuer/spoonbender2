// Copyright 2025 the Runebender Authors
// SPDX-License-Identifier: Apache-2.0

//! Runebender Xilem: A font editor built with Xilem
//!
//! This is a port of Runebender from Druid to Xilem, using modern
//! Linebender crates for rendering and UI.

use masonry::properties::types::{AsUnit, UnitPoint};
use masonry::vello::peniko::Color;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use xilem::core::one_of::Either;
use xilem::style::Style;
use xilem::view::{button, flex_col, flex_row, indexed_stack, label, portal, sized_box, transformed, zstack, ChildAlignment, CrossAxisAlignment, FlexExt, ZStackExt};
use xilem::{window, EventLoopBuilder, WidgetView, WindowView, Xilem};

mod actions;
mod cubic_path;
mod data;
mod edit_session;
mod edit_type;
mod entity_id;
mod glyph_renderer;
mod hit_test;
mod mouse;
mod path;
mod point;
mod point_list;
mod quadrant;
mod segment;
mod selection;
mod settings;
mod theme;
mod toolbar;
mod tools;
mod undo;
mod welcome;
mod widgets;
mod workspace;

use data::{AppState, Tab};
use welcome::welcome_view;
use widgets::{coordinate_info_pane, calculate_coordinate_selection, editor_view, glyph_view, grid_toolbar_view, toolbar_view};

/// Entry point for the Runebender Xilem application
pub fn run(event_loop: EventLoopBuilder) -> Result<(), EventLoopError> {
    let mut initial_state = AppState::new();

    // Check for command-line argument (UFO path)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let ufo_path = std::path::PathBuf::from(&args[1]);

        // Validate that the path exists
        if ufo_path.exists() {
            println!("Loading UFO from: {}", ufo_path.display());
            initial_state.load_ufo(ufo_path);
        } else {
            eprintln!("Error: Path does not exist: {}", ufo_path.display());
            eprintln!("Usage: spoonbender [path/to/font.ufo]");
        }
    }

    let app = Xilem::new(initial_state, app_logic);
    app.run_in(event_loop)?;
    Ok(())
}

/// Main application logic - single window with tabs
fn app_logic(state: &mut AppState) -> impl Iterator<Item = WindowView<AppState>> + use<> {
    let content = if state.workspace.is_some() {
        // Font is loaded - show tabbed interface
        Either::A(tabbed_view(state))
    } else {
        // No font loaded - show welcome screen
        Either::B(welcome_view(state))
    };

    // Set initial window size to accommodate 8 columns of glyph cells
    // 8 cells * 120px + 7 gaps * 6px + 6px left + 6px right + ~16px scrollbar = 1030px width
    // Height: comfortable for scrolling glyph grid
    let window_size = LogicalSize::new(1030.0, 800.0);

    // Use a stable window ID stored in state so the window persists across rebuilds
    // This prevents the window from resetting to default size when switching modes
    std::iter::once(
        window(state.main_window_id, "Runebender Xilem", content)
            .with_options(|o| {
                o.with_initial_inner_size(window_size)
                    .on_close(|state: &mut AppState| {
                        state.running = false;
                    })
            })
    )
}

/// Tabbed interface with glyph grid and editor tabs
fn tabbed_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    indexed_stack((
        // Tab 0: Glyph Grid
        glyph_grid_tab(state),
        // Tab 1: Editor
        editor_tab(state),
    ))
    .active(state.active_tab as usize)
}

/// Tab 0: Glyph grid view with header
fn glyph_grid_tab(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    // Just the glyph grid - info sections commented out for now
    flex_col((
        glyph_grid_view(state),
    ))
    .background_color(theme::app::BACKGROUND)

    // TODO: Re-enable header and info sections later if needed
    // flex_col((
    //     // Top margin
    //     sized_box(label("")).height(10.px()),
    //     // Header bar
    //     header_bar(state),
    //     // Selected glyph info bar
    //     selected_glyph_info(state),
    //     // Main content: glyph grid
    //     glyph_grid_view(state),
    // ))
    // .background_color(theme::app::BACKGROUND)
}

/// Tab 1: Editor view with toolbar floating over canvas
fn editor_tab(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
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
                }
            ),
            // Foreground: floating toolbar positioned in top-left with fixed margin
            transformed(
                toolbar_view(current_tool, |state: &mut AppState, tool_id| {
                    state.set_editor_tool(tool_id);
                })
            )
            .translate((MARGIN, MARGIN))
            .alignment(ChildAlignment::SelfAligned(UnitPoint::TOP_LEFT)),
            // Bottom-left: glyph preview pane with fixed margin
            transformed(
                glyph_preview_pane(session_arc.clone(), glyph_name.clone())
            )
            .translate((MARGIN, -MARGIN))
            .alignment(ChildAlignment::SelfAligned(UnitPoint::BOTTOM_LEFT)),
            // Bottom-right: coordinate info pane with fixed margin
            transformed(
                coordinate_info_pane_from_session(&session_arc)
            )
            .translate((-MARGIN, -MARGIN))
            .alignment(ChildAlignment::SelfAligned(UnitPoint::BOTTOM_RIGHT)),
            // Top-right: Grid toolbar for navigation
            transformed(
                grid_toolbar_view(|state: &mut AppState, button| {
                    use widgets::grid_toolbar::GridToolbarButton;
                    match button {
                        GridToolbarButton::Grid => state.close_editor(),
                    }
                })
            )
            .translate((-MARGIN, MARGIN))
            .alignment(ChildAlignment::SelfAligned(UnitPoint::TOP_RIGHT)),
        )))
    } else {
        // No session - show empty view (shouldn't happen)
        Either::B(flex_col((label("No editor session"),)))
    }
}


/// Helper to create coordinate info pane from session data
fn coordinate_info_pane_from_session(session: &Arc<crate::edit_session::EditSession>) -> impl WidgetView<AppState> + use<> {
    println!("[coordinate_info_pane_from_session] Building view with quadrant={:?}", session.coord_selection.quadrant);
    coordinate_info_pane(Arc::clone(session), |state: &mut AppState, updated_session| {
        // Replace the editor session with the updated one
        println!("[coordinate_info_pane callback] Session updated, new quadrant={:?}", updated_session.coord_selection.quadrant);
        state.editor_session = Some(updated_session);
    })
}

/// Header bar with font name and action buttons
fn header_bar(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let font_name = state.font_display_name().unwrap_or_default();

    sized_box(
        flex_row((
            label(font_name).text_size(16.0),
            button(label("Font Info"), |_state: &mut AppState| {
                // TODO: Open font info dialog
                println!("Font info clicked");
            }),
        ))
    ).height(40.px())
}

/// Horizontal info bar showing selected glyph details
fn selected_glyph_info(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let glyph_name = state.selected_glyph.clone().unwrap_or_else(|| "None".to_string());
    let advance = state.selected_glyph_advance()
        .map(|w| format!("{:.0}", w))
        .unwrap_or_else(|| "—".to_string());
    let unicode = state.selected_glyph_unicode()
        .unwrap_or_else(|| "—".to_string());

    // Build compact info display
    let info_text = if state.selected_glyph.is_some() {
        if let Some(workspace) = &state.workspace {
            if let Some(glyph) = workspace.get_glyph(&glyph_name) {
                let bounds = glyph_renderer::glyph_bounds(&glyph);
                let bounds_str = bounds
                    .map(|b| format!("{:.0}×{:.0}", b.width(), b.height()))
                    .unwrap_or_else(|| "empty".to_string());

                format!(
                    "Selected: {} | Unicode: {} | Advance: {} | Contours: {} | Bounds: {}",
                    glyph_name, unicode, advance, glyph.contours.len(), bounds_str
                )
            } else {
                format!("Selected: {} (no data)", glyph_name)
            }
        } else {
            "No font loaded".to_string()
        }
    } else {
        "No glyph selected".to_string()
    };

    sized_box(
        flex_row((
            label(info_text).text_size(16.0),
        ))
    )
    .height(30.px())
}

/// Glyph grid showing all glyphs
fn glyph_grid_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let glyph_names = state.glyph_names();

    // Get UPM from workspace for uniform scaling
    let upm = state.workspace.as_ref()
        .and_then(|w| w.units_per_em)
        .unwrap_or(1000.0);

    // Pre-compute glyph data to avoid capturing state reference
    // Pass the actual Glyph instead of pre-computed BezPath so it can be rendered fresh each time
    let glyph_data: Vec<(String, Option<Arc<workspace::Glyph>>, Vec<char>, usize)> = if let Some(workspace) = &state.workspace {
        glyph_names.iter().map(|name| {
            if let Some(glyph) = workspace.get_glyph(name) {
                let count = glyph.contours.len();
                // Debug logging only for glyph "a"
                if name == "a" {
                    println!("[glyph_grid_view] Glyph 'a' has {} contours", count);
                }
                let codepoints = glyph.codepoints.clone();
                (name.clone(), Some(Arc::new(glyph.clone())), codepoints, count)
            } else {
                (name.clone(), None, Vec::new(), 0)
            }
        }).collect()
    } else {
        glyph_names.iter().map(|name| (name.clone(), None, Vec::new(), 0)).collect()
    };

    // Create rows of glyphs using flex layout (grid doesn't work well with expand)
    // TODO: Make this truly responsive - Xilem 0.4 doesn't have window resize events
    // For now, use a column count that works well with typical window sizes
    let columns = 8; // Works well for 1100-1200px wide windows

    let mut rows_of_cells = Vec::new();
    let selected_glyph = state.selected_glyph.clone();

    for chunk in glyph_data.chunks(columns) {
        let row_items: Vec<_> = chunk.iter()
            .map(|(name, glyph_opt, codepoints, contour_count)| {
                let is_selected = selected_glyph.as_ref() == Some(name);
                glyph_cell(name.clone(), glyph_opt.clone(), codepoints.clone(), is_selected, upm, *contour_count)
            })
            .collect();
        // Add gap between cells in each row
        rows_of_cells.push(flex_row(row_items).gap(6.px()));
    }

    // Wrap in portal for scrolling - now works because data is thread-safe!
    // Add margins matching the 6px gap between grid items
    flex_col((
        sized_box(label("")).height(6.px()), // Top margin - matches grid gaps
        flex_row((
            sized_box(label("")).width(6.px()), // Left margin
            portal(
                flex_col(rows_of_cells)
                    .gap(6.px())
            ),
            sized_box(label("")).width(6.px()), // Right margin
        )),
    ))
}

/// Glyph preview pane showing the rendered glyph
fn glyph_preview_pane(session: Arc<crate::edit_session::EditSession>, glyph_name: String) -> impl WidgetView<AppState> + use<> {
    // Get the glyph outline path from the session
    let mut glyph_path = kurbo::BezPath::new();
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

    sized_box(
        flex_col((
            // Add 4px spacer above glyph preview
            sized_box(label("")).height(4.px()),
            // Glyph preview - use theme color with custom baseline offset
            // Use bounding box centering for visual centering (not advance width)
            if !glyph_path.is_empty() {
                Either::A(glyph_view(glyph_path.clone(), preview_size, preview_size, upm)
                    .color(theme::panel::GLYPH_PREVIEW)
                    .baseline_offset(0.15))
            } else {
                Either::B(label(""))
            },
            // Glyph name and unicode labels - use primary UI text color
            sized_box(
                flex_col((
                    label(glyph_name)
                        .text_size(11.0)
                        .color(theme::text::PRIMARY),
                    label(unicode_display)
                        .text_size(11.0)
                        .color(theme::text::PRIMARY),
                    sized_box(label("")).height(4.px()), // Add 4px spacer below labels for margin
                ))
                .gap(2.px())
            ).height(32.px()), // Increased from 28px to account for spacer
        ))
    )
    .width(160.px())
    .height(180.px())
    .background_color(theme::panel::BACKGROUND)
    .border_color(theme::panel::OUTLINE)
    .border_width(1.5)
    .corner_radius(8.0)
}


/// Individual glyph cell in the grid
///
/// The contour_count parameter is used as a version marker to force widget updates when glyph data changes
fn glyph_cell(glyph_name: String, glyph_opt: Option<Arc<workspace::Glyph>>, codepoints: Vec<char>, is_selected: bool, upm: f64, _contour_count: usize) -> impl WidgetView<AppState> + use<> {
    let name_clone = glyph_name.clone();
    let display_name = if glyph_name.len() > 12 {
        format!("{}...", &glyph_name[..9])
    } else {
        glyph_name.clone()
    };

    // Format Unicode codepoint (use first codepoint if available)
    // Also show contour count (just the number, no label)
    let unicode_display = if let Some(first_char) = codepoints.first() {
        format!("U+{:04X} {}", *first_char as u32, _contour_count)
    } else {
        format!("{}", _contour_count)
    };

    // Convert glyph to BezPath fresh each time the view is built
    // This ensures the preview always shows the latest glyph data
    // Smaller glyph preview (60x60) - 20% smaller than before
    let glyph_view_widget = if let Some(glyph) = glyph_opt {
        let path = glyph_renderer::glyph_to_bezpath(&glyph);
        Either::A(sized_box(
            flex_col((
                sized_box(label("")).height(4.px()), // Add 4px spacer above glyph
                glyph_view(path, 60.0, 60.0, upm)
                    .baseline_offset(0.06), // Move glyph down slightly in grid cell
            ))
        ).height(78.px())) // Reduced height to bring labels closer to glyph
    } else {
        Either::B(sized_box(
            flex_col((
                sized_box(label("")).height(4.px()), // Add 4px spacer above glyph
                label("?").text_size(40.0),
            ))
        ).height(78.px())) // Reduced height to bring labels closer to glyph
    };

    // Create label with glyph name and unicode
    // Use primary UI text color for consistency
    // Smaller text size and minimal gap between lines
    let name_label = label(display_name)
        .text_size(11.0)
        .color(theme::text::PRIMARY);

    let unicode_label = label(unicode_display)
        .text_size(11.0)
        .color(theme::text::PRIMARY);

    // Wrap labels in a centered column with minimal gap, constrained height
    let label_with_spacing = sized_box(
        flex_col((
            name_label,
            unicode_label,
            sized_box(label("")).height(4.px()), // Add 4px spacer below labels for margin
        ))
            .gap(2.px())
    ).height(32.px()); // Increased from 28px to account for spacer

    // Choose colors based on selection state
    let (bg_color, border_color) = if is_selected {
        // Darker forest green background with light green outline when selected
        (theme::grid::CELL_SELECTED_BACKGROUND, theme::grid::CELL_SELECTED_OUTLINE)
    } else {
        // Dark gray background with mid gray outline when not selected
        (theme::grid::CELL_BACKGROUND, theme::grid::CELL_OUTLINE)
    };

    // Glyph cell - fixed size works best with flex layout
    sized_box(
        button(
            flex_col((
                glyph_view_widget,
                label_with_spacing,
            )),
            move |state: &mut AppState| {
                println!("Opening editor for glyph: {}", name_clone);
                state.select_glyph(name_clone.clone());
                state.open_editor(name_clone.clone());
            }
        )
        .background_color(bg_color)
        .border_color(border_color)
    )
    .width(120.px())
    .height(120.px())
}
