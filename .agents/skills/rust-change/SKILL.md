---
name: rust-change
description: Load before writing or editing Rust code in this repo: choosing where code goes, error types, adding a dependency, changing public API, writing docs or doctests.
---

# Rust change

Read `docs/architecture.md` first: it says where code lives for this project kind. Read `docs/rust-style.md` once per task; it separates **[tool]** rules (the gate enforces them) from **[review]** rules (you enforce them).

## Before editing

- Find an existing pattern and copy its shape. Do not add a second convention. `grep` for a sibling before inventing.
- Smallest diff that does the job. No drive-by refactors, no renames of untouched code. Note unrelated problems as a new issue.

## Where code goes

- `main.rs` stays thin: parse config/args, set up `tracing`, call into the library, map the error to an exit. No logic there.
- Logic goes in the library, tested without spawning anything.
- Split a file by its main item (`foo.rs` for `Foo`) when it grows. `lib.rs` keeps `mod` lines, re-exports, crate docs.
- Unit tests: `#[cfg(test)] mod test` at the bottom of the same file. Black-box tests: `tests/`.

## Errors

| Where | Use | Rules |
|---|---|---|
| Library / core crate | `thiserror` enum | lowercase message, no trailing period, `#[source]` for causes, `#[non_exhaustive]` on public enums |
| Binary / application glue | `anyhow::Result` | `.context("what we were doing")` at each boundary |

- No `unwrap()` outside tests (`unwrap_used` is gated). Use `expect("why this cannot fail")` only for invariants types cannot express.
- Document `# Errors` and `# Panics` on every public fallible fn (`missing_errors_doc`, `missing_panics_doc` are gated).
- No `anyhow` in a library's public API.
- Big error type (over 128 bytes) gets boxed (`result_large_err`).

## Dependencies

1. Can `std` do it in a few lines? Then no dependency.
2. `cargo add <crate>` (add `-p <crate>` in a workspace). Never edit `Cargo.lock` by hand.
3. Trim features: `cargo add <crate> --no-default-features --features a,b` when defaults are heavy.
4. Run `cargo deny check`. A licence or advisory failure means pick another crate, not edit `deny.toml`.
5. Say in the commit body which crate, why it is needed, and what you rejected.

## Public API

- Every `pub` item has docs (`missing_docs`). Make things `pub(crate)` unless the API really needs them (`unreachable_pub`).
- Libraries: `#[non_exhaustive]` or private fields on public structs and enums (`exhaustive_*` are warned for libs; application kinds allow them).
- Add a runnable doctest for any non-obvious usage. Use `?` with a hidden `# Ok::<(), Error>(())` tail.
- `#[must_use]` on pure returns (`must_use_candidate`).
- Breaking change: removing or changing a public item, adding a variant to an exhaustive enum, adding a required trait method, tightening a bound. Mark it in the commit (`!` and a `BREAKING CHANGE:` footer), bump the version as `docs/architecture.md` says.

## Style that review catches

- `&str`, `&[T]`, `&Path` in parameters, not `&String`, `&Vec<T>`, `&PathBuf` (`ptr_arg`).
- Enums over bool flags; more than three bools in a fn or struct is warned.
- `tracing` macros with fields, never `println!` for diagnostics: `info!(path = %p.display(), "loaded")`.
- Name things in the domain's words; no `utils`/`helpers` modules.

## Layout

Types that sit in big collections, cross a hot path, or are error types: apply skill `rust-layout`. Add a size pin when you introduce or reshape such a type.

## Finish the step

1. `cargo check -p <crate>`, then the narrow test (skill `verify`).
2. `cargo fmt --all` on files you touched.
3. `check` before closing the task.
4. Public API changed: confirm `cargo doc --no-deps` shows the docs you wrote, and doctests pass: `cargo test --doc`.
