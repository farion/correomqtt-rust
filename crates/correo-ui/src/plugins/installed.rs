use correo_core::{AppCommand, AppCommandSender, PluginRow, PluginSurfaceSnapshot};
use egui::{RichText, ScrollArea, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;
use correo_style::layout;

use crate::widgets::{
    fill_remaining_tile_rows, tile_list_content_width, tile_scroll_bar_rect_with_height,
};

use super::{
    capability_chips, plugin_detail, plugin_split, plugin_tile, search_field, send, status_color,
    TILE_HEIGHT,
};

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let filtered = plugins.filtered_plugins();
    plugin_split(
        ui,
        tokens,
        |ui| {
            plugin_list(ui, plugins, &filtered, tokens, commands, i18n);
        },
        |ui| selected_detail(ui, plugins, tokens, commands, i18n),
    );
}

fn plugin_list(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    filtered: &[&PluginRow],
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.heading(i18n.text("plugin-tab-installed"));
    ui.add_space(4.0);
    search_field(ui, plugins, commands, i18n);
    ui.add_space(8.0);
    let list_height = ui.available_height().max(layout::TABLE_MIN_HEIGHT);
    ScrollArea::vertical()
        .id_salt("plugin-installed-list")
        .max_height(list_height)
        .auto_shrink([false, false])
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, list_height))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(tile_list_content_width(ui));
            for (index, plugin) in filtered.into_iter().enumerate() {
                plugin_row(ui, index, plugins, plugin, tokens, commands, i18n);
            }
            fill_remaining_tile_rows(ui, filtered.len(), TILE_HEIGHT, list_height, tokens);
        });
}

fn plugin_row(
    ui: &mut Ui,
    index: usize,
    plugins: &PluginSurfaceSnapshot,
    plugin: &PluginRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let response = plugin_tile(
        ui,
        index,
        plugins.selected_plugin_id == plugin.id,
        tokens,
        |ui| {
            ui.label(RichText::new(&plugin.name).strong());
            ui.label(RichText::new(&plugin.description).color(tokens.text_secondary));
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(i18n.plugin_status_label(plugin.status))
                        .color(status_color(plugin.status, tokens)),
                );
                ui.label(RichText::new(&plugin.version).color(tokens.text_secondary));
                ui.label(
                    RichText::new(i18n.plugin_source_label(plugin.source))
                        .color(tokens.text_secondary),
                );
            });
            if !plugin.capabilities.is_empty() {
                capability_chips(ui, plugin, tokens);
            }
        },
    );
    if response.clicked() {
        send(commands, AppCommand::SelectPlugin(plugin.id.clone()));
    }
}

fn selected_detail(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let Some(plugin) = plugins.selected_plugin() else {
        ui.heading(i18n.text("plugin-details"));
        ui.label(
            RichText::new(i18n.text("plugin-no-installed-selected")).color(tokens.text_secondary),
        );
        return;
    };
    plugin_detail(ui, plugin, tokens, commands, i18n);
}
