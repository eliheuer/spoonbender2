// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Path abstraction for glyph outlines

use crate::cubic_path::CubicPath;
use crate::entity_id::EntityId;
use crate::workspace;
use kurbo::BezPath;

/// A path in a glyph outline
///
/// Currently only supports cubic bezier paths.
/// HyperPath support can be added later.
#[derive(Debug, Clone)]
pub enum Path {
    /// A cubic bezier path
    Cubic(CubicPath),
}

impl Path {
    /// Convert this path to a kurbo BezPath for rendering
    pub fn to_bezpath(&self) -> BezPath {
        match self {
            Path::Cubic(cubic) => cubic.to_bezpath(),
        }
    }

    /// Get the unique identifier for this path
    pub fn id(&self) -> EntityId {
        match self {
            Path::Cubic(cubic) => cubic.id,
        }
    }

    /// Get the number of points in this path
    pub fn len(&self) -> usize {
        match self {
            Path::Cubic(cubic) => cubic.len(),
        }
    }

    /// Check if this path is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Path::Cubic(cubic) => cubic.is_empty(),
        }
    }

    /// Check if this path is closed
    pub fn is_closed(&self) -> bool {
        match self {
            Path::Cubic(cubic) => cubic.closed,
        }
    }

    /// Get the bounding box of this path
    pub fn bounding_box(&self) -> Option<kurbo::Rect> {
        match self {
            Path::Cubic(cubic) => cubic.bounding_box(),
        }
    }

    /// Convert from a workspace contour (norad format)
    pub fn from_contour(contour: &workspace::Contour) -> Self {
        // For now, always create cubic paths
        // HyperPath support can be added later
        Path::Cubic(CubicPath::from_contour(contour))
    }

    /// Convert this path to a workspace contour (for saving)
    pub fn to_contour(&self) -> workspace::Contour {
        match self {
            Path::Cubic(cubic) => cubic.to_contour(),
        }
    }
}
