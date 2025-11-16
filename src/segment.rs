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
    ///
    /// # The de Casteljau Algorithm
    ///
    /// This algorithm subdivides a bezier curve by repeatedly interpolating between
    /// control points. It's both geometrically intuitive and numerically stable.
    ///
    /// ## How it works:
    ///
    /// Given a cubic bezier with control points P0, P1, P2, P3, we want to split it
    /// at parameter t (where 0 ≤ t ≤ 1) into two curves that together form the original.
    ///
    /// The algorithm works by building a "pyramid" of linear interpolations:
    ///
    /// ```text
    /// Level 0 (original):          P0      P1      P2      P3
    ///                                \    /  \    /  \    /
    /// Level 1 (lerp at t):            Q0      Q1      Q2
    ///                                   \    /  \    /
    /// Level 2 (lerp at t):                 R0      R1
    ///                                        \    /
    /// Level 3 (lerp at t):                  split_point
    /// ```
    ///
    /// Each level interpolates between adjacent points from the previous level using
    /// the same parameter t. The split point is where the curve is at parameter t.
    ///
    /// ## The magic:
    ///
    /// The points computed along the left edge (P0, Q0, R0, split_point) form the
    /// control points for the left subcurve (from t=0 to t=split).
    ///
    /// The points along the right edge (split_point, R1, Q2, P3) form the control
    /// points for the right subcurve (from t=split to t=1).
    ///
    /// Together, these two curves are mathematically identical to the original curve,
    /// so there's zero distortion when subdividing.
    ///
    /// ## Why this matters for font editing:
    ///
    /// When a user clicks on a curve to add a point, we need to insert that point
    /// without changing the curve's shape. The de Casteljau algorithm gives us the
    /// exact control points needed to maintain the curve perfectly.
    pub fn subdivide_cubic(cubic: CubicBez, t: f64) -> (CubicBez, CubicBez) {
        // Extract the original curve's control points
        // P0 is the start point, P3 is the end point
        // P1 and P2 are the off-curve control points (bezier handles)
        let p0 = cubic.p0;
        let p1 = cubic.p1;
        let p2 = cubic.p2;
        let p3 = cubic.p3;

        // Level 1: Linearly interpolate between adjacent control points
        // This creates three new points at parameter t along the three line segments:
        // - Q0 is at position t along the line from P0 to P1
        // - Q1 is at position t along the line from P1 to P2
        // - Q2 is at position t along the line from P2 to P3
        let q0 = p0 + (p1 - p0) * t;
        let q1 = p1 + (p2 - p1) * t;
        let q2 = p2 + (p3 - p2) * t;

        // Level 2: Interpolate between the Level 1 points
        // This creates two new points:
        // - R0 is at position t along the line from Q0 to Q1
        // - R1 is at position t along the line from Q1 to Q2
        let r0 = q0 + (q1 - q0) * t;
        let r1 = q1 + (q2 - q1) * t;

        // Level 3: Final interpolation to find the split point
        // The split point is at position t along the line from R0 to R1
        // This is the actual point on the curve at parameter t
        let split_point = r0 + (r1 - r0) * t;

        // Construct the left subcurve from the left edge of the pyramid
        // This curve goes from the original start (P0) to the split point
        // with control points Q0 and R0
        let left = CubicBez::new(p0, q0, r0, split_point);

        // Construct the right subcurve from the right edge of the pyramid
        // This curve goes from the split point to the original end (P3)
        // with control points R1 and Q2
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
