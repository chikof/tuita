# tuita

tuita: a small social platform with a 1-bit dither look that federates over ActivityPub. Kind: `workspace`. Layout and where code goes: `docs/architecture.md`.

## Commands

- `check` (same as `scripts/check.sh`): the gate. Fmt, clippy, test, doc, deny, shellcheck, typos. Outside the shell: `devenv shell -- check`.
- `topcoat` (from `topcoat-cli` 0.12.1, installed by devenv): `topcoat asset bundle --package tuita` (the web client's asset bundle, required before `cargo run` and the `tuita-web` tests), `topcoat fmt --check`, `topcoat l10n check --package tuita`; `check` runs all three.
- `scripts/e2e.sh [filter]`: the Rust and Chromium browser suite (`crates/tuita-e2e`) against a release server and a throwaway database.
- `scripts/lint-strings.sh`: flags literal user-visible text in `crates/tuita-web` markup; use `t!(key)`. Part of `check`.
- `cargo nextest run [filter]` or `cargo test [name]`: tests. `bacon`: background checker.
- `scripts/type-sizes.sh`: print type layouts, used for size pins.
- `scripts/release.sh [X.Y.Z|--dry-run]`: version, `CHANGELOG.md` (git-cliff) and tag. Only when the human asks.

## Workflow

1. Work is tracked in GitHub issues and pull requests. Parallel agents: one worktree per change, never a shared tree.
2. Work one step at a time: narrow check, change, narrow verify. Commit after each green step.
3. Report the command you ran and what you saw. No "should work".
4. Done means `check` is green.

## Rules

- Never edit generated or lock files by hand. `Cargo.lock` changes only through cargo; `CHANGELOG.md` only through `scripts/release.sh`.
- Add dependencies with `cargo add`; give the reason in the commit body.
- No `unwrap`/`expect` outside tests. Errors: `thiserror` in libraries, `anyhow` in binaries.
- Hot or collected types get a size pin; measure with `scripts/type-sizes.sh`.
- Never weaken a lint, test or gate to get green. Fix the cause.
- Never commit `.env`, `target/`, secrets.
- Ask the human only for product decisions.

## Skills

Load a skill from `.agents/skills/<name>/SKILL.md` only when its trigger applies.

| Skill | Load when |
|---|---|
| `verify` | proving a change works, reproducing a bug, deciding what counts as evidence |
| `rust-change` | editing Rust: where code goes, errors, deps, public API, docs |
| `rust-layout` | adding or changing a struct/enum, size pins, `clippy.toml` or layout lint output |
| `debug` | a test fails, behaviour is wrong, or the cause is unknown |
| `commit` | committing, branching, merging back to `main` |

If `.agents/skills/service-ops/` exists, load it to run, configure or containerise the service.
