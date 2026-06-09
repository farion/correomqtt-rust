use crate::{ConfigSchemaMetadata, MessageValidatorResponse, ValidationResultDto, ABI_VERSION};
use correo_plugins_advanced_validator::{
    validate_payload, AdvancedValidation, AdvancedValidatorError,
};
use serde_json::{json, Value};

pub fn validate_message(
    config: Value,
    payload: &[u8],
) -> Result<MessageValidatorResponse, AdvancedValidatorError> {
    let result = match validate_payload(config, payload)? {
        AdvancedValidation::Valid => ValidationResultDto::Valid,
        AdvancedValidation::Invalid { message } => ValidationResultDto::Invalid { message },
    };
    Ok(MessageValidatorResponse {
        abi_version: ABI_VERSION,
        result,
    })
}

pub fn config_schema() -> ConfigSchemaMetadata {
    ConfigSchemaMetadata {
        schema_version: 1,
        document: json!({
            "type": "object",
            "properties": {
                "and": { "type": "array", "items": { "$ref": "#/$defs/rule" } },
                "or": { "type": "array", "items": { "$ref": "#/$defs/rule" } },
                "extensions": {
                    "type": "array",
                    "items": { "$ref": "#/$defs/extension" }
                }
            },
            "additionalProperties": false,
            "$defs": {
                "rule": {
                    "type": "object",
                    "properties": {
                        "and": {
                            "type": "array",
                            "items": { "$ref": "#/$defs/rule" }
                        },
                        "or": {
                            "type": "array",
                            "items": { "$ref": "#/$defs/rule" }
                        },
                        "extensions": {
                            "type": "array",
                            "items": { "$ref": "#/$defs/extension" }
                        }
                    },
                    "additionalProperties": false
                },
                "extension": {
                    "type": "object",
                    "properties": {
                        "pluginId": { "type": "string" },
                        "plugin_id": { "type": "string" },
                        "plugin": { "type": "string" },
                        "name": { "type": "string" },
                        "id": { "type": "string" },
                        "extensionId": { "type": "string" },
                        "extension_id": { "type": "string" },
                        "config": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string" },
                                "case_sensitive": {
                                    "type": "boolean",
                                    "default": true
                                }
                            },
                            "additionalProperties": false
                        }
                    },
                    "required": ["config"],
                    "additionalProperties": false
                }
            }
        }),
    }
}
