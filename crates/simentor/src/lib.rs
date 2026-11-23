//! Simentor service credentials for Pubgen (docs/plans/2026-11-23-supabase-managed-ai.md,
//! desktop side; the server contract lives in `desktop-service-credentials-plan.md`).
//!
//! The desktop app signs in to the Simentor Supabase project with email/password,
//! enrolls this device with a locally generated libsodium keypair (the private key
//! never leaves Windows Credential Manager), fetches per-device encrypted service
//! credentials as ciphertext, caches the ciphertext next to the app settings and
//! decrypts it in memory only at use time. A copied cache is useless on another
//! device; revoking the device or the credential blocks future fetches only —
//! rotating the upstream provider key is the kill switch for already issued ones.
//!
//! Without the `online` feature (and on wasm) every entry point reports
//! "unavailable"; the rest of the app keeps working with its own providers.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod error;
pub use error::{Error, Result};

#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub mod auth;
#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub mod client;
#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub mod config;
#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub mod creds;
#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub mod crypto;
#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub mod enroll;
#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub mod vault;

#[cfg(not(all(feature = "online", not(target_arch = "wasm32"))))]
mod stubs;
#[cfg(not(all(feature = "online", not(target_arch = "wasm32"))))]
pub use stubs::{auth, creds, enroll};

/// Whether the native Simentor client is compiled in. Without it, the managed
/// provider must resolve to "unavailable", never to another provider.
pub fn online_available() -> bool {
    cfg!(all(feature = "online", not(target_arch = "wasm32")))
}

/// The sealed-box algorithm id sent at enrollment and expected on ciphertext rows.
pub const KEY_ALG: &str = "libsodium-sealed-box";

/// Standard base64 of raw bytes (the wire format of public keys and ciphertext).
#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub fn crypto_b64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
