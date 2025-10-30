# Xilem 0.4.0 Documentation Index

Complete API reference and migration guide for Xilem 0.4.0 (current version)

Generated: October 30, 2025  
Repository: https://github.com/linebender/xilem  
Current Version: 0.4.0  
Rust Edition: 2024  
Minimum Rust: 1.88

---

## Quick Navigation

### For Immediate Use
Start here if you just need to fix compilation errors or understand the API quickly:

1. **XILEM_0_4_QUICK_REFERENCE.md** - One-page reference card with all common patterns
   - Essential imports
   - Application setup
   - Layout system cheat sheet
   - Common patterns
   - Compiler error fixes

2. **XILEM_0_4_SUMMARY.txt** - Plain text reference for terminal viewing
   - Key answers to your questions
   - API patterns cheat sheet
   - Common mistakes and fixes
   - Minimal complete example

### For In-Depth Learning
Read these for comprehensive understanding:

3. **XILEM_0_4_API_GUIDE.md** - Complete API documentation
   - Detailed explanations of all major features
   - Code examples for each concept
   - Working patterns and best practices
   - Explains the "why" behind API choices

4. **XILEM_0_3_VS_0_4_COMPARISON.md** - Migration guide
   - Side-by-side API comparisons
   - Breaking changes and why they happened
   - Migration checklist
   - Performance notes

### Working Code
5. **XILEM_0_4_EXAMPLE.rs** - Complete working application
   - Full counter app with multiple features
   - Demonstrates best practices
   - Can be copied into a Cargo project
   - Heavily commented

---

## Core Answers to Your Questions

### Question 1: How to run applications in 0.4?

**Quick Answer:** Use `Xilem::new_simple(state, app_logic, WindowOptions::new("Title"))` and call `.run_in(EventLoop::with_user_event())?`

**Details:**
- Single window apps: `Xilem::new_simple()` - auto-exits on window close
- Multi-window apps: `Xilem::new()` - requires `AppState` trait implementation
- Always call `.run_in(EventLoop)` not `.run()`

See: XILEM_0_4_QUICK_REFERENCE.md (Application Entry Point section)

### Question 2: How is flex() used?

**Quick Answer:** Use `flex_row()` or `flex_col()` for most cases, with `FlexSpacer::Fixed()` or `FlexSpacer::Flex()` for spacing, and `.flex()` method on children for sizing.

**Details:**
- `flex_row((item1, item2, item3))` - horizontal layout
- `flex_col((item1, item2, item3))` - vertical layout
- `FlexSpacer::Fixed(10.px())` - fixed spacing
- `FlexSpacer::Flex(1.0)` - flexible spacing
- `widget.flex(2.0)` - apply flex factor to child
- `.gap(10.px())`, `.main_axis_alignment()`, `.cross_axis_alignment()`

See: XILEM_0_4_QUICK_REFERENCE.md (Layout System section)  
Full Details: XILEM_0_4_API_GUIDE.md (Flex Layout section)

### Question 3: How are sized_box dimensions specified?

**Quick Answer:** Use `.width(100.px()).height(50.px())` with the `AsUnit` trait for `.px()` conversion.

**Details:**
- `sized_box(widget).width(100.px()).height(50.px())`
- `sized_box(widget).expand()` - fill all space
- `sized_box(widget).expand_width()` or `.expand_height()`
- Import: `use masonry::properties::types::AsUnit;`

See: XILEM_0_4_QUICK_REFERENCE.md (Sizing section)

### Question 4: What does AppState trait require?

**Quick Answer:** Single method: `fn keep_running(&self) -> bool;` - only needed for `Xilem::new()` with multiple windows, NOT for `Xilem::new_simple()`.

**Details:**
- Trait: `pub trait AppState { fn keep_running(&self) -> bool; }`
- For `new_simple()`: automatically wrapped in `ExitOnClose<State>`
- For `new()`: must implement the trait
- Checked after window close request

See: XILEM_0_4_QUICK_REFERENCE.md (AppState Trait section)

### Question 5: Complete working example?

See: **XILEM_0_4_EXAMPLE.rs** - Full counter app with:
- Multiple buttons
- Flex layouts (rows and columns)
- Sized boxes with fixed dimensions
- Button callbacks and state mutation
- Label styling and text sizes
- Complex nested layouts
- Color styling

Can copy directly into a Cargo project with appropriate dependencies.

---

## File-by-File Breakdown

### 1. XILEM_0_4_QUICK_REFERENCE.md (START HERE)
- **Type:** Quick reference card
- **Best for:** Quick lookups, fixing immediate errors
- **Length:** ~2 pages when printed
- **Contains:**
  - Application entry point template
  - Layout system tables
  - Sizing patterns
  - Common widgets table
  - Styling methods
  - State management patterns
  - Alignment options reference
  - Common compiler errors and fixes
  - Tips and tricks
  - Architecture overview diagram

### 2. XILEM_0_4_SUMMARY.txt
- **Type:** Plain text reference
- **Best for:** Terminal viewing, quick lookups without markdown
- **Length:** ~4 pages
- **Contains:**
  - Key answers to your 5 questions
  - Essential imports block
  - API patterns cheat sheet
  - Complete minimal example
  - Alignment options
  - Resource file locations
  - Rust version requirements
  - Common mistakes and fixes

### 3. XILEM_0_4_API_GUIDE.md
- **Type:** Comprehensive documentation
- **Best for:** Understanding concepts deeply
- **Length:** ~8 pages
- **Contains:**
  - Version information
  - Detailed explanations of all major features
  - Runnings applications (new_simple vs new)
  - AppState trait details
  - Flex layout system explanation
  - Sized box dimension system
  - Widget layout and styling
  - Lens for component composition
  - Common widget types with examples
  - Complete working counter example (simpler than EXAMPLE.rs)
  - Key API patterns cheat sheet
  - Common compilation errors and fixes
  - Resources and links

### 4. XILEM_0_3_VS_0_4_COMPARISON.md
- **Type:** Migration guide
- **Best for:** Upgrading from 0.3.0 or understanding what changed
- **Length:** ~9 pages
- **Contains:**
  - Summary of major changes
  - Side-by-side API comparisons (8 sections)
  - Application initialization changes
  - Flex layout system evolution
  - Sized box dimension API changes
  - View function signature updates
  - Imports organization changes
  - State management evolution
  - Button callback patterns
  - AppState trait changes
  - Side-by-side 0.3 vs 0.4 counter examples
  - Migration checklist
  - Performance and compatibility notes

### 5. XILEM_0_4_EXAMPLE.rs
- **Type:** Complete working Rust source code
- **Best for:** Copy-paste reference, learning by example
- **Lines:** 224 lines including comments
- **Contains:**
  - Full counter application
  - Multiple features (count, multiplier, squared toggle, reset)
  - Demonstrates:
    - State struct setup
    - Application initialization
    - Complex nested flex layouts
    - Button handling
    - Label styling
    - Color styling
    - Sized boxes with fixed dimensions
    - Event loop setup
    - Extensive inline comments
  - Can be directly compiled and run

---

## Recommended Reading Order

### For Quick Fixes (15 minutes)
1. XILEM_0_4_QUICK_REFERENCE.md - Get patterns
2. XILEM_0_4_SUMMARY.txt - Find your specific error
3. Done!

### For Understanding the API (1 hour)
1. XILEM_0_4_QUICK_REFERENCE.md - Overview
2. XILEM_0_4_API_GUIDE.md - Deep dive
3. XILEM_0_4_EXAMPLE.rs - See it in action
4. Reference other docs as needed

### For Migration from 0.3.0 (30-45 minutes)
1. XILEM_0_3_VS_0_4_COMPARISON.md - Read full comparison
2. XILEM_0_4_QUICK_REFERENCE.md - Learn new patterns
3. XILEM_0_4_API_GUIDE.md - Fill in gaps
4. Use migration checklist in comparison doc

### For Learning Everything (2-3 hours)
1. XILEM_0_4_API_GUIDE.md - Complete reference
2. XILEM_0_3_VS_0_4_COMPARISON.md - Context on changes
3. XILEM_0_4_EXAMPLE.rs - See best practices
4. XILEM_0_4_QUICK_REFERENCE.md - Bookmark for later

---

## Quick Reference Snippets

### Absolute Minimum Setup
```rust
use xilem::{EventLoop, Xilem, WidgetView, WindowOptions};
use xilem::view::{label};

fn app_logic(state: &mut i32) -> impl WidgetView<i32> + use<> {
    label(format!("Value: {}", state))
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let app = Xilem::new_simple(0, app_logic, WindowOptions::new("App"));
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

### Common Layout Pattern
```rust
flex_row((
    sized_box(button("-", |s| *s -= 1)).width(60.px()).height(60.px()),
    FlexSpacer::Flex(1.0),
    label(format!("Value: {}", value)).text_size(24.),
    FlexSpacer::Flex(1.0),
    sized_box(button("+", |s| *s += 1)).width(60.px()).height(60.px()),
))
.cross_axis_alignment(CrossAxisAlignment::Center)
```

### Import Everything You Need
```rust
use xilem::{EventLoop, Xilem, WidgetView, WindowOptions, Color};
use xilem::view::{
    flex_row, flex_col, FlexSpacer, FlexExt,
    button, label, sized_box,
};
use xilem::style::Style as _;
use masonry::properties::types::{AsUnit, CrossAxisAlignment, MainAxisAlignment};
```

---

## Troubleshooting

### "error: unresolved path `FlexExt`"
Add import: `use xilem::view::FlexExt;`

### "cannot find method `flex` for..."
Import the trait: `use xilem::view::FlexExt;`

### "cannot find method `px` for..."
Import trait: `use masonry::properties::types::AsUnit;`

### "type mismatch" with flex children
Use tuple syntax: `flex_row((a, b, c))` not `flex_row(vec![a, b, c])`

### "child doesn't fill space"
Add flex factor: `widget.flex(1.0)`

For more issues, see:
- XILEM_0_4_QUICK_REFERENCE.md (Common Compiler Errors table)
- XILEM_0_4_SUMMARY.txt (Common Mistakes & Fixes section)
- XILEM_0_4_API_GUIDE.md (Common Compilation Errors section)

---

## External Resources

- **Official Repository:** https://github.com/linebender/xilem
- **Issue Tracker:** https://github.com/linebender/xilem/issues
- **Community Chat:** https://xi.zulipchat.com/
- **Examples in Repo:** /xilem/examples/ directory
  - flex.rs - Layout system demo
  - calc.rs - Calculator app
  - widgets.rs - Widget gallery
  - components.rs - State composition
  - to_do_mvc.rs - Todo app

---

## Document Metadata

| File | Lines | Topics | Best For |
|------|-------|--------|----------|
| XILEM_0_4_QUICK_REFERENCE.md | 364 | 10+ tables & code blocks | Quick lookups |
| XILEM_0_4_SUMMARY.txt | 266 | Comprehensive summary | Plain text reference |
| XILEM_0_4_API_GUIDE.md | 434 | Detailed explanations | Learning |
| XILEM_0_3_VS_0_4_COMPARISON.md | 380 | Side-by-side comparisons | Migration |
| XILEM_0_4_EXAMPLE.rs | 224 | Complete working code | Implementation |

**Total:** 1668 lines of documentation and examples

---

## Version Information

- **Xilem Version:** 0.4.0
- **Rust Edition:** 2024
- **Minimum Rust Version:** 1.88
- **Documentation Generated:** October 30, 2025
- **Status:** Complete and ready for use

---

## How to Use These Documents

1. **Keep them bookmarked** for quick reference while coding
2. **Print XILEM_0_4_QUICK_REFERENCE.md** as a desk reference
3. **Use XILEM_0_4_SUMMARY.txt** in your terminal for offline reference
4. **Copy patterns from XILEM_0_4_API_GUIDE.md** for new features
5. **Reference XILEM_0_3_VS_0_4_COMPARISON.md** when upgrading
6. **Copy code from XILEM_0_4_EXAMPLE.rs** as a starting template

These documents comprehensively cover Xilem 0.4.0 API changes and should answer all compilation errors related to the new API.

