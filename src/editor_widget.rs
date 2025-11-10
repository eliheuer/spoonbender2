// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Glyph editor widget - the main canvas for editing glyphs

use crate::edit_session::EditSession;
use crate::glyph_renderer;
use kurbo::{Affine, Point, Stroke};
use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, LayoutCtx, NoAction, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget,
};
use masonry::kurbo::Size;
use masonry::util::fill_color;
use masonry::vello::peniko::{Brush, Color};
use masonry::vello::Scene;
use std::sync::Arc;

/// The main glyph editor canvas widget
pub struct EditorWidget {
    /// The editing session
    session: Arc<EditSession>,

    /// Canvas size
    size: Size,
}

impl EditorWidget {
    /// Create a new editor widget
    pub fn new(session: Arc<EditSession>) -> Self {
        Self {
            session,
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

        // Get the glyph outline as a bezier path
        let glyph_path = glyph_renderer::glyph_to_bezpath(&self.session.glyph);

        if glyph_path.is_empty() {
            return;
        }

        // Get viewport transformation
        let viewport_transform = self.session.viewport.affine();

        // Calculate initial viewport positioning to center the glyph
        // We'll center based on the advance width and font metrics
        let upm = self.session.units_per_em;
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

        // Create transform: translate to origin, scale, flip Y, translate to canvas center
        let transform = Affine::translate((center_x, center_y))
            * Affine::scale_non_uniform(scale, -scale) // Negative Y scale for flip
            * Affine::translate((-design_center_x, -design_center_y));

        // Apply transform to path
        let transformed_path = transform * &glyph_path;

        // Draw the glyph outline
        let stroke_width = 2.0;
        let stroke = Stroke::new(stroke_width);
        let brush = Brush::Solid(Color::from_rgb8(200, 200, 200));
        scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &transformed_path);

        // Draw font metrics guides
        draw_metrics_guides(scene, &transform, &self.session, canvas_size);
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
    canvas_size: Size,
) {
    let guide_color = Color::from_rgba8(100, 100, 255, 128); // Semi-transparent blue
    let stroke_width = 1.0;
    let stroke = Stroke::new(stroke_width);

    // Helper to draw a horizontal line at a given Y coordinate in design space
    let draw_hline = |scene: &mut Scene, y: f64| {
        let start = Point::new(-1000.0, y);
        let end = Point::new(session.glyph.width + 1000.0, y);

        let start_screen = *transform * start;
        let end_screen = *transform * end;

        let line = kurbo::Line::new(start_screen, end_screen);
        let brush = Brush::Solid(guide_color);
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
