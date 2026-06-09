use correo_core::{AppSnapshot, Diagnostic};
use egui::{RichText, ScrollArea, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

pub fn workspace(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, i18n: &I18n) {
    if snapshot.diagnostics.is_empty() {
        ui.label(RichText::new(i18n.text("diagnostics-empty")).color(tokens.text_secondary));
        return;
    }

    ScrollArea::vertical()
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for diagnostic in snapshot.diagnostics.iter().rev() {
                ui.add(
                    egui::Label::new(
                        RichText::new(log_line(diagnostic))
                            .monospace()
                            .color(tokens.severity(diagnostic.severity)),
                    )
                    .wrap(),
                );
            }
        });
}

fn log_line(diagnostic: &Diagnostic) -> String {
    let time = diagnostic.occurred_at.time();
    format!(
        "{:02}:{:02}:{:02} {:<7} {}",
        time.hour(),
        time.minute(),
        time.second(),
        diagnostic.severity.label().to_ascii_uppercase(),
        diagnostic.message
    )
}
