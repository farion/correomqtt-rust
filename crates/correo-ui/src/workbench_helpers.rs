use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionState, QosLevel, WorkflowFeedback,
    WorkflowFeedbackSeverity,
};
use egui::{ComboBox, RichText, Ui};

use crate::theme::ThemeTokens;

pub(crate) fn connected(snapshot: &AppSnapshot) -> bool {
    snapshot
        .selected_connection()
        .is_some_and(|connection| connection.state == ConnectionState::Connected)
}

pub(crate) fn validation_rows(ui: &mut Ui, rows: &[String], tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        for row in rows {
            ui.label(RichText::new(row).color(tokens.text_secondary));
        }
    });
}

pub(crate) fn feedback_row(ui: &mut Ui, feedback: Option<&WorkflowFeedback>, tokens: ThemeTokens) {
    let Some(feedback) = feedback else {
        return;
    };
    let color = match feedback.severity {
        WorkflowFeedbackSeverity::Info => tokens.text_secondary,
        WorkflowFeedbackSeverity::Warning => tokens.warning,
        WorkflowFeedbackSeverity::Error => tokens.danger,
    };
    ui.label(RichText::new(&feedback.message).color(color));
}

pub(crate) fn topic_history_buttons(
    ui: &mut Ui,
    history: &[String],
    commands: &AppCommandSender,
    command: impl Fn(String) -> AppCommand,
) {
    if history.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for topic in history.iter().take(6) {
            if ui.small_button(topic).clicked() {
                send(commands, command(topic.clone()));
            }
        }
    });
}

pub(crate) fn qos_selector(
    ui: &mut Ui,
    id: &'static str,
    current: QosLevel,
    mut on_change: impl FnMut(QosLevel),
) {
    let mut selected = current;
    ComboBox::from_id_salt(id)
        .selected_text(current.label())
        .width(78.0)
        .show_ui(ui, |ui| {
            for qos in QosLevel::ALL {
                ui.selectable_value(&mut selected, qos, qos.label());
            }
        });
    if selected != current {
        on_change(selected);
    }
}

pub(crate) fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
