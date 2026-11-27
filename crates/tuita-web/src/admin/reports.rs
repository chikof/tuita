//! The reports tab: the queue of open reports and the resolved ones.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{AdminReportDto, Page};

use crate::{
    admin::{common::tr, report_card::report_card},
    api,
    l10n::t,
    ui::{
        button::{ButtonVariant, button},
        empty_state::empty_state,
        segmented::segmented,
        tabs::Choice,
    },
};

/// The most pages one list fetches; the server pages 20 at a time.
const MAX_PAGES: u32 = 50;

/// The reports of `status` (`open` or `resolved`) as the first `pages` pages of the API's list,
/// again whenever the status, the page count or `revision` changes.
#[shard]
async fn report_list(
    cx: &Cx,
    status: Signal<String>,
    pages: Signal<u32>,
    revision: Signal<u32>,
) -> Result<impl View> {
    let _ = revision.get();
    let status = if status.get() == "resolved" {
        "resolved"
    } else {
        "open"
    };
    let wanted = pages.get().clamp(1, MAX_PAGES);
    let mut items: Vec<AdminReportDto> = Vec::new();
    let mut next: Option<String> = None;
    for _ in 0..wanted {
        let query = api::query(&[("status", Some(status)), ("before", next.as_deref())]);
        let page = api::get::<Page<AdminReportDto>>(cx, &format!("/admin/reports{query}")).await?;
        items.extend(page.items);
        next = page.next;
        if next.is_none() {
            break;
        }
    }
    let label = tr!(cx, admin_tab_reports);
    let empty_title = if status == "open" {
        tr!(cx, admin_reports_empty_open)
    } else {
        tr!(cx, admin_reports_empty_resolved)
    };
    let empty_body = (status == "open").then(|| tr!(cx, admin_reports_empty_open_body));
    let more = next.is_some();
    let context = format!("admin-reports-{status}");
    Ok(view! {
        if items.is_empty() {
            empty_state(
                context: &context,
                title: &empty_title,
                body: empty_body.as_deref()
            )
        } else {
            <div class="list" role="list" aria-label=(&label)>
                #[key(report.id.as_bytes())]
                for report in &items {
                    <div role="listitem">
                        report_card(report: report, revision: &revision)
                    </div>
                }
            </div>
            if more {
                <div class="more">
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! {
                            data-auto-more=""
                            @click=$(|_e: Event| pages.increment())
                        },
                        (t!(rail_show_more))
                    )
                </div>
            }
        }
    })
}

#[component]
pub(crate) async fn reports(cx: &Cx) -> Result<impl View> {
    let status = signal(cx, || "open".to_owned());
    let pages = signal(cx, || 1u32);
    let revision = signal(cx, || 0u32);
    let open = tr!(cx, admin_report_open);
    let resolved = tr!(cx, admin_report_resolved);
    let label = tr!(cx, admin_report_status);
    Ok(view! {
        <div data-c="admin-reports">
            <div class="tab" @click=$(|_e: Event| pages.set(1u32))>
                segmented(
                    items: vec![
                        Choice::new("open", &open),
                        Choice::new("resolved", &resolved),
                    ],
                    value: &status,
                    label: &label
                )
            </div>
            report_list(
                status: status.clone(),
                pages: pages.clone(),
                revision: revision.clone()
            )
        </div>
    })
}
