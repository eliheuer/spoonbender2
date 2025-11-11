// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Toolbar widget with icon-based tool buttons

use crate::tools::ToolId;
use kurbo::{Affine, BezPath, Point, Rect, Shape, Size};
use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, EventCtx, LayoutCtx, PaintCtx,
    PointerButton, PointerButtonEvent, PointerEvent,
    PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx, Widget,
};
use masonry::util::{fill_color, stroke};
use masonry::vello::peniko::Color;
use masonry::vello::Scene;

/// Toolbar dimensions
const TOOLBAR_ITEM_SIZE: f64 = 48.0;
const TOOLBAR_ITEM_SPACING: f64 = 6.0;  // Space between buttons
const TOOLBAR_PADDING: f64 = 8.0;  // Padding around the entire toolbar (space between buttons and container)
const ICON_PADDING: f64 = 8.0;
const ITEM_STROKE_WIDTH: f64 = 1.5;
const BUTTON_RADIUS: f64 = 6.0;  // Rounded corner radius
const BORDER_WIDTH: f64 = 1.5;  // Border thickness for buttons and panel

/// Toolbar colors
const COLOR_PANEL: Color = crate::theme::panel::BACKGROUND;           // Panel background (shared)
const COLOR_UNSELECTED: Color = Color::from_rgb8(0xA8, 0xA8, 0xA8);  // Light gray buttons
const COLOR_SELECTED: Color = Color::from_rgb8(0xD8, 0xD8, 0xD8);     // Very light gray when selected (brightest)
const COLOR_ICON: Color = Color::from_rgb8(0x40, 0x40, 0x40);  // Medium-dark gray icons
const COLOR_PANEL_BORDER: Color = crate::theme::panel::OUTLINE;       // Panel container border
const COLOR_BUTTON_BORDER: Color = crate::theme::panel::BUTTON_OUTLINE; // Toolbar button borders

/// Available tools in display order
const TOOLBAR_TOOLS: &[ToolId] = &[
    ToolId::Select,
    ToolId::Pen,
    ToolId::Preview,
    ToolId::Knife,
    ToolId::Rectangle,
    ToolId::Ellipse,
    ToolId::Measure,
];

/// Toolbar widget
pub struct ToolbarWidget {
    /// Currently selected tool
    selected_tool: ToolId,
}

impl ToolbarWidget {
    pub fn new(selected_tool: ToolId) -> Self {
        Self { selected_tool }
    }

    /// Get the icon path for a tool
    fn icon_for_tool(tool: ToolId) -> BezPath {
        match tool {
            ToolId::Select => select_icon(),
            ToolId::Pen => pen_icon(),
            ToolId::Preview => preview_icon(),
            ToolId::Knife => knife_icon(),
            ToolId::Rectangle => rect_icon(),
            ToolId::Ellipse => ellipse_icon(),
            ToolId::Measure => measure_icon(),
        }
    }

    /// Get the rect for a tool button by index
    fn button_rect(&self, index: usize) -> Rect {
        let x = TOOLBAR_PADDING + index as f64 * (TOOLBAR_ITEM_SIZE + TOOLBAR_ITEM_SPACING);
        let y = TOOLBAR_PADDING;
        Rect::new(x, y, x + TOOLBAR_ITEM_SIZE, y + TOOLBAR_ITEM_SIZE)
    }

    /// Find which tool was clicked
    fn tool_at_point(&self, point: Point) -> Option<ToolId> {
        for (i, &tool) in TOOLBAR_TOOLS.iter().enumerate() {
            if self.button_rect(i).contains(point) {
                return Some(tool);
            }
        }
        None
    }
}

/// Action sent when a tool is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolSelected(pub ToolId);

impl Widget for ToolbarWidget {
    type Action = ToolSelected;

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {
        // Leaf widget - no children
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
        // No update logic needed
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        // Calculate total width needed for all tools plus padding
        let num_tools = TOOLBAR_TOOLS.len();
        let width = TOOLBAR_PADDING * 2.0 + num_tools as f64 * TOOLBAR_ITEM_SIZE + (num_tools - 1) as f64 * TOOLBAR_ITEM_SPACING;
        let height = TOOLBAR_ITEM_SIZE + TOOLBAR_PADDING * 2.0;
        let size = Size::new(width, height);
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        // Draw a solid background panel behind all buttons to prevent transparency issues
        let size = ctx.size();
        let panel_rect = size.to_rect();
        let panel_rrect = kurbo::RoundedRect::from_rect(panel_rect, 8.0);

        // Solid opaque background - darker than buttons but brighter than canvas
        fill_color(scene, &panel_rrect, COLOR_PANEL);

        // Draw panel border - inset slightly to prevent corner artifacts
        let border_inset = BORDER_WIDTH / 2.0;
        let inset_rect = panel_rect.inset(-border_inset);
        let inset_rrect = kurbo::RoundedRect::from_rect(inset_rect, 8.0);
        stroke(scene, &inset_rrect, COLOR_PANEL_BORDER, BORDER_WIDTH);

        // Draw each toolbar button as a separate rounded rectangle
        for (i, &tool) in TOOLBAR_TOOLS.iter().enumerate() {
            let button_rect = self.button_rect(i);
            let is_selected = tool == self.selected_tool;

            // Create rounded rectangle for button
            let button_rrect = kurbo::RoundedRect::from_rect(button_rect, BUTTON_RADIUS);

            // Draw button background
            let bg_color = if is_selected { COLOR_SELECTED } else { COLOR_UNSELECTED };
            fill_color(scene, &button_rrect, bg_color);

            // Draw button border (thicker)
            stroke(scene, &button_rrect, COLOR_BUTTON_BORDER, BORDER_WIDTH);

            // Draw icon
            let icon_path = Self::icon_for_tool(tool);
            let constrained_path = constrain_icon(icon_path, button_rect);

            // Stroke icon (no fill, just outline) - dark color for all buttons
            stroke(scene, &constrained_path, COLOR_ICON, ITEM_STROKE_WIDTH);
        }
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

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
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
                if let Some(tool) = self.tool_at_point(local_pos) {
                    if tool != self.selected_tool {
                        self.selected_tool = tool;
                        ctx.submit_action::<ToolSelected>(ToolSelected(tool));
                        ctx.request_render();
                    }
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

/// Constrain an icon path to fit within a button rect with padding
fn constrain_icon(mut path: BezPath, button_rect: Rect) -> BezPath {
    // Get bounding box of the icon
    let bounds = path.bounding_box();

    // Calculate available space (button size minus padding)
    let available = TOOLBAR_ITEM_SIZE - 2.0 * ICON_PADDING;

    // Calculate scale to fit icon in available space
    let scale_x = available / bounds.width();
    let scale_y = available / bounds.height();
    let scale = scale_x.min(scale_y);

    // Center the icon in the button
    let scaled_width = bounds.width() * scale;
    let scaled_height = bounds.height() * scale;
    let offset_x = button_rect.min_x() + (TOOLBAR_ITEM_SIZE - scaled_width) / 2.0 - bounds.min_x() * scale;
    let offset_y = button_rect.min_y() + (TOOLBAR_ITEM_SIZE - scaled_height) / 2.0 - bounds.min_y() * scale;

    // Apply transformation
    let transform = Affine::translate((offset_x, offset_y)) * Affine::scale(scale);
    path.apply_affine(transform);

    path
}

// --- Icon Definitions ---

fn select_icon() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((110.0, 500.0));
    bez.line_to((110.0, 380.0));
    bez.line_to((2.0, 410.0));
    bez.line_to((0.0, 410.0));
    bez.line_to((159.0, 0.0));
    bez.line_to((161.0, 0.0));
    bez.line_to((320.0, 410.0));
    bez.line_to((318.0, 410.0));
    bez.line_to((210.0, 380.0));
    bez.line_to((210.0, 500.0));
    bez.line_to((110.0, 500.0));
    bez.close_path();
    bez
}

fn pen_icon() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((40.0, 500.0));
    bez.line_to((240.0, 500.0));
    bez.line_to((240.0, 410.0));
    bez.line_to((40.0, 410.0));
    bez.line_to((40.0, 500.0));
    bez.close_path();
    bez.move_to((40.0, 410.0));
    bez.line_to((240.0, 410.0));
    bez.line_to((239.0, 370.0));
    bez.line_to((280.0, 290.0));
    bez.curve_to((240.0, 220.0), (205.0, 130.0), (195.0, 0.0));
    bez.line_to((85.0, 0.0));
    bez.curve_to((75.0, 130.0), (40.0, 220.0), (0.0, 290.0));
    bez.line_to((40.0, 370.0));
    bez.line_to((40.0, 410.0));
    bez.close_path();
    bez.move_to((140.0, 0.0));
    bez.line_to((140.0, 266.0));
    bez.move_to((173.0, 300.0));
    bez.curve_to((173.0, 283.0), (159.0, 267.0), (140.0, 267.0));
    bez.curve_to((121.0, 267.0), (107.0, 283.0), (107.0, 300.0));
    bez.curve_to((107.0, 317.0), (121.0, 333.0), (140.0, 333.0));
    bez.curve_to((159.0, 333.0), (173.0, 317.0), (173.0, 300.0));
    bez.close_path();
    bez
}

fn preview_icon() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((130.0, 500.0));
    bez.line_to((310.0, 500.0));
    bez.line_to((310.0, 410.0));
    bez.curve_to((336.0, 375.0), (360.0, 351.0), (360.0, 310.0));
    bez.line_to((360.0, 131.0));
    bez.curve_to((360.0, 89.0), (352.0, 70.0), (336.0, 70.0));
    bez.curve_to((316.0, 70.0), (310.0, 85.0), (310.0, 101.0));
    bez.curve_to((310.0, 60.0), (309.0, 20.0), (280.0, 20.0));
    bez.curve_to((260.0, 20.0), (250.0, 36.0), (250.0, 60.0));
    bez.curve_to((250.0, 26.0), (242.0, 0.0), (216.0, 0.0));
    bez.curve_to((192.0, 0.0), (180.0, 16.0), (180.0, 75.0));
    bez.curve_to((180.0, 48.0), (169.0, 30.0), (150.0, 30.0));
    bez.curve_to((130.0, 30.0), (120.0, 53.0), (120.0, 75.0));
    bez.line_to((120.0, 250.0));
    bez.curve_to((120.0, 270.0), (110.0, 270.0), (100.0, 270.0));
    bez.curve_to((85.0, 270.0), (77.0, 264.0), (70.0, 250.0));
    bez.curve_to((45.0, 199.0), (32.0, 190.0), (20.0, 190.0));
    bez.curve_to((8.0, 190.0), (0.0, 197.0), (0.0, 210.0));
    bez.curve_to((0.0, 234.0), (19.0, 313.0), (30.0, 330.0));
    bez.curve_to((41.0, 347.0), (87.0, 383.0), (130.0, 410.0));
    bez.line_to((130.0, 500.0));
    bez.close_path();
    bez.move_to((130.0, 410.0));
    bez.line_to((310.0, 410.0));
    bez.move_to((180.0, 75.0));
    bez.line_to((180.0, 210.0));
    bez.move_to((250.0, 60.0));
    bez.line_to((250.0, 210.0));
    bez.move_to((310.0, 101.0));
    bez.line_to((310.0, 220.0));
    bez
}

fn knife_icon() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((30.0, 500.0));
    bez.line_to((190.0, 500.0));
    bez.line_to((190.0, 410.0));
    bez.line_to((30.0, 410.0));
    bez.line_to((30.0, 500.0));
    bez.close_path();
    bez.move_to((40.0, 360.0));
    bez.line_to((180.0, 360.0));
    bez.line_to((180.0, 330.0));
    bez.line_to((220.0, 290.0));
    bez.line_to((42.0, 0.0));
    bez.line_to((40.0, 0.0));
    bez.line_to((40.0, 360.0));
    bez.close_path();
    bez.move_to((30.0, 410.0));
    bez.line_to((190.0, 410.0));
    bez.curve_to((205.0, 410.0), (220.0, 405.0), (220.0, 385.0));
    bez.curve_to((220.0, 365.0), (205.0, 360.0), (190.0, 360.0));
    bez.line_to((30.0, 360.0));
    bez.curve_to((15.0, 360.0), (0.0, 365.0), (0.0, 385.0));
    bez.curve_to((0.0, 405.0), (15.0, 410.0), (30.0, 410.0));
    bez.close_path();
    bez
}

fn rect_icon() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((0.0, 500.0));
    bez.line_to((220.0, 500.0));
    bez.line_to((220.0, 0.0));
    bez.line_to((0.0, 0.0));
    bez.line_to((0.0, 500.0));
    bez.close_path();
    bez
}

fn ellipse_icon() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((110.0, 0.0));
    bez.curve_to((50.0, 0.0), (0.0, 100.0), (0.0, 240.0));
    bez.curve_to((0.0, 380.0), (50.0, 480.0), (110.0, 480.0));
    bez.curve_to((170.0, 480.0), (220.0, 380.0), (220.0, 240.0));
    bez.curve_to((220.0, 100.0), (170.0, 0.0), (110.0, 0.0));
    bez.close_path();
    bez
}

// --- XILEM VIEW WRAPPER ---

use std::marker::PhantomData;
use xilem::core::{MessageContext, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

/// Create a toolbar view
pub fn toolbar_view<State, Action>(
    selected_tool: ToolId,
    callback: impl Fn(&mut State, ToolId) + Send + Sync + 'static,
) -> ToolbarView<State, Action>
where
    Action: 'static,
{
    ToolbarView {
        selected_tool,
        callback: Box::new(callback),
        phantom: PhantomData,
    }
}

/// The Xilem View for ToolbarWidget
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct ToolbarView<State, Action = ()> {
    selected_tool: ToolId,
    callback: Box<dyn Fn(&mut State, ToolId) + Send + Sync>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action> ViewMarker for ToolbarView<State, Action> {}

impl<State: 'static, Action: 'static> View<State, Action, ViewCtx> for ToolbarView<State, Action> {
    type Element = Pod<ToolbarWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let widget = ToolbarWidget::new(self.selected_tool);
        (ctx.with_action_widget(|ctx| ctx.create_pod(widget)), ())
    }

    fn rebuild(
        &self,
        _prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        // Update widget if selected tool changed
        let mut widget = element.downcast::<ToolbarWidget>();
        if widget.widget.selected_tool != self.selected_tool {
            widget.widget.selected_tool = self.selected_tool;
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
    ) -> MessageResult<Action> {
        // Handle tool selection actions from widget
        match message.take_message::<ToolSelected>() {
            Some(action) => {
                (self.callback)(app_state, action.0);
                MessageResult::Nop
            }
            None => MessageResult::Stale,
        }
    }
}

fn measure_icon() -> BezPath {
    let mut bez = BezPath::new();
    bez.move_to((0.0, 500.0));
    bez.line_to((140.0, 500.0));
    bez.line_to((140.0, 0.0));
    bez.line_to((0.0, 0.0));
    bez.line_to((0.0, 500.0));
    bez.close_path();
    bez.move_to((190.0, 0.0));
    bez.line_to((330.0, 0.0));
    bez.move_to((190.0, 500.0));
    bez.line_to((330.0, 500.0));
    bez.move_to((210.0, 100.0));
    bez.line_to((310.0, 100.0));
    bez.line_to((260.0, 10.0));
    bez.line_to((210.0, 100.0));
    bez.close_path();
    bez.move_to((210.0, 400.0));
    bez.line_to((310.0, 400.0));
    bez.line_to((260.0, 490.0));
    bez.line_to((210.0, 400.0));
    bez.close_path();
    bez.move_to((260.0, 100.0));
    bez.line_to((260.0, 400.0));
    bez.move_to((70.0, 350.0));
    bez.line_to((140.0, 350.0));
    bez.move_to((100.0, 400.0));
    bez.line_to((140.0, 400.0));
    bez.move_to((50.0, 450.0));
    bez.line_to((140.0, 450.0));
    bez.move_to((100.0, 300.0));
    bez.line_to((140.0, 300.0));
    bez.move_to((50.0, 250.0));
    bez.line_to((140.0, 250.0));
    bez.move_to((70.0, 150.0));
    bez.line_to((140.0, 150.0));
    bez.move_to((100.0, 200.0));
    bez.line_to((140.0, 200.0));
    bez.move_to((100.0, 100.0));
    bez.line_to((140.0, 100.0));
    bez.move_to((50.0, 50.0));
    bez.line_to((140.0, 50.0));
    bez
}
