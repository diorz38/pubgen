//! Windows file association helper for `.pubgen` documents.
//!
//! Registers `.pubgen` file type under `HKEY_CURRENT_USER\Software\Classes`
//! so that:
//! 1. Windows Explorer associates `.pubgen` files with PubGen Studio.
//! 2. `.pubgen` files display the PubGen document/app icon in File Explorer.
//! 3. Double-clicking any `.pubgen` file launches `pgs.exe "<path>"`.
//!
//! Because this writes to `HKEY_CURRENT_USER`, it requires zero administrator/UAC elevation.

#[cfg(target_os = "windows")]
pub fn register_pubgen_file_association() {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let Ok(current_exe) = std::env::current_exe() else {
        return;
    };
    let exe_path_str = current_exe.to_string_lossy().to_string();
    if exe_path_str.trim().is_empty() {
        return;
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(classes) = hkcu.open_subkey_with_flags("Software\\Classes", KEY_READ | KEY_WRITE) else {
        return;
    };

    // 1. Map .pubgen -> PubGen.Document
    if let Ok((dot_pubgen, _)) = classes.create_subkey(".pubgen") {
        let _ = dot_pubgen.set_value("", &"PubGen.Document");
        let _ = dot_pubgen.set_value("Content Type", &"application/x-pubgen");

        if let Ok((open_with, _)) = dot_pubgen.create_subkey("OpenWithProgids") {
            let _ = open_with.set_value("PubGen.Document", &"");
        }
    }

    // 2. Define ProgId: PubGen.Document
    if let Ok((prog_id, _)) = classes.create_subkey("PubGen.Document") {
        let _ = prog_id.set_value("", &"Publication Generator Document");
        let _ = prog_id.set_value("FriendlyTypeName", &"Publication Generator Document");

        // Icon: points to pgs.exe,0 (the embedded Windows resource icon)
        if let Ok((default_icon, _)) = prog_id.create_subkey("DefaultIcon") {
            let icon_spec = format!("\"{exe_path_str}\",0");
            let _ = default_icon.set_value("", &icon_spec);
        }

        // Open command: "C:\path\to\pgs.exe" "%1"
        if let Ok((shell_open_cmd, _)) = prog_id.create_subkey("shell\\open\\command") {
            let cmd_spec = format!("\"{exe_path_str}\" \"%1\"");
            let _ = shell_open_cmd.set_value("", &cmd_spec);
        }
    }

    // 3. Register under Applications\pgs.exe
    if let Ok((app_key, _)) = classes.create_subkey("Applications\\pgs.exe") {
        let _ = app_key.set_value("FriendlyAppName", &"Publication Generator Studio");
        if let Ok((supported_types, _)) = app_key.create_subkey("SupportedTypes") {
            let _ = supported_types.set_value(".pubgen", &"");
            let _ = supported_types.set_value(".designcraft", &"");
        }
        if let Ok((shell_open_cmd, _)) = app_key.create_subkey("shell\\open\\command") {
            let cmd_spec = format!("\"{exe_path_str}\" \"%1\"");
            let _ = shell_open_cmd.set_value("", &cmd_spec);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn register_pubgen_file_association() {
    // Linux and macOS packaging handle mime associations via .desktop or Info.plist.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_pubgen_file_association_runs_without_panic() {
        register_pubgen_file_association();
    }
}
