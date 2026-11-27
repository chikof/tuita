//! Local account handles: the sign-up rule.
//!
//! A handle is 3 to 15 ASCII letters, digits or underscores, and not reserved. It keeps the case
//! it was typed in. Uniqueness is case-insensitive and belongs to the database.
//!
//! ```
//! use tuita_core::handle::{HandleError, parse_handle};
//!
//! assert_eq!(parse_handle("Ana_99")?.as_str(), "Ana_99");
//! assert_eq!(parse_handle("admin"), Err(HandleError::Reserved));
//! assert_eq!(parse_handle("a b"), Err(HandleError::InvalidChars));
//! # Ok::<(), HandleError>(())
//! ```

use std::fmt;

use crate::wordfilter::{FilterConfig, is_reserved_handle};

/// Fewest characters in a handle.
pub const MIN_LEN: usize = 3;
/// Most characters in a handle.
pub const MAX_LEN: usize = 15;

/// A valid, unreserved handle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle(Box<str>);

// 16: a Box<str> (pointer + length); the null pointer is the niche for Option.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Handle>() == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Option<Handle>>() == size_of::<Handle>());

impl Handle {
    /// The handle as typed, without a leading `@`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Handle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Why a handle was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum HandleError {
    /// Fewer than [`MIN_LEN`] characters.
    #[error("handle must be at least 3 characters")]
    TooShort,
    /// More than [`MAX_LEN`] characters.
    #[error("handle must be at most 15 characters")]
    TooLong,
    /// A character other than an ASCII letter, digit or underscore.
    #[error("handle may only use letters, numbers and underscores")]
    InvalidChars,
    /// A system word such as `admin`, or a `deleted_` name.
    #[error("handle is reserved")]
    Reserved,
}

// 1: a fieldless enum.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<HandleError>() == 1);

impl HandleError {
    /// The sentence shown to the user, for API responses.
    #[must_use]
    pub const fn user_message(self) -> &'static str {
        match self {
            Self::TooShort => "Handle must be at least 3 characters.",
            Self::TooLong => "Handle must be at most 15 characters.",
            Self::InvalidChars => "Handles use 3–15 letters, numbers or underscores.",
            Self::Reserved => "That handle is reserved. Please pick another.",
        }
    }
}

/// Validates a handle typed by a user. Surrounding whitespace is trimmed first.
///
/// Admin-reserved words and the word filter are separate checks
/// ([`is_reserved_handle`] with the admin config, and
/// [`find_blocked`](crate::wordfilter::find_blocked) with [`Target::Handle`](crate::wordfilter::Target::Handle)).
///
/// # Errors
///
/// Returns the first rule the handle breaks: length, characters, then reserved words.
pub fn parse_handle(input: &str) -> Result<Handle, HandleError> {
    let handle = input.trim();
    let len = handle.chars().count();
    if len < MIN_LEN {
        return Err(HandleError::TooShort);
    }
    if len > MAX_LEN {
        return Err(HandleError::TooLong);
    }
    if !handle
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(HandleError::InvalidChars);
    }
    let deleted = handle
        .get(..8)
        .is_some_and(|p| p.eq_ignore_ascii_case("deleted_"));
    if deleted || is_reserved_handle(handle, &FilterConfig::default()) {
        return Err(HandleError::Reserved);
    }
    Ok(Handle(handle.into()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn accepts_letters_digits_underscores() {
        for ok in ["abc", "Ana_99", "_under_", "123", "a1b2c3d4e5f6g7h"] {
            assert_eq!(
                parse_handle(ok).map(|h| h.to_string()),
                Ok(ok.to_owned()),
                "{ok}"
            );
        }
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            parse_handle("  ana  ").map(|h| h.to_string()),
            Ok("ana".to_owned())
        );
    }

    #[test]
    fn rejects_short_and_long() {
        assert_eq!(parse_handle(""), Err(HandleError::TooShort));
        assert_eq!(parse_handle("ab"), Err(HandleError::TooShort));
        assert_eq!(parse_handle("éé"), Err(HandleError::TooShort));
        assert_eq!(parse_handle("a234567890123456"), Err(HandleError::TooLong));
        assert!(parse_handle("a23456789012345").is_ok());
    }

    #[test]
    fn rejects_other_characters() {
        for bad in [
            "a b",
            "ana!",
            "ana-b",
            "ana.b",
            "añá",
            "@ana",
            "ana@x.org",
            "日本語",
        ] {
            assert_eq!(parse_handle(bad), Err(HandleError::InvalidChars), "{bad}");
        }
    }

    #[test]
    fn rejects_reserved() {
        for bad in [
            "admin",
            "Admin",
            "ADMIN_01",
            "a_d_m_i_n",
            "support",
            "tuita",
            "Tuita1",
            "deleted_5",
            "DELETED_x",
        ] {
            assert_eq!(parse_handle(bad), Err(HandleError::Reserved), "{bad}");
        }
        assert!(parse_handle("deleted").is_err(), "reserved word");
        assert!(parse_handle("administrate").is_ok());
        assert!(parse_handle("bossy").is_ok());
    }

    #[test]
    fn checks_length_before_characters() {
        assert_eq!(parse_handle("a!"), Err(HandleError::TooShort));
        assert_eq!(parse_handle("this_is_too_long!"), Err(HandleError::TooLong));
    }

    #[test]
    fn keeps_case() {
        assert_eq!(
            parse_handle("MiXeD").map(|h| h.as_str().to_owned()),
            Ok("MiXeD".to_owned())
        );
    }

    #[test]
    fn messages() {
        assert_eq!(
            HandleError::InvalidChars.user_message(),
            "Handles use 3–15 letters, numbers or underscores."
        );
        assert_eq!(HandleError::Reserved.to_string(), "handle is reserved");
    }
}
