use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ScriptDetailTab, ScriptExecutionError,
    ScriptExecutionStatus, ScriptFeedbackSeverity, ScriptFileStatus, ScriptLogLevel,
    ScriptSurfaceSnapshot,
};
use egui::{Button, Frame, RichText, ScrollArea, Stroke, TextEdit, Ui, Window};
use egui_extras::{Column, TableBuilder};

use crate::theme::ThemeTokens;

pub fn sidebar(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let mut filter = scripts.script_filter.clone();
    if ui
        .add_sized(
            [ui.available_width(), 28.0],
            TextEdit::singleline(&mut filter).hint_text("Search scripts..."),
        )
        .changed()
    {
        send(commands, AppCommand::SearchScripts(filter));
    }
    ui.add_space(8.0);
    if ui
        .add_sized([ui.available_width(), 28.0], Button::new("+ New Script"))
        .clicked()
    {
        send(commands, AppCommand::CreateScript);
    }
    ui.separator();
    script_list(ui, scripts, tokens, commands);
}

pub fn show(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, commands: &AppCommandSender) {
    ui.heading("Scripting");
    ui.add_space(8.0);
    panel(tokens).show(ui, |ui| {
        toolbar(ui, &snapshot.scripts, tokens, commands);
        ui.separator();
        script_detail(ui, &snapshot.scripts, tokens, commands);
    });
    rename_dialog(ui, &snapshot.scripts, commands);
    delete_dialog(ui, &snapshot.scripts, commands);
}

fn script_list(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filtered_scripts = scripts.filtered_scripts();
    if filtered_scripts.is_empty() {
        ui.label(RichText::new("No scripts").color(tokens.text_secondary));
        return;
    }
    for script in filtered_scripts {
        let selected = scripts.selected_script == script.name;
        let title = if script.is_dirty() {
            format!("{} *", script.name)
        } else {
            script.name.clone()
        };
        if ui
            .selectable_label(selected, RichText::new(title).strong())
            .clicked()
        {
            send(commands, AppCommand::SelectScript(script.name.clone()));
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(script.status.label())
                    .color(file_status_color(script.status, tokens)),
            );
            ui.label(
                RichText::new(format!("{} runs", script.execution_count))
                    .color(tokens.text_secondary),
            );
        });
        ui.label(
            RichText::new(&script.relative_path)
                .color(tokens.text_secondary)
                .small(),
        );
        ui.separator();
    }
}

fn toolbar(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        if ui.button("+ New Script").clicked() {
            send(commands, AppCommand::CreateScript);
        }
        ui.separator();
        ui.label(RichText::new(format!("Connection: {}", scripts.selected_connection)).strong());
        ui.separator();
        ui.label(selected_label(scripts, tokens));
        if scripts.selected_script_is_dirty() {
            ui.label(RichText::new("Unsaved").color(tokens.warning));
        }
        if ui
            .add_enabled(scripts.can_save(), Button::new("Save"))
            .on_hover_text("Save script source through the storage service")
            .clicked()
        {
            send(commands, AppCommand::SaveScript);
        }
        if ui
            .add_enabled(scripts.can_run(), Button::new("Run"))
            .on_hover_text("Queue script execution through core")
            .clicked()
        {
            send(commands, AppCommand::RunScript);
        }
        if ui
            .add_enabled(scripts.running, Button::new("Cancel"))
            .on_hover_text("Cancel the active script execution")
            .clicked()
        {
            send(commands, AppCommand::CancelScript);
        }
        if ui
            .add_enabled(scripts.selected_script().is_some(), Button::new("Rename"))
            .clicked()
        {
            send(commands, AppCommand::RequestRenameScript);
        }
        if ui
            .add_enabled(scripts.selected_script().is_some(), Button::new("Delete"))
            .clicked()
        {
            send(commands, AppCommand::RequestDeleteScript);
        }
    });
    if let Some(feedback) = &scripts.feedback {
        ui.label(RichText::new(&feedback.message).color(feedback_color(feedback.severity, tokens)));
    }
}

fn selected_label(scripts: &ScriptSurfaceSnapshot, tokens: ThemeTokens) -> RichText {
    if scripts.selected_script.is_empty() {
        RichText::new("No script selected").color(tokens.text_secondary)
    } else {
        RichText::new(format!("Script: {}", scripts.selected_script)).color(tokens.text_primary)
    }
}

fn script_detail(
    ui: &mut Ui,
    scripts: &ScriptSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    tab_bar(ui, scripts.active_tab, commands);
    ui.separator();
    match scripts.active_tab {
        ScriptDetailTab::Editor => editor(ui, scripts, commands),
        ScriptDetailTab::Executions => executions(ui, scripts, tokens),
    }
    ui.separator();
    error_summary(ui, scripts.last_error.as_ref(), tokens);
    log_view(ui, scripts, tokens);
}

fn tab_bar(ui: &mut Ui, selected: ScriptDetailTab, commands: &AppCommandSender) {
    ui.horizontal(|ui| {
        for tab in ScriptDetailTab::ALL {
            if ui.selectable_label(selected == tab, tab.label()).clicked() {
                send(commands, AppCommand::SelectScriptDetailTab(tab));
            }
        }
    });
}

fn editor(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, commands: &AppCommandSender) {
    if let Some(script) = scripts.selected_script() {
        let mut source = script.source.clone();
        let editor_height = (ui.available_height() - 180.0).max(220.0);
        if ui
            .add_sized(
                [ui.available_width(), editor_height],
                TextEdit::multiline(&mut source)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            send(commands, AppCommand::UpdateScriptSource(source));
        }
    } else {
        ui.label("Select or create a script.");
    }
}

fn executions(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, tokens: ThemeTokens) {
    ui.heading("Executions");
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::remainder())
        .column(Column::exact(92.0))
        .column(Column::exact(84.0))
        .column(Column::exact(76.0))
        .column(Column::remainder())
        .header(22.0, |mut header| {
            header.col(|ui| {
                ui.strong("Script");
            });
            header.col(|ui| {
                ui.strong("Status");
            });
            header.col(|ui| {
                ui.strong("Duration");
            });
            header.col(|ui| {
                ui.strong("Time");
            });
            header.col(|ui| {
                ui.strong("Error");
            });
        })
        .body(|mut body| {
            for execution in &scripts.executions {
                body.row(28.0, |mut row| {
                    row.col(|ui| {
                        ui.label(&execution.script_name);
                    });
                    row.col(|ui| {
                        ui.label(
                            RichText::new(execution.status.label())
                                .color(execution_color(execution.status, tokens)),
                        );
                    });
                    row.col(|ui| {
                        ui.label(&execution.duration);
                    });
                    row.col(|ui| {
                        ui.label(&execution.timestamp);
                    });
                    row.col(|ui| {
                        if let Some(error) = &execution.error {
                            ui.label(format!("{}: {}", error.kind.label(), error.message));
                        }
                    });
                });
            }
        });
}

fn log_view(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, tokens: ThemeTokens) {
    ui.label(RichText::new("Incremental log").color(tokens.text_secondary));
    ScrollArea::vertical()
        .id_salt("script-log")
        .max_height(160.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in &scripts.log_lines {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(&line.timestamp)
                            .color(tokens.text_secondary)
                            .monospace(),
                    );
                    ui.label(
                        RichText::new(line.level.label())
                            .color(log_color(line.level, tokens))
                            .monospace(),
                    );
                    ui.label(RichText::new(&line.message).monospace());
                });
            }
        });
}

fn error_summary(ui: &mut Ui, error: Option<&ScriptExecutionError>, tokens: ThemeTokens) {
    if let Some(error) = error {
        ui.label(
            RichText::new(format!("{} error: {}", error.kind.label(), error.message))
                .color(tokens.warning),
        );
    }
}

fn rename_dialog(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, commands: &AppCommandSender) {
    if !scripts.rename_dialog_open {
        return;
    }
    let mut open = true;
    Window::new("Rename Script")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            let mut name = scripts.rename_script_name.clone();
            if ui
                .add_sized([320.0, 28.0], TextEdit::singleline(&mut name))
                .changed()
            {
                send(commands, AppCommand::UpdateRenameScriptName(name));
            }
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    send(commands, AppCommand::CancelRenameScript);
                }
                if ui.button("Rename").clicked() {
                    send(commands, AppCommand::ConfirmRenameScript);
                }
            });
        });
    if !open {
        send(commands, AppCommand::CancelRenameScript);
    }
}

fn delete_dialog(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, commands: &AppCommandSender) {
    if !scripts.delete_confirmation_open {
        return;
    }
    Window::new("Delete Script")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.label(format!("Delete {}?", scripts.selected_script));
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    send(commands, AppCommand::CancelDeleteScript);
                }
                if ui.button("Delete").clicked() {
                    send(commands, AppCommand::ConfirmDeleteScript);
                }
            });
        });
}

fn panel(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

fn file_status_color(status: ScriptFileStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        ScriptFileStatus::Ready => tokens.success,
        ScriptFileStatus::Dirty => tokens.warning,
        ScriptFileStatus::Running => tokens.script,
        ScriptFileStatus::Error => tokens.danger,
    }
}

fn execution_color(status: ScriptExecutionStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        ScriptExecutionStatus::Queued | ScriptExecutionStatus::Running => tokens.script,
        ScriptExecutionStatus::Succeeded => tokens.success,
        ScriptExecutionStatus::Failed => tokens.danger,
        ScriptExecutionStatus::Cancelled => tokens.warning,
    }
}

fn log_color(level: ScriptLogLevel, tokens: ThemeTokens) -> egui::Color32 {
    match level {
        ScriptLogLevel::Debug => tokens.text_secondary,
        ScriptLogLevel::Info => tokens.text_primary,
        ScriptLogLevel::Warning => tokens.warning,
        ScriptLogLevel::Error => tokens.danger,
    }
}

fn feedback_color(severity: ScriptFeedbackSeverity, tokens: ThemeTokens) -> egui::Color32 {
    match severity {
        ScriptFeedbackSeverity::Info => tokens.success,
        ScriptFeedbackSeverity::Warning => tokens.warning,
        ScriptFeedbackSeverity::Error => tokens.danger,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
