//! OAuth consent: the request the app sent, the checks the backend makes on it, and the card that
//! asks the user.
//!
//! The page asks `GET /api/v1/oauth/authorization` while it renders, so a refused request is a
//! card on first paint and the app is never contacted. The answer is a procedure that calls
//! `POST /oauth/authorize`; the browser navigates only to the `redirect` that call returns.

pub(crate) mod consent_card;

use axum::http::StatusCode;
use topcoat::{Result, context::Cx, router::query_params};
use tuita_server::dto::{OAuthAuthorizeRequest, OAuthConsentDto};
use url::Url;

use crate::api::{self, ApiError};

#[query_params(error = bad_request)]
struct AskQuery {
    client_id: Option<String>,
    redirect_uri: Option<String>,
    scope: Option<String>,
    state: Option<String>,
    response_type: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

/// What the app put in the authorization URL. The server checks every field; the page only
/// forwards them. `approve` is set when the user answers.
pub(crate) type Ask = OAuthAuthorizeRequest;

/// Reads the authorization request from the page's query string. An absent `client_id` is empty,
/// which the server refuses like a missing one.
///
/// # Errors
///
/// A `400` for a malformed query string.
pub(crate) fn ask(cx: &Cx) -> Result<Ask> {
    let query = query_params::<AskQuery>(cx)?;
    Ok(Ask {
        client_id: query.client_id.clone().unwrap_or_default(),
        redirect_uri: query.redirect_uri.clone(),
        scope: query.scope.clone(),
        state: query.state.clone(),
        response_type: query.response_type.clone(),
        code_challenge: query.code_challenge.clone(),
        code_challenge_method: query.code_challenge_method.clone(),
        approve: false,
    })
}

/// Why a request cannot be answered. The OAuth codes are the server's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Failure {
    InvalidRequest,
    InvalidClient,
    InvalidScope,
    UnsupportedResponseType,
    Rate,
    Unreachable,
}

impl Failure {
    /// The server's `{error}` code of a 400, or a transport failure. A 401 is not a failure of
    /// the request: signing in again is the answer, and callers handle it before asking.
    #[must_use]
    pub(crate) fn of(error: &ApiError) -> Self {
        match error.status {
            StatusCode::TOO_MANY_REQUESTS => Self::Rate,
            StatusCode::BAD_REQUEST => match error.message.as_str() {
                "invalid_client" => Self::InvalidClient,
                "invalid_scope" => Self::InvalidScope,
                "unsupported_response_type" => Self::UnsupportedResponseType,
                _ => Self::InvalidRequest,
            },
            _ => Self::Unreachable,
        }
    }

    /// The word the answer procedure sends the browser.
    #[must_use]
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::InvalidClient => "invalid_client",
            Self::InvalidScope => "invalid_scope",
            Self::UnsupportedResponseType => "unsupported_response_type",
            Self::Rate => "rate",
            Self::Unreachable => "unreachable",
        }
    }

    /// Only the network may recover by asking again; a refused request stays refused.
    #[must_use]
    pub(crate) const fn transient(self) -> bool {
        matches!(self, Self::Rate | Self::Unreachable)
    }
}

/// Asks the server whether the request is good and what the card should show.
///
/// # Errors
///
/// The `401` of a signed-out visitor, which the error boundary turns into a sign-in redirect.
pub(crate) async fn check(
    cx: &Cx,
    ask: &Ask,
) -> Result<std::result::Result<OAuthConsentDto, Failure>> {
    let path = format!(
        "/oauth/authorization{}",
        api::query(&[
            ("client_id", Some(ask.client_id.as_str())),
            ("redirect_uri", ask.redirect_uri.as_deref()),
            ("scope", ask.scope.as_deref()),
            ("state", ask.state.as_deref()),
            ("response_type", ask.response_type.as_deref()),
            ("code_challenge", ask.code_challenge.as_deref()),
            (
                "code_challenge_method",
                ask.code_challenge_method.as_deref()
            ),
        ])
    );
    match api::get::<OAuthConsentDto>(cx, &path).await {
        Ok(consent) => Ok(Ok(consent)),
        Err(error) if error.status == StatusCode::UNAUTHORIZED => Err(error.into()),
        Err(error) => Ok(Err(Failure::of(&error))),
    }
}

/// The `http(s)` URL of an app's website, or `None`: the text is the app's own and may be anything.
#[must_use]
pub(crate) fn website_link(website: Option<&str>) -> Option<String> {
    let url = Url::parse(website.filter(|text| !text.is_empty())?).ok()?;
    matches!(url.scheme(), "https" | "http").then(|| url.to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    fn error(status: StatusCode, message: &str) -> ApiError {
        ApiError {
            status,
            message: message.to_owned(),
            retry_after: None,
        }
    }

    #[test]
    fn a_400_carries_the_servers_code() {
        for (code, failure) in [
            ("invalid_request", Failure::InvalidRequest),
            ("invalid_client", Failure::InvalidClient),
            ("invalid_scope", Failure::InvalidScope),
            (
                "unsupported_response_type",
                Failure::UnsupportedResponseType,
            ),
        ] {
            assert_eq!(Failure::of(&error(StatusCode::BAD_REQUEST, code)), failure);
        }
        assert_eq!(
            Failure::of(&error(StatusCode::BAD_REQUEST, "something else")),
            Failure::InvalidRequest
        );
    }

    #[test]
    fn rate_limits_and_transport_failures_may_pass() {
        assert_eq!(
            Failure::of(&error(StatusCode::TOO_MANY_REQUESTS, "slow down")),
            Failure::Rate
        );
        assert_eq!(
            Failure::of(&error(StatusCode::INTERNAL_SERVER_ERROR, "boom")),
            Failure::Unreachable
        );
        assert!(Failure::Rate.transient() && Failure::Unreachable.transient());
        assert!(!Failure::InvalidClient.transient());
    }

    #[test]
    fn only_a_web_address_becomes_a_link() {
        assert_eq!(
            website_link(Some("https://client.example")).as_deref(),
            Some("https://client.example/")
        );
        assert_eq!(
            website_link(Some("http://client.example/a?b=1")).as_deref(),
            Some("http://client.example/a?b=1")
        );
        for text in [
            "javascript:alert(1)",
            "client.example",
            "ftp://x.example",
            "",
        ] {
            assert_eq!(website_link(Some(text)), None, "{text}");
        }
        assert_eq!(website_link(None), None);
    }
}
