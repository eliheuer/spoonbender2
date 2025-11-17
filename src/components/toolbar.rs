// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Toolbar widget with icon-based tool buttons

use crate::tools::ToolId;
use kurbo::{Affine, BezPath, Point, Rect, Shape, Size};
use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, EventCtx, LayoutCtx, PaintCtx, PointerButton,
    PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update,
    UpdateCtx, Widget,
};
use masonry::util::{fill_color, stroke};
use masonry::vello::Scene;
use masonry::vello::peniko::Color;

/// Toolbar dimensions
const TOOLBAR_ITEM_SIZE: f64 = 48.0;
const TOOLBAR_ITEM_SPACING: f64 = 6.0; // Space between buttons
const TOOLBAR_PADDING: f64 = 8.0; // Padding around the entire toolbar (space between buttons and container)
const ICON_PADDING: f64 = 8.0;
const ITEM_STROKE_WIDTH: f64 = 1.5;
const BUTTON_RADIUS: f64 = 6.0; // Rounded corner radius
const BORDER_WIDTH: f64 = 1.5; // Border thickness for buttons and panel

/// Toolbar colors (from theme)
const COLOR_PANEL: Color = crate::theme::panel::BACKGROUND; // Panel background
const COLOR_UNSELECTED: Color = crate::theme::toolbar::BUTTON_UNSELECTED; // Unselected buttons
const COLOR_SELECTED: Color = crate::theme::toolbar::BUTTON_SELECTED; // Selected button
const COLOR_ICON: Color = crate::theme::toolbar::ICON; // Icon color
const COLOR_PANEL_BORDER: Color = crate::theme::panel::OUTLINE; // Panel container border
const COLOR_BUTTON_BORDER: Color = crate::theme::panel::BUTTON_OUTLINE; // Toolbar button borders

/// Available tools in display order
/// Currently only showing implemented tools: Select, Pen, Preview
const TOOLBAR_TOOLS: &[ToolId] = &[ToolId::Select, ToolId::Pen, ToolId::Preview];

/// Toolbar widget
pub struct ToolbarWidget {
    /// Currently selected tool
    selected_tool: ToolId,
    /// Currently hovered tool (if any)
    hover_tool: Option<ToolId>,
}

impl ToolbarWidget {
    pub fn new(selected_tool: ToolId) -> Self {
        Self {
            selected_tool,
            hover_tool: None,
        }
    }

    /// Get the icon path for a tool
    fn icon_for_tool(tool: ToolId) -> BezPath {
        match tool {
            ToolId::Select => select_icon(),
            ToolId::Pen => pen_icon(),
            ToolId::Preview => preview_icon(),
            _ => {
                // For any unimplemented tools, return empty path
                BezPath::new()
            }
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
        let width = TOOLBAR_PADDING * 2.0
            + num_tools as f64 * TOOLBAR_ITEM_SIZE
            + (num_tools - 1) as f64 * TOOLBAR_ITEM_SPACING;
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
            let bg_color = if is_selected {
                COLOR_SELECTED
            } else {
                COLOR_UNSELECTED
            };
            fill_color(scene, &button_rrect, bg_color);

            // Draw button border (thicker)
            stroke(scene, &button_rrect, COLOR_BUTTON_BORDER, BORDER_WIDTH);

            // Draw icon
            let icon_path = Self::icon_for_tool(tool);
            let constrained_path = constrain_icon(icon_path, button_rect, tool);

            // Determine icon color based on state
            let is_hovered = self.hover_tool == Some(tool);
            let icon_color = if is_selected || is_hovered {
                crate::theme::base::C // Darker icon for selected or hovered button
            } else {
                COLOR_ICON // Normal icon color for unselected, unhovered buttons
            };
            fill_color(scene, &constrained_path, icon_color);
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
            PointerEvent::Down(PointerButtonEvent {
                button: Some(PointerButton::Primary),
                state,
                ..
            }) => {
                println!(
                    "[ToolbarWidget::on_pointer_event] Down at {:?}",
                    state.position
                );
                let local_pos = ctx.local_position(state.position);
                println!(
                    "[ToolbarWidget::on_pointer_event] local_pos: {:?}",
                    local_pos
                );
                if let Some(tool) = self.tool_at_point(local_pos) {
                    println!("[ToolbarWidget::on_pointer_event] Hit tool: {:?}", tool);
                    if tool != self.selected_tool {
                        self.selected_tool = tool;
                        ctx.submit_action::<ToolSelected>(ToolSelected(tool));
                        ctx.request_render();
                    }
                    // Mark event as handled to prevent it from reaching widgets below in zstack
                    ctx.set_handled();
                } else {
                    // Even if we didn't hit a tool, consume the event so it doesn't go to the editor
                    ctx.set_handled();
                }
            }
            PointerEvent::Move(pointer_move) => {
                let local_pos = ctx.local_position(pointer_move.current.position);
                let new_hover = self.tool_at_point(local_pos);
                if new_hover != self.hover_tool {
                    self.hover_tool = new_hover;
                    ctx.request_render();
                }
            }
            PointerEvent::Leave(_) => {
                if self.hover_tool.is_some() {
                    self.hover_tool = None;
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

/// Constrain an icon path to fit within a button rect with padding
fn constrain_icon(mut path: BezPath, button_rect: Rect, tool: ToolId) -> BezPath {
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
    let mut offset_x =
        button_rect.min_x() + (TOOLBAR_ITEM_SIZE - scaled_width) / 2.0 - bounds.min_x() * scale;
    let offset_y =
        button_rect.min_y() + (TOOLBAR_ITEM_SIZE - scaled_height) / 2.0 - bounds.min_y() * scale;

    // Apply per-tool visual centering adjustments
    match tool {
        ToolId::Select | ToolId::Preview => {
            // Shift 1px to the right for better visual centering
            offset_x += 1.0;
        }
        _ => {}
    }

    // Apply transformation
    let transform = Affine::translate((offset_x, offset_y)) * Affine::scale(scale);
    path.apply_affine(transform);

    path
}

// --- Icon Definitions ---

fn select_icon() -> BezPath {
    // Icon from VirtuaGrotesk E010 (U+E010) - selection cursor
    // Y coordinates flipped to convert from UFO (Y-up) to screen (Y-down)
    let mut bez = BezPath::new();

    bez.move_to((328.0, 768.0));
    bez.curve_to((314.0, 768.0), (308.0, 762.0), (300.0, 734.0));
    bez.line_to((246.0, 542.0));
    bez.curve_to((240.0, 522.0), (236.0, 514.0), (224.0, 514.0));
    bez.curve_to((210.0, 514.0), (202.0, 520.0), (192.0, 530.0));
    bez.line_to((138.0, 584.0));
    bez.curve_to((122.0, 600.0), (108.0, 608.0), (90.0, 608.0));
    bez.curve_to((72.0, 608.0), (66.0, 598.0), (66.0, 574.0));
    bez.line_to((64.0, 50.0));
    bez.curve_to((64.0, 18.0), (78.0, 0.0), (96.0, 0.0));
    bez.curve_to((120.0, 0.0), (142.0, 16.0), (176.0, 50.0));
    bez.line_to((506.0, 368.0));
    bez.curve_to((526.0, 386.0), (540.0, 404.0), (540.0, 422.0));
    bez.curve_to((540.0, 440.0), (528.0, 450.0), (502.0, 450.0));
    bez.line_to((388.0, 450.0));
    bez.curve_to((368.0, 450.0), (360.0, 458.0), (360.0, 470.0));
    bez.curve_to((360.0, 484.0), (370.0, 496.0), (378.0, 510.0));
    bez.line_to((450.0, 634.0));
    bez.curve_to((460.0, 650.0), (478.0, 674.0), (478.0, 688.0));
    bez.curve_to((478.0, 706.0), (462.0, 714.0), (444.0, 722.0));
    bez.line_to((366.0, 760.0));
    bez.curve_to((352.0, 766.0), (344.0, 768.0), (328.0, 768.0));
    bez.close_path();

    bez
}

fn pen_icon() -> BezPath {
    // Icon from VirtuaGrotesk E011 (U+E011) - pen tool
    // Y coordinates flipped to convert from UFO (Y-up) to screen (Y-down)
    let mut bez = BezPath::new();

    // Contour 1 - top rectangle (nib)
    bez.move_to((200.0, 768.0));
    bez.line_to((432.0, 768.0));
    bez.curve_to((452.0, 768.0), (456.0, 764.0), (456.0, 744.0));
    bez.line_to((456.0, 678.0));
    bez.curve_to((456.0, 658.0), (452.0, 654.0), (432.0, 654.0));
    bez.line_to((200.0, 654.0));
    bez.curve_to((180.0, 654.0), (176.0, 658.0), (176.0, 678.0));
    bez.line_to((176.0, 744.0));
    bez.curve_to((176.0, 764.0), (180.0, 768.0), (200.0, 768.0));
    bez.close_path();

    // Contour 2 - pen body
    bez.move_to((200.0, 602.0));
    bez.line_to((432.0, 602.0));
    bez.curve_to((454.0, 602.0), (460.0, 604.0), (480.0, 576.0));
    bez.line_to((548.0, 484.0));
    bez.curve_to((556.0, 472.0), (564.0, 462.0), (564.0, 452.0));
    bez.line_to((564.0, 416.0));
    bez.curve_to((564.0, 410.0), (560.0, 396.0), (556.0, 384.0));
    bez.line_to((440.0, 32.0));
    bez.curve_to((430.0, 0.0), (416.0, 0.0), (400.0, 0.0));
    bez.line_to((364.0, 0.0));
    bez.curve_to((348.0, 0.0), (342.0, 8.0), (342.0, 32.0));
    bez.line_to((342.0, 336.0));
    bez.curve_to((342.0, 358.0), (346.0, 362.0), (352.0, 366.0));
    bez.curve_to((374.0, 378.0), (392.0, 400.0), (392.0, 434.0));
    bez.curve_to((392.0, 478.0), (360.0, 510.0), (316.0, 510.0));
    bez.curve_to((272.0, 510.0), (240.0, 478.0), (240.0, 434.0));
    bez.curve_to((240.0, 400.0), (258.0, 378.0), (280.0, 366.0));
    bez.curve_to((286.0, 362.0), (290.0, 358.0), (290.0, 336.0));
    bez.line_to((290.0, 32.0));
    bez.curve_to((290.0, 8.0), (284.0, 0.0), (268.0, 0.0));
    bez.line_to((232.0, 0.0));
    bez.curve_to((216.0, 0.0), (202.0, 0.0), (192.0, 32.0));
    bez.line_to((76.0, 384.0));
    bez.curve_to((72.0, 396.0), (68.0, 410.0), (68.0, 416.0));
    bez.line_to((68.0, 452.0));
    bez.curve_to((68.0, 462.0), (76.0, 472.0), (84.0, 484.0));
    bez.line_to((152.0, 576.0));
    bez.curve_to((172.0, 602.0), (180.0, 602.0), (200.0, 602.0));
    bez.close_path();

    bez
}

fn preview_icon() -> BezPath {
    // Icon from VirtuaGrotesk E014 (U+E014) - preview/hand tool
    // Y coordinates flipped to convert from UFO (Y-up) to screen (Y-down)
    let mut bez = BezPath::new();

    bez.move_to((256.0, 798.0));
    bez.line_to((240.0, 798.0));
    bez.curve_to((232.0, 788.0), (232.0, 774.0), (232.0, 774.0));
    bez.line_to((232.0, 726.0));
    bez.curve_to((232.0, 714.0), (226.0, 704.0), (208.0, 686.0));
    bez.curve_to((128.0, 606.0), (90.0, 466.0), (90.0, 272.0));
    bez.curve_to((90.0, 202.0), (114.0, 168.0), (138.0, 168.0));
    bez.curve_to((152.0, 168.0), (158.0, 178.0), (158.0, 192.0));
    bez.curve_to((158.0, 208.0), (154.0, 224.0), (154.0, 264.0));
    bez.curve_to((154.0, 290.0), (168.0, 356.0), (182.0, 384.0));
    bez.curve_to((186.0, 392.0), (194.0, 394.0), (200.0, 394.0));
    bez.curve_to((206.0, 394.0), (212.0, 392.0), (212.0, 384.0));
    bez.curve_to((212.0, 372.0), (200.0, 332.0), (200.0, 296.0));
    bez.curve_to((200.0, 194.0), (230.0, 56.0), (266.0, 56.0));
    bez.curve_to((302.0, 56.0), (298.0, 80.0), (298.0, 92.0));
    bez.curve_to((298.0, 110.0), (286.0, 136.0), (286.0, 222.0));
    bez.curve_to((286.0, 292.0), (290.0, 318.0), (292.0, 326.0));
    bez.curve_to((294.0, 334.0), (302.0, 340.0), (308.0, 340.0));
    bez.curve_to((314.0, 340.0), (322.0, 334.0), (322.0, 326.0));
    bez.curve_to((322.0, 174.0), (370.0, 66.0), (396.0, 30.0));
    bez.curve_to((412.0, 8.0), (428.0, 0.0), (450.0, 0.0));
    bez.curve_to((462.0, 0.0), (470.0, 12.0), (470.0, 30.0));
    bez.curve_to((470.0, 54.0), (416.0, 118.0), (416.0, 272.0));
    bez.curve_to((416.0, 298.0), (416.0, 318.0), (418.0, 324.0));
    bez.curve_to((420.0, 330.0), (424.0, 332.0), (428.0, 332.0));
    bez.curve_to((432.0, 332.0), (440.0, 328.0), (442.0, 322.0));
    bez.curve_to((470.0, 194.0), (518.0, 122.0), (552.0, 90.0));
    bez.curve_to((566.0, 76.0), (578.0, 72.0), (592.0, 72.0));
    bez.curve_to((606.0, 72.0), (610.0, 82.0), (610.0, 98.0));
    bez.curve_to((610.0, 118.0), (522.0, 268.0), (522.0, 406.0));
    bez.curve_to((522.0, 464.0), (558.0, 490.0), (582.0, 490.0));
    bez.curve_to((612.0, 490.0), (638.0, 442.0), (660.0, 402.0));
    bez.curve_to((686.0, 356.0), (708.0, 336.0), (734.0, 336.0));
    bez.curve_to((748.0, 336.0), (756.0, 344.0), (756.0, 362.0));
    bez.curve_to((756.0, 402.0), (668.0, 668.0), (518.0, 734.0));
    bez.curve_to((500.0, 742.0), (490.0, 752.0), (490.0, 764.0));
    bez.line_to((490.0, 774.0));
    bez.curve_to((490.0, 790.0), (484.0, 798.0), (470.0, 798.0));
    bez.line_to((256.0, 798.0));
    bez.close_path();

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

impl<State: 'static, Action: 'static + Default> View<State, Action, ViewCtx>
    for ToolbarView<State, Action>
{
    type Element = Pod<ToolbarWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let widget = ToolbarWidget::new(self.selected_tool);
        let pod = ctx.create_pod(widget);
        ctx.record_action(pod.new_widget.id());
        (pod, ())
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
                println!("[ToolbarView::message] Tool selected: {:?}", action.0);
                (self.callback)(app_state, action.0);
                // Return Action to trigger full app rebuild so the toolbar gets the updated tool
                // This causes app_logic() to be called, which reads the fresh tool from the session
                MessageResult::Action(Action::default())
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
