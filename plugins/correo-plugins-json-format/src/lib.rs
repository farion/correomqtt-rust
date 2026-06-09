use serde_json::Value;
use std::string::FromUtf8Error;

pub const ABI_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonFormatOutput {
    pub format: JsonDetailFormat,
    pub text: String,
    pub diagnostics: Vec<JsonFormatDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonDetailFormat {
    Json,
    PlainText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonFormatDiagnostic {
    pub severity: JsonFormatDiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonFormatDiagnosticSeverity {
    Warning,
}

#[derive(Debug)]
pub struct JsonFormatError {
    source: FromUtf8Error,
}

impl JsonFormatError {
    pub fn into_source(self) -> FromUtf8Error {
        self.source
    }
}

impl From<FromUtf8Error> for JsonFormatError {
    fn from(source: FromUtf8Error) -> Self {
        Self { source }
    }
}

pub fn format_json_bytes(bytes: Vec<u8>) -> Result<JsonFormatOutput, JsonFormatError> {
    let text = String::from_utf8(bytes)?;
    Ok(match serde_json::from_str::<Value>(&text) {
        Ok(value) => JsonFormatOutput {
            format: JsonDetailFormat::Json,
            text: serde_json::to_string_pretty(&value).expect("serde_json::Value serializes"),
            diagnostics: Vec::new(),
        },
        Err(_) => JsonFormatOutput {
            format: JsonDetailFormat::PlainText,
            text,
            diagnostics: vec![JsonFormatDiagnostic {
                severity: JsonFormatDiagnosticSeverity::Warning,
                message: "Input is not valid JSON; returned unchanged.".to_owned(),
            }],
        },
    })
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
    match format_json_bytes(bytes) {
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
impl JsonFormatOutput {
    fn into_response(self) -> DetailFormatterResponse {
        DetailFormatterResponse {
            abi_version: ABI_VERSION,
            output: FormattedDetailDto {
                format: self.format.into(),
                text: self.text,
                diagnostics: self
                    .diagnostics
                    .into_iter()
                    .map(JsonFormatDiagnostic::into_dto)
                    .collect(),
            },
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<JsonDetailFormat> for DetailFormatDto {
    fn from(format: JsonDetailFormat) -> Self {
        match format {
            JsonDetailFormat::Json => Self::Json,
            JsonDetailFormat::PlainText => Self::PlainText,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl JsonFormatDiagnostic {
    fn into_dto(self) -> HookDiagnosticDto {
        HookDiagnosticDto {
            severity: self.severity.into(),
            message: self.message,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<JsonFormatDiagnosticSeverity> for HookDiagnosticSeverityDto {
    fn from(severity: JsonFormatDiagnosticSeverity) -> Self {
        match severity {
            JsonFormatDiagnosticSeverity::Warning => Self::Warning,
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
#[derive(Debug, serde::Deserialize)]
struct DetailFormatterRequest {
    bytes: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, serde::Serialize)]
struct DetailFormatterResponse {
    abi_version: u16,
    output: FormattedDetailDto,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, serde::Serialize)]
struct FormattedDetailDto {
    format: DetailFormatDto,
    text: String,
    diagnostics: Vec<HookDiagnosticDto>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum DetailFormatDto {
    Json,
    PlainText,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, serde::Serialize)]
struct HookDiagnosticDto {
    severity: HookDiagnosticSeverityDto,
    message: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum HookDiagnosticSeverityDto {
    Warning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_valid_json_payload() {
        let output = format_json_bytes(br#"{"ok":true,"items":[1,2]}"#.to_vec()).unwrap();

        assert_eq!(output.format, JsonDetailFormat::Json);
        assert!(output.text.contains("\"items\": ["));
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn returns_plain_text_with_warning_for_invalid_json() {
        let output = format_json_bytes(b"not json".to_vec()).unwrap();

        assert_eq!(output.format, JsonDetailFormat::PlainText);
        assert_eq!(output.text, "not json");
        assert_eq!(output.diagnostics.len(), 1);
    }
}
