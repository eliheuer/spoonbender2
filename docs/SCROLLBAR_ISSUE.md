# Scrollbar Customization Issue

## Problem

The glyph grid uses Xilem's `portal()` widget for scrolling, which displays a visible scrollbar that:
- Cannot be hidden or made invisible
- Cannot be customized in appearance (color, width, etc.)
- Appears over the content rather than in a dedicated gutter/margin
- Looks visually prominent and can distract from the grid

## Why Can't We Customize It?

Based on research of Xilem/Masonry 0.4 documentation and GitHub issues:

1. **No API for customization**: Neither `portal()` in Xilem nor `Portal` in Masonry expose methods to hide or style scrollbars

2. **Portal widget complexity**: Portal is described as "some of the oldest and most cursed code in Masonry" (issue #1344)

3. **Bespoke implementation**: The scrollbar state management is handled in a custom way, with truth sources stored in child `ScrollBar` widgets, making it hard to customize

4. **Active development**: There's a tracking issue (#1344) to improve Portal/Scroll widget, but scrollbar customization isn't part of the current roadmap

## Current Workaround

We've added 12px margins around the glyph grid:
- Top: 12px
- Left: 12px
- Right: 12px

This gives the scrollbar some breathing room and keeps it from touching the window edge, but it still appears over the grid content.

## Attempted Solutions

### Investigated
- ✗ Hide scrollbar via `portal()` options - no such API exists
- ✗ Style scrollbar colors - no styling API exposed
- ✗ Move scrollbar to margin area - scrollbar is part of Portal widget, can't be repositioned

### Not Attempted (would require significant work)
- Create custom scrolling widget from scratch using Masonry's low-level APIs
- Fork Portal widget and modify scrollbar rendering
- Wait for Masonry improvements to Portal (#1344)

## Related Issues

- [#1344](https://github.com/linebender/xilem/issues/1344) - Tracking issue for Portal/Scroll widget improvements
- [#857](https://github.com/linebender/xilem/issues/857) - Portal scroll bar stays activated when mouse is released outside of window
- [#1402](https://github.com/linebender/xilem/pull/1402) - Fix horizontal scrolling in Portal component

## Recommendation

Accept the default scrollbar appearance for now. This is a framework limitation that affects all Xilem apps using Portal. When Masonry improves the Portal widget (issue #1344), scrollbar customization may become available.

## Files Affected

- `src/lib.rs` - Lines 286-298: Glyph grid with portal and margins

## Date

2025-01-13 - Initial documentation of scrollbar limitation
