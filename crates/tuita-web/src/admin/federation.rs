//! Helpers of the federation tab.

use topcoat::context::Cx;
use tuita_server::dto::DomainBlockSeverity;

use crate::l10n::t;

/// A hex fingerprint in groups of `size`, so two of them are easy to compare by eye.
pub(crate) fn group_hex(hex: &str, size: usize) -> String {
    let chars: Vec<char> = hex.chars().collect();
    chars
        .chunks(size)
        .map(|group| group.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn severity_label(cx: &Cx, severity: DomainBlockSeverity) -> String {
    match severity {
        DomainBlockSeverity::Suspend => t!(admin_fed_suspend),
        DomainBlockSeverity::Silence => t!(admin_fed_silence),
    }
    .resolve(cx)
    .into_owned()
}

/// The two things a block can do, mildest first: `(value, label)`.
pub(crate) fn severity_options(cx: &Cx) -> Vec<(&'static str, String)> {
    [DomainBlockSeverity::Silence, DomainBlockSeverity::Suspend]
        .into_iter()
        .map(|severity| (severity.as_str(), severity_label(cx, severity)))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_fingerprint_splits_into_groups_of_eight() {
        assert_eq!(
            group_hex("0123456789abcdef0123456789abcdef", 8),
            "01234567 89abcdef 01234567 89abcdef"
        );
    }

    #[test]
    fn there_is_no_trailing_space_and_short_input_stays_whole() {
        assert_eq!(group_hex("0123456789abcdef", 8), "01234567 89abcdef");
        assert_eq!(group_hex("abc", 8), "abc");
        assert_eq!(group_hex("", 8), "");
    }
}
