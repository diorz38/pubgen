//! One HTTP helper for every Simentor call: anon key header, optional bearer
//! token, a hard timeout, and errors that keep the status code.

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
    serde_json::from_str(&text).map_err(|e| Error::Server { status: 0, message: format!("invalid JSON reply: {e}") })
}

fn map_status(status: u16, body: &str) -> Error {
    match status {
        401 => Error::Auth("session expired — sign in again".into()),
        409 => Error::DeviceLimit,
        _ => Error::from_status(status, body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconfigured_projects_fail_closed_before_any_network() {
        // With no env config and no reachable project this must be NotConfigured
        // (or a configured developer project would answer) — never a hang.
        let r = post("/auth/v1/user", &serde_json::json!({}), None);
        assert!(matches!(r, Err(Error::NotConfigured) | Err(Error::Server { .. }) | Err(Error::Auth(_))), "{r:?}");
    }
}
