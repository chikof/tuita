//! RSA-2048 keys for HTTP signatures: generation, PEM in and out, signing, verifying.
//!
//! Private-key operations use `rsa` 0.10, which blinds them (RUSTSEC-2023-0071 is fixed in that
//! release). Generating a key takes up to a few seconds in release builds, so callers run
//! [`generate_keypair`] and [`PrivateKey::sign_sha256`] on a blocking thread.

use std::fmt;

use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs8::{
    DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding,
};
use rsa::sha2::{Digest, Sha256};
use rsa::traits::PublicKeyParts;
use thiserror::Error;

/// Modulus size of generated keys.
pub const KEY_BITS: usize = 2048;

/// Smallest modulus accepted from a remote server.
const MIN_REMOTE_BITS: usize = 1024;
/// Largest modulus accepted from a remote server; bigger keys make verification a CPU sink.
const MAX_REMOTE_BITS: usize = 8192;

/// Why a key could not be made, parsed or used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum KeyError {
    /// Key generation failed.
    #[error("key generation failed")]
    Generate,
    /// The text is not an RSA key of an accepted size.
    #[error("not a usable RSA key")]
    Parse,
    /// The key could not be encoded.
    #[error("key encoding failed")]
    Encode,
    /// The private-key operation failed.
    #[error("signing failed")]
    Sign,
}

/// A freshly generated key pair, as stored: PKCS#8 PEM for the private half and SPKI PEM for the
/// public half.
#[derive(Clone)]
pub struct KeyPair {
    /// `-----BEGIN PUBLIC KEY-----` text, published in the actor's `publicKey`.
    pub public_pem: String,
    /// `-----BEGIN PRIVATE KEY-----` text. Never leaves the server.
    pub private_pem: String,
}

impl fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyPair").finish_non_exhaustive()
    }
}

/// Generates an RSA-2048 key pair. CPU heavy: call it from a blocking thread.
///
/// # Errors
///
/// [`KeyError::Generate`] when the random source fails, [`KeyError::Encode`] when the PEM cannot
/// be written.
pub fn generate_keypair() -> Result<KeyPair, KeyError> {
    let mut rng = rand::rng();
    let private = rsa::PrivateKey::new(&mut rng, KEY_BITS).map_err(|_| KeyError::Generate)?;
    let private_pem = private
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|_| KeyError::Encode)?
        .to_string();
    let public_pem = private
        .to_public_key()
        .to_public_key_pem(LineEnding::LF)
        .map_err(|_| KeyError::Encode)?;
    Ok(KeyPair {
        public_pem,
        private_pem,
    })
}

/// A private signing key.
#[derive(Clone)]
pub struct PrivateKey(rsa::PrivateKey);

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKey").finish_non_exhaustive()
    }
}

impl PrivateKey {
    /// Parses a PKCS#8 PEM private key.
    ///
    /// # Errors
    ///
    /// [`KeyError::Parse`] when the text is not a PKCS#8 RSA key.
    pub fn from_pem(pem: &str) -> Result<Self, KeyError> {
        rsa::PrivateKey::from_pkcs8_pem(pem)
            .map(Self)
            .map_err(|_| KeyError::Parse)
    }

    /// The matching public key.
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.to_public_key())
    }

    /// RSASSA-PKCS1-v1_5 over the SHA-256 of `message`.
    ///
    /// # Errors
    ///
    /// [`KeyError::Sign`] when the private-key operation fails.
    pub fn sign_sha256(&self, message: &[u8]) -> Result<Vec<u8>, KeyError> {
        self.0
            .pkcs1v15_sign::<Sha256>(message)
            .map_err(|_| KeyError::Sign)
    }
}

/// A public verification key.
#[derive(Clone)]
pub struct PublicKey(rsa::PublicKey);

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublicKey").finish_non_exhaustive()
    }
}

impl PublicKey {
    /// Parses an SPKI (`BEGIN PUBLIC KEY`) or PKCS#1 (`BEGIN RSA PUBLIC KEY`) PEM key of 1024 to
    /// 8192 bits.
    ///
    /// # Errors
    ///
    /// [`KeyError::Parse`] for anything else.
    pub fn from_pem(pem: &str) -> Result<Self, KeyError> {
        let key = rsa::PublicKey::from_public_key_pem(pem)
            .or_else(|_| rsa::PublicKey::from_pkcs1_pem(pem))
            .map_err(|_| KeyError::Parse)?;
        let bits = key.size().saturating_mul(8);
        if (MIN_REMOTE_BITS..=MAX_REMOTE_BITS).contains(&bits) {
            Ok(Self(key))
        } else {
            Err(KeyError::Parse)
        }
    }

    /// Whether `signature` is a valid RSASSA-PKCS1-v1_5 SHA-256 signature of `message`.
    #[must_use]
    pub fn verify_sha256(&self, message: &[u8], signature: &[u8]) -> bool {
        self.0.pkcs1v15_verify::<Sha256>(message, signature).is_ok()
    }

    /// SHA-256 of the DER encoding, as lower-case hex: the fingerprint admins compare.
    ///
    /// # Errors
    ///
    /// [`KeyError::Encode`] when the key cannot be encoded.
    pub fn fingerprint(&self) -> Result<String, KeyError> {
        let der = self.0.to_public_key_der().map_err(|_| KeyError::Encode)?;
        let digest = Sha256::digest(der.as_bytes());
        let mut hex = String::with_capacity(digest.len() * 2);
        for byte in digest {
            hex.push(char::from(HEX[usize::from(byte >> 4)]));
            hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Ok(hex)
    }
}

const HEX: &[u8; 16] = b"0123456789abcdef";

/// The fingerprint of a public key in PEM form.
///
/// # Errors
///
/// [`KeyError::Parse`] when `pem` is not a usable key.
pub fn fingerprint(public_pem: &str) -> Result<String, KeyError> {
    PublicKey::from_pem(public_pem)?
        .fingerprint()
        .map_err(|_| KeyError::Encode)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_generated_pair_signs_and_verifies() {
        let pair = generate_keypair().expect("generate");
        assert!(pair.private_pem.starts_with("-----BEGIN PRIVATE KEY-----"));
        assert!(pair.public_pem.starts_with("-----BEGIN PUBLIC KEY-----"));
        let private = PrivateKey::from_pem(&pair.private_pem).expect("private pem");
        let public = PublicKey::from_pem(&pair.public_pem).expect("public pem");
        let signature = private.sign_sha256(b"hello").expect("sign");
        assert_eq!(signature.len(), 256);
        assert!(public.verify_sha256(b"hello", &signature));
        assert!(!public.verify_sha256(b"hellO", &signature));
        assert_eq!(
            fingerprint(&pair.public_pem).expect("fingerprint").len(),
            64
        );
    }

    #[test]
    fn junk_is_not_a_key() {
        assert_eq!(
            PublicKey::from_pem("nope").expect_err("junk"),
            KeyError::Parse
        );
        assert_eq!(
            PrivateKey::from_pem("nope").expect_err("junk"),
            KeyError::Parse
        );
    }
}
