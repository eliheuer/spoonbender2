# Spoonbender Development Status

## What Works Now

### ✅ Core Infrastructure
- [x] Xilem 0.3 application framework
- [x] Norad 0.13 UFO file loading
- [x] Modern Linebender crates (Kurbo 0.12, Parley 0.3, Peniko 0.5)
- [x] Native file dialogs (rfd)
- [x] Error handling and display

### ✅ UI Layout (Runebender-compatible)
- [x] Welcome screen with "Open UFO" and "New Font" buttons
- [x] Main editor view (shown when font is loaded)
- [x] Header bar with font name and Font Info button
- [x] **Sidebar** (left, 200px)
  - Selected glyph name
  - Unicode codepoint display
  - Advance width display
  - Preview area (placeholder for glyph rendering)
- [x] **Glyph Grid** (right panel)
  - Glyph count display
  - Vertical list of clickable glyph buttons (100x100px each)
  - Selection interaction

### ✅ State Management
- [x] AppState with workspace, errors, and selected glyph
- [x] Reactive UI updates on state changes
- [x] Font metadata extraction
- [x] Glyph enumeration and access

## Current Limitations

### UI
- Glyph grid shows names only (no glyph outlines yet)
- No scrolling (all glyphs shown in vertical list)
- No proper grid layout (uses flex column instead)
- Sidebar preview is empty (no glyph rendering)
- No selection highlighting

### Functionality
- No glyph editing (read-only for now)
- No UFO saving
- New font creation not implemented
- Font info dialog not implemented

## Next Priorities

### 1. Glyph Rendering (High Priority)
To match Runebender's visual design, we need to render actual glyph outlines.

**Required:**
- Convert Norad `Contour` → Kurbo `BezPath`
- Create custom Vello-based rendering widget
- Implement design space → screen space coordinate conversion
- Draw glyph outlines, components, and guides

**Files to create:**
- `src/glyph_renderer.rs` - Rendering logic
- Custom widget or use Xilem's painting APIs

### 2. Proper Grid Layout (Medium Priority)
- Use Xilem's `grid` widget for proper multi-column layout
- Calculate columns based on window width
- Add `portal` for scrolling when it works with Send bounds
- Or implement custom scrollable grid widget

### 3. Glyph Editor View (High Priority)
Port Runebender's editor canvas:
- Canvas widget with zoom/pan
- Viewport management (ViewPort from Runebender)
- Tool system integration
- Metrics guides display

### 4. Tool System (High Priority)
Port Runebender's 7 tools:
- Select tool (move points, segments, guides)
- Pen tool (draw bezier curves)
- Knife tool (cut paths)
- Ellipse tool
- Rectangle tool
- Measure tool
- Preview tool

### 5. Additional Features
- Undo/redo system
- Copy/paste support
- UFO saving
- Font metadata editing dialog
- Component editing
- Kerning support

## Architecture Decisions

### Xilem Patterns Used
- `Either::A` / `Either::B` for conditional view types
- Separate view functions for composition (`welcome_view`, `main_editor_view`, etc.)
- Direct state mutation in callbacks
- Vec-based dynamic children for glyph lists

### Not Yet Addressed
- Custom widgets (will need for glyph rendering)
- Paint/canvas API (need to investigate Xilem's approach)
- Window management (multiple glyph editors)
- Menus and keyboard shortcuts

## Comparison to Runebender

### What's the Same
- Overall UI layout and structure
- Sidebar with glyph details
- Glyph grid concept
- Font metadata handling via Norad

### What's Different
- Xilem's reactive paradigm vs Druid's imperative widgets
- No lenses - direct state mutation
- View functions rebuild on changes
- Type system requires Either/OneOf for conditional views
- Vello rendering (when implemented) vs Piet

## Testing

### How to Test Current Build

```bash
cargo run --release
```

1. Click "Open UFO..."
2. Select a .ufo directory (e.g., from Google Fonts or Adobe open source fonts)
3. See font name in header
4. See glyph list on right
5. Click glyph buttons
6. Watch sidebar update with glyph details

### Known Issues
- Long glyph lists don't scroll (everything shown)
- No visual feedback on selection
- Window doesn't resize well (fixed layout)

## Files Overview

| File | Purpose | Status |
|------|---------|--------|
| `main.rs` | UI views and application logic | ✅ Core layout done |
| `data.rs` | Application state management | ✅ Complete |
| `workspace.rs` | UFO/Font management via Norad | ✅ Complete |
| `glyph_renderer.rs` | Glyph outline rendering | ❌ Not started |
| `editor.rs` | Glyph editor canvas | ❌ Not started |
| `tools/` | Drawing tools | ❌ Not started |
| `viewport.rs` | Coordinate system conversion | ❌ Not started |

## Code Quality

- ✅ Compiles without errors
- ⚠️  Some unused code warnings (metrics getters)
- ✅ Good separation of concerns
- ✅ Error handling with anyhow
- ✅ Documentation comments
- ❌ No tests yet

## Performance

- Fast compile times (Xilem 0.3 is reasonably quick)
- Rendering not tested yet (no glyph drawing)
- Should handle fonts with 100s of glyphs fine
- 1000s of glyphs may need virtualization

## Blockers

**None currently** - all features are implementable with current architecture.

**Challenges ahead:**
1. Custom rendering in Xilem (need to learn Vello integration)
2. Complex tool state management
3. Undo/redo design for reactive system
4. Multiple windows (if we want multi-glyph editing)
