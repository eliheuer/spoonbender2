// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Theme colors and constants
//!
//! Format: Color::from_rgb8(red, green, blue) values are 0-255 or 0x00-0xFF

use masonry::vello::peniko::Color;

// ============================================================================
// EDITOR CANVAS
// ============================================================================
const CANVAS_BACKGROUND: Color = Color::from_rgb8(0x15, 0x15, 0x15);

// ============================================================================
// UI PANELS (Toolbar, Coordinate Pane, Glyph Preview)
// ============================================================================
const PANEL_BACKGROUND: Color = Color::from_rgb8(0x40, 0x40, 0x40);
const PANEL_OUTLINE: Color = Color::from_rgb8(0x60, 0x60, 0x60);
const TOOLBAR_BUTTON_OUTLINE: Color = Color::from_rgb8(0x10, 0x10, 0x10);
const GLYPH_PREVIEW_COLOR: Color = Color::from_rgb8(0x10, 0x10, 0x10);

// ============================================================================
// PATHS AND OUTLINES
// ============================================================================
const PATH_STROKE: Color = Color::from_rgb8(200, 200, 200);  // Light gray
const PATH_FILL: Color = Color::from_rgb8(100, 100, 100);    // Medium gray

// ============================================================================
// METRICS GUIDES
// ============================================================================
const METRICS_GUIDE: Color = Color::from_rgb8(100, 100, 100);  // Medium gray

// ============================================================================
// GRID
// ============================================================================
const GRID_LINE: Color = Color::from_rgba8(60, 60, 60, 255);  // Dark gray

// ============================================================================
// CONTROL POINT HANDLES
// ============================================================================
const HANDLE_LINE: Color = Color::from_rgba8(150, 150, 150, 255);  // Light gray

// ============================================================================
// POINT COLORS
// ============================================================================

// Smooth on-curve points (circles) - BLUE
const SMOOTH_POINT_INNER: Color = Color::from_rgb8(0x57, 0x9a, 0xff);  // Light blue
const SMOOTH_POINT_OUTER: Color = Color::from_rgb8(0x44, 0x28, 0xec);  // Dark blue

// Corner on-curve points (squares) - GREEN
const CORNER_POINT_INNER: Color = Color::from_rgb8(0x6a, 0xe7, 0x56);  // Light green
const CORNER_POINT_OUTER: Color = Color::from_rgb8(0x20, 0x8e, 0x56);  // Dark green

// Off-curve control points (circles) - PURPLE
const OFFCURVE_POINT_INNER: Color = Color::from_rgb8(0xcc, 0x99, 0xff);  // Light purple
const OFFCURVE_POINT_OUTER: Color = Color::from_rgb8(0x99, 0x00, 0xff);  // Dark purple

// Selected points (any type) - YELLOW/ORANGE
const SELECTED_POINT_INNER: Color = Color::from_rgb8(0xff, 0xee, 0x55);  // Yellow
const SELECTED_POINT_OUTER: Color = Color::from_rgb8(0xff, 0xaa, 0x33);  // Orange

// ============================================================================
// PUBLIC API - Don't edit below this line
// ============================================================================

/// Colors for editor canvas
pub mod canvas {
    use super::Color;
    pub const BACKGROUND: Color = super::CANVAS_BACKGROUND;
}

/// Colors for UI panels (toolbar, info panes, etc.)
pub mod panel {
    use super::Color;
    pub const BACKGROUND: Color = super::PANEL_BACKGROUND;
    pub const OUTLINE: Color = super::PANEL_OUTLINE;
    pub const BUTTON_OUTLINE: Color = super::TOOLBAR_BUTTON_OUTLINE;
    pub const GLYPH_PREVIEW: Color = super::GLYPH_PREVIEW_COLOR;
}

/// Colors for paths and outlines
pub mod path {
    use super::Color;
    pub const STROKE: Color = super::PATH_STROKE;
    pub const FILL: Color = super::PATH_FILL;
}

/// Colors for font metrics guides
pub mod metrics {
    use super::Color;
    pub const GUIDE: Color = super::METRICS_GUIDE;
}

/// Colors for grid
pub mod grid {
    use super::Color;
    pub const LINE: Color = super::GRID_LINE;
}

/// Colors for control point lines (handles)
pub mod handle {
    use super::Color;
    pub const LINE: Color = super::HANDLE_LINE;
}

/// Colors for points
pub mod point {
    use super::Color;
    pub const SMOOTH_INNER: Color = super::SMOOTH_POINT_INNER;
    pub const SMOOTH_OUTER: Color = super::SMOOTH_POINT_OUTER;
    pub const CORNER_INNER: Color = super::CORNER_POINT_INNER;
    pub const CORNER_OUTER: Color = super::CORNER_POINT_OUTER;
    pub const OFFCURVE_INNER: Color = super::OFFCURVE_POINT_INNER;
    pub const OFFCURVE_OUTER: Color = super::OFFCURVE_POINT_OUTER;
    pub const SELECTED_INNER: Color = super::SELECTED_POINT_INNER;
    pub const SELECTED_OUTER: Color = super::SELECTED_POINT_OUTER;
}

/// Sizes for rendering
pub mod size {
    /// Radius for smooth on-curve points
    pub const SMOOTH_POINT_RADIUS: f64 = 4.0;
    /// Radius for smooth on-curve points when selected
    pub const SMOOTH_POINT_SELECTED_RADIUS: f64 = 5.0;

    /// Half-size for corner on-curve points (square)
    pub const CORNER_POINT_HALF_SIZE: f64 = 3.5;
    /// Half-size for corner on-curve points when selected
    pub const CORNER_POINT_SELECTED_HALF_SIZE: f64 = 4.5;

    /// Radius for off-curve control points
    pub const OFFCURVE_POINT_RADIUS: f64 = 3.0;
    /// Radius for off-curve control points when selected
    pub const OFFCURVE_POINT_SELECTED_RADIUS: f64 = 4.0;

    /// Width of path strokes
    pub const PATH_STROKE_WIDTH: f64 = 1.0;

    /// Width of control point lines
    pub const HANDLE_LINE_WIDTH: f64 = 1.0;

    /// Width of metric guide lines
    pub const METRIC_LINE_WIDTH: f64 = 1.0;
}
