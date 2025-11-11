// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Coordinate pane widget - displays and allows editing of point coordinates
//!
//! This widget shows the x, y, width, and height of the current selection,
//! and includes a quadrant picker to choose which corner/edge to use as
//! the reference point for multi-point selections.

use crate::edit_session::CoordinateSelection;
use crate::quadrant::Quadrant;
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

/// Size constants for the coordinate pane
const PANE_WIDTH: f64 = 240.0;
const PANE_HEIGHT: f64 = 130.0;
const PADDING: f64 = 8.0;
const QUADRANT_PICKER_SIZE: f64 = 50.0;
const DOT_RADIUS: f64 = 6.0;  // Bigger dots like Runebender
const DOT_STROKE_WIDTH: f64 = 1.5;
const ROW_HEIGHT: f64 = 24.0;
const LABEL_WIDTH: f64 = 12.0;
const VALUE_WIDTH: f64 = 50.0;

/// Colors - matching the point colors from the editor
const TEXT_COLOR: Color = Color::from_rgb8(200, 200, 200);
// Selected quadrant dot uses the same yellow/orange as selected points
const DOT_SELECTED_INNER: Color = crate::theme::point::SELECTED_INNER;
const DOT_SELECTED_OUTER: Color = crate::theme::point::SELECTED_OUTER;
// Unselected quadrant dots are gray
const DOT_UNSELECTED_INNER: Color = Color::from_rgb8(100, 100, 100);
const DOT_UNSELECTED_OUTER: Color = Color::from_rgb8(70, 70, 70);
const QUADRANT_FRAME_COLOR: Color = Color::from_rgb8(100, 100, 100);

/// Coordinate pane widget
pub struct CoordPaneWidget {
    coord_sel: CoordinateSelection,
    /// Which quadrant dot is currently being hovered (if any)
    hover_quadrant: Option<Quadrant>,
}

impl CoordPaneWidget {
    pub fn new(coord_sel: CoordinateSelection) -> Self {
        Self {
            coord_sel,
            hover_quadrant: None,
        }
    }

    /// Get the bounds of the quadrant picker within the widget
    fn quadrant_picker_bounds(&self) -> Rect {
        Rect::new(
            PADDING,
            PADDING,
            PADDING + QUADRANT_PICKER_SIZE,
            PADDING + QUADRANT_PICKER_SIZE,
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

    /// Determine which quadrant (if any) a point is hovering over
    fn quadrant_at_point(&self, point: Point) -> Option<Quadrant> {
        let bounds = self.quadrant_picker_bounds();

        if !bounds.contains(point) {
            return None;
        }

        // Check all 9 quadrant dots
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
            let circle = Circle::new(center, DOT_RADIUS * 2.0); // Larger hit area
            if circle.contains(point) {
                return Some(*quadrant);
            }
        }

        None
    }
}

impl Widget for CoordPaneWidget {
    type Action = Option<Quadrant>;

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
        bc.constrain(Size::new(PANE_WIDTH, PANE_HEIGHT))
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
                if let Some(quadrant) = self.quadrant_at_point(local_pos) {
                    ctx.submit_action::<Option<Quadrant>>(Some(quadrant));
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        // Background and border are now handled by the sized_box wrapper in lib.rs
        // This widget only paints its content (quadrant picker and coordinate values)

        // Debug: always show the quadrant picker for testing
        println!("CoordPane paint: count={}, frame={:?}", self.coord_sel.count, self.coord_sel.frame);

        // Always show quadrant picker (user can select quadrant even without points selected)
        self.paint_quadrant_picker(scene);

        // Paint coordinate values
        self.paint_coordinates(scene);
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

        // Draw frame around picker
        masonry::util::stroke(scene, &bounds, QUADRANT_FRAME_COLOR, 1.0);

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

        // Draw all grid lines
        masonry::util::stroke(scene, &h_line_top, QUADRANT_FRAME_COLOR, 1.0);
        masonry::util::stroke(scene, &h_line_middle, QUADRANT_FRAME_COLOR, 1.0);
        masonry::util::stroke(scene, &h_line_bottom, QUADRANT_FRAME_COLOR, 1.0);
        masonry::util::stroke(scene, &v_line_left, QUADRANT_FRAME_COLOR, 1.0);
        masonry::util::stroke(scene, &v_line_middle, QUADRANT_FRAME_COLOR, 1.0);
        masonry::util::stroke(scene, &v_line_right, QUADRANT_FRAME_COLOR, 1.0);

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
            let is_selected = *quadrant == self.coord_sel.quadrant;

            let (inner_color, outer_color) = if is_selected {
                (DOT_SELECTED_INNER, DOT_SELECTED_OUTER)
            } else {
                (DOT_UNSELECTED_INNER, DOT_UNSELECTED_OUTER)
            };

            // Draw outer circle
            let outer_circle = Circle::new(center, DOT_RADIUS);
            masonry::util::fill_color(scene, &outer_circle, outer_color);

            // Draw inner circle (slightly smaller)
            let inner_circle = Circle::new(center, DOT_RADIUS * 0.6);
            masonry::util::fill_color(scene, &inner_circle, inner_color);

            // Draw outline
            masonry::util::stroke(scene, &outer_circle, outer_color, DOT_STROKE_WIDTH);
        }
    }

    /// Paint the coordinate values (x, y, w, h)
    fn paint_coordinates(&self, scene: &mut Scene) {
        // Calculate positions for labels and values
        // Quadrant picker is always shown, so coordinates always go to the right of it
        let label_x = PADDING + QUADRANT_PICKER_SIZE + PADDING;
        let value_x = label_x + LABEL_WIDTH + 4.0;

        // Get coordinate values
        let (x_text, y_text, w_text, h_text) = if self.coord_sel.count == 0 {
            ("—".to_string(), "—".to_string(), "—".to_string(), "—".to_string())
        } else {
            let pt = self.coord_sel.reference_point();
            let x = format!("{:.0}", pt.x);
            let y = format!("{:.0}", pt.y);
            let w = if self.coord_sel.count > 1 {
                format!("{:.0}", self.coord_sel.width())
            } else {
                "—".to_string()
            };
            let h = if self.coord_sel.count > 1 {
                format!("{:.0}", self.coord_sel.height())
            } else {
                "—".to_string()
            };
            (x, y, w, h)
        };

        // For now, we'll just draw placeholder boxes where text would go
        // Full text rendering would require using the text API
        let y_offset = PADDING + 4.0;

        // Draw labels: x, y, w, h
        self.draw_text_placeholder(scene, label_x, y_offset, "x");
        self.draw_text_placeholder(scene, value_x, y_offset, &x_text);

        self.draw_text_placeholder(scene, label_x, y_offset + ROW_HEIGHT, "y");
        self.draw_text_placeholder(scene, value_x, y_offset + ROW_HEIGHT, &y_text);

        self.draw_text_placeholder(scene, label_x, y_offset + ROW_HEIGHT * 2.0, "w");
        self.draw_text_placeholder(scene, value_x, y_offset + ROW_HEIGHT * 2.0, &w_text);

        self.draw_text_placeholder(scene, label_x, y_offset + ROW_HEIGHT * 3.0, "h");
        self.draw_text_placeholder(scene, value_x, y_offset + ROW_HEIGHT * 3.0, &h_text);
    }

    /// Draw text using simple rectangle representation for now
    ///
    /// TODO: Use proper text rendering with system fonts
    fn draw_text_placeholder(&self, scene: &mut Scene, x: f64, y: f64, text: &str) {
        // Draw a filled rectangle representing the text
        let width = (text.len() as f64) * 8.0; // Rough character width
        let rect = Rect::new(x, y, x + width, y + 14.0);

        // Fill with a lighter color to make it visible
        masonry::util::fill_color(scene, &rect, Color::from_rgb8(150, 150, 200));
        masonry::util::stroke(scene, &rect, TEXT_COLOR, 0.5);
    }
}

// --- MARK: XILEM VIEW WRAPPER ---

use std::marker::PhantomData;
use xilem::core::{MessageContext, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

/// Create a coordinate pane view from a CoordinateSelection
pub fn coord_pane_view<State>(
    coord_sel: CoordinateSelection,
) -> CoordPaneView<State> {
    CoordPaneView {
        coord_sel,
        phantom: PhantomData,
    }
}

/// The Xilem View for CoordPaneWidget
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct CoordPaneView<State> {
    coord_sel: CoordinateSelection,
    phantom: PhantomData<fn() -> State>,
}

impl<State> ViewMarker for CoordPaneView<State> {}

impl<State: 'static> View<State, (), ViewCtx> for CoordPaneView<State>
{
    type Element = Pod<CoordPaneWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let widget = CoordPaneWidget::new(self.coord_sel);
        (ctx.create_pod(widget), ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        _element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        // Log rebuilds for debugging
        if self.coord_sel != prev.coord_sel {
            println!("CoordPane rebuild: coord_sel changed from count={} to count={}", prev.coord_sel.count, self.coord_sel.count);
        }
        // Xilem will handle widget updates automatically
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
    ) -> MessageResult<()> {
        // For now, don't handle quadrant changes
        // TODO: Implement quadrant change handling
        MessageResult::Stale
    }
}
