# Topcoat 0.12.1: decisions and tested API snippets

The facts below were run in Chromium and cargo against Topcoat 0.12.1; the snippets compile. Topcoat's own docs are in the crate sources: `~/.cargo/registry/src/index.crates.io-*/topcoat-0.12.1/docs/*.md`, with the code in the sibling `topcoat-*-0.12.1/` directories. The CLI is `topcoat` (`topcoat-cli` 0.12.1, put on `PATH` by `devenv.nix`). How `tuita-web` uses all this is in [`docs/web-client.md`](web-client.md).

## Decisions

1. **Routing.** A mixed segment `@{handle}` is impossible (`segment name must start with a letter or underscore`). A root param module `handle` has a type `Handle: FromStr + Display` that parses `^@[A-Za-z0-9_]{1,15}(@[A-Za-z0-9.-]+(:\d+)?)?$` (the leading `@` is kept in the value; `module_param!(pub(crate) handle: Handle else not_found)`). `href!(handle::profile, handle: h)` renders `/@alice` (`@` and `:` unencoded). Static segments (`/explore`) beat the root param. `/@alice/{post_id}` and `/@alice/followers` coexist.
2. **404 page.** A root `not_found!()` panics at build when any root param exists. Use `module_router!().layer(LayerFn::new(None::<&str>, edge))`, a pathless layer that catches the router's `NotFoundError` and renders the root layout around the branded `ui::not_found` page with status 404. `/%40alice` also matches.
3. **Links.** `runtime::link` is NOT used: it morphs the old DOM in place, so `mount()` never re-inits nodes that survive the morph. All internal links are plain `<a href>` (full page loads). Islands use the `mount()` helper (full loads make it safe). Shard reruns replace shard DOM; JS never writes inside a shard.
4. **Events.** The Topcoat `Event` type has no `detail`. An island dispatches `const ev = new CustomEvent('tuita-stream', {bubbles: true, detail}); ev.data = JSON.stringify(detail); target.dispatchEvent(ev);` and Topcoat reads `e.data` (a `String`) in `@tuita-stream=$(|e: Event| sig.set(e.data))`. Same for every island-to-Topcoat event (`tuita-posted`; `tuita-toast` is the other direction below).
5. **Toasts.** `raw!` after `.await` works: `raw!("document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))", msg.is_empty());` (a Rust fallback expression that reads the variable is required, else `unused_variables`). `toast.js` listens on `document`.
6. **l10n compact numbers.** Messages that show a compact count carry a `{ $compact }` placeable next to the `$count` plural selector. Callers pass both: `t!(profile_posts, count: n, compact: fmt::compact(cx, n))`. Compact formatting is `icu_decimal` (see ICU below), not Fluent `NUMBER`.
7. **Assets.** Assets are located at runtime by `AssetBundle::load_dir(path)`; the bundle dir is flat `<stem>-<16hex>.<ext>` + `manifest.toml`; asset IDs are identical in debug and release; unused `asset!` consts are DROPPED from the bundle (each `asset!` must reach a rendered view, or the bundle is missing it and a later render panics). `url(...)` inside bundled CSS is NOT rewritten, so `@font-face` lives in an inline `<style>` in the head using the font asset URLs. `topcoat asset bundle --package <pkg> [--release] [--out DIR]`. Editing any asset changes its URL, so re-bundle then restart.
8. **Tooling.** `topcoat fmt --check` walks `*.rs` under cwd, formats only the bodies of Topcoat macros, exit 1 and `<file>: needs formatting` on diffs, does not need `edition` in rustfmt.toml unless `--rustfmt` is passed (do not use `--rustfmt`). `topcoat l10n check --package P` builds, reads recorded `t!` calls: errors (exit 1) on a used key missing from a locale and on a `$var` the `.ftl` needs but `t!` does not pass; warns on extra keys; never checks en keys that no `t!` references. So `check` also needs the Rust test `every_locale_has_every_message` (fluent-syntax) in tuita-web.
9. **Dependencies.** `cargo deny check` is green with topcoat and chromiumoxide.
10. **CSS scope.** `@scope ([data-c="a"]) to ([data-c]) { ... }` works in nix Chromium 156: it styles a's own descendants, excludes any nested `[data-c]` subtree entirely, and sibling scopes with the same class names do not collide; global rules outside scopes still apply everywhere. IMPORTANT: a bare class selector `.x {}` inside the scope does NOT match the scope root itself. Rules that target the component's root element are written `:scope { ... }` or `:scope.x { ... }` (or `&`). Descendant rules are plain selectors.
11. **Origin policy.** `OriginPolicy::trust_origins([...])` is checked on every non-GET/HEAD/OPTIONS request: cross-site (`Sec-Fetch-Site: cross-site`, foreign `Origin`, `Origin: null`) gets 403; same-origin, trusted origins and requests with neither header pass.
12. **chromiumoxide 0.9.1.** `default-features = false` works (tokio is the only runtime; no `tokio-runtime` feature). WebAuthn needs the page served from `http://localhost:<port>`, so the e2e base URL host MUST be `localhost` (`scripts/e2e.sh` defaults to it). `SetDeviceMetricsOverrideParams::new(w, h, 2.0, false)` (mobile must be false, else innerWidth 1560). CSP violations arrive as `Log.entryAdded` (source security), not console. `exception_details.text` is only "Uncaught": read `exception.description`. Page-side console error checks must ignore `/favicon.ico` 404 noise (tuita serves `/favicon.svg` and the layout has `<link rel=icon>`).
13. **ICU.** Compact numbers: `icu_decimal = { version = "2.4.0", features = ["unstable"] }`, `CompactDecimalFormatter::try_new_short((&loc).into(), Default::default())?.format(&Decimal::from(n))`. Dates: `icu_datetime = "2.4.0"`, `DateTimeFormatter::try_new((&loc).into(), fieldsets::MD::medium())?.format(&Date::try_new_gregorian(y,m,d)?)` plus `icu_calendar = "2.4.0"`, `icu_locale = "2.3.0"`, `fixed_decimal = "0.7"`. Known bug: `fieldsets::YM::long()` panics for `pt` (DecimalFormatterNotLoaded): for pt build month-year from `M::long()` and `Y::long()` or use `YM::medium()`. Outputs (en): `1.2K`, `Nov 12`; ja: `1.2万`, `11月12日`; de: `1,2 Mio.`, `12. Nov.`. Topcoat's own `Number`/`DateTime` have no compact/MD field sets.
14. **Fluent.** `topcoat::l10n::fluent::messages!(pub(crate) "{mod}/l10n/{locale}.ftl")` creates `t!` and `t::locales()`. Ids use underscores (`t!(char_counter_left, count: 2)`). `{ $name }` renders with U+2068/U+2069 isolation marks around interpolations (numbers too): strip them in client-strings JSON and in the e2e harness. `t!(key).resolve(cx)` uses the locale in `cx`; `t!(key).resolve_in(&locale!("en"))`. A key missing from a locale renders the placeholder `key(arg: v)` (no cross-language fallback). Regional codes fall back to the language. There is no API to enumerate messages: `client_strings!` expands each key to a `t!` call. Plural selectors verified for en, pl, uk, ja; exact `[1]`/`[2]` arms win over `*[other]`.

## API cheat sheet (compiled and exercised)

```rust
// lib.rs / app.rs skeleton (module root `app.rs`)
pub(crate) fn routes() -> RouterBuilder { module_router!().layer(LayerFn::new(None::<&str>, edge)) }

// serving
let router = routes()
    .discover()                                  // collects absolute-path #[route]/#[page]/#[procedure]; ABSENT => those 404
    .with(OriginPolicy::trust_origins(["http://127.0.0.1:8091"]))
    .assets(AssetBundle::load_dir(&dir)?)
    .layer(RuntimeLayer::new())                  // after app layers
    .build();
axum::Router::new().fallback_service(TowerService::new(router))   // no body adaptation needed
// ConnectInfo<SocketAddr> reaches cx only when serving with into_make_service_with_connect_info::<SocketAddr>()
// (tuita-server's serve_on does); read it with extensions(cx).get::<axum::extract::ConnectInfo<SocketAddr>>().
// remote_addr(cx) is None unless a layer copies it into RemoteAddr: do not rely on it.
```

Reading the request:
```rust
use topcoat::router::request::{headers, method, uri, extensions, remote_addr, original_method};
headers(cx).get("cookie").and_then(|v| v.to_str().ok());
uri(cx).path(); uri(cx).query(); method(cx);
#[query_params(error = bad_request)] struct Q { page: Option<u32> }   // then query_params::<Q>(cx)?
```

Procedures / shards / `$()`:
```rust
#[procedure]                       // or #[procedure("/_proc/x")] for a fixed path; default /_topcoat/runtime/procedures/<hash>
async fn set_cookie(cx: &Cx, name: String) -> Result<Result<String, String>> {
    if name.is_empty() { return Ok(Err("empty".into())); }       // inner Err = user-level error
    response_headers(cx).append(header::SET_COOKIE, HeaderValue::from_str(&format!("sid={name}; Path=/; SameSite=Lax"))?);
    Ok(Ok(name))
}
#[shard]                           // POST /_topcoat/runtime/shards/<hash>, NDJSON; args are signals or $(..)
async fn sig_shard(cx: &Cx, msg: Signal<String>) -> Result<impl View> { let m = msg.get(); Ok(view!{ <p>(m)</p> }) }
// in a component:
let q = signal(cx, || "x".to_owned());
view! {
  <input :value=$(q.get()) @input=$(|e: Event| q.set(e.target.value))>
  sig_shard(msg: q)                // or echo_shard(msg: $(q.get())) for a String-typed arg
  <button @click=$(async |_e| {
      let r = set_cookie("abc".to_owned()).await;
      if r.is_ok() { out.set(r.unwrap()); } else { err.set(r.unwrap_err()); }   // `match` is NOT supported inside $()
  })>"go"</button>
  <section @tuita-stream=$(|e: Event| msg.set(e.data))> ... </section>
  raw!("document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${m}}))", m.is_empty());
}
```
Wire format: procedures `POST` JSON array of args, answer `{"t":"Result","ok":..}` / `{"t":"Result","err":..}`; a bare `Err(..)` (outer) is an HTTP error and the browser expression fails. `#[component]` async fns are called in `view!` as `name(arg: value, attrs: attributes!{..}, child content)`.

Responses from pages:
```rust
view! { (StatusCode::FORBIDDEN) ... }                                  // status node, first wins
view! { ((header::CACHE_CONTROL, HeaderValue::from_static("no-store"))) ... }   // header node
Err(see_other("/x").into())                                            // 303; Err(forbidden().into()) => 403
#[route(GET "/_tuita/x/{acct}")] async fn f(cx: &Cx) -> Result<ViewHandle> { view!{..}.first().await }   // absolute fragment route, NO layout; needs .discover(); `use topcoat::view::ViewExt`
// absolute #[page("/..")] DOES get the root layout.
```
`Box::pin(page.first()).await?` for large futures (`large_futures` lint). Variables used only inside `raw!` need the Rust fallback arg. No lint fired inside Topcoat macro expansions under the workspace lints.

Fragment layout: root `#[layout]` checks the header and returns only `(slot)` for `x-tuita-fragment: 1`; error pages under the fragment header are plain text unless the `error_boundary` sits outside the `if`.

Links: `href!(handle::profile, handle: h).resolve(cx)`; module params must be `pub(crate)` to be named from other modules; `href!(explore::explore).query([("before","x y")]).fragment("top")`. Trailing slash `/explore/` => 308 to `/explore`. A guard in a layer: `Err(see_other(..).into())` gives 303.

`live!`/`emit!`: `(live! { emit!{ <p>"loading"</p> }?; ...; emit!{ <p>"done"</p> } })` streams into a region; not exercised beyond rendering.

Network shapes: procedures/shards are plain POSTs; a same-origin Chromium POST passes the origin policy via `Sec-Fetch-Site: same-origin`.

## chromiumoxide snippets (tested)

```rust
use chromiumoxide::{Browser, BrowserConfig, Page, cdp::browser_protocol::{
  dom::SetFileInputFilesParams, browser::BrowserContextId, log::EventEntryAdded,
  emulation::{MediaFeature, SetDeviceMetricsOverrideParams, SetEmulatedMediaParams},
  network::{EnableParams as NetEnable, Headers, SetExtraHttpHeadersParams},
  page::{AddScriptToEvaluateOnNewDocumentParams, CaptureScreenshotFormat},
  target::{CreateBrowserContextParams, CreateTargetParams},
  web_authn::{AddVirtualAuthenticatorParams, AuthenticatorProtocol, AuthenticatorTransport,
    EnableParams as WaEnable, GetCredentialsParams, VirtualAuthenticatorOptions}},
  cdp::js_protocol::runtime::{EvaluateParams, EventConsoleApiCalled, EventExceptionThrown}};
use futures_util::StreamExt;

let cfg = BrowserConfig::builder().chrome_executable(chrome_path).no_sandbox().new_headless_mode().build().map_err(anyhow::Error::msg)?;
let (browser, mut handler) = Browser::launch(cfg).await?;
let h = tokio::spawn(async move { while handler.next().await.is_some() {} });
// Browser is not Clone: share as Arc<Browser>; Arc::try_unwrap before close.
browser.close().await?; browser.wait().await?; h.abort();

let ctx = browser.create_browser_context(CreateBrowserContextParams::default()).await?;
let page = browser.new_page(CreateTargetParams::builder().url("about:blank").browser_context_id(ctx.clone()).build().map_err(anyhow::Error::msg)?).await?;
page.execute(NetEnable::default()).await?;
page.execute(SetExtraHttpHeadersParams::new(Headers::new(serde_json::json!({"x-forwarded-for": ip})))).await?;
// teardown: page.close().await?; browser.dispose_browser_context(ctx).await?;

page.execute(AddScriptToEvaluateOnNewDocumentParams::new(include_str!("../js/find.js"))).await?;
page.execute(SetEmulatedMediaParams::builder().features(vec![MediaFeature::new("prefers-color-scheme","dark"), MediaFeature::new("prefers-reduced-motion","reduce")]).build()).await?;
page.execute(SetDeviceMetricsOverrideParams::new(390, 844, 2.0, false)).await?;

page.enable_runtime().await?; page.enable_log().await?;
let mut c = page.event_listener::<EventConsoleApiCalled>().await?;  // ev.r#type, ev.args[i].value
let mut e = page.event_listener::<EventExceptionThrown>().await?;   // ev.exception_details.exception.as_ref().and_then(|x| x.description.clone())
let mut l = page.event_listener::<EventEntryAdded>().await?;        // ev.entry.source/level/text (CSP violations land here)

page.execute(WaEnable::default()).await?;
let opts = VirtualAuthenticatorOptions::builder().protocol(AuthenticatorProtocol::Ctap2).transport(AuthenticatorTransport::Internal)
    .has_resident_key(true).has_user_verification(true).is_user_verified(true).automatic_presence_simulation(true).build().map_err(anyhow::Error::msg)?;
let auth = page.execute(AddVirtualAuthenticatorParams::new(opts)).await?.result.authenticator_id;

let el = page.find_element("#f").await?;
page.execute(SetFileInputFilesParams::builder().files(vec![path_string]).backend_node_id(el.backend_node_id).build().map_err(anyhow::Error::msg)?).await?;

let v: serde_json::Value = page.evaluate(EvaluateParams::builder().expression("(async()=>({a:1}))()").await_promise(true).return_by_value(true).build().map_err(anyhow::Error::msg)?).await?.into_value()?;
let png = page.screenshot(chromiumoxide::page::ScreenshotParams::builder().format(CaptureScreenshotFormat::Png).build()).await?;
```
Launch takes ~200 ms; 4 parallel contexts worked; `browser.close()` + `wait()` leaves no chromium processes.
