//! Secrets at rest: AES-256-GCM sealing of the ActivityPub signing keys with `SECRET_KEY`.
//!
//! A sealed value is `enc:v1:<base64url nonce>:<base64url ciphertext>`. The nonce is 96 random
//! bits. The additional data is a context string naming the row (`users.private_key_pem:<id>`,
//! `instance_actor`), so a ciphertext copied to another row does not open. A stored value without
//! the `enc:` prefix is a plaintext row from before `SECRET_KEY` was set; [`open`] returns it as
//! it is, and [`crate::federation::keys::protect_at_rest`] seals it at startup.
//!
//! Rotating `SECRET_KEY` is not supported: a changed key makes every sealed row unreadable and
//! startup refuses to continue. Rotation would need a second key slot and a re-seal pass.

use std::fmt;

use aes_gcm::aead::{Aead, KeyInit, Nonce, Payload};
use aes_gcm::{Aes256Gcm, Key};
use base64::Engine as _;
use base64::alphabet;
use base64::engine::DecodePaddingMode;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig, URL_SAFE_NO_PAD};
use subtle::ConstantTimeEq as _;
use thiserror::Error;
use uuid::Uuid;

/// Marks a sealed value; a PEM never starts with it.
const PREFIX: &str = "enc:";
const VERSION_PREFIX: &str = "enc:v1:";
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

/// Context of the instance actor's private key.
pub(crate) const INSTANCE_CONTEXT: &str = "instance_actor";

/// Standard base64 where the trailing `=` is optional.
const LENIENT_STANDARD: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// `SECRET_KEY`: 32 bytes. [`fmt::Debug`] prints nothing of it and `==` runs in constant time.
#[derive(Clone)]
pub(crate) struct SecretKey([u8; KEY_LEN]);

impl SecretKey {
    /// Reads 64 hex characters or standard base64 of 32 bytes; `None` for anything else.
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let bytes = hex(raw).or_else(|| {
            let decoded = LENIENT_STANDARD.decode(raw).ok()?;
            <[u8; KEY_LEN]>::try_from(decoded.as_slice()).ok()
        })?;
        Some(Self(bytes))
    }
}

fn hex(raw: &str) -> Option<[u8; KEY_LEN]> {
    if raw.len() != KEY_LEN * 2 {
        return None;
    }
    let mut bytes = [0; KEY_LEN];
    let (pairs, _) = raw.as_bytes().as_chunks::<2>();
    for (byte, [high, low]) in bytes.iter_mut().zip(pairs) {
        let nibble = |digit: u8| {
            char::from(digit)
                .to_digit(16)
                .and_then(|d| u8::try_from(d).ok())
        };
        *byte = nibble(*high)? << 4 | nibble(*low)?;
    }
    Some(bytes)
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey(<redacted>)")
    }
}

impl PartialEq for SecretKey {
    fn eq(&self, other: &Self) -> bool {
        self.0[..].ct_eq(&other.0[..]).into()
    }
}

impl Eq for SecretKey {}

/// Why a stored value did not open. Every message names `SECRET_KEY` where it matters.
#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum SecretError {
    /// A sealed value and no key.
    #[error("a stored secret is encrypted but SECRET_KEY is not set")]
    MissingKey,
    /// Authentication failed.
    #[error(
        "cannot decrypt a stored secret: SECRET_KEY is not the key that sealed it, or the value was copied from another row"
    )]
    WrongKey,
    /// `enc:` without a parseable `v1` body.
    #[error("a stored secret starts with `enc:` but is not a valid `enc:v1` value")]
    Malformed,
}

/// The context of the private key of local account `user`.
pub(crate) fn user_context(user: Uuid) -> String {
    format!("users.private_key_pem:{user}")
}

/// Whether `stored` is a sealed value rather than plaintext.
pub(crate) fn is_sealed(stored: &str) -> bool {
    stored.starts_with(PREFIX)
}

fn cipher(key: &SecretKey) -> Aes256Gcm {
    Aes256Gcm::new(&Key::<Aes256Gcm>::from(key.0))
}

/// Seals `plaintext` for the row named by `context`.
pub(crate) fn seal(key: &SecretKey, context: &str, plaintext: &str) -> String {
    let nonce: [u8; NONCE_LEN] = rand::random();
    let sealed = cipher(key)
        .encrypt(
            &Nonce::<Aes256Gcm>::from(nonce),
            Payload {
                msg: plaintext.as_bytes(),
                aad: context.as_bytes(),
            },
        )
        .expect("a PEM is far below the AES-GCM length limit");
    format!(
        "{VERSION_PREFIX}{}:{}",
        URL_SAFE_NO_PAD.encode(nonce),
        URL_SAFE_NO_PAD.encode(sealed)
    )
}

/// Seals `plaintext` when a key is configured, else returns it unchanged.
pub(crate) fn store(key: Option<&SecretKey>, context: &str, plaintext: &str) -> String {
    key.map_or_else(|| plaintext.to_owned(), |key| seal(key, context, plaintext))
}

/// The plaintext of a stored value. A value that is not sealed passes through, which is how rows
/// written before `SECRET_KEY` was set keep working.
///
/// # Errors
///
/// [`SecretError::MissingKey`] for a sealed value and no key, [`SecretError::WrongKey`] when it
/// does not authenticate under `key` and `context`, [`SecretError::Malformed`] for a broken body.
pub(crate) fn open(
    key: Option<&SecretKey>,
    context: &str,
    stored: &str,
) -> Result<String, SecretError> {
    let Some(body) = stored.strip_prefix(VERSION_PREFIX) else {
        return if is_sealed(stored) {
            Err(SecretError::Malformed)
        } else {
            Ok(stored.to_owned())
        };
    };
    let key = key.ok_or(SecretError::MissingKey)?;
    let (nonce, sealed) = body.split_once(':').ok_or(SecretError::Malformed)?;
    let nonce = URL_SAFE_NO_PAD
        .decode(nonce)
        .map_err(|_| SecretError::Malformed)?;
    let sealed = URL_SAFE_NO_PAD
        .decode(sealed)
        .map_err(|_| SecretError::Malformed)?;
    let nonce =
        Nonce::<Aes256Gcm>::try_from(nonce.as_slice()).map_err(|_| SecretError::Malformed)?;
    let plain = cipher(key)
        .decrypt(
            &nonce,
            Payload {
                msg: &sealed,
                aad: context.as_bytes(),
            },
        )
        .map_err(|_| SecretError::WrongKey)?;
    String::from_utf8(plain).map_err(|_| SecretError::Malformed)
}

#[cfg(test)]
mod test {
    use super::*;

    const HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    fn key(byte: u8) -> SecretKey {
        SecretKey([byte; KEY_LEN])
    }

    #[test]
    fn parses_hex_and_base64() {
        let from_hex = SecretKey::parse(HEX).expect("hex");
        assert_eq!(from_hex.0[31], 0x1f);
        assert_eq!(
            SecretKey::parse(&format!(" {} \n", HEX.to_uppercase())),
            Some(from_hex)
        );

        let b64 = base64::engine::general_purpose::STANDARD.encode([7; KEY_LEN]);
        assert_eq!(SecretKey::parse(&b64), Some(key(7)));
        assert_eq!(SecretKey::parse(b64.trim_end_matches('=')), Some(key(7)));
    }

    #[test]
    fn rejects_other_lengths_and_junk() {
        assert_eq!(SecretKey::parse(""), None);
        assert_eq!(SecretKey::parse(&HEX[..62]), None);
        assert_eq!(SecretKey::parse(&format!("{HEX}00")), None);
        assert_eq!(SecretKey::parse(&HEX.replace('0', "g")), None);
        let short = base64::engine::general_purpose::STANDARD.encode([1; 16]);
        assert_eq!(SecretKey::parse(&short), None);
        let long = base64::engine::general_purpose::STANDARD.encode([1; 33]);
        assert_eq!(SecretKey::parse(&long), None);
    }

    #[test]
    fn debug_prints_nothing_of_the_key() {
        let shown = format!("{:?}", SecretKey::parse(HEX).expect("hex"));
        assert_eq!(shown, "SecretKey(<redacted>)");
    }

    #[test]
    fn round_trips() {
        let sealed = seal(&key(1), "ctx", "-----BEGIN PRIVATE KEY-----");
        assert!(sealed.starts_with("enc:v1:"), "{sealed}");
        assert!(!sealed.contains("BEGIN"), "{sealed}");
        assert_eq!(
            open(Some(&key(1)), "ctx", &sealed).as_deref(),
            Ok("-----BEGIN PRIVATE KEY-----")
        );
    }

    #[test]
    fn each_seal_uses_a_fresh_nonce() {
        assert_ne!(seal(&key(1), "ctx", "x"), seal(&key(1), "ctx", "x"));
    }

    #[test]
    fn a_wrong_key_or_context_does_not_open() {
        let sealed = seal(&key(1), "users.private_key_pem:a", "secret");
        assert_eq!(
            open(Some(&key(2)), "users.private_key_pem:a", &sealed),
            Err(SecretError::WrongKey)
        );
        assert_eq!(
            open(Some(&key(1)), "users.private_key_pem:b", &sealed),
            Err(SecretError::WrongKey)
        );
        assert_eq!(
            open(None, "users.private_key_pem:a", &sealed),
            Err(SecretError::MissingKey)
        );
    }

    #[test]
    fn plaintext_passes_through_with_or_without_a_key() {
        for key in [None, Some(key(1))] {
            assert_eq!(
                open(key.as_ref(), "ctx", "-----BEGIN PRIVATE KEY-----").as_deref(),
                Ok("-----BEGIN PRIVATE KEY-----")
            );
        }
    }

    #[test]
    fn broken_bodies_are_malformed_not_plaintext() {
        for stored in [
            "enc:",
            "enc:v2:a:b",
            "enc:v1:",
            "enc:v1:abc",
            "enc:v1:!!:!!",
            "enc:v1:AAAA:AAAA",
        ] {
            assert_ne!(
                open(Some(&key(1)), "ctx", stored),
                Ok(stored.to_owned()),
                "{stored}"
            );
        }
        assert_eq!(
            open(Some(&key(1)), "ctx", "enc:v2:a:b"),
            Err(SecretError::Malformed)
        );
        assert_eq!(
            open(Some(&key(1)), "ctx", "enc:v1:abc"),
            Err(SecretError::Malformed)
        );
    }

    #[test]
    fn store_seals_only_with_a_key() {
        assert_eq!(store(None, "ctx", "plain"), "plain");
        assert!(is_sealed(&store(Some(&key(1)), "ctx", "plain")));
    }
}
