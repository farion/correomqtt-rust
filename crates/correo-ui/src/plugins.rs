use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, PluginDisableConfirmation, PluginFeedbackSeverity,
    PluginLoadState, PluginRow, PluginStatus, PluginSurfaceSnapshot, PluginSurfaceTab,
};
use egui::{Frame, RichText, ScrollArea, Stroke, TextEdit, Ui, Window};

use crate::theme::ThemeTokens;

#[path = "plugins/diagnostics.rs"]
mod diagnostics;
#[path = "plugins/hooks.rs"]
mod hooks;
#[path = "plugins/installed.rs"]
mod installed;
#[path = "plugins/keyboard.rs"]
mod keyboard;

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
    ScrollArea::vertical()
        .id_salt("plugin-sidebar")
        .show(ui, |ui| {
            let filtered = plugins.filtered_plugins();
            if filtered.is_empty() {
                ui.label(
                    RichText::new("No plugins match this search.").color(tokens.text_secondary),
                );
                return;
            }
            for plugin in filtered {
                let selected = plugins.selected_plugin_id == plugin.id;
                if ui
                    .selectable_label(selected, RichText::new(&plugin.name).strong())
                    .clicked()
                {
                    send(commands, AppCommand::SelectPlugin(plugin.id.clone()));
                }
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(plugin.status.label())
                            .color(status_color(plugin.status, tokens)),
                    );
                    ui.label(RichText::new(plugin.source.label()).color(tokens.text_secondary));
                });
                ui.label(
                    RichText::new(format!("{} diagnostics", plugin.diagnostic_count()))
                        .color(tokens.text_secondary)
                        .small(),
                );
                ui.separator();
            }
        });
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
            PluginSurfaceTab::Configuration => config_tab(ui, &snapshot.plugins, tokens, commands),
            PluginSurfaceTab::Hooks => hooks::tab(ui, &snapshot.plugins, tokens, commands),
            PluginSurfaceTab::Diagnostics => {
                diagnostics::tab(ui, &snapshot.plugins, tokens, commands)
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

fn config_tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if ui.available_width() < 720.0 {
        selected_summary(ui, plugins, tokens, commands);
        ui.separator();
        config_editor(ui, plugins, tokens, commands);
    } else {
        ui.columns(2, |columns| {
            selected_summary(&mut columns[0], plugins, tokens, commands);
            config_editor(&mut columns[1], plugins, tokens, commands);
        });
    }
}

fn config_editor(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading("Config Schema");
    let Some(plugin) = plugins.selected_plugin() else {
        ui.label("No plugin selected");
        return;
    };
    if plugin.config_fields.is_empty() {
        ui.label("No editable config schema");
        return;
    }
    ui.horizontal_wrapped(|ui| {
        if ui.button("Apply").clicked() {
            send(
                commands,
                AppCommand::ApplyPluginConfig {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
        if ui.button("Cancel").clicked() {
            send(
                commands,
                AppCommand::CancelPluginConfig {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
        if ui.button("Reset to saved").clicked() {
            send(
                commands,
                AppCommand::ResetPluginConfig {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
    });
    ui.add_space(8.0);
    for field in &plugin.config_fields {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&field.label).strong());
            if field.required {
                ui.label(RichText::new("required").color(tokens.warning));
            }
            if field.sensitive {
                ui.label(RichText::new("keyring").color(tokens.text_secondary));
            }
        });
        let mut value = field.value.clone();
        if ui
            .add_enabled(
                !field.sensitive,
                TextEdit::singleline(&mut value).desired_width(f32::INFINITY),
            )
            .changed()
        {
            send(
                commands,
                AppCommand::UpdatePluginConfigValue {
                    plugin_id: plugin.id.clone(),
                    key: field.key.clone(),
                    value,
                },
            );
        }
        ui.label(RichText::new(&field.schema_hint).color(tokens.text_secondary));
        if let Some(error) = &field.error {
            ui.label(RichText::new(error).color(tokens.danger));
        }
        ui.add_space(8.0);
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

fn selected_summary(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let Some(plugin) = plugins.selected_plugin() else {
        ui.label("No plugin selected");
        return;
    };
    ui.heading(&plugin.name);
    enabled_checkbox(ui, plugin, commands);
    ui.label(format!("{} by {}", plugin.version, plugin.provider));
    ui.label(RichText::new(plugin.source.label()).color(tokens.text_secondary));
    ui.label(RichText::new(plugin.status.label()).color(status_color(plugin.status, tokens)));
    if let Some(note) = &plugin.legacy_note {
        ui.label(RichText::new(note).color(tokens.warning));
    }
    ui.separator();
    capability_chips(ui, plugin, tokens);
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

pub(super) fn enabled_checkbox(ui: &mut Ui, plugin: &PluginRow, commands: &AppCommandSender) {
    let mut enabled = plugin.enabled;
    if ui
        .checkbox(&mut enabled, "")
        .on_hover_text("Enable plugin")
        .changed()
    {
        send(
            commands,
            AppCommand::SetPluginEnabled {
                plugin_id: plugin.id.clone(),
                enabled,
            },
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
