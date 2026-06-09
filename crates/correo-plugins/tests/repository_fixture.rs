use correo_plugins::{
    PluginInstallSource, PluginRepositoryDefinition, PluginRepositoryEntry,
    PLUGIN_REPOSITORY_FORMAT_VERSION,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/repository.json");

#[test]
fn repository_fixture_matches_generated_bundled_manifest_catalog() {
    let repository = PluginRepositoryDefinition::from_bundled_plugins(
        "local-bundled-rust",
        "Bundled Rust Plugins",
    );
    repository.validate().unwrap();

    let mut generated = serde_json::to_string_pretty(&repository).unwrap();
    generated.push('\n');

    assert_eq!(generated, FIXTURE);
    assert_eq!(
        repository.repository_format_version,
        PLUGIN_REPOSITORY_FORMAT_VERSION
    );
}

#[test]
fn repository_fixture_is_path_safe_and_contains_only_bundled_sources() {
    let repository = serde_json::from_str::<PluginRepositoryDefinition>(FIXTURE).unwrap();

    repository.validate().unwrap();
    assert!(!repository.plugins.is_empty());

    for plugin in &repository.plugins {
        assert!(matches!(
            &plugin.install_source,
            PluginInstallSource::Bundled { plugin_id } if plugin_id == &plugin.manifest.id
        ));
    }

    let fixture_json = serde_json::from_str::<Value>(FIXTURE).unwrap();
    let fixture_text = fixture_json.to_string();
    assert!(!fixture_text.contains(".jar"));
    assert!(!fixture_text.contains("plugins/jars"));
}

#[test]
fn repository_rejects_absolute_or_parent_package_paths() {
    let manifest = PluginRepositoryDefinition::from_bundled_plugins("local", "Local")
        .plugins
        .into_iter()
        .next()
        .unwrap()
        .manifest;

    assert!(PluginRepositoryEntry::local_package(manifest.clone(), "/tmp/plugin").is_err());
    assert!(PluginRepositoryEntry::local_package(manifest, "../plugin").is_err());
}
