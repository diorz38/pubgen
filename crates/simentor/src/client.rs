//! One HTTP helper for every Simentor call: anon/publishable key header, optional
//! bearer token, a hard timeout, and the status-code mapping the handoff §4.3
//! defines. Every Edge Function reply carries `success`; a `200` with
//! `success: false` is an error too.

use crate::{Error, Result, config};

/// POST JSON to `{url}{path}`; the parsed 2xx body. `bearer` is the Supabase
/// access token (JWT) of the signed-in user.
pub fn post(path: &str, body: &serde_json::Value, bearer: Option<&str>) -> Result<serde_json::Value> {
    let (url, key) = config::endpoints()?;
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(15)).build();
    let mut req = agent.post(&format!("{url}{path}")).set("apikey", &key).set("Content-Type", "application/json");
    if let Some(token) = bearer {
        req = req.set("Authorization", &format!("Bearer {token}"));
    }
    let res = req.send_string(&body.to_string()).map_err(|e| match e {
        ureq::Error::Status(status, response) => {
            let body = response.into_string().unwrap_or_default();
            map_status(status, &body)
        }
        other => Error::Server { status: 0, message: other.to_string() },
    })?;
    let text = res.into_string().map_err(|e| Error::Server { status: 0, message: e.to_string() })?;
    if text.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| Error::Server { status: 0, message: format!("invalid JSON reply: {e}") })?;
    // Edge Functions answer `{ success: false, message }` with HTTP 200 in some paths.
    if value.get("success").and_then(|v| v.as_bool()) == Some(false) {
        return Err(Error::from_status(400, &text));
    }
    Ok(value)
}

/// Handoff §4.3. The message names what the user can do, never a secret.
fn map_status(status: u16, body: &str) -> Error {
    match status {
        400 => Error::BadRequest(Error::message_field(body).unwrap_or_else(|| "invalid request".into())),
        401 => Error::Auth("session expired — sign in again".into()),
        403 => Error::Forbidden(
            Error::message_field(body)
                .unwrap_or_else(|| "a verified @bps.go.id account with a Simentor profile and an active device is required".into()),
        ),
        404 => Error::NotFound(Error::message_field(body).unwrap_or_else(|| "device not found".into())),
        409 => Error::DeviceLimit,
        _ => Error::from_status(status, body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_code_maps_to_its_own_variant() {
        // Handoff §4.3: these must not collapse into one generic error, because
        // only some are retryable.
        assert!(matches!(map_status(400, ""), Error::BadRequest(_)));
        assert!(matches!(map_status(401, ""), Error::Auth(_)));
        assert!(matches!(map_status(403, ""), Error::Forbidden(_)));
        assert!(matches!(map_status(404, ""), Error::NotFound(_)));
        assert!(matches!(map_status(409, ""), Error::DeviceLimit));
        assert!(matches!(map_status(500, r#"{"message":"boom"}"#), Error::Server { status: 500, .. }));
    }

    #[test]
    fn unconfigured_projects_fail_closed_before_any_network() {
        // With no config at all this must be NotConfigured — never a hang.
        if config::endpoints().is_ok() {
            return; // a developer machine with a real appsettings.json / env
        }
        let r = post("/auth/v1/user", &serde_json::json!({}), None);
        assert!(matches!(r, Err(Error::NotConfigured)), "{r:?}");
    }
}
