use correo_mqtt::{IncomingMessage, Qos, SessionState, TopicFilter, TopicName};

use crate::{
    AppModel, ConnectDisabledReason, ConnectionState, Diagnostic, MessageRow, MqttCommand,
    MqttEvent, MqttFailure, MqttOperation, PublishHistoryRow, QosLevel, SubscriptionRow,
    WorkflowFeedback,
};

impl AppModel {
    pub(super) fn update_publish_topic(&mut self, topic: String) {
        self.snapshot.workbench.publish.topic = topic;
        self.refresh_publish_validation();
    }

    pub(super) fn update_publish_payload(&mut self, payload: String) {
        self.snapshot.workbench.publish.payload = payload;
        self.refresh_publish_validation();
    }

    pub(super) fn update_publish_qos(&mut self, qos: QosLevel) {
        self.snapshot.workbench.publish.qos = qos;
        self.refresh_publish_validation();
    }

    pub(super) fn update_subscribe_topic(&mut self, topic: String) {
        self.snapshot.workbench.subscribe.topic = topic;
        self.refresh_subscribe_validation();
    }

    pub(super) fn update_subscribe_qos(&mut self, qos: QosLevel) {
        self.snapshot.workbench.subscribe.qos = qos;
        self.refresh_subscribe_validation();
    }

    pub(super) fn apply_mqtt_command(&mut self, command: MqttCommand) {
        match command {
            MqttCommand::Connect { options } => {
                self.mark_command_accepted(options.connection_id, MqttOperation::Connect);
            }
            MqttCommand::Reconnect { options } => {
                self.mark_reconnecting(options.connection_id, 1);
            }
            MqttCommand::Disconnect { connection_id } => {
                self.mark_command_accepted(connection_id, MqttOperation::Disconnect);
            }
            MqttCommand::Publish { connection_id, .. } => {
                self.mark_command_accepted(connection_id, MqttOperation::Publish);
            }
            MqttCommand::Subscribe {
                connection_id,
                subscription,
            } => {
                let _ = subscription;
                self.mark_command_accepted(connection_id, MqttOperation::Subscribe);
            }
            MqttCommand::Unsubscribe {
                connection_id,
                request,
            } => {
                let _ = request;
                self.mark_command_accepted(connection_id, MqttOperation::Unsubscribe);
            }
            MqttCommand::Shutdown => {}
        }
    }

    pub(super) fn apply_mqtt_event(&mut self, event: MqttEvent) {
        match event {
            MqttEvent::CommandAccepted {
                connection_id,
                operation,
            } => self.mark_command_accepted(connection_id, operation),
            MqttEvent::Connected { connection_id } => self.mark_connected(connection_id),
            MqttEvent::Disconnected { connection_id } => self.mark_disconnected(connection_id),
            MqttEvent::Reconnecting {
                connection_id,
                attempt,
            } => self.mark_reconnecting(connection_id, attempt),
            MqttEvent::StateChanged {
                connection_id,
                state,
            } => self.apply_session_state(connection_id, state),
            MqttEvent::IncomingMessage(message) => self.add_incoming_message(message),
            MqttEvent::Published {
                connection_id,
                topic,
                payload,
                qos,
                retain,
            } => {
                self.add_publish_success(topic.as_str(), payload.len(), qos_level(qos), retain);
                self.push_diagnostic(Diagnostic::info(format!(
                    "MQTT publish completed for {} on {}.",
                    topic.as_str(),
                    connection_label(self, connection_id)
                )));
            }
            MqttEvent::Subscribed {
                connection_id,
                subscription,
            } => {
                self.add_subscription(SubscriptionRow {
                    topic_filter: subscription.topic_filter.as_str().to_owned(),
                    qos: qos_level(subscription.qos),
                    message_count: 0,
                    active: true,
                });
                self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(format!(
                    "Subscribed to {}.",
                    subscription.topic_filter.as_str()
                )));
                self.push_diagnostic(Diagnostic::info(format!(
                    "MQTT subscribe completed for {} on {}.",
                    subscription.topic_filter.as_str(),
                    connection_label(self, connection_id)
                )));
            }
            MqttEvent::Unsubscribed {
                connection_id,
                request,
            } => {
                self.remove_subscription(request.topic_filter.as_str());
                self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(format!(
                    "Unsubscribed from {}.",
                    request.topic_filter.as_str()
                )));
                self.push_diagnostic(Diagnostic::info(format!(
                    "MQTT unsubscribe completed for {} on {}.",
                    request.topic_filter.as_str(),
                    connection_label(self, connection_id)
                )));
            }
            MqttEvent::Failure(failure) => self.apply_mqtt_failure(failure),
            MqttEvent::ShutdownComplete => {
                self.push_diagnostic(Diagnostic::info("MQTT service shutdown completed."));
            }
        }
    }

    fn mark_command_accepted(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        operation: MqttOperation,
    ) {
        match operation {
            MqttOperation::Connect => {
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Connecting,
                    Some(ConnectDisabledReason::Busy),
                    "connect command queued".to_owned(),
                );
            }
            MqttOperation::Disconnect => {
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Disconnected,
                    None,
                    "disconnect command queued".to_owned(),
                );
            }
            MqttOperation::Publish => {
                self.snapshot.workbench.publish.feedback =
                    Some(WorkflowFeedback::info("Publish accepted by MQTT service."));
            }
            MqttOperation::Subscribe => {
                self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(
                    "Subscribe accepted by MQTT service.",
                ));
            }
            MqttOperation::Unsubscribe => {
                self.snapshot.workbench.subscribe.feedback = Some(WorkflowFeedback::info(
                    "Unsubscribe accepted by MQTT service.",
                ));
            }
            _ => {}
        }
    }

    fn apply_session_state(
        &mut self,
        connection_id: correo_mqtt::ConnectionId,
        state: SessionState,
    ) {
        match state {
            SessionState::Disconnected => self.mark_disconnected(connection_id),
            SessionState::Connecting => self.update_connection_state(
                connection_id,
                ConnectionState::Connecting,
                Some(ConnectDisabledReason::Busy),
                "connecting".to_owned(),
            ),
            SessionState::Connected => self.mark_connected(connection_id),
            SessionState::Disconnecting => self.update_connection_state(
                connection_id,
                ConnectionState::Disconnected,
                Some(ConnectDisabledReason::Busy),
                "disconnecting".to_owned(),
            ),
            SessionState::Reconnecting { attempt } => {
                self.mark_reconnecting(connection_id, attempt)
            }
            SessionState::Faulted { error } => {
                self.update_connection_state(
                    connection_id,
                    ConnectionState::Error,
                    None,
                    "MQTT session faulted".to_owned(),
                );
                self.push_diagnostic(Diagnostic::error(error.message));
            }
        }
    }

    fn mark_connected(&mut self, connection_id: correo_mqtt::ConnectionId) {
        self.snapshot.active_connection = Some(connection_id);
        self.update_connection_state(
            connection_id,
            ConnectionState::Connected,
            Some(ConnectDisabledReason::AlreadyConnected),
            "connected".to_owned(),
        );
    }

    fn mark_disconnected(&mut self, connection_id: correo_mqtt::ConnectionId) {
        if self.snapshot.active_connection == Some(connection_id) {
            self.snapshot.active_connection = None;
        }
        self.update_connection_state(
            connection_id,
            ConnectionState::Disconnected,
            None,
            "disconnected".to_owned(),
        );
    }

    fn mark_reconnecting(&mut self, connection_id: correo_mqtt::ConnectionId, attempt: u32) {
        self.update_connection_state(
            connection_id,
            ConnectionState::Reconnecting,
            Some(ConnectDisabledReason::Busy),
            format!("reconnect attempt {attempt}"),
        );
        self.snapshot.workbench.reconnect_status = format!("Reconnect attempt {attempt}");
    }

    fn add_incoming_message(&mut self, message: IncomingMessage) {
        let topic = message.topic.as_str().to_owned();
        let row = MessageRow {
            id: self.next_message_id(),
            topic: topic.clone(),
            timestamp: "now".to_owned(),
            qos: qos_level(message.qos),
            retained: message.retain,
            payload: message.payload.clone(),
            payload_preview: payload_preview(&message.payload),
            byte_size: message.payload.len(),
            badges: incoming_badges(&message),
            diagnostics: Vec::new(),
            formatted_detail: None,
        };
        self.snapshot.workbench.messages.insert(0, row);
        self.snapshot.workbench.selected_message_id = self
            .snapshot
            .workbench
            .messages
            .first()
            .map(|message| message.id);
        self.increment_matching_subscriptions(&topic);
        self.update_recent_message_count(message.connection_id);
    }

    fn next_message_id(&self) -> u32 {
        self.snapshot
            .workbench
            .messages
            .iter()
            .map(|message| message.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    fn increment_matching_subscriptions(&mut self, topic: &str) {
        for subscription in &mut self.snapshot.workbench.subscribe.subscriptions {
            if subscription.active && topic_matches_filter(topic, &subscription.topic_filter) {
                subscription.message_count = subscription.message_count.saturating_add(1);
            }
        }
    }

    fn update_recent_message_count(&mut self, connection_id: correo_mqtt::ConnectionId) {
        if let Some(index) = self.connection_index(connection_id) {
            self.snapshot.connections[index].recent_messages = self.snapshot.connections[index]
                .recent_messages
                .saturating_add(1);
            self.snapshot.connections[index].last_activity = "message received".to_owned();
        }
    }

    fn apply_mqtt_failure(&mut self, failure: MqttFailure) {
        let message = format!(
            "MQTT {} failed: {}",
            failure.operation.label(),
            failure.report.message
        );
        match failure.operation {
            MqttOperation::Publish => {
                self.snapshot.workbench.publish.feedback =
                    Some(WorkflowFeedback::error(message.clone()));
            }
            MqttOperation::Subscribe | MqttOperation::Unsubscribe => {
                self.snapshot.workbench.subscribe.feedback =
                    Some(WorkflowFeedback::error(message.clone()));
            }
            _ => {}
        }
        if let Some(connection_id) = failure.connection_id {
            self.update_connection_state(
                connection_id,
                ConnectionState::Error,
                None,
                format!("{} failed", failure.operation.label()),
            );
        }
        self.push_diagnostic(Diagnostic::error(message));
    }

    fn add_subscription(&mut self, row: SubscriptionRow) {
        push_recent_unique(
            &mut self.snapshot.workbench.subscribe.topic_history,
            &row.topic_filter,
        );
        if let Some(existing) = self
            .snapshot
            .workbench
            .subscribe
            .subscriptions
            .iter_mut()
            .find(|subscription| subscription.topic_filter == row.topic_filter)
        {
            existing.qos = row.qos;
            existing.active = true;
            return;
        }
        self.snapshot
            .workbench
            .subscribe
            .subscriptions
            .insert(0, row);
    }

    fn remove_subscription(&mut self, topic_filter: &str) {
        self.snapshot
            .workbench
            .subscribe
            .subscriptions
            .retain(|subscription| subscription.topic_filter != topic_filter);
    }

    fn add_publish_success(
        &mut self,
        topic: &str,
        byte_size: usize,
        qos: QosLevel,
        retained: bool,
    ) {
        push_recent_unique(&mut self.snapshot.workbench.publish.topic_history, topic);
        self.snapshot.workbench.publish.history.insert(
            0,
            PublishHistoryRow {
                topic: topic.to_owned(),
                timestamp: "now".to_owned(),
                qos,
                retained,
                byte_size,
            },
        );
        self.snapshot.workbench.publish.feedback = Some(WorkflowFeedback::info(format!(
            "Published {byte_size} bytes to {topic}."
        )));
    }

    fn refresh_publish_validation(&mut self) {
        let publish = &mut self.snapshot.workbench.publish;
        let topic = publish.topic.trim();
        publish.validation = publish_validation(topic, publish.payload.len());
        publish.valid = TopicName::new(topic).is_ok();
    }

    fn refresh_subscribe_validation(&mut self) {
        let subscribe = &mut self.snapshot.workbench.subscribe;
        let topic = subscribe.topic.trim();
        subscribe.validation = subscribe_validation(topic);
        subscribe.valid = TopicFilter::new(topic).is_ok();
    }
}

fn connection_label(model: &AppModel, connection_id: correo_mqtt::ConnectionId) -> String {
    model
        .snapshot
        .connections
        .iter()
        .find(|connection| connection.id == connection_id)
        .map(|connection| connection.name.clone())
        .unwrap_or_else(|| "unknown connection".to_owned())
}

fn qos_level(qos: Qos) -> QosLevel {
    match qos {
        Qos::AtMostOnce => QosLevel::Zero,
        Qos::AtLeastOnce => QosLevel::One,
        Qos::ExactlyOnce => QosLevel::Two,
    }
}

fn payload_preview(payload: &[u8]) -> String {
    const LIMIT: usize = 96;
    let mut preview = String::from_utf8_lossy(payload).replace(['\n', '\r'], " ");
    if preview.len() > LIMIT {
        let truncated = preview.chars().take(LIMIT).collect::<String>();
        preview = format!("{truncated}...");
    }
    preview
}

fn incoming_badges(message: &IncomingMessage) -> Vec<String> {
    let mut badges = Vec::new();
    if message.retain {
        badges.push("retained".to_owned());
    }
    if message.duplicate {
        badges.push("duplicate".to_owned());
    }
    badges
}

fn publish_validation(topic: &str, payload_len: usize) -> Vec<String> {
    let topic_message = match TopicName::new(topic) {
        Ok(_) => "Topic is valid".to_owned(),
        Err(error) => format!("Topic error: {}", error.to_report().message),
    };
    vec![topic_message, format!("Payload: {payload_len} bytes")]
}

fn subscribe_validation(topic: &str) -> Vec<String> {
    vec![match TopicFilter::new(topic) {
        Ok(_) => "Topic filter is valid".to_owned(),
        Err(error) => format!("Topic filter error: {}", error.to_report().message),
    }]
}

fn push_recent_unique(entries: &mut Vec<String>, topic: &str) {
    entries.retain(|entry| entry != topic);
    entries.insert(0, topic.to_owned());
    entries.truncate(100);
}

fn topic_matches_filter(topic: &str, filter: &str) -> bool {
    let topic_levels = topic.split('/').collect::<Vec<_>>();
    let filter_levels = filter.split('/').collect::<Vec<_>>();

    for (index, filter_level) in filter_levels.iter().enumerate() {
        match *filter_level {
            "#" => return index == filter_levels.len() - 1,
            "+" => {
                if topic_levels.get(index).is_none() {
                    return false;
                }
            }
            literal if topic_levels.get(index) != Some(&literal) => return false,
            _ => {}
        }
    }

    topic_levels.len() == filter_levels.len()
}
