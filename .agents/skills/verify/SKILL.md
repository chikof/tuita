---
name: verify
description: Load before claiming any change works, when choosing which checks to run, when reproducing a bug, or when a test is flaky or failing.
---

# Verify

Prove it with a command and its observed output. Claims without output do not count.

## Ladder: narrow to wide

Run the cheapest rung that can fail for your change. Climb only when it passes.

| Rung | Command | Catches |
|---|---|---|
| 1 | `cargo check -p <crate>` | type and borrow errors, seconds |
| 2 | `cargo nextest run <filter>` (or `cargo test <name>`) | the one behaviour you touched |
| 3 | `cargo nextest run -p <crate>` | the crate's other tests |
| 4 | `cargo test --doc -p <crate>` | doctests (nextest does not run them) |
| 5 | `check` | the full gate: everything below |

`check` is `scripts/check.sh`: `cargo fmt --check`, clippy with `-D warnings`, tests, docs with `-D warnings`, `cargo deny`, `shellcheck`, `typos`. All cargo steps use `--locked`.
Never end a task on rung 1 to 4. Done needs `check` exit 0.

Background loop while editing: `bacon` (clippy), `bacon test`, `bacon check`. Read it, but still run the commands for the log.

## Exercise the changed path

A passing suite proves nothing if no test reaches your change.

- **Binary / CLI**: run it. `cargo run -- <args>`; check stdout, stderr and `echo $?`.
- **Service**: `devenv up`, then `curl -i` the route. See skill `service-ops`.
- **Library**: a doctest or `tests/api.rs` case that calls the new public item.
- **Internal logic**: a unit test that fails when you break the line. Break it on purpose once, see red, restore.
- **Types and layout**: `scripts/type-sizes.sh <Type>` (skill `rust-layout`).

## Bug fixes: reproduce first

1. Write a failing test (or exact command) that shows the bug.
2. Run it. Record the failure text. A repro you never saw fail is not a repro.
3. Fix the root cause (skill `debug`).
4. Run it again. Record the pass. Keep the test as the regression test.

## What to report

One line per verification, in the form `<what> | <command> | <observed>`, in the issue or pull request:

```
empty input -> Err | cargo nextest run parse_empty | 1 passed, was failing before fix
full gate | check | exit 0
```

- Quote the decisive line of output: counts, status code, error text.
- Failure counts too: report what failed and what you changed.
- Say which rung you reached. "Only rung 2 so far" is honest and fine mid-task.

## Never

- Delete, `#[ignore]`, or loosen an assertion to get green.
- Add `#[allow(...)]`, edit `[lints]`, `clippy.toml`, `deny.toml` or `scripts/check.sh` to silence a gate. Fix the code. A real false positive: one narrow `#[allow]` or `#[expect]` with a comment stating why.
- Run `cargo fmt` over unrelated files then call it a fix.
- Report green from a stale run. Re-run after the last edit.
- Skip `--locked`. A changed `Cargo.lock` must come from `cargo`, never by hand.

## Failing test you did not touch

1. Run it alone: `cargo nextest run <name>`. Fails alone, so it is a real failure. Passes alone, so it depends on order or shared state.
2. On the base commit, in a scratch worktree: `git worktree add ../<repo>-base main`, run the test there, then `git worktree remove ../<repo>-base`. Fails there too, so it is pre-existing: report it to the human, do not mask it.
3. Yours: you broke it. Fix the change, not the test, unless the change is meant to alter that behaviour. Then update the test and say why.

## Flaky tests

- Re-run 20 times: `cargo nextest run <name> --retries 0 --stress-count 20` (or a shell loop with `cargo test <name>`).
- Find the cause: shared files or ports, wall-clock sleeps, hash-map order, thread timing. Fix it: temp dirs per test, port `0`, injected clock, sorted output.
- `--retries` in `check` or `#[ignore]` hides it. Not allowed. Report the cause and the fix.
