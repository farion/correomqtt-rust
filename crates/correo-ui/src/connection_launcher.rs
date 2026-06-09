use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, ConnectionBadge, ConnectionState, ConnectionSummary,
};
use egui::{
    Button, CornerRadius, CursorIcon, Label, Layout, Rect, Response, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, TextEdit, Ui, UiBuilder,
};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::theme::{ThemeTokens, CONTROL_HEIGHT};
use crate::widgets::{
    disable_tile_text_selection, tile_list_content_width, tile_scroll_bar_rect, TILE_GAP,
};

const ROW_HEIGHT: f32 = 58.0;
const ROW_PADDING_X: f32 = 8.0;
const ROW_PADDING_Y: f32 = 6.0;
const STATUS_ICON_WIDTH: f32 = 26.0;
const STATUS_ICON_SIZE: f32 = 21.0;
const FEATURE_ICON_WIDTH: f32 = 19.0;
const FEATURE_ICON_SIZE: f32 = 17.0;
const FEATURE_ICON_GAP: f32 = 2.0;
const ACTION_BUTTON_SIDE: f32 = 28.0;
const ACTION_ICON_SIZE: f32 = 15.0;
const ACTION_BUTTON_PADDING: f32 = 5.0;
const ACTION_BUTTON_GAP: f32 = 4.0;

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
            if icon_button(ui, regular::PLUS, true)
                .on_hover_text(i18n.text("common-add-connection"))
                .clicked()
            {
                send(commands, AppCommand::AddConnection);
            }
        });
    });

    let mut filter = snapshot.connection_filter.clone();
    let response = ui.add_sized(
        [tile_list_content_width(ui), CONTROL_HEIGHT],
        crate::widgets::padded_text_edit(TextEdit::singleline(&mut filter))
            .hint_text(i18n.text("common-search")),
    );
    if response.changed() {
        send(commands, AppCommand::SearchConnections(filter));
    }

    ui.add_space(8.0);
    let connections = snapshot.filtered_connections();
    let list_height = (ui.available_height() - CONTROL_HEIGHT - 14.0).max(120.0);
    ScrollArea::vertical()
        .id_salt("connection-list")
        .max_height(list_height)
        .auto_shrink([false, false])
        .scroll_bar_rect(tile_scroll_bar_rect(ui))
        .show(ui, |ui| {
            ui.set_width(tile_list_content_width(ui));
            for connection in connections {
                connection_row(ui, connection, snapshot, tokens, commands, i18n);
            }
        });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
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

    let content_rect = rect.shrink2(egui::vec2(ROW_PADDING_X, ROW_PADDING_Y));
    let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
    disable_tile_text_selection(&mut content_ui);
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

    ui.add_space(TILE_GAP);
}

fn row_contents(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) -> bool {
    let mut button_clicked = false;
    let content_height = ROW_HEIGHT - (ROW_PADDING_Y * 2.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.set_height(content_height);
        ui.allocate_ui(egui::vec2(STATUS_ICON_WIDTH, content_height), |ui| {
            ui.add_sized(
                [STATUS_ICON_WIDTH, STATUS_ICON_SIZE + 2.0],
                Label::new(
                    RichText::new(state_icon(connection.state))
                        .size(STATUS_ICON_SIZE)
                        .color(state_color(connection.state, tokens)),
                ),
            )
            .on_hover_text(i18n.connection_state_label(connection.state));
        });

        let info_width = ui.available_width().max(96.0);
        ui.allocate_ui(egui::vec2(info_width, content_height), |ui| {
            button_clicked = connection_info(ui, connection, tokens, commands, i18n);
        });
    });
    button_clicked
}

fn connection_info(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) -> bool {
    let mut button_clicked = false;
    let rect = ui.max_rect();
    let action_width = (ACTION_BUTTON_SIDE * 2.0) + ACTION_BUTTON_GAP;
    let features = feature_icons(connection);
    let feature_width = feature_group_width(features.len());
    let right_width = action_width.max(feature_width);
    let right = rect.right();
    let right_left = (right - right_width).max(rect.left());
    let right_rect = Rect::from_min_max(egui::pos2(right_left, rect.top()), rect.right_bottom());
    let action_rect = Rect::from_min_max(
        egui::pos2(
            (right - action_width).max(right_left),
            rect.bottom() - ACTION_BUTTON_SIDE,
        ),
        rect.right_bottom(),
    );
    let text_right = (right_rect.left() - 6.0).max(rect.left());
    let line_height = ui.text_style_height(&egui::TextStyle::Body);
    let title_rect = Rect::from_min_max(
        rect.left_top(),
        egui::pos2(text_right, rect.top() + line_height),
    );
    let endpoint_rect = Rect::from_min_max(
        egui::pos2(rect.left(), title_rect.bottom()),
        egui::pos2(text_right, title_rect.bottom() + line_height),
    );
    let feature_rect = Rect::from_min_max(
        egui::pos2((right - feature_width).max(right_left), rect.top()),
        egui::pos2(right, title_rect.bottom()),
    );
    let text_rect = Rect::from_min_max(rect.left_top(), endpoint_rect.right_bottom());
    connection_text_rows(ui, connection, tokens, text_rect);
    connection_feature_row(ui, features, tokens, feature_rect, line_height);
    if connection_action_buttons(ui, connection, commands, i18n, action_rect) {
        button_clicked = true;
    }
    button_clicked
}

fn connection_text_rows(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    tokens: ThemeTokens,
    rect: Rect,
) {
    let mut text_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::top_down(egui::Align::Min)),
    );
    text_ui.spacing_mut().item_spacing.y = 0.0;
    text_ui.set_width(rect.width());
    text_ui
        .add(Label::new(RichText::new(&connection.name).strong()).truncate())
        .on_hover_text(&connection.name);
    let endpoint = endpoint_label(connection);
    text_ui
        .add(Label::new(RichText::new(&endpoint).color(tokens.text_secondary)).truncate())
        .on_hover_text(&connection.endpoint);
}

fn connection_feature_row(
    ui: &mut Ui,
    features: Vec<FeatureIcon>,
    tokens: ThemeTokens,
    rect: Rect,
    line_height: f32,
) {
    let mut feature_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(egui::Align::Center)),
    );
    feature_ui.spacing_mut().item_spacing.x = FEATURE_ICON_GAP;
    for feature in features {
        feature_icon(&mut feature_ui, feature, tokens, line_height);
    }
}

fn connection_action_buttons(
    ui: &mut Ui,
    connection: &ConnectionSummary,
    commands: &AppCommandSender,
    i18n: &I18n,
    rect: Rect,
) -> bool {
    let mut button_clicked = false;
    let mut action_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::right_to_left(egui::Align::Center)),
    );
    action_ui.spacing_mut().item_spacing.x = ACTION_BUTTON_GAP;
    if edit_button(&mut action_ui, i18n).clicked() {
        send(commands, AppCommand::OpenConnectionSettings(connection.id));
        button_clicked = true;
    }
    if connect_button(&mut action_ui, connection, i18n).clicked() {
        send(commands, AppCommand::Connect(connection.id));
        button_clicked = true;
    }

    button_clicked
}

fn connect_button(ui: &mut Ui, connection: &ConnectionSummary, i18n: &I18n) -> Response {
    let tooltip = if connection.can_connect() {
        i18n.text("common-connect")
    } else {
        i18n.disabled_reason_label(disabled_reason(connection))
    };
    icon_button(ui, regular::PLUG, connection.can_connect()).on_hover_text(tooltip)
}

fn edit_button(ui: &mut Ui, i18n: &I18n) -> Response {
    icon_button(ui, regular::PENCIL_SIMPLE, true)
        .on_hover_text(i18n.text("connection-edit-tooltip"))
}

fn icon_button(ui: &mut Ui, icon: &'static str, enabled: bool) -> Response {
    let button = Button::new(RichText::new(icon).size(ACTION_ICON_SIZE))
        .min_size(egui::vec2(ACTION_BUTTON_SIDE, ACTION_BUTTON_SIDE));
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(ACTION_BUTTON_PADDING, ACTION_BUTTON_PADDING);
        ui.spacing_mut().interact_size = egui::Vec2::splat(ACTION_BUTTON_SIDE);
        ui.add_enabled(enabled, button)
    })
    .inner
}

fn feature_icon(ui: &mut Ui, feature: FeatureIcon, tokens: ThemeTokens, line_height: f32) {
    ui.add_sized(
        [FEATURE_ICON_WIDTH, line_height],
        Label::new(
            RichText::new(feature.icon)
                .size(FEATURE_ICON_SIZE)
                .color(tokens.text_secondary),
        ),
    )
    .on_hover_text(feature.label);
}

fn feature_icons(connection: &ConnectionSummary) -> Vec<FeatureIcon> {
    let mut icons = vec![mqtt_feature_icon(&connection.mqtt_version)];
    for badge in &connection.badges {
        if let Some(icon) = badge_feature_icon(*badge) {
            icons.push(icon);
        }
    }
    icons
}

fn badge_feature_icon(badge: ConnectionBadge) -> Option<FeatureIcon> {
    match badge {
        ConnectionBadge::Credentials => Some(FeatureIcon::new(regular::KEY, "Credentials set")),
        ConnectionBadge::Tls => Some(FeatureIcon::new(regular::LOCK_KEY, "TLS/SSL")),
        ConnectionBadge::Proxy => Some(FeatureIcon::new(regular::SUBWAY, "Tunnel")),
        ConnectionBadge::Lwt => None,
    }
}

fn mqtt_feature_icon(version: &str) -> FeatureIcon {
    if version.contains('5') {
        FeatureIcon::new(regular::NUMBER_CIRCLE_FIVE, "MQTT 5")
    } else {
        FeatureIcon::new(regular::NUMBER_CIRCLE_THREE, "MQTT 3")
    }
}

fn feature_group_width(count: usize) -> f32 {
    if count == 0 {
        0.0
    } else {
        (count as f32 * FEATURE_ICON_WIDTH) + ((count - 1) as f32 * FEATURE_ICON_GAP)
    }
}

fn endpoint_label(connection: &ConnectionSummary) -> String {
    if has_tunnel(connection) {
        format!("via {} (tunnel)", connection.endpoint)
    } else {
        connection.endpoint.clone()
    }
}

fn has_tunnel(connection: &ConnectionSummary) -> bool {
    connection.badges.contains(&ConnectionBadge::Proxy)
}

fn state_icon(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connected
        | ConnectionState::Connecting
        | ConnectionState::Reconnecting => regular::WIFI_HIGH,
        ConnectionState::Disconnected | ConnectionState::Error => regular::WIFI_SLASH,
    }
}

fn disabled_reason(connection: &ConnectionSummary) -> correo_core::ConnectDisabledReason {
    connection
        .disabled_reason
        .unwrap_or(correo_core::ConnectDisabledReason::Busy)
}

fn state_color(state: ConnectionState, tokens: ThemeTokens) -> egui::Color32 {
    match state {
        ConnectionState::Connected => tokens.success,
        ConnectionState::Connecting | ConnectionState::Reconnecting => tokens.warning,
        ConnectionState::Error => tokens.danger,
        ConnectionState::Disconnected => tokens.text_secondary,
    }
}

#[derive(Clone, Copy)]
struct FeatureIcon {
    icon: &'static str,
    label: &'static str,
}

impl FeatureIcon {
    fn new(icon: &'static str, label: &'static str) -> Self {
        Self { icon, label }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mqtt_feature_uses_number_circle_icons() {
        assert_eq!(
            mqtt_feature_icon("MQTT v5").icon,
            regular::NUMBER_CIRCLE_FIVE
        );
        assert_eq!(
            mqtt_feature_icon("MQTT 3.1.1").icon,
            regular::NUMBER_CIRCLE_THREE
        );
    }

    #[test]
    fn badge_features_match_connection_tile_spec() {
        assert_eq!(
            badge_feature_icon(ConnectionBadge::Credentials)
                .expect("credentials icon")
                .icon,
            regular::KEY
        );
        assert_eq!(
            badge_feature_icon(ConnectionBadge::Tls)
                .expect("tls icon")
                .icon,
            regular::LOCK_KEY
        );
        assert_eq!(
            badge_feature_icon(ConnectionBadge::Proxy)
                .expect("tunnel icon")
                .icon,
            regular::SUBWAY
        );
        assert!(badge_feature_icon(ConnectionBadge::Lwt).is_none());
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
