// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Spoonbender: A font editor built with Xilem
//!
//! This is a port of Runebender from Druid to Xilem, using modern
//! Linebender crates for rendering and UI.

use masonry::properties::types::AsUnit;
use winit::error::EventLoopError;
use xilem::core::one_of::Either;
use xilem::view::{button, flex_col, flex_row, label, portal, sized_box};
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
            label(font_name).text_size(18.0),
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
            label(info_text).text_size(12.0),
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
        label(format!("{} glyphs", glyph_count)).text_size(14.0),
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

    // Style label based on selection state
    let name_label = if is_selected {
        label(format!("→ {}", display_name)).text_size(11.0)
    } else {
        label(display_name).text_size(11.0)
    };

    // Wrap label with bottom spacing
    let label_with_spacing = sized_box(name_label).height(20.px());

    // Glyph cell - fixed size works best with flex layout
    sized_box(
        button(
            flex_col((
                glyph_view_widget,
                label_with_spacing,
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
