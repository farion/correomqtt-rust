use correo_core::{AppCommandSender, AppSnapshot};
use egui::Ui;

use crate::{
    theme::ThemeTokens, workbench_dialogs, workbench_header, workbench_layout, workbench_messages,
    workbench_publish, workbench_subscribe,
};

pub fn show(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, commands: &AppCommandSender) {
    if snapshot.selected_connection().is_none() {
        ui.label("No connection available");
        return;
    }

    workbench_header::connection_header(ui, snapshot, tokens, commands);
    ui.add_space(6.0);
    workbench_layout::show(
        ui,
        tokens,
        snapshot.workbench.narrow_tab,
        commands,
        |ui| workbench_publish::editor(ui, snapshot, tokens, commands),
        |ui| workbench_subscribe::editor(ui, snapshot, tokens, commands),
        |ui| workbench_publish::outgoing_messages(ui, snapshot, tokens, commands),
        |ui| workbench_subscribe::incoming_messages(ui, snapshot, tokens, commands),
    );
    workbench_messages::show(ui.ctx(), snapshot, tokens, commands);
    workbench_dialogs::unsubscribe_all_confirmation(ui, snapshot, commands);
}
