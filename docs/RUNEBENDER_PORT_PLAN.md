# Runebender to Xilem Port Plan

This document outlines the complete plan for porting Runebender from Druid to Xilem (Spoonbender).

## Status: In Progress

Last Updated: 2025-11-10
**Current Phase:** Phase 8 (UI Panels and Polish)
**Completed Phases:** 1-7 (Basic feature-complete editor)

---

## Architecture Overview

Runebender is a UFO-based font editor with the following major components:

1. **Main Window**: Glyph grid showing all glyphs in the font
2. **Editor Windows**: Individual glyph editing canvases with tools
3. **Data Model**: Path/point representation with undo/redo
4. **Tools System**: Multiple editing tools (Select, Pen, Rectangle, etc.)
5. **UI Panels**: Toolbar, coordinate panel, glyph metrics panel
6. **State Management**: Arc-based immutable state with efficient cloning

---

## Phase 1: Core Data Structures ✅ COMPLETE

### Completed
- [x] `EditSession` - Session management
- [x] `ViewPort` - Coordinate transformation (design space ↔ screen space)
- [x] Basic `EditorWidget` - Canvas rendering
- [x] Multi-window support
- [x] Window lifecycle management
- [x] `Path` / `CubicPath` - Bezier path representation
- [x] `PathPoints` - Point storage and management
- [x] `PathPoint` / `PointType` - Point types (OnCurve, OffCurve)
- [x] `Selection` - Selection management (BTreeSet of EntityIds)
- [x] `EntityId` - Unique identifiers for points/paths/components

### Details

**Path Representation**
```rust
// src/path.rs
pub enum Path {
    Cubic(CubicPath),
    // Hyper can be deferred
}

// src/cubic_path.rs
pub struct CubicPath {
    points: PathPoints,
    closed: bool,
}

// src/point_list.rs
pub struct PathPoints {
    points: Arc<Vec<PathPoint>>,
}

// src/point.rs
pub struct PathPoint {
    id: EntityId,
    point: DPoint,  // (x, y) in design space
    typ: PointType,
}

pub enum PointType {
    OnCurve { smooth: bool },
    OffCurve { auto: bool },
}
```

**Selection**
```rust
// src/selection.rs
pub struct Selection {
    inner: Arc<BTreeSet<EntityId>>,
}
```

---

## Phase 2: Basic Editor Rendering ✅ COMPLETE

### Goal
Display glyph outline with proper rendering of paths, points, and handles.

### Completed Tasks
- [x] Update `EditSession` to store `Vec<Path>` instead of raw `Glyph`
- [x] Convert `Glyph` contours to internal `Path` representation on session creation
- [x] Implement `Path::to_bezpath()` for rendering
- [x] Draw control point lines (off-curve to on-curve connections)
- [x] Draw points with proper styling:
  - [x] Smooth points as circles
  - [x] Corner points as squares
  - [x] Off-curve points as smaller circles
  - [x] Selected vs unselected colors
- [x] Improve metrics guide rendering
- [ ] Add grid rendering (visible at high zoom) - DEFERRED

### Files Modified
- `src/edit_session.rs` - Added path storage
- `src/editor_widget.rs` - Enhanced rendering
- `src/theme.rs` - Color/size constants

---

## Phase 3: Mouse Handling & Tool Infrastructure ✅ COMPLETE

### Goal
Create the foundation for interactive editing tools.

### Completed Tasks
- [x] Create `Mouse` state machine
  - [x] Track button states
  - [x] Detect drag gestures
  - [x] Calculate drag deltas
- [x] Create `MouseDelegate` trait
  - [x] `left_down()`, `left_up()`, `left_click()`
  - [x] `left_drag_began()`, `left_drag_changed()`, `left_drag_ended()`
  - [x] `cancel()`
- [x] Create `Tool` trait/enum
  - [x] `paint()` - Draw tool overlays
  - [x] `edit_type()` - For undo grouping
- [x] Integrate mouse handling into `EditorWidget`
- [ ] Add tool switching via keyboard shortcuts - DEFERRED

### Files Created
- `src/mouse.rs` - Mouse abstraction
- `src/tools/mod.rs` - Tool trait and registry
- `src/edit_type.rs` - Edit type enum for undo

---

## Phase 4: Selection Tool ✅ COMPLETE

### Goal
Implement the select tool for basic interaction.

### Completed Tasks
- [x] Point selection
  - [x] Click to select single point
  - [x] Shift+click to toggle selection (multi-select)
  - [ ] Rectangular selection (drag) - DEFERRED
  - [ ] Tab to cycle through selectable items - DEFERRED
- [x] Hit testing
  - [x] `hit_test_point()` - Find point under cursor
  - [x] Hit test with radius tolerance
  - [x] Penalty system (favors off-curve point selection)
  - [ ] `hit_test_segment()` - Find segment under cursor - DEFERRED
  - [ ] `hit_test_guide()` - Find guide under cursor - DEFERRED
- [x] Moving selected items
  - [x] Drag to move selected points
  - [x] Update path points using Arc::make_mut
  - [x] Visual feedback for selection (blue highlights, larger size)
  - [ ] Shift+drag for axis-lock - DEFERRED
- [x] Keyboard nudging
  - [x] Arrow keys: nudge 1 unit
  - [x] Shift+arrow: nudge 10 units
  - [x] Cmd/Ctrl+arrow: nudge 100 units
- [ ] Point type toggling - DEFERRED
  - [ ] Double-click to toggle smooth/corner

### Files Created
- `src/tools/select.rs` - Select tool implementation
- `src/hit_test.rs` - Hit testing utilities

### Files Modified
- `src/edit_session.rs` - Added `hit_test_point()`, `move_selection()`, and `nudge_selection()` methods
- `src/cubic_path.rs` - Added `points()` accessor method
- `src/mouse.rs` - Added `Modifiers` struct for tracking shift/ctrl/alt/meta keys
- `src/editor_widget.rs` - Added `on_pointer_event()` and `on_text_event()` handlers
- `src/tools/mod.rs` - Implemented `MouseDelegate` for `ToolBox` with `left_click()` support

---

## Phase 5: Undo/Redo System

### Goal
Support undo/redo for all editing operations.

### Tasks
- [ ] Create `UndoState<T>` generic wrapper
  - [ ] Deque-based history (max 128 entries)
  - [ ] `undo()` / `redo()`
  - [ ] `add_undo_group()` - Add new state
  - [ ] `update_current_undo()` - Modify current without new entry
- [ ] Add undo state to `EditorWidget`
- [ ] Implement undo grouping logic
  - [ ] Group consecutive edits of same type
  - [ ] New group on edit type change
- [ ] Add keyboard shortcuts (Cmd+Z, Cmd+Shift+Z)
- [ ] Store entire `EditSession` clones (cheap due to Arc)

### Files to Create
- `src/undo.rs` - Undo system

### Files to Modify
- `src/editor_widget.rs` - Integrate undo state
- `src/edit_session.rs` - Make fully cloneable

---

## Phase 6: Path Editing Operations

### Goal
Support basic path manipulation.

### Tasks
- [ ] `EditSession::nudge_selection()` - Move selected points
- [ ] `EditSession::delete_selection()` - Remove selected points/segments
- [ ] `EditSession::split_segment()` - Split segment at point
- [ ] `EditSession::toggle_point_type()` - Switch smooth/corner
- [ ] `EditSession::reverse_contours()` - Reverse path direction
- [ ] Path mutation helpers
  - [ ] `paths_mut()` - Get mutable access with Arc::make_mut
  - [ ] Automatic change tracking
- [ ] Convert back to Norad glyph for saving
  - [ ] `EditSession::to_norad_glyph()`

### Files to Modify
- `src/edit_session.rs` - Add editing operations
- `src/cubic_path.rs` - Path manipulation methods
- `src/point_list.rs` - Point collection operations

---

## Phase 7: Pen Tool

### Goal
Draw new paths and add points to existing paths.

### Tasks
- [ ] State machine
  - [ ] Ready - No active drawing
  - [ ] AddPoint - Clicked, waiting for drag
  - [ ] DragHandle - Dragging handle position
- [ ] Point addition
  - [ ] Click empty space: start new path
  - [ ] Click on segment: split and add point
  - [ ] Click path start: close path
  - [ ] Alt+click when closing: smooth closure
- [ ] Curve creation
  - [ ] Click and drag: add point with handles
  - [ ] Shift+drag: axis-lock handle
  - [ ] Automatically upgrade lines to curves
- [ ] Visual feedback
  - [ ] Preview of new point
  - [ ] Dragging handle indicator
  - [ ] Path closing indicator

### Files to Create
- `src/tools/pen.rs` - Pen tool implementation

---

## Phase 8: UI Panels

### Goal
Add auxiliary UI panels for editing context.

### Tasks

#### Toolbar (`src/widgets/toolbar.rs`)
- [ ] Create toolbar widget
- [ ] Tool icons (48x48px squares)
- [ ] Tools: Select (V), Pen (P), Preview (H), Knife (E), Rectangle (U), Ellipse (Shift+U), Measure (M)
- [ ] Selected tool highlighting
- [ ] Keyboard shortcuts
- [ ] Position: floating top-left

#### Coordinate Panel (`src/widgets/coord_pane.rs`)
- [ ] Show when points selected
- [ ] Display x, y coordinates
- [ ] Multi-selection: show w, h (bounding box)
- [ ] Quadrant picker for multi-selection anchor
- [ ] Editable labels (convert input to nudge/scale)
- [ ] Position: floating bottom-right

#### Glyph Metrics Panel (`src/widgets/glyph_pane.rs`)
- [ ] GlyphPainter - small preview (128px)
- [ ] Left sidebearing (editable)
- [ ] Right sidebearing (editable)
- [ ] Advance width (editable)
- [ ] Position: floating bottom-left

#### Layout Controller (`src/widgets/controller.rs`)
- [ ] Coordinate floating panel positions
- [ ] Intercept keyboard shortcuts
- [ ] Manage focus between panels
- [ ] Wrap ScrollZoom + Editor + Panels

### Files to Create
- `src/widgets/toolbar.rs`
- `src/widgets/coord_pane.rs`
- `src/widgets/glyph_pane.rs`
- `src/widgets/controller.rs`

---

## Phase 9: Scroll & Zoom

### Goal
Pan and zoom the editor canvas.

### Tasks
- [ ] Create `ScrollZoom` widget wrapper
- [ ] Wheel zoom
  - [ ] Zoom centered on mouse position
  - [ ] Constrain between 0.02x and 50x
- [ ] Pinch zoom (trackpad)
- [ ] Pan (space+drag or middle-click drag)
- [ ] Zoom commands
  - [ ] Zoom in (Cmd+Plus)
  - [ ] Zoom out (Cmd+Minus)
  - [ ] Zoom to fit (Cmd+0)
- [ ] Update `ViewPort` in `EditSession`

### Files to Create
- `src/widgets/scroll_zoom.rs` - Scroll/zoom wrapper

---

## Phase 10: Additional Tools

### Goal
Implement remaining drawing tools.

### Tasks

#### Preview Tool (`src/tools/preview.rs`)
- [ ] View-only mode
- [ ] Spacebar for temporary toggle
- [ ] Hide points and handles
- [ ] Show filled paths

#### Rectangle Tool (`src/tools/rectangle.rs`)
- [ ] Click and drag to create rectangle
- [ ] Shift+drag for square
- [ ] Convert to path on completion

#### Ellipse Tool (`src/tools/ellipse.rs`)
- [ ] Click and drag to create ellipse
- [ ] Shift+drag for circle
- [ ] Convert to path on completion

#### Knife Tool (`src/tools/knife.rs`)
- [ ] Draw cutting line
- [ ] Split paths at intersection
- [ ] Visual feedback while drawing

#### Measure Tool (`src/tools/measure.rs`)
- [ ] Click two points to measure distance
- [ ] Display measurement overlay

### Files to Create
- `src/tools/preview.rs`
- `src/tools/rectangle.rs`
- `src/tools/ellipse.rs`
- `src/tools/knife.rs`
- `src/tools/measure.rs`

---

## Phase 11: Guides

### Goal
Add alignment guides.

### Tasks
- [ ] Create `Guide` enum
  - [ ] Horizontal (constant y)
  - [ ] Vertical (constant x)
  - [ ] Angled (two points)
- [ ] Add guides to `EditSession`
- [ ] Render guides in editor
- [ ] Guide manipulation
  - [ ] Add new guide
  - [ ] Move guide (drag)
  - [ ] Delete guide
  - [ ] Toggle orientation (double-click)
- [ ] Guide snapping
  - [ ] Snap points to guides when near
  - [ ] Visual snap indicator
- [ ] Serialize guides to/from UFO

### Files to Create
- `src/guides.rs` - Guide types and operations

---

## Phase 12: Components

### Goal
Support composite glyphs.

### Tasks
- [ ] Create `Component` struct
  - [ ] Base glyph name
  - [ ] Affine transform
  - [ ] Unique ID
- [ ] Add components to `EditSession`
- [ ] Render components
  - [ ] Fetch base glyph outline from workspace
  - [ ] Apply transform
  - [ ] Fill with component color
- [ ] Component selection
  - [ ] Select component as unit
  - [ ] Move component (translate transform)
- [ ] Component cache invalidation
  - [ ] When base glyph changes
  - [ ] Update all dependents

### Files to Create
- `src/component.rs` - Component type

### Files to Modify
- `src/edit_session.rs` - Add components field
- `src/workspace.rs` - Track component dependencies

---

## Phase 13: Bezier Cache

### Goal
Optimize glyph outline rendering.

### Tasks
- [ ] Create `BezCache`
  - [ ] Three-level cache: PreCache (8 LRU), main HashMap, ComponentMap
  - [ ] Cache key: glyph name + modification counter
- [ ] Automatic invalidation
  - [ ] When glyph edited
  - [ ] When component base changes
  - [ ] Cascade to component users
- [ ] Integrate with workspace
- [ ] Use cache in glyph grid rendering
- [ ] Use cache in component rendering

### Files to Create
- `src/bez_cache.rs` - Bezier cache

---

## Phase 14: Clipboard Operations

### Goal
Copy/paste paths and points.

### Tasks
- [ ] Copy (Cmd+C)
  - [ ] Multiple formats: JSON, Glyphs plist, PDF, SVG
  - [ ] Use `GLYPHS_APP_PASTEBOARD_TYPE` for cross-app compatibility
- [ ] Paste (Cmd+V)
  - [ ] Auto-detect format
  - [ ] Paste as new paths
  - [ ] Offset pasted content if overlapping
- [ ] Cut (Cmd+X)
- [ ] Duplicate (Cmd+D)

### Files to Create
- `src/clipboard.rs` - Clipboard utilities

---

## Phase 15: Advanced Operations

### Goal
Implement advanced editing features.

### Tasks
- [ ] Alignment operations
  - [ ] Align left/center/right
  - [ ] Align top/middle/bottom
  - [ ] Distribute horizontally/vertically
- [ ] Transform operations
  - [ ] Scale around anchor
  - [ ] Rotate around anchor
  - [ ] Skew
- [ ] Path operations
  - [ ] Combine paths
  - [ ] Break apart
  - [ ] Add overlap / Remove overlap
- [ ] Sidebearing adjustments
  - [ ] Edit via coordinate panel
  - [ ] Commands to adjust

### Files to Create
- `src/operations.rs` - Advanced operations

---

## Phase 16: Font-Level Features

### Goal
Handle font-wide operations.

### Tasks
- [ ] Font Info dialog
  - [ ] Edit family name, style name
  - [ ] Edit UPM, metrics (ascender, descender, x-height, cap-height)
  - [ ] Copyright, designer, etc.
- [ ] New glyph creation
- [ ] Glyph deletion
- [ ] Glyph renaming
- [ ] Unicode assignment
- [ ] Glyph import/export
- [ ] Save UFO
  - [ ] Convert all sessions back to Norad format
  - [ ] Write to disk
  - [ ] Handle dirty state tracking

### Files to Create
- `src/widgets/font_info_dialog.rs`
- `src/widgets/glyph_dialogs.rs`

### Files to Modify
- `src/workspace.rs` - Implement save functionality

---

## Phase 17: Theme System

### Goal
Customizable visual appearance.

### Tasks
- [ ] Create theme structure
  - [ ] Colors: UI, selection, paths, points, metrics, guides, components
  - [ ] Sizes: point radii, stroke widths
  - [ ] Fonts: detail font for overlays
- [ ] Theme file format (JSON or similar)
- [ ] Runtime theme loading
- [ ] Environment variable configuration
- [ ] Default theme (match Runebender)

### Files to Create
- `src/theme.rs` - Theme system
- `resources/default_theme.json` - Default theme

---

## Phase 18: Polish & Bug Fixes

### Goal
Refinement and stability.

### Tasks
- [ ] Error handling improvements
- [ ] Input validation
- [ ] Edge case handling
  - [ ] Empty glyphs
  - [ ] Degenerate paths
  - [ ] Invalid transforms
- [ ] Performance optimization
  - [ ] Rendering optimizations
  - [ ] Large font handling
  - [ ] Memory usage
- [ ] Keyboard shortcut documentation
- [ ] Help system
- [ ] User preferences
  - [ ] Auto-save interval
  - [ ] Grid settings
  - [ ] Default tool

---

## Phase 19: Testing & Documentation

### Goal
Ensure quality and usability.

### Tasks
- [ ] Unit tests
  - [ ] Path operations
  - [ ] Coordinate transformations
  - [ ] Hit testing
  - [ ] Undo/redo
- [ ] Integration tests
  - [ ] UFO loading/saving
  - [ ] Multi-window behavior
  - [ ] Component dependencies
- [ ] Update CLAUDE.md
- [ ] User documentation
  - [ ] Tool usage
  - [ ] Keyboard shortcuts
  - [ ] Workflow examples
- [ ] README updates
  - [ ] Feature list
  - [ ] Screenshots
  - [ ] Installation instructions

---

## Deferred Features

These Runebender features can be implemented later:

- [ ] HyperPen / HyperPath (experimental bezier format)
- [ ] Multiple workspaces
- [ ] Advanced kerning/spacing features
- [ ] OpenType feature editing
- [ ] Python scripting integration
- [ ] Plugin system

---

## Dependencies to Add

```toml
[dependencies]
# Existing dependencies remain

# May need to add:
# clipboard = "0.5"  # For clipboard operations
# serde_json = "1.0" # Already have for theme files
# image = "0.25"     # For icon rendering (if needed)
```

---

## Success Criteria

### MVP (Minimum Viable Product)
- [ ] Load and display UFO fonts
- [ ] Edit glyph outlines with Select and Pen tools
- [ ] Undo/redo support
- [ ] Save changes back to UFO
- [ ] Multiple glyph editors open simultaneously

### Feature Complete
- [ ] All Runebender tools implemented
- [ ] All UI panels functional
- [ ] Clipboard operations
- [ ] Guide and component support
- [ ] Font info editing
- [ ] Stable and performant

---

## Current Status

**Phase 1**: ✅ Partially complete
- EditSession, ViewPort, EditorWidget, multi-window support complete
- Path data structures pending

**Phase 2**: 🔄 Next phase
- Basic rendering improvements needed

**Overall Progress**: ~10% complete

---

## Notes

### Xilem-Specific Considerations

1. **State Management**: Use Arc for efficient cloning, follow Xilem's immutable data patterns
2. **Widget Composition**: Build UI declaratively, avoid imperative mutations
3. **Event Handling**: Adapt Druid event model to Xilem's view updates
4. **Custom Widgets**: May need custom Masonry widgets for editor canvas
5. **Performance**: Xilem rebuilds views frequently - ensure path pre-computation and caching

### Differences from Runebender

1. **Framework**: Druid → Xilem (reactive architecture)
2. **Rendering**: Piet → Vello (GPU-accelerated)
3. **State**: More explicit Arc usage for Xilem's patterns
4. **Events**: Less direct event handling, more state-driven updates

### Key Files from Runebender to Reference

- `runebender-lib/src/edit_session.rs` - Core editing state
- `runebender-lib/src/editor.rs` - Main editor widget
- `runebender-lib/src/path.rs` - Path abstraction
- `runebender-lib/src/cubic_path.rs` - Cubic bezier paths
- `runebender-lib/src/tools/*.rs` - Tool implementations
- `runebender-lib/src/mouse.rs` - Mouse handling
- `runebender-lib/src/undo.rs` - Undo system
- `runebender-lib/src/widgets/*.rs` - UI widgets
