use correo_core::ScriptSurfaceSnapshot;
use egui::{RichText, Ui};

use crate::theme::{ThemeTokens, CONTROL_HEIGHT};

const SCRIPTING_HELP_URL: &str = "https://github.com/EXXETA/correomqtt/wiki/scripting";
const SUMMARY_TEXT_SIZE: f32 = 12.0;
const SUMMARY_TOP_OFFSET: f32 = 4.0;

pub(super) fn execution_summary(ui: &mut Ui, scripts: &ScriptSurfaceSnapshot, tokens: ThemeTokens) {
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
    ui.allocate_ui_with_layout(
        egui::vec2(86.0, CONTROL_HEIGHT),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.add_space(SUMMARY_TOP_OFFSET);
            ui.spacing_mut().item_spacing.y = -2.0;
            ui.label(
                RichText::new(format!("{running} running"))
                    .size(SUMMARY_TEXT_SIZE)
                    .color(tokens.text_secondary),
            );
            ui.label(
                RichText::new(format!("{finished} finished"))
                    .size(SUMMARY_TEXT_SIZE)
                    .color(tokens.text_secondary),
            );
        },
    );
}

pub(super) fn help_link(ui: &mut Ui) {
    ui.hyperlink_to("Scripting help", SCRIPTING_HELP_URL);
}
