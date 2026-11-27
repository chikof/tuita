//! The profile card that opens on a handle.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::UserDto;

use crate::{
    fmt::{acct_of, compact, handle_of},
    l10n::{strip_isolates, t},
    paths,
    post::rich_text::{rich_text, segments_of, user_name},
    profile::follow_button::follow_button,
    ui::{
        avatar::{AvatarShape, avatar},
        dither_field::{FieldKind, dither_field},
    },
};

/// The card of `user`: banner, picture, name, a "follows you" tag, bio and counts.
///
/// It is one `popover="manual"` element; `hovercard.js` fetches it from `/_tuita/hovercard/<acct>`,
/// shows it next to the handle that was hovered and removes it when it closes.
#[component]
pub(crate) async fn user_hover_card(cx: &Cx, user: &UserDto) -> Result<impl View> {
    let acct = acct_of(&user.handle, user.domain.as_deref());
    let profile = paths::profile(&acct);
    let label = strip_isolates(&t!(hover_card_label, name: user.display_name.as_str()).resolve(cx));
    let shape = AvatarShape::from_name(user.avatar_shape.as_str()).unwrap_or_default();
    let segments = segments_of(&user.bio);
    let following = compact(cx, i64::from(user.counts.following));
    let followers = compact(cx, i64::from(user.counts.followers));
    let followers_label =
        strip_isolates(&t!(profile_followers, count: user.counts.followers).resolve(cx));
    Ok(view! {
        <div
            class="card"
            popover="manual"
            role="dialog"
            tabindex="-1"
            aria-label=(&label)
            data-c="user-hover-card"
        >
            <div class="banner" data-ink=(user.ink.as_str())>
                if let Some(banner) = &user.banner_url {
                    <img src=(banner) alt="">
                } else {
                    dither_field(
                        field: FieldKind::Seeded,
                        seed: user.handle.as_str(),
                        cell: 6,
                        opacity: 0.55,
                        animate: false
                    )
                }
            </div>
            <div class="body">
                <div class="top">
                    <a class="ring" href=(&profile) tabindex="-1" aria-hidden="true">
                        avatar(
                            seed: user.handle.as_str(),
                            src: user.avatar_url.as_deref(),
                            alt: "",
                            shape: shape,
                            ink: user.ink.as_str(),
                            size: 64
                        )
                    </a>
                    follow_button(user: user)
                </div>
                <a class="who" href=(&profile)>
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
                if user.relation.follows_you {
                    <span class="tag">(t!(follows_you))</span>
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
                <p class="counts">
                    <a href=(paths::following(&acct))>
                        <b>(following)</b>
                        " "
                        (t!(profile_following_label))
                    </a>
                    <a href=(paths::followers(&acct))>
                        <b>(followers)</b>
                        " "
                        (followers_label)
                    </a>
                </p>
            </div>
        </div>
    })
}
