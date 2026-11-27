//! A dithered placeholder.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

/// What a [`skeleton`] stands in for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SkeletonShape {
    /// A 64px tall box.
    #[default]
    Block,
    /// A 12px tall line of text.
    Line,
    /// A round avatar of `size` px.
    Avatar,
}

impl SkeletonShape {
    const fn class(self) -> &'static str {
        match self {
            Self::Block => "skeleton block",
            Self::Line => "skeleton line",
            Self::Avatar => "skeleton avatar",
        }
    }
}

/// An `aria-hidden` dithered block with a band sweeping across it. `width` and `height` are CSS
/// lengths (`"60%"`, `"1.5rem"`) that override the shape's default; an avatar is `size` px square.
#[component]
pub(crate) async fn skeleton(
    cx: &Cx,
    #[default] shape: SkeletonShape,
    #[default]
    #[into]
    width: Option<&str>,
    #[default]
    #[into]
    height: Option<&str>,
    #[default(40)] size: u32,
) -> Result<impl View> {
    let (w, h) = if shape == SkeletonShape::Avatar {
        (Some(format!("{size}px")), Some(format!("{size}px")))
    } else {
        (width.map(str::to_owned), height.map(str::to_owned))
    };
    let style: String = [("--w", w), ("--h", h)]
        .into_iter()
        .filter_map(|(name, value)| value.map(|value| format!("{name}:{value};")))
        .collect();
    Ok(view! {
        <span
            class=(shape.class())
            aria-hidden="true"
            style=((!style.is_empty()).then_some(style))
            data-c="skeleton"
        ></span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn a_block_is_hidden_and_unsized_by_default() {
        let html = render("en", |cx| view! { skeleton() }.boxed());
        assert!(html.contains(r#"class="skeleton block""#), "{html}");
        assert!(html.contains(r#"aria-hidden="true""#), "{html}");
        assert!(!html.contains("style"), "{html}");
    }

    #[test]
    fn a_line_takes_a_width_and_an_avatar_a_size() {
        let line = render("en", |cx| {
            view! { skeleton(shape: SkeletonShape::Line, width: "60%") }.boxed()
        });
        assert!(
            line.contains(r#"class="skeleton line""#) && line.contains("--w:60%;"),
            "{line}"
        );
        let face = render("en", |cx| {
            view! { skeleton(shape: SkeletonShape::Avatar, size: 48) }.boxed()
        });
        assert!(face.contains("--w:48px;--h:48px;"), "{face}");
    }
}
