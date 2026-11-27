//! A labelled `<select>` whose options are owned strings (the markup of `ui::select`, which takes
//! borrowed options that cannot outlive a tab's own locals).

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, component, element_id, view},
};

use crate::{icons::Icon, ui::icon::icon};

/// `options` are `(value, label)`; `value` is the selected one. `attrs` go on the `<select>`.
#[component]
pub(crate) async fn select_field(
    cx: &Cx,
    label: &str,
    options: &[(String, String)],
    #[default] value: &str,
    #[default] attrs: Attributes,
) -> Result<impl View> {
    let id = element_id(cx);
    Ok(view! {
        <div class="field" data-c="select">
            <label for=(&id)>(label)</label>
            <div class="box">
                <select (attrs) id=(&id)>
                    for (option, text) in options {
                        <option value=(option) selected=(option == value)>
                            (text)
                        </option>
                    }
                </select>
                <span class="arrow" aria-hidden="true">
                    icon(name: Icon::ChevronDown, size: 20)
                </span>
            </div>
        </div>
    })
}
