// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Edit session - manages editing state for a single glyph

use crate::workspace::Glyph;
use std::sync::Arc;

/// Unique identifier for an editing session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    /// Create a new unique session ID
    pub fn next() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Viewport transformation between design space and screen space
#[derive(Debug, Clone)]
pub struct ViewPort {
    /// Scroll offset in screen space
    pub offset: kurbo::Vec2,

    /// Zoom level (screen pixels per design unit)
    pub zoom: f64,
}

impl ViewPort {
    /// Create a new viewport with default settings
    pub fn new() -> Self {
        Self {
            offset: kurbo::Vec2::ZERO,
            zoom: 1.0,
        }
    }

    /// Convert a point from design space to screen space
    pub fn to_screen(&self, point: kurbo::Point) -> kurbo::Point {
        // Design space: Y increases upward (font coordinates)
        // Screen space: Y increases downward (UI coordinates)
        // Apply: scale, flip Y, translate by offset
        kurbo::Point::new(
            point.x * self.zoom + self.offset.x,
            -point.y * self.zoom + self.offset.y,
        )
    }

    /// Convert a point from screen space to design space
    pub fn from_screen(&self, point: kurbo::Point) -> kurbo::Point {
        kurbo::Point::new(
            (point.x - self.offset.x) / self.zoom,
            -(point.y - self.offset.y) / self.zoom,
        )
    }

    /// Get the affine transformation from design space to screen space
    pub fn affine(&self) -> kurbo::Affine {
        // Build transformation: scale, flip Y, translate
        kurbo::Affine::new([
            self.zoom,    // x scale
            0.0,          // x skew
            0.0,          // y skew
            -self.zoom,   // y scale (negative for Y-flip)
            self.offset.x,  // x translation
            self.offset.y,  // y translation
        ])
    }
}

impl Default for ViewPort {
    fn default() -> Self {
        Self::new()
    }
}

/// Editing session for a single glyph
///
/// This holds all the state needed to edit a glyph, including the
/// outline data, selection, viewport, and metadata.
#[derive(Debug, Clone)]
pub struct EditSession {
    /// Unique identifier for this session
    pub id: SessionId,

    /// Name of the glyph being edited
    pub glyph_name: String,

    /// The glyph data being edited (owned copy for editing)
    pub glyph: Arc<Glyph>,

    /// Viewport transformation
    pub viewport: ViewPort,

    /// Font metrics (for drawing guides)
    pub units_per_em: f64,
    pub ascender: f64,
    pub descender: f64,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
}

impl EditSession {
    /// Create a new editing session for a glyph
    pub fn new(
        glyph_name: String,
        glyph: Glyph,
        units_per_em: f64,
        ascender: f64,
        descender: f64,
        x_height: Option<f64>,
        cap_height: Option<f64>,
    ) -> Self {
        Self {
            id: SessionId::next(),
            glyph_name,
            glyph: Arc::new(glyph),
            viewport: ViewPort::new(),
            units_per_em,
            ascender,
            descender,
            x_height,
            cap_height,
        }
    }

    /// Get a displayable title for this session
    pub fn title(&self) -> String {
        format!("Edit: {}", self.glyph_name)
    }

    /// Set the viewport for this session
    pub fn set_viewport(&mut self, viewport: ViewPort) {
        self.viewport = viewport;
    }
}
