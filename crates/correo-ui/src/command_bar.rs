use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionState, ThemeMode};
use egui::{Align, Button, ComboBox, Layout, RichText, Ui};

use crate::theme::ThemeTokens;

pub fn menu_bar(ui: &mut Ui, commands: &AppCommandSender) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Add connection").clicked() {
                let _ = commands.send(AppCommand::AddConnection);
                ui.close_menu();
            }
            if ui.button("Import .cqc").clicked() {
                let _ = commands.send(AppCommand::ImportConnections);
                ui.close_menu();
            }
            if ui.button("Export .cqc").clicked() {
                let _ = commands.send(AppCommand::ExportConnections);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Import messages...").clicked() {
                let _ = commands.send(AppCommand::ImportMessages);
                ui.close_menu();
            }
            if ui.button("Export messages...").clicked() {
                let _ = commands.send(AppCommand::ExportMessages);
                ui.close_menu();
            }
        });
        ui.menu_button("Edit", |ui| {
            ui.add_enabled(false, Button::new("Undo"));
            ui.add_enabled(false, Button::new("Redo"));
        });
        ui.menu_button("View", |ui| {
            if ui.button("Toggle diagnostics").clicked() {
                let _ = commands.send(AppCommand::ToggleDiagnostics);
                ui.close_menu();
            }
        });
        ui.menu_button("Tools", |ui| {
            ui.add_enabled(false, Button::new("Run script"));
            ui.add_enabled(false, Button::new("Plugin manager"));
        });
        ui.menu_button("Help", |ui| {
            ui.add_enabled(false, Button::new("About CorreoMQTT"));
        });
    });
}

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
            let diagnostics = format!("Diagnostics {}", snapshot.diagnostics.len());
            if ui
                .add(Button::new(
                    RichText::new(diagnostics).color(highest_diagnostic_color(snapshot, tokens)),
                ))
                .on_hover_text("Open diagnostics")
                .clicked()
            {
                let _ = commands.send(AppCommand::ToggleDiagnostics);
            }
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

fn highest_diagnostic_color(snapshot: &AppSnapshot, tokens: ThemeTokens) -> egui::Color32 {
    if snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == correo_core::DiagnosticSeverity::Error)
    {
        tokens.danger
    } else if snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == correo_core::DiagnosticSeverity::Warning)
    {
        tokens.warning
    } else {
        tokens.accent
    }
}
