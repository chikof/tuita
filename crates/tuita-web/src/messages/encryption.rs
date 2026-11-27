//! The encrypted-messages settings section.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::{
    l10n::t,
    messages::e2ee_prompt::{PromptKind, e2ee_prompt},
    session,
    ui::{
        button::{ButtonVariant, button},
        dialog::dialog,
    },
};

/// Whether a card is hidden: a forced `status` shows the cards listing it in `whens`; without
/// one every card starts as `initially` says and `e2ee.js` takes over.
fn hidden(status: Option<&str>, whens: &str, initially: bool) -> bool {
    status.map_or(initially, |status| {
        !whens.split(' ').any(|when| when == status)
    })
}

/// The cards of `/settings/encryption`: status, turn on or unlock, forget on this device, reset.
///
/// Every card is a wrapper `[data-when="none locked ..."]` that `e2ee.js` shows while the keys
/// have one of those states (`loading`, `error`, `none`, `locked`, `ready`). The cards use the
/// markup of a settings card (`data-c="setting-card"`).
#[component]
pub(crate) async fn encryption_section(cx: &Cx) -> Result<impl View> {
    let me = session::member(cx).await?.me.user.id.to_string();
    Ok(view! { encryption_cards(me: &me) })
}

/// [`encryption_section`] for a known account. With `status` (`none`, `locked` or `ready`) the
/// cards of that key state are shown and no island runs, for a page that demonstrates a fixed
/// state whatever this browser holds. `prefix` starts every id, so several copies can share a
/// page.
#[component]
pub(crate) async fn encryption_cards(
    cx: &Cx,
    me: &str,
    #[default]
    #[into]
    status: Option<&str>,
    #[default("e2ee")] prefix: &str,
) -> Result<impl View> {
    let reset_title = t!(e2ee_reset_title).resolve(cx).into_owned();
    let island = status.is_none().then_some("e2ee-settings");
    let status_title = format!("{prefix}-status-title");
    let setup_title = format!("{prefix}-setup-title");
    let unlock_title = format!("{prefix}-unlock-title");
    let forget_title = format!("{prefix}-forget-title");
    let reset_card_title = format!("{prefix}-reset-card-title");
    let reset_id = format!("{prefix}-reset");
    Ok(view! {
        <div data-island=(island) data-me=(me) data-c="e2ee-settings">
            <div
                data-when="loading error none locked ready"
                hidden=(hidden(status, "loading error none locked ready", false))
            >
                <section
                    class="card"
                    data-c="setting-card"
                    aria-labelledby=(&status_title)
                >
                    <h2 id=(&status_title)>(t!(e2ee_status_label))</h2>
                    <p
                        class="state"
                        role="status"
                        data-when="loading"
                        hidden=(hidden(status, "loading", false))
                    >
                        (t!(loading))
                    </p>
                    <p
                        class="state error"
                        role="alert"
                        data-when="error"
                        hidden=(hidden(status, "error", true))
                    >
                        (t!(e2ee_keys_error))
                    </p>
                    <div
                        class="end"
                        data-when="error"
                        hidden=(hidden(status, "error", true))
                    >
                        button(
                            variant: ButtonVariant::Outline,
                            attrs: attributes! { data-act="e2ee-retry" },
                            (t!(retry))
                        )
                    </div>
                    <p
                        class="state"
                        data-when="none"
                        hidden=(hidden(status, "none", true))
                    >
                        (t!(e2ee_status_none))
                    </p>
                    <p
                        class="state"
                        data-when="locked"
                        hidden=(hidden(status, "locked", true))
                    >
                        (t!(e2ee_status_locked))
                    </p>
                    <p
                        class="state"
                        data-when="ready"
                        hidden=(hidden(status, "ready", true))
                    >
                        (t!(e2ee_status_ready))
                    </p>
                </section>
            </div>
            <div data-when="none" hidden=(hidden(status, "none", true))>
                <section
                    class="card"
                    data-c="setting-card"
                    aria-labelledby=(&setup_title)
                >
                    <h2 id=(&setup_title)>(t!(e2ee_setup_title))</h2>
                    e2ee_prompt(kind: PromptKind::Setup, bare: true, toast: true)
                </section>
            </div>
            <div data-when="locked" hidden=(hidden(status, "locked", true))>
                <section
                    class="card"
                    data-c="setting-card"
                    aria-labelledby=(&unlock_title)
                >
                    <h2 id=(&unlock_title)>(t!(e2ee_unlock_title))</h2>
                    e2ee_prompt(kind: PromptKind::Unlock, bare: true, toast: true)
                </section>
            </div>
            <div data-when="ready" hidden=(hidden(status, "ready", true))>
                <section
                    class="card"
                    data-c="setting-card"
                    aria-labelledby=(&forget_title)
                >
                    <h2 id=(&forget_title)>(t!(e2ee_forget))</h2>
                    <p class="description">(t!(e2ee_forget_hint))</p>
                    <div class="end">
                        button(
                            variant: ButtonVariant::Outline,
                            attrs: attributes! { data-act="e2ee-forget" },
                            (t!(e2ee_forget))
                        )
                    </div>
                </section>
            </div>
            <div data-when="locked ready" hidden=(hidden(status, "locked ready", true))>
                <section
                    class="card"
                    data-c="setting-card"
                    aria-labelledby=(&reset_card_title)
                >
                    <h2 id=(&reset_card_title)>(t!(e2ee_reset))</h2>
                    <p class="description">(t!(e2ee_reset_hint))</p>
                    <div class="end">
                        button(
                            variant: ButtonVariant::Danger,
                            attrs: attributes! { command="show-modal" commandfor=(&reset_id) },
                            (t!(e2ee_reset))
                        )
                    </div>
                </section>
            </div>
            dialog(
                id: &reset_id,
                title: &reset_title,
                <p class="body">(t!(e2ee_reset_body))</p>
                e2ee_prompt(kind: PromptKind::Reset, bare: true, toast: true)
            )
        </div>
    })
}
