use super::BundledPluginError;
use crate::{
    DetailFormatDto, DetailFormatterRequest, DetailFormatterResponse, FormattedDetailDto,
    HookDiagnosticDto, HookDiagnosticSeverityDto, HookKind, ABI_VERSION,
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
    let text = utf8_payload(request.bytes, plugin_id, HookKind::DetailFormatter)?;
    match pretty_xml(&text) {
        Some(pretty) => Ok(formatted_detail(DetailFormatDto::Xml, pretty, Vec::new())),
        None => Ok(formatted_detail(
            DetailFormatDto::PlainText,
            text,
            vec![warning(
                "Input is not valid XML-like text; returned unchanged.",
            )],
        )),
    }
}

fn pretty_xml(input: &str) -> Option<String> {
    let text = input.trim();
    if !text.starts_with('<') || !text.ends_with('>') {
        return None;
    }

    let mut out = String::new();
    let mut indent = 0usize;
    let mut cursor = 0usize;
    let mut saw_tag = false;

    while cursor < text.len() {
        let open = text[cursor..].find('<')?;
        let content = text[cursor..cursor + open].trim();
        if !content.is_empty() {
            push_xml_line(&mut out, indent, content);
        }
        cursor += open;

        let close = text[cursor..].find('>')?;
        let tag = &text[cursor..cursor + close + 1];
        if tag.len() <= 2 {
            return None;
        }

        if tag.starts_with("</") {
            indent = indent.checked_sub(1)?;
            push_xml_line(&mut out, indent, tag);
        } else if tag.starts_with("<?") || tag.starts_with("<!") || tag.ends_with("/>") {
            push_xml_line(&mut out, indent, tag);
        } else {
            push_xml_line(&mut out, indent, tag);
            indent += 1;
        }
        saw_tag = true;
        cursor += close + 1;
    }

    if saw_tag && indent == 0 {
        Some(out.trim_end().to_owned())
    } else {
        None
    }
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

fn push_xml_line(out: &mut String, indent: usize, line: &str) {
    out.push_str(&"  ".repeat(indent));
    out.push_str(line);
    out.push('\n');
}

fn warning(message: &str) -> HookDiagnosticDto {
    HookDiagnosticDto {
        severity: HookDiagnosticSeverityDto::Warning,
        message: message.to_owned(),
    }
}
