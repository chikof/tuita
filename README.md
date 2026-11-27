# tuita

tuita: a small social platform with a 1-bit dither look that federates over ActivityPub.

## Setup

With Nix, `direnv allow` (or `devenv shell`) gives you the pinned Rust toolchain, the tools and the git hooks.
Without Nix, install the toolchain from `rust-toolchain.toml`, `cargo-deny`, `cargo-nextest`, `shellcheck`, `typos`, and for releases `git-cliff` and `cargo-edit`.

```sh
check                         # fmt, clippy, tests, docs, cargo-deny, shellcheck, typos (scripts/check.sh)
scripts/release.sh --dry-run  # next version and its changelog, from Conventional Commits
scripts/release.sh            # bump version, regenerate CHANGELOG.md with git-cliff, commit, tag
```

## Mastodon clients

A tuita instance speaks the Mastodon client API. To connect a Mastodon client (Phanpy, Elk, Ice Cubes, Tusky, ...), enter the instance's domain in the client and sign in on tuita's consent page when it opens. Supported routes, scopes and known gaps are in [`docs/architecture.md`](docs/architecture.md#mastodon-api). After a deploy, `scripts/mastodon-smoke.sh <base-url>` checks the whole flow against the running instance.

## Layout of the docs

- [`docs/architecture.md`](docs/architecture.md): code layout and where to add things.
- [`docs/rust-style.md`](docs/rust-style.md): style rules and which lint enforces each.
- [`AGENTS.md`](AGENTS.md): the short entry point for coding agents.

## License

AGPL-3.0-or-later. See [`LICENSE`](LICENSE). Operators must offer the source to their users: set `SOURCE_URL`.
