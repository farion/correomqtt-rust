use correo_style::layout;
use egui::{
    pos2, vec2, Align, Button, Color32, CursorIcon, Id, Layout, Rect, RichText, Sense, Stroke, Ui,
    UiBuilder,
};
use egui_phosphor::regular;

use crate::{
    theme::ThemeTokens,
    widgets::{square_icon_button_size, with_icon_button_padding},
};

const PANE_TITLE_SIZE: f32 = 18.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchPaneSide {
    Publish,
    Subscribe,
}

impl WorkbenchPaneSide {
    fn hide_icon(self) -> &'static str {
        match self {
            Self::Publish => regular::ARROW_SQUARE_LEFT,
            Self::Subscribe => regular::ARROW_SQUARE_RIGHT,
        }
    }

    fn show_icon(self) -> &'static str {
        match self {
            Self::Publish => regular::ARROW_SQUARE_RIGHT,
            Self::Subscribe => regular::ARROW_SQUARE_LEFT,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Publish => "Publish",
            Self::Subscribe => "Subscribe",
        }
    }
}

pub(crate) fn show(
    ui: &mut Ui,
    tokens: ThemeTokens,
    publish: impl FnOnce(&mut Ui),
    subscribe: impl FnOnce(&mut Ui),
    outgoing: impl FnOnce(&mut Ui),
    incoming: impl FnOnce(&mut Ui),
) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    match (
        is_collapsed(ui, WorkbenchPaneSide::Publish),
        is_collapsed(ui, WorkbenchPaneSide::Subscribe),
    ) {
        (false, false) => {
            let (left, right) = center_split(ui, rect, tokens);
            stack_split(
                ui,
                Id::new("workbench-left-stack-ratio"),
                left,
                tokens,
                publish,
                outgoing,
            );
            stack_split(
                ui,
                Id::new("workbench-right-stack-ratio"),
                right,
                tokens,
                subscribe,
                incoming,
            );
        }
        (true, false) => {
            let (collapsed, content) = left_collapsed_split(ui, rect, tokens);
            collapsed_pane(ui, collapsed, WorkbenchPaneSide::Publish);
            stack_split(
                ui,
                Id::new("workbench-subscribe-expanded-stack-ratio"),
                content,
                tokens,
                subscribe,
                incoming,
            );
        }
        (false, true) => {
            let (content, collapsed) = right_collapsed_split(ui, rect, tokens);
            stack_split(
                ui,
                Id::new("workbench-publish-expanded-stack-ratio"),
                content,
                tokens,
                publish,
                outgoing,
            );
            collapsed_pane(ui, collapsed, WorkbenchPaneSide::Subscribe);
        }
        (true, true) => {
            let left = Rect::from_min_size(
                rect.left_top(),
                vec2(layout::WORKBENCH_COLLAPSED_PANE_WIDTH, rect.height()),
            );
            let right = Rect::from_min_max(
                pos2(
                    (rect.right() - layout::WORKBENCH_COLLAPSED_PANE_WIDTH).max(left.right()),
                    rect.top(),
                ),
                rect.right_bottom(),
            );
            collapsed_pane(ui, left, WorkbenchPaneSide::Publish);
            collapsed_pane(ui, right, WorkbenchPaneSide::Subscribe);
        }
    }
}

pub(crate) fn pane_title(ui: &mut Ui, title: &str, side: WorkbenchPaneSide) {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), layout::CONTROL_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(RichText::new(title).strong().size(PANE_TITLE_SIZE));
            if !is_collapsed(ui, opposite_side(side)) {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let response =
                        collapse_button(ui, side.hide_icon(), &format!("Hide {title} pane"));
                    if response.clicked() {
                        set_collapsed(ui, side, true);
                    }
                });
            }
        },
    );
}

fn center_split(ui: &mut Ui, rect: Rect, tokens: ThemeTokens) -> (Rect, Rect) {
    let usable = (rect.width() - layout::WORKBENCH_DIVIDER_SIZE).max(1.0);
    let min_left = layout::WORKBENCH_MIN_PANE_WIDTH.min(usable * 0.45);
    let min_right = layout::WORKBENCH_MIN_PANE_WIDTH.min((usable - min_left).max(0.0));
    let max_left = (usable - min_right).max(min_left);
    let id = Id::new("workbench-center-ratio");
    let mut left_width = ratio(ui, id, layout::WORKBENCH_DEFAULT_CENTER_RATIO) * usable;
    left_width = left_width.clamp(min_left, max_left);

    let divider = Rect::from_min_size(
        pos2(rect.left() + left_width, rect.top()),
        vec2(layout::WORKBENCH_DIVIDER_SIZE, rect.height()),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeHorizontal);
    if response.dragged() {
        left_width = (left_width + response.drag_delta().x).clamp(min_left, max_left);
        store_ratio(ui, id, left_width / usable);
    }
    draw_divider(ui, divider, tokens.border, true);

    (
        Rect::from_min_max(
            rect.left_top(),
            pos2(
                divider.left() - layout::WORKBENCH_CENTER_SPLIT_GUTTER,
                rect.bottom(),
            ),
        ),
        Rect::from_min_max(
            pos2(
                divider.right() + layout::WORKBENCH_CENTER_SPLIT_GUTTER,
                rect.top(),
            ),
            rect.right_bottom(),
        ),
    )
}

fn stack_split(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    tokens: ThemeTokens,
    top: impl FnOnce(&mut Ui),
    bottom: impl FnOnce(&mut Ui),
) {
    let usable = (rect.height() - layout::WORKBENCH_DIVIDER_SIZE).max(1.0);
    let min_top = layout::WORKBENCH_MIN_TOP_HEIGHT.min(usable * 0.6);
    let min_bottom = layout::WORKBENCH_MIN_BOTTOM_HEIGHT.min((usable - min_top).max(0.0));
    let max_top = (usable - min_bottom).max(min_top);
    let mut top_height = ratio(ui, id, layout::WORKBENCH_DEFAULT_STACK_RATIO) * usable;
    top_height = top_height.clamp(min_top, max_top);

    let divider = Rect::from_min_size(
        pos2(rect.left(), rect.top() + top_height),
        vec2(rect.width(), layout::WORKBENCH_DIVIDER_SIZE),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeVertical);
    if response.dragged() {
        top_height = (top_height + response.drag_delta().y).clamp(min_top, max_top);
        store_ratio(ui, id, top_height / usable);
    }
    draw_divider(ui, divider, tokens.border, false);

    top_pane(
        ui,
        Rect::from_min_max(rect.left_top(), pos2(rect.right(), divider.top())),
        top,
    );
    bottom_pane(
        ui,
        Rect::from_min_max(pos2(rect.left(), divider.bottom()), rect.right_bottom()),
        bottom,
    );
}

fn top_pane(ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut Ui)) {
    pane(ui, rect, layout::WORKBENCH_PANE_PADDING_Y, add_contents);
}

fn bottom_pane(ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut Ui)) {
    pane(ui, rect, 0.0, add_contents);
}

fn pane(ui: &mut Ui, rect: Rect, bottom_padding: f32, add_contents: impl FnOnce(&mut Ui)) {
    let content_rect = Rect::from_min_max(
        pos2(
            rect.left() + layout::WORKBENCH_PANE_PADDING_X,
            rect.top() + layout::WORKBENCH_PANE_PADDING_Y,
        ),
        pos2(
            rect.right() - layout::WORKBENCH_PANE_PADDING_X,
            rect.bottom() - bottom_padding,
        ),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(content_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(content_rect);
    add_contents(&mut child);
}

fn left_collapsed_split(ui: &mut Ui, rect: Rect, tokens: ThemeTokens) -> (Rect, Rect) {
    let column_width = collapsed_column_width(rect);
    let collapsed = Rect::from_min_size(rect.left_top(), vec2(column_width, rect.height()));
    let divider = Rect::from_min_size(
        pos2(collapsed.right(), rect.top()),
        vec2(layout::WORKBENCH_DIVIDER_SIZE, rect.height()),
    );
    draw_divider(ui, divider, tokens.border, true);
    let content_left = (divider.right() + layout::WORKBENCH_CENTER_SPLIT_GUTTER).min(rect.right());
    let content = Rect::from_min_max(pos2(content_left, rect.top()), rect.right_bottom());
    (collapsed, content)
}

fn right_collapsed_split(ui: &mut Ui, rect: Rect, tokens: ThemeTokens) -> (Rect, Rect) {
    let column_width = collapsed_column_width(rect);
    let collapsed = Rect::from_min_max(
        pos2((rect.right() - column_width).max(rect.left()), rect.top()),
        rect.right_bottom(),
    );
    let divider = Rect::from_min_size(
        pos2(
            collapsed.left() - layout::WORKBENCH_DIVIDER_SIZE,
            rect.top(),
        ),
        vec2(layout::WORKBENCH_DIVIDER_SIZE, rect.height()),
    );
    draw_divider(ui, divider, tokens.border, true);
    let content_right = (divider.left() - layout::WORKBENCH_CENTER_SPLIT_GUTTER).max(rect.left());
    let content = Rect::from_min_max(rect.left_top(), pos2(content_right, rect.bottom()));
    (content, collapsed)
}

fn collapsed_column_width(rect: Rect) -> f32 {
    layout::WORKBENCH_COLLAPSED_PANE_WIDTH.min(rect.width().max(0.0))
}

fn collapsed_pane(ui: &mut Ui, rect: Rect, side: WorkbenchPaneSide) {
    let content_rect = rect.shrink2(vec2(0.0, layout::WORKBENCH_PANE_PADDING_Y));
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(content_rect)
            .layout(Layout::top_down(Align::Center)),
    );
    child.set_clip_rect(content_rect);
    let tooltip = format!("Show {} pane", side.label());
    if collapse_button(&mut child, side.show_icon(), &tooltip).clicked() {
        set_collapsed(&child, side, false);
    }
}

fn collapse_button(ui: &mut Ui, icon: &str, tooltip: &str) -> egui::Response {
    with_icon_button_padding(ui, |ui| {
        ui.add_sized(
            square_icon_button_size(),
            Button::new(RichText::new(icon).size(16.0)),
        )
    })
    .on_hover_text(tooltip)
}

fn is_collapsed(ui: &Ui, side: WorkbenchPaneSide) -> bool {
    ui.ctx()
        .data_mut(|data| *data.get_persisted_mut_or(collapse_id(side), false))
}

fn set_collapsed(ui: &Ui, side: WorkbenchPaneSide, collapsed: bool) {
    ui.ctx().data_mut(|data| {
        data.insert_persisted(collapse_id(side), collapsed);
        if collapsed {
            data.insert_persisted(collapse_id(opposite_side(side)), false);
        }
    });
}

fn collapse_id(side: WorkbenchPaneSide) -> Id {
    match side {
        WorkbenchPaneSide::Publish => Id::new("workbench-publish-collapsed"),
        WorkbenchPaneSide::Subscribe => Id::new("workbench-subscribe-collapsed"),
    }
}

fn opposite_side(side: WorkbenchPaneSide) -> WorkbenchPaneSide {
    match side {
        WorkbenchPaneSide::Publish => WorkbenchPaneSide::Subscribe,
        WorkbenchPaneSide::Subscribe => WorkbenchPaneSide::Publish,
    }
}

fn ratio(ui: &Ui, id: Id, default: f32) -> f32 {
    ui.ctx()
        .data_mut(|data| *data.get_persisted_mut_or(id, default))
        .clamp(0.15, 0.85)
}

fn store_ratio(ui: &Ui, id: Id, value: f32) {
    ui.ctx()
        .data_mut(|data| data.insert_persisted(id, value.clamp(0.15, 0.85)));
}

fn draw_divider(ui: &Ui, rect: Rect, color: Color32, vertical: bool) {
    let center = rect.center();
    let points = if vertical {
        [pos2(center.x, rect.top()), pos2(center.x, rect.bottom())]
    } else {
        [pos2(rect.left(), center.y), pos2(rect.right(), center.y)]
    };
    ui.painter().line_segment(points, Stroke::new(1.0, color));
}
