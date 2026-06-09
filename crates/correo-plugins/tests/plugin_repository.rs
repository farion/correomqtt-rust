use correo_plugins::{
    PluginInstallSource, PluginManifest, PluginRepositoryDefinition, PluginRepositoryEntry,
};
use std::collections::BTreeSet;

#[test]
fn bundled_repository_definition_lists_installable_replacements() {
    let repository =
        PluginRepositoryDefinition::from_bundled_plugins("local.bundled", "Bundled replacements");
    let ids = repository
        .plugins
        .iter()
        .map(|plugin| plugin.manifest.id.as_str())
        .collect::<BTreeSet<_>>();

    repository.validate().unwrap();
    assert_eq!(repository.repository_format_version, 1);
    assert_eq!(repository.id, "local.bundled");
    assert_eq!(
        ids,
        BTreeSet::from([
            "builtin.base64",
            "builtin.contains-string-validator",
            "builtin.json-format",
            "builtin.xml-format",
        ])
    );

    for plugin in &repository.plugins {
        assert!(!plugin.manifest.capabilities.hooks.is_empty());
        assert_eq!(plugin.manifest.provider, "CorreoMQTT");
        assert_eq!(
            plugin.manifest.compatible_correomqtt.to_string(),
            ">=0.1.0, <1.0.0"
        );
        assert!(matches!(
            &plugin.install_source,
            PluginInstallSource::Bundled { plugin_id } if plugin_id == &plugin.manifest.id
        ));
    }

    let json = serde_json::to_value(&repository).unwrap();
    assert_eq!(json["repository_format_version"], 1);
    assert_eq!(json["plugins"][0]["install_source"]["kind"], "bundled");
}

#[test]
fn repository_definition_points_at_local_package_roots() {
    let manifest = PluginManifest::from_toml_str(
        r#"
manifest_version = 1
id = "workspace.example"
name = "Workspace Example"
version = "0.1.0"
description = "Example local package."
provider = "Workspace"
license = "GPL-3.0-or-later"
compatible_correomqtt = ">=0.1.0, <1.0.0"

[capabilities]
hooks = ["detail_formatter"]

[[entrypoints]]
hook = "detail_formatter"
export = "correo_detail_formatter"
"#,
    )
    .unwrap();
    let entry = PluginRepositoryEntry::local_package(manifest.clone(), "plugins/workspace-example")
        .unwrap();
    let repository = PluginRepositoryDefinition {
        repository_format_version: 1,
        id: "local.workspace".to_owned(),
        name: "Workspace".to_owned(),
        plugins: vec![entry],
    };

    repository.validate().unwrap();
    assert_eq!(repository.plugins.len(), 1);
    let plugin = &repository.plugins[0];
    assert_eq!(plugin.manifest.id, "workspace.example");
    assert_eq!(
        plugin.manifest.capabilities.hooks,
        vec![correo_plugins::HookKind::DetailFormatter]
    );
    assert!(matches!(
        &plugin.install_source,
        PluginInstallSource::LocalPackage { path } if path == "plugins/workspace-example"
    ));
    assert!(PluginRepositoryEntry::local_package(manifest, "../outside").is_err());
}
