// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Application state and data structures

use crate::workspace::Workspace;
use std::path::PathBuf;

/// Main application state
pub struct AppState {
    /// The loaded font workspace, if any
    pub workspace: Option<Workspace>,

    /// Error message to display, if any
    pub error_message: Option<String>,

    /// Currently selected glyph name
    pub selected_glyph: Option<String>,
}

impl AppState {
    /// Create a new empty application state
    pub fn new() -> Self {
        Self {
            workspace: None,
            error_message: None,
            selected_glyph: None,
        }
    }

    /// Open a file dialog to select a UFO directory
    pub fn open_font_dialog(&mut self) {
        self.error_message = None;

        // Show folder picker dialog
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select UFO Directory")
            .pick_folder()
        {
            self.load_ufo(path);
        }
    }

    /// Load a UFO from a path
    pub fn load_ufo(&mut self, path: PathBuf) {
        match Workspace::load(&path) {
            Ok(workspace) => {
                println!("Loaded font: {}", workspace.display_name());
                println!("Glyphs: {}", workspace.glyph_count());
                self.workspace = Some(workspace);
                self.error_message = None;
            }
            Err(e) => {
                let error = format!("Failed to load UFO: {}", e);
                eprintln!("{}", error);
                self.error_message = Some(error);
            }
        }
    }

    /// Create a new empty font
    pub fn create_new_font(&mut self) {
        // TODO: Implement new font creation
        println!("Creating new font...");
        self.error_message = Some("New font creation not yet implemented".to_string());
    }

    /// Get the current font display name
    pub fn font_display_name(&self) -> Option<String> {
        self.workspace.as_ref().map(|w| w.display_name())
    }

    /// Get the number of glyphs in the current font
    pub fn glyph_count(&self) -> Option<usize> {
        self.workspace.as_ref().map(|w| w.glyph_count())
    }

    /// Select a glyph by name
    pub fn select_glyph(&mut self, name: String) {
        self.selected_glyph = Some(name);
    }

    /// Get all glyph names
    pub fn glyph_names(&self) -> Vec<String> {
        self.workspace
            .as_ref()
            .map(|w| w.glyph_names())
            .unwrap_or_default()
    }

    /// Get the selected glyph's advance width
    pub fn selected_glyph_advance(&self) -> Option<f64> {
        if let (Some(workspace), Some(glyph_name)) = (&self.workspace, &self.selected_glyph) {
            workspace.get_glyph(glyph_name)
                .map(|g| g.width)
        } else {
            None
        }
    }

    /// Get the selected glyph's unicode value
    pub fn selected_glyph_unicode(&self) -> Option<String> {
        if let (Some(workspace), Some(glyph_name)) = (&self.workspace, &self.selected_glyph) {
            workspace.get_glyph(glyph_name)
                .and_then(|g| {
                    if g.codepoints.is_empty() {
                        None
                    } else {
                        g.codepoints.iter().next()
                            .map(|c| format!("U+{:04X}", *c as u32))
                    }
                })
        } else {
            None
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
