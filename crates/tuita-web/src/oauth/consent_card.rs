//! The consent card and the procedure behind its two buttons.
//!
//! The card is rendered once with the page and never re-fetched, so its states are signals: the
//! sections are all in the markup and `phase` decides which one shows. A request the server
//! refuses is a different, static card ([`failed_card`]).

use axum::http::StatusCode;
use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, procedure, signal},
    view::{View, attributes, component, element_id, view},
};
use tuita_server::dto::{OAuthAuthorizeDto, OAuthAuthorizeRequest, OAuthConsentDto, TokenScope};

use crate::{
    api,
    icons::Icon,
    l10n::t,
    oauth::{Failure, website_link},
    ui::{
        button::{ButtonVariant, button},
        icon::icon,
        loader::{LoaderVariant, loader},
    },
};

/// The user's answer: `POST /oauth/authorize`. `ask` is the request as JSON (what the page
/// rendered), `approve` the choice.
///
/// Answers the URL to open (an app with a redirect), the code to show (an out-of-band app that was
/// approved) or nothing (an out-of-band app that was denied). The `Err` is `rate` or `unreachable`
/// (the buttons come back with that message) or `reload` (the request was refused or the session is
/// gone: the page reloads and shows the card of the new state).
#[procedure]
pub(crate) async fn answer(cx: &Cx, ask: String, approve: bool) -> Result<Result<String, String>> {
    let Ok(mut request) = serde_json::from_str::<OAuthAuthorizeRequest>(&ask) else {
        return Ok(Err("reload".to_owned()));
    };
    request.approve = approve;
    match api::post_root::<_, OAuthAuthorizeDto>(cx, "/oauth/authorize", &request).await {
        Ok(OAuthAuthorizeDto { redirect, code }) => {
            let next = redirect.or(code).unwrap_or_default();
            if approve && next.is_empty() {
                Ok(Err(Failure::Unreachable.code().to_owned()))
            } else {
                Ok(Ok(next))
            }
        }
        Err(error) => {
            let failure = Failure::of(&error);
            let word = if failure.transient() && error.status != StatusCode::UNAUTHORIZED {
                failure.code()
            } else {
                "reload"
            };
            Ok(Err(word.to_owned()))
        }
    }
}

fn scope_label(cx: &Cx, scope: TokenScope) -> String {
    match scope {
        TokenScope::Read => t!(oauth_scope_read).resolve(cx).into_owned(),
        TokenScope::Write => t!(oauth_scope_write).resolve(cx).into_owned(),
        TokenScope::Messages => t!(oauth_scope_messages).resolve(cx).into_owned(),
    }
}

fn reason(cx: &Cx, failure: Failure) -> String {
    match failure {
        Failure::InvalidRequest => t!(oauth_error_invalid_request).resolve(cx).into_owned(),
        Failure::InvalidClient => t!(oauth_error_invalid_client).resolve(cx).into_owned(),
        Failure::InvalidScope => t!(oauth_error_invalid_scope).resolve(cx).into_owned(),
        Failure::UnsupportedResponseType => t!(oauth_error_unsupported_response_type)
            .resolve(cx)
            .into_owned(),
        Failure::Rate => t!(oauth_error_rate).resolve(cx).into_owned(),
        Failure::Unreachable => t!(oauth_error_unreachable).resolve(cx).into_owned(),
    }
}

/// The card for a request the server refused: why, and that nothing was sent to the app. A
/// failure that may pass (the network, the rate limit) also offers a retry, which reloads.
#[component]
pub(crate) async fn failed_card(cx: &Cx, failure: Failure) -> Result<impl View> {
    let text = reason(cx, failure);
    let retry = failure.transient();
    Ok(view! {
        <div class="consent" data-c="consent-card" data-state="failed">
            <div class="center" role="alert">
                <h2>(t!(oauth_error_title))</h2>
                <p>(text)</p>
                <p class="dim">(t!(oauth_error_note))</p>
                <div class="actions">
                    if retry {
                        button(
                            variant: ButtonVariant::Outline,
                            attrs: attributes! { @click="() => location.reload()" },
                            (t!(retry))
                        )
                    }
                    <a class="link" href="/">(t!(oauth_home))</a>
                </div>
            </div>
        </div>
    })
}

/// Where a [`consent_card`] starts. A visitor always starts at `Ask`; the other states are what
/// answering leads to, and the design gallery draws each of them as it would look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Start {
    /// The question.
    #[default]
    Ask,
    /// An answer is on its way: both buttons are locked and the chosen one is busy.
    Sending { approve: bool },
    /// The answer failed in a way that may pass: the question is back with the reason.
    Retry(Failure),
    /// The browser is leaving for the app.
    Sent { approved: bool },
    /// An out-of-band app was approved: the code (`code`) is shown.
    Code,
    /// An out-of-band app was denied.
    Denied,
}

impl Start {
    const fn phase(self) -> &'static str {
        match self {
            Self::Ask | Self::Sending { .. } | Self::Retry(_) => "ask",
            Self::Sent { approved: true } => "sent-approved",
            Self::Sent { approved: false } => "sent-denied",
            Self::Code => "code",
            Self::Denied => "denied",
        }
    }
}

/// The card that asks: the app, what it may do, where the answer goes and the two buttons.
///
/// `start` is the `data-state` of the root at first paint (`ask` for a visitor). Answering moves
/// it to `sent-approved` or `sent-denied` for an app with a redirect (the browser is leaving), or
/// `code` or `denied` for an out-of-band app. The out-of-band code is shown in a read-only text
/// area with a copy button (`data-ok` and `data-bad` on the text area are the toast texts).
/// `code_text` is what that text area holds at first paint. `ask` is the request as JSON for
/// [`answer`]. `switch_href` is where "Use another account" goes. Every id is made from the card's
/// own, so any number of cards fit on one page.
#[component]
pub(crate) async fn consent_card(
    cx: &Cx,
    consent: &OAuthConsentDto,
    account: &str,
    ask: &str,
    switch_href: &str,
    #[default] start: Start,
    #[default]
    #[into]
    code_text: Option<&str>,
) -> Result<impl View> {
    let phase = signal(cx, || start.phase().to_owned());
    let locked = signal(cx, || matches!(start, Start::Sending { .. }));
    let denying = signal(cx, || matches!(start, Start::Sending { approve: false }));
    let approving = signal(cx, || matches!(start, Start::Sending { approve: true }));
    let failure = signal(cx, || match start {
        Start::Retry(why) => why.code().to_owned(),
        _ => String::new(),
    });
    let code = signal(cx, || code_text.unwrap_or_default().to_owned());
    let uid = element_id(cx);
    let allow_id = format!("{uid}-allow");
    let code_id = format!("{uid}-code");
    let title_id = format!("{uid}-code-title");
    let focus_id = code_id.clone();
    let copy_id = code_id.clone();
    let oob = consent.oob;
    let ask = ask.to_owned();
    let (ask_deny, ask_approve) = (ask.clone(), ask);
    let site = website_link(consent.website.as_deref());
    let scopes: Vec<(&'static str, String)> = consent
        .scopes
        .iter()
        .map(|scope| (scope.as_str(), scope_label(cx, *scope)))
        .collect();
    let host = consent.redirect_host.clone();
    let copied = t!(oauth_code_copied).resolve(cx).into_owned();
    let copy_failed = t!(oauth_code_copy_failed).resolve(cx).into_owned();
    let rate = reason(cx, Failure::Rate);
    let unreachable = reason(cx, Failure::Unreachable);
    Ok(view! {
        <div class="consent" data-c="consent-card" :data-state=$(phase.get())>
            <section class="part" :hidden=$(phase.get() != "ask")>
                <p class="app" data-app-name="">(&consent.app_name)</p>
                if let Some(website) = consent
                    .website
                    .as_deref()
                    .filter(|text| !text.is_empty()) {
                    <p class="site">
                        <span class="mono-tag">(t!(oauth_website))</span>
                        if let Some(site) = site.as_deref() {
                            <a
                                href=(site)
                                target="_blank"
                                rel="noopener noreferrer nofollow"
                            >
                                (website)
                            </a>
                        } else {
                            <span>(website)</span>
                        }
                    </p>
                }
                <p>(t!(oauth_wants, handle: account))</p>
                <p class="note" role="note">(t!(oauth_name_warning))</p>

                <p class="mono-tag" id=(&allow_id)>(t!(oauth_allow))</p>
                <ul class="scopes" aria-labelledby=(&allow_id)>
                    for (scope, label) in &scopes {
                        <li data-scope=(*scope)>
                            icon(name: Icon::Check, size: 20)
                            <span>(label)</span>
                        </li>
                    }
                </ul>
                <p class="dim small">(t!(oauth_push_note))</p>

                <p class="mono-tag">(t!(oauth_redirect))</p>
                if oob {
                    <p data-redirect="">(t!(oauth_redirect_oob))</p>
                } else {
                    <p class="target" data-redirect="">
                        <strong>(&consent.redirect_host)</strong>
                        <span class="uri">(&consent.redirect_uri)</span>
                    </p>
                }

                <p class="error" role="alert" :hidden=$(failure.get() != "rate")>
                    (&rate)
                </p>
                <p class="error" role="alert" :hidden=$(failure.get() != "unreachable")>
                    (&unreachable)
                </p>
                <div class="actions">
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! {
                            :disabled=$(locked.get())
                            :aria-busy=$(if denying.get() { "true" } else { "false" })
                            @click=$(async |_e: Event| {
                                locked.set(true);
                                denying.set(true);
                                failure.set("".to_owned());
                                let done = answer(ask_deny.clone(), false).await;
                                if done.is_ok() {
                                    let to = done.unwrap();
                                    if oob {
                                        phase.set("denied".to_owned());
                                        false
                                    } else {
                                        phase.set("sent-denied".to_owned());
                                        raw!("location.assign(${to})", to.is_empty())
                                    }
                                } else {
                                    let why = done.unwrap_err();
                                    if why != "reload" {
                                        failure.set(why);
                                        locked.set(false);
                                        denying.set(false);
                                        approving.set(false);
                                        false
                                    } else {
                                        raw!("location.reload()")
                                    }
                                }
                            })
                        },
                        (t!(oauth_deny))
                    )
                    button(
                        attrs: attributes! {
                            :disabled=$(locked.get())
                            :aria-busy=$(if approving.get() { "true" } else { "false" })
                            @click=$(async |_e: Event| {
                                locked.set(true);
                                approving.set(true);
                                failure.set("".to_owned());
                                let done = answer(ask_approve.clone(), true).await;
                                if done.is_ok() {
                                    let to = done.unwrap();
                                    if oob {
                                        code.set(to);
                                        phase.set("code".to_owned());
                                        false
                                    } else {
                                        phase.set("sent-approved".to_owned());
                                        raw!("location.assign(${to})", to.is_empty())
                                    }
                                } else {
                                    let why = done.unwrap_err();
                                    if why != "reload" {
                                        failure.set(why);
                                        locked.set(false);
                                        denying.set(false);
                                        approving.set(false);
                                        false
                                    } else {
                                        raw!("location.reload()")
                                    }
                                }
                            })
                        },
                        (t!(oauth_approve))
                    )
                </div>
                <a class="link" href=(switch_href)>(t!(oauth_switch))</a>
            </section>

            <div class="center" role="status" :hidden=$(phase.get() != "sent-approved")>
                loader(variant: LoaderVariant::Dots)
                <p>(t!(oauth_sent_approved, host: &host))</p>
            </div>
            <div class="center" role="status" :hidden=$(phase.get() != "sent-denied")>
                loader(variant: LoaderVariant::Dots)
                <p>(t!(oauth_sent_denied, host: &host))</p>
            </div>

            <section class="part" :hidden=$(phase.get() != "code")>
                <h2 id=(&title_id)>(t!(oauth_code_title))</h2>
                <p>(t!(oauth_code_body))</p>
                <textarea
                    id=(&code_id)
                    class="code"
                    aria-labelledby=(&title_id)
                    readonly=""
                    rows="3"
                    spellcheck="false"
                    autocomplete="off"
                    data-ok=(&copied)
                    data-bad=(&copy_failed)
                    :value=$(code.get())
                    @focus=$(|_e: Event| raw!(
                            "document.getElementById(${focus_id}).select()",
                            focus_id.is_empty(),
                        ))
                ></textarea>
                <div class="actions">
                    button(
                        attrs: attributes! {
                            @click=$(|_e: Event| raw!(
                                    "(() => { const f = document.getElementById(${copy_id}); navigator.clipboard.writeText(f.value).then(() => f.dataset.ok, () => f.dataset.bad).then((detail) => document.dispatchEvent(new CustomEvent('tuita-toast', { bubbles: true, detail }))); })()",
                                    copy_id.is_empty(),
                                ))
                        },
                        (t!(oauth_code_copy))
                    )
                </div>
            </section>

            <div class="center" role="status" :hidden=$(phase.get() != "denied")>
                <p>(t!(oauth_denied))</p>
                <a class="link" href="/">(t!(oauth_home))</a>
            </div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    fn consent(oob: bool) -> OAuthConsentDto {
        OAuthConsentDto {
            app_name: "Tuba".to_owned(),
            website: Some("https://client.example".to_owned()),
            scopes: vec![TokenScope::Read, TokenScope::Write],
            redirect_uri: if oob {
                "urn:ietf:wg:oauth:2.0:oob".to_owned()
            } else {
                "https://client.example/callback".to_owned()
            },
            redirect_host: if oob {
                String::new()
            } else {
                "client.example".to_owned()
            },
            oob,
            pkce: true,
        }
    }

    fn card(consent: &OAuthConsentDto) -> String {
        let consent: &'static OAuthConsentDto = Box::leak(Box::new(consent.clone()));
        render("en", |cx| {
            view! {
                consent_card(
                    consent: consent,
                    account: "@ana",
                    ask: "{}",
                    switch_href: "/login?add=1"
                )
            }
            .boxed()
        })
    }

    #[test]
    fn the_card_names_the_app_the_account_the_scopes_and_the_target() {
        let html = card(&consent(false));
        assert!(html.contains(r#"data-state="ask""#), "{html}");
        assert!(html.contains("data-app-name"));
        assert!(html.contains("Tuba"));
        assert!(html.contains("@ana"));
        assert!(html.contains(r#"data-scope="read""#));
        assert!(html.contains(r#"data-scope="write""#));
        assert!(!html.contains(r#"data-scope="messages""#));
        assert!(html.contains(r#"href="https://client.example/""#));
        assert!(html.contains(r#"rel="noopener noreferrer nofollow""#));
        assert!(html.contains("client.example"));
        assert!(html.contains("https://client.example/callback"));
        assert!(html.contains(r#"href="/login?add=1""#));
        assert!(html.contains("Push notifications aren’t supported."));
    }

    #[test]
    fn an_out_of_band_app_gets_no_target_and_a_code_field() {
        let html = card(&consent(true));
        assert!(html.contains("A code is shown here"), "{html}");
        assert!(!html.contains("urn:ietf:wg:oauth:2.0:oob"));
        assert!(html.contains("-code\""));
        assert!(html.contains("Authorization code"));
        assert!(html.contains("Copy code"));
    }

    #[test]
    fn a_website_that_is_not_a_web_address_is_text_not_a_link() {
        let mut odd = consent(false);
        odd.website = Some("javascript:alert(1)".to_owned());
        let html = card(&odd);
        assert!(!html.contains(r#"href="javascript:"#), "{html}");
        assert!(html.contains("javascript:alert(1)"));
    }

    #[test]
    fn a_refused_request_says_why_and_that_nothing_was_sent() {
        let html = render("en", |cx| {
            view! { failed_card(failure: Failure::InvalidClient) }.boxed()
        });
        assert!(html.contains(r#"data-state="failed""#), "{html}");
        assert!(html.contains(r#"role="alert""#));
        assert!(html.contains("This request can’t be authorized"));
        assert!(html.contains("This app isn’t registered here."));
        assert!(html.contains("Nothing was sent to the app."));
        assert!(!html.contains("Try again"));
    }

    #[test]
    fn only_a_failure_that_may_pass_offers_a_retry() {
        for (failure, retry) in [
            (Failure::Rate, true),
            (Failure::Unreachable, true),
            (Failure::InvalidRequest, false),
            (Failure::InvalidScope, false),
            (Failure::UnsupportedResponseType, false),
        ] {
            let html = render("en", |cx| view! { failed_card(failure: failure) }.boxed());
            assert_eq!(html.contains("Try again"), retry, "{failure:?}");
        }
    }
}
