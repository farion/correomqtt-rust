use super::BundledPluginError;
use crate::{
    DetailFormatDto, DetailFormatterRequest, DetailFormatterResponse, FormattedDetailDto,
    HookDiagnosticDto, HookDiagnosticSeverityDto, HookKind, ABI_VERSION,
};
use correo_plugin_xml_format::{
    format_xml_bytes, XmlDetailFormat, XmlFormatDiagnostic, XmlFormatDiagnosticSeverity,
    XmlFormatOutput,
};
use correo_plugins_json_format::{
    format_json_bytes, JsonDetailFormat, JsonFormatDiagnostic, JsonFormatDiagnosticSeverity,
    JsonFormatOutput,
};
use correo_plugins_systopic::{
    format_sys_topic_detail, SysTopicDetailFormat, SysTopicFormatDiagnostic,
    SysTopicFormatDiagnosticSeverity, SysTopicFormatOutput,
};

pub(super) fn format_json(
    request: DetailFormatterRequest,
    plugin_id: &str,
) -> Result<DetailFormatterResponse, BundledPluginError> {
    format_json_bytes(request.bytes)
        .map(json_output)
        .map_err(|source| BundledPluginError::InvalidUtf8 {
            plugin_id: plugin_id.to_owned(),
            hook: HookKind::DetailFormatter,
            source: source.into_source(),
        })
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

pub(super) fn format_system_topic(request: DetailFormatterRequest) -> DetailFormatterResponse {
    let topic = request.context.subscription_topic.as_deref();
    sys_topic_output(format_sys_topic_detail(topic, request.bytes))
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

fn json_output(output: JsonFormatOutput) -> DetailFormatterResponse {
    formatted_detail(
        json_format(output.format),
        output.text,
        output
            .diagnostics
            .into_iter()
            .map(json_diagnostic)
            .collect(),
    )
}

fn json_format(format: JsonDetailFormat) -> DetailFormatDto {
    match format {
        JsonDetailFormat::Json => DetailFormatDto::Json,
        JsonDetailFormat::PlainText => DetailFormatDto::PlainText,
    }
}

fn json_diagnostic(diagnostic: JsonFormatDiagnostic) -> HookDiagnosticDto {
    HookDiagnosticDto {
        severity: match diagnostic.severity {
            JsonFormatDiagnosticSeverity::Warning => HookDiagnosticSeverityDto::Warning,
        },
        message: diagnostic.message,
    }
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

fn sys_topic_output(output: SysTopicFormatOutput) -> DetailFormatterResponse {
    formatted_detail(
        sys_topic_format(output.format),
        output.text,
        output
            .diagnostics
            .into_iter()
            .map(sys_topic_diagnostic)
            .collect(),
    )
}

fn sys_topic_format(format: SysTopicDetailFormat) -> DetailFormatDto {
    match format {
        SysTopicDetailFormat::PlainText => DetailFormatDto::PlainText,
    }
}

fn sys_topic_diagnostic(diagnostic: SysTopicFormatDiagnostic) -> HookDiagnosticDto {
    HookDiagnosticDto {
        severity: match diagnostic.severity {
            SysTopicFormatDiagnosticSeverity::Info => HookDiagnosticSeverityDto::Info,
            SysTopicFormatDiagnosticSeverity::Warning => HookDiagnosticSeverityDto::Warning,
        },
        message: diagnostic.message,
    }
}
