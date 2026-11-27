//! Small helpers every tab shares.

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

/// `at` as an RFC 3339 string, the form `fmt::rel_time` and `fmt::full_time` read.
pub(crate) fn stamp(at: OffsetDateTime) -> String {
    at.format(&Rfc3339).unwrap_or_default()
}

/// The API's message of a failed call.
pub(crate) fn message(error: crate::api::ApiError) -> String {
    error.message
}

/// `tr!(cx, key, var: value)`: the text of a message as an owned `String`, for props that take
/// `&str`.
macro_rules! tr {
    ($cx:expr, $($message:tt)*) => {
        $crate::l10n::t!($($message)*).resolve($cx).into_owned()
    };
}
pub(crate) use tr;
