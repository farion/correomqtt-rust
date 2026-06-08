use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSurfaceSnapshot {
    pub active_tab: PluginSurfaceTab,
    pub load_state: PluginLoadState,
    pub plugin_filter: String,
    pub diagnostic_filter: String,
    pub plugins: Vec<PluginRow>,
    pub selected_plugin_id: String,
    pub selected_diagnostic_id: Option<String>,
    pub feedback: Option<PluginFeedback>,
    pub disable_confirmation: Option<PluginDisableConfirmation>,
    pub hook_editor: Option<PluginHookEditor>,
}

impl PluginSurfaceSnapshot {
    pub fn selected_plugin(&self) -> Option<&PluginRow> {
        self.plugins
            .iter()
            .find(|plugin| plugin.id == self.selected_plugin_id)
    }

    pub fn filtered_plugins(&self) -> Vec<&PluginRow> {
        if self.load_state != PluginLoadState::Ready {
            return Vec::new();
        }

        let filter = self.plugin_filter.trim().to_ascii_lowercase();
        self.plugins
            .iter()
            .filter(|plugin| {
                filter.is_empty()
                    || plugin.name.to_ascii_lowercase().contains(&filter)
                    || plugin.id.to_ascii_lowercase().contains(&filter)
                    || plugin
                        .capabilities
                        .iter()
                        .any(|capability| capability.label.to_ascii_lowercase().contains(&filter))
            })
            .collect()
    }

    pub fn diagnostics(&self) -> Vec<&PluginDiagnosticRow> {
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.diagnostics.iter())
            .collect()
    }

    pub fn filtered_diagnostics(&self) -> Vec<&PluginDiagnosticRow> {
        let filter = self.diagnostic_filter.trim().to_ascii_lowercase();
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.diagnostics.iter())
            .filter(|diagnostic| {
                filter.is_empty()
                    || diagnostic.plugin_id.to_ascii_lowercase().contains(&filter)
                    || diagnostic.message.to_ascii_lowercase().contains(&filter)
                    || diagnostic.detail.to_ascii_lowercase().contains(&filter)
                    || diagnostic
                        .hook
                        .is_some_and(|hook| hook.label().to_ascii_lowercase().contains(&filter))
            })
            .collect()
    }

    pub fn selected_diagnostic(&self) -> Option<&PluginDiagnosticRow> {
        let selected = self.selected_diagnostic_id.as_ref()?;
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.diagnostics.iter())
            .find(|diagnostic| &diagnostic.id == selected)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginLoadState {
    Loading,
    Empty,
    #[default]
    Ready,
}

impl PluginLoadState {
    pub fn message(self) -> &'static str {
        match self {
            Self::Loading => "Loading plugin manifests...",
            Self::Empty => {
                "No plugins installed. Bundled replacements can be restored from settings."
            }
            Self::Ready => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRow {
    pub id: String,
    pub name: String,
    pub version: String,
    pub provider: String,
    pub source: PluginSource,
    pub enabled: bool,
    pub status: PluginStatus,
    pub capabilities: Vec<PluginCapabilityRow>,
    pub config_fields: Vec<PluginConfigField>,
    pub hooks: Vec<PluginHookAssignment>,
    pub diagnostics: Vec<PluginDiagnosticRow>,
    pub legacy_note: Option<String>,
}

impl PluginRow {
    pub fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginSurfaceTab {
    #[default]
    Installed,
    Configuration,
    Hooks,
    Diagnostics,
}

impl PluginSurfaceTab {
    pub const ALL: [Self; 4] = [
        Self::Installed,
        Self::Configuration,
        Self::Hooks,
        Self::Diagnostics,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Installed => "Installed",
            Self::Configuration => "Configuration",
            Self::Hooks => "Hooks",
            Self::Diagnostics => "Diagnostics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginSource {
    Bundled,
    UserManifest,
    LegacyJava,
}

impl PluginSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bundled => "Bundled WASM",
            Self::UserManifest => "User manifest",
            Self::LegacyJava => "Legacy Java",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    Active,
    Disabled,
    NeedsConfig,
    CapabilityDenied,
    LoadError,
    HookFailed,
    UnsupportedLegacy,
}

impl PluginStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Disabled => "Disabled",
            Self::NeedsConfig => "Needs config",
            Self::CapabilityDenied => "Capability denied",
            Self::LoadError => "Load error",
            Self::HookFailed => "Hook failed",
            Self::UnsupportedLegacy => "Unsupported legacy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCapabilityRow {
    pub label: String,
    pub granted: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginConfigField {
    pub key: String,
    pub label: String,
    pub value: String,
    pub saved_value: String,
    pub required: bool,
    pub sensitive: bool,
    pub schema_hint: String,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookAssignment {
    pub hook: PluginHookKind,
    pub enabled: bool,
    pub target: String,
    pub config_json: String,
    pub status: PluginHookStatus,
    pub last_run: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginHookKind {
    IncomingTransform,
    OutgoingTransform,
    Validator,
    DetailTransform,
    DetailFormatter,
}

impl PluginHookKind {
    pub const ALL: [Self; 5] = [
        Self::IncomingTransform,
        Self::OutgoingTransform,
        Self::Validator,
        Self::DetailTransform,
        Self::DetailFormatter,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::IncomingTransform => "Incoming transform",
            Self::OutgoingTransform => "Outgoing transform",
            Self::Validator => "Validator",
            Self::DetailTransform => "Detail transform",
            Self::DetailFormatter => "Detail formatter",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginHookStatus {
    Ready,
    Disabled,
    Denied,
    Failed,
}

impl PluginHookStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Disabled => "Disabled",
            Self::Denied => "Denied",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDiagnosticRow {
    pub id: String,
    pub plugin_id: String,
    pub severity: PluginDiagnosticSeverity,
    pub hook: Option<PluginHookKind>,
    pub message: String,
    pub detail: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl PluginDiagnosticSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginFeedback {
    pub severity: PluginFeedbackSeverity,
    pub message: String,
}

impl PluginFeedback {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: PluginFeedbackSeverity::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginFeedbackSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDisableConfirmation {
    pub plugin_id: String,
    pub plugin_name: String,
    pub active_hooks: Vec<PluginHookKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookEditor {
    pub plugin_id: String,
    pub plugin_name: String,
    pub original: Option<PluginHookDraft>,
    pub draft: PluginHookDraft,
    pub error: Option<String>,
}

impl PluginHookEditor {
    pub fn is_new(&self) -> bool {
        self.original.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHookDraft {
    pub hook: PluginHookKind,
    pub enabled: bool,
    pub target: String,
    pub config_json: String,
}

impl From<&PluginHookAssignment> for PluginHookDraft {
    fn from(assignment: &PluginHookAssignment) -> Self {
        Self {
            hook: assignment.hook,
            enabled: assignment.enabled,
            target: assignment.target.clone(),
            config_json: assignment.config_json.clone(),
        }
    }
}
