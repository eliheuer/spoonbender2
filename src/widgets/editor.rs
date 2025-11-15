// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Glyph editor widget - the main canvas for editing glyphs

use crate::edit_session::EditSession;
use crate::edit_type::EditType;
use crate::glyph_renderer;
use crate::mouse::{Mouse, MouseButton, MouseEvent as MouseEvt};
use crate::point::PointType;
use crate::theme;
use crate::undo::UndoState;
use kurbo::{Affine, Circle, Point, Rect as KurboRect, Shape, Stroke};
use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, EventCtx, LayoutCtx, NoAction, PaintCtx,
    PointerButton, PointerButtonEvent, PointerEvent, PointerUpdate,
    PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx, Widget,
};
use masonry::kurbo::Size;
use masonry::util::fill_color;
use masonry::vello::peniko::{Brush, Color};
use masonry::vello::Scene;
use std::sync::Arc;

/// Zoom constraints (from Runebender)
const MIN_ZOOM: f64 = 0.02;
const MAX_ZOOM: f64 = 50.0;
const ZOOM_SCALE: f64 = 0.001;

/// The main glyph editor canvas widget
pub struct EditorWidget {
    /// The editing session (mutable copy for editing)
    pub session: EditSession,

    /// Mouse state machine
    mouse: Mouse,

    /// Canvas size
    size: Size,

    /// Undo/redo state
    undo: UndoState<EditSession>,

    /// The last edit type (for grouping consecutive edits)
    last_edit_type: Option<EditType>,

    /// Tool to return to when spacebar is released (for temporary preview mode)
    previous_tool: Option<crate::tools::ToolId>,
}

impl EditorWidget {
    /// Create a new editor widget
    pub fn new(session: Arc<EditSession>) -> Self {
        // Clone the session to get a mutable copy
        // This is cheap due to Arc-based fields
        Self {
            session: (*session).clone(),
            mouse: Mouse::new(),
            size: Size::new(800.0, 600.0),
            undo: UndoState::new(),
            last_edit_type: None,
            previous_tool: None,
        }
    }

    /// Set the canvas size
    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Record an edit operation for undo
    ///
    /// This manages undo grouping:
    /// - If the edit type matches the last edit, update the current undo group
    /// - If the edit type is different, create a new undo group
    fn record_edit(&mut self, edit_type: EditType) {
        match self.last_edit_type {
            Some(last) if last == edit_type => {
                // Same edit type - update current undo group
                self.undo.update_current_undo(self.session.clone());
            }
            _ => {
                // Different edit type or first edit - create new undo group
                self.undo.add_undo_group(self.session.clone());
                self.last_edit_type = Some(edit_type);
            }
        }
    }

    /// Undo the last edit
    fn undo(&mut self) {
        if let Some(previous) = self.undo.undo(self.session.clone()) {
            self.session = previous;
            println!("Undo: restored previous state");
        }
    }

    /// Redo the last undone edit
    fn redo(&mut self) {
        if let Some(next) = self.undo.redo(self.session.clone()) {
            self.session = next;
            println!("Redo: restored next state");
        }
    }

}

/// Action emitted by the editor widget when the session is updated
#[derive(Debug, Clone)]
pub struct SessionUpdate {
    pub session: EditSession,
}

impl Widget for EditorWidget {
    type Action = SessionUpdate;

    fn accepts_focus(&self) -> bool {
        // Allow this widget to receive keyboard events
        true
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {
        // Leaf widget - no children
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
        // TODO: Handle updates to the session
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        // Use all available space (expand to fill the window)
        let size = bc.max();
        self.size = size;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        let canvas_size = ctx.size();

        // Fill background
        let bg_rect = canvas_size.to_rect();
        fill_color(scene, &bg_rect, crate::theme::canvas::BACKGROUND);

        // Get the glyph outline from the editable paths
        let mut glyph_path = kurbo::BezPath::new();
        for path in self.session.paths.iter() {
            glyph_path.extend(path.to_bezpath());
        }

        // Initialize viewport on first paint
        if !self.session.viewport_initialized {
            // Calculate initial viewport positioning to center the glyph
            let ascender = self.session.ascender;
            let descender = self.session.descender;

            // Calculate the visible height in design space
            let design_height = ascender - descender;

            // Center the viewport on the canvas
            let center_x = canvas_size.width / 2.0;
            let center_y = canvas_size.height / 2.0;

            // Create a transform that:
            // 1. Scales to fit the canvas (with some padding)
            // 2. Centers the glyph
            let padding = 0.8; // Leave 20% padding
            let scale = (canvas_size.height * padding) / design_height;

            // Center point in design space (middle of advance width, middle of height)
            let design_center_x = self.session.glyph.width / 2.0;
            let design_center_y = (ascender + descender) / 2.0;

            // Update the viewport to match our rendering transform
            // The viewport uses: zoom (scale) and offset (translation after scale)
            self.session.viewport.zoom = scale;
            // Offset calculation based on to_screen formula:
            // screen.x = design.x * zoom + offset.x
            // screen.y = -design.y * zoom + offset.y
            // For design_center to map to canvas_center:
            self.session.viewport.offset = kurbo::Vec2::new(
                center_x - design_center_x * scale,
                center_y + design_center_y * scale, // Y is flipped in to_affine
            );

            self.session.viewport_initialized = true;
        }

        // Build transform from viewport (always uses current zoom/offset)
        let transform = self.session.viewport.affine();

        // Check if we're in preview mode (Preview tool is active)
        let is_preview_mode = self.session.current_tool.id() == crate::tools::ToolId::Preview;

        if !is_preview_mode {
            // Edit mode: Draw font metrics guides
            draw_metrics_guides(scene, &transform, &self.session, canvas_size);
        }

        if glyph_path.is_empty() {
            return;
        }

        // Apply transform to path
        let transformed_path = transform * &glyph_path;

        if is_preview_mode {
            // Preview mode: Fill the glyph with light gray (visible on dark theme)
            let fill_brush = Brush::Solid(theme::path::PREVIEW_FILL);
            scene.fill(
                peniko::Fill::NonZero,
                Affine::IDENTITY,
                &fill_brush,
                None,
                &transformed_path,
            );
        } else {
            // Edit mode: Draw the glyph outline with stroke
            let stroke = Stroke::new(theme::size::PATH_STROKE_WIDTH);
            let brush = Brush::Solid(theme::path::STROKE);
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &transformed_path);

            // Draw control point lines and points
            draw_paths_with_points(scene, &self.session, &transform);

            // Draw tool overlays (e.g., selection rectangle for marquee)
            // Temporarily take ownership of the tool to call paint (requires &mut)
            let mut tool = std::mem::replace(
                &mut self.session.current_tool,
                crate::tools::ToolBox::for_id(crate::tools::ToolId::Select)
            );
            tool.paint(scene, &self.session, &transform);
            self.session.current_tool = tool;
        }
    }

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        use crate::mouse::{MouseButton, MouseEvent};
        use crate::tools::{ToolBox, ToolId};

        match event {
            PointerEvent::Down(PointerButtonEvent { button: Some(PointerButton::Primary), state, .. }) => {
                println!("[EditorWidget::on_pointer_event] Down at {:?}, current_tool: {:?}",
                         state.position, self.session.current_tool.id());

                // Request focus to receive keyboard events
                println!("[EditorWidget] Requesting focus!");
                ctx.request_focus();

                // Capture pointer to receive drag events
                ctx.capture_pointer();

                let local_pos = ctx.local_position(state.position);

                // Extract modifier keys from pointer state
                // state.modifiers is keyboard_types::Modifiers from ui-events crate
                use crate::mouse::Modifiers;
                let mods = Modifiers {
                    shift: state.modifiers.shift(),
                    ctrl: state.modifiers.ctrl(),
                    alt: state.modifiers.alt(),
                    meta: state.modifiers.meta(),
                };

                // Create MouseEvent for our mouse state machine
                let mouse_event = MouseEvent::with_modifiers(local_pos, Some(MouseButton::Left), mods);

                // Temporarily take ownership of the tool to avoid borrow conflicts
                let mut tool = std::mem::replace(&mut self.session.current_tool, ToolBox::for_id(ToolId::Select));
                self.mouse.mouse_down(mouse_event, &mut tool, &mut self.session);
                self.session.current_tool = tool;

                ctx.request_render();
            }

            PointerEvent::Move(PointerUpdate { current, .. }) => {
                let local_pos = ctx.local_position(current.position);

                // Create MouseEvent
                let mouse_event = MouseEvent::new(local_pos, None);

                // Temporarily take ownership of the tool
                let mut tool = std::mem::replace(&mut self.session.current_tool, ToolBox::for_id(ToolId::Select));
                self.mouse.mouse_moved(mouse_event, &mut tool, &mut self.session);
                self.session.current_tool = tool;

                // During active drag, just render - don't emit actions (to avoid breaking mouse state)
                if ctx.is_active() {
                    ctx.request_render();
                }
            }

            PointerEvent::Up(PointerButtonEvent { button: Some(PointerButton::Primary), state, .. }) => {
                let local_pos = ctx.local_position(state.position);

                // Extract modifier keys from pointer state
                use crate::mouse::Modifiers;
                let mods = Modifiers {
                    shift: state.modifiers.shift(),
                    ctrl: state.modifiers.ctrl(),
                    alt: state.modifiers.alt(),
                    meta: state.modifiers.meta(),
                };

                // Create MouseEvent with modifiers
                let mouse_event = MouseEvent::with_modifiers(local_pos, Some(MouseButton::Left), mods);

                // Temporarily take ownership of the tool
                let mut tool = std::mem::replace(&mut self.session.current_tool, ToolBox::for_id(ToolId::Select));
                self.mouse.mouse_up(mouse_event, &mut tool, &mut self.session);

                // Record undo if an edit occurred
                if let Some(edit_type) = tool.edit_type() {
                    self.record_edit(edit_type);
                }

                self.session.current_tool = tool;

                // Update coordinate selection after tool operation
                self.session.update_coord_selection();

                // Emit action to notify view of session changes
                ctx.submit_action::<SessionUpdate>(SessionUpdate {
                    session: self.session.clone(),
                });

                ctx.release_pointer();
                ctx.request_render();
            }

            PointerEvent::Cancel(_) => {
                // Temporarily take ownership of the tool
                let mut tool = std::mem::replace(&mut self.session.current_tool, ToolBox::for_id(ToolId::Select));
                self.mouse.cancel(&mut tool, &mut self.session);
                self.session.current_tool = tool;

                ctx.request_render();
            }

            _ => {
                // TODO: Implement wheel event handling once Masonry exposes it
                // For now, zooming can be done via keyboard shortcuts or commands
            }
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        use masonry::core::keyboard::{Key, KeyState, NamedKey};

        match event {
            TextEvent::Keyboard(key_event) => {
                println!("[EditorWidget::on_text_event] key: {:?}, state: {:?}", key_event.key, key_event.state);

                // Handle spacebar for temporary preview mode (both down and up)
                if matches!(&key_event.key, Key::Character(c) if c == " ") {
                    println!("[EditorWidget] Spacebar detected! state: {:?}, previous_tool: {:?}", key_event.state, self.previous_tool);

                    if key_event.state == KeyState::Down && self.previous_tool.is_none() {
                        // Spacebar pressed: save current tool and switch to Preview
                        let current_tool = self.session.current_tool.id();
                        if current_tool != crate::tools::ToolId::Preview {
                            self.previous_tool = Some(current_tool);

                            // Cancel the current tool and reset mouse state (like Runebender)
                            use crate::tools::ToolBox;
                            let mut tool = std::mem::replace(&mut self.session.current_tool, ToolBox::for_id(crate::tools::ToolId::Select));
                            self.mouse.cancel(&mut tool, &mut self.session);

                            // Reset mouse state by creating new instance
                            self.mouse = Mouse::new();

                            // Switch to Preview tool
                            self.session.current_tool = ToolBox::for_id(crate::tools::ToolId::Preview);

                            println!("Spacebar down: switched to Preview, will return to {:?}", current_tool);

                            // Emit SessionUpdate so the toolbar reflects the change
                            ctx.submit_action::<SessionUpdate>(SessionUpdate {
                                session: self.session.clone(),
                            });

                            ctx.request_render();
                            ctx.set_handled();
                        }
                        return;
                    } else if key_event.state == KeyState::Up && self.previous_tool.is_some() {
                        // Spacebar released: return to previous tool
                        if let Some(previous) = self.previous_tool.take() {
                            // Reset mouse state by creating new instance
                            self.mouse = Mouse::new();

                            self.session.current_tool = crate::tools::ToolBox::for_id(previous);
                            println!("Spacebar up: returned to {:?}", previous);

                            // Emit SessionUpdate so the toolbar reflects the change
                            ctx.submit_action::<SessionUpdate>(SessionUpdate {
                                session: self.session.clone(),
                            });

                            ctx.request_render();
                            ctx.set_handled();
                        }
                        return;
                    }
                }

                // Only handle key down events for other keys
                if key_event.state != KeyState::Down {
                    return;
                }

                // Check for keyboard shortcuts
                let cmd = key_event.modifiers.meta() || key_event.modifiers.ctrl();
                let shift = key_event.modifiers.shift();

                // Undo/Redo
                if cmd && matches!(&key_event.key, Key::Character(c) if c == "z") {
                    if shift {
                        // Cmd+Shift+Z = Redo
                        self.redo();
                        ctx.request_render();
                        ctx.set_handled();
                        return;
                    } else {
                        // Cmd+Z = Undo
                        self.undo();
                        ctx.request_render();
                        ctx.set_handled();
                        return;
                    }
                }

                // Zoom in (Cmd/Ctrl + or =)
                if cmd && matches!(&key_event.key, Key::Character(c) if c == "+" || c == "=") {
                    let new_zoom = (self.session.viewport.zoom * 1.1).min(MAX_ZOOM);
                    self.session.viewport.zoom = new_zoom;
                    println!("Zoom in: new zoom = {:.2}", new_zoom);
                    ctx.request_render();
                    ctx.set_handled();
                    return;
                }

                // Zoom out (Cmd/Ctrl -)
                if cmd && matches!(&key_event.key, Key::Character(c) if c == "-") {
                    let new_zoom = (self.session.viewport.zoom / 1.1).max(MIN_ZOOM);
                    self.session.viewport.zoom = new_zoom;
                    println!("Zoom out: new zoom = {:.2}", new_zoom);
                    ctx.request_render();
                    ctx.set_handled();
                    return;
                }

                // Fit to window (Cmd/Ctrl+0)
                if cmd && matches!(&key_event.key, Key::Character(c) if c == "0") {
                    // Reset viewport to fit glyph in window
                    self.session.viewport_initialized = false;
                    println!("Fit to window: resetting viewport");
                    ctx.request_render();
                    ctx.set_handled();
                    return;
                }

                // Delete selected points (Backspace or Delete key)
                if matches!(&key_event.key, Key::Named(NamedKey::Backspace) | Key::Named(NamedKey::Delete)) {
                    self.session.delete_selection();
                    self.record_edit(EditType::Normal);
                    ctx.request_render();
                    ctx.set_handled();
                    return;
                }

                // Toggle point type (T key)
                if matches!(&key_event.key, Key::Character(c) if c == "t") {
                    self.session.toggle_point_type();
                    self.record_edit(EditType::Normal);
                    ctx.request_render();
                    ctx.set_handled();
                    return;
                }

                // Reverse contours (R key)
                if matches!(&key_event.key, Key::Character(c) if c == "r") {
                    self.session.reverse_contours();
                    self.record_edit(EditType::Normal);
                    ctx.request_render();
                    ctx.set_handled();
                    return;
                }

                // Handle arrow keys for nudging
                let (dx, dy) = match &key_event.key {
                    Key::Named(NamedKey::ArrowLeft) => {
                        println!("Arrow Left pressed");
                        (-1.0, 0.0)
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        println!("Arrow Right pressed");
                        (1.0, 0.0)
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        println!("Arrow Up pressed");
                        (0.0, 1.0)    // Design space: Y increases upward
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        println!("Arrow Down pressed");
                        (0.0, -1.0)  // Design space: Y increases upward
                    }
                    _ => return,
                };

                let shift = key_event.modifiers.shift();
                let ctrl = key_event.modifiers.ctrl() || key_event.modifiers.meta();

                println!("Nudging selection: dx={} dy={} shift={} ctrl={} selection_len={}",
                    dx, dy, shift, ctrl, self.session.selection.len());

                self.session.nudge_selection(dx, dy, shift, ctrl);
                ctx.request_render();
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Canvas
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.set_label(format!("Editing glyph: {}", self.session.glyph_name));
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }
}

/// Draw font metric guidelines
fn draw_metrics_guides(
    scene: &mut Scene,
    transform: &Affine,
    session: &EditSession,
    _canvas_size: Size,
) {
    let stroke = Stroke::new(theme::size::METRIC_LINE_WIDTH);
    let brush = Brush::Solid(theme::metrics::GUIDE);

    // Helper to draw a horizontal line at a given Y coordinate in design space
    // Lines are contained within the metrics box (from x=0 to x=advance_width)
    let draw_hline = |scene: &mut Scene, y: f64| {
        let start = Point::new(0.0, y);
        let end = Point::new(session.glyph.width, y);

        let start_screen = *transform * start;
        let end_screen = *transform * end;

        let line = kurbo::Line::new(start_screen, end_screen);
        scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line);
    };

    // Helper to draw a vertical line at a given X coordinate in design space
    // Lines are contained within the metrics box (from y=descender to y=ascender)
    let draw_vline = |scene: &mut Scene, x: f64| {
        let start = Point::new(x, session.descender);
        let end = Point::new(x, session.ascender);

        let start_screen = *transform * start;
        let end_screen = *transform * end;

        let line = kurbo::Line::new(start_screen, end_screen);
        scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line);
    };

    // Draw vertical lines (left and right edges of metrics box)
    draw_vline(scene, 0.0);
    draw_vline(scene, session.glyph.width);

    // Draw horizontal lines
    // Descender (bottom of metrics box)
    draw_hline(scene, session.descender);

    // Baseline (y=0)
    draw_hline(scene, 0.0);

    // X-height (if available)
    if let Some(x_height) = session.x_height {
        draw_hline(scene, x_height);
    }

    // Cap-height (if available)
    if let Some(cap_height) = session.cap_height {
        draw_hline(scene, cap_height);
    }

    // Ascender (top of metrics box)
    draw_hline(scene, session.ascender);
}

/// Draw paths with control point lines and styled points
fn draw_paths_with_points(scene: &mut Scene, session: &EditSession, transform: &Affine) {
    use crate::path::Path;

    // First pass: draw control point lines (handles)
    // In cubic bezier curves, handles connect on-curve points to their adjacent off-curve control points
    for path in session.paths.iter() {
        match path {
            Path::Cubic(cubic) => {
                let points: Vec<_> = cubic.points.iter().collect();
                if points.is_empty() {
                    continue;
                }

                // For each point, if it's on-curve, draw handles to adjacent off-curve points
                for i in 0..points.len() {
                    let pt = points[i];

                    if pt.is_on_curve() {
                        // Look at the next point (with wrapping for closed paths)
                        let next_i = if i + 1 < points.len() {
                            i + 1
                        } else if cubic.closed {
                            0
                        } else {
                            continue;
                        };

                        // Look at the previous point (with wrapping for closed paths)
                        let prev_i = if i > 0 {
                            i - 1
                        } else if cubic.closed {
                            points.len() - 1
                        } else {
                            continue;
                        };

                        // Draw handle to next point if it's off-curve
                        if next_i < points.len() && points[next_i].is_off_curve() {
                            let start = *transform * pt.point;
                            let end = *transform * points[next_i].point;
                            let line = kurbo::Line::new(start, end);
                            let stroke = Stroke::new(theme::size::HANDLE_LINE_WIDTH);
                            let brush = Brush::Solid(theme::handle::LINE);
                            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line);
                        }

                        // Draw handle to previous point if it's off-curve
                        if prev_i < points.len() && points[prev_i].is_off_curve() {
                            let start = *transform * pt.point;
                            let end = *transform * points[prev_i].point;
                            let line = kurbo::Line::new(start, end);
                            let stroke = Stroke::new(theme::size::HANDLE_LINE_WIDTH);
                            let brush = Brush::Solid(theme::handle::LINE);
                            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line);
                        }
                    }
                }
            }
        }
    }

    // Second pass: draw points
    for path in session.paths.iter() {
        match path {
            Path::Cubic(cubic) => {
                for pt in cubic.points.iter() {
                    let screen_pos = *transform * pt.point;
                    let is_selected = session.selection.contains(&pt.id);

                    match pt.typ {
                        PointType::OnCurve { smooth } => {
                            if smooth {
                                // Draw smooth point as circle
                                let radius = if is_selected {
                                    theme::size::SMOOTH_POINT_SELECTED_RADIUS
                                } else {
                                    theme::size::SMOOTH_POINT_RADIUS
                                };

                                let (inner_color, outer_color) = if is_selected {
                                    (theme::point::SELECTED_INNER, theme::point::SELECTED_OUTER)
                                } else {
                                    (theme::point::SMOOTH_INNER, theme::point::SMOOTH_OUTER)
                                };

                                // Outer circle (border)
                                let outer_circle = Circle::new(screen_pos, radius + 1.0);
                                fill_color(scene, &outer_circle, outer_color);

                                // Inner circle
                                let inner_circle = Circle::new(screen_pos, radius);
                                fill_color(scene, &inner_circle, inner_color);
                            } else {
                                // Draw corner point as square
                                let half_size = if is_selected {
                                    theme::size::CORNER_POINT_SELECTED_HALF_SIZE
                                } else {
                                    theme::size::CORNER_POINT_HALF_SIZE
                                };

                                let (inner_color, outer_color) = if is_selected {
                                    (theme::point::SELECTED_INNER, theme::point::SELECTED_OUTER)
                                } else {
                                    (theme::point::CORNER_INNER, theme::point::CORNER_OUTER)
                                };

                                // Outer square (border)
                                let outer_rect = KurboRect::new(
                                    screen_pos.x - half_size - 1.0,
                                    screen_pos.y - half_size - 1.0,
                                    screen_pos.x + half_size + 1.0,
                                    screen_pos.y + half_size + 1.0,
                                );
                                fill_color(scene, &outer_rect, outer_color);

                                // Inner square
                                let inner_rect = KurboRect::new(
                                    screen_pos.x - half_size,
                                    screen_pos.y - half_size,
                                    screen_pos.x + half_size,
                                    screen_pos.y + half_size,
                                );
                                fill_color(scene, &inner_rect, inner_color);
                            }
                        }
                        PointType::OffCurve { .. } => {
                            // Draw off-curve point as small circle
                            let radius = if is_selected {
                                theme::size::OFFCURVE_POINT_SELECTED_RADIUS
                            } else {
                                theme::size::OFFCURVE_POINT_RADIUS
                            };

                            let (inner_color, outer_color) = if is_selected {
                                (theme::point::SELECTED_INNER, theme::point::SELECTED_OUTER)
                            } else {
                                (theme::point::OFFCURVE_INNER, theme::point::OFFCURVE_OUTER)
                            };

                            // Outer circle (border)
                            let outer_circle = Circle::new(screen_pos, radius + 1.0);
                            fill_color(scene, &outer_circle, outer_color);

                            // Inner circle
                            let inner_circle = Circle::new(screen_pos, radius);
                            fill_color(scene, &inner_circle, inner_color);
                        }
                    }
                }
            }
        }
    }
}

// --- MARK: XILEM VIEW WRAPPER ---

use std::marker::PhantomData;
use xilem::core::{MessageContext, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

/// Create an editor view from an edit session with a callback for session updates
pub fn editor_view<State, F>(
    session: Arc<EditSession>,
    on_session_update: F,
) -> EditorView<State, F>
where
    F: Fn(&mut State, EditSession),
{
    EditorView {
        session,
        on_session_update,
        phantom: PhantomData,
    }
}

/// The Xilem View for EditorWidget
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct EditorView<State, F> {
    session: Arc<EditSession>,
    on_session_update: F,
    phantom: PhantomData<fn() -> State>,
}

impl<State, F> ViewMarker for EditorView<State, F> {}

impl<State: 'static, F: Fn(&mut State, EditSession) + 'static> View<State, (), ViewCtx> for EditorView<State, F> {
    type Element = Pod<EditorWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let widget = EditorWidget::new(self.session.clone());
        let pod = ctx.create_pod(widget);
        ctx.record_action(pod.new_widget.id());
        (pod, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        // Update the widget's session if it changed (e.g., tool selection changed)
        // We compare Arc pointers - if they're different, the session was updated
        if !Arc::ptr_eq(&self.session, &prev.session) {
            println!("[EditorView::rebuild] Session Arc changed, updating widget");
            println!("[EditorView::rebuild] Old tool: {:?}, New tool: {:?}",
                     prev.session.current_tool.id(), self.session.current_tool.id());

            // Get mutable access to the widget
            let mut widget = element.downcast::<EditorWidget>();

            // Update the session, but preserve:
            // - Mouse state (to avoid breaking active drag operations)
            // - Undo state
            // - Canvas size
            // This allows tool changes and other session updates to take effect
            widget.widget.session = (*self.session).clone();
            widget.ctx.request_render();
        }
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        _element: Mut<'_, Self::Element>,
    ) {
        // No cleanup needed
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        message: &mut MessageContext,
        _element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<()> {
        // Handle SessionUpdate messages from the widget
        match message.take_message::<SessionUpdate>() {
            Some(update) => {
                println!("[EditorView::message] Handling SessionUpdate, calling callback, selection.len()={}", update.session.selection.len());
                (self.on_session_update)(app_state, update.session);
                println!("[EditorView::message] Callback complete, returning RequestRebuild");
                // Use RequestRebuild instead of Action to avoid destroying the window
                // We're a single-window app with tabs, not multi-window
                MessageResult::RequestRebuild
            }
            None => MessageResult::Stale,
        }
    }
}
