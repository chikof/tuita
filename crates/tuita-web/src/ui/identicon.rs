//! Generated avatar bitmaps.
//!
//! A seed becomes a mirrored 8 by 8 grid, per-cell tones and a dithered bitmap of lit pixels.
//! Everything is deterministic and matches the browser's copy bit for bit.

use crate::ui::{
    hash::{Mulberry32, fnv1a},
    mask::bayer_thresholds,
};

/// Mirrored left-half grid (row-major, `size` by `size`) from an FNV-1a hash of `seed`.
#[must_use]
pub(crate) fn identicon(seed: &str, size: usize) -> Vec<bool> {
    let mut rnd = Mulberry32::new(fnv1a(seed));
    let half = size.div_ceil(2);
    let mut cells = vec![false; size * size];
    for y in 0..size {
        for x in 0..half {
            let on = rnd.next_f64() < 0.52;
            cells[y * size + x] = on;
            cells[y * size + (size - 1 - x)] = on;
        }
    }
    cells
}

/// Per-cell Bayer levels for the same grid: lit cells are dense, with a soft top-lit gradient.
#[must_use]
pub(crate) fn avatar_levels(seed: &str, size: usize) -> Vec<f64> {
    let mut rnd = Mulberry32::new(fnv1a(&format!("{seed}:shade")));
    let half = size.div_ceil(2);
    let mut shade = vec![0.0; size * size];
    for y in 0..size {
        for x in 0..half {
            let value = rnd.next_f64();
            shade[y * size + x] = value;
            shade[y * size + (size - 1 - x)] = value;
        }
    }
    identicon(seed, size)
        .into_iter()
        .enumerate()
        .map(|(i, on)| {
            let y = (i / size) as f64 / (size.saturating_sub(1).max(1)) as f64;
            if on {
                0.55 + 0.4 * shade[i] - 0.15 * y
            } else {
                0.04 + 0.16 * shade[i]
            }
        })
        .collect()
}

/// The identicon as lit pixels (`1`) and dark ones (`0`), row-major: each cell is `scale` by
/// `scale` pixels, dithered with Bayer-4.
#[must_use]
pub(crate) fn avatar_bits(seed: &str, size: usize, scale: usize) -> Vec<u8> {
    let levels = avatar_levels(seed, size);
    let thresholds = bayer_thresholds(4);
    let px = size * scale;
    let mut bits = vec![0; px * px];
    for y in 0..px {
        for x in 0..px {
            let level = levels[(y / scale) * size + (x / scale)];
            let threshold = f64::from(thresholds[(y % 4) * 4 + (x % 4)]);
            bits[y * px + x] = u8::from(level > threshold);
        }
    }
    bits
}

#[cfg(test)]
mod test {
    use super::*;

    /// A `0`/`1` string of the grid, as the browser's `identicon` prints it.
    fn grid(seed: &str) -> String {
        identicon(seed, 8)
            .iter()
            .map(|on| if *on { '1' } else { '0' })
            .collect()
    }

    fn pixels(seed: &str) -> String {
        avatar_bits(seed, 8, 2)
            .iter()
            .map(|bit| char::from(b'0' + bit))
            .collect()
    }

    #[test]
    fn grid_is_mirrored_deterministic_and_seed_dependent() {
        let size = 8;
        let cells = identicon("ada", size);
        assert_eq!(cells.len(), size * size);
        for y in 0..size {
            for x in 0..size {
                assert_eq!(cells[y * size + x], cells[y * size + (size - 1 - x)]);
            }
        }
        assert_eq!(identicon("ada", size), cells);
        assert_ne!(identicon("bob", size), cells);
    }

    #[test]
    fn levels_are_one_per_cell_in_unit_range() {
        let levels = avatar_levels("ada", 8);
        assert_eq!(levels.len(), 64);
        assert!(levels.iter().all(|level| (0.0..=1.0).contains(level)));
    }

    /// Reference vectors for the bitmap generator.
    #[test]
    fn grids_match_the_browser() {
        assert_eq!(
            grid("ada"),
            "0011110011011011110110111111111101000010101001010101101001100110"
        );
        assert_eq!(
            grid(""),
            "0101101001011010001111001110011100011000000110001110011100111100"
        );
        assert_eq!(
            grid("héllo😀"),
            "0001100000011000000110001001100110100101001001000110011010111101"
        );
    }

    #[test]
    fn levels_match_the_browser() {
        let levels = avatar_levels("ada", 8);
        assert_eq!(
            levels[..4],
            [
                0.128_621_197_678_148_75,
                0.100_425_118_394_196_04,
                0.847_282_131_295_651_3,
                0.752_559_983_544_051_7,
            ]
        );
    }

    #[test]
    fn pixels_match_the_browser() {
        assert_eq!(
            pixels("ada"),
            "1000111111111000000011010101000010110011101010110101000101000101111110111100111101010001010001011010111110111010010101010111010110111010100011000001000000000100101011100011001101000100000100011010101010001010000100010100010000111010001010000001010000010100"
        );
        assert_eq!(
            pixels("a"),
            "1011111010101100001101000001110000101110001100000000010000010000100011111111100000001101110100000000001000100010000000000000000011101010101010111100010000010001001111000011100000010100000101001010111111111110010101010101010100110010100011100001000100000100"
        );
    }
}
