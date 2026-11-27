//! A dithered progress bar.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::ui::mask::{Axis, ramp_mask};

/// A `role="progressbar"` named `label`. With a `value` (0 to `max`) it fills that share; without
/// one it sweeps a dithered comet.
#[component]
pub(crate) async fn progress_bar(
    cx: &Cx,
    #[default]
    #[into]
    value: Option<f64>,
    #[default(100.0)] max: f64,
    label: &str,
) -> Result<impl View> {
    let percent = value.map_or(0.0, |value| (value / max * 100.0).clamp(0.0, 100.0));
    Ok(view! {
        <div
            class="track"
            role="progressbar"
            aria-label=(label)
            aria-valuemin="0"
            aria-valuemax=(max)
            aria-valuenow=(value)
            style=(format!("--comet:{}", ramp_mask(0.0, 1.0, 12, Axis::X)))
            data-c="progress-bar"
        >
            if value.is_some() {
                <i class="fill" aria-hidden="true" style=(format!("width:{percent}%"))></i>
            } else {
                <i class="comet" aria-hidden="true"></i>
            }
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn a_value_fills_that_share() {
        let html = render("en", |cx| {
            view! { progress_bar(value: 25.0, label: "Uploading") }.boxed()
        });
        assert!(html.contains(r#"role="progressbar""#), "{html}");
        assert!(html.contains(r#"aria-label="Uploading""#), "{html}");
        assert!(
            html.contains(r#"aria-valuenow="25""#) && html.contains(r#"aria-valuemax="100""#),
            "{html}"
        );
        assert!(html.contains("width:25%"), "{html}");
        assert!(!html.contains("comet\""), "{html}");
    }

    #[test]
    fn the_fill_is_clamped() {
        let html = render("en", |cx| {
            view! { progress_bar(value: 300.0, max: 200.0, label: "x") }.boxed()
        });
        assert!(html.contains("width:100%"), "{html}");
    }

    #[test]
    fn no_value_sweeps_a_comet() {
        let html = render("en", |cx| view! { progress_bar(label: "Working") }.boxed());
        assert!(html.contains(r#"class="comet""#), "{html}");
        assert!(!html.contains("aria-valuenow"), "{html}");
    }
}
