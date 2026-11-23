//! Simentor Supabase Auth: email/password sign-in, refresh-token rotation,
//! sign-out, and the `@bps.go.id` eligibility gate. The refresh token lives in
//! the OS vault; the access token exists only in caller memory.

use crate::{Error, Result, client, vault};

/// The vault entry holding the refresh token.
pub const REFRESH_ENTRY: &str = "refresh_token";
/// The exact domain eligible for managed AI.
pub const ELIGIBLE_DOMAIN: &str = "@bps.go.id";

/// Eligibility: a confirmed address whose normalized form ends with exactly
/// `@bps.go.id` (a subdomain is not the domain).
pub fn eligible(email: &str, confirmed: bool) -> bool {
    confirmed && email.trim().to_ascii_lowercase().ends_with(ELIGIBLE_DOMAIN)
}

/// Sign in and store the (rotated) refresh token. Returns the signed-in email.
pub fn login(email: &str, password: &str) -> Result<String> {
    let reply = client::post("/auth/v1/token?grant_type=password", &serde_json::json!({ "email": email.trim(), "password": password }), None)?;
    let user = reply.get("user").cloned().unwrap_or(serde_json::Value::Null);
    let confirmed = user.get("email_confirmed_at").and_then(|v| v.as_str()).is_some_and(|s| !s.is_empty());
    let address = user.get("email").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    if !eligible(&address, confirmed) {
        // Do not keep a session that cannot manage credentials anyway.
        let _ = vault::delete(REFRESH_ENTRY);
        return Err(Error::Eligibility("hanya akun @bps.go.id terverifikasi yang bisa masuk".into()));
    }
    store_refresh(&reply)?;
    Ok(address)
}

/// A fresh access token, refreshing (and rotating) the stored refresh token.
pub fn access_token() -> Result<String> {
    let Some(refresh) = vault::get(REFRESH_ENTRY)? else {
        return Err(Error::Auth("no stored session — sign in first".into()));
    };
    let reply = client::post("/auth/v1/token?grant_type=refresh_token", &serde_json::json!({ "refresh_token": refresh }), None)?;
    store_refresh(&reply)?;
    reply.get("access_token").and_then(|v| v.as_str()).map(str::to_string).ok_or_else(|| Error::Auth("reply carried no access token".into()))
}

/// Sign out: revoke the session best effort, then drop the stored refresh token
/// either way. The device keypair and ciphertext cache stay (see the plan).
pub fn logout() -> Result<()> {
    if let Ok(token) = access_token() {
        let _ = client::post("/auth/v1/logout", &serde_json::json!({}), Some(&token));
    }
    vault::delete(REFRESH_ENTRY)
}

/// Is a session currently stored? (Says nothing about its server-side validity.)
pub fn signed_in() -> bool {
    matches!(vault::get(REFRESH_ENTRY), Ok(Some(_)))
}

fn store_refresh(reply: &serde_json::Value) -> Result<()> {
    let refresh = reply.get("refresh_token").and_then(|v| v.as_str()).ok_or_else(|| Error::Auth("reply carried no refresh token".into()))?;
    vault::set(REFRESH_ENTRY, refresh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_confirmed_bps_addresses_are_eligible() {
        assert!(eligible("user@bps.go.id", true));
        assert!(eligible("User@BPS.GO.ID", true), "case-insensitive");
        assert!(!eligible("user@bps.go.id", false), "unconfirmed is rejected");
        assert!(!eligible("user@mail.bps.go.id", true), "a subdomain is not the domain");
        assert!(!eligible("user@bps.go.id.evil.com", true), "the suffix must be the domain");
        assert!(!eligible("user@gmail.com", true));
        assert!(!eligible("", true));
    }
}
