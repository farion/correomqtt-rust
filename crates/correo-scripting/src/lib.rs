mod bindings;
mod cancellation;
mod client_args;
mod client_bindings;
mod client_callbacks;
mod client_results;
mod error;
mod executor;
mod host;
mod metadata;

pub use cancellation::{ScriptCancellationHandle, ScriptCancellationToken};
pub use error::{ScriptingError, ScriptingResult};
pub use executor::{ScriptExecutionOutcome, ScriptExecutionRequest, ScriptRuntime};
pub use host::{
    NoopScriptHost, ScriptHost, ScriptLogEntry, ScriptLogLevel, ScriptMqttClient,
    ScriptPublishRequest,
};
pub use metadata::{ScriptExecutionId, ScriptExecutionMetadata, ScriptExecutionStatus};

pub const COMPATIBILITY_ALIASES: &[&str] = &[
    "clientFactory.getBlockingClient()",
    "clientFactory.getAsyncClient()",
    "clientFactory.getPromiseClient()",
    "new ClientFactory().getPromiseClient()",
    "client.toPromised()",
    "client.toBlocking()",
    "sleep(ms)",
    "logger",
    "plugins.base64.decode(payload)",
    "plugins.base64.encode(payload)",
    "client.onIncomingMessage(topicFilter, callback)",
    "queue.process()",
    "queue.jumpOut()",
    "join()",
];
