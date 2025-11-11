// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Pen tool for drawing new paths

use crate::edit_session::EditSession;
use crate::edit_type::EditType;
use crate::entity_id::EntityId;
use crate::mouse::{Drag, MouseDelegate, MouseEvent};
use crate::point::{PathPoint, PointType};
use crate::path::Path;
use crate::cubic_path::CubicPath;
use crate::point_list::PathPoints;
use crate::tools::{Tool, ToolId};
use kurbo::Affine;
use masonry::vello::Scene;
use std::sync::Arc;

/// The pen tool - used for drawing new paths
#[derive(Debug, Clone, Default)]
pub struct PenTool {
    /// Points being added to the current path
    current_path_points: Vec<PathPoint>,

    /// Whether we're actively drawing a path
    drawing: bool,
}

impl Tool for PenTool {
    fn id(&self) -> ToolId {
        ToolId::Pen
    }

    fn paint(&mut self, _scene: &mut Scene, _session: &EditSession, _transform: &Affine) {
        // TODO: Draw preview of new path
    }

    fn edit_type(&self) -> Option<EditType> {
        if self.drawing {
            Some(EditType::Normal)
        } else {
            None
        }
    }
}

impl MouseDelegate for PenTool {
    type Data = EditSession;

    fn left_click(&mut self, event: MouseEvent, data: &mut EditSession) {
        // Convert screen position to design space
        let design_pos = data.viewport.from_screen(event.pos);

        // Create a new on-curve point
        let point = PathPoint {
            id: EntityId::next(),
            point: design_pos,
            typ: PointType::OnCurve { smooth: false },
        };

        self.current_path_points.push(point);
        self.drawing = true;

        println!("Pen tool: added point at {:?}, total points: {}", design_pos, self.current_path_points.len());

        // If we have at least 2 points, we can create/update a path
        if self.current_path_points.len() >= 2 {
            self.commit_path(data);
        }
    }

    fn cancel(&mut self, data: &mut EditSession) {
        // Commit any pending path
        if self.current_path_points.len() >= 2 {
            self.commit_path(data);
        }

        self.current_path_points.clear();
        self.drawing = false;
        println!("Pen tool: cancelled");
    }
}

impl PenTool {
    /// Commit the current path to the session
    fn commit_path(&mut self, data: &mut EditSession) {
        if self.current_path_points.len() < 2 {
            return;
        }

        // Create a new path from the points
        let path_points = PathPoints::from_vec(self.current_path_points.clone());
        let cubic_path = CubicPath {
            points: path_points,
            closed: false,
            id: EntityId::next(),
        };

        // Add the path to the session
        let mut paths = (*data.paths).clone();
        paths.push(Path::Cubic(cubic_path));
        data.paths = Arc::new(paths);

        println!("Pen tool: committed path with {} points", self.current_path_points.len());
    }

    /// Finish drawing and reset for next path
    pub fn finish_path(&mut self, data: &mut EditSession) {
        self.commit_path(data);
        self.current_path_points.clear();
        self.drawing = false;
    }
}
