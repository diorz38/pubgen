//! The Simentor project endpoint the handoff §2 defines: `appsettings.json` beside
//! the executable,
//!
//! ```json
//! { "SupabaseUrl": "https://<ref>.supabase.co", "PublishableKey": "sb_publishable_..." }
//! ```
//!
//! with a developer fallback to `SIMENTOR_URL` / `SIMENTOR_ANON_KEY` (process env,
//! then `.env`) so a source checkout runs without shipping a config file. Only the
//! publishable/anon key is ever read: the desktop never holds a `service_role` key.
//! HTTPS is required; plain HTTP is accepted for loopback only (local stacks).

use crate::{Error, Result};

/// The config file name, beside the executable (and the working directory).
pub const SETTINGS_FILE: &str = "appsettings.json";

/// The configured project URL (no trailing slash) and publishable key.
pub fn endpoints() -> Result<(String, String)> {
    let (mut url, mut key) = from_settings_files().unwrap_or_default();
    if url.is_empty() || key.is_empty() {
        let (env_url, env_key) = from_env();
        url = if url.is_empty() { env_url } else { url };
        key = if key.is_empty() { env_key } else { key };
    }
    if url.is_empty() || key.is_empty() {
        return Err(Error::NotConfigured);
    }
    while url.ends_with('/') {
        url.pop();
    }
    if !url.starts_with("https://") && !is_loopback_http(&url) {
        return Err(Error::NotConfigured);
    }
    Ok((url, key))
}

/// `appsettings.json` from the executable's folder, then the working directory.
fn from_settings_files() -> Option<(String, String)> {
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        dirs.push(dir.to_path_buf());
    }
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd);
    }
    dirs.into_iter().find_map(|dir| {
        let text = std::fs::read_to_string(dir.join(SETTINGS_FILE)).ok()?;
        parse_settings(&text)
    })
}

/// `SupabaseUrl` + `PublishableKey`, also accepting the snake_case spellings and the
/// legacy `AnonKey` name. Both must be non-empty or the file is no configuration.
pub fn parse_settings(text: &str) -> Option<(String, String)> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let field =
        |names: &[&str]| names.iter().find_map(|n| v.get(n).and_then(|f| f.as_str())).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let url = field(&["SupabaseUrl", "supabaseUrl", "supabase_url", "Url"])?;
    let key = field(&["PublishableKey", "publishableKey", "publishable_key", "AnonKey", "anonKey"])?;
    Some((url, key))
}

fn from_env() -> (String, String) {
    let mut url = std::env::var("SIMENTOR_URL").unwrap_or_default();
    let mut key = std::env::var("SIMENTOR_ANON_KEY").unwrap_or_default();
    if (url.is_empty() || key.is_empty())
        && let Ok(content) = std::fs::read_to_string(".env")
    {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim().trim_matches('"').trim_matches('\'').trim();
                if url.is_empty() && k.eq_ignore_ascii_case("SIMENTOR_URL") {
                    url = v.to_string();
                } else if key.is_empty() && k.eq_ignore_ascii_case("SIMENTOR_ANON_KEY") {
                    key = v.to_string();
                }
            }
        }
    }
    (url.trim().to_string(), key.trim().to_string())
}

fn is_loopback_http(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("http://") else { return false };
    let host = rest.split(['/', ':']).next().unwrap_or_default();
    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_file_accepts_the_documented_and_legacy_names() {
        let (u, k) = parse_settings(r#"{"SupabaseUrl":"https://abc.supabase.co","PublishableKey":"sb_publishable_x"}"#).unwrap();
        assert_eq!(u, "https://abc.supabase.co");
        assert_eq!(k, "sb_publishable_x");
        assert!(parse_settings(r#"{"supabase_url":"https://abc.supabase.co","anonKey":"anon"}"#).is_some());
        // Half a configuration is no configuration: it must not silently pass.
        assert!(parse_settings(r#"{"SupabaseUrl":"https://abc.supabase.co"}"#).is_none());
        assert!(parse_settings(r#"{"SupabaseUrl":"","PublishableKey":"k"}"#).is_none());
        assert!(parse_settings("not json").is_none());
    }

    #[test]
    fn https_is_required_except_on_loopback() {
        assert!(is_loopback_http("http://localhost:54321"));
        assert!(is_loopback_http("http://127.0.0.1:54321"));
        assert!(!is_loopback_http("http://abc.supabase.co"), "plain HTTP off loopback is refused");
        assert!(!is_loopback_http("https://abc.supabase.co"));
    }

    #[test]
    fn missing_configuration_is_an_error_not_a_guess() {
        // Only probed when this checkout has no config of its own.
        if endpoints().is_ok() {
            return;
        }
        assert!(matches!(endpoints(), Err(Error::NotConfigured)));
    }
}
