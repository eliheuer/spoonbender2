// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Spoonbender: A font editor built with Xilem
//!
//! This is a port of Runebender from Druid to Xilem, using modern
//! Linebender crates for rendering and UI.

use masonry::properties::types::{AsUnit, UnitPoint};
use masonry::vello::peniko::Color;
use std::sync::Arc;
use winit::error::EventLoopError;
use xilem::core::one_of::Either;
use xilem::style::Style;
use xilem::view::{button, flex_col, flex_row, label, portal, sized_box, zstack, zstack_item, ChildAlignment, ZStackExt};
use xilem::{window, EventLoopBuilder, WidgetView, WindowView, Xilem};

mod actions;
mod cubic_path;
mod data;
mod edit_session;
mod edit_type;
mod editor_widget;
mod entity_id;
mod glyph_renderer;
mod glyph_widget;
mod hit_test;
mod mouse;
mod path;
mod point;
mod point_list;
mod selection;
mod theme;
mod toolbar;
mod toolbar_widget;
mod tools;
mod undo;
mod workspace;

use data::AppState;
use editor_widget::editor_view;
use glyph_widget::glyph_view;
use toolbar_widget::toolbar_view;

/// Entry point for the Spoonbender application
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

/// Main application logic - builds multiple windows
fn app_logic(state: &mut AppState) -> impl Iterator<Item = WindowView<AppState>> + use<> {
    let main_window_view = if state.workspace.is_some() {
        // Font is loaded - show main editor view
        Either::A(main_editor_view(state))
    } else {
        // No font loaded - show welcome screen
        Either::B(welcome_view(state))
    };

    let main_window = window(state.main_window_id, "Spoonbender", main_window_view)
        .with_options(|o| o.on_close(|state: &mut AppState| {
            state.running = false;
        }));

    // Create editor windows for each open session
    let editor_windows = state.editor_sessions.iter().map(|(window_id, (glyph_name, session))| {
        let window_id = *window_id;
        let window_title = format!("Edit: {}", glyph_name);

        window(window_id, window_title, editor_window_view(session.clone()))
            .with_options(move |o| o.on_close(move |state: &mut AppState| {
                state.close_editor(window_id);
            }))
    });

    std::iter::once(main_window)
        .chain(editor_windows)
        .collect::<Vec<_>>()
        .into_iter()
}

/// Editor window view with toolbar floating over canvas
fn editor_window_view(session: Arc<crate::edit_session::EditSession>) -> impl WidgetView<AppState> {
    let current_tool = session.current_tool.id();

    // Use zstack to layer the toolbar over the canvas
    zstack((
        // Background: the editor canvas (full screen)
        editor_view(session),
        // Foreground: floating toolbar positioned in top-left
        toolbar_view(current_tool, |state: &mut AppState, tool_id| {
            state.set_editor_tool(tool_id);
        })
        .alignment(ChildAlignment::SelfAligned(UnitPoint::new(0.03, 0.03))),  // 3% from top-left (equal margins)
    ))
}

/// Welcome screen shown when no font is loaded
fn welcome_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let error_text = state
        .error_message
        .as_ref()
        .map(|msg| format!("Error: {}", msg))
        .unwrap_or_default();

    flex_col((
        label("Spoonbender - Font Editor").text_size(16.0),
        label("No font loaded").text_size(16.0),
        label(error_text).text_size(16.0),
        button(label("Open UFO..."), |state: &mut AppState| {
            state.open_font_dialog();
        }),
        button(label("New Font"), |state: &mut AppState| {
            state.create_new_font();
        }),
    ))
}

/// Main editor view with sidebar and glyph grid
fn main_editor_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    flex_col((
        // Top margin
        sized_box(label("")).height(10.px()),
        // Header bar
        header_bar(state),
        // Selected glyph info bar
        selected_glyph_info(state),
        // Main content: glyph grid
        glyph_grid_view(state),
    ))
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
    let glyph_count = glyph_names.len();

    // Get UPM from workspace for uniform scaling
    let upm = state.workspace.as_ref()
        .and_then(|w| w.units_per_em)
        .unwrap_or(1000.0);

    // Pre-compute glyph data to avoid capturing state reference
    let glyph_data: Vec<_> = if let Some(workspace) = &state.workspace {
        glyph_names.iter().map(|name| {
            let path = workspace.get_glyph(name)
                .map(|g| glyph_renderer::glyph_to_bezpath(&g));
            (name.clone(), path)
        }).collect()
    } else {
        glyph_names.iter().map(|name| (name.clone(), None)).collect()
    };

    // Create rows of glyphs using flex layout (grid doesn't work well with expand)
    let columns = 9;
    let mut rows_of_cells = Vec::new();
    let selected_glyph = state.selected_glyph.clone();

    for chunk in glyph_data.chunks(columns) {
        let row_items: Vec<_> = chunk.iter()
            .map(|(name, path_opt)| {
                let is_selected = selected_glyph.as_ref() == Some(name);
                glyph_cell(name.clone(), path_opt.clone(), is_selected, upm)
            })
            .collect();
        rows_of_cells.push(flex_row(row_items));
    }

    flex_col((
        label(format!("{} glyphs", glyph_count)).text_size(16.0),
        // Spacer between label and grid
        sized_box(label("")).height(10.px()),
        // Wrap in portal for scrolling - now works because data is thread-safe!
        portal(flex_col(rows_of_cells)),
    ))
}

/// Individual glyph cell in the grid
fn glyph_cell(glyph_name: String, path_opt: Option<kurbo::BezPath>, is_selected: bool, upm: f64) -> impl WidgetView<AppState> + use<> {
    let name_clone = glyph_name.clone();
    let display_name = if glyph_name.len() > 12 {
        format!("{}...", &glyph_name[..9])
    } else {
        glyph_name.clone()
    };

    // Create glyph view widget from pre-computed path
    let glyph_view_widget = if let Some(path) = path_opt {
        Either::A(glyph_view(path, 100.0, 100.0, upm))
    } else {
        Either::B(label("?").text_size(60.0))
    };

    // Create label (selection styling will be on the cell background)
    let name_label = label(display_name).text_size(16.0);

    // Wrap label with bottom spacing
    let label_with_spacing = sized_box(name_label).height(32.px());

    // Choose colors based on selection state
    let (bg_color, border_color) = if is_selected {
        // Darker forest green background with light green outline when selected
        (Color::from_rgb8(20, 100, 20), Color::from_rgb8(144, 238, 144))
    } else {
        // Dark gray background with mid gray outline when not selected
        (Color::from_rgb8(50, 50, 50), Color::from_rgb8(100, 100, 100))
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
