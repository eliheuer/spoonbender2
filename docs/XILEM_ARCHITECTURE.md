# Xilem Placehero Application Architecture Analysis

## Overview
Placehero is a Mastodon client built with Xilem that demonstrates idiomatic patterns for organizing a medium-scale GUI application in Rust. The application is split into separate modules that handle different concerns: state management, views, actions, and async operations.

---

## 1. File Organization

### Directory Structure
```
placehero/src/
├── main.rs              # Minimal entry point (11 lines)
├── lib.rs               # Main application logic and state management
├── actions.rs           # Action type definitions
├── avatars.rs           # Avatar resource management with caching
├── components.rs        # Exports view components
├── html_content.rs      # HTML parsing utilities
├── login_flow.rs        # Login feature (in development)
└── components/          # View submodules
    ├── timeline.rs      # Timeline view logic
    ├── thread.rs        # Thread/conversation view
    └── media.rs         # Media attachment rendering
```

### Cargo.toml Implications
- Single binary target (main.rs is the entry point)
- lib.rs contains the main `run()` function that's called from main
- Modular organization for easier testing and reuse

---

## 2. Entry Point Pattern (main.rs)

### Code Example
```rust
// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! The boilerplate run function for desktop platforms

use xilem::EventLoop;
use xilem::winit::error::EventLoopError;

fn main() -> Result<(), EventLoopError> {
    placehero::run(EventLoop::with_user_event())
}
```

### Key Principles
1. **Minimal Responsibility**: Only handles event loop creation and invokes `lib::run()`
2. **No Business Logic**: No state initialization, no view creation
3. **Error Propagation**: Simple error handling - returns Result directly
4. **Platform Abstraction**: Event loop is created in platform-independent way

---

## 3. Main Application Logic (lib.rs)

### State Definition Pattern
```rust
// Enum-based state routing for different application modes
enum MainState {
    Selecting,           // Initial welcome screen
    Old(Placehero),      // Anonymous browsing mode
    New(PlaceheroWithLogin),  // Login feature (in development)
}

// Main application state structure
struct Placehero {
    mastodon: Mastodon,              // API client (Arc for sharing)
    instance: Option<Instance>,      // Server info
    timeline: Option<Timeline>,      // Current timeline view state
    show_context: Option<Status>,    // Currently displayed thread
    context: Option<Context>,        // Thread data
    context_sender: Option<UnboundedSender<String>>,  // Async task channel
    account_sender: Option<UnboundedSender<String>>,  // Async task channel
    // ... other state fields
}
```

### Key Patterns

#### 1. Multi-Modal State with Enums
- Use enums to represent distinct application states/modes
- Each variant can hold completely different state structures
- `select_app()` function acts as the root view dispatcher

#### 2. Resource Sharing with Arc
```rust
type Mastodon = Arc<mastodon::Mastodon>;
```
- Mastodon API client wrapped in Arc for cheap cloning
- Allows shared access to API client across async tasks

#### 3. Channel-Based Async Communication
```rust
context_sender: Option<UnboundedSender<String>>,
account_sender: Option<UnboundedSender<String>>,
```
- Channels store senders for background tasks
- Receivers managed in worker/task views
- Decouples UI state updates from async operations

---

## 4. View Organization Pattern

### Root View Dispatcher
```rust
fn select_app(state: &mut MainState) -> impl WidgetView<MainState> + use<> {
    match state {
        MainState::Selecting => OneOf3::A(/* welcome screen */),
        MainState::Old(_) => OneOf::B(lens(app_logic, /* ... */)),
        MainState::New(_) => OneOf::C(lens(login_flow::app_logic, /* ... */)),
    }
}
```

### Pattern: lens() for State Extraction
```rust
lens(app_logic, |state| {
    let MainState::Old(placehero) = state else {
        unreachable!()
    };
    placehero
})
```
- `lens()` adapts view functions to work with nested state
- Extracts relevant state variant before passing to view function
- Allows view modules to operate on their specific state type

### Main View Composition
```rust
fn app_logic(app_state: &mut Placehero) -> impl WidgetView<Placehero> + use<> {
    Avatars::provide(fork(
        map_action(
            split(
                app_state.sidebar(),
                app_state.main_view()
            ).split_point(0.2),
            |state, action| { /* handle Navigation action */ }
        ),
        (
            load_instance(app_state.mastodon.clone()),
            load_account(app_state.mastodon.clone()),
            load_contexts(app_state.mastodon.clone()),
        ),
    ))
}
```

### Pattern: Separating Layout Concerns
View methods on state struct keep layout logic organized:
```rust
impl Placehero {
    fn sidebar(&mut self) -> impl WidgetView<Self, Navigation> + use<> {
        // Sidebar UI logic
    }

    fn main_view(&mut self) -> impl WidgetView<Self, Navigation> + use<> {
        // Main content UI logic
    }
}
```

---

## 5. Action/Navigation Pattern (actions.rs)

### Action Enum
```rust
pub(crate) enum Navigation {
    LoadContext(Status),  // Load thread
    LoadUser(String),     // Load user's timeline
    Home,                 // Return to main timeline
    None,                 // Hack for optional actions
}
```

### Usage Pattern
```rust
map_action(
    split(...),
    |state, action| match action {
        Navigation::LoadContext(status) => {
            state.context_sender.as_ref().unwrap().send(status.id.clone()).unwrap();
            state.show_context = Some(status);
            state.context = None;
        }
        Navigation::LoadUser(user) => {
            state.account_sender.as_ref().unwrap().send(user).unwrap();
            state.loading_timeline = true;
        }
        // ...
    },
)
```

### Key Principles
1. **Centralized Action Handling**: All state updates in one match statement
2. **Actions Are Separate Type**: Not mixed with state definitions
3. **Async Signaling**: Actions often trigger channel sends to background tasks
4. **Optional Actions**: `None` variant is a workaround for Xilem's optional handling

---

## 6. Component/View Modules Pattern (components/)

### File Structure
```
components/
├── timeline.rs     # Timeline data + view method
├── thread.rs       # Thread view function
└── media.rs        # Media attachment rendering
```

### Export Pattern (components.rs)
```rust
mod timeline;
pub(crate) use timeline::Timeline;

mod thread;
pub(crate) use thread::thread;

mod media;
```

### Pattern: Pub(crate) Exports
- Keep internals private within crate
- Only export what's needed from parent module
- Single re-export location (components.rs)

---

## 7. Async/Resource Pattern (avatars.rs)

### Resource-Based Caching
```rust
pub(crate) struct Avatars {
    icons: HashMap<String, Option<ImageData>>,
    requester: Option<UnboundedSender<AvatarRequest>>,
}

impl Resource for Avatars {}
```

### Providing Context
```rust
pub(crate) fn provide<State, Action, Child>(
    child: Child,
) -> impl WidgetView<State, Action, Element = Child::Element>
where
    Child: WidgetView<State, Action>,
    State: 'static,
    Action: 'static,
{
    provides(
        |_| Self {
            icons: HashMap::default(),
            requester: None,
        },
        fork(child, Self::worker()),
    )
}
```

### Pattern: Provider + Worker
- Provider creates resource and forks with worker task
- Worker manages async operations and updates resource
- Child views access resource via context

### Usage in Views
```rust
pub(crate) fn avatar<State: 'static, Action: 'static>(
    url: String,
) -> impl WidgetView<State, Action> + use<State, Action> {
    with_context(move |this: &mut Self, _: &mut State| {
        if let Some(maybe_image) = this.icons.get(&url) {
            // Render cached image
        } else if let Some(requester) = this.requester.as_ref() {
            // Queue image load
            drop(requester.send(AvatarRequest { avatar_url: url.to_string() }));
        }
        // Render placeholder while loading
    })
}
```

---

## 8. Async Task Pattern

### Task-Based Data Loading
```rust
fn load_instance(
    mastodon: Mastodon,
) -> impl View<Placehero, (), ViewCtx, Element = NoElement> + use<> {
    task_raw(
        move |result| {
            let mastodon = mastodon.clone();
            async move {
                let instance_result = mastodon.get_instance().await;
                drop(result.message(instance_result));
            }
        },
        |app_state: &mut Placehero, event| match event {
            Ok(instance) => app_state.instance = Some(instance.json),
            Err(e) => { /* error handling */ }
        },
    )
}
```

### Worker-Based Streaming Operations
```rust
fn load_account(
    mastodon: Mastodon,
) -> impl View<Placehero, (), ViewCtx, Element = NoElement> + use<> {
    worker_raw(
        move |result, mut recv: UnboundedReceiver<String>| {
            let mastodon = mastodon.clone();
            async move {
                while let Some(req) = recv.recv().await {
                    let result = mastodon.lookup_account(req.clone()).await;
                    drop(result.message((result, req)));
                }
            }
        },
        |app_state: &mut Placehero, sender| app_state.account_sender = Some(sender),
        |app_state: &mut Placehero, (event, acct)| match event {
            Ok(instance) => {
                app_state.timeline = Some(Timeline::new_for_account(instance.json));
                app_state.loading_timeline = false;
            }
            Err(e) => { /* error handling */ }
        },
    )
}
```

### Key Differences
- `task_raw`: For one-shot async operations that fire automatically
- `worker_raw`: For long-lived operations that respond to channel messages
- Both return `View` with `Element = NoElement` (invisible to UI)

---

## 9. State Update Patterns

### Conditional Rendering with OneOf
```rust
fn main_view(&mut self) -> impl WidgetView<Self, Navigation> + use<> {
    if let Some(show_context) = self.show_context.as_ref() {
        if let Some(context) = self.context.as_ref() {
            OneOf6::A(thread(show_context, context))
        } else {
            OneOf::B(prose("Loading thread"))
        }
    } else if self.loading_timeline {
        OneOf::C(flex_col(/* spinner */))
    } else if let Some(acct) = self.not_found_acct.as_ref() {
        OneOf::D(prose(format!("Could not find account @{acct}")))
    } else if let Some(timeline) = self.timeline.as_mut() {
        OneOf::E(map_state(timeline.view(...)))
    } else {
        OneOf::F(prose("No statuses yet loaded"))
    }
}
```

### Pattern Benefits
- Clear state-to-view mapping
- Only one branch renders at a time
- Type-safe through OneOfN enums
- State-driven UI (declarative)

---

## 10. Module Structure Summary

### lib.rs
- Contains all application state definitions
- Root view dispatcher (`select_app`, `app_logic`)
- Async task definitions
- High-level orchestration

### Module Separation
- `actions.rs` - Just the action enum definition
- `avatars.rs` - Self-contained resource with provider pattern
- `components.rs` - View function exports from submodules
- `components/` - Specific view implementations
- `login_flow.rs` - Feature-in-development isolation

### Separation Benefits
1. **Single Responsibility**: Each file has clear purpose
2. **Reusability**: Views can be easily moved/shared
3. **Testing**: Smaller units easier to test
4. **Maintainability**: Changes isolated to relevant files
5. **Feature Development**: New features can develop in isolation

---

## 11. Key Xilem-Specific Patterns

### 1. The `use<>` Syntax
```rust
fn view() -> impl WidgetView<State> + use<> { }
```
- Explicit lifetime/capture specification
- Allows compiler to reason about captured values

### 2. fork() for Side Effects
```rust
fork(main_view, async_tasks_tuple)
```
- Combines main view with invisible async operations
- Doesn't affect rendering, only state updates

### 3. Lens Pattern for State Adaptation
```rust
lens(view_fn, |state| extract_relevant_state(state))
```
- Adapts view functions to different state types
- Enables reusable views for different contexts

### 4. map_action() for Action Translation
```rust
map_action(split(...), |state, action| {
    // Update state based on action
})
```
- Centralizes action handling
- Keeps actions type-safe and documented

### 5. OneOf Types for Conditional Rendering
```rust
OneOf6::A(...) | OneOf::B(...) | OneOf::C(...)
```
- Type-safe conditional rendering
- Only valid branches compile

---

## 12. Refactoring Guide for Spoonbender

### Current Issues to Address
1. All views in main.rs - leads to large, hard-to-navigate file
2. State management mixed with view logic
3. Unclear separation of concerns
4. Difficult to test individual components

### Recommended Structure for Spoonbender
```
src/
├── main.rs              # Minimal entry point
├── lib.rs               # State definitions + root dispatcher
├── actions.rs           # Action types
├── state/
│   ├── editor.rs        # Editor state
│   ├── document.rs      # Document state
│   └── selection.rs     # Selection state
├── views/
│   ├── sidebar.rs       # Sidebar view
│   ├── canvas.rs        # Canvas/glyph editor view
│   ├── properties.rs     # Properties panel
│   └── menu.rs          # Menu bar
├── components/
│   ├── glyph_widget.rs
│   ├── outline.rs
│   └── metrics.rs
└── async_tasks/         # Background operations
    ├── loader.rs        # File loading
    ├── saver.rs         # File saving
    └── metrics.rs       # Metric calculations
```

### Step-by-Step Refactoring
1. Extract all state structs to `lib.rs`
2. Create action enum in `actions.rs`
3. Separate view functions by UI area (sidebar, main, etc.)
4. Extract async operations to dedicated modules
5. Create component modules for reusable UI pieces
6. Keep `main.rs` as minimal entry point

### Patterns to Adopt
- Use `lens()` to adapt views to nested state
- Use `fork()` to attach async tasks
- Use `map_action()` to handle all navigation in one place
- Separate state definition from view logic
- Use `pub(crate)` for module privacy boundaries
- Export view functions from parent modules

---

## 13. Code Example: Sidebar Pattern

### Pattern from Placehero
```rust
impl Placehero {
    fn sidebar(&mut self) -> impl WidgetView<Self, Navigation> + use<> {
        if let Some(instance) = &self.instance {
            Either::A(flex_col((
                label("Connected to:"),
                prose(instance.title.as_str()),
                // Dynamic content based on state
                if self.show_context.is_some() {
                    Either::A(text_button("⬅️ Back to Timeline", |_| Navigation::Home))
                } else {
                    Either::B(flex_col(/* search interface */))
                }
            )))
        } else {
            Either::B(prose("Not yet connected"))
        }
    }
}
```

### For Spoonbender Font Editor
```rust
impl EditorState {
    fn sidebar(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
        Either::A(flex_col((
            label("Glyphs"),
            // Glyph list
            self.glyph_list_view(),
            
            if let Some(glyph) = &self.selected_glyph {
                Either::A(flex_col((
                    label("Properties"),
                    self.glyph_properties_view(),
                )))
            } else {
                Either::B(prose("Select a glyph"))
            }
        )))
    }

    fn main_view(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
        if let Some(glyph) = &mut self.selected_glyph {
            OneOf2::A(self.canvas_view())
        } else {
            OneOf::B(prose("No glyph selected"))
        }
    }
}
```

---

## Summary

Placehero demonstrates that idiomatic Xilem applications:

1. **Keep main.rs minimal** - Just event loop setup and calling `lib::run()`
2. **Define state as enums and structs** - Separate from view logic
3. **Use a root view dispatcher** - Routes to different UI modes
4. **Separate views by concern** - Sidebar, main content, etc. as methods
5. **Use actions for state updates** - Central match statement for all navigation
6. **Isolate async operations** - `task_raw`/`worker_raw` with channels
7. **Organize modules by feature** - Views, components, async tasks in separate files
8. **Provide context/resources** - For shared state like avatar cache
9. **Use lens/fork/map_action** - Xilem combinators for composing views
10. **Maintain clear boundaries** - Use pub(crate) to control visibility

This structure makes the codebase more maintainable, testable, and easier to expand as new features are added.
