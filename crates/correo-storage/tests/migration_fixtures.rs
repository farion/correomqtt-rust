use correo_storage::current::{
    redact_script_log_text, ConfigStore, ScriptExecution, ScriptExecutionStatus, ScriptLogLevel,
    ScriptLogRecord, ScriptStore,
};
use correo_storage::legacy::passwords::{LegacyPasswords, SecretKind};
use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::{
    connection_secrets, IgnoredJavaPluginStateKind, MigrationApplier, MigrationDiagnostics,
    MigrationPreview, MigrationWarning,
};
use correo_storage::StorageError;
use std::path::Path;
use std::path::PathBuf;

const MASTER_PASSWORD: &str = "synthetic-master-passphrase";

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

fn legacy_preview() -> MigrationPreview {
    let profile = LegacyProfile::read_from(fixture("legacy_profile")).unwrap();
    MigrationPreview::from_legacy_profile(profile).unwrap()
}

fn seed_existing_target(target: &Path) {
    std::fs::create_dir_all(target.join("scripts")).unwrap();
    std::fs::write(
        target.join("config.json"),
        r#"{"connections":[],"settings":{"saved_locale":"before"}}"#,
    )
    .unwrap();
    std::fs::write(target.join("scripts/original.js"), "logger.info('before');").unwrap();
}

#[test]
fn loads_legacy_profile_fixtures_and_reinitializes_plugins() {
    let profile = LegacyProfile::read_from(fixture("legacy_profile")).unwrap();

    assert_eq!(profile.config.connections.len(), 2);
    assert_eq!(profile.hooks.incoming_messages.len(), 1);
    assert_eq!(profile.hooks.outgoing_messages.len(), 1);
    assert_eq!(
        profile
            .histories
            .publish_topics
            .get("local-broker-01")
            .unwrap()
            .topics,
        ["sensors/temperature", "alerts/status"]
    );
    assert_eq!(
        profile
            .histories
            .subscription_topics
            .get("local-broker-01")
            .unwrap()
            .topics,
        ["sensors/#", "alerts/status"]
    );
    assert_eq!(
        profile
            .histories
            .publish_messages
            .get("local-broker-01")
            .unwrap()
            .messages
            .len(),
        1
    );
    assert_eq!(profile.scripts.len(), 1);
    assert_eq!(
        profile.scripts[0].relative_path,
        Path::new("publish_heartbeat.js")
    );
    assert_eq!(profile.scripts[0].executions.len(), 1);
    assert_eq!(
        profile.scripts[0].executions[0].execution_id.as_deref(),
        Some("execution-001")
    );
    assert_eq!(profile.scripts[0].logs.len(), 1);
    assert_eq!(profile.connection_exports.len(), 1);
    assert_eq!(
        profile.connection_exports[0].connection_config_dtos.len(),
        1
    );
    assert!(profile
        .old_plugin_paths
        .iter()
        .any(|path| path == Path::new("plugins/jars/java-only-plugin.jar")));

    let preview = MigrationPreview::from_legacy_profile(profile).unwrap();
    assert_eq!(preview.settings.saved_locale.as_deref(), Some("de_DE"));
    assert_eq!(preview.settings.current_locale.as_deref(), Some("en_US"));
    assert!(preview.settings.use_regex_for_search);
    assert!(preview.settings.use_ignore_case);
    assert!(!preview.settings.search_updates);
    assert!(!preview.settings.use_default_repo);
    assert!(!preview.settings.install_bundled_plugins);
    assert_eq!(
        preview.settings.keyring_identifier.as_deref(),
        Some("LibSecret")
    );
    assert_eq!(
        preview.settings.plugin_repositories.get("synthetic"),
        Some(&"https://example.invalid/plugins.json".to_owned())
    );
    let global_ui = preview.settings.global_ui_settings.as_ref().unwrap();
    assert_eq!(global_ui.window_width, 1280.0);
    assert_eq!(global_ui.window_height, 800.0);
    let plugin_ids = preview
        .plugin_state
        .manifests
        .iter()
        .map(|manifest| manifest.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        plugin_ids,
        [
            "org.correomqtt.plugins.base64",
            "org.correomqtt.plugins.json-format",
            "org.correomqtt.plugins.xml-format",
            "org.correomqtt.plugins.contains-string-validator",
            "org.correomqtt.plugins.advanced-validator",
            "org.correomqtt.plugins.xml-xsd-validator"
        ]
    );
    assert!(preview
        .plugin_state
        .ignored_legacy_paths
        .iter()
        .any(|path| path == Path::new("plugins/config/java-only-plugin.json")));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_hooks_not_mapped"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_plugins_ignored"));
    assert!(preview
        .report
        .unsupported_fields
        .iter()
        .any(|field| field.path == "config.connections[0].futureJavaField"));
    assert!(preview
        .report
        .unsupported_fields
        .iter()
        .any(|field| field.path
            == "scripts.executions.publish_heartbeat.js.execution-001.futureExecutionField"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "unsupported_legacy_field"));
    assert!(preview
        .report
        .ignored_java_plugin_state
        .iter()
        .any(|state| state.kind == IgnoredJavaPluginStateKind::HookConfig));
    assert!(preview
        .report
        .ignored_java_plugin_state
        .iter()
        .any(|state| state.kind == IgnoredJavaPluginStateKind::JarDirectory));
    assert!(preview
        .report
        .ignored_java_plugin_state
        .iter()
        .any(
            |state| state.kind == IgnoredJavaPluginStateKind::Pf4jMetadata
                && state.path == Path::new("plugins/enabled.txt")
        ));
    assert_eq!(preview.scripts.files.len(), 1);
    assert_eq!(preview.scripts.files[0].name, "publish_heartbeat.js");
    let local_history = preview
        .histories
        .connections
        .get("local-broker-01")
        .unwrap();
    assert_eq!(
        local_history.publish_topics.topics,
        ["sensors/temperature", "alerts/status"]
    );
    assert_eq!(
        local_history.subscriptions.topics,
        ["sensors/#", "alerts/status"]
    );
    assert_eq!(local_history.publish_messages.messages.len(), 1);
    assert_eq!(preview.scripts.executions.len(), 1);
    assert_eq!(
        preview.scripts.executions[0].status,
        ScriptExecutionStatus::Succeeded
    );
    assert_eq!(preview.scripts.executions[0].duration_ms, Some(42));
    assert!(!preview.scripts.executions[0].cancelled);
    assert_eq!(preview.scripts.logs.len(), 3);
    assert!(preview
        .scripts
        .logs
        .iter()
        .any(|record| record.message.contains("[REDACTED]")));
    assert!(!preview
        .scripts
        .logs
        .iter()
        .any(|record| record.message.contains("synthetic-log-password")
            || record.message.contains("synthetic-export-password")));
    assert_eq!(preview.warnings, preview.report.warnings);
}

#[test]
fn decrypts_current_aes_gcm_password_fixture() {
    let passwords = LegacyPasswords::read_from(fixture("legacy_profile/passwords.json"))
        .unwrap()
        .decrypt(MASTER_PASSWORD)
        .unwrap();

    let secrets = connection_secrets(&passwords, "local-broker-01");
    assert_eq!(secrets.len(), 3);
    assert!(secrets.contains(&(SecretKind::Password, "synthetic-mqtt-password")));
    assert!(secrets.contains(&(SecretKind::AuthPassword, "synthetic-ssh-password")));
    assert!(secrets.contains(&(
        SecretKind::SslKeystorePassword,
        "synthetic-keystore-password"
    )));
}

#[test]
fn decrypts_legacy_aes_cbc_password_fixture() {
    let passwords = LegacyPasswords::read_from(fixture("password_formats/passwords_cbc.json"))
        .unwrap()
        .decrypt(MASTER_PASSWORD)
        .unwrap();

    assert_eq!(
        passwords
            .get("local-broker-01_password")
            .map(String::as_str),
        Some("synthetic-mqtt-password")
    );
    assert_eq!(
        passwords
            .get("local-broker-01_auth_password")
            .map(String::as_str),
        Some("synthetic-ssh-password")
    );
    assert_eq!(
        passwords
            .get("local-broker-01_ssl_keystore_password")
            .map(String::as_str),
        Some("synthetic-keystore-password")
    );
}

#[test]
fn script_store_crud_tracks_dirty_state_and_redacts_logs() {
    let temp = tempfile::tempdir().unwrap();
    let store = ScriptStore::new(temp.path());

    let script = store
        .create_script("alerts/publish.js", "logger.info('ok');")
        .unwrap();
    assert_eq!(script.name, "publish.js");
    assert_eq!(store.list_scripts().unwrap().len(), 1);
    assert!(
        !store
            .dirty_state("alerts/publish.js", "logger.info('ok');")
            .unwrap()
            .dirty
    );
    assert!(
        store
            .dirty_state("alerts/publish.js", "logger.info('changed');")
            .unwrap()
            .dirty
    );

    store
        .update_script("alerts/publish.js", "logger.info('changed');")
        .unwrap();
    let renamed = store
        .rename_script("alerts/publish.js", "alerts/publish_renamed.js")
        .unwrap();
    assert_eq!(
        renamed.relative_path,
        Path::new("alerts/publish_renamed.js")
    );
    assert!(matches!(
        store.create_script("../escape.js", ""),
        Err(StorageError::InvalidScriptFileName(_))
    ));

    let execution = ScriptExecution {
        execution_id: "execution-002".to_owned(),
        script_name: "publish_renamed.js".to_owned(),
        script_path: Path::new("alerts/publish_renamed.js").to_path_buf(),
        connection_id: Some("local-broker-01".to_owned()),
        status: ScriptExecutionStatus::Running,
        error: None,
        started_at: Some("2026-06-08T17:20:00.000".to_owned()),
        ended_at: None,
        duration_ms: None,
        cancelled: false,
        log_path: None,
    };
    store
        .save_execution("alerts/publish_renamed.js", &execution)
        .unwrap();
    assert_eq!(
        store
            .load_executions("alerts/publish_renamed.js")
            .unwrap()
            .first()
            .unwrap()
            .execution_id,
        "execution-002"
    );

    for (sequence, message) in [
        "INFO first line",
        "password=synthetic-runtime-password",
        "private key material: synthetic-key-material",
    ]
    .into_iter()
    .enumerate()
    {
        store
            .append_log_record(
                "alerts/publish_renamed.js",
                &ScriptLogRecord {
                    execution_id: "execution-002".to_owned(),
                    sequence: sequence as u64,
                    timestamp: None,
                    level: ScriptLogLevel::Info,
                    message: message.to_owned(),
                },
            )
            .unwrap();
    }
    let log = store
        .load_log("alerts/publish_renamed.js", "execution-002", 2)
        .unwrap();
    assert_eq!(log.records.len(), 2);
    assert_eq!(log.truncated_count, 1);
    assert!(log.records.iter().all(|record| !record
        .message
        .contains("synthetic-runtime-password")
        && !record.message.contains("synthetic-key-material")));

    store.delete_script("alerts/publish_renamed.js").unwrap();
    assert!(matches!(
        store.load_script("alerts/publish_renamed.js"),
        Err(StorageError::ScriptNotFound(_))
    ));
}

#[test]
fn redacts_sensitive_script_log_shapes() {
    let redacted = redact_script_log_text(
        "password=synthetic-password\nexport password: synthetic-export\n-----BEGIN PRIVATE KEY-----",
    );

    assert!(redacted.contains("password= [REDACTED]"));
    assert!(redacted.contains("export password: [REDACTED]"));
    assert!(redacted.contains("[REDACTED KEY MATERIAL]"));
    assert!(!redacted.contains("synthetic-password"));
    assert!(!redacted.contains("synthetic-export"));
}

#[test]
fn migration_apply_creates_backup_and_rolls_back_from_temp_fixture() {
    let preview = legacy_preview();
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = temp.path().join("backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let outcome = applier.apply_preview(&preview).unwrap();

    assert!(outcome.backup.id.starts_with("migration-backup-"));
    assert_eq!(outcome.backup.target_root, target);
    assert!(outcome.backup.path.join("config.json").exists());
    assert!(
        std::fs::read_to_string(outcome.backup.path.join("config.json"))
            .unwrap()
            .contains("before")
    );
    assert!(target.join("migration-diagnostics.json").exists());

    let migrated = ConfigStore::new(&target).load().unwrap();
    assert_eq!(migrated.settings.saved_locale.as_deref(), Some("de_DE"));
    assert!(target.join("scripts/publish_heartbeat.js").exists());

    let rollback = applier.rollback(&outcome.backup).unwrap();
    assert!(rollback
        .recovery_steps
        .iter()
        .any(|step| step.contains("Restored migration target")));
    assert!(std::fs::read_to_string(target.join("config.json"))
        .unwrap()
        .contains("before"));
    assert!(target.join("scripts/original.js").exists());
    assert!(!target.join("scripts/publish_heartbeat.js").exists());
}

#[test]
fn migration_apply_restores_backup_when_write_fails() {
    let mut preview = legacy_preview();
    preview.scripts.files[0].relative_path = PathBuf::from("../escape.js");
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = temp.path().join("backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let error = applier.apply_preview(&preview).unwrap_err();

    assert!(matches!(error, StorageError::InvalidScriptFileName(_)));
    assert!(std::fs::read_to_string(target.join("config.json"))
        .unwrap()
        .contains("before"));
    assert!(target.join("scripts/original.js").exists());
    assert!(std::fs::read_dir(backup_root).unwrap().next().is_some());
}

#[test]
fn migration_rollback_refuses_after_target_changes() {
    let preview = legacy_preview();
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = temp.path().join("backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let outcome = applier.apply_preview(&preview).unwrap();
    std::fs::write(target.join("newer-data.txt"), "newer").unwrap();

    let error = applier.rollback(&outcome.backup).unwrap_err();
    assert!(matches!(
        error,
        StorageError::MigrationRollbackSafety { .. }
    ));
    assert!(target.join("newer-data.txt").exists());
}

#[test]
fn migration_diagnostics_capture_fields_and_redact_sensitive_text() {
    let mut preview = legacy_preview();
    preview.report.warnings.push(MigrationWarning {
        code: "synthetic_secret_shape",
        message: "password=synthetic-diagnostic-secret".to_owned(),
    });

    let diagnostics = MigrationDiagnostics::from_preview(&preview, None);

    assert!(diagnostics
        .mapped_fields
        .iter()
        .any(|field| field.legacy_path == "config.connections[].url"));
    assert!(diagnostics
        .unmapped_fields
        .iter()
        .any(|field| field == "config.connections[0].futureJavaField"));
    assert!(diagnostics
        .warnings
        .iter()
        .any(|warning| warning.message.contains("[REDACTED]")));
    let serialized = serde_json::to_string(&diagnostics).unwrap();
    assert!(!serialized.contains("synthetic-diagnostic-secret"));
    assert!(!serialized.contains("synthetic-log-password"));
    assert!(!serialized.contains("synthetic-export-password"));
}
