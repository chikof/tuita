//! A drop target for files.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

use crate::l10n::t;

/// Wraps `child` in a zone that shows a marching dither ring and a hint while files are dragged
/// over it. `accept` is a comma-separated MIME list (`image/*,video/*`, empty for anything); the
/// zone turns `--danger` while a refused type hovers.
///
/// The markup is complete and styled by attributes: an island sets `data-over` on the zone while
/// a file drag is over it (and `data-rejected` too when a dragged type is refused), and on a drop
/// dispatches a bubbling `tuita-files` event with `detail.files`. `disabled` renders
/// `data-disabled`, which the island must honour.
#[component]
pub(crate) async fn dropzone(
    cx: &Cx,
    #[default] accept: &str,
    #[default] disabled: bool,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div
            class="zone"
            role="presentation"
            data-island="dropzone"
            data-accept=(accept)
            data-disabled=(disabled.then_some("true"))
            data-c="dropzone"
        >
            (child)
            <div class="ring" aria-hidden="true">
                <i class="top"></i>
                <i class="bottom"></i>
                <i class="left"></i>
                <i class="right"></i>
            </div>
            <p class="hint" role="status">
                <span class="drop">(t!(dropzone_drop))</span>
                <span class="reject">(t!(dropzone_reject))</span>
            </p>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_wraps_the_content_and_carries_both_hints() {
        let html = render("en", |cx| {
            view! { dropzone(accept: "image/*", <p>"inside"</p>) }.boxed()
        });
        assert!(html.contains(r#"class="zone""#), "{html}");
        assert!(html.contains(r#"data-accept="image/*""#), "{html}");
        assert!(html.contains("<p>inside</p>"), "{html}");
        assert!(
            html.contains(r#"<span class="drop">Drop to attach</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"class="reject">Can’t use this file</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"role="status""#), "{html}");
        assert!(!html.contains("data-disabled"), "{html}");
    }

    #[test]
    fn disabled_is_flagged() {
        let html = render("en", |cx| view! { dropzone(disabled: true, "x") }.boxed());
        assert!(html.contains(r#"data-disabled="true""#), "{html}");
    }
}
