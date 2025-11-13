# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Runebender Xilem is a font editor built with Xilem, a modern Rust UI framework from the Linebender ecosystem. This is a port of Runebender from Druid to Xilem, leveraging GPU-accelerated rendering via Vello.

**Status**: Very alpha and mostly Claude-generated.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run the application (opens file picker)
cargo run

# Open a specific UFO file
cargo run path/to/font.ufo

# Build for release
cargo build --release

# Run release binary with a file
./target/release/runebender path/to/font.ufo

# Check for compilation errors without building
cargo check

# Run with verbose output
cargo run -- --verbose
```

**Requirements**: Rust 1.88+ (uses 2024 edition features)

## Architecture Overview

### Core Design Pattern: Xilem View Layer

Spoonbender follows Xilem's reactive architecture where the UI is rebuilt from app state on each update. The app uses a single-direction data flow:

```
AppState → app_logic() → View Tree → Masonry Widgets → Vello Rendering
```

### Key Modules

- **src/main.rs**: Entry point, minimal - just calls `spoonbender::run()`
- **src/lib.rs**: Application logic and view construction
  - `app_logic()`: Root view builder that decides between welcome screen and main editor
  - View composition using `flex_col`, `flex_row`, `button`, `label`, `portal` from Xilem
  - Glyph grid rendered as rows of cells (9 columns per row)

- **src/data.rs**: AppState struct and state management
  - `AppState`: Holds workspace, error messages, selected glyph
  - File dialog integration via `rfd` crate
  - Glyph selection and metadata access methods

- **src/workspace.rs**: Font data model and UFO loading
  - `Workspace`: Represents a loaded UFO font with all glyphs and metrics
  - Internal `Glyph`, `Contour`, `ContourPoint` types (thread-safe, owned data)
  - Converts from `norad` (UFO library) types to internal representation
  - Glyphs sorted by Unicode codepoint in `glyph_names()`

- **src/glyph_renderer.rs**: Glyph outline conversion
  - Converts workspace `Glyph` to `kurbo::BezPath`
  - Handles UFO point types: Move, Line, OffCurve, Curve, QCurve
  - Complex curve reconstruction logic for cubic/quadratic beziers
  - Provides `glyph_bounds()` for bounding box calculations

- **src/glyph_widget.rs**: Custom Masonry widget for glyph rendering
  - `GlyphWidget`: Masonry widget that paints glyphs using Vello
  - Uniform scaling based on UPM (units per em) for consistent glyph sizes
  - Y-axis flipping transform (UFO coords are Y-up, screen is Y-down)
  - `GlyphView`: Xilem View wrapper implementing View trait
  - `glyph_view()`: View constructor function used in UI code

- **src/actions.rs**: Action enum (currently minimal/unused)

### Data Flow: UFO Loading

1. User clicks "Open UFO..." → `AppState::open_font_dialog()`
2. `rfd::FileDialog` shows folder picker
3. Selected path → `AppState::load_ufo(path)`
4. `Workspace::load(path)` uses `norad` to parse UFO
5. Convert all glyphs to internal representation
6. `Workspace` stored in `AppState::workspace`
7. `app_logic()` rebuilds UI showing main editor view

### Glyph Rendering Pipeline

1. `glyph_grid_view()` iterates workspace glyphs
2. For each glyph: `glyph_renderer::glyph_to_bezpath()` converts contours to `BezPath`
3. `glyph_cell()` creates a button containing `glyph_view(path, ...)`
4. `GlyphWidget::paint()` applies transform (scale + Y-flip + centering)
5. Vello renders transformed path via `fill_color()`

### UI Layout Structure

```
flex_col
├─ header_bar (font name + action buttons)
├─ selected_glyph_info (horizontal info bar)
└─ glyph_grid_view
   └─ portal (scrollable)
      └─ flex_col (rows)
         └─ flex_row (9 glyphs per row)
            └─ glyph_cell (button with glyph + label)
```

## Key Dependencies

- **Xilem (0.4)**: Reactive UI framework
- **Masonry (0.4)**: Widget system and layout
- **Vello**: GPU rendering (via Masonry)
- **norad (0.13)**: UFO file format parsing
- **kurbo (0.12)**: 2D geometry and bezier paths
- **parley (0.6)**: Text layout (Linebender)
- **peniko (0.5)**: Color and brush types
- **rfd (0.15)**: Native file dialogs

## Important Patterns

### Xilem View Composition
- Views are immutable descriptions of UI, rebuilt on each update
- Use `Either::A`/`Either::B` for conditional views (e.g., welcome vs editor)
- State mutation happens in button callbacks: `button(label, |state: &mut AppState| { ... })`

### Thread-Safety Requirements
- Xilem views must be `Send + Sync` to work with `portal()` (scrolling)
- Internal glyph data (`Workspace`, `Glyph`, etc.) is cloneable and owned (no references)
- Pre-compute data before view construction to avoid capturing mutable state references

### Coordinate System
- UFO coordinates: Y-axis increases upward, origin at baseline
- Screen coordinates: Y-axis increases downward, origin at top-left
- Transformation in `GlyphWidget::paint()` handles Y-flip and baseline positioning

### UPM Scaling
- `units_per_em` (UPM) is the font's design grid size (typically 1000 or 2048)
- All glyphs scaled uniformly by `widget_height / upm` for consistent visual size
- Prevents large glyphs from dominating and tiny glyphs from disappearing

## Development Notes

### Adding New UI Features
- Modify `AppState` in data.rs for new state
- Add methods to `AppState` for state mutations
- Update `app_logic()` or view functions in lib.rs
- Button callbacks can mutate state directly

### Working with Glyphs
- Access glyphs via `workspace.get_glyph(name)` → returns `Option<&Glyph>`
- Glyph names come from `workspace.glyph_names()` (sorted by Unicode)
- Use `glyph_renderer::glyph_to_bezpath()` to get drawable paths
- UFO point types require special handling (see glyph_renderer.rs)

### Font Saving
- Currently unimplemented (`Workspace::save()` returns error)
- Would require converting internal types back to `norad` format

### Custom Widget Reactivity in Multi-Window Apps

When creating custom Masonry widgets that emit actions to update AppState in a multi-window Xilem application, use `MessageResult::Action(())` instead of `MessageResult::RequestRebuild`:

```rust
fn message(
    &self,
    _view_state: &mut Self::ViewState,
    message: &mut MessageContext,
    _element: Mut<'_, Self::Element>,
    app_state: &mut State,
) -> MessageResult<()> {
    match message.take_message::<SessionUpdate>() {
        Some(update) => {
            // Update AppState via callback
            (self.on_session_update)(app_state, update.session);

            // Return Action(()) to propagate to root and trigger full app rebuild
            // MessageResult::RequestRebuild doesn't work for child windows in multi-window apps
            MessageResult::Action(())
        }
        None => MessageResult::Stale,
    }
}
```

**Why this is necessary:**
- In multi-window apps, `MessageResult::RequestRebuild` only rebuilds the current window
- It doesn't trigger `app_logic()` to be called, so other windows won't see state updates
- `MessageResult::Action(())` propagates the action to the root, triggering a full app rebuild
- This causes `app_logic()` to run, recreating all windows with fresh state from AppState

**Data flow pattern:**
1. Custom widget emits action: `ctx.submit_action::<SessionUpdate>(SessionUpdate { session })`
2. View's `message()` method handles action and updates AppState
3. Return `MessageResult::Action(())` to trigger app rebuild
4. Xilem calls `app_logic()` which reads fresh state from AppState
5. All windows recreated with updated data, UI reflects changes

This pattern is essential for reactive UI in multi-window Xilem apps where state changes in one window need to be reflected in others.

### Testing UFO Files
- Project requires valid UFO v3 directory structure
- Use existing font editor UFOs or UFO test files
- Path must exist and be a valid UFO directory
- **Recommended test file**: `~/FG/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo`
  - Run with: `cargo run -- ~/FG/repos/virtua-grotesk/sources/VirtuaGrotesk-Regular.ufo`
