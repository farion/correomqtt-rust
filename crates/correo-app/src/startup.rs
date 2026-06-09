use std::path::{Path, PathBuf};

use correo_core::{startup_state_from_current, startup_state_from_migration, Diagnostic};
use correo_core::{StartupState, ThemeMode};
use correo_storage::current::{
    AppConfig, ConfigStore, HistoryPersistenceSnapshot, HistoryStore, ScriptPersistenceSnapshot,
    ScriptStore,
};
use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::MigrationPreview;
use directories::ProjectDirs;

pub fn load_startup_state(fallback_theme: ThemeMode) -> StartupState {
    for root in current_roots() {
        if !root.join("config.json").exists() {
            continue;
        }

        return load_root(&root, fallback_theme).unwrap_or_else(|error| {
            StartupState::empty(
                fallback_theme,
                Diagnostic::error(format!(
                    "Existing CorreoMQTT config at {} could not be opened: {error}",
                    root.display()
                )),
            )
        });
    }

    for root in legacy_roots() {
        if root.join("config.json").exists() {
            return StartupState::legacy_migration_detected(
                fallback_theme,
                root.display().to_string(),
            );
        }
    }

    StartupState::empty(
        fallback_theme,
        Diagnostic::info("No existing CorreoMQTT config found; empty workspace ready."),
    )
}

pub fn history_root() -> PathBuf {
    if let Some(path) = std::env::var_os("CORREOMQTT_CONFIG_DIR") {
        return PathBuf::from(path);
    }
    ProjectDirs::from("org", "CorreoMQTT", "CorreoMQTT")
        .map(|project_dirs| project_dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".correomqtt"))
}

fn load_root(root: &Path, fallback_theme: ThemeMode) -> Result<StartupState, String> {
    match read_current_config(root) {
        Ok(config) => {
            let histories = load_current_histories(root, &config)?;
            let scripts = load_current_scripts(root)?;
            Ok(startup_state_from_current(
                config,
                histories,
                scripts,
                Vec::new(),
                fallback_theme,
            ))
        }
        Err(current_error) => match LegacyProfile::read_from(root) {
            Ok(profile) => {
                let preview = MigrationPreview::from_legacy_profile(profile)
                    .map_err(|error| error.to_string())?;
                Ok(startup_state_from_migration(preview, fallback_theme))
            }
            Err(legacy_error) => Err(format!(
                "current config error: {current_error}; legacy migration error: {legacy_error}"
            )),
        },
    }
}

fn load_current_scripts(root: &Path) -> Result<ScriptPersistenceSnapshot, String> {
    ScriptStore::new(root)
        .load_snapshot(200)
        .map_err(|error| error.to_string())
}

fn read_current_config(root: &Path) -> Result<AppConfig, String> {
    ConfigStore::new(root)
        .load()
        .map_err(|error| error.to_string())
}

fn load_current_histories(
    root: &Path,
    config: &AppConfig,
) -> Result<HistoryPersistenceSnapshot, String> {
    let store = HistoryStore::new(root);
    let mut histories = HistoryPersistenceSnapshot::default();
    for connection in &config.connections {
        let history = store
            .load_connection(&connection.id)
            .map_err(|error| error.to_string())?;
        histories.connections.insert(connection.id.clone(), history);
    }
    Ok(histories)
}

fn current_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(path) = std::env::var_os("CORREOMQTT_CONFIG_DIR") {
        roots.push(PathBuf::from(path));
    }
    if let Some(project_dirs) = ProjectDirs::from("org", "CorreoMQTT", "CorreoMQTT") {
        roots.push(project_dirs.data_dir().to_path_buf());
    }
    dedup_existing_order(roots)
}

fn legacy_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        roots.push(PathBuf::from(appdata).join("CorreoMqtt"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        roots.push(
            home.join("Library")
                .join("Application Support")
                .join("CorreoMqtt"),
        );
        roots.push(home.join(".correomqtt"));
    }
    roots
}

fn dedup_existing_order(roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();
    for root in roots {
        if !deduped.iter().any(|existing| existing == &root) {
            deduped.push(root);
        }
    }
    deduped
}
