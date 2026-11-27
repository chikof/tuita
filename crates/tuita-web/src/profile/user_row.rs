//! One person in a list: picture, name, handle, bio and the follow
//! button. Plain markup, because lists of people arrive as fragments `infinite.js` inserts.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::UserDto;
use uuid::Uuid;

use crate::{
    fmt::{acct_of, handle_of},
    l10n::t,
    paths,
    post::{
        domain_chip::domain_chip,
        rich_text::{rich_text, segments_of, user_name},
    },
    profile::follow_button::follow_button_view,
    session,
    ui::avatar::{AvatarShape, avatar},
};

/// A person row for the signed-in viewer.
#[component]
pub(crate) async fn user_row(cx: &Cx, user: &UserDto) -> Result<impl View> {
    let viewer = session::viewer(cx).await?.as_ref().map(|v| v.me.user.id);
    Ok(view! { user_row_view(user: user, viewer: viewer) })
}

/// [`user_row`] for a known viewer id.
#[component]
pub(crate) async fn user_row_view(
    cx: &Cx,
    user: &UserDto,
    viewer: Option<Uuid>,
) -> Result<impl View> {
    let acct = acct_of(&user.handle, user.domain.as_deref());
    let href = paths::profile(&acct);
    let shape = AvatarShape::from_name(user.avatar_shape.as_str()).unwrap_or_default();
    let segments = segments_of(&user.bio);
    Ok(view! {
        <div class="row wash" data-c="user-row">
            <a
                class="avatar"
                href=(&href)
                tabindex="-1"
                aria-hidden="true"
                data-hovercard=(&acct)
            >
                avatar(
                    seed: &user.handle,
                    src: user.avatar_url.as_deref(),
                    alt: "",
                    shape: shape,
                    ink: user.ink.as_str(),
                    size: 48
                )
            </a>
            <div class="text">
                <a class="who" href=(&href) data-hovercard=(&acct)>
                    user_name(
                        name: &user.display_name,
                        verified: user.verified,
                        bot: user.bot,
                        emojis: &user.emojis
                    )
                    <span class="handle">
                        (handle_of(&user.handle, user.domain.as_deref()))
                    </span>
                </a>
                if user.domain.is_some() || user.relation.follows_you {
                    <span class="badges">
                        if let Some(domain) = &user.domain {
                            domain_chip(domain: domain)
                        }
                        if user.relation.follows_you {
                            <span class="tag">(t!(follows_you))</span>
                        }
                    </span>
                }
                if !user.bio.is_empty() {
                    <p class="bio">
                        rich_text(
                            segments: &segments,
                            domain: user.domain.as_deref(),
                            emojis: &user.emojis
                        )
                    </p>
                }
            </div>
            follow_button_view(user: user, viewer: viewer)
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{profile::testing, testing::render};

    fn show(user: UserDto, viewer: Option<Uuid>) -> String {
        render("en", |cx| {
            view! { user_row_view(user: &user, viewer: viewer) }.boxed()
        })
    }

    #[test]
    fn a_row_links_the_profile_and_offers_the_follow_button() {
        let mut ada = testing::user("ada");
        ada.bio = "likes dithering".to_owned();
        ada.relation.follows_you = true;
        let html = show(ada, Some(Uuid::from_u128(1)));
        assert!(html.contains(r#"href="/@ada""#), "{html}");
        assert!(html.contains(r#"data-hovercard="ada""#), "{html}");
        assert!(html.contains("@ada"), "{html}");
        assert!(html.contains("Follows you"), "{html}");
        assert!(html.contains("likes dithering"), "{html}");
        assert!(html.contains("data-follow"), "{html}");
    }

    #[test]
    fn a_remote_row_shows_its_domain_and_links_the_full_account() {
        let html = show(testing::remote("mara", "fedi.example"), None);
        assert!(html.contains(r#"href="/@mara@fedi.example""#), "{html}");
        assert!(html.contains("fedi.example"), "{html}");
        assert!(!html.contains("data-follow"), "{html}");
    }
}
