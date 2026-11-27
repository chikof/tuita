---
name: debug
description: Load when something fails and the cause is not obvious: a panic, wrong output, a failing or hanging test, a regression, or a second fix attempt that did not work.
---

# Debug

Find the cause before changing code. A fix you cannot explain is a guess.

## Loop

1. **Reproduce.** Get a command that fails every time. Record it and the failure text.
2. **Minimize.** Cut input, config and code until the failure still happens and nothing else is left. Smaller repro, fewer suspects.
3. **Hypothesize.** Write one sentence: "X happens because Y". It must predict something you can observe.
4. **Instrument.** Add the cheapest probe that confirms or kills the hypothesis. Run. Read the result.
5. **Bisect** when the cause is not near the symptom (below).
6. **Fix the root cause.** Not the symptom, not the test.
7. **Regression test.** The step-1 repro becomes a test; it fails without the fix, passes with it (skill `verify`).
8. **Clean up** probes. Report cause, fix and evidence.

One variable per experiment. If a hypothesis dies, write down why, then form the next. Two failed fixes in a row: stop editing, go back to step 2.

## Reading failures

```sh
RUST_BACKTRACE=1 cargo run -- <args>          # panic with frames
RUST_BACKTRACE=full cargo nextest run <name>  # untrimmed frames
cargo nextest run <name> --no-capture         # see stdout/stderr of a test
cargo test <name> -- --nocapture --test-threads=1
```

- Read the first error, not the last. Compiler errors cascade from the top one.
- Panic message plus the first frame in your crate names the line. Frames below it are the caller.
- `Err` chain from `anyhow`: print with `{err:#}` for the full chain.
- Clippy or `check` failure: fix the first reported item, rerun, repeat.

## Instrument

Prefer `tracing`; it stays useful:

```sh
RUST_LOG=debug cargo run -- <args>
RUST_LOG=info,<crate>=trace cargo nextest run <name> --no-capture
```

- In a test, install a subscriber once: `let _ = tracing_subscriber::fmt().with_test_writer().try_init();`.
- Log fields, not strings: `debug!(len = buf.len(), ?state, "parsed header")`.
- Quick look: `dbg!(&value)` prints file, line and value to stderr. It moves its argument, so pass a reference. **Remove every `dbg!` before the commit**: `grep -rn 'dbg!' src tests`.
- Assertions as probes: `debug_assert!` for invariants you suspect.
- No debugger needed for most bugs. If you do need one: `rust-gdb target/debug/<bin>` or `rust-lldb`.
- Macro-heavy code: read the compiler's note on the failing expansion; no `cargo expand` required. Split the macro call into plain code and compile that.

## Bisect

Over history (the bug is a regression):

```sh
git bisect start
git bisect bad                 # current commit
git bisect good <known-good>   # an older commit or tag
git bisect run scripts/check.sh            # slow, whole gate
git bisect run cargo nextest run <name>    # fast, only the repro
git bisect reset
```

The command must exit 0 for good and non-zero for bad. Use a worktree so your task tree is untouched: `git worktree add ../<repo>-bisect HEAD`, run there, remove it after (`git worktree remove ../<repo>-bisect`).

Over code (no history): halve it. Comment out or stub half the suspect path, test, narrow. Over input: delete half the lines, keep the half that still fails.

Over time (hang): `RUST_LOG=trace`, find the last event printed, the cause is the next step. Add a `timeout 10 <cmd>` around runs so the loop never blocks.

## Common Rust culprits

| Symptom | Check |
|---|---|
| Panic on index/slice | length assumptions, empty input, UTF-8 byte offsets in `&s[a..b]` |
| Test passes alone, fails in suite | shared state, env vars, fixed port, working directory, order |
| Wrong value after refactor | shadowed variable, clone vs reference, `Option` default hides `None` |
| Hang | lock held across `.await`, channel never closed, blocking call in async code |
| Works in debug, fails in release | integer overflow (wraps in release), `debug_assert!`, uninitialised timing |
| Parse accepts bad input | `trim` / `parse` leniency, missing range check |

## Report

One entry per finding, in this order: repro command, observed failure, cause, fix commit, regression test name and its pass. Cause unknown after a real effort: stop and tell the human what you ruled out.
