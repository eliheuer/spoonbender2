// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! UI components for the Spoonbender font editor

pub mod coordinate_panel;
pub mod editor;
pub mod glyph;
pub mod grid_toolbar;
pub mod toolbar;

// Re-export commonly used widget views and types
pub use coordinate_panel::{coordinate_panel, calculate_coordinate_selection, CoordinateSelection};
pub use editor::editor_view;
pub use glyph::glyph_view;
pub use grid_toolbar::grid_toolbar_view;
pub use toolbar::toolbar_view;
