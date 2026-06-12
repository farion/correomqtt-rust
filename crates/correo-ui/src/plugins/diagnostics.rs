use correo_core::{
    AppCommand, AppCommandSender, PluginDiagnosticRow, PluginDiagnosticSeverity,
    PluginSurfaceSnapshot,
};
use egui::{RichText, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};

use crate::theme::ThemeTokens;
use crate::widgets::clearable_search_edit;

const COMPACT_WIDTH: f32 = 760.0;

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        let mut filter = plugins.diagnostic_filter.clone();
        if clearable_search_edit(
            ui,
            Some(super::keyboard::plugin_search_id()),
            &mut filter,
            "Filter diagnostics...",
            220.0,
        )
        .changed()
        {
            send(commands, AppCommand::SearchPluginDiagnostics(filter));
        }
        ui.label(RichText::new(diagnostic_count(plugins)).color(tokens.text_secondary));
        if ui.button("Clear diagnostics").clicked() {
            send(commands, AppCommand::ClearPluginDiagnostics);
        }
    });
    ui.separator();
    if ui.available_width() < COMPACT_WIDTH {
        compact_diagnostics(ui, plugins, tokens, commands);
    } else {
        ui.columns(2, |columns| {
            diagnostics_table(&mut columns[0], plugins, tokens, commands);
            diagnostic_detail(&mut columns[1], plugins.selected_diagnostic(), tokens);
        });
    }
}

fn compact_diagnostics(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ScrollArea::vertical()
        .id_salt("plugin-diagnostics-compact")
        .max_height(260.0)
        .show(ui, |ui| {
            for diagnostic in plugins.filtered_diagnostics() {
                compact_diagnostic_row(ui, plugins, diagnostic, tokens, commands);
                ui.separator();
            }
        });
    ui.add_space(8.0);
    diagnostic_detail(ui, plugins.selected_diagnostic(), tokens);
}

fn compact_diagnostic_row(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    diagnostic: &PluginDiagnosticRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(diagnostic.severity.label())
                .color(diagnostic_color(diagnostic.severity, tokens)),
        );
        ui.label(RichText::new(&diagnostic.plugin_id).color(tokens.text_secondary));
        ui.label(RichText::new(&diagnostic.occurred_at).small());
    });
    if ui
        .selectable_label(
            plugins.selected_diagnostic_id.as_ref() == Some(&diagnostic.id),
            &diagnostic.message,
        )
        .clicked()
    {
        send(
            commands,
            AppCommand::SelectPluginDiagnostic(diagnostic.id.clone()),
        );
    }
}

fn diagnostics_table(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(82.0))
        .column(Column::exact(160.0))
        .column(Column::remainder())
        .column(Column::exact(86.0))
        .header(22.0, |mut header| {
            for title in ["Severity", "Plugin", "Message", "Time"] {
                header.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|mut body| {
            for diagnostic in plugins.filtered_diagnostics() {
                body.row(38.0, |mut row| {
                    row.col(|ui| {
                        ui.label(
                            RichText::new(diagnostic.severity.label())
                                .color(diagnostic_color(diagnostic.severity, tokens)),
                        );
                    });
                    row.col(|ui| {
                        ui.label(&diagnostic.plugin_id);
                    });
                    row.col(|ui| {
                        if ui
                            .selectable_label(
                                plugins.selected_diagnostic_id.as_ref() == Some(&diagnostic.id),
                                &diagnostic.message,
                            )
                            .clicked()
                        {
                            send(
                                commands,
                                AppCommand::SelectPluginDiagnostic(diagnostic.id.clone()),
                            );
                        }
                    });
                    row.col(|ui| {
                        ui.label(&diagnostic.occurred_at);
                    });
                });
            }
        });
}

fn diagnostic_detail(ui: &mut Ui, diagnostic: Option<&PluginDiagnosticRow>, tokens: ThemeTokens) {
    let Some(diagnostic) = diagnostic else {
        ui.label(RichText::new("No diagnostic selected").color(tokens.text_secondary));
        return;
    };

    ui.heading(diagnostic.severity.label());
    ui.label(format!("Plugin: {}", diagnostic.plugin_id));
    if let Some(hook) = diagnostic.hook {
        ui.label(format!("Hook: {}", hook.label()));
    }
    ui.label(RichText::new(&diagnostic.occurred_at).color(tokens.text_secondary));
    ui.separator();
    ui.label(RichText::new(&diagnostic.message).strong());
    ui.label(&diagnostic.detail);
}

fn diagnostic_count(plugins: &PluginSurfaceSnapshot) -> String {
    let total = plugins.diagnostics().len();
    let filtered = plugins.filtered_diagnostics().len();
    if plugins.diagnostic_filter.trim().is_empty() {
        format!("{total} diagnostics")
    } else {
        format!("{filtered} of {total} diagnostics")
    }
}

fn diagnostic_color(severity: PluginDiagnosticSeverity, tokens: ThemeTokens) -> egui::Color32 {
    match severity {
        PluginDiagnosticSeverity::Info => tokens.accent,
        PluginDiagnosticSeverity::Warning => tokens.warning,
        PluginDiagnosticSeverity::Error => tokens.danger,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
