// Copyright 2025 the Runebender Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Point types for bezier paths

use crate::entity_id::EntityId;
use crate::workspace;
use kurbo::Point;

/// A point type in a bezier path
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointType {
    /// A point on the curve
    OnCurve {
        /// Whether this is a smooth point (tangent continuity) or a corner
        smooth: bool,
    },
    /// An off-curve control point (bezier handle)
    OffCurve {
        /// Whether this is an automatically positioned handle
        auto: bool,
    },
}

#[allow(dead_code)]
impl PointType {
    /// Check if this is an on-curve point
    pub fn is_on_curve(&self) -> bool {
        matches!(self, PointType::OnCurve { .. })
    }

    /// Check if this is an off-curve point
    pub fn is_off_curve(&self) -> bool {
        matches!(self, PointType::OffCurve { .. })
    }

    /// Check if this is a smooth on-curve point
    pub fn is_smooth(&self) -> bool {
        matches!(self, PointType::OnCurve { smooth: true })
    }

    /// Check if this is a corner on-curve point
    pub fn is_corner(&self) -> bool {
        matches!(self, PointType::OnCurve { smooth: false })
    }
}

/// A point in a bezier path with unique identity
#[derive(Debug, Clone)]
pub struct PathPoint {
    /// Unique identifier for this point
    pub id: EntityId,

    /// The position in design space
    pub point: Point,

    /// The type of point (on-curve smooth/corner, or off-curve)
    pub typ: PointType,
}

#[allow(dead_code)]
impl PathPoint {
    /// Create a new path point
    pub fn new(point: Point, typ: PointType) -> Self {
        Self {
            id: EntityId::next(),
            point,
            typ,
        }
    }

    /// Create a new on-curve smooth point
    pub fn on_curve_smooth(point: Point) -> Self {
        Self::new(point, PointType::OnCurve { smooth: true })
    }

    /// Create a new on-curve corner point
    pub fn on_curve_corner(point: Point) -> Self {
        Self::new(point, PointType::OnCurve { smooth: false })
    }

    /// Create a new off-curve control point
    pub fn off_curve(point: Point) -> Self {
        Self::new(point, PointType::OffCurve { auto: false })
    }

    /// Check if this point is on the curve
    pub fn is_on_curve(&self) -> bool {
        self.typ.is_on_curve()
    }

    /// Check if this point is off the curve (control point)
    pub fn is_off_curve(&self) -> bool {
        self.typ.is_off_curve()
    }

    /// Convert from a workspace contour point (norad format)
    pub fn from_contour_point(pt: &workspace::ContourPoint) -> Self {
        let point = Point::new(pt.x, pt.y);
        let typ = PointType::from_workspace_type(pt.point_type);
        Self::new(point, typ)
    }
}

#[allow(dead_code)]
impl PointType {
    /// Convert from workspace point type (norad format)
    pub fn from_workspace_type(pt_type: workspace::PointType) -> Self {
        match pt_type {
            workspace::PointType::Move => PointType::OnCurve { smooth: false },
            workspace::PointType::Line => PointType::OnCurve { smooth: false },
            workspace::PointType::Curve => PointType::OnCurve { smooth: true },
            workspace::PointType::OffCurve => PointType::OffCurve { auto: false },
            workspace::PointType::QCurve => {
                // For now, treat QCurve as a smooth on-curve point
                // Proper quadratic support would require more work
                PointType::OnCurve { smooth: true }
            }
        }
    }
}
