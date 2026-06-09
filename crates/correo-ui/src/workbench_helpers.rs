use correo_core::{AppCommand, AppCommandSender};
use egui::{Frame, Stroke};

use crate::theme::ThemeTokens;

pub(crate) fn panel(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

pub(crate) fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
