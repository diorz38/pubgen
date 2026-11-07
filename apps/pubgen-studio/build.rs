//! Windows only: embed the PGS icon and version info (VERSIONINFO) into `pgs.exe`.

fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let ico_path = manifest_dir.join("../../assets/app-icon/pgs_icon.ico");
    let ico_canonical = ico_path.canonicalize().unwrap_or(ico_path);
    let ico_str = ico_canonical.display().to_string().replace('\\', "/");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed={ico_str}");
    println!("cargo:rerun-if-env-changed=PGS_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon(&ico_str)
        .set("ProductName", "Publication Generator Studio")
        .set("FileDescription", "Publication Generator Studio")
        .set("CompanyName", "Learning Machines LLC")
        .set("LegalCopyright", "Copyright (c) the authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "pgs.exe")
        .set("InternalName", "pgs");
    if let Err(e) = res.compile() {
        if std::env::var_os("PGS_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=pgs.exe built without icon/version resources: {e}");
    }
}
