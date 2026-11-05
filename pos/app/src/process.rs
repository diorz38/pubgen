use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};

pub struct DesignCraftProcess {
    child: Child,
}

fn find_portable_zip() -> Option<PathBuf> {
    for p in &[
        "assets/designcraft-0.17.0-windows-x64-portable.zip",
        "../assets/designcraft-0.17.0-windows-x64-portable.zip",
        "../../assets/designcraft-0.17.0-windows-x64-portable.zip",
        "assets/designcraft-0.16.0-windows-x64-portable.zip",
        "../assets/designcraft-0.16.0-windows-x64-portable.zip",
        "../../assets/designcraft-0.16.0-windows-x64-portable.zip",
    ] {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    None
}

fn find_extracted_portable(file: &str) -> Option<PathBuf> {
    // 1. Direct check in assets/designcraft-portable relative to current working directory
    for base in &[
        "assets/designcraft-portable",
        "../assets/designcraft-portable",
        "../../assets/designcraft-portable",
        "designcraft-portable",
    ] {
        let p = PathBuf::from(base).join(file);
        if p.is_file() {
            return Some(p);
        }
    }
    // 2. Relative to the running executable (for standalone / release bundles)
    if let Ok(cur_exe) = std::env::current_exe() {
        if let Some(exe_dir) = cur_exe.parent() {
            for sub in &[
                exe_dir.join("assets").join("designcraft-portable").join(file),
                exe_dir.join("designcraft-portable").join(file),
                exe_dir.join("..").join("assets").join("designcraft-portable").join(file),
            ] {
                if sub.is_file() {
                    return Some(sub.clone());
                }
            }
        }
    }
    None
}

/// Automatically detects and unzips the portable DesignCraft ZIP into assets/designcraft-portable if present.
pub fn auto_extract_portable() -> Option<PathBuf> {
    if let Some(exe) = find_extracted_portable("designcraft.exe") {
        return Some(exe);
    }
    let zip_path = find_portable_zip()?;
    let target_dir = zip_path.parent().unwrap_or(Path::new("assets")).join("designcraft-portable");
    let file = std::fs::File::open(&zip_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let _ = std::fs::create_dir_all(&target_dir);
    let _ = archive.extract(&target_dir);
    find_extracted_portable("designcraft.exe")
}

pub fn resolve_exe() -> Option<PathBuf> {
    // Only resolve from assets/designcraft-portable, do not use other sources
    if let Some(p) = find_extracted_portable("designcraft.exe") {
        return Some(p);
    }
    auto_extract_portable()
}

pub fn resolve_cli() -> Option<PathBuf> {
    // Only resolve from assets/designcraft-portable, do not use other sources
    if let Some(p) = find_extracted_portable("designcraft-cli.exe") {
        return Some(p);
    }
    if let Some(exe) = resolve_exe()
        && let Some(dir) = exe.parent()
    {
        let cli = dir.join("designcraft-cli.exe");
        if cli.is_file() {
            return Some(cli);
        }
    }
    None
}

pub fn spawn(exe: &Path, port: u16) -> Result<DesignCraftProcess> {
    let child = Command::new(exe)
        .arg("--control")
        .arg(port.to_string())
        .spawn()
        .map_err(|e| anyhow!("failed to launch {}: {e}", exe.display()))?;
    Ok(DesignCraftProcess { child })
}

pub fn poll(proc: &mut DesignCraftProcess) -> Option<ExitStatus> {
    proc.child.try_wait().ok().flatten()
}

pub fn kill(proc: &mut DesignCraftProcess) {
    let _ = proc.child.kill();
    let _ = proc.child.wait();
}

pub fn open_in_system_viewer(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("file not found: {}", path.display()));
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn()
            .map_err(|e| anyhow!("failed to open file in system viewer: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| anyhow!("failed to open file in system viewer: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| anyhow!("failed to open file in system viewer: {e}"))?;
    }
    Ok(())
}

pub fn launch_gui_with_doc(exe: &Path, doc_path: &Path) -> Result<()> {
    Command::new(exe)
        .arg(doc_path)
        .spawn()
        .map_err(|e| anyhow!("failed to launch DesignCraft with {}: {e}", doc_path.display()))?;
    Ok(())
}

impl Drop for DesignCraftProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portable_zip_detection() {
        let zip = find_portable_zip();
        assert!(zip.is_some(), "portable zip should be found in assets/");
    }

    #[test]
    fn test_resolve_cli_finds_monorepo_target() {
        let cli = resolve_cli();
        assert!(cli.is_some(), "CLI should be resolved from assets/designcraft-portable");
        let cli_path = cli.unwrap();
        assert!(cli_path.to_string_lossy().contains("designcraft-portable"), "CLI must come from designcraft-portable: {cli_path:?}");
        println!("Resolved CLI: {:?}", cli_path);
    }

    #[test]
    fn test_resolve_exe_finds_portable() {
        let exe = resolve_exe();
        assert!(exe.is_some(), "GUI exe should be resolved from assets/designcraft-portable");
        let exe_path = exe.unwrap();
        assert!(exe_path.to_string_lossy().contains("designcraft-portable"), "GUI exe must come from designcraft-portable: {exe_path:?}");
        println!("Resolved GUI exe: {:?}", exe_path);
    }
}
