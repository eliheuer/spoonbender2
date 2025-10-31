// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Spoonbender: A font editor built with Xilem
//!
//! This is a port of Runebender from Druid to Xilem, using modern
//! Linebender crates for rendering and UI.

use masonry::properties::types::AsUnit;
use winit::error::EventLoopError;
use xilem::core::one_of::Either;
use xilem::view::{button, flex_col, flex_row, label, sized_box};
use xilem::{EventLoopBuilder, WidgetView, WindowOptions, Xilem};

mod actions;
mod data;
mod glyph_renderer;
mod glyph_widget;
mod workspace;

use data::AppState;
use glyph_widget::glyph_view;

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

    let window_options = WindowOptions::new("Spoonbender")
        .with_initial_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0));

    let app = Xilem::new_simple(initial_state, app_logic, window_options);
    app.run_in(event_loop)?;
    Ok(())
}

/// Main application logic - builds the view tree from app state
fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    if state.workspace.is_some() {
        // Font is loaded - show main editor view
        Either::A(main_editor_view(state))
    } else {
        // No font loaded - show welcome screen
        Either::B(welcome_view(state))
    }
}

/// Welcome screen shown when no font is loaded
fn welcome_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let error_text = state
        .error_message
        .as_ref()
        .map(|msg| format!("Error: {}", msg))
        .unwrap_or_default();

    flex_col((
        label("Spoonbender - Font Editor").text_size(24.0),
        label("No font loaded").text_size(16.0),
        label(error_text).text_size(14.0),
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
        // Header bar
        header_bar(state),
        // Main content: sidebar + glyph grid
        flex_row((
            sidebar_view(state),
            glyph_grid_view(state),
        )),
    ))
}

/// Header bar with font name and action buttons
fn header_bar(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let font_name = state.font_display_name().unwrap_or_default();

    sized_box(
        flex_row((
            label(font_name).text_size(18.0),
            button(label("Font Info"), |_state: &mut AppState| {
                // TODO: Open font info dialog
                println!("Font info clicked");
            }),
        ))
    ).height(40.px())
}

/// Sidebar showing selected glyph details
fn sidebar_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let glyph_name = state.selected_glyph.clone().unwrap_or_else(|| "None".to_string());
    println!("Sidebar rebuilding, selected_glyph: {:?}", state.selected_glyph);
    let advance = state.selected_glyph_advance()
        .map(|w| format!("{:.0}", w))
        .unwrap_or_else(|| "—".to_string());
    let unicode = state.selected_glyph_unicode()
        .unwrap_or_else(|| "—".to_string());

    // Build glyph preview section
    let (glyph_preview, info_labels) = if let (Some(workspace), Some(name)) = (&state.workspace, &state.selected_glyph) {
        if let Some(glyph) = workspace.get_glyph(name) {
            let path = glyph_renderer::glyph_to_bezpath(glyph);
            let bounds = glyph_renderer::glyph_bounds(glyph);
            let path_info = format!("Contours: {}\nBounds: {}",
                glyph.contours.len(),
                bounds.map(|b| format!("{:.0}×{:.0}", b.width(), b.height()))
                    .unwrap_or_else(|| "empty".to_string())
            );

            (
                Either::A(glyph_view(path, 150.0, 150.0)),
                (
                    label(format!("Name: {}", glyph_name)).text_size(12.0),
                    label(format!("Unicode: {}", unicode)).text_size(12.0),
                    label(format!("Advance: {}", advance)).text_size(12.0),
                    label(path_info).text_size(10.0),
                )
            )
        } else {
            (
                Either::B(label("No glyph data").text_size(14.0)),
                (
                    label(format!("Name: {}", glyph_name)).text_size(12.0),
                    label("").text_size(12.0),
                    label("").text_size(12.0),
                    label("").text_size(10.0),
                )
            )
        }
    } else {
        (
            Either::B(label("No selection").text_size(14.0)),
            (
                label("Select a glyph").text_size(12.0),
                label("").text_size(12.0),
                label("").text_size(12.0),
                label("").text_size(10.0),
            )
        )
    };

    sized_box(
        flex_col((
            label("Selected Glyph").text_size(16.0),
            glyph_preview,
            info_labels.0,
            info_labels.1,
            info_labels.2,
            info_labels.3,
        ))
    )
    .width(250.px())
    .expand_height()
}

/// Glyph grid showing all glyphs
fn glyph_grid_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let glyph_names = state.glyph_names();
    let glyph_count = glyph_names.len();

    // Pre-compute glyph data to avoid capturing state reference
    let glyph_data: Vec<_> = if let Some(workspace) = &state.workspace {
        glyph_names.iter().map(|name| {
            let path = workspace.get_glyph(name)
                .map(|g| glyph_renderer::glyph_to_bezpath(g));
            (name.clone(), path)
        }).collect()
    } else {
        glyph_names.iter().map(|name| (name.clone(), None)).collect()
    };

    // Create rows of glyphs - use flex instead of grid to have more control
    let columns = 6;
    let mut rows_of_cells = Vec::new();
    let selected_glyph = state.selected_glyph.clone();

    for chunk in glyph_data.chunks(columns) {
        let row_items: Vec<_> = chunk.iter()
            .map(|(name, path_opt)| {
                let is_selected = selected_glyph.as_ref() == Some(name);
                glyph_cell(name.clone(), path_opt.clone(), is_selected)
            })
            .collect();
        rows_of_cells.push(flex_row(row_items));
    }

    flex_col((
        label(format!("{} glyphs", glyph_count)).text_size(14.0),
        flex_col(rows_of_cells),
    ))
}

/// Individual glyph cell in the grid
fn glyph_cell(glyph_name: String, path_opt: Option<kurbo::BezPath>, is_selected: bool) -> impl WidgetView<AppState> + use<> {
    let name_clone = glyph_name.clone();
    let display_name = if glyph_name.len() > 12 {
        format!("{}...", &glyph_name[..9])
    } else {
        glyph_name.clone()
    };

    // Create glyph view widget from pre-computed path
    let glyph_view_widget = if let Some(path) = path_opt {
        Either::A(glyph_view(path, 100.0, 100.0))
    } else {
        Either::B(label("?").text_size(60.0))
    };

    // Style label based on selection state
    let name_label = if is_selected {
        label(format!("→ {}", display_name)).text_size(11.0)
    } else {
        label(display_name).text_size(11.0)
    };

    // Glyph cell - force square with sized_box
    sized_box(
        button(
            flex_col((
                glyph_view_widget,
                name_label,
            )),
            move |state: &mut AppState| {
                println!("Selected glyph: {}", name_clone);
                state.select_glyph(name_clone.clone());
            }
        )
    )
    .width(120.px())
    .height(120.px())
}
