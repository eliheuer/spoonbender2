// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Application state and data structures

use crate::edit_session::EditSession;
use crate::workspace::Workspace;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use xilem::WindowId;

/// Main application state
pub struct AppState {
    /// The loaded font workspace, if any
    pub workspace: Option<Workspace>,

    /// Error message to display, if any
    pub error_message: Option<String>,

    /// Currently selected glyph name
    pub selected_glyph: Option<String>,

    /// Open editor sessions (window ID -> (glyph name, session))
    pub editor_sessions: HashMap<WindowId, (String, Arc<EditSession>)>,

    /// Main window ID
    pub main_window_id: WindowId,

    /// Whether the app should keep running
    pub running: bool,
}

impl AppState {
    /// Create a new empty application state
    pub fn new() -> Self {
        Self {
            workspace: None,
            error_message: None,
            selected_glyph: None,
            editor_sessions: HashMap::new(),
            main_window_id: WindowId::next(),
            running: true,
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

    /// Create an edit session for a glyph
    pub fn create_edit_session(&self, glyph_name: &str) -> Option<EditSession> {
        let workspace = self.workspace.as_ref()?;
        let glyph = workspace.get_glyph(glyph_name)?;

        Some(EditSession::new(
            glyph_name.to_string(),
            glyph.clone(),
            workspace.units_per_em.unwrap_or(1000.0),
            workspace.ascender.unwrap_or(800.0),
            workspace.descender.unwrap_or(-200.0),
            workspace.x_height,
            workspace.cap_height,
        ))
    }

    /// Open or focus an editor for a glyph
    pub fn open_editor(&mut self, glyph_name: String) {
        // Check if we already have a window for this glyph
        let already_open = self.editor_sessions.values()
            .any(|(name, _)| name == &glyph_name);

        if !already_open {
            if let Some(session) = self.create_edit_session(&glyph_name) {
                let window_id = WindowId::next();
                self.editor_sessions.insert(window_id, (glyph_name, Arc::new(session)));
            }
        }
    }

    /// Close an editor session by window ID
    pub fn close_editor(&mut self, window_id: WindowId) {
        self.editor_sessions.remove(&window_id);
    }

    /// Set the tool for all editor sessions
    ///
    /// Note: In a real implementation, each editor would have its own tool state.
    /// For now, we'll just update all editors to use the same tool.
    pub fn set_editor_tool(&mut self, tool_id: crate::tools::ToolId) {
        for (_window_id, (_glyph_name, session)) in self.editor_sessions.iter_mut() {
            // We need to make the session mutable
            // For now this is a no-op, but the tool change will be reflected
            // when the editor widget recreates from the session
            println!("Setting tool to {:?}", tool_id);
            // TODO: Actually update the session's tool
            // This requires making session mutable or using interior mutability
        }
    }

    /// Set the coordinate quadrant for an editor session
    pub fn set_coord_quadrant(&mut self, _window_id: WindowId, quadrant: crate::quadrant::Quadrant) {
        // TODO: Update the specific session's coordinate quadrant
        // For now, just log it
        println!("Setting coordinate quadrant to {:?}", quadrant);
    }

    /// Update an editor session with new state
    pub fn update_editor_session(&mut self, window_id: WindowId, session: EditSession) {
        if let Some((_glyph_name, stored_session)) = self.editor_sessions.get_mut(&window_id) {
            *stored_session = Arc::new(session);
            println!("Updated session for window {:?}: selection count = {}", window_id, stored_session.selection.len());
        }
    }
}

/// Implement the Xilem AppState trait
impl xilem::AppState for AppState {
    fn keep_running(&self) -> bool {
        self.running
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
