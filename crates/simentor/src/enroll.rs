//! Device enrollment against the `device-enroll` Edge Function
//! (`desktop-service-credentials-plan.md` Phase 2): register, list_mine,
//! unregister. The DB enforces the 2-active-device limit; a `409` maps to
//! [`crate::Error::DeviceLimit`].

use crate::{KEY_ALG, Result, auth, client, crypto, vault};

/// The vault entry holding the device keypair (base64 public ‖ secret).
pub const KEYPAIR_ENTRY: &str = "device_keypair";
/// The vault entry holding this device's registered id (from the register reply).
pub const DEVICE_ID_ENTRY: &str = "device_id";

/// This device's keypair, generated and stored on first use.
pub fn keypair() -> Result<crypto::DeviceKeypair> {
    if let Some(stored) = vault::get(KEYPAIR_ENTRY)? {
        return crypto::DeviceKeypair::from_stored(&stored);
    }
    let kp = crypto::DeviceKeypair::generate()?;
    vault::set(KEYPAIR_ENTRY, &kp.to_stored())?;
    Ok(kp)
}

/// `register` in the suggested device contract. Auto-approves for confirmed
/// `@bps.go.id` logins; returns the new device id.
pub fn register(device_label: &str, app_version: &str) -> Result<String> {
    let kp = keypair()?;
    let public = crate::crypto_b64(&kp.public);
    let token = auth::access_token()?;
    let reply = client::post(
        "/functions/v1/device-enroll",
        &serde_json::json!({
            "action": "register",
            "device_label": device_label,
            "public_key": public,
            "key_alg": KEY_ALG,
            "app_version": app_version,
        }),
        Some(&token),
    )?;
    let id = reply.get("device_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    if id.is_empty() {
        return Err(crate::Error::Server { status: 200, message: "register reply carried no device_id".into() });
    }
    vault::set(DEVICE_ID_ENTRY, &id)?;
    Ok(id)
}

/// `list_mine`: the caller's own devices and slot state, as raw JSON for the UI
/// (the shape is the server's to evolve; the app only displays it).
pub fn list_mine() -> Result<serde_json::Value> {
    let token = auth::access_token()?;
    client::post("/functions/v1/device-enroll", &serde_json::json!({ "action": "list_mine" }), Some(&token))
}

/// `unregister`: soft-revoke this device (or `device_id` when given), freeing a
/// slot and blocking future fetches. Already-unregistered own devices are fine.
pub fn unregister(device_id: Option<&str>) -> Result<()> {
    let token = auth::access_token()?;
    let id = match device_id {
        Some(id) => id.to_string(),
        None => vault::get(DEVICE_ID_ENTRY)?.unwrap_or_default(),
    };
    client::post("/functions/v1/device-enroll", &serde_json::json!({ "action": "unregister", "device_id": id }), Some(&token))?;
    let _ = vault::delete(DEVICE_ID_ENTRY);
    Ok(())
}
