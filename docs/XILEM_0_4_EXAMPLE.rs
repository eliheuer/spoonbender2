// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0
//
// Xilem 0.4.0 Complete Example - Multi-feature Counter Application
// This example demonstrates:
// - Application initialization with Xilem::new_simple
// - Flex layouts (flex_row, flex_col)
// - Sized boxes with absolute dimensions
// - Button handling and state mutation
// - Label styling and text sizes
// - Complex nested layouts
// - Color styling

use masonry::properties::types::{AsUnit, CrossAxisAlignment, MainAxisAlignment};
use winit::error::EventLoopError;
use xilem::view::{FlexExt, FlexSpacer, button, flex_col, flex_row, label, sized_box};
use xilem::{Color, EventLoop, WidgetView, WindowOptions, Xilem};

/// Application state - all data owned by Xilem
#[derive(Clone, Copy)]
struct CounterAppState {
    /// Main counter value
    count: i32,
    /// Multiplier for display
    multiplier: i32,
    /// Toggle for alternate display mode
    show_squared: bool,
}

impl Default for CounterAppState {
    fn default() -> Self {
        Self {
            count: 0,
            multiplier: 1,
            show_squared: false,
        }
    }
}

/// Main application logic function
/// 
/// This is called by Xilem on every state change to rebuild the view tree.
/// Signature: `fn(&mut State) -> impl WidgetView<State> + use<>`
fn app_logic(state: &mut CounterAppState) -> impl WidgetView<CounterAppState> + use<> {
    // Compute display value
    let display_value = if state.show_squared {
        state.count * state.count
    } else {
        state.count * state.multiplier
    };

    // Main container - vertical flex layout
    flex_col((
        // Title section
        label("Counter Application")
            .text_size(28.),
        
        FlexSpacer::Fixed(15.px()),
        
        // Main control section - horizontal layout
        flex_row((
            // Left: Decrease button
            sized_box(
                button("−", |state: &mut CounterAppState| {
                    state.count -= 1;
                })
                .background_color(Color::from_rgb8(200, 50, 50))
                .corner_radius(8.)
                .text_size(20.)
            )
            .width(80.px())
            .height(80.px()),
            
            // Center: Display with value
            FlexSpacer::Flex(1.0),
            flex_col((
                label("Current Value:").text_size(14.),
                label(format!("{display_value}"))
                    .text_size(48.)
                    .flex(1.0),
            ))
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center),
            FlexSpacer::Flex(1.0),
            
            // Right: Increase button
            sized_box(
                button("+", |state: &mut CounterAppState| {
                    state.count += 1;
                })
                .background_color(Color::from_rgb8(50, 150, 50))
                .corner_radius(8.)
                .text_size(20.)
            )
            .width(80.px())
            .height(80.px()),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .gap(15.px()),
        
        FlexSpacer::Fixed(15.px()),
        
        // Multiplier section
        flex_row((
            label(format!("Multiplier: {}", state.multiplier))
                .text_size(14.),
            FlexSpacer::Flex(1.0),
            sized_box(
                button("−", |s: &mut CounterAppState| {
                    s.multiplier = (s.multiplier - 1).max(1);
                })
                .background_color(Color::from_rgb8(150, 100, 200))
                .corner_radius(5.)
            )
            .width(40.px())
            .height(40.px()),
            FlexSpacer::Fixed(10.px()),
            sized_box(
                button("+", |s: &mut CounterAppState| {
                    s.multiplier += 1;
                })
                .background_color(Color::from_rgb8(150, 100, 200))
                .corner_radius(5.)
            )
            .width(40.px())
            .height(40.px()),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .gap(5.px()),
        
        FlexSpacer::Fixed(15.px()),
        
        // Toggle and reset section
        flex_row((
            button(
                if state.show_squared { "Show Multiplied" } else { "Show Squared" },
                |s: &mut CounterAppState| {
                    s.show_squared = !s.show_squared;
                }
            )
            .background_color(Color::from_rgb8(100, 150, 200))
            .corner_radius(5.)
            .flex(1.0),
            
            FlexSpacer::Fixed(10.px()),
            
            button("Reset", |s: &mut CounterAppState| {
                *s = CounterAppState::default();
            })
            .background_color(Color::from_rgb8(200, 150, 50))
            .corner_radius(5.)
            .flex(1.0),
        ))
        .gap(5.px()),
    ))
    .main_axis_alignment(MainAxisAlignment::Start)
    .cross_axis_alignment(CrossAxisAlignment::Fill)
    .gap(5.px())
}

/// Application entry point
fn main() -> Result<(), EventLoopError> {
    // 1. Create initial state
    let initial_state = CounterAppState::default();
    
    // 2. Build the Xilem app with:
    //    - Initial state
    //    - View logic function
    //    - Window configuration
    let app = Xilem::new_simple(
        initial_state,
        app_logic,
        WindowOptions::new("Xilem 0.4.0 Counter Example")
            .with_initial_inner_size(winit::dpi::LogicalSize::new(600.0, 500.0))
            .with_min_inner_size(winit::dpi::LogicalSize::new(400.0, 350.0)),
    );
    
    // 3. Run the application with event loop
    // This blocks until the window is closed
    app.run_in(EventLoop::with_user_event())?;
    
    Ok(())
}

// Key API patterns demonstrated in this example:
//
// 1. Application Setup:
//    - Xilem::new_simple(state, app_logic, WindowOptions)
//    - app.run_in(EventLoop::with_user_event())
//
// 2. View Function:
//    - fn app_logic(state: &mut State) -> impl WidgetView<State> + use<>
//
// 3. Flex Layouts:
//    - flex_col((child1, child2, child3))  // Vertical stacking
//    - flex_row((child1, child2, child3))  // Horizontal arrangement
//
// 4. Flex Spacing:
//    - FlexSpacer::Fixed(15.px())  // Fixed space
//    - FlexSpacer::Flex(1.0)       // Flexible space
//
// 5. Flex Alignment:
//    - .main_axis_alignment(MainAxisAlignment::Center)
//    - .cross_axis_alignment(CrossAxisAlignment::Center)
//    - .gap(10.px())
//
// 6. Sized Boxes:
//    - sized_box(widget).width(80.px()).height(80.px())
//    - sized_box(widget).expand()
//    - sized_box(widget).expand_width()
//
// 7. Button Handling:
//    - button("Label", |state: &mut State| { /* mutate state */ })
//
// 8. Text Styling:
//    - label("text").text_size(28.)
//    - button("text").text_size(20.).background_color(Color::...)
//
// 9. Flex Child Configuration:
//    - widget.flex(1.0)  // Apply flex factor
//
// 10. Colors:
//     - Color::from_rgb8(r, g, b)
//     - Color::WHITE, Color::BLACK, etc. (predefined constants)
