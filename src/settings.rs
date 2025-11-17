// Copyright 2025 the Runebender Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! Application settings and configuration constants
//!
//! This module contains non-visual settings that are independent of theme changes.
//! Visual styling (colors, sizes) should go in `theme.rs`.

// ============================================================================
// EDITOR SETTINGS
// ============================================================================
/// Minimum zoom level (2% of original size)
const MIN_ZOOM: f64 = 0.02;

/// Maximum zoom level (50x original size)
const MAX_ZOOM: f64 = 50.0;

/// Zoom scale factor for scroll wheel sensitivity
const ZOOM_SCALE: f64 = 0.001;

// ============================================================================
// PERFORMANCE SETTINGS
// ============================================================================
/// Throttle drag updates to every Nth frame to reduce Xilem view rebuilds
///
/// During active drag operations, emitting SessionUpdate on every mouse move
/// causes significant lag because each update triggers a full Xilem view rebuild.
/// By throttling to every Nth frame, we reduce rebuilds while maintaining smooth
/// visual feedback. The main canvas still redraws every frame - only the expensive
/// Xilem rebuild is throttled.
///
/// Higher values = better performance, lower responsiveness
/// Lower values = better responsiveness, worse performance
const DRAG_UPDATE_THROTTLE: u32 = 3;

// ============================================================================
// PUBLIC API - Don't edit below this line
// ============================================================================

/// Editor settings (zoom, viewport, etc.)
pub mod editor {
    /// Minimum zoom level (2% of original size)
    pub const MIN_ZOOM: f64 = super::MIN_ZOOM;

    /// Maximum zoom level (50x original size)
    pub const MAX_ZOOM: f64 = super::MAX_ZOOM;

    /// Zoom scale factor for scroll wheel sensitivity
    #[allow(dead_code)]
    pub const ZOOM_SCALE: f64 = super::ZOOM_SCALE;
}

/// Performance optimization settings
pub mod performance {
    /// Throttle drag updates to every Nth frame
    ///
    /// Set to 1 to disable throttling (updates every frame)
    /// Set to 3 to update every 3rd frame (67% reduction in rebuilds)
    pub const DRAG_UPDATE_THROTTLE: u32 = super::DRAG_UPDATE_THROTTLE;
}
