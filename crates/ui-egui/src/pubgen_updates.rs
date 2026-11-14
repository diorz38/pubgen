//! Pubgen's unsigned portable update flow.
//!
//! The generic updater intentionally refuses unsigned Windows copies. Pubgen therefore downloads
//! the fork's portable ZIP, requires its published SHA-256, and hands the verified archive to the
//! app layer, which replaces the portable installation after the app exits.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use craft_update::http::{Fetched, Transfer};
use craft_update::{Http, Release};

const PORTABLE_SUFFIX: &str = "-windows-x64-portable.zip";

fn portable_suffix() -> &'static str {
    match std::env::consts::ARCH {
        "x86" => "-windows-x86-portable.zip",
        "aarch64" => "-windows-arm64-portable.zip",
        _ => PORTABLE_SUFFIX,
    }
}

/// A verified portable archive waiting for the app layer to install it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingPortableUpdate {
    pub zip: PathBuf,
    pub staging: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Checking,
    Downloading { done: u64, total: u64 },
    Ready { version: String },
    UpToDate,
    Failed(String),
}

enum Event {
    Status(Status),
    Ready { version: String, zip: PathBuf, staging: PathBuf, sha256: String },
}

pub(crate) struct Runtime {
    startup_check: bool,
    check_requested: bool,
    pending: Option<PendingPortableUpdate>,
    install_requested: bool,
    status: Status,
    events: Option<std::sync::mpsc::Receiver<Event>>,
    worker_running: bool,
    progress: Arc<AtomicU64>,
    progress_total: Arc<AtomicU64>,
    repaint: Option<egui::Context>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            startup_check: false,
            check_requested: false,
            pending: None,
            install_requested: false,
            status: Status::Idle,
            events: None,
            worker_running: false,
            progress: Arc::new(AtomicU64::new(0)),
            progress_total: Arc::new(AtomicU64::new(0)),
            repaint: None,
        }
    }
}

impl Runtime {
    pub(crate) fn request_check(&mut self) {
        self.check_requested = true;
    }

    pub(crate) fn set_startup_check(&mut self, on: bool) {
        self.startup_check = on;
    }

    pub(crate) fn dismiss(&mut self) {
        self.status = Status::Idle;
        self.install_requested = false;
        self.pending = None;
    }

    pub(crate) fn tick(&mut self, ctx: &egui::Context) {
        self.repaint = Some(ctx.clone());
        if self.check_requested || self.startup_check {
            if self.worker_running {
                // Keep the request queued until the current worker finishes.
            } else {
                self.check_requested = false;
                self.startup_check = false;
                self.start();
            }
        }
        if let Some(receiver) = self.events.as_ref() {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    Event::Status(status) => {
                        if matches!(status, Status::Failed(_) | Status::UpToDate) {
                            self.worker_running = false;
                        }
                        self.status = status;
                    }
                    Event::Ready { version, zip, staging, sha256 } => {
                        self.status = Status::Ready { version };
                        self.pending = Some(PendingPortableUpdate { zip, staging, sha256 });
                        self.worker_running = false;
                    }
                }
            }
            if !self.worker_running && matches!(self.status, Status::Checking | Status::Downloading { .. }) {
                self.worker_running = false;
            }
        }
        let downloading = matches!(self.status, Status::Downloading { .. });
        if downloading {
            ctx.request_repaint();
        }
    }

    pub(crate) fn request_install(&mut self) {
        self.install_requested = true;
    }

    pub(crate) fn cancel_install(&mut self) {
        self.install_requested = false;
    }

    pub(crate) fn take_pending(&mut self) -> Option<PendingPortableUpdate> {
        if !self.install_requested {
            return None;
        }
        self.install_requested = false;
        self.pending.take()
    }

    pub(crate) fn status(&self) -> &Status {
        &self.status
    }

    pub(crate) fn progress(&self) -> (u64, u64) {
        (self.progress.load(Ordering::Relaxed), self.progress_total.load(Ordering::Relaxed))
    }

    fn start(&mut self) {
        if self.worker_running {
            return;
        }
        self.events = None;
        self.pending = None;
        self.install_requested = false;
        let (sender, receiver) = std::sync::mpsc::channel();
        self.events = Some(receiver);
        self.worker_running = true;
        self.status = Status::Checking;
        let progress = Arc::clone(&self.progress);
        let progress_total = Arc::clone(&self.progress_total);
        std::thread::spawn(move || {
            if let Err(error) = run_check(&sender, progress, progress_total) {
                let _ = sender.send(Event::Status(Status::Failed(error)));
            }
        });
    }
}

fn run_check(sender: &std::sync::mpsc::Sender<Event>, progress: Arc<AtomicU64>, progress_total: Arc<AtomicU64>) -> Result<(), String> {
    let http = Http::new("Pubgen Studio/Update")?;
    let release = latest_offered_release(&http)?;
    let Some(mut release) = release else {
        let _ = sender.send(Event::Status(Status::UpToDate));
        return Ok(());
    };
    let mut asset = select_portable_asset(&release)?;
    if asset.sha256.is_none() {
        if let Some(sums_url) = release.sums_asset().map(|sums| sums.url.clone()) {
            if let Some(text) = http.get_text(&sums_url)? {
                release.apply_sums(&craft_update::release::parse_sha256sums(&text));
                asset = select_portable_asset(&release)?;
            }
        }
    }
    let sha256 = asset.sha256.as_deref().ok_or_else(|| "the release publishes no SHA-256 for the portable archive".to_string())?;
    let cache = update_cache_dir()?;
    std::fs::create_dir_all(&cache).map_err(|e| format!("couldn't create the update cache ({e})"))?;
    let zip = cache.join(&asset.name);
    let staging = cache.join(format!("staging-{}", release.tag));
    let _ = sender.send(Event::Status(Status::Downloading { done: 0, total: asset.size }));
    let mut report = |transfer: Transfer| match transfer {
        Transfer::Bytes { done, total } | Transfer::Reconnecting { done, total } => {
            progress.store(done, Ordering::Relaxed);
            progress_total.store(total, Ordering::Relaxed);
        }
    };
    http.download(&asset.url, &zip, asset.size, sha256, &craft_update::Stop::default(), &mut report)?;
    let _ = sender.send(Event::Ready { version: release.version_label(), zip, staging, sha256: sha256.to_string() });
    Ok(())
}

fn latest_offered_release(http: &Http) -> Result<Option<Release>, String> {
    let url = format!("https://api.github.com/repos/{}/releases?per_page=30", crate::PUBGEN_UPDATE_APP.repo);
    let text = match http.get_api(&url, None)? {
        Fetched::Body { text, .. } => text,
        Fetched::NotModified | Fetched::NotFound => return Ok(None),
        Fetched::RateLimited { .. } => return Err("GitHub's request limit was reached".into()),
    };
    let releases = craft_update::release::parse_github_release_list(&text, crate::PUBGEN_UPDATE_APP.repo, true)?;
    let running = craft_update::version::parse(env!("CARGO_PKG_VERSION"));
    Ok(releases.filter(|release| release_is_offered(release, running.as_ref())))
}

fn release_is_offered(release: &Release, running: Option<&craft_update::version::Version>) -> bool {
    let is_pubgen_channel = release.version.as_ref().is_some_and(|version| version.pre.as_str().starts_with("pgs."));
    is_pubgen_channel && release.version.as_ref().zip(running).is_some_and(|(latest, running)| craft_update::version::is_newer(latest, running))
}

fn select_portable_asset(release: &Release) -> Result<craft_update::Asset, String> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.starts_with("designcraft-") && asset.name.ends_with(portable_suffix()))
        .cloned()
        .ok_or_else(|| "the release has no Windows x64 portable archive".to_string())
}

fn update_cache_dir() -> Result<PathBuf, String> {
    let base = std::env::temp_dir().join("PubgenStudio").join("updates");
    std::fs::create_dir_all(&base).map_err(|e| format!("couldn't create the update cache ({e})"))?;
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_update::Asset;

    fn asset(name: &str) -> Asset {
        Asset {
            name: name.into(),
            url: format!("https://github.com/{}/releases/download/v1/{name}", crate::PUBGEN_UPDATE_APP.repo),
            ..Asset::default()
        }
    }

    #[test]
    fn selects_only_current_windows_architecture_portable_archive() {
        let release = Release {
            assets: vec![asset("designcraft-1.0.0-macos-universal.dmg"), asset("designcraft-1.0.0-windows-x64-portable.zip")],
            ..Release::default()
        };
        let expected = format!("designcraft-1.0.0{}", portable_suffix());
        assert_eq!(select_portable_asset(&release).map(|a| a.name), Ok(expected));
    }

    #[test]
    fn rejects_release_without_portable_archive() {
        let release = Release { assets: vec![asset("designcraft-1.0.0-windows-x64.msi")], ..Release::default() };
        assert!(select_portable_asset(&release).is_err());
    }

    #[test]
    fn only_newer_pubgen_channel_releases_are_offered() {
        let running = craft_update::version::parse("0.20.1-pgs.0.9.4");
        let pubgen = Release { version: craft_update::version::parse("0.20.1-pgs.0.9.5"), ..Release::default() };
        let rc = Release { version: craft_update::version::parse("0.21.0-rc.1"), ..Release::default() };
        let older = Release { version: craft_update::version::parse("0.20.1-pgs.0.9.3"), ..Release::default() };
        assert!(release_is_offered(&pubgen, running.as_ref()));
        assert!(!release_is_offered(&rc, running.as_ref()));
        assert!(!release_is_offered(&older, running.as_ref()));
    }

    #[test]
    fn dismiss_discards_stale_pending_update() {
        let mut runtime = Runtime::default();
        runtime.pending = Some(PendingPortableUpdate { zip: PathBuf::from("a.zip"), staging: PathBuf::from("staging"), sha256: "a".repeat(64) });
        runtime.request_install();
        runtime.dismiss();
        assert!(runtime.take_pending().is_none());
    }

    #[test]
    fn pending_update_is_installed_only_after_explicit_request() {
        let mut runtime = Runtime::default();
        runtime.pending = Some(PendingPortableUpdate { zip: PathBuf::from("a.zip"), staging: PathBuf::from("staging"), sha256: "a".repeat(64) });
        assert!(runtime.take_pending().is_none());
        runtime.request_install();
        assert!(runtime.take_pending().is_some());
    }

    #[test]
    fn pending_update_exposes_zip_and_staging() {
        let pending = PendingPortableUpdate { zip: PathBuf::from("a.zip"), staging: PathBuf::from("staging"), sha256: "a".repeat(64) };
        assert_eq!(pending.zip.file_name().and_then(|n| n.to_str()), Some("a.zip"));
        assert_eq!(pending.staging.file_name().and_then(|n| n.to_str()), Some("staging"));
    }
}
