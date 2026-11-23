//! Same-signature stubs for builds without the `online` feature (and for wasm):
//! every entry point reports "unavailable", so callers can code one path and
//! never silently fall back to another provider.

use crate::{Error, Result};

pub mod auth {
    use super::*;

    pub fn eligible(_email: &str, _confirmed: bool) -> bool {
        false
    }
    pub fn login(_email: &str, _password: &str) -> Result<String> {
        Err(Error::Unavailable)
    }
    pub fn access_token() -> Result<String> {
        Err(Error::Unavailable)
    }
    pub fn logout() -> Result<()> {
        Err(Error::Unavailable)
    }
    pub fn signed_in() -> bool {
        false
    }
}

pub mod enroll {
    use super::*;

    /// Placeholder keypair type: never constructible without the online feature.
    #[derive(Clone)]
    pub struct DeviceKeypair {
        pub _no_online_feature: (),
    }

    #[derive(Clone, Debug, Default)]
    pub struct Registration {
        pub device_id: String,
        pub active_device_count: Option<i64>,
        pub max_active_devices: Option<i64>,
    }

    #[derive(Clone, Debug, Default)]
    pub struct Device {
        pub id: String,
        pub device_label: String,
        pub revoked_at: Option<String>,
    }

    impl Device {
        pub fn active(&self) -> bool {
            false
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct DeviceList {
        pub devices: Vec<Device>,
        pub active_device_count: Option<i64>,
        pub max_active_devices: Option<i64>,
    }

    pub fn keypair() -> Result<DeviceKeypair> {
        Err(Error::Unavailable)
    }
    pub fn register(_device_label: &str, _app_version: &str) -> Result<Registration> {
        Err(Error::Unavailable)
    }
    pub fn list_mine() -> Result<DeviceList> {
        Err(Error::Unavailable)
    }
    pub fn device_id() -> Result<Option<String>> {
        Ok(None)
    }
    pub fn unregister(_device_id: Option<&str>) -> Result<()> {
        Err(Error::Unavailable)
    }
}

pub mod creds {
    use super::*;
    use std::path::Path;

    pub const SERVICE_GROQ: &str = "groq";
    pub const SERVICE_CLOUDFLARE: &str = "cloudflare";
    pub const CACHE_FILE: &str = "simentor_credentials.json";

    /// Placeholder row type: the stub cache is always empty.
    #[derive(Clone, Debug, Default)]
    pub struct CachedCred {
        pub service_name: String,
        pub credential_name: String,
    }

    pub fn fetch_and_cache(_settings_dir: &Path) -> Result<usize> {
        Err(Error::Unavailable)
    }
    pub fn cached(_settings_dir: &Path) -> Vec<CachedCred> {
        Vec::new()
    }
    pub fn cached_services(_settings_dir: &Path) -> Vec<String> {
        Vec::new()
    }
    pub fn decrypt_any(_settings_dir: &Path, _service: &str) -> Result<Vec<u8>> {
        Err(Error::Unavailable)
    }
    pub fn groq_key(_settings_dir: &Path) -> Result<String> {
        Err(Error::Unavailable)
    }
    pub fn cloudflare(_settings_dir: &Path) -> Result<(String, String)> {
        Err(Error::Unavailable)
    }
}
