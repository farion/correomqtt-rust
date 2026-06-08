use correo_core::{AppCommand, AppCommandSender, AppSnapshot};
use egui::{Ui, Window};

pub(crate) fn unsubscribe_all_confirmation(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    commands: &AppCommandSender,
) {
    let Some(count) = snapshot
        .workbench
        .subscribe
        .unsubscribe_all_confirmation_count
    else {
        return;
    };

    let mut open = true;
    Window::new("Unsubscribe all")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ui.label(format!("Unsubscribe from {count} active subscriptions?"));
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    send(commands, AppCommand::CancelUnsubscribeAll);
                }
                if ui.button("Unsubscribe all").clicked() {
                    send(commands, AppCommand::ConfirmUnsubscribeAll);
                }
            });
        });

    if !open {
        send(commands, AppCommand::CancelUnsubscribeAll);
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
