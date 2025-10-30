# Xilem 0.4.0 API Changes and Guide

## Version Information
- Current version: **0.4.0**
- Edition: 2024
- Minimum Rust: 1.88

## Core API Changes from 0.3.0 to 0.4.0

### 1. Running Applications

#### Application Entry Point
```rust
use xilem::{EventLoop, WidgetView, WindowOptions, Xilem};

// Most common pattern for single-window apps:
let app = Xilem::new_simple(
    state,                    // Initial state (any type)
    app_logic,               // Closure: &mut State -> View
    WindowOptions::new("Title")
);

app.run_in(EventLoop::with_user_event())?;
```

Key patterns:
- `Xilem::new_simple()` - Single window, exits on close (most common)
- `Xilem::new()` - Multiple windows, requires `AppState` trait
- `run_in(EventLoop)` - Run with event loop
- No separate `.run()` method with custom event loop handling

### 2. AppState Trait

Only required when using `Xilem::new()` for multi-window apps:

```rust
pub trait AppState {
    /// Returns whether the application should keep running.
    /// Only checked after a close request.
    fn keep_running(&self) -> bool;
}
```

For `new_simple()`: Uses `ExitOnClose<State>` wrapper internally - no trait needed.

### 3. Flex Layout

#### Core flex functions:
```rust
use xilem::view::{flex, flex_row, flex_col, FlexSpacer, FlexExt};
use masonry::properties::types::{Axis, CrossAxisAlignment, MainAxisAlignment};

// Generic flex - specify axis explicitly
flex(Axis::Horizontal, children)
flex(Axis::Vertical, children)

// Convenience functions
flex_row(children)
flex_col(children)
```

#### FlexSpacer (spacing):
```rust
// Fixed space
FlexSpacer::Fixed(30.px())    // or Length::const_px(30.0)

// Flexible space (grows to fill available space)
FlexSpacer::Flex(1.0)         // 1.0 is flex factor
```

#### Flex child configuration (FlexExt trait):
```rust
// Apply flex factor to any view
label("text").flex(2.0)
button("Click", callback).flex(1.0)

// Make item fill perpendicular axis
item.flex(CrossAxisAlignment::Fill)

// Expand to fill all available space
item.flex_fill()
```

#### Flex container configuration:
```rust
flex_row(children)
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .main_axis_alignment(MainAxisAlignment::Start)
    .gap(10.px())
```

Main axis alignment options:
- `MainAxisAlignment::Start` (default)
- `MainAxisAlignment::Center`
- `MainAxisAlignment::End`
- `MainAxisAlignment::SpaceBetween`
- `MainAxisAlignment::SpaceAround`
- `MainAxisAlignment::SpaceEvenly`

Cross axis alignment options:
- `CrossAxisAlignment::Start`
- `CrossAxisAlignment::Center` (default for Xilem)
- `CrossAxisAlignment::End`
- `CrossAxisAlignment::Fill`

### 4. Sized Box Dimensions

#### Creating a sized box:
```rust
use xilem::view::sized_box;
use masonry::properties::types::AsUnit;

let boxed = sized_box(inner_widget);
```

#### Setting dimensions:
```rust
// Absolute dimensions (using Length type)
sized_box(widget)
    .width(100.px())        // Length::from(100.0) via AsUnit trait
    .height(50.px())
    .width(Length::const_px(100.0))

// Relative sizing
sized_box(widget)
    .expand()               // Fill all available space
    .expand_width()         // Fill horizontally only
    .expand_height()        // Fill vertically only

// Chaining is supported
sized_box(button("Click", callback))
    .width(80.px())
    .height(40.px())
```

Available Length types:
- `Length::const_px(f64)` - Constant pixels
- `Value::Px(f64).into()` - Dynamic pixels
- Via `AsUnit` trait: `.px()`, `.em()`, etc.

### 5. Widget Layout and Styling

#### Import styles and properties:
```rust
use xilem::style::Style as _;  // For fluent style methods
use masonry::properties::types::AsUnit;  // For unit conversions
```

#### Common style methods on widgets:
```rust
button(label, callback)
    .background_color(Color::WHITE)
    .corner_radius(10.)
    .border_color(Color::WHITE)
    .border(Color::YELLOW, 2.)
    .hovered_border_color(Color::WHITE)
    .text_size(32.)
    .disabled(condition)

sized_box(widget)
    .border(Color::WHITE, 2.)
    .padding(10.0)
    .expand()
```

### 6. Lens for Component Composition

Access nested state in subcomponents:

```rust
use xilem_core::lens;

struct AppState {
    counter: i32,
    name: String,
}

fn counter_component(count: &mut i32) -> impl WidgetView<i32> + use<> {
    flex_col((
        label(format!("Count: {count}")),
        button("+", |c| *c += 1),
    ))
}

fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    flex_row((
        lens(
            counter_component,
            |state: &mut AppState| &mut state.counter
        ),
        label(&state.name),
    ))
}
```

### 7. Common Widget Types

```rust
use xilem::view::{
    button, text_button, label, checkbox, 
    progress_bar, text_input, flex_row, flex_col,
    sized_box, indexed_stack
};

// Simple text button
text_button("Click Me", |state: &mut State| {
    // Handle click
})

// Button with custom content
button(label("Custom"), |state: &mut State| {
    // Handle click
})

// Text display
label(format!("Count: {}", count))
    .text_size(32.)

// Checkbox
checkbox("Label", checked_bool, |state: &mut State, is_checked| {
    state.flag = is_checked;
})

// Conditional display with indexed_stack
indexed_stack((
    view_for_tab_0,
    view_for_tab_1,
    view_for_tab_2,
))
.active(current_tab_index)
```

## Complete Working Example: Simple Counter

```rust
// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

use masonry::properties::types::{AsUnit, CrossAxisAlignment, MainAxisAlignment};
use winit::error::EventLoopError;
use xilem::view::{FlexExt, FlexSpacer, button, flex_row, label, sized_box};
use xilem::{Color, EventLoop, WidgetView, WindowOptions, Xilem};

/// Simple application state
#[derive(Default)]
struct CounterApp {
    count: i32,
    multiplier: i32,
}

fn app_logic(state: &mut CounterApp) -> impl WidgetView<CounterApp> + use<> {
    let display_value = state.count * state.multiplier;
    
    flex_row((
        // Left side: Decrease button
        sized_box(
            button("−", |state: &mut CounterApp| {
                state.count -= 1;
            })
            .background_color(Color::from_rgb8(200, 50, 50))
            .corner_radius(5.)
        )
        .width(60.px())
        .height(60.px()),
        
        // Center: Display and multiplier controls
        FlexSpacer::Flex(1.0),
        sized_box(
            flex_row((
                flex_row((
                    label(format!("Value: {display_value}"))
                        .text_size(24.),
                    FlexSpacer::Flex(1.0),
                ))
                .flex(2.0),
                FlexSpacer::Fixed(10.px()),
                flex_row((
                    button("×2", |s: &mut CounterApp| s.multiplier += 1)
                        .flex(1.0),
                    button("÷2", |s: &mut CounterApp| s.multiplier -= 1)
                        .flex(1.0),
                ))
                .flex(1.0),
            ))
            .gap(5.px())
            .cross_axis_alignment(CrossAxisAlignment::Center)
        )
        .expand_width(),
        FlexSpacer::Flex(1.0),
        
        // Right side: Increase button
        sized_box(
            button("+", |state: &mut CounterApp| {
                state.count += 1;
            })
            .background_color(Color::from_rgb8(50, 150, 50))
            .corner_radius(5.)
        )
        .width(60.px())
        .height(60.px()),
    ))
    .main_axis_alignment(MainAxisAlignment::Center)
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(20.px())
}

fn main() -> Result<(), EventLoopError> {
    let app = Xilem::new_simple(
        CounterApp::default(),
        app_logic,
        WindowOptions::new("Counter App")
            .with_initial_inner_size(winit::dpi::LogicalSize::new(500.0, 150.0)),
    );
    
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

## Key API Patterns Cheat Sheet

### Application Setup
```rust
// Single window (most common)
let app = Xilem::new_simple(state, app_logic, WindowOptions::new("Title"));
app.run_in(EventLoop::with_user_event())?;

// Multi-window (requires AppState)
impl AppState for MyState {
    fn keep_running(&self) -> bool { true }
}
let app = Xilem::new(state, app_logic);
app.run_in(EventLoop::with_user_event())?;
```

### Layout Containers
```rust
// Row layout
flex_row((item1, item2, item3))

// Column layout
flex_col((item1, item2, item3))

// Generic with axis
flex(Axis::Horizontal, sequence)
```

### Sizing
```rust
// Absolute
sized_box(widget).width(100.px()).height(50.px())

// Relative
sized_box(widget).expand()
sized_box(widget).expand_width()

// Length types
100.px()              // Via AsUnit trait
Length::const_px(100.0)
```

### Flex Child Configuration
```rust
// Grow to fill space
widget.flex(2.0)      // Flex factor of 2.0

// Fill perpendicular axis
widget.flex(CrossAxisAlignment::Fill)

// Spacing
FlexSpacer::Fixed(10.px())
FlexSpacer::Flex(1.0)
```

### Container Configuration
```rust
flex_row(children)
    .gap(10.px())
    .main_axis_alignment(MainAxisAlignment::Center)
    .cross_axis_alignment(CrossAxisAlignment::Center)
```

### State Management
```rust
// Simple state
fn app_logic(state: &mut MyState) -> impl WidgetView<MyState> + use<> { }

// Nested state with lens
lens(component_view, |state: &mut Parent| &mut state.child)
```

## Common Compilation Errors and Fixes

### Error: "type mismatch in fn" for app_logic
Make sure app_logic returns `impl WidgetView<State> + use<>`:
```rust
fn app_logic(state: &mut State) -> impl WidgetView<State> + use<> {
    // Return a view
}
```

### Error: "cannot infer type" for flex children
Wrap in tuple or use type hints:
```rust
// Good - tuple inference works
flex_row((item1, item2, item3))

// Also good - explicit type
flex_row::<State, Vec<_>>(vec![...])
```

### Error: "child doesn't fill space"
Add flex factor to children in flex containers:
```rust
// Bad - text_input might not fill space
flex_row((text_input(...), button("OK", callback)))

// Good - explicitly set flex
flex_row((
    text_input(...).flex(1.0),
    button("OK", callback)
))
```

## Resources

- Xilem Repository: https://github.com/linebender/xilem
- Examples: /xilem/examples/ directory
- Main modules:
  - `xilem` - High-level UI framework
  - `masonry` - Layout and rendering engine
  - `masonry_winit` - Windowing integration
  - `xilem_core` - Core traits and lens utilities

