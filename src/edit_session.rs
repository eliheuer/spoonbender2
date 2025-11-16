// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Edit session - manages editing state for a single glyph

use crate::entity_id::EntityId;
use crate::hit_test::{self, HitTestResult};
use crate::path::Path;
use crate::quadrant::Quadrant;
use crate::selection::Selection;
use crate::tools::{ToolBox, ToolId};
use crate::widgets::CoordinateSelection;
use crate::workspace::Glyph;
use kurbo::{Point, Rect};
use std::sync::Arc;

/// Unique identifier for an editing session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    /// Create a new unique session ID
    pub fn next() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Viewport transformation between design space and screen space
#[derive(Debug, Clone)]
pub struct ViewPort {
    /// Scroll offset in screen space
    pub offset: kurbo::Vec2,

    /// Zoom level (screen pixels per design unit)
    pub zoom: f64,
}

impl ViewPort {
    /// Create a new viewport with default settings
    pub fn new() -> Self {
        Self {
            offset: kurbo::Vec2::ZERO,
            zoom: 1.0,
        }
    }

    /// Convert a point from design space to screen space
    pub fn to_screen(&self, point: kurbo::Point) -> kurbo::Point {
        // Design space: Y increases upward (font coordinates)
        // Screen space: Y increases downward (UI coordinates)
        // Apply: scale, flip Y, translate by offset
        kurbo::Point::new(
            point.x * self.zoom + self.offset.x,
            -point.y * self.zoom + self.offset.y,
        )
    }

    /// Convert a point from screen space to design space
    pub fn from_screen(&self, point: kurbo::Point) -> kurbo::Point {
        kurbo::Point::new(
            (point.x - self.offset.x) / self.zoom,
            -(point.y - self.offset.y) / self.zoom,
        )
    }

    /// Get the affine transformation from design space to screen space
    pub fn affine(&self) -> kurbo::Affine {
        // Build transformation: scale, flip Y, translate
        kurbo::Affine::new([
            self.zoom,    // x scale
            0.0,          // x skew
            0.0,          // y skew
            -self.zoom,   // y scale (negative for Y-flip)
            self.offset.x,  // x translation
            self.offset.y,  // y translation
        ])
    }
}

impl Default for ViewPort {
    fn default() -> Self {
        Self::new()
    }
}

// CoordinateSelection has been moved to widgets::coord_pane module

/// Editing session for a single glyph
///
/// This holds all the state needed to edit a glyph, including the
/// outline data, selection, viewport, and metadata.
#[derive(Debug, Clone)]
pub struct EditSession {
    /// Unique identifier for this session
    pub id: SessionId,

    /// Name of the glyph being edited
    pub glyph_name: String,

    /// Path to the UFO file
    pub ufo_path: std::path::PathBuf,

    /// The original glyph data (for metadata, unicode, etc.)
    pub glyph: Arc<Glyph>,

    /// The editable path representation (converted from glyph contours)
    pub paths: Arc<Vec<Path>>,

    /// Currently selected entities (points, paths, etc.)
    pub selection: Selection,

    /// Coordinate selection (for the coordinate pane)
    pub coord_selection: CoordinateSelection,

    /// Current editing tool
    pub current_tool: ToolBox,

    /// Viewport transformation
    pub viewport: ViewPort,

    /// Whether the viewport has been initialized (to avoid recalculating on every frame)
    pub viewport_initialized: bool,

    /// Font metrics (for drawing guides)
    pub units_per_em: f64,
    pub ascender: f64,
    pub descender: f64,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
}

impl EditSession {
    /// Create a new editing session for a glyph
    pub fn new(
        glyph_name: String,
        ufo_path: std::path::PathBuf,
        glyph: Glyph,
        units_per_em: f64,
        ascender: f64,
        descender: f64,
        x_height: Option<f64>,
        cap_height: Option<f64>,
    ) -> Self {
        // Convert glyph contours to editable paths
        let paths: Vec<Path> = glyph
            .contours
            .iter()
            .map(|contour| Path::from_contour(contour))
            .collect();

        Self {
            id: SessionId::next(),
            glyph_name,
            ufo_path,
            glyph: Arc::new(glyph),
            paths: Arc::new(paths),
            selection: Selection::new(),
            coord_selection: CoordinateSelection::default(),
            current_tool: ToolBox::for_id(ToolId::Select),
            viewport: ViewPort::new(),
            viewport_initialized: false,
            units_per_em,
            ascender,
            descender,
            x_height,
            cap_height,
        }
    }

    /// Compute the coordinate selection from the current selection
    ///
    /// This calculates the bounding box of all selected points and updates
    /// the coord_selection field.
    pub fn update_coord_selection(&mut self) {
        if self.selection.is_empty() {
            self.coord_selection = CoordinateSelection::default();
            return;
        }

        // Calculate bounding box of selected points
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut count = 0;

        for path in self.paths.iter() {
            match path {
                crate::path::Path::Cubic(cubic) => {
                    for pt in cubic.points.iter() {
                        if self.selection.contains(&pt.id) {
                            min_x = min_x.min(pt.point.x);
                            max_x = max_x.max(pt.point.x);
                            min_y = min_y.min(pt.point.y);
                            max_y = max_y.max(pt.point.y);
                            count += 1;
                        }
                    }
                }
            }
        }

        if min_x.is_finite() {
            let frame = Rect::new(min_x, min_y, max_x, max_y);
            self.coord_selection = CoordinateSelection::new(
                count,
                frame,
                self.coord_selection.quadrant, // Preserve the current quadrant selection
            );
        } else {
            self.coord_selection = CoordinateSelection::default();
        }
    }

    /// Switch to a different tool
    pub fn set_tool(&mut self, tool_id: ToolId) {
        self.current_tool = ToolBox::for_id(tool_id);
    }

    /// Get the current tool ID
    pub fn current_tool_id(&self) -> ToolId {
        self.current_tool.id()
    }

    /// Get a displayable title for this session
    pub fn title(&self) -> String {
        format!("Edit: {}", self.glyph_name)
    }

    /// Set the viewport for this session
    pub fn set_viewport(&mut self, viewport: ViewPort) {
        self.viewport = viewport;
    }

    /// Hit test for a point at screen coordinates
    ///
    /// Returns the EntityId of the closest point within max_dist screen pixels
    pub fn hit_test_point(&self, screen_pos: Point, max_dist: Option<f64>) -> Option<HitTestResult> {
        let max_dist = max_dist.unwrap_or(hit_test::MIN_CLICK_DISTANCE);

        // Collect all points from all paths as screen coordinates
        let candidates = self.paths.iter().flat_map(|path| {
            match path {
                Path::Cubic(cubic) => {
                    cubic.points().iter().map(|pt| {
                        // Convert point to screen space for distance calculation
                        let screen_pt = self.viewport.to_screen(pt.point);
                        (pt.id, screen_pt, pt.is_on_curve())
                    }).collect::<Vec<_>>()
                }
            }
        });

        // Find closest point in screen space
        hit_test::find_closest(screen_pos, candidates, max_dist)
    }

    /// Move selected points by a delta in design space
    ///
    /// This mutates the paths using Arc::make_mut, which will clone
    /// the path data if there are other references to it.
    ///
    /// When moving on-curve points, their adjacent off-curve control points
    /// (handles) are also moved to maintain curve shape. This is standard
    /// font editor behavior.
    pub fn move_selection(&mut self, delta: kurbo::Vec2) {
        if self.selection.is_empty() {
            return;
        }

        use crate::entity_id::EntityId;
        use std::collections::HashSet;

        // We need to mutate paths, so convert Arc<Vec<Path>> to mutable Vec
        let paths_vec = Arc::make_mut(&mut self.paths);

        // Build a set of IDs to move:
        // - All selected points
        // - Adjacent off-curve points of selected on-curve points
        let mut points_to_move: HashSet<EntityId> = self.selection.iter().copied().collect();

        // First pass: identify adjacent off-curve points of selected on-curve points
        for path in paths_vec.iter() {
            match path {
                Path::Cubic(cubic) => {
                    let points: Vec<_> = cubic.points.iter().collect();
                    let len = points.len();

                    for i in 0..len {
                        let point = points[i];

                        // If this on-curve point is selected, mark its adjacent off-curve points
                        if point.is_on_curve() && self.selection.contains(&point.id) {
                            // Check previous point
                            let prev_i = if i > 0 { i - 1 } else if cubic.closed { len - 1 } else { continue };
                            if prev_i < len && points[prev_i].is_off_curve() {
                                points_to_move.insert(points[prev_i].id);
                            }

                            // Check next point
                            let next_i = if i + 1 < len { i + 1 } else if cubic.closed { 0 } else { continue };
                            if next_i < len && points[next_i].is_off_curve() {
                                points_to_move.insert(points[next_i].id);
                            }
                        }
                    }
                }
            }
        }

        // Second pass: move all identified points
        for path in paths_vec.iter_mut() {
            match path {
                Path::Cubic(cubic) => {
                    // Get mutable access to points (will clone if needed)
                    let points = cubic.points.make_mut();

                    // Update positions of points to move
                    for point in points.iter_mut() {
                        if points_to_move.contains(&point.id) {
                            point.point = Point::new(
                                point.point.x + delta.x,
                                point.point.y + delta.y,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Nudge selected points in a direction
    ///
    /// Nudge amounts:
    /// - Normal: 1 unit
    /// - Shift: 10 units
    /// - Cmd/Ctrl: 100 units
    pub fn nudge_selection(&mut self, dx: f64, dy: f64, shift: bool, ctrl: bool) {
        let multiplier = if ctrl {
            100.0
        } else if shift {
            10.0
        } else {
            1.0
        };

        let delta = kurbo::Vec2::new(dx * multiplier, dy * multiplier);
        self.move_selection(delta);
    }

    /// Delete selected points
    ///
    /// This removes selected points from paths. If all points in a path are
    /// deleted, the entire path is removed.
    pub fn delete_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }

        // Get mutable access to paths
        let paths_vec = Arc::make_mut(&mut self.paths);

        // Filter out paths that become empty after deletion
        paths_vec.retain_mut(|path| {
            match path {
                Path::Cubic(cubic) => {
                    // Remove selected points
                    let points = cubic.points.make_mut();
                    points.retain(|point| !self.selection.contains(&point.id));

                    // Keep path only if it has at least 2 points
                    points.len() >= 2
                }
            }
        });

        // Clear selection since deleted points are gone
        self.selection = Selection::new();
    }

    /// Toggle point type between smooth and corner for selected on-curve points
    pub fn toggle_point_type(&mut self) {
        if self.selection.is_empty() {
            return;
        }

        let paths_vec = Arc::make_mut(&mut self.paths);

        for path in paths_vec.iter_mut() {
            match path {
                Path::Cubic(cubic) => {
                    let points = cubic.points.make_mut();

                    for point in points.iter_mut() {
                        if self.selection.contains(&point.id) {
                            // Only toggle on-curve points
                            if let crate::point::PointType::OnCurve { smooth } = &mut point.typ {
                                *smooth = !*smooth;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Reverse the direction of all paths
    pub fn reverse_contours(&mut self) {
        let paths_vec = Arc::make_mut(&mut self.paths);

        for path in paths_vec.iter_mut() {
            match path {
                Path::Cubic(cubic) => {
                    let points = cubic.points.make_mut();
                    points.reverse();
                }
            }
        }
    }
}
