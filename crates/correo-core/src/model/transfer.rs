use crate::{
    AppModel, Diagnostic, ExportPasswordConfirmation, ExportPathState, ImportPasswordState,
    TransferFeedback, TransferOutcome, TransferSection, TransferStep, Workspace,
};

impl AppModel {
    pub(super) fn import_connections(&mut self) {
        self.snapshot.active_workspace = Workspace::ImportExport;
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.active_step = TransferStep::ChooseFile;
        self.push_diagnostic(Diagnostic::info(
            "Connection import command queued for a .cqc file.",
        ));
    }

    pub(super) fn open_connection_export(&mut self) {
        self.snapshot.active_workspace = Workspace::ImportExport;
        self.snapshot.transfer.active_section = TransferSection::Export;
        self.snapshot.transfer.export.feedback = Some(TransferFeedback::info(
            "Connection export is ready. Plain exports omit sensitive auth values.",
        ));
        self.push_diagnostic(Diagnostic::info("Connection export command queued."));
    }

    pub(super) fn choose_connection_import_file(&mut self) {
        self.snapshot.active_workspace = Workspace::ImportExport;
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.active_step = if self.snapshot.transfer.import.encrypted {
            TransferStep::Password
        } else {
            TransferStep::Review
        };
        self.snapshot.transfer.import.feedback = Some(TransferFeedback::info(
            "Selected .cqc file is ready for review.",
        ));
    }

    pub(super) fn submit_connection_import_password(&mut self) {
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.import.password_state = ImportPasswordState::Accepted;
        self.snapshot.transfer.import.feedback =
            Some(TransferFeedback::info("Encrypted .cqc file unlocked."));
        self.snapshot.transfer.active_step = TransferStep::Review;
    }

    pub(super) fn clear_connection_import_error(&mut self) {
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.import.password_state = ImportPasswordState::Needed;
        self.snapshot.transfer.import.feedback = None;
        self.snapshot.transfer.active_step = TransferStep::Password;
    }

    pub(super) fn select_import_step(&mut self, step: TransferStep) {
        self.snapshot.transfer.active_section = TransferSection::Import;
        self.snapshot.transfer.active_step = step;
    }

    pub(super) fn select_connection_import_row(&mut self, row_id: &str, selected: bool) {
        if let Some(row) = self
            .snapshot
            .transfer
            .import
            .rows
            .iter_mut()
            .find(|row| row.id == row_id)
        {
            row.selected = selected;
        }
        self.snapshot.transfer.selected_connections =
            self.snapshot.transfer.import.selected_count();
    }

    pub(super) fn start_connection_import(&mut self) {
        self.snapshot.transfer.active_section = TransferSection::Import;
        let selected = self.snapshot.transfer.import.selected_count();
        self.snapshot.transfer.active_step = TransferStep::Complete;
        if selected == 0 {
            self.snapshot.transfer.import.outcome = Some(TransferOutcome::failure(
                "Import failed",
                "Select at least one connection before importing.",
            ));
            self.snapshot.transfer.import.feedback =
                Some(TransferFeedback::warning("No connections selected."));
        } else {
            self.snapshot.transfer.import.outcome = Some(TransferOutcome::success(
                "Import complete",
                format!("{selected} connection profiles imported; secrets stay in keyring."),
            ));
            self.snapshot.transfer.import.feedback = None;
        }
    }

    pub(super) fn select_connection_export_row(&mut self, row_id: &str, selected: bool) {
        if let Some(row) = self
            .snapshot
            .transfer
            .export
            .rows
            .iter_mut()
            .find(|row| row.id == row_id)
        {
            row.selected = selected;
        }
        self.snapshot.transfer.selected_connections =
            self.snapshot.transfer.export.selected_count();
    }

    pub(super) fn set_connection_export_encrypted(&mut self, encrypted: bool) {
        self.snapshot.transfer.active_section = TransferSection::Export;
        self.snapshot.transfer.export.encrypted = encrypted;
        self.snapshot.transfer.encrypted_export = encrypted;
        self.snapshot.transfer.export.password_confirmation = if encrypted {
            ExportPasswordConfirmation::Needed
        } else {
            ExportPasswordConfirmation::NotRequired
        };
        self.snapshot.transfer.export.feedback = Some(if encrypted {
            TransferFeedback::info("Encrypted export will require password confirmation.")
        } else {
            TransferFeedback::warning("Plain export excludes sensitive auth values.")
        });
    }

    pub(super) fn update_connection_export_path(&mut self, path: String) {
        self.snapshot.transfer.active_section = TransferSection::Export;
        self.snapshot.transfer.export.path_state = export_path_state(&path);
        self.snapshot.transfer.export.output_path = path;
        self.snapshot.transfer.export.feedback =
            path_feedback(self.snapshot.transfer.export.path_state);
    }

    pub(super) fn start_connection_export(&mut self) {
        self.snapshot.transfer.active_section = TransferSection::Export;
        let selected = self.snapshot.transfer.export.selected_count();
        let path_state = self.snapshot.transfer.export.path_state;
        if selected == 0 {
            self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "Select at least one connection before exporting.",
            ));
            return;
        }
        if path_state == ExportPathState::InvalidPath {
            self.snapshot.transfer.export.outcome = Some(TransferOutcome::failure(
                "Export failed",
                "Choose a writable target path before exporting.",
            ));
            return;
        }
        let detail = if self.snapshot.transfer.export.encrypted {
            format!("{selected} encrypted connection profiles exported.")
        } else {
            format!("{selected} plain profiles exported without sensitive auth values.")
        };
        self.snapshot.transfer.export.outcome =
            Some(TransferOutcome::success("Export complete", detail));
    }

    pub(super) fn import_messages(&mut self) {
        self.snapshot.active_workspace = Workspace::ImportExport;
        self.snapshot.transfer.active_section = TransferSection::Messages;
        self.snapshot.transfer.messages.feedback = Some(TransferFeedback::info(
            "Message import command queued for a JSON message archive.",
        ));
        self.push_diagnostic(Diagnostic::info("Message import command queued."));
    }

    pub(super) fn export_messages(&mut self) {
        self.snapshot.active_workspace = Workspace::ImportExport;
        self.snapshot.transfer.active_section = TransferSection::Messages;
        let count = self.snapshot.transfer.messages.selected_messages;
        self.snapshot.transfer.messages.outcome = Some(TransferOutcome::success(
            "Message export ready",
            format!("{count} retained message snapshots queued for export."),
        ));
        self.push_diagnostic(Diagnostic::info("Message export command queued."));
    }
}

fn export_path_state(path: &str) -> ExportPathState {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        ExportPathState::InvalidPath
    } else if !trimmed.ends_with(".cqc") {
        ExportPathState::MissingExtension
    } else {
        ExportPathState::Ready
    }
}

fn path_feedback(state: ExportPathState) -> Option<TransferFeedback> {
    match state {
        ExportPathState::Ready => None,
        ExportPathState::MissingExtension => Some(TransferFeedback::warning(
            "The target file should end with .cqc.",
        )),
        ExportPathState::InvalidPath => Some(TransferFeedback::error(
            "Choose a writable target path before exporting.",
        )),
    }
}
