use egui::{
    pos2, vec2, FontId, Response, Sense, TextEdit, TextStyle, Ui, Vec2, Widget, WidgetInfo,
    WidgetText, WidgetType,
};
use egui_phosphor::regular;

use crate::theme::control_margin;

const CHECKBOX_ICON_SCALE: f32 = 2.0;
const CHECKBOX_TEXT_TRAILING_PADDING: f32 = 8.0;

pub(crate) fn padded_text_edit<'a>(text_edit: TextEdit<'a>) -> TextEdit<'a> {
    text_edit.margin(control_margin())
}

pub(crate) fn with_icon_button_padding<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = crate::theme::control_padding();
        add_contents(ui)
    })
    .inner
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
}
