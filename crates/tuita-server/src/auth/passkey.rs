//! The WebAuthn relying party, built once from `PUBLIC_URL`.
//!
//! The relying party id is the host of `PUBLIC_URL` and the origin is `PUBLIC_URL` itself.
//! WebAuthn does not allow an IP address as relying party id, so an instance served on one has
//! no passkeys; `localhost` works.

use url::Url;
use webauthn_rs::prelude::{Webauthn, WebauthnBuilder};

/// The relying party: its id and the verifier built for it.
pub struct RelyingParty {
    rp_id: String,
    webauthn: Webauthn,
}

impl RelyingParty {
    /// Builds the relying party for `public_url`. `None` when its host is an IP address (or
    /// there is none), which WebAuthn cannot use.
    #[must_use]
    pub fn new(public_url: &str) -> Option<Self> {
        let origin = Url::parse(public_url).ok()?;
        let rp_id = origin.domain()?.to_owned();
        let webauthn = WebauthnBuilder::new(&rp_id, &origin)
            .ok()?
            .rp_name("tuita")
            .build()
            .ok()?;
        Some(Self { rp_id, webauthn })
    }

    /// The relying party id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.rp_id
    }

    /// The verifier.
    #[must_use]
    pub fn webauthn(&self) -> &Webauthn {
        &self.webauthn
    }
}

#[cfg(test)]
mod test {
    use super::RelyingParty;

    #[test]
    fn the_host_is_the_relying_party_id() {
        let rp = RelyingParty::new("https://social.example").expect("a domain works");
        assert_eq!(rp.id(), "social.example");
        let rp = RelyingParty::new("http://localhost:8081").expect("localhost works");
        assert_eq!(rp.id(), "localhost");
    }

    #[test]
    fn an_ip_address_has_no_passkeys() {
        assert!(RelyingParty::new("http://127.0.0.1:8080").is_none());
        assert!(RelyingParty::new("http://[::1]:8080").is_none());
    }
}
