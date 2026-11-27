//! What a profile page says when an account on another instance cannot be shown.
//!
//! The server waits for the lookup before it answers, so a "looking up…" state is the
//! browser's own pending navigation.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::{
    l10n::t,
    profile::load_user::LookupFailure,
    shell::page_bar::page_bar,
    ui::{
        button::{ButtonVariant, button},
        empty_state::empty_state,
    },
};

/// The reason an account could not be looked up: unknown there (`404`), or the instance did not
/// answer, with a retry that asks again.
#[component]
pub(crate) async fn remote_lookup(
    cx: &Cx,
    handle: &str,
    domain: &str,
    failure: &LookupFailure,
) -> Result<impl View> {
    let missing = failure.status == 404;
    let context = format!(
        "lookup-{}:{handle}",
        if missing { "missing" } else { "failed" }
    );
    let title = if missing {
        t!(remote_lookup_notfound_title).resolve(cx).into_owned()
    } else {
        t!(remote_lookup_failed_title, handle: handle)
            .resolve(cx)
            .into_owned()
    };
    let body = if missing {
        t!(remote_lookup_notfound_body, handle: handle, domain: domain)
            .resolve(cx)
            .into_owned()
    } else {
        failure.message.clone()
    };
    Ok(view! {
        if missing {
            empty_state(context: &context, title: &title, body: body.as_str())
        } else {
            <div role="alert">
                empty_state(
                    context: &context,
                    title: &title,
                    body: body.as_str(),
                    action: Some(
                        view! {
                            button(
                                variant: ButtonVariant::Outline,
                                attrs: attributes! { @click="() => location.reload()" },
                                (t!(retry))
                            )
                        }.into(

                        ),
                    )
                )
            </div>
        }
    })
}

/// The whole page of an account that could not be looked up: a bar named after the handle, then
/// [`remote_lookup`].
#[component]
pub(crate) async fn remote_page(
    cx: &Cx,
    handle: &str,
    failure: &LookupFailure,
) -> Result<impl View> {
    let domain = handle.rsplit_once('@').map_or("", |(_, domain)| domain);
    Ok(view! {
        page_bar(back: true, title: handle)
        remote_lookup(handle: handle, domain: domain, failure: failure)
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    fn show(status: u16, message: &str) -> String {
        let failure = LookupFailure {
            status,
            message: message.to_owned(),
        };
        render("en", |cx| {
            view! {
                remote_lookup(
                    handle: "@mara@fedi.example",
                    domain: "fedi.example",
                    failure: &failure
                )
            }
            .boxed()
        })
    }

    #[test]
    fn an_unknown_account_says_so_without_a_retry() {
        let html = show(404, "not found");
        assert!(html.contains("fedi.example"), "{html}");
        assert!(!html.contains("Try again"), "{html}");
        assert!(!html.contains(r#"role="alert""#), "{html}");
    }

    #[test]
    fn an_unreachable_instance_gives_its_reason_and_a_retry() {
        let html = show(502, "Couldn’t reach fedi.example right now.");
        assert!(html.contains(r#"role="alert""#), "{html}");
        assert!(
            html.contains("Couldn’t reach fedi.example right now."),
            "{html}"
        );
        assert!(html.contains("Try again"), "{html}");
    }
}
