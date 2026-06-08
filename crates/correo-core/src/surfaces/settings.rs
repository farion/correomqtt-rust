use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsSection {
    #[default]
    Appearance,
    Language,
    Search,
    Keyring,
    Updates,
    Plugins,
    Data,
}

impl SettingsSection {
    pub const ALL: [Self; 7] = [
        Self::Appearance,
        Self::Language,
        Self::Search,
        Self::Keyring,
        Self::Updates,
        Self::Plugins,
        Self::Data,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Language => "Language",
            Self::Search => "Search",
            Self::Keyring => "Keyring",
            Self::Updates => "Updates",
            Self::Plugins => "Plugins",
            Self::Data => "Data",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalSettingField {
    Language,
    KeyringBackend,
    BundledPluginsUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalSettingFlag {
    UseRegexForSearch,
    UseIgnoreCase,
    SearchUpdates,
    UseDefaultPluginRepository,
    InstallBundledPlugins,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRepositoryRow {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsFeedbackKind {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsFeedback {
    pub kind: SettingsFeedbackKind,
    pub message: String,
}

impl SettingsFeedback {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            kind: SettingsFeedbackKind::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            kind: SettingsFeedbackKind::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            kind: SettingsFeedbackKind::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSettingsSnapshot {
    pub selected_section: SettingsSection,
    pub language: String,
    pub language_options: Vec<SettingsOption>,
    pub keyring_backend: String,
    pub keyring_options: Vec<SettingsOption>,
    pub search_use_regex: bool,
    pub search_ignore_case: bool,
    pub update_checks_enabled: bool,
    pub last_update_check: String,
    pub use_default_plugin_repository: bool,
    pub install_bundled_plugins: bool,
    pub bundled_plugins_url: String,
    pub plugin_repositories: Vec<PluginRepositoryRow>,
    pub first_start: bool,
    pub config_version: String,
    pub window_geometry: String,
    pub cleanup_status: String,
    pub legacy_migration: LegacyMigrationSettingsSnapshot,
    pub dirty: bool,
    pub feedback: Option<SettingsFeedback>,
}

impl Default for GlobalSettingsSnapshot {
    fn default() -> Self {
        Self {
            selected_section: SettingsSection::Appearance,
            language: "system".to_owned(),
            language_options: default_language_options(),
            keyring_backend: "os".to_owned(),
            keyring_options: default_keyring_options(),
            search_use_regex: false,
            search_ignore_case: false,
            update_checks_enabled: false,
            last_update_check: "Not checked this session".to_owned(),
            use_default_plugin_repository: true,
            install_bundled_plugins: true,
            bundled_plugins_url: String::new(),
            plugin_repositories: Vec::new(),
            first_start: true,
            config_version: "unknown".to_owned(),
            window_geometry: "No saved window geometry".to_owned(),
            cleanup_status: "Sensitive values remain outside the UI snapshot".to_owned(),
            legacy_migration: LegacyMigrationSettingsSnapshot::default(),
            dirty: false,
            feedback: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyMigrationSettingsSnapshot {
    pub status: LegacyMigrationStatus,
    pub last_status: String,
    pub legacy_path_hint: Option<String>,
    pub backup_name: Option<String>,
    pub backup_path_hint: Option<String>,
    pub diagnostics_available: bool,
    pub restore_available: bool,
    pub warning_count: usize,
}

impl Default for LegacyMigrationSettingsSnapshot {
    fn default() -> Self {
        Self {
            status: LegacyMigrationStatus::NotRun,
            last_status: "No legacy migration recorded".to_owned(),
            legacy_path_hint: None,
            backup_name: None,
            backup_path_hint: None,
            diagnostics_available: false,
            restore_available: false,
            warning_count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegacyMigrationStatus {
    NotRun,
    Detected,
    Skipped,
    Complete,
    PartialSuccess,
    Failed,
    Restored,
}

impl LegacyMigrationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotRun => "Not run",
            Self::Detected => "Legacy data detected",
            Self::Skipped => "Skipped",
            Self::Complete => "Complete",
            Self::PartialSuccess => "Completed with warnings",
            Self::Failed => "Failed",
            Self::Restored => "Backup restored",
        }
    }
}

fn default_language_options() -> Vec<SettingsOption> {
    options(&[
        ("system", "System"),
        ("en_US", "English"),
        ("de_DE", "Deutsch"),
    ])
}

fn default_keyring_options() -> Vec<SettingsOption> {
    options(&[
        ("os", "OS keyring"),
        ("WinDPAPI", "Windows DPAPI"),
        ("OSXKeychain", "macOS Keychain"),
        ("LibSecret", "LibSecret"),
        ("KWallet5", "KWallet 5"),
        ("UserInput", "Prompt on startup"),
    ])
}

fn options(values: &[(&str, &str)]) -> Vec<SettingsOption> {
    values
        .iter()
        .map(|(id, label)| SettingsOption {
            id: (*id).to_owned(),
            label: (*label).to_owned(),
        })
        .collect()
}
