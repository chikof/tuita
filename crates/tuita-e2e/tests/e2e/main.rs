//! Browser end-to-end suite. Run it with `scripts/e2e.sh [filter]`, which builds the server,
//! starts it on a throwaway database and sets the `E2E_*` environment; `cargo test` skips this
//! target (`test = false` in Cargo.toml). One module per area, one owner per file.

mod admin;
mod admin_tabs;
mod auth;
mod compose;
mod design;
mod e2ee;
mod emotes;
mod extras;
mod federation;
mod js_units;
mod languages;
mod media;
mod messages;
mod oauth;
mod passkeys;
mod posts;
mod profiles;
mod regressions;
mod reply_dialog;
mod settings;
mod smoke;
mod social;
mod visual;
