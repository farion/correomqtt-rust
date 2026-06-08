use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionState, ConnectionSummary};
use egui::{Frame, RichText, Stroke, Ui};

use crate::theme::ThemeTokens;

pub fn connection_header(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let Some(connection) = snapshot.selected_connection() else {
        return;
    };

    header_frame(tokens).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&connection.name).strong().size(18.0));
            ui.label(
                RichText::new(connection.state.label())
                    .color(state_color(connection.state, tokens)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                connection_actions(ui, connection, commands);
            });
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            metadata_label(ui, &connection.endpoint, tokens);
            ui.separator();
            ui.label(&connection.mqtt_version);
            for badge in &connection.badges {
                ui.label(RichText::new(badge.label()).color(tokens.accent).strong());
            }
            ui.separator();
            ui.label(
                RichText::new(&snapshot.workbench.reconnect_status).color(tokens.text_secondary),
            )
            .on_hover_text(&snapshot.workbench.reconnect_status);
        });
    });
}

fn connection_actions(ui: &mut Ui, connection: &ConnectionSummary, commands: &AppCommandSender) {
    if ui.button("Disconnect").clicked() {
        send(commands, AppCommand::Disconnect(connection.id));
    }
    if ui.button("Reconnect").clicked() {
        send(commands, AppCommand::Reconnect(connection.id));
    }
}

fn metadata_label(ui: &mut Ui, label: &str, tokens: ThemeTokens) {
    ui.label(RichText::new(label).color(tokens.text_secondary))
        .on_hover_text(label);
}

fn header_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

fn state_color(state: ConnectionState, tokens: ThemeTokens) -> egui::Color32 {
    match state {
        ConnectionState::Connected => tokens.success,
        ConnectionState::Connecting | ConnectionState::Reconnecting => tokens.warning,
        ConnectionState::Error => tokens.danger,
        ConnectionState::Disconnected => tokens.text_secondary,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
