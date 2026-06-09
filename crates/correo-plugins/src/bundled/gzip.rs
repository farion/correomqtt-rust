use super::BundledPluginError;
use crate::{
    ConfigSchemaMetadata, DetailByteTransformRequest, DetailByteTransformResponse, HookKind,
    ABI_VERSION,
};
use correo_plugin_zip_manipulator::{
    transform_detail_bytes_from_config, ZipManipulatorError, DEFAULT_MAX_INPUT_BYTES,
    DEFAULT_MAX_OUTPUT_BYTES,
};
use serde_json::json;

pub(super) fn transform_gzip_detail(
    request: DetailByteTransformRequest,
    plugin_id: &str,
) -> Result<DetailByteTransformResponse, BundledPluginError> {
    let result =
        transform_detail_bytes_from_config(request.config, request.bytes, request.content_type)
            .map_err(|source| BundledPluginError::TransformFailed {
                plugin_id: plugin_id.to_owned(),
                hook: HookKind::DetailByteTransform,
                source,
            })?;

    Ok(DetailByteTransformResponse {
        abi_version: ABI_VERSION,
        bytes: result.bytes,
        content_type: result.content_type,
        host_actions: Vec::new(),
    })
}

pub(super) fn gzip_config_schema() -> ConfigSchemaMetadata {
    ConfigSchemaMetadata {
        schema_version: 1,
        document: json!({
            "type": "object",
            "required": ["operation"],
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["zip", "unzip"],
                    "default": "unzip"
                },
                "max_input_bytes": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": DEFAULT_MAX_INPUT_BYTES,
                    "default": DEFAULT_MAX_INPUT_BYTES
                },
                "max_output_bytes": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": DEFAULT_MAX_OUTPUT_BYTES,
                    "default": DEFAULT_MAX_OUTPUT_BYTES
                }
            },
            "additionalProperties": false
        }),
    }
}

pub(super) type GzipTransformError = ZipManipulatorError;
