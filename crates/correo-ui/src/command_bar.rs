use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionState, ThemeMode};
use egui::{Align, ComboBox, Layout, RichText, Ui};

use crate::theme::ThemeTokens;

pub fn command_bar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_centered(|ui| {
        ui.label(
            RichText::new(snapshot.active_workspace.label())
                .strong()
                .size(16.0),
        );
        ui.separator();
        if let Some(connection) = snapshot.selected_connection() {
            ui.label(RichText::new(&connection.name).color(tokens.text_primary));
            ui.label(
                RichText::new(connection.state.label())
                    .color(state_color(connection.state, tokens)),
            );
        } else {
            ui.label(RichText::new("No connection selected").color(tokens.text_secondary));
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            theme_selector(ui, snapshot.theme_mode, commands);
        });
    });
}

fn theme_selector(ui: &mut Ui, current: ThemeMode, commands: &AppCommandSender) {
    let mut selected = current;
    ComboBox::from_id_salt("theme-mode")
        .selected_text(current.label())
        .width(96.0)
        .show_ui(ui, |ui| {
            for mode in ThemeMode::ALL {
                ui.selectable_value(&mut selected, mode, mode.label());
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
