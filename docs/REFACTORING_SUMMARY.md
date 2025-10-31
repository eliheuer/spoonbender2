# Spoonbender Refactoring Guide - Based on Xilem Placehero Patterns

## Quick Start Summary

Placehero demonstrates the ideal structure for a Xilem application with clean separation of concerns. Your Spoonbender application should follow these same patterns to improve maintainability and testability.

---

## Current Problems in Spoonbender

As identified in your request:
1. **All views in main.rs** - Makes the file large and hard to navigate
2. **Mixed state and view logic** - Difficult to understand state flow
3. **Unclear separation of concerns** - Hard to find where changes should go
4. **Difficult to test** - Monolithic structure prevents unit testing

---

## Target Architecture

After refactoring, Spoonbender should look like this:

```
spoonbender/src/
├── main.rs                      # 10 lines - entry point only
├── lib.rs                        # 400-500 lines - state + root view logic
├── actions.rs                    # 30-50 lines - action enum
│
├── views/                        # View functions organized by UI area
│   ├── sidebar.rs               # Glyph list, properties sidebar
│   ├── canvas.rs                # Drawing canvas area
│   ├── menu.rs                  # Menu bar
│   └── properties.rs            # Properties panel
│
├── components/                   # Reusable UI components
│   ├── glyph_widget.rs          # Single glyph display widget
│   ├── outline.rs               # Outline/contour display
│   └── metrics.rs               # Metrics display
│
└── async_tasks/                  # Background operations
    ├── loader.rs                # UFO file loading
    ├── saver.rs                 # UFO file saving
    └── metrics.rs               # Metric calculations
```

---

## Refactoring Steps

### Phase 1: Extract State (High Priority)

1. **Create lib.rs** with state definitions:
   ```rust
   // All state structs moved from main.rs to lib.rs
   pub struct EditorState {
       document: Document,
       selected_glyph: Option<GlyphId>,
       viewport: ViewportState,
       // ... other fields
   }

   // Entry point function
   pub fn run(event_loop: EventLoopBuilder) -> Result<(), EventLoopError> {
       Xilem::new_simple(
           EditorState::default(),
           app_view,
           WindowOptions::new("Spoonbender"),
       )
       .run_in(event_loop)
   }
   ```

2. **Minimize main.rs**:
   ```rust
   use xilem::EventLoop;
   use xilem::winit::error::EventLoopError;

   fn main() -> Result<(), EventLoopError> {
       spoonbender::run(EventLoop::with_user_event())
   }
   ```

3. **Create actions.rs** with action enum:
   ```rust
   pub(crate) enum EditorAction {
       SelectGlyph(GlyphId),
       DeselectGlyph,
       EditPoint(PointEdit),
       Save,
       Load(String),
       // ... other actions
   }
   ```

### Phase 2: Create Root View Dispatcher

In lib.rs, create the root view function:

```rust
fn app_view(state: &mut EditorState) -> impl WidgetView<EditorState> + use<> {
    // This is the entry point for all view logic
    // It handles the overall layout and coordinates with child views
    
    fork(
        map_action(
            split(
                state.sidebar_view(),
                state.canvas_view()
            ).split_point(0.2),
            |state, action| {
                // Central action handler - all state updates here
                match action {
                    EditorAction::SelectGlyph(id) => {
                        state.selected_glyph = Some(id);
                    }
                    EditorAction::DeselectGlyph => {
                        state.selected_glyph = None;
                    }
                    // ... other actions
                }
            }
        ),
        (
            // Async background tasks
            load_document_task(),
            save_document_task(),
        ),
    )
}
```

### Phase 3: Separate Views by Area

Create `views/` directory with separate files:

**views/sidebar.rs**:
```rust
use crate::{EditorState, EditorAction};

pub fn sidebar(state: &mut EditorState) -> impl WidgetView<EditorState, EditorAction> + use<> {
    flex_col((
        flex_col(/* glyph list */).flex(1.0),
        flex_col(/* properties panel */),
    ))
}
```

**views/canvas.rs**:
```rust
pub fn canvas(state: &mut EditorState) -> impl WidgetView<EditorState, EditorAction> + use<> {
    // Drawing canvas logic
}
```

Add to lib.rs:
```rust
mod views;
use views::{sidebar, canvas};

impl EditorState {
    fn sidebar_view(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
        sidebar(self)
    }

    fn canvas_view(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
        canvas(self)
    }
}
```

### Phase 4: Extract Components

Create `components/` directory:

**components/glyph_widget.rs**:
```rust
pub fn glyph_widget<State: 'static, Action: 'static>(
    glyph: &Glyph,
) -> impl WidgetView<State, Action> + use<State, Action> {
    // Display a single glyph
}
```

Use from views:
```rust
// In sidebar view
glyph_widget(&glyph)
```

### Phase 5: Extract Async Operations

Create `async_tasks/` directory:

**async_tasks/loader.rs**:
```rust
pub fn load_document_task() 
    -> impl View<EditorState, (), ViewCtx, Element = NoElement> + use<> {
    task_raw(
        |result| {
            async move {
                // Load UFO file
                let doc = load_ufo("path/to/file.ufo").await;
                drop(result.message(doc));
            }
        },
        |state: &mut EditorState, doc| {
            state.document = doc;
        },
    )
}
```

Use in app_view:
```rust
fork(
    map_action(/* ... */),
    (
        load_document_task(),
        save_document_task(),
    ),
)
```

### Phase 6: Module Organization

**lib.rs** file structure:
```rust
// 1. Imports
use xilem::{...};

// 2. Module declarations
mod actions;
mod views;
mod components;
mod async_tasks;

// 3. State definitions
pub struct EditorState { ... }
impl Default for EditorState { ... }
impl EditorState {
    fn sidebar_view(&mut self) -> ... { ... }
    fn canvas_view(&mut self) -> ... { ... }
}

// 4. Entry point
pub fn run(event_loop: EventLoopBuilder) -> Result<(), EventLoopError> { ... }

// 5. Root view
fn app_view(state: &mut EditorState) -> impl WidgetView<EditorState> + use<> { ... }

// 6. Async tasks
fn load_document_task() -> ... { ... }
fn save_document_task() -> ... { ... }
```

---

## Key Patterns to Adopt

### 1. Central Action Handling

All state updates happen in ONE place - the map_action closure in app_view:

```rust
map_action(/* view */, |state, action| match action {
    EditorAction::SelectGlyph(id) => state.selected_glyph = Some(id),
    EditorAction::Save => save_sender.send(SaveRequest {}).unwrap(),
    EditorAction::Load(path) => load_sender.send(LoadRequest { path }).unwrap(),
    // ... all state mutations here
})
```

**Benefits**:
- Find all state changes in one place
- Easy to understand state flow
- Prevents scattered logic

### 2. View Methods on State

Keep view logic organized by putting view methods on the state struct:

```rust
impl EditorState {
    fn sidebar_view(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
        // Sidebar-specific logic
    }

    fn canvas_view(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
        // Canvas-specific logic
    }
}
```

**Benefits**:
- Grouped by UI area
- Can access all state directly
- Easier to refactor later

### 3. Conditional Rendering with OneOf

Instead of complex if-let chains, use OneOf types:

```rust
fn canvas_view(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
    if let Some(glyph) = &self.selected_glyph {
        if self.is_editing {
            OneOf3::A(/* editing view */)
        } else {
            OneOf3::B(/* preview view */)
        }
    } else {
        OneOf3::C(/* empty state */)
    }
}
```

**Benefits**:
- Type-safe (compiler ensures consistency)
- Clear state-to-view mapping
- State drives UI

### 4. Resource Providers for Shared State

For things like glyph cache, use Resource providers:

```rust
pub struct GlyphCache {
    cache: HashMap<GlyphId, CachedGlyph>,
}

impl GlyphCache {
    pub fn provide<Child>(child: Child) 
        -> impl WidgetView<State, Action>
    where
        Child: WidgetView<State, Action>
    {
        provides(
            |_| Self { cache: HashMap::new() },
            child,
        )
    }
}
```

Then use with_context in views to access.

### 5. Async Operations with Fork

Keep async operations separate from UI logic:

```rust
fork(
    map_action(/* main view */, /* action handler */),
    (
        load_task(),
        save_task(),
        metrics_task(),
    ),
)
```

**Benefits**:
- Async logic doesn't clutter UI
- Easy to add/remove tasks
- Clear separation of concerns

---

## Migration Checklist

As you refactor, check off these items:

### Initial Setup
- [ ] Create lib.rs with state definitions
- [ ] Create actions.rs with action enum
- [ ] Minimize main.rs to 10 lines

### State Organization
- [ ] Move all state structs to lib.rs
- [ ] Implement Default for main state
- [ ] Create pub fn run() in lib.rs

### View Organization
- [ ] Create root view dispatcher (app_view)
- [ ] Create map_action for central state updates
- [ ] Move sidebar logic to methods/modules
- [ ] Move canvas logic to methods/modules
- [ ] Add remaining view methods as needed

### Module Separation
- [ ] Create views/ directory
- [ ] Create components/ directory
- [ ] Create async_tasks/ directory
- [ ] Move view logic to appropriate modules

### Async Operations
- [ ] Identify async operations (load, save, etc.)
- [ ] Create async task functions
- [ ] Add tasks to fork() in app_view
- [ ] Update action handler to trigger tasks via channels

### Code Quality
- [ ] Add documentation comments
- [ ] Add module-level doc comments
- [ ] Organize exports with pub(crate)
- [ ] Clean up compiler warnings

---

## Estimated Effort

- **Phase 1 (State Extraction)**: 2-4 hours
- **Phase 2 (Root View)**: 2-3 hours
- **Phase 3 (View Separation)**: 3-5 hours
- **Phase 4 (Components)**: 2-4 hours
- **Phase 5 (Async Tasks)**: 2-3 hours
- **Phase 6 (Polish)**: 1-2 hours

**Total**: 12-21 hours (1-3 days of focused work)

---

## Testing Strategy

After refactoring, you can:

1. **Test state logic independently**:
   ```rust
   #[test]
   fn test_select_glyph() {
       let mut state = EditorState::default();
       // Manually call action handler
       state.selected_glyph = Some(GlyphId(1));
       assert_eq!(state.selected_glyph, Some(GlyphId(1)));
   }
   ```

2. **Test view functions independently**:
   ```rust
   #[test]
   fn test_sidebar_renders() {
       let mut state = EditorState::default();
       let _view = state.sidebar_view();
       // Compile check - view builds without errors
   }
   ```

3. **Test async tasks independently**:
   ```rust
   #[tokio::test]
   async fn test_load_document() {
       let doc = load_ufo("test.ufo").await;
       assert!(!doc.glyphs.is_empty());
   }
   ```

---

## Benefits After Refactoring

1. **Maintainability**: Changes isolated to relevant files
2. **Testability**: Can test components independently
3. **Scalability**: Easy to add new features
4. **Clarity**: Clear separation of concerns
5. **Performance**: Possible to optimize specific modules
6. **Reusability**: Views and components can be moved/shared

---

## Learning Resources

1. **Xilem Documentation**: https://github.com/linebender/xilem
2. **Placehero Example**: https://github.com/linebender/xilem/tree/main/placehero
3. **Masonry (Layout)**: https://github.com/linebender/masonry-rs
4. **Druid (UI Framework)**: https://github.com/linebender/druid

---

## Next Steps

1. Start with Phase 1: Create lib.rs and move state
2. Create root view dispatcher (app_view)
3. Test compilation at each step
4. Gradually move views to separate modules
5. Extract async operations
6. Polish and optimize

The detailed documentation files provide:
- **XILEM_ARCHITECTURE.md**: Complete architectural overview
- **PLACEHERO_STRUCTURE.txt**: Visual structure diagrams
- **XILEM_CODE_PATTERNS.md**: Exact code examples to follow

Use these as references when implementing each phase.
