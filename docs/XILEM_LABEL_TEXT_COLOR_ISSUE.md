# Xilem 0.4 Label Text Color Issue

## Summary

Xilem 0.4's `Label` view does not provide a way to customize text color, making it impossible to implement consistent theming for UI text elements across an application.

## Problem Description

### Current Behavior

The `xilem::view::Label` struct in Xilem 0.4 does not expose any method to set text color. The available methods are:
- `.text_size(f32)` - Sets font size
- `.text_alignment(Alignment)` - Sets text alignment
- No `.text_color()`, `.brush()`, `.text_brush()`, or similar method exists

### Expected Behavior

Labels should support customizable text color, similar to how other UI properties can be customized. Ideally through a method like:
```rust
label("Some text")
    .text_size(14.0)
    .text_color(Color::from_rgb8(200, 200, 200))
```

Or alternatively:
```rust
label("Some text")
    .text_size(14.0)
    .brush(Brush::Solid(Color::from_rgb8(200, 200, 200)))
```

## Impact on Application Development

### Use Case: Theme System

In our font editor application (Spoonbender), we need consistent text colors across:
- Glyph grid labels (showing glyph names)
- Coordinate pane labels (showing x, y, w, h values)
- Other UI text elements

We have a centralized theme system (`src/theme.rs`) that defines color constants:

```rust
// theme.rs
const PRIMARY_UI_TEXT: Color = Color::from_rgb8(200, 200, 200);  // Light gray

pub mod coord_pane {
    pub const TEXT: Color = super::PRIMARY_UI_TEXT;
}
```

However, we cannot apply these colors to Xilem labels, resulting in inconsistent theming.

### Attempted Workarounds

1. **Attempted `.text_color()` method** - Does not exist
   ```rust
   // This fails to compile:
   label("x: 100").text_color(theme::coord_pane::TEXT)
   ```

   **Error:**
   ```
   error[E0599]: no method named `text_color` found for struct `xilem::view::Label`
   ```

2. **Attempted `.brush()` method** - Does not exist
   ```rust
   // This also fails:
   label("x: 100").brush(Brush::Solid(theme::coord_pane::TEXT))
   ```

3. **Custom widget wrapper** - Possible but defeats the purpose of Xilem's declarative API
   - Would require creating a custom widget that wraps Masonry's Label widget
   - Would lose Xilem's reactivity benefits
   - Adds significant boilerplate

## Technical Details

### Xilem Version
- **Package:** `xilem = "0.4.0"`
- **Masonry Version:** `masonry = "0.4.0"`

### Code Location

The issue is in the `xilem::view::Label` implementation. Looking at the available methods:

```rust
// From xilem 0.4 API
pub struct Label<State, Action> { /* ... */ }

impl<State, Action> Label<State, Action> {
    pub fn text_size(self, size: f32) -> Self { /* ... */ }
    pub fn text_alignment(self, alignment: Alignment) -> Self { /* ... */ }
    // No text_color or brush method
}
```

### Underlying Widget Support

Masonry's `Label` widget (the underlying widget used by Xilem's Label view) **does** support text color through the `TextBrush` property. The limitation is purely in the Xilem View API wrapper.

## Proposed Solution

Add a method to `xilem::view::Label` that allows setting text color/brush:

### Option 1: Simple Color Method
```rust
impl<State, Action> Label<State, Action> {
    pub fn text_color(self, color: Color) -> Self {
        // Set text brush to solid color
    }
}
```

### Option 2: Full Brush Support
```rust
use masonry::text::TextBrush;

impl<State, Action> Label<State, Action> {
    pub fn text_brush(self, brush: impl Into<TextBrush>) -> Self {
        // Set text brush (supports solid colors, gradients, etc.)
    }
}
```

### Option 3: Both Methods
Provide both for convenience:
```rust
impl<State, Action> Label<State, Action> {
    /// Convenience method for solid color text
    pub fn text_color(self, color: Color) -> Self {
        self.text_brush(TextBrush::Solid(color))
    }

    /// Full brush control (gradients, patterns, etc.)
    pub fn text_brush(self, brush: impl Into<TextBrush>) -> Self {
        // Set text brush
    }
}
```

## Workaround Currently Used

We have removed text color customization from our theme system and accepted the default text color provided by the system/Masonry. This is not ideal for our dark-themed font editor.

## Related Xilem Views

Other Xilem views that might have similar limitations:
- `Button` - Does button text support color customization?
- Other text-containing views

## References

- **Project:** Spoonbender (font editor built with Xilem 0.4)
- **Theme file:** `src/theme.rs`
- **Affected code:** `src/lib.rs` (coordinate_info_pane_reactive function, lines 446-455)
- **Build error example:** See compilation errors in project build logs

## Additional Context

This limitation makes it difficult to build applications with custom themes or dark mode support in Xilem 0.4. While Masonry (the underlying widget toolkit) supports text color customization, the Xilem declarative API wrapper does not expose this functionality.

For a UI framework aiming to provide a modern, declarative API, text color customization is a fundamental requirement that should be available alongside other text properties like size and alignment.
