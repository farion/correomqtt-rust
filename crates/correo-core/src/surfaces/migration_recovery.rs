use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRecoverySnapshot {
    pub state: MigrationRecoveryState,
    pub legacy_path: Option<String>,
    pub backup_name: Option<String>,
    pub backup_path_hint: Option<String>,
    pub backup_status: String,
    pub counts: MigrationRecoveryCounts,
    pub rows: Vec<MigrationRecoveryRow>,
    pub warnings: Vec<MigrationRecoveryWarning>,
    pub diagnostics: Vec<MigrationRecoveryDiagnostic>,
    pub current_stage: Option<MigrationApplyStage>,
    pub password_error: Option<MigrationPasswordError>,
    pub failure: Option<MigrationRecoveryFailure>,
    pub completion: Option<MigrationRecoveryCompletion>,
    pub empty_profile_confirmation_open: bool,
    pub secrets_skipped: bool,
}

impl MigrationRecoverySnapshot {
    pub fn detected(path: impl Into<String>) -> Self {
        Self {
            state: MigrationRecoveryState::NeedsDecision,
            legacy_path: Some(path.into()),
            backup_status: "Backup not created yet".to_owned(),
            counts: MigrationRecoveryCounts::default(),
            rows: Self::review_rows(),
            warnings: default_detection_warnings(),
            diagnostics: vec![MigrationRecoveryDiagnostic::info(
                MigrationDiagnosticCategory::Backup,
                "Legacy data detected; backup has not started.",
            )],
            ..Self::default()
        }
    }

    pub fn blocks_normal_shell(&self) -> bool {
        !matches!(self.state, MigrationRecoveryState::NotDetected)
    }

    pub fn active_step(&self) -> MigrationRecoveryStep {
        match self.state {
            MigrationRecoveryState::NotDetected
            | MigrationRecoveryState::Detecting
            | MigrationRecoveryState::NeedsDecision
            | MigrationRecoveryState::CreatingBackup
            | MigrationRecoveryState::Failed => MigrationRecoveryStep::Detect,
            MigrationRecoveryState::NeedsPassword => MigrationRecoveryStep::Unlock,
            MigrationRecoveryState::Reviewing | MigrationRecoveryState::RestoreConfirm => {
                MigrationRecoveryStep::Review
            }
            MigrationRecoveryState::Applying | MigrationRecoveryState::Restoring => {
                MigrationRecoveryStep::Apply
            }
            MigrationRecoveryState::Complete => MigrationRecoveryStep::Complete,
        }
    }

    pub fn selected_count(&self) -> usize {
        self.rows.iter().filter(|row| row.selected).count()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn plugin_compatibility_body() -> &'static str {
        "Java/PF4J plugins are not migrated in CorreoMQTT Beta. Old .jar files, PF4J metadata, plugin config, hook config, and protocol.xml were left in the backup."
    }

    pub fn plugin_replacement_body() -> &'static str {
        "Bundled Rust/WASM replacements were initialized where available. Review Plugins to enable replacements and rebuild hook assignments."
    }

    pub fn plugin_diagnostic() -> &'static str {
        "Legacy Java plugin state was left in the backup and Rust/WASM plugin manifests were initialized."
    }

    pub fn review_rows() -> Vec<MigrationRecoveryRow> {
        default_review_rows()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationRecoveryState {
    #[default]
    NotDetected,
    Detecting,
    NeedsDecision,
    CreatingBackup,
    NeedsPassword,
    Reviewing,
    Applying,
    Complete,
    Failed,
    RestoreConfirm,
    Restoring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationRecoveryStep {
    Detect,
    Unlock,
    Review,
    Apply,
    Complete,
}

impl MigrationRecoveryStep {
    pub const ALL: [Self; 5] = [
        Self::Detect,
        Self::Unlock,
        Self::Review,
        Self::Apply,
        Self::Complete,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Detect => "Detect",
            Self::Unlock => "Unlock",
            Self::Review => "Review",
            Self::Apply => "Apply",
            Self::Complete => "Complete",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRecoveryCounts {
    pub connections: usize,
    pub histories: usize,
    pub scripts: usize,
    pub plugin_artifacts_ignored: usize,
    pub warnings: usize,
    pub skipped_secrets: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRecoveryRow {
    pub id: String,
    pub task: MigrationRecoveryTask,
    pub label: String,
    pub detail: String,
    pub selected: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationRecoveryTask {
    Connections,
    Secrets,
    Histories,
    Scripts,
    Settings,
    Plugins,
}

impl MigrationRecoveryTask {
    pub fn label(self) -> &'static str {
        match self {
            Self::Connections => "Connections",
            Self::Secrets => "Secrets",
            Self::Histories => "Histories",
            Self::Scripts => "Scripts",
            Self::Settings => "Settings",
            Self::Plugins => "Plugins",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRecoveryWarning {
    pub kind: MigrationRecoveryWarningKind,
    pub message: String,
}

impl MigrationRecoveryWarning {
    pub fn new(kind: MigrationRecoveryWarningKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationRecoveryWarningKind {
    UnsupportedLegacyField,
    JavaPluginStateIgnored,
    HookConfigIgnored,
    SecretSkipped,
    ConnectionNeedsReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRecoveryDiagnostic {
    pub severity: MigrationDiagnosticSeverity,
    pub category: MigrationDiagnosticCategory,
    pub message: String,
}

impl MigrationRecoveryDiagnostic {
    pub fn info(category: MigrationDiagnosticCategory, message: impl Into<String>) -> Self {
        Self::new(MigrationDiagnosticSeverity::Info, category, message)
    }

    pub fn warning(category: MigrationDiagnosticCategory, message: impl Into<String>) -> Self {
        Self::new(MigrationDiagnosticSeverity::Warning, category, message)
    }

    pub fn error(category: MigrationDiagnosticCategory, message: impl Into<String>) -> Self {
        Self::new(MigrationDiagnosticSeverity::Error, category, message)
    }

    fn new(
        severity: MigrationDiagnosticSeverity,
        category: MigrationDiagnosticCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationDiagnosticCategory {
    Backup,
    Migration,
    Restore,
    Secret,
    LegacyField,
    Plugin,
    ConnectionReview,
    Keyring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationApplyStage {
    BackupVerified,
    ConfigWritten,
    SecretsImported,
    HistoriesCopied,
    ScriptsCopied,
    RustPluginsInitialized,
    DiagnosticsRecorded,
}

impl MigrationApplyStage {
    pub fn label(self) -> &'static str {
        match self {
            Self::BackupVerified => "Backup verified",
            Self::ConfigWritten => "Config written",
            Self::SecretsImported => "Secrets imported to OS keyring",
            Self::HistoriesCopied => "Histories copied",
            Self::ScriptsCopied => "Scripts copied",
            Self::RustPluginsInitialized => "Rust plugin state initialized",
            Self::DiagnosticsRecorded => "Diagnostics recorded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationPasswordError {
    WrongPassword,
    UnsupportedEncryption,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRecoveryFailure {
    pub stage: MigrationFailureStage,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationFailureStage {
    Backup,
    BeforeWrite,
    AfterWrite,
    Restore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationRecoveryCompletion {
    Success,
    PartialSuccess,
    RestoreSuccess,
}

fn default_review_rows() -> Vec<MigrationRecoveryRow> {
    [
        (
            "connections",
            MigrationRecoveryTask::Connections,
            "Connection profiles",
            "Profiles are migrated without exposing secrets.",
            None,
        ),
        (
            "secrets",
            MigrationRecoveryTask::Secrets,
            "Saved secrets",
            "Secrets import into the OS keyring after unlock.",
            Some("Secret must be restored before connecting."),
        ),
        (
            "histories",
            MigrationRecoveryTask::Histories,
            "Publish and subscription histories",
            "History files are copied into the beta profile.",
            None,
        ),
        (
            "scripts",
            MigrationRecoveryTask::Scripts,
            "JavaScript scripts",
            "Scripts and execution metadata are preserved.",
            None,
        ),
        (
            "settings",
            MigrationRecoveryTask::Settings,
            "Settings and UI state",
            "Known settings are mapped; unsupported fields are warnings.",
            Some("Imported connection needs review before first connect."),
        ),
        (
            "plugins",
            MigrationRecoveryTask::Plugins,
            "Java plugin state",
            MigrationRecoverySnapshot::plugin_replacement_body(),
            Some(MigrationRecoverySnapshot::plugin_compatibility_body()),
        ),
    ]
    .into_iter()
    .map(|(id, task, label, detail, warning)| MigrationRecoveryRow {
        id: id.to_owned(),
        task,
        label: label.to_owned(),
        detail: detail.to_owned(),
        selected: true,
        warning: warning.map(str::to_owned),
    })
    .collect()
}

fn default_detection_warnings() -> Vec<MigrationRecoveryWarning> {
    vec![MigrationRecoveryWarning::new(
        MigrationRecoveryWarningKind::JavaPluginStateIgnored,
        MigrationRecoverySnapshot::plugin_diagnostic(),
    )]
}
