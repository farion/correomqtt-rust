use correo_core::{
    ConnectDisabledReason, ConnectionSettingsTab, ConnectionState, KeyringState,
    LegacyMigrationStatus, SettingsSection, ThemeMode, Workspace,
};
use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

const EN_US: &str = include_str!("../i18n/en-US.ftl");
const DE_DE: &str = include_str!("../i18n/de-DE.ftl");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Locale {
    EnUs,
    DeDe,
}

impl Locale {
    fn from_setting(value: &str) -> Self {
        let normalized = value.trim().replace('_', "-").to_ascii_lowercase();
        if normalized == "system" {
            return system_locale();
        }
        if normalized == "de" || normalized.starts_with("de-") {
            Self::DeDe
        } else {
            Self::EnUs
        }
    }

    fn langid(self) -> LanguageIdentifier {
        match self {
            Self::EnUs => "en-US",
            Self::DeDe => "de-DE",
        }
        .parse()
        .expect("bundled locale id should parse")
    }

    fn source(self) -> &'static str {
        match self {
            Self::EnUs => EN_US,
            Self::DeDe => DE_DE,
        }
    }
}

pub(crate) struct I18n {
    locale: Locale,
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub(crate) fn new(language: &str) -> Self {
        let locale = Locale::from_setting(language);
        let resource = FluentResource::try_new(locale.source().to_owned())
            .expect("bundled Fluent catalog should parse");
        let mut bundle = FluentBundle::new(vec![locale.langid()]);
        bundle
            .add_resource(resource)
            .expect("bundled Fluent catalog should add cleanly");
        Self { locale, bundle }
    }

    pub(crate) fn set_language(&mut self, language: &str) {
        let locale = Locale::from_setting(language);
        if self.locale != locale {
            *self = Self::new(language);
        }
    }

    pub(crate) fn text(&self, key: &str) -> String {
        let Some(message) = self.bundle.get_message(key) else {
            return key.to_owned();
        };
        let Some(pattern) = message.value() else {
            return key.to_owned();
        };
        let mut errors = Vec::new();
        self.bundle
            .format_pattern(pattern, None, &mut errors)
            .into_owned()
    }

    pub(crate) fn workspace_label(&self, workspace: Workspace) -> String {
        self.text(match workspace {
            Workspace::Connections => "workspace-connections",
            Workspace::ImportExport => "workspace-import-export",
            Workspace::Scripts => "workspace-scripts",
            Workspace::Plugins => "workspace-plugins",
            Workspace::Diagnostics => "workspace-diagnostics",
            Workspace::Settings => "workspace-settings",
            Workspace::About => "workspace-about",
        })
    }

    pub(crate) fn theme_label(&self, mode: ThemeMode) -> String {
        self.text(match mode {
            ThemeMode::System => "theme-system",
            ThemeMode::Light => "theme-light",
            ThemeMode::Dark => "theme-dark",
        })
    }

    pub(crate) fn settings_section_label(&self, section: SettingsSection) -> String {
        self.text(match section {
            SettingsSection::Appearance => "settings-appearance",
            SettingsSection::Language => "settings-language",
            SettingsSection::Search => "settings-search",
            SettingsSection::Keyring => "settings-keyring",
            SettingsSection::Updates => "settings-updates",
            SettingsSection::Plugins => "settings-plugins",
            SettingsSection::Data => "settings-data",
        })
    }

    pub(crate) fn connection_state_label(&self, state: ConnectionState) -> String {
        self.text(match state {
            ConnectionState::Disconnected => "state-disconnected",
            ConnectionState::Connecting => "state-connecting",
            ConnectionState::Connected => "state-connected",
            ConnectionState::Reconnecting => "state-reconnecting",
            ConnectionState::Error => "state-error",
        })
    }

    pub(crate) fn connection_settings_tab_label(&self, tab: ConnectionSettingsTab) -> String {
        self.text(match tab {
            ConnectionSettingsTab::Mqtt => "connection-tab-mqtt",
            ConnectionSettingsTab::Tls => "connection-tab-tls",
            ConnectionSettingsTab::Proxy => "connection-tab-proxy",
            ConnectionSettingsTab::Lwt => "connection-tab-lwt",
        })
    }

    pub(crate) fn disabled_reason_label(&self, reason: ConnectDisabledReason) -> String {
        self.text(match reason {
            ConnectDisabledReason::AlreadyConnected => "disabled-already-connected",
            ConnectDisabledReason::MissingHost => "disabled-missing-host",
            ConnectDisabledReason::MissingSecret => "disabled-missing-secret",
            ConnectDisabledReason::Busy => "disabled-busy",
        })
    }

    pub(crate) fn legacy_migration_label(&self, status: LegacyMigrationStatus) -> String {
        self.text(match status {
            LegacyMigrationStatus::NotRun => "legacy-not-run",
            LegacyMigrationStatus::Detected => "legacy-detected",
            LegacyMigrationStatus::Skipped => "legacy-skipped",
            LegacyMigrationStatus::Complete => "legacy-complete",
            LegacyMigrationStatus::PartialSuccess => "legacy-partial-success",
            LegacyMigrationStatus::Failed => "legacy-failed",
            LegacyMigrationStatus::Restored => "legacy-restored",
        })
    }

    pub(crate) fn keyring_state_label(&self, state: KeyringState) -> String {
        self.text(match state {
            KeyringState::Available => "keyring-available",
            KeyringState::Locked => "keyring-locked",
            KeyringState::Unavailable => "keyring-unavailable",
            KeyringState::MigrationRequired => "keyring-migration-required",
        })
    }

    pub(crate) fn language_option_label(&self, id: &str, fallback: &str) -> String {
        match id {
            "system" => self.text("common-system"),
            "en_US" | "en-US" => self.text("language-english"),
            "de_DE" | "de-DE" => self.text("language-german"),
            _ => fallback.to_owned(),
        }
    }
}

fn system_locale() -> Locale {
    std::env::var("LANG")
        .ok()
        .as_deref()
        .map(Locale::from_setting)
        .unwrap_or(Locale::EnUs)
}

#[cfg(test)]
mod tests {
    use super::I18n;
    use correo_core::{ThemeMode, Workspace};

    #[test]
    fn german_catalog_uses_original_settings_label() {
        let i18n = I18n::new("de_DE");

        assert_eq!(i18n.text("settings-header"), "Einstellungen für CorreoMQTT");
        assert_eq!(i18n.text("common-save"), "Speichern");
        assert_eq!(i18n.workspace_label(Workspace::Settings), "Einstellungen");
        assert_eq!(i18n.theme_label(ThemeMode::Dark), "Dunkel");
    }

    #[test]
    fn unsupported_locale_falls_back_to_english() {
        let i18n = I18n::new("fr_FR");

        assert_eq!(i18n.text("settings-header"), "Settings for CorreoMQTT");
        assert_eq!(i18n.workspace_label(Workspace::Connections), "Connections");
    }
}
