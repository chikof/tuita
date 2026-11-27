//! `/settings/affiliation`: organization invitations and, for an organization, its affiliates.

use serde_json::json;
use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{AffiliationDto, AffiliationStatus, AffiliationsDto, OkDto};

use crate::{
    api, fmt,
    l10n::t,
    post::rich_text::user_name,
    settings::{
        actions::{Done, failed},
        card::setting_card,
        page::settings_page,
    },
    shell::parts::account_avatar,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        confirm_dialog::confirm_dialog,
        tag::tag,
        text_field::text_field,
    },
};

/// Answers an invitation or ends an affiliation: `accept`, `decline` or `end`. Answers the toast.
#[procedure]
pub(crate) async fn answer_affiliation(cx: &Cx, action: String, id: String) -> Done {
    let id = api::enc(&id);
    let done = match action.as_str() {
        "accept" => failed(
            api::post::<_, OkDto>(cx, &format!("/affiliations/{id}/accept"), &json!({})).await,
        )
        .map(|_| t!(aff_accepted).resolve(cx).into_owned()),
        "decline" => failed(api::delete::<OkDto>(cx, &format!("/affiliations/{id}")).await)
            .map(|_| t!(aff_declined).resolve(cx).into_owned()),
        "end" => failed(api::delete::<OkDto>(cx, &format!("/affiliations/{id}")).await)
            .map(|_| t!(aff_ended).resolve(cx).into_owned()),
        _ => Err("Unknown action".to_owned()),
    };
    Ok(done)
}

/// Invites `handle` to affiliate with the organization; answers the toast.
#[procedure]
pub(crate) async fn invite_affiliate(cx: &Cx, handle: String, label: String) -> Done {
    let label = label.trim();
    let body = json!({
        "handle": handle.trim().trim_start_matches('@'),
        "label": if label.is_empty() { None } else { Some(label) },
    });
    let done = failed(api::post::<_, OkDto>(cx, "/affiliations", &body).await);
    Ok(done.map(|_| t!(aff_invited).resolve(cx).into_owned()))
}

/// Which list a row belongs to; it decides the buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    /// The organization the account belongs to: Leave.
    Current,
    /// An invitation: Accept and Decline.
    Invite,
    /// An affiliate of the account's own organization: Remove.
    Member,
}

/// One person (or organization) with the buttons of its list.
#[component]
async fn affiliation_row(
    cx: &Cx,
    affiliation: &AffiliationDto,
    row: Row,
    rev: &Signal<u32>,
) -> Result<impl View> {
    let user = &affiliation.user;
    let id = affiliation.id.to_string();
    let handle = fmt::handle_of(&user.handle, user.domain.as_deref());
    let dim = if affiliation.label.is_empty() {
        handle.clone()
    } else {
        format!("{handle} · {}", affiliation.label)
    };
    let member = row == Row::Member;
    let (title, body, confirm_label) = if member {
        (
            t!(aff_remove_title, handle: handle.as_str())
                .resolve(cx)
                .into_owned(),
            t!(aff_remove_body).resolve(cx).into_owned(),
            t!(aff_remove).resolve(cx).into_owned(),
        )
    } else {
        (
            t!(aff_leave_title).resolve(cx).into_owned(),
            t!(aff_leave_body).resolve(cx).into_owned(),
            t!(aff_leave).resolve(cx).into_owned(),
        )
    };
    let remove_label = t!(aff_remove_label, handle: handle.as_str())
        .resolve(cx)
        .into_owned();
    let leave_id = format!("leave-{id}");
    let pending = affiliation.status == AffiliationStatus::Pending;
    Ok(view! {
        <li data-affiliation=(&id)>
            <span aria-hidden="true">
                account_avatar(
                    handle: &user.handle,
                    src: user.avatar_url.as_deref(),
                    shape: user.avatar_shape,
                    ink: user.ink,
                    size: 40
                )
            </span>
            <div class="who">
                user_name(
                    name: &user.display_name,
                    verified: user.verified,
                    bot: user.bot,
                    emojis: &user.emojis
                )
                <span class="dim">(&dim)</span>
            </div>
            if row == Row::Invite {
                button(
                    size: ButtonSize::Sm,
                    attrs: attributes! {
                        @click=$(async |_e: Event| {
                            let done = answer_affiliation("accept".to_owned(), id).await;
                            if done.is_ok() {
                                let msg = done.unwrap();
                                rev.increment();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                    msg.is_empty(),
                                )
                            } else {
                                let msg = done.unwrap_err();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                    msg.is_empty(),
                                )
                            }
                        })
                    },
                    (t!(aff_accept))
                )
                button(
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    attrs: attributes! {
                        @click=$(async |_e: Event| {
                            let done = answer_affiliation("decline".to_owned(), id).await;
                            if done.is_ok() {
                                let msg = done.unwrap();
                                rev.increment();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                    msg.is_empty(),
                                )
                            } else {
                                let msg = done.unwrap_err();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                    msg.is_empty(),
                                )
                            }
                        })
                    },
                    (t!(aff_decline))
                )
            } else {
                if member && pending {
                    tag(pill: true, compact: true, (t!(aff_pending)))
                }
                button(
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    attrs: attributes! {
                        aria-label=(member.then_some(remove_label.as_str()))
                        command="show-modal"
                        commandfor=(&leave_id)
                    },
                    (&confirm_label)
                )
                confirm_dialog(
                    id: &leave_id,
                    title: &title,
                    body: &body,
                    confirm_label: &confirm_label,
                    danger: true,
                    action: $(async |_e: Event| {
                        let done = answer_affiliation("end".to_owned(), id).await;
                        if done.is_ok() {
                            let msg = done.unwrap();
                            rev.increment();
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                msg.is_empty(),
                            )
                        } else {
                            let msg = done.unwrap_err();
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                msg.is_empty(),
                            )
                        }
                    })
                )
            }
        </li>
    })
}

/// The invitation form of an organization.
#[component]
async fn invite_form(cx: &Cx, rev: &Signal<u32>) -> Result<impl View> {
    let handle = signal(cx, String::new);
    let label = signal(cx, String::new);
    let error = signal(cx, String::new);
    let busy = signal(cx, || false);
    let handle_label = t!(aff_handle).resolve(cx).into_owned();
    let label_label = t!(aff_label).resolve(cx).into_owned();
    let label_hint = t!(aff_label_hint).resolve(cx).into_owned();
    Ok(view! {
        <form
            class="invite"
            novalidate=(true)
            @submit=$(async |e: Event| {
                e.prevent_default();
                error.set("".to_owned());
                busy.set(true);
                let done = invite_affiliate(handle.get(), label.get()).await;
                busy.set(false);
                if done.is_ok() {
                    let msg = done.unwrap();
                    handle.set("".to_owned());
                    label.set("".to_owned());
                    rev.increment();
                    raw!(
                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                        msg.is_empty(),
                    )
                } else {
                    error.set(done.unwrap_err());
                    false
                }
            })
        >
            text_field(
                label: &handle_label,
                attrs: attributes! {
                    name="handle"
                    autocapitalize="off"
                    spellcheck="false"
                    maxlength="16"
                    required=(true)
                    :value=$(handle.get())
                    @input=$(|e: Event| handle.set(e.target.value))
                }
            )
            text_field(
                label: &label_label,
                hint: label_hint.as_str(),
                attrs: attributes! {
                    name="label"
                    maxlength="30"
                    :value=$(label.get())
                    @input=$(|e: Event| label.set(e.target.value))
                }
            )
            <p class="error" role="alert" :hidden=$(error.get().is_empty())>
                $(error.get())
            </p>
            <div class="end">
                button(
                    submit: true,
                    attrs: attributes! {
                        :disabled=$(if busy.get() {
                            true
                        } else {
                            handle.get().trim().is_empty()
                        })
                    },
                    (t!(aff_invite))
                )
            </div>
        </form>
    })
}

/// The cards of the account's affiliations; reruns when `rev` moves.
#[shard]
async fn affiliation_cards(cx: &Cx, rev: Signal<u32>) -> Result<impl View> {
    let _ = rev.get();
    let data = api::get::<AffiliationsDto>(cx, "/affiliations").await?;
    let yours = t!(aff_yours_title).resolve(cx).into_owned();
    let yours_desc = t!(aff_yours_desc).resolve(cx).into_owned();
    let invites = t!(aff_invites_title).resolve(cx).into_owned();
    let org = t!(aff_org_title).resolve(cx).into_owned();
    let org_desc = t!(aff_org_desc).resolve(cx).into_owned();
    Ok(view! {
        setting_card(
            title: &yours,
            description: &yours_desc,
            if let Some(current) = &data.current {
                <ul>
                    affiliation_row(affiliation: current, row: Row::Current, rev: &rev)
                </ul>
            } else {
                <p class="dim">(t!(aff_none))</p>
            }
        )
        if !data.invites.is_empty() {
            setting_card(
                title: &invites,
                <ul>
                    #[key(invite.id.as_bytes())]
                    for invite in &data.invites {
                        affiliation_row(
                            affiliation: invite,
                            row: Row::Invite,
                            rev: &rev
                        )
                    }
                </ul>
            )
        }
        if data.org {
            setting_card(
                title: &org,
                description: &org_desc,
                invite_form(rev: &rev)
                if data.members.is_empty() {
                    <p class="dim">(t!(aff_members_none))</p>
                } else {
                    <ul>
                        #[key(member.id.as_bytes())]
                        for member in &data.members {
                            affiliation_row(
                                affiliation: member,
                                row: Row::Member,
                                rev: &rev
                            )
                        }
                    </ul>
                }
            )
        } else {
            <p class="dim note">(t!(aff_not_org))</p>
        }
    })
}

/// The affiliation page.
#[component]
pub(crate) async fn affiliation(cx: &Cx) -> Result<impl View> {
    let rev = signal(cx, || 0u32);
    let title = t!(settings_affiliation).resolve(cx).into_owned();
    Ok(view! {
        settings_page(
            title: &title,
            <div data-c="affiliation" class="cards">affiliation_cards(rev: rev)</div>
        )
    })
}
