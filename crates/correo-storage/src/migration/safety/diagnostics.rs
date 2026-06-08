use serde::{Deserialize, Serialize};

use super::super::{MigrationPreview, MigrationWarning};
use super::MigrationBackup;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationDiagnostics {
    pub mapped_fields: Vec<MappedLegacyField>,
    pub unmapped_fields: Vec<String>,
    pub warnings: Vec<MigrationDiagnostic>,
    pub recovery_steps: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappedLegacyField {
    pub legacy_path: String,
    pub current_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationDiagnostic {
    pub code: String,
    pub message: String,
}

impl MigrationDiagnostics {
    pub fn from_preview(preview: &MigrationPreview, backup: Option<&MigrationBackup>) -> Self {
        let mut diagnostics = Self {
            mapped_fields: mapped_legacy_fields(),
            unmapped_fields: preview
                .report
                .unsupported_fields
                .iter()
                .map(|field| redact_migration_text(&field.path))
                .collect(),
            warnings: preview
                .report
                .warnings
                .iter()
                .map(sanitized_warning)
                .collect(),
            recovery_steps: Vec::new(),
        };
        diagnostics
            .recovery_steps
            .push("Keep the timestamped backup until the migrated profile is accepted.".to_owned());
        diagnostics.recovery_steps.push(
            "Rollback restores only when the rollback marker still matches the migrated data."
                .to_owned(),
        );
        if let Some(backup) = backup {
            diagnostics
                .recovery_steps
                .push(format!("Backup created at {}", backup.path.display()));
        }
        diagnostics
    }

    pub(super) fn rollback_complete(backup: &MigrationBackup) -> Self {
        Self {
            mapped_fields: Vec::new(),
            unmapped_fields: Vec::new(),
            warnings: Vec::new(),
            recovery_steps: vec![format!(
                "Restored migration target from backup {}",
                backup.path.display()
            )],
        }
    }
}

fn sanitized_warning(warning: &MigrationWarning) -> MigrationDiagnostic {
    MigrationDiagnostic {
        code: warning.code.to_owned(),
        message: redact_migration_text(&warning.message),
    }
}

fn mapped_legacy_fields() -> Vec<MappedLegacyField> {
    [
        ("config.connections[].id", "config.connections[].id"),
        ("config.connections[].name", "config.connections[].name"),
        ("config.connections[].url", "config.connections[].url"),
        ("config.connections[].port", "config.connections[].port"),
        (
            "config.connections[].clientId",
            "config.connections[].client_id",
        ),
        ("config.settings", "config.settings"),
        (
            "themesSettings.activeTheme.name",
            "config.theme_settings.active_theme.name",
        ),
        ("*_publishHistory.json", "history.publish_topics"),
        ("*_publishMessageHistory.json", "history.publish_messages"),
        ("*_subscriptionHistory.json", "history.subscriptions"),
        ("scripts/**/*.js", "scripts.files"),
        ("scripts/executions/**/*.json", "scripts.executions"),
        ("scripts/logs/**/*.log", "scripts.logs"),
    ]
    .into_iter()
    .map(|(legacy_path, current_path)| MappedLegacyField {
        legacy_path: legacy_path.to_owned(),
        current_path: current_path.to_owned(),
    })
    .collect()
}

fn redact_migration_text(input: &str) -> String {
    input
        .lines()
        .map(redact_migration_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn redact_migration_line(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if lower.contains("-----begin") && lower.contains("private key") {
        return "[REDACTED KEY MATERIAL]".to_owned();
    }
    if !contains_sensitive_term(&lower) {
        return line.to_owned();
    }
    if let Some(index) = line.find(['=', ':']) {
        let (label, _) = line.split_at(index + 1);
        format!("{label} [REDACTED]")
    } else {
        "[REDACTED MIGRATION DIAGNOSTIC: sensitive material]".to_owned()
    }
}

fn contains_sensitive_term(lowercase_line: &str) -> bool {
    [
        "password",
        "passphrase",
        "private key",
        "key material",
        "key_material",
        "keymaterial",
        "decrypted password map",
        "export password",
    ]
    .iter()
    .any(|term| lowercase_line.contains(term))
}
