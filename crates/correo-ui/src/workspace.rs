use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionSurface, Workspace};
use egui::{RichText, Ui};

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
        ConnectionSurface::Launcher | ConnectionSurface::Workbench => {
            connection_workbench(ui, snapshot, tokens, commands, i18n)
        }
        ConnectionSurface::Settings => {
            connection_settings::show(ui, snapshot, tokens, commands, i18n)
        }
        ConnectionSurface::Transfer => {
            skeletons::connection_transfer(ui, snapshot, tokens, commands)
        }
    }
    if matches!(
        snapshot.connection_surface,
        ConnectionSurface::Launcher | ConnectionSurface::Workbench
    ) {
        connection_settings::overlay(ui, connection_view_rect, snapshot, tokens, commands, i18n);
    }
}

fn connection_workbench(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    if snapshot.selected_connection().is_some() {
        workbench::show(ui, snapshot, tokens, commands);
    } else {
        no_connection_available(ui, tokens, i18n);
    }
}

fn no_connection_available(ui: &mut Ui, tokens: ThemeTokens, i18n: &I18n) {
    ui.allocate_ui_with_layout(
        ui.available_size(),
        egui::Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| {
            ui.label(
                RichText::new(i18n.text("connection-none-available"))
                    .size(16.0)
                    .color(tokens.text_secondary),
            );
        },
    );
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
