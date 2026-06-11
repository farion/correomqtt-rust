#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

mod topics;
use topics::{metadata_for_topic, SysTopicMetadata};

pub const ABI_VERSION: u16 = 1;
pub const PLUGIN_ID: &str = "org.correomqtt.plugins.system-topic";
pub const LEGACY_PLUGIN_ID: &str = "systopic";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysTopicFormatOutput {
    pub format: SysTopicDetailFormat,
    pub text: String,
    pub diagnostics: Vec<SysTopicFormatDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysTopicDetailFormat {
    PlainText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysTopicFormatDiagnostic {
    pub severity: SysTopicFormatDiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysTopicFormatDiagnosticSeverity {
    Info,
    Warning,
}

pub fn format_sys_topic_detail(topic: Option<&str>, bytes: Vec<u8>) -> SysTopicFormatOutput {
    let payload = String::from_utf8_lossy(&bytes);
    let Some(topic) = topic.map(str::trim).filter(|topic| !topic.is_empty()) else {
        return plain(
            payload.into_owned(),
            vec![warning(
                "System topic formatter needs context.subscription_topic; returned payload only.",
            )],
        );
    };

    if !topic.starts_with("$SYS/") {
        return plain(
            payload.into_owned(),
            vec![warning(
                "Topic is not a $SYS broker metric; returned payload only.",
            )],
        );
    }

    match metadata_for_topic(topic) {
        Some(metadata) => plain(detail_text(topic, &payload, metadata), Vec::new()),
        None => plain(
            generic_detail_text(topic, &payload),
            vec![info(
                "Unrecognized $SYS topic; returned generic system-topic detail.",
            )],
        ),
    }
}

fn detail_text(topic: &str, payload: &str, metadata: SysTopicMetadata<'_>) -> String {
    let mut text = format!(
        "{}\n{}\n\nTopic: {}\n",
        metadata.label, metadata.description, topic
    );
    if let Some(window) = metadata.window {
        text.push_str("Window: ");
        text.push_str(window);
        text.push('\n');
    }
    text.push_str("Value: ");
    text.push_str(payload);
    text
}

fn generic_detail_text(topic: &str, payload: &str) -> String {
    format!(
        "System topic\nUnrecognized $SYS broker metric.\n\nTopic: {}\nValue: {}",
        topic, payload
    )
}

fn plain(text: String, diagnostics: Vec<SysTopicFormatDiagnostic>) -> SysTopicFormatOutput {
    SysTopicFormatOutput {
        format: SysTopicDetailFormat::PlainText,
        text,
        diagnostics,
    }
}

fn info(message: &str) -> SysTopicFormatDiagnostic {
    SysTopicFormatDiagnostic {
        severity: SysTopicFormatDiagnosticSeverity::Info,
        message: message.to_owned(),
    }
}

fn warning(message: &str) -> SysTopicFormatDiagnostic {
    SysTopicFormatDiagnostic {
        severity: SysTopicFormatDiagnosticSeverity::Warning,
        message: message.to_owned(),
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
pub extern "C" fn correo_detail_formatter(ptr: i32, len: i32) -> i64 {
    let response = read_request(ptr, len)
        .and_then(|bytes| serde_json::from_slice::<DetailFormatterRequest>(bytes).ok())
        .filter(|request| request.abi_version == ABI_VERSION)
        .map(detail_response)
        .unwrap_or_else(invalid_request_response);
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
fn detail_response(request: DetailFormatterRequest) -> DetailFormatterResponse {
    let topic = request.context.subscription_topic.as_deref();
    format_sys_topic_detail(topic, request.bytes).into_response()
}

#[cfg(target_arch = "wasm32")]
fn invalid_request_response() -> DetailFormatterResponse {
    SysTopicFormatOutput {
        format: SysTopicDetailFormat::PlainText,
        text: String::new(),
        diagnostics: vec![SysTopicFormatDiagnostic {
            severity: SysTopicFormatDiagnosticSeverity::Warning,
            message: "Invalid formatter request; returned empty text.".to_owned(),
        }],
    }
    .into_response()
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
impl SysTopicFormatOutput {
    fn into_response(self) -> DetailFormatterResponse {
        DetailFormatterResponse {
            abi_version: ABI_VERSION,
            output: FormattedDetailDto {
                format: self.format.into(),
                text: self.text,
                diagnostics: self
                    .diagnostics
                    .into_iter()
                    .map(SysTopicFormatDiagnostic::into_dto)
                    .collect(),
            },
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<SysTopicDetailFormat> for DetailFormatDto {
    fn from(format: SysTopicDetailFormat) -> Self {
        match format {
            SysTopicDetailFormat::PlainText => Self::PlainText,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl SysTopicFormatDiagnostic {
    fn into_dto(self) -> HookDiagnosticDto {
        HookDiagnosticDto {
            severity: self.severity.into(),
            message: self.message,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<SysTopicFormatDiagnosticSeverity> for HookDiagnosticSeverityDto {
    fn from(severity: SysTopicFormatDiagnosticSeverity) -> Self {
        match severity {
            SysTopicFormatDiagnosticSeverity::Info => Self::Info,
            SysTopicFormatDiagnosticSeverity::Warning => Self::Warning,
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
struct DetailFormatterRequest {
    abi_version: u16,
    #[serde(default)]
    context: HookContext,
    bytes: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default, Deserialize)]
struct HookContext {
    #[serde(default)]
    subscription_topic: Option<String>,
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
    Info,
    Warning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_system_topic_with_label_description_and_value() {
        let output = format_sys_topic_detail(Some("$SYS/broker/clients/connected"), b"7".to_vec());

        assert_eq!(output.format, SysTopicDetailFormat::PlainText);
        assert!(output.text.contains("Connected clients"));
        assert!(output.text.contains("currently connected clients"));
        assert!(output.text.contains("Topic: $SYS/broker/clients/connected"));
        assert!(output.text.contains("Value: 7"));
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn formats_aggregated_system_topic_window() {
        let output = format_sys_topic_detail(
            Some("$SYS/broker/load/messages/received/15min"),
            b"42".to_vec(),
        );

        assert!(output.text.contains("Aggregated messages received"));
        assert!(output.text.contains("Window: 15min"));
        assert!(output.text.contains("Value: 42"));
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn returns_generic_detail_for_unknown_system_topic() {
        let output = format_sys_topic_detail(Some("$SYS/broker/custom"), b"ok".to_vec());

        assert!(output.text.contains("System topic"));
        assert_eq!(
            output.diagnostics[0].severity,
            SysTopicFormatDiagnosticSeverity::Info
        );
    }

    #[test]
    fn leaves_non_system_topics_unchanged_with_warning() {
        let output = format_sys_topic_detail(Some("devices/demo"), b"plain".to_vec());

        assert_eq!(output.text, "plain");
        assert_eq!(
            output.diagnostics[0].severity,
            SysTopicFormatDiagnosticSeverity::Warning
        );
    }
}
