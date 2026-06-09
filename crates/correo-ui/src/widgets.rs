use egui::{
    pos2, vec2, Response, Sense, TextEdit, TextStyle, Ui, Vec2, Widget, WidgetInfo, WidgetText,
    WidgetType,
};
use egui_phosphor::regular;

use crate::theme::control_margin;

pub(crate) fn padded_text_edit<'a>(text_edit: TextEdit<'a>) -> TextEdit<'a> {
    text_edit.margin(control_margin())
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

        let icon_galley = ui.painter().layout_no_wrap(
            checkbox_icon(*checked).to_owned(),
            TextStyle::Button.resolve(ui.style()),
            ui.visuals().text_color(),
        );
        let galley = if has_text {
            let wrap_width = (ui.available_width() - icon_side - icon_spacing).max(0.0);
            Some(text.into_galley(ui, None, wrap_width, TextStyle::Button))
        } else {
            None
        };

        let text_size = galley.as_ref().map_or(Vec2::ZERO, |galley| galley.size());
        let desired_size = if has_text {
            vec2(
                (icon_side + icon_spacing + text_size.x).max(spacing.interact_size.x),
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
}
