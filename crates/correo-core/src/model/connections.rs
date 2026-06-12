use correo_mqtt::ConnectionId;

use crate::{
    AppModel, ConnectDisabledReason, ConnectionBadge, ConnectionSecretField,
    ConnectionSettingField, ConnectionSettingFlag, ConnectionSettingsSnapshot,
    ConnectionSettingsTab, ConnectionState, ConnectionSummary, Diagnostic, KeyringState,
    SecretInput,
};

impl AppModel {
    pub(super) fn normalize_connection_surface(&mut self) {
        self.ensure_selected_connection();
        if self.snapshot.connection_surface == crate::ConnectionSurface::Launcher {
            self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
        }
    }

    pub(super) fn open_default_connection_surface(&mut self) {
        self.snapshot.active_workspace = crate::Workspace::Connections;
        self.ensure_selected_connection();
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
    }

    pub(super) fn add_connection(&mut self) {
        self.snapshot.active_workspace = crate::Workspace::Connections;
        self.snapshot.selected_connection = None;
        self.snapshot.connection_surface = crate::ConnectionSurface::Settings;
        self.snapshot.connection_settings_overlay = None;
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
        if !self.snapshot.connection_settings.valid {
            for error in self.snapshot.connection_settings.validation_errors.clone() {
                self.push_diagnostic(Diagnostic::warning(error));
            }
            return;
        }
        if !self.snapshot.connection_settings.dirty {
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
            if settings.internal_id.trim().is_empty() {
                settings.internal_id = id.to_string();
            }
            self.connection_settings.insert(id, settings.clone());
            self.snapshot.connection_settings = settings.clone();
            self.update_connection_summary(id, &settings);
            self.snapshot.connection_settings_overlay = None;
            self.push_diagnostic(Diagnostic::info("Connection settings save command queued."));
            return;
        }

        let id = ConnectionId::new();
        settings.internal_id = id.to_string();
        self.connection_settings.insert(id, settings.clone());
        self.storage_connection_ids.insert(id, id.to_string());
        self.snapshot
            .connections
            .push(connection_summary(id, &settings));
        self.snapshot.connection_count = self.snapshot.connections.len();
        self.snapshot.selected_connection = Some(id);
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
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
            self.snapshot.connection_settings_overlay = None;
            self.push_diagnostic(Diagnostic::info("Connection settings discarded."));
            return;
        }

        self.snapshot.connection_settings = ConnectionSettingsSnapshot::default();
        self.snapshot.connection_settings_overlay = None;
        self.open_default_connection_surface();
        self.push_diagnostic(Diagnostic::info("New connection draft discarded."));
    }

    pub(super) fn request_delete_connection(&mut self) {
        let Some(id) = self.snapshot.selected_connection else {
            self.snapshot.connection_settings.delete_confirmation_open = false;
            return;
        };
        if self.snapshot.connection_settings_overlay != Some(id)
            && self.snapshot.connection_surface != crate::ConnectionSurface::Settings
        {
            self.load_connection_settings(id);
            if !self.connection_settings.contains_key(&id) {
                if let Some(connection) = self.snapshot.connections.iter().find(|row| row.id == id)
                {
                    self.snapshot.connection_settings.profile_name = connection.name.clone();
                }
            }
        }
        self.snapshot.connection_settings.delete_confirmation_open = true;
    }

    pub(super) fn delete_selected_connection(&mut self) {
        let Some(id) = self.snapshot.selected_connection else {
            self.snapshot.connection_settings.delete_confirmation_open = false;
            return;
        };
        let Some(index) = self.connection_index(id) else {
            self.snapshot.connection_settings.delete_confirmation_open = false;
            self.snapshot.connection_settings_overlay = None;
            self.open_default_connection_surface();
            return;
        };

        let name = self.snapshot.connections.remove(index).name;
        self.connection_settings.remove(&id);
        self.storage_connection_ids.remove(&id);
        self.workbenches.remove(&id);
        self.dirty_workbenches.remove(&id);
        if self.snapshot.active_connection == Some(id) {
            self.snapshot.active_connection = None;
        }

        self.snapshot.connection_count = self.snapshot.connections.len();
        self.snapshot.connection_settings_overlay = None;
        self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
        self.snapshot.selected_connection = self
            .snapshot
            .connections
            .first()
            .map(|connection| connection.id);
        self.snapshot.workbench = self
            .snapshot
            .selected_connection
            .and_then(|selected| self.workbenches.get(&selected).cloned())
            .unwrap_or_default();
        self.snapshot.connection_settings = self
            .snapshot
            .selected_connection
            .and_then(|selected| self.connection_settings.get(&selected).cloned())
            .unwrap_or_default();
        self.push_diagnostic(Diagnostic::info(format!("Deleted connection {name}.")));
    }

    pub(super) fn move_connection(
        &mut self,
        connection_id: ConnectionId,
        target_connection_id: ConnectionId,
        after: bool,
    ) {
        if connection_id == target_connection_id {
            return;
        }

        let Some(from_index) = self.connection_index(connection_id) else {
            return;
        };
        let Some(target_index) = self.connection_index(target_connection_id) else {
            return;
        };

        let connection = self.snapshot.connections.remove(from_index);
        let mut insert_index = if from_index < target_index {
            target_index.saturating_sub(1)
        } else {
            target_index
        };
        if after {
            insert_index += 1;
        }
        let insert_index = insert_index.min(self.snapshot.connections.len());
        self.snapshot.connections.insert(insert_index, connection);
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
            ConnectionSettingField::Username => settings.username = value,
            ConnectionSettingField::TlsMode => settings.tls_mode = value,
            ConnectionSettingField::TlsStore => settings.tls_store = value,
            ConnectionSettingField::ProxyMode => settings.proxy_mode = value,
            ConnectionSettingField::SshHost => settings.ssh_host = value,
            ConnectionSettingField::SshPort => settings.ssh_port = value,
            ConnectionSettingField::LocalMqttPort => settings.local_mqtt_port = value,
            ConnectionSettingField::AuthMode => settings.auth_mode = value,
            ConnectionSettingField::AuthUsername => settings.auth_username = value,
            ConnectionSettingField::SshKeyFile => settings.ssh_key_file = value,
            ConnectionSettingField::LwtTopic => settings.lwt_topic = value,
            ConnectionSettingField::LwtPayload => settings.lwt_payload = value,
        }
        settings.dirty = true;
        refresh_connection_settings_validation(settings);
    }

    pub(super) fn update_connection_secret(
        &mut self,
        field: ConnectionSecretField,
        value: SecretInput,
    ) {
        let settings = &mut self.snapshot.connection_settings;
        match field {
            ConnectionSecretField::MqttPassword => settings.password = value,
            ConnectionSecretField::TlsKeystorePassword => settings.tls_keystore_password = value,
            ConnectionSecretField::SshPassword => settings.ssh_password = value,
        }
        settings.dirty = true;
    }

    pub(super) fn set_connection_setting_flag(
        &mut self,
        flag: ConnectionSettingFlag,
        enabled: bool,
    ) {
        let settings = &mut self.snapshot.connection_settings;
        match flag {
            ConnectionSettingFlag::CleanSession => settings.clean_session = enabled,
            ConnectionSettingFlag::TlsHostVerification => settings.tls_host_verification = enabled,
            ConnectionSettingFlag::LwtRetained => settings.lwt_retained = enabled,
        }
        settings.dirty = true;
        refresh_connection_settings_validation(settings);
    }

    pub(super) fn generate_client_id(&mut self) {
        self.snapshot.connection_settings.client_id =
            format!("correomqtt-{}", uuid::Uuid::new_v4().simple());
        self.snapshot.connection_settings.dirty = true;
        refresh_connection_settings_validation(&mut self.snapshot.connection_settings);
    }

    pub(super) fn refresh_connection_settings_validation(&mut self) {
        refresh_connection_settings_validation(&mut self.snapshot.connection_settings);
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

    fn ensure_selected_connection(&mut self) {
        if self
            .snapshot
            .selected_connection
            .is_some_and(|id| self.connection_index(id).is_some())
        {
            return;
        }

        self.snapshot.selected_connection = self
            .snapshot
            .connections
            .first()
            .map(|connection| connection.id);
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
        internal_id: "Generated on save".to_owned(),
        profile_name: "New connection".to_owned(),
        port: "1883".to_owned(),
        mqtt_version: "MQTT v5".to_owned(),
        clean_session: true,
        password_status: "No MQTT password configured".to_owned(),
        tls_mode: "No TLS/SSL".to_owned(),
        tls_password_status: "No SSL password configured".to_owned(),
        tls_host_verification: true,
        proxy_mode: "No proxy/tunnel".to_owned(),
        ssh_port: "22".to_owned(),
        local_mqtt_port: "1883".to_owned(),
        auth_mode: "No Auth".to_owned(),
        ssh_password_status: "No SSH password configured".to_owned(),
        lwt_retained: false,
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
    if settings.tls_mode == "Keystore" && settings.tls_store.trim().is_empty() {
        errors.push("SSL keystore is required when TLS/SSL uses Keystore".to_owned());
    }
    if settings.proxy_mode == "SSH" {
        if settings.ssh_host.trim().is_empty() {
            errors.push("SSH host is required".to_owned());
        }
        validate_optional_port(&settings.ssh_port, "SSH port", true, &mut errors);
        validate_optional_port(
            &settings.local_mqtt_port,
            "Local MQTT port",
            false,
            &mut errors,
        );
        if settings.auth_mode != "No Auth" && settings.auth_username.trim().is_empty() {
            errors.push("SSH username is required".to_owned());
        }
        if settings.auth_mode == "Keyfile" && settings.ssh_key_file.trim().is_empty() {
            errors.push("SSH key file is required".to_owned());
        }
    }

    settings.valid = errors.is_empty();
    settings.save_disabled_reason = if settings.valid {
        "No changes to save".to_owned()
    } else {
        "Resolve validation errors before saving".to_owned()
    };
    settings.validation_errors = errors;
}

fn validate_optional_port(value: &str, label: &str, required: bool, errors: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        if required {
            errors.push(format!("{label} is required"));
        }
        return;
    }
    match trimmed.parse::<u16>() {
        Ok(0) | Err(_) => errors.push(format!("{label} must be between 1 and 65535")),
        Ok(_) => {}
    }
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
    if !settings.username.trim().is_empty() || !settings.password.is_empty() {
        badges.push(ConnectionBadge::Credentials);
    }
    if settings.tls_mode != "No TLS/SSL" {
        badges.push(ConnectionBadge::Tls);
    }
    if settings.proxy_mode != "No proxy/tunnel" {
        badges.push(ConnectionBadge::Proxy);
    }
    if settings.lwt_enabled {
        badges.push(ConnectionBadge::Lwt);
    }
    badges
}
