// Copyright 2025 the Runebender Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Application actions that can be dispatched to modify state

/// Actions that can be performed in the Spoonbender application
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum AppAction {
    /// Open a file dialog to select a UFO directory
    OpenFontDialog,

    /// Create a new empty font
    CreateNewFont,

    /// Select a glyph by name
    SelectGlyph(String),

    /// Open font info dialog
    OpenFontInfo,
}
