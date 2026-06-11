use correo_core::ScriptSurfaceSnapshot;
use egui::{RichText, Ui};

use crate::theme::ThemeTokens;

const SCRIPTING_HELP_URL: &str = "https://github.com/EXXETA/correomqtt/wiki/scripting";

pub(super) fn footer(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, tokens: ThemeTokens) {
    let running = scripts
        .executions
        .iter()
        .filter(|execution| !execution.status.is_terminal())
        .count();
    let finished = scripts
        .executions
        .iter()
        .filter(|execution| execution.status.is_terminal())
        .count();
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{running} running / {finished} finished"))
                .color(tokens.text_secondary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.hyperlink_to("Scripting help", SCRIPTING_HELP_URL);
        });
    });
}
