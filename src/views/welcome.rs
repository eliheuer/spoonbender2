// Copyright 2025 the Runebender Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Welcome screen view for Runebender Xilem

use crate::data::AppState;
use crate::edit_session::EditSession;
use crate::workspace::Glyph;
use masonry::properties::types::AsUnit;
use masonry::properties::types::UnitPoint;
use std::sync::Arc;
use xilem::WidgetView;
use xilem::style::Style;
use xilem::view::{
    ChildAlignment, CrossAxisAlignment, MainAxisAlignment, ZStackExt, button, flex_col, label,
    sized_box, transformed, zstack,
};

/// Create a hardcoded "R" glyph from VirtuaGrotesk-Regular
/// This is used as the background for the welcome screen
#[allow(clippy::vec_init_then_push)]
fn create_r_glyph() -> Glyph {
    let mut contours = Vec::new();

    // First contour (inner counter of R)
    let mut contour1 = Vec::new();
    contour1.push(crate::workspace::ContourPoint {
        x: 192.0,
        y: 416.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 184.0,
        y: 424.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 184.0,
        y: 664.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 192.0,
        y: 672.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 368.0,
        y: 672.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 440.0,
        y: 672.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 496.0,
        y: 616.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 496.0,
        y: 544.0,
        point_type: crate::workspace::PointType::Curve,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 496.0,
        y: 472.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 440.0,
        y: 416.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour1.push(crate::workspace::ContourPoint {
        x: 368.0,
        y: 416.0,
        point_type: crate::workspace::PointType::Curve,
    });
    contours.push(crate::workspace::Contour { points: contour1 });

    // Second contour (outer outline of R)
    let mut contour2 = Vec::new();
    contour2.push(crate::workspace::ContourPoint {
        x: 96.0,
        y: 0.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 168.0,
        y: 0.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 184.0,
        y: 16.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 184.0,
        y: 320.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 192.0,
        y: 328.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 360.0,
        y: 328.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 456.0,
        y: 328.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 496.0,
        y: 288.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 496.0,
        y: 192.0,
        point_type: crate::workspace::PointType::Curve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 496.0,
        y: 16.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 512.0,
        y: 0.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 584.0,
        y: 0.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 600.0,
        y: 16.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 600.0,
        y: 208.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 600.0,
        y: 304.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 544.0,
        y: 360.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 472.0,
        y: 368.0,
        point_type: crate::workspace::PointType::Curve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 472.0,
        y: 376.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 528.0,
        y: 392.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 604.0,
        y: 448.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 604.0,
        y: 544.0,
        point_type: crate::workspace::PointType::Curve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 604.0,
        y: 672.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 504.0,
        y: 768.0,
        point_type: crate::workspace::PointType::OffCurve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 376.0,
        y: 768.0,
        point_type: crate::workspace::PointType::Curve,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 96.0,
        y: 768.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 80.0,
        y: 752.0,
        point_type: crate::workspace::PointType::Line,
    });
    contour2.push(crate::workspace::ContourPoint {
        x: 80.0,
        y: 16.0,
        point_type: crate::workspace::PointType::Line,
    });
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

    let mut session = EditSession::new(
        "R".to_string(),
        std::path::PathBuf::from("demo.ufo"), // Dummy path for demo
        glyph,
        1024.0,      // UPM (units per em) - from VirtuaGrotesk
        832.0,       // ascender - from VirtuaGrotesk
        -256.0,      // descender - from VirtuaGrotesk
        Some(576.0), // x_height - from VirtuaGrotesk
        Some(768.0), // cap_height - from VirtuaGrotesk
    );

    // Center the glyph perfectly in the window
    session.viewport.offset = kurbo::Vec2::new(500.0, 700.0);

    // Adjust zoom level for nice preview size
    session.viewport.zoom = 0.7;

    session.viewport_initialized = true;

    session
}

/// Welcome screen shown when no font is loaded.
pub fn welcome(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let error_text = state
        .error_message
        .as_ref()
        .map(|msg| format!("Error: {}", msg))
        .unwrap_or_default();

    // Create or reuse the demo edit session with the hardcoded R glyph
    if state.welcome_session.is_none() {
        state.welcome_session = Some(create_demo_session());
    }

    let session = state.welcome_session.as_ref().unwrap();
    let session_arc = Arc::new(session.clone());

    const MARGIN: f64 = 16.0;

    // Layer welcome UI over interactive editor
    zstack((
        // Background: Interactive editor with demo R glyph
        crate::components::editor_view(session_arc, |state: &mut AppState, updated_session| {
            // Save changes back to the welcome session so they persist
            state.welcome_session = Some(updated_session);
        }),
        // Foreground: Welcome UI in upper left (constrained size so it doesn't block editor)
        transformed(
            sized_box(
                flex_col((
                    label("Runebender Xilem")
                        .text_size(48.0)
                        .color(crate::theme::text::PRIMARY),
                    label(error_text)
                        .text_size(12.0)
                        .color(crate::theme::text::PRIMARY),
                    sized_box(label("")).height(8.px()),
                    sized_box(button(
                        label("Open UFO...").color(crate::theme::text::PRIMARY),
                        |state: &mut AppState| {
                            state.open_font_dialog();
                        },
                    ))
                    .width(200.px()),
                    sized_box(button(
                        label("New Font").color(crate::theme::text::PRIMARY),
                        |state: &mut AppState| {
                            state.create_new_font();
                        },
                    ))
                    .width(200.px()),
                ))
                .main_axis_alignment(MainAxisAlignment::Start)
                .cross_axis_alignment(CrossAxisAlignment::Start),
            )
            .width(220.px()) // Constrained width
            .height(200.px()), // Constrained height
        )
        .translate((MARGIN, MARGIN))
        .alignment(ChildAlignment::SelfAligned(UnitPoint::TOP_LEFT)),
    ))
}
