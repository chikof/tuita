//! Landing page, sign-in and sign-up.

pub(crate) mod auth_card;
pub(crate) mod flows;
pub(crate) mod landing;
pub(crate) mod language_picker;
pub(crate) mod login;
pub(crate) mod signup;

use topcoat::{Result, context::Cx, router::query_params};
use tuita_server::dto::{InstanceDto, SignupsMode};

use crate::next::safe_next;

/// How a visitor may join.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SignupState {
    /// Anyone (or no admin exists yet).
    Open,
    /// A code is required.
    Invite,
    /// Nobody.
    Closed,
}

/// The sign-up state of an instance.
#[must_use]
pub(crate) fn signup_state(instance: &InstanceDto) -> SignupState {
    if instance.registrations_open {
        SignupState::Open
    } else if instance.signups == SignupsMode::Invite {
        SignupState::Invite
    } else {
        SignupState::Closed
    }
}

#[query_params(error = bad_request)]
struct AuthQuery {
    add: Option<String>,
    next: Option<String>,
    invite: Option<String>,
}

/// What the sign-in and sign-up pages read from their URL.
pub(crate) struct AuthAsk {
    /// `?add`: keep the accounts already signed in.
    pub(crate) add: bool,
    /// `?next=`: where the visitor was going; only a path of this site counts.
    pub(crate) next: Option<String>,
    /// `?invite=CODE` links prefill the invite field.
    pub(crate) invite: String,
}

/// Reads `?add`, `?next` and `?invite`.
///
/// # Errors
///
/// A `400` for a malformed query string.
pub(crate) fn ask(cx: &Cx) -> Result<AuthAsk> {
    let query = query_params::<AuthQuery>(cx)?;
    Ok(AuthAsk {
        add: query.add.is_some(),
        next: safe_next(query.next.as_deref()),
        invite: query.invite.clone().unwrap_or_default(),
    })
}

#[cfg(test)]
mod test {
    use tuita_server::dto::InstanceStats;

    use super::*;

    fn instance(registrations_open: bool, signups: SignupsMode) -> InstanceDto {
        InstanceDto {
            name: String::new(),
            description: String::new(),
            domain: String::new(),
            version: String::new(),
            signups,
            registrations_open,
            contact: String::new(),
            rules: Vec::new(),
            min_age: 13,
            source_url: None,
            max_video_mb: 0,
            detected_language: "en".to_owned(),
            stats: InstanceStats { users: 0, posts: 0 },
        }
    }

    #[test]
    fn open_registrations_win_over_the_mode() {
        assert_eq!(
            signup_state(&instance(true, SignupsMode::Closed)),
            SignupState::Open
        );
    }

    #[test]
    fn closed_registrations_are_invite_only_or_closed() {
        assert_eq!(
            signup_state(&instance(false, SignupsMode::Invite)),
            SignupState::Invite
        );
        assert_eq!(
            signup_state(&instance(false, SignupsMode::Closed)),
            SignupState::Closed
        );
        assert_eq!(
            signup_state(&instance(false, SignupsMode::Open)),
            SignupState::Closed
        );
    }
}
