//! The people tab: every account with a search and a filter.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{AdminUserDto, Page, UserFilter};

use crate::{
    admin::{
        common::tr, select_field::select_field, user_detail::user_detail_sheet,
        user_row::admin_user_row,
    },
    api,
    l10n::t,
    ui::{
        button::{ButtonVariant, button},
        empty_state::empty_state,
        text_field::text_field,
    },
};

/// The most pages one list fetches; the server pages 20 at a time.
const MAX_PAGES: u32 = 50;

/// The accounts of `filter` matching `query`, the first `pages` pages of the API's list, again
/// whenever an argument or `revision` changes.
#[shard]
async fn user_list(
    cx: &Cx,
    filter: Signal<String>,
    query: Signal<String>,
    pages: Signal<u32>,
    revision: Signal<u32>,
    detail: Signal<String>,
    nonce: Signal<u32>,
) -> Result<impl View> {
    let _ = revision.get();
    let filter = UserFilter::parse(&filter.get()).unwrap_or(UserFilter::Local);
    let query = query.get();
    let wanted = pages.get().clamp(1, MAX_PAGES);
    let mut items: Vec<AdminUserDto> = Vec::new();
    let mut next: Option<String> = None;
    for _ in 0..wanted {
        let search = api::query(&[
            ("filter", Some(filter.as_str())),
            ("q", Some(query.trim())),
            ("before", next.as_deref()),
        ]);
        let page = api::get::<Page<AdminUserDto>>(cx, &format!("/admin/users{search}")).await?;
        items.extend(page.items);
        next = page.next;
        if next.is_none() {
            break;
        }
    }
    let label = tr!(cx, admin_tab_users);
    let empty_title = tr!(cx, admin_users_empty);
    let more = next.is_some();
    Ok(view! {
        if items.is_empty() {
            empty_state(context: "admin-users", title: &empty_title)
        } else {
            <div class="list" role="list" aria-label=(&label)>
                #[key(entry.user.id.as_bytes())]
                for entry in &items {
                    <div role="listitem">
                        admin_user_row(
                            entry: entry,
                            scope: "row",
                            revision: &revision,
                            detail: &detail,
                            nonce: &nonce
                        )
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
pub(crate) async fn users(cx: &Cx) -> Result<impl View> {
    let filter = signal(cx, || "local".to_owned());
    let query = signal(cx, String::new);
    let pages = signal(cx, || 1u32);
    let revision = signal(cx, || 0u32);
    let detail = signal(cx, String::new);
    let nonce = signal(cx, || 0u32);
    let search_label = tr!(cx, admin_users_search);
    let placeholder = tr!(cx, admin_users_search_placeholder);
    let show_label = tr!(cx, admin_users_show);
    let options: Vec<(String, String)> = [
        ("local", tr!(cx, admin_filter_local)),
        ("remote", tr!(cx, admin_filter_remote)),
        ("staff", tr!(cx, admin_filter_staff)),
        ("verified", tr!(cx, admin_filter_verified)),
        ("suspended", tr!(cx, admin_filter_suspended)),
        ("all", tr!(cx, admin_filter_all)),
    ]
    .into_iter()
    .map(|(value, label)| (value.to_owned(), label))
    .collect();
    Ok(view! {
        <div data-c="admin-users">
            <div class="tab">
                <form
                    class="controls"
                    role="search"
                    @submit=$(|e: Event| e.prevent_default())
                    @input=$(|_e: Event| pages.set(1u32))
                    @change=$(|_e: Event| pages.set(1u32))
                >
                    text_field(
                        label: &search_label,
                        attrs: attributes! {
                            type="search"
                            name="q"
                            placeholder=(&placeholder)
                            autocomplete="off"
                            :value=$(query.get())
                            @input=$(|e: Event| query.set(e.target.value))
                        }
                    )
                    select_field(
                        label: &show_label,
                        options: &options,
                        value: "local",
                        attrs: attributes! { @change=$(|e: Event| filter.set(e.target.value)) }
                    )
                </form>
            </div>
            user_list(
                filter: filter.clone(),
                query: query.clone(),
                pages: pages.clone(),
                revision: revision.clone(),
                detail: detail.clone(),
                nonce: nonce.clone()
            )
            user_detail_sheet(detail: &detail, nonce: &nonce, revision: &revision)
        </div>
    })
}
