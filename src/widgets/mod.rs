// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! UI widgets for the Spoonbender font editor

pub mod coord_pane;
pub mod editor;
pub mod glyph;
pub mod toolbar;

// Re-export commonly used widget views and types
pub use coord_pane::{coord_pane_view, coordinate_info_pane, calculate_coordinate_selection, CoordinateSelection};
pub use editor::editor_view;
pub use glyph::glyph_view;
pub use toolbar::toolbar_view;
