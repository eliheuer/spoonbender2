# Scrollbar Customization in Xilem Portal Widget

## Summary

The `portal` widget in Xilem 0.4 doesn't expose APIs to customize scrollbar appearance, positioning, or styling. This limits the ability to create polished UIs where scrollbar appearance needs to match the application's design system.

## Use Case

In Spoonbender (a font editor), we have a scrollable glyph grid with the following requirements:

1. **Visual consistency**: Glyphs are displayed in cells with dark gray backgrounds (RGB: 50, 50, 50) and mid-gray outlines (RGB: 100, 100, 100)
2. **Scrollbar overlap issue**: The scrollbar currently overlays the rightmost column of glyphs, obscuring them
3. **Visual harmony**: The default scrollbar appearance doesn't match the application's color scheme

## Current Limitations

The `portal` widget doesn't provide methods to:

- **Adjust scrollbar width** (e.g., `.scrollbar_width(value)`)
- **Change scrollbar colors** (e.g., `.scrollbar_color(color)` or `.scrollbar_thumb_color(color)`)
- **Position the scrollbar** (e.g., force it to not overlay content)
- **Hide the scrollbar visually** while keeping scroll functionality (e.g., `.hide_scrollbar(true)`)
- **Style the scrollbar track/thumb separately**

## Current Code Example

```rust
// Current implementation - no customization available
portal(flex_col(rows_of_cells))
```

## Desired API

Here are some potential API designs that would solve this issue:

### Option 1: Style Methods on Portal

```rust
portal(flex_col(rows_of_cells))
    .scrollbar_width(8.0)
    .scrollbar_thumb_color(Color::from_rgb8(100, 100, 100))
    .scrollbar_track_color(Color::from_rgb8(50, 50, 50))
    .hide_scrollbar(false)  // or true to hide visually but keep functionality
```

### Option 2: Scrollbar Configuration Struct

```rust
portal(flex_col(rows_of_cells))
    .scrollbar_style(ScrollbarStyle {
        width: 8.0,
        thumb_color: Color::from_rgb8(100, 100, 100),
        track_color: Color::from_rgb8(50, 50, 50),
        visible: true,
    })
```

### Option 3: Separate Styling via Style Trait

```rust
portal(flex_col(rows_of_cells))
    .scrollbar_thumb_color(Color::from_rgb8(100, 100, 100))
    .scrollbar_track_color(Color::from_rgb8(50, 50, 50))
```

## Technical Background

The scrollbar styling is currently handled at the Masonry widget layer (the `Portal` widget implementation), and these properties aren't exposed through Xilem's declarative view layer.

To implement this feature, we would need:

1. **Masonry level**: The underlying `Portal` widget to support customizable scrollbar styling
2. **Xilem level**: View methods that pass these style properties down to the Masonry widget

## Workarounds Attempted

We tried the following approaches, none of which work with the current API:

```rust
// ❌ These methods don't exist
portal(content).scrollbar_width(0.0)
portal(content).scrollbar_color(Color::TRANSPARENT)
portal(content).hide_scrollbar()

// ❌ Can't add padding to prevent overlap
// (insets API not available at this level)
portal(sized_box(content).padding(...))

// ❌ Styling on the portal wrapper doesn't affect the scrollbar
sized_box(portal(content))
    .background_color(...)  // This doesn't style the scrollbar
```

## Similar Features in Other UI Frameworks

For reference, here's how other UI frameworks handle this:

- **Flutter**: `Scrollbar` widget with `thickness`, `thumbColor`, `trackColor`, etc.
- **SwiftUI**: `scrollIndicators(.hidden)` modifier
- **HTML/CSS**: `::-webkit-scrollbar` pseudo-elements for full customization
- **Druid** (Xilem's predecessor): Scrollbar styling was configurable through theme values

## Impact

This limitation affects:

- **Visual consistency**: Can't match scrollbars to application design
- **Content visibility**: Scrollbar can overlay important content
- **UX polish**: Can't create seamless scrolling experiences
- **Accessibility**: May need to hide scrollbar but keep functionality for touch/trackpad users

## Proposed Solution

Add scrollbar styling support to the `portal` widget, following Xilem's existing style pattern (similar to how `background_color`, `border_color`, etc. work via the `Style` trait).

Minimum viable API:
- `.scrollbar_thumb_color(Color)` - Color of the draggable thumb
- `.scrollbar_track_color(Color)` - Color of the track
- `.scrollbar_width(f64)` - Width of the scrollbar (0.0 to hide)

## Questions for Maintainers

1. Is scrollbar customization planned for a future Xilem release?
2. Would you be open to a PR implementing this feature?
3. Are there any architectural concerns with exposing scrollbar styling at the view level?
4. Should scrollbar styling follow the existing `Style` trait pattern, or use a different approach?

## Related Context

- **Project**: Spoonbender - Font editor built with Xilem
- **Xilem Version**: 0.4.0
- **Masonry Version**: (whatever version Xilem 0.4 uses)
- **Platform**: Linux (but affects all platforms)

## Additional Notes

This is a common UI customization need that would benefit many Xilem applications. The ability to style scrollbars is essential for creating polished, professional applications with consistent visual design.
