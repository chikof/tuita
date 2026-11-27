//! Who can see a post.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::Visibility;

use crate::{icons::Icon, l10n::t, ui::icon::icon};

/// Says who can see a post. Public posts need no badge, so they render nothing.
#[component]
pub(crate) async fn visibility_badge(cx: &Cx, visibility: Visibility) -> Result<impl View> {
    let kind = match visibility {
        Visibility::Public => None,
        Visibility::Unlisted => Some((
            Icon::EyeOff,
            t!(visibility_unlisted).resolve(cx).into_owned(),
            t!(visibility_unlisted_hint).resolve(cx).into_owned(),
        )),
        Visibility::Followers => Some((
            Icon::Lock,
            t!(visibility_followers).resolve(cx).into_owned(),
            t!(visibility_followers_hint).resolve(cx).into_owned(),
        )),
        Visibility::Direct => Some((
            Icon::Mail,
            t!(visibility_direct).resolve(cx).into_owned(),
            t!(visibility_direct_hint).resolve(cx).into_owned(),
        )),
    };
    Ok(view! {
        if let Some((glyph, label, hint)) = kind {
            <span
                class="badge"
                data-visibility=(visibility.as_str())
                title=(hint)
                data-c="visibility-badge"
            >
                icon(name: glyph, size: 12)
                (label)
            </span>
        }
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn public_posts_have_no_badge() {
        let html = render("en", |cx| {
            view! { visibility_badge(visibility: Visibility::Public) }.boxed()
        });
        assert!(html.is_empty(), "{html}");
    }

    #[test]
    fn restricted_posts_say_who_sees_them() {
        let html = render("en", |cx| {
            view! { visibility_badge(visibility: Visibility::Followers) }.boxed()
        });
        assert!(html.contains(r#"data-visibility="followers""#), "{html}");
        assert!(html.contains("Followers only"), "{html}");
    }
}
