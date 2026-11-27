//! The characters-left ring of a composer.

use topcoat::{
    Result,
    context::Cx,
    view::{View, class, component, view},
};

use crate::l10n::t;

/// Segments of the ring.
const SEGMENTS: usize = 12;

/// `round(value / 2) * 2` with JavaScript's `Math.round` (ties go up).
fn snap(value: f64) -> i32 {
    ((value / 2.0 + 0.5).floor() * 2.0) as i32
}

/// Top-left corner of each segment's 4px square, on the 2px grid.
fn cells() -> Vec<(i32, i32)> {
    (0..SEGMENTS)
        .map(|i| {
            let angle = (i as f64 * 2.0 * std::f64::consts::PI) / SEGMENTS as f64;
            (
                snap(12.0 + 8.0 * angle.sin() - 2.0),
                snap(12.0 - 8.0 * angle.cos() - 2.0),
            )
        })
        .collect()
}

/// A ring of twelve pixel segments that fill as `value` characters approach `max`, an
/// `role="img"` named "N characters left" or "N characters over". The number beside it shows
/// when at most 20 are left. This renders the state the server knows; a composer island that
/// counts as the user types redraws the segments (`rect.on`), the number and the `over` class.
#[component]
pub(crate) async fn char_counter(cx: &Cx, value: i64, max: i64) -> Result<impl View> {
    let remaining = max - value;
    let over = remaining < 0;
    let filled = if max > 0 {
        ((value.max(0) as f64 / max as f64) * SEGMENTS as f64)
            .ceil()
            .min(SEGMENTS as f64) as usize
    } else {
        SEGMENTS
    };
    let name = if over {
        t!(char_counter_over, count: -remaining)
            .resolve(cx)
            .into_owned()
    } else {
        t!(char_counter_left, count: remaining)
            .resolve(cx)
            .into_owned()
    };
    Ok(view! {
        <span
            class=(class!("counter", "over" if over))
            role="img"
            aria-label=(name)
            data-c="char-counter"
        >
            <svg
                viewBox="0 0 24 24"
                width="24"
                height="24"
                shape-rendering="crispEdges"
                aria-hidden="true"
            >
                for (i, (x, y)) in cells().into_iter().enumerate() {
                    <rect
                        x=(x)
                        y=(y)
                        width="4"
                        height="4"
                        class=((i < filled).then_some("on"))
                    ></rect>
                }
            </svg>
            if remaining <= 20 {
                <span class="number" aria-hidden="true">(remaining)</span>
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
    fn the_segments_sit_on_a_ring_snapped_to_two_pixels() {
        let cells = cells();
        assert_eq!(cells.len(), 12);
        assert_eq!(cells[0], (10, 2));
        assert_eq!(cells[3], (18, 10));
        assert!(cells.iter().all(|(x, y)| x % 2 == 0 && y % 2 == 0));
    }

    #[test]
    fn it_is_named_by_what_is_left_and_fills_in_proportion() {
        let html = render("en", |cx| {
            view! { char_counter(value: 250, max: 500) }.boxed()
        });
        assert!(html.contains(r#"role="img""#), "{html}");
        assert!(html.contains("250"), "{html}");
        assert_eq!(html.matches(r#"class="on""#).count(), 6, "{html}");
        assert!(!html.contains(r#"class="number""#), "{html}");
    }

    #[test]
    fn near_the_limit_the_number_shows_and_over_it_turns_danger() {
        let near = render("en", |cx| {
            view! { char_counter(value: 485, max: 500) }.boxed()
        });
        assert!(
            near.contains(r#"<span class="number" aria-hidden="true">15</span>"#),
            "{near}"
        );
        assert_eq!(near.matches(r#"class="on""#).count(), 12, "{near}");
        let over = render("en", |cx| {
            view! { char_counter(value: 505, max: 500) }.boxed()
        });
        assert!(over.contains(r#"class="counter over""#), "{over}");
        assert!(over.contains(r#"aria-hidden="true">-5</span>"#), "{over}");
        assert!(
            crate::l10n::strip_isolates(&over).contains("5 characters over"),
            "{over}"
        );
    }
}
