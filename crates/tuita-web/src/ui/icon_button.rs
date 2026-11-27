//! A square button that holds one icon.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, class, component, view},
};

use crate::{
    icons::Icon,
    ui::{button::ButtonSize, icon::icon},
};

/// The look of an [`icon_button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum IconButtonVariant {
    Outline,
    #[default]
    Ghost,
}

impl IconButtonVariant {
    const fn class(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Ghost => "ghost",
        }
    }
}

/// The icon size in px that goes with a button size.
const fn icon_size(size: ButtonSize) -> u32 {
    match size {
        ButtonSize::Sm => 16,
        ButtonSize::Md => 24,
        ButtonSize::Lg => 32,
    }
}

/// An icon-only button named by `label` (`aria-label`). `pressed` renders `aria-pressed`; for a
/// state the browser changes, leave it `None` and bind it through `attrs`:
/// `attributes! { :aria-pressed=$(if on.get() { "true" } else { "false" }) }`.
///
/// `attrs` go on the `<button>` (`command`/`commandfor`, `@click=$(..)`, `id`, ...).
#[component]
pub(crate) async fn icon_button(
    cx: &Cx,
    glyph: Icon,
    label: &str,
    #[default]
    #[into]
    pressed: Option<bool>,
    #[default] variant: IconButtonVariant,
    #[default] size: ButtonSize,
    #[default] mut attrs: Attributes,
) -> Result<impl View> {
    let pressed = pressed.map(|on| if on { "true" } else { "false" });
    Ok(view! {
        <button
            type="button"
            class=(class!(
                "icon-button",
                variant.class(),
                size.class(),
                "wash",
                attrs.remove("class"),
            ))
            aria-label=(label)
            aria-pressed=(pressed)
            data-c="icon-button"
            (attrs)
        >
            icon(name: glyph, size: icon_size(size))
        </button>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_is_a_button_named_by_its_label_holding_a_sized_icon() {
        let html = render("en", |cx| {
            view! {
                icon_button(glyph: Icon::Close, label: "Close", size: ButtonSize::Lg)
            }
            .boxed()
        });
        assert!(html.contains(r#"aria-label="Close""#), "{html}");
        assert!(
            html.contains(r#"class="icon-button ghost lg wash""#),
            "{html}"
        );
        assert!(html.contains(r#"width="32""#), "{html}");
        assert!(!html.contains("aria-pressed"), "{html}");
    }

    #[test]
    fn pressed_renders_a_string_state() {
        let on = render("en", |cx| {
            view! { icon_button(glyph: Icon::Heart, label: "Like", pressed: true) }.boxed()
        });
        let off = render("en", |cx| {
            view! {
                icon_button(
                    glyph: Icon::Heart,
                    label: "Like",
                    pressed: false,
                    variant: IconButtonVariant::Outline
                )
            }
            .boxed()
        });
        assert!(on.contains(r#"aria-pressed="true""#), "{on}");
        assert!(off.contains(r#"aria-pressed="false""#), "{off}");
        assert!(
            off.contains(r#"class="icon-button outline md wash""#),
            "{off}"
        );
    }
}
