use correo_plugins::{
    bundled_plugin_by_id, bundled_plugin_manifests, legacy_plugin_replacement_decisions,
    DetailFormatDto, DetailFormatterRequest, HookInvocation, HookOutput, MessageDto,
    MessageTransformOutcomeDto, MessageValidatorRequest, ValidationResultDto,
};
use serde_json::json;
use std::collections::BTreeSet;

#[test]
fn bundled_manifests_cover_mvp_replacements_with_config_schemas() {
    let manifests = bundled_plugin_manifests();
    let ids = manifests
        .iter()
        .map(|manifest| manifest.id.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        ids,
        BTreeSet::from([
            "builtin.base64",
            "builtin.contains-string-validator",
            "builtin.json-format",
            "builtin.xml-format",
        ])
    );

    for manifest in manifests {
        manifest.validate().unwrap();
        assert!(manifest.config_schema.is_some());
        assert!(!manifest
            .capabilities
            .grants_host_surface(correo_plugins::HostSurface::Filesystem));
        assert!(!manifest
            .capabilities
            .grants_host_surface(correo_plugins::HostSurface::Network));
        assert!(!manifest
            .capabilities
            .grants_host_surface(correo_plugins::HostSurface::Secrets));
        assert!(!manifest
            .capabilities
            .grants_host_surface(correo_plugins::HostSurface::Mqtt));
    }
}

#[test]
fn legacy_plugin_decisions_are_explicit_for_supported_and_deferred_plugins() {
    let decisions = legacy_plugin_replacement_decisions();
    let supported = decisions
        .iter()
        .filter(|decision| {
            decision.status == correo_plugins::LegacyPluginReplacementStatus::Supported
        })
        .map(|decision| decision.legacy_plugin_id)
        .collect::<BTreeSet<_>>();
    let unsupported = decisions
        .iter()
        .filter(|decision| {
            decision.status == correo_plugins::LegacyPluginReplacementStatus::Unsupported
        })
        .map(|decision| decision.legacy_plugin_id)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        supported,
        BTreeSet::from([
            "base64",
            "contains-string-validator",
            "json-format",
            "xml-format",
        ])
    );
    assert_eq!(
        unsupported,
        BTreeSet::from([
            "advanced-validator",
            "save-manipulator",
            "systopic",
            "xml-xsd-validator",
            "zip-manipulator",
        ])
    );
    assert!(decisions
        .iter()
        .filter(|decision| {
            decision.status == correo_plugins::LegacyPluginReplacementStatus::Unsupported
        })
        .all(|decision| !decision.reason.is_empty() && decision.bundled_plugin_id.is_none()));
}

#[test]
fn base64_replacement_encodes_outgoing_and_decodes_incoming() {
    let plugin = bundled_plugin_by_id("builtin.base64").unwrap();
    let outgoing = plugin
        .dispatch(HookInvocation::OutgoingMessageTransform(
            correo_plugins::OutgoingMessageTransformRequest::new(MessageDto::new(
                "demo/topic",
                b"hello".to_vec(),
            )),
        ))
        .unwrap();

    assert_transformed_payload(outgoing, b"aGVsbG8=");

    let incoming = plugin
        .dispatch(HookInvocation::IncomingMessageTransform(
            correo_plugins::IncomingMessageTransformRequest::new(MessageDto::new(
                "demo/topic",
                b"aGVsbG8=".to_vec(),
            )),
        ))
        .unwrap();

    match incoming {
        HookOutput::IncomingMessageTransform(response) => match response.outcome {
            MessageTransformOutcomeDto::Replace { message } => {
                assert_eq!(message.payload, b"hello");
            }
            other => panic!("unexpected outcome: {other:?}"),
        },
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn detail_formatters_return_pretty_json_and_xml_text() {
    let json = bundled_plugin_by_id("builtin.json-format").unwrap();
    let output = json
        .dispatch(HookInvocation::DetailFormatter(
            DetailFormatterRequest::new(br#"{"ok":true,"items":[1,2]}"#.to_vec()),
        ))
        .unwrap();
    assert_formatted(output, DetailFormatDto::Json, "\"items\": [");

    let xml = bundled_plugin_by_id("builtin.xml-format").unwrap();
    let output = xml
        .dispatch(HookInvocation::DetailFormatter(
            DetailFormatterRequest::new(br#"<root><item id="1">ok</item></root>"#.to_vec()),
        ))
        .unwrap();
    assert_formatted(output, DetailFormatDto::Xml, "  <item id=\"1\">");
}

#[test]
fn contains_string_validator_supports_case_sensitive_config() {
    let plugin = bundled_plugin_by_id("builtin.contains-string-validator").unwrap();
    let mut request =
        MessageValidatorRequest::new(MessageDto::new("demo/topic", b"Telemetry READY".to_vec()));
    request.config = json!({
        "text": "ready",
        "case_sensitive": false
    });

    let output = plugin
        .dispatch(HookInvocation::MessageValidator(request))
        .unwrap();
    match output {
        HookOutput::MessageValidator(response) => {
            assert_eq!(response.result, ValidationResultDto::Valid);
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

fn assert_transformed_payload(output: HookOutput, expected: &[u8]) {
    match output {
        HookOutput::OutgoingMessageTransform(response) => match response.outcome {
            MessageTransformOutcomeDto::Replace { message } => {
                assert_eq!(message.payload, expected);
            }
            other => panic!("unexpected outcome: {other:?}"),
        },
        other => panic!("unexpected output: {other:?}"),
    }
}

fn assert_formatted(output: HookOutput, format: DetailFormatDto, expected_text: &str) {
    match output {
        HookOutput::DetailFormatter(response) => {
            assert_eq!(response.output.format, format);
            assert!(response.output.text.contains(expected_text));
            assert!(response.output.diagnostics.is_empty());
        }
        other => panic!("unexpected output: {other:?}"),
    }
}
