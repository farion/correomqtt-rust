use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use correo_storage::current::{ConfigStore, Settings};
use thiserror::Error;

use crate::{GlobalSettingsSnapshot, PluginRepositoryRow, ThemeMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceCommand {
    Save {
        theme_mode: ThemeMode,
        settings: GlobalSettingsSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceEvent {
    Saved,
    Failed { error: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SettingsDispatchError {
    #[error("settings persistence worker is stopped")]
    Stopped,
}

#[derive(Debug)]
pub struct SettingsPersistenceWorker {
    sender: Sender<SettingsPersistenceCommand>,
    events: Receiver<SettingsPersistenceEvent>,
}

impl SettingsPersistenceWorker {
    pub fn start(root: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let store = ConfigStore::new(root.into());

        std::thread::spawn(move || {
            while let Ok(command) = receiver.recv() {
                let event = apply_settings_command(&store, command);
                let _ = events_sender.send(event);
            }
        });

        Self { sender, events }
    }

    pub fn dispatch(
        &self,
        command: SettingsPersistenceCommand,
    ) -> Result<(), SettingsDispatchError> {
        self.sender
            .send(command)
            .map_err(|_| SettingsDispatchError::Stopped)
    }

    pub fn try_recv_event(&self) -> Option<SettingsPersistenceEvent> {
        self.events.try_recv().ok()
    }

    pub fn recv_event_timeout(&self, timeout: Duration) -> Option<SettingsPersistenceEvent> {
        match self.events.recv_timeout(timeout) {
            Ok(event) => Some(event),
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => None,
        }
    }
}

fn apply_settings_command(
    store: &ConfigStore,
    command: SettingsPersistenceCommand,
) -> SettingsPersistenceEvent {
    let result = match command {
        SettingsPersistenceCommand::Save {
            theme_mode,
            settings,
        } => store.save_global_settings(theme_name(theme_mode), storage_settings(settings)),
    };

    match result {
        Ok(_) => SettingsPersistenceEvent::Saved,
        Err(error) => SettingsPersistenceEvent::Failed {
            error: error.to_string(),
        },
    }
}

fn storage_settings(snapshot: GlobalSettingsSnapshot) -> Settings {
    let mut settings = Settings::default();
    settings.saved_locale = locale(snapshot.language);
    settings.use_regex_for_search = snapshot.search_use_regex;
    settings.use_ignore_case = snapshot.search_ignore_case;
    settings.search_updates = snapshot.update_checks_enabled;
    settings.use_default_repo = snapshot.use_default_plugin_repository;
    settings.install_bundled_plugins = snapshot.install_bundled_plugins;
    settings.bundled_plugins_url = non_empty(snapshot.bundled_plugins_url);
    settings.plugin_repositories = snapshot
        .plugin_repositories
        .into_iter()
        .map(repository_entry)
        .collect();
    settings.first_start = snapshot.first_start;
    settings.keyring_identifier = keyring_identifier(snapshot.keyring_backend);
    settings.config_created_with_correo_version = non_unknown(snapshot.config_version);
    settings
}

fn repository_entry(row: PluginRepositoryRow) -> (String, String) {
    (row.id, row.url)
}

fn locale(value: String) -> Option<String> {
    (value != "system").then_some(value)
}

fn keyring_identifier(value: String) -> Option<String> {
    (value != "os").then_some(value)
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn non_unknown(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty() && trimmed != "unknown").then(|| trimmed.to_owned())
}

fn theme_name(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "System",
        ThemeMode::Light => "Light",
        ThemeMode::Dark => "Dark",
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use correo_storage::current::ConfigStore;

    use crate::{
        GlobalSettingFlag, GlobalSettingsSnapshot, SettingsPersistenceCommand,
        SettingsPersistenceEvent, SettingsPersistenceWorker, ThemeMode,
    };

    #[test]
    fn worker_persists_global_settings_off_the_caller_thread() {
        let temp = tempfile::tempdir().unwrap();
        let worker = SettingsPersistenceWorker::start(temp.path());
        let mut settings = GlobalSettingsSnapshot::default();
        settings.language = "de_DE".to_owned();
        settings.search_use_regex = true;
        settings.search_ignore_case = true;
        settings.keyring_backend = "LibSecret".to_owned();

        worker
            .dispatch(SettingsPersistenceCommand::Save {
                theme_mode: ThemeMode::Dark,
                settings,
            })
            .unwrap();

        assert_eq!(
            worker.recv_event_timeout(Duration::from_secs(2)),
            Some(SettingsPersistenceEvent::Saved)
        );

        let config = ConfigStore::new(temp.path()).load().unwrap();
        assert_eq!(config.settings.saved_locale.as_deref(), Some("de_DE"));
        assert!(config.settings.use_regex_for_search);
        assert!(config.settings.use_ignore_case);
        assert_eq!(
            config.settings.keyring_identifier.as_deref(),
            Some("LibSecret")
        );
        assert_eq!(
            config
                .theme_settings
                .unwrap()
                .active_theme
                .unwrap()
                .name
                .as_deref(),
            Some("Dark")
        );
    }

    #[test]
    fn settings_flag_enum_stays_exhaustive_for_persistence() {
        let _ = GlobalSettingFlag::InstallBundledPlugins;
    }
}
