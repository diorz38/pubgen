//! Windows only: embed the app icon and version info (VERSIONINFO) into `pos.exe`, so it shows
//! in Explorer, the taskbar, the Start menu and Alt-Tab.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `POS_REQUIRE_WINRES=1` turns it into an
//! error (for release builds).

use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let ico_path = manifest_dir.join("../assets/app-icon/pos.ico");
    let ico_canonical = ico_path.canonicalize().unwrap_or(ico_path);
    let ico_str = ico_canonical.display().to_string().replace('\\', "/");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed={ico_str}");
    println!("cargo:rerun-if-changed=../assets/templates");
    println!("cargo:rerun-if-changed=../assets/maps");
    println!("cargo:rerun-if-env-changed=PUBGEN_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=POS_REQUIRE_WINRES");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default());
    let workspace_root = manifest_dir.parent().unwrap_or(&manifest_dir);

    // 1. Pack embedded templates into OUT_DIR/embedded_templates.zip
    let templates_dir = workspace_root.join("assets").join("templates");
    if templates_dir.is_dir() {
        let zip_path = out_dir.join("embedded_templates.zip");
        zip_directory(&templates_dir, &zip_path);
    }

    // 2. Pack embedded maps into OUT_DIR/embedded_maps.zip
    let maps_dir = workspace_root.join("assets").join("maps");
    if maps_dir.is_dir() {
        let zip_path = out_dir.join("embedded_maps.zip");
        zip_directory(&maps_dir, &zip_path);
    }

    // In release mode, automatically bundle assets (including designcraft-portable) and filtered samples
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        bundle_release_assets(&manifest_dir);
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon(&ico_str)
        .set("ProductName", "Publication Orchestrator Studio")
        .set("FileDescription", "Publication Orchestrator Studio")
        .set("OriginalFilename", "pos.exe")
        .set("InternalName", "pos");
    if let Err(e) = res.compile() {
        if std::env::var_os("POS_REQUIRE_WINRES").is_some()
            || std::env::var_os("PUBGEN_REQUIRE_WINRES").is_some()
        {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=pos.exe built without icon/version resources: {e}");
    }
}

fn zip_directory(src_dir: &Path, zip_path: &Path) {
    let file = match std::fs::File::create(zip_path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip_dir_walk(src_dir, src_dir, &mut zip, options);
    let _ = zip.finish();
}

fn zip_dir_walk(
    root: &Path,
    current: &Path,
    zip: &mut zip::ZipWriter<std::fs::File>,
    options: zip::write::SimpleFileOptions,
) {
    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            zip_dir_walk(root, &path, zip, options);
        } else if path.is_file() {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let name = rel.to_string_lossy().replace('\\', "/");
            if zip.start_file(&name, options).is_ok() {
                if let Ok(mut f) = std::fs::File::open(&path) {
                    let _ = std::io::copy(&mut f, zip);
                }
            }
        }
    }
}

fn bundle_release_assets(manifest_dir: &Path) {
    let workspace_root = match manifest_dir.parent() {
        Some(p) => p,
        None => return,
    };

    let mut target_dirs = Vec::new();
    if let Ok(out_dir_str) = std::env::var("OUT_DIR") {
        let out_dir = PathBuf::from(out_dir_str);
        if let Some(p) = out_dir
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
        {
            target_dirs.push(p.to_path_buf());
        }
    }
    let standard_release = workspace_root.join("target").join("release");
    if !target_dirs.contains(&standard_release) {
        target_dirs.push(standard_release);
    }

    for target_dir in target_dirs {
        let _ = std::fs::create_dir_all(&target_dir);

        // 1. Bundle assets/ EXCLUDING maps and templates (which are now embedded in pos.exe)
        let src_assets = workspace_root.join("assets");
        let dst_assets = target_dir.join("assets");
        if src_assets.is_dir() {
            copy_dir_recursive(&src_assets, &dst_assets, &|rel_path, _file| {
                let parts: Vec<&str> = rel_path
                    .split(['/', '\\'])
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.contains(&"maps") || parts.contains(&"templates") {
                    return false;
                }
                true
            });
        }

        // 2. Bundle samples/ with the exclusion rules
        let src_samples = workspace_root.join("samples");
        let dst_samples = target_dir.join("samples");
        if src_samples.is_dir() {
            // Rule 1: samples/brs -> exclude folder csv and md, exclude *.designcraft, *.pdf
            let src_brs = src_samples.join("brs");
            let dst_brs = dst_samples.join("brs");
            if src_brs.is_dir() {
                copy_dir_recursive(&src_brs, &dst_brs, &|rel_path, file_name| {
                    let parts: Vec<&str> = rel_path
                        .split(['/', '\\'])
                        .filter(|s| !s.is_empty())
                        .collect();
                    if parts.contains(&"csv") || parts.contains(&"md") {
                        return false;
                    }
                    if let Some(f) = file_name {
                        let lower = f.to_lowercase();
                        if lower.ends_with(".designcraft") || lower.ends_with(".pdf") {
                            return false;
                        }
                    }
                    true
                });
            }

            // Rule 2: samples/DUAL_LANG -> INCLUDE maps/ (sample geojson + csv), exclude *.designcraft, exclude *.csv outside maps
            let src_dual = src_samples.join("DUAL_LANG");
            let dst_dual = dst_samples.join("DUAL_LANG");
            if src_dual.is_dir() {
                copy_dir_recursive(&src_dual, &dst_dual, &|rel_path, file_name| {
                    let parts: Vec<&str> = rel_path
                        .split(['/', '\\'])
                        .filter(|s| !s.is_empty())
                        .collect();
                    if parts.contains(&"maps") {
                        return true;
                    }
                    if let Some(f) = file_name {
                        let lower = f.to_lowercase();
                        if lower.ends_with(".csv") || lower.ends_with(".designcraft") {
                            return false;
                        }
                    }
                    true
                });
            }

            // Rule 3: samples/SINGLE_LANG -> exclude folder maps, exclude *.csv, *.designcraft
            let src_single = src_samples.join("SINGLE_LANG");
            let dst_single = dst_samples.join("SINGLE_LANG");
            if src_single.is_dir() {
                copy_dir_recursive(&src_single, &dst_single, &|rel_path, file_name| {
                    let parts: Vec<&str> = rel_path
                        .split(['/', '\\'])
                        .filter(|s| !s.is_empty())
                        .collect();
                    if parts.contains(&"maps") {
                        return false;
                    }
                    if let Some(f) = file_name {
                        let lower = f.to_lowercase();
                        if lower.ends_with(".csv") || lower.ends_with(".designcraft") {
                            return false;
                        }
                    }
                    true
                });
            }
        }
    }
}

fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    filter: &dyn Fn(&str, Option<&str>) -> bool,
) {
    copy_dir_recursive_inner(src, dst, "", filter);
}

fn copy_dir_recursive_inner(
    src_root: &Path,
    dst_root: &Path,
    rel_path: &str,
    filter: &dyn Fn(&str, Option<&str>) -> bool,
) {
    let current_src = if rel_path.is_empty() {
        src_root.to_path_buf()
    } else {
        src_root.join(rel_path)
    };

    if !filter(rel_path, None) {
        return;
    }

    let entries = match std::fs::read_dir(&current_src) {
        Ok(e) => e,
        Err(_) => return,
    };

    let current_dst = if rel_path.is_empty() {
        dst_root.to_path_buf()
    } else {
        dst_root.join(rel_path)
    };

    let _ = std::fs::create_dir_all(&current_dst);

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        let child_rel = if rel_path.is_empty() {
            name_str.to_string()
        } else {
            format!("{rel_path}/{name_str}")
        };

        if path.is_dir() {
            if filter(&child_rel, None) {
                copy_dir_recursive_inner(src_root, dst_root, &child_rel, filter);
            }
        } else if path.is_file() && filter(&child_rel, Some(&name_str)) {
            let target_file = current_dst.join(&file_name);
            let needs_copy = match (std::fs::metadata(&path), std::fs::metadata(&target_file)) {
                (Ok(sm), Ok(dm)) => {
                    sm.len() != dm.len()
                        || sm
                            .modified()
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                            > dm
                                .modified()
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                }
                _ => true,
            };
            if needs_copy {
                let _ = std::fs::copy(&path, &target_file);
            }
        }
    }
}
