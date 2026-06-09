use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionSecretField, ConnectionSettingField,
    ConnectionSettingFlag, ConnectionSettingsSnapshot, ConnectionSettingsTab, KeyringState,
};
use egui::{Button, RichText, TextEdit, Ui, Window};

use crate::{i18n::I18n, theme::ThemeTokens};

#[path = "connection_settings_controls.rs"]
mod controls;
use controls::{
    combo, field, field_with_button, file_field, flag, panel, readonly, secret_field,
    secret_field_enabled, send,
};

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let settings = &snapshot.connection_settings;
    header(ui, snapshot, tokens, i18n);
    ui.add_space(8.0);
    tab_bar(ui, settings.selected_tab, commands, i18n);
    ui.separator();

    panel(tokens).show(ui, |ui| match settings.selected_tab {
        ConnectionSettingsTab::Mqtt => mqtt_tab(ui, settings, tokens, commands, i18n),
        ConnectionSettingsTab::Tls => tls_tab(ui, settings, tokens, commands, i18n),
        ConnectionSettingsTab::Proxy => proxy_tab(ui, settings, tokens, commands, i18n),
        ConnectionSettingsTab::Lwt => lwt_tab(ui, settings, tokens, commands, i18n),
    });

    ui.add_space(8.0);
    validation(ui, settings, tokens, i18n);
    keyring_status(ui, settings.keyring_state, tokens, i18n);
    ui.add_space(8.0);
    action_bar(ui, settings, commands, i18n);

    if settings.delete_confirmation_open {
        delete_confirmation(ui, settings, commands, i18n);
    }
}

fn header(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, i18n: &I18n) {
    ui.horizontal_wrapped(|ui| {
        ui.heading(i18n.text("connection-settings-title"));
        if let Some(connection) = snapshot.selected_connection() {
            ui.label(RichText::new(&connection.name).strong());
            ui.label(RichText::new(&connection.endpoint).color(tokens.text_secondary));
        }
    });
}

fn tab_bar(ui: &mut Ui, selected: ConnectionSettingsTab, commands: &AppCommandSender, i18n: &I18n) {
    ui.horizontal_wrapped(|ui| {
        for tab in ConnectionSettingsTab::ALL {
            if ui
                .selectable_label(selected == tab, i18n.connection_settings_tab_label(tab))
                .clicked()
            {
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
    i18n: &I18n,
) {
    readonly(
        ui,
        &i18n.text("connection-internal-id"),
        &settings.internal_id,
    );
    field(
        ui,
        &i18n.text("connection-name"),
        &settings.profile_name,
        ConnectionSettingField::ProfileName,
        commands,
    );
    field(
        ui,
        &i18n.text("connection-host"),
        &settings.host,
        ConnectionSettingField::Host,
        commands,
    );
    field(
        ui,
        &i18n.text("connection-port"),
        &settings.port,
        ConnectionSettingField::Port,
        commands,
    );
    combo(
        ui,
        &i18n.text("connection-mqtt-version"),
        &settings.mqtt_version,
        ConnectionSettingField::MqttVersion,
        &["MQTT 3.1.1", "MQTT v5"],
        commands,
    );
    flag(
        ui,
        &i18n.text("connection-clean-session"),
        settings.clean_session,
        ConnectionSettingFlag::CleanSession,
        commands,
    );
    field_with_button(
        ui,
        &i18n.text("connection-client-id"),
        &settings.client_id,
        ConnectionSettingField::ClientId,
        &i18n.text("connection-generate"),
        AppCommand::GenerateClientId,
        commands,
    );
    field(
        ui,
        &i18n.text("connection-username"),
        &settings.username,
        ConnectionSettingField::Username,
        commands,
    );
    secret_field(
        ui,
        &i18n.text("connection-password"),
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
    i18n: &I18n,
) {
    combo(
        ui,
        &i18n.text("connection-tls-ssl"),
        &settings.tls_mode,
        ConnectionSettingField::TlsMode,
        &["No TLS/SSL", "Keystore"],
        commands,
    );
    file_field(
        ui,
        &i18n.text("connection-ssl-keystore"),
        &settings.tls_store,
        ConnectionSettingField::TlsStore,
        true,
        commands,
    );
    secret_field(
        ui,
        &i18n.text("connection-ssl-password"),
        &settings.tls_keystore_password,
        &settings.tls_password_status,
        ConnectionSecretField::TlsKeystorePassword,
        _tokens,
        commands,
    );
    flag(
        ui,
        &i18n.text("connection-verify-hostname"),
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
    i18n: &I18n,
) {
    combo(
        ui,
        &i18n.text("connection-proxy-mode"),
        &settings.proxy_mode,
        ConnectionSettingField::ProxyMode,
        &["No proxy/tunnel", "SSH"],
        commands,
    );
    ui.add_enabled_ui(settings.proxy_mode == "SSH", |ui| {
        field(
            ui,
            &i18n.text("connection-ssh-host"),
            &settings.ssh_host,
            ConnectionSettingField::SshHost,
            commands,
        );
        field(
            ui,
            &i18n.text("connection-ssh-port"),
            &settings.ssh_port,
            ConnectionSettingField::SshPort,
            commands,
        );
        field(
            ui,
            &i18n.text("connection-local-mqtt-port"),
            &settings.local_mqtt_port,
            ConnectionSettingField::LocalMqttPort,
            commands,
        );
        combo(
            ui,
            &i18n.text("connection-authentication"),
            &settings.auth_mode,
            ConnectionSettingField::AuthMode,
            &["No Auth", "Keyfile", "Password"],
            commands,
        );
        field(
            ui,
            &i18n.text("connection-ssh-username"),
            &settings.auth_username,
            ConnectionSettingField::AuthUsername,
            commands,
        );
        secret_field_enabled(
            ui,
            &i18n.text("connection-ssh-password"),
            &settings.ssh_password,
            &settings.ssh_password_status,
            ConnectionSecretField::SshPassword,
            settings.auth_mode == "Password",
            _tokens,
            commands,
        );
        file_field(
            ui,
            &i18n.text("connection-ssh-key-file"),
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
    i18n: &I18n,
) {
    let mut enabled = settings.lwt_enabled;
    if ui
        .checkbox(&mut enabled, i18n.text("connection-enable-last-will"))
        .changed()
    {
        send(commands, AppCommand::SetLwtEnabled(enabled));
    }
    ui.add_enabled_ui(settings.lwt_enabled, |ui| {
        field(
            ui,
            &i18n.text("connection-lwt-topic"),
            &settings.lwt_topic,
            ConnectionSettingField::LwtTopic,
            commands,
        );
        flag(
            ui,
            &i18n.text("connection-lwt-retained"),
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
        ui.label(
            RichText::new(i18n.text("connection-last-will-inactive")).color(tokens.text_disabled),
        );
    }
}

fn validation(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    tokens: ThemeTokens,
    i18n: &I18n,
) {
    if settings.validation_errors.is_empty() {
        ui.label(RichText::new(i18n.text("connection-no-validation-errors")).color(tokens.success));
    } else {
        for error in &settings.validation_errors {
            ui.label(RichText::new(error).color(tokens.warning));
        }
    }
}

fn keyring_status(ui: &mut Ui, state: KeyringState, tokens: ThemeTokens, i18n: &I18n) {
    let color = match state {
        KeyringState::Available => tokens.success,
        KeyringState::Locked | KeyringState::MigrationRequired => tokens.warning,
        KeyringState::Unavailable => tokens.danger,
    };
    ui.label(RichText::new(i18n.keyring_state_label(state)).color(color));
}

fn action_bar(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(settings.dirty, Button::new(i18n.text("common-discard")))
            .clicked()
        {
            send(commands, AppCommand::DiscardConnectionSettings);
        }
        if ui
            .button(format!("{}...", i18n.text("common-delete")))
            .clicked()
        {
            send(commands, AppCommand::RequestDeleteConnection);
        }
        let can_save = settings.dirty && settings.valid;
        let save = ui.add_enabled(can_save, Button::new(i18n.text("common-save")));
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
    i18n: &I18n,
) {
    Window::new(i18n.text("connection-delete-title"))
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.label(format!(
                "{} {}?",
                i18n.text("common-delete"),
                settings.profile_name
            ));
            ui.label(i18n.text("connection-delete-detail"));
            ui.horizontal(|ui| {
                if ui.button(i18n.text("common-cancel")).clicked() {
                    send(commands, AppCommand::CancelDeleteConnection);
                }
                if ui.button(i18n.text("common-delete")).clicked() {
                    send(commands, AppCommand::ConfirmDeleteConnection);
                }
            });
        });
}
