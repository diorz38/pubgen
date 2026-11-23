//! Service-credential fetch, local ciphertext cache and decrypt-at-use.
//!
//! Handoff §4.2 (`fetch_mine`) and §6: the cache holds ciphertext + metadata
//! only, encrypted at rest with a key kept in the OS vault, and plaintext exists
//! in caller memory just long enough to use it. Ciphertext survives sign-out,
//! unregister and revocation, and works offline — that is the documented
//! compromise model: only rotating the upstream key invalidates it.

use std::path::Path;

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use serde_json::Value;

use crate::{Error, Result, auth, client, crypto, enroll, vault};

/// The service name of the managed Groq translation credential.
pub const SERVICE_GROQ: &str = "groq";
/// The service name of the managed Cloudflare cover credential.
pub const SERVICE_CLOUDFLARE: &str = "cloudflare";
/// The cache file name inside the app settings directory.
pub const CACHE_FILE: &str = "simentor_credentials.json";
/// The vault entry holding the cache's own encryption key (32 random bytes).
pub const CACHE_KEY_ENTRY: &str = "cache_key";

/// One cached credential row: ciphertext plus the metadata the UI shows.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CachedCred {
    pub service_name: String,
    pub credential_name: String,
    pub ciphertext: String,
    pub cipher_alg: String,
    pub key_version: i64,
    pub cached_at: i64,
}

/// Fetch the caller's credential ciphertext for this active device and replace
/// the local cache. Returns how many credentials were stored.
pub fn fetch_and_cache(settings_dir: &Path) -> Result<usize> {
    let token = auth::access_token()?;
    let device_id = enroll::device_id()?.ok_or_else(|| Error::NotFound("this device is not registered — sign in again".into()))?;
    let reply =
        client::post("/functions/v1/service-credentials", &serde_json::json!({ "action": "fetch_mine", "device_id": device_id }), Some(&token))?;
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
        .filter(|c| !c.service_name.is_empty() && !c.ciphertext.is_empty() && c.cipher_alg.eq_ignore_ascii_case(crate::KEY_ALG))
        .collect();
    if creds.is_empty() {
        // Handoff §4.1: enrolment grants nothing; a grant is a separate admin step.
        return Err(Error::Cache("no credential is addressed to this device yet — ask the admin to issue one in Simentor".into()));
    }
    write_cache(settings_dir, &creds)?;
    Ok(creds.len())
}

/// The cached rows (cheap: no decrypt). Empty when no cache exists.
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

/// Any cached credential, decrypted for an explicit user action (handoff §8.4:
/// `Copy` is enabled only after a successful decrypt). Callers must not persist
/// the bytes.
pub fn decrypt_any(settings_dir: &Path, service: &str) -> Result<Vec<u8>> {
    decrypt_service(settings_dir, service)
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

// ---------------------------------------------------------------------------
// Cache at rest: AEAD with a key held in the OS vault (handoff §6)
// ---------------------------------------------------------------------------

/// Encrypt and store the rows. Legacy plaintext caches from an earlier build are
/// overwritten in the same step, so the upgrade needs no user action.
fn write_cache(settings_dir: &Path, creds: &[CachedCred]) -> Result<()> {
    let doc = serde_json::json!({ "version": 2, "credentials": creds }).to_string();
    let sealed = seal_local(doc.as_bytes())?;
    let path = settings_dir.join(CACHE_FILE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::Cache(e.to_string()))?;
    }
    std::fs::write(&path, sealed.to_string()).map_err(|e| Error::Cache(e.to_string()))?;
    Ok(())
}

fn read_cache(settings_dir: &Path) -> Result<Vec<CachedCred>> {
    let text = std::fs::read_to_string(settings_dir.join(CACHE_FILE)).map_err(|e| Error::Cache(e.to_string()))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| Error::Cache(e.to_string()))?;
    let doc = if v.get("version").and_then(|n| n.as_u64()) == Some(2) {
        let plain = open_local(&v)?;
        serde_json::from_slice::<Value>(&plain).map_err(|e| Error::Cache(e.to_string()))?
    } else {
        v // a pre-encryption cache: ciphertext only, re-encrypted on the next fetch
    };
    let list = doc.get("credentials").and_then(|c| c.as_array()).cloned().unwrap_or_default();
    Ok(list
        .into_iter()
        .filter_map(|row| serde_json::from_value::<CachedCred>(row).ok())
        .filter(|c| !c.service_name.is_empty() && !c.ciphertext.is_empty())
        .collect())
}

/// AEAD envelope: `{"version":2,"nonce":b64,"data":b64}`, key in the vault.
fn seal_local(plain: &[u8]) -> Result<Value> {
    let key = cache_key()?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let mut nonce_bytes = [0u8; 12];
    getrandom::fill(&mut nonce_bytes).map_err(|e| Error::Vault(format!("no random source: {e}")))?;
    let data = cipher.encrypt(Nonce::from_slice(&nonce_bytes), plain).map_err(|_| Error::Cache("cannot encrypt the cache".into()))?;
    Ok(serde_json::json!({ "version": 2, "nonce": crate::crypto_b64(&nonce_bytes), "data": crate::crypto_b64(&data) }))
}

fn open_local(v: &Value) -> Result<Vec<u8>> {
    let field = |k: &str| v.get(k).and_then(|f| f.as_str()).unwrap_or_default().to_string();
    let nonce = decode(&field("nonce"))?;
    let data = decode(&field("data"))?;
    if nonce.len() != 12 {
        return Err(Error::Cache("the cache nonce has the wrong length".into()));
    }
    let key = cache_key()?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    cipher.decrypt(Nonce::from_slice(&nonce), data.as_slice()).map_err(|_| Error::Cache("cannot decrypt the cache on this device".into()))
}

/// The cache key: 32 random bytes in the vault, created on first use.
fn cache_key() -> Result<[u8; 32]> {
    if let Some(stored) = vault::get(CACHE_KEY_ENTRY)? {
        let raw = decode(&stored)?;
        return raw.try_into().map_err(|_| Error::Vault("the stored cache key has the wrong length".into()));
    }
    let mut key = [0u8; 32];
    getrandom::fill(&mut key).map_err(|e| Error::Vault(format!("no random source: {e}")))?;
    vault::set(CACHE_KEY_ENTRY, &crate::crypto_b64(&key))?;
    Ok(key)
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
    base64::engine::general_purpose::STANDARD.decode(text).map_err(|e| Error::Crypto(format!("not valid base64: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder of this test's own: two tests run in parallel and each removes
    /// what it made, so the pid alone would let them delete each other's files.
    fn dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("simentor-creds-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn row(service: &str) -> CachedCred {
        CachedCred {
            service_name: service.into(),
            credential_name: "shared".into(),
            ciphertext: "AA".into(),
            cipher_alg: crate::KEY_ALG.into(),
            key_version: 1,
            cached_at: 7,
        }
    }

    /// The cache on disk must not contain the ciphertext or the metadata in
    /// clear text (handoff §6), and must round-trip through the vault key.
    #[cfg(windows)]
    #[test]
    fn cache_is_encrypted_at_rest_and_round_trips() {
        let d = dir("encrypted");
        write_cache(&d, &[row("groq"), row("cloudflare")]).unwrap();
        let raw = std::fs::read_to_string(d.join(CACHE_FILE)).unwrap();
        assert!(!raw.contains("groq"), "service name must not be plaintext: {raw}");
        assert!(!raw.contains("AA"), "ciphertext must not be plaintext: {raw}");
        assert_eq!(cached_services(&d), vec!["groq".to_string(), "cloudflare".to_string()]);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// A cache file copied from another Windows account cannot be opened.
    #[cfg(windows)]
    #[test]
    fn a_cache_written_with_another_key_does_not_open() {
        let d = dir("other-key");
        write_cache(&d, &[row("groq")]).unwrap();
        let mut text: Value = serde_json::from_str(&std::fs::read_to_string(d.join(CACHE_FILE)).unwrap()).unwrap();
        text["data"] = serde_json::json!(crate::crypto_b64(&[0u8; 32]));
        std::fs::write(d.join(CACHE_FILE), text.to_string()).unwrap();
        assert!(read_cache(&d).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn cloudflare_plaintext_accepts_snake_and_camel_and_rejects_wrong_shapes() {
        assert_eq!(parse_cloudflare_plaintext(r#"{"account_id":"acct","api_token":"tok"}"#).unwrap(), ("acct".into(), "tok".into()));
        assert_eq!(parse_cloudflare_plaintext(r#"{"accountId":"a","apiToken":"t"}"#).unwrap(), ("a".into(), "t".into()));
        assert!(parse_cloudflare_plaintext("bare-token").is_err());
        assert!(parse_cloudflare_plaintext(r#"{"account_id":"","api_token":"t"}"#).is_err());
    }

    #[test]
    fn a_legacy_plaintext_cache_still_lists_its_services() {
        let d = dir("legacy");
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
