//! Dither masks as CSS `url()` values.

/// Bayer rank matrix of side `n` (a power of two), row-major: `M(2n) = [[4M, 4M+2], [4M+3, 4M+1]]`.
pub(crate) fn bayer_ranks(n: usize) -> Vec<usize> {
    if n <= 1 {
        return vec![0];
    }
    let half = n / 2;
    let prev = bayer_ranks(half);
    let mut ranks = vec![0; n * n];
    for y in 0..n {
        for x in 0..n {
            let quadrant = match (y < half, x < half) {
                (true, true) => 0,
                (true, false) => 2,
                (false, true) => 3,
                (false, false) => 1,
            };
            ranks[y * n + x] = 4 * prev[(y % half) * half + (x % half)] + quadrant;
        }
    }
    ranks
}

/// CSS `url("data:image/svg+xml,...")` of an SVG with the markup `rects` over a `w` by `h` grid.
fn data_url(w: usize, h: usize, rects: &str) -> String {
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' viewBox='0 0 {w} {h}' shape-rendering='crispEdges'>{rects}</svg>"
    );
    let svg = svg
        .replace('<', "%3C")
        .replace('>', "%3E")
        .replace('#', "%23");
    format!("url(\"data:image/svg+xml,{svg}\")")
}

/// CSS `url()` of a 4 by 4 SVG with `round(level * 16)` lit cells, lit in Bayer order. Use it as
/// `mask-image` over a solid ink background. `level` outside `0..=1` is clamped.
#[must_use]
pub(crate) fn level_mask(level: f32) -> String {
    level_mask_n(level, 4)
}

/// [`level_mask`] over an `n` by `n` grid (`n` a power of two: 2, 4 or 8).
#[must_use]
pub(crate) fn level_mask_n(level: f32, n: usize) -> String {
    let lit = (level.clamp(0.0, 1.0) * (n * n) as f32).round() as usize;
    let rects: String = bayer_ranks(n)
        .iter()
        .enumerate()
        .filter(|(_, rank)| **rank < lit)
        .map(|(i, _)| rect(i % n, i / n, 1))
        .collect();
    data_url(n, n, &rects)
}

/// Bayer thresholds `(rank + 0.5) / n^2`, row-major, rounded to `f32` as the browser's
/// `Float32Array` stores them. A cell is lit when its tone exceeds its threshold.
#[must_use]
pub(crate) fn bayer_thresholds(n: usize) -> Vec<f32> {
    let ranks = bayer_ranks(n);
    let cells = ranks.len() as f64;
    ranks
        .iter()
        .map(|rank| ((*rank as f64 + 0.5) / cells) as f32)
        .collect()
}

/// The direction a [`ramp_mask`] runs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    X,
    Y,
}

/// CSS `url()` of a dithered ramp from tone `from` to tone `to` (both `0..=1`) over `steps` cells
/// along `axis`, 4 cells across. Stretch it with `mask-size` and repeat it along the other axis.
#[must_use]
pub(crate) fn ramp_mask(from: f64, to: f64, steps: usize, axis: Axis) -> String {
    const N: usize = 4;
    let thresholds = bayer_thresholds(N);
    let mut rects = String::new();
    for i in 0..steps {
        let tone = from + (to - from) * (i as f64 + 0.5) / steps as f64;
        for j in 0..N {
            let (x, y) = match axis {
                Axis::X => (i, j),
                Axis::Y => (j, i),
            };
            if tone > f64::from(thresholds[(y % N) * N + (x % N)]) {
                rects.push_str(&rect(x, y, 1));
            }
        }
    }
    match axis {
        Axis::X => data_url(steps, N, &rects),
        Axis::Y => data_url(N, steps, &rects),
    }
}

/// CSS `url()` of a `size` by `size` bitmap (row-major, non-zero is lit), one `<rect>` per
/// horizontal run of lit cells.
#[must_use]
pub(crate) fn bits_mask(bits: &[u8], size: usize) -> String {
    let mut rects = String::new();
    for y in 0..size {
        let mut x = 0;
        while x < size {
            if bits[y * size + x] == 0 {
                x += 1;
                continue;
            }
            let start = x;
            while x < size && bits[y * size + x] != 0 {
                x += 1;
            }
            rects.push_str(&rect(start, y, x - start));
        }
    }
    data_url(size, size, &rects)
}

/// A `<rect>` of height 1 at `(x, y)`, `width` cells wide.
fn rect(x: usize, y: usize, width: usize) -> String {
    format!("<rect x='{x}' y='{y}' width='{width}' height='1'/>")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bayer_4_matches_the_classic_matrix() {
        assert_eq!(
            bayer_ranks(4),
            [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5]
        );
    }

    #[test]
    fn level_mask_lights_that_share_of_cells() {
        let lit = |level| level_mask(level).matches("%3Crect").count();
        assert_eq!(lit(0.0), 0);
        assert_eq!(lit(0.25), 4);
        assert_eq!(lit(0.5), 8);
        assert_eq!(lit(2.0), 16);
    }

    #[test]
    fn level_mask_is_a_css_url_without_raw_angle_brackets() {
        let mask = level_mask(0.5);
        assert!(mask.starts_with("url(\"data:image/svg+xml,%3Csvg"));
        assert!(!mask.contains('<') && !mask.contains('>') && !mask.contains('#'));
    }

    #[test]
    fn bayer_2_matches_the_classic_matrix() {
        assert_eq!(bayer_ranks(2), [0, 2, 3, 1]);
    }

    #[test]
    fn bayer_ranks_every_cell_once() {
        for n in [2, 4, 8] {
            let mut ranks = bayer_ranks(n);
            ranks.sort_unstable();
            assert_eq!(ranks, (0..n * n).collect::<Vec<_>>());
        }
    }

    #[test]
    fn thresholds_are_rank_plus_half_over_cells() {
        let thresholds = bayer_thresholds(4);
        assert_eq!(thresholds[0], 0.5 / 16.0);
        assert_eq!(thresholds[1], 8.5 / 16.0);
        assert_eq!(thresholds[15], 5.5 / 16.0);
    }

    #[test]
    fn level_zero_draws_nothing_and_level_one_draws_every_cell() {
        assert_eq!(level_mask(0.0).matches("%3Crect").count(), 0);
        assert_eq!(level_mask(1.0).matches("%3Crect").count(), 16);
        assert_eq!(level_mask(0.26).matches("%3Crect").count(), 4);
        assert_eq!(level_mask(-1.0).matches("%3Crect").count(), 0);
    }

    #[test]
    fn level_one_sixteenth_lights_the_rank_zero_cell_only() {
        assert!(level_mask(1.0 / 16.0).contains("x='0' y='0'"));
    }

    /// Cells lit in column (`Axis::X`) or row (`Axis::Y`) `index` of a mask.
    fn lit(mask: &str, axis: Axis, index: usize) -> usize {
        let name = match axis {
            Axis::X => "x='",
            Axis::Y => "y='",
        };
        mask.split("%3Crect ")
            .skip(1)
            .filter(|rect| {
                let at = rect
                    .split(name)
                    .nth(1)
                    .and_then(|rest| rest.split('\'').next())
                    .and_then(|digits| digits.parse::<usize>().ok());
                at == Some(index)
            })
            .count()
    }

    #[test]
    fn ramps_go_from_sparse_to_dense_along_the_axis() {
        let mask = ramp_mask(0.0, 1.0, 8, Axis::X);
        assert!(lit(&mask, Axis::X, 0) < lit(&mask, Axis::X, 7));
        assert_eq!(lit(&mask, Axis::X, 7), 4);
        assert!(lit(&mask, Axis::X, 0) <= 1);
    }

    #[test]
    fn bits_masks_merge_horizontal_runs() {
        let mask = bits_mask(&[1, 1, 0, 1], 2);
        assert!(mask.contains("width='2' height='1'"));
        assert_eq!(mask.matches("%3Crect").count(), 2);
    }

    /// The browser's `rampMask(.., 12, ..)` hashed with FNV-1a, so the mask is the same bytes.
    #[test]
    fn ramps_match_the_browser() {
        use crate::ui::hash::fnv1a;
        assert_eq!(fnv1a(&ramp_mask(0.0, 1.0, 12, Axis::X)), 2_402_834_565);
        assert_eq!(fnv1a(&ramp_mask(1.0, 0.0, 12, Axis::X)), 2_124_622_963);
        assert_eq!(fnv1a(&ramp_mask(0.0, 1.0, 12, Axis::Y)), 1_613_875_545);
    }

    /// `ui.css` holds the page-wide masks as literals; they must be what `level_mask` draws.
    #[test]
    fn the_stylesheet_masks_match_level_mask() {
        let css = include_str!("../../assets/css/ui.css");
        for (name, level) in [
            ("--l0", 0.0),
            ("--l1", 0.25),
            ("--l2", 0.5),
            ("--l3", 0.75),
            ("--l4", 1.0),
        ] {
            assert!(
                css.contains(&format!("{name}: {};", level_mask(level))),
                "{name}"
            );
        }
    }
}
