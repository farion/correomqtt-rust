use super::BundledPluginError;
use crate::{
    DetailFormatDto, DetailFormatterRequest, DetailFormatterResponse, FormattedDetailDto,
    HookDiagnosticDto, HookDiagnosticSeverityDto, HookKind, ABI_VERSION,
};
use correo_plugin_xml_format::{
    format_xml_bytes, XmlDetailFormat, XmlFormatDiagnostic, XmlFormatDiagnosticSeverity,
    XmlFormatOutput,
};
use serde_json::Value;

pub(super) fn format_json(
    request: DetailFormatterRequest,
    plugin_id: &str,
) -> Result<DetailFormatterResponse, BundledPluginError> {
    let text = utf8_payload(request.bytes, plugin_id, HookKind::DetailFormatter)?;
    match serde_json::from_str::<Value>(&text) {
        Ok(value) => Ok(formatted_detail(
            DetailFormatDto::Json,
            serde_json::to_string_pretty(&value).expect("serde_json::Value serializes"),
            Vec::new(),
        )),
        Err(_) => Ok(formatted_detail(
            DetailFormatDto::PlainText,
            text,
            vec![warning("Input is not valid JSON; returned unchanged.")],
        )),
    }
}

pub(super) fn format_xml(
    request: DetailFormatterRequest,
    plugin_id: &str,
) -> Result<DetailFormatterResponse, BundledPluginError> {
    format_xml_bytes(request.bytes)
        .map(xml_output)
        .map_err(|source| BundledPluginError::InvalidUtf8 {
            plugin_id: plugin_id.to_owned(),
            hook: HookKind::DetailFormatter,
            source: source.into_source(),
        })
}

fn formatted_detail(
    format: DetailFormatDto,
    text: String,
    diagnostics: Vec<HookDiagnosticDto>,
) -> DetailFormatterResponse {
    DetailFormatterResponse {
        abi_version: ABI_VERSION,
        output: FormattedDetailDto {
            format,
            text,
            diagnostics,
        },
    }
}

fn utf8_payload(
    bytes: Vec<u8>,
    plugin_id: &str,
    hook: HookKind,
) -> Result<String, BundledPluginError> {
    String::from_utf8(bytes).map_err(|source| BundledPluginError::InvalidUtf8 {
        plugin_id: plugin_id.to_owned(),
        hook,
        source,
    })
}

fn xml_output(output: XmlFormatOutput) -> DetailFormatterResponse {
    formatted_detail(
        xml_format(output.format),
        output.text,
        output.diagnostics.into_iter().map(xml_diagnostic).collect(),
    )
}

fn xml_format(format: XmlDetailFormat) -> DetailFormatDto {
    match format {
        XmlDetailFormat::Xml => DetailFormatDto::Xml,
        XmlDetailFormat::PlainText => DetailFormatDto::PlainText,
    }
}

fn xml_diagnostic(diagnostic: XmlFormatDiagnostic) -> HookDiagnosticDto {
    HookDiagnosticDto {
        severity: match diagnostic.severity {
            XmlFormatDiagnosticSeverity::Warning => HookDiagnosticSeverityDto::Warning,
        },
        message: diagnostic.message,
    }
}

fn warning(message: &str) -> HookDiagnosticDto {
    HookDiagnosticDto {
        severity: HookDiagnosticSeverityDto::Warning,
        message: message.to_owned(),
    }
}
