// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Select tool for selecting and moving points

use crate::edit_session::EditSession;
use crate::edit_type::EditType;
use crate::mouse::{Drag, MouseDelegate, MouseEvent};
use crate::selection::Selection;
use crate::tools::{Tool, ToolId};
use kurbo::Affine;
use masonry::vello::Scene;

/// Internal state for the select tool
#[derive(Debug, Clone)]
enum State {
    /// Ready to start an interaction
    Ready,
    /// Dragging selected points
    DraggingPoints {
        /// Last mouse position in design space
        last_pos: kurbo::Point,
    },
    /// Marquee selection (dragging out a rectangle)
    MarqueeSelect {
        /// Selection before this marquee started (for shift+toggle mode)
        previous_selection: Selection,
        /// The selection rectangle in screen space
        rect: kurbo::Rect,
        /// Whether shift is held (toggle mode)
        toggle: bool,
    },
}

impl Default for State {
    fn default() -> Self {
        State::Ready
    }
}

/// The select tool - used for selecting and moving points
#[derive(Debug, Clone, Default)]
pub struct SelectTool {
    /// Current tool state
    state: State,
}

impl Tool for SelectTool {
    fn id(&self) -> ToolId {
        ToolId::Select
    }

    fn paint(&mut self, scene: &mut Scene, _session: &EditSession, _transform: &Affine) {
        // Draw selection rectangle if in marquee mode
        if let State::MarqueeSelect { rect, .. } = &self.state {
            use masonry::util::fill_color;
            use masonry::vello::peniko::Brush;

            // Fill the selection rectangle with semi-transparent orange
            fill_color(scene, rect, crate::theme::selection::RECT_FILL);

            // Stroke the selection rectangle with dashed bright orange
            // Create a dashed stroke pattern: 4px dash, 4px gap
            let stroke = kurbo::Stroke::new(1.5)
                .with_dashes(0.0, [4.0, 4.0]);
            let brush = Brush::Solid(crate::theme::selection::RECT_STROKE);
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, rect);
        }
    }

    fn edit_type(&self) -> Option<EditType> {
        match &self.state {
            State::DraggingPoints { .. } => Some(EditType::Drag),
            _ => None,
        }
    }
}

impl MouseDelegate for SelectTool {
    type Data = EditSession;

    fn left_down(&mut self, event: MouseEvent, data: &mut EditSession) {
        println!("SelectTool::left_down pos={:?} shift={}", event.pos, event.mods.shift);

        // Hit test for a point at the cursor - selection happens HERE, on mouse down
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
                data.update_coord_selection();
            } else {
                // Normal click: replace selection with just this point
                // UNLESS the point is already selected (then keep current selection for dragging)
                if !data.selection.contains(&hit.entity) {
                    let mut new_selection = crate::selection::Selection::new();
                    new_selection.insert(hit.entity);
                    data.selection = new_selection;
                    data.update_coord_selection();
                }
            }
        } else {
            if !event.mods.shift {
                // Clicked on empty space without shift - clear selection
                data.selection = crate::selection::Selection::new();
                data.update_coord_selection();
            }
            // Shift+click on empty space - keep selection (no action needed)
        }
    }

    fn left_up(&mut self, _event: MouseEvent, _data: &mut EditSession) {
        // Selection already happened in left_down, nothing to do here
    }

    fn left_click(&mut self, _event: MouseEvent, _data: &mut EditSession) {
        // Click is now handled entirely by left_down
        // This method is called after left_up if no drag occurred
        // But we don't need to do anything here since selection already happened
    }

    fn left_drag_began(&mut self, event: MouseEvent, drag: Drag, data: &mut EditSession) {
        // Check if we have any selected points
        // (They were already selected in left_down)
        if !data.selection.is_empty() {
            // Check if we're starting the drag on a selected point
            if let Some(hit) = data.hit_test_point(event.pos, None) {
                if data.selection.contains(&hit.entity) {
                    // We're dragging a selected point
                    let design_pos = data.viewport.from_screen(event.pos);
                    self.state = State::DraggingPoints { last_pos: design_pos };
                    println!("Select tool: started dragging {} selected point(s)", data.selection.len());
                    return;
                }
            }
        }

        // Start marquee selection
        // Store the previous selection for toggle mode
        let previous_selection = data.selection.clone();
        let rect = kurbo::Rect::from_points(drag.start, drag.current);

        println!("Select tool: started marquee selection, toggle={}", event.mods.shift);
        self.state = State::MarqueeSelect {
            previous_selection,
            rect,
            toggle: event.mods.shift,
        };
    }

    fn left_drag_changed(&mut self, event: MouseEvent, drag: Drag, data: &mut EditSession) {
        match &mut self.state {
            State::DraggingPoints { last_pos } => {
                // Convert current mouse position to design space
                let current_pos = data.viewport.from_screen(event.pos);

                // Calculate delta in design space
                let delta = kurbo::Vec2::new(
                    current_pos.x - last_pos.x,
                    current_pos.y - last_pos.y,
                );

                // Move selected points
                data.move_selection(delta);

                // Update last position
                *last_pos = current_pos;
            }
            State::MarqueeSelect { previous_selection, rect, toggle } => {
                // Update the selection rectangle
                *rect = kurbo::Rect::from_points(drag.start, drag.current);

                // Update selection based on points in rectangle
                update_selection_for_marquee(
                    data,
                    previous_selection,
                    *rect,
                    *toggle,
                );
            }
            State::Ready => {}
        }
    }

    fn left_drag_ended(&mut self, _event: MouseEvent, _drag: Drag, data: &mut EditSession) {
        match &self.state {
            State::DraggingPoints { .. } => {
                println!("Select tool: finished dragging points");
            }
            State::MarqueeSelect { .. } => {
                println!("Select tool: finished marquee selection, selected {} points", data.selection.len());
                // Update coordinate selection after marquee
                data.update_coord_selection();
            }
            State::Ready => {}
        }

        // Return to ready state
        self.state = State::Ready;
    }

    fn cancel(&mut self, data: &mut EditSession) {
        // If we were in marquee mode, restore the previous selection
        if let State::MarqueeSelect { previous_selection, .. } = &self.state {
            data.selection = previous_selection.clone();
            data.update_coord_selection();
        }

        self.state = State::Ready;
        println!("Select tool: cancelled");
    }
}

/// Update selection based on points in the marquee rectangle
///
/// This filters all points to find those within the rectangle (in screen space),
/// and applies toggle logic if shift is held.
fn update_selection_for_marquee(
    data: &mut EditSession,
    previous_selection: &Selection,
    rect: kurbo::Rect,
    toggle: bool,
) {
    use crate::path::Path;

    // Collect all points that are within the selection rectangle
    let mut new_selection = Selection::new();

    for path in data.paths.iter() {
        match path {
            Path::Cubic(cubic) => {
                for pt in cubic.points.iter() {
                    // Convert point to screen space for hit testing
                    let screen_pos = data.viewport.to_screen(pt.point);

                    // Check if point is inside the rectangle
                    if rect.contains(screen_pos) {
                        new_selection.insert(pt.id);
                    }
                }
            }
        }
    }

    // Apply toggle logic if shift is held
    if toggle {
        // Symmetric difference: (previous ∪ new) - (previous ∩ new)
        // This toggles: adds new points, removes previously selected points that are also in new
        let mut result = Selection::new();

        // Add points that are in previous but not in new
        for id in previous_selection.iter() {
            if !new_selection.contains(id) {
                result.insert(*id);
            }
        }

        // Add points that are in new but not in previous
        for id in new_selection.iter() {
            if !previous_selection.contains(id) {
                result.insert(*id);
            }
        }

        data.selection = result;
    } else {
        // Normal mode: replace selection with points in rectangle
        data.selection = new_selection;
    }
}
