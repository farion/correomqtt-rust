use correo_core::{
    AppSnapshot, Diagnostic, MigrationDiagnosticCategory, MigrationFailureStage,
    MigrationRecoveryCompletion, MigrationRecoveryCounts, MigrationRecoveryDiagnostic,
    MigrationRecoveryFailure, MigrationRecoverySnapshot, MigrationRecoveryState,
    MigrationRecoveryWarning, MigrationRecoveryWarningKind, ThemeMode,
};

#[derive(Clone)]
pub(super) struct RecoveryCapture {
    pub(super) scenario: RecoveryScenario,
    pub(super) mode: ThemeMode,
    pub(super) size: (u32, u32),
    pub(super) file_name: String,
}

impl RecoveryCapture {
    fn new(scenario: RecoveryScenario, mode: ThemeMode, size: (u32, u32)) -> Self {
        Self {
            scenario,
            mode,
            size,
            file_name: format!(
                "correo-{}-{}-{}x{}.png",
                scenario.slug(),
                mode_slug(mode),
                size.0,
                size.1
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RecoveryScenario {
    Detection,
    PasswordNeeded,
    PasswordError,
    ReviewWarnings,
    PartialSuccess,
    FailureAfterWrite,
    RestoreConfirmation,
    RestoreFailure,
}

impl RecoveryScenario {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Detection => "migration recovery detection",
            Self::PasswordNeeded => "migration recovery password needed",
            Self::PasswordError => "migration recovery password error",
            Self::ReviewWarnings => "migration recovery review warnings",
            Self::PartialSuccess => "migration recovery partial success",
            Self::FailureAfterWrite => "migration recovery failure after write",
            Self::RestoreConfirmation => "migration recovery restore confirmation",
            Self::RestoreFailure => "migration recovery restore failure",
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Detection => "migration-recovery-detection",
            Self::PasswordNeeded => "migration-recovery-password-needed",
            Self::PasswordError => "migration-recovery-password-error",
            Self::ReviewWarnings => "migration-recovery-review-warnings",
            Self::PartialSuccess => "migration-recovery-partial-success",
            Self::FailureAfterWrite => "migration-recovery-failure-after-write",
            Self::RestoreConfirmation => "migration-recovery-restore-confirmation",
            Self::RestoreFailure => "migration-recovery-restore-failure",
        }
    }
}

pub(super) fn recovery_captures() -> Vec<RecoveryCapture> {
    let light_size = (900, 640);
    let mut captures = vec![
        RecoveryCapture::new(RecoveryScenario::Detection, ThemeMode::Light, light_size),
        RecoveryCapture::new(
            RecoveryScenario::PasswordNeeded,
            ThemeMode::Light,
            light_size,
        ),
        RecoveryCapture::new(
            RecoveryScenario::PasswordError,
            ThemeMode::Light,
            light_size,
        ),
        RecoveryCapture::new(
            RecoveryScenario::ReviewWarnings,
            ThemeMode::Light,
            light_size,
        ),
        RecoveryCapture::new(
            RecoveryScenario::PartialSuccess,
            ThemeMode::Light,
            light_size,
        ),
        RecoveryCapture::new(
            RecoveryScenario::FailureAfterWrite,
            ThemeMode::Light,
            light_size,
        ),
        RecoveryCapture::new(
            RecoveryScenario::RestoreConfirmation,
            ThemeMode::Light,
            light_size,
        ),
        RecoveryCapture::new(
            RecoveryScenario::RestoreFailure,
            ThemeMode::Light,
            light_size,
        ),
    ];
    captures.push(RecoveryCapture::new(
        RecoveryScenario::Detection,
        ThemeMode::Dark,
        (1024, 768),
    ));
    captures
}

pub(super) fn snapshot_for(capture: &RecoveryCapture) -> AppSnapshot {
    let mut snapshot = AppSnapshot::empty();
    snapshot.theme_mode = capture.mode;
    snapshot.migration_recovery = recovery_snapshot(capture.scenario);
    snapshot.diagnostics = vec![
        Diagnostic::warning(MigrationRecoverySnapshot::plugin_diagnostic()),
        Diagnostic::info("Migration recovery fixture uses synthetic data only."),
    ];
    snapshot
}

pub(super) fn mode_slug(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
        ThemeMode::System => "system",
    }
}

fn recovery_snapshot(scenario: RecoveryScenario) -> MigrationRecoverySnapshot {
    let mut recovery =
        MigrationRecoverySnapshot::detected("/home/user/.correomqtt/synthetic-beta-profile");
    recovery.counts = counts();
    if scenario != RecoveryScenario::Detection {
        recovery.backup_name = Some("migration-backup-1780952519".to_owned());
        recovery.backup_path_hint =
            Some("/home/user/.correomqtt-backups/migration-backup-1780952519".to_owned());
        recovery.backup_status = "Backup created: migration-backup-1780952519".to_owned();
    }
    match scenario {
        RecoveryScenario::Detection => {}
        RecoveryScenario::PasswordNeeded => {
            recovery.state = MigrationRecoveryState::NeedsPassword;
        }
        RecoveryScenario::PasswordError => {
            recovery.state = MigrationRecoveryState::NeedsPassword;
            recovery.password_error = Some(correo_core::MigrationPasswordError::WrongPassword);
        }
        RecoveryScenario::ReviewWarnings => review_warnings(&mut recovery),
        RecoveryScenario::PartialSuccess => {
            review_warnings(&mut recovery);
            recovery.state = MigrationRecoveryState::Complete;
            recovery.completion = Some(MigrationRecoveryCompletion::PartialSuccess);
        }
        RecoveryScenario::FailureAfterWrite => {
            recovery.state = MigrationRecoveryState::Failed;
            recovery.failure = Some(MigrationRecoveryFailure {
                stage: MigrationFailureStage::AfterWrite,
                message: "Synthetic config write failed after backup verification.".to_owned(),
            });
        }
        RecoveryScenario::RestoreConfirmation => {
            recovery.state = MigrationRecoveryState::RestoreConfirm;
        }
        RecoveryScenario::RestoreFailure => {
            recovery.state = MigrationRecoveryState::Failed;
            recovery.failure = Some(MigrationRecoveryFailure {
                stage: MigrationFailureStage::Restore,
                message: "Synthetic restore target changed before rollback.".to_owned(),
            });
        }
    }
    recovery
}

fn review_warnings(recovery: &mut MigrationRecoverySnapshot) {
    recovery.state = MigrationRecoveryState::Reviewing;
    recovery.secrets_skipped = true;
    recovery.counts.skipped_secrets = 2;
    recovery.warnings = vec![
        MigrationRecoveryWarning::new(
            MigrationRecoveryWarningKind::SecretSkipped,
            "2 connection secret(s) were skipped during migration.",
        ),
        MigrationRecoveryWarning::new(
            MigrationRecoveryWarningKind::UnsupportedLegacyField,
            "Unsupported legacy field ignored: config.settings.proxyPromptMode",
        ),
        MigrationRecoveryWarning::new(
            MigrationRecoveryWarningKind::JavaPluginStateIgnored,
            MigrationRecoverySnapshot::plugin_diagnostic(),
        ),
    ];
    recovery.counts.warnings = recovery.warnings.len();
    recovery
        .diagnostics
        .push(MigrationRecoveryDiagnostic::warning(
            MigrationDiagnosticCategory::Plugin,
            MigrationRecoverySnapshot::plugin_diagnostic(),
        ));
}

fn counts() -> MigrationRecoveryCounts {
    MigrationRecoveryCounts {
        connections: 3,
        histories: 6,
        scripts: 2,
        plugin_artifacts_ignored: 5,
        warnings: 3,
        skipped_secrets: 0,
    }
}
