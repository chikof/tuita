//! The search operators popover.
//!
//! Each operator is a `button[data-op]`; `explore.js` closes the popover and appends the token to
//! the search field.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::{icon::icon, popover::popover},
};

/// The operator tokens and the hint of each.
fn operators(cx: &Cx) -> Vec<(&'static str, String)> {
    vec![
        ("from:ada", t!(explore_op_from).resolve(cx).into_owned()),
        ("to:ada", t!(explore_op_to).resolve(cx).into_owned()),
        ("has:media", t!(explore_op_media).resolve(cx).into_owned()),
        (
            "since:2026-11-01",
            t!(explore_op_since).resolve(cx).into_owned(),
        ),
        (
            "until:2026-11-05",
            t!(explore_op_until).resolve(cx).into_owned(),
        ),
        (
            "\"exact phrase\"",
            t!(explore_op_phrase).resolve(cx).into_owned(),
        ),
        ("-word", t!(explore_op_exclude).resolve(cx).into_owned()),
    ]
}

/// The info button and the list of operators.
#[component]
pub(crate) async fn search_tips(cx: &Cx) -> Result<impl View> {
    let title = t!(explore_tips_title).resolve(cx).into_owned();
    let ops = operators(cx);
    Ok(view! {
        <div class="tips" data-c="search-tips">
            popover(
                id: "search-tips",
                label: &title,
                trigger: view! {
                    icon(name: Icon::InfoBox, size: 20)
                    <span class="sr-only">(t!(explore_tips))</span>
                }.into(

                ),
                <p class="title">(t!(explore_tips_title))</p>
                <dl>
                    for (token, hint) in ops {
                        <div class="entry">
                            <dt>
                                <button type="button" class="op" data-op=(token)>
                                    (token)
                                </button>
                            </dt>
                            <dd>(hint)</dd>
                        </div>
                    }
                </dl>
            )
        </div>
    })
}
