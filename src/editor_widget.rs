// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Glyph editor widget - the main canvas for editing glyphs

use crate::edit_session::EditSession;
use crate::glyph_renderer;
use crate::mouse::{Mouse, MouseButton, MouseEvent as MouseEvt};
use crate::point::PointType;
use crate::theme;
use kurbo::{Affine, Circle, Point, Rect as KurboRect, Shape, Stroke};
use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, EventCtx, LayoutCtx, NoAction, PaintCtx,
    PointerButton, PointerButtonEvent, PointerEvent, PointerUpdate,
    PropertiesMut, PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget,
};
use masonry::kurbo::Size;
use masonry::util::fill_color;
use masonry::vello::peniko::{Brush, Color};
use masonry::vello::Scene;
use std::sync::Arc;

/// The main glyph editor canvas widget
pub struct EditorWidget {
    /// The editing session (mutable copy for editing)
    session: EditSession,

    /// Mouse state machine
    mouse: Mouse,

    /// Canvas size
    size: Size,
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
        }
    }

    /// Set the canvas size
    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl Widget for EditorWidget {
    type Action = NoAction;

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
        // Use the requested size, constrained by BoxConstraints
        bc.constrain(self.size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        let canvas_size = ctx.size();

        // Fill background
        let bg_rect = canvas_size.to_rect();
        fill_color(scene, &bg_rect, Color::from_rgb8(30, 30, 30));

        // Get the glyph outline from the editable paths
        let mut glyph_path = kurbo::BezPath::new();
        for path in self.session.paths.iter() {
            glyph_path.extend(path.to_bezpath());
        }

        if glyph_path.is_empty() {
            return;
        }

        // Calculate initial viewport positioning to center the glyph
        // We'll center based on the advance width and font metrics
        let ascender = self.session.ascender;
        let descender = self.session.descender;

        // Calculate the visible height in design space
        let design_height = ascender - descender;

        // Center the viewport on the canvas
        // We want the glyph to be centered horizontally and vertically
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
            center_y - (-design_center_y * scale), // Note the Y-flip
        );

        // Create transform: translate to origin, scale, flip Y, translate to canvas center
        let transform = Affine::translate((center_x, center_y))
            * Affine::scale_non_uniform(scale, -scale) // Negative Y scale for flip
            * Affine::translate((-design_center_x, -design_center_y));

        // Apply transform to path
        let transformed_path = transform * &glyph_path;

        // Draw the glyph outline
        let stroke = Stroke::new(theme::size::PATH_STROKE_WIDTH);
        let brush = Brush::Solid(theme::path::STROKE);
        scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &transformed_path);

        // Draw font metrics guides
        draw_metrics_guides(scene, &transform, &self.session, canvas_size);

        // Draw control point lines and points
        draw_paths_with_points(scene, &self.session, &transform);
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
                ctx.capture_pointer();
                let local_pos = ctx.local_position(state.position);
                println!("EditorWidget: PointerEvent::Down at {:?}", local_pos);

                // Create MouseEvent for our mouse state machine
                let mouse_event = MouseEvent::new(local_pos, Some(MouseButton::Left));

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

                if ctx.is_active() {
                    ctx.request_render();
                }
            }

            PointerEvent::Up(PointerButtonEvent { button: Some(PointerButton::Primary), state, .. }) => {
                let local_pos = ctx.local_position(state.position);

                // Create MouseEvent
                let mouse_event = MouseEvent::new(local_pos, Some(MouseButton::Left));

                // Temporarily take ownership of the tool
                let mut tool = std::mem::replace(&mut self.session.current_tool, ToolBox::for_id(ToolId::Select));
                self.mouse.mouse_up(mouse_event, &mut tool, &mut self.session);
                self.session.current_tool = tool;

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

    // Helper to draw a horizontal line at a given Y coordinate in design space
    let draw_hline = |scene: &mut Scene, y: f64| {
        let start = Point::new(-1000.0, y);
        let end = Point::new(session.glyph.width + 1000.0, y);

        let start_screen = *transform * start;
        let end_screen = *transform * end;

        let line = kurbo::Line::new(start_screen, end_screen);
        let brush = Brush::Solid(theme::metrics::GUIDE);
        scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line);
    };

    // Baseline (y=0)
    draw_hline(scene, 0.0);

    // Ascender
    draw_hline(scene, session.ascender);

    // Descender
    draw_hline(scene, session.descender);

    // X-height (if available)
    if let Some(x_height) = session.x_height {
        draw_hline(scene, x_height);
    }

    // Cap-height (if available)
    if let Some(cap_height) = session.cap_height {
        draw_hline(scene, cap_height);
    }
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
                                    (theme::point::SMOOTH_SELECTED_INNER, theme::point::SMOOTH_SELECTED_OUTER)
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
                                    (theme::point::CORNER_SELECTED_INNER, theme::point::CORNER_SELECTED_OUTER)
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
                                (theme::point::OFFCURVE_SELECTED_INNER, theme::point::OFFCURVE_SELECTED_OUTER)
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

/// Create an editor view from an edit session
pub fn editor_view<State, Action>(session: Arc<EditSession>) -> EditorView<State, Action> {
    EditorView {
        session,
        phantom: PhantomData,
    }
}

/// The Xilem View for EditorWidget
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct EditorView<State, Action = ()> {
    session: Arc<EditSession>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action> ViewMarker for EditorView<State, Action> {}

impl<State: 'static, Action: 'static> View<State, Action, ViewCtx> for EditorView<State, Action> {
    type Element = Pod<EditorWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let widget = EditorWidget::new(self.session.clone());
        (ctx.create_pod(widget), ())
    }

    fn rebuild(
        &self,
        _prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        _element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        // For now, no incremental updates
        // TODO: Check if session has changed and update widget
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
        _message: &mut MessageContext,
        _element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) -> MessageResult<Action> {
        // EditorWidget doesn't produce messages yet
        MessageResult::Stale
    }
}
