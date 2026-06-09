use correo_plugins::{
    bundled_plugin_by_id, HookInvocation, HookKind, HookOutput, HostSurface, MessageDto,
    MessageValidatorRequest, PluginManifest, ValidationResultDto,
};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

#[test]
fn contains_string_plugin_manifest_declares_only_validator_hook() {
    let manifest_text = fs::read_to_string(plugin_manifest_path()).unwrap();
    let manifest = PluginManifest::from_toml_str(&manifest_text).unwrap();

    assert_eq!(
        manifest.id,
        "org.correomqtt.plugins.contains-string-validator"
    );
    assert!(manifest
        .capabilities
        .grants_hook(HookKind::MessageValidator));
    assert_eq!(manifest.entrypoints.len(), 1);
    assert_eq!(
        manifest
            .entrypoint_for(HookKind::MessageValidator)
            .map(|entrypoint| entrypoint.export.as_str()),
        Some("correo_message_validator")
    );

    for surface in [
        HostSurface::Filesystem,
        HostSurface::Network,
        HostSurface::Secrets,
        HostSurface::Mqtt,
    ] {
        assert!(!manifest.capabilities.grants_host_surface(surface));
    }

    let schema = manifest.config_schema.unwrap();
    assert_eq!(schema.schema_version, 1);
    assert_eq!(schema.document["required"][0].as_str(), Some("text"));
    assert_eq!(
        schema.document["properties"]["case_sensitive"]["default"].as_bool(),
        Some(true)
    );
}

#[test]
fn bundled_contains_string_validator_preserves_legacy_matching_modes() {
    assert_validation("Telemetry READY", "READY", true, ValidationResultDto::Valid);
    assert_validation(
        "Telemetry READY",
        "ready",
        true,
        ValidationResultDto::Invalid {
            message: "Payload does not contain the configured text.".to_owned(),
        },
    );
    assert_validation(
        "Telemetry READY",
        "ready",
        false,
        ValidationResultDto::Valid,
    );
    assert_validation(
        "Telemetry READY",
        "missing",
        false,
        ValidationResultDto::Invalid {
            message: "Payload does not contain the configured text, ignoring case.".to_owned(),
        },
    );
}

fn assert_validation(
    payload: &str,
    text: &str,
    case_sensitive: bool,
    expected: ValidationResultDto,
) {
    let plugin = bundled_plugin_by_id("builtin.contains-string-validator").unwrap();
    let mut request =
        MessageValidatorRequest::new(MessageDto::new("demo/topic", payload.as_bytes().to_vec()));
    request.config = json!({
        "text": text,
        "case_sensitive": case_sensitive
    });

    let output = plugin
        .dispatch(HookInvocation::MessageValidator(request))
        .unwrap();

    match output {
        HookOutput::MessageValidator(response) => assert_eq!(response.result, expected),
        other => panic!("unexpected output: {other:?}"),
    }
}

fn plugin_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../plugins/correo-plugins-contains-string-validator/plugin.toml")
}
