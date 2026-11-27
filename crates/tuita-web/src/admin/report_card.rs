//! One report with its decision buttons.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{AdminReportDto, OkDto, ReportAction, ResolveReportRequest};
use uuid::Uuid;

use crate::{
    admin::{
        common::{message, stamp, tr},
        labels::{reason_label, report_button, resolution_label},
        post_preview::post_preview,
        rules::report_actions,
    },
    api,
    fmt::{acct_of, handle_of, rel_label, rel_time},
    paths,
    post::rich_text::user_name,
    shell::parts::account_avatar,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        confirm_dialog::confirm_dialog,
        tag::tag,
    },
};

/// `POST /admin/reports/{id}/resolve`. Answers what staff did, in words, for the toast.
#[procedure]
pub(crate) async fn resolve_report(
    cx: &Cx,
    id: String,
    action: String,
) -> Result<std::result::Result<String, String>> {
    let (Ok(id), Some(action)) = (id.parse::<Uuid>(), ReportAction::parse(&action)) else {
        return Ok(Err(tr!(cx, error_title)));
    };
    let body = ResolveReportRequest { action };
    Ok(
        api::post::<_, OkDto>(cx, &format!("/admin/reports/{id}/resolve"), &body)
            .await
            .map(|_| resolution_label(cx, action))
            .map_err(message),
    )
}

/// A decision button with the words its confirmation needs.
struct Decision {
    name: &'static str,
    variant: ButtonVariant,
    label: String,
    body: String,
    dialog: String,
}

/// The report, its target, the note, the post, and either the buttons or how it was decided.
/// `revision` is bumped after a decision so the list around the card renders again.
#[component]
pub(crate) async fn report_card(
    cx: &Cx,
    report: &AdminReportDto,
    revision: &Signal<u32>,
) -> Result<impl View> {
    let busy = signal(cx, || false);
    let target = &report.target;
    let id = report.id.to_string();
    let unknown = tr!(cx, admin_unknown);
    let by = tr!(
        cx,
        admin_report_by,
        handle: report
            .reporter
            .as_ref()
            .map_or_else(|| unknown.clone(), |who| handle_of(&who.handle, who.domain.as_deref()))
    );
    let created = stamp(report.created_at);
    let target_handle = handle_of(&target.handle, target.domain.as_deref());
    let target_href = paths::profile(&acct_of(&target.handle, target.domain.as_deref()));
    let same_target = (report.same_target_open > 1)
        .then(|| tr!(cx, admin_report_same_target, count: report.same_target_open));
    let suspended = report
        .target_suspended
        .then(|| tr!(cx, admin_tag_suspended));
    let confirm_title = tr!(cx, admin_report_confirm_title);
    let confirm = tr!(cx, admin_report_confirm);
    let decisions: Vec<Decision> = report_actions(report)
        .into_iter()
        .map(|action| Decision {
            name: action.as_str(),
            variant: match action {
                ReportAction::Dismiss => ButtonVariant::Ghost,
                ReportAction::DeletePost => ButtonVariant::Outline,
                _ => ButtonVariant::Danger,
            },
            label: report_button(cx, action),
            body: tr!(cx, admin_report_confirm_body, action: resolution_label(cx, action)),
            dialog: format!("confirm-{id}-{}", action.as_str()),
        })
        .collect();
    let resolution = (report.status == tuita_server::dto::ReportStatus::Resolved).then(|| {
        tr!(
            cx,
            admin_report_resolution,
            action: report
                .resolution
                .map_or_else(|| tr!(cx, admin_report_resolved), |done| resolution_label(cx, done)),
            handle: report
                .resolved_by
                .as_ref()
                .map_or_else(|| unknown.clone(), |who| handle_of(&who.handle, who.domain.as_deref())),
            time: report.resolved_at.map_or_else(String::new, |at| rel_label(cx, &stamp(at)))
        )
    });
    let reason = reason_label(cx, report.reason);
    Ok(view! {
        <article class="report" data-report=(&id) data-c="admin-report-card">
            <header>
                tag(pill: true, compact: true, (&reason))
                <span class="dim">
                    (&by)
                    " · "
                    rel_time(ts: &created)
                </span>
            </header>
            <a class="target" href=(&target_href)>
                <span aria-hidden="true">
                    account_avatar(
                        handle: &target.handle,
                        src: target.avatar_url.as_deref(),
                        shape: target.avatar_shape,
                        ink: target.ink,
                        size: 32
                    )
                </span>
                user_name(
                    name: &target.display_name,
                    verified: target.verified,
                    bot: target.bot,
                    emojis: &target.emojis
                )
                <span class="dim">(&target_handle)</span>
            </a>
            <p class="tags">
                if let Some(text) = &suspended {
                    tag(pill: true, compact: true, (text))
                }
                if let Some(text) = &same_target {
                    tag(pill: true, compact: true, (text))
                }
            </p>
            if !report.note.is_empty() {
                <blockquote>(&report.note)</blockquote>
            }
            if let Some(post) = &report.post {
                post_preview(post: post)
            }
            if let Some(text) = &resolution {
                <p class="dim resolved">(text)</p>
            } else {
                <div class="actions">
                    #[key(decision.name.as_bytes())]
                    for decision in decisions {
                        let name = decision.name;
                        let report_id = id.clone();
                        if name == "dismiss" {
                            button(
                                variant: decision.variant,
                                size: ButtonSize::Sm,
                                attrs: attributes! {
                                    data-resolve=(name)
                                    :disabled=$(busy.get())
                                    @click=$(async |_e: Event| {
                                        busy.set(true);
                                        let done = resolve_report(report_id, name.to_owned()).await;
                                        busy.set(false);
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
                                },
                                (&decision.label)
                            )
                        } else {
                            button(
                                variant: decision.variant,
                                size: ButtonSize::Sm,
                                attrs: attributes! {
                                    data-resolve=(name)
                                    :disabled=$(busy.get())
                                    command="show-modal"
                                    commandfor=(&decision.dialog)
                                },
                                (&decision.label)
                            )
                            confirm_dialog(
                                id: &decision.dialog,
                                title: &confirm_title,
                                body: &decision.body,
                                confirm_label: &confirm,
                                danger: true,
                                action: $(async |_e: Event| {
                                    busy.set(true);
                                    let done = resolve_report(report_id, name.to_owned()).await;
                                    busy.set(false);
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
                        }
                    }
                </div>
            }
        </article>
    })
}
