use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectDisabledReason, ConnectionState,
    ConnectionSummary,
};
use egui::{Button, RichText, Ui};
use egui_phosphor::regular;

use crate::{
    responsive,
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
    if responsive::connections_context_is_compact(ui.ctx(), snapshot.active_workspace) {
        compact_connection_header(ui, connection, tokens, commands);
        return;
    }

    ui.horizontal(|ui| {
        ui.heading(&connection.name);
        if header_icon_button(ui, regular::PENCIL_SIMPLE, "Edit connection").clicked() {
            send(commands, AppCommand::OpenConnectionSettings(connection.id));
        }
        if header_icon_button(ui, regular::TRASH, "Delete connection").clicked() {
            send(commands, AppCommand::RequestDeleteConnection);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            connection_action(ui, connection, commands, false);
            connection_summary(ui, connection, tokens);
        });
    });
}

fn compact_connection_header(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), square_icon_button_size()[1]),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let icon_actions = responsive::workbench_uses_icon_actions(ui.available_width());
            let connection_action_width = if icon_actions {
                square_icon_button_size()[0]
            } else {
                104.0
            };
            let center_width = (ui.available_width()
                - square_icon_button_size()[0]
                - connection_action_width
                - square_icon_button_size()[0]
                - (ui.spacing().item_spacing.x * 3.0))
                .max(80.0);
            if header_icon_button(ui, regular::LIST, "Show connections").clicked() {
                responsive::open_connection_flyout(ui.ctx());
            }
            connection_title(ui, connection, tokens, center_width);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                compact_overflow_menu(ui, connection, commands);
                connection_action(ui, connection, commands, icon_actions);
            });
        },
    );
}

fn connection_title(ui: &mut Ui, connection: &ConnectionSummary, tokens: ThemeTokens, width: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, square_icon_button_size()[1]),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_clip_rect(ui.max_rect());
            ui.label(
                RichText::new(state_icon(connection.state))
                    .size(16.0)
                    .color(state_color(connection.state, tokens)),
            )
            .on_hover_text(connection.state.label());
            ui.label(RichText::new(&connection.name).strong().size(18.0))
                .on_hover_text(format!(
                    "{} · {}",
                    connection.state.label(),
                    connection.endpoint
                ));
        },
    );
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

fn connection_action(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    commands: &AppCommandSender,
    compact: bool,
) {
    let action = action_for(connection);
    let response = with_icon_button_padding(ui, |ui| {
        if compact {
            ui.add_enabled(
                action.enabled,
                Button::new(RichText::new(regular::PLUG).size(16.0)).min_size(egui::vec2(
                    square_icon_button_size()[0],
                    square_icon_button_size()[1],
                )),
            )
        } else {
            ui.add_enabled(
                action.enabled,
                Button::new(format!("{}  {}", regular::PLUG, action.label))
                    .min_size(egui::vec2(104.0, square_icon_button_size()[1])),
            )
        }
    })
    .on_hover_text(if compact {
        format!("{}: {}", action.label, action.tooltip)
    } else {
        action.tooltip
    });
    if response.clicked() {
        send(commands, action.command);
    }
}

fn compact_overflow_menu(ui: &mut Ui, connection: &ConnectionSummary, commands: &AppCommandSender) {
    let response = with_icon_button_padding(ui, |ui| {
        ui.menu_button(
            RichText::new(regular::DOTS_THREE_VERTICAL).size(16.0),
            |ui| {
                if ui
                    .button(menu_label(regular::PENCIL_SIMPLE, "Edit connection"))
                    .clicked()
                {
                    send(commands, AppCommand::OpenConnectionSettings(connection.id));
                    ui.close_menu();
                }
                if ui
                    .button(menu_label(regular::TRASH, "Delete connection..."))
                    .clicked()
                {
                    send(commands, AppCommand::RequestDeleteConnection);
                    ui.close_menu();
                }
            },
        )
        .response
    });
    response.on_hover_text("Connection actions");
}

fn menu_label(icon: &str, label: &str) -> String {
    format!("{icon}  {label}")
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
