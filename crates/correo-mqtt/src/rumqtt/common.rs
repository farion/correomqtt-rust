use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::stream::BoxStream;
use futures::{stream, StreamExt};
use tokio::sync::{broadcast, oneshot};

use crate::{
    IncomingMessage, MqttConnectionOptions, MqttError, MqttResult, MqttSessionEvent, SessionState,
};

pub(crate) type SharedState = Arc<Mutex<SessionState>>;
pub(crate) type StartupSignal = oneshot::Sender<MqttResult<()>>;

#[derive(Clone)]
pub(crate) struct SessionChannels {
    state: SharedState,
    events: broadcast::Sender<MqttSessionEvent>,
    incoming: broadcast::Sender<Result<IncomingMessage, MqttError>>,
}

impl SessionChannels {
    pub(crate) fn new() -> Self {
        let (events, _) = broadcast::channel(256);
        let (incoming, _) = broadcast::channel(256);
        Self {
            state: Arc::new(Mutex::new(SessionState::Disconnected)),
            events,
            incoming,
        }
    }

    pub(crate) fn current_state(&self) -> SessionState {
        self.state
            .lock()
            .map(|state| state.clone())
            .unwrap_or_else(|_| SessionState::Faulted {
                error: MqttError::io("MQTT session state lock was poisoned").to_report(),
            })
    }

    pub(crate) fn event_stream(&self) -> BoxStream<'static, MqttSessionEvent> {
        broadcast_stream(self.events.subscribe())
    }

    pub(crate) fn incoming_stream(&self) -> BoxStream<'static, Result<IncomingMessage, MqttError>> {
        broadcast_stream(self.incoming.subscribe())
    }

    pub(crate) fn set_state(&self, state: SessionState) {
        if let Ok(mut current) = self.state.lock() {
            *current = state.clone();
        }
        let _ = self.events.send(MqttSessionEvent::StateChanged(state));
    }

    pub(crate) fn report_error(&self, error: MqttError) {
        let _ = self.events.send(MqttSessionEvent::Error(error.to_report()));
    }

    pub(crate) fn report_incoming(&self, message: IncomingMessage) {
        let _ = self.incoming.send(Ok(message.clone()));
        let _ = self.events.send(MqttSessionEvent::Incoming(message));
    }

    pub(crate) fn report_incoming_error(&self, error: MqttError) {
        let _ = self.incoming.send(Err(error.clone()));
        self.report_error(error);
    }

    pub(crate) fn report_published(&self, event: MqttSessionEvent) {
        let _ = self.events.send(event);
    }
}

pub(crate) fn finish_startup(startup: &mut Option<StartupSignal>, result: MqttResult<()>) {
    if let Some(sender) = startup.take() {
        let _ = sender.send(result);
    }
}

pub(crate) fn client_id(options: &MqttConnectionOptions) -> MqttResult<String> {
    match (&options.client_id, options.clean_start) {
        (Some(client_id), _) => Ok(client_id.clone()),
        (None, true) => Ok(String::new()),
        (None, false) => Err(MqttError::invalid_options(
            "persistent MQTT sessions require an explicit client id",
        )),
    }
}

pub(crate) fn keep_alive_seconds(duration: Duration) -> MqttResult<u16> {
    let mut seconds = duration.as_secs();
    if duration.subsec_nanos() > 0 {
        seconds = seconds.saturating_add(1);
    }

    u16::try_from(seconds).map_err(|_| {
        MqttError::invalid_options("MQTT keep alive must fit in a 16-bit seconds value")
    })
}

fn broadcast_stream<T>(receiver: broadcast::Receiver<T>) -> BoxStream<'static, T>
where
    T: Clone + Send + 'static,
{
    stream::unfold(receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(item) => return Some((item, receiver)),
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    })
    .boxed()
}
