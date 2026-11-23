//! The Simentor project endpoint: `SIMENTOR_URL` + `SIMENTOR_ANON_KEY` from the
//! process env, falling back to a local `.env` (same lookup order as the Cloudflare
//! and quota credentials in `crates/imagegen`). The anon key is public by design;
//! no secret key is ever read here.

use crate::{Error, Result};

/// The configured project URL (no trailing slash) and anon key.
pub fn endpoints() -> Result<(String, String)> {
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
    if url.is_empty() || key.is_empty() {
        return Err(Error::NotConfigured);
    }
    while url.ends_with('/') {
        url.pop();
    }
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(Error::NotConfigured);
    }
    Ok((url, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_configuration_is_an_error_not_a_guess() {
        // Only the env lookup is probed here; the ambient `.env` may exist on a
        // developer machine and legitimately provide the values.
        let from_env = std::env::var("SIMENTOR_URL").is_ok() && std::env::var("SIMENTOR_ANON_KEY").is_ok();
        let r = endpoints();
        assert!(from_env || r.is_err(), "without env config: {r:?}");
    }
}
