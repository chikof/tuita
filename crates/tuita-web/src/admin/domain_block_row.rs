//! A blocked instance: its severity, why and a way to lift the block.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{DomainBlockDto, DomainBlockSeverity, OkDto, PatchDomainBlockRequest};

use crate::{
    admin::{
        common::{message, stamp, tr},
        federation::{severity_label, severity_options},
        select_field::select_field,
    },
    api,
    fmt::{compact, full_time},
    ui::{
        button::{ButtonVariant, button},
        confirm_dialog::confirm_dialog,
    },
};

/// `PATCH /admin/federation/blocks/{domain}`. Answers the toast text.
#[procedure]
pub(crate) async fn set_severity(
    cx: &Cx,
    domain: String,
    severity: String,
) -> Result<std::result::Result<String, String>> {
    let Some(severity) = DomainBlockSeverity::parse(&severity) else {
        return Ok(Err(tr!(cx, error_title)));
    };
    let body = PatchDomainBlockRequest {
        severity: Some(severity),
        reason: None,
    };
    let path = format!("/admin/federation/blocks/{}", api::enc(&domain));
    Ok(api::patch::<_, DomainBlockDto>(cx, &path, &body)
        .await
        .map(|fresh| {
            tr!(
                cx,
                admin_fed_row_changed,
                domain: fresh.domain.as_str(),
                severity: severity_label(cx, fresh.severity)
            )
        })
        .map_err(message))
}

/// `DELETE /admin/federation/blocks/{domain}`. Answers the toast text.
#[procedure]
pub(crate) async fn remove_block(
    cx: &Cx,
    domain: String,
) -> Result<std::result::Result<String, String>> {
    let path = format!("/admin/federation/blocks/{}", api::enc(&domain));
    Ok(api::delete::<OkDto>(cx, &path)
        .await
        .map(|_| tr!(cx, admin_fed_removed, domain: domain.as_str()))
        .map_err(message))
}

/// The block of `block.domain`. `revision` is bumped after a change so the list around the row
/// renders again.
#[component]
pub(crate) async fn domain_block_row(
    cx: &Cx,
    block: &DomainBlockDto,
    revision: &Signal<u32>,
) -> Result<impl View> {
    let changed_domain = block.domain.clone();
    let removed_domain = block.domain.clone();
    let dialog = format!("remove-block-{}", block.domain);
    let options: Vec<(String, String)> = severity_options(cx)
        .into_iter()
        .map(|(value, label)| (value.to_owned(), label))
        .collect();
    let reason = if block.reason.is_empty() {
        tr!(cx, admin_fed_row_no_reason)
    } else {
        block.reason.clone()
    };
    let accounts = tr!(
        cx,
        admin_fed_row_accounts,
        count: block.accounts,
        compact: compact(cx, i64::from(block.accounts))
    );
    let created = stamp(block.created_at);
    let created_text = full_time(cx, &created);
    let severity_text = tr!(cx, admin_fed_severity);
    let remove = tr!(cx, admin_fed_remove);
    let remove_label = tr!(cx, admin_fed_remove_label, domain: block.domain.as_str());
    let title = tr!(cx, admin_fed_remove_title, domain: block.domain.as_str());
    let body = tr!(cx, admin_fed_remove_body);
    Ok(view! {
        <div
            class="block"
            data-block=(&block.domain)
            data-severity=(block.severity.as_str())
            data-c="admin-domain-block-row"
        >
            <div class="text">
                <p class="domain"><code>(&block.domain)</code></p>
                <p class="dim">(&reason)</p>
                <p class="dim">
                    (&accounts)
                    " · "
                    <time datetime=(&created)>(&created_text)</time>
                </p>
            </div>
            <div class="controls">
                select_field(
                    label: &severity_text,
                    options: &options,
                    value: block.severity.as_str(),
                    attrs: attributes! {
                        name="severity"
                        @change=$(async |e: Event| {
                            let done = set_severity(changed_domain, e.target.value).await;
                            if done.is_ok() {
                                let msg = done.unwrap();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                    drop(msg),
                                );
                                revision.increment();
                                false
                            } else {
                                let msg = done.unwrap_err();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                    drop(msg),
                                );
                                revision.increment();
                                false
                            }
                        })
                    }
                )
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! {
                        aria-label=(&remove_label)
                        command="show-modal"
                        commandfor=(&dialog)
                    },
                    (&remove)
                )
            </div>
            confirm_dialog(
                id: &dialog,
                title: &title,
                body: &body,
                confirm_label: &remove,
                danger: true,
                action: $(async |_e: Event| {
                    let done = remove_block(removed_domain).await;
                    if done.is_ok() {
                        let msg = done.unwrap();
                        raw!(
                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                            drop(msg),
                        );
                        revision.increment();
                        false
                    } else {
                        let msg = done.unwrap_err();
                        raw!(
                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                            msg.is_empty(),
                        )
                    }
                })
            )
        </div>
    })
}
