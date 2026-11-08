//! Provider settings for Studio. `pgs` does not read or write DesignCraft's
//! prefs.json, so without this a configured endpoint is lost on restart.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_LOCAL_BASE_URL: &str = "http://localhost:11434/v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProviderSettings {
    pub provider: String,
    pub groq_key: String,
    pub groq_model: String,
    pub custom_base_url: String,
    pub custom_key: String,
    pub custom_model: String,
    pub local_base_url: String,
    pub local_model: String,
}

impl Default for ProviderSettings {
    fn default() -> Self {
        Self {
            provider: "groq".into(),
            groq_key: String::new(),
            groq_model: "llama-3.3-70b-versatile".into(),
            custom_base_url: String::new(),
            custom_key: String::new(),
            custom_model: String::new(),
            local_base_url: DEFAULT_LOCAL_BASE_URL.into(),
            local_model: String::new(),
        }
    }
}

impl ProviderSettings {
    /// Platform config dir from environment variables (no new dependency);
    /// falls back to the current dir when none is set.
    pub fn path() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("pubgen-studio").join("settings.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::path())
    }

    /// A missing or malformed file is not an error: the user gets defaults and
    /// configures a provider in the setup dialog.
    pub fn load_from(path: &Path) -> Self {
        std::fs::read(path).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::path())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("settings dir: {e}"))?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(|e| format!("settings encode: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("settings write: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_without_a_key_in_the_default_file() {
        let dir = std::env::temp_dir().join(format!("pgs-settings-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("settings.json");
        let s = ProviderSettings { provider: "local".into(), local_model: "qwen2.5:7b".into(), ..Default::default() };
        s.save_to(&path).expect("save");
        let back = ProviderSettings::load_from(&path);
        assert_eq!(back.provider, "local");
        assert_eq!(back.local_model, "qwen2.5:7b");
        assert_eq!(back.local_base_url, DEFAULT_LOCAL_BASE_URL);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_missing_settings_file_yields_defaults() {
        let path = std::env::temp_dir().join(format!("pgs-absent-{}.json", std::process::id()));
        let s = ProviderSettings::load_from(&path);
        assert_eq!(s.provider, "groq");
        assert!(!path.exists(), "loading must not create the file");
    }
}
