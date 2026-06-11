use std::sync::Arc;

use correo_mqtt::{MqttError, PublishRequest, Qos};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{ScriptCancellationHandle, ScriptCancellationToken, ScriptExecutionMetadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptLogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptLogEntry {
    pub level: ScriptLogLevel,
    pub message: String,
    pub occurred_at: OffsetDateTime,
}

impl ScriptLogEntry {
    pub fn new(level: ScriptLogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            occurred_at: OffsetDateTime::now_utc(),
        }
    }
}

pub trait ScriptHost: Send + Sync {
    fn execution_metadata_changed(&self, _metadata: &ScriptExecutionMetadata) {}

    fn log(&self, entry: ScriptLogEntry);

    fn mqtt_client(&self) -> Option<Arc<dyn ScriptMqttClient>> {
        None
    }
}

pub trait ScriptMqttClient: Send + Sync {
    fn connect(&self, _cancellation: &ScriptCancellationToken) -> Result<(), MqttError> {
        Ok(())
    }

    fn disconnect(&self, _cancellation: &ScriptCancellationToken) -> Result<(), MqttError> {
        Ok(())
    }

    fn publish(
        &self,
        request: ScriptPublishRequest,
        cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError>;

    fn subscribe(
        &self,
        topic_filter: String,
        qos: Qos,
        cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError>;

    fn unsubscribe(
        &self,
        topic_filter: String,
        cancellation: &ScriptCancellationToken,
    ) -> Result<(), MqttError>;

    fn cancellation_handle(&self) -> Option<Arc<dyn ScriptCancellationHandle>> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptPublishRequest {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: Qos,
    pub retain: bool,
}

impl ScriptPublishRequest {
    pub fn new(topic: impl Into<String>, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
            qos: Qos::AtMostOnce,
            retain: false,
        }
    }

    pub fn into_publish_request(self) -> Result<PublishRequest, MqttError> {
        PublishRequest::new(self.topic.as_str(), self.payload, self.qos, self.retain)
    }
}

#[derive(Debug, Default)]
pub struct NoopScriptHost;

impl ScriptHost for NoopScriptHost {
    fn log(&self, _entry: ScriptLogEntry) {}
}
