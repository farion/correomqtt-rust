use correo_plugins::{
    DetailFormatDto, DetailFormatterRequest, HookInvocation, HookKind, HookOutput, HostSurface,
    PluginManifest, PluginPackage, WasmtimePluginRuntime,
};
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn systopic_manifest_declares_only_detail_formatter_without_host_caps() {
    let manifest = PluginManifest::from_toml_str(include_str!(
        "../../../plugins/correo-plugins-systopic/plugin.toml"
    ))
    .unwrap();

    assert_eq!(manifest.id, "builtin.system-topic");
    assert_eq!(manifest.capabilities.hooks, vec![HookKind::DetailFormatter]);
    assert_eq!(
        manifest
            .entrypoint_for(HookKind::DetailFormatter)
            .map(|entrypoint| entrypoint.export.as_str()),
        Some("correo_detail_formatter")
    );
    for surface in [
        HostSurface::Filesystem,
        HostSurface::Network,
        HostSurface::Secrets,
        HostSurface::Mqtt,
    ] {
        assert!(!manifest.capabilities.grants_host_surface(surface));
    }
}

#[test]
fn systopic_wasm_plugin_formats_known_and_aggregated_metrics() {
    let workspace = workspace_root();
    let wasm_path = build_systopic_wasm(&workspace);
    let package_dir = package_temp_dir(&workspace);
    fs::copy(
        workspace.join("plugins/correo-plugins-systopic/plugin.toml"),
        package_dir.path().join("plugin.toml"),
    )
    .unwrap();
    fs::copy(wasm_path, package_dir.path().join("plugin.wasm")).unwrap();

    let package = PluginPackage::load(package_dir.path()).unwrap();
    let plugin = WasmtimePluginRuntime::default()
        .compile_package(package, &Version::new(0, 1, 0))
        .unwrap();

    let connected = plugin
        .dispatch(HookInvocation::DetailFormatter(request(
            "$SYS/broker/clients/connected",
            b"7",
        )))
        .unwrap();
    assert_formatted(connected, "Connected clients");

    let aggregate = plugin
        .dispatch(HookInvocation::DetailFormatter(request(
            "$SYS/broker/load/messages/received/15min",
            b"42",
        )))
        .unwrap();
    assert_formatted(aggregate, "Window: 15min");
}

fn request(topic: &str, payload: &[u8]) -> DetailFormatterRequest {
    let mut request = DetailFormatterRequest::new(payload.to_vec());
    request.context.subscription_topic = Some(topic.to_owned());
    request
}

fn package_temp_dir(workspace: &Path) -> TempDir {
    let root = workspace.join("target/systopic-plugin-test/packages");
    fs::create_dir_all(&root).unwrap();
    TempDir::new_in(root).unwrap()
}

fn build_systopic_wasm(workspace: &Path) -> PathBuf {
    let target_dir = workspace.join("target/systopic-plugin-test");
    fs::create_dir_all(&target_dir).unwrap();
    let status = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", &target_dir)
        .args([
            "build",
            "-p",
            "correo-plugins-systopic",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ])
        .status()
        .unwrap();
    assert!(status.success(), "failed to build systopic WASM plugin");

    target_dir
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("correo_plugins_systopic.wasm")
}

fn assert_formatted(output: HookOutput, expected_text: &str) {
    match output {
        HookOutput::DetailFormatter(response) => {
            assert_eq!(response.output.format, DetailFormatDto::PlainText);
            assert!(response.output.text.contains(expected_text));
            assert!(response.output.diagnostics.is_empty());
        }
        other => panic!("unexpected hook output: {other:?}"),
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}
