use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::redaction::redact_sensitive;

pub type MqttResult<T> = Result<T, MqttError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MqttError {
    #[error("MQTT authentication failed: {detail}")]
    Authentication { detail: ErrorDetail },
    #[error("MQTT connection failed: {detail}")]
    Connect { detail: ErrorDetail },
    #[error("MQTT protocol error: {detail}")]
    Protocol { detail: ErrorDetail },
    #[error("MQTT TLS error: {detail}")]
    Tls { detail: ErrorDetail },
    #[error("MQTT SSH tunnel {failure} error: {detail}")]
    Ssh {
        failure: SshFailureKind,
        detail: ErrorDetail,
    },
    #[error("MQTT I/O error: {detail}")]
    Io { detail: ErrorDetail },
    #[error("MQTT operation cancelled")]
    Cancelled,
    #[error("MQTT session is disconnected")]
    Disconnected,
    #[error("invalid MQTT options: {detail}")]
    InvalidOptions { detail: ErrorDetail },
}

impl MqttError {
    pub fn auth(detail: impl Into<String>) -> Self {
        Self::Authentication {
            detail: ErrorDetail::new(detail),
        }
    }

    pub fn connect(detail: impl Into<String>) -> Self {
        Self::Connect {
            detail: ErrorDetail::new(detail),
        }
    }

    pub fn protocol(detail: impl Into<String>) -> Self {
        Self::Protocol {
            detail: ErrorDetail::new(detail),
        }
    }

    pub fn tls(detail: impl Into<String>) -> Self {
        Self::Tls {
            detail: ErrorDetail::new(detail),
        }
    }

    pub fn ssh(detail: impl Into<String>) -> Self {
        Self::ssh_failure(SshFailureKind::General, detail)
    }

    pub fn ssh_failure(failure: SshFailureKind, detail: impl Into<String>) -> Self {
        Self::Ssh {
            failure,
            detail: ErrorDetail::new(detail),
        }
    }

    pub fn io(detail: impl Into<String>) -> Self {
        Self::Io {
            detail: ErrorDetail::new(detail),
        }
    }

    pub fn invalid_options(detail: impl Into<String>) -> Self {
        Self::InvalidOptions {
            detail: ErrorDetail::new(detail),
        }
    }

    pub fn kind(&self) -> MqttErrorKind {
        match self {
            Self::Authentication { .. } => MqttErrorKind::Authentication,
            Self::Connect { .. } => MqttErrorKind::Connect,
            Self::Protocol { .. } => MqttErrorKind::Protocol,
            Self::Tls { .. } => MqttErrorKind::Tls,
            Self::Ssh { .. } => MqttErrorKind::Ssh,
            Self::Io { .. } => MqttErrorKind::Io,
            Self::Cancelled => MqttErrorKind::Cancelled,
            Self::Disconnected => MqttErrorKind::Disconnected,
            Self::InvalidOptions { .. } => MqttErrorKind::InvalidOptions,
        }
    }

    pub fn to_report(&self) -> MqttErrorReport {
        MqttErrorReport {
            kind: self.kind(),
            message: self.to_string(),
        }
    }

    pub fn diagnostic_message(&self) -> String {
        self.to_report().message
    }
}

impl From<std::io::Error> for MqttError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorDetail {
    message: String,
}

impl ErrorDetail {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: redact_sensitive(&message.into()),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for ErrorDetail {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshFailureKind {
    General,
    Connect,
    Auth,
    Bind,
    RemoteConnect,
    Teardown,
}

impl std::fmt::Display for SshFailureKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::General => "general",
            Self::Connect => "connect",
            Self::Auth => "auth",
            Self::Bind => "bind",
            Self::RemoteConnect => "remote_connect",
            Self::Teardown => "teardown",
        };
        formatter.write_str(label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MqttErrorKind {
    Authentication,
    Connect,
    Protocol,
    Tls,
    Ssh,
    Io,
    Cancelled,
    Disconnected,
    InvalidOptions,
}

impl std::fmt::Display for MqttErrorKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Authentication => "authentication",
            Self::Connect => "connect",
            Self::Protocol => "protocol",
            Self::Tls => "tls",
            Self::Ssh => "ssh",
            Self::Io => "io",
            Self::Cancelled => "cancelled",
            Self::Disconnected => "disconnected",
            Self::InvalidOptions => "invalid_options",
        };
        formatter.write_str(label)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MqttErrorReport {
    pub kind: MqttErrorKind,
    pub message: String,
}
