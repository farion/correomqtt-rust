use crate::{
    normalize_keyring_backend, AppModel, Diagnostic, GlobalSettingField, GlobalSettingFlag,
    GlobalSettingsSnapshot, SettingsFeedback, SettingsSection, ThemeMode,
};

impl AppModel {
    pub(super) fn set_theme_mode(&mut self, mode: ThemeMode) {
        if self.snapshot.theme_mode != mode {
            self.snapshot.theme_mode = mode;
            self.mark_global_settings_dirty();
        }
    }

    pub(super) fn load_global_settings(&mut self, settings: GlobalSettingsSnapshot) {
        self.saved_global_settings = settings.clone();
        self.snapshot.global_settings = settings;
    }

    pub(super) fn select_global_settings_section(&mut self, section: SettingsSection) {
        self.snapshot.global_settings.selected_section = section;
    }

    pub(super) fn update_global_setting(&mut self, field: GlobalSettingField, value: String) {
        let settings = &mut self.snapshot.global_settings;
        match field {
            GlobalSettingField::Language => settings.language = value,
            GlobalSettingField::KeyringBackend => {
                settings.keyring_backend = normalize_keyring_backend(value);
            }
            GlobalSettingField::BundledPluginsUrl => settings.bundled_plugins_url = value,
        }
        self.mark_global_settings_dirty();
    }

    pub(super) fn set_global_setting_flag(&mut self, flag: GlobalSettingFlag, enabled: bool) {
        let settings = &mut self.snapshot.global_settings;
        match flag {
            GlobalSettingFlag::UseRegexForSearch => settings.search_use_regex = enabled,
            GlobalSettingFlag::UseIgnoreCase => settings.search_ignore_case = enabled,
            GlobalSettingFlag::SearchUpdates => settings.update_checks_enabled = enabled,
            GlobalSettingFlag::UseDefaultPluginRepository => {
                settings.use_default_plugin_repository = enabled;
            }
            GlobalSettingFlag::InstallBundledPlugins => {
                settings.install_bundled_plugins = enabled;
            }
        }
        self.mark_global_settings_dirty();
    }

    pub(super) fn save_global_settings(&mut self) {
        if !self.snapshot.global_settings.dirty {
            self.snapshot.global_settings.feedback = Some(SettingsFeedback::info(
                "No global settings changes to save.",
            ));
            return;
        }

        self.snapshot.global_settings.dirty = false;
        self.snapshot.global_settings.feedback = Some(SettingsFeedback::info(
            "Global settings save command queued.",
        ));
        self.saved_global_settings = self.snapshot.global_settings.clone();
        self.saved_theme_mode = self.snapshot.theme_mode;
        self.push_diagnostic(Diagnostic::info("Global settings save command queued."));
    }

    pub(super) fn discard_global_settings(&mut self) {
        let section = self.snapshot.global_settings.selected_section;
        self.snapshot.global_settings = self.saved_global_settings.clone();
        self.snapshot.global_settings.selected_section = section;
        self.snapshot.global_settings.feedback =
            Some(SettingsFeedback::info("Global settings changes discarded."));
        self.snapshot.theme_mode = self.saved_theme_mode;
        self.push_diagnostic(Diagnostic::info("Global settings changes discarded."));
    }

    fn mark_global_settings_dirty(&mut self) {
        self.snapshot.global_settings.dirty = true;
        self.snapshot.global_settings.feedback = None;
    }
}
