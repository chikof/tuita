//! Sensitive media behind a veil.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, attributes, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonSize, button},
        dither_fill::dither_fill,
        icon_button::{IconButtonVariant, icon_button},
    },
};

/// Blurs `child` behind a veil with a "show" button.
///
/// Markup starts hidden: `content` is `inert` and `aria-hidden`, the veil shows and the hide
/// button is `hidden`. `post.js` handles `data-act="sensitive-show"` and `"sensitive-hide"` by
/// flipping those and the `shown` class of the root.
#[component]
pub(crate) async fn sensitive_cover(
    cx: &Cx,
    #[default] label: Option<&str>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let note = label.map_or_else(
        || t!(sensitive_label).resolve(cx).into_owned(),
        str::to_owned,
    );
    let hide = t!(sensitive_hide).resolve(cx).into_owned();
    Ok(view! {
        <div class="cover" data-c="sensitive-cover">
            <div class="content" inert="" aria-hidden="true">(child)</div>
            <div class="veil" role="group" aria-label=(&note)>
                dither_fill(level: 0.5)
                <div class="prompt">
                    <span class="note">(&note)</span>
                    button(
                        size: ButtonSize::Sm,
                        attrs: attributes! { data-act="sensitive-show" },
                        (t!(sensitive_show))
                    )
                </div>
            </div>
            <span class="hide" hidden="">
                icon_button(
                    glyph: Icon::EyeOff,
                    label: &hide,
                    variant: IconButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    attrs: attributes! { data-act="sensitive-hide" }
                )
            </span>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[component]
    async fn demo(cx: &Cx) -> Result<impl View> {
        Ok(view! { sensitive_cover(<p>"photo"</p>) })
    }

    #[test]
    fn the_content_is_inert_behind_a_labelled_veil() {
        let html = render("en", |cx| view! { demo() }.boxed());
        assert!(html.contains(r#"class="cover""#), "{html}");
        assert!(html.contains("inert"), "{html}");
        assert!(
            html.contains(r#"role="group""#) && html.contains(r#"aria-label="Sensitive media""#),
            "{html}"
        );
        assert!(html.contains("data-act=\"sensitive-show\""), "{html}");
        assert!(
            html.contains(r#"aria-label="Hide sensitive media""#),
            "{html}"
        );
    }
}
