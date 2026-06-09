use correo_mqtt::ConnectionId;

use crate::{
    AppModel, ConnectDisabledReason, ConnectionBadge, ConnectionSettingField,
    ConnectionSettingsSnapshot, ConnectionSettingsTab, ConnectionState, ConnectionSummary,
    Diagnostic, KeyringState,
};

impl AppModel {
    pub(super) fn add_connection(&mut self) {
        self.snapshot.active_workspace = crate::Workspace::Connections;
        self.snapshot.selected_connection = None;
        self.snapshot.connection_surface = crate::ConnectionSurface::Settings;
        self.snapshot.connection_settings = new_connection_settings();
        self.push_diagnostic(Diagnostic::info("New connection draft opened."));
    }

    pub(super) fn connect(&mut self, id: ConnectionId) {
        let Some(index) = self.connection_index(id) else {
            return;
        };

        if !self.snapshot.connections[index].can_connect() {
            let reason = self.snapshot.connections[index]
                .disabled_reason
                .unwrap_or(ConnectDisabledReason::Busy)
                .label();
            self.push_diagnostic(Diagnostic::warning(reason));
            return;
        }

        let name = self.snapshot.connections[index].name.clone();
        self.update_connection_state(
            id,
            ConnectionState::Connecting,
            Some(ConnectDisabledReason::Busy),
            "connect command queued".to_owned(),
        );
        self.snapshot.selected_connection = Some(id);
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
        self.push_diagnostic(Diagnostic::info(format!("Connect requested for {name}.")));
    }

    pub(super) fn disconnect(&mut self, id: ConnectionId) {
        let Some(index) = self.connection_index(id) else {
            return;
        };
        let name = {
            let connection = &mut self.snapshot.connections[index];
            connection.state = ConnectionState::Disconnected;
            connection.disabled_reason = None;
            connection.name.clone()
        };
        self.snapshot.active_connection = None;
        self.push_diagnostic(Diagnostic::info(format!(
            "{name} disconnect command queued."
        )));
    }

    pub(super) fn save_connection_settings(&mut self) {
        refresh_connection_settings_validation(&mut self.snapshot.connection_settings);
        if !self.snapshot.connection_settings.dirty || !self.snapshot.connection_settings.valid {
            let reason = self
                .snapshot
                .connection_settings
                .save_disabled_reason
                .clone();
            self.push_diagnostic(Diagnostic::warning(reason));
            return;
        }

        let mut settings = self.snapshot.connection_settings.clone();
        settings.dirty = false;
        settings.delete_confirmation_open = false;
        settings.save_disabled_reason = "No changes to save".to_owned();

        if let Some(id) = self.snapshot.selected_connection {
            self.connection_settings.insert(id, settings.clone());
            self.snapshot.connection_settings = settings.clone();
            self.update_connection_summary(id, &settings);
            self.push_diagnostic(Diagnostic::info("Connection settings save command queued."));
            return;
        }

        let id = ConnectionId::new();
        self.connection_settings.insert(id, settings.clone());
        self.storage_connection_ids.insert(id, id.to_string());
        self.snapshot
            .connections
            .push(connection_summary(id, &settings));
        self.snapshot.connection_count = self.snapshot.connections.len();
        self.snapshot.selected_connection = Some(id);
        self.snapshot.connection_settings = settings;
        self.push_diagnostic(Diagnostic::info(
            "New connection profile added to the current session.",
        ));
    }

    pub(super) fn discard_connection_settings(&mut self) {
        if let Some(id) = self.snapshot.selected_connection {
            if let Some(settings) = self.connection_settings.get(&id) {
                self.snapshot.connection_settings = settings.clone();
            } else {
                self.snapshot.connection_settings.dirty = false;
            }
            self.push_diagnostic(Diagnostic::info("Connection settings discarded."));
            return;
        }

        self.snapshot.connection_settings = ConnectionSettingsSnapshot::default();
        self.snapshot.connection_surface = crate::ConnectionSurface::Launcher;
        self.push_diagnostic(Diagnostic::info("New connection draft discarded."));
    }

    pub(super) fn update_connection_setting(
        &mut self,
        field: ConnectionSettingField,
        value: String,
    ) {
        let settings = &mut self.snapshot.connection_settings;
        match field {
            ConnectionSettingField::ProfileName => settings.profile_name = value,
            ConnectionSettingField::Host => settings.host = value,
            ConnectionSettingField::Port => settings.port = value,
            ConnectionSettingField::MqttVersion => settings.mqtt_version = value,
            ConnectionSettingField::ClientId => settings.client_id = value,
            ConnectionSettingField::AuthMode => settings.auth_mode = value,
            ConnectionSettingField::TlsMode => settings.tls_mode = value,
            ConnectionSettingField::TlsStore => settings.tls_store = value,
            ConnectionSettingField::ProxyMode => settings.proxy_mode = value,
            ConnectionSettingField::ProxyEndpoint => settings.proxy_endpoint = value,
            ConnectionSettingField::LwtTopic => settings.lwt_topic = value,
            ConnectionSettingField::LwtPayload => settings.lwt_payload = value,
        }
        settings.dirty = true;
        refresh_connection_settings_validation(settings);
    }

    pub(super) fn record_action(&mut self, id: ConnectionId, action: &'static str) {
        let name = self
            .snapshot
            .connections
            .iter()
            .find(|connection| connection.id == id)
            .map(|connection| connection.name.clone())
            .unwrap_or_else(|| "Unknown connection".to_owned());
        self.push_diagnostic(Diagnostic::info(format!("{action}: {name}.")));
    }

    pub(super) fn connection_index(&self, id: ConnectionId) -> Option<usize> {
        self.snapshot
            .connections
            .iter()
            .position(|connection| connection.id == id)
    }

    pub(super) fn update_connection_state(
        &mut self,
        id: ConnectionId,
        state: ConnectionState,
        disabled_reason: Option<ConnectDisabledReason>,
        last_activity: String,
    ) {
        if let Some(index) = self.connection_index(id) {
            let connection = &mut self.snapshot.connections[index];
            connection.state = state;
            connection.disabled_reason = disabled_reason;
            connection.last_activity = last_activity;
        }
    }

    pub(super) fn load_connection_settings(&mut self, id: ConnectionId) {
        if let Some(settings) = self.connection_settings.get(&id) {
            self.snapshot.connection_settings = settings.clone();
        }
    }

    fn update_connection_summary(
        &mut self,
        id: ConnectionId,
        settings: &ConnectionSettingsSnapshot,
    ) {
        if let Some(index) = self.connection_index(id) {
            let state = self.snapshot.connections[index].state;
            let disabled_reason = if settings.host.trim().is_empty() {
                Some(ConnectDisabledReason::MissingHost)
            } else {
                self.snapshot.connections[index].disabled_reason
            };
            self.snapshot.connections[index] = ConnectionSummary {
                state,
                disabled_reason,
                recent_subscriptions: self.snapshot.connections[index].recent_subscriptions,
                recent_messages: self.snapshot.connections[index].recent_messages,
                last_activity: self.snapshot.connections[index].last_activity.clone(),
                ..connection_summary(id, settings)
            };
        }
    }
}

fn new_connection_settings() -> ConnectionSettingsSnapshot {
    let mut settings = ConnectionSettingsSnapshot {
        selected_tab: ConnectionSettingsTab::Mqtt,
        profile_name: "New connection".to_owned(),
        port: "1883".to_owned(),
        mqtt_version: "MQTT v5".to_owned(),
        auth_mode: "Disabled".to_owned(),
        username_status: "No MQTT username configured".to_owned(),
        tls_mode: "Disabled".to_owned(),
        tls_store: "No certificate store selected".to_owned(),
        proxy_mode: "Disabled".to_owned(),
        proxy_endpoint: "No tunnel configured".to_owned(),
        advanced_options: vec![
            "Clean session enabled".to_owned(),
            "TLS hostname verification enabled".to_owned(),
            "Last will retained no".to_owned(),
        ],
        dirty: true,
        keyring_state: KeyringState::Available,
        ..ConnectionSettingsSnapshot::default()
    };
    refresh_connection_settings_validation(&mut settings);
    settings
}

fn refresh_connection_settings_validation(settings: &mut ConnectionSettingsSnapshot) {
    let mut errors = Vec::new();
    if settings.profile_name.trim().is_empty() {
        errors.push("Name is required".to_owned());
    }
    if settings.host.trim().is_empty() {
        errors.push("Host is required".to_owned());
    }
    match settings.port.trim().parse::<u16>() {
        Ok(0) | Err(_) => errors.push("Port must be between 1 and 65535".to_owned()),
        Ok(_) => {}
    }

    settings.valid = errors.is_empty();
    settings.save_disabled_reason = if settings.valid {
        "No changes to save".to_owned()
    } else {
        "Resolve validation errors before saving".to_owned()
    };
    settings.validation_errors = errors;
}

fn connection_summary(
    id: ConnectionId,
    settings: &ConnectionSettingsSnapshot,
) -> ConnectionSummary {
    ConnectionSummary {
        id,
        name: settings.profile_name.trim().to_owned(),
        endpoint: format!("{}:{}", settings.host.trim(), settings.port.trim()),
        mqtt_version: settings.mqtt_version.clone(),
        badges: connection_badges(settings),
        state: ConnectionState::Disconnected,
        disabled_reason: settings
            .host
            .trim()
            .is_empty()
            .then_some(ConnectDisabledReason::MissingHost),
        recent_subscriptions: 0,
        recent_messages: 0,
        last_activity: "Ready".to_owned(),
    }
}

fn connection_badges(settings: &ConnectionSettingsSnapshot) -> Vec<ConnectionBadge> {
    let mut badges = Vec::new();
    if settings.tls_mode != "Disabled" {
        badges.push(ConnectionBadge::Tls);
    }
    if settings.proxy_mode != "Disabled" {
        badges.push(ConnectionBadge::Proxy);
    }
    if settings.lwt_enabled {
        badges.push(ConnectionBadge::Lwt);
    }
    badges
}
