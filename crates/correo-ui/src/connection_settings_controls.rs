use correo_core::{
    AppCommand, AppCommandSender, ConnectionSecretField, ConnectionSettingField,
    ConnectionSettingFlag, SecretInput,
};
use egui::{ComboBox, Frame, RichText, Stroke, TextEdit, Ui};

use crate::theme::ThemeTokens;

pub(super) fn field(
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

pub(super) fn field_with_button(
    ui: &mut Ui,
    label: &str,
    value: &str,
    field: ConnectionSettingField,
    button: &str,
    command: AppCommand,
    commands: &AppCommandSender,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);
        ui.label(label);
        let mut edited = value.to_owned();
        if ui
            .add_sized(
                [(ui.available_width() - 96.0).max(160.0), 26.0],
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
        if ui.button(button).clicked() {
            send(commands, command);
        }
    });
}

pub(super) fn combo(
    ui: &mut Ui,
    label: &str,
    value: &str,
    field: ConnectionSettingField,
    options: &[&str],
    commands: &AppCommandSender,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);
        ui.label(label);
        let mut selected = value.to_owned();
        ComboBox::from_id_salt(label)
            .selected_text(value)
            .width((ui.available_width() - 8.0).max(160.0))
            .show_ui(ui, |ui| {
                for option in options {
                    ui.selectable_value(&mut selected, (*option).to_owned(), *option);
                }
            });
        if selected != value {
            send(
                commands,
                AppCommand::UpdateConnectionSetting {
                    field,
                    value: selected,
                },
            );
        }
    });
}

pub(super) fn flag(
    ui: &mut Ui,
    label: &str,
    value: bool,
    flag: ConnectionSettingFlag,
    commands: &AppCommandSender,
) {
    let mut enabled = value;
    if ui.checkbox(&mut enabled, label).changed() {
        send(
            commands,
            AppCommand::SetConnectionSettingFlag { flag, enabled },
        );
    }
}

pub(super) fn secret_field(
    ui: &mut Ui,
    label: &str,
    value: &SecretInput,
    status: &str,
    field: ConnectionSecretField,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    secret_field_enabled(ui, label, value, status, field, true, tokens, commands);
}

pub(super) fn secret_field_enabled(
    ui: &mut Ui,
    label: &str,
    value: &SecretInput,
    status: &str,
    field: ConnectionSecretField,
    enabled: bool,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            ui.set_min_height(30.0);
            ui.label(label);
            let mut edited = value.expose_for_ui().to_owned();
            if ui
                .add_sized(
                    [(ui.available_width() - 8.0).max(160.0), 26.0],
                    TextEdit::singleline(&mut edited).password(true),
                )
                .changed()
            {
                send(
                    commands,
                    AppCommand::UpdateConnectionSecret {
                        field,
                        value: SecretInput::new(edited),
                    },
                );
            }
        });
    });
    if !status.is_empty() {
        ui.label(RichText::new(status).color(tokens.text_secondary));
    }
}

pub(super) fn file_field(
    ui: &mut Ui,
    label: &str,
    value: &str,
    field: ConnectionSettingField,
    enabled: bool,
    commands: &AppCommandSender,
) {
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            ui.set_min_height(30.0);
            ui.label(label);
            let mut edited = value.to_owned();
            if ui
                .add_sized(
                    [(ui.available_width() - 96.0).max(160.0), 26.0],
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
            if ui.button("Choose...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    send(
                        commands,
                        AppCommand::UpdateConnectionSetting {
                            field,
                            value: path.display().to_string(),
                        },
                    );
                }
            }
        });
    });
}

pub(super) fn readonly(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);
        ui.label(label);
        ui.monospace(value);
    });
}

pub(super) fn panel(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

pub(super) fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
