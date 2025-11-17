// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Coordinate pane widget - displays and allows editing of point coordinates
//!
//! This widget shows the x, y, width, and height of the current selection,
//! and includes a quadrant picker to choose which corner/edge to use as
//! the reference point for multi-point selections.

use crate::quadrant::Quadrant;
use crate::theme;
use kurbo::{Circle, Point, Rect, Shape};
use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, EventCtx, LayoutCtx, PaintCtx,
    PointerButton, PointerButtonEvent, PointerEvent,
    PropertiesMut, PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget,
};
use masonry::kurbo::Size;
use masonry::vello::peniko::Color;
use masonry::vello::Scene;

/// Size constants for the coordinate pane - using theme
const PANE_WIDTH: f64 = 240.0;
const PANE_HEIGHT: f64 = 80.0;  // Matches container height
const LABEL_WIDTH: f64 = 12.0;
const VALUE_WIDTH: f64 = 50.0;

// Import from theme (includes all sizing and color constants)
use crate::theme::coord_pane::*;

// --- MARK: DATA MODEL ---

/// Coordinate selection information for displaying/editing point coordinates
///
/// This stores the bounding box of the current selection and which quadrant
/// to use as the reference point for coordinate display.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoordinateSelection {
    /// Number of points selected
    pub count: usize,
    /// Bounding box of the selection (in design space)
    pub frame: Rect,
    /// Which quadrant/anchor point to use for coordinate display
    pub quadrant: Quadrant,
}

impl CoordinateSelection {
    /// Create a new coordinate selection
    pub fn new(count: usize, frame: Rect, quadrant: Quadrant) -> Self {
        Self {
            count,
            frame,
            quadrant,
        }
    }

    /// Get the reference point based on the selected quadrant
    pub fn reference_point(&self) -> Point {
        self.quadrant.point_in_dspace_rect(self.frame)
    }

    /// Get the width of the selection
    pub fn width(&self) -> f64 {
        self.frame.width()
    }

    /// Get the height of the selection
    pub fn height(&self) -> f64 {
        self.frame.height()
    }
}

impl Default for CoordinateSelection {
    fn default() -> Self {
        Self {
            count: 0,
            frame: Rect::ZERO,
            quadrant: Quadrant::default(),
        }
    }
}

// --- MARK: WIDGET ---

/// Coordinate pane widget
pub struct CoordPaneWidget {
    session: crate::edit_session::EditSession,
    /// Which quadrant dot is currently being hovered (if any)
    hover_quadrant: Option<Quadrant>,
    /// Current widget size (updated during layout)
    widget_size: Size,
}

impl CoordPaneWidget {
    pub fn new(session: crate::edit_session::EditSession) -> Self {
        Self {
            session,
            hover_quadrant: None,
            widget_size: Size::ZERO,
        }
    }

    /// Get the bounds of the quadrant picker within the widget
    ///
    /// This calculates the selector size dynamically based on available space
    /// to ensure it fits with proper margins on all sides.
    fn quadrant_picker_bounds(&self) -> Rect {
        if self.widget_size.width == 0.0 || self.widget_size.height == 0.0 {
            // Widget hasn't been laid out yet, use default size
            return Rect::new(
                PADDING,
                PADDING,
                PADDING + SELECTOR_SIZE,
                PADDING + SELECTOR_SIZE,
            );
        }

        // Calculate available space after accounting for padding
        let available_width = self.widget_size.width - (PADDING * 2.0);
        let available_height = self.widget_size.height - (PADDING * 2.0);

        // Selector should be square, so use the smaller dimension
        let selector_size = available_width.min(available_height);

        // Center the selector vertically if there's extra vertical space
        let top = PADDING + ((available_height - selector_size) / 2.0).max(0.0);

        Rect::new(
            PADDING,
            top,
            PADDING + selector_size,
            top + selector_size,
        )
    }

    /// Get the center point for a specific quadrant dot within the picker bounds
    fn quadrant_dot_center(&self, quadrant: Quadrant, bounds: Rect) -> Point {
        let center_x = bounds.center().x;
        let center_y = bounds.center().y;

        match quadrant {
            Quadrant::TopLeft => Point::new(bounds.min_x(), bounds.min_y()),
            Quadrant::Top => Point::new(center_x, bounds.min_y()),
            Quadrant::TopRight => Point::new(bounds.max_x(), bounds.min_y()),
            Quadrant::Left => Point::new(bounds.min_x(), center_y),
            Quadrant::Center => Point::new(center_x, center_y),
            Quadrant::Right => Point::new(bounds.max_x(), center_y),
            Quadrant::BottomLeft => Point::new(bounds.min_x(), bounds.max_y()),
            Quadrant::Bottom => Point::new(center_x, bounds.max_y()),
            Quadrant::BottomRight => Point::new(bounds.max_x(), bounds.max_y()),
        }
    }

    /// Calculate the dot radius based on the selector size
    ///
    /// This scales the dot radius proportionally to the selector size
    /// to maintain consistent appearance at different sizes.
    fn dot_radius(&self, bounds: Rect) -> f64 {
        let selector_size = bounds.width();
        // Scale dot radius based on selector size
        // Default is 8.0 for 64.0 selector (8/64 = 0.125 ratio)
        selector_size * (DOT_RADIUS / SELECTOR_SIZE)
    }

    /// Determine which quadrant (if any) a point is hovering over
    ///
    /// Uses grid-based hit detection (matching Runebender's approach):
    /// Divides the widget into a 3x3 grid and returns which zone was clicked.
    /// This eliminates overlapping hit areas and ensures every part of the widget is clickable.
    fn quadrant_at_point(&self, point: Point) -> Option<Quadrant> {
        // Use the FULL widget bounds for hit detection, not just the visual picker bounds
        // The padding is just for visual spacing, not for limiting clickability
        let hit_bounds = Rect::from_origin_size(
            kurbo::Point::ZERO,
            self.widget_size,
        );

        if !hit_bounds.contains(point) {
            return None;
        }

        // Use grid-based hit detection instead of circle-based
        // This matches Runebender's approach and eliminates overlapping hit areas
        Some(Quadrant::for_point_in_bounds(point, hit_bounds))
    }
}

/// Action emitted by the coord pane widget when the quadrant is changed
#[derive(Debug, Clone)]
pub struct SessionUpdate {
    pub session: crate::edit_session::EditSession,
}

impl Widget for CoordPaneWidget {
    type Action = SessionUpdate;

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {
        // Leaf widget - no children
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
        // State updates handled externally
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        // Store the widget size so we can use it in paint
        self.widget_size = bc.constrain(Size::new(PANE_WIDTH, PANE_HEIGHT));
        self.widget_size
    }

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(PointerButtonEvent { button: Some(PointerButton::Primary), state, .. }) => {
                let local_pos = ctx.local_position(state.position);
                println!("[CoordPaneWidget::on_pointer_event] Pointer down at local_pos: {:?}", local_pos);
                if let Some(quadrant) = self.quadrant_at_point(local_pos) {
                    println!("[CoordPaneWidget::on_pointer_event] Clicked on quadrant: {:?}", quadrant);
                    println!("[CoordPaneWidget::on_pointer_event] Old quadrant: {:?}", self.session.coord_selection.quadrant);

                    // Update the session's quadrant selection
                    self.session.coord_selection.quadrant = quadrant;
                    println!("[CoordPaneWidget::on_pointer_event] New quadrant: {:?}", self.session.coord_selection.quadrant);

                    // Emit SessionUpdate action
                    ctx.submit_action::<SessionUpdate>(SessionUpdate {
                        session: self.session.clone(),
                    });

                    // Request a repaint to show the new selected quadrant
                    ctx.request_render();
                } else {
                    println!("[CoordPaneWidget::on_pointer_event] Click was not on any quadrant dot");
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        // Background and border are now handled by the sized_box wrapper in lib.rs
        // This widget only paints the quadrant picker
        // Coordinate text values are handled by Xilem views in lib.rs

        // Always show quadrant picker (user can select quadrant even without points selected)
        self.paint_quadrant_picker(scene);
    }

    fn accessibility_role(&self) -> Role {
        Role::Group
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
        // Could add accessibility info for coordinate display
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }
}

impl CoordPaneWidget {
    /// Paint the quadrant picker (3x3 grid of dots)
    fn paint_quadrant_picker(&self, scene: &mut Scene) {
        let bounds = self.quadrant_picker_bounds();
        let dot_radius = self.dot_radius(bounds);

        // Draw frame around picker using theme stroke width
        masonry::util::stroke(scene, &bounds, GRID_LINE, STROKE_WIDTH);

        // Draw grid lines (horizontal and vertical lines forming 3x3 grid)
        let center_x = bounds.center().x;
        let center_y = bounds.center().y;

        // Horizontal lines
        let h_line_top = kurbo::Line::new(
            kurbo::Point::new(bounds.min_x(), bounds.min_y()),
            kurbo::Point::new(bounds.max_x(), bounds.min_y()),
        );
        let h_line_middle = kurbo::Line::new(
            kurbo::Point::new(bounds.min_x(), center_y),
            kurbo::Point::new(bounds.max_x(), center_y),
        );
        let h_line_bottom = kurbo::Line::new(
            kurbo::Point::new(bounds.min_x(), bounds.max_y()),
            kurbo::Point::new(bounds.max_x(), bounds.max_y()),
        );

        // Vertical lines
        let v_line_left = kurbo::Line::new(
            kurbo::Point::new(bounds.min_x(), bounds.min_y()),
            kurbo::Point::new(bounds.min_x(), bounds.max_y()),
        );
        let v_line_middle = kurbo::Line::new(
            kurbo::Point::new(center_x, bounds.min_y()),
            kurbo::Point::new(center_x, bounds.max_y()),
        );
        let v_line_right = kurbo::Line::new(
            kurbo::Point::new(bounds.max_x(), bounds.min_y()),
            kurbo::Point::new(bounds.max_x(), bounds.max_y()),
        );

        // Draw all grid lines using theme stroke width
        masonry::util::stroke(scene, &h_line_top, GRID_LINE, STROKE_WIDTH);
        masonry::util::stroke(scene, &h_line_middle, GRID_LINE, STROKE_WIDTH);
        masonry::util::stroke(scene, &h_line_bottom, GRID_LINE, STROKE_WIDTH);
        masonry::util::stroke(scene, &v_line_left, GRID_LINE, STROKE_WIDTH);
        masonry::util::stroke(scene, &v_line_middle, GRID_LINE, STROKE_WIDTH);
        masonry::util::stroke(scene, &v_line_right, GRID_LINE, STROKE_WIDTH);

        // Draw all 9 quadrant dots with two-tone style like editor points
        for quadrant in &[
            Quadrant::TopLeft,
            Quadrant::Top,
            Quadrant::TopRight,
            Quadrant::Left,
            Quadrant::Center,
            Quadrant::Right,
            Quadrant::BottomLeft,
            Quadrant::Bottom,
            Quadrant::BottomRight,
        ] {
            let center = self.quadrant_dot_center(*quadrant, bounds);
            let is_selected = *quadrant == self.session.coord_selection.quadrant;

            let (inner_color, outer_color) = if is_selected {
                (DOT_SELECTED_INNER, DOT_SELECTED_OUTER)
            } else {
                (DOT_UNSELECTED_INNER, DOT_UNSELECTED_OUTER)
            };

            // Draw two-tone filled circles to simulate outlined circles
            // Outer circle - use calculated dot radius
            let outer_circle = Circle::new(center, dot_radius);
            masonry::util::fill_color(scene, &outer_circle, outer_color);

            // Inner circle - make the "outline" match the container border width (1.5px)
            // by subtracting 1.5 from the radius
            let inner_circle = Circle::new(center, (dot_radius - 1.5).max(0.0));
            masonry::util::fill_color(scene, &inner_circle, inner_color);
        }
    }
}

// --- MARK: XILEM VIEW WRAPPER ---

use std::marker::PhantomData;
use std::sync::Arc;
use xilem::core::{MessageContext, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

/// Create a coordinate pane view from an EditSession
pub fn coord_pane_view<State, F>(
    session: Arc<crate::edit_session::EditSession>,
    on_session_update: F,
) -> CoordPaneView<State, F>
where
    F: Fn(&mut State, crate::edit_session::EditSession) + Send + Sync + 'static,
{
    CoordPaneView {
        session,
        on_session_update,
        phantom: PhantomData,
    }
}

/// The Xilem View for CoordPaneWidget
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct CoordPaneView<State, F> {
    session: Arc<crate::edit_session::EditSession>,
    on_session_update: F,
    phantom: PhantomData<fn() -> State>,
}

impl<State, F> ViewMarker for CoordPaneView<State, F> {}

impl<State: 'static, F: Fn(&mut State, crate::edit_session::EditSession) + Send + Sync + 'static> View<State, (), ViewCtx> for CoordPaneView<State, F> {
    type Element = Pod<CoordPaneWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let widget = CoordPaneWidget::new((*self.session).clone());
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
        // Update the widget's session if it changed
        // We compare Arc pointers - if they're different, the session was updated
        if !Arc::ptr_eq(&self.session, &prev.session) {
            println!("[CoordPaneView::rebuild] Session Arc changed, updating widget");
            println!("[CoordPaneView::rebuild] Old quadrant: {:?}, New quadrant: {:?}",
                     prev.session.coord_selection.quadrant, self.session.coord_selection.quadrant);

            // Get mutable access to the widget and update the session
            let mut widget = element.downcast::<CoordPaneWidget>();
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
                println!("[CoordPaneView::message] Handling SessionUpdate, quadrant={:?}", update.session.coord_selection.quadrant);
                (self.on_session_update)(app_state, update.session);
                println!("[CoordPaneView::message] Callback complete, returning RequestRebuild");
                // Use RequestRebuild instead of Action to avoid destroying the window
                MessageResult::RequestRebuild
            }
            None => MessageResult::Stale,
        }
    }
}

// --- MARK: COORDINATE CALCULATION ---

/// Calculate coordinate selection from edit session
///
/// Returns a CoordinateSelection with bounding box information for all selected points
pub fn calculate_coordinate_selection(session: &crate::edit_session::EditSession) -> CoordinateSelection {
    let selection = &session.selection;
    let paths = &session.paths;

    println!("[calculate_coordinate_selection] selection.len()={}, paths.len()={}", selection.len(), paths.len());

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut count = 0;

    for path in paths.iter() {
        match path {
            crate::path::Path::Cubic(cubic) => {
                println!("[calculate_coordinate_selection] Checking cubic path with {} points", cubic.points.len());
                for pt in cubic.points.iter() {
                    if selection.contains(&pt.id) {
                        println!("[calculate_coordinate_selection] Found selected point at ({}, {})", pt.point.x, pt.point.y);
                        min_x = min_x.min(pt.point.x);
                        max_x = max_x.max(pt.point.x);
                        min_y = min_y.min(pt.point.y);
                        max_y = max_y.max(pt.point.y);
                        count += 1;
                    }
                }
            }
        }
    }

    println!("[calculate_coordinate_selection] count={}, min_x={}, max_x={}, min_y={}, max_y={}", count, min_x, max_x, min_y, max_y);

    if count > 0 && min_x.is_finite() {
        let frame = Rect::new(min_x, min_y, max_x, max_y);
        CoordinateSelection::new(
            count,
            frame,
            session.coord_selection.quadrant, // Preserve the user's quadrant selection
        )
    } else {
        CoordinateSelection::default()
    }
}

// --- MARK: COMPLETE COORDINATE PANE VIEW ---

use masonry::properties::types::MainAxisAlignment;
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label, sized_box, CrossAxisAlignment};
use xilem::{WidgetView};
use masonry::properties::types::AsUnit;

/// Complete coordinate info pane with quadrant picker and coordinate labels
///
/// This is the main entry point for displaying the coordinate pane in the editor window.
/// It combines the quadrant picker widget with coordinate text labels.
pub fn coordinate_info_pane<State: 'static, F>(
    session: Arc<crate::edit_session::EditSession>,
    on_session_update: F,
) -> impl WidgetView<State>
where
    F: Fn(&mut State, crate::edit_session::EditSession) + Send + Sync + 'static,
{
    let coord_sel = session.coord_selection;

    // Calculate coordinate values based on the selection
    let (x_text, y_text, w_text, h_text) = if coord_sel.count == 0 {
        ("—".to_string(), "—".to_string(), "—".to_string(), "—".to_string())
    } else {
        let pt = coord_sel.reference_point();
        let x = format!("{:.0}", pt.x);
        let y = format!("{:.0}", pt.y);

        // Width and height only shown when multiple points are selected
        let w = if coord_sel.count > 1 {
            format!("{:.0}", coord_sel.width())
        } else {
            "—".to_string()
        };
        let h = if coord_sel.count > 1 {
            format!("{:.0}", coord_sel.height())
        } else {
            "—".to_string()
        };

        (x, y, w, h)
    };

    // Helper function to create styled coordinate labels
    let coord_label = |text: String| {
        label(text)
            .text_size(12.0)
            .text_alignment(parley::Alignment::Start)
            .color(theme::text::PRIMARY)
    };

    sized_box(
        flex_row((
            // Quadrant selector on the left
            sized_box(coord_pane_view(session, on_session_update)).width(80.px()),
            // Coordinate values with fixed-width formatting
            flex_col((
                coord_label(format!("x: {:<6}", x_text)),
                coord_label(format!("y: {:<6}", y_text)),
                coord_label(format!("w: {:<6}", w_text)),
                coord_label(format!("h: {:<6}", h_text)),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .gap(0.px()),
        ))
        .main_axis_alignment(MainAxisAlignment::Start)
        .gap(8.px())
    )
    .width(150.px())
    .height(80.px())
    .background_color(crate::theme::panel::BACKGROUND)
    .border_color(crate::theme::panel::OUTLINE)
    .border_width(1.5)
    .corner_radius(8.0)
}
