use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use tracing_subscriber::EnvFilter;

pub fn install_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("correo=info,warn"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .try_init();
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub occurred_at: OffsetDateTime,
}

impl Diagnostic {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            message: message.into(),
            occurred_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            occurred_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            occurred_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn redacted(mut self) -> Self {
        self.message = redact_sensitive(&self.message);
        self
    }
}

pub fn redact_sensitive(message: &str) -> String {
    let redacted = [
        "password=",
        "password:",
        "passwd=",
        "pwd=",
        "secret=",
        "secret:",
        "token=",
        "token:",
        "api_key=",
        "apikey=",
        "private_key=",
        "key_material=",
        "export_password=",
    ]
    .into_iter()
    .fold(message.to_owned(), redact_after_marker);

    redact_uri_userinfo(&redacted)
}

fn redact_after_marker(message: String, marker: &str) -> String {
    let lower = message.to_ascii_lowercase();
    let mut output = String::with_capacity(message.len());
    let mut index = 0;

    while let Some(relative_start) = lower[index..].find(marker) {
        let marker_start = index + relative_start;
        let value_start = marker_start + marker.len();
        output.push_str(&message[index..value_start]);
        output.push_str("[REDACTED]");
        index = value_end(&message, value_start);
    }

    output.push_str(&message[index..]);
    output
}

fn value_end(message: &str, start: usize) -> usize {
    let mut end = start;
    for (offset, character) in message[start..].char_indices() {
        if character.is_whitespace() || matches!(character, ',' | ';' | ')' | ']' | '}') {
            break;
        }
        end = start + offset + character.len_utf8();
    }
    end
}

fn redact_uri_userinfo(message: &str) -> String {
    let mut redacted = message.to_owned();
    for scheme in ["mqtt://", "mqtts://", "tcp://", "ssl://", "ws://", "wss://"] {
        redacted = redact_scheme_userinfo(redacted, scheme);
    }
    redacted
}

fn redact_scheme_userinfo(message: String, scheme: &str) -> String {
    let lower = message.to_ascii_lowercase();
    let mut output = String::with_capacity(message.len());
    let mut index = 0;

    while let Some(relative_start) = lower[index..].find(scheme) {
        let scheme_start = index + relative_start;
        let authority_start = scheme_start + scheme.len();
        output.push_str(&message[index..authority_start]);

        let authority_end = message[authority_start..]
            .find(['/', '?', '#'])
            .map(|offset| authority_start + offset)
            .unwrap_or(message.len());
        let authority = &message[authority_start..authority_end];
        if let Some(at) = authority.find('@') {
            output.push_str("[REDACTED]");
            output.push_str(&authority[at..]);
        } else {
            output.push_str(authority);
        }
        index = authority_end;
    }

    output.push_str(&message[index..]);
    output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl DiagnosticSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Error)]
pub enum DiagnosticsError {
    #[error("diagnostics log could not be initialized")]
    LogInitialization,
}

#[cfg(test)]
mod tests {
    use crate::redact_sensitive;

    #[test]
    fn redacts_key_value_markers_and_uri_userinfo() {
        let message = redact_sensitive(
            "auth password=hunter2 token:abcd mqtt://user:secret@broker.local/topic",
        );

        assert!(message.contains("password=[REDACTED]"));
        assert!(message.contains("token:[REDACTED]"));
        assert!(message.contains("mqtt://[REDACTED]@broker.local/topic"));
        assert!(!message.contains("hunter2"));
        assert!(!message.contains("secret@"));
    }
}
