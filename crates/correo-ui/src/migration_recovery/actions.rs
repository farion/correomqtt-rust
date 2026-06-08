use correo_core::{
    AppCommand, AppCommandSender, MigrationRecoveryCommand, MigrationRecoverySnapshot,
    MigrationRecoveryState,
};
use egui::{Button, Ui};

pub(super) fn action_bar(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    commands: &AppCommandSender,
) {
    ui.horizontal_wrapped(|ui| match snapshot.state {
        MigrationRecoveryState::NeedsDecision => {
            button(
                ui,
                "Start empty beta profile",
                commands,
                MigrationRecoveryCommand::StartEmptyProfile,
            );
            button(
                ui,
                "Create backup and migrate",
                commands,
                MigrationRecoveryCommand::ChooseMigrate,
            );
        }
        MigrationRecoveryState::CreatingBackup
        | MigrationRecoveryState::Applying
        | MigrationRecoveryState::Restoring => {
            ui.add_enabled(false, Button::new("Migration in progress"));
        }
        MigrationRecoveryState::NeedsPassword => {
            button(ui, "Back", commands, MigrationRecoveryCommand::Retry);
            button(
                ui,
                "Skip secrets for now",
                commands,
                MigrationRecoveryCommand::SkipSecrets,
            );
            button(
                ui,
                "Unlock secrets",
                commands,
                MigrationRecoveryCommand::SubmitPassword,
            );
        }
        MigrationRecoveryState::Reviewing => {
            button(ui, "Back", commands, MigrationRecoveryCommand::Retry);
            if ui
                .add_enabled(
                    snapshot.selected_count() > 0,
                    Button::new("Migrate selected").min_size(egui::vec2(190.0, 28.0)),
                )
                .clicked()
            {
                send(commands, MigrationRecoveryCommand::ApplyMigration);
            }
        }
        MigrationRecoveryState::Complete => {
            button(
                ui,
                "View diagnostics",
                commands,
                MigrationRecoveryCommand::OpenDiagnostics,
            );
            button(
                ui,
                "Open Connections",
                commands,
                MigrationRecoveryCommand::OpenConnections,
            );
        }
        MigrationRecoveryState::Failed => {
            button(
                ui,
                "Start empty beta profile",
                commands,
                MigrationRecoveryCommand::StartEmptyProfile,
            );
            button(ui, "Retry", commands, MigrationRecoveryCommand::Retry);
            if snapshot.backup_name.is_some() {
                button(
                    ui,
                    "Restore backup...",
                    commands,
                    MigrationRecoveryCommand::RequestRestoreBackup,
                );
            }
            button(
                ui,
                "View diagnostics",
                commands,
                MigrationRecoveryCommand::OpenDiagnostics,
            );
        }
        MigrationRecoveryState::RestoreConfirm => {
            button(
                ui,
                "Cancel",
                commands,
                MigrationRecoveryCommand::CancelRestoreBackup,
            );
            button(
                ui,
                "Restore backup",
                commands,
                MigrationRecoveryCommand::ConfirmRestoreBackup,
            );
        }
        MigrationRecoveryState::NotDetected | MigrationRecoveryState::Detecting => {}
    });
}

pub(super) fn handle_keyboard(
    ui: &mut Ui,
    snapshot: &MigrationRecoverySnapshot,
    commands: &AppCommandSender,
) {
    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        if snapshot.empty_profile_confirmation_open {
            send(commands, MigrationRecoveryCommand::CancelEmptyProfile);
        } else if snapshot.state == MigrationRecoveryState::RestoreConfirm {
            send(commands, MigrationRecoveryCommand::CancelRestoreBackup);
        }
    }
    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        if let Some(command) = primary_command(snapshot) {
            send(commands, command);
        }
    }
}

pub(super) fn button(
    ui: &mut Ui,
    label: &str,
    commands: &AppCommandSender,
    command: MigrationRecoveryCommand,
) {
    if ui.add_sized([190.0, 28.0], Button::new(label)).clicked() {
        send(commands, command);
    }
}

pub(super) fn send(commands: &AppCommandSender, command: MigrationRecoveryCommand) {
    let _ = commands.send(AppCommand::MigrationRecovery(command));
}

fn primary_command(snapshot: &MigrationRecoverySnapshot) -> Option<MigrationRecoveryCommand> {
    match snapshot.state {
        MigrationRecoveryState::NeedsDecision => Some(MigrationRecoveryCommand::ChooseMigrate),
        MigrationRecoveryState::NeedsPassword => Some(MigrationRecoveryCommand::SubmitPassword),
        MigrationRecoveryState::Reviewing if snapshot.selected_count() > 0 => {
            Some(MigrationRecoveryCommand::ApplyMigration)
        }
        MigrationRecoveryState::Complete => Some(MigrationRecoveryCommand::OpenConnections),
        MigrationRecoveryState::RestoreConfirm => {
            Some(MigrationRecoveryCommand::ConfirmRestoreBackup)
        }
        _ => None,
    }
}
