use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::{MigrationApplier, MigrationPreview};
use correo_storage::StorageError;
use std::path::{Path, PathBuf};

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
fn migration_apply_rejects_nested_backup_root_before_copying() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = target.join("migration-backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let error = applier.apply_preview(&legacy_preview()).unwrap_err();

    assert!(matches!(
        &error,
        StorageError::MigrationRollbackSafety { .. }
    ));
    assert!(error.to_string().contains("backup destination"));
    assert!(!backup_root.exists());
    assert!(std::fs::read_to_string(target.join("config.json"))
        .unwrap()
        .contains("before"));
    assert!(target.join("scripts/original.js").exists());
}
