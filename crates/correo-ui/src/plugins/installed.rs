use correo_core::{AppCommand, AppCommandSender, PluginRow, PluginSurfaceSnapshot};
use egui::{RichText, ScrollArea, Ui};

use crate::theme::ThemeTokens;

use super::{capability_chips, plugin_detail, send, status_color};

const SPLIT_WIDTH: f32 = 760.0;

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filtered = plugins.filtered_plugins();
    if filtered.is_empty() {
        ui.label(
            RichText::new("No installed plugins match this search.").color(tokens.text_secondary),
        );
        return;
    }

    if ui.available_width() < SPLIT_WIDTH {
        plugin_list(ui, plugins, &filtered, tokens, commands);
        ui.separator();
        selected_detail(ui, plugins, tokens, commands);
    } else {
        ui.columns(2, |columns| {
            plugin_list(&mut columns[0], plugins, &filtered, tokens, commands);
            selected_detail(&mut columns[1], plugins, tokens, commands);
        });
    }
}

fn plugin_list(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    filtered: &[&PluginRow],
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading("Installed");
    ui.add_space(4.0);
    ScrollArea::vertical()
        .id_salt("plugin-installed-list")
        .show(ui, |ui| {
            for plugin in filtered {
                plugin_row(ui, plugins, plugin, tokens, commands);
                ui.separator();
            }
        });
}

fn plugin_row(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    plugin: &PluginRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if ui
        .selectable_label(
            plugins.selected_plugin_id == plugin.id,
            RichText::new(&plugin.name).strong(),
        )
        .clicked()
    {
        send(commands, AppCommand::SelectPlugin(plugin.id.clone()));
    }
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(plugin.status.label()).color(status_color(plugin.status, tokens)));
        ui.label(RichText::new(&plugin.version).color(tokens.text_secondary));
        ui.label(RichText::new(plugin.source.label()).color(tokens.text_secondary));
    });
    if !plugin.capabilities.is_empty() {
        capability_chips(ui, plugin, tokens);
    }
}

fn selected_detail(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let Some(plugin) = plugins.selected_plugin() else {
        ui.heading("Plugin Details");
        ui.label(RichText::new("No installed plugin selected").color(tokens.text_secondary));
        return;
    };
    plugin_detail(ui, plugin, tokens, commands);
}
