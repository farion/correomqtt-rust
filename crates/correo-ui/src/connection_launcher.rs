use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectDisabledReason, ConnectionBadge,
    ConnectionState, ConnectionSummary,
};
use egui::{Button, RichText, TextEdit, Ui};

use crate::theme::ThemeTokens;

pub fn panel(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal(|ui| {
        ui.heading("Connections");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Import").on_hover_text("Import .cqc").clicked() {
                send(commands, AppCommand::ImportConnections);
            }
            if ui.button("Add").on_hover_text("Add connection").clicked() {
                send(commands, AppCommand::AddConnection);
            }
        });
    });
    ui.separator();

    let mut filter = snapshot.connection_filter.clone();
    let response = ui.add(
        TextEdit::singleline(&mut filter)
            .hint_text("Search")
            .desired_width(f32::INFINITY),
    );
    if response.changed() {
        send(commands, AppCommand::SearchConnections(filter));
    }

    ui.add_space(8.0);
    for connection in snapshot.filtered_connections() {
        connection_row(ui, connection, snapshot, tokens, commands);
    }
}

fn connection_row(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let selected = snapshot.selected_connection == Some(connection.id);
    let response = ui.selectable_label(selected, RichText::new(&connection.name).strong());
    if response.clicked() {
        send(commands, AppCommand::SelectConnection(connection.id));
    }

    ui.horizontal(|ui| {
        ui.label(RichText::new(&connection.endpoint).color(tokens.text_secondary));
        ui.label(
            RichText::new(connection.state.label()).color(state_color(connection.state, tokens)),
        );
        for badge in &connection.badges {
            ui.label(RichText::new(badge_label(*badge)).color(tokens.accent));
        }
    });
    ui.horizontal(|ui| {
        if connection.state == ConnectionState::Connected {
            if ui.button("Open").clicked() {
                send(commands, AppCommand::OpenConnectionWorkbench(connection.id));
            }
        } else {
            let connect = ui.add_enabled(connection.can_connect(), Button::new("Connect"));
            if connect.clicked() {
                send(commands, AppCommand::Connect(connection.id));
            }
            if !connection.can_connect() {
                connect.on_hover_text(disabled_reason(connection).label());
            }
        }
        if ui.button("Edit").clicked() {
            send(commands, AppCommand::OpenConnectionSettings(connection.id));
        }
        if ui.button("Duplicate").clicked() {
            send(commands, AppCommand::DuplicateConnection(connection.id));
        }
    });
    if let Some(reason) = connection.disabled_reason {
        ui.label(RichText::new(reason.label()).color(tokens.warning));
    }
    ui.add_space(8.0);
}

fn disabled_reason(connection: &ConnectionSummary) -> ConnectDisabledReason {
    connection
        .disabled_reason
        .unwrap_or(ConnectDisabledReason::Busy)
}

fn badge_label(badge: ConnectionBadge) -> &'static str {
    badge.label()
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
