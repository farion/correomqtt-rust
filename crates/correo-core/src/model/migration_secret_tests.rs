use super::{AppEvent, AppModel};
use crate::{AppCommand, MigrationRecoveryCommand, MigrationRecoveryEvent, MigrationRecoveryState};

#[test]
fn skipped_secret_review_does_not_create_connection_disabled_copy() {
    let mut model = AppModel::empty();
    model.apply_event(AppEvent::MigrationRecovery(
        MigrationRecoveryEvent::LegacyDetected {
            legacy_path: "/home/user/.correomqtt".to_owned(),
            counts: Default::default(),
            warnings: Vec::new(),
        },
    ));
    let diagnostic_count = model.snapshot().migration_recovery.diagnostics.len();

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::SkipSecrets,
    ));

    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::Reviewing
    );
    assert!(model.snapshot().migration_recovery.secrets_skipped);
    assert_eq!(
        model.snapshot().migration_recovery.diagnostics.len(),
        diagnostic_count
    );
}
