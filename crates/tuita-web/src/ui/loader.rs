//! A pixel loader: a ring, three dots or a sweeping bar.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    l10n::t,
    ui::mask::{Axis, ramp_mask},
};

/// The look of a [`loader`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum LoaderVariant {
    #[default]
    Ring,
    Dots,
    Bar,
}

impl LoaderVariant {
    /// The class and the number of `<i>` cells it draws.
    const fn parts(self) -> (&'static str, u8) {
        match self {
            Self::Ring => ("ring", 8),
            Self::Dots => ("dots", 3),
            Self::Bar => ("bar", 1),
        }
    }
}

/// An animated `role="status"` loader of `size` px whose accessible name is "Loading".
///
/// Only the bar draws a dithered comet (`--comet`), so only it carries that mask.
#[component]
pub(crate) async fn loader(
    cx: &Cx,
    #[default] variant: LoaderVariant,
    #[default(24)] size: u32,
) -> Result<impl View> {
    let (class, cells) = variant.parts();
    let style = match variant {
        LoaderVariant::Bar => format!(
            "--size:{size}px;--comet:{}",
            ramp_mask(0.0, 1.0, 12, Axis::X)
        ),
        _ => format!("--size:{size}px"),
    };
    Ok(view! {
        <span
            class=(format!("loader {class}"))
            role="status"
            style=(style)
            data-c="loader"
        >
            <span class="sr-only">(t!(loading))</span>
            for i in 0..cells {
                <i style=(format!("--i:{i}")) aria-hidden="true"></i>
            }
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn every_variant_is_a_status_named_loading() {
        for variant in [LoaderVariant::Ring, LoaderVariant::Dots, LoaderVariant::Bar] {
            let html = render("en", |cx| view! { loader(variant: variant) }.boxed());
            assert!(html.contains(r#"role="status""#), "{html}");
            assert!(
                html.contains(r#"<span class="sr-only">Loading</span>"#),
                "{html}"
            );
        }
    }

    #[test]
    fn the_variant_draws_its_cells() {
        let cells = |variant| {
            render("en", |cx| {
                view! { loader(variant: variant, size: 16) }.boxed()
            })
            .matches("<i ")
            .count()
        };
        assert_eq!(cells(LoaderVariant::Ring), 8);
        assert_eq!(cells(LoaderVariant::Dots), 3);
        assert_eq!(cells(LoaderVariant::Bar), 1);
    }

    #[test]
    fn only_the_bar_carries_the_comet_mask() {
        let bar = render("en", |cx| {
            view! { loader(variant: LoaderVariant::Bar) }.boxed()
        });
        let ring = render("en", |cx| view! { loader() }.boxed());
        assert!(bar.contains("--comet:url("), "{bar}");
        assert!(
            ring.contains("--size:24px") && !ring.contains("--comet"),
            "{ring}"
        );
    }
}
