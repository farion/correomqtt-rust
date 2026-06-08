use thiserror::Error;

pub type ScriptingResult<T> = Result<T, ScriptingError>;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScriptingError {
    #[error("script host API failed: {0}")]
    HostApi(String),
    #[error("JavaScript guest failed: {0}")]
    JavaScriptGuest(String),
    #[error("script execution was cancelled")]
    Cancelled,
    #[error("script MQTT operation failed: {0}")]
    MqttOperation(String),
    #[error("script runtime failed: {0}")]
    Runtime(String),
}
