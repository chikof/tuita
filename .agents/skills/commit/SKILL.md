---
name: commit
description: Load before every git commit, when staging changes, creating or merging a branch, or integrating work back into main.
---

# Commit

The repo is local. There is no forge, no pull request, no CI. `main` is the integration branch and you merge into it yourself.

## Message format

Conventional Commits, enforced by the `convco` commit-msg hook:

```
type(scope): subject

optional body: why, not what
```

- `type`: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `chore`. Breaking change: `feat(scope)!: subject` plus a `BREAKING CHANGE:` footer.
- `scope`: the crate or module a reader would recognise (`feat(parser): accept empty input`), or none.
- Subject: imperative, lowercase start, no trailing period, about 50 characters, up to 72.
- Body: the reason and any trade-off, wrapped at 72. Skip it when the subject says enough.
- The subject is the changelog entry: `CHANGELOG.md` is generated from `feat`, `fix`, `perf`, `refactor` and `docs` subjects by git-cliff (`cliff.toml`). Write it for a user, not for the diff. `test`, `style`, `chore`, `build` and `ci` stay out of the changelog.
- Hook rejects the message: read its output, reword, commit again. Never `--no-verify`.

## When to commit

- After each green step. Small commits that each pass.
- Green means the step's narrow verification passed and the code compiles (skill `verify`). Run `check` before the last commit of the change.
- Never commit with a failing `check` or a step that does not compile. `wip` is not an accepted type (the `convco` hook rejects it): a half-done step stays uncommitted, or goes in `git stash`.
- Never commit directly on `main`: work on a branch.

## Stage with care

```sh
git status --short
git add -p                  # review each hunk; stage only this step's change
git diff --cached           # read what will be committed
git commit
```

- No `git add -A` or `git add .` unless `git status` shows only your files.
- Unrelated change in the tree: leave it unstaged, or put it in its own commit.
- Whitespace-only or formatting-only churn in untouched files: revert it.
- Read the staged diff for `dbg!`, `println!` debugging, `todo!()`, commented-out code, stray files.

## Never commit

- `.env`, `.env.local`, tokens, keys, any secret (the `ripsecrets` hook blocks many; do not rely on it).
- `target/`, `.devenv*`, `.direnv`, `.pre-commit-config.yaml`, `.bacon-locations`, `devenv.local.nix`. These are in `.gitignore`; if one shows in `git status`, the ignore is broken: fix it.
- Generated or lock files edited by hand. `Cargo.lock` changes only via `cargo` and is committed.
- Large binaries and build output.

Committed a secret: stop. Do not push it anywhere. Tell the human; rotating the secret is theirs to do. Rewriting history is their call, not yours.

## Branches and worktrees

```sh
git switch -c <branch> main                        # a branch from a clean main
git worktree add ../<repo>-<branch> -b <branch> main   # or a separate worktree
git worktree list
git worktree remove ../<repo>-<branch>             # after merging
```

One agent, one change, one branch, one working tree. Parallel agents each get their own worktree. Never run `git checkout`, `reset --hard`, `clean` or `stash` in a tree another agent uses.

## Merge back to main

After the final commit:

```sh
git switch <branch>
git rebase main                 # resolve conflicts here, not on main
check                           # must be green after the rebase
git switch main
git merge --ff-only <branch>    # refuses if main moved; rebase again
git branch -d <branch>
```

- Several commits for one logical change: `git rebase -i main` and squash them, each result with a valid Conventional message. Tool without a TTY: `git reset --soft main` then recommit is the fallback, on your own branch only.
- No merge commits on `main`: history stays linear.
- Conflict you cannot resolve with certainty: stop and ask the human.
- Pick the type for the reader: `feat`, `fix`, `perf`, `refactor` and `docs` reach the changelog; `test`, `style`, `chore`, `build` and `ci` do not.

## Check yourself

- `git log --oneline -5` reads as a story.
- `git status` is clean after the commit.
- `git diff main --stat` shows no surprise files.

## Releases

Only when the human asks. `scripts/release.sh --dry-run` prints the next version (from commits since the last `v*` tag: `feat` bumps minor, `fix` bumps patch, minor for breaking before 1.0) and its changelog section. `scripts/release.sh` (or `scripts/release.sh X.Y.Z`) runs `check`, sets the Cargo version, regenerates `CHANGELOG.md`, commits `chore(release): vX.Y.Z` and tags it. It refuses off `main`, on a dirty tree, for an existing tag, or with nothing user-facing to release. Nothing is pushed.
