# The web client (`crates/tuita-web`): developer guide

How to work in `tuita-web`: the helper API, the rules every component follows, how islands and Topcoat divide the page, and how to test. Topcoat behaviour and tested snippets are in [`docs/topcoat.md`](topcoat.md); the place of the crate in the system is in [`docs/architecture.md`](architecture.md#web-client).

## Map

`src/lib.rs` exposes `router(state, assets)` and `Assets`. Everything else is private modules.

| Module | Role |
|---|---|
| `app.rs`, `app/` | The module router. Files and directories are URL segments (`handle.rs` is `/@{handle}`); `_member/` is the group whose layer sends guests to `/login`. `app.rs` holds the root layout, the pathless `edge` layer (locale, `no-store`, branded 404), `error_page` and the document `<head>`. |
| `api`, `session`, `paths`, `next`, `fragment`, `cookies`, `locale`, `fmt`, `icons`, `l10n`, `assets`, `static_files` | Shared helpers, see the next section. |
| `ui/` | Presentational components with no page knowledge (buttons, dialogs, menus, fields, charts, the dither components). |
| `shell/` | The member frame: rail, top bar, bottom bar, drawer, announcements, right rail, global dialogs. |
| `admin`, `auth`, `compose`, `design`, `docs`, `explore`, `feed`, `lists`, `messages`, `notifications`, `oauth`, `post`, `profile`, `settings` | One module per area: the area's components and its `/_tuita/..` fragment routes. The page modules in `app/` only fetch data and compose these. |
| `testing.rs` | `#[cfg(test)]` render helpers. |
| `assets/` | `css/` (one stylesheet per area plus `tokens`, `base`, `dither`, `ui`), `js/` (islands), `fonts/`, `icons/`, `media/` (design-gallery clips), `static/` (fixed-URL files). |
| `tests/` | Full-app page tests over Postgres. |

## Skeleton API

```rust
use crate::{api, session, paths, fragment, l10n::t, assets, cookies, next, locale, fmt};
use crate::ui::{icon::icon, not_found::not_found, toaster::toaster}; use crate::icons::Icon;

api::get::<Dto>(cx, "/users/ada").await?          // path relative to /api/v1, query included; ApiError implements std::error::Error
api::post::<_, Dto>(cx, "/posts", &body).await    // also patch; api::delete::<T>(cx, path)
api::send(cx, Method::POST, path, Some(json)) / api::send_raw(..)   // send_raw returns non-2xx unchanged
api::forward_cookies(cx, &response)               // appends the answer's Set-Cookie to the page response
api::query(&[("before", cursor), ("q", Some(q))]) // "?a=b" or ""; skips None and ""
api::enc("a b")                                   // encodeURIComponent
ApiError { status, message, retry_after }         // message = the API's `error` text; a procedure returns Ok(Err(e.message))

session::me(cx).await?          // Option<&MeDto>, one /me call
session::viewer(cx).await?      // Option<&Viewer { me, accounts, badges }>
session::instance(cx).await?    // &InstanceDto
session::member(cx).await?      // &Viewer, or 303 to /login?next=.. for a guest
session::is_staff(&viewer)
paths::{profile, post, followers, following, messages, tag, list, list_members}   // acct = `ada` or `ada@host`, no `@`
next::{safe_next, login_url, auth_link}; fragment::requested(cx); cookies::{get, set, expire}
t!(key, var: v)                 // crate::l10n::t; outside a view: t!(key).resolve(cx).into_owned()
icon(name: Icon, size: u32 = 24, label: Option<&str> = None)    // Icon::ChevronDown, Icon::ALL
fmt::{compact, rel_time (component), full_time, month_year, handle_of, acct_of, host_of}
```

`api` calls the backend `Router` in-process with the browser's cookies and headers, so a page never needs its own authentication or CSRF code. `session::me`, `viewer` and `instance` are memoized per request.

## Pages, layouts and the shell

- A page under `app/_member/` gets the guest guard automatically; still call `session::member(cx)` to read the viewer.
- Titles: `#[page(with(Metadata::new().title(t!(explore_title))))]`; the default is `t!(app_name)`. The `· tuita` suffix is the page's job.
- Errors: `app::error_page(cx, err)` is the error-boundary mapper (API 401 to `/login`, 404 to the branded page, others to a status page).
- Fragment routes live at absolute paths (`post/fragments.rs`, `feed/new_posts.rs`); `#[layer("/_tuita")]` in `fragment.rs` gives guests an empty 401 and turns an API failure into an empty answer with its status.
- The member shell (`shell::app_shell`: rail, top bar, bottom bar, drawer, announcements, right rail, `main#main`) is applied by the ROOT layout in `app.rs` to every page of a signed-in visitor except `shell::wants_shell`'s bare routes (`/login`, `/signup`, `/oauth/*`, `/design`), so `/`, `/terms`, `/privacy`, `/developers` and error pages get it too and a page just renders its content.
- A page renders its header with `shell::page_bar::page_bar(title: .., sub: .., back: .., heading: .., actions: Some(view!{..}.into()), below: ..)` and sets its tab title with `#[page(with(Metadata::from_fn(async |cx| shell::titled(cx, "..").await)))]` (`title · tuita`, unread count in front).
- `shell::announcement_banner` and `shell::parts::{nav_badge, account_avatar, user_name, nav_button}` are shared. `shell/globals.rs` renders the toaster, the stream marker (`[data-island="stream"][data-handle]`) and the shortcuts dialog (`#shortcuts`); other global dialogs add a call there. The new-post buttons are `[data-compose="new"]`.

## Components

### Conventions

1. One `#[component] pub(crate) async fn snake_name(cx: &Cx, props...) -> Result<impl View>` in `src/<area>/snake_name.rs`. Signatures are documented in each file's doc comment.
2. Props are typed arguments: enums for variants, `#[default]` / `#[default(v)]` for optional ones, `#[into]` on `Option<&str>` and similar so callers pass a bare value.
3. Children: `#[default] child: Child<'_>`. A second slot: `#[default] actions: Option<Child<'_>>`, passed as `actions: Some(view!{..}.into())`.
4. Extra attributes: `#[default] mut attrs: Attributes`, spread with `(attrs)` on the root. Merge classes with `class!("btn", variant.class(), "x" if cond, attrs.remove("class"))`.
5. An element id that something else must reference comes from `element_id(cx)`, or from a caller-supplied `id` prop.
6. The root carries `data-c="kebab-name"`. All text goes through `t!(key)`; `scripts/lint-strings.sh` flags literals (`// lint-strings: allow` escapes a line). For a `&str` prop use `t!(key).resolve(cx).into_owned()`.
7. Styles live in the area's stylesheet inside `@scope ([data-c="x"]) to ([data-c]) { ... }`. Rules that hit the component's ROOT element are written `:scope { }` / `:scope.cls { }` (a bare class selector inside `@scope` never matches the root).
8. Rules for a nested component's elements go outside the scope as `[data-c="x"] ...`. Prefix `@keyframes` names with the component name. Append to a stylesheet; never reorder other components' blocks.
9. State that lives in the browser is a `signal` created inside a `#[component]`, bound with `:attr=$(..)` and `@event=$(|_e: Event| ..)`. A closure inside `attributes!` needs its parameter typed. `match` is not supported inside `$()`: use `is_ok()` / `unwrap()`.
10. Native features first: `<dialog>` + `command`/`commandfor`/`closedby`, `popover`/`popovertarget`, CSS anchor positioning. Anything that needs a script gets a data attribute or class and a documented island contract.
11. Pure maths get Rust unit tests with literal vectors (`post::poll_math`, `ui::chart`).
12. Procedures return `Result<Result<T, String>>`; the inner `Err` is the message for a toast (`raw!("document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${m}}))", m.is_empty())`).
13. Island-to-Topcoat events are `const ev = new CustomEvent(name, {bubbles: true, detail}); ev.data = JSON.stringify(detail); el.dispatchEvent(ev)`; Topcoat reads `e.data`.

### Recipe: add a component or a page

1. Write the component in `src/<area>/<name>.rs` following the conventions, declare it in the area's `mod.rs`, and give it a render test (see [Testing](#testing)).
2. Add its CSS to the area stylesheet inside the `@scope` block. A new stylesheet is an `asset!` constant plus an entry in `STYLESHEETS` in `src/assets.rs` (the array length changes with it).
3. Add every new message to all 12 `src/l10n/<locale>.ftl` files. A message an island reads is also an entry in `CLIENT_STRINGS` in `src/l10n.rs` (`plain { key(vars..) }`, `plural { key(count, vars..) }`).
4. A page is a module under `app/` (a file for a leaf, a directory plus file for a segment with children) with `#[page]`; it fetches with `api::*`, renders `shell::page_bar` and the area's components. A paged page also answers the fragment request (see [Paged lists](#paged-lists-and-feeds)).
5. A browser-only behaviour is an island: a new `assets/js/<name>.js` plus an entry in `SCRIPTS`, and a vector in `crates/tuita-e2e/js/units/<name>.test.js` with one `unit!` line in `crates/tuita-e2e/tests/e2e/js_units.rs`.
6. Run `topcoat fmt` on the files, then `check`.

### Core `ui` signatures

```rust
button(variant: ButtonVariant = Solid, size: ButtonSize = Md, loading: bool, disabled: bool, submit: bool, mut attrs: Attributes, child: Child)
icon_button(glyph: Icon, label: &str, #[into] pressed: Option<bool>, variant: IconButtonVariant = Ghost, size: ButtonSize = Md, mut attrs: Attributes)
avatar(seed: &str, #[into] src: Option<&str>, alt: &str, shape: AvatarShape = Circle, size: u32 = 40, #[into] ink: Option<&str>)
brand(size: u32 = 34, wordmark: bool = true)
dialog(id: &str, title: &str, variant: DialogVariant = Dialog, actions: Option<Child>, child: Child)
menu(id: &str, label: &str, items: Vec<MenuItem<'_>>, plain: bool, trigger: Child)       // MenuItem { id, label, ..Default::default() }
popover(id: &str, label: &str, #[into] glyph: Option<Icon>, trigger: Child, child: Child)
text_field(label: &str, value: &str, #[into] error: Option<&str>, #[into] hint: Option<&str>, attrs: Attributes)
text_area(label, value, #[into] error, #[into] hint, rows: u32 = 3, attrs: Attributes)
checkbox(checked: bool, label: &str, attrs: Attributes, child: Child)
switch(label: &str, checked: &Signal<bool>, attrs: Attributes)
empty_state(context: &str, title: &str, #[into] body: Option<&str>, action: Option<Child>)
divider(vertical: bool, #[into] label: Option<&str>); badge(#[into] count: Option<u64>, max: u64 = 99, dot: bool, #[into] label: Option<&str>)
loader(variant: LoaderVariant = Ring, size: u32 = 24)
confirm_dialog<F: EventHandlerFn + Send + Sync>(id: &str, title: &str, body: &str, confirm_label: &str, danger: bool, action: Expr<F>)
ui::link_tabs::link_tabs(items: Vec<LinkTab{id,label,href}>, selected, label)   // URL-driven tabs (`<a role="tab">`), the `tabs` look without a signal
```

The `ui` module holds: `avatar` (+`identicon`), `avatar_stack`, `badge`, `bar_chart`, `bento`, `brand`, `button`, `card`, `char_counter`, `chart` (maths), `chart_frame`, `checkbox`, `confirm_dialog`, `cropper`, `dialog`, `dither_field`, `dither_fill`, `dither_number`, `dither_text`, `divider`, `dropzone`, `empty_state`, `hash`, `heatmap`, `icon`, `icon_button`, `link_tabs`, `loader`, `mask`, `menu`, `not_found`, `number_format`, `popover`, `progress_bar`, `radio`, `segmented`, `select`, `sheet`, `skeleton`, `slider`, `sparkline`, `stat_card`, `switch`, `tabs`, `tag`, `text_area`, `text_field`, `toaster`, `tooltip`. Signatures not listed above are in each file's doc comment.

Notes:

- `icon` props are named `glyph` (a prop named `icon` collides with the `icon(..)` component).
- Opening: `command="show-modal" commandfor=(id)`; closing: `command="close"` plus `closedby="any"`. Menu items carry `popovertargetaction="hide"`, so use `@click` in an item's `attrs` rather than `command`.
- Dialog drag-to-dismiss (`.handle`) and menu keyboard/focus (`[popover][role="menu"]`) are done by `shell.js`.
- `empty_state` renders `<canvas data-island="dither-field" ...>` for `dither.js`.
- Use `var(--l0)`..`var(--l4)` and `var(--m25)`/`var(--m50)` (page-wide masks in `ui.css`) instead of inlining `level_mask(..)` for quarter levels.

## Islands

Islands are plain ES2022 modules in `assets/js/`. There is no bundler and no JS build step; no module imports another. `assets.rs` lists them in `SCRIPTS` and the root layout loads every one on every page as `<script type="module" data-js="<stem>" src=..>` (the `data-js` stem is how the e2e suite finds a module). `boot.js` is a classic blocking script in `<head>` that applies the stored theme before first paint.

Rules every island follows:

- It attaches to elements by a data attribute (`[data-island="<name>"]`, `[data-act]`, `[data-more]`, ...), with a small `mount(selector, init)` helper that also scans markup inserted later (a `MutationObserver`). Delegated listeners sit on `document`.
- It reaches the server only through the JSON API (`/api/v1/...`) or `/_tuita/...` fragment routes.
- It talks to Topcoat through bubbling `CustomEvent`s that carry `ev.data = JSON.stringify(detail)`.
- Client strings reach it through the `#tuita-l10n` JSON block: add entries to `CLIENT_STRINGS` in `src/l10n.rs`.
- A fetched fragment is parsed into a `<template>` before it is inserted, so no script in it runs.

The contract of each island (the markup it expects and the events it fires) is written at the top of its file. The ones other code depends on:

| Island | Contract |
|---|---|
| `prefs.js` | `[data-pref="mode\|ink\|textScale\|contrast\|motion"]` controls; storage key `tuita:prefs`. |
| `toast.js` | `tuita-toast` on `document` (detail: a string or `{message, label, href}`); `[data-toast]` elements (the server flash). Items land in `#toaster`. |
| `time.js` | `time[data-rel]` text kept current from the `time_*` client strings. |
| `infinite.js` | `a[data-more]` sentinel; fetches its href with header `x-tuita-fragment: 1` and swaps the host item. |
| `stream.js` | Starts only when `[data-island="stream"][data-handle="<viewer handle>"]` is in the DOM. Badges `[data-badge="notifications\|messages"]` with `[data-count]` and `[data-label]` children. Events reach `[data-stream~="<type>"]` as `tuita-stream` with `ev.data` = JSON; `card` events refill `[data-post-id=ID] [data-slot="card"]`; `[data-island="desktop-notifications"]`. It closes its `EventSource` on `pagehide`. |
| `shell.js` | `bottom-bar`, `[data-back]`, menu and tablist keyboard handling (`[data-roving]`), dialog drag handle, `announcements`, `[data-grain]`, `clock`. |
| `shortcuts.js` | `?` help dialog `#shortcuts`, `g`-prefixed navigation, post keys; off without the stream marker. |
| `dither.js` | `[data-island="dither-field\|dither-number\|dither-text"]`; contract at the top of the file. |
| `cropper.js` | `[data-island="cropper"]` with events `tuita-crop-load`, `tuita-crop`, `tuita-crop-cancel`; `[data-island="slider"]`. |
| `post.js` | The post card, see [Posts](#posts). |
| `hovercard.js` | Any `[data-hovercard="<acct>"]` link opens the profile card from `/_tuita/hovercard/<acct>`. |
| `lightbox.js`, `video.js` | `[data-island="media-grid"]` strip and the lightbox dialog (`data-act="lightbox"`, `data-index`, `data-post-id`); `[data-island="video"]` player. |
| `share.js`, `views.js` | Copy, share and download actions on `button[data-act]`; batched `POST /views` for posts half on screen for 1 s. |
| `composer.js`, `upload.js` | The composer, see [Composer](#composer). |
| `pull.js`, `newposts.js` | The home feed, see [Paged lists](#paged-lists-and-feeds). |
| `lists.js`, `scheduled.js` | List editing and the scheduled-posts page, see [Paged lists](#paged-lists-and-feeds). |
| `explore.js`, `follow.js`, `notifications.js` | See [Profiles and people](#profiles-and-people). |
| `e2ee.js`, `messages.js` | See [Messages](#messages). |
| `settings.js`, `webauthn.js`, `admin.js`, `design.js` | Region names, copy button and token box; passkey login and registration; admin copy, tabs and emote form; the `/design` gallery extras. |

## Hydration limit

The Topcoat runtime hydrates markup only at page start, after a shard rerun and after a runtime navigation (it has no `MutationObserver`). Markup an island inserts later (infinite-list fragments, `/_tuita/*` fragments, dialog contents fetched on demand) is NEVER hydrated: `@click=$()`, `:attr=$()`, signals, procedures and shards do not work inside it.

Anything that can appear inside such markup (post cards, list rows, people rows) is therefore plain HTML driven by a vanilla island that listens on `document` and reads `data-*` hooks (`post.js` for posts: `data-act` attributes, optimistic updates, rollback toasts). Use Topcoat reactivity only for markup rendered in the initial page response and not re-fetched.

All internal links are plain `<a href>` (full page loads); the Topcoat `runtime::link` navigation is not used because it morphs the old DOM in place and leaves island state behind. Shard reruns replace shard DOM, so an island never writes inside a shard.

## Runtime pitfalls

- Never call `signal.get()` outside a `$(..)` in a component body. A server-side read makes the signal a dependency of the page: every change reruns the page and morphs the DOM back to the server's HTML, wiping whatever an island added (the cropper's picture, a canvas's size). Bind the value in an attribute with `:attr=$(..)` instead.
- A string literal handler (`@click="..."`) is evaluated once at hydration and its RESULT is called on the event, so it must be a function: `@click="() => history.back()"`. The handler receives Topcoat's `Event` surrogate, not the DOM event.
- Runtime expressions have no string concatenation or `format!`. Compare with `==`/`!=`, branch with `if`, and use captured `&str`s (a field access like `item.id` cannot be captured: bind it in the `for` pattern, `for (id, label) in pairs`).
- Do not compute borrowed values (`let items = &page.items`, a `Vec<LinkTab<'_>>` of local strings) outside `view!` and return them inside it: `view!` moves only what it mentions, so such references dangle. Mention the owned value inside the macro (`&page.items`, `vec![LinkTab { label: &local, .. }]`). The same holds for option lists and chart points: create the borrows inside the `view!` arguments.

## Shards and procedures

- A list that changes after an action is a `#[shard]` taking a `revision: Signal<u32>` it reads with `let _ = revision.get();`; the action is a `#[procedure]` returning `Result<Result<String, String>>` (toast text or API message) and the handler bumps the signal. Loops that create signals or element ids need `#[key(..)]`.
- Dispatch the toast BEFORE `revision.increment()`: the shard rerun tears the handler's scope down, so a `raw!` after it never runs. Use `;` after a statement `raw!`, and a `()`-typed fallback such as `drop(msg)` (only a tail `raw!` may use `msg.is_empty()`).
- A string captured into `raw!` arrives as a surrogate object, not a string: write `detail:''+${msg}` (`toast.js` also unwraps it with `.dehydrate()`, but do not rely on that for other consumers).

## Posts

What a page that shows posts can rely on:

- A post card carries no signals: the same card appears in feeds, threads, dialogs and the lightbox panel. `post.js` makes it work: `[data-action=like|bookmark]` and `[data-act=repost]` toggle at once and roll back with a toast, `.poll [data-act=vote]` votes and swaps the poll for `/_tuita/poll/<id>`, menu items carry `data-act` (`goto`, `pin`, `delete`, `hide`, `mute`, `block`, `edit`, `history`, `report`, `report-user`, `list-picker`, `post-list`), `data-compose="reply|quote|new"` is for the composer.
- Components: `post::post_card::post_card(post: &PostDto, main, thread: Option<ThreadLine>, hide_reply_context, footer: Option<Child>)`, `post::post_thread::{post_thread, post_stats}`, `post::rich_text::{rich_text, emote_text, user_name, segments_of, short_url}`, `post::poll::poll`, `post::media_grid::media_grid`, `post::quote_card::quote_card`, `post::link_card::link_card`, `post::user_hover_card`, `post::dialogs::*`.
- Test fixtures: `post::testing::{sample_post, sample_user, sample_image, sample_poll, sample_card, sample_quote, sample_emoji}`; `post::post_card::card_view(post, me: Option<Me>)` renders a card without a session.
- `[data-act="report-user"][data-acct]`, `[data-act="list-picker"][data-acct]` and `[data-act="mute"|"block"]` with `data-acct`, `data-user-id`, `data-handle` work on any button.
- A card root has `data-post`, `data-post-id`, `data-author-id`; `/_tuita/post/<id>?main=1&thread=up|down&hide_reply=1` returns a fresh card.

## Composer

`compose::composer::composer(reply_to: Option<&PostDto> = None, autofocus: bool, placeholder: Option<&str>, in_dialog: bool)` is the post box (a reply box with `reply_to`). The compose dialog (`dialog#compose`) is rendered once for members by `shell::globals`.

- Open it from any element: `<button data-compose="new">`, or `data-compose="reply|quote" data-post-id="<id>"` (`composer.js` fetches the post).
- A finished post fires a bubbling `tuita-posted` on the form with `ev.data` = JSON `{ id, replyTo, quote, scheduled, count }` (`id` is null for a scheduled post).
- `upload.js` provides `[data-island="media-tray"]`, the dropzone drag states (`ui::dropzone`, event `tuita-files`), `[data-island="multipart-form"]` (a form posted by XHR to `data-action`, page reload on success, error toast otherwise) and `[data-island="crop-upload"]` (cropper dialog, then upload of an avatar or banner). Both are documented at the top of the file.

## Paged lists and feeds

- A paged page renders its entries with `feed::list::{post_list, post_items, more_href}`: `post_list(label, posts, more, empty)` is `<div role="list" data-c="post-list">` of `<div role="listitem">` entries ending in `<div role="listitem" class="more" data-more-host><a data-more href="PATH?before=CURSOR">`. `more_href(cx, keep, cursor)` builds that URL from the page path, the query pairs to keep (`feed`, `tab`) and the cursor.
- With `?before=` and `x-tuita-fragment: 1` the page answers only `post_items(..)` (check `fragment::requested(cx)` inside the page component). `infinite.js` replaces the whole `[data-more-host]` item, so pages of items do not nest. For other row kinds write the same `role=listitem` + `data-more-host` sentinel by hand: `profile::list::{infinite_list(label, child), user_items(users, more), more_item(href), next_href(cx, cursor)}` do it for people rows.
- Home (`feed::home`): tabs are `ui::link_tabs` links to `/?feed=foryou|following`; a visit with a valid `?feed=` sets the session cookie `tuita_feed` (not on fragment requests) and `/` alone reads it. The list region is one `[data-feed-body][data-since]` element inside `[data-island="feed"]`; `newposts.js` swaps it (fragment of `/?feed=` without `before`), puts a finished post (`tuita-posted`, no `replyTo`) on top from `/_tuita/post/<id>`, polls `/_tuita/new-posts?feed=&since=` (30 s and on `tuita-stream` badges) into `[data-slot="pill"]`, and answers `tuita-feed-refresh` (dispatched by `pull.js`, whose touch pull works on phones) by pushing its promise onto `event.detail.waits`.
- Lists: `lists.js` handles `[data-act="list-new|list-edit|list-delete-yes|list-remove-member"]` document-wide; `list-new` with `data-acct` (the list picker's button) adds that account to the new list and reopens the picker. `scheduled.js` drives `/scheduled` (`sched-resched|sched-publish|sched-cancel|sched-cancel-yes`) and re-fetches `[data-scheduled-body]`.

## Profiles and people

- `profile::follow_button::{follow_button(user, size), follow_button_view(user, viewer, size)}`: a plain `<button data-follow data-acct data-handle data-remote data-state>`; it renders nothing for a guest, the viewer's own account or a blocked pair. `follow.js` drives every one (profile, rows, hover card, right rail): optimistic flip, `POST|DELETE /users/<acct>/follow`, siblings of the same account settle together, rollback with a toast.
- `profile::user_row::{user_row(user), user_row_view(user, viewer)}` is a people row (picture, name, handle, bio, follow button).
- Profile menu: `data-profile-act="mute|unmute|block|unblock"` (`follow.js`; block asks in `#confirm-profile-block`, every act reloads the page and `follow.js` shows the stored toast after the reload); "Add to lists" and "Report" are `data-act` entries of `post.js`.
- Recent searches live in `explore.js` (`[data-island="recent-searches"]`, `localStorage` `tuita:recent-searches:<user id>`, cleared by `[data-forget-keys]`). `notifications.js` owns `[data-island="notifications"]`: it POSTs `/notifications/read` after the page shows (the API's CSRF guard needs an `Origin`, which a navigation lacks) and refetches page 1 as a fragment on a live `notification` event.
- A remote `@user@host` page waits for the API's lookup; when it fails the page shows `remote_lookup` (404: no such account; otherwise the server's sentence and a retry that reloads).

## Messages

- Pages: `/messages` and `/messages/{peer}` render `messages::frame::messages_frame` (list column plus pane). Nothing there is hydrated; `messages.js` drives it. The list is plain `a.row` links (no `<ul>`), ended by `a[data-more]`; `?before=` and `?open=<acct>` with the fragment header return rows only. A thread answers `?before=`, `?after=` and a bare request with the fragment header by `thread_fragment`: a `[data-view]` marker (mode, peer key, blocked reason), the `li[data-id]` items and, for `?before=`, `a[data-older]`.
- Components (also shown in `/design`): `messages::{chat_thread::{chat_thread, chat_item, chat_items}, chat_footer::chat_footer, chat_note::chat_note, conversation_row::conversation_row, e2ee_notice::e2ee_notice, e2ee_prompt::e2ee_prompt, safety_dialog::safety_dialog, people::people_dialog}`. The server cannot know the key state of this browser, so `chat_note` and `chat_footer` render every variant hidden (`[data-note]`, `[data-block]`) and `messages.js` shows the one `conversationState` (in `e2ee.js`) picks.
- `e2ee.js` exports every pure piece (`createIdentity`, `seal`, `open`, `Keys`, `conversationState`, `sendMessage`, stores, `parseText`) and sets `globalThis.tuitaE2ee` for `messages.js`. A sealed bubble is `p.bubble[data-ciphertext][data-mid][data-mine]` inside `[data-e2ee-lines]`; `e2ee.js` opens it. `data-forget-keys` controls hold the click until the store dropped the keys, then replay it. The crypto design is in [`docs/architecture.md`](architecture.md#direct-messages).
- Share to chat: `data-act="share-chat"` (post menu) opens `#share-chat` (`messages::globals`); the link comes from the sibling `data-act="copy-link"` item. `/_tuita/chat-people?q=` renders the picker rows.

## Assets, CSS and CSP

- Every stylesheet, island, font and clip is an `asset!` in `src/assets.rs`. The bundle keeps only assets some rendered view uses, so the root layout renders every constant there; a new file goes into `STYLESHEETS` or `SCRIPTS`, in load order. The stylesheets are `tokens`, `base`, `dither`, `ui`, then `shell` and one per area.
- `topcoat asset bundle --package tuita [--release]` writes `<stem>-<16hex>.<ext>` files and `manifest.toml` to an `assets/` directory beside the built binary (`target/debug/assets` in development). Editing any asset changes its URL, so re-bundle and restart.
- `url(...)` inside bundled CSS is not rewritten, so `@font-face` is an inline `<style>` in the head built from the font asset URLs (`app::font_faces`).
- Files whose URL browsers and crawlers fix (favicon, app icons, web manifest, `robots.txt`) are plain routes in `src/static_files.rs`, cached for an hour; add new ones to its `PATHS`.
- CSP is one static policy, `tuita_server::CONTENT_SECURITY_POLICY`: `script-src 'self' 'unsafe-eval'` (Topcoat compiles `$(..)` expressions in the browser with `new Function`), `style-src 'self' 'unsafe-inline'`, `img-src 'self' data: blob:`, `connect-src 'self'`. No inline `<script>` is allowed anywhere, so an island is always a file. Pages and fragments are `Cache-Control: no-store`; `/_topcoat/assets/*` is immutable.
- Origin policy: `OriginPolicy::trust_origins([public origin])` refuses cross-site non-GET requests with 403; same-origin Chromium POSTs (procedures, shards) pass.

## Testing

- Render tests: `crate::testing::render("en", |cx| view!{..}.boxed())` (needs `use topcoat::view::ViewExt`) returns the HTML of a component without a router, backend or database. Assert roles, accessible names, classes and ARIA with `contains`; attribute order varies between runs, and `attr_values(html, "class")` lists an attribute's values. Components using `signal(cx, ..)` or `element_id` must be rendered inside a `#[component]` (see the `demo` components in `ui/switch.rs`). Test behaviour; do not pin wording or implementation.
- Page tests: the files in the crate's `tests/` use `tests/common/mod.rs` (`app(db)`, `Browser` with `.get`, `.request`, `.signup`, `.set_cookie`) over the full app in-process. They need Postgres (`DATABASE_URL=postgres://tuita@localhost/tuita`) and the asset bundle: `TUITA_TEST_ASSETS=<dir>` or `target/debug/assets` (`topcoat asset bundle --package tuita`). Rebuild the bundle after adding any asset.
- Browser tests: `crates/tuita-e2e`, a Chromium harness over CDP with strict auto-waiting locators (`Ctx`, `Page`, `Loc`; `tests/selftest.rs` shows every API call). One module per area in `tests/e2e/`, plus `visual.rs` (screenshots, accessibility with axe, overflow checks) and `federation.rs` (a second instance on `E2E_PEER_PORT`). Run with `scripts/e2e.sh [filter]`, for example `scripts/e2e.sh profiles::`: it builds a release server, bundles assets, starts the server on a throwaway database and runs the suite (`test = false` keeps it out of `cargo test`). Concurrent runs in other worktrees MUST set unique `E2E_PORT`, `E2E_PEER_PORT` and `E2E_DB_NAME` (defaults 8081, 8083, `tuita_e2e`; the peer database is `<name>_peer`). The base URL host MUST stay `localhost`, because WebAuthn refuses IP-literal origins.
- Island vectors: `crates/tuita-e2e/js/units/<name>.test.js` exports `async function run(mod)` and throws on the first failed assertion; `tests/e2e/js_units.rs` loads the page, imports the island module by its `data-js` URL and runs the vector (with the CSP bypassed, because the vector loads from a `blob:` URL). Use it for pure functions of an island and for known-answer vectors such as the e2ee wire format.
- Harness notes: `Page::text` matches an element by the `textContent` of its subtree, which includes the inline `#tuita-l10n` JSON. A `text(..).expect_count(0)` on a string that is also a client string (for example `e2ee_line_peer_key`) matches `<body>`; scope such a check to the element that should not show it. A post by any parallel test raises the "show new posts" pill of the For you feed, which can cover links of the first card: open a post from its own page or use the Following feed.
- Gate: `check` (`scripts/check.sh`) runs formatting (`cargo fmt`, `topcoat fmt --check`), clippy, rustdoc, sqlx checks, the bundle step, `cargo test --workspace`, `topcoat l10n check --package tuita`, `scripts/lint-strings.sh`, `cargo deny`, shellcheck and typos. The browser suite is separate (`scripts/e2e.sh`).
