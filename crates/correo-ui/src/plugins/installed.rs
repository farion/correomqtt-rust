use correo_core::{AppCommand, AppCommandSender, PluginRow, PluginSurfaceSnapshot};
use egui::{RichText, ScrollArea, Ui};

use crate::theme::ThemeTokens;
use crate::widgets::tile_list_content_width;

use super::{
    capability_chips, plugin_detail, plugin_split, plugin_tile, search_field, send, status_color,
};

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filtered = plugins.filtered_plugins();
    plugin_split(
        ui,
        tokens,
        |ui| {
            plugin_list(ui, plugins, &filtered, tokens, commands);
        },
        |ui| selected_detail(ui, plugins, tokens, commands),
    );
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
    search_field(ui, plugins, commands);
    ui.add_space(8.0);
    ScrollArea::vertical()
        .id_salt("plugin-installed-list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(tile_list_content_width(ui));
            if filtered.is_empty() {
                ui.label(
                    RichText::new("No installed plugins match this search.")
                        .color(tokens.text_secondary),
                );
                return;
            }
            for plugin in filtered {
                plugin_row(ui, plugins, plugin, tokens, commands);
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
    let response = plugin_tile(ui, plugins.selected_plugin_id == plugin.id, tokens, |ui| {
        ui.label(RichText::new(&plugin.name).strong());
        ui.label(RichText::new(&plugin.description).color(tokens.text_secondary));
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(plugin.status.label()).color(status_color(plugin.status, tokens)),
            );
            ui.label(RichText::new(&plugin.version).color(tokens.text_secondary));
            ui.label(RichText::new(plugin.source.label()).color(tokens.text_secondary));
        });
        if !plugin.capabilities.is_empty() {
            capability_chips(ui, plugin, tokens);
        }
    });
    if response.clicked() {
        send(commands, AppCommand::SelectPlugin(plugin.id.clone()));
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
