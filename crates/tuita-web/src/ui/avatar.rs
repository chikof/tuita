//! Avatar: a generated or uploaded picture in the account's shape.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::ui::{identicon::avatar_bits, mask::bits_mask};

/// The sizes the design uses, in px.
pub(crate) const AVATAR_SIZES: [u32; 7] = [24, 32, 40, 48, 64, 96, 128];

/// The outline an avatar is clipped to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AvatarShape {
    Square,
    #[default]
    Circle,
    Squircle,
    Octagon,
}

impl AvatarShape {
    /// The shape for the account setting `name` (`"square"`, `"circle"`, ...); `None` for an
    /// unknown name.
    #[must_use]
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "square" => Some(Self::Square),
            "circle" => Some(Self::Circle),
            "squircle" => Some(Self::Squircle),
            "octagon" => Some(Self::Octagon),
            _ => None,
        }
    }
}

/// `round(value / grid) * grid` with JavaScript's `Math.round` (ties go up; values here are
/// never negative).
fn snap(value: f64, grid: f64) -> i64 {
    ((value / grid).round() * grid) as i64
}

/// `polygon(...)` of `points`, dropping a point equal to the one before it.
fn polygon(points: &[(i64, i64)]) -> String {
    let mut unique: Vec<(i64, i64)> = Vec::with_capacity(points.len());
    for point in points {
        if unique.last() != Some(point) {
            unique.push(*point);
        }
    }
    let list: Vec<String> = unique.iter().map(|(x, y)| format!("{x}px {y}px")).collect();
    format!("polygon({})", list.join(", "))
}

/// Points of `|x/a|^n + |y/a|^n = 1` sampled `count` times and snapped to the 2px grid.
fn superellipse(size: i64, n: f64, count: usize) -> Vec<(i64, i64)> {
    let half = size as f64 / 2.0;
    (0..count)
        .map(|i| {
            let t = std::f64::consts::TAU * i as f64 / count as f64;
            let (c, s) = (t.cos(), t.sin());
            let x = half + half * c.signum_js() * c.abs().powf(2.0 / n);
            let y = half + half * s.signum_js() * s.abs().powf(2.0 / n);
            (size.min(snap(x, 2.0)), size.min(snap(y, 2.0)))
        })
        .collect()
}

/// JavaScript's `Math.sign` (zero stays zero), unlike `f64::signum`.
trait SignumJs {
    fn signum_js(self) -> f64;
}

impl SignumJs for f64 {
    fn signum_js(self) -> f64 {
        if self == 0.0 { 0.0 } else { self.signum() }
    }
}

/// CSS `clip-path` for an avatar of `size` px. Every vertex lies on the 2px grid.
#[must_use]
pub(crate) fn avatar_clip(shape: AvatarShape, size: u32) -> String {
    let size = i64::from(size);
    match shape {
        AvatarShape::Circle => polygon(&superellipse(size, 2.0, 48)),
        AvatarShape::Squircle => polygon(&superellipse(size, 4.0, 32)),
        AvatarShape::Octagon => {
            let c = snap(size as f64 * 0.29, 2.0);
            polygon(&[
                (c, 0),
                (size - c, 0),
                (size, c),
                (size, size - c),
                (size - c, size),
                (c, size),
                (0, size - c),
                (0, c),
            ])
        }
        AvatarShape::Square => {
            let c = 2.max(snap(size as f64 / 16.0, 2.0));
            let k = 2 * c;
            polygon(&[
                (0, k),
                (c, k),
                (c, c),
                (k, c),
                (k, 0),
                (size - k, 0),
                (size - k, c),
                (size - c, c),
                (size - c, k),
                (size, k),
                (size, size - k),
                (size - c, size - k),
                (size - c, size - c),
                (size - k, size - c),
                (size - k, size),
                (k, size),
                (k, size - c),
                (c, size - c),
                (c, size - k),
                (0, size - k),
            ])
        }
    }
}

/// A picture `size` px square, clipped to `shape`: `src` when the account has uploaded one, else a
/// dithered identicon of `seed`. `alt` names the picture; with an empty `alt` the identicon is
/// hidden from assistive technology (the name sits next to it). `ink` is the account's own ink
/// for the generated face (`data-ink`); without it the page ink shows.
#[component]
pub(crate) async fn avatar(
    cx: &Cx,
    seed: &str,
    #[default]
    #[into]
    src: Option<&str>,
    alt: &str,
    #[default] shape: AvatarShape,
    #[default(40)] size: u32,
    #[default]
    #[into]
    ink: Option<&str>,
) -> Result<impl View> {
    let style = format!("--size:{size}px;clip-path:{}", avatar_clip(shape, size));
    let mask = match src {
        Some(_) => String::new(),
        None => bits_mask(&avatar_bits(seed, 8, 2), 16),
    };
    let face_style = format!("mask-image:{mask};-webkit-mask-image:{mask}");
    Ok(view! {
        <span class="avatar" data-ink=(ink) style=(style) data-c="avatar">
            if let Some(src) = src {
                <img src=(src) alt=(alt) width=(size) height=(size)>
            } else {
                <span
                    class="face"
                    role=((!alt.is_empty()).then_some("img"))
                    aria-label=((!alt.is_empty()).then_some(alt))
                    aria-hidden=(alt.is_empty().then_some("true"))
                >
                    <i style=(face_style)></i>
                </span>
            }
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{testing::render, ui::hash::fnv1a};

    const SHAPES: [AvatarShape; 4] = [
        AvatarShape::Square,
        AvatarShape::Circle,
        AvatarShape::Squircle,
        AvatarShape::Octagon,
    ];

    /// The `(x, y)` pairs of a `polygon(...)`.
    fn points(clip: &str) -> Vec<(i64, i64)> {
        clip.trim_start_matches("polygon(")
            .trim_end_matches(')')
            .split(", ")
            .map(|pair| {
                let (x, y) = pair.split_once(' ').expect("x y");
                (
                    x.trim_end_matches("px").parse().expect("x"),
                    y.trim_end_matches("px").parse().expect("y"),
                )
            })
            .collect()
    }

    #[test]
    fn every_vertex_is_on_the_two_pixel_grid_inside_the_box() {
        for shape in SHAPES {
            for size in AVATAR_SIZES {
                let pts = points(&avatar_clip(shape, size));
                assert!(pts.len() >= 8, "{shape:?} {size}");
                for (x, y) in pts {
                    assert!(x % 2 == 0 && y % 2 == 0, "{shape:?} {size}: {x} {y}");
                    assert!(x <= i64::from(size) && y <= i64::from(size));
                }
            }
        }
    }

    #[test]
    fn an_octagon_cuts_the_corners_and_leaves_the_edge_midpoints() {
        let pts = points(&avatar_clip(AvatarShape::Octagon, 48));
        assert_eq!(pts.len(), 8);
        assert!(!pts.contains(&(0, 0)));
        assert!(pts.contains(&(14, 0)));
    }

    #[test]
    fn a_squircle_reaches_nearer_the_corners_than_a_circle() {
        let diagonal = |shape| {
            points(&avatar_clip(shape, 96))
                .into_iter()
                .map(|(x, y)| x.min(y))
                .max()
        };
        assert!(diagonal(AvatarShape::Squircle) > diagonal(AvatarShape::Circle));
        assert!(
            !points(&avatar_clip(AvatarShape::Squircle, 96))
                .iter()
                .any(|(x, y)| *x < 4 && *y < 4)
        );
    }

    /// FNV-1a of the clip strings `avatarClip` returns.
    #[test]
    fn clips_match_the_browser() {
        let cases = [
            (AvatarShape::Circle, 24, 1_385_578_626),
            (AvatarShape::Circle, 40, 4_262_497_958),
            (AvatarShape::Circle, 96, 1_674_221_384),
            (AvatarShape::Square, 24, 1_702_449_770),
            (AvatarShape::Square, 40, 891_136_910),
            (AvatarShape::Square, 96, 1_949_923_734),
            (AvatarShape::Squircle, 24, 1_009_482_006),
            (AvatarShape::Squircle, 40, 2_293_843_278),
            (AvatarShape::Squircle, 96, 2_835_309_624),
            (AvatarShape::Octagon, 24, 3_202_210_622),
            (AvatarShape::Octagon, 40, 1_543_386_194),
            (AvatarShape::Octagon, 96, 3_333_118_258),
        ];
        for (shape, size, hash) in cases {
            assert_eq!(fnv1a(&avatar_clip(shape, size)), hash, "{shape:?} {size}");
        }
    }

    #[test]
    fn face_masks_match_the_browser() {
        let mask = |seed| bits_mask(&avatar_bits(seed, 8, 2), 16);
        assert_eq!(fnv1a(&mask("")), 2_158_976_628);
        assert_eq!(fnv1a(&mask("a")), 1_108_304_327);
        assert_eq!(fnv1a(&mask("ada")), 3_389_049_035);
        assert_eq!(fnv1a(&mask("héllo😀")), 2_702_631_790);
    }

    #[test]
    fn a_picture_is_an_img_with_its_alt_and_size() {
        let html = render("en", |cx| {
            view! { avatar(seed: "ada", src: "/m/a.png", alt: "Ada", size: 48) }.boxed()
        });
        assert!(
            html.contains(r#"<img src="/m/a.png" alt="Ada" width="48" height="48">"#),
            "{html}"
        );
        assert!(!html.contains("face"), "{html}");
    }

    #[test]
    fn a_generated_face_is_named_by_alt_or_hidden() {
        let named = render("en", |cx| view! { avatar(seed: "ada", alt: "Ada") }.boxed());
        assert!(named.contains(r#"role="img""#) && named.contains(r#"aria-label="Ada""#));
        assert!(!named.contains("aria-hidden"), "{named}");
        let hidden = render("en", |cx| view! { avatar(seed: "ada", alt: "") }.boxed());
        assert!(
            hidden.contains(r#"aria-hidden="true""#) && !hidden.contains("role="),
            "{hidden}"
        );
        assert!(hidden.contains("mask-image:url("), "{hidden}");
    }

    #[test]
    fn a_decorative_picture_keeps_an_empty_alt() {
        let html = render("en", |cx| {
            view! { avatar(seed: "ada", src: "/m/a.png", alt: "") }.boxed()
        });
        assert!(html.contains(r#"alt="""#), "{html}");
    }

    #[test]
    fn an_account_ink_is_a_data_attribute() {
        let html = render("en", |cx| {
            view! { avatar(seed: "ada", alt: "", ink: "amber") }.boxed()
        });
        assert!(html.contains(r#"data-ink="amber""#), "{html}");
    }
}
