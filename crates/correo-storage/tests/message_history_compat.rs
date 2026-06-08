use std::path::{Path, PathBuf};

use correo_storage::current::{
    message_export_json, parse_message_export_json, read_message_export, write_message_export,
    ConnectionHistorySnapshot, HistoryPersistenceSnapshot, HistoryStore, Message, MessageType,
    PublishMessageHistory, PublishStatus, PublishTopicHistory, Qos, SubscriptionHistory,
};
use serde_json::Value;

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

#[test]
fn message_export_preserves_java_message_dto_metadata() {
    let message = read_message_export(fixture("message_exports/full_message.json")).unwrap();

    assert_eq!(message.topic, "alerts/status");
    assert_eq!(
        message.payload.as_deref(),
        Some("{\"state\":\"ok\",\"count\":2}")
    );
    assert!(message.retained);
    assert_eq!(message.qos, Some(Qos::ExactlyOnce));
    assert_eq!(
        message.date_time.as_deref(),
        Some("2026-06-08T19:04:03.210Z")
    );
    assert_eq!(
        message.message_id.as_deref(),
        Some("synthetic-message-export-001")
    );
    assert_eq!(message.message_type, Some(MessageType::Outgoing));
    assert_eq!(message.publish_status, Some(PublishStatus::Failed));

    let json = message_export_json(&message).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["topic"].as_str(), Some("alerts/status"));
    assert_eq!(
        value["payload"].as_str(),
        Some("{\"state\":\"ok\",\"count\":2}")
    );
    assert_eq!(value["retained"].as_bool(), Some(true));
    assert!(value.get("isRetained").is_none());
    assert_eq!(value["qos"].as_u64(), Some(2));
    assert_eq!(value["dateTime"].as_str(), Some("2026-06-08T19:04:03.210Z"));
    assert_eq!(
        value["messageId"].as_str(),
        Some("synthetic-message-export-001")
    );
    assert_eq!(value["messageType"].as_str(), Some("OUTGOING"));
    assert_eq!(value["publishStatus"].as_str(), Some("FAILED"));
    assert_eq!(parse_message_export_json(&json).unwrap(), message);

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("message.json");
    write_message_export(&path, &message).unwrap();
    assert_eq!(read_message_export(&path).unwrap(), message);
}

#[test]
fn message_import_tolerates_unmappable_java_qos_as_absent() {
    let message = parse_message_export_json(r#"{"topic":"alerts/status","qos":9}"#).unwrap();

    assert_eq!(message.topic, "alerts/status");
    assert_eq!(message.qos, None);
}

#[test]
fn message_import_accepts_legacy_is_retained_alias() {
    let message =
        parse_message_export_json(r#"{"topic":"alerts/status","isRetained":true}"#).unwrap();

    assert_eq!(message.topic, "alerts/status");
    assert!(message.retained);
}

#[test]
fn publish_and_subscription_histories_round_trip_known_fields() {
    let temp = tempfile::tempdir().unwrap();
    let store = HistoryStore::new(temp.path());
    let snapshot = ConnectionHistorySnapshot {
        connection_id: "connection-01".to_owned(),
        publish_topics: PublishTopicHistory {
            topics: vec!["alerts/status".to_owned(), "sensors/temperature".to_owned()],
        },
        publish_messages: PublishMessageHistory {
            messages: vec![
                Message {
                    topic: "alerts/status".to_owned(),
                    payload: Some("{\"state\":\"ok\"}".to_owned()),
                    retained: true,
                    qos: Some(Qos::AtLeastOnce),
                    date_time: Some("2026-06-08T19:05:00.000Z".to_owned()),
                    message_id: Some("synthetic-history-message-001".to_owned()),
                    message_type: Some(MessageType::Outgoing),
                    publish_status: Some(PublishStatus::Succeeded),
                },
                Message {
                    topic: "sensors/temperature".to_owned(),
                    payload: Some("21.4".to_owned()),
                    retained: false,
                    qos: Some(Qos::AtMostOnce),
                    date_time: Some("2026-06-08T19:06:00.000Z".to_owned()),
                    message_id: Some("synthetic-history-message-002".to_owned()),
                    message_type: Some(MessageType::Incoming),
                    publish_status: Some(PublishStatus::Published),
                },
            ],
        },
        subscriptions: SubscriptionHistory {
            topics: vec!["alerts/#".to_owned(), "sensors/+".to_owned()],
        },
    };
    let mut all = HistoryPersistenceSnapshot::default();
    all.connections
        .insert(snapshot.connection_id.clone(), snapshot.clone());

    store.replace_all(&all).unwrap();

    assert_eq!(store.load_connection("connection-01").unwrap(), snapshot);

    let text =
        std::fs::read_to_string(temp.path().join("connection-01_publishMessageHistory.json"))
            .unwrap();
    assert!(text.contains("\"retained\""));
    assert!(!text.contains("\"isRetained\""));
    assert!(text.contains("\"qos\": 1"));
    assert!(text.contains("\"dateTime\": \"2026-06-08T19:05:00.000Z\""));
    assert!(text.contains("\"messageId\": \"synthetic-history-message-001\""));
    assert!(text.contains("\"messageType\": \"OUTGOING\""));
    assert!(text.contains("\"publishStatus\": \"SUCCEEDED\""));
}
