// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Path segment representation for hit-testing and curve operations

use kurbo::{CubicBez, Line, ParamCurve, ParamCurveNearest, Point};

/// A segment of a path (line or cubic bezier curve)
#[derive(Debug, Clone, Copy)]
pub enum Segment {
    /// A line segment
    Line(Line),
    /// A cubic bezier curve segment
    Cubic(CubicBez),
}

/// Information about a segment within a path
#[derive(Debug, Clone, Copy)]
pub struct SegmentInfo {
    /// The geometric segment
    pub segment: Segment,
    /// The index of the starting point in the path
    pub start_idx: usize,
    /// The index of the ending point in the path
    pub end_idx: usize,
}

impl Segment {
    /// Find the nearest point on this segment to the given point
    ///
    /// Returns (t, distance_squared) where t is the parametric position (0.0 to 1.0)
    /// and distance_squared is the squared distance from the point to the curve
    pub fn nearest(&self, point: Point) -> (f64, f64) {
        match self {
            Segment::Line(line) => {
                let t = line_nearest_param(*line, point);
                let nearest_pt = line.eval(t);
                let dist_sq = (nearest_pt - point).hypot2();
                (t, dist_sq)
            }
            Segment::Cubic(cubic) => {
                // Use kurbo's nearest function
                let result = cubic.nearest(point, 1e-6);
                (result.t, result.distance_sq)
            }
        }
    }

    /// Evaluate the segment at parameter t (0.0 to 1.0)
    pub fn eval(&self, t: f64) -> Point {
        match self {
            Segment::Line(line) => line.eval(t),
            Segment::Cubic(cubic) => cubic.eval(t),
        }
    }
}

/// Find the parameter t on a line segment nearest to a point
fn line_nearest_param(line: Line, point: Point) -> f64 {
    let p0 = line.p0;
    let p1 = line.p1;

    // Vector from p0 to p1
    let line_vec = p1 - p0;
    // Vector from p0 to point
    let pt_vec = point - p0;

    // Project pt_vec onto line_vec
    let line_len_sq = line_vec.hypot2();
    if line_len_sq < 1e-12 {
        // Degenerate line (p0 == p1)
        return 0.0;
    }

    let t = (pt_vec.x * line_vec.x + pt_vec.y * line_vec.y) / line_len_sq;

    // Clamp t to [0, 1]
    t.clamp(0.0, 1.0)
}
