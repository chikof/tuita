//! The follow button.
//!
//! The button appears in lists and fragments Topcoat never hydrates (people rows, the hover card,
//! the rail), so it carries no signals: the server draws the state and `follow.js` makes it work.
//! Hooks: `[data-follow]` with `data-acct`, `data-remote`, `data-handle`, `data-state`
//! (`none`, `pending`, `following`). A click flips the state at once, calls
//! `POST|DELETE /users/<acct>/follow` and settles every button of the same account.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::{Relation, UserDto};
use uuid::Uuid;

use crate::{
    fmt::{acct_of, handle_of},
    l10n::{strip_isolates, t},
    session,
    ui::button::{ButtonSize, ButtonVariant, button},
};

/// What a follow button shows: no follow, a request the remote account has not answered, or a
/// follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FollowView {
    None,
    Pending,
    Following,
}

impl FollowView {
    /// The state the relation describes.
    pub(crate) const fn of(relation: Relation) -> Self {
        if relation.following {
            Self::Following
        } else if relation.follow_pending {
            Self::Pending
        } else {
            Self::None
        }
    }

    /// The value of `data-state`.
    const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Pending => "pending",
            Self::Following => "following",
        }
    }
}

/// Whether the viewer may follow `user`: someone else, and neither blocked the other.
pub(crate) fn may_follow(viewer: Option<Uuid>, user: &UserDto) -> bool {
    viewer.is_some_and(|id| id != user.id) && !user.relation.blocked && !user.relation.blocks_you
}

/// The follow button of `user` for the signed-in viewer, nothing for a guest, the viewer's own
/// account or a blocked pair.
#[component]
pub(crate) async fn follow_button(
    cx: &Cx,
    user: &UserDto,
    #[default] size: ButtonSize,
) -> Result<impl View> {
    let viewer = session::viewer(cx).await?.as_ref().map(|v| v.me.user.id);
    Ok(view! { follow_button_view(user: user, viewer: viewer, size: size) })
}

/// [`follow_button`] for a known viewer id.
#[component]
pub(crate) async fn follow_button_view(
    cx: &Cx,
    user: &UserDto,
    viewer: Option<Uuid>,
    #[default] size: ButtonSize,
) -> Result<impl View> {
    let show = may_follow(viewer, user);
    let state = FollowView::of(user.relation);
    let acct = acct_of(&user.handle, user.domain.as_deref());
    let handle = handle_of(&user.handle, user.domain.as_deref());
    let label = match state {
        FollowView::Following => t!(following).resolve(cx).into_owned(),
        FollowView::Pending => t!(follow_requested).resolve(cx).into_owned(),
        FollowView::None => t!(follow).resolve(cx).into_owned(),
    };
    let cancel =
        (state == FollowView::Pending).then(|| t!(follow_cancel_request).resolve(cx).into_owned());
    let variant = if state == FollowView::None {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };
    let remote = user.domain.is_some();
    Ok(view! {
        if show {
            button(
                variant: variant,
                size: size,
                attrs: attributes! {
                    data-follow=""
                    data-acct=(&acct)
                    data-handle=(strip_isolates(&handle))
                    data-remote=(remote.to_string())
                    data-state=(state.name())
                    aria-pressed=((state != FollowView::None).to_string())
                    title=(cancel)
                },
                (label)
            )
        }
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{profile::testing, testing::render};

    fn show(user: UserDto, viewer: Option<Uuid>, size: ButtonSize) -> String {
        render("en", |cx| {
            view! { follow_button_view(user: &user, viewer: viewer, size: size) }.boxed()
        })
    }

    #[test]
    fn it_draws_the_state_the_relation_describes() {
        let mut ada = testing::user("ada");
        let me = Some(Uuid::from_u128(1));
        let none = show(ada.clone(), me, ButtonSize::Sm);
        assert!(none.contains(r#"data-state="none""#), "{none}");
        assert!(none.contains(r#"aria-pressed="false""#), "{none}");
        assert!(none.contains(r#"class="btn solid sm wash""#), "{none}");
        assert!(none.contains(">Follow<"), "{none}");
        assert!(none.contains(r#"data-remote="false""#), "{none}");

        ada.relation.following = true;
        let following = show(ada, me, ButtonSize::Md);
        assert!(
            following.contains(r#"data-state="following""#),
            "{following}"
        );
        assert!(following.contains(r#"aria-pressed="true""#), "{following}");
        assert!(
            following.contains(r#"class="btn outline md wash""#),
            "{following}"
        );
        assert!(following.contains(">Following<"), "{following}");
    }

    #[test]
    fn a_pending_request_can_be_cancelled_from_the_button() {
        let mut mara = testing::remote("mara", "fedi.example");
        mara.relation.follow_pending = true;
        let html = show(mara, Some(Uuid::from_u128(1)), ButtonSize::Sm);
        assert!(html.contains(r#"data-state="pending""#), "{html}");
        assert!(html.contains(r#"data-remote="true""#), "{html}");
        assert!(html.contains(r#"data-acct="mara@fedi.example""#), "{html}");
        assert!(html.contains(r#"title="Cancel follow request""#), "{html}");
        assert!(html.contains(">Requested<"), "{html}");
    }

    #[test]
    fn nobody_follows_themselves_guests_or_blocked_accounts() {
        let mut ada = testing::user("ada");
        let someone = Some(Uuid::from_u128(1));
        assert!(!show(ada.clone(), None, ButtonSize::Sm).contains("data-follow"));
        assert!(!show(ada.clone(), Some(ada.id), ButtonSize::Sm).contains("data-follow"));
        ada.relation.blocked = true;
        assert!(!show(ada.clone(), someone, ButtonSize::Sm).contains("data-follow"));
        ada.relation.blocked = false;
        ada.relation.blocks_you = true;
        assert!(!show(ada, someone, ButtonSize::Sm).contains("data-follow"));
    }
}
