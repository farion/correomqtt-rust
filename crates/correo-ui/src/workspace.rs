use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectDisabledReason, ConnectionSummary,
    ConnectionSurface, Workspace,
};
use egui::{Button, RichText, Ui};

use crate::{
    about, connection_settings, diagnostics, i18n::I18n, plugins, scripts, settings, skeletons,
    theme::ThemeTokens, workbench,
};

pub fn sidebar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    workspace: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.heading(i18n.workspace_label(workspace));
    ui.separator();
    match workspace {
        Workspace::ImportExport => transfer_sidebar(ui, snapshot, tokens, commands, i18n),
        Workspace::Scripts => scripts::sidebar(ui, &snapshot.scripts, tokens, commands),
        Workspace::Plugins => {}
        Workspace::Diagnostics => {}
        Workspace::Settings => {}
        Workspace::About => {}
        Workspace::Connections => {}
    }
}

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    match snapshot.active_workspace {
        Workspace::Connections => connections(ui, snapshot, tokens, commands, i18n),
        Workspace::ImportExport => skeletons::import_export(ui, snapshot, tokens, commands),
        Workspace::Scripts => scripts::show(ui, snapshot, tokens, commands),
        Workspace::Plugins => plugins::show(ui, snapshot, tokens, commands, i18n),
        Workspace::Diagnostics => diagnostics::workspace(ui, snapshot, tokens, i18n),
        Workspace::Settings => settings::show(ui, snapshot, tokens, commands, i18n),
        Workspace::About => about::show(ui, tokens, i18n),
    }
}

fn connections(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let connection_view_rect = ui.max_rect();
    match snapshot.connection_surface {
        ConnectionSurface::Launcher => launcher_detail(ui, snapshot, tokens, commands, i18n),
        ConnectionSurface::Workbench => workbench::show(ui, snapshot, tokens, commands),
        ConnectionSurface::Settings => {
            connection_settings::show(ui, snapshot, tokens, commands, i18n)
        }
        ConnectionSurface::Transfer => {
            skeletons::connection_transfer(ui, snapshot, tokens, commands)
        }
    }
    if snapshot.connection_surface == ConnectionSurface::Workbench {
        connection_settings::overlay(ui, connection_view_rect, snapshot, tokens, commands, i18n);
    }
}

fn launcher_detail(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.heading(i18n.text("connection-launcher"));
    ui.separator();
    if let Some(connection) = snapshot.selected_connection() {
        ui.label(RichText::new(&connection.name).strong().size(18.0));
        ui.label(RichText::new(&connection.endpoint).color(tokens.text_secondary));
        ui.add_space(8.0);
        action_bar(ui, connection, commands, i18n);
        if let Some(reason) = connection.disabled_reason {
            ui.label(
                RichText::new(format!(
                    "{}: {}",
                    i18n.text("connection-connect-disabled"),
                    i18n.disabled_reason_label(reason)
                ))
                .color(tokens.warning),
            );
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(format!(
                "{}: {}",
                i18n.text("connection-state"),
                i18n.connection_state_label(connection.state)
            ));
            ui.separator();
            ui.label(format!(
                "{}: {}",
                i18n.text("connection-protocol"),
                connection.mqtt_version
            ));
            ui.separator();
            ui.label(format!(
                "{}: {}",
                i18n.text("connection-subscriptions"),
                connection.recent_subscriptions
            ));
            ui.separator();
            ui.label(format!(
                "{}: {}",
                i18n.text("connection-messages"),
                connection.recent_messages
            ));
        });
        ui.label(RichText::new(&connection.last_activity).color(tokens.text_secondary));
    } else {
        ui.label(i18n.text("connection-no-selected"));
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        ui.vertical(|ui| {
            if ui.button(i18n.text("common-add-connection")).clicked() {
                send(commands, AppCommand::AddConnection);
            }
            if ui.button(i18n.text("common-import-cqc")).clicked() {
                send(commands, AppCommand::ImportConnections);
            }
            if ui.button(i18n.text("common-export-cqc")).clicked() {
                send(commands, AppCommand::ExportConnections);
            }
        });
    });
}

fn action_bar(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        if connection.state == correo_core::ConnectionState::Connected {
            if ui.button(i18n.text("common-open")).clicked() {
                send(commands, AppCommand::OpenConnectionWorkbench(connection.id));
            }
        } else {
            let connect = ui.add_enabled(
                connection.can_connect(),
                Button::new(i18n.text("common-connect")),
            );
            if connect.clicked() {
                send(commands, AppCommand::Connect(connection.id));
            }
            if !connection.can_connect() {
                connect.on_hover_text(i18n.disabled_reason_label(disabled_reason(connection)));
            }
        }
        if ui.button(i18n.text("common-edit")).clicked() {
            send(commands, AppCommand::OpenConnectionSettings(connection.id));
        }
        if ui.button(i18n.text("common-duplicate")).clicked() {
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
    i18n: &I18n,
) {
    for section in [
        correo_core::TransferSection::Import,
        correo_core::TransferSection::Export,
    ] {
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
    ui.separator();
    if ui.button(i18n.text("common-import-cqc")).clicked() {
        send(commands, AppCommand::ImportConnections);
    }
    if ui.button(i18n.text("common-export-cqc")).clicked() {
        send(commands, AppCommand::ExportConnections);
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
