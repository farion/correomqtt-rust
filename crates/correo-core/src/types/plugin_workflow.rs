use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use crate::PluginMarketplaceRow;
use crate::{PluginDiagnosticSeverity, PluginHookKind, QosLevel};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageDetailSnapshot {
    pub selected_transform_plugin_id: Option<String>,
    pub selected_formatter_plugin_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormattedMessageDetail {
    pub format: MessageDetailFormat,
    pub text: String,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<MessageDiagnosticRow>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDetailFormat {
    #[default]
    PlainText,
    Json,
    Xml,
    Hex,
}

impl MessageDetailFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::PlainText => "Plain text",
            Self::Json => "JSON",
            Self::Xml => "XML",
            Self::Hex => "Hex",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageDiagnosticRow {
    pub severity: PluginDiagnosticSeverity,
    pub hook: Option<PluginHookKind>,
    pub plugin_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: QosLevel,
    pub retained: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluginHookCall {
    pub plugin_id: String,
    pub hook: PluginHookKind,
    pub target: String,
    pub config: Value,
    pub input: PluginHookInput,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginHookInput {
    Message(PluginMessage),
    DetailBytes {
        bytes: Vec<u8>,
        content_type: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginHookOutput {
    MessageTransform(MessageTransform),
    Validation(PluginValidation),
    DetailBytes(DetailBytesOutput),
    DetailFormat(FormattedMessageDetail),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageTransform {
    Unchanged,
    Replace(PluginMessage),
    Drop { reason: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginValidation {
    Valid,
    Warning { message: String },
    Block { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailBytesOutput {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginHookErrorKind {
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginHookError {
    pub kind: PluginHookErrorKind,
    pub message: String,
}

impl PluginHookError {
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            kind: PluginHookErrorKind::Failed,
            message: message.into(),
        }
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self {
            kind: PluginHookErrorKind::Cancelled,
            message: message.into(),
        }
    }
}

impl fmt::Display for PluginHookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

pub trait PluginHookExecutor: fmt::Debug + Send + Sync + 'static {
    fn execute(&self, call: PluginHookCall) -> Result<PluginHookOutput, PluginHookError>;
}

pub trait PluginInstaller: fmt::Debug + Send + Sync + 'static {
    fn install(&self, plugin: &PluginMarketplaceRow) -> Result<String, String>;
    fn uninstall(&self, plugin_id: &str) -> Result<(), String>;
}

#[derive(Debug, Default)]
pub struct NoopPluginHookExecutor;

impl PluginHookExecutor for NoopPluginHookExecutor {
    fn execute(&self, call: PluginHookCall) -> Result<PluginHookOutput, PluginHookError> {
        match (call.hook, call.input) {
            (PluginHookKind::IncomingTransform | PluginHookKind::OutgoingTransform, _) => Ok(
                PluginHookOutput::MessageTransform(MessageTransform::Unchanged),
            ),
            (PluginHookKind::Validator, _) => {
                Ok(PluginHookOutput::Validation(PluginValidation::Valid))
            }
            (
                PluginHookKind::DetailTransform,
                PluginHookInput::DetailBytes {
                    bytes,
                    content_type,
                },
            ) => Ok(PluginHookOutput::DetailBytes(DetailBytesOutput {
                bytes,
                content_type,
            })),
            (
                PluginHookKind::DetailFormatter,
                PluginHookInput::DetailBytes {
                    bytes,
                    content_type,
                },
            ) => Ok(PluginHookOutput::DetailFormat(FormattedMessageDetail {
                format: MessageDetailFormat::PlainText,
                text: String::from_utf8_lossy(&bytes).into_owned(),
                content_type,
                diagnostics: Vec::new(),
            })),
            _ => Err(PluginHookError::failed(
                "plugin hook input did not match hook kind",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginWorkflowEvent {
    PublishBlocked {
        message: String,
    },
    PublishWarning {
        message: String,
    },
    HookDiagnostic(PluginHookDiagnosticEvent),
    MessageDiagnosticsAppended {
        message_id: u32,
        diagnostics: Vec<MessageDiagnosticRow>,
    },
    MessageDetailUpdated {
        message_id: u32,
        detail: FormattedMessageDetail,
    },
    MessageDetailCleared {
        message_id: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginHookDiagnosticEvent {
    pub plugin_id: String,
    pub hook: Option<PluginHookKind>,
    pub severity: PluginDiagnosticSeverity,
    pub message: String,
    pub detail: String,
    pub mark_hook_failed: bool,
}
