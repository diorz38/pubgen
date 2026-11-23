//! The crate error type. Display strings reach the setup dialogs, so they must
//! never embed a secret (key, token, ciphertext) — only status codes and the
//! server's own `message`/`error` field, truncated.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The Simentor project URL / anon key are not configured (env or `.env`).
    #[error("Simentor is not configured: set SIMENTOR_URL and SIMENTOR_ANON_KEY")]
    NotConfigured,
    /// Not compiled in (no `online` feature, or a wasm build).
    #[error("Simentor-managed AI is unavailable in this build")]
    Unavailable,
    /// Login failed, token rejected, or sign-out is required first.
    #[error("Sign-in required: {0}")]
    Auth(String),
    /// The account is not eligible (unconfirmed email, wrong domain).
    #[error("Not eligible: {0}")]
    Eligibility(String),
    /// The 2-active-device limit was hit.
    #[error("Maksimal 2 perangkat aktif. Cabut perangkat lama di Simentor/Desktop Credential Manager.")]
    DeviceLimit,
    /// The OS credential vault is missing, locked or unwritable.
    #[error("Secure storage error: {0}")]
    Vault(String),
    /// A sealed box could not be opened (wrong device key or corrupted data).
    #[error("Cannot decrypt the credential on this device: {0}")]
    Crypto(String),
    /// The local ciphertext cache is missing or unreadable.
    #[error("No cached Simentor credential: {0}")]
    Cache(String),
    /// Any other network or server failure, with the HTTP status for context.
    #[error("Simentor request failed (HTTP {status}): {message}")]
    Server { status: u16, message: String },
}

impl Error {
    /// A server reply as an error: prefers the JSON `message`/`error`/`msg` field,
    /// else a short body excerpt. Secrets never appear in these fields by contract
    /// (the server plan forbids logging or returning them).
    pub fn from_status(status: u16, body: &str) -> Self {
        let message = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| ["message", "error", "msg"].iter().find_map(|k| v.get(k).and_then(|m| m.as_str()).map(str::to_string)))
            .unwrap_or_else(|| {
                let cut = body.char_indices().take(160).last().map(|(i, c)| i + c.len_utf8()).unwrap_or(0);
                body.get(..cut).unwrap_or_default().trim().to_string()
            });
        Error::Server { status, message }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_error_extracts_the_message_field_and_truncates_long_bodies() {
        let e = Error::from_status(500, r#"{"message":"boom"}"#);
        assert!(e.to_string().contains("boom"), "{e}");
        let long = "x".repeat(500);
        let e = Error::from_status(502, &long);
        let Error::Server { message, .. } = &e else { panic!("variant") };
        assert!(message.chars().count() <= 160, "{} chars", message.chars().count());
    }

    #[test]
    fn server_error_cuts_at_a_char_boundary() {
        // 3-byte characters: a naive byte slice at 160 would split one.
        let body: String = "é".repeat(100);
        let e = Error::from_status(500, &body);
        let Error::Server { message, .. } = &e else { panic!("variant") };
        assert!(message.chars().all(|c| c == 'é'), "no broken char: {message:?}");
    }
}
