// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Select tool for selecting and moving points

use crate::edit_session::EditSession;
use crate::edit_type::EditType;
use crate::mouse::{Drag, MouseDelegate, MouseEvent};
use crate::tools::{Tool, ToolId};
use kurbo::Affine;
use masonry::vello::Scene;

/// The select tool - used for selecting and moving points
#[derive(Debug, Clone, Default)]
pub struct SelectTool {
    /// Whether we're currently dragging points
    dragging_points: bool,

    /// The last mouse position during drag (in design space)
    last_drag_pos: Option<kurbo::Point>,
}

impl Tool for SelectTool {
    fn id(&self) -> ToolId {
        ToolId::Select
    }

    fn paint(&mut self, _scene: &mut Scene, _session: &EditSession, _transform: &Affine) {
        // TODO: Draw selection rectangle if dragging
    }

    fn edit_type(&self) -> Option<EditType> {
        if self.dragging_points {
            Some(EditType::Drag)
        } else {
            None
        }
    }
}

impl MouseDelegate for SelectTool {
    type Data = EditSession;

    fn left_down(&mut self, event: MouseEvent, _data: &mut EditSession) {
        // TODO: Hit test for points under cursor
        println!("SelectTool::left_down pos={:?} shift={}", event.pos, event.mods.shift);
    }

    fn left_up(&mut self, event: MouseEvent, _data: &mut EditSession) {
        // TODO: Finalize selection
        println!("SelectTool::left_up pos={:?} shift={}", event.pos, event.mods.shift);
    }

    fn left_click(&mut self, event: MouseEvent, data: &mut EditSession) {
        println!("SelectTool::left_click called! pos={:?} shift={} current_selection={}", event.pos, event.mods.shift, data.selection.len());

        // Hit test for a point at the cursor
        if let Some(hit) = data.hit_test_point(event.pos, None) {
            println!("Hit point: {:?} distance={}", hit.entity, hit.distance);
            if event.mods.shift {
                println!("Multi-select mode");
                // Shift+click: toggle selection
                let mut new_selection = data.selection.clone();
                if data.selection.contains(&hit.entity) {
                    // Deselect if already selected
                    new_selection.remove(&hit.entity);
                } else {
                    // Add to selection
                    new_selection.insert(hit.entity);
                }
                data.selection = new_selection;
            } else {
                // Normal click: replace selection
                let mut new_selection = crate::selection::Selection::new();
                new_selection.insert(hit.entity);
                data.selection = new_selection;
            }
        } else {
            if !event.mods.shift {
                // Clicked on empty space without shift - clear selection
                data.selection = crate::selection::Selection::new();
            }
            // Shift+click on empty space - keep selection (no action needed)
        }
    }

    fn left_drag_began(&mut self, event: MouseEvent, _drag: Drag, data: &mut EditSession) {
        // Check if we're clicking on a selected point
        if let Some(hit) = data.hit_test_point(event.pos, None) {
            if data.selection.contains(&hit.entity) {
                // We're dragging a selected point
                self.dragging_points = true;
                // Store the starting position in design space
                self.last_drag_pos = Some(data.viewport.from_screen(event.pos));
                println!("Select tool: started dragging {} selected point(s)", data.selection.len());
                return;
            }
        }

        // TODO: Start marquee selection rectangle
        println!("Select tool: drag began (marquee not yet implemented)");
    }

    fn left_drag_changed(&mut self, event: MouseEvent, _drag: Drag, data: &mut EditSession) {
        if self.dragging_points {
            // Convert current mouse position to design space
            let current_pos = data.viewport.from_screen(event.pos);

            if let Some(last_pos) = self.last_drag_pos {
                // Calculate delta in design space
                let delta = kurbo::Vec2::new(
                    current_pos.x - last_pos.x,
                    current_pos.y - last_pos.y,
                );

                // Move selected points
                data.move_selection(delta);

                // Update last position
                self.last_drag_pos = Some(current_pos);
            }
        } else {
            // TODO: Update marquee selection rectangle
        }
    }

    fn left_drag_ended(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut EditSession) {
        if self.dragging_points {
            println!("Select tool: finished dragging points");
        }

        self.dragging_points = false;
        self.last_drag_pos = None;
    }

    fn cancel(&mut self, _data: &mut EditSession) {
        self.dragging_points = false;
        self.last_drag_pos = None;
        println!("Select tool: cancelled");
    }
}
