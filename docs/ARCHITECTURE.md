# Spoonbender Architecture

## Overview

Spoonbender is a port of Runebender from Druid to Xilem, maintaining the same feature set while using modern Linebender crates and Xilem's reactive paradigm.

## Key Architectural Differences: Druid vs Xilem

### State Management
- **Druid**: Lens-based immutable data with `Data` trait
- **Xilem**: Direct mutable state in view functions
- **Impact**: Simpler state updates, no need for lens composition

### UI Paradigm
- **Druid**: Imperative widget tree with event handlers
- **Xilem**: Declarative view composition (React/SwiftUI-style)
- **Impact**: Views rebuild on state changes, automatic reactivity

### View Construction
- **Druid**: Widgets created once, updated via `update()` method
- **Xilem**: View functions called on every state change, diffed for minimal updates
- **Impact**: Easier to reason about UI state, but need to handle branching types with `Either`/`OneOf`

## Module Structure

```
src/
├── main.rs          - UI views and application logic
├── data.rs          - Application state (AppState)
├── workspace.rs     - Font/UFO management (Workspace)
└── (future modules for tools, rendering, etc.)
```

## Current UI Structure

Based on Runebender's design, the UI consists of:

### 1. Welcome Screen (No Font Loaded)
- App title
- Error messages (if any)
- "Open UFO..." button
- "New Font" button (not yet implemented)

### 2. Main Editor View (Font Loaded)

#### Header Bar
- Font name display (Family + Style)
- Font Info button (opens metadata editor)

#### Main Content (Horizontal Split)

**Left Sidebar (200px fixed width)**
- Section title: "Selected Glyph"
- Glyph preview area (150x150, placeholder for now)
- Glyph name
- Unicode codepoint
- Advance width

**Right Panel - Glyph Grid**
- Glyph count display
- Scrollable list of glyph buttons
- Each glyph shows its name
- Click to select and update sidebar

## State Flow

```
User Action → State Mutation → View Rebuild → UI Update
```

Example: Clicking a glyph
1. User clicks glyph button
2. Callback mutates `state.selected_glyph`
3. Xilem detects state change
4. `app_logic()` runs, rebuilds views
5. `sidebar_view()` shows updated glyph info
6. UI updates automatically

## Data Model

### AppState
- `workspace: Option<Workspace>` - Loaded font
- `error_message: Option<String>` - UI error display
- `selected_glyph: Option<String>` - Currently selected glyph name

### Workspace
- `font: Font` - Norad's UFO representation
- `path: PathBuf` - UFO directory path
- `family_name: String` - Font family
- `style_name: String` - Font style

## View Composition Pattern

Xilem requires handling different view types carefully:

```rust
// Problem: Different return types
fn app_logic(state: &mut State) -> impl WidgetView<State> {
    if condition {
        view_a(state)  // Type A
    } else {
        view_b(state)  // Type B - ERROR!
    }
}

// Solution: Use Either/OneOf
fn app_logic(state: &mut State) -> impl WidgetView<State> {
    if condition {
        Either::A(view_a(state))
    } else {
        Either::B(view_b(state))
    }
}
```

## Integration with Norad

Norad provides the UFO data structures:
- `Font` - Top-level UFO container
- `Layer` - Glyph layers
- `Glyph` - Individual glyph with contours, components, guides
- `FontInfo` - Font metadata

We wrap this in `Workspace` to provide:
- Convenient accessor methods
- Font metric extraction
- Glyph enumeration
- Error handling

## Next Steps

1. **Grid Layout** - Use proper grid or portal (scrolling) widget
2. **Glyph Rendering** - Draw actual glyph outlines in thumbnails and sidebar
3. **Glyph Editor** - Open detailed view with canvas
4. **Tools** - Selection, Pen, etc.
5. **Undo/Redo** - State history management
6. **Saving** - Write UFO changes to disk

## Rendering Strategy

For glyph rendering, we'll need to:
1. Convert Norad's contour data to Kurbo paths
2. Use Vello (via Xilem) for GPU-accelerated rendering
3. Implement coordinate system conversion (design space → screen space)
4. Handle zooming and panning

This mirrors Runebender's approach but uses Vello instead of Piet/Druid rendering.
