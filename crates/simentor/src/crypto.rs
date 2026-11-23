//! The device keypair and libsodium sealed boxes
//! (`crypto_box_seal`: X25519 + XSalsa20-Poly1305, dryoc's libsodium-compatible
//! wire format — the same bytes the Simentor issuance side produces).
//!
//! The private key is generated on this device and never leaves
//! [`crate::vault`]; only the public key is uploaded at enrollment.
//! Sealed boxes do not authenticate the sender — authorization and audit live
//! in the issuance API, not in the ciphertext.

use base64::Engine as _;
use dryoc::types::Bytes as _;

use crate::{Error, Result};

/// The public + secret key bytes of a Curve25519 keypair, 32 bytes each.
#[derive(Clone)]
pub struct DeviceKeypair {
    pub public: Vec<u8>,
    pub secret: Vec<u8>,
}

impl DeviceKeypair {
    /// A fresh random keypair (first run on a new device).
    pub fn generate() -> Result<Self> {
        let kp = dryoc::dryocbox::StackKeyPair::generate();
        Ok(Self { public: kp.public_key.as_slice().to_vec(), secret: kp.secret_key.as_slice().to_vec() })
    }

    /// Decode the vault representation: base64(public ‖ secret).
    pub fn from_stored(stored: &str) -> Result<Self> {
        let raw = decode(stored)?;
        if raw.len() != 64 {
            return Err(Error::Crypto("stored device key has the wrong length".into()));
        }
        Ok(Self { public: raw[..32].to_vec(), secret: raw[32..].to_vec() })
    }

    /// The vault representation: base64(public ‖ secret).
    pub fn to_stored(&self) -> String {
        let mut raw = self.public.clone();
        raw.extend_from_slice(&self.secret);
        encode(&raw)
    }
}

/// Seal `message` to `public` (tests and local checks; issuance normally
/// happens in the Simentor web admin).
pub fn seal(public: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    // `from_slices` validates the 32-byte length and yields the stack public key
    // `seal_to_vecbox` wants; the zero secret never participates in sealing.
    let kp = dryoc::dryocbox::StackKeyPair::from_slices(public, &[0u8; 32]).map_err(|e| Error::Crypto(format!("bad public key: {e}")))?;
    let boxed = dryoc::dryocbox::DryocBox::seal_to_vecbox(message, &kp.public_key).map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(boxed.to_vec())
}

/// Open a sealed box with the device keypair (libsodium `crypto_box_seal_open`).
/// The wire format is `ephemeral public key ‖ tag ‖ ciphertext`.
pub fn open(keypair: &DeviceKeypair, sealed: &[u8]) -> Result<Vec<u8>> {
    let kp =
        dryoc::dryocbox::StackKeyPair::from_slices(&keypair.public, &keypair.secret).map_err(|e| Error::Crypto(format!("bad device key: {e}")))?;
    let boxed = dryoc::dryocbox::DryocBox::from_sealed_bytes(sealed).map_err(|e| Error::Crypto(e.to_string()))?;
    boxed.open_to_vec(&kp).map_err(|e| Error::Crypto(e.to_string()))
}

fn encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn decode(text: &str) -> Result<Vec<u8>> {
    base64::engine::general_purpose::STANDARD.decode(text.trim()).map_err(|e| Error::Crypto(format!("invalid base64: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealed_box_round_trips_through_the_stored_form() {
        let kp = DeviceKeypair::generate().unwrap();
        let restored = DeviceKeypair::from_stored(&kp.to_stored()).unwrap();
        let sealed = seal(&restored.public, b"groq-key-123").unwrap();
        // A sealed box carries the ephemeral public key first (48 + message bytes).
        assert!(sealed.len() > 48);
        assert_eq!(open(&restored, &sealed).unwrap(), b"groq-key-123");
    }

    #[test]
    fn a_sealed_box_for_another_device_does_not_open() {
        let a = DeviceKeypair::generate().unwrap();
        let b = DeviceKeypair::generate().unwrap();
        let sealed = seal(&a.public, b"secret").unwrap();
        assert!(open(&b, &sealed).is_err(), "wrong device must not decrypt");
    }

    #[test]
    fn stored_keys_reject_wrong_lengths() {
        assert!(DeviceKeypair::from_stored(&encode(&[0u8; 32])).is_err());
        assert!(DeviceKeypair::from_stored("not base64 !!").is_err());
    }
}
