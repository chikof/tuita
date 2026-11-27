//! Seeded hashing and a seeded PRNG. Every generated picture derives
//! from these two, so the output must stay bit-identical to the browser's copy.

/// 32-bit FNV-1a over the UTF-16 code units of `input`, as JavaScript's `charCodeAt` sees them.
#[must_use]
pub(crate) fn fnv1a(input: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for unit in input.encode_utf16() {
        hash ^= u32::from(unit);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// The mulberry32 generator: floats in `[0, 1)` from a 32-bit seed.
#[derive(Clone, Debug)]
pub(crate) struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    #[must_use]
    pub(crate) const fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// The next float in `[0, 1)`.
    pub(crate) fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6d2b_79f5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fnv1a_matches_the_known_vectors() {
        assert_eq!(fnv1a(""), 0x811c_9dc5);
        assert_eq!(fnv1a("a"), 0xe40c_292c);
    }

    #[test]
    fn fnv1a_hashes_utf16_code_units() {
        // The browser's `charCodeAt` walks surrogate pairs as two units; `chars()` would not.
        assert_eq!(fnv1a("héllo😀"), 4_113_148_722);
    }

    #[test]
    fn mulberry32_matches_the_browser() {
        let mut rnd = Mulberry32::new(fnv1a("ada"));
        let drawn = [rnd.next_f64(), rnd.next_f64(), rnd.next_f64()];
        assert_eq!(
            drawn,
            [
                0.984_111_188_678_070_9,
                0.951_921_328_669_413_9,
                0.364_809_552_207_589_15
            ]
        );
    }
}
