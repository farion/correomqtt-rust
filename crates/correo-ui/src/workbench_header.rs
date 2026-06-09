use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectDisabledReason, ConnectionState,
    ConnectionSummary,
};
use egui::{Button, RichText, Ui};
use egui_phosphor::regular;

use crate::{
    theme::ThemeTokens,
    widgets::{square_icon_button_size, with_icon_button_padding},
    workbench_layout::{self, WorkbenchLayoutMode},
};

pub fn connection_header(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) -> WorkbenchLayoutMode {
    let Some(connection) = snapshot.selected_connection() else {
        return workbench_layout::current_mode(ui);
    };

    let mut mode = workbench_layout::current_mode(ui);
    ui.horizontal(|ui| {
        ui.label(RichText::new(&connection.name).strong().size(18.0));
        mode = workbench_layout::mode_buttons(ui, tokens);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            connection_action(ui, connection, commands);
            state_label(ui, connection, tokens);
        });
    });
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        metadata_label(ui, &connection.endpoint, tokens);
        ui.separator();
        ui.label(&connection.mqtt_version);
        for badge in &connection.badges {
            ui.label(RichText::new(badge.label()).color(tokens.accent).strong());
        }
        if !snapshot.workbench.reconnect_status.is_empty() {
            ui.separator();
            ui.label(
                RichText::new(&snapshot.workbench.reconnect_status).color(tokens.text_secondary),
            )
            .on_hover_text(&snapshot.workbench.reconnect_status);
        }
    });
    mode
}

fn metadata_label(ui: &mut Ui, label: &str, tokens: ThemeTokens) {
    ui.label(RichText::new(label).color(tokens.text_secondary))
        .on_hover_text(label);
}

fn connection_action(ui: &mut Ui, connection: &ConnectionSummary, commands: &AppCommandSender) {
    let action = action_for(connection);
    let response = with_icon_button_padding(ui, |ui| {
        ui.add_enabled(
            action.enabled,
            Button::new(action.label).min_size(egui::vec2(96.0, square_icon_button_size()[1])),
        )
    })
    .on_hover_text(action.tooltip);
    if response.clicked() {
        send(commands, action.command);
    }
}

fn state_label(ui: &mut Ui, connection: &ConnectionSummary, tokens: ThemeTokens) {
    ui.label(
        RichText::new(format!(
            "{} {}",
            state_icon(connection.state),
            connection.state.label()
        ))
        .color(state_color(connection.state, tokens)),
    )
    .on_hover_text(connection.state.label());
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
