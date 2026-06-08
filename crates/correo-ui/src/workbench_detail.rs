use correo_core::{AppCommand, AppCommandSender, AppSnapshot, MessageInspectorTab, MessageRow};
use egui::{Frame, RichText, Stroke, Ui};

use crate::theme::ThemeTokens;

pub(crate) fn inspector(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    panel(tokens).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Message Detail");
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
        match snapshot.workbench.selected_message() {
            Some(message) => {
                selected_message(ui, snapshot.workbench.inspector_tab, message, tokens)
            }
            None => {
                ui.label(RichText::new("No message selected").color(tokens.text_secondary));
            }
        }
    });
}

fn selected_message(
    ui: &mut Ui,
    tab: MessageInspectorTab,
    message: &MessageRow,
    tokens: ThemeTokens,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&message.topic).strong());
        ui.label(RichText::new(&message.timestamp).color(tokens.text_secondary));
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
        RichText::new(format!("Timestamp: {}", message.timestamp)).color(tokens.text_secondary),
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

fn panel(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
