// Copyright 2025 the Runebender Authors
// SPDX-License-Identifier: Apache-2.0

//! Welcome screen view for Runebender Xilem

use crate::data::AppState;
use crate::edit_session::EditSession;
use crate::workspace::Glyph;
use masonry::properties::types::AsUnit;
use std::sync::Arc;
use xilem::view::{button, flex_col, label, sized_box, zstack, CrossAxisAlignment, MainAxisAlignment};
use xilem::WidgetView;

/// Create a hardcoded "R" glyph from VirtuaGrotesk-Regular
/// This is used as the background for the welcome screen
fn create_r_glyph() -> Glyph {
    let mut contours = Vec::new();

    // First contour (inner counter of R)
    let mut contour1 = Vec::new();
    contour1.push(crate::workspace::ContourPoint { x: 192.0, y: 416.0, point_type: crate::workspace::PointType::Line });
    contour1.push(crate::workspace::ContourPoint { x: 184.0, y: 424.0, point_type: crate::workspace::PointType::Line });
    contour1.push(crate::workspace::ContourPoint { x: 184.0, y: 664.0, point_type: crate::workspace::PointType::Line });
    contour1.push(crate::workspace::ContourPoint { x: 192.0, y: 672.0, point_type: crate::workspace::PointType::Line });
    contour1.push(crate::workspace::ContourPoint { x: 368.0, y: 672.0, point_type: crate::workspace::PointType::Line });
    contour1.push(crate::workspace::ContourPoint { x: 440.0, y: 672.0, point_type: crate::workspace::PointType::OffCurve });
    contour1.push(crate::workspace::ContourPoint { x: 496.0, y: 616.0, point_type: crate::workspace::PointType::OffCurve });
    contour1.push(crate::workspace::ContourPoint { x: 496.0, y: 544.0, point_type: crate::workspace::PointType::Curve });
    contour1.push(crate::workspace::ContourPoint { x: 496.0, y: 472.0, point_type: crate::workspace::PointType::OffCurve });
    contour1.push(crate::workspace::ContourPoint { x: 440.0, y: 416.0, point_type: crate::workspace::PointType::OffCurve });
    contour1.push(crate::workspace::ContourPoint { x: 368.0, y: 416.0, point_type: crate::workspace::PointType::Curve });
    contours.push(crate::workspace::Contour { points: contour1 });

    // Second contour (outer outline of R)
    let mut contour2 = Vec::new();
    contour2.push(crate::workspace::ContourPoint { x: 96.0, y: 0.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 168.0, y: 0.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 184.0, y: 16.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 184.0, y: 320.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 192.0, y: 328.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 360.0, y: 328.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 456.0, y: 328.0, point_type: crate::workspace::PointType::OffCurve });
    contour2.push(crate::workspace::ContourPoint { x: 496.0, y: 288.0, point_type: crate::workspace::PointType::OffCurve });
    contour2.push(crate::workspace::ContourPoint { x: 496.0, y: 192.0, point_type: crate::workspace::PointType::Curve });
    contour2.push(crate::workspace::ContourPoint { x: 496.0, y: 16.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 512.0, y: 0.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 584.0, y: 0.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 600.0, y: 16.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 600.0, y: 208.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 600.0, y: 304.0, point_type: crate::workspace::PointType::OffCurve });
    contour2.push(crate::workspace::ContourPoint { x: 544.0, y: 360.0, point_type: crate::workspace::PointType::OffCurve });
    contour2.push(crate::workspace::ContourPoint { x: 472.0, y: 368.0, point_type: crate::workspace::PointType::Curve });
    contour2.push(crate::workspace::ContourPoint { x: 472.0, y: 376.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 528.0, y: 392.0, point_type: crate::workspace::PointType::OffCurve });
    contour2.push(crate::workspace::ContourPoint { x: 604.0, y: 448.0, point_type: crate::workspace::PointType::OffCurve });
    contour2.push(crate::workspace::ContourPoint { x: 604.0, y: 544.0, point_type: crate::workspace::PointType::Curve });
    contour2.push(crate::workspace::ContourPoint { x: 604.0, y: 672.0, point_type: crate::workspace::PointType::OffCurve });
    contour2.push(crate::workspace::ContourPoint { x: 504.0, y: 768.0, point_type: crate::workspace::PointType::OffCurve });
    contour2.push(crate::workspace::ContourPoint { x: 376.0, y: 768.0, point_type: crate::workspace::PointType::Curve });
    contour2.push(crate::workspace::ContourPoint { x: 96.0, y: 768.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 80.0, y: 752.0, point_type: crate::workspace::PointType::Line });
    contour2.push(crate::workspace::ContourPoint { x: 80.0, y: 16.0, point_type: crate::workspace::PointType::Line });
    contours.push(crate::workspace::Contour { points: contour2 });

    Glyph {
        name: "R".to_string(),
        width: 668.0,
        height: None,
        codepoints: vec!['R'],
        contours,
    }
}

/// Create a demo edit session with the hardcoded R glyph
fn create_demo_session() -> EditSession {
    let glyph = create_r_glyph();

    EditSession::new(
        "R".to_string(),
        glyph,
        1000.0,     // UPM (units per em)
        800.0,      // ascender
        -200.0,     // descender
        Some(500.0), // x_height
        Some(700.0), // cap_height
    )
}

/// Welcome screen shown when no font is loaded
pub fn welcome_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let error_text = state
        .error_message
        .as_ref()
        .map(|msg| format!("Error: {}", msg))
        .unwrap_or_default();

    // Create a demo edit session with the hardcoded R glyph for the background
    let demo_session = create_demo_session();
    let session_arc = Arc::new(demo_session);

    // Use zstack to layer the welcome UI over the interactive editor background
    zstack((
        // Background: Interactive editor canvas with the R glyph
        crate::widgets::editor_view(
            session_arc.clone(),
            |_state: &mut AppState, _updated_session| {
                // No-op: we don't need to update state from the background editor
            }
        ),
        // Foreground: Welcome screen UI
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
        .cross_axis_alignment(CrossAxisAlignment::Center),
    ))
}
