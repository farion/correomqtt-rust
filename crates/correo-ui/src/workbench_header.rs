use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectDisabledReason, ConnectionState,
    ConnectionSummary,
};
use egui::{Button, RichText, Ui};
use egui_phosphor::regular;

use crate::{
    theme::ThemeTokens,
    widgets::{square_icon_button_size, with_icon_button_padding},
};

pub fn connection_header(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let Some(connection) = snapshot.selected_connection() else {
        return;
    };

    ui.horizontal(|ui| {
        ui.heading(&connection.name);
        if header_icon_button(ui, regular::PENCIL_SIMPLE, "Edit connection").clicked() {
            send(commands, AppCommand::OpenConnectionSettings(connection.id));
        }
        if header_icon_button(ui, regular::TRASH, "Delete connection").clicked() {
            send(commands, AppCommand::RequestDeleteConnection);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            connection_action(ui, connection, commands);
            connection_summary(ui, connection, tokens);
        });
    });
}

fn header_icon_button(ui: &mut Ui, icon: &'static str, hover_text: &'static str) -> egui::Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(icon).size(16.0)),
        )
    })
    .on_hover_text(hover_text)
}

fn connection_action(ui: &mut Ui, connection: &ConnectionSummary, commands: &AppCommandSender) {
    let action = action_for(connection);
    let response = with_icon_button_padding(ui, |ui| {
        ui.add_enabled(
            action.enabled,
            Button::new(format!("{}  {}", regular::PLUG, action.label))
                .min_size(egui::vec2(104.0, square_icon_button_size()[1])),
        )
    })
    .on_hover_text(action.tooltip);
    if response.clicked() {
        send(commands, action.command);
    }
}

fn connection_summary(ui: &mut Ui, connection: &ConnectionSummary, tokens: ThemeTokens) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(state_icon(connection.state))
                .size(22.0)
                .color(state_color(connection.state, tokens)),
        )
        .on_hover_text(connection.state.label());
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(
                    RichText::new(connection.state.label())
                        .color(state_color(connection.state, tokens)),
                )
                .on_hover_text(connection.state.label());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(
                    RichText::new(&connection.endpoint)
                        .size(12.0)
                        .color(tokens.text_disabled),
                )
                .on_hover_text(&connection.endpoint);
            });
        });
    });
}

fn action_for(connection: &ConnectionSummary) -> HeaderAction {
    match connection.state {
        ConnectionState::Connected
        | ConnectionState::Connecting
        | ConnectionState::Reconnecting => HeaderAction::new(
            "Disconnect",
            "Disconnect from broker",
            true,
            AppCommand::Disconnect(connection.id),
        ),
        ConnectionState::Error => HeaderAction::new(
            "Reconnect",
            "Reconnect to broker",
            true,
            AppCommand::Reconnect(connection.id),
        ),
        ConnectionState::Disconnected => {
            let tooltip = if connection.can_connect() {
                "Connect to broker".to_owned()
            } else {
                disabled_reason(connection).label().to_owned()
            };
            HeaderAction::new(
                "Connect",
                tooltip,
                connection.can_connect(),
                AppCommand::Connect(connection.id),
            )
        }
    }
}

fn state_icon(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connected
        | ConnectionState::Connecting
        | ConnectionState::Reconnecting => regular::WIFI_HIGH,
        ConnectionState::Disconnected | ConnectionState::Error => regular::WIFI_SLASH,
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

fn disabled_reason(connection: &ConnectionSummary) -> ConnectDisabledReason {
    connection
        .disabled_reason
        .unwrap_or(ConnectDisabledReason::Busy)
}

struct HeaderAction {
    label: &'static str,
    tooltip: String,
    enabled: bool,
    command: AppCommand,
}

impl HeaderAction {
    fn new(
        label: &'static str,
        tooltip: impl Into<String>,
        enabled: bool,
        command: AppCommand,
    ) -> Self {
        Self {
            label,
            tooltip: tooltip.into(),
            enabled,
            command,
        }
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
