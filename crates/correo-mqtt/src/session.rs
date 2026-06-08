use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::{
    IncomingMessage, MqttConnectionOptions, MqttError, MqttResult, MqttSessionEvent,
    PublishRequest, SessionState, Subscription, UnsubscribeRequest,
};

#[async_trait]
pub trait MqttSession: Send + Sync {
    async fn connect(&mut self, options: MqttConnectionOptions) -> MqttResult<()>;

    async fn disconnect(&mut self) -> MqttResult<()>;

    async fn publish(&mut self, request: PublishRequest) -> MqttResult<()>;

    async fn subscribe(&mut self, subscription: Subscription) -> MqttResult<()>;

    async fn unsubscribe(&mut self, request: UnsubscribeRequest) -> MqttResult<()>;

    fn current_state(&self) -> SessionState;

    fn events(&mut self) -> BoxStream<'static, MqttSessionEvent>;

    fn incoming(&mut self) -> BoxStream<'static, Result<IncomingMessage, MqttError>>;
}
