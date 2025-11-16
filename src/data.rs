// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Application state and data structures

use crate::edit_session::EditSession;
use crate::workspace::Workspace;
use std::path::PathBuf;
use xilem::WindowId;

/// Which tab is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Tab {
    /// Glyph grid view (font overview)
    GlyphGrid = 0,
    /// Editor view for a specific glyph
    Editor = 1,
}

/// Main application state
pub struct AppState {
    /// The loaded font workspace, if any
    pub workspace: Option<Workspace>,

    /// Error message to display, if any
    pub error_message: Option<String>,

    /// Currently selected glyph name (for showing in grid)
    pub selected_glyph: Option<String>,

    /// Current editor session (when Editor tab is active)
    pub editor_session: Option<EditSession>,

    /// Which tab is currently active
    pub active_tab: Tab,

    /// Whether the app should keep running
    pub running: bool,

    /// Main window ID (stable across rebuilds to prevent window recreation)
    pub main_window_id: WindowId,
}

impl AppState {
    /// Create a new empty application state
    pub fn new() -> Self {
        Self {
            workspace: None,
            error_message: None,
            selected_glyph: None,
            editor_session: None,
            active_tab: Tab::GlyphGrid,
            running: true,
            main_window_id: WindowId::next(),
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
            workspace.path.clone(),
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
        if let Some(session) = self.create_edit_session(&glyph_name) {
            self.editor_session = Some(session);
            self.active_tab = Tab::Editor;
        }
    }

    /// Close the editor and return to glyph grid
    ///
    /// This syncs any final changes to the workspace before closing.
    pub fn close_editor(&mut self) {
        // Sync final changes to workspace before closing
        if let Some(session) = &self.editor_session {
            if let Some(workspace) = &mut self.workspace {
                let updated_glyph = session.to_glyph();
                workspace.update_glyph(&session.glyph_name, updated_glyph.clone());

                // Debug logging only for glyph "a"
                if session.glyph_name == "a" {
                    println!("[close_editor] Synced glyph 'a' with {} contours to workspace", updated_glyph.contours.len());
                }
            }
        }

        self.editor_session = None;
        self.active_tab = Tab::GlyphGrid;
    }

    /// Set the tool for the current editor session
    pub fn set_editor_tool(&mut self, tool_id: crate::tools::ToolId) {
        println!("[AppState::set_editor_tool] Setting tool to {:?}", tool_id);
        if let Some(session) = &mut self.editor_session {
            session.current_tool = crate::tools::ToolBox::for_id(tool_id);
            println!("[AppState::set_editor_tool] Updated session, current_tool is now {:?}", session.current_tool.id());
        }
    }

    /// Update the current editor session with new state
    ///
    /// This also syncs the edited glyph back to the workspace so changes
    /// persist when switching views.
    pub fn update_editor_session(&mut self, session: EditSession) {
        // Sync edited glyph back to workspace
        if let Some(workspace) = &mut self.workspace {
            let updated_glyph = session.to_glyph();

            // Debug logging only for glyph "a"
            if session.glyph_name == "a" {
                println!("[update_editor_session] Syncing glyph 'a' with {} contours back to workspace", updated_glyph.contours.len());
            }

            workspace.update_glyph(&session.glyph_name, updated_glyph.clone());

            // Verify the update worked (only for "a")
            if session.glyph_name == "a" {
                if let Some(glyph_from_workspace) = workspace.get_glyph(&session.glyph_name) {
                    println!("[update_editor_session] Verified: workspace now has glyph 'a' with {} contours", glyph_from_workspace.contours.len());
                }
            }
        }

        self.editor_session = Some(session);
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
