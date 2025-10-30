# Xilem 0.4.0 Quick Reference Card

## Application Entry Point

```rust
use xilem::{EventLoop, Xilem, WidgetView, WindowOptions};

fn app_logic(state: &mut MyState) -> impl WidgetView<MyState> + use<> {
    // Build UI here
}

fn main() -> Result<(), EventLoopError> {
    let app = Xilem::new_simple(
        initial_state,
        app_logic,
        WindowOptions::new("App Title"),
    );
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

---

## Layout System

### Flex Containers
| Purpose | Code |
|---------|------|
| Horizontal row | `flex_row((item1, item2, item3))` |
| Vertical column | `flex_col((item1, item2, item3))` |
| Dynamic axis | `flex(Axis::Horizontal, children)` |

### Spacing
```rust
FlexSpacer::Fixed(10.px())   // Fixed space
FlexSpacer::Flex(1.0)        // Flexible space
```

### Flex Container Methods
```rust
flex_row(children)
    .gap(10.px())                              // Space between items
    .main_axis_alignment(MainAxisAlignment::Center)
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .fill_major_axis(true)
```

### Flex Child Methods
```rust
widget
    .flex(2.0)                                 // Flex factor
    .flex(CrossAxisAlignment::Fill)            // Fill perpendicular axis
```

---

## Sizing

### Absolute Dimensions
```rust
use masonry::properties::types::AsUnit;

sized_box(widget)
    .width(100.px())
    .height(50.px())
```

### Relative Sizing
```rust
sized_box(widget).expand()          // Fill all available space
sized_box(widget).expand_width()    // Fill width only
sized_box(widget).expand_height()   // Fill height only
```

### Length Types
```rust
100.px()                    // Via AsUnit trait
Length::const_px(100.0)     // Explicit constant
50.em()                     // Em units
```

---

## Widgets

| Widget | Code |
|--------|------|
| Button | `button("Label", \|state\| { state.val += 1; })` |
| Text Button | `text_button("Click", \|state\| { })` |
| Label | `label("Text").text_size(24.)` |
| Checkbox | `checkbox("Label", bool_state, \|state, checked\| { })` |
| Sized Box | `sized_box(widget).width(100.px()).height(50.px())` |
| Progress Bar | `progress_bar(Optional<f64>)` |

---

## Styling

```rust
widget
    .background_color(Color::from_rgb8(255, 0, 0))
    .corner_radius(10.)
    .border(Color::WHITE, 2.)
    .border_color(Color::BLUE)
    .hovered_border_color(Color::YELLOW)
    .text_size(28.)
    .disabled(condition)
```

### Colors
```rust
Color::from_rgb8(r, g, b)
Color::WHITE
Color::BLACK
Color::TRANSPARENT
```

---

## State Management

### Simple State
```rust
fn component(state: &mut i32) -> impl WidgetView<i32> + use<> {
    label(format!("Value: {}", state))
}
```

### Nested State (Lens)
```rust
use xilem_core::lens;

struct Parent {
    child_count: i32,
}

lens(
    |count: &mut i32| { /* view for count */ },
    |parent: &mut Parent| &mut parent.child_count
)
```

---

## Alignment Options

### Main Axis (Flow Direction)
```
MainAxisAlignment::Start           // Default
MainAxisAlignment::Center
MainAxisAlignment::End
MainAxisAlignment::SpaceBetween
MainAxisAlignment::SpaceAround
MainAxisAlignment::SpaceEvenly
```

### Cross Axis (Perpendicular)
```
CrossAxisAlignment::Start
CrossAxisAlignment::Center         // Xilem default
CrossAxisAlignment::End
CrossAxisAlignment::Fill
```

---

## AppState Trait

Only needed for `Xilem::new()` with multiple windows:

```rust
pub trait AppState {
    fn keep_running(&self) -> bool;
}

impl AppState for MyState {
    fn keep_running(&self) -> bool {
        true  // Return false to exit app
    }
}
```

For `Xilem::new_simple()`: Trait implementation handled automatically!

---

## Common Patterns

### Counter Button
```rust
button("+", |state: &mut i32| *state += 1)
```

### Conditional Display
```rust
if condition {
    label("Show this")
} else {
    label("Show that")
}
```

### Dynamic Text
```rust
label(format!("Count: {}", state.count))
    .text_size(24.)
```

### Centered Layout
```rust
flex_row((item1, item2, item3))
    .main_axis_alignment(MainAxisAlignment::Center)
    .cross_axis_alignment(CrossAxisAlignment::Center)
```

### Expanding Container
```rust
sized_box(
    flex_col((item1, item2))
)
.expand()
```

### Fixed Size Button
```rust
sized_box(
    button("+", |s| *s += 1)
        .background_color(Color::from_rgb8(50, 150, 50))
        .corner_radius(5.)
)
.width(60.px())
.height(60.px())
```

---

## Essential Imports

```rust
// Application framework
use xilem::{EventLoop, Xilem, WidgetView, WindowOptions, Color};
use winit::error::EventLoopError;

// Layout and views
use xilem::view::{
    flex_row, flex_col, FlexSpacer, FlexExt,
    button, text_button, label, checkbox,
    sized_box, progress_bar,
};

// Properties and types
use xilem::style::Style as _;
use masonry::properties::types::{
    AsUnit, Axis,
    CrossAxisAlignment, MainAxisAlignment,
};

// State composition
use xilem_core::lens;
```

---

## Common Compiler Errors

| Error | Solution |
|-------|----------|
| Type inference fails for flex children | Wrap in tuple: `flex_row((a, b, c))` |
| `FlexExt` not found | Import trait: `use xilem::view::FlexExt;` |
| `AsUnit` not found | Import trait: `use masonry::properties::types::AsUnit;` |
| Can't use `.px()` | Import `AsUnit` trait |
| Missing `+ use<>` on view function | Add to return type: `impl WidgetView<State> + use<>` |
| Child doesn't fill space | Add flex: `widget.flex(1.0)` |

---

## Version Info

- **Current Version**: 0.4.0
- **Rust Edition**: 2024
- **Minimum Rust**: 1.88
- **Repository**: https://github.com/linebender/xilem

---

## Documentation & Examples

Local docs:
```bash
cargo doc --open
```

Run example:
```bash
cargo run --example flex
cargo run --example calc
cargo run --example widgets
```

Browse examples:
```
/xilem/examples/
  - flex.rs           (layout demo)
  - calc.rs           (calculator)
  - widgets.rs        (widget gallery)
  - components.rs     (state composition)
  - to_do_mvc.rs      (todo app)
```

---

## Tips & Tricks

1. **Always use `flex_row()` or `flex_col()`** instead of raw `flex()` for clarity
2. **Import `FlexExt` trait** to get fluent `.flex()` and alignment methods
3. **Use `FlexSpacer`** for spacing instead of empty containers
4. **Set flex on long children** like text inputs in horizontal layouts
5. **Use `lens()` for components** to decompose state management
6. **Wrap buttons in `sized_box`** to control their size
7. **Use `+ use<>`** in view function signatures (Rust 2024 requirement)
8. **Import traits as `_ `** for extension methods: `use xilem::style::Style as _;`

---

## Architecture Overview

```
┌─────────────────────────────────────┐
│  Your App State                     │
│  (struct with #[derive(Clone)])     │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│  app_logic() Function               │
│  fn(&mut State) -> impl WidgetView  │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│  View Tree                          │
│  flex_row/col, buttons, labels      │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│  Xilem Framework                    │
│  Layout → Rendering → Event Loop    │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│  Masonry Layout Engine              │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│  Vello Renderer (GPU)               │
└─────────────────────────────────────┘
```

Each state change triggers a rebuild of the view tree (reactive paradigm).

