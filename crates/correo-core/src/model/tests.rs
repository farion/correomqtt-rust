use std::path::{Path, PathBuf};

use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::MigrationPreview;

use super::{AppEvent, AppModel};
use crate::{
    startup_state_from_migration, AppCommand, ConnectionBadge, ConnectionState, ConnectionSurface,
    Diagnostic, GlobalSettingField, GlobalSettingFlag, KeyringState, LegacyMigrationStatus,
    MigrationFailureStage, MigrationRecoveryCommand, MigrationRecoveryCompletion,
    MigrationRecoveryCounts, MigrationRecoveryEvent, MigrationRecoveryFailure,
    MigrationRecoverySnapshot, MigrationRecoveryState, MqttCommand, StartupState, ThemeMode,
    TransferSection, Workspace,
};

fn storage_fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../correo-storage/tests/fixtures")
        .join(path)
}

#[test]
fn applies_connection_events_to_snapshot() {
    let mut model = AppModel::default();
    let connection_id = model.snapshot().connections[1].id;

    model.apply_event(AppEvent::ConnectionOpened { connection_id });

    assert_eq!(model.snapshot().active_connection, Some(connection_id));
    assert_eq!(model.snapshot().connection_count, 4);
    assert_eq!(
        model.snapshot().connections[1].state,
        ConnectionState::Connected
    );

    model.apply_event(AppEvent::ConnectionClosed { connection_id });

    assert_eq!(model.snapshot().active_connection, None);
    assert_eq!(
        model.snapshot().connections[1].state,
        ConnectionState::Disconnected
    );
}

#[test]
fn applies_migration_recovery_events_to_snapshot() {
    let mut model = AppModel::empty();

    model.apply_event(AppEvent::MigrationRecovery(
        MigrationRecoveryEvent::LegacyDetected {
            legacy_path: "/tmp/CorreoMqtt".to_owned(),
            counts: MigrationRecoveryCounts {
                connections: 2,
                histories: 3,
                scripts: 1,
                plugin_artifacts_ignored: 4,
                warnings: 0,
                skipped_secrets: 0,
            },
            warnings: Vec::new(),
        },
    ));

    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::NeedsDecision
    );
    assert_eq!(
        model.snapshot().migration_recovery.legacy_path.as_deref(),
        Some("/tmp/CorreoMqtt")
    );
    assert_eq!(model.snapshot().migration_recovery.counts.connections, 2);
    assert_eq!(
        model.snapshot().global_settings.legacy_migration.status,
        LegacyMigrationStatus::Detected
    );
}

#[test]
fn publish_command_sets_feedback_without_recording_success_history() {
    let mut model = AppModel::default();
    let initial_count = model.snapshot().workbench.publish.history.len();

    model.apply_command(crate::AppCommand::Publish);

    assert_eq!(
        model.snapshot().workbench.publish.history.len(),
        initial_count
    );
    assert!(model
        .snapshot()
        .workbench
        .publish
        .feedback
        .as_ref()
        .is_some_and(|feedback| feedback.message.contains("queued")));
}

#[test]
fn topic_updates_refresh_core_validation_state() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::UpdatePublishTopic("alerts/#".to_owned()));
    assert!(!model.snapshot().workbench.publish.valid);
    assert!(model.snapshot().workbench.publish.validation[0].contains("wildcards"));

    model.apply_command(AppCommand::UpdateSubscribeTopic("alerts/#".to_owned()));
    assert!(model.snapshot().workbench.subscribe.valid);
    assert_eq!(
        model.snapshot().workbench.subscribe.validation,
        ["Topic filter is valid"]
    );
}

#[test]
fn unsubscribe_all_requires_confirmation_and_cancel_suppresses_dispatch() {
    let mut model = AppModel::default();

    let direct = model
        .mqtt_commands_for_app_command(&AppCommand::UnsubscribeAll)
        .expect("unsubscribe all request should build safely");
    assert!(direct.is_empty());

    model.apply_command(AppCommand::UnsubscribeAll);
    assert_eq!(
        model
            .snapshot()
            .workbench
            .subscribe
            .unsubscribe_all_confirmation_count,
        Some(3)
    );

    let confirmed = model
        .mqtt_commands_for_app_command(&AppCommand::ConfirmUnsubscribeAll)
        .expect("confirmed unsubscribe all should build safely");
    assert_eq!(confirmed.len(), 3);
    assert!(confirmed
        .iter()
        .all(|command| matches!(command, MqttCommand::Unsubscribe { .. })));

    model.apply_command(AppCommand::CancelUnsubscribeAll);
    assert_eq!(
        model
            .snapshot()
            .workbench
            .subscribe
            .unsubscribe_all_confirmation_count,
        None
    );
    let after_cancel = model
        .mqtt_commands_for_app_command(&AppCommand::ConfirmUnsubscribeAll)
        .expect("cancelled unsubscribe all should build safely");
    assert!(after_cancel.is_empty());
    assert_eq!(model.snapshot().workbench.subscribe.subscriptions.len(), 3);
}

#[test]
fn global_settings_commands_track_dirty_save_and_discard() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::UpdateGlobalSetting {
        field: GlobalSettingField::Language,
        value: "de_DE".to_owned(),
    });
    model.apply_command(AppCommand::SetGlobalSettingFlag {
        flag: GlobalSettingFlag::UseRegexForSearch,
        enabled: true,
    });
    model.apply_command(AppCommand::SetThemeMode(ThemeMode::Dark));

    assert!(model.snapshot().global_settings.dirty);
    assert_eq!(model.snapshot().global_settings.language, "de_DE");
    assert_eq!(model.snapshot().theme_mode, ThemeMode::Dark);

    model.apply_command(AppCommand::SaveGlobalSettings);
    assert!(!model.snapshot().global_settings.dirty);

    model.apply_command(AppCommand::UpdateGlobalSetting {
        field: GlobalSettingField::KeyringBackend,
        value: "LibSecret".to_owned(),
    });
    model.apply_command(AppCommand::SetThemeMode(ThemeMode::Light));
    model.apply_command(AppCommand::DiscardGlobalSettings);

    assert_eq!(model.snapshot().global_settings.language, "de_DE");
    assert_eq!(model.snapshot().global_settings.keyring_backend, "os");
    assert!(model.snapshot().global_settings.search_use_regex);
    assert_eq!(model.snapshot().theme_mode, ThemeMode::Dark);
    assert!(!model.snapshot().global_settings.dirty);
}

#[test]
fn connect_command_queues_service_work_without_marking_open() {
    let mut model = AppModel::default();
    let connection_id = model.snapshot().connections[2].id;

    model.apply_command(AppCommand::Connect(connection_id));

    assert_eq!(
        model.snapshot().active_connection,
        model.snapshot().connections[0].id.into()
    );
    assert_eq!(
        model.snapshot().connections[2].state,
        ConnectionState::Connecting
    );
}

#[test]
fn add_connection_opens_settings_draft_and_save_adds_profile() {
    let mut model = AppModel::empty();

    model.apply_command(AppCommand::AddConnection);

    assert_eq!(model.snapshot().active_workspace, Workspace::Connections);
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Settings
    );
    assert_eq!(model.snapshot().selected_connection, None);
    assert_eq!(
        model.snapshot().connection_settings.profile_name,
        "New connection"
    );
    assert!(model.snapshot().connection_settings.dirty);
    assert!(!model.snapshot().connection_settings.valid);
    assert!(model
        .snapshot()
        .connection_settings
        .validation_errors
        .iter()
        .any(|error| error == "Host is required"));

    model.apply_command(AppCommand::UpdateConnectionSetting {
        field: crate::ConnectionSettingField::Host,
        value: "localhost".to_owned(),
    });
    assert!(model.snapshot().connection_settings.valid);

    model.apply_command(AppCommand::SaveConnectionSettings);

    let connection_id = model
        .snapshot()
        .selected_connection
        .expect("saved draft should become selected");
    let connection = model
        .snapshot()
        .selected_connection()
        .expect("saved draft should be visible in launcher");
    assert_eq!(model.snapshot().connection_count, 1);
    assert_eq!(connection.name, "New connection");
    assert_eq!(connection.endpoint, "localhost:1883");
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert!(!model.snapshot().connection_settings.dirty);
    assert!(model
        .mqtt_commands_for_app_command(&AppCommand::Connect(connection_id))
        .expect("new profile should build connect command")
        .iter()
        .any(|command| matches!(command, MqttCommand::Connect { .. })));
}

#[test]
fn transfer_commands_focus_the_requested_section() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::ExportConnections);
    assert_eq!(model.snapshot().active_workspace, Workspace::Connections);
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Transfer
    );
    assert_eq!(
        model.snapshot().transfer.active_section,
        TransferSection::Export
    );

    model.apply_command(AppCommand::ImportMessages);
    assert_eq!(model.snapshot().active_workspace, Workspace::Connections);
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert!(model.snapshot().workbench.publish.feedback.is_some());

    model.apply_command(AppCommand::ImportConnections);
    assert_eq!(model.snapshot().active_workspace, Workspace::Connections);
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Transfer
    );
    assert_eq!(
        model.snapshot().transfer.active_section,
        TransferSection::Import
    );
}

#[test]
fn diagnostic_events_are_redacted_before_snapshot_exposure() {
    let mut model = AppModel::empty();

    model.apply_event(AppEvent::DiagnosticRaised(Diagnostic::error(
        "mqtt auth failed password=super-secret token:abcd",
    )));

    let message = &model.snapshot().diagnostics[0].message;
    assert!(message.contains("[REDACTED]"));
    assert!(!message.contains("super-secret"));
    assert!(!message.contains("abcd"));
}

#[test]
fn migrated_fixture_opens_workbench_and_settings_without_secret_values() {
    let profile = LegacyProfile::read_from(storage_fixture("legacy_profile")).unwrap();
    let preview = MigrationPreview::from_legacy_profile(profile).unwrap();
    let state = startup_state_from_migration(preview, ThemeMode::Dark);
    let mut model = AppModel::with_startup_state(state);

    assert_eq!(model.snapshot().theme_mode, ThemeMode::Light);
    assert_eq!(model.snapshot().connection_count, 2);

    let first = &model.snapshot().connections[0];
    assert_eq!(first.name, "Synthetic Local Broker");
    assert_eq!(first.endpoint, "localhost:1883");
    assert_eq!(first.mqtt_version, "MQTT v5");
    assert!(first.badges.contains(&ConnectionBadge::Credentials));
    assert!(first.badges.contains(&ConnectionBadge::Proxy));
    assert!(first.badges.contains(&ConnectionBadge::Lwt));
    assert_eq!(first.recent_subscriptions, 2);
    assert_eq!(first.recent_messages, 1);

    assert_eq!(
        model.snapshot().workbench.publish.topic_history,
        ["sensors/temperature", "alerts/status"]
    );
    assert_eq!(
        model.snapshot().workbench.subscribe.topic_history,
        ["sensors/#", "alerts/status"]
    );
    assert!(model
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic
            .message
            .contains("Unsupported legacy field ignored")));

    let first_id = model.snapshot().connections[0].id;
    model.apply_command(AppCommand::OpenConnectionSettings(first_id));
    let settings = &model.snapshot().connection_settings;
    assert_eq!(settings.profile_name, "Synthetic Local Broker");
    assert_eq!(settings.host, "localhost");
    assert_eq!(settings.port, "1883");
    assert_eq!(settings.mqtt_version, "MQTT v5");
    assert_eq!(settings.proxy_mode, "SSH");
    assert!(settings.lwt_enabled);
    assert_eq!(settings.lwt_topic, "status/local-broker-01");
    assert_eq!(settings.keyring_state, KeyringState::Available);

    let exposed = format!("{:?}", model.snapshot());
    assert!(!exposed.contains("synthetic-mqtt-password"));
    assert!(!exposed.contains("synthetic-ssh-password"));
    assert!(!exposed.contains("synthetic-keystore-password"));
}

#[test]
fn first_run_without_legacy_data_keeps_connections_workspace_available() {
    let state = StartupState::empty(
        ThemeMode::Light,
        Diagnostic::info("No existing CorreoMQTT config found; empty workspace ready."),
    );
    let model = AppModel::with_startup_state(state);

    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::NotDetected
    );
    assert!(!model.snapshot().migration_recovery.blocks_normal_shell());
}

#[test]
fn legacy_detection_blocks_launcher_until_user_choice() {
    let state = StartupState::legacy_migration_detected(
        ThemeMode::Dark,
        "/home/user/.correomqtt".to_owned(),
    );
    let mut model = AppModel::with_startup_state(state);

    assert!(model.snapshot().migration_recovery.blocks_normal_shell());
    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::NeedsDecision
    );

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::StartEmptyProfile,
    ));
    assert!(
        model
            .snapshot()
            .migration_recovery
            .empty_profile_confirmation_open
    );

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::ConfirmStartEmptyProfile,
    ));
    assert!(!model.snapshot().migration_recovery.blocks_normal_shell());
    assert_eq!(
        model.snapshot().global_settings.legacy_migration.status,
        LegacyMigrationStatus::Skipped
    );
}

#[test]
fn failure_after_write_offers_restore_and_settings_data_status() {
    let mut recovery = MigrationRecoverySnapshot::detected("/home/user/.correomqtt");
    recovery.backup_name = Some("migration-backup-123".to_owned());
    recovery.backup_path_hint = Some("/tmp/backups/migration-backup-123".to_owned());
    let mut model = AppModel::with_snapshot(crate::AppSnapshot {
        migration_recovery: recovery,
        ..crate::AppSnapshot::empty()
    });

    model.apply_event(AppEvent::MigrationRecovery(
        MigrationRecoveryEvent::ApplyFailed {
            failure: MigrationRecoveryFailure {
                stage: MigrationFailureStage::AfterWrite,
                message: "config write failed after backup".to_owned(),
            },
        },
    ));

    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::Failed
    );
    assert_eq!(
        model.snapshot().global_settings.legacy_migration.status,
        LegacyMigrationStatus::Failed
    );
    assert!(
        model
            .snapshot()
            .global_settings
            .legacy_migration
            .restore_available
    );

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::RequestRestoreBackup,
    ));
    assert_eq!(
        model.snapshot().migration_recovery.state,
        MigrationRecoveryState::RestoreConfirm
    );
}

#[test]
fn partial_success_keeps_recovery_complete_until_connections_opened() {
    let mut model = AppModel::with_snapshot(crate::AppSnapshot {
        migration_recovery: MigrationRecoverySnapshot::detected("/home/user/.correomqtt"),
        ..crate::AppSnapshot::empty()
    });

    model.apply_event(AppEvent::MigrationRecovery(
        MigrationRecoveryEvent::ApplyCompleted {
            completion: MigrationRecoveryCompletion::PartialSuccess,
            diagnostics: Vec::new(),
        },
    ));

    assert!(model.snapshot().migration_recovery.blocks_normal_shell());
    assert_eq!(
        model.snapshot().global_settings.legacy_migration.status,
        LegacyMigrationStatus::PartialSuccess
    );

    model.apply_command(AppCommand::MigrationRecovery(
        MigrationRecoveryCommand::OpenConnections,
    ));
    assert!(!model.snapshot().migration_recovery.blocks_normal_shell());
}
