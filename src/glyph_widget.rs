// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Custom widget for rendering font glyphs using Vello
//!
//! This module provides a Masonry widget that renders glyph outlines
//! from Kurbo BezPath data using Vello's GPU-accelerated rendering.

use kurbo::{Affine, BezPath, Shape};
use masonry::accesskit::{Node, Role};
use masonry::core::{AccessCtx, BoxConstraints, ChildrenIds, LayoutCtx, NoAction, PaintCtx, PropertiesMut, PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget};
use masonry::kurbo::Size;
use masonry::util::fill_color;
use masonry::vello::peniko::Color;
use masonry::vello::Scene;

/// A widget that renders a glyph from a BezPath
pub struct GlyphWidget {
    /// The bezier path representing the glyph outline
    path: BezPath,
    /// The color to fill the glyph with
    color: Color,
    /// Target display size for the widget
    size: Size,
}

impl GlyphWidget {
    /// Create a new GlyphWidget from a BezPath
    pub fn new(path: BezPath, size: Size) -> Self {
        Self {
            path,
            color: Color::from_rgb8(0, 0, 0), // Default to black
            size,
        }
    }

    /// Set the fill color for the glyph
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Widget for GlyphWidget {
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
        // No state to update
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        // Use the requested size, constrained by the BoxConstraints
        bc.constrain(self.size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        if self.path.is_empty() {
            return;
        }

        // Get the bounding box of the glyph path
        let bounds = self.path.bounding_box();
        let widget_size = ctx.size();

        // Calculate scale to fit the glyph in the widget
        // Leave some padding (10% on each side = 80% fill)
        let padding_factor = 0.8;
        let scale_x = (widget_size.width * padding_factor) / bounds.width();
        let scale_y = (widget_size.height * padding_factor) / bounds.height();
        let scale = scale_x.min(scale_y);

        // Center the glyph in the widget
        let scaled_width = bounds.width() * scale;
        let scaled_height = bounds.height() * scale;
        let offset_x = (widget_size.width - scaled_width) / 2.0 - bounds.x0 * scale;
        let offset_y = (widget_size.height - scaled_height) / 2.0 - bounds.y0 * scale;

        // Create transform: translate to center, then scale
        let transform = Affine::translate((offset_x, offset_y)) * Affine::scale(scale);

        // Apply transform to path
        let transformed_path = transform * &self.path;

        // Render the glyph
        fill_color(scene, &transformed_path, self.color);
    }

    fn accessibility_role(&self) -> Role {
        Role::Image
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
        // Could add accessibility description for the glyph if needed
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }
}

// --- MARK: XILEM VIEW WRAPPER ---

use std::marker::PhantomData;
use xilem::core::{MessageContext, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

/// Create a glyph view from a BezPath
pub fn glyph_view<State, Action>(
    path: BezPath,
    width: f64,
    height: f64,
) -> GlyphView<State, Action> {
    GlyphView {
        path,
        size: Size::new(width, height),
        color: None,
        phantom: PhantomData,
    }
}

/// The Xilem View for GlyphWidget
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct GlyphView<State, Action = ()> {
    path: BezPath,
    size: Size,
    color: Option<Color>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action> GlyphView<State, Action> {
    /// Set the glyph fill color
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl<State, Action> ViewMarker for GlyphView<State, Action> {}

impl<State: 'static, Action: 'static> View<State, Action, ViewCtx> for GlyphView<State, Action> {
    type Element = Pod<GlyphWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let mut widget = GlyphWidget::new(self.path.clone(), self.size);
        if let Some(color) = self.color {
            widget = widget.with_color(color);
        }
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
        // GlyphWidget doesn't support incremental updates yet.
        // If properties change significantly, the parent should recreate the view.
        // For simple property changes, the widget will repaint on next frame.
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
        // GlyphWidget doesn't produce any messages
        MessageResult::Stale
    }
}
