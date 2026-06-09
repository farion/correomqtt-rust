use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionState, MessageRow, QosLevel, WorkbenchTab,
    WorkflowFeedback, WorkflowFeedbackSeverity,
};
use egui::{Button, ComboBox, RichText, ScrollArea, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    theme::{ThemeTokens, CONTROL_HEIGHT},
    widgets::{checkbox, padded_text_edit},
    workbench_detail, workbench_dialogs, workbench_header,
    workbench_helpers::{panel, send},
};

pub fn show(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, commands: &AppCommandSender) {
    let Some(connection) = snapshot.selected_connection() else {
        ui.label("No connection available");
        return;
    };

    workbench_header::connection_header(ui, snapshot, tokens, commands);
    ui.add_space(8.0);

    workbench_body(ui, snapshot, tokens, commands);
    ui.add_space(8.0);
    workbench_detail::inspector(ui, snapshot, tokens, commands);

    if connection.state != ConnectionState::Connected {
        ui.label(
            RichText::new(
                "Publish and subscribe commands are disabled until the connection returns.",
            )
            .color(tokens.warning),
        );
    }

    workbench_dialogs::unsubscribe_all_confirmation(ui, snapshot, commands);
}

fn workbench_body(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let max_height = workbench_body_height(ui.available_height());
    ScrollArea::vertical()
        .id_salt("workbench-body")
        .max_height(max_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            if ui.available_width() < 760.0 {
                narrow_workbench(ui, snapshot, tokens, commands);
            } else {
                ui.columns(2, |columns| {
                    publish_pane(&mut columns[0], snapshot, tokens, commands);
                    subscribe_pane(&mut columns[1], snapshot, tokens, commands);
                });
            }
        });
}

fn workbench_body_height(available_height: f32) -> f32 {
    let reserved_inspector = if available_height < 500.0 {
        136.0
    } else {
        164.0
    };
    (available_height - reserved_inspector - 8.0).max(240.0)
}

fn narrow_workbench(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.horizontal(|ui| {
        for tab in [WorkbenchTab::Publish, WorkbenchTab::Subscribe] {
            if ui
                .selectable_label(snapshot.workbench.narrow_tab == tab, tab.label())
                .clicked()
            {
                send(commands, AppCommand::SelectWorkbenchTab(tab));
            }
        }
    });
    ui.separator();
    match snapshot.workbench.narrow_tab {
        WorkbenchTab::Publish => publish_pane(ui, snapshot, tokens, commands),
        WorkbenchTab::Subscribe => subscribe_pane(ui, snapshot, tokens, commands),
    }
}

fn publish_pane(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    panel(tokens).show(ui, |ui| {
        ui.heading("Publish");
        ui.separator();
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
        let payload_response = ui.add(
            padded_text_edit(TextEdit::multiline(&mut payload))
                .font(egui::TextStyle::Monospace)
                .desired_rows(8)
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

        ui.separator();
        let mut filter = snapshot.workbench.publish.history_filter.clone();
        if ui
            .add(padded_text_edit(
                TextEdit::singleline(&mut filter).hint_text("Search publish history"),
            ))
            .changed()
        {
            send(commands, AppCommand::SearchPublishHistory(filter));
        }
        publish_history(ui, snapshot, tokens, commands);
    });
}

fn subscribe_pane(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    panel(tokens).show(ui, |ui| {
        ui.heading("Subscribe");
        ui.separator();
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
        topic_history_buttons(
            ui,
            &snapshot.workbench.subscribe.topic_history,
            commands,
            AppCommand::UpdateSubscribeTopic,
        );
        validation_rows(ui, &snapshot.workbench.subscribe.validation, tokens);
        feedback_row(ui, snapshot.workbench.subscribe.feedback.as_ref(), tokens);

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
        if active_subscription_count(snapshot) > 1 {
            if ui
                .add_enabled(connected(snapshot), Button::new("Unsubscribe All"))
                .clicked()
            {
                send(commands, AppCommand::UnsubscribeAll);
            }
        }

        ui.separator();
        let mut filter = snapshot.workbench.subscribe.message_filter.clone();
        if ui
            .add(padded_text_edit(
                TextEdit::singleline(&mut filter).hint_text("Search messages"),
            ))
            .changed()
        {
            send(commands, AppCommand::SearchMessages(filter));
        }
        message_table(ui, snapshot, tokens, commands);
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
    let max_scroll_height = table_scroll_height(ui, 220.0);
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
            for row in
                snapshot.workbench.publish.history.iter().filter(|row| {
                    filter.is_empty() || row.topic.to_ascii_lowercase().contains(&filter)
                })
            {
                body.row(28.0, |mut row_ui| {
                    row_ui.col(|ui| {
                        ui.label(&row.topic).context_menu(|ui| {
                            if ui.button("Export .cqm").clicked() {
                                send(
                                    commands,
                                    AppCommand::ExportPublishHistoryMessage(row.topic.clone()),
                                );
                                ui.close_menu();
                            }
                        });
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
    let max_scroll_height = table_scroll_height(ui, 240.0);
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
                        response.context_menu(|ui| {
                            if ui.button("Export .cqm").clicked() {
                                send(commands, AppCommand::ExportIncomingMessage(message.id));
                                ui.close_menu();
                            }
                        });
                        if clicked {
                            send(commands, AppCommand::SelectMessage(message.id));
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

fn table_scroll_height(ui: &Ui, preferred: f32) -> f32 {
    (ui.available_height() - 52.0).clamp(128.0, preferred)
}

fn validation_rows(ui: &mut Ui, rows: &[String], tokens: ThemeTokens) {
    ui.horizontal_wrapped(|ui| {
        for row in rows {
            ui.label(RichText::new(row).color(tokens.text_secondary));
        }
    });
}

fn feedback_row(ui: &mut Ui, feedback: Option<&WorkflowFeedback>, tokens: ThemeTokens) {
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

fn topic_history_buttons(
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

fn connected(snapshot: &AppSnapshot) -> bool {
    snapshot
        .selected_connection()
        .is_some_and(|connection| connection.state == ConnectionState::Connected)
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

fn qos_selector(
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
