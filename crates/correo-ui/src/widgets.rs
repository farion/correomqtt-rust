use egui::{
    pos2, vec2, FontId, Response, Sense, TextEdit, TextStyle, Ui, Vec2, Widget, WidgetInfo,
    WidgetText, WidgetType,
};
use egui_phosphor::regular;

use crate::theme::{control_margin, control_padding, CONTROL_HEIGHT};

const CHECKBOX_ICON_SCALE: f32 = 2.0;
const CHECKBOX_TEXT_TRAILING_PADDING: f32 = 8.0;
const TILE_SCROLLBAR_GUTTER: f32 = 12.0;
const TILE_SCROLLBAR_INSET: f32 = 4.0;
pub(crate) const TILE_GAP: f32 = 4.0;
pub(crate) const TILE_LINE_GAP: f32 = 0.0;
const TILE_PADDING_X: i8 = 8;
const TILE_PADDING_Y: i8 = 4;

pub(crate) fn padded_text_edit<'a>(text_edit: TextEdit<'a>) -> TextEdit<'a> {
    text_edit.margin(control_margin())
}

pub(crate) fn with_icon_button_padding<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = control_padding();
        ui.spacing_mut().interact_size = Vec2::splat(square_icon_button_side());
        add_contents(ui)
    })
    .inner
}

pub(crate) const fn square_icon_button_side() -> f32 {
    CONTROL_HEIGHT
}

pub(crate) const fn square_icon_button_size() -> [f32; 2] {
    [square_icon_button_side(), square_icon_button_side()]
}

pub(crate) fn checkbox(ui: &mut Ui, checked: &mut bool, text: impl Into<WidgetText>) -> Response {
    ui.add(IconCheckbox::new(checked, text))
}

pub(crate) fn checkbox_icon(checked: bool) -> &'static str {
    if checked {
        regular::CHECK_SQUARE
    } else {
        regular::SQUARE
    }
}

pub(crate) fn disable_tile_text_selection(ui: &mut Ui) {
    ui.style_mut().interaction.selectable_labels = false;
    ui.style_mut().interaction.multi_widget_text_select = false;
}

pub(crate) fn tile_list_content_width(ui: &Ui) -> f32 {
    (ui.available_width() - TILE_SCROLLBAR_GUTTER).max(0.0)
}

pub(crate) fn tile_scroll_bar_rect(ui: &Ui) -> egui::Rect {
    let mut rect = ui.available_rect_before_wrap();
    rect.max.x = (rect.max.x - TILE_SCROLLBAR_INSET).max(rect.min.x);
    rect
}

pub(crate) fn tile_inner_margin() -> egui::Margin {
    egui::Margin::symmetric(TILE_PADDING_X, TILE_PADDING_Y)
}

pub(crate) fn tile_inner_padding() -> egui::Vec2 {
    egui::vec2(TILE_PADDING_X as f32, TILE_PADDING_Y as f32)
}

pub(crate) fn tighten_tile_spacing(ui: &mut Ui) {
    ui.spacing_mut().item_spacing.y = TILE_LINE_GAP;
}

fn checkbox_icon_font(mut font: FontId) -> FontId {
    font.size *= CHECKBOX_ICON_SCALE;
    font
}

struct IconCheckbox<'a> {
    checked: &'a mut bool,
    text: WidgetText,
}

impl<'a> IconCheckbox<'a> {
    fn new(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            text: text.into(),
        }
    }
}

impl Widget for IconCheckbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { checked, text } = self;
        let spacing = ui.spacing();
        let icon_side = spacing.interact_size.y.max(spacing.icon_width);
        let icon_spacing = spacing.icon_spacing;
        let has_text = !text.is_empty();
        let trailing_padding = if has_text {
            CHECKBOX_TEXT_TRAILING_PADDING
        } else {
            0.0
        };

        let icon_font = checkbox_icon_font(TextStyle::Button.resolve(ui.style()));
        let icon_galley = ui.painter().layout_no_wrap(
            checkbox_icon(*checked).to_owned(),
            icon_font,
            egui::Color32::PLACEHOLDER,
        );
        let galley = if has_text {
            let wrap_width =
                (ui.available_width() - icon_side - icon_spacing - trailing_padding).max(0.0);
            Some(text.into_galley(ui, None, wrap_width, TextStyle::Button))
        } else {
            None
        };

        let text_size = galley.as_ref().map_or(Vec2::ZERO, |galley| galley.size());
        let desired_size = if has_text {
            vec2(
                (icon_side + icon_spacing + text_size.x + trailing_padding)
                    .max(spacing.interact_size.x),
                icon_side.max(text_size.y).max(spacing.interact_size.y),
            )
        } else {
            Vec2::splat(icon_side.max(spacing.interact_size.y))
        };

        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        if response.clicked() {
            *checked = !*checked;
            response.mark_changed();
        }
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::Checkbox,
                ui.is_enabled(),
                *checked,
                galley.as_ref().map_or("", |galley| galley.text()),
            )
        });

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            if response.hovered() || response.has_focus() || response.is_pointer_button_down_on() {
                ui.painter().rect(
                    rect,
                    visuals.corner_radius,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            let icon_color = if *checked {
                ui.visuals().selection.stroke.color
            } else {
                visuals.fg_stroke.color
            };
            let icon_pos = pos2(
                rect.left() + (icon_side - icon_galley.size().x) * 0.5,
                rect.center().y - icon_galley.size().y * 0.5,
            );
            ui.painter().galley(icon_pos, icon_galley, icon_color);

            if let Some(galley) = galley {
                let text_pos = pos2(
                    rect.left() + icon_side + icon_spacing,
                    rect.center().y - galley.size().y * 0.5,
                );
                ui.painter().galley(text_pos, galley, visuals.text_color());
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkbox_icons_match_styling_spec() {
        assert_eq!(checkbox_icon(false), regular::SQUARE);
        assert_eq!(checkbox_icon(true), regular::CHECK_SQUARE);
    }

    #[test]
    fn checkbox_icon_font_doubles_button_font_size() {
        let font = FontId::proportional(13.0);
        assert_eq!(checkbox_icon_font(font).size, 26.0);
    }

    #[test]
    fn checkbox_text_hover_trailing_padding_matches_spec() {
        assert_eq!(CHECKBOX_TEXT_TRAILING_PADDING, 8.0);
    }

    #[test]
    fn tile_rhythm_is_compact_and_symmetric() {
        assert_eq!(TILE_GAP, 4.0);
        assert_eq!(TILE_LINE_GAP, 0.0);
        assert_eq!(TILE_SCROLLBAR_GUTTER, 12.0);
        assert_eq!(TILE_SCROLLBAR_INSET, 4.0);
        assert_eq!(tile_inner_margin(), egui::Margin::symmetric(8, 4));
        assert_eq!(tile_inner_padding(), egui::vec2(8.0, 4.0));
    }

    #[test]
    fn icon_button_size_is_square_control_height() {
        let [width, height] = square_icon_button_size();
        assert_eq!(width, height);
        assert_eq!(width, CONTROL_HEIGHT);
    }
}
