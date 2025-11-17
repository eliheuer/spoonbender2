// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Grid toolbar widget - navigation toolbar for editor view
//! Shows button to return to grid and will have workspace switching later

use kurbo::{Affine, BezPath, Point, Rect, Shape, Size};
use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, EventCtx, LayoutCtx, PaintCtx,
    PointerButton, PointerButtonEvent, PointerEvent,
    PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx, Widget,
};
use masonry::util::{fill_color, stroke};
use masonry::vello::peniko::Color;
use masonry::vello::Scene;

/// Toolbar dimensions (matching edit toolbar style)
const TOOLBAR_ITEM_SIZE: f64 = 48.0;
const TOOLBAR_ITEM_SPACING: f64 = 6.0;  // Space between buttons
const TOOLBAR_PADDING: f64 = 8.0;  // Padding around the entire toolbar
const ICON_PADDING: f64 = 8.0;
const ITEM_STROKE_WIDTH: f64 = 1.5;
const BUTTON_RADIUS: f64 = 6.0;  // Rounded corner radius
const BORDER_WIDTH: f64 = 1.5;  // Border thickness for buttons and panel

/// Toolbar colors (from theme)
const COLOR_PANEL: Color = crate::theme::panel::BACKGROUND;
const COLOR_BUTTON: Color = crate::theme::toolbar::BUTTON_UNSELECTED;
const COLOR_ICON: Color = crate::theme::toolbar::ICON;
const COLOR_PANEL_BORDER: Color = crate::theme::panel::OUTLINE;
const COLOR_BUTTON_BORDER: Color = crate::theme::panel::BUTTON_OUTLINE;

/// Grid toolbar button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridToolbarButton {
    /// Return to grid view
    Grid,
}

/// Grid toolbar widget
pub struct GridToolbarWidget {
    /// Currently hovered button
    hover_button: Option<GridToolbarButton>,
}

impl GridToolbarWidget {
    pub fn new() -> Self {
        Self { hover_button: None }
    }

    /// Get the icon path for a button
    fn icon_for_button(button: GridToolbarButton) -> BezPath {
        match button {
            GridToolbarButton::Grid => grid_icon(),
        }
    }

    /// Get the rect for a button by index
    fn button_rect(&self, index: usize) -> Rect {
        let x = TOOLBAR_PADDING + index as f64 * (TOOLBAR_ITEM_SIZE + TOOLBAR_ITEM_SPACING);
        let y = TOOLBAR_PADDING;
        Rect::new(x, y, x + TOOLBAR_ITEM_SIZE, y + TOOLBAR_ITEM_SIZE)
    }

    /// Find which button was clicked
    fn button_at_point(&self, point: Point) -> Option<GridToolbarButton> {
        // Currently only one button
        if self.button_rect(0).contains(point) {
            return Some(GridToolbarButton::Grid);
        }
        None
    }

    /// Calculate toolbar size based on number of buttons
    fn toolbar_size(&self) -> Size {
        let button_count = 1; // Currently only grid button
        let width = TOOLBAR_PADDING * 2.0
            + button_count as f64 * TOOLBAR_ITEM_SIZE
            + (button_count - 1) as f64 * TOOLBAR_ITEM_SPACING;
        let height = TOOLBAR_PADDING * 2.0 + TOOLBAR_ITEM_SIZE;
        Size::new(width, height)
    }
}

/// Action sent when a grid toolbar button is clicked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridToolbarAction(pub GridToolbarButton);

impl Widget for GridToolbarWidget {
    type Action = GridToolbarAction;

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {
        // Leaf widget - no children
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
        // No updates needed
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        bc.constrain(self.toolbar_size())
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        // Draw a solid background panel behind all buttons to prevent transparency issues
        let size = self.toolbar_size();
        let panel_rect = Rect::new(0.0, 0.0, size.width, size.height);
        let panel_rrect = kurbo::RoundedRect::from_rect(panel_rect, 8.0);

        // Solid opaque background - darker than buttons but brighter than canvas
        fill_color(scene, &panel_rrect, COLOR_PANEL);

        // Draw panel border - inset slightly to prevent corner artifacts
        let border_inset = BORDER_WIDTH / 2.0;
        let inset_rect = panel_rect.inset(-border_inset);
        let inset_rrect = kurbo::RoundedRect::from_rect(inset_rect, 8.0);
        stroke(scene, &inset_rrect, COLOR_PANEL_BORDER, BORDER_WIDTH);

        // Draw grid button
        let button_rect = self.button_rect(0);
        let is_hovered = self.hover_button == Some(GridToolbarButton::Grid);

        // Button background (slightly lighter on hover)
        let button_color = if is_hovered {
            crate::theme::toolbar::BUTTON_SELECTED
        } else {
            COLOR_BUTTON
        };

        let button_path = kurbo::RoundedRect::from_rect(button_rect, BUTTON_RADIUS);
        fill_color(scene, &button_path, button_color);

        // Button border
        stroke(
            scene,
            &button_path,
            COLOR_BUTTON_BORDER,
            BORDER_WIDTH,
        );

        // Draw icon
        let icon = GridToolbarWidget::icon_for_button(GridToolbarButton::Grid);
        let icon_bounds = icon.bounding_box();
        let icon_center = icon_bounds.center();
        let button_center = button_rect.center();

        // Scale icon to fit with padding
        let icon_size = icon_bounds.width().max(icon_bounds.height());
        let target_size = TOOLBAR_ITEM_SIZE - ICON_PADDING * 2.0;
        let scale = target_size / icon_size;

        // Create transform: scale then translate to center
        let transform = Affine::translate((button_center.x, button_center.y))
            * Affine::scale(scale)
            * Affine::translate((-icon_center.x, -icon_center.y));

        // Fill icon with dark gray color (no stroke)
        fill_color(scene, &(transform * icon), COLOR_ICON);
    }

    fn accessibility_role(&self) -> Role {
        Role::Toolbar
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
        // TODO: Add accessibility info
    }

    fn children_ids(&self) -> masonry::core::ChildrenIds {
        masonry::core::ChildrenIds::new()
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
                if let Some(button) = self.button_at_point(local_pos) {
                    ctx.submit_action::<GridToolbarAction>(GridToolbarAction(button));
                    ctx.request_render();
                }
                // Always consume the event to prevent it from reaching the editor
                ctx.set_handled();
            }
            PointerEvent::Move(pointer_move) => {
                let local_pos = ctx.local_position(pointer_move.current.position);
                let new_hover = self.button_at_point(local_pos);
                if new_hover != self.hover_button {
                    self.hover_button = new_hover;
                    ctx.request_render();
                }
            }
            PointerEvent::Leave(_) => {
                if self.hover_button.is_some() {
                    self.hover_button = None;
                    ctx.request_render();
                }
            }
            _ => {}
        }
    }

    fn on_text_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &TextEvent,
    ) {
        // No text handling needed
    }
}

/// Grid icon - 3x3 grid of squares
fn grid_icon() -> BezPath {
    let mut path = BezPath::new();

    // Draw a 3x3 grid of small squares
    let grid_size = 32.0;
    let cell_size = 8.0;
    let gap = 4.0;
    let offset = -(grid_size / 2.0);

    for row in 0..3 {
        for col in 0..3 {
            let x = offset + col as f64 * (cell_size + gap);
            let y = offset + row as f64 * (cell_size + gap);
            let rect = Rect::new(x, y, x + cell_size, y + cell_size);
            let rounded_rect = kurbo::RoundedRect::from_rect(rect, 1.0);
            path.extend(rounded_rect.path_elements(0.1));
        }
    }

    path
}

// ============================================================================
// XILEM VIEW WRAPPER
// ============================================================================

use std::marker::PhantomData;
use xilem::core::{MessageContext, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

/// Xilem view for the grid toolbar
pub struct GridToolbarView<State, Action = ()> {
    callback: Box<dyn Fn(&mut State, GridToolbarButton) + Send + Sync>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action> ViewMarker for GridToolbarView<State, Action> {}

impl<State: 'static, Action: 'static + Default> View<State, Action, ViewCtx> for GridToolbarView<State, Action> {
    type Element = Pod<GridToolbarWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let widget = GridToolbarWidget::new();
        (ctx.with_action_widget(|ctx| ctx.create_pod(widget)), ())
    }

    fn rebuild(
        &self,
        _prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        _element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        // No state to rebuild
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        _element: Mut<'_, Self::Element>,
    ) {
        // No teardown needed
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        message: &mut MessageContext,
        _element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_message::<GridToolbarAction>() {
            Some(action) => {
                (self.callback)(app_state, action.0);
                MessageResult::Action(Action::default())
            }
            None => MessageResult::Stale,
        }
    }
}

/// Helper function to create a grid toolbar view
pub fn grid_toolbar_view<State, Action>(
    callback: impl Fn(&mut State, GridToolbarButton) + Send + Sync + 'static,
) -> GridToolbarView<State, Action>
where
    Action: 'static,
{
    GridToolbarView {
        callback: Box::new(callback),
        phantom: PhantomData,
    }
}
