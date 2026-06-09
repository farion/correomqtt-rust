use correo_core::{AppCommand, AppCommandSender, AppSnapshot, PublishHistoryRow};
use egui::{Button, RichText, ScrollArea, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    theme::{ThemeTokens, CONTROL_HEIGHT},
    widgets::{checkbox, padded_text_edit},
    workbench_helpers::{
        connected, feedback_row, qos_selector, send, topic_history_buttons, validation_rows,
    },
    workbench_messages,
};

pub(crate) fn editor(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ScrollArea::vertical()
        .id_salt("publish-editor")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.heading("Publish");
            ui.add_space(4.0);
            topic_row(ui, snapshot, commands);
            topic_history_buttons(
                ui,
                &snapshot.workbench.publish.topic_history,
                commands,
                AppCommand::UpdatePublishTopic,
            );
            if ui
                .button("Load .cqm...")
                .on_hover_text("Load a message file into the publish editor")
                .clicked()
            {
                send(commands, AppCommand::ImportMessages);
            }

            let mut payload = snapshot.workbench.publish.payload.clone();
            let payload_height = (ui.available_height() - 92.0).max(96.0);
            let payload_response = ui.add_sized(
                [ui.available_width(), payload_height],
                padded_text_edit(TextEdit::multiline(&mut payload))
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            );
            if payload_response.changed() {
                send(commands, AppCommand::UpdatePublishPayload(payload));
            }

            validation_rows(ui, &snapshot.workbench.publish.validation, tokens);
            feedback_row(ui, snapshot.workbench.publish.feedback.as_ref(), tokens);
            let can_publish = snapshot.workbench.publish.valid && connected(snapshot);
            let publish = ui.add_enabled(can_publish, Button::new("Publish"));
            if publish.clicked() {
                send(commands, AppCommand::Publish);
            }
            if !can_publish {
                publish.on_hover_text("Requires a connected broker and a valid topic.");
            }
        });
}

pub(crate) fn outgoing_messages(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading("Outgoing Messages");
    ui.add_space(4.0);
    let mut filter = snapshot.workbench.publish.history_filter.clone();
    if ui
        .add_sized(
            [ui.available_width(), CONTROL_HEIGHT],
            padded_text_edit(TextEdit::singleline(&mut filter).hint_text("Search outgoing")),
        )
        .changed()
    {
        send(commands, AppCommand::SearchPublishHistory(filter));
    }
    ui.add_space(4.0);
    publish_history(ui, snapshot, tokens, commands);
}

fn topic_row(ui: &mut Ui, snapshot: &AppSnapshot, commands: &AppCommandSender) {
    let mut topic = snapshot.workbench.publish.topic.clone();
    ui.horizontal(|ui| {
        let topic_response = ui.add_sized(
            [(ui.available_width() - 176.0).max(120.0), CONTROL_HEIGHT],
            padded_text_edit(TextEdit::singleline(&mut topic).hint_text("Topic")),
        );
        if topic_response.changed() {
            send(commands, AppCommand::UpdatePublishTopic(topic));
        }
        qos_selector(ui, "publish-qos", snapshot.workbench.publish.qos, |qos| {
            send(commands, AppCommand::UpdatePublishQos(qos));
        });
        let mut retained = snapshot.workbench.publish.retained;
        if checkbox(ui, &mut retained, "Retained").changed() {
            send(commands, AppCommand::SetPublishRetained(retained));
        }
    });
}

fn publish_history(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filter = snapshot
        .workbench
        .publish
        .history_filter
        .to_ascii_lowercase();
    let max_scroll_height = (ui.available_height() - 8.0).max(96.0);
    TableBuilder::new(ui)
        .id_salt("publish-history")
        .striped(true)
        .max_scroll_height(max_scroll_height)
        .column(Column::remainder())
        .column(Column::exact(72.0))
        .column(Column::exact(60.0))
        .column(Column::exact(58.0))
        .header(22.0, |mut header| {
            header.col(|ui| {
                ui.strong("Topic");
            });
            header.col(|ui| {
                ui.strong("Time");
            });
            header.col(|ui| {
                ui.strong("QoS");
            });
            header.col(|ui| {
                ui.strong("Bytes");
            });
        })
        .body(|mut body| {
            for (index, row) in snapshot
                .workbench
                .publish
                .history
                .iter()
                .enumerate()
                .filter(|(_, row)| publish_row_matches(row, &filter))
            {
                body.row(30.0, |mut row_ui| {
                    row_ui.col(|ui| {
                        let response =
                            ui.selectable_label(false, RichText::new(&row.topic).strong());
                        let double_clicked = response.double_clicked();
                        response.context_menu(|ui| {
                            if ui.button("Export .cqm").clicked() {
                                send(
                                    commands,
                                    AppCommand::ExportPublishHistoryMessage(row.topic.clone()),
                                );
                                ui.close_menu();
                            }
                        });
                        if double_clicked {
                            workbench_messages::open_outgoing_message(ui.ctx(), index);
                        }
                    });
                    row_ui.col(|ui| {
                        ui.label(RichText::new(&row.timestamp).color(tokens.text_secondary));
                    });
                    row_ui.col(|ui| {
                        ui.label(row.qos.label());
                    });
                    row_ui.col(|ui| {
                        ui.label(row.byte_size.to_string());
                    });
                });
            }
        });
}

fn publish_row_matches(row: &PublishHistoryRow, filter: &str) -> bool {
    filter.is_empty() || row.topic.to_ascii_lowercase().contains(filter)
}
