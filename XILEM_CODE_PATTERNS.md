# Xilem Code Patterns - Placehero Examples

This document contains exact code examples from Placehero demonstrating idiomatic Xilem patterns.

## 1. Minimal main.rs

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

Key points:
- No imports beyond xilem essentials
- No state initialization in main
- Simple result propagation
- Single responsibility: create event loop and pass to lib


## 2. State Definition in lib.rs

```rust
/// We are developing a version of Placehero which supports login.
///
/// This requires some quite gnarly refactors, so for now we built it "alongside"
/// the main Placehero.
#[expect(clippy::large_enum_variant, reason = "Not passed around.")]
enum MainState {
    Selecting,
    Old(Placehero),
    New(PlaceheroWithLogin),
}

struct Placehero {
    mastodon: Mastodon,
    instance: Option<Instance>,
    timeline: Option<Timeline>,
    show_context: Option<Status>,
    context: Option<Context>,
    context_sender: Option<UnboundedSender<String>>,
    account_sender: Option<UnboundedSender<String>>,
    timeline_box_contents: String,
    loading_timeline: bool,
    not_found_acct: Option<String>,
}

impl Default for Placehero {
    fn default() -> Self {
        let base_url = "https://mastodon.online".to_string();
        let user_agent = None;

        let mastodon = mastodon::Mastodon::new(base_url, None, user_agent)
            .expect("Provided User Agent is valid");

        Self {
            mastodon: Arc::new(mastodon),
            instance: None,
            timeline: None,
            show_context: None,
            context: None,
            context_sender: None,
            account_sender: None,
            timeline_box_contents: "raph".to_string(),
            loading_timeline: false,
            not_found_acct: None,
        }
    }
}
```

Key points:
- Enum represents distinct application modes
- Each variant can hold different state
- Main state struct holds all current data
- Optional channels for async communication
- Default implementation handles initialization


## 3. Root View Dispatcher (select_app)

```rust
/// Execute the app in the given winit event loop.
pub fn run(event_loop: EventLoopBuilder) -> Result<(), EventLoopError> {
    Xilem::new_simple(
        MainState::Selecting,
        select_app,
        WindowOptions::new("Placehero: A placeholder named Mastodon client"),
    )
    .run_in(event_loop)
}

fn select_app(state: &mut MainState) -> impl WidgetView<MainState> + use<> {
    match state {
        MainState::Selecting => OneOf3::A(
            flex_col((
                prose("Welcome to Placehero. This is an example of the Xilem GUI framework, which is a Mastodon client.\n\
                    We currently have decent support for browsing anonymously, and are currently developing our logged-in support in parallel to avoid regressions.")
                    .text_alignment(xilem::TextAlign::Center)
                    .flex(CrossAxisAlignment::Center),
                flex_row((
                    text_button("Browse Anonymously", |state: &mut MainState| {
                        *state = MainState::Old(Placehero::default());
                    }),
                    text_button("Log In", |state: &mut MainState| {
                        *state = MainState::New(PlaceheroWithLogin::new());
                    }),
                ))
                .main_axis_alignment(xilem::view::MainAxisAlignment::Center),
            ))
            .main_axis_alignment(xilem::view::MainAxisAlignment::Center),
        ),
        MainState::Old(_) => OneOf::B(lens(app_logic, |state| {
            let MainState::Old(placehero) = state else {
                unreachable!()
            };
            placehero
        })),
        MainState::New(_) => OneOf::C(lens(login_flow::app_logic, |state| {
            let MainState::New(placehero) = state else {
                unreachable!()
            };
            placehero
        })),
    }
}
```

Key points:
- Root function creates Xilem with root view
- Root view matches on MainState enum
- Each branch returns different OneOf variant
- Uses lens() to adapt view functions to nested state


## 4. Main View Composition (app_logic)

```rust
fn app_logic(app_state: &mut Placehero) -> impl WidgetView<Placehero> + use<> {
    Avatars::provide(fork(
        map_action(
            split(app_state.sidebar(), app_state.main_view()).split_point(0.2),
            |state, action| match action {
                Navigation::LoadContext(status) => {
                    state
                        .context_sender
                        .as_ref()
                        .unwrap()
                        .send(status.id.clone())
                        .unwrap();
                    state.show_context = Some(status);
                    state.context = None;
                }
                Navigation::LoadUser(user) => {
                    state.timeline_box_contents = user.clone();
                    state.account_sender.as_ref().unwrap().send(user).unwrap();
                    state.loading_timeline = true;
                    state.context = None;
                    state.show_context = None;
                }
                Navigation::Home => {
                    state.context = None;
                    state.show_context = None;
                }
                Navigation::None => {}
            },
        ),
        (
            load_instance(app_state.mastodon.clone()),
            load_account(app_state.mastodon.clone()),
            load_contexts(app_state.mastodon.clone()),
        ),
    ))
}
```

Key points:
- Avatars::provide wraps entire layout
- fork() combines visible views with invisible workers
- map_action() centralizes all state updates
- split() creates two-panel layout
- Async tasks in tuple are invisible but update state
- Actions trigger channel sends to workers


## 5. View Methods (Sidebar and Main)

```rust
impl Placehero {
    fn sidebar(&mut self) -> impl WidgetView<Self, Navigation> + use<> {
        if let Some(instance) = &self.instance {
            let back = if self.show_context.is_some() {
                Either::A(text_button("⬅️ Back to Timeline", |_: &mut Self| {
                    Navigation::Home
                }))
            } else {
                Either::B(sized_box(flex_col((
                    text_input(
                        self.timeline_box_contents.clone(),
                        |state: &mut Self, string| {
                            state.timeline_box_contents = string;
                            Navigation::None
                        },
                    )
                    .on_enter(|_, user| Navigation::LoadUser(user))
                    .disabled(self.loading_timeline),
                    self.loading_timeline
                        .then(|| sized_box(spinner()).width(50.px()).height(50.px())),
                    text_button("Go", |state: &mut Self| {
                        Navigation::LoadUser(state.timeline_box_contents.clone())
                    }),
                ))))
            };
            Either::A(flex_col((
                label("Connected to:"),
                prose(instance.title.as_str()),
                back,
            )))
        } else {
            Either::B(prose("Not yet connected (or other unhandled error)"))
        }
    }

    fn main_view(&mut self) -> impl WidgetView<Self, Navigation> + use<> {
        if let Some(show_context) = self.show_context.as_ref() {
            if let Some(context) = self.context.as_ref() {
                OneOf6::A(thread(show_context, context))
            } else {
                OneOf::B(prose("Loading thread"))
            }
        } else if self.loading_timeline {
            OneOf::C(flex_col(
                sized_box(spinner()).width(50.px()).height(50.px()),
            ))
        } else if let Some(acct) = self.not_found_acct.as_ref() {
            OneOf::D(prose(format!(
                "Could not find account @{acct} on this server. \
                 You might need to include the server name of the account, if it's on a different server."
            )))
        } else if let Some(timline) = self.timeline.as_mut() {
            OneOf::E(map_state(
                timline.view(self.mastodon.clone()),
                |this: &mut Self| this.timeline.as_mut().unwrap(),
            ))
        } else {
            OneOf::F(prose("No statuses yet loaded"))
        }
    }
}
```

Key points:
- View methods on state struct organize UI logic
- Use Either/OneOf for conditional rendering
- State determines which view is shown
- No view logic in separate functions
- Actions bubble up from child views


## 6. Action Enum (actions.rs)

```rust
/// Ways that the app can navigate within itself.
#[expect(clippy::large_enum_variant, reason = "Who cares?")]
pub(crate) enum Navigation {
    /// Load the context (i.e. replies and ancestors) of a given
    /// (non-repost) status.
    LoadContext(Status),
    /// Load the timeline of the user with the given account id.
    LoadUser(String),
    /// Return to the main timeline.
    Home,
    /// HACK: The null navigation, because Xilem's handling of optional/None actions is not good.
    None,
}
```

Key points:
- Simple enum with clear semantics
- Carries data needed for action
- No state mutation here
- Handled centrally in map_action()


## 7. Async Task Pattern (One-shot)

```rust
fn load_instance(
    mastodon: Mastodon,
) -> impl View<Placehero, (), ViewCtx, Element = NoElement> + use<> {
    task_raw(
        move |result| {
            let mastodon = mastodon.clone();
            async move {
                // We choose not to handle the case where the event loop has ended
                let instance_result = mastodon.get_instance().await;
                // Note that error handling is deferred to the on_event handler
                drop(result.message(instance_result));
            }
        },
        |app_state: &mut Placehero, event| match event {
            Ok(instance) => app_state.instance = Some(instance.json),
            Err(megalodon::error::Error::RequestError(e)) if e.is_connect() => {
                todo!()
            }
            Err(e) => {
                todo!("handle {e}")
            }
        },
    )
}
```

Key points:
- task_raw for fire-once operations
- Returns Element = NoElement (invisible)
- First closure: async work
- Second closure: handle result and update state
- Uses result.message() to send result


## 8. Async Worker Pattern (Streaming)

```rust
fn load_account(
    mastodon: Mastodon,
) -> impl View<Placehero, (), ViewCtx, Element = NoElement> + use<> {
    worker_raw(
        move |result, mut recv: UnboundedReceiver<String>| {
            let mastodon = mastodon.clone();
            async move {
                while let Some(req) = recv.recv().await {
                    let instance_result = mastodon.lookup_account(req.clone()).await;
                    // We choose not to handle the case where the event loop has ended
                    // Note that error handling is deferred to the on_event handler
                    drop(result.message((instance_result, req)));
                }
            }
        },
        |app_state: &mut Placehero, sender| app_state.account_sender = Some(sender),
        |app_state: &mut Placehero, (event, acct)| match event {
            Ok(instance) => {
                app_state.timeline = Some(Timeline::new_for_account(instance.json));
                app_state.loading_timeline = false;
                app_state.not_found_acct = None;
            }
            Err(megalodon::error::Error::RequestError(e)) if e.is_connect() => {
                todo!()
            }
            Err(megalodon::error::Error::OwnError(
                e @ OwnError {
                    kind: Kind::HTTPStatusError,
                    ..
                },
            )) => {
                tracing::error!("Failure to to load account: {e}.");
                app_state.loading_timeline = false;
                app_state.not_found_acct = Some(acct);
            }
            Err(e) => {
                todo!("handle {e}")
            }
        },
    )
}
```

Key points:
- worker_raw for streaming operations
- First closure: loop reading from channel
- Second closure: store sender in state
- Third closure: handle responses and update state
- Allows multiple async operations via channel


## 9. Resource Provider Pattern (avatars.rs)

```rust
#[derive(Debug)]
pub(crate) struct Avatars {
    icons: HashMap<String, Option<ImageData>>,
    requester: Option<UnboundedSender<AvatarRequest>>,
}

impl Resource for Avatars {}

impl Avatars {
    pub(crate) fn avatar<State: 'static, Action: 'static>(
        url: String,
    ) -> impl WidgetView<State, Action> + use<State, Action> {
        with_context(move |this: &mut Self, _: &mut State| {
            let width = 50.px();
            let height = 50.px();
            if let Some(maybe_image) = this.icons.get(&url) {
                if let Some(image_) = maybe_image {
                    return Either::A(sized_box(image(image_.clone())).width(width).height(height));
                }
            } else if let Some(requester) = this.requester.as_ref() {
                drop(requester.send(AvatarRequest {
                    avatar_url: url.to_string(),
                }));
                this.icons.insert(url.to_string(), None);
            }
            Either::B(
                sized_box(spinner().color(css::BLACK))
                    .width(width)
                    .height(height)
                    .background_gradient(
                        Gradient::new_linear(const { -45_f64.to_radians() })
                            .with_stops([css::YELLOW, css::LIME]),
                    )
                    .padding(4.0),
            )
        })
    }

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

    fn worker<State, Action>()
    -> impl View<State, Action, ViewCtx, Element = NoElement> + use<State, Action>
    where
        State: 'static,
        Action: 'static,
    {
        map_message(
            on_action_with_context(
                |_: &mut State, this: &mut Self, response| {
                    let ret = this.icons.insert(response.url, Some(response.image));
                    if !matches!(ret, Some(None)) {
                        tracing::warn!("Potentially loaded or tried to load same avatar twice.");
                    }
                },
                env_worker(
                    |proxy: MessageProxy<AvatarResponse>,
                     mut rx: UnboundedReceiver<AvatarRequest>| async move {
                        while let Some(url) = rx.recv().await {
                            let proxy = proxy.clone();
                            tokio::task::spawn(async move {
                                let url = url.avatar_url;
                                let result = image_from_url(&url).await;
                                match result {
                                    Ok(image) => drop(proxy.message(AvatarResponse { url, image })),
                                    Err(err) => {
                                        tracing::warn!(
                                            "Loading avatar from {url:?} failed: {err}."
                                        );
                                    }
                                }
                            });
                        }
                    },
                    |_: &mut State, this: &mut Self, tx| {
                        if this.requester.is_some() {
                            tracing::warn!(
                                "Unexpectedly got a second worker for requesting avatars."
                            );
                        }
                        this.requester = Some(tx);
                    },
                    |_: &mut State, response| response,
                ),
            ),
            |_, action| match action {
                MessageResult::Action(_) => MessageResult::RequestRebuild,
                MessageResult::RequestRebuild => MessageResult::RequestRebuild,
                MessageResult::Nop => MessageResult::Nop,
                MessageResult::Stale => MessageResult::Stale,
            },
        )
    }
}
```

Key points:
- Struct implements Resource trait
- provide() wraps child with resource provider
- avatar() uses with_context to access resource
- worker() manages async operations
- fork() combines visible views with worker
- Resources injected into context automatically


## 10. Conditional Rendering (OneOf Pattern)

```rust
// Example: OneOf6 with 6 branches
if let Some(show_context) = self.show_context.as_ref() {
    if let Some(context) = self.context.as_ref() {
        OneOf6::A(thread(show_context, context))
    } else {
        OneOf::B(prose("Loading thread"))
    }
} else if self.loading_timeline {
    OneOf::C(flex_col(
        sized_box(spinner()).width(50.px()).height(50.px()),
    ))
} else if let Some(acct) = self.not_found_acct.as_ref() {
    OneOf::D(prose(format!(
        "Could not find account @{acct} on this server. \
         You might need to include the server name of the account, if it's on a different server."
    )))
} else if let Some(timline) = self.timeline.as_mut() {
    OneOf::E(map_state(
        timline.view(self.mastodon.clone()),
        |this: &mut Self| this.timeline.as_mut().unwrap(),
    ))
} else {
    OneOf::F(prose("No statuses yet loaded"))
}
```

Key points:
- OneOf types for type-safe conditional rendering
- OneOf2, OneOf3, ..., OneOf6 available
- Only one branch active at time
- Compiler ensures type safety
- Pattern-based (no runtime costs)


## 11. Component Function Pattern (components.rs)

```rust
/// Renders the key parts of a Status, in a shared way.
fn base_status<State: 'static>(
    status: &Status,
) -> impl FlexSequence<State, Navigation> + use<State> {
    let status_clone: Status = status.clone();
    let acct_clone = status.account.acct.clone();
    
    (
        // Account info/message time
        flex_row((
            Avatars::avatar(status.account.avatar_static.clone()),
            flex_col((
                inline_prose(status.account.display_name.as_str())
                    .weight(FontWeight::SEMI_BOLD)
                    .text_alignment(TextAlign::Start)
                    .text_size(20.)
                    .flex(CrossAxisAlignment::Start),
                inline_prose(status.account.acct.as_str())
                    .weight(FontWeight::SEMI_LIGHT)
                    .text_alignment(TextAlign::Start)
                    .flex(CrossAxisAlignment::Start),
            ))
            .main_axis_alignment(MainAxisAlignment::Start)
            .gap(1.px()),
            FlexSpacer::Flex(1.0),
            text_button("Open Profile", move |_| {
                Navigation::LoadUser(acct_clone.clone())
            }),
            inline_prose(status.created_at.format("%Y-%m-%d %H:%M:%S").to_string())
                .text_alignment(TextAlign::End),
        ))
        .must_fill_major_axis(true),
        prose(status_html_to_plaintext(status.content.as_str())).flex(CrossAxisAlignment::Start),
        status
            .media_attachments
            .iter()
            .map(|attachment| {
                media::attachment::<State>(attachment).flex(CrossAxisAlignment::Start)
            })
            .collect::<Vec<_>>(),
        flex_row((
            label(format!("💬 {}", status.replies_count)).flex(1.0),
            label(format!("🔄 {}", status.reblogs_count)).flex(1.0),
            label(format!("⭐ {}", status.favourites_count)).flex(1.0),
            text_button("View Replies", move |_| {
                Navigation::LoadContext(status_clone.clone())
            }),
        ))
        .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
        .flex(CrossAxisAlignment::Start),
    )
}
```

Key points:
- Reusable component function
- Returns FlexSequence (sequence of flex items)
- Generic over State type (works with any state)
- Captures values from environment
- Returns actions via closures


## 12. Module Export Pattern

### components.rs
```rust
mod timeline;
pub(crate) use timeline::Timeline;

mod thread;
pub(crate) use thread::thread;

mod media;
```

Key points:
- Submodules private by default (mod)
- Export via pub(crate) for crate-wide access
- Single location for all exports
- Easy to see what's public


## Refactoring Template for Spoonbender

Based on these patterns, here's a template structure:

### src/lib.rs
```rust
use xilem::{EventLoop, EventLoopError, WidgetView};

// 1. Define all state structs and enums
enum MainState {
    Welcome,
    Editing(EditorState),
    Saving,
}

struct EditorState {
    document: Document,
    selected_glyph: Option<GlyphId>,
    // ... other state
}

// 2. Entry point
pub fn run(event_loop: EventLoopBuilder) -> Result<(), EventLoopError> {
    Xilem::new_simple(
        MainState::Welcome,
        app_view,
        WindowOptions::new("Spoonbender"),
    )
    .run_in(event_loop)
}

// 3. Root dispatcher
fn app_view(state: &mut MainState) -> impl WidgetView<MainState> + use<> {
    match state {
        MainState::Welcome => OneOf3::A(/* welcome view */),
        MainState::Editing(_) => OneOf3::B(lens(editor_view, /* extract */)),
        MainState::Saving => OneOf3::C(/* saving view */),
    }
}

// 4. Main layout
fn editor_view(state: &mut EditorState) -> impl WidgetView<EditorState> + use<> {
    fork(
        map_action(
            split(state.sidebar(), state.canvas()).split_point(0.2),
            |state, action| { /* handle EditorAction */ }
        ),
        (
            load_document(/* ... */),
            save_document(/* ... */),
        ),
    )
}

// 5. View methods on state
impl EditorState {
    fn sidebar(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
        // ...
    }
    
    fn canvas(&mut self) -> impl WidgetView<Self, EditorAction> + use<> {
        // ...
    }
}
```

### src/actions.rs
```rust
pub(crate) enum EditorAction {
    SelectGlyph(GlyphId),
    EditGlyph(GlyphEdit),
    Save,
    Load(String),
}
```

### src/main.rs
```rust
use xilem::EventLoop;
use xilem::winit::error::EventLoopError;

fn main() -> Result<(), EventLoopError> {
    spoonbender::run(EventLoop::with_user_event())
}
```

