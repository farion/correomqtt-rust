use correo_plugins::{
    bundled_plugin_by_id, HookInvocation, HookOutput, MessageDto, MessageValidatorRequest,
    ValidationResultDto,
};
use serde_json::{json, Value};

#[test]
fn advanced_validator_composes_legacy_contains_string_extensions() {
    let plugin = bundled_plugin_by_id("org.correomqtt.plugins.advanced-validator").unwrap();
    let mut request = MessageValidatorRequest::new(MessageDto::new(
        "demo/topic",
        b"Test payload with another okay".to_vec(),
    ));
    request.config = json!({
        "and": [
            { "extensions": [contains_extension("ignoreCase", "test")] },
            { "extensions": [contains_extension("ignoreCase", "another")] }
        ],
        "or": [
            { "extensions": [contains_extension("caseSensitive", "missing")] },
            { "extensions": [contains_extension("caseSensitive", "okay")] }
        ]
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

#[test]
fn advanced_validator_rejects_when_required_and_rule_fails() {
    let plugin = bundled_plugin_by_id("org.correomqtt.plugins.advanced-validator").unwrap();
    let mut request =
        MessageValidatorRequest::new(MessageDto::new("demo/topic", b"Test payload".to_vec()));
    request.config = json!({
        "and": [
            { "extensions": [contains_extension("ignoreCase", "test")] },
            { "extensions": [contains_extension("ignoreCase", "missing")] }
        ]
    });

    let output = plugin
        .dispatch(HookInvocation::MessageValidator(request))
        .unwrap();

    match output {
        HookOutput::MessageValidator(response) => {
            assert_eq!(
                response.result,
                ValidationResultDto::Invalid {
                    message: "Advanced validator composition rejected the payload.".to_owned()
                }
            );
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

fn contains_extension(id: &str, text: &str) -> Value {
    json!({
        "pluginId": "contains-string-validator",
        "id": id,
        "config": { "text": text }
    })
}
