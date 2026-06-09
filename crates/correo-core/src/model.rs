use std::collections::HashMap;

use correo_mqtt::ConnectionId;

use crate::{
    AppCommand, AppEvent, AppSnapshot, ConnectDisabledReason, ConnectionSettingsSnapshot,
    ConnectionState, Diagnostic, MqttCommand, MqttCommandBuildError, StartupState,
};

mod connections;
mod history;
mod migration_recovery;
mod mqtt;
mod plugin_workflows;
mod plugins;
mod scripting;
mod scripting_commands;
#[cfg(test)]
mod scripting_tests;
mod settings;
mod subscriptions;
mod transfer;

#[derive(Debug, Clone)]
pub struct AppModel {
    snapshot: AppSnapshot,
    connection_settings: HashMap<ConnectionId, ConnectionSettingsSnapshot>,
    storage_connection_ids: HashMap<ConnectionId, String>,
    saved_global_settings: crate::GlobalSettingsSnapshot,
    saved_theme_mode: crate::ThemeMode,
}

impl AppModel {
    pub fn new() -> Self {
        Self::with_snapshot(crate::sample_snapshot(crate::ThemeMode::System))
    }

    pub fn empty() -> Self {
        Self::with_snapshot(AppSnapshot::empty())
    }

    pub fn with_snapshot(snapshot: AppSnapshot) -> Self {
        Self::from_parts(snapshot, HashMap::new(), HashMap::new())
    }

    pub fn with_startup_state(state: StartupState) -> Self {
        Self::from_parts(
            state.snapshot,
            state.connection_settings,
            state.storage_connection_ids,
        )
    }

    fn from_parts(
        snapshot: AppSnapshot,
        connection_settings: HashMap<ConnectionId, ConnectionSettingsSnapshot>,
        storage_connection_ids: HashMap<ConnectionId, String>,
    ) -> Self {
        let saved_global_settings = snapshot.global_settings.clone();
        let saved_theme_mode = snapshot.theme_mode;
        Self {
            snapshot,
            connection_settings,
            storage_connection_ids,
            saved_global_settings,
            saved_theme_mode,
        }
    }

    pub fn snapshot(&self) -> &AppSnapshot {
        &self.snapshot
    }

    pub(crate) fn mqtt_commands_for_app_command(
        &self,
        command: &AppCommand,
    ) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
        crate::commands_for_app_command(command, &self.snapshot, &self.connection_settings)
    }

    pub fn apply_command(&mut self, command: AppCommand) {
        if self.apply_migration_recovery_command(&command)
            || self.apply_scripting_command(&command)
            || self.apply_plugin_command(&command)
        {
            return;
        }

        match command {
            AppCommand::SelectWorkspace(workspace) => self.snapshot.active_workspace = workspace,
            AppCommand::SetThemeMode(mode) => self.set_theme_mode(mode),
            AppCommand::SearchConnections(filter) => self.snapshot.connection_filter = filter,
            AppCommand::SelectConnection(id) => {
                self.snapshot.selected_connection = Some(id);
                self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
            }
            AppCommand::MoveConnection {
                connection_id,
                target_connection_id,
                after,
            } => self.move_connection(connection_id, target_connection_id, after),
            AppCommand::OpenConnectionLauncher => {
                self.snapshot.connection_surface = crate::ConnectionSurface::Launcher;
            }
            AppCommand::OpenConnectionWorkbench(id) => {
                self.snapshot.selected_connection = Some(id);
                self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
            }
            AppCommand::Connect(id) => self.connect(id),
            AppCommand::OpenConnectionSettings(id) | AppCommand::EditConnection(id) => {
                self.snapshot.selected_connection = Some(id);
                self.load_connection_settings(id);
                self.snapshot.connection_surface = crate::ConnectionSurface::Workbench;
                self.snapshot.connection_settings_overlay = Some(id);
            }
            AppCommand::Reconnect(id) => self.record_action(id, "Reconnect requested"),
            AppCommand::Disconnect(id) => self.disconnect(id),
            AppCommand::DuplicateConnection(id) => self.record_action(id, "Duplicate requested"),
            AppCommand::AddConnection => self.add_connection(),
            AppCommand::ImportConnections => self.import_connections(),
            AppCommand::ExportConnections => self.open_connection_export(),
            AppCommand::ChooseConnectionImportFile => self.choose_connection_import_file(),
            AppCommand::SubmitConnectionImportPassword => self.submit_connection_import_password(),
            AppCommand::ClearConnectionImportError => self.clear_connection_import_error(),
            AppCommand::SelectConnectionImportRow { row_id, selected } => {
                self.select_connection_import_row(&row_id, selected);
            }
            AppCommand::StartConnectionImport => self.start_connection_import(),
            AppCommand::SelectConnectionExportRow { row_id, selected } => {
                self.select_connection_export_row(&row_id, selected);
            }
            AppCommand::SetConnectionExportEncrypted(encrypted) => {
                self.set_connection_export_encrypted(encrypted);
            }
            AppCommand::UpdateConnectionExportPath(path) => {
                self.update_connection_export_path(path)
            }
            AppCommand::StartConnectionExport => self.start_connection_export(),
            AppCommand::ImportMessages => self.import_messages(),
            AppCommand::ExportMessages => self.export_messages(),
            AppCommand::ExportPublishHistoryMessage(topic) => {
                self.export_publish_history_message(topic)
            }
            AppCommand::ExportIncomingMessage(id) => self.export_incoming_message(id),
            AppCommand::SelectWorkbenchTab(tab) => self.snapshot.workbench.narrow_tab = tab,
            AppCommand::UpdatePublishTopic(topic) => self.update_publish_topic(topic),
            AppCommand::UpdatePublishPayload(payload) => self.update_publish_payload(payload),
            AppCommand::UpdatePublishQos(qos) => self.update_publish_qos(qos),
            AppCommand::SetPublishRetained(retained) => {
                self.snapshot.workbench.publish.retained = retained;
            }
            AppCommand::SearchPublishHistory(filter) => {
                self.snapshot.workbench.publish.history_filter = filter;
            }
            AppCommand::Publish => self.publish_from_snapshot(),
            AppCommand::UpdateSubscribeTopic(topic) => self.update_subscribe_topic(topic),
            AppCommand::UpdateSubscribeQos(qos) => self.update_subscribe_qos(qos),
            AppCommand::Subscribe => self.subscribe_from_snapshot(),
            AppCommand::Unsubscribe(topic) => self.unsubscribe(&topic),
            AppCommand::UnsubscribeAll => self.request_unsubscribe_all(),
            AppCommand::CancelUnsubscribeAll => self.cancel_unsubscribe_all(),
            AppCommand::ConfirmUnsubscribeAll => self.confirm_unsubscribe_all(),
            AppCommand::SearchMessages(filter) => {
                self.snapshot.workbench.subscribe.message_filter = filter;
            }
            AppCommand::SelectMessage(id) => self.snapshot.workbench.selected_message_id = Some(id),
            AppCommand::SelectInspectorTab(tab) => self.snapshot.workbench.inspector_tab = tab,
            AppCommand::SelectDetailTransform(plugin_id) => self.select_detail_transform(plugin_id),
            AppCommand::SelectDetailFormatter(plugin_id) => self.select_detail_formatter(plugin_id),
            AppCommand::RefreshMessageDetail => {}
            AppCommand::SelectConnectionSettingsTab(tab) => {
                self.snapshot.connection_settings.selected_tab = tab;
            }
            AppCommand::UpdateConnectionSetting { field, value } => {
                self.update_connection_setting(field, value);
            }
            AppCommand::UpdateConnectionSecret { field, value } => {
                self.update_connection_secret(field, value);
            }
            AppCommand::SetConnectionSettingFlag { flag, enabled } => {
                self.set_connection_setting_flag(flag, enabled);
            }
            AppCommand::GenerateClientId => self.generate_client_id(),
            AppCommand::SetLwtEnabled(enabled) => {
                self.snapshot.connection_settings.lwt_enabled = enabled;
                self.snapshot.connection_settings.dirty = true;
                self.refresh_connection_settings_validation();
            }
            AppCommand::SaveConnectionSettings => self.save_connection_settings(),
            AppCommand::DiscardConnectionSettings => self.discard_connection_settings(),
            AppCommand::RequestDeleteConnection => {
                self.snapshot.connection_settings.delete_confirmation_open = true;
            }
            AppCommand::CancelDeleteConnection => {
                self.snapshot.connection_settings.delete_confirmation_open = false;
            }
            AppCommand::ConfirmDeleteConnection => {
                self.snapshot.connection_settings.delete_confirmation_open = false;
                self.push_diagnostic(Diagnostic::warning(
                    "Delete connection command queued; secrets remain redacted.",
                ));
            }
            AppCommand::SelectTransferSection(section) => {
                self.snapshot.transfer.active_section = section;
            }
            AppCommand::SelectTransferStep(step) => self.select_import_step(step),
            AppCommand::SelectGlobalSettingsSection(section) => {
                self.select_global_settings_section(section);
            }
            AppCommand::UpdateGlobalSetting { field, value } => {
                self.update_global_setting(field, value);
            }
            AppCommand::SetGlobalSettingFlag { flag, enabled } => {
                self.set_global_setting_flag(flag, enabled);
            }
            AppCommand::SaveGlobalSettings => self.save_global_settings(),
            AppCommand::DiscardGlobalSettings => self.discard_global_settings(),
            AppCommand::Mqtt(command) => self.apply_mqtt_command(command),
            AppCommand::Shutdown => {}
            _ => unreachable!("handled before main dispatch"),
        }
    }

    pub fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::ConnectionListLoaded { connections } => {
                self.snapshot.connection_count = connections.len();
                self.snapshot.connections = connections;
                if self
                    .snapshot
                    .selected_connection
                    .is_none_or(|id| self.connection_index(id).is_none())
                {
                    self.snapshot.selected_connection = self
                        .snapshot
                        .connections
                        .first()
                        .map(|connection| connection.id);
                }
            }
            AppEvent::ConnectionOpened { connection_id } => {
                self.snapshot.active_connection = Some(connection_id);
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Connected,
                    Some(ConnectDisabledReason::AlreadyConnected),
                    "connected".to_owned(),
                );
            }
            AppEvent::ConnectionClosed { connection_id } => {
                if self.snapshot.active_connection == Some(connection_id) {
                    self.snapshot.active_connection = None;
                }
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Disconnected,
                    None,
                    "disconnected".to_owned(),
                );
            }
            AppEvent::ConnectionStateChanged {
                connection_id,
                state,
                disabled_reason,
                last_activity,
            } => {
                self.update_connection_state(connection_id, state, disabled_reason, last_activity);
            }
            AppEvent::ConnectionSettingsLoaded {
                connection_id,
                settings,
            } => {
                self.snapshot.selected_connection = Some(connection_id);
                self.connection_settings
                    .insert(connection_id, settings.clone());
                self.snapshot.connection_settings = settings;
            }
            AppEvent::GlobalSettingsLoaded { settings } => self.load_global_settings(settings),
            AppEvent::ThemeModeChanged { mode } => self.snapshot.theme_mode = mode,
            AppEvent::MigrationApplied {
                state,
                completion,
                diagnostics,
            } => self.apply_migrated_startup_state(*state, completion, diagnostics),
            AppEvent::DiagnosticRaised(diagnostic) => self.push_diagnostic(diagnostic),
            AppEvent::ScriptExecutionLogAppended {
                execution_id,
                level,
                message,
                timestamp,
            } => self.append_script_log(execution_id, level, message, timestamp),
            AppEvent::ScriptExecutionUpdated {
                execution_id,
                status,
                duration,
                error,
            } => self.update_script_execution(execution_id, status, duration, error),
            AppEvent::Mqtt(event) => self.apply_mqtt_event(event),
            AppEvent::MigrationRecovery(event) => self.apply_migration_recovery_event(event),
            AppEvent::PluginWorkflow(event) => self.apply_plugin_workflow_event(event),
        }
    }

    fn publish_from_snapshot(&mut self) {
        if self.snapshot.active_connection.is_none() {
            self.snapshot.workbench.publish.feedback = Some(crate::WorkflowFeedback::warning(
                "Publish requires an active MQTT connection.",
            ));
            self.push_diagnostic(Diagnostic::warning(
                "Publish requires an active MQTT connection.",
            ));
            return;
        }
        let topic = self.snapshot.workbench.publish.topic.trim().to_owned();
        if topic.is_empty() {
            self.snapshot.workbench.publish.feedback = Some(crate::WorkflowFeedback::warning(
                "Publish topic is required.",
            ));
            self.push_diagnostic(Diagnostic::warning("Publish topic is required."));
            return;
        }
        if !self.snapshot.workbench.publish.valid {
            self.snapshot.workbench.publish.feedback = Some(crate::WorkflowFeedback::warning(
                "Publish topic is invalid.",
            ));
            return;
        }
        self.snapshot.workbench.publish.feedback = Some(crate::WorkflowFeedback::info(format!(
            "Publish queued for {topic}."
        )));
        self.push_diagnostic(Diagnostic::info(format!(
            "Publish command queued for {topic}."
        )));
    }

    fn subscribe_from_snapshot(&mut self) {
        if self.snapshot.active_connection.is_none() {
            self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::warning(
                "Subscribe requires an active MQTT connection.",
            ));
            self.push_diagnostic(Diagnostic::warning(
                "Subscribe requires an active MQTT connection.",
            ));
            return;
        }
        let topic = self.snapshot.workbench.subscribe.topic.trim().to_owned();
        if topic.is_empty() {
            self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::warning(
                "Subscribe topic is required.",
            ));
            self.push_diagnostic(Diagnostic::warning("Subscribe topic is required."));
            return;
        }
        if !self.snapshot.workbench.subscribe.valid {
            self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::warning(
                "Subscribe topic filter is invalid.",
            ));
            return;
        }
        self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::info(format!(
            "Subscribe queued for {topic}."
        )));
        self.push_diagnostic(Diagnostic::info(format!(
            "Subscribe command queued for {topic}."
        )));
    }

    fn unsubscribe(&mut self, topic: &str) {
        self.snapshot.workbench.subscribe.feedback = Some(crate::WorkflowFeedback::info(format!(
            "Unsubscribe queued for {topic}."
        )));
        self.push_diagnostic(Diagnostic::info(format!(
            "Unsubscribe command queued for {topic}."
        )));
    }

    fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.snapshot.diagnostics.insert(0, diagnostic.redacted());
        self.snapshot.diagnostics.truncate(12);
    }
}

impl Default for AppModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "model/connection_list_tests.rs"]
mod connection_list_tests;
#[cfg(test)]
#[path = "model/connection_settings_tests.rs"]
mod connection_settings_tests;
#[cfg(test)]
#[path = "model/migration_secret_tests.rs"]
mod migration_secret_tests;
#[cfg(test)]
#[path = "model/plugin_tests.rs"]
mod plugin_tests;
#[cfg(test)]
#[path = "model/tests.rs"]
mod tests;
