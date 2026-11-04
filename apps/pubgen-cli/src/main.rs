//! Headless DesignCraft publication compiler.
//!
//! ```text
//! pubgen-cli compile <MANIFEST> [--pdf OUT.pdf]   # compile pubgen_project.json via a pubgen-enabled designcraft
//! pubgen-cli info <MANIFEST>                      # summary of the manifest, no build
//! pubgen-cli validate <MANIFEST>                  # chapters and version-2 blocks
//! pubgen-cli --version
//! ```
//!
//! `compile` spawns `designcraft --control <port>` (a pubgen-feature build), waits for the
//! control channel, then calls `engine.execute {"command":"pubgen.compileProject"}` with
//! `{path, pdf?}` — the only parameters that command accepts. The child is killed when this
//! process exits.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const USAGE: &str = "usage: pubgen-cli compile <MANIFEST> [--pdf OUT.pdf]\n       pubgen-cli info <MANIFEST>\n       pubgen-cli validate <MANIFEST>\n       pubgen-cli --version";
const PUBGEN_DESIGNCRAFT_EXE: &str = "PUBGEN_DESIGNCRAFT_EXE";
/// ms between connection attempts to the spawned control server.
const POLL_MS: u64 = 250;
/// Connection attempts before giving up (40 × 250 ms = 10 s).
const CONNECT_ATTEMPTS: u32 = 40;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("pubgen-cli {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Some("compile") => report(run_compile(&args[1..])),
        Some("info") => report(run_info(&args[1..])),
        Some("validate") => report(run_validate(&args[1..])),
        _ => {
            eprintln!("{USAGE}");
            1
        }
    };
    // The child guard's Drop kills designcraft even on the early-return paths.
    std::process::exit(code);
}

fn report(r: Result<(), String>) -> i32 {
    match r {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("pubgen-cli: {e}");
            1
        }
    }
}

/// `compile <MANIFEST> [--pdf OUT.pdf]`.
fn run_compile(args: &[String]) -> Result<(), String> {
    let mut manifest: Option<PathBuf> = None;
    let mut pdf: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--pdf" => {
                i += 1;
                pdf = Some(args.get(i).filter(|s| !s.starts_with('-')).cloned().ok_or("--pdf needs a path")?);
            }
            a if a.starts_with('-') => return Err(format!("unknown option {a}")),
            a if manifest.is_none() => manifest = Some(PathBuf::from(a)),
            a => return Err(format!("unexpected argument {a}")),
        }
        i += 1;
    }
    let manifest = manifest.ok_or("missing MANIFEST (pubgen_project.json)")?;
    if !manifest.is_file() {
        return Err(format!("{}: no such file", manifest.display()));
    }

    // Try each candidate designcraft in order; the first that actually exposes the pubgen
    // compiler wins (a checkout can hold a stock release build beside a pubgen debug build).
    let mut tried: Vec<String> = Vec::new();
    let mut attempt = None;
    for exe in resolve_candidates() {
        let port = free_port()?;
        let mut child = match ChildGuard::spawn(&exe, port) {
            Ok(c) => c,
            Err(e) => {
                tried.push(format!("{} (launch failed: {e})", exe.display()));
                continue;
            }
        };
        let mut client = match connect(port, &mut child) {
            Ok(c) => c,
            Err(e) => {
                tried.push(format!("{} ({e})", exe.display()));
                child.kill();
                continue;
            }
        };
        let has_pubgen = client
            .call("engine.commands", json!({}))
            .ok()
            .and_then(|cmds| cmds.as_array().cloned())
            .is_some_and(|a| a.iter().any(|c| c.get("id").and_then(Value::as_str) == Some("pubgen.compileProject")));
        if has_pubgen {
            attempt = Some((client, child));
            break;
        }
        tried.push(format!("{} (no pubgen.compileProject)", exe.display()));
        child.kill();
    }
    let Some((mut client, mut child)) = attempt else {
        return Err(format!(
            "no pubgen-enabled designcraft found (tried: {}); build one with `cargo build -p designcraft --features pubgen` or set {PUBGEN_DESIGNCRAFT_EXE}",
            tried.join("; ")
        ));
    };

    let mut params = json!({"path": manifest.to_string_lossy()});
    if let Some(pdf) = pdf {
        params["pdf"] = json!(pdf);
    }
    let started = Instant::now();
    let report = client
        .call("engine.execute", json!({"command": "pubgen.compileProject", "params": params}))
        .map_err(|e| format!("pubgen.compileProject: {e}"))?;

    let pages = report.get("pages").and_then(Value::as_u64).unwrap_or(0);
    let chapter_count = report.get("chapters").and_then(Value::as_array).map_or(0, Vec::len);
    println!(
        "compiled {} in {} ms: {} pages, {} chapters",
        manifest.display(),
        report.get("elapsedMs").and_then(Value::as_u64).unwrap_or_else(|| started.elapsed().as_millis() as u64),
        pages,
        chapter_count
    );
    if let Some(doc) = report.get("document").and_then(Value::as_str) {
        println!("  document: {doc}");
    }
    if let Some(out) = report.get("pdf").and_then(Value::as_str) {
        println!("  pdf: {out}");
    }
    for w in report.get("warnings").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
        eprintln!("  warning: {}", w.as_str().unwrap_or("?"));
    }
    child.kill();
    if pages == 0 {
        return Err("compile produced no pages".into());
    }
    Ok(())
}

/// `info <MANIFEST>`.
fn run_info(args: &[String]) -> Result<(), String> {
    let manifest = manifest_arg(args)?;
    let v = read_json(&manifest)?;
    println!("pubgen_project.json (version {})", v.get("version").and_then(Value::as_u64).unwrap_or(0));
    println!("  title: {}", v.get("publicationTitle").and_then(Value::as_str).unwrap_or("Untitled"));
    println!("  bilingual: {}", v.get("bilingual").and_then(Value::as_bool).unwrap_or(false));
    println!("  profile: {}", v.get("profile").and_then(Value::as_str).unwrap_or("(embedded or default)"));
    println!("  chapters: {}", v.get("chapters").and_then(Value::as_array).map(Vec::len).unwrap_or(0));
    println!("  pdf export: {}", v.pointer("/build/exportPdf").and_then(Value::as_bool).unwrap_or(false));
    Ok(())
}

/// `validate <MANIFEST>`: JSON parses, chapters resolve against the manifest folder.
fn run_validate(args: &[String]) -> Result<(), String> {
    let manifest = manifest_arg(args)?;
    let v = read_json(&manifest)?;
    let dir = manifest.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let chapters = v.get("chapters").and_then(Value::as_array).ok_or("missing `chapters` array")?;
    if chapters.is_empty() {
        return Err("manifest has no chapters".into());
    }
    for (i, c) in chapters.iter().enumerate() {
        let num = c.get("chapter").and_then(Value::as_u64).unwrap_or((i + 1) as u64);
        let path = c.get("path").and_then(Value::as_str).unwrap_or_default();
        let state = if path.is_empty() {
            "no path".to_string()
        } else if dir.join(path).is_file() {
            "found".to_string()
        } else {
            "MISSING".to_string()
        };
        println!("  chapter {num}: {} — {state}", c.get("title").and_then(Value::as_str).unwrap_or(path));
        if state == "MISSING" {
            return Err(format!("chapter {num} source not found: {}", dir.join(path).display()));
        }
    }
    println!(
        "  version-2 recipe: profileData={}, generateSettings={}, pagePlan={}",
        v.get("profileData").is_some(),
        v.get("generateSettings").is_some(),
        v.get("pagePlan").is_some()
    );
    Ok(())
}

fn manifest_arg(args: &[String]) -> Result<PathBuf, String> {
    let Some(first) = args.first().filter(|a| !a.starts_with('-')) else {
        return Err(format!("missing MANIFEST\n{USAGE}"));
    };
    Ok(PathBuf::from(first))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: not valid JSON: {e}", path.display()))
}

/// A free localhost port (bind to :0, read it, release it).
fn free_port() -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|e| format!("no free port: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    drop(listener);
    Ok(port)
}

/// Candidate designcraft executables, best first: env var, beside this exe (the portable
/// layout: one pubgen build next to pubgen-cli, launched from any folder), dev targets (debug
/// before release — the freshly built feature binary), the CWD bundle, then PATH.
fn resolve_candidates() -> Vec<PathBuf> {
    let ext = std::env::consts::EXE_SUFFIX;
    let mut out = Vec::new();
    if let Ok(exe) = std::env::var(PUBGEN_DESIGNCRAFT_EXE) {
        out.push(PathBuf::from(exe));
    }
    let name = format!("designcraft{ext}");
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
        out.push(dir.join(&name));
        out.push(dir.join("assets/designcraft-pubgen").join(&name));
    }
    for base in ["target/debug", "target/release", "../target/debug", "../target/release", "assets/designcraft-pubgen"] {
        out.push(Path::new(base).join(&name));
    }
    if let Some(path) = std::env::var_os("PATH") {
        out.extend(std::env::split_paths(&path).map(|d| d.join(&name)));
    }
    out.into_iter().filter(|p| p.is_file()).collect()
}

/// Kills designcraft when dropped, so a failing CLI never leaks the windowed child.
struct ChildGuard(Option<Child>);

impl ChildGuard {
    fn spawn(exe: &Path, port: u16) -> std::io::Result<Self> {
        Command::new(exe).arg("--control").arg(port.to_string()).spawn().map(|c| Self(Some(c)))
    }
    fn exited(&mut self) -> bool {
        self.0.as_mut().is_some_and(|c| c.try_wait().map(|s| s.is_some()).unwrap_or(true))
    }
    fn kill(&mut self) {
        if let Some(mut c) = self.0.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill();
    }
}

/// Blocking JSON-lines client for the control channel (same protocol as pos.app's).
struct ControlClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    next_id: u64,
}

fn connect(port: u16, child: &mut ChildGuard) -> Result<ControlClient, String> {
    for _ in 0..CONNECT_ATTEMPTS {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ = stream.set_nodelay(true);
            return Ok(ControlClient {
                reader: BufReader::new(stream.try_clone().map_err(|e| e.to_string())?),
                writer: BufWriter::new(stream),
                next_id: 1,
            });
        }
        if child.exited() {
            return Err("designcraft exited before opening the control channel".into());
        }
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
    Err("designcraft did not accept a control connection in time".into())
}

impl ControlClient {
    /// One request/reply. No read timeout: a compile can run for minutes.
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        let line = json!({"id": id, "method": method, "params": params});
        writeln!(self.writer, "{line}").map_err(|e| format!("write: {e}"))?;
        self.writer.flush().map_err(|e| format!("flush: {e}"))?;
        loop {
            let mut buf = String::new();
            let n = self.reader.read_line(&mut buf).map_err(|e| format!("read: {e}"))?;
            if n == 0 {
                return Err("control channel closed by designcraft".into());
            }
            let v: Value = serde_json::from_str(buf.trim()).map_err(|e| format!("bad reply: {e}"))?;
            match v.get("id").and_then(Value::as_u64) {
                Some(rid) if rid != id => continue,
                _ => {
                    return if v.get("ok").and_then(Value::as_bool).unwrap_or(false) {
                        Ok(v.get("result").cloned().unwrap_or(Value::Null))
                    } else {
                        Err(v.get("error").and_then(Value::as_str).unwrap_or("unknown error").to_string())
                    };
                }
            }
        }
    }
}
