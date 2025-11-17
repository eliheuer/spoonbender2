// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Cubic bezier path representation

use crate::entity_id::EntityId;
use crate::point::{PathPoint, PointType};
use crate::point_list::PathPoints;
use crate::workspace;
use kurbo::{BezPath, Shape};

/// A single contour represented as a cubic bezier path
///
/// This corresponds to a UFO contour. Points are stored in order,
/// with the convention that for closed paths, the first point (index 0)
/// is conceptually the last point in the cyclic sequence.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CubicPath {
    /// The points in this path
    pub points: PathPoints,

    /// Whether this path is closed
    pub closed: bool,

    /// Unique identifier for this path
    pub id: EntityId,
}

#[allow(dead_code)]
impl CubicPath {
    /// Create a new cubic path
    pub fn new(points: PathPoints, closed: bool) -> Self {
        Self {
            points,
            closed,
            id: EntityId::next(),
        }
    }

    /// Create a new empty cubic path
    pub fn empty() -> Self {
        Self::new(PathPoints::new(), false)
    }

    /// Get the number of points in this path
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if this path is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Get a reference to the points in this path
    pub fn points(&self) -> &PathPoints {
        &self.points
    }

    /// Convert this cubic path to a kurbo BezPath for rendering
    pub fn to_bezpath(&self) -> BezPath {
        let mut path = BezPath::new();

        if self.points.is_empty() {
            return path;
        }

        let points: Vec<&PathPoint> = self.points.iter().collect();

        // Find the first on-curve point to start
        let start_idx = points.iter().position(|p| p.is_on_curve()).unwrap_or(0);

        // Rotate points so we start at an on-curve point
        let rotated: Vec<&PathPoint> = points[start_idx..]
            .iter()
            .chain(points[..start_idx].iter())
            .copied()
            .collect();

        if rotated.is_empty() {
            return path;
        }

        // Start at the first point
        path.move_to(rotated[0].point);

        // Process remaining points
        let mut i = 1;
        while i < rotated.len() {
            let pt = rotated[i];

            match pt.typ {
                PointType::OnCurve { .. } => {
                    // Look back to see if there are off-curve points before this
                    let mut off_curve_before = Vec::new();
                    let mut j = i - 1;

                    while j > 0 && rotated[j].is_off_curve() {
                        off_curve_before.insert(0, rotated[j]);
                        j -= 1;
                    }

                    match off_curve_before.len() {
                        0 => {
                            // No control points - draw line
                            path.line_to(pt.point);
                        }
                        1 => {
                            // One control point - quadratic curve
                            path.quad_to(off_curve_before[0].point, pt.point);
                        }
                        2 => {
                            // Two control points - cubic curve
                            path.curve_to(
                                off_curve_before[0].point,
                                off_curve_before[1].point,
                                pt.point,
                            );
                        }
                        _ => {
                            // More than 2 - use last two
                            let len = off_curve_before.len();
                            path.curve_to(
                                off_curve_before[len - 2].point,
                                off_curve_before[len - 1].point,
                                pt.point,
                            );
                        }
                    }
                    i += 1;
                }
                PointType::OffCurve { .. } => {
                    // Off-curve points are processed with the next on-curve point
                    i += 1;
                }
            }
        }

        // Handle trailing off-curve points (if path is closed)
        if self.closed {
            let mut trailing_off_curve = Vec::new();
            let mut j = rotated.len() - 1;

            while j > 0 && rotated[j].is_off_curve() {
                trailing_off_curve.insert(0, rotated[j]);
                j -= 1;
            }

            if !trailing_off_curve.is_empty() {
                // These off-curve points connect back to the first point
                let first_pt = rotated[0];

                match trailing_off_curve.len() {
                    1 => {
                        path.quad_to(trailing_off_curve[0].point, first_pt.point);
                    }
                    2 => {
                        path.curve_to(
                            trailing_off_curve[0].point,
                            trailing_off_curve[1].point,
                            first_pt.point,
                        );
                    }
                    _ => {
                        let len = trailing_off_curve.len();
                        path.curve_to(
                            trailing_off_curve[len - 2].point,
                            trailing_off_curve[len - 1].point,
                            first_pt.point,
                        );
                    }
                }
            }

            path.close_path();
        }

        path
    }

    /// Get the bounding box of this path
    pub fn bounding_box(&self) -> Option<kurbo::Rect> {
        let bez = self.to_bezpath();
        if bez.is_empty() {
            None
        } else {
            Some(bez.bounding_box())
        }
    }

    /// Convert from a workspace contour (norad format)
    pub fn from_contour(contour: &workspace::Contour) -> Self {
        if contour.points.is_empty() {
            return Self::empty();
        }

        // Determine if the path is closed
        // In UFO, a contour is closed unless the first point is a Move
        let closed = !matches!(contour.points[0].point_type, workspace::PointType::Move);

        // Convert all points
        let mut path_points: Vec<PathPoint> = contour
            .points
            .iter()
            .map(PathPoint::from_contour_point)
            .collect();

        // If closed, rotate left by 1 to match Runebender's convention
        // (first point in closed path is last in vector)
        if closed && !path_points.is_empty() {
            path_points.rotate_left(1);
        }

        Self::new(PathPoints::from_vec(path_points), closed)
    }

    /// Convert this cubic path to a workspace contour (for saving)
    pub fn to_contour(&self) -> workspace::Contour {
        use crate::point::PointType;
        use crate::workspace::{Contour, ContourPoint, PointType as WsPointType};

        let mut contour_points: Vec<PathPoint> = self.points.to_vec();

        // If closed, rotate right by 1 to reverse what from_contour() did
        if self.closed && !contour_points.is_empty() {
            contour_points.rotate_right(1);
        }

        // Convert points back to workspace format
        let points: Vec<ContourPoint> = contour_points
            .iter()
            .map(|pt| {
                let point_type = match pt.typ {
                    PointType::OnCurve { smooth: true } => WsPointType::Curve,
                    PointType::OnCurve { smooth: false } => WsPointType::Line,
                    PointType::OffCurve { .. } => WsPointType::OffCurve,
                };

                ContourPoint {
                    x: pt.point.x,
                    y: pt.point.y,
                    point_type,
                }
            })
            .collect();

        Contour { points }
    }

    /// Iterate over the segments in this path
    ///
    /// Returns an iterator that yields SegmentInfo for each segment (line or curve)
    pub fn iter_segments(&self) -> impl Iterator<Item = crate::segment::SegmentInfo> + '_ {
        SegmentIterator::new(&self.points, self.closed)
    }
}

/// Iterator over path segments
#[allow(dead_code)]
struct SegmentIterator {
    points: Vec<PathPoint>,
    closed: bool,
    index: usize,
    prev_on_curve: kurbo::Point,
    prev_on_curve_idx: usize,
}

impl SegmentIterator {
    fn new(points: &crate::point_list::PathPoints, closed: bool) -> Self {
        let points_vec: Vec<PathPoint> = points.iter().cloned().collect();

        // Find first on-curve point
        let (start_idx, start_pt) = points_vec
            .iter()
            .enumerate()
            .find(|(_, p)| p.is_on_curve())
            .map(|(i, p)| (i, p.point))
            .unwrap_or((0, kurbo::Point::ZERO));

        let index = if closed { 0 } else { start_idx + 1 };

        Self {
            points: points_vec,
            closed,
            index,
            prev_on_curve: start_pt,
            prev_on_curve_idx: start_idx,
        }
    }
}

impl Iterator for SegmentIterator {
    type Item = crate::segment::SegmentInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.points.len() {
            return None;
        }

        let pt = &self.points[self.index];

        if pt.is_on_curve() {
            // Line segment from prev to current
            let start_idx = self.prev_on_curve_idx;
            let end_idx = self.index;
            let segment =
                crate::segment::Segment::Line(kurbo::Line::new(self.prev_on_curve, pt.point));

            self.prev_on_curve = pt.point;
            self.prev_on_curve_idx = self.index;
            self.index += 1;

            Some(crate::segment::SegmentInfo {
                segment,
                start_idx,
                end_idx,
            })
        } else {
            // Cubic curve: need 2 off-curve + 1 on-curve
            if self.index + 2 < self.points.len() {
                let cp1 = pt.point;
                let cp2 = self.points[self.index + 1].point;
                let end = self.points[self.index + 2].point;

                let start_idx = self.prev_on_curve_idx;
                let end_idx = self.index + 2;
                let segment = crate::segment::Segment::Cubic(kurbo::CubicBez::new(
                    self.prev_on_curve,
                    cp1,
                    cp2,
                    end,
                ));

                self.prev_on_curve = end;
                self.prev_on_curve_idx = self.index + 2;
                self.index += 3;

                Some(crate::segment::SegmentInfo {
                    segment,
                    start_idx,
                    end_idx,
                })
            } else {
                None
            }
        }
    }
}
