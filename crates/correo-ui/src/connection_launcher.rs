use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionBadge, ConnectionState, ConnectionSummary,
};
use egui::{
    Button, CornerRadius, CursorIcon, Layout, RichText, ScrollArea, Sense, Stroke, StrokeKind,
    TextEdit, Ui, UiBuilder,
};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

const ROW_HEIGHT: f32 = 96.0;
const ROW_GAP: f32 = 6.0;
const HANDLE_WIDTH: f32 = 18.0;
const ACTION_WIDTH: f32 = 104.0;

pub fn panel(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal(|ui| {
        ui.heading(i18n.text("connections-heading"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.vertical(|ui| {
                if ui
                    .button(i18n.text("common-add-connection"))
                    .on_hover_text(i18n.text("common-add-connection"))
                    .clicked()
                {
                    send(commands, AppCommand::AddConnection);
                }
                if ui
                    .button(i18n.text("common-import-cqc"))
                    .on_hover_text(i18n.text("connection-import-tooltip"))
                    .clicked()
                {
                    send(commands, AppCommand::ImportConnections);
                }
                if ui
                    .button(i18n.text("common-export-cqc"))
                    .on_hover_text(i18n.text("connection-export-tooltip"))
                    .clicked()
                {
                    send(commands, AppCommand::ExportConnections);
                }
            });
        });
    });
    ui.separator();

    let mut filter = snapshot.connection_filter.clone();
    let response = ui.add(
        crate::widgets::padded_text_edit(TextEdit::singleline(&mut filter))
            .hint_text(i18n.text("common-search"))
            .desired_width(f32::INFINITY),
    );
    if response.changed() {
        send(commands, AppCommand::SearchConnections(filter));
    }

    ui.add_space(8.0);
    let connections = snapshot.filtered_connections();
    ScrollArea::vertical()
        .id_salt("connection-list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            for connection in connections {
                connection_row(ui, connection, snapshot, tokens, commands, i18n);
            }
        });
}

fn connection_row(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let selected = snapshot.selected_connection == Some(connection.id);
    let row_width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(row_width, ROW_HEIGHT), Sense::click_and_drag());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    response.dnd_set_drag_payload(connection.id);
    let dragged = response.dragged();
    let drop_target =
        response.contains_pointer() && egui::DragAndDrop::has_any_payload(ui.ctx()) && !dragged;

    let fill = if selected {
        tokens.accent_selected_bg
    } else if dragged {
        tokens.panel_raised
    } else if response.hovered() || response.contains_pointer() {
        tokens.panel_raised
    } else {
        tokens.panel_bg
    };
    let stroke = if selected || dragged || drop_target {
        tokens.accent
    } else {
        tokens.border
    };
    ui.painter().rect_filled(rect, CornerRadius::same(4), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );
    if dragged {
        ui.painter().rect_stroke(
            rect.shrink(2.0),
            CornerRadius::same(4),
            Stroke::new(2.0, tokens.accent),
            StrokeKind::Inside,
        );
    }
    if drop_target {
        let after = ui
            .ctx()
            .pointer_interact_pos()
            .is_some_and(|pointer| pointer.y > rect.center().y);
        let y = if after {
            rect.bottom() - 2.0
        } else {
            rect.top() + 2.0
        };
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 6.0, y),
                egui::pos2(rect.right() - 6.0, y),
            ],
            Stroke::new(3.0, tokens.accent),
        );
    }

    let content_rect = rect.shrink(8.0);
    let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
    let button_clicked = row_contents(&mut content_ui, connection, tokens, commands, i18n);

    if let Some(dropped) = response.dnd_release_payload() {
        let connection_id = *dropped;
        if connection_id != connection.id {
            let after = response
                .interact_pointer_pos()
                .or_else(|| ui.ctx().pointer_interact_pos())
                .is_some_and(|pointer| pointer.y > response.rect.center().y);
            send(
                commands,
                AppCommand::MoveConnection {
                    connection_id,
                    target_connection_id: connection.id,
                    after,
                },
            );
        }
    }

    if response.clicked() && !button_clicked {
        send(commands, AppCommand::SelectConnection(connection.id));
    }

    ui.add_space(ROW_GAP);
}

fn row_contents(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) -> bool {
    let mut button_clicked = false;
    ui.horizontal(|ui| {
        ui.set_height(ROW_HEIGHT - 16.0);
        ui.add_sized(
            [HANDLE_WIDTH, ROW_HEIGHT - 16.0],
            egui::Label::new(RichText::new(regular::DOTS_SIX_VERTICAL).color(tokens.text_disabled)),
        )
        .on_hover_text(i18n.text("connection-drag-reorder"));

        let info_width = (ui.available_width() - ACTION_WIDTH - 8.0).max(96.0);
        ui.allocate_ui(egui::vec2(info_width, ROW_HEIGHT - 16.0), |ui| {
            connection_info(ui, connection, tokens, i18n);
        });

        ui.allocate_ui_with_layout(
            egui::vec2(ACTION_WIDTH, ROW_HEIGHT - 16.0),
            Layout::right_to_left(egui::Align::Center),
            |ui| {
                if edit_button(ui, i18n).clicked() {
                    send(commands, AppCommand::OpenConnectionSettings(connection.id));
                    button_clicked = true;
                }
                if connection.state != ConnectionState::Connected {
                    let connect = ui.add_enabled(
                        connection.can_connect(),
                        Button::new(i18n.text("common-connect")),
                    );
                    if connect.clicked() {
                        send(commands, AppCommand::Connect(connection.id));
                        button_clicked = true;
                    }
                }
            },
        );
    });
    button_clicked
}

fn connection_info(ui: &mut Ui, connection: &ConnectionSummary, tokens: ThemeTokens, i18n: &I18n) {
    ui.vertical(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&connection.name).strong());
            ui.label(
                RichText::new(i18n.connection_state_label(connection.state))
                    .color(state_color(connection.state, tokens)),
            );
            for badge in &connection.badges {
                ui.label(RichText::new(badge_label(*badge)).color(tokens.accent));
            }
        });
        ui.label(RichText::new(&connection.endpoint).color(tokens.text_secondary));
    });
}

fn edit_button(ui: &mut Ui, i18n: &I18n) -> egui::Response {
    ui.add(
        Button::new(RichText::new(regular::GEAR).size(16.0)).min_size(egui::vec2(
            crate::theme::CONTROL_HEIGHT,
            crate::theme::CONTROL_HEIGHT,
        )),
    )
    .on_hover_text(i18n.text("connection-edit-tooltip"))
}

fn badge_label(badge: ConnectionBadge) -> &'static str {
    badge.label()
}

fn state_color(state: ConnectionState, tokens: ThemeTokens) -> egui::Color32 {
    match state {
        ConnectionState::Connected => tokens.success,
        ConnectionState::Connecting | ConnectionState::Reconnecting => tokens.warning,
        ConnectionState::Error => tokens.danger,
        ConnectionState::Disconnected => tokens.text_secondary,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
