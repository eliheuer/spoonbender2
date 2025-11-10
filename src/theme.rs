// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Theme colors and constants for the editor

use masonry::vello::peniko::Color;

/// Colors for paths and outlines
pub mod path {
    use super::Color;

    /// Stroke color for path outlines
    pub const STROKE: Color = Color::from_rgb8(200, 200, 200);

    /// Fill color for paths in preview mode
    pub const FILL: Color = Color::from_rgb8(100, 100, 100);
}

/// Colors for font metrics guides
pub mod metrics {
    use super::Color;

    /// Baseline, ascender, descender, x-height, cap-height
    pub const GUIDE: Color = Color::from_rgba8(100, 100, 255, 128);
}

/// Colors for grid
pub mod grid {
    use super::Color;

    /// Grid line color
    pub const LINE: Color = Color::from_rgba8(60, 60, 60, 255);
}

/// Colors for control point lines (handles)
pub mod handle {
    use super::Color;

    /// Line connecting off-curve to on-curve points
    pub const LINE: Color = Color::from_rgba8(150, 150, 150, 255);
}

/// Colors for points
pub mod point {
    use super::Color;

    /// Smooth on-curve point (inner)
    pub const SMOOTH_INNER: Color = Color::from_rgb8(255, 255, 255);
    /// Smooth on-curve point (outer)
    pub const SMOOTH_OUTER: Color = Color::from_rgb8(0, 0, 0);

    /// Smooth on-curve point when selected (inner)
    pub const SMOOTH_SELECTED_INNER: Color = Color::from_rgb8(100, 200, 255);
    /// Smooth on-curve point when selected (outer)
    pub const SMOOTH_SELECTED_OUTER: Color = Color::from_rgb8(0, 100, 200);

    /// Corner on-curve point (inner)
    pub const CORNER_INNER: Color = Color::from_rgb8(255, 255, 255);
    /// Corner on-curve point (outer)
    pub const CORNER_OUTER: Color = Color::from_rgb8(0, 0, 0);

    /// Corner on-curve point when selected (inner)
    pub const CORNER_SELECTED_INNER: Color = Color::from_rgb8(100, 200, 255);
    /// Corner on-curve point when selected (outer)
    pub const CORNER_SELECTED_OUTER: Color = Color::from_rgb8(0, 100, 200);

    /// Off-curve control point (inner)
    pub const OFFCURVE_INNER: Color = Color::from_rgb8(150, 150, 150);
    /// Off-curve control point (outer)
    pub const OFFCURVE_OUTER: Color = Color::from_rgb8(50, 50, 50);

    /// Off-curve control point when selected (inner)
    pub const OFFCURVE_SELECTED_INNER: Color = Color::from_rgb8(100, 200, 255);
    /// Off-curve control point when selected (outer)
    pub const OFFCURVE_SELECTED_OUTER: Color = Color::from_rgb8(0, 100, 200);
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
    pub const PATH_STROKE_WIDTH: f64 = 2.0;

    /// Width of control point lines
    pub const HANDLE_LINE_WIDTH: f64 = 1.0;

    /// Width of metric guide lines
    pub const METRIC_LINE_WIDTH: f64 = 1.0;
}
