use correo_core::{AppCommand, AppCommandSender, PluginRow, PluginSurfaceSnapshot};
use egui::{RichText, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};

use crate::theme::ThemeTokens;

use super::{capability_chips, enabled_checkbox, send, status_color};

const COMPACT_WIDTH: f32 = 720.0;

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filtered = plugins.filtered_plugins();
    if filtered.is_empty() {
        ui.label(RichText::new("No plugins match this search.").color(tokens.text_secondary));
        return;
    }

    if ui.available_width() < COMPACT_WIDTH {
        compact_list(ui, plugins, filtered, tokens, commands);
    } else {
        table(ui, plugins, filtered, tokens, commands);
    }
}

fn compact_list(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    filtered: Vec<&PluginRow>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ScrollArea::vertical()
        .id_salt("plugin-installed-compact")
        .show(ui, |ui| {
            for plugin in filtered {
                compact_row(ui, plugins, plugin, tokens, commands);
                ui.separator();
            }
        });
}

fn compact_row(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    plugin: &PluginRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        enabled_checkbox(ui, plugin, commands);
        if ui
            .selectable_label(
                plugins.selected_plugin_id == plugin.id,
                RichText::new(&plugin.name).strong(),
            )
            .clicked()
        {
            send(commands, AppCommand::SelectPlugin(plugin.id.clone()));
        }
        ui.label(RichText::new(&plugin.version).color(tokens.text_secondary));
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(plugin.status.label()).color(status_color(plugin.status, tokens)));
        ui.label(RichText::new(plugin.source.label()).color(tokens.text_secondary));
        ui.label(RichText::new(format!("{} diagnostics", plugin.diagnostic_count())).small());
    });
    if !plugin.capabilities.is_empty() {
        capability_chips(ui, plugin, tokens);
    }
    if let Some(note) = &plugin.legacy_note {
        ui.label(RichText::new(note).color(tokens.warning));
    }
}

fn table(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    filtered: Vec<&PluginRow>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(64.0))
        .column(Column::remainder())
        .column(Column::exact(74.0))
        .column(Column::exact(120.0))
        .column(Column::remainder())
        .column(Column::exact(78.0))
        .header(22.0, |mut header| {
            for title in [
                "Enabled",
                "Plugin",
                "Version",
                "Status",
                "Capabilities",
                "Diagnostics",
            ] {
                header.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|mut body| {
            for plugin in filtered {
                body.row(48.0, |mut row| {
                    row.col(|ui| {
                        enabled_checkbox(ui, plugin, commands);
                    });
                    row.col(|ui| {
                        if ui
                            .selectable_label(
                                plugins.selected_plugin_id == plugin.id,
                                RichText::new(&plugin.name).strong(),
                            )
                            .clicked()
                        {
                            send(commands, AppCommand::SelectPlugin(plugin.id.clone()));
                        }
                        ui.label(RichText::new(plugin.source.label()).color(tokens.text_secondary));
                    });
                    row.col(|ui| {
                        ui.label(&plugin.version);
                    });
                    row.col(|ui| {
                        ui.label(
                            RichText::new(plugin.status.label())
                                .color(status_color(plugin.status, tokens)),
                        );
                    });
                    row.col(|ui| capability_chips(ui, plugin, tokens));
                    row.col(|ui| {
                        ui.label(plugin.diagnostic_count().to_string());
                    });
                });
            }
        });
}
