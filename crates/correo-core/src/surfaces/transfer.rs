use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStep {
    #[default]
    ChooseFile,
    Password,
    Review,
    Complete,
}

impl TransferStep {
    pub const ALL: [Self; 4] = [
        Self::ChooseFile,
        Self::Password,
        Self::Review,
        Self::Complete,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ChooseFile => "Choose file",
            Self::Password => "Password",
            Self::Review => "Review",
            Self::Complete => "Complete",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferSection {
    #[default]
    Import,
    Export,
    Messages,
}

impl TransferSection {
    pub const ALL: [Self; 3] = [Self::Import, Self::Export, Self::Messages];

    pub fn label(self) -> &'static str {
        match self {
            Self::Import => "Import .cqc",
            Self::Export => "Export .cqc",
            Self::Messages => "Messages",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferSurfaceSnapshot {
    pub active_section: TransferSection,
    pub active_step: TransferStep,
    pub import: ConnectionImportSnapshot,
    pub export: ConnectionExportSnapshot,
    pub messages: MessageTransferSnapshot,
    pub selected_connections: usize,
    pub warnings: Vec<String>,
    pub encrypted_export: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionImportSnapshot {
    pub file: Option<TransferFileSnapshot>,
    pub encrypted: bool,
    pub password_state: ImportPasswordState,
    pub rows: Vec<TransferConnectionRow>,
    pub warnings: Vec<String>,
    pub feedback: Option<TransferFeedback>,
    pub outcome: Option<TransferOutcome>,
}

impl ConnectionImportSnapshot {
    pub fn password_required(&self) -> bool {
        self.encrypted && self.password_state.requires_input()
    }

    pub fn selected_count(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.selected && row.status.importable())
            .count()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportPasswordState {
    #[default]
    NotNeeded,
    Needed,
    InvalidRecoverable,
    Accepted,
}

impl ImportPasswordState {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotNeeded => "Not needed",
            Self::Needed => "Password required",
            Self::InvalidRecoverable => "Password did not unlock file",
            Self::Accepted => "Password accepted",
        }
    }

    pub fn requires_input(self) -> bool {
        matches!(self, Self::Needed | Self::InvalidRecoverable)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionExportSnapshot {
    pub rows: Vec<TransferConnectionRow>,
    pub output_path: String,
    pub path_state: ExportPathState,
    pub encrypted: bool,
    pub password_confirmation: ExportPasswordConfirmation,
    pub feedback: Option<TransferFeedback>,
    pub outcome: Option<TransferOutcome>,
}

impl ConnectionExportSnapshot {
    pub fn selected_count(&self) -> usize {
        self.rows.iter().filter(|row| row.selected).count()
    }

    pub fn password_required(&self) -> bool {
        self.encrypted
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportPathState {
    #[default]
    Ready,
    MissingExtension,
    InvalidPath,
}

impl ExportPathState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "Path ready",
            Self::MissingExtension => "Missing .cqc extension",
            Self::InvalidPath => "Path is not writable",
        }
    }

    pub fn severity(self) -> Option<TransferSeverity> {
        match self {
            Self::Ready => None,
            Self::MissingExtension => Some(TransferSeverity::Warning),
            Self::InvalidPath => Some(TransferSeverity::Error),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportPasswordConfirmation {
    #[default]
    NotRequired,
    Needed,
    Mismatch,
    Confirmed,
}

impl ExportPasswordConfirmation {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotRequired => "Not required",
            Self::Needed => "Password and confirmation required",
            Self::Mismatch => "Password confirmation does not match",
            Self::Confirmed => "Password confirmation ready",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageTransferSnapshot {
    pub import_file: Option<TransferFileSnapshot>,
    pub export_path: String,
    pub selected_messages: usize,
    pub available_messages: usize,
    pub feedback: Option<TransferFeedback>,
    pub outcome: Option<TransferOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferFileSnapshot {
    pub display_name: String,
    pub path_hint: String,
    pub byte_size: usize,
    pub detected_connections: usize,
    pub encrypted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferConnectionRow {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub mqtt_version: String,
    pub selected: bool,
    pub status: TransferConnectionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferConnectionStatus {
    New,
    Update,
    Conflict,
    MissingSecret,
    Exportable,
}

impl TransferConnectionStatus {
    pub fn importable(self) -> bool {
        matches!(self, Self::New | Self::MissingSecret)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Update => "Update",
            Self::Conflict => "Conflict",
            Self::MissingSecret => "Needs secret",
            Self::Exportable => "Ready",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferFeedback {
    pub severity: TransferSeverity,
    pub message: String,
}

impl TransferFeedback {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: TransferSeverity::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: TransferSeverity::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: TransferSeverity::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferOutcome {
    pub success: bool,
    pub title: String,
    pub detail: String,
}

impl TransferOutcome {
    pub fn success(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            success: true,
            title: title.into(),
            detail: detail.into(),
        }
    }

    pub fn failure(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            success: false,
            title: title.into(),
            detail: detail.into(),
        }
    }
}
