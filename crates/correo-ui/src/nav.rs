use correo_core::{AppCommand, AppCommandSender, Workspace};
use egui::{Button, CornerRadius, RichText, Stroke, Ui};

use crate::theme::ThemeTokens;

pub fn rail(ui: &mut Ui, active: Workspace, tokens: ThemeTokens, commands: &AppCommandSender) {
    ui.vertical_centered(|ui| {
        ui.add_space(4.0);
        for workspace in Workspace::ALL {
            let selected = workspace == active;
            let fill = if selected {
                tokens.accent_selected_bg
            } else {
                tokens.panel_bg
            };
            let response = ui
                .add_sized(
                    [32.0, 32.0],
                    Button::new(RichText::new(workspace.rail_label()).strong())
                        .fill(fill)
                        .stroke(Stroke::new(1.0, tokens.border))
                        .corner_radius(CornerRadius::same(4)),
                )
                .on_hover_text(workspace.label());
            if selected {
                let rect = response.rect;
                let accent =
                    egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height()));
                ui.painter()
                    .rect_filled(accent, CornerRadius::same(1), tokens.accent);
            }
            if response.clicked() {
                let _ = commands.send(AppCommand::SelectWorkspace(workspace));
            }
            ui.add_space(4.0);
        }
    });
}
