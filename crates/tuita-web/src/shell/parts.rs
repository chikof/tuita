//! Small pieces the shell components share.

use topcoat::{
    Result,
    context::Cx,
    runtime::Event,
    view::{Child, View, attributes, component, view},
};
use tuita_server::dto::{AvatarShape as ShapeDto, Ink};

use crate::{
    l10n::t,
    shell::nav::BadgeKind,
    ui::{
        avatar::{AvatarShape, avatar},
        button::{ButtonSize, ButtonVariant, button},
    },
};

/// The unread counter of a nav entry.
///
/// It is the `ui::badge` pill with the hooks `stream.js` updates live: `data-badge` names the
/// counter, `[data-count]` holds the digits (capped at `99+`) and `[data-label]` the screen-reader
/// sentence. At zero the pill is `hidden`.
#[component]
pub(crate) async fn nav_badge(cx: &Cx, kind: BadgeKind, count: i64) -> Result<impl View> {
    let shown = if count > 99 {
        "99+".to_owned()
    } else {
        count.max(0).to_string()
    };
    let label = t!(badge_unread, count: i32::try_from(count).unwrap_or(i32::MAX));
    Ok(view! {
        <span class="badge" data-c="badge" data-badge=(kind.name()) hidden=(count <= 0)>
            <span data-count="" aria-hidden="true">(shown)</span>
            <span class="sr-only" data-label="">(label)</span>
        </span>
    })
}

/// The avatar of an account card: `src` when it uploaded a picture, else its identicon, clipped
/// to its own shape and drawn in its own ink. The name sits next to it, so the picture has no
/// alt text.
#[component]
pub(crate) async fn account_avatar(
    cx: &Cx,
    handle: &str,
    #[default]
    #[into]
    src: Option<&str>,
    shape: ShapeDto,
    ink: Ink,
    #[default(40)] size: u32,
) -> Result<impl View> {
    let shape = AvatarShape::from_name(shape.as_str()).unwrap_or_default();
    Ok(view! {
        avatar(
            seed: handle,
            src: src,
            alt: "",
            shape: shape,
            size: size,
            ink: ink.as_str()
        )
    })
}

/// A button that opens `href` with a full page load (a link styled as a button).
#[component]
pub(crate) async fn nav_button(
    cx: &Cx,
    href: &str,
    #[default] variant: ButtonVariant,
    #[default] size: ButtonSize,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let to = href.to_owned();
    Ok(view! {
        button(
            variant: variant,
            size: size,
            attrs: attributes! {
                @click=$(|_e: Event| raw!("location.assign(${to})", to.is_empty()))
            },
            (child)
        )
    })
}
