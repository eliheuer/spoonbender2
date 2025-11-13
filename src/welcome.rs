// Copyright 2025 the Runebender Authors
// SPDX-License-Identifier: Apache-2.0

//! Welcome screen view for Runebender Xilem

use crate::data::AppState;
use crate::theme;
use masonry::properties::types::AsUnit;
use xilem::style::Style;
use xilem::view::{button, flex_col, label, sized_box, CrossAxisAlignment, MainAxisAlignment};
use xilem::WidgetView;

/// Welcome screen shown when no font is loaded
pub fn welcome_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let error_text = state
        .error_message
        .as_ref()
        .map(|msg| format!("Error: {}", msg))
        .unwrap_or_default();

    flex_col((
        label("Runebender Xilem").text_size(48.0),
        label("No font loaded"),
        label(error_text).text_size(12.0),
        sized_box(
            button(label("Open UFO..."), |state: &mut AppState| {
                state.open_font_dialog();
            })
        ).width(150.px()),
        sized_box(
            button(label("New Font"), |state: &mut AppState| {
                state.create_new_font();
            })
        ).width(150.px()),
    ))
    .main_axis_alignment(MainAxisAlignment::Center)
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .background_color(theme::app::BACKGROUND)
}
