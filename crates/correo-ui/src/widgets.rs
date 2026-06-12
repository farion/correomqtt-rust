pub(crate) use correo_style::widgets::*;

use egui::{Align2, Color32, CursorIcon, FontId, Id, Response, Sense, TextEdit, Ui, WidgetText};
use egui_phosphor::regular;

const SEARCH_CLEAR_ICON_SIZE: f32 = 13.0;

pub(crate) fn clearable_search_edit(
    ui: &mut Ui,
    id: Option<Id>,
    text: &mut String,
    hint: impl Into<WidgetText>,
    width: f32,
) -> Response {
    let mut edit = TextEdit::singleline(text).hint_text(hint);
    if let Some(id) = id {
        edit = edit.id(id);
    }
    let mut response = ui.add_sized(
        [width, crate::theme::CONTROL_HEIGHT],
        padded_text_edit(edit),
    );

    if !text.is_empty() {
        let control_rect = response
            .rect
            .expand2(correo_style::layout::control_padding());
        let side = crate::theme::CONTROL_HEIGHT;
        let clear_rect = egui::Rect::from_center_size(
            egui::pos2(control_rect.right() - side * 0.5, control_rect.center().y),
            egui::vec2(side, side),
        );
        let clear_response = ui
            .interact(clear_rect, response.id.with("clear"), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        if clear_response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            ui.painter().rect_filled(
                clear_rect.shrink(5.0),
                ui.visuals().widgets.hovered.corner_radius,
                ui.visuals().widgets.hovered.bg_fill,
            );
        }
        let color = if clear_response.hovered() {
            ui.visuals().widgets.hovered.fg_stroke.color
        } else {
            ui.visuals().widgets.inactive.fg_stroke.color
        };
        ui.painter().text(
            clear_rect.center(),
            Align2::CENTER_CENTER,
            regular::X,
            FontId::proportional(SEARCH_CLEAR_ICON_SIZE),
            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 210),
        );
        if clear_response.clicked() {
            text.clear();
            response.mark_changed();
        }
    }

    response
}
