use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, PluginDisableConfirmation, PluginFeedbackSeverity,
    PluginLoadState, PluginRow, PluginStatus, PluginSurfaceSnapshot, PluginSurfaceTab,
};
use egui::{Button, Frame, RichText, Stroke, TextEdit, Ui, Window};

use crate::theme::ThemeTokens;

#[path = "plugins/installed.rs"]
mod installed;
#[path = "plugins/keyboard.rs"]
mod keyboard;
#[path = "plugins/marketplace.rs"]
mod marketplace;

pub fn sidebar(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let mut filter = plugins.plugin_filter.clone();
    if ui
        .add_sized(
            [ui.available_width(), 28.0],
            TextEdit::singleline(&mut filter)
                .id(keyboard::plugin_search_id())
                .hint_text("Search plugins..."),
        )
        .changed()
    {
        send(commands, AppCommand::SearchPlugins(filter));
    }
    ui.add_space(8.0);
    if plugins.load_state != PluginLoadState::Ready {
        ui.label(RichText::new(plugins.load_state.message()).color(tokens.text_secondary));
        return;
    }
    ui.label(RichText::new(plugin_counts(plugins)).color(tokens.text_secondary));
    ui.separator();
    ui.label(
        RichText::new(format!(
            "{} marketplace entries",
            plugins.marketplace_plugins.len()
        ))
        .color(tokens.text_secondary),
    );
    ui.label(RichText::new(repository_count(plugins)).color(tokens.text_secondary));
}

pub fn show(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, commands: &AppCommandSender) {
    keyboard::handle(ui.ctx(), &snapshot.plugins, commands);

    ui.heading("Plugins");
    ui.add_space(8.0);
    panel(tokens).show(ui, |ui| {
        toolbar(ui, &snapshot.plugins, tokens, commands);
        ui.separator();
        if snapshot.plugins.load_state != PluginLoadState::Ready {
            empty_state(ui, &snapshot.plugins, tokens);
            return;
        }
        match snapshot.plugins.active_tab {
            PluginSurfaceTab::Installed => installed::tab(ui, &snapshot.plugins, tokens, commands),
            PluginSurfaceTab::Marketplace => {
                marketplace::tab(ui, &snapshot.plugins, tokens, commands)
            }
            PluginSurfaceTab::Configuration
            | PluginSurfaceTab::Hooks
            | PluginSurfaceTab::Diagnostics => {
                installed::tab(ui, &snapshot.plugins, tokens, commands)
            }
        }
    });
    if let Some(confirmation) = &snapshot.plugins.disable_confirmation {
        disable_confirmation(ui, confirmation, tokens, commands);
    }
}

fn toolbar(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        for tab in PluginSurfaceTab::ALL {
            if ui
                .selectable_label(plugins.active_tab == tab, tab.label())
                .clicked()
            {
                send(commands, AppCommand::SelectPluginSurfaceTab(tab));
            }
        }
        ui.separator();
        ui.label(RichText::new(plugin_counts(plugins)).color(tokens.text_secondary));
    });
    if let Some(feedback) = &plugins.feedback {
        ui.label(RichText::new(&feedback.message).color(feedback_color(feedback.severity, tokens)));
    }
}

fn empty_state(ui: &mut Ui, plugins: &PluginSurfaceSnapshot, tokens: ThemeTokens) {
    ui.add_space(24.0);
    ui.label(RichText::new(plugins.load_state.message()).color(tokens.text_secondary));
}

fn disable_confirmation(
    ui: &mut Ui,
    confirmation: &PluginDisableConfirmation,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    Window::new("Disable plugin")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.label(RichText::new(&confirmation.plugin_name).strong());
            ui.label("Disabling this plugin will turn off active hook assignments.");
            for hook in &confirmation.active_hooks {
                ui.label(RichText::new(hook.label()).color(tokens.warning));
            }
            ui.horizontal(|ui| {
                if ui.button("Disable plugin").clicked() {
                    send(commands, AppCommand::ConfirmPluginDisable);
                }
                if ui.button("Cancel").clicked() {
                    send(commands, AppCommand::CancelPluginDisable);
                }
            });
        });
}

pub(super) fn plugin_detail(
    ui: &mut Ui,
    plugin: &PluginRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading(&plugin.name);
    ui.label(format!("{} by {}", plugin.version, plugin.provider));
    ui.label(RichText::new(plugin.source.label()).color(tokens.text_secondary));
    ui.label(RichText::new(plugin.status.label()).color(status_color(plugin.status, tokens)));
    if let Some(note) = &plugin.legacy_note {
        ui.label(RichText::new(note).color(tokens.warning));
    }
    ui.separator();
    plugin_action_bar(ui, plugin, commands);
    ui.separator();
    capability_chips(ui, plugin, tokens);
    plugin_operational_summary(ui, plugin, tokens);
}

pub(super) fn capability_chips(ui: &mut Ui, plugin: &PluginRow, tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        for capability in &plugin.capabilities {
            let color = if capability.granted {
                tokens.success
            } else {
                tokens.warning
            };
            ui.label(RichText::new(&capability.label).color(color));
        }
    });
}

pub(super) fn marketplace_capability_chips(
    ui: &mut Ui,
    capabilities: &[correo_core::PluginCapabilityRow],
    tokens: ThemeTokens,
) {
    ui.horizontal_wrapped(|ui| {
        for capability in capabilities {
            let color = if capability.granted {
                tokens.success
            } else {
                tokens.warning
            };
            ui.label(RichText::new(&capability.label).color(color));
        }
    });
}

pub(super) fn plugin_action_bar(ui: &mut Ui, plugin: &PluginRow, commands: &AppCommandSender) {
    ui.horizontal_wrapped(|ui| {
        let toggle_label = if plugin.enabled { "Disable" } else { "Enable" };
        if ui.button(toggle_label).clicked() {
            send(
                commands,
                AppCommand::SetPluginEnabled {
                    plugin_id: plugin.id.clone(),
                    enabled: !plugin.enabled,
                },
            );
        }
        if ui.add(Button::new("Uninstall")).clicked() {
            send(
                commands,
                AppCommand::UninstallPlugin {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
    });
}

pub(super) fn install_button(
    ui: &mut Ui,
    marketplace_plugin_id: &str,
    commands: &AppCommandSender,
) {
    if ui.button("Install").clicked() {
        send(
            commands,
            AppCommand::InstallMarketplacePlugin {
                marketplace_plugin_id: marketplace_plugin_id.to_owned(),
            },
        );
    }
}

fn plugin_operational_summary(ui: &mut Ui, plugin: &PluginRow, tokens: ThemeTokens) {
    ui.add_space(8.0);
    ui.label(
        RichText::new(format!("{} hook assignments", plugin.hooks.len()))
            .color(tokens.text_secondary),
    );
    ui.label(
        RichText::new(format!("{} diagnostics", plugin.diagnostic_count()))
            .color(tokens.text_secondary),
    );
    if !plugin.config_fields.is_empty() {
        ui.label(
            RichText::new(format!("{} config fields", plugin.config_fields.len()))
                .color(tokens.text_secondary),
        );
    }
}

fn plugin_counts(plugins: &PluginSurfaceSnapshot) -> String {
    let enabled = plugins
        .plugins
        .iter()
        .filter(|plugin| plugin.enabled)
        .count();
    format!(
        "{} installed, {enabled} enabled, {} diagnostics",
        plugins.plugins.len(),
        plugins.diagnostics().len()
    )
}

fn repository_count(plugins: &PluginSurfaceSnapshot) -> String {
    let mut repositories = plugins
        .marketplace_plugins
        .iter()
        .map(|plugin| plugin.repository.as_str())
        .collect::<Vec<_>>();
    repositories.sort_unstable();
    repositories.dedup();
    format!("{} repositories", repositories.len())
}

fn panel(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

pub(super) fn status_color(status: PluginStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        PluginStatus::Active => tokens.success,
        PluginStatus::Disabled => tokens.text_secondary,
        PluginStatus::NeedsConfig
        | PluginStatus::CapabilityDenied
        | PluginStatus::UnsupportedLegacy => tokens.warning,
        PluginStatus::LoadError | PluginStatus::HookFailed => tokens.danger,
    }
}

fn feedback_color(severity: PluginFeedbackSeverity, tokens: ThemeTokens) -> egui::Color32 {
    match severity {
        PluginFeedbackSeverity::Info => tokens.success,
        PluginFeedbackSeverity::Warning => tokens.warning,
        PluginFeedbackSeverity::Error => tokens.danger,
    }
}

pub(super) fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
