use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionSecretField, ConnectionSettingField,
    ConnectionSettingFlag, ConnectionSettingsSnapshot, ConnectionSettingsTab, KeyringState,
};
use egui::{
    Button, Color32, CornerRadius, Rect, RichText, ScrollArea, Sense, Stroke, StrokeKind, TextEdit,
    Ui, UiBuilder, Window,
};

use crate::{i18n::I18n, theme::ThemeTokens};

#[path = "connection_settings_controls.rs"]
mod controls;
use controls::{
    combo, field, field_with_button, file_field, flag, panel, secret_field, secret_field_enabled,
    send,
};

const MODAL_SCALE: f32 = 0.95;
const SCRIM_ALPHA: u8 = 112;
const MODAL_RADIUS: u8 = 4;

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    show_body(ui, snapshot, tokens, commands, i18n, false);
}

fn show_body(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    modal: bool,
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
    validation(ui, settings, tokens);
    keyring_status(ui, settings.keyring_state, tokens, i18n);
    ui.add_space(8.0);
    action_bar(ui, settings, commands, i18n, modal);
    ui.add_space(8.0);
    internal_id_hint(ui, settings, tokens, i18n);

    if settings.delete_confirmation_open {
        delete_confirmation(ui, settings, commands, i18n);
    }
}

pub fn overlay(
    ui: &mut Ui,
    view_rect: Rect,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let Some(editor_id) = snapshot.connection_settings_overlay else {
        return;
    };
    if snapshot.selected_connection != Some(editor_id) {
        return;
    }

    let modal_size = view_rect.size() * MODAL_SCALE;
    egui::Area::new(egui::Id::new((
        "connection-settings-overlay",
        editor_id.to_string(),
    )))
    .order(egui::Order::Foreground)
    .fixed_pos(view_rect.min)
    .movable(false)
    .show(ui.ctx(), |ui| {
        let (scrim_rect, _) = ui.allocate_exact_size(view_rect.size(), Sense::click());
        ui.painter().rect_filled(
            scrim_rect,
            CornerRadius::ZERO,
            Color32::from_black_alpha(SCRIM_ALPHA),
        );

        let modal_rect = Rect::from_center_size(scrim_rect.center(), modal_size);
        ui.painter().rect_filled(
            modal_rect,
            CornerRadius::same(MODAL_RADIUS),
            tokens.panel_bg,
        );
        ui.painter().rect_stroke(
            modal_rect,
            CornerRadius::same(MODAL_RADIUS),
            Stroke::new(1.0, tokens.border),
            StrokeKind::Inside,
        );

        let content_rect = modal_rect.shrink(12.0);
        let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
        content_ui.set_min_size(content_rect.size());
        content_ui.set_max_size(content_rect.size());
        let body_id = egui::Id::new(("connection-settings-overlay-body", editor_id.to_string()));
        content_ui.vertical(|ui| {
            ScrollArea::vertical()
                .id_salt(body_id)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    show_body(ui, snapshot, tokens, commands, i18n, true);
                });
        });
    });
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

fn validation(ui: &mut Ui, settings: &ConnectionSettingsSnapshot, tokens: ThemeTokens) {
    for error in &settings.validation_errors {
        ui.label(RichText::new(error).color(tokens.warning));
    }
}

fn keyring_status(ui: &mut Ui, state: KeyringState, tokens: ThemeTokens, i18n: &I18n) {
    let color = match state {
        KeyringState::Available => return,
        KeyringState::Locked => tokens.warning,
        KeyringState::Unavailable => tokens.danger,
    };
    ui.label(RichText::new(i18n.keyring_state_label(state)).color(color));
}

fn internal_id_hint(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    tokens: ThemeTokens,
    i18n: &I18n,
) {
    ui.label(
        RichText::new(format!(
            "{}: {}",
            i18n.text("connection-internal-id"),
            settings.internal_id
        ))
        .monospace()
        .color(tokens.text_secondary),
    );
}

fn action_bar(
    ui: &mut Ui,
    settings: &ConnectionSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
    modal: bool,
) {
    ui.horizontal(|ui| {
        let cancel_label = if modal {
            i18n.text("common-cancel")
        } else {
            i18n.text("common-discard")
        };
        if ui
            .add_enabled(settings.dirty || modal, Button::new(cancel_label))
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
