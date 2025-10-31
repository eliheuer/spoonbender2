# Xilem Placehero - Quick Reference Card

## The 8 Core Patterns

### 1. Minimal Entry Point
```rust
// src/main.rs - Just 10 lines
fn main() -> Result<(), EventLoopError> {
    spoonbender::run(EventLoop::with_user_event())
}
```

### 2. State Definition
```rust
// src/lib.rs - Define state, not in main.rs
pub struct AppState {
    // All your state here
}
```

### 3. Root Dispatcher
```rust
// src/lib.rs - Routes between UI modes
fn app_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    match state {
        AppState::Mode1 => OneOf::A(/* ... */),
        AppState::Mode2 => OneOf::B(/* ... */),
    }
}
```

### 4. Main Layout
```rust
// src/lib.rs - Assembles the UI
fn main_view(state: &mut AppState) -> impl WidgetView<AppState> + use<> {
    fork(
        map_action(
            split(state.sidebar(), state.canvas()).split_point(0.2),
            |state, action| { /* handle actions */ }
        ),
        (load_task(), save_task()) // Async workers
    )
}
```

### 5. Central Action Handling
```rust
// All state updates in ONE place
map_action(view, |state, action| match action {
    Action::SelectGlyph(id) => state.selected = Some(id),
    Action::Save => save_sender.send(data),
    // ...
})
```

### 6. View Methods
```rust
// src/lib.rs - View logic grouped on state struct
impl AppState {
    fn sidebar(&mut self) -> impl WidgetView<Self, Action> + use<> { }
    fn canvas(&mut self) -> impl WidgetView<Self, Action> + use<> { }
}
```

### 7. Conditional Rendering
```rust
// Use OneOf for type-safe conditional views
if condition {
    OneOf3::A(view1)
} else if condition2 {
    OneOf::B(view2)
} else {
    OneOf::C(view3)
}
```

### 8. Async Operations
```rust
// src/async_tasks/loader.rs - Separate async logic
fn load_task() -> impl View<AppState, (), ViewCtx, Element = NoElement> {
    task_raw(
        |result| async { /* async work */ result.message(data) },
        |state, data| { state.data = Some(data); } // Update state
    )
}
```

---

## File Organization Template

```
src/
├── main.rs                  # 10 lines: entry point
├── lib.rs                   # 400-500 lines: state + orchestration
├── actions.rs               # 30 lines: action enum
├── views/
│   ├── sidebar.rs          # View function
│   ├── canvas.rs           # View function
│   └── menu.rs             # View function
├── components/
│   ├── glyph_widget.rs     # Reusable component
│   └── outline.rs          # Reusable component
└── async_tasks/
    ├── loader.rs           # Load operations
    └── saver.rs            # Save operations
```

---

## State Update Flow

```
User clicks button
    ↓
View closure captures Action
    ↓
Action sent to map_action handler
    ↓
match action { ... } updates state
    ↓
Xilem detects state change
    ↓
Re-render affected views
    ↓
Display updates on screen
```

---

## Key Xilem Functions

| Function | Purpose | Location |
|----------|---------|----------|
| `fork()` | Combine view with async tasks | app_logic |
| `map_action()` | Central state update handler | app_logic |
| `split()` | Two-panel layout | app_logic |
| `lens()` | Adapt view to nested state | root dispatcher |
| `OneOf/OneOf2/...OneOf6` | Type-safe conditional | views |
| `provides()` | Inject resource into context | resource setup |
| `with_context()` | Access injected resource | views |
| `task_raw()` | One-shot async operation | async_tasks |
| `worker_raw()` | Streaming async operation | async_tasks |

---

## Common Patterns

### Conditional Rendering
```rust
if let Some(value) = &self.field {
    OneOf2::A(view_for_some)
} else {
    OneOf::B(view_for_none)
}
```

### Triggering Async Work
```rust
// In action handler
Action::LoadFile(path) => {
    state.load_sender.send(LoadRequest { path }).unwrap();
}
```

### Creating View Method
```rust
impl AppState {
    fn my_view(&mut self) -> impl WidgetView<Self, Action> + use<> {
        flex_col((/* view elements */))
    }
}
```

### Reusable Component
```rust
fn my_component<State, Action>(data: &Data) 
    -> impl WidgetView<State, Action> + use<State, Action> 
{
    flex_row((/* view elements */))
}
```

### Resource Provider
```rust
pub struct MyCache { /* state */ }
impl MyCache {
    pub fn provide<Child>(child: Child) -> impl WidgetView { }
}

// Use: MyCache::provide(fork(main_view, worker()))
```

---

## Refactoring Checklist (Quick Version)

Phase 1: State (2-4 hours)
- [ ] Create lib.rs
- [ ] Move state structs to lib.rs
- [ ] Create pub fn run()

Phase 2: Root View (2-3 hours)
- [ ] Create app_view dispatcher
- [ ] Create map_action handler
- [ ] Create fork with async tasks

Phase 3: View Organization (3-5 hours)
- [ ] Create views/ directory
- [ ] Create sidebar.rs, canvas.rs
- [ ] Add view methods to state

Phase 4: Components (2-4 hours)
- [ ] Create components/ directory
- [ ] Extract reusable pieces

Phase 5: Async (2-3 hours)
- [ ] Create async_tasks/ directory
- [ ] Move async logic to separate files

Phase 6: Polish (1-2 hours)
- [ ] Add documentation
- [ ] Clean up exports
- [ ] Verify compilation

---

## Estimated Lines of Code

Based on Placehero:

- main.rs: 10-15 lines
- lib.rs: 400-500 lines
- actions.rs: 20-50 lines
- views/sidebar.rs: 100-200 lines
- views/canvas.rs: 200-400 lines
- components/*.rs: 50-150 lines each
- async_tasks/*.rs: 100-200 lines each

Total: 1500-2000 lines organized modularly

---

## Common Gotchas

1. **Don't put state updates in view functions**
   - Bad: Text button that mutates state directly
   - Good: Button returns Action, handled centrally

2. **Don't mix async logic with UI**
   - Bad: Async code in view functions
   - Good: Async in task_raw/worker_raw, UI in views

3. **Don't put all views in one file**
   - Bad: 2000 lines in lib.rs
   - Good: Views in separate modules

4. **Don't use main.rs for logic**
   - Bad: State initialization in main
   - Good: State in lib.rs, main just calls run()

5. **Don't forget OneOf for conditionals**
   - Bad: Complex if-let chains that might not compile
   - Good: OneOf guarantees type safety

---

## Documentation Files Reference

| File | Read Time | Purpose |
|------|-----------|---------|
| XILEM_DOCS_INDEX.md | 5 min | Navigation guide |
| REFACTORING_SUMMARY.md | 30 min | Overview and planning |
| XILEM_CODE_PATTERNS.md | 30 min | Code examples |
| XILEM_ARCHITECTURE.md | 45 min | Deep understanding |
| PLACEHERO_STRUCTURE.txt | 20 min | Visual diagrams |

Start with XILEM_DOCS_INDEX.md for navigation.

---

## Getting Help

1. Example code? Check XILEM_CODE_PATTERNS.md
2. How things connect? Check PLACEHERO_STRUCTURE.txt
3. Deep understanding? Check XILEM_ARCHITECTURE.md
4. Navigation? Check XILEM_DOCS_INDEX.md
5. What to do next? Check REFACTORING_SUMMARY.md

---

**Print this card and keep it handy while refactoring!**
