use correo_mqtt::{ConnectionId, Qos};
use correo_storage::current::{Message, MessageType, PublishStatus};

use crate::{AppModel, HistoryPersistenceCommand, MqttEvent};

impl AppModel {
    pub(crate) fn history_commands_for_mqtt_event(
        &self,
        event: &MqttEvent,
    ) -> Vec<HistoryPersistenceCommand> {
        match event {
            MqttEvent::Published {
                connection_id,
                topic,
                payload,
                qos,
                retain,
            } => vec![HistoryPersistenceCommand::RecordPublish {
                connection_id: self.storage_connection_id(*connection_id),
                message: Message {
                    topic: topic.as_str().to_owned(),
                    payload: Some(String::from_utf8_lossy(payload).into_owned()),
                    retained: *retain,
                    qos: Some(storage_qos(*qos)),
                    date_time: Some("now".to_owned()),
                    message_id: None,
                    message_type: Some(MessageType::Outgoing),
                    publish_status: Some(PublishStatus::Succeeded),
                },
            }],
            MqttEvent::Subscribed {
                connection_id,
                subscription,
            } => vec![HistoryPersistenceCommand::RecordSubscription {
                connection_id: self.storage_connection_id(*connection_id),
                topic: subscription.topic_filter.as_str().to_owned(),
                hidden: false,
            }],
            _ => Vec::new(),
        }
    }

    fn storage_connection_id(&self, connection_id: ConnectionId) -> String {
        self.storage_connection_ids
            .get(&connection_id)
            .cloned()
            .unwrap_or_else(|| connection_id.to_string())
    }
}

fn storage_qos(qos: Qos) -> correo_storage::current::Qos {
    match qos {
        Qos::AtMostOnce => correo_storage::current::Qos::AtMostOnce,
        Qos::AtLeastOnce => correo_storage::current::Qos::AtLeastOnce,
        Qos::ExactlyOnce => correo_storage::current::Qos::ExactlyOnce,
    }
}
