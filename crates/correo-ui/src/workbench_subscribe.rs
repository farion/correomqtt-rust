use correo_core::{AppCommand, AppCommandSender, AppSnapshot, MessageRow};
use egui::{Button, RichText, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    theme::{ThemeTokens, CONTROL_HEIGHT},
    widgets::padded_text_edit,
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
    ui.heading("Subscribe");
    ui.add_space(4.0);
    topic_row(ui, snapshot, commands);
    topic_history_buttons(
        ui,
        &snapshot.workbench.subscribe.topic_history,
        commands,
        AppCommand::UpdateSubscribeTopic,
    );
    validation_rows(ui, &snapshot.workbench.subscribe.validation, tokens);
    feedback_row(ui, snapshot.workbench.subscribe.feedback.as_ref(), tokens);
    subscriptions(ui, snapshot, commands);
}

pub(crate) fn incoming_messages(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.heading("Incoming Messages");
    ui.add_space(4.0);
    let mut filter = snapshot.workbench.subscribe.message_filter.clone();
    if ui
        .add_sized(
            [ui.available_width(), CONTROL_HEIGHT],
            padded_text_edit(TextEdit::singleline(&mut filter).hint_text("Search incoming")),
        )
        .changed()
    {
        send(commands, AppCommand::SearchMessages(filter));
    }
    ui.add_space(4.0);
    message_table(ui, snapshot, tokens, commands);
}

fn topic_row(ui: &mut Ui, snapshot: &AppSnapshot, commands: &AppCommandSender) {
    let mut topic = snapshot.workbench.subscribe.topic.clone();
    ui.horizontal(|ui| {
        let topic_response = ui.add_sized(
            [(ui.available_width() - 154.0).max(120.0), CONTROL_HEIGHT],
            padded_text_edit(TextEdit::singleline(&mut topic).hint_text("Topic filter")),
        );
        if topic_response.changed() {
            send(commands, AppCommand::UpdateSubscribeTopic(topic));
        }
        qos_selector(
            ui,
            "subscribe-qos",
            snapshot.workbench.subscribe.qos,
            |qos| {
                send(commands, AppCommand::UpdateSubscribeQos(qos));
            },
        );
        let subscribe = ui.add_enabled(
            snapshot.workbench.subscribe.valid && connected(snapshot),
            Button::new("Subscribe"),
        );
        if subscribe.clicked() {
            send(commands, AppCommand::Subscribe);
        }
        if !snapshot.workbench.subscribe.valid || !connected(snapshot) {
            subscribe.on_hover_text("Requires a connected broker and a valid topic filter.");
        }
    });
}

fn subscriptions(ui: &mut Ui, snapshot: &AppSnapshot, commands: &AppCommandSender) {
    ui.horizontal_wrapped(|ui| {
        for subscription in &snapshot.workbench.subscribe.subscriptions {
            let label = format!(
                "{} · {} · {}",
                subscription.topic_filter,
                subscription.qos.label(),
                subscription.message_count
            );
            if ui
                .add_enabled(connected(snapshot), Button::new(label))
                .on_hover_text(format!("Unsubscribe {}", subscription.topic_filter))
                .clicked()
            {
                send(
                    commands,
                    AppCommand::Unsubscribe(subscription.topic_filter.clone()),
                );
            }
        }
    });
    if active_subscription_count(snapshot) > 1
        && ui
            .add_enabled(connected(snapshot), Button::new("Unsubscribe All"))
            .clicked()
    {
        send(commands, AppCommand::UnsubscribeAll);
    }
}

fn message_table(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let filter = snapshot
        .workbench
        .subscribe
        .message_filter
        .to_ascii_lowercase();
    let max_scroll_height = (ui.available_height() - 8.0).max(96.0);
    TableBuilder::new(ui)
        .id_salt("message-table")
        .striped(true)
        .max_scroll_height(max_scroll_height)
        .column(Column::remainder())
        .column(Column::exact(66.0))
        .column(Column::exact(52.0))
        .column(Column::exact(58.0))
        .column(Column::exact(66.0))
        .header(22.0, |mut header| {
            header.col(|ui| {
                ui.strong("Topic / preview");
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
            header.col(|ui| {
                ui.strong("Badges");
            });
        })
        .body(|mut body| {
            for message in snapshot
                .workbench
                .messages
                .iter()
                .filter(|message| message_matches_filter(message, &filter))
            {
                body.row(52.0, |mut row| {
                    row.col(|ui| {
                        let selected = snapshot.workbench.selected_message_id == Some(message.id);
                        let response =
                            ui.selectable_label(selected, RichText::new(&message.topic).strong());
                        let clicked = response.clicked();
                        let double_clicked = response.double_clicked();
                        response.context_menu(|ui| {
                            if ui.button("Export .cqm").clicked() {
                                send(commands, AppCommand::ExportIncomingMessage(message.id));
                                ui.close_menu();
                            }
                        });
                        if clicked {
                            send(commands, AppCommand::SelectMessage(message.id));
                        }
                        if double_clicked {
                            send(commands, AppCommand::SelectMessage(message.id));
                            workbench_messages::open_incoming_message(ui.ctx(), message.id);
                        }
                        ui.label(
                            RichText::new(&message.payload_preview).color(tokens.text_secondary),
                        );
                    });
                    row.col(|ui| {
                        ui.label(&message.timestamp);
                    });
                    row.col(|ui| {
                        ui.label(message.qos.label());
                    });
                    row.col(|ui| {
                        ui.label(message.byte_size.to_string());
                    });
                    row.col(|ui| {
                        for badge in &message.badges {
                            ui.label(RichText::new(badge).color(tokens.accent));
                        }
                    });
                });
            }
        });
}

fn active_subscription_count(snapshot: &AppSnapshot) -> usize {
    snapshot
        .workbench
        .subscribe
        .subscriptions
        .iter()
        .filter(|subscription| subscription.active)
        .count()
}

fn message_matches_filter(message: &MessageRow, filter: &str) -> bool {
    filter.is_empty()
        || message.topic.to_ascii_lowercase().contains(filter)
        || message
            .payload_preview
            .to_ascii_lowercase()
            .contains(filter)
        || message
            .badges
            .iter()
            .any(|badge| badge.to_ascii_lowercase().contains(filter))
}
