#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
use std::string::FromUtf8Error;

pub const ABI_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlFormatOutput {
    pub format: XmlDetailFormat,
    pub text: String,
    pub diagnostics: Vec<XmlFormatDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlDetailFormat {
    Xml,
    PlainText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlFormatDiagnostic {
    pub severity: XmlFormatDiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlFormatDiagnosticSeverity {
    Warning,
}

#[derive(Debug)]
pub struct XmlFormatError {
    source: FromUtf8Error,
}

impl XmlFormatError {
    pub fn into_source(self) -> FromUtf8Error {
        self.source
    }
}

impl From<FromUtf8Error> for XmlFormatError {
    fn from(source: FromUtf8Error) -> Self {
        Self { source }
    }
}

pub fn format_xml_bytes(bytes: Vec<u8>) -> Result<XmlFormatOutput, XmlFormatError> {
    let text = String::from_utf8(bytes)?;
    Ok(match pretty_xml(&text) {
        Some(pretty) => XmlFormatOutput {
            format: XmlDetailFormat::Xml,
            text: pretty,
            diagnostics: Vec::new(),
        },
        None => XmlFormatOutput {
            format: XmlDetailFormat::PlainText,
            text,
            diagnostics: vec![XmlFormatDiagnostic {
                severity: XmlFormatDiagnosticSeverity::Warning,
                message: "Input is not valid XML-like text; returned unchanged.".to_owned(),
            }],
        },
    })
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

fn push_xml_line(out: &mut String, indent: usize, line: &str) {
    out.push_str(&"  ".repeat(indent));
    out.push_str(line);
    out.push('\n');
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correomqtt_alloc(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }

    let Ok(layout) = std::alloc::Layout::from_size_align(len as usize, 1) else {
        return 0;
    };
    unsafe { std::alloc::alloc(layout) as i32 }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correomqtt_dealloc(ptr: i32, len: i32) {
    if ptr <= 0 || len <= 0 {
        return;
    }

    if let Ok(layout) = std::alloc::Layout::from_size_align(len as usize, 1) {
        unsafe {
            std::alloc::dealloc(ptr as *mut u8, layout);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn correo_detail_formatter(ptr: i32, len: i32) -> i64 {
    let request = read_request(ptr, len);
    let response = match request
        .and_then(|bytes| serde_json::from_slice::<DetailFormatterRequest>(bytes).ok())
    {
        Some(request) => detail_response(request.bytes),
        None => DetailFormatterResponse {
            abi_version: ABI_VERSION,
            output: FormattedDetailDto {
                format: DetailFormatDto::PlainText,
                text: String::new(),
                diagnostics: vec![warning("Invalid formatter request; returned empty text.")],
            },
        },
    };
    write_response(&response)
}

#[cfg(target_arch = "wasm32")]
fn read_request<'a>(ptr: i32, len: i32) -> Option<&'a [u8]> {
    if ptr <= 0 || len < 0 {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) })
}

#[cfg(target_arch = "wasm32")]
fn detail_response(bytes: Vec<u8>) -> DetailFormatterResponse {
    match format_xml_bytes(bytes) {
        Ok(output) => output.into_response(),
        Err(_) => DetailFormatterResponse {
            abi_version: ABI_VERSION,
            output: FormattedDetailDto {
                format: DetailFormatDto::PlainText,
                text: String::new(),
                diagnostics: vec![warning("Input is not valid UTF-8; returned empty text.")],
            },
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn write_response(response: &DetailFormatterResponse) -> i64 {
    let Ok(bytes) = serde_json::to_vec(response) else {
        return 0;
    };
    let len = bytes.len();
    let ptr = correomqtt_alloc(len as i32);
    if ptr <= 0 {
        return 0;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, len);
    }
    ((ptr as u64) << 32 | len as u64) as i64
}

#[cfg(target_arch = "wasm32")]
impl XmlFormatOutput {
    fn into_response(self) -> DetailFormatterResponse {
        DetailFormatterResponse {
            abi_version: ABI_VERSION,
            output: FormattedDetailDto {
                format: self.format.into(),
                text: self.text,
                diagnostics: self
                    .diagnostics
                    .into_iter()
                    .map(XmlFormatDiagnostic::into_dto)
                    .collect(),
            },
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<XmlDetailFormat> for DetailFormatDto {
    fn from(format: XmlDetailFormat) -> Self {
        match format {
            XmlDetailFormat::Xml => Self::Xml,
            XmlDetailFormat::PlainText => Self::PlainText,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl XmlFormatDiagnostic {
    fn into_dto(self) -> HookDiagnosticDto {
        HookDiagnosticDto {
            severity: self.severity.into(),
            message: self.message,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<XmlFormatDiagnosticSeverity> for HookDiagnosticSeverityDto {
    fn from(severity: XmlFormatDiagnosticSeverity) -> Self {
        match severity {
            XmlFormatDiagnosticSeverity::Warning => Self::Warning,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn warning(message: &str) -> HookDiagnosticDto {
    HookDiagnosticDto {
        severity: HookDiagnosticSeverityDto::Warning,
        message: message.to_owned(),
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
struct DetailFormatterRequest {
    bytes: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Serialize)]
struct DetailFormatterResponse {
    abi_version: u16,
    output: FormattedDetailDto,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Serialize)]
struct FormattedDetailDto {
    format: DetailFormatDto,
    text: String,
    diagnostics: Vec<HookDiagnosticDto>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum DetailFormatDto {
    Xml,
    PlainText,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Serialize)]
struct HookDiagnosticDto {
    severity: HookDiagnosticSeverityDto,
    message: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum HookDiagnosticSeverityDto {
    Warning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_xml_like_payload() {
        let output = format_xml_bytes(br#"<root><item id="1">ok</item></root>"#.to_vec()).unwrap();

        assert_eq!(output.format, XmlDetailFormat::Xml);
        assert!(output.text.contains("  <item id=\"1\">"));
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn returns_plain_text_with_warning_for_non_xml() {
        let output = format_xml_bytes(b"not xml".to_vec()).unwrap();

        assert_eq!(output.format, XmlDetailFormat::PlainText);
        assert_eq!(output.text, "not xml");
        assert_eq!(output.diagnostics.len(), 1);
    }
}
