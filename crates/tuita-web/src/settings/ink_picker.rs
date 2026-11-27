//! The ink radios.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal},
    view::{View, component, element_id, view},
};

use crate::l10n::t;

/// The five inks and their localized names.
pub(crate) fn inks(cx: &Cx) -> [(&'static str, String); 5] {
    [
        ("bone", t!(settings_ink_bone).resolve(cx).into_owned()),
        (
            "phosphor",
            t!(settings_ink_phosphor).resolve(cx).into_owned(),
        ),
        ("amber", t!(settings_ink_amber).resolve(cx).into_owned()),
        ("cyan", t!(settings_ink_cyan).resolve(cx).into_owned()),
        ("rose", t!(settings_ink_rose).resolve(cx).into_owned()),
    ]
}

/// One radio per ink. The chosen ink is the `value` signal.
#[component]
pub(crate) async fn ink_picker(cx: &Cx, value: &Signal<String>, label: &str) -> Result<impl View> {
    let group = element_id(cx);
    let options = inks(cx);
    Ok(view! {
        <fieldset data-c="ink-picker">
            <legend class="mono-tag">(label)</legend>
            <div class="options">
                for (id, name) in options {
                    <label class="option" data-ink=(id)>
                        <input
                            type="radio"
                            name=(&group)
                            value=(id)
                            :checked=$(value.get() == id)
                            @change=$(|_e: Event| value.set(id.to_owned()))
                        >
                        <span class="swatch" aria-hidden="true"></span>
                        <span class="name">(name)</span>
                    </label>
                }
            </div>
        </fieldset>
    })
}

/// The same radios for a device preference: `prefs.js` reflects and stores the choice
/// (`[data-pref="ink"]`), so no signal is involved.
#[component]
pub(crate) async fn pref_ink_picker(cx: &Cx, label: &str) -> Result<impl View> {
    let group = element_id(cx);
    let options = inks(cx);
    Ok(view! {
        <fieldset data-c="ink-picker">
            <legend class="mono-tag">(label)</legend>
            <div class="options">
                for (id, name) in options {
                    <label class="option" data-ink=(id)>
                        <input type="radio" name=(&group) value=(id) data-pref="ink">
                        <span class="swatch" aria-hidden="true"></span>
                        <span class="name">(name)</span>
                    </label>
                }
            </div>
        </fieldset>
    })
}
