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

    /// Subdivide a cubic bezier curve at parameter t
    ///
    /// Returns (left_curve, right_curve) where left_curve goes from t=0 to t=t_split
    /// and right_curve goes from t=t_split to t=1. The curves together exactly match
    /// the original curve.
    ///
    /// Uses the de Casteljau algorithm for numerically stable subdivision.
    pub fn subdivide_cubic(cubic: CubicBez, t: f64) -> (CubicBez, CubicBez) {
        // de Casteljau subdivision algorithm
        // Given control points P0, P1, P2, P3 and parameter t:

        let p0 = cubic.p0;
        let p1 = cubic.p1;
        let p2 = cubic.p2;
        let p3 = cubic.p3;

        // First level of interpolation
        let q0 = p0 + (p1 - p0) * t;
        let q1 = p1 + (p2 - p1) * t;
        let q2 = p2 + (p3 - p2) * t;

        // Second level
        let r0 = q0 + (q1 - q0) * t;
        let r1 = q1 + (q2 - q1) * t;

        // Third level - the split point
        let split_point = r0 + (r1 - r0) * t;

        // Left curve: P0, Q0, R0, split_point
        let left = CubicBez::new(p0, q0, r0, split_point);

        // Right curve: split_point, R1, Q2, P3
        let right = CubicBez::new(split_point, r1, q2, p3);

        (left, right)
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
