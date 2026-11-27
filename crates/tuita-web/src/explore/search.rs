//! The search field with its recent searches.
//!
//! The form is a plain `GET /explore?q=`. `explore.js` (`[data-island="recent-searches"]`) shows
//! the stored searches of the signed-in account under the field, moves through them with the
//! arrow keys, records a search when the form is submitted and clears them from the panel's
//! button. Storage is `localStorage` `tuita:recent-searches:<user id>`; signing out clears it
//! through `data-forget-keys`.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::{
    explore::search_tips::search_tips,
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        icon::icon,
    },
};

/// The id of the list of recent searches, which the field controls.
const LIST_ID: &str = "recent-searches-list";

/// The search form for query `q` and the account `user_id` whose recent searches it keeps.
#[component]
pub(crate) async fn explore_search(cx: &Cx, q: &str, user_id: &str) -> Result<impl View> {
    let label = t!(explore_search_label).resolve(cx).into_owned();
    let placeholder = t!(explore_search_placeholder).resolve(cx).into_owned();
    let clear_label = t!(explore_clear).resolve(cx).into_owned();
    let recent = t!(explore_recent).resolve(cx).into_owned();
    Ok(view! {
        <form
            class="search"
            role="search"
            action="/explore"
            method="get"
            data-c="explore-search"
            data-island="recent-searches"
            data-user-id=(user_id)
        >
            <div class="wrap">
                <div class="field">
                    icon(name: Icon::Search, size: 20)
                    <input
                        type="search"
                        role="combobox"
                        name="q"
                        value=(q)
                        autocomplete="off"
                        enterkeyhint="search"
                        aria-label=(&label)
                        aria-autocomplete="list"
                        aria-haspopup="listbox"
                        aria-controls=(LIST_ID)
                        aria-expanded="false"
                        placeholder=(&placeholder)
                    >
                    if !q.is_empty() {
                        <a class="clear" href="/explore" aria-label=(&clear_label)>
                            icon(name: Icon::Close, size: 20)
                        </a>
                    }
                </div>
                <div class="recent" hidden="">
                    <div class="head">
                        <h2>(&recent)</h2>
                        button(
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            attrs: attributes! { data-recent="clear" },
                            (t!(explore_recent_clear))
                        )
                    </div>
                    <ul id=(LIST_ID) role="listbox" aria-label=(&recent)></ul>
                </div>
                <template data-recent-icon="">
                    icon(name: Icon::Clock, size: 16)
                </template>
            </div>
            search_tips()
        </form>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_field_is_a_combobox_over_an_island_owned_list() {
        let html = render("en", |cx| {
            view! { explore_search(q: "rust", user_id: "u1") }.boxed()
        });
        assert!(html.contains(r#"role="search""#), "{html}");
        assert!(html.contains(r#"data-island="recent-searches""#), "{html}");
        assert!(html.contains(r#"data-user-id="u1""#), "{html}");
        assert!(html.contains(r#"role="combobox""#), "{html}");
        assert!(html.contains(r#"value="rust""#), "{html}");
        assert!(
            html.contains(r#"aria-label="Search posts, people and tags""#),
            "{html}"
        );
        assert!(
            html.contains(r#"aria-controls="recent-searches-list""#),
            "{html}"
        );
        assert!(html.contains(r#"aria-label="Clear search""#), "{html}");
        assert!(html.contains("Search operators"), "{html}");
        assert!(html.contains(r#"data-op="from:ada""#), "{html}");
    }

    #[test]
    fn an_empty_query_has_no_clear_link() {
        let html = render("en", |cx| {
            view! { explore_search(q: "", user_id: "u1") }.boxed()
        });
        assert!(!html.contains(r#"aria-label="Clear search""#), "{html}");
    }
}
