// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Toolbar for selecting editing tools

use crate::tools::ToolId;

/// Toolbar state
#[derive(Debug, Clone, PartialEq)]
pub struct ToolbarState {
    /// Currently selected tool
    pub selected_tool: ToolId,
}

impl ToolbarState {
    pub fn new() -> Self {
        Self {
            selected_tool: ToolId::Select,
        }
    }

    pub fn select_tool(&mut self, tool: ToolId) {
        self.selected_tool = tool;
    }
}

impl Default for ToolbarState {
    fn default() -> Self {
        Self::new()
    }
}

/// Available tools in order of display
pub const TOOLBAR_TOOLS: &[ToolId] = &[
    ToolId::Select,
    ToolId::Pen,
    ToolId::Preview,
    ToolId::Knife,
    ToolId::Rectangle,
    ToolId::Ellipse,
    ToolId::Measure,
];

/// Get the display label for a tool
pub fn tool_label(tool: ToolId) -> &'static str {
    match tool {
        ToolId::Select => "Select (V)",
        ToolId::Pen => "Pen (P)",
        ToolId::Preview => "Preview (H)",
        ToolId::Knife => "Knife (E)",
        ToolId::Rectangle => "Rect (U)",
        ToolId::Ellipse => "Ellipse",
        ToolId::Measure => "Measure (M)",
    }
}
