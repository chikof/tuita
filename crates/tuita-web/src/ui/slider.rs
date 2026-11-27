//! A dithered range slider.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, component, element_id, view},
};

use crate::ui::mask::{Axis, level_mask, ramp_mask};

/// The share of the track `value` fills, in percent.
fn percent(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() < f64::EPSILON {
        0.0
    } else {
        (value - min) / (max - min) * 100.0
    }
}

/// A native `<input type="range">` over a dithered rail, with the label and the current value
/// (`<output>`) above it. `attrs` go on the `<input>` (`name`, `disabled`, ...).
///
/// The fill (`--pct`) and the `<output>` follow the input through `data-island="slider"`
/// (`cropper.js`); a form that reads the value needs nothing else.
#[component]
pub(crate) async fn slider(
    cx: &Cx,
    label: &str,
    #[default] value: f64,
    #[default(0.0)] min: f64,
    #[default(100.0)] max: f64,
    #[default(1.0)] step: f64,
    #[default] attrs: Attributes,
) -> Result<impl View> {
    let id = element_id(cx);
    let style = format!(
        "--pct:{}%;--base:{};--ramp:{}",
        percent(value, min, max),
        level_mask(0.15),
        ramp_mask(0.15, 1.0, 16, Axis::X),
    );
    Ok(view! {
        <div class="slider" data-island="slider" style=(style) data-c="slider">
            <label for=(&id)>
                (label)
                <output>(value)</output>
            </label>
            <div class="control">
                <span class="rail" aria-hidden="true"></span>
                <span class="fill" aria-hidden="true"></span>
                <input
                    (attrs)
                    id=(&id)
                    type="range"
                    min=(min)
                    max=(max)
                    step=(step)
                    value=(value)
                >
            </div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::{ViewExt, attributes};

    use super::*;
    use crate::testing::{attr_values, render};

    #[test]
    fn the_percentage_is_the_position_between_min_and_max() {
        assert!((percent(25.0, 0.0, 100.0) - 25.0).abs() < 1e-9);
        assert!((percent(3.5, 1.0, 6.0) - 50.0).abs() < 1e-9);
        assert!(percent(5.0, 5.0, 5.0).abs() < 1e-9);
    }

    #[test]
    fn the_label_names_a_range_input_and_the_fill_starts_at_the_value() {
        let html = render("en", |cx| {
            view! {
                slider(
                    label: "Zoom",
                    value: 3.5,
                    min: 1.0,
                    max: 6.0,
                    step: 0.01,
                    attrs: attributes! { name="zoom" }
                )
            }
            .boxed()
        });
        let id = attr_values(&html, "for").pop().expect("label for");
        assert!(html.contains(&format!(r#"id="{id}""#)), "{html}");
        assert!(html.contains(r#"type="range""#), "{html}");
        assert!(
            html.contains(r#"min="1""#)
                && html.contains(r#"max="6""#)
                && html.contains(r#"step="0.01""#),
            "{html}"
        );
        assert!(html.contains("<output>3.5</output>"), "{html}");
        assert!(html.contains("--pct:50%"), "{html}");
        assert!(html.contains(r#"name="zoom""#), "{html}");
    }
}
