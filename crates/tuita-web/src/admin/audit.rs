//! The audit log tab.
//!
//! Plain markup, no signals: the next page is the `a[data-more]` link `infinite.js` follows as a
//! fragment (`?before=<cursor>` with `x-tuita-fragment: 1`).

use topcoat::{
    Result,
    context::Cx,
    router::query_params,
    view::{View, component, view},
};
use tuita_server::dto::{AuditEntryDto, Page};

use crate::{
    admin::{
        common::{stamp, tr},
        labels::audit_label,
    },
    api,
    fmt::rel_time,
    fragment,
    icons::Icon,
    l10n::t,
    ui::{empty_state::empty_state, icon::icon},
};

#[query_params(error = bad_request)]
struct AuditQuery {
    before: Option<String>,
}

/// The icon for an action family: `user.suspend` is a user action.
fn icon_of(action: &str) -> Icon {
    match action.split('.').next().unwrap_or_default() {
        "user" => Icon::User,
        "post" => Icon::Message,
        "report" => Icon::Flag,
        "invite" => Icon::Link,
        "announcement" => Icon::Bell,
        _ => Icon::Gear,
    }
}

/// One row of the log.
struct Entry {
    action: String,
    icon: Icon,
    what: String,
    target: String,
    who: String,
    at: String,
    detail: String,
}

/// `GET /admin/audit`: a page of the log, as a list followed by the link to the next page.
#[component]
pub(crate) async fn audit(cx: &Cx) -> Result<impl View> {
    let before = query_params::<AuditQuery>(cx)?.before.clone();
    let path = api::query(&[("before", before.as_deref())]);
    let page = api::get::<Page<AuditEntryDto>>(cx, &format!("/admin/audit{path}")).await?;
    let system = tr!(cx, admin_audit_system);
    let entries: Vec<Entry> = page
        .items
        .iter()
        .map(|entry| Entry {
            action: entry.action.clone(),
            icon: icon_of(&entry.action),
            what: audit_label(cx, &entry.action),
            target: entry.target.clone(),
            who: entry
                .actor
                .as_ref()
                .map_or_else(|| system.clone(), |actor| format!("@{actor}")),
            at: stamp(entry.created_at),
            detail: entry.detail.clone(),
        })
        .collect();
    let first = before.is_none();
    let empty = entries.is_empty() && first;
    let label = tr!(cx, admin_tab_audit);
    let empty_title = tr!(cx, admin_audit_empty);
    let more = page
        .next
        .map(|cursor| format!("/admin/audit{}", api::query(&[("before", Some(&cursor))])));
    let fragment = fragment::requested(cx);
    Ok(view! {
        if empty {
            <div class="audit" data-c="admin-audit">
                empty_state(context: "admin-audit", title: &empty_title)
            </div>
        } else {
            if fragment {
                audit_page(entries: entries, label: &label, more: more.as_deref())
            } else {
                <div class="audit" data-c="admin-audit">
                    audit_page(entries: entries, label: &label, more: more.as_deref())
                </div>
            }
        }
    })
}

#[component]
async fn audit_page(
    cx: &Cx,
    entries: Vec<Entry>,
    label: &str,
    #[default] more: Option<&str>,
) -> Result<impl View> {
    Ok(view! {
        <div class="list" role="list" aria-label=(label)>
            for entry in entries {
                <div role="listitem">
                    <div class="entry" data-action=(&entry.action)>
                        <span class="icon">icon(name: entry.icon, size: 20)</span>
                        <div class="text">
                            <p class="head" title=(&entry.action)>
                                <span class="what">(&entry.what)</span>
                                " "
                                (&entry.target)
                            </p>
                            <p class="dim">
                                (&entry.who)
                                " · "
                                rel_time(ts: &entry.at)
                                if !entry.detail.is_empty() {
                                    (format!(" · {}", entry.detail))
                                }
                            </p>
                        </div>
                    </div>
                </div>
            }
        </div>
        if let Some(more) = more {
            <a data-more="" href=(more)>(t!(loading))</a>
        }
    })
}
