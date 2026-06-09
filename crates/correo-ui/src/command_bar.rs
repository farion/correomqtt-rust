use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionState, ThemeMode};
use egui::{Align, ComboBox, Layout, RichText, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

pub fn command_bar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal_centered(|ui| {
        ui.label(
            RichText::new(i18n.workspace_label(snapshot.active_workspace))
                .strong()
                .size(16.0),
        );
        ui.separator();
        if let Some(connection) = snapshot.selected_connection() {
            ui.label(RichText::new(&connection.name).color(tokens.text_primary));
            ui.label(
                RichText::new(i18n.connection_state_label(connection.state))
                    .color(state_color(connection.state, tokens)),
            );
        } else {
            ui.label(
                RichText::new(i18n.text("connection-no-selected")).color(tokens.text_secondary),
            );
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            theme_selector(ui, snapshot.theme_mode, commands, i18n);
        });
    });
}

fn theme_selector(ui: &mut Ui, current: ThemeMode, commands: &AppCommandSender, i18n: &I18n) {
    let mut selected = current;
    ComboBox::from_id_salt("theme-mode")
        .selected_text(i18n.theme_label(current))
        .width(96.0)
        .show_ui(ui, |ui| {
            for mode in ThemeMode::ALL {
                ui.selectable_value(&mut selected, mode, i18n.theme_label(mode));
            }
        });
    if selected != current {
        let _ = commands.send(AppCommand::SetThemeMode(selected));
    }
}

fn state_color(state: ConnectionState, tokens: ThemeTokens) -> egui::Color32 {
    match state {
        ConnectionState::Connected => tokens.success,
        ConnectionState::Connecting | ConnectionState::Reconnecting => tokens.warning,
        ConnectionState::Error => tokens.danger,
        ConnectionState::Disconnected => tokens.text_secondary,
    }
}
