//! Shape checks for end-to-end encrypted direct messages.
//!
//! The browser makes a P-256 key pair, seals the private half under a passphrase (the *backup*)
//! and seals each message to the other person's public key. The server never opens any of it: it
//! only checks that what it stores has the right shape, and compares the key parts of a sealed
//! message with the keys it holds.
//!
//! | Value | Shape |
//! |---|---|
//! | public key | exactly 87 base64url characters that decode to 65 bytes starting `0x04` |
//! | backup | 1 to 2048 characters, `v1.<iterations>.<salt>.<iv>.<sealed>` with 6 to 8 digits of iterations |
//! | sealed message | at most 16 KiB, `v1.<sender key>.<recipient key>.<salt>.<iv>.<ciphertext>` |
//!
//! ```
//! use tuita_core::sealed::{SealedError, validate_public_key, validate_sealed};
//!
//! let key = format!("BE{}A", "Q".repeat(84));
//! validate_public_key(&key)?;
//! assert_eq!(validate_public_key("not a key"), Err(SealedError::PublicKey));
//!
//! let sealed = format!("v1.{key}.{key}.c2FsdA.aXY.Y2lwaGVy");
//! let keys = validate_sealed(&sealed)?;
//! assert_eq!((keys.sender, keys.recipient), (key.as_str(), key.as_str()));
//! # Ok::<(), SealedError>(())
//! ```

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use thiserror::Error;

/// Characters of a public key: 65 bytes in unpadded base64url.
const PUBLIC_KEY_CHARS: usize = 87;
/// Bytes of an uncompressed P-256 point.
const PUBLIC_KEY_BYTES: usize = 65;
/// First byte of an uncompressed point.
const UNCOMPRESSED: u8 = 0x04;
/// Most characters of a backup.
const MAX_BACKUP_CHARS: usize = 2048;
/// Most bytes of a sealed message.
const MAX_SEALED_BYTES: usize = 16 * 1024;
/// The format version every value starts with.
const VERSION: &str = "v1";

/// Why a value is not a key, a backup or a sealed message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum SealedError {
    /// Not a 65-byte uncompressed P-256 point in 87 base64url characters.
    #[error("not a message public key")]
    PublicKey,
    /// Not a `v1` passphrase-sealed key of at most 2048 characters.
    #[error("not a sealed key backup")]
    Backup,
    /// Not a `v1` sealed message of at most 16 KiB.
    #[error("not a sealed message")]
    Ciphertext,
}

// 1: a fieldless enum is one byte; it is returned by value on the send path.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SealedError>() == 1);

/// The two public keys a sealed message names, borrowed from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct SealedKeys<'a> {
    /// The sender's public key.
    pub sender: &'a str,
    /// The recipient's public key.
    pub recipient: &'a str,
}

fn is_url_safe(part: &str) -> bool {
    part.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// A non-empty run of base64url characters.
fn is_chunk(part: &str) -> bool {
    !part.is_empty() && is_url_safe(part)
}

/// Checks `key` is a message public key.
///
/// # Errors
///
/// [`SealedError::PublicKey`] unless `key` is exactly 87 base64url characters that decode to
/// 65 bytes, the first one `0x04`.
pub fn validate_public_key(key: &str) -> Result<(), SealedError> {
    if key.len() != PUBLIC_KEY_CHARS || !is_url_safe(key) {
        return Err(SealedError::PublicKey);
    }
    match URL_SAFE_NO_PAD.decode(key) {
        Ok(bytes) if bytes.len() == PUBLIC_KEY_BYTES && bytes[0] == UNCOMPRESSED => Ok(()),
        _ => Err(SealedError::PublicKey),
    }
}

/// Checks `backup` is a passphrase-sealed private key.
///
/// # Errors
///
/// [`SealedError::Backup`] unless `backup` is at most 2048 characters of
/// `v1.<6 to 8 digits>.<salt>.<iv>.<sealed>`, the last three base64url.
pub fn validate_backup(backup: &str) -> Result<(), SealedError> {
    let mut parts = backup.split('.');
    let shaped = backup.len() <= MAX_BACKUP_CHARS
        && parts.next() == Some(VERSION)
        && parts
            .next()
            .is_some_and(|n| (6..=8).contains(&n.len()) && n.bytes().all(|b| b.is_ascii_digit()))
        && parts.by_ref().take(3).filter(|part| is_chunk(part)).count() == 3
        && parts.next().is_none();
    if shaped {
        Ok(())
    } else {
        Err(SealedError::Backup)
    }
}

/// Checks `sealed` is a sealed message and returns the two keys it names. Whether those are
/// the accounts' current keys is for the caller to decide.
///
/// # Errors
///
/// [`SealedError::Ciphertext`] unless `sealed` is at most 16 KiB of
/// `v1.<sender key>.<recipient key>.<salt>.<iv>.<ciphertext>`, every part 1 to 16384 base64url
/// characters.
pub fn validate_sealed(sealed: &str) -> Result<SealedKeys<'_>, SealedError> {
    if sealed.len() > MAX_SEALED_BYTES {
        return Err(SealedError::Ciphertext);
    }
    let mut parts = sealed.split('.');
    let version = parts.next();
    let (Some(sender), Some(recipient), Some(salt), Some(iv), Some(body), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(SealedError::Ciphertext);
    };
    if version != Some(VERSION)
        || ![sender, recipient, salt, iv, body]
            .into_iter()
            .all(is_chunk)
    {
        return Err(SealedError::Ciphertext);
    }
    Ok(SealedKeys { sender, recipient })
}

#[cfg(test)]
mod test {
    use super::*;

    fn key(fill: char) -> String {
        format!("BE{}A", fill.to_string().repeat(84))
    }

    #[test]
    fn a_public_key_is_87_characters_of_a_65_byte_point() {
        assert_eq!(validate_public_key(&key('Q')), Ok(()));
        assert_eq!(validate_public_key(&key('-')), Ok(()));
    }

    #[test]
    fn a_public_key_of_the_wrong_shape_is_refused() {
        let bad = [
            String::new(),
            "BE".to_owned(),
            format!("{}A", key('Q')),
            key('Q').replacen('B', "C", 1),
            format!("BE{}=", "Q".repeat(84)),
            format!("BE{}+", "Q".repeat(84)),
            format!("BE{}B", "Q".repeat(84)),
        ];
        for value in bad {
            assert_eq!(
                validate_public_key(&value),
                Err(SealedError::PublicKey),
                "{value}"
            );
        }
    }

    #[test]
    fn a_backup_has_the_iteration_count_and_three_chunks() {
        for good in [
            "v1.600000.c2FsdA.aXY.c2VhbGVk",
            "v1.10000000.a-_.b.c",
            "v1.123456.a.b.c",
        ] {
            assert_eq!(validate_backup(good), Ok(()), "{good}");
        }
    }

    #[test]
    fn a_backup_of_the_wrong_shape_is_refused() {
        let too_long = format!("v1.600000.a.b.{}", "c".repeat(2048));
        let bad = [
            "",
            "v1",
            "v2.600000.a.b.c",
            "v1.60000.a.b.c",
            "v1.123456789.a.b.c",
            "v1.6x0000.a.b.c",
            "v1.600000.a.b",
            "v1.600000.a.b.c.d",
            "v1.600000..b.c",
            "v1.600000.a.b.c+",
            "v1.600000.a.b.c ",
            too_long.as_str(),
        ];
        for value in bad {
            assert_eq!(validate_backup(value), Err(SealedError::Backup), "{value}");
        }
    }

    #[test]
    fn a_sealed_message_names_its_two_keys() {
        let (sender, recipient) = (key('Q'), key('R'));
        let sealed = format!("v1.{sender}.{recipient}.c2FsdA.aXY.Y2lwaGVy");
        let keys = validate_sealed(&sealed).expect("valid");
        assert_eq!(keys.sender, sender);
        assert_eq!(keys.recipient, recipient);
    }

    #[test]
    fn a_sealed_message_of_the_wrong_shape_is_refused() {
        let k = key('Q');
        let too_long = format!("v1.{k}.{k}.a.b.{}", "c".repeat(16 * 1024));
        let bad = [
            String::new(),
            "v1".to_owned(),
            format!("v2.{k}.{k}.a.b.c"),
            format!("v1.{k}.{k}.a.b"),
            format!("v1.{k}.{k}.a.b.c.d"),
            format!("v1.{k}.{k}.a..c"),
            format!("v1.{k}.{k}.a.b.c="),
            format!("v1.{k}.{k}.a.b.é"),
            too_long,
        ];
        for value in bad {
            assert_eq!(
                validate_sealed(&value),
                Err(SealedError::Ciphertext),
                "{value}"
            );
        }
    }

    #[test]
    fn a_sealed_message_of_exactly_16_kib_passes() {
        let k = key('Q');
        let head = format!("v1.{k}.{k}.a.b.");
        let sealed = format!("{head}{}", "c".repeat(16 * 1024 - head.len()));
        assert_eq!(sealed.len(), 16 * 1024);
        assert!(validate_sealed(&sealed).is_ok());
        assert!(validate_sealed(&format!("{sealed}c")).is_err());
    }
}
