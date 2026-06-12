use correo_core::{AppCommand, AppCommandSender, AppSnapshot, MessageRow, PublishHistoryRow};
use correo_style::layout;
use egui::{Button, Label, Layout, RichText, TextEdit, Ui};
use egui_phosphor::regular;

use crate::{
    payload_highlight,
    theme::ThemeTokens,
    widgets::{padded_text_edit, square_icon_button_size, with_icon_button_padding},
    workbench_connection_messages_text::formatted_size,
};

const DETAIL_TOOLBAR_HEIGHT: f32 = 48.0;

pub(crate) fn message_window_content(
    ui: &mut Ui,
    _snapshot: &AppSnapshot,
    message: &MessageRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    detail_view(
        ui,
        MessageDetail {
            topic: &message.topic,
            timestamp: &message.timestamp,
            qos: message.qos.label(),
            retained: message.retained,
            byte_size: message.byte_size,
            payload: &message.payload,
            fallback_payload: &message.payload_preview,
            export: AppCommand::ExportIncomingMessage(message.id),
        },
        tokens,
        commands,
    );
}

pub(crate) fn outgoing_window_content(
    ui: &mut Ui,
    row: &PublishHistoryRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    detail_view(
        ui,
        MessageDetail {
            topic: &row.topic,
            timestamp: &row.timestamp,
            qos: row.qos.label(),
            retained: row.retained,
            byte_size: row.byte_size,
            payload: &row.payload,
            fallback_payload: &row.payload_preview,
            export: AppCommand::ExportPublishHistoryMessage(row.id),
        },
        tokens,
        commands,
    );
}

fn detail_view(
    ui: &mut Ui,
    detail: MessageDetail<'_>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    detail_toolbar(ui, &detail, tokens, commands);
    ui.add_space(6.0);
    payload_area(ui, &detail);
}

fn detail_toolbar(
    ui: &mut Ui,
    detail: &MessageDetail<'_>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), DETAIL_TOOLBAR_HEIGHT),
        Layout::left_to_right(egui::Align::Center),
        |ui| {
            let button_width = square_icon_button_size()[0] + layout::TOOLBAR_GAP;
            ui.allocate_ui_with_layout(
                egui::vec2(
                    (ui.available_width() - button_width).max(0.0),
                    DETAIL_TOOLBAR_HEIGHT,
                ),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add(Label::new(RichText::new(detail.topic).strong()).truncate());
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.label(
                            RichText::new(crate::time_format::local_date_time(detail.timestamp))
                                .color(tokens.text_secondary),
                        );
                        ui.label(RichText::new(detail.qos).color(tokens.text_secondary));
                        ui.label(
                            RichText::new(formatted_size(detail.byte_size))
                                .color(tokens.text_secondary),
                        );
                        if detail.retained {
                            ui.label(RichText::new("retained").color(tokens.accent).strong());
                        }
                    });
                },
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if with_icon_button_padding(ui, |ui| {
                    ui.add_sized(
                        square_icon_button_size(),
                        Button::new(RichText::new(regular::DOWNLOAD_SIMPLE).size(16.0)),
                    )
                })
                .on_hover_text("Export message to .cqm file")
                .clicked()
                {
                    send(commands, detail.export.clone());
                }
            });
        },
    );
}

fn payload_area(ui: &mut Ui, detail: &MessageDetail<'_>) {
    let mut payload = payload_text(detail.payload, detail.fallback_payload);
    let height = ui.available_height().max(0.0);
    let mut layouter = payload_highlight::layouter();
    ui.add_sized(
        [ui.available_width(), height],
        padded_text_edit(
            TextEdit::multiline(&mut payload)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .layouter(&mut layouter)
                .interactive(false),
        ),
    );
}

fn payload_text(payload: &[u8], fallback: &str) -> String {
    if payload.is_empty() {
        fallback.to_owned()
    } else {
        String::from_utf8_lossy(payload).into_owned()
    }
}

struct MessageDetail<'a> {
    topic: &'a str,
    timestamp: &'a str,
    qos: &'a str,
    retained: bool,
    byte_size: usize,
    payload: &'a [u8],
    fallback_payload: &'a str,
    export: AppCommand,
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
