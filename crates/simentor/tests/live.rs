//! Live checks against the real Simentor project. Ignored by default: they need
//! `SIMENTOR_URL` and `SIMENTOR_ANON_KEY` in the environment (or an
//! `appsettings.json` beside the test binary) and a reachable network.
//!
//! ```sh
//! SIMENTOR_URL=https://<ref>.supabase.co SIMENTOR_ANON_KEY=<anon> \
//!   cargo test -p designcraft-simentor --features online --test live -- --ignored
//! ```
//!
//! They prove the transport and the status mapping end to end without a real
//! account: nothing here logs in successfully or fetches a credential.

#![cfg(all(feature = "online", not(target_arch = "wasm32")))]

use designcraft_simentor::{Error, client};

fn configured() -> bool {
    std::env::var("SIMENTOR_URL").is_ok() && std::env::var("SIMENTOR_ANON_KEY").is_ok()
}

/// The project answers with the publishable/anon key: a wrong password is the
/// project's own `400`, not a transport error.
#[test]
#[ignore = "touches the live Simentor project"]
fn auth_endpoint_answers_and_rejects_bad_credentials() {
    if !configured() {
        eprintln!("SIMENTOR_URL / SIMENTOR_ANON_KEY unset: skipping");
        return;
    }
    let r = client::post(
        "/auth/v1/token?grant_type=password",
        &serde_json::json!({ "email": "nobody@bps.go.id", "password": "definitely-not-the-password" }),
        None,
    );
    match r {
        Err(Error::Auth(_)) | Err(Error::BadRequest(_)) => {}
        other => panic!("expected the project to reject the credentials, got {other:?}"),
    }
}

/// A project whose Edge Functions are not deployed yet answers `404`; the client
/// maps that to `NotFound` and the UI names the missing piece instead of a
/// generic failure. Once `device-enroll` is deployed this turns into `401`.
#[test]
#[ignore = "touches the live Simentor project"]
fn an_undeployed_edge_function_is_not_found_not_a_transport_error() {
    if !configured() {
        eprintln!("SIMENTOR_URL / SIMENTOR_ANON_KEY unset: skipping");
        return;
    }
    let r = client::post("/functions/v1/device-enroll", &serde_json::json!({ "action": "list_mine" }), None);
    match r {
        Err(Error::NotFound(_)) | Err(Error::Auth(_)) | Err(Error::Forbidden(_)) => {}
        other => panic!("expected 404/401/403 from the Edge Function, got {other:?}"),
    }
}

/// The sealed-box self-test the handoff §5 asks every build to include: a
/// keypair seals and opens its own message, and a stranger's key does not.
#[test]
fn sealed_box_self_test() {
    use designcraft_simentor::crypto;

    let kp = crypto::DeviceKeypair::generate().unwrap();
    let sealed = crypto::seal(&kp.public, b"self-test").unwrap();
    assert_eq!(crypto::open(&kp, &sealed).unwrap(), b"self-test");
    let other = crypto::DeviceKeypair::generate().unwrap();
    assert!(crypto::open(&other, &sealed).is_err());
}
