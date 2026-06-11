use serde::{Deserialize, Serialize};

#[path = "surfaces/migration_recovery.rs"]
mod migration_recovery;
#[path = "surfaces/plugins.rs"]
mod plugins;
#[path = "surfaces/settings.rs"]
mod settings;
#[path = "surfaces/transfer.rs"]
mod transfer;
pub use migration_recovery::*;
pub use plugins::*;
pub use settings::*;
pub use transfer::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSurfaceSnapshot {
    pub selected_connection: String,
    pub selected_connection_id: Option<String>,
    pub selected_script: String,
    pub script_filter: String,
    pub new_script_name: String,
    #[serde(default)]
    pub create_dialog_open: bool,
    #[serde(default)]
    pub create_error: Option<String>,
    pub rename_script_name: String,
    pub active_tab: ScriptDetailTab,
    pub scripts: Vec<ScriptRow>,
    pub running: bool,
    pub active_execution_id: Option<String>,
    pub selected_execution_id: Option<String>,
    pub executions: Vec<ScriptExecutionRow>,
    pub log_lines: Vec<ScriptLogLine>,
    pub feedback: Option<ScriptFeedback>,
    pub last_error: Option<ScriptExecutionError>,
    pub rename_dialog_open: bool,
    pub rename_error: Option<String>,
    pub delete_confirmation_open: bool,
}

impl ScriptSurfaceSnapshot {
    pub fn selected_script(&self) -> Option<&ScriptRow> {
        self.scripts
            .iter()
            .find(|script| script.name == self.selected_script)
    }

    pub fn selected_script_is_dirty(&self) -> bool {
        self.selected_script()
            .map(ScriptRow::needs_save)
            .unwrap_or(false)
    }

    pub fn can_run(&self) -> bool {
        !self.running && self.selected_script().is_some()
    }

    pub fn can_save(&self) -> bool {
        self.selected_script_is_dirty()
    }

    pub fn filtered_scripts(&self) -> Vec<&ScriptRow> {
        let filter = self.script_filter.trim().to_ascii_lowercase();
        self.scripts
            .iter()
            .filter(|script| {
                filter.is_empty()
                    || script.name.to_ascii_lowercase().contains(&filter)
                    || script.relative_path.to_ascii_lowercase().contains(&filter)
            })
            .collect()
    }

    pub fn selected_execution_id(&self) -> Option<&str> {
        self.selected_execution_id
            .as_deref()
            .or(self.active_execution_id.as_deref())
            .or_else(|| {
                self.executions
                    .first()
                    .map(|execution| execution.execution_id.as_str())
            })
    }

    pub fn running_execution_id(&self) -> Option<&str> {
        self.active_execution_id.as_deref().or_else(|| {
            self.executions
                .iter()
                .find(|execution| !execution.status.is_terminal())
                .map(|execution| execution.execution_id.as_str())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptRow {
    pub name: String,
    pub relative_path: String,
    pub status: ScriptFileStatus,
    pub execution_count: usize,
    pub source: String,
    pub saved_source: String,
    #[serde(default = "default_script_persisted")]
    pub persisted: bool,
}

impl ScriptRow {
    pub fn is_dirty(&self) -> bool {
        self.source != self.saved_source
    }

    pub fn needs_save(&self) -> bool {
        !self.persisted || self.is_dirty()
    }
}

fn default_script_persisted() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptExecutionRow {
    pub execution_id: String,
    pub script_name: String,
    pub status: ScriptExecutionStatus,
    pub duration: String,
    pub timestamp: String,
    pub error: Option<ScriptExecutionError>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptDetailTab {
    #[default]
    Editor,
    Executions,
}

impl ScriptDetailTab {
    pub const ALL: [Self; 2] = [Self::Editor, Self::Executions];

    pub fn label(self) -> &'static str {
        match self {
            Self::Editor => "Editor",
            Self::Executions => "Executions",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptFileStatus {
    Ready,
    Dirty,
    Running,
    Error,
}

impl ScriptFileStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Dirty => "Dirty",
            Self::Running => "Running",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptExecutionStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl ScriptExecutionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Succeeded => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptExecutionError {
    pub kind: ScriptExecutionErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptExecutionErrorKind {
    HostApi,
    JavaScriptGuest,
    Cancellation,
    MqttOperation,
    Runtime,
}

impl ScriptExecutionErrorKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::HostApi => "Host API",
            Self::JavaScriptGuest => "JavaScript",
            Self::Cancellation => "Cancellation",
            Self::MqttOperation => "MQTT",
            Self::Runtime => "Runtime",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptLogLine {
    pub execution_id: String,
    pub timestamp: String,
    pub level: ScriptLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptLogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl ScriptLogLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptFeedback {
    pub severity: ScriptFeedbackSeverity,
    pub message: String,
}

impl ScriptFeedback {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: ScriptFeedbackSeverity::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: ScriptFeedbackSeverity::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: ScriptFeedbackSeverity::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptFeedbackSeverity {
    Info,
    Warning,
    Error,
}
