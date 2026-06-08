use correo_core::{
    AppCommand, AppCommandSender, PluginHookAssignment, PluginHookEditor, PluginHookStatus,
    PluginRow, PluginSurfaceSnapshot,
};
use egui::{RichText, ScrollArea, TextEdit, Ui, Window};
use egui_extras::{Column, TableBuilder};

use crate::theme::ThemeTokens;

const COMPACT_WIDTH: f32 = 760.0;

pub(super) fn tab(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if ui.available_width() < COMPACT_WIDTH {
        compact_hooks(ui, plugins, tokens, commands);
    } else {
        ui.columns(2, |columns| {
            hook_table(&mut columns[0], plugins, tokens, commands);
            selected_plugin_detail(&mut columns[1], plugins.selected_plugin(), tokens, commands);
        });
    }

    if let Some(editor) = &plugins.hook_editor {
        hook_editor(ui, editor, tokens, commands);
    }
}

fn compact_hooks(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let Some(plugin) = plugins.selected_plugin() else {
        ui.label(RichText::new("No plugin selected").color(tokens.text_secondary));
        return;
    };

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&plugin.name).strong());
        ui.label(RichText::new(plugin.status.label()).color(tokens.text_secondary));
        if ui.button("Add hook").clicked() {
            send(
                commands,
                AppCommand::StartAddPluginHook {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
    });
    ui.separator();

    if plugin.hooks.is_empty() {
        ui.label(RichText::new("No hook assignments").color(tokens.text_secondary));
        return;
    }

    ScrollArea::vertical()
        .id_salt("plugin-hooks-compact")
        .show(ui, |ui| {
            for assignment in &plugin.hooks {
                compact_hook_row(ui, plugin, assignment, tokens, commands);
                ui.separator();
            }
        });
}

fn compact_hook_row(
    ui: &mut Ui,
    plugin: &PluginRow,
    assignment: &PluginHookAssignment,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        hook_enabled(ui, plugin, assignment, commands);
        ui.label(RichText::new(assignment.hook.label()).strong());
        ui.label(
            RichText::new(assignment.status.label())
                .color(hook_status_color(assignment.status, tokens)),
        );
        ui.label(RichText::new(&assignment.last_run).small());
        if ui.button("Edit").clicked() {
            send(
                commands,
                AppCommand::StartEditPluginHook {
                    plugin_id: plugin.id.clone(),
                    hook: assignment.hook,
                },
            );
        }
    });
    ui.label(&assignment.target);
    if !assignment.message.is_empty() {
        ui.label(RichText::new(&assignment.message).color(tokens.text_secondary));
    }
}

fn hook_table(
    ui: &mut Ui,
    plugins: &PluginSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let Some(plugin) = plugins.selected_plugin() else {
        ui.label(RichText::new("No plugin selected").color(tokens.text_secondary));
        return;
    };

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&plugin.name).strong());
        if ui.button("Add hook").clicked() {
            send(
                commands,
                AppCommand::StartAddPluginHook {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
    });
    ui.add_space(8.0);

    if plugin.hooks.is_empty() {
        ui.label(RichText::new("No hook assignments").color(tokens.text_secondary));
        return;
    }

    TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(72.0))
        .column(Column::exact(150.0))
        .column(Column::remainder())
        .column(Column::exact(92.0))
        .column(Column::exact(82.0))
        .column(Column::exact(58.0))
        .header(22.0, |mut header| {
            for title in ["Enabled", "Hook", "Target", "Status", "Last run", ""] {
                header.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|mut body| {
            for assignment in &plugin.hooks {
                body.row(46.0, |mut row| {
                    row.col(|ui| hook_enabled(ui, plugin, assignment, commands));
                    row.col(|ui| {
                        ui.label(assignment.hook.label());
                    });
                    row.col(|ui| {
                        ui.label(&assignment.target);
                        if !assignment.message.is_empty() {
                            ui.label(
                                RichText::new(&assignment.message).color(tokens.text_secondary),
                            );
                        }
                    });
                    row.col(|ui| {
                        ui.label(
                            RichText::new(assignment.status.label())
                                .color(hook_status_color(assignment.status, tokens)),
                        );
                    });
                    row.col(|ui| {
                        ui.label(&assignment.last_run);
                    });
                    row.col(|ui| {
                        if ui.button("Edit").clicked() {
                            send(
                                commands,
                                AppCommand::StartEditPluginHook {
                                    plugin_id: plugin.id.clone(),
                                    hook: assignment.hook,
                                },
                            );
                        }
                    });
                });
            }
        });
}

fn selected_plugin_detail(
    ui: &mut Ui,
    plugin: Option<&PluginRow>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let Some(plugin) = plugin else {
        ui.label(RichText::new("Select a plugin to manage hooks.").color(tokens.text_secondary));
        return;
    };

    ui.heading("Hook Assignments");
    ui.label(RichText::new(plugin.status.label()).color(tokens.text_secondary));
    ui.separator();
    ui.label(format!("{} hook assignments", plugin.hooks.len()));
    ui.label(
        RichText::new("Hooks run only while the plugin is enabled.").color(tokens.text_secondary),
    );
    ui.add_space(8.0);
    if ui.button("Add hook").clicked() {
        send(
            commands,
            AppCommand::StartAddPluginHook {
                plugin_id: plugin.id.clone(),
            },
        );
    }
}

fn hook_editor(
    ui: &mut Ui,
    editor: &PluginHookEditor,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let title = if editor.is_new() {
        format!("Add {} hook", editor.plugin_name)
    } else {
        format!("Edit {} hook", editor.plugin_name)
    };

    Window::new(title)
        .collapsible(false)
        .resizable(true)
        .show(ui.ctx(), |ui| {
            ui.label(RichText::new(editor.draft.hook.label()).strong());

            let mut enabled = editor.draft.enabled;
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                send(commands, AppCommand::SetPluginHookDraftEnabled(enabled));
            }

            ui.label("Target topic/filter");
            let mut target = editor.draft.target.clone();
            if ui
                .add(TextEdit::singleline(&mut target).desired_width(f32::INFINITY))
                .changed()
            {
                send(commands, AppCommand::UpdatePluginHookTarget(target));
            }

            ui.label("Config JSON");
            let mut config_json = editor.draft.config_json.clone();
            if ui
                .add(
                    TextEdit::multiline(&mut config_json)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY),
                )
                .changed()
            {
                send(
                    commands,
                    AppCommand::UpdatePluginHookConfigJson(config_json),
                );
            }

            if let Some(error) = &editor.error {
                ui.label(RichText::new(error).color(tokens.danger));
            }

            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    send(commands, AppCommand::ApplyPluginHookEdit);
                }
                if ui.button("Reset to saved").clicked() {
                    send(commands, AppCommand::ResetPluginHookEdit);
                }
                if ui.button("Cancel").clicked() {
                    send(commands, AppCommand::CancelPluginHookEdit);
                }
            });
        });
}

fn hook_enabled(
    ui: &mut Ui,
    plugin: &PluginRow,
    assignment: &PluginHookAssignment,
    commands: &AppCommandSender,
) {
    let mut enabled = assignment.enabled;
    if ui.checkbox(&mut enabled, "").changed() {
        send(
            commands,
            AppCommand::SetPluginHookEnabled {
                plugin_id: plugin.id.clone(),
                hook: assignment.hook,
                enabled,
            },
        );
    }
}

fn hook_status_color(status: PluginHookStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        PluginHookStatus::Ready => tokens.success,
        PluginHookStatus::Disabled => tokens.text_secondary,
        PluginHookStatus::Denied => tokens.warning,
        PluginHookStatus::Failed => tokens.danger,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
