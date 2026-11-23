//! Device enrollment against the `device-enroll` Edge Function (handoff §4.1):
//! `register`, `list_mine`, `unregister`. The server enforces the 2-active-device
//! cap and answers `409`; `active_device_count` is informational because it is
//! computed before the insert.
//!
//! Id-shape gotcha (handoff §4.1): `register` returns `device_id`, `list_mine`
//! returns `id`. [`list_mine`] normalizes both.

use crate::{KEY_ALG, Result, auth, client, crypto, vault};

/// The vault entry holding the device keypair (base64 public ‖ secret).
pub const KEYPAIR_ENTRY: &str = "device_keypair";
/// The vault entry holding this device's registered id (from the register reply).
pub const DEVICE_ID_ENTRY: &str = "device_id";

/// What `register` reported: the device id plus the slot state the UI shows.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Registration {
    pub device_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub approval_mode: String,
    pub active_device_count: Option<i64>,
    pub max_active_devices: Option<i64>,
}

/// One row of `list_mine`, normalized so `id` is the device id.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Device {
    #[serde(alias = "device_id")]
    pub id: String,
    #[serde(default)]
    pub device_label: String,
    #[serde(default)]
    pub status: String,
    pub approval_mode: Option<String>,
    pub approved_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: Option<String>,
}

impl Device {
    /// An unrevoked device still occupies one of the two slots.
    pub fn active(&self) -> bool {
        self.revoked_at.is_none() && !self.status.eq_ignore_ascii_case("revoked")
    }
}

/// The `list_mine` reply: the caller's devices plus the slot state.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceList {
    #[serde(default)]
    pub devices: Vec<Device>,
    pub active_device_count: Option<i64>,
    pub max_active_devices: Option<i64>,
}

/// This device's keypair, generated and stored on first use. Immutable for the
/// device lifetime: a new keypair means the old ciphertext can never be opened.
pub fn keypair() -> Result<crypto::DeviceKeypair> {
    if let Some(stored) = vault::get(KEYPAIR_ENTRY)? {
        return crypto::DeviceKeypair::from_stored(&stored);
    }
    let kp = crypto::DeviceKeypair::generate()?;
    vault::set(KEYPAIR_ENTRY, &kp.to_stored())?;
    Ok(kp)
}

/// `register` — enrol this device. Auto-approved for a verified `@bps.go.id`
/// login. The public key is uploaded; the secret never leaves the vault.
pub fn register(device_label: &str, app_version: &str) -> Result<Registration> {
    let label = device_label.trim();
    if label.is_empty() || label.chars().count() > 120 {
        return Err(crate::Error::BadRequest("device label must be 1–120 characters".into()));
    }
    let kp = keypair()?;
    let public = crate::crypto_b64(&kp.public);
    let token = auth::access_token()?;
    let reply = client::post(
        "/functions/v1/device-enroll",
        &serde_json::json!({
            "action": "register",
            "device_label": label,
            "public_key": public,
            "key_alg": KEY_ALG,
            "app_version": app_version,
        }),
        Some(&token),
    )?;
    let reg: Registration =
        serde_json::from_value(reply).map_err(|e| crate::Error::Server { status: 200, message: format!("unexpected register reply: {e}") })?;
    if reg.device_id.trim().is_empty() {
        return Err(crate::Error::Server { status: 200, message: "register reply carried no device_id".into() });
    }
    vault::set(DEVICE_ID_ENTRY, &reg.device_id)?;
    Ok(reg)
}

/// `list_mine` — the caller's own devices and slot state.
pub fn list_mine() -> Result<DeviceList> {
    let token = auth::access_token()?;
    let reply = client::post("/functions/v1/device-enroll", &serde_json::json!({ "action": "list_mine" }), Some(&token))?;
    serde_json::from_value(reply).map_err(|e| crate::Error::Server { status: 200, message: format!("unexpected list_mine reply: {e}") })
}

/// This device's stored id, if it was registered on this Windows account.
pub fn device_id() -> Result<Option<String>> {
    Ok(vault::get(DEVICE_ID_ENTRY)?.filter(|id| !id.trim().is_empty()))
}

/// `unregister` — soft-revoke a device, freeing a slot. Idempotent for an own
/// already-revoked device; another user's id answers `404`.
///
/// Per handoff §7 the local keypair and ciphertext cache are kept by default: the
/// device warning is the caller's to show, and re-enrolling is the recovery path.
pub fn unregister(target: Option<&str>) -> Result<()> {
    let token = auth::access_token()?;
    let id = match target {
        Some(id) if !id.trim().is_empty() => id.trim().to_string(),
        _ => device_id()?.ok_or_else(|| crate::Error::NotFound("this device is not registered".into()))?,
    };
    client::post("/functions/v1/device-enroll", &serde_json::json!({ "action": "unregister", "device_id": id }), Some(&token))?;
    let _ = vault::delete(DEVICE_ID_ENTRY);
    Ok(())
}
