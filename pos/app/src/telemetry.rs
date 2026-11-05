use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const EXPERIMENT_ID: &str = "POS-EXP-001";
static TELEMETRY_MUTEX: Mutex<()> = Mutex::new(());
static OPT_OUT: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentVariant {
    #[serde(rename = "A_CONTROL")]
    Control, // Variant A: Linear / Direct Form
    #[serde(rename = "B_WIZARD")]
    Wizard,  // Variant B: Modal Step-by-Step Onboarding Wizard
}

impl ExperimentVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Control => "A_CONTROL",
            Self::Wizard => "B_WIZARD",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub experiment_id: String,
    pub client_id: String,
    pub variant: ExperimentVariant,
    pub event_name: String,
    pub timestamp_epoch_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_chosen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<serde_json::Value>,
}

pub struct TelemetryTracker {
    client_id: String,
    variant: ExperimentVariant,
    log_path: PathBuf,
}

impl TelemetryTracker {
    pub fn init() -> Self {
        let log_dir = default_telemetry_dir();
        let _ = std::fs::create_dir_all(&log_dir);

        let id_file = log_dir.join("client_id.txt");
        let client_id = if let Ok(existing) = std::fs::read_to_string(&id_file) {
            let trimmed = existing.trim().to_string();
            if !trimmed.is_empty() {
                trimmed
            } else {
                Self::generate_and_store_client_id(&id_file)
            }
        } else {
            Self::generate_and_store_client_id(&id_file)
        };

        let variant = Self::assign_variant(&client_id);
        let log_path = log_dir.join("telemetry_events.jsonl");

        Self {
            client_id,
            variant,
            log_path,
        }
    }

    fn generate_and_store_client_id(path: &PathBuf) -> String {
        let ts = current_epoch_ms();
        let rand_val = std::process::id();
        let raw = format!("{}:{}:{:?}", ts, rand_val, std::env::var("COMPUTERNAME").ok());
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        raw.hash(&mut hasher);
        let hash = hasher.finish();
        let client_id = format!("usr_{:016x}", hash);
        let _ = std::fs::write(path, &client_id);
        client_id
    }

    pub fn assign_variant(client_id: &str) -> ExperimentVariant {
        // Allow environment override for testing / manual audit
        if let Ok(forced) = std::env::var("POS_EXPERIMENT_VARIANT") {
            if forced.eq_ignore_ascii_case("A") || forced.eq_ignore_ascii_case("CONTROL") {
                return ExperimentVariant::Control;
            }
            if forced.eq_ignore_ascii_case("B") || forced.eq_ignore_ascii_case("WIZARD") {
                return ExperimentVariant::Wizard;
            }
        }

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        client_id.hash(&mut hasher);
        EXPERIMENT_ID.hash(&mut hasher);
        let hash = hasher.finish();

        if hash % 2 == 0 {
            ExperimentVariant::Control
        } else {
            ExperimentVariant::Wizard
        }
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn variant(&self) -> ExperimentVariant {
        self.variant
    }

    pub fn set_opt_out(opt_out: bool) {
        OPT_OUT.store(opt_out, Ordering::SeqCst);
    }

    pub fn is_opt_out() -> bool {
        OPT_OUT.load(Ordering::SeqCst)
    }

    pub fn record_event(&self, event_name: &str, profile: Option<&str>, duration_ms: Option<u64>, step: Option<usize>) {
        if Self::is_opt_out() {
            return;
        }

        let event = TelemetryEvent {
            experiment_id: EXPERIMENT_ID.to_string(),
            client_id: self.client_id.clone(),
            variant: self.variant,
            event_name: event_name.to_string(),
            timestamp_epoch_ms: current_epoch_ms(),
            profile_chosen: profile.map(|s| s.to_string()),
            duration_ms,
            step,
            extra_data: None,
        };

        if let Ok(serialized) = serde_json::to_string(&event) {
            let _lock = TELEMETRY_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.log_path) {
                let _ = writeln!(file, "{}", serialized);
            }
        }
    }

    pub fn track_setup_started(&self, source: &str) {
        let event = TelemetryEvent {
            experiment_id: EXPERIMENT_ID.to_string(),
            client_id: self.client_id.clone(),
            variant: self.variant,
            event_name: "project_setup_started".to_string(),
            timestamp_epoch_ms: current_epoch_ms(),
            profile_chosen: None,
            duration_ms: None,
            step: Some(1),
            extra_data: Some(serde_json::json!({ "source": source })),
        };
        if let Ok(serialized) = serde_json::to_string(&event) {
            let _lock = TELEMETRY_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.log_path) {
                let _ = writeln!(file, "{}", serialized);
            }
        }
    }

    pub fn track_setup_completed(&self, profile_id: &str, duration_ms: u64) {
        self.record_event("project_setup_completed", Some(profile_id), Some(duration_ms), None);
    }

    pub fn track_setup_cancelled(&self, last_step: usize, duration_ms: u64) {
        self.record_event("project_setup_cancelled", None, Some(duration_ms), Some(last_step));
    }
}

fn current_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn default_telemetry_dir() -> PathBuf {
    if let Some(app_data) = std::env::var_os("APPDATA") {
        PathBuf::from(app_data).join("PublicationOrchestratorStudio").join("telemetry")
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(".telemetry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_variant_assignment() {
        let id_a = "usr_001_test_client";
        let v1 = TelemetryTracker::assign_variant(id_a);
        let v2 = TelemetryTracker::assign_variant(id_a);
        assert_eq!(v1, v2, "Assignment must be deterministic for identical client_id");
    }

    #[test]
    fn test_event_serialization() {
        let event = TelemetryEvent {
            experiment_id: EXPERIMENT_ID.to_string(),
            client_id: "usr_test_123".to_string(),
            variant: ExperimentVariant::Wizard,
            event_name: "project_setup_completed".to_string(),
            timestamp_epoch_ms: 1730717900000,
            profile_chosen: Some("bps_dda".to_string()),
            duration_ms: Some(15420),
            step: None,
            extra_data: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("POS-EXP-001"));
        assert!(json.contains("B_WIZARD"));
        assert!(json.contains("bps_dda"));
        assert!(json.contains("15420"));
    }

    #[test]
    fn test_file_logging_and_reading() {
        let temp_dir = std::env::temp_dir().join(format!("pos_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let log_file = temp_dir.join("test_events.jsonl");

        let tracker = TelemetryTracker {
            client_id: "usr_test_writer".to_string(),
            variant: ExperimentVariant::Wizard,
            log_path: log_file.clone(),
        };

        tracker.track_setup_started("test_suite");
        tracker.track_setup_completed("bps_dda", 3450);
        tracker.track_setup_cancelled(2, 1200);

        let content = std::fs::read_to_string(&log_file).expect("log file should exist");
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 3, "Expected 3 serialized events in jsonl");

        let ev1: TelemetryEvent = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(ev1.event_name, "project_setup_started");
        assert_eq!(ev1.variant, ExperimentVariant::Wizard);

        let ev2: TelemetryEvent = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(ev2.event_name, "project_setup_completed");
        assert_eq!(ev2.profile_chosen.as_deref(), Some("bps_dda"));
        assert_eq!(ev2.duration_ms, Some(3450));

        let ev3: TelemetryEvent = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(ev3.event_name, "project_setup_cancelled");
        assert_eq!(ev3.step, Some(2));
        assert_eq!(ev3.duration_ms, Some(1200));

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}

