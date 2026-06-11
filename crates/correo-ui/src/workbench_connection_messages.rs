use correo_core::{AppCommand, AppCommandSender, AppSnapshot, MessageRow, PublishHistoryRow};
use correo_style::layout;
use egui::{Button, Id, Rect, RichText, ScrollArea, Sense, TextEdit, Ui};
use egui_phosphor::regular;

use crate::{
    theme::{ThemeTokens, CONTROL_HEIGHT},
    widgets::{
        padded_text_edit, square_icon_button_size, tile_scroll_bar_rect_with_height,
        tile_table_fill, with_icon_button_padding,
    },
    workbench_connection_messages_filters::{message_visible_for_subscriptions, row_matches},
    workbench_connection_messages_text::{
        formatted_size, middle_ellipsis, right_aligned_text, text_width, truncated_text,
    },
    workbench_helpers::send,
    workbench_messages,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MessageOrigin {
    Outgoing,
    Incoming,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MessageKey {
    Outgoing(u32),
    Incoming(u32),
}

struct ConnectionMessageRow<'a> {
    key: MessageKey,
    topic: &'a str,
    timestamp: &'a str,
    qos: &'a str,
    retained: bool,
    payload_preview: &'a str,
    byte_size: usize,
    selected: bool,
}

pub(crate) fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    toolbar(ui, snapshot, origin, commands);
    ui.add_space(4.0);

    let rows = rows(snapshot, origin);
    message_table(
        ui,
        snapshot,
        origin,
        &rows,
        auto_scroll_enabled(ui, origin),
        tokens,
        commands,
    );
}

fn toolbar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    commands: &AppCommandSender,
) {
    let selected = selected_key(snapshot, origin);
    let toolbar_width = ui.available_width();
    ui.set_width(toolbar_width);
    ui.horizontal(|ui| {
        ui.set_width(toolbar_width);
        if icon_button(
            ui,
            regular::UPLOAD_SIMPLE,
            "Copy selected message to publish form",
            selected.is_some(),
            false,
        )
        .clicked()
        {
            if let Some(key) = selected {
                send(commands, copy_command(key));
            }
        }

        if icon_button(
            ui,
            regular::SHARE,
            "Show selected message in extra window",
            selected.is_some(),
            false,
        )
        .clicked()
        {
            if let Some(key) = selected {
                open_message(ui, snapshot, key);
            }
        }

        let mut filter = filter_text(snapshot, origin).to_owned();
        let button_width = layout::square_icon_button_side();
        let search_width = (toolbar_width - button_width * 4.0 - ui.spacing().item_spacing.x * 4.0)
            .max(button_width);
        if ui
            .add_sized(
                [search_width, CONTROL_HEIGHT],
                padded_text_edit(TextEdit::singleline(&mut filter).hint_text(search_hint(origin))),
            )
            .changed()
        {
            send(commands, search_command(origin, filter));
        }

        if icon_button(
            ui,
            regular::TRASH,
            "Clear messages",
            source_has_messages(snapshot, origin),
            false,
        )
        .clicked()
        {
            send(commands, clear_command(origin));
        }

        let auto_scroll = auto_scroll_enabled(ui, origin);
        if icon_button(
            ui,
            regular::MOUSE_SCROLL,
            "Toggle automatic scrolling",
            true,
            auto_scroll,
        )
        .clicked()
        {
            set_auto_scroll_enabled(ui, origin, !auto_scroll);
        }
    });
}

fn icon_button(
    ui: &mut Ui,
    icon: &str,
    hover_text: &str,
    enabled: bool,
    active: bool,
) -> egui::Response {
    let button = Button::new(RichText::new(icon).size(16.0)).selected(active);
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            with_icon_button_padding(ui, |ui| ui.add_sized(square_icon_button_size(), button))
        })
        .inner;
    response.on_hover_text(hover_text)
}

fn filter_text(snapshot: &AppSnapshot, origin: MessageOrigin) -> &str {
    match origin {
        MessageOrigin::Outgoing => &snapshot.workbench.publish.history_filter,
        MessageOrigin::Incoming => &snapshot.workbench.subscribe.message_filter,
    }
}

fn search_hint(origin: MessageOrigin) -> &'static str {
    match origin {
        MessageOrigin::Outgoing => "Search outgoing",
        MessageOrigin::Incoming => "Search incoming",
    }
}

fn search_command(origin: MessageOrigin, filter: String) -> AppCommand {
    match origin {
        MessageOrigin::Outgoing => AppCommand::SearchPublishHistory(filter),
        MessageOrigin::Incoming => AppCommand::SearchMessages(filter),
    }
}

fn rows(snapshot: &AppSnapshot, origin: MessageOrigin) -> Vec<ConnectionMessageRow<'_>> {
    let filter = filter_text(snapshot, origin).to_ascii_lowercase();
    match origin {
        MessageOrigin::Outgoing => snapshot
            .workbench
            .publish
            .history
            .iter()
            .filter(|row| row_matches(row.topic.as_str(), row.payload_preview.as_str(), &filter))
            .map(|row| outgoing_row(snapshot, row))
            .collect(),
        MessageOrigin::Incoming => snapshot
            .workbench
            .messages
            .iter()
            .filter(|message| message_visible_for_subscriptions(message, snapshot))
            .filter(|message| row_matches(&message.topic, &message.payload_preview, &filter))
            .map(|message| incoming_row(snapshot, message))
            .collect(),
    }
}

fn outgoing_row<'a>(
    snapshot: &AppSnapshot,
    row: &'a PublishHistoryRow,
) -> ConnectionMessageRow<'a> {
    ConnectionMessageRow {
        key: MessageKey::Outgoing(row.id),
        topic: &row.topic,
        timestamp: &row.timestamp,
        qos: row.qos.label(),
        retained: row.retained,
        payload_preview: &row.payload_preview,
        byte_size: row.byte_size,
        selected: snapshot.workbench.publish.selected_history_id == Some(row.id),
    }
}

fn incoming_row<'a>(snapshot: &AppSnapshot, message: &'a MessageRow) -> ConnectionMessageRow<'a> {
    ConnectionMessageRow {
        key: MessageKey::Incoming(message.id),
        topic: &message.topic,
        timestamp: &message.timestamp,
        qos: message.qos.label(),
        retained: message.retained,
        payload_preview: &message.payload_preview,
        byte_size: message.byte_size,
        selected: snapshot.workbench.selected_message_id == Some(message.id),
    }
}

fn message_table(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    origin: MessageOrigin,
    rows: &[ConnectionMessageRow<'_>],
    auto_scroll: bool,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    ui.spacing_mut().item_spacing.y = 0.0;
    let table_height = ui
        .available_rect_before_wrap()
        .height()
        .max(layout::TABLE_MIN_HEIGHT);
    ScrollArea::vertical()
        .id_salt(match origin {
            MessageOrigin::Outgoing => "outgoing-messages-table",
            MessageOrigin::Incoming => "incoming-messages-table",
        })
        .max_height(table_height)
        .scroll_bar_rect(tile_scroll_bar_rect_with_height(ui, table_height))
        .stick_to_bottom(auto_scroll)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (index, row) in rows.iter().enumerate() {
                message_row(ui, snapshot, index, row, tokens, commands);
            }
            fill_remaining_table_space(ui, rows.len(), table_height, tokens);
        });
}

fn fill_remaining_table_space(
    ui: &mut Ui,
    row_count: usize,
    table_height: f32,
    tokens: ThemeTokens,
) {
    let used_height = row_count as f32 * layout::MESSAGE_TABLE_ROW_HEIGHT;
    let mut remaining = (table_height - used_height).max(0.0);
    let mut index = row_count;
    while remaining > 0.0 {
        let height = remaining.min(layout::MESSAGE_TABLE_ROW_HEIGHT);
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::ZERO,
            tile_table_fill(index, tokens),
        );
        remaining -= height;
        index += 1;
    }
}

fn message_row(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    index: usize,
    row: &ConnectionMessageRow<'_>,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let row_width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(row_width, layout::MESSAGE_TABLE_ROW_HEIGHT),
        Sense::click(),
    );
    let fill = if row.selected {
        ui.visuals().selection.bg_fill
    } else if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        tile_table_fill(index, tokens)
    };
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, fill);

    response.context_menu(|ui| {
        if ui.button("Export .cqm").clicked() {
            send(commands, export_command(row.key));
            ui.close_menu();
        }
    });
    if response.clicked() {
        send(commands, select_command(row.key));
    }
    if response.double_clicked() {
        send(commands, select_command(row.key));
        open_message(ui, snapshot, row.key);
    }

    let right = rect.right() - layout::MESSAGE_ROW_PADDING_RIGHT;
    let meta_rect = right_rect(rect, right, layout::MESSAGE_ROW_META_WIDTH);
    let topic_y = rect.top() + 7.0;
    let preview_y = rect.top() + 28.0;
    let topic_font = egui::TextStyle::Button.resolve(ui.style());
    let meta_font = egui::TextStyle::Small.resolve(ui.style());
    let timestamp = crate::time_format::local_date_time(row.timestamp);
    let size = formatted_size(row.byte_size);
    let qos_and_size = if row.retained {
        format!("Retained · {} · {size}", row.qos)
    } else {
        format!("{} · {size}", row.qos)
    };
    let topic_left = rect.left() + layout::SUBSCRIPTION_ROW_PADDING_X;
    let topic_right =
        right - text_width(ui, &timestamp, meta_font.clone()) - layout::MESSAGE_ROW_TOPIC_META_GAP;
    let preview_right = right
        - text_width(ui, &qos_and_size, meta_font.clone())
        - layout::MESSAGE_ROW_TOPIC_META_GAP;
    let topic_width = (topic_right - topic_left).max(0.0);
    let preview_width = (preview_right - topic_left).max(0.0);
    let topic = middle_ellipsis(ui, row.topic, topic_font.clone(), topic_width);
    ui.painter().text(
        egui::pos2(topic_left, topic_y),
        egui::Align2::LEFT_TOP,
        topic,
        topic_font,
        ui.visuals().text_color(),
    );
    truncated_text(
        ui,
        egui::pos2(topic_left, preview_y),
        preview_width,
        row.payload_preview,
        meta_font,
        tokens.text_secondary,
    );
    right_aligned_text(
        ui,
        meta_rect.right_top() + egui::vec2(0.0, 7.0),
        &timestamp,
        ui.visuals().text_color(),
    );
    right_aligned_text(
        ui,
        meta_rect.right_top() + egui::vec2(0.0, 28.0),
        &qos_and_size,
        tokens.text_secondary,
    );
}

fn right_rect(row: Rect, right: f32, width: f32) -> Rect {
    Rect::from_min_max(
        egui::pos2((right - width).max(row.left()), row.top()),
        egui::pos2(right, row.bottom()),
    )
}

fn select_command(key: MessageKey) -> AppCommand {
    match key {
        MessageKey::Outgoing(id) => AppCommand::SelectPublishHistoryMessage(id),
        MessageKey::Incoming(id) => AppCommand::SelectMessage(id),
    }
}

fn copy_command(key: MessageKey) -> AppCommand {
    match key {
        MessageKey::Outgoing(id) => AppCommand::CopyPublishHistoryMessageToPublishForm(id),
        MessageKey::Incoming(id) => AppCommand::CopyIncomingMessageToPublishForm(id),
    }
}

fn clear_command(origin: MessageOrigin) -> AppCommand {
    match origin {
        MessageOrigin::Outgoing => AppCommand::ClearPublishHistory,
        MessageOrigin::Incoming => AppCommand::ClearIncomingMessages,
    }
}

fn export_command(key: MessageKey) -> AppCommand {
    match key {
        MessageKey::Outgoing(id) => AppCommand::ExportPublishHistoryMessage(id),
        MessageKey::Incoming(id) => AppCommand::ExportIncomingMessage(id),
    }
}

fn open_message(ui: &Ui, snapshot: &AppSnapshot, key: MessageKey) {
    match key {
        MessageKey::Outgoing(id) => workbench_messages::open_outgoing_message(ui.ctx(), id),
        MessageKey::Incoming(id) => {
            if snapshot
                .workbench
                .messages
                .iter()
                .any(|message| message.id == id)
            {
                workbench_messages::open_incoming_message(ui.ctx(), id);
            }
        }
    }
}

fn selected_key(snapshot: &AppSnapshot, origin: MessageOrigin) -> Option<MessageKey> {
    match origin {
        MessageOrigin::Outgoing => snapshot
            .workbench
            .publish
            .selected_history_id
            .filter(|id| {
                snapshot
                    .workbench
                    .publish
                    .history
                    .iter()
                    .any(|row| row.id == *id)
            })
            .map(MessageKey::Outgoing),
        MessageOrigin::Incoming => snapshot
            .workbench
            .selected_message_id
            .filter(|id| {
                snapshot
                    .workbench
                    .messages
                    .iter()
                    .any(|message| message.id == *id)
            })
            .map(MessageKey::Incoming),
    }
}

fn source_has_messages(snapshot: &AppSnapshot, origin: MessageOrigin) -> bool {
    match origin {
        MessageOrigin::Outgoing => !snapshot.workbench.publish.history.is_empty(),
        MessageOrigin::Incoming => !snapshot.workbench.messages.is_empty(),
    }
}

fn auto_scroll_enabled(ui: &Ui, origin: MessageOrigin) -> bool {
    ui.ctx()
        .data_mut(|data| *data.get_persisted_mut_or(auto_scroll_id(origin), true))
}

fn set_auto_scroll_enabled(ui: &Ui, origin: MessageOrigin, enabled: bool) {
    ui.ctx()
        .data_mut(|data| data.insert_persisted(auto_scroll_id(origin), enabled));
}

fn auto_scroll_id(origin: MessageOrigin) -> Id {
    match origin {
        MessageOrigin::Outgoing => Id::new("outgoing-messages-auto-scroll"),
        MessageOrigin::Incoming => Id::new("incoming-messages-auto-scroll"),
    }
}
