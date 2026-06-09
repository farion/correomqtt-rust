use egui::{
    pos2, vec2, Align, Button, Color32, CursorIcon, Id, Layout, Rect, RichText, Sense, Stroke, Ui,
    UiBuilder,
};
use egui_phosphor::regular;

use crate::{
    theme::ThemeTokens,
    widgets::{square_icon_button_size, with_icon_button_padding},
};

const DEFAULT_CENTER_RATIO: f32 = 0.5;
const DEFAULT_STACK_RATIO: f32 = 0.55;
const DIVIDER_SIZE: f32 = 8.0;
const MIN_PANE_WIDTH: f32 = 240.0;
const MIN_TOP_HEIGHT: f32 = 180.0;
const MIN_BOTTOM_HEIGHT: f32 = 150.0;
const PANE_PADDING: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchLayoutMode {
    Publish,
    Both,
    Subscribe,
}

impl WorkbenchLayoutMode {
    fn icon(self) -> &'static str {
        match self {
            Self::Publish => regular::ALIGN_RIGHT_SIMPLE,
            Self::Both => regular::LAYOUT,
            Self::Subscribe => regular::ALIGN_LEFT_SIMPLE,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Publish => "Publish only",
            Self::Both => "Publish and subscriptions",
            Self::Subscribe => "Subscription only",
        }
    }

    fn value(self) -> u8 {
        match self {
            Self::Publish => 0,
            Self::Both => 1,
            Self::Subscribe => 2,
        }
    }

    fn from_value(value: u8) -> Self {
        match value {
            0 => Self::Publish,
            2 => Self::Subscribe,
            _ => Self::Both,
        }
    }
}

pub(crate) fn current_mode(ui: &Ui) -> WorkbenchLayoutMode {
    ui.ctx().data_mut(|data| {
        WorkbenchLayoutMode::from_value(
            *data.get_persisted_mut_or(mode_id(), WorkbenchLayoutMode::Both.value()),
        )
    })
}

pub(crate) fn mode_buttons(ui: &mut Ui, tokens: ThemeTokens) -> WorkbenchLayoutMode {
    let mut selected = current_mode(ui);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for mode in [
            WorkbenchLayoutMode::Publish,
            WorkbenchLayoutMode::Both,
            WorkbenchLayoutMode::Subscribe,
        ] {
            let active = selected == mode;
            let response = with_icon_button_padding(ui, |ui| {
                ui.add_sized(
                    square_icon_button_size(),
                    Button::new(RichText::new(mode.icon()).size(16.0)).fill(if active {
                        tokens.accent_selected_bg
                    } else {
                        tokens.panel_raised
                    }),
                )
            })
            .on_hover_text(mode.label());
            if response.clicked() {
                selected = mode;
                ui.ctx()
                    .data_mut(|data| data.insert_persisted(mode_id(), mode.value()));
            }
        }
    });
    selected
}

pub(crate) fn show(
    ui: &mut Ui,
    tokens: ThemeTokens,
    mode: WorkbenchLayoutMode,
    publish: impl FnOnce(&mut Ui),
    subscribe: impl FnOnce(&mut Ui),
    outgoing: impl FnOnce(&mut Ui),
    incoming: impl FnOnce(&mut Ui),
) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    match mode {
        WorkbenchLayoutMode::Both => {
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
        WorkbenchLayoutMode::Publish => {
            stack_split(
                ui,
                Id::new("workbench-publish-only-stack-ratio"),
                rect,
                tokens,
                publish,
                outgoing,
            );
        }
        WorkbenchLayoutMode::Subscribe => {
            stack_split(
                ui,
                Id::new("workbench-subscribe-only-stack-ratio"),
                rect,
                tokens,
                subscribe,
                incoming,
            );
        }
    }
}

fn center_split(ui: &mut Ui, rect: Rect, tokens: ThemeTokens) -> (Rect, Rect) {
    let usable = (rect.width() - DIVIDER_SIZE).max(1.0);
    let min_left = MIN_PANE_WIDTH.min(usable * 0.45);
    let min_right = MIN_PANE_WIDTH.min((usable - min_left).max(0.0));
    let max_left = (usable - min_right).max(min_left);
    let id = Id::new("workbench-center-ratio");
    let mut left_width = ratio(ui, id, DEFAULT_CENTER_RATIO) * usable;
    left_width = left_width.clamp(min_left, max_left);

    let divider = Rect::from_min_size(
        pos2(rect.left() + left_width, rect.top()),
        vec2(DIVIDER_SIZE, rect.height()),
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
        Rect::from_min_max(rect.left_top(), pos2(divider.left(), rect.bottom())),
        Rect::from_min_max(pos2(divider.right(), rect.top()), rect.right_bottom()),
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
    let usable = (rect.height() - DIVIDER_SIZE).max(1.0);
    let min_top = MIN_TOP_HEIGHT.min(usable * 0.6);
    let min_bottom = MIN_BOTTOM_HEIGHT.min((usable - min_top).max(0.0));
    let max_top = (usable - min_bottom).max(min_top);
    let mut top_height = ratio(ui, id, DEFAULT_STACK_RATIO) * usable;
    top_height = top_height.clamp(min_top, max_top);

    let divider = Rect::from_min_size(
        pos2(rect.left(), rect.top() + top_height),
        vec2(rect.width(), DIVIDER_SIZE),
    );
    let response = ui
        .allocate_rect(divider, Sense::click_and_drag())
        .on_hover_cursor(CursorIcon::ResizeVertical);
    if response.dragged() {
        top_height = (top_height + response.drag_delta().y).clamp(min_top, max_top);
        store_ratio(ui, id, top_height / usable);
    }
    draw_divider(ui, divider, tokens.border, false);

    pane(
        ui,
        Rect::from_min_max(rect.left_top(), pos2(rect.right(), divider.top())),
        top,
    );
    pane(
        ui,
        Rect::from_min_max(pos2(rect.left(), divider.bottom()), rect.right_bottom()),
        bottom,
    );
}

fn pane(ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut Ui)) {
    let content_rect = rect.shrink(PANE_PADDING);
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(content_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(content_rect);
    add_contents(&mut child);
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

fn mode_id() -> Id {
    Id::new("workbench-layout-mode")
}
