use crate::current::{
    ConnectionHistorySnapshot, HistoryPersistenceSnapshot, Message, MessageType,
    PublishMessageHistory as CurrentPublishMessageHistory, PublishStatus, PublishTopicHistory, Qos,
    SubscriptionHistory,
};
use crate::legacy::{LegacyHistories, LegacyMessage};

use super::{record_extra_fields, MigrationReport, MigrationWarning};

pub(super) fn migrate_histories(
    histories: &LegacyHistories,
    report: &mut MigrationReport,
) -> HistoryPersistenceSnapshot {
    let mut snapshot = HistoryPersistenceSnapshot::default();
    let ids = histories
        .publish_topics
        .keys()
        .chain(histories.publish_messages.keys())
        .chain(histories.subscription_topics.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();

    for id in ids {
        let publish_topics = histories
            .publish_topics
            .get(&id)
            .map(|history| {
                record_extra_fields(
                    &format!("histories.{id}.publishTopics"),
                    &history.extra,
                    report,
                );
                PublishTopicHistory {
                    topics: history.topics.clone(),
                }
            })
            .unwrap_or_default();
        let subscriptions = histories
            .subscription_topics
            .get(&id)
            .map(|history| {
                record_extra_fields(
                    &format!("histories.{id}.subscriptions"),
                    &history.extra,
                    report,
                );
                SubscriptionHistory {
                    topics: history.topics.clone(),
                }
            })
            .unwrap_or_default();
        let publish_messages = histories
            .publish_messages
            .get(&id)
            .map(|history| migrate_publish_messages(&id, history, report))
            .unwrap_or_default();

        snapshot.connections.insert(
            id.clone(),
            ConnectionHistorySnapshot {
                connection_id: id,
                publish_topics,
                publish_messages,
                subscriptions,
            },
        );
    }

    snapshot
}

fn migrate_publish_messages(
    connection_id: &str,
    history: &crate::legacy::PublishMessageHistory,
    report: &mut MigrationReport,
) -> CurrentPublishMessageHistory {
    record_extra_fields(
        &format!("histories.{connection_id}.publishMessages"),
        &history.extra,
        report,
    );
    let messages = history
        .messages
        .iter()
        .enumerate()
        .filter_map(|(index, message)| {
            let path = format!("histories.{connection_id}.publishMessages[{index}]");
            record_extra_fields(&path, &message.extra, report);
            migrate_history_message(message, &path, report)
        })
        .collect();
    CurrentPublishMessageHistory { messages }
}

fn migrate_history_message(
    message: &LegacyMessage,
    path: &str,
    report: &mut MigrationReport,
) -> Option<Message> {
    let Some(topic) = message.topic.clone() else {
        report.warnings.push(MigrationWarning {
            code: "legacy_history_message_missing_topic",
            message: format!("Legacy publish message ignored because {path}.topic is missing"),
        });
        return None;
    };

    Some(Message {
        topic,
        payload: message.payload.clone(),
        retained: message.is_retained,
        qos: migrate_qos(message.qos, path, report),
        date_time: message.date_time.clone(),
        message_id: message.message_id.clone(),
        message_type: migrate_message_type(message.message_type.as_deref(), path, report),
        publish_status: migrate_publish_status(message.publish_status.as_deref(), path, report),
    })
}

fn migrate_qos(qos: Option<u8>, path: &str, report: &mut MigrationReport) -> Option<Qos> {
    qos.and_then(|value| {
        Qos::from_legacy(value).or_else(|| {
            report.warnings.push(MigrationWarning {
                code: "legacy_history_qos_unknown",
                message: format!("Legacy publish message {path}.qos={value} ignored"),
            });
            None
        })
    })
}

fn migrate_message_type(
    value: Option<&str>,
    path: &str,
    report: &mut MigrationReport,
) -> Option<MessageType> {
    match value {
        Some("INCOMING") => Some(MessageType::Incoming),
        Some("OUTGOING") => Some(MessageType::Outgoing),
        Some(other) => {
            report.warnings.push(MigrationWarning {
                code: "legacy_history_message_type_unknown",
                message: format!("Legacy publish message {path}.messageType={other} ignored"),
            });
            None
        }
        None => None,
    }
}

fn migrate_publish_status(
    value: Option<&str>,
    path: &str,
    report: &mut MigrationReport,
) -> Option<PublishStatus> {
    match value {
        Some("PUBLISHED") => Some(PublishStatus::Published),
        Some("SUCCEEDED") => Some(PublishStatus::Succeeded),
        Some("FAILED") => Some(PublishStatus::Failed),
        Some(other) => {
            report.warnings.push(MigrationWarning {
                code: "legacy_history_publish_status_unknown",
                message: format!("Legacy publish message {path}.publishStatus={other} ignored"),
            });
            None
        }
        None => None,
    }
}
