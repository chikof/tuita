//! The language select of the landing page.

use topcoat::{
    Result,
    context::Cx,
    runtime::Event,
    view::{View, attributes, component, view},
};

use crate::{
    auth::flows::choose_language,
    l10n::t,
    locale::{self, LANGUAGES},
    ui::select::{SelectOption, select},
};

/// A labelled select of the twelve languages, each named in itself. Choosing one saves it (on the
/// account when signed in, and in the `tuita_lang` cookie) and reloads the page in that language.
#[component]
pub(crate) async fn language_picker(
    cx: &Cx,
    #[default]
    #[into]
    hint: Option<&str>,
) -> Result<impl View> {
    let current = locale::resolve(cx).await?;
    let label = t!(language_label).resolve(cx).into_owned();
    let options: Vec<_> = LANGUAGES
        .iter()
        .map(|language| SelectOption::new(language.code, language.name))
        .collect();
    Ok(view! {
        select(
            label: &label,
            options: options,
            value: current,
            hint: hint,
            attrs: attributes! {
                @change=$(async |e: Event| {
                    let done = choose_language(e.target.value).await;
                    if done.is_ok() {
                        raw!("location.reload()")
                    } else {
                        let msg = done.unwrap_err();
                        raw!(
                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                            msg.is_empty(),
                        )
                    }
                })
            }
        )
    })
}
