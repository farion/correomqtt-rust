use correo_core::{AppCommand, AppCommandSender, PluginMarketplaceRow, PluginSurfaceSnapshot};
use egui::{Button, RichText, ScrollArea, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;
use crate::widgets::{tile_list_content_width, tile_scroll_bar_rect};

use super::{
    install_button, marketplace_capability_chips, metadata_row, plugin_split, plugin_tile,
    search_field, send,
};

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let filtered = plugins.filtered_marketplace_plugins();
    plugin_split(
        ui,
        tokens,
        |ui| {
            marketplace_list(ui, plugins, &filtered, tokens, commands, i18n);
        },
        |ui| selected_detail(ui, plugins, tokens, commands, i18n),
    );
}

fn marketplace_list(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    filtered: &[&PluginMarketplaceRow],
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.heading(i18n.text("plugin-tab-marketplace"));
    ui.add_space(4.0);
    search_field(ui, plugins, commands, i18n);
    ui.add_space(8.0);
    ScrollArea::vertical()
        .id_salt("plugin-marketplace-list")
        .auto_shrink([false, false])
        .scroll_bar_rect(tile_scroll_bar_rect(ui))
        .show(ui, |ui| {
            ui.set_width(tile_list_content_width(ui));
            if filtered.is_empty() {
                ui.label(
                    RichText::new(i18n.text("plugin-no-marketplace-match"))
                        .color(tokens.text_secondary),
                );
                return;
            }
            for plugin in filtered {
                marketplace_row(ui, plugins, plugin, tokens, commands, i18n);
            }
        });
}

fn marketplace_row(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    plugin: &PluginMarketplaceRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let response = plugin_tile(
        ui,
        plugins.selected_marketplace_plugin_id == plugin.id,
        tokens,
        |ui| {
            ui.label(RichText::new(&plugin.name).strong());
            ui.label(RichText::new(&plugin.description).color(tokens.text_secondary));
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&plugin.version).color(tokens.text_secondary));
                ui.label(RichText::new(&plugin.provider).color(tokens.text_secondary));
                if plugin.installed_plugin_id.is_some() {
                    ui.label(RichText::new(i18n.text("plugin-installed")).color(tokens.success));
                }
            });
            if !plugin.capabilities.is_empty() {
                marketplace_capability_chips(ui, &plugin.capabilities, tokens);
            }
        },
    );
    if response.clicked() {
        send(
            commands,
            AppCommand::SelectMarketplacePlugin(plugin.id.clone()),
        );
    }
}

fn selected_detail(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let Some(plugin) = plugins.selected_marketplace_plugin() else {
        ui.heading(i18n.text("plugin-marketplace-detail"));
        ui.label(
            RichText::new(i18n.text("plugin-no-marketplace-selected")).color(tokens.text_secondary),
        );
        return;
    };
    ui.heading(&plugin.name);
    ui.label(format!(
        "{} {} {}",
        plugin.version,
        i18n.text("plugin-by"),
        plugin.provider
    ));
    ui.label(RichText::new(&plugin.repository).color(tokens.text_secondary));
    ui.add_space(8.0);
    ui.label(&plugin.description);
    metadata_row(
        ui,
        &i18n.text("plugin-license"),
        &plugin.license,
        tokens,
        i18n,
    );
    metadata_row(
        ui,
        &i18n.text("plugin-path"),
        &plugin.location,
        tokens,
        i18n,
    );
    ui.add_space(8.0);
    action_bar(ui, plugin, commands, i18n);
    ui.add_space(8.0);
    marketplace_capability_chips(ui, &plugin.capabilities, tokens);
    if let Some(installed) = plugins.installed_plugin_for_marketplace(plugin) {
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "{} {}",
                i18n.text("plugin-installed-as"),
                installed.name
            ))
            .color(tokens.text_secondary),
        );
    }
}

fn action_bar(
    ui: &mut Ui,
    plugin: &PluginMarketplaceRow,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal_wrapped(|ui| {
        if plugin.installed_plugin_id.is_some() {
            ui.add_enabled(false, Button::new(i18n.text("plugin-installed")));
        } else {
            install_button(ui, &plugin.id, commands, i18n);
        }
    });
}
