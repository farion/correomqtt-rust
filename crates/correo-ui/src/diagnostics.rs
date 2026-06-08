use correo_core::{AppCommand, AppCommandSender, AppSnapshot, Diagnostic};
use egui::{Button, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::theme::ThemeTokens;

pub fn strip(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.vertical(|ui| {
        ui.horizontal_centered(|ui| {
            let label = if snapshot.diagnostics_expanded {
                "Collapse"
            } else {
                "Expand"
            };
            if ui.add_sized([76.0, 22.0], Button::new(label)).clicked() {
                let _ = commands.send(AppCommand::ToggleDiagnostics);
            }
            ui.label(
                RichText::new(format!(
                    "{} diagnostics: {}",
                    snapshot.diagnostics.len(),
                    latest_message(snapshot)
                ))
                .color(tokens.text_secondary),
            );
        });

        if snapshot.diagnostics_expanded {
            ui.separator();
            table(ui, &snapshot.diagnostics, tokens, 24.0);
        }
    });
}

pub fn workspace(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens) {
    ui.heading("Diagnostics");
    ui.label(RichText::new(latest_message(snapshot)).color(tokens.text_secondary));
    ui.add_space(8.0);
    table(ui, &snapshot.diagnostics, tokens, 28.0);
}

fn table(ui: &mut Ui, diagnostics: &[Diagnostic], tokens: ThemeTokens, row_height: f32) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(96.0))
        .column(Column::remainder())
        .header(22.0, |mut header| {
            header.col(|ui| {
                ui.strong("Severity");
            });
            header.col(|ui| {
                ui.strong("Message");
            });
        })
        .body(|mut body| {
            for diagnostic in diagnostics {
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label(
                            RichText::new(diagnostic.severity.label())
                                .color(tokens.severity(diagnostic.severity)),
                        );
                    });
                    row.col(|ui| {
                        ui.label(&diagnostic.message);
                    });
                });
            }
        });
}

fn latest_message(snapshot: &AppSnapshot) -> &str {
    snapshot
        .diagnostics
        .first()
        .map(|diagnostic| diagnostic.message.as_str())
        .unwrap_or("No diagnostics")
}
