use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionSecretField, ConnectionSettingField,
    ConnectionSettingFlag, ConnectionSettingsSnapshot, ConnectionSettingsTab, KeyringState,
};
use egui::{Button, RichText, TextEdit, Ui, Window};

use crate::theme::ThemeTokens;

#[path = "connection_settings_controls.rs"]
mod controls;
use controls::{
    combo, field, field_with_button, file_field, flag, panel, readonly, secret_field,
    secret_field_enabled, send,
};

pub fn show(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, commands: &AppCommandSender) {
    let settings = &snapshot.connection_settings;
    header(ui, snapshot, tokens);
    ui.add_space(8.0);
    tab_bar(ui, settings.selected_tab, commands);
    ui.separator();

    panel(tokens).show(ui, |ui| match settings.selected_tab {
        ConnectionSettingsTab::Mqtt => mqtt_tab(ui, settings, tokens, commands),
        ConnectionSettingsTab::Tls => tls_tab(ui, settings, tokens, commands),
        ConnectionSettingsTab::Proxy => proxy_tab(ui, settings, tokens, commands),
        ConnectionSettingsTab::Lwt => lwt_tab(ui, settings, tokens, commands),
    });

    ui.add_space(8.0);
    validation(ui, settings, tokens);
    keyring_status(ui, settings.keyring_state, tokens);
    ui.add_space(8.0);
    action_bar(ui, settings, commands);

    if settings.delete_confirmation_open {
        delete_confirmation(ui, settings, commands);
    }
}

fn header(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("Connection Settings");
        if let Some(connection) = snapshot.selected_connection() {
            ui.label(RichText::new(&connection.name).strong());
            ui.label(RichText::new(&connection.endpoint).color(tokens.text_secondary));
        }
    });
}

fn tab_bar(ui: &mut Ui, selected: ConnectionSettingsTab, commands: &AppCommandSender) {
    ui.horizontal_wrapped(|ui| {
        for tab in ConnectionSettingsTab::ALL {
            if ui.selectable_label(selected == tab, tab.label()).clicked() {
                send(commands, AppCommand::SelectConnectionSettingsTab(tab));
            }
        }
    });
}

fn mqtt_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    readonly(ui, "Internal id", &settings.internal_id);
    field(
        ui,
        "Name",
        &settings.profile_name,
        ConnectionSettingField::ProfileName,
        commands,
    );
    field(
        ui,
        "Host",
        &settings.host,
        ConnectionSettingField::Host,
        commands,
    );
    field(
        ui,
        "Port",
        &settings.port,
        ConnectionSettingField::Port,
        commands,
    );
    combo(
        ui,
        "MQTT version",
        &settings.mqtt_version,
        ConnectionSettingField::MqttVersion,
        &["MQTT 3.1.1", "MQTT v5"],
        commands,
    );
    flag(
        ui,
        "Clean session",
        settings.clean_session,
        ConnectionSettingFlag::CleanSession,
        commands,
    );
    field_with_button(
        ui,
        "Client id",
        &settings.client_id,
        ConnectionSettingField::ClientId,
        "Generate",
        AppCommand::GenerateClientId,
        commands,
    );
    field(
        ui,
        "Username",
        &settings.username,
        ConnectionSettingField::Username,
        commands,
    );
    secret_field(
        ui,
        "Password",
        &settings.password,
        &settings.password_status,
        ConnectionSecretField::MqttPassword,
        tokens,
        commands,
    );
}

fn tls_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    combo(
        ui,
        "TLS/SSL",
        &settings.tls_mode,
        ConnectionSettingField::TlsMode,
        &["No TLS/SSL", "Keystore"],
        commands,
    );
    file_field(
        ui,
        "SSL keystore",
        &settings.tls_store,
        ConnectionSettingField::TlsStore,
        true,
        commands,
    );
    secret_field(
        ui,
        "SSL password",
        &settings.tls_keystore_password,
        &settings.tls_password_status,
        ConnectionSecretField::TlsKeystorePassword,
        _tokens,
        commands,
    );
    flag(
        ui,
        "Verify broker hostname",
        settings.tls_host_verification,
        ConnectionSettingFlag::TlsHostVerification,
        commands,
    );
}

fn proxy_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    combo(
        ui,
        "Proxy mode",
        &settings.proxy_mode,
        ConnectionSettingField::ProxyMode,
        &["No proxy/tunnel", "SSH"],
        commands,
    );
    ui.add_enabled_ui(settings.proxy_mode == "SSH", |ui| {
        field(
            ui,
            "SSH Host",
            &settings.ssh_host,
            ConnectionSettingField::SshHost,
            commands,
        );
        field(
            ui,
            "SSH Port",
            &settings.ssh_port,
            ConnectionSettingField::SshPort,
            commands,
        );
        field(
            ui,
            "Local MQTT port",
            &settings.local_mqtt_port,
            ConnectionSettingField::LocalMqttPort,
            commands,
        );
        combo(
            ui,
            "Authentication",
            &settings.auth_mode,
            ConnectionSettingField::AuthMode,
            &["No Auth", "Keyfile", "Password"],
            commands,
        );
        field(
            ui,
            "SSH username",
            &settings.auth_username,
            ConnectionSettingField::AuthUsername,
            commands,
        );
        secret_field_enabled(
            ui,
            "SSH password",
            &settings.ssh_password,
            &settings.ssh_password_status,
            ConnectionSecretField::SshPassword,
            settings.auth_mode == "Password",
            _tokens,
            commands,
        );
        file_field(
            ui,
            "SSH key file",
            &settings.ssh_key_file,
            ConnectionSettingField::SshKeyFile,
            settings.auth_mode == "Keyfile",
            commands,
        );
    });
}

fn lwt_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let mut enabled = settings.lwt_enabled;
    if ui.checkbox(&mut enabled, "Enable last will").changed() {
        send(commands, AppCommand::SetLwtEnabled(enabled));
    }
    ui.add_enabled_ui(settings.lwt_enabled, |ui| {
        field(
            ui,
            "LWT topic",
            &settings.lwt_topic,
            ConnectionSettingField::LwtTopic,
            commands,
        );
        flag(
            ui,
            "LWT retained",
            settings.lwt_retained,
            ConnectionSettingFlag::LwtRetained,
            commands,
        );
        let mut payload = settings.lwt_payload.clone();
        if ui
            .add(
                TextEdit::multiline(&mut payload)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(5)
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            send(
                commands,
                AppCommand::UpdateConnectionSetting {
                    field: ConnectionSettingField::LwtPayload,
                    value: payload,
                },
            );
        }
    });
    if !settings.lwt_enabled {
        ui.label(RichText::new("Last will payload is inactive.").color(tokens.text_disabled));
    }
}

fn validation(ui: &mut Ui, settings: &ConnectionSettingsSnapshot, tokens: ThemeTokens) {
    if settings.validation_errors.is_empty() {
        ui.label(RichText::new("No validation errors").color(tokens.success));
    } else {
        for error in &settings.validation_errors {
            ui.label(RichText::new(error).color(tokens.warning));
        }
    }
}

fn keyring_status(ui: &mut Ui, state: KeyringState, tokens: ThemeTokens) {
    let color = match state {
        KeyringState::Available => tokens.success,
        KeyringState::Locked | KeyringState::MigrationRequired => tokens.warning,
        KeyringState::Unavailable => tokens.danger,
    };
    ui.label(RichText::new(state.label()).color(color));
}

fn action_bar(ui: &mut Ui, settings: &ConnectionSettingsSnapshot, commands: &AppCommandSender) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(settings.dirty, Button::new("Discard"))
            .clicked()
        {
            send(commands, AppCommand::DiscardConnectionSettings);
        }
        if ui.button("Delete...").clicked() {
            send(commands, AppCommand::RequestDeleteConnection);
        }
        let can_save = settings.dirty && settings.valid;
        let save = ui.add_enabled(can_save, Button::new("Save"));
        if save.clicked() {
            send(commands, AppCommand::SaveConnectionSettings);
        }
        if !can_save {
            save.on_hover_text(&settings.save_disabled_reason);
        }
    });
}

fn delete_confirmation(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    commands: &AppCommandSender,
) {
    Window::new("Delete connection")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.label(format!("Delete {}?", settings.profile_name));
            ui.label("Secrets and histories can be removed or kept by storage support.");
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    send(commands, AppCommand::CancelDeleteConnection);
                }
                if ui.button("Delete").clicked() {
                    send(commands, AppCommand::ConfirmDeleteConnection);
                }
            });
        });
}
