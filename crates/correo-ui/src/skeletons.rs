use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionExportSnapshot, ConnectionImportSnapshot,
    ExportPathState, ImportPasswordState, MessageTransferSnapshot, TransferConnectionRow,
    TransferConnectionStatus, TransferFeedback, TransferOutcome, TransferSection, TransferSeverity,
    TransferStep, TransferSurfaceSnapshot,
};
use egui::{Frame, Grid, RichText, ScrollArea, Stroke, TextEdit, Ui};

use crate::theme::ThemeTokens;

pub fn import_export(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal(|ui| {
        ui.heading("Import/Export");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Export messages...").clicked() {
                send(commands, AppCommand::ExportMessages);
            }
            if ui.button("Import messages...").clicked() {
                send(commands, AppCommand::ImportMessages);
            }
        });
    });
    ui.separator();
    section_tabs(ui, snapshot.transfer.active_section, tokens, commands);
    ui.add_space(8.0);

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            match snapshot.transfer.active_section {
                TransferSection::Import => import_panel(ui, &snapshot.transfer, tokens, commands),
                TransferSection::Export => {
                    export_panel(ui, &snapshot.transfer.export, tokens, commands)
                }
                TransferSection::Messages => {
                    message_panel(ui, &snapshot.transfer.messages, tokens, commands)
                }
            }
            if !snapshot.transfer.warnings.is_empty() {
                ui.add_space(10.0);
                warnings(ui, "Transfer warnings", &snapshot.transfer.warnings, tokens);
            }
        });
}

fn section_tabs(
    ui: &mut Ui,
    active: TransferSection,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        for section in TransferSection::ALL {
            let selected = active == section;
            let label = if selected {
                RichText::new(section.label()).strong().color(tokens.accent)
            } else {
                RichText::new(section.label()).color(tokens.text_secondary)
            };
            if ui.selectable_label(selected, label).clicked() {
                send(commands, AppCommand::SelectTransferSection(section));
            }
        }
    });
}

fn import_panel(
    ui: &mut Ui,
    transfer: &TransferSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    panel(tokens).show(ui, |ui| {
        section_header(ui, "Import .cqc", "Connection profiles", tokens);
        stepper(ui, transfer, tokens, commands);
        ui.separator();
        match transfer.active_step {
            TransferStep::ChooseFile => import_choose_file(ui, &transfer.import, tokens, commands),
            TransferStep::Password => import_password(ui, &transfer.import, tokens, commands),
            TransferStep::Review => import_review(ui, &transfer.import, tokens, commands),
            TransferStep::Complete => import_complete(ui, &transfer.import, tokens),
        }
    });
}

fn import_choose_file(
    ui: &mut Ui,
    import: &ConnectionImportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if let Some(file) = &import.file {
        file_summary(ui, file, tokens);
    } else {
        ui.label(RichText::new("No .cqc file selected").color(tokens.text_secondary));
        ui.label("Choose a legacy CorreoMQTT connection export to inspect before import.");
    }
    if ui.button("Choose .cqc file...").clicked() {
        send(commands, AppCommand::ChooseConnectionImportFile);
    }
    feedback(ui, import.feedback.as_ref(), tokens);
}

fn import_password(
    ui: &mut Ui,
    import: &ConnectionImportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if let Some(file) = &import.file {
        file_summary(ui, file, tokens);
    }
    ui.label(format!("Password state: {}", import.password_state.label()));
    if import.password_required() {
        let mut password = String::new();
        ui.horizontal(|ui| {
            ui.add_sized(
                [220.0, 24.0],
                TextEdit::singleline(&mut password)
                    .password(true)
                    .hint_text("Import password"),
            );
            if ui.button("Unlock file").clicked() {
                send(commands, AppCommand::SubmitConnectionImportPassword);
            }
        });
    }
    if import.password_state == ImportPasswordState::InvalidRecoverable {
        ui.label(
            RichText::new("Wrong password; file selection is kept so you can retry.")
                .color(tokens.danger),
        );
        if ui.button("Retry password").clicked() {
            send(commands, AppCommand::ClearConnectionImportError);
        }
    }
    feedback(ui, import.feedback.as_ref(), tokens);
}

fn import_review(
    ui: &mut Ui,
    import: &ConnectionImportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    if let Some(file) = &import.file {
        file_summary(ui, file, tokens);
    }
    ui.horizontal(|ui| {
        ui.label(format!("{} selected", import.selected_count()));
        ui.separator();
        ui.label(RichText::new(import.password_state.label()).color(tokens.text_secondary));
        ui.separator();
        ui.label(if import.encrypted {
            "Encrypted import"
        } else {
            "Plain import"
        });
    });
    connection_rows(
        ui,
        "import-rows",
        &import.rows,
        tokens,
        |row_id, selected| AppCommand::SelectConnectionImportRow { row_id, selected },
        commands,
    );
    warnings(ui, "Import warnings", &import.warnings, tokens);
    feedback(ui, import.feedback.as_ref(), tokens);
    if ui.button("Import selected").clicked() {
        send(commands, AppCommand::StartConnectionImport);
    }
}

fn import_complete(ui: &mut Ui, import: &ConnectionImportSnapshot, tokens: ThemeTokens) {
    outcome(ui, import.outcome.as_ref(), tokens);
    warnings(ui, "Import warnings", &import.warnings, tokens);
}

fn export_panel(
    ui: &mut Ui,
    export: &ConnectionExportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    panel(tokens).show(ui, |ui| {
        section_header(ui, "Export .cqc", "Connection profiles", tokens);
        connection_rows(
            ui,
            "export-rows",
            &export.rows,
            tokens,
            |row_id, selected| AppCommand::SelectConnectionExportRow { row_id, selected },
            commands,
        );
        ui.separator();
        export_options(ui, export, tokens, commands);
        feedback(ui, export.feedback.as_ref(), tokens);
        outcome(ui, export.outcome.as_ref(), tokens);
        if ui.button("Export connections").clicked() {
            send(commands, AppCommand::StartConnectionExport);
        }
    });
}

fn export_options(
    ui: &mut Ui,
    export: &ConnectionExportSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal(|ui| {
        ui.label(format!("{} selected", export.selected_count()));
        ui.separator();
        let mut encrypted = export.encrypted;
        if ui.radio_value(&mut encrypted, false, "Plain").changed() {
            send(commands, AppCommand::SetConnectionExportEncrypted(false));
        }
        if ui.radio_value(&mut encrypted, true, "Encrypted").changed() {
            send(commands, AppCommand::SetConnectionExportEncrypted(true));
        }
    });
    if !export.encrypted {
        ui.label(
            RichText::new("Plain export excludes sensitive auth values.").color(tokens.warning),
        );
    }
    if export.password_required() {
        export_password_fields(ui);
        ui.label(RichText::new(export.password_confirmation.label()).color(tokens.text_secondary));
    }
    ui.horizontal(|ui| {
        ui.label("Target");
        let mut path = export.output_path.clone();
        let response = ui.add_sized([360.0, 24.0], TextEdit::singleline(&mut path));
        if response.changed() {
            send(commands, AppCommand::UpdateConnectionExportPath(path));
        }
        ui.label(path_state_label(export.path_state, tokens));
    });
}

fn export_password_fields(ui: &mut Ui) {
    let mut password = String::new();
    let mut confirm = String::new();
    ui.horizontal(|ui| {
        ui.add_sized(
            [180.0, 24.0],
            TextEdit::singleline(&mut password)
                .password(true)
                .hint_text("Export password"),
        );
        ui.add_sized(
            [180.0, 24.0],
            TextEdit::singleline(&mut confirm)
                .password(true)
                .hint_text("Confirm password"),
        );
    });
}

fn message_panel(
    ui: &mut Ui,
    messages: &MessageTransferSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    panel(tokens).show(ui, |ui| {
        section_header(ui, "Messages", "Import/export history archives", tokens);
        if let Some(file) = &messages.import_file {
            file_summary(ui, file, tokens);
        }
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} of {} messages selected",
                messages.selected_messages, messages.available_messages
            ));
            ui.separator();
            ui.label(format!("Export target: {}", messages.export_path));
        });
        ui.horizontal(|ui| {
            if ui.button("Import message archive...").clicked() {
                send(commands, AppCommand::ImportMessages);
            }
            if ui.button("Export selected messages...").clicked() {
                send(commands, AppCommand::ExportMessages);
            }
        });
        feedback(ui, messages.feedback.as_ref(), tokens);
        outcome(ui, messages.outcome.as_ref(), tokens);
    });
}

fn stepper(
    ui: &mut Ui,
    transfer: &TransferSurfaceSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        for step in TransferStep::ALL {
            let selected = transfer.active_step == step;
            let label = if selected {
                RichText::new(step.label()).strong().color(tokens.accent)
            } else {
                RichText::new(step.label()).color(tokens.text_secondary)
            };
            if ui.button(label).clicked() {
                send(commands, AppCommand::SelectTransferStep(step));
            }
        }
    });
}

fn connection_rows(
    ui: &mut Ui,
    id: &str,
    rows: &[TransferConnectionRow],
    tokens: ThemeTokens,
    command: impl Fn(String, bool) -> AppCommand,
    commands: &AppCommandSender,
) {
    Grid::new(id)
        .striped(true)
        .min_col_width(72.0)
        .show(ui, |ui| {
            ui.label(RichText::new("Use").strong());
            ui.label(RichText::new("Connection").strong());
            ui.label(RichText::new("Endpoint").strong());
            ui.label(RichText::new("Version").strong());
            ui.label(RichText::new("State").strong());
            ui.end_row();
            for row in rows {
                let mut selected = row.selected;
                if ui.checkbox(&mut selected, "").changed() {
                    send(commands, command(row.id.clone(), selected));
                }
                ui.label(&row.name);
                ui.label(RichText::new(&row.endpoint).color(tokens.text_secondary));
                ui.label(&row.mqtt_version);
                ui.label(RichText::new(row.status.label()).color(status_color(row.status, tokens)));
                ui.end_row();
            }
        });
}

fn file_summary(ui: &mut Ui, file: &correo_core::TransferFileSnapshot, tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&file.display_name).strong());
        ui.separator();
        ui.label(RichText::new(&file.path_hint).color(tokens.text_secondary));
        ui.separator();
        ui.label(format!("{} bytes", file.byte_size));
        if file.detected_connections > 0 {
            ui.separator();
            ui.label(format!("{} profiles", file.detected_connections));
        }
        ui.separator();
        ui.label(if file.encrypted { "Encrypted" } else { "Plain" });
    });
}

fn warnings(ui: &mut Ui, title: &str, warnings: &[String], tokens: ThemeTokens) {
    if warnings.is_empty() {
        return;
    }
    ui.label(RichText::new(title).strong());
    for warning in warnings {
        ui.label(RichText::new(warning).color(tokens.warning));
    }
}

fn feedback(ui: &mut Ui, feedback: Option<&TransferFeedback>, tokens: ThemeTokens) {
    if let Some(feedback) = feedback {
        ui.label(RichText::new(&feedback.message).color(severity_color(feedback.severity, tokens)));
    }
}

fn outcome(ui: &mut Ui, outcome: Option<&TransferOutcome>, tokens: ThemeTokens) {
    if let Some(outcome) = outcome {
        let color = if outcome.success {
            tokens.success
        } else {
            tokens.danger
        };
        ui.label(RichText::new(&outcome.title).strong().color(color));
        ui.label(RichText::new(&outcome.detail).color(tokens.text_secondary));
    }
}

fn section_header(ui: &mut Ui, title: &str, subtitle: &str, tokens: ThemeTokens) {
    ui.horizontal(|ui| {
        ui.heading(title);
        ui.label(RichText::new(subtitle).color(tokens.text_secondary));
    });
}

fn path_state_label(state: ExportPathState, tokens: ThemeTokens) -> RichText {
    let color = state
        .severity()
        .map(|severity| severity_color(severity, tokens))
        .unwrap_or(tokens.success);
    RichText::new(state.label()).color(color)
}

fn status_color(status: TransferConnectionStatus, tokens: ThemeTokens) -> egui::Color32 {
    match status {
        TransferConnectionStatus::New | TransferConnectionStatus::Exportable => tokens.success,
        TransferConnectionStatus::Update => tokens.accent,
        TransferConnectionStatus::Conflict | TransferConnectionStatus::MissingSecret => {
            tokens.warning
        }
    }
}

fn severity_color(severity: TransferSeverity, tokens: ThemeTokens) -> egui::Color32 {
    match severity {
        TransferSeverity::Info => tokens.accent,
        TransferSeverity::Warning => tokens.warning,
        TransferSeverity::Error => tokens.danger,
    }
}

fn panel(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
