//! Loading the account of an `@handle` page.

use tuita_server::dto::UserDto;

use crate::api::ApiError;

/// Why looking up an account on another instance failed; `message` is the server's own sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LookupFailure {
    pub(crate) status: u16,
    pub(crate) message: String,
}

/// What the account of an `@handle` page came to.
#[derive(Clone, Debug)]
pub(crate) enum Profile {
    /// The account, ready to show.
    Found(Box<UserDto>),
    /// The account is on another instance and the lookup failed. A local account that fails
    /// is an error of the page instead (a missing one is the not-found page).
    Failed(LookupFailure),
}

/// `ada@remote.example` names an account on another instance.
pub(crate) fn is_remote(acct: &str) -> bool {
    acct.contains('@')
}

/// Settles the answer of `GET /users/<acct>`: a remote account that could not be looked up
/// becomes a [`Profile::Failed`] the page explains; everything else passes through, so a missing
/// or forbidden local account reaches the error page.
pub(crate) fn settle(acct: &str, result: Result<UserDto, ApiError>) -> Result<Profile, ApiError> {
    match result {
        Ok(user) => Ok(Profile::Found(Box::new(user))),
        Err(error) if is_remote(acct) && error.status != axum::http::StatusCode::UNAUTHORIZED => {
            Ok(Profile::Failed(LookupFailure {
                status: error.status.as_u16(),
                message: error.message,
            }))
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;

    use super::*;
    use crate::profile::testing;

    fn error(status: StatusCode, message: &str) -> ApiError {
        ApiError {
            status,
            message: message.to_owned(),
            retry_after: None,
        }
    }

    #[test]
    fn a_handle_with_a_second_at_is_on_another_instance() {
        assert!(!is_remote("ada"));
        assert!(is_remote("ada@remote.example"));
        assert!(is_remote("ada@remote.example:8443"));
    }

    #[test]
    fn a_found_account_is_shown() {
        let ada = testing::user("ada");
        let settled = settle("ada", Ok(ada.clone()));
        assert!(matches!(settled, Ok(Profile::Found(user)) if *user == ada));
    }

    #[test]
    fn a_failed_remote_lookup_keeps_the_status_and_the_server_sentence() {
        let sentence = "Couldn’t reach fedi.example right now.";
        let settled = settle(
            "mara@fedi.example",
            Err(error(StatusCode::BAD_GATEWAY, sentence)),
        );
        assert!(matches!(
            settled,
            Ok(Profile::Failed(LookupFailure { status: 502, ref message })) if message == sentence
        ));
    }

    #[test]
    fn a_missing_local_account_is_an_error_of_the_page() {
        let settled = settle("ghost", Err(error(StatusCode::NOT_FOUND, "no such user")));
        assert!(matches!(settled, Err(e) if e.status == StatusCode::NOT_FOUND));
    }

    #[test]
    fn a_signed_out_request_still_goes_to_the_login_page() {
        let settled = settle(
            "mara@fedi.example",
            Err(error(StatusCode::UNAUTHORIZED, "sign in")),
        );
        assert!(matches!(settled, Err(e) if e.status == StatusCode::UNAUTHORIZED));
    }
}
