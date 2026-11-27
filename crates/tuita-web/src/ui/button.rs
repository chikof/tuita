//! Button.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, View, class, component, view},
};

use crate::ui::loader::{LoaderVariant, loader};

/// The look of a [`button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ButtonVariant {
    #[default]
    Solid,
    Outline,
    Ghost,
    Danger,
}

impl ButtonVariant {
    pub(crate) const fn class(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Danger => "danger",
        }
    }
}

/// The size of a [`button`] or an icon button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl ButtonSize {
    pub(crate) const fn class(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// A text button. `loading` swaps the label for a spinner (the label keeps its width) and sets
/// `aria-busy`. `submit` makes it `type="submit"` (otherwise `type="button"`).
///
/// `attrs` go on the `<button>`: `id`, `name`, `form`, `command`/`commandfor`, `@click=$(..)`,
/// `class` (merged). The dither masks of the press animation (`--m25`, `--m50`) are page-wide
/// custom properties in `ui.css`, not set per button.
#[component]
pub(crate) async fn button(
    cx: &Cx,
    #[default] variant: ButtonVariant,
    #[default] size: ButtonSize,
    #[default] loading: bool,
    #[default] disabled: bool,
    #[default] submit: bool,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let kind = if submit { "submit" } else { "button" };
    Ok(view! {
        <button
            type=(kind)
            class=(class!(
                "btn",
                variant.class(),
                size.class(),
                "wash",
                attrs.remove("class"),
            ))
            disabled=(disabled || loading)
            aria-busy=(loading.then_some("true"))
            data-c="button"
            (attrs)
        >
            <span class=(class!("label", "hidden" if loading))>(child)</span>
            if loading {
                <span class="spinner">
                    loader(variant: LoaderVariant::Dots, size: 16)
                </span>
            }
        </button>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::{ViewExt, attributes};

    use super::*;
    use crate::testing::render;

    #[test]
    fn defaults_are_a_solid_medium_type_button() {
        let html = render("en", |cx| view! { button("Save") }.boxed());
        assert!(html.contains(r#"class="btn solid md wash""#), "{html}");
        assert!(html.contains(r#"type="button""#), "{html}");
        assert!(
            html.contains(r#"<span class="label">Save</span>"#),
            "{html}"
        );
        assert!(
            !html.contains("disabled") && !html.contains("aria-busy"),
            "{html}"
        );
    }

    #[test]
    fn variant_size_and_submit_pick_the_classes_and_type() {
        let html = render("en", |cx| {
            view! {
                button(
                    variant: ButtonVariant::Danger,
                    size: ButtonSize::Lg,
                    submit: true,
                    "Delete"
                )
            }
            .boxed()
        });
        assert!(html.contains(r#"class="btn danger lg wash""#), "{html}");
        assert!(html.contains(r#"type="submit""#), "{html}");
    }

    #[test]
    fn loading_disables_hides_the_label_and_shows_a_status_spinner() {
        let html = render("en", |cx| view! { button(loading: true, "Save") }.boxed());
        assert!(html.contains("disabled"), "{html}");
        assert!(html.contains(r#"aria-busy="true""#), "{html}");
        assert!(
            html.contains(r#"<span class="label hidden">Save</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"role="status""#) && html.contains("Loading"),
            "{html}"
        );
    }

    #[test]
    fn attrs_land_on_the_button_and_classes_merge() {
        let html = render("en", |cx| {
            view! { button(attrs: attributes! { id="go" class="wide" }, "Go") }.boxed()
        });
        assert!(html.contains(r#"id="go""#), "{html}");
        assert!(html.contains(r#"class="btn solid md wash wide""#), "{html}");
    }
}
