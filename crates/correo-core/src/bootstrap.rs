use std::collections::HashMap;

use correo_mqtt::ConnectionId;
use correo_storage::current::{
    AppConfig, Auth, ConnectionConfig, ConnectionHistorySnapshot, HistoryPersistenceSnapshot, Lwt,
    MqttVersion, Proxy, Qos as StorageQos, Settings, ThemeSettings, TlsSsl,
};
use correo_storage::migration::MigrationPreview;

use crate::{
    AppSnapshot, ConnectDisabledReason, ConnectionBadge, ConnectionSettingsSnapshot,
    ConnectionState, ConnectionSummary, Diagnostic, GlobalSettingsSnapshot, KeyringState,
    LegacyMigrationStatus, MigrationRecoverySnapshot, PluginRepositoryRow, PublishHistoryRow,
    QosLevel, SubscribePaneSnapshot, SubscriptionRow, ThemeMode,
};

#[derive(Debug, Clone)]
pub struct StartupState {
    pub snapshot: AppSnapshot,
    pub connection_settings: HashMap<ConnectionId, ConnectionSettingsSnapshot>,
    pub storage_connection_ids: HashMap<ConnectionId, String>,
}

impl StartupState {
    pub fn empty(theme_mode: ThemeMode, diagnostic: Diagnostic) -> Self {
        let mut snapshot = AppSnapshot::empty();
        snapshot.theme_mode = theme_mode;
        snapshot.diagnostics = vec![diagnostic.redacted()];
        Self {
            snapshot,
            connection_settings: HashMap::new(),
            storage_connection_ids: HashMap::new(),
        }
    }

    pub fn legacy_migration_detected(
        theme_mode: ThemeMode,
        legacy_path: impl Into<String>,
    ) -> Self {
        let legacy_path = legacy_path.into();
        let mut snapshot = AppSnapshot::empty();
        snapshot.theme_mode = theme_mode;
        snapshot.migration_recovery = MigrationRecoverySnapshot::detected(legacy_path.clone());
        snapshot.global_settings.legacy_migration.status = LegacyMigrationStatus::Detected;
        snapshot.global_settings.legacy_migration.last_status =
            LegacyMigrationStatus::Detected.label().to_owned();
        snapshot.global_settings.legacy_migration.legacy_path_hint = Some(legacy_path.clone());
        snapshot.diagnostics = vec![Diagnostic::info(format!(
            "Legacy CorreoMQTT data detected at {legacy_path}; migration is waiting for user choice."
        ))
        .redacted()];
        Self {
            snapshot,
            connection_settings: HashMap::new(),
            storage_connection_ids: HashMap::new(),
        }
    }
}

pub fn startup_state_from_current(
    config: AppConfig,
    histories: HistoryPersistenceSnapshot,
    warnings: Vec<String>,
    fallback_theme: ThemeMode,
) -> StartupState {
    let theme_mode = theme_mode(config.theme_settings.as_ref()).unwrap_or(fallback_theme);
    let mut snapshot = AppSnapshot::empty();
    let mut connection_settings = HashMap::new();
    let mut storage_connection_ids = HashMap::new();
    let mut mapped = Vec::new();

    for connection in &config.connections {
        let id = ConnectionId::new();
        let history = histories.connections.get(&connection.id);
        mapped.push(summary(id, connection, history));
        connection_settings.insert(id, settings_snapshot(connection, &warnings));
        storage_connection_ids.insert(id, connection.id.clone());
    }

    snapshot.connection_count = mapped.len();
    snapshot.selected_connection = mapped.first().map(|connection| connection.id);
    snapshot.connections = mapped;
    snapshot.theme_mode = theme_mode;
    snapshot.global_settings = global_settings(&config.settings);
    snapshot.diagnostics = warnings
        .into_iter()
        .map(|warning| Diagnostic::warning(warning).redacted())
        .collect();

    if let Some(selected) = snapshot.selected_connection {
        if let Some(settings) = connection_settings.get(&selected) {
            snapshot.connection_settings = settings.clone();
        }
        if let Some(storage_id) = config
            .connections
            .first()
            .map(|connection| connection.id.as_str())
        {
            apply_history(&mut snapshot, histories.connections.get(storage_id));
        }
    }

    StartupState {
        snapshot,
        connection_settings,
        storage_connection_ids,
    }
}

pub fn startup_state_from_migration(
    preview: MigrationPreview,
    fallback_theme: ThemeMode,
) -> StartupState {
    let warnings = preview
        .warnings
        .iter()
        .map(|warning| warning.message.clone())
        .collect();
    startup_state_from_current(
        AppConfig {
            connections: preview.connections,
            theme_settings: preview.theme_settings,
            settings: preview.settings,
        },
        preview.histories,
        warnings,
        fallback_theme,
    )
}

fn summary(
    id: ConnectionId,
    connection: &ConnectionConfig,
    history: Option<&ConnectionHistorySnapshot>,
) -> ConnectionSummary {
    let needs_secret = needs_secret_restore(connection);
    ConnectionSummary {
        id,
        name: connection.name.clone(),
        endpoint: format!("{}:{}", connection.url, connection.port),
        mqtt_version: mqtt_label(connection.mqtt_version).to_owned(),
        badges: badges(connection, needs_secret),
        state: ConnectionState::Disconnected,
        disabled_reason: needs_secret.then_some(ConnectDisabledReason::MissingSecret),
        recent_subscriptions: history
            .map(|history| history.subscriptions.topics.len())
            .unwrap_or_default(),
        recent_messages: history
            .map(|history| history.publish_messages.messages.len())
            .unwrap_or_default(),
        last_activity: history
            .map(history_activity)
            .unwrap_or_else(|| "Ready".to_owned()),
    }
}

fn settings_snapshot(
    connection: &ConnectionConfig,
    warnings: &[String],
) -> ConnectionSettingsSnapshot {
    let valid = !connection.name.trim().is_empty() && !connection.url.trim().is_empty();
    ConnectionSettingsSnapshot {
        profile_name: connection.name.clone(),
        host: connection.url.clone(),
        port: connection.port.to_string(),
        mqtt_version: mqtt_label(connection.mqtt_version).to_owned(),
        client_id: connection.client_id.clone().unwrap_or_default(),
        auth_mode: auth_label(connection).to_owned(),
        username_status: username_status(connection).to_owned(),
        tls_mode: tls_label(connection).to_owned(),
        tls_store: tls_store_label(connection).to_owned(),
        proxy_mode: proxy_label(connection).to_owned(),
        proxy_endpoint: proxy_endpoint(connection),
        lwt_enabled: connection.lwt == Lwt::On,
        lwt_topic: connection.lwt_topic.clone().unwrap_or_default(),
        lwt_payload: connection.lwt_payload.clone().unwrap_or_default(),
        advanced_options: advanced_options(connection),
        dirty: false,
        valid,
        save_disabled_reason: "No changes to save".to_owned(),
        keyring_state: if needs_secret_restore(connection) {
            KeyringState::MigrationRequired
        } else {
            KeyringState::Available
        },
        validation_errors: warnings.to_vec(),
        ..ConnectionSettingsSnapshot::default()
    }
}

fn apply_history(snapshot: &mut AppSnapshot, history: Option<&ConnectionHistorySnapshot>) {
    let Some(history) = history else {
        return;
    };
    snapshot.workbench.publish.topic_history = history.publish_topics.topics.clone();
    snapshot.workbench.publish.history = history
        .publish_messages
        .messages
        .iter()
        .map(|message| PublishHistoryRow {
            topic: message.topic.clone(),
            timestamp: message
                .date_time
                .clone()
                .unwrap_or_else(|| "migrated".to_owned()),
            qos: message.qos.map(qos).unwrap_or_default(),
            retained: message.retained,
            byte_size: message.payload.as_deref().map(str::len).unwrap_or_default(),
        })
        .collect();
    snapshot.workbench.subscribe = SubscribePaneSnapshot {
        topic_history: history.subscriptions.topics.clone(),
        subscriptions: history
            .subscriptions
            .topics
            .iter()
            .map(|topic| SubscriptionRow {
                topic_filter: topic.clone(),
                qos: QosLevel::Zero,
                message_count: 0,
                active: true,
            })
            .collect(),
        ..SubscribePaneSnapshot::default()
    };
}

fn global_settings(settings: &Settings) -> GlobalSettingsSnapshot {
    let mut snapshot = GlobalSettingsSnapshot {
        language: settings
            .saved_locale
            .clone()
            .or_else(|| settings.current_locale.clone())
            .unwrap_or_else(|| "system".to_owned()),
        keyring_backend: settings
            .keyring_identifier
            .clone()
            .unwrap_or_else(|| "os".to_owned()),
        update_checks_enabled: settings.search_updates,
        last_update_check: "Not checked this session".to_owned(),
        cleanup_status: "Sensitive values remain outside the UI snapshot".to_owned(),
        search_use_regex: settings.use_regex_for_search,
        search_ignore_case: settings.use_ignore_case,
        use_default_plugin_repository: settings.use_default_repo,
        install_bundled_plugins: settings.install_bundled_plugins,
        bundled_plugins_url: settings.bundled_plugins_url.clone().unwrap_or_default(),
        plugin_repositories: settings
            .plugin_repositories
            .iter()
            .map(|(id, url)| PluginRepositoryRow {
                id: id.clone(),
                url: url.clone(),
            })
            .collect(),
        first_start: settings.first_start,
        config_version: settings
            .config_created_with_correo_version
            .clone()
            .unwrap_or_else(|| "unknown".to_owned()),
        ..GlobalSettingsSnapshot::default()
    };
    if let Some(geometry) = &settings.global_ui_settings {
        snapshot.window_geometry = format!(
            "{:.0}x{:.0} at {:.0},{:.0}",
            geometry.window_width,
            geometry.window_height,
            geometry.window_position_x,
            geometry.window_position_y
        );
    }
    snapshot
}

fn badges(connection: &ConnectionConfig, needs_secret: bool) -> Vec<ConnectionBadge> {
    let mut badges = Vec::new();
    if connection.ssl != TlsSsl::Off {
        badges.push(ConnectionBadge::Tls);
    }
    if connection.proxy != Proxy::Off {
        badges.push(ConnectionBadge::Proxy);
    }
    if connection.lwt == Lwt::On {
        badges.push(ConnectionBadge::Lwt);
    }
    if needs_secret {
        badges.push(ConnectionBadge::KeyringWarning);
    }
    badges
}

fn needs_secret_restore(connection: &ConnectionConfig) -> bool {
    connection.username.is_some()
        || connection.auth == Auth::Password
        || connection.ssl_keystore.is_some()
}

fn theme_mode(settings: Option<&ThemeSettings>) -> Option<ThemeMode> {
    match settings
        .and_then(|settings| settings.active_theme.as_ref())
        .and_then(|theme| theme.name.as_deref())
    {
        Some("Light" | "light" | "Light Legacy" | "light_legacy") => Some(ThemeMode::Light),
        Some("Dark" | "dark" | "Dark Legacy" | "dark_legacy") => Some(ThemeMode::Dark),
        _ => None,
    }
}

fn mqtt_label(version: MqttVersion) -> &'static str {
    match version {
        MqttVersion::Mqtt311 => "MQTT 3.1.1",
        MqttVersion::Mqtt50 => "MQTT v5",
    }
}

fn qos(qos: StorageQos) -> QosLevel {
    match qos {
        StorageQos::AtMostOnce => QosLevel::Zero,
        StorageQos::AtLeastOnce => QosLevel::One,
        StorageQos::ExactlyOnce => QosLevel::Two,
    }
}

fn history_activity(history: &ConnectionHistorySnapshot) -> String {
    format!(
        "{} publish topic(s), {} subscription(s) migrated",
        history.publish_topics.topics.len(),
        history.subscriptions.topics.len()
    )
}

fn auth_label(connection: &ConnectionConfig) -> &'static str {
    if connection.username.is_some() {
        "Username/password in keyring"
    } else {
        match connection.auth {
            Auth::Off => "Disabled",
            Auth::Password => "SSH password in keyring",
            Auth::Keyfile => "SSH key file configured",
        }
    }
}

fn username_status(connection: &ConnectionConfig) -> &'static str {
    if connection.username.is_some() {
        "Username configured; password stays in keyring"
    } else {
        "No MQTT username configured"
    }
}

fn tls_label(connection: &ConnectionConfig) -> &'static str {
    match connection.ssl {
        TlsSsl::Off => "Disabled",
        TlsSsl::Keystore => "TLS enabled",
    }
}

fn tls_store_label(connection: &ConnectionConfig) -> &'static str {
    if connection.ssl_keystore.is_some() {
        "Keystore path configured"
    } else {
        "No certificate store selected"
    }
}

fn proxy_label(connection: &ConnectionConfig) -> &'static str {
    match connection.proxy {
        Proxy::Off => "Disabled",
        Proxy::Ssh => "SSH tunnel",
    }
}

fn proxy_endpoint(connection: &ConnectionConfig) -> String {
    match (&connection.ssh_host, connection.local_port) {
        (Some(host), Some(local_port)) => {
            format!("{host}:{} via localhost:{local_port}", connection.ssh_port)
        }
        (Some(host), None) => format!("{host}:{}", connection.ssh_port),
        _ => "No tunnel configured".to_owned(),
    }
}

fn advanced_options(connection: &ConnectionConfig) -> Vec<String> {
    vec![
        format!(
            "Clean session {}",
            if connection.clean_session {
                "enabled"
            } else {
                "disabled"
            }
        ),
        format!(
            "TLS hostname verification {}",
            if connection.ssl_host_verification {
                "enabled"
            } else {
                "disabled"
            }
        ),
        format!(
            "Last will retained {}",
            if connection.lwt_retained { "yes" } else { "no" }
        ),
    ]
}
