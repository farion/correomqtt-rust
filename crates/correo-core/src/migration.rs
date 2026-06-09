use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use correo_diagnostics::redact_sensitive;
use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::{
    MigrationApplier, MigrationBackup, MigrationDiagnostics, MigrationPreview, MigrationWarning,
};
use thiserror::Error;

use crate::{
    startup_state_from_migration, AppEvent, MigrationApplyStage, MigrationDiagnosticCategory,
    MigrationFailureStage, MigrationRecoveryCompletion, MigrationRecoveryCounts,
    MigrationRecoveryDiagnostic, MigrationRecoveryEvent, MigrationRecoveryFailure,
    MigrationRecoveryRow, MigrationRecoverySnapshot, MigrationRecoveryTask,
    MigrationRecoveryWarning, MigrationRecoveryWarningKind, ThemeMode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationPersistenceCommand {
    Prepare { legacy_path: String },
    LoadReview,
    Apply { fallback_theme: ThemeMode },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MigrationDispatchError {
    #[error("migration persistence worker is stopped")]
    Stopped,
}

#[derive(Debug)]
pub struct MigrationPersistenceWorker {
    sender: Sender<MigrationPersistenceCommand>,
    events: Receiver<AppEvent>,
}

#[derive(Debug)]
struct PendingMigration {
    preview: MigrationPreview,
    backup: MigrationBackup,
}

impl MigrationPersistenceWorker {
    pub fn start(current_root: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let applier = MigrationApplier::new(current_root);

        std::thread::spawn(move || {
            let mut pending = None;
            while let Ok(command) = receiver.recv() {
                for event in handle_command(&applier, &mut pending, command) {
                    let _ = events_sender.send(event);
                }
            }
        });

        Self { sender, events }
    }

    pub fn dispatch(
        &self,
        command: MigrationPersistenceCommand,
    ) -> Result<(), MigrationDispatchError> {
        self.sender
            .send(command)
            .map_err(|_| MigrationDispatchError::Stopped)
    }

    pub fn try_recv_event(&self) -> Option<AppEvent> {
        self.events.try_recv().ok()
    }

    pub fn recv_event_timeout(&self, timeout: Duration) -> Option<AppEvent> {
        match self.events.recv_timeout(timeout) {
            Ok(event) => Some(event),
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => None,
        }
    }
}

fn handle_command(
    applier: &MigrationApplier,
    pending: &mut Option<PendingMigration>,
    command: MigrationPersistenceCommand,
) -> Vec<AppEvent> {
    match command {
        MigrationPersistenceCommand::Prepare { legacy_path } => {
            prepare(applier, pending, legacy_path)
        }
        MigrationPersistenceCommand::LoadReview => review_ready(pending.as_ref()),
        MigrationPersistenceCommand::Apply { fallback_theme } => {
            apply(applier, pending.take(), fallback_theme)
        }
    }
}

fn prepare(
    applier: &MigrationApplier,
    pending: &mut Option<PendingMigration>,
    legacy_path: String,
) -> Vec<AppEvent> {
    match prepare_pending(applier, legacy_path) {
        Ok(prepared) => {
            let backup_name = prepared.backup.id.clone();
            let backup_path_hint = prepared.backup.path.display().to_string();
            *pending = Some(prepared);
            vec![
                migration_event(MigrationRecoveryEvent::BackupCreated {
                    backup_name,
                    backup_path_hint,
                }),
                migration_event(MigrationRecoveryEvent::PasswordNeeded),
            ]
        }
        Err(message) => vec![migration_event(MigrationRecoveryEvent::BackupFailed {
            message,
        })],
    }
}

fn prepare_pending(
    applier: &MigrationApplier,
    legacy_path: String,
) -> Result<PendingMigration, String> {
    let profile = LegacyProfile::read_from(PathBuf::from(legacy_path)).map_err(|error| {
        format!("Legacy profile could not be read before migration backup: {error}")
    })?;
    let preview = MigrationPreview::from_legacy_profile(profile)
        .map_err(|error| format!("Legacy profile could not be prepared for migration: {error}"))?;
    let backup = applier
        .create_backup()
        .map_err(|error| format!("Migration backup could not be created: {error}"))?;
    Ok(PendingMigration { preview, backup })
}

fn review_ready(pending: Option<&PendingMigration>) -> Vec<AppEvent> {
    match pending {
        Some(pending) => vec![migration_event(MigrationRecoveryEvent::ReviewReady {
            counts: preview_counts(&pending.preview),
            rows: review_rows(&pending.preview),
            warnings: preview_warnings(&pending.preview.warnings),
        })],
        None => vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
            failure: MigrationRecoveryFailure {
                stage: MigrationFailureStage::BeforeWrite,
                message: "Migration preview is not prepared; start migration again.".to_owned(),
            },
        })],
    }
}

fn apply(
    applier: &MigrationApplier,
    pending: Option<PendingMigration>,
    fallback_theme: ThemeMode,
) -> Vec<AppEvent> {
    let Some(pending) = pending else {
        return vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
            failure: MigrationRecoveryFailure {
                stage: MigrationFailureStage::BeforeWrite,
                message: "Migration preview is not prepared; start migration again.".to_owned(),
            },
        })];
    };
    match applier.apply_preview_with_backup(&pending.preview, &pending.backup) {
        Ok(diagnostics) => {
            let completion = completion_from_diagnostics(&diagnostics);
            let state = startup_state_from_migration(pending.preview, fallback_theme);
            vec![
                migration_event(MigrationRecoveryEvent::ApplyStageChanged {
                    stage: MigrationApplyStage::ConfigWritten,
                }),
                AppEvent::MigrationApplied {
                    state: Box::new(state),
                    completion,
                    diagnostics: recovery_diagnostics(&diagnostics),
                },
            ]
        }
        Err(error) => vec![migration_event(MigrationRecoveryEvent::ApplyFailed {
            failure: MigrationRecoveryFailure {
                stage: MigrationFailureStage::AfterWrite,
                message: format!("Migration apply failed after backup: {error}"),
            },
        })],
    }
}

fn migration_event(event: MigrationRecoveryEvent) -> AppEvent {
    AppEvent::MigrationRecovery(event)
}

fn preview_counts(preview: &MigrationPreview) -> MigrationRecoveryCounts {
    MigrationRecoveryCounts {
        connections: preview.connections.len(),
        histories: preview.histories.connections.len(),
        scripts: preview.scripts.files.len(),
        plugin_artifacts_ignored: preview.plugin_state.ignored_legacy_paths.len(),
        warnings: preview.warnings.len(),
        skipped_secrets: 0,
    }
}

fn review_rows(preview: &MigrationPreview) -> Vec<MigrationRecoveryRow> {
    let mut rows = MigrationRecoverySnapshot::review_rows();
    for row in &mut rows {
        row.detail = match row.task {
            MigrationRecoveryTask::Connections => {
                format!("{} profile(s) ready to migrate.", preview.connections.len())
            }
            MigrationRecoveryTask::Histories => {
                format!(
                    "{} connection history set(s) ready.",
                    preview.histories.connections.len()
                )
            }
            MigrationRecoveryTask::Scripts => {
                format!(
                    "{} script file(s) and metadata ready.",
                    preview.scripts.files.len()
                )
            }
            MigrationRecoveryTask::Plugins => {
                MigrationRecoverySnapshot::plugin_replacement_body().to_owned()
            }
            _ => row.detail.clone(),
        };
    }
    rows
}

fn preview_warnings(warnings: &[MigrationWarning]) -> Vec<MigrationRecoveryWarning> {
    warnings
        .iter()
        .map(|warning| {
            MigrationRecoveryWarning::new(
                warning_kind(warning.code),
                redact_sensitive(&warning.message),
            )
        })
        .collect()
}

fn warning_kind(code: &str) -> MigrationRecoveryWarningKind {
    match code {
        "unsupported_legacy_field" => MigrationRecoveryWarningKind::UnsupportedLegacyField,
        "legacy_hooks_not_mapped" => MigrationRecoveryWarningKind::HookConfigIgnored,
        "legacy_plugins_ignored" => MigrationRecoveryWarningKind::JavaPluginStateIgnored,
        _ => MigrationRecoveryWarningKind::ConnectionNeedsReview,
    }
}

fn completion_from_diagnostics(diagnostics: &MigrationDiagnostics) -> MigrationRecoveryCompletion {
    if diagnostics.warnings.is_empty() && diagnostics.unmapped_fields.is_empty() {
        MigrationRecoveryCompletion::Success
    } else {
        MigrationRecoveryCompletion::PartialSuccess
    }
}

fn recovery_diagnostics(diagnostics: &MigrationDiagnostics) -> Vec<MigrationRecoveryDiagnostic> {
    let mut recovery = Vec::new();
    recovery.extend(diagnostics.warnings.iter().map(|warning| {
        MigrationRecoveryDiagnostic::warning(
            MigrationDiagnosticCategory::Migration,
            format!("{}: {}", warning.code, warning.message),
        )
    }));
    recovery.extend(diagnostics.unmapped_fields.iter().map(|field| {
        MigrationRecoveryDiagnostic::warning(
            MigrationDiagnosticCategory::LegacyField,
            format!("Unsupported legacy field ignored: {field}"),
        )
    }));
    recovery.extend(diagnostics.recovery_steps.iter().map(|step| {
        MigrationRecoveryDiagnostic::info(MigrationDiagnosticCategory::Backup, step.clone())
    }));
    recovery
}
