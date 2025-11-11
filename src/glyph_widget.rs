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
    /// Units per em from the font (for uniform scaling)
    upm: f64,
    /// Baseline offset as a fraction of height (0.0 = bottom, 1.0 = top)
    baseline_offset: f64,
}

impl GlyphWidget {
    /// Create a new GlyphWidget from a BezPath
    pub fn new(path: BezPath, size: Size, upm: f64) -> Self {
        Self {
            path,
            color: Color::from_rgb8(0, 0, 0), // Default to black
            size,
            upm,
            baseline_offset: 0.02, // Default baseline offset
        }
    }

    /// Set the fill color for the glyph
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the baseline offset (0.0 = bottom, 1.0 = top)
    pub fn with_baseline_offset(mut self, offset: f64) -> Self {
        self.baseline_offset = offset;
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

        // Calculate uniform scale based on UPM (units per em)
        // This ensures all glyphs are rendered at the same scale
        let scale = widget_size.height / self.upm;
        let scale = scale * 1.0; // No padding - fill the space

        // Center the glyph horizontally based on its bounding box
        let scaled_width = bounds.width() * scale;
        let l_pad = (widget_size.width - scaled_width) / 2.0;

        // Position baseline to center glyphs vertically (adjusted for better visual balance)
        // Higher percentage = baseline higher in cell = more space at bottom, less at top
        let baseline = widget_size.height * self.baseline_offset;

        // UFO coordinates have Y increasing upward, but screen coords have Y increasing downward
        // Create affine transformation: scale (with Y-flip) and translate
        let transform = Affine::new([
            scale,                          // x scale
            0.0,                            // x skew
            0.0,                            // y skew
            -scale,                         // y scale (negative to flip Y axis)
            l_pad - bounds.x0 * scale,      // x translation (centering)
            widget_size.height - baseline,  // y translation (baseline positioning)
        ]);

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
    upm: f64,
) -> GlyphView<State, Action> {
    GlyphView {
        path,
        size: Size::new(width, height),
        color: None,
        upm,
        baseline_offset: None,
        phantom: PhantomData,
    }
}

/// The Xilem View for GlyphWidget
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct GlyphView<State, Action = ()> {
    path: BezPath,
    size: Size,
    color: Option<Color>,
    upm: f64,
    baseline_offset: Option<f64>,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<State, Action> GlyphView<State, Action> {
    /// Set the glyph fill color
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the baseline offset (0.0 = bottom, 1.0 = top)
    pub fn baseline_offset(mut self, offset: f64) -> Self {
        self.baseline_offset = Some(offset);
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
        let mut widget = GlyphWidget::new(self.path.clone(), self.size, self.upm);
        if let Some(color) = self.color {
            widget = widget.with_color(color);
        }
        if let Some(offset) = self.baseline_offset {
            widget = widget.with_baseline_offset(offset);
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
