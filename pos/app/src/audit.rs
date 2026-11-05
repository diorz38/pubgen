use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::control_client::ControlClient;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditIssue {
    pub severity: String, // "error", "warning", "info"
    pub kind: String,
    pub message: String,
    pub item: Option<u64>,
    pub page: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuditReport {
    pub profile: String,
    pub errors: usize,
    pub warnings: usize,
    pub issues: Vec<AuditIssue>,
}

impl AuditReport {
    pub fn is_clean(&self) -> bool {
        self.errors == 0 && self.warnings == 0
    }
}

pub fn parse_audit_report(val: &Value) -> Result<AuditReport> {
    let profile = val.get("profile").and_then(Value::as_str).unwrap_or("print").to_string();
    let issues_val = val.get("issues").cloned().unwrap_or(Value::Array(Vec::new()));
    let issues: Vec<AuditIssue> = serde_json::from_value(issues_val).unwrap_or_default();
    
    let errors = val.get("errors").and_then(Value::as_u64).map(|n| n as usize).unwrap_or_else(|| {
        issues.iter().filter(|i| i.severity.eq_ignore_ascii_case("error")).count()
    });
    let warnings = val.get("warnings").and_then(Value::as_u64).map(|n| n as usize).unwrap_or_else(|| {
        issues.iter().filter(|i| i.severity.eq_ignore_ascii_case("warning")).count()
    });

    Ok(AuditReport {
        profile,
        errors,
        warnings,
        issues,
    })
}

pub fn run_pubgen_audit(client: &mut ControlClient, profile: &str) -> Result<AuditReport> {
    let reply = client.execute("preflight.pubgen", &json!({
        "profile": profile,
        "minPpi": 300
    }))?;
    parse_audit_report(&reply)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clean_report() {
        let json_data = json!({
            "profile": "print",
            "errors": 0,
            "warnings": 0,
            "issues": []
        });
        let report = parse_audit_report(&json_data).unwrap();
        assert_eq!(report.profile, "print");
        assert_eq!(report.errors, 0);
        assert_eq!(report.warnings, 0);
        assert!(report.is_clean());
    }

    #[test]
    fn test_parse_issues_report() {
        let json_data = json!({
            "profile": "print",
            "errors": 1,
            "warnings": 1,
            "issues": [
                {
                    "severity": "error",
                    "kind": "missing-source",
                    "message": "Tabel 1.1 missing Sumber line",
                    "item": 42,
                    "page": 3
                },
                {
                    "severity": "warning",
                    "kind": "overset",
                    "message": "Text frame overflow in story 10",
                    "item": 15,
                    "page": 2
                }
            ]
        });
        let report = parse_audit_report(&json_data).unwrap();
        assert_eq!(report.profile, "print");
        assert_eq!(report.errors, 1);
        assert_eq!(report.warnings, 1);
        assert_eq!(report.issues.len(), 2);
        assert!(!report.is_clean());
        assert_eq!(report.issues[0].kind, "missing-source");
        assert_eq!(report.issues[0].page, Some(3));
    }
}

