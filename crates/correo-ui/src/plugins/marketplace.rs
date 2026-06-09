use correo_core::{AppCommand, AppCommandSender, PluginMarketplaceRow, PluginSurfaceSnapshot};
use egui::{Button, CentralPanel, Frame, RichText, ScrollArea, SidePanel, Ui};

use crate::theme::ThemeTokens;

use super::{
    install_button, marketplace_capability_chips, metadata_row, plugin_tile, search_field, send,
};

const LIST_WIDTH: f32 = 340.0;
const MIN_LIST_WIDTH: f32 = 260.0;
const MAX_LIST_WIDTH: f32 = 520.0;

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filtered = plugins.filtered_marketplace_plugins();
    SidePanel::left("plugin-marketplace-list-pane")
        .default_width(LIST_WIDTH)
        .width_range(MIN_LIST_WIDTH..=MAX_LIST_WIDTH)
        .resizable(true)
        .frame(pane_frame())
        .show_inside(ui, |ui| {
            marketplace_list(ui, plugins, &filtered, tokens, commands);
        });
    CentralPanel::default()
        .frame(pane_frame())
        .show_inside(ui, |ui| selected_detail(ui, plugins, tokens, commands));
}

fn marketplace_list(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    filtered: &[&PluginMarketplaceRow],
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading("Marketplace");
    ui.add_space(4.0);
    search_field(ui, plugins, commands);
    ui.add_space(8.0);
    ScrollArea::vertical()
        .id_salt("plugin-marketplace-list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if filtered.is_empty() {
                ui.label(
                    RichText::new("No marketplace plugins match this search.")
                        .color(tokens.text_secondary),
                );
                return;
            }
            for plugin in filtered {
                marketplace_row(ui, plugins, plugin, tokens, commands);
            }
        });
}

fn marketplace_row(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    plugin: &PluginMarketplaceRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let response = plugin_tile(
        ui,
        plugins.selected_marketplace_plugin_id == plugin.id,
        tokens,
        |ui| {
            ui.label(RichText::new(&plugin.name).strong());
            ui.label(
                RichText::new(&plugin.description)
                    .color(tokens.text_secondary)
                    .small(),
            );
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&plugin.version).color(tokens.text_secondary));
                ui.label(RichText::new(&plugin.provider).color(tokens.text_secondary));
                if plugin.installed_plugin_id.is_some() {
                    ui.label(RichText::new("Installed").color(tokens.success));
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
) {
    let Some(plugin) = plugins.selected_marketplace_plugin() else {
        ui.heading("Marketplace Plugin");
        ui.label(RichText::new("No marketplace plugin selected").color(tokens.text_secondary));
        return;
    };
    ui.heading(&plugin.name);
    ui.label(format!("{} by {}", plugin.version, plugin.provider));
    ui.label(RichText::new(&plugin.repository).color(tokens.text_secondary));
    ui.add_space(8.0);
    ui.label(&plugin.description);
    metadata_row(ui, "License", &plugin.license, tokens);
    metadata_row(ui, "Location", &plugin.location, tokens);
    ui.separator();
    action_bar(ui, plugin, commands);
    ui.separator();
    marketplace_capability_chips(ui, &plugin.capabilities, tokens);
    if let Some(installed) = plugins.installed_plugin_for_marketplace(plugin) {
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("Installed as {}", installed.name)).color(tokens.text_secondary),
        );
    }
}

fn action_bar(ui: &mut Ui, plugin: &PluginMarketplaceRow, commands: &AppCommandSender) {
    ui.horizontal_wrapped(|ui| {
        if plugin.installed_plugin_id.is_some() {
            ui.add_enabled(false, Button::new("Installed"));
        } else {
            install_button(ui, &plugin.id, commands);
        }
    });
}

fn pane_frame() -> Frame {
    Frame::NONE.inner_margin(egui::Margin::same(8))
}
