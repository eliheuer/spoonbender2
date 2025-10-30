// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Font workspace management - handles UFO loading and glyph access

use anyhow::{Context, Result};
use norad::{Font, Glyph};
use std::path::{Path, PathBuf};

/// A workspace represents a loaded UFO font with all its glyphs and metadata
#[derive(Debug)]
pub struct Workspace {
    /// The loaded UFO font
    pub font: Font,

    /// Path to the UFO directory
    pub path: PathBuf,

    /// Name of the font family
    pub family_name: String,

    /// Style name (e.g., "Regular", "Bold")
    pub style_name: String,
}

impl Workspace {
    /// Load a UFO from a directory path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Load the UFO using norad
        let font = Font::load(path)
            .with_context(|| format!("Failed to load UFO from {:?}", path))?;

        // Extract font metadata
        let family_name = font
            .font_info
            .family_name
            .clone()
            .unwrap_or_else(|| "Untitled Font".to_string());

        let style_name = font
            .font_info
            .style_name
            .clone()
            .unwrap_or_else(|| "Regular".to_string());

        Ok(Self {
            font,
            path: path.to_path_buf(),
            family_name,
            style_name,
        })
    }

    /// Get the display name of the font (Family + Style)
    pub fn display_name(&self) -> String {
        format!("{} {}", self.family_name, self.style_name)
    }

    /// Get the number of glyphs in the default layer
    pub fn glyph_count(&self) -> usize {
        self.font.default_layer().len()
    }

    /// Get a list of all glyph names in the default layer
    pub fn glyph_names(&self) -> Vec<String> {
        self.font
            .default_layer()
            .iter()
            .map(|g| g.name().to_string())
            .collect()
    }

    /// Get a glyph by name from the default layer
    pub fn get_glyph(&self, name: &str) -> Option<&Glyph> {
        self.font.default_layer().get_glyph(name)
    }

    /// Get font metrics
    pub fn units_per_em(&self) -> Option<f64> {
        self.font.font_info.units_per_em.map(|n| n.as_f64())
    }

    pub fn ascender(&self) -> Option<f64> {
        self.font.font_info.ascender
    }

    pub fn descender(&self) -> Option<f64> {
        self.font.font_info.descender
    }

    pub fn x_height(&self) -> Option<f64> {
        self.font.font_info.x_height
    }

    pub fn cap_height(&self) -> Option<f64> {
        self.font.font_info.cap_height
    }

    /// Save the UFO back to disk
    pub fn save(&self) -> Result<()> {
        self.font
            .save(&self.path)
            .with_context(|| format!("Failed to save UFO to {:?}", self.path))
    }
}
