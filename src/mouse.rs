// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Mouse event handling and state machine
//!
//! This module provides a state machine for tracking mouse events and
//! converting them into high-level gestures (clicks, drags, etc.).

use kurbo::Point;

/// Threshold distance (in screen pixels) before a drag is recognized
const DRAG_THRESHOLD: f64 = 3.0;

/// Mouse button states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Other,
}

/// Modifier keys state
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// A mouse event with position
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    /// Position in screen coordinates
    pub pos: Point,
    /// Which button (if any)
    pub button: Option<MouseButton>,
    /// Modifier keys
    pub mods: Modifiers,
}

impl MouseEvent {
    /// Create a new mouse event
    pub fn new(pos: Point, button: Option<MouseButton>) -> Self {
        Self {
            pos,
            button,
            mods: Modifiers::default(),
        }
    }

    /// Create a new mouse event with modifiers
    pub fn with_modifiers(pos: Point, button: Option<MouseButton>, mods: Modifiers) -> Self {
        Self { pos, button, mods }
    }
}

/// Information about a drag gesture
#[derive(Debug, Clone, Copy)]
pub struct Drag {
    /// Where the drag started
    pub start: Point,
    /// Previous mouse position
    pub prev: Point,
    /// Current mouse position
    pub current: Point,
}

impl Drag {
    /// Get the delta from start to current
    pub fn delta_from_start(&self) -> (f64, f64) {
        (self.current.x - self.start.x, self.current.y - self.start.y)
    }

    /// Get the delta from previous to current
    pub fn delta_from_prev(&self) -> (f64, f64) {
        (self.current.x - self.prev.x, self.current.y - self.prev.y)
    }
}

/// Mouse state machine internal state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MouseState {
    /// No buttons pressed
    Up,
    /// Button pressed but not dragging yet
    Down,
    /// Dragging (moved beyond threshold)
    Drag,
}

/// Mouse state machine
///
/// Tracks mouse events and converts them into high-level gestures.
pub struct Mouse {
    /// Current state
    state: MouseState,
    /// Button that is currently down (if any)
    current_button: Option<MouseButton>,
    /// Position where button was pressed
    down_pos: Point,
    /// Last known mouse position
    last_pos: Point,
}

impl Mouse {
    /// Create a new mouse state machine
    pub fn new() -> Self {
        Self {
            state: MouseState::Up,
            current_button: None,
            down_pos: Point::ZERO,
            last_pos: Point::ZERO,
        }
    }

    /// Handle a mouse down event
    pub fn mouse_down<T: MouseDelegate>(
        &mut self,
        event: MouseEvent,
        delegate: &mut T,
        data: &mut T::Data,
    ) {
        match self.state {
            MouseState::Up => {
                self.state = MouseState::Down;
                self.current_button = event.button;
                self.down_pos = event.pos;
                self.last_pos = event.pos;

                // Call appropriate delegate method
                match event.button {
                    Some(MouseButton::Left) => delegate.left_down(event, data),
                    Some(MouseButton::Right) => delegate.right_down(event, data),
                    Some(MouseButton::Other) => delegate.other_down(event, data),
                    None => {}
                }
            }
            _ => {
                // Ignore if already down or dragging
            }
        }
    }

    /// Handle a mouse up event
    pub fn mouse_up<T: MouseDelegate>(
        &mut self,
        event: MouseEvent,
        delegate: &mut T,
        data: &mut T::Data,
    ) {
        match self.state {
            MouseState::Down => {
                // It was a click (not a drag)
                match event.button {
                    Some(MouseButton::Left) => {
                        delegate.left_up(event, data);
                        delegate.left_click(event, data);
                    }
                    Some(MouseButton::Right) => {
                        delegate.right_up(event, data);
                        delegate.right_click(event, data);
                    }
                    Some(MouseButton::Other) => {
                        delegate.other_up(event, data);
                        delegate.other_click(event, data);
                    }
                    None => {}
                }
                self.state = MouseState::Up;
                self.current_button = None;
            }
            MouseState::Drag => {
                // End of drag
                let drag = Drag {
                    start: self.down_pos,
                    prev: self.last_pos,
                    current: event.pos,
                };

                match self.current_button {
                    Some(MouseButton::Left) => {
                        delegate.left_drag_ended(event, drag, data);
                        delegate.left_up(event, data);
                    }
                    Some(MouseButton::Right) => {
                        delegate.right_drag_ended(event, drag, data);
                        delegate.right_up(event, data);
                    }
                    Some(MouseButton::Other) => {
                        delegate.other_drag_ended(event, drag, data);
                        delegate.other_up(event, data);
                    }
                    None => {}
                }

                self.state = MouseState::Up;
                self.current_button = None;
            }
            MouseState::Up => {
                // Ignore up event when already up
            }
        }
    }

    /// Handle a mouse move event
    pub fn mouse_moved<T: MouseDelegate>(
        &mut self,
        event: MouseEvent,
        delegate: &mut T,
        data: &mut T::Data,
    ) {
        match self.state {
            MouseState::Up => {
                // Just moving, no button pressed
                self.last_pos = event.pos;
                delegate.mouse_moved(event, data);
            }
            MouseState::Down => {
                // Check if we've moved far enough to start a drag
                let delta_x = event.pos.x - self.down_pos.x;
                let delta_y = event.pos.y - self.down_pos.y;
                let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

                if distance >= DRAG_THRESHOLD {
                    // Start dragging
                    self.state = MouseState::Drag;

                    let drag = Drag {
                        start: self.down_pos,
                        prev: self.last_pos,
                        current: event.pos,
                    };

                    match self.current_button {
                        Some(MouseButton::Left) => delegate.left_drag_began(event, drag, data),
                        Some(MouseButton::Right) => delegate.right_drag_began(event, drag, data),
                        Some(MouseButton::Other) => delegate.other_drag_began(event, drag, data),
                        None => {}
                    }
                }

                self.last_pos = event.pos;
            }
            MouseState::Drag => {
                // Continue dragging
                let drag = Drag {
                    start: self.down_pos,
                    prev: self.last_pos,
                    current: event.pos,
                };

                match self.current_button {
                    Some(MouseButton::Left) => delegate.left_drag_changed(event, drag, data),
                    Some(MouseButton::Right) => delegate.right_drag_changed(event, drag, data),
                    Some(MouseButton::Other) => delegate.other_drag_changed(event, drag, data),
                    None => {}
                }

                self.last_pos = event.pos;
            }
        }
    }

    /// Cancel any ongoing gesture
    pub fn cancel<T: MouseDelegate>(&mut self, delegate: &mut T, data: &mut T::Data) {
        delegate.cancel(data);
        self.state = MouseState::Up;
        self.current_button = None;
    }

    /// Get the current mouse position
    pub fn pos(&self) -> Point {
        self.last_pos
    }
}

impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for handling high-level mouse events
///
/// Tools implement this trait to respond to clicks, drags, etc.
pub trait MouseDelegate {
    /// The data type that mouse events operate on
    type Data;

    /// Left mouse button pressed
    fn left_down(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Left mouse button released
    fn left_up(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Left mouse button clicked (down and up without drag)
    fn left_click(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Left mouse drag began (moved beyond threshold)
    fn left_drag_began(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Left mouse drag continued (mouse moved while dragging)
    fn left_drag_changed(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Left mouse drag ended (button released after drag)
    fn left_drag_ended(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Right mouse button pressed
    fn right_down(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Right mouse button released
    fn right_up(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Right mouse button clicked
    fn right_click(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Right mouse drag began
    fn right_drag_began(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Right mouse drag continued
    fn right_drag_changed(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Right mouse drag ended
    fn right_drag_ended(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Other mouse button pressed
    fn other_down(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Other mouse button released
    fn other_up(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Other mouse button clicked
    fn other_click(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Other mouse drag began
    fn other_drag_began(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Other mouse drag continued
    fn other_drag_changed(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Other mouse drag ended
    fn other_drag_ended(&mut self, _event: MouseEvent, _drag: Drag, _data: &mut Self::Data) {}

    /// Mouse moved (no button pressed)
    fn mouse_moved(&mut self, _event: MouseEvent, _data: &mut Self::Data) {}

    /// Cancel any ongoing gesture
    fn cancel(&mut self, _data: &mut Self::Data) {}
}
