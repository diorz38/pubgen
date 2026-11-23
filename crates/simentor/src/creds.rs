//! Service-credential fetch, local ciphertext cache and decrypt-at-use.
//!
//! The cache file holds ciphertext + metadata only (ciphertext is useless on
//! another device); plaintext exists only in the caller's memory, produced by
//! [`decrypt_service`] right before use and dropped after. The cache survives
//! sign-out and works offline, matching the server plan's compromise model.

use std::path::Path;

use serde_json::Value;

use crate::{Error, Result, auth, client, crypto, enroll, vault};

/// The service name of the managed Groq translation credential.
pub const SERVICE_GROQ: &str = "groq";
/// The service name of the managed Cloudflare cover credential.
pub const SERVICE_CLOUDFLARE: &str = "cloudflare";
/// The cache file name inside the app settings directory.
pub const CACHE_FILE: &str = "simentor_credentials.json";

/// One cached credential row: ciphertext plus the metadata the UI shows.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CachedCred {
    pub service_name: String,
    pub credential_name: String,
    pub ciphertext: String,
    pub cipher_alg: String,
    pub key_version: i64,
    pub cached_at: i64,
}

impl Default for CachedCred {
    fn default() -> Self {
        Self {
            service_name: String::new(),
            credential_name: String::new(),
            ciphertext: String::new(),
            cipher_alg: String::new(),
            key_version: 1,
            cached_at: 0,
        }
    }
}

/// Fetch the caller's credential ciphertext for their active device and replace
/// the local cache. Returns how many credentials were stored.
pub fn fetch_and_cache(settings_dir: &Path) -> Result<usize> {
    let token = auth::access_token()?;
    let device_id = vault::get(enroll::DEVICE_ID_ENTRY)?.unwrap_or_default();
    let reply = client::post("/functions/v1/service-credentials", &serde_json::json!({ "action": "fetch", "device_id": device_id }), Some(&token))?;
    let list = reply.get("credentials").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let creds: Vec<CachedCred> = list
        .into_iter()
        .map(|row| {
            let get_str = |k: &str| row.get(k).and_then(|v| v.as_str()).unwrap_or_default().to_string();
            CachedCred {
                service_name: get_str("service_name"),
                credential_name: get_str("credential_name"),
                ciphertext: get_str("ciphertext"),
                cipher_alg: get_str("cipher_alg"),
                key_version: row.get("key_version").and_then(|v| v.as_i64()).unwrap_or(1),
                cached_at: now,
            }
        })
        .filter(|c| !c.service_name.is_empty() && !c.ciphertext.is_empty())
        .collect();
    if creds.is_empty() {
        return Err(Error::Cache("the server returned no credentials for this device — ask the admin to grant one".into()));
    }
    let doc = serde_json::json!({ "version": 1, "credentials": creds });
    let path = settings_dir.join(CACHE_FILE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::Cache(e.to_string()))?;
    }
    std::fs::write(&path, doc.to_string()).map_err(|e| Error::Cache(e.to_string()))?;
    Ok(creds.len())
}

/// The cached rows (cheap: no decrypt, no vault). Empty when no cache exists.
pub fn cached(settings_dir: &Path) -> Vec<CachedCred> {
    read_cache(settings_dir).unwrap_or_default()
}

/// Which services have a cached credential — the per-frame readiness check.
pub fn cached_services(settings_dir: &Path) -> Vec<String> {
    cached(settings_dir).into_iter().map(|c| c.service_name).collect()
}

/// The managed Groq key, decrypted in memory for immediate use only.
pub fn groq_key(settings_dir: &Path) -> Result<String> {
    let plain = decrypt_service(settings_dir, SERVICE_GROQ)?;
    let key = String::from_utf8(plain).map_err(|_| Error::Crypto("credential is not valid UTF-8".into()))?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(Error::Cache("the stored groq credential is empty".into()));
    }
    Ok(key)
}

/// The managed Cloudflare credentials (account id, api token), decrypted in
/// memory for immediate use only.
pub fn cloudflare(settings_dir: &Path) -> Result<(String, String)> {
    let plain = decrypt_service(settings_dir, SERVICE_CLOUDFLARE)?;
    let text = String::from_utf8(plain).map_err(|_| Error::Crypto("credential is not valid UTF-8".into()))?;
    parse_cloudflare_plaintext(&text)
}

/// The Cloudflare credential plaintext format: JSON `{"account_id", "api_token"}`
/// (camelCase accepted). A bare token cannot identify the account, so it is an
/// error that names the expected shape for the issuing admin.
pub fn parse_cloudflare_plaintext(text: &str) -> Result<(String, String)> {
    let v: Value =
        serde_json::from_str(text.trim()).map_err(|_| Error::Crypto("expected {\"account_id\",\"api_token\"} JSON from the issuer".into()))?;
    let field =
        |names: &[&str]| names.iter().find_map(|n| v.get(n).and_then(|f| f.as_str()).map(str::trim)).filter(|s| !s.is_empty()).map(str::to_string);
    let account = field(&["account_id", "accountId"]);
    let token = field(&["api_token", "apiToken", "token"]);
    match (account, token) {
        (Some(a), Some(t)) => Ok((a, t)),
        _ => Err(Error::Crypto("the cloudflare credential JSON needs non-empty account_id and api_token".into())),
    }
}

fn read_cache(settings_dir: &Path) -> Result<Vec<CachedCred>> {
    let text = std::fs::read_to_string(settings_dir.join(CACHE_FILE)).map_err(|e| Error::Cache(e.to_string()))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| Error::Cache(e.to_string()))?;
    let list = v.get("credentials").and_then(|c| c.as_array()).cloned().unwrap_or_default();
    Ok(list
        .into_iter()
        .filter_map(|row| serde_json::from_value::<CachedCred>(row).ok())
        .filter(|c| !c.service_name.is_empty() && !c.ciphertext.is_empty())
        .collect())
}

fn decrypt_service(settings_dir: &Path, service: &str) -> Result<Vec<u8>> {
    let rows = read_cache(settings_dir)?;
    let row = rows
        .iter()
        .find(|c| c.service_name.eq_ignore_ascii_case(service))
        .ok_or_else(|| Error::Cache(format!("no cached '{service}' credential — sign in and fetch first")))?;
    let kp = enroll::keypair()?;
    let sealed = decode(row.ciphertext.trim())?;
    crypto::open(&kp, &sealed)
}

fn decode(text: &str) -> Result<Vec<u8>> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.decode(text).map_err(|e| Error::Crypto(format!("ciphertext is not valid base64: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("simentor-creds-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn cloudflare_plaintext_accepts_snake_and_camel_and_rejects_wrong_shapes() {
        assert_eq!(parse_cloudflare_plaintext(r#"{"account_id":"acct","api_token":"tok"}"#).unwrap(), ("acct".into(), "tok".into()));
        assert_eq!(parse_cloudflare_plaintext(r#"{"accountId":"a","apiToken":"t"}"#).unwrap(), ("a".into(), "t".into()));
        assert!(parse_cloudflare_plaintext("bare-token").is_err());
        assert!(parse_cloudflare_plaintext(r#"{"account_id":"","api_token":"t"}"#).is_err());
    }

    #[test]
    fn cache_round_trip_lists_services_without_decrypting() {
        let d = dir();
        let doc = serde_json::json!({
            "version": 1,
            "credentials": [
                {"serviceName": "groq", "credentialName": "shared", "ciphertext": "AA", "cipherAlg": "libsodium-sealed-box", "keyVersion": 1, "cachedAt": 7},
                {"serviceName": "", "ciphertext": "BB"},
                {"serviceName": "cloudflare"}
            ]
        });
        std::fs::write(d.join(CACHE_FILE), doc.to_string()).unwrap();
        assert_eq!(cached_services(&d), vec!["groq".to_string()]);
        // Missing cache behaves like an empty one, never as an error to the UI.
        assert!(cached_services(&d.join("nope")).is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }
}
