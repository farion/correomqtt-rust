use correo_core::{AppCommand, AppCommandSender, PluginRow, PluginSurfaceSnapshot};
use egui::{CentralPanel, Frame, RichText, ScrollArea, SidePanel, Ui};

use crate::theme::ThemeTokens;

use super::{capability_chips, plugin_detail, plugin_tile, search_field, send, status_color};

const LIST_WIDTH: f32 = 340.0;
const MIN_LIST_WIDTH: f32 = 260.0;
const MAX_LIST_WIDTH: f32 = 520.0;

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filtered = plugins.filtered_plugins();
    SidePanel::left("plugin-installed-list-pane")
        .default_width(LIST_WIDTH)
        .width_range(MIN_LIST_WIDTH..=MAX_LIST_WIDTH)
        .resizable(true)
        .frame(pane_frame())
        .show_inside(ui, |ui| {
            plugin_list(ui, plugins, &filtered, tokens, commands);
        });
    CentralPanel::default()
        .frame(pane_frame())
        .show_inside(ui, |ui| selected_detail(ui, plugins, tokens, commands));
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
            ui.set_width(ui.available_width());
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
        ui.label(
            RichText::new(&plugin.description)
                .color(tokens.text_secondary)
                .small(),
        );
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

fn pane_frame() -> Frame {
    Frame::NONE.inner_margin(egui::Margin::same(8))
}
