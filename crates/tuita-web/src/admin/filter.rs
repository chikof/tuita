//! The content filter tab: the word lists and the scan for accounts they flag.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{
    AdminSettingsDto, AdminUserDetailDto, ContentFilterDto, FilterField, FilterScanDto,
    PatchSettingsRequest, Role,
};
use uuid::Uuid;

use crate::{
    admin::{
        common::{message, tr},
        setting_card::setting_card,
        user_detail::user_detail_sheet,
        user_row::admin_user_row,
    },
    api,
    l10n::t,
    session,
    ui::{
        button::{ButtonVariant, button},
        text_area::text_area,
    },
};

/// `PATCH /admin/settings` with the three word lists. Answers the toast text.
#[procedure]
pub(crate) async fn save_filter(
    cx: &Cx,
    blocked: String,
    allowed: String,
    reserved: String,
) -> Result<std::result::Result<String, String>> {
    let body = PatchSettingsRequest {
        name: None,
        description: None,
        contact: None,
        rules: None,
        min_age: None,
        signups: None,
        filter: Some(ContentFilterDto {
            blocked,
            allowed,
            reserved,
        }),
    };
    Ok(
        api::patch::<_, AdminSettingsDto>(cx, "/admin/settings", &body)
            .await
            .map(|_| tr!(cx, admin_filter_saved))
            .map_err(message),
    )
}

fn field_label(cx: &Cx, field: FilterField) -> String {
    match field {
        FilterField::Handle => tr!(cx, admin_filter_field_handle),
        FilterField::Name => tr!(cx, admin_filter_field_name),
        FilterField::Location => tr!(cx, admin_filter_field_location),
        FilterField::Bio => tr!(cx, admin_filter_field_bio),
    }
}

/// One flagged account, read again when `revision` changes so its row shows what the menu just
/// did. An account that was deleted leaves the list.
#[shard]
async fn scan_row(
    cx: &Cx,
    id: String,
    note: String,
    revision: Signal<u32>,
    detail: Signal<String>,
    nonce: Signal<u32>,
) -> Result<impl View> {
    let _ = revision.get();
    let loaded = match id.parse::<Uuid>() {
        Ok(id) => {
            let path = format!("/admin/users/{}", api::enc(&format!("id:{id}")));
            Some(api::get::<AdminUserDetailDto>(cx, &path).await?.user)
        }
        Err(_) => None,
    };
    Ok(view! {
        if let Some(entry) = &loaded {
            if !entry.deleted {
                admin_user_row(
                    entry: entry,
                    scope: "scan",
                    revision: &revision,
                    detail: &detail,
                    nonce: &nonce,
                    note: Some(note.as_str())
                )
            }
        }
    })
}

/// The accounts the filter flags, from a scan that runs each time `run` is bumped (the server
/// allows a few a minute).
#[shard]
async fn scan_results(
    cx: &Cx,
    run: Signal<u32>,
    revision: Signal<u32>,
    detail: Signal<String>,
    nonce: Signal<u32>,
) -> Result<impl View> {
    let started = run.get() > 0;
    let scan = if started {
        Some(api::get::<FilterScanDto>(cx, "/admin/filter/scan").await)
    } else {
        None
    };
    let (flagged, failure) = match scan {
        Some(Ok(done)) => (Some(done.flagged), None),
        Some(Err(error)) => (None, Some(error.message)),
        None => (None, None),
    };
    let rows: Vec<(String, String)> = flagged
        .iter()
        .flatten()
        .map(|hit| {
            (
                hit.user.user.id.to_string(),
                tr!(
                    cx,
                    admin_scan_reason,
                    field: field_label(cx, hit.field),
                    reason: hit.reason.as_str()
                ),
            )
        })
        .collect();
    let clean = flagged.as_ref().is_some_and(Vec::is_empty);
    let found = flagged.as_ref().filter(|list| !list.is_empty()).map(
        |list| tr!(cx, admin_scan_found, count: i32::try_from(list.len()).unwrap_or(i32::MAX)),
    );
    Ok(view! {
        if let Some(text) = &failure {
            <p class="error" role="alert">(text)</p>
        }
        if clean {
            <p class="ok" role="status">(t!(admin_scan_clean))</p>
        }
        if let Some(text) = &found {
            <p role="status">(text)</p>
            <ul class="hits">
                #[key(id.as_bytes())]
                for (id, note) in rows {
                    <li>
                        scan_row(
                            id: id,
                            note: note,
                            revision: revision.clone(),
                            detail: detail.clone(),
                            nonce: nonce.clone()
                        )
                    </li>
                }
            </ul>
        }
    })
}

#[component]
pub(crate) async fn filter(cx: &Cx) -> Result<impl View> {
    let viewer = session::member(cx).await?;
    let admin = viewer.me.role == Role::Admin;
    let settings = if admin {
        Some(api::get::<AdminSettingsDto>(cx, "/admin/settings").await?)
    } else {
        None
    };
    let blocked = signal(cx, || {
        settings
            .as_ref()
            .map(|s| s.filter.blocked.clone())
            .unwrap_or_default()
    });
    let allowed = signal(cx, || {
        settings
            .as_ref()
            .map(|s| s.filter.allowed.clone())
            .unwrap_or_default()
    });
    let reserved = signal(cx, || {
        settings
            .as_ref()
            .map(|s| s.filter.reserved.clone())
            .unwrap_or_default()
    });
    let error = signal(cx, String::new);
    let busy = signal(cx, || false);
    let run = signal(cx, || 0u32);
    let revision = signal(cx, || 0u32);
    let detail = signal(cx, String::new);
    let nonce = signal(cx, || 0u32);
    let blocked_text = settings
        .as_ref()
        .map(|s| s.filter.blocked.clone())
        .unwrap_or_default();
    let allowed_text = settings
        .as_ref()
        .map(|s| s.filter.allowed.clone())
        .unwrap_or_default();
    let reserved_text = settings
        .as_ref()
        .map(|s| s.filter.reserved.clone())
        .unwrap_or_default();
    let title = tr!(cx, admin_filter_title);
    let desc = tr!(cx, admin_filter_desc);
    let blocked_label = tr!(cx, admin_filter_blocked);
    let blocked_hint = tr!(cx, admin_filter_blocked_hint);
    let allowed_label = tr!(cx, admin_filter_allowed);
    let allowed_hint = tr!(cx, admin_filter_allowed_hint);
    let reserved_label = tr!(cx, admin_filter_reserved);
    let reserved_hint = tr!(cx, admin_filter_reserved_hint);
    let scan_title = tr!(cx, admin_scan_title);
    let scan_desc = tr!(cx, admin_scan_desc);
    Ok(view! {
        <div class="tab" data-c="admin-filter">
            if admin {
                <form
                    class="form"
                    novalidate=""
                    @submit=$(async |e: Event| {
                        e.prevent_default();
                        error.set("".to_owned());
                        busy.set(true);
                        let done = save_filter(
                            blocked.get(),
                            allowed.get(),
                            reserved.get(),
                        ).await;
                        busy.set(false);
                        if done.is_ok() {
                            let msg = done.unwrap();
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                                msg.is_empty(),
                            )
                        } else {
                            error.set(done.unwrap_err());
                            false
                        }
                    })
                >
                    setting_card(
                        title: &title,
                        description: desc.as_str(),
                        text_area(
                            label: &blocked_label,
                            hint: blocked_hint.as_str(),
                            value: &blocked_text,
                            rows: 3,
                            attrs: attributes! {
                                name="blocked"
                                maxlength="5000"
                                :value=$(blocked.get())
                                @input=$(|e: Event| blocked.set(e.target.value))
                            }
                        )
                        text_area(
                            label: &allowed_label,
                            hint: allowed_hint.as_str(),
                            value: &allowed_text,
                            rows: 2,
                            attrs: attributes! {
                                name="allowed"
                                maxlength="5000"
                                :value=$(allowed.get())
                                @input=$(|e: Event| allowed.set(e.target.value))
                            }
                        )
                        text_area(
                            label: &reserved_label,
                            hint: reserved_hint.as_str(),
                            value: &reserved_text,
                            rows: 2,
                            attrs: attributes! {
                                name="reserved"
                                maxlength="5000"
                                :value=$(reserved.get())
                                @input=$(|e: Event| reserved.set(e.target.value))
                            }
                        )
                    )
                    <p class="error" role="alert" :hidden=$(error.get().is_empty())>
                        $(error.get())
                    </p>
                    <div class="end">
                        button(
                            submit: true,
                            attrs: attributes! {
                                :disabled=$(busy.get())
                                :aria-busy=$(if busy.get() { "true" } else { "false" })
                            },
                            (t!(admin_filter_save))
                        )
                    </div>
                </form>
            }
            setting_card(
                title: &scan_title,
                description: scan_desc.as_str(),
                <div class="end">
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { @click=$(|_e: Event| run.increment()) },
                        (t!(admin_scan_run))
                    )
                </div>
                scan_results(
                    run: run.clone(),
                    revision: revision.clone(),
                    detail: detail.clone(),
                    nonce: nonce.clone()
                )
            )
            user_detail_sheet(detail: &detail, nonce: &nonce, revision: &revision)
        </div>
    })
}
