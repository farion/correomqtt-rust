use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectDisabledReason, ConnectionSummary,
    ConnectionSurface, Workspace,
};
use egui::{Button, RichText, Ui};

use crate::{
    connection_settings, diagnostics, plugins, scripts, settings, skeletons, theme::ThemeTokens,
    workbench,
};

pub fn sidebar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    workspace: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading(workspace.label());
    ui.separator();
    match workspace {
        Workspace::ImportExport => transfer_sidebar(ui, snapshot, tokens, commands),
        Workspace::Scripts => scripts::sidebar(ui, &snapshot.scripts, tokens, commands),
        Workspace::Plugins => plugins::sidebar(ui, &snapshot.plugins, tokens, commands),
        Workspace::Diagnostics => diagnostics_sidebar(ui, tokens, commands),
        Workspace::Settings => settings::sidebar(ui, snapshot, tokens, commands),
        Workspace::Connections => {}
    }
}

pub fn show(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, commands: &AppCommandSender) {
    match snapshot.active_workspace {
        Workspace::Connections => connections(ui, snapshot, tokens, commands),
        Workspace::ImportExport => skeletons::import_export(ui, snapshot, tokens, commands),
        Workspace::Scripts => scripts::show(ui, snapshot, tokens, commands),
        Workspace::Plugins => plugins::show(ui, snapshot, tokens, commands),
        Workspace::Diagnostics => diagnostics::workspace(ui, snapshot, tokens),
        Workspace::Settings => settings::show(ui, snapshot, tokens, commands),
    }
}

fn connections(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    match snapshot.connection_surface {
        ConnectionSurface::Launcher => launcher_detail(ui, snapshot, tokens, commands),
        ConnectionSurface::Workbench => workbench::show(ui, snapshot, tokens, commands),
        ConnectionSurface::Settings => connection_settings::show(ui, snapshot, tokens, commands),
    }
}

fn launcher_detail(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading("Connection Launcher");
    ui.separator();
    if let Some(connection) = snapshot.selected_connection() {
        ui.label(RichText::new(&connection.name).strong().size(18.0));
        ui.label(RichText::new(&connection.endpoint).color(tokens.text_secondary));
        ui.add_space(8.0);
        action_bar(ui, connection, commands);
        if let Some(reason) = connection.disabled_reason {
            ui.label(
                RichText::new(format!("Connect disabled: {}", reason.label()))
                    .color(tokens.warning),
            );
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(format!("State: {}", connection.state.label()));
            ui.separator();
            ui.label(format!("Protocol: {}", connection.mqtt_version));
            ui.separator();
            ui.label(format!(
                "Subscriptions: {}",
                connection.recent_subscriptions
            ));
            ui.separator();
            ui.label(format!("Messages: {}", connection.recent_messages));
        });
        ui.label(RichText::new(&connection.last_activity).color(tokens.text_secondary));
    } else {
        ui.label("No connection selected");
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        if ui.button("Import").clicked() {
            send(commands, AppCommand::ImportConnections);
        }
        if ui.button("Add").clicked() {
            send(commands, AppCommand::AddConnection);
        }
    });
}

fn action_bar(ui: &mut Ui, connection: &ConnectionSummary, commands: &AppCommandSender) {
    ui.horizontal(|ui| {
        if connection.state == correo_core::ConnectionState::Connected {
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
}

fn disabled_reason(connection: &ConnectionSummary) -> ConnectDisabledReason {
    connection
        .disabled_reason
        .unwrap_or(ConnectDisabledReason::Busy)
}

fn transfer_sidebar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    for section in correo_core::TransferSection::ALL {
        let selected = snapshot.transfer.active_section == section;
        if ui.selectable_label(selected, section.label()).clicked() {
            send(commands, AppCommand::SelectTransferSection(section));
        }
    }
    ui.separator();
    ui.label(
        RichText::new(format!(
            "{} import / {} export selected",
            snapshot.transfer.import.selected_count(),
            snapshot.transfer.export.selected_count()
        ))
        .color(tokens.text_secondary),
    );
    ui.label(
        RichText::new(format!(
            "{} messages selected",
            snapshot.transfer.messages.selected_messages
        ))
        .color(tokens.text_secondary),
    );
    ui.separator();
    if ui.button("Import .cqc").clicked() {
        send(commands, AppCommand::ImportConnections);
    }
    if ui.button("Export .cqc").clicked() {
        send(commands, AppCommand::ExportConnections);
    }
    if ui.button("Import messages").clicked() {
        send(commands, AppCommand::ImportMessages);
    }
    if ui.button("Export messages").clicked() {
        send(commands, AppCommand::ExportMessages);
    }
}

fn diagnostics_sidebar(ui: &mut Ui, tokens: ThemeTokens, commands: &AppCommandSender) {
    if ui.button("Toggle strip").clicked() {
        send(commands, AppCommand::ToggleDiagnostics);
    }
    ui.add_space(8.0);
    for item in ["MQTT", "Migration", "Scripts", "Plugins"] {
        ui.label(RichText::new(item).color(tokens.text_secondary));
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
