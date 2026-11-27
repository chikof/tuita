//! The frame of the sign-in and sign-up pages.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

use crate::ui::{brand::brand, card::card, dither_field::dither_field};

/// A centred card on a dither field: the brand link, `title` (`h1`), `sub` and the content, with
/// `footer` links under it. The page's tab title is set by the page.
///
/// `[data-grain]` makes `shell.js` put the film grain on the body while the page is open.
#[component]
pub(crate) async fn auth_card(
    cx: &Cx,
    title: &str,
    #[default]
    #[into]
    sub: Option<&str>,
    #[default] footer: Option<Child<'_>>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <main data-c="auth-card" data-grain="">
            dither_field(opacity: 0.18)
            <div class="col">
                <a class="home" href="/">brand(size: 36)</a>
                card(
                    <h1>(title)</h1>
                    if let Some(sub) = sub {
                        <p class="sub">(sub)</p>
                    }
                    (child)
                )
                if let Some(footer) = footer {
                    <div class="foot">(footer)</div>
                }
            </div>
        </main>
    })
}
