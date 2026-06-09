use correo_core::{AppCommand, AppCommandSender, PluginMarketplaceRow, PluginSurfaceSnapshot};
use egui::{Button, RichText, ScrollArea, Ui};

use crate::theme::ThemeTokens;

use super::{install_button, marketplace_capability_chips, send};

const SPLIT_WIDTH: f32 = 760.0;

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filtered = plugins.filtered_marketplace_plugins();
    if filtered.is_empty() {
        ui.label(
            RichText::new("No marketplace plugins match this search.").color(tokens.text_secondary),
        );
        return;
    }

    if ui.available_width() < SPLIT_WIDTH {
        marketplace_list(ui, plugins, &filtered, tokens, commands);
        ui.separator();
        selected_detail(ui, plugins, tokens, commands);
    } else {
        ui.columns(2, |columns| {
            marketplace_list(&mut columns[0], plugins, &filtered, tokens, commands);
            selected_detail(&mut columns[1], plugins, tokens, commands);
        });
    }
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
    ScrollArea::vertical()
        .id_salt("plugin-marketplace-list")
        .show(ui, |ui| {
            for plugin in filtered {
                marketplace_row(ui, plugins, plugin, tokens, commands);
                ui.separator();
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
    if ui
        .selectable_label(
            plugins.selected_marketplace_plugin_id == plugin.id,
            RichText::new(&plugin.name).strong(),
        )
        .clicked()
    {
        send(
            commands,
            AppCommand::SelectMarketplacePlugin(plugin.id.clone()),
        );
    }
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
