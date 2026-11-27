//! How a count reads in a chart axis, a stat card or a counting number.

use topcoat::context::Cx;

use crate::fmt;

/// A number format a component can name without holding a closure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum NumberFormat {
    /// The digits as they are (`1234`).
    #[default]
    Plain,
    /// The language's compact form (`1.2K`).
    Compact,
}

impl NumberFormat {
    /// `n` in this format, in the language of the request.
    #[must_use]
    pub(crate) fn apply(self, cx: &Cx, n: u64) -> String {
        match self {
            Self::Plain => n.to_string(),
            Self::Compact => fmt::compact(cx, i64::try_from(n).unwrap_or(i64::MAX)),
        }
    }

    /// The `data-format` value `dither.js` reads.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Compact => "compact",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::context;

    #[test]
    fn plain_keeps_the_digits_and_compact_shortens_them() {
        let cx = context("en");
        assert_eq!(NumberFormat::Plain.apply(&cx, 1234), "1234");
        assert_eq!(NumberFormat::Compact.apply(&cx, 1234), "1.2K");
        assert_eq!(NumberFormat::Compact.apply(&cx, 7), "7");
    }
}
