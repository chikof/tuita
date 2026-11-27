//! The page frame: toaster, shortcuts dialog, drawer, rails and bars.
//!
//! The root layout wraps the page in [`app_shell::app_shell`] for a signed-in visitor on every
//! route except the [bare](wants_shell) ones; guests and the bare pages (`/login`, `/signup`,
//! `/oauth/*`, `/design`) render without it. Pages call [`page_bar::page_bar`] for their header
//! and set their tab title with [`titled`].

pub(crate) mod account_menu;
pub(crate) mod announcement_banner;
pub(crate) mod app_shell;
pub(crate) mod bottom_bar;
pub(crate) mod drawer;
pub(crate) mod globals;
pub(crate) mod nav;
pub(crate) mod page_bar;
pub(crate) mod parts;
pub(crate) mod rail_nav;
pub(crate) mod right_rail;
pub(crate) mod shortcuts_dialog;
pub(crate) mod top_bar;

use topcoat::{Result, context::Cx, metadata::Metadata};

use crate::{l10n::t, session};

/// Routes that never get the shell, even when signed in.
const BARE: [&str; 4] = ["/design", "/login", "/signup", "/oauth"];

/// Whether a signed-in visitor sees `path` inside the app shell.
#[must_use]
pub(crate) fn wants_shell(path: &str) -> bool {
    !BARE.iter().any(|root| {
        path == *root
            || path
                .strip_prefix(root)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// The tab title of a page: `title · tuita`, with the unread count in front (`(3) `, `(99+) `)
/// for a signed-in visitor. Use it from `#[page(with(Metadata::from_fn(..)))]`.
///
/// # Errors
///
/// Fails when the viewer cannot be loaded.
pub(crate) async fn titled(cx: &Cx, title: &str) -> Result<Metadata> {
    let unread = match session::viewer(cx).await? {
        Some(viewer) => viewer.badges.notifications + viewer.badges.messages,
        None => 0,
    };
    let prefix = match unread {
        0 => String::new(),
        1..=99 => format!("({unread}) "),
        _ => "(99+) ".to_owned(),
    };
    let app = t!(app_name).resolve(cx);
    Ok(Metadata::new().title(format!("{prefix}{title} · {app}")))
}

#[cfg(test)]
mod test {
    use super::wants_shell;

    #[test]
    fn the_bare_routes_and_their_subtrees_have_no_shell() {
        for bare in [
            "/login",
            "/signup",
            "/design",
            "/design/x",
            "/oauth/authorize",
        ] {
            assert!(!wants_shell(bare), "{bare}");
        }
        for shelled in [
            "/",
            "/explore",
            "/@ada",
            "/settings/about",
            "/terms",
            "/loginx",
        ] {
            assert!(wants_shell(shelled), "{shelled}");
        }
    }
}
