# Xilem 0.3.0 vs 0.4.0 API Comparison

## Summary of Major Changes

Xilem 0.4.0 introduces significant API refinements focusing on:
- Simplified application initialization
- Improved flex layout system
- Better type inference for views
- Cleaner state management patterns

---

## 1. Application Initialization

### 0.3.0 Pattern
```rust
// Old approach - more verbose
use xilem::EventLoop;

fn main() -> Result<(), EventLoopError> {
    let app = App {
        initial_state,
        app_logic,
    };
    
    app.run(EventLoop::with_user_event())?;
    Ok(())
}
```

### 0.4.0 Pattern (CURRENT)
```rust
// New approach - cleaner
use xilem::{EventLoop, Xilem};

fn main() -> Result<(), EventLoopError> {
    let app = Xilem::new_simple(
        initial_state,
        app_logic,
        WindowOptions::new("Title"),
    );
    
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

**Changes:**
- `Xilem::new_simple()` is the primary API for single-window apps
- Explicit `WindowOptions` configuration
- `.run_in()` instead of `.run()`
- Better error handling and type safety

---

## 2. Flex Layout System

### 0.3.0 Flex API
```rust
// Flex needed explicit axis parameter
use xilem::view::{flex, Axis};

flex(Axis::Horizontal, children)
flex(Axis::Vertical, children)

// FlexSpacer syntax similar but less flexible
FlexSpacer::Fixed(10.0)
FlexSpacer::Flex(1.0)
```

### 0.4.0 Flex API (CURRENT)
```rust
// Convenience functions for common cases
use xilem::view::{flex_row, flex_col};

flex_row(children)      // Explicit, preferred
flex_col(children)      // Explicit, preferred
flex(Axis::Horizontal, children)  // Also available

// FlexSpacer with Length type
use masonry::properties::types::AsUnit;

FlexSpacer::Fixed(10.px())  // Uses px() trait method
FlexSpacer::Flex(1.0)       // Unchanged

// New trait for flex configuration
use xilem::view::FlexExt;

widget.flex(2.0)                          // Set flex factor
widget.flex(CrossAxisAlignment::Fill)     // Fill cross axis

// Container configuration methods
flex_row(children)
    .gap(10.px())
    .main_axis_alignment(MainAxisAlignment::Center)
    .cross_axis_alignment(CrossAxisAlignment::Center)
```

**Changes:**
- Dedicated `flex_row()` and `flex_col()` functions
- `FlexExt` trait provides fluent configuration
- Length type system with `.px()` conversion
- Explicit alignment configuration

---

## 3. Sized Box Dimensions

### 0.3.0 API
```rust
// Direct dimension setting (less clear)
sized_box(widget)
    .width(100.0)
    .height(50.0)

// Expand methods similar
sized_box(widget).expand()
```

### 0.4.0 API (CURRENT)
```rust
// Clear Length type with units
use masonry::properties::types::AsUnit;

sized_box(widget)
    .width(100.px())        // Via AsUnit trait
    .height(50.px())
    .width(Length::const_px(100.0))  // Explicit

// Expand methods unchanged
sized_box(widget).expand()
sized_box(widget).expand_width()
sized_box(widget).expand_height()
```

**Changes:**
- Explicit unit system with `Length` type
- `.px()` conversion via `AsUnit` trait
- Better type safety and clarity
- Support for other units (`em`, `rem`, etc.)

---

## 4. View Function Signature

### 0.3.0 Pattern
```rust
fn app_logic(state: &mut MyState) -> impl WidgetView<MyState> {
    // return a view
}
```

### 0.4.0 Pattern (CURRENT)
```rust
fn app_logic(state: &mut MyState) -> impl WidgetView<MyState> + use<> {
    // return a view
}
```

**Changes:**
- `+ use<>` syntax for explicit capture tracking (Rust 2024 feature)
- Improves type inference and compiler diagnostics
- More explicit about closure captures

---

## 5. Imports Organization

### 0.3.0 Typical Imports
```rust
use xilem::{EventLoop, Xilem, WidgetView, WindowOptions};
use xilem::view::{flex, button, label};
```

### 0.4.0 Typical Imports (CURRENT)
```rust
// Application setup
use xilem::{EventLoop, Xilem, WidgetView, WindowOptions};

// Views and layout
use xilem::view::{flex_row, flex_col, FlexSpacer, FlexExt, button, label, sized_box};

// Styling and alignment
use xilem::style::Style as _;  // Import as trait
use masonry::properties::types::{AsUnit, CrossAxisAlignment, MainAxisAlignment};

// Colors
use xilem::Color;
```

**Changes:**
- `FlexExt` trait must be imported for fluent syntax
- `Style` trait import for styling methods
- Unit traits like `AsUnit` for dimension conversions

---

## 6. State Management and Lens

### 0.3.0 Lens Pattern
```rust
// Similar to 0.4.0 but slightly different syntax
use xilem::lens;

lens(component_view, |state: &mut Parent| &mut state.field)
```

### 0.4.0 Pattern (CURRENT)
```rust
// Same core API, better type inference
use xilem_core::lens;

lens(component_view, |state: &mut Parent| &mut state.field)
```

**Changes:**
- Moved to `xilem_core` (clearer module hierarchy)
- Better type inference in complex scenarios
- More consistent with modular component patterns

---

## 7. Button Callbacks

### 0.3.0 Style
```rust
button("Text", |state: &mut MyState| {
    state.value += 1;
})
```

### 0.4.0 Style (CURRENT - SAME)
```rust
button("Text", |state: &mut MyState| {
    state.value += 1;
})

// Alternative with text_button (convenience)
text_button("Click Me", |state: &mut MyState| {
    state.value += 1;
})
```

**Changes:**
- No breaking changes to callback style
- `text_button` convenience function clearer intent
- Better type inference with newer type system

---

## 8. AppState Trait

### 0.3.0 (if multi-window)
```rust
// Required for multi-window apps
impl AppState for MyState {
    fn keep_running(&self) -> bool { true }
}
```

### 0.4.0 (CURRENT)
```rust
// Same trait, but NEW_SIMPLE handles it automatically
pub trait AppState {
    fn keep_running(&self) -> bool;
}

// For new_simple() - trait is wrapped automatically in ExitOnClose<T>
let app = Xilem::new_simple(state, app_logic, options);
// ^ No trait impl needed!

// For new() with multiple windows - trait still required
impl AppState for MyState {
    fn keep_running(&self) -> bool { true }
}
let app = Xilem::new(state, app_logic);
```

**Changes:**
- Automatic `AppState` wrapping for `new_simple()`
- Single-window apps don't need trait implementation
- Cleaner API for common use case

---

## Side-by-Side Example Comparison

### 0.3.0 Simple Counter
```rust
fn counter_view(state: &mut i32) -> impl WidgetView<i32> {
    flex(
        Axis::Horizontal,
        (
            button("-", |s| *s -= 1),
            label(format!("count: {}", state)),
            button("+", |s| *s += 1),
        ),
    )
}

fn main() -> Result<(), EventLoopError> {
    let app = App {
        initial_state: 0,
        app_logic: counter_view,
    };
    app.run(EventLoop::with_user_event())?;
    Ok(())
}
```

### 0.4.0 Simple Counter (CURRENT)
```rust
fn counter_view(state: &mut i32) -> impl WidgetView<i32> + use<> {
    flex_row((
        button("−", |s| *s -= 1).width(60.px()),
        FlexSpacer::Flex(1.0),
        label(format!("count: {}", state))
            .text_size(24.)
            .flex(1.0),
        FlexSpacer::Flex(1.0),
        button("+", |s| *s += 1).width(60.px()),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
}

fn main() -> Result<(), EventLoopError> {
    let app = Xilem::new_simple(
        0,
        counter_view,
        WindowOptions::new("Counter"),
    );
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

---

## Migration Checklist

If updating from 0.3.0 to 0.4.0:

- [ ] Update `Xilem` initialization:
  - Use `Xilem::new_simple()` for single window
  - Replace `.run()` with `.run_in()`
  - Add `WindowOptions` parameter

- [ ] Update flex usage:
  - Replace `flex(Axis::*, ...)` with `flex_row()` or `flex_col()`
  - Import `FlexExt` trait for `.flex()` and alignment methods
  - Add `.gap()` calls instead of manual spacing

- [ ] Update sizing:
  - Import `AsUnit` trait
  - Convert raw numbers to `.px()` format
  - Use `Length::const_px()` for explicit constants

- [ ] Update imports:
  - Add `use xilem::style::Style as _;`
  - Add `use masonry::properties::types::AsUnit;`
  - Import `FlexExt` trait

- [ ] Update view function signature:
  - Add `+ use<>` to return type

- [ ] Remove `AppState` impl if using `new_simple()`

- [ ] Update closure captures:
  - Verify `move` closures work with new type system

---

## Performance and Compatibility

- 0.4.0 has improved performance through better type inference
- Compile times may be slightly longer due to Rust 2024 edition features
- Requires Rust 1.88+
- Binary compatibility: No (new major version)
- ABI: Depends on masonry/winit versions

