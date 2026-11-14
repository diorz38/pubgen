//! Portable PubGen update helper.
//!
//! A detached helper copy waits for PubGen Studio to exit, then replaces only files from the
//! verified portable ZIP. User data such as `DesignCraftData` is never part of the replacement.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const HELPER_FLAG: &str = "--pgs-portable-update";
const EXECUTABLE: &str = "pgs.exe";
const WAIT_TIMEOUT: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelperRequest {
    pub parent_pid: u32,
    pub zip: PathBuf,
    pub staging: PathBuf,
    pub install_dir: PathBuf,
    pub sha256: String,
}

/// Parses helper arguments. `None` means this is a normal Studio launch.
pub fn parse_helper_args(args: &[OsString]) -> Option<Result<HelperRequest, String>> {
    if args.first().is_none_or(|arg| arg != OsStr::new(HELPER_FLAG)) {
        return None;
    }
    let parsed = (|| {
        let [_, pid, zip, staging, install_dir, sha256] = args else {
            return Err("the update helper's arguments are incomplete".into());
        };
        let parent_pid = pid.to_str().and_then(|value| value.parse::<u32>().ok()).ok_or("the update helper's process ID is invalid")?;
        let sha256 = sha256.to_str().ok_or("the update helper's checksum is invalid")?;
        if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("the update helper's checksum is invalid".into());
        }
        Ok(HelperRequest {
            parent_pid,
            zip: PathBuf::from(zip),
            staging: PathBuf::from(staging),
            install_dir: PathBuf::from(install_dir),
            sha256: sha256.to_string(),
        })
    })();
    Some(parsed)
}

/// Starts a detached helper that replaces this portable installation after Studio exits.
///
/// The helper runs from a copy outside the installation folder. Copying it is required on
/// Windows, where the running executable cannot be replaced while it is locked.
pub fn start_portable_update(zip: &Path, staging: &Path, sha256: &str) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|e| format!("couldn't find PubGen Studio's executable ({e})"))?;
    let install_dir = executable.parent().ok_or("PubGen Studio's executable has no parent folder")?.to_path_buf();
    start_portable_update_for(&executable, &install_dir, zip, staging, sha256)
}

fn start_portable_update_for(executable: &Path, install_dir: &Path, zip: &Path, staging: &Path, sha256: &str) -> Result<(), String> {
    validate_paths(install_dir, zip, staging)?;
    let helper = staging.with_extension("helper.exe");
    let _ = std::fs::remove_file(&helper);
    std::fs::copy(executable, &helper).map_err(|e| format!("couldn't copy the update helper ({e})"))?;

    let request = HelperRequest {
        parent_pid: std::process::id(),
        zip: zip.to_path_buf(),
        staging: staging.to_path_buf(),
        install_dir: install_dir.to_path_buf(),
        sha256: sha256.to_string(),
    };
    let args = helper_args(&request);
    let mut command = craft_update::process::command(&helper);
    command.args(args);
    craft_update::process::spawn_detached(command).inspect_err(|_| {
        let _ = std::fs::remove_file(&helper);
    })
}

fn helper_args(request: &HelperRequest) -> Vec<OsString> {
    vec![
        OsString::from(HELPER_FLAG),
        OsString::from(request.parent_pid.to_string()),
        request.zip.clone().into_os_string(),
        request.staging.clone().into_os_string(),
        request.install_dir.clone().into_os_string(),
        OsString::from(&request.sha256),
    ]
}

fn validate_paths(install_dir: &Path, zip: &Path, staging: &Path) -> Result<(), String> {
    if !install_dir.is_absolute() || !zip.is_absolute() || !staging.is_absolute() {
        return Err("the update helper needs absolute paths".into());
    }
    if staging.starts_with(install_dir) {
        return Err("the update staging folder must be outside the installation folder".into());
    }
    if !zip.is_file() {
        return Err("the downloaded update ZIP is missing".into());
    }
    if !install_dir.join(EXECUTABLE).is_file() {
        return Err(format!("the installation folder has no {EXECUTABLE}"));
    }
    Ok(())
}

/// Handles helper mode. `None` means normal application startup.
pub fn run_helper(args: &[OsString]) -> Option<Result<(), String>> {
    Some(match parse_helper_args(args)? {
        Ok(request) => run_helper_request(&request),
        Err(error) => Err(error),
    })
}

fn run_helper_request(request: &HelperRequest) -> Result<(), String> {
    let result = run_helper_steps(request);
    // The helper is a detached GUI-subsystem process: its only diagnostic channel is a
    // log beside the staging archive, so failures are never silently invisible.
    let log = request.staging.with_extension("log");
    let outcome = match &result {
        Ok(()) => "ok".to_string(),
        Err(error) => format!("error: {error}"),
    };
    let line = format!(
        "{} pubgen portable update {}\n",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        outcome
    );
    let _ = std::fs::write(&log, line);
    result
}

fn run_helper_steps(request: &HelperRequest) -> Result<(), String> {
    wait_for_parent(request.parent_pid)?;
    validate_paths(&request.install_dir, &request.zip, &request.staging)?;
    verify_archive(&request.zip, &request.sha256)?;
    let rollback = request.staging.with_extension("rollback");
    let _ = std::fs::remove_dir_all(&rollback);
    backup_archive_files(&request.zip, &request.install_dir, &rollback)?;
    install_portable_zip(&request.zip, &request.staging, &request.install_dir)?;
    if let Err(error) = craft_update::process::launch_binary(&request.install_dir.join(EXECUTABLE), &[]) {
        let restore = restore_archive_files(&request.zip, &request.install_dir, &rollback);
        return match restore {
            Ok(()) => Err(format!("the update failed to restart and was rolled back ({error})")),
            Err(restore_error) => Err(format!("the update failed to restart ({error}); rollback also failed ({restore_error})")),
        };
    }
    let _ = std::fs::remove_dir_all(&rollback);
    Ok(())
}

#[cfg(windows)]
fn wait_for_parent(pid: u32) -> Result<(), String> {
    if craft_update::process::wait_for_exit(pid, WAIT_TIMEOUT) { Ok(()) } else { Err("PubGen Studio didn't exit before the update timeout".into()) }
}

#[cfg(not(windows))]
fn wait_for_parent(_pid: u32) -> Result<(), String> {
    Err("portable updates are supported only on Windows".into())
}

fn verify_archive(zip: &Path, expected_sha256: &str) -> Result<(), String> {
    let actual = craft_update::http::sha256_file(zip)?;
    if !actual.eq_ignore_ascii_case(expected_sha256) {
        return Err("the downloaded update ZIP does not match its published SHA-256".into());
    }
    Ok(())
}

fn install_portable_zip(zip: &Path, staging: &Path, install_dir: &Path) -> Result<(), String> {
    validate_archive_targets(zip, install_dir)?;
    craft_update::portable::unpack_over(zip, staging, install_dir, OsStr::new(EXECUTABLE), &|_path| Ok(()))
}

fn validate_archive_targets(zip: &Path, install_dir: &Path) -> Result<(), String> {
    for relative in archive_install_files(zip)? {
        let mut components = relative.components();
        if components.any(|component| matches!(component, std::path::Component::ParentDir)) {
            return Err("the update ZIP contains an unsafe path".into());
        }
        if relative.starts_with("DesignCraftData") {
            return Err("the update ZIP must not contain DesignCraftData".into());
        }
        let target = install_dir.join(&relative);
        if let Ok(metadata) = std::fs::symlink_metadata(&target)
            && metadata.file_type().is_symlink()
        {
            return Err("the update target must not be a symbolic link".into());
        }
    }
    Ok(())
}

fn archive_install_files(zip: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let file = std::fs::File::open(zip).map_err(|e| format!("couldn't read the downloaded update ZIP ({e})"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("couldn't read the downloaded update ZIP ({e})"))?;
    let mut files = Vec::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|e| format!("couldn't read the downloaded update ZIP ({e})"))?;
        if !entry.is_file() {
            continue;
        }
        files.push(PathBuf::from(entry.name()));
    }
    let wrapper = files.first().and_then(|first| first.components().next()).map(|first| first.as_os_str().to_owned());
    if let Some(wrapper) = wrapper
        && files.iter().all(|file| file.components().next().is_some_and(|part| part.as_os_str() == wrapper) && file.components().count() > 1)
    {
        return Err("the update ZIP must use a flat layout".into());
    }
    Ok(files)
}

fn backup_archive_files(zip: &Path, install_dir: &Path, rollback: &Path) -> Result<(), String> {
    std::fs::create_dir_all(rollback).map_err(|e| format!("couldn't create the update rollback folder ({e})"))?;
    for relative in archive_install_files(zip)? {
        let source = install_dir.join(&relative);
        if source.is_file() {
            let target = rollback.join(&relative);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("couldn't create the update rollback folder ({e})"))?;
            }
            std::fs::copy(&source, &target).map_err(|e| format!("couldn't back up {} ({e})", source.display()))?;
        }
    }
    Ok(())
}

fn restore_archive_files(zip: &Path, install_dir: &Path, rollback: &Path) -> Result<(), String> {
    for relative in archive_install_files(zip)? {
        let backup = rollback.join(&relative);
        let target = install_dir.join(&relative);
        if backup.is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("couldn't restore the previous update ({e})"))?;
            }
            std::fs::copy(&backup, &target).map_err(|e| format!("couldn't restore {} ({e})", target.display()))?;
        } else if target.is_file() {
            std::fs::remove_file(&target).map_err(|e| format!("couldn't remove {} ({e})", target.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn write_zip(path: &Path, files: &[(&str, &[u8])]) {
        let file = std::fs::File::create(path).expect("create ZIP");
        let mut writer = zip::ZipWriter::new(file);
        for (name, contents) in files {
            writer.start_file(*name, SimpleFileOptions::default()).expect("start ZIP file");
            writer.write_all(contents).expect("write ZIP file");
        }
        writer.finish().expect("finish ZIP");
    }

    #[test]
    fn normal_arguments_do_not_start_the_helper() {
        let args = vec![OsString::from("project.pubgen")];
        assert_eq!(parse_helper_args(&args), None);
    }

    #[test]
    fn helper_arguments_round_trip() {
        let request = HelperRequest {
            parent_pid: 42,
            zip: PathBuf::from(r"C:\cache\update.zip"),
            staging: PathBuf::from(r"C:\cache\pgs-update"),
            install_dir: PathBuf::from(r"C:\apps\PubGen"),
            sha256: "a".repeat(64),
        };
        assert_eq!(parse_helper_args(&helper_args(&request)), Some(Ok(request)));
    }

    #[test]
    fn incomplete_helper_arguments_fail() {
        assert!(parse_helper_args(&[OsString::from(HELPER_FLAG)]).is_some_and(|result| result.is_err()));
    }

    #[test]
    fn archive_with_wrong_checksum_is_rejected() {
        let root = tempfile::tempdir().expect("temp dir");
        let zip = root.path().join("update.zip");
        write_zip(&zip, &[(EXECUTABLE, b"new")]);
        assert!(verify_archive(&zip, &"0".repeat(64)).is_err());
    }

    #[test]
    fn archive_with_user_data_is_rejected() {
        let root = tempfile::tempdir().expect("temp dir");
        let install = root.path().join("PubGen");
        std::fs::create_dir_all(&install).expect("create install");
        std::fs::write(install.join(EXECUTABLE), b"old").expect("write executable");
        let zip = root.path().join("update.zip");
        write_zip(&zip, &[(EXECUTABLE, b"new"), ("DesignCraftData/settings.json", b"overwrite")]);
        assert!(validate_archive_targets(&zip, &install).is_err());
    }

    #[test]
    fn portable_update_replaces_archive_files_and_keeps_user_data() {
        let root = tempfile::tempdir().expect("temp dir");
        let install = root.path().join("PubGen");
        std::fs::create_dir_all(install.join("DesignCraftData")).expect("create install");
        std::fs::write(install.join(EXECUTABLE), b"old").expect("write executable");
        std::fs::write(install.join("DesignCraftData").join("settings.json"), b"user data").expect("write user data");
        std::fs::write(install.join("old-only.txt"), b"keep").expect("write unrelated file");
        let zip = root.path().join("update.zip");
        write_zip(&zip, &[(EXECUTABLE, b"new"), ("README.md", b"new readme")]);

        install_portable_zip(&zip, &root.path().join("staging"), &install).expect("install update");

        assert_eq!(std::fs::read(install.join(EXECUTABLE)).expect("read executable"), b"new");
        assert_eq!(std::fs::read(install.join("README.md")).expect("read readme"), b"new readme");
        assert_eq!(std::fs::read(install.join("DesignCraftData").join("settings.json")).expect("read user data"), b"user data");
        assert!(install.join("old-only.txt").is_file());
    }
}
