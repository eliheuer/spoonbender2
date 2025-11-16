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
use masonry::vello::peniko;
use masonry::vello::Scene;
use std::sync::Arc;

/// Distance threshold for closing a path (in design units)
const CLOSE_PATH_DISTANCE: f64 = 20.0;

/// The pen tool - used for drawing new paths
#[derive(Debug, Clone, Default)]
pub struct PenTool {
    /// Points being added to the current path
    current_path_points: Vec<PathPoint>,

    /// Whether we're actively drawing a path
    drawing: bool,

    /// Current mouse position (for hover detection)
    mouse_pos: Option<kurbo::Point>,
}

impl Tool for PenTool {
    fn id(&self) -> ToolId {
        ToolId::Pen
    }

    fn paint(&mut self, scene: &mut Scene, session: &EditSession, _transform: &Affine) {
        // Only draw preview while actively drawing (before finishing/closing)
        if !self.drawing || self.current_path_points.is_empty() {
            return;
        }

        use masonry::vello::peniko::Brush;
        use kurbo::{BezPath, Point};

        let orange_color = crate::theme::point::SELECTED_OUTER;
        let brush = Brush::Solid(orange_color);

        // Check if mouse is hovering near first point (for close feedback)
        let hovering_close = if self.current_path_points.len() >= 3 {
            if let Some(mouse_screen) = self.mouse_pos {
                let mouse_design = session.viewport.from_screen(mouse_screen);
                let first_point = self.current_path_points[0].point;
                let distance = ((mouse_design.x - first_point.x).powi(2) +
                               (mouse_design.y - first_point.y).powi(2)).sqrt();
                distance < CLOSE_PATH_DISTANCE
            } else {
                false
            }
        } else {
            false
        };

        // Draw the preview path (orange lines between points)
        if self.current_path_points.len() >= 2 {
            let mut bez_path = BezPath::new();
            for (i, pt) in self.current_path_points.iter().enumerate() {
                let design_pt = Point::new(pt.point.x, pt.point.y);
                let screen_pt = session.viewport.to_screen(design_pt);

                if i == 0 {
                    bez_path.move_to(screen_pt);
                } else {
                    bez_path.line_to(screen_pt);
                }
            }

            // If hovering near first point, also draw closing line
            if hovering_close {
                if let Some(first_pt) = self.current_path_points.first() {
                    let design_pt = Point::new(first_pt.point.x, first_pt.point.y);
                    let screen_pt = session.viewport.to_screen(design_pt);
                    bez_path.line_to(screen_pt);
                }
            }

            let stroke = kurbo::Stroke::new(2.0);
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &bez_path);
        }

        // Draw circles at each point
        for (i, pt) in self.current_path_points.iter().enumerate() {
            let design_pt = Point::new(pt.point.x, pt.point.y);
            let screen_pt = session.viewport.to_screen(design_pt);

            // First point gets special treatment when hovering
            if i == 0 && hovering_close {
                // Draw larger circle to show close zone
                let close_zone = kurbo::Circle::new(screen_pt, CLOSE_PATH_DISTANCE * session.viewport.zoom);
                let zone_stroke = kurbo::Stroke::new(1.0);
                scene.stroke(&zone_stroke, Affine::IDENTITY, &brush, None, &close_zone);
            }

            // Draw point circle
            let circle = kurbo::Circle::new(screen_pt, 4.0);
            scene.fill(
                peniko::Fill::NonZero,
                Affine::IDENTITY,
                &brush,
                None,
                &circle,
            );
        }
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

        // Check if we're clicking near the first point to close the path
        if self.current_path_points.len() >= 3 {
            let first_point = self.current_path_points[0].point;
            let distance = ((design_pos.x - first_point.x).powi(2) +
                           (design_pos.y - first_point.y).powi(2)).sqrt();

            if distance < CLOSE_PATH_DISTANCE {
                println!("Pen tool: closing path at first point");
                self.close_path(data);
                return;
            }
        }

        // Create a new on-curve point
        let point = PathPoint {
            id: EntityId::next(),
            point: design_pos,
            typ: PointType::OnCurve { smooth: false },
        };

        self.current_path_points.push(point);
        self.drawing = true;

        println!("Pen tool: added point at {:?}, total points: {}", design_pos, self.current_path_points.len());
    }

    fn mouse_moved(&mut self, event: MouseEvent, _data: &mut EditSession) {
        // Track mouse position for hover feedback
        self.mouse_pos = Some(event.pos);
    }

    fn cancel(&mut self, data: &mut EditSession) {
        // Finish the path if we have enough points (Escape key)
        if self.current_path_points.len() >= 2 {
            self.finish_path(data);
        } else {
            // Cancel completely if not enough points
            self.current_path_points.clear();
            self.drawing = false;
        }
        println!("Pen tool: finished/cancelled");
    }
}

impl PenTool {
    /// Add the finished path to the session (open path)
    fn add_open_path(&mut self, data: &mut EditSession) {
        if self.current_path_points.len() < 2 {
            return;
        }

        // Create a new open path from the points
        let path_points = PathPoints::from_vec(self.current_path_points.clone());
        let cubic_path = CubicPath {
            points: path_points,
            closed: false,
            id: EntityId::next(),
        };

        let path = Path::Cubic(cubic_path);
        let mut paths = (*data.paths).clone();
        paths.push(path);
        data.paths = Arc::new(paths);

        println!("Pen tool: added open path with {} points", self.current_path_points.len());
    }

    /// Close the current path and finish drawing
    fn close_path(&mut self, data: &mut EditSession) {
        if self.current_path_points.len() < 3 {
            return;
        }

        // Create a closed path from the points
        let path_points = PathPoints::from_vec(self.current_path_points.clone());
        let cubic_path = CubicPath {
            points: path_points,
            closed: true, // Mark as closed
            id: EntityId::next(),
        };

        let path = Path::Cubic(cubic_path);
        let mut paths = (*data.paths).clone();
        paths.push(path);
        data.paths = Arc::new(paths);

        println!("Pen tool: closed path with {} points", self.current_path_points.len());

        // Reset for next path
        self.current_path_points.clear();
        self.drawing = false;
    }

    /// Finish drawing and reset for next path (called on Escape or tool change)
    pub fn finish_path(&mut self, data: &mut EditSession) {
        // If we have enough points, add the open path
        if self.current_path_points.len() >= 2 {
            self.add_open_path(data);
        }

        self.current_path_points.clear();
        self.drawing = false;
    }
}
