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

/// Size constants for the coordinate pane - using theme
const PANE_WIDTH: f64 = 240.0;
const PANE_HEIGHT: f64 = 80.0;  // Matches container height
const LABEL_WIDTH: f64 = 12.0;
const VALUE_WIDTH: f64 = 50.0;

// Import from theme (includes all sizing and color constants)
use crate::theme::coord_pane::*;

/// Coordinate pane widget
pub struct CoordPaneWidget {
    coord_sel: CoordinateSelection,
    /// Which quadrant dot is currently being hovered (if any)
    hover_quadrant: Option<Quadrant>,
    /// Current widget size (updated during layout)
    widget_size: Size,
}

impl CoordPaneWidget {
    pub fn new(coord_sel: CoordinateSelection) -> Self {
        Self {
            coord_sel,
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
    fn quadrant_at_point(&self, point: Point) -> Option<Quadrant> {
        let bounds = self.quadrant_picker_bounds();

        if !bounds.contains(point) {
            return None;
        }

        let dot_radius = self.dot_radius(bounds);

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
            let circle = Circle::new(center, dot_radius * 2.0); // Larger hit area
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
                if let Some(quadrant) = self.quadrant_at_point(local_pos) {
                    ctx.submit_action::<Option<Quadrant>>(Some(quadrant));
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
            let is_selected = *quadrant == self.coord_sel.quadrant;

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
