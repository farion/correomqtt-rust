use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, MessageInspectorTab, MessageRow, PublishHistoryRow,
};
use egui::{RichText, Ui};

use crate::theme::ThemeTokens;

pub(crate) fn message_window_content(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    message: &MessageRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| {
        for tab in MessageInspectorTab::ALL {
            if ui
                .selectable_label(snapshot.workbench.inspector_tab == tab, tab.label())
                .clicked()
            {
                send(commands, AppCommand::SelectInspectorTab(tab));
            }
        }
    });
    ui.separator();
    selected_message(ui, snapshot.workbench.inspector_tab, message, tokens);
}

pub(crate) fn outgoing_window_content(ui: &mut Ui, row: &PublishHistoryRow, tokens: ThemeTokens) {
    let timestamp = crate::time_format::local_date_time(&row.timestamp);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&row.topic).strong());
        ui.label(RichText::new(&timestamp).color(tokens.text_secondary));
        ui.label(row.qos.label());
        if row.retained {
            badge_label(ui, "retained", tokens);
        }
    });
    ui.add_space(6.0);
    ui.label(RichText::new(&row.payload_preview).monospace());
    ui.label(format!("Bytes: {}", row.byte_size));
    ui.label(RichText::new("Published message history").color(tokens.text_secondary));
}

fn selected_message(
    ui: &mut Ui,
    tab: MessageInspectorTab,
    message: &MessageRow,
    tokens: ThemeTokens,
) {
    let timestamp = crate::time_format::local_date_time(&message.timestamp);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&message.topic).strong());
        ui.label(RichText::new(&timestamp).color(tokens.text_secondary));
        ui.label(message.qos.label());
        if message.retained {
            badge_label(ui, "retained", tokens);
        }
    });
    ui.add_space(4.0);
    match tab {
        MessageInspectorTab::Payload => payload_tab(ui, message, tokens),
        MessageInspectorTab::Properties => properties_tab(ui, message, tokens),
        MessageInspectorTab::Formatted => formatted_tab(ui, message, tokens),
        MessageInspectorTab::Diagnostics => diagnostics_tab(ui, message, tokens),
    }
}

fn payload_tab(ui: &mut Ui, message: &MessageRow, tokens: ThemeTokens) {
    ui.label(RichText::new(&message.payload_preview).monospace());
    ui.label(RichText::new(format!("{} bytes", message.byte_size)).color(tokens.text_secondary));
}

fn properties_tab(ui: &mut Ui, message: &MessageRow, tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("Topic: {}", message.topic));
        ui.label(format!("QoS: {}", message.qos.label()));
        ui.label(format!("Bytes: {}", message.byte_size));
        ui.label(format!(
            "Retained: {}",
            if message.retained { "yes" } else { "no" }
        ));
    });
    ui.label(
        RichText::new(format!(
            "Timestamp: {}",
            crate::time_format::local_date_time(&message.timestamp)
        ))
        .color(tokens.text_secondary),
    );
}

fn formatted_tab(ui: &mut Ui, message: &MessageRow, tokens: ThemeTokens) {
    match &message.formatted_detail {
        Some(detail) => {
            ui.label(RichText::new(&detail.text).monospace());
            ui.label(
                RichText::new(format!("{} formatter output", detail.format.label()))
                    .color(tokens.text_secondary),
            );
        }
        None => {
            ui.label(RichText::new(&message.payload_preview).monospace());
        }
    }
}

fn diagnostics_tab(ui: &mut Ui, message: &MessageRow, tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        for badge in &message.badges {
            badge_label(ui, badge, tokens);
        }
    });
    for diagnostic in &message.diagnostics {
        ui.label(RichText::new(format!(
            "{}: {}",
            diagnostic.severity.label(),
            diagnostic.message
        )));
    }
    if message.badges.is_empty() && message.diagnostics.is_empty() {
        ui.label(RichText::new("No message diagnostics").color(tokens.text_secondary));
    }
}

fn badge_label(ui: &mut Ui, label: &str, tokens: ThemeTokens) {
    ui.label(RichText::new(label).color(tokens.accent).strong());
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
