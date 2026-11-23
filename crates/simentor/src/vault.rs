//! The OS credential vault. Portable x64 Windows is the shipped target, so the
//! vault is Windows Credential Manager (DPAPI-protected); other platforms get a
//! clear "unsupported" error instead of a plaintext fallback.

use crate::{Error, Result};

/// The keyring service every Simentor entry lives under.
#[cfg(windows)]
const SERVICE: &str = "DesignCraft Pubgen Simentor";

/// Store `value` under `name`. Overwrites any previous value.
#[cfg(windows)]
pub fn set(name: &str, value: &str) -> Result<()> {
    keyring::Entry::new(SERVICE, name).map_err(|e| Error::Vault(e.to_string()))?.set_password(value).map_err(|e| Error::Vault(e.to_string()))
}

/// Read the entry, `None` when absent.
#[cfg(windows)]
pub fn get(name: &str) -> Result<Option<String>> {
    match keyring::Entry::new(SERVICE, name).map_err(|e| Error::Vault(e.to_string()))?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(Error::Vault(e.to_string())),
    }
}

/// Delete the entry; absent is fine.
#[cfg(windows)]
pub fn delete(name: &str) -> Result<()> {
    match keyring::Entry::new(SERVICE, name).map_err(|e| Error::Vault(e.to_string()))?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(Error::Vault(e.to_string())),
    }
}

#[cfg(not(windows))]
pub fn set(_name: &str, _value: &str) -> Result<()> {
    Err(Error::Vault("this build targets Windows Credential Manager; no vault is available here".into()))
}

#[cfg(not(windows))]
pub fn get(_name: &str) -> Result<Option<String>> {
    Err(Error::Vault("this build targets Windows Credential Manager; no vault is available here".into()))
}

#[cfg(not(windows))]
pub fn delete(_name: &str) -> Result<()> {
    Err(Error::Vault("this build targets Windows Credential Manager; no vault is available here".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Windows-only round trip through the real vault; a portable test run of this
    /// crate on the CI Windows job proves the plumbing end to end.
    #[cfg(windows)]
    #[test]
    fn round_trip_and_delete() {
        let name = format!("simentor-test-{}", std::process::id());
        set(&name, "value-1").unwrap();
        assert_eq!(get(&name).unwrap().as_deref(), Some("value-1"));
        set(&name, "value-2").unwrap();
        assert_eq!(get(&name).unwrap().as_deref(), Some("value-2"));
        delete(&name).unwrap();
        assert_eq!(get(&name).unwrap(), None, "deleted");
        delete(&name).unwrap(); // idempotent
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_fails_closed() {
        assert!(matches!(get("x"), Err(Error::Vault(_))));
    }
}
