//! The size of a media strip.
//!
//! The server draws the first frame at the fallback column width; `lightbox.js` measures the real
//! column and applies the same maths again.

/// Pixels between two items of the strip.
pub(crate) const GAP: f64 = 2.0;
/// How much of the next item stays visible so the strip reads as scrollable.
pub(crate) const PEEK: f64 = 24.0;
/// The tallest a strip gets, as for a single photo.
pub(crate) const MAX_HEIGHT: f64 = 560.0;
/// The column width used before the real one is measured.
pub(crate) const COLUMN_FALLBACK: f64 = 598.0;

const NO_SIZE_ASPECT: f64 = 4.0 / 3.0;
/// The first picture sets the strip height within 4:5 and 16:9.
const FIRST_MIN: f64 = 4.0 / 5.0;
const WIDE: f64 = 16.0 / 9.0;
/// An item is never narrower than 1:2 nor wider than 16:9.
const ITEM_MIN: f64 = 1.0 / 2.0;

/// The strip height and one width per item, in px.
#[derive(Debug, PartialEq)]
pub(crate) struct Layout {
    pub(crate) height: f64,
    pub(crate) widths: Vec<f64>,
}

/// Width over height; an item without usable dimensions counts as 4:3.
#[must_use]
pub(crate) fn aspect_of(width: Option<i32>, height: Option<i32>) -> f64 {
    match (width, height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => f64::from(w) / f64::from(h),
        _ => NO_SIZE_ASPECT,
    }
}

/// Item sizes for a strip in a column `column_width` px wide. The first item sets the height;
/// every item keeps its own shape within limits, so wide ones are cropped by the caller's
/// `object-fit: cover`. A portrait first item gets a height that fits two items side by side,
/// plus a peek of the third when there is one.
#[must_use]
pub(crate) fn carousel_layout(aspects: &[f64], column_width: f64) -> Layout {
    if aspects.is_empty() || column_width <= 0.0 {
        return Layout {
            height: 0.0,
            widths: vec![0.0; aspects.len()],
        };
    }
    let many = aspects.len() > 1;
    let first_shape = aspects[0].clamp(ITEM_MIN, WIDE);
    let mut height = column_width / aspects[0].clamp(FIRST_MIN, WIDE);
    if many && first_shape < 1.0 {
        let room = column_width - GAP - if aspects.len() > 2 { PEEK } else { 0.0 };
        height = height.min(room / (2.0 * first_shape));
    }
    let height = height.min(MAX_HEIGHT).floor();
    let widest = if many {
        column_width - PEEK
    } else {
        column_width
    };
    let widths = aspects
        .iter()
        .map(|aspect| (height * aspect.clamp(ITEM_MIN, WIDE)).min(widest).floor())
        .collect();
    Layout { height, widths }
}

/// Whether the strip overflows its column, so it needs arrows and a pager.
#[must_use]
pub(crate) fn scrolls(widths: &[f64], viewport: f64) -> bool {
    let content: f64 = widths.iter().map(|width| width + GAP).sum::<f64>() - GAP;
    (content - viewport).max(0.0) > 1.0
}

#[cfg(test)]
mod test {
    use super::*;

    const PORTRAIT: f64 = 800.0 / 1200.0;
    const LANDSCAPE: f64 = 1600.0 / 900.0;
    const ULTRAWIDE: f64 = 3000.0 / 1000.0;

    #[test]
    fn two_portrait_items_and_a_peek_of_the_third_fit() {
        let layout = carousel_layout(&[PORTRAIT, PORTRAIT, PORTRAIT], 600.0);
        assert_eq!(layout.height, 430.0);
        assert_eq!(layout.widths, [286.0, 286.0, 286.0]);
        let third = 2.0 * (286.0 + GAP);
        assert!((600.0 - PEEK..600.0).contains(&third));
    }

    #[test]
    fn exactly_two_portrait_items_need_no_scrolling() {
        let layout = carousel_layout(&[PORTRAIT, PORTRAIT], 600.0);
        assert!(layout.widths.iter().sum::<f64>() + GAP <= 600.0);
    }

    #[test]
    fn wide_items_crop_to_sixteen_by_nine_and_the_next_one_peeks() {
        let layout = carousel_layout(&[ULTRAWIDE; 4], 600.0);
        assert_eq!(layout.height, 337.0);
        assert_eq!(layout.widths, [576.0; 4]);
    }

    #[test]
    fn the_first_item_sets_the_height_and_each_the_own_width() {
        let layout = carousel_layout(&[LANDSCAPE, PORTRAIT, LANDSCAPE], 600.0);
        assert_eq!(layout.height, 337.0);
        assert_eq!(layout.widths[0], 576.0);
        assert_eq!(layout.widths[1], (337.0 * PORTRAIT).floor());
        assert_eq!(layout.widths[2], 576.0);
        let mixed = carousel_layout(&[PORTRAIT, LANDSCAPE], 600.0);
        assert_eq!(mixed.height, 448.0);
        assert_eq!(mixed.widths[1], 576.0);
    }

    #[test]
    fn items_without_size_count_as_four_by_three() {
        assert!((aspect_of(None, None) - 4.0 / 3.0).abs() < 1e-9);
        assert!((aspect_of(Some(100), None) - 4.0 / 3.0).abs() < 1e-9);
        assert!((aspect_of(Some(0), Some(0)) - 4.0 / 3.0).abs() < 1e-9);
        let unknown = aspect_of(None, None);
        let layout = carousel_layout(&[unknown, unknown], 600.0);
        assert_eq!(layout.height, 450.0);
        assert_eq!(layout.widths, [576.0, 576.0]);
    }

    #[test]
    fn the_height_is_capped_and_a_narrow_column_shrinks_everything() {
        let thin = 500.0 / 2000.0;
        assert_eq!(carousel_layout(&[thin; 3], 1200.0).height, MAX_HEIGHT);
        let narrow = carousel_layout(&[PORTRAIT; 3], 300.0);
        assert_eq!(
            narrow.height,
            ((300.0 - GAP - PEEK) / (2.0 * (2.0 / 3.0))).floor()
        );
        assert!(narrow.widths.iter().all(|width| *width < 300.0));
    }

    #[test]
    fn an_unmeasured_column_has_no_size() {
        assert_eq!(
            carousel_layout(&[PORTRAIT, PORTRAIT], 0.0),
            Layout {
                height: 0.0,
                widths: vec![0.0, 0.0]
            }
        );
        assert_eq!(
            carousel_layout(&[], 600.0),
            Layout {
                height: 0.0,
                widths: vec![]
            }
        );
    }
}
