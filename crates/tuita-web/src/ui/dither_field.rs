//! A canvas of dithered, animated tone.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

/// The picture a [`dither_field`] draws, by the name `dither.js` knows it under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FieldKind {
    /// The demo's flowing luminance field.
    #[default]
    Waves,
    /// Light falling off from a centre (`cx`, `cy`).
    Radial,
    /// A gradient at `angle` degrees.
    Linear,
    /// Drifting value noise of `seed`.
    Noise,
    /// Rings from a centre drawn from `seed`.
    Ripples,
    /// A shaded sphere with a light circling it.
    Orb,
    /// One of the above with parameters drawn from a hash of `seed`: the same text always draws
    /// the same picture.
    Seeded,
}

impl FieldKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Waves => "waves",
            Self::Radial => "radial",
            Self::Linear => "linear",
            Self::Noise => "noise",
            Self::Ripples => "ripples",
            Self::Orb => "orb",
            Self::Seeded => "seeded",
        }
    }
}

/// A `<canvas>` that fills its positioned parent (`position: absolute; inset: 0`), drawn in the
/// ink colour by the `dither.js` island: `data-island="dither-field" data-field=(kind)`. The
/// attribute contract is documented at the top of `dither.js`.
///
/// `seed` is the text a `Seeded` field hashes; for `Noise` and `Ripples` a decimal number is the
/// seed itself and other text is hashed. `centre_x`/`centre_y` apply to `Radial`, `angle` to `Linear`.
/// `cell` is CSS pixels per dither cell, `matrix` the Bayer size (2, 4 or 8), `speed` a multiplier
/// on the animation, `still_time` the moment a still picture shows. `animate: false` draws one
/// frame; so does a reduced-motion page.
#[component]
pub(crate) async fn dither_field(
    cx: &Cx,
    #[default] field: FieldKind,
    #[default(6)] cell: u32,
    #[default(4)] matrix: u32,
    #[default(true)] animate: bool,
    #[default(0.5)] opacity: f32,
    #[default(1.0)] speed: f32,
    #[default(1.0)] still_time: f32,
    #[default]
    #[into]
    seed: Option<&str>,
    #[default(0.5)] centre_x: f32,
    #[default(0.5)] centre_y: f32,
    #[default(0.0)] angle: f32,
) -> Result<impl View> {
    let radial = field == FieldKind::Radial;
    let linear = field == FieldKind::Linear;
    Ok(view! {
        <canvas
            aria-hidden="true"
            data-island="dither-field"
            data-field=(field.name())
            data-seed=(seed)
            data-cx=(radial.then_some(centre_x))
            data-cy=(radial.then_some(centre_y))
            data-angle=(linear.then_some(angle))
            data-cell=(cell)
            data-matrix=((matrix != 4).then_some(matrix))
            data-animate=((!animate).then_some("false"))
            data-speed=((speed != 1.0).then_some(speed))
            data-still-time=((still_time != 1.0).then_some(still_time))
            data-opacity=(opacity)
            style=(format!("opacity:{opacity}"))
            data-c="dither-field"
        ></canvas>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_defaults_draw_the_waves_at_half_opacity() {
        let html = render("en", |cx| view! { dither_field() }.boxed());
        assert!(html.starts_with(r#"<canvas aria-hidden="true""#), "{html}");
        assert!(html.contains(r#"data-island="dither-field""#), "{html}");
        assert!(html.contains(r#"data-field="waves""#), "{html}");
        assert!(html.contains(r#"data-cell="6""#), "{html}");
        assert!(html.contains(r#"data-opacity="0.5""#), "{html}");
        assert!(html.contains(r#"style="opacity:0.5""#), "{html}");
        for absent in ["data-seed", "data-matrix", "data-animate", "data-angle"] {
            assert!(!html.contains(absent), "{absent}: {html}");
        }
    }

    #[test]
    fn a_seeded_still_field_names_its_seed_and_options() {
        let html = render("en", |cx| {
            view! {
                dither_field(
                    field: FieldKind::Seeded,
                    seed: "ada",
                    cell: 4,
                    matrix: 8,
                    animate: false,
                    opacity: 0.6
                )
            }
            .boxed()
        });
        assert!(html.contains(r#"data-field="seeded""#), "{html}");
        assert!(html.contains(r#"data-seed="ada""#), "{html}");
        assert!(html.contains(r#"data-matrix="8""#), "{html}");
        assert!(html.contains(r#"data-animate="false""#), "{html}");
        assert!(html.contains(r#"data-opacity="0.6""#), "{html}");
    }

    #[test]
    fn centre_and_angle_only_ride_with_their_fields() {
        let radial = render("en", |cx| {
            view! {
                dither_field(field: FieldKind::Radial, centre_x: 0.25, centre_y: 0.75)
            }
            .boxed()
        });
        assert!(
            radial.contains(r#"data-cx="0.25""#) && radial.contains(r#"data-cy="0.75""#),
            "{radial}"
        );
        let linear = render("en", |cx| {
            view! { dither_field(field: FieldKind::Linear, angle: 90.0) }.boxed()
        });
        assert!(linear.contains(r#"data-angle="90""#), "{linear}");
        assert!(!linear.contains("data-cx"), "{linear}");
    }
}
