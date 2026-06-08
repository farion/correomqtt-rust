use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionSettingField, ConnectionSettingsSnapshot,
    ConnectionSettingsTab, KeyringState,
};
use egui::{Button, Frame, RichText, Stroke, TextEdit, Ui, Window};

use crate::theme::ThemeTokens;

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
        ConnectionSettingsTab::Advanced => advanced_tab(ui, settings, tokens),
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
    field(
        ui,
        "MQTT version",
        &settings.mqtt_version,
        ConnectionSettingField::MqttVersion,
        commands,
    );
    field(
        ui,
        "Client id",
        &settings.client_id,
        ConnectionSettingField::ClientId,
        commands,
    );
    field(
        ui,
        "Auth mode",
        &settings.auth_mode,
        ConnectionSettingField::AuthMode,
        commands,
    );
    ui.label(RichText::new(&settings.username_status).color(tokens.text_secondary));
}

fn tls_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    field(
        ui,
        "TLS mode",
        &settings.tls_mode,
        ConnectionSettingField::TlsMode,
        commands,
    );
    field(
        ui,
        "Certificate store",
        &settings.tls_store,
        ConnectionSettingField::TlsStore,
        commands,
    );
    let mut verify_hostname = true;
    let mut client_certificate = false;
    ui.checkbox(&mut verify_hostname, "Verify broker hostname");
    ui.checkbox(
        &mut client_certificate,
        "Use client certificate from keyring-backed profile",
    );
}

fn proxy_tab(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    field(
        ui,
        "Proxy mode",
        &settings.proxy_mode,
        ConnectionSettingField::ProxyMode,
        commands,
    );
    field(
        ui,
        "Tunnel endpoint",
        &settings.proxy_endpoint,
        ConnectionSettingField::ProxyEndpoint,
        commands,
    );
    ui.label("SSH local port conflict check: clear");
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

fn advanced_tab(ui: &mut Ui, settings: &ConnectionSettingsSnapshot, tokens: ThemeTokens) {
    for option in &settings.advanced_options {
        ui.label(RichText::new(option).color(tokens.text_primary));
    }
    ui.separator();
    ui.label(
        RichText::new("Protocol and reconnect defaults are ready for core binding.")
            .color(tokens.text_secondary),
    );
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

fn field(
    ui: &mut Ui,
    label: &str,
    value: &str,
    field: ConnectionSettingField,
    commands: &AppCommandSender,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);
        ui.label(label);
        let mut edited = value.to_owned();
        if ui
            .add_sized(
                [(ui.available_width() - 8.0).max(160.0), 26.0],
                TextEdit::singleline(&mut edited),
            )
            .changed()
        {
            send(
                commands,
                AppCommand::UpdateConnectionSetting {
                    field,
                    value: edited,
                },
            );
        }
    });
}

fn panel(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
