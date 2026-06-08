use correo_storage::current::{
    HistoryStore, Message, MessageType, PublishStatus, Qos, MAX_HISTORY_ENTRIES,
};
use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::MigrationPreview;
use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

fn message(topic: &str, payload: &str) -> Message {
    Message {
        topic: topic.to_owned(),
        payload: Some(payload.to_owned()),
        retained: false,
        qos: Some(Qos::AtLeastOnce),
        date_time: Some("2026-06-08T18:00:00".to_owned()),
        message_id: Some(format!("synthetic-{payload}")),
        message_type: Some(MessageType::Outgoing),
        publish_status: Some(PublishStatus::Succeeded),
    }
}

#[test]
fn loads_legacy_history_shapes() {
    let store = HistoryStore::new(fixture("legacy_profile"));

    let snapshot = store.load_connection("local-broker-01").unwrap();

    assert_eq!(
        snapshot.publish_topics.topics,
        ["sensors/temperature", "alerts/status"]
    );
    assert_eq!(
        snapshot.subscriptions.topics,
        ["sensors/#", "alerts/status"]
    );
    let stored_message = snapshot.publish_messages.messages.first().unwrap();
    assert_eq!(stored_message.topic, "alerts/status");
    assert_eq!(stored_message.qos, Some(Qos::AtLeastOnce));
    assert_eq!(stored_message.message_type, Some(MessageType::Outgoing));
    assert_eq!(
        stored_message.publish_status,
        Some(PublishStatus::Succeeded)
    );
}

#[test]
fn records_successful_publish_history_with_legacy_ordering_and_shape() {
    let temp = tempfile::tempdir().unwrap();
    let store = HistoryStore::new(temp.path());

    store
        .record_publish_success("connection-01", message("alerts/status", "first"))
        .unwrap();
    store
        .record_publish_success("connection-01", message("alerts/status", "second"))
        .unwrap();

    let snapshot = store.load_connection("connection-01").unwrap();
    assert_eq!(snapshot.publish_topics.topics, ["alerts/status"]);
    assert_eq!(
        snapshot
            .publish_messages
            .messages
            .iter()
            .map(|message| message.payload.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["second", "first"]
    );

    let text =
        std::fs::read_to_string(temp.path().join("connection-01_publishMessageHistory.json"))
            .unwrap();
    assert!(text.contains("\"retained\""));
    assert!(!text.contains("\"isRetained\""));
    assert!(text.contains("\"qos\": 1"));
    assert!(text.contains("\"messageType\": \"OUTGOING\""));
    assert!(text.contains("\"publishStatus\": \"SUCCEEDED\""));
}

#[test]
fn caps_and_deduplicates_topic_histories_deterministically() {
    let temp = tempfile::tempdir().unwrap();
    let store = HistoryStore::new(temp.path());

    for index in 0..=MAX_HISTORY_ENTRIES {
        store
            .record_publish_topic("connection-01", format!("topic/{index}"))
            .unwrap();
        store
            .record_subscription("connection-01", format!("subscription/{index}"), false)
            .unwrap();
    }
    store
        .record_publish_topic("connection-01", "topic/50")
        .unwrap();
    store
        .record_subscription("connection-01", "subscription/50", false)
        .unwrap();
    store
        .record_subscription("connection-01", "hidden/not-persisted", true)
        .unwrap();

    let snapshot = store.load_connection("connection-01").unwrap();
    assert_eq!(snapshot.publish_topics.topics.len(), MAX_HISTORY_ENTRIES);
    assert_eq!(snapshot.publish_topics.topics.last().unwrap(), "topic/50");
    assert_eq!(
        snapshot
            .publish_topics
            .topics
            .iter()
            .filter(|topic| topic.as_str() == "topic/50")
            .count(),
        1
    );
    assert_eq!(snapshot.subscriptions.topics.len(), MAX_HISTORY_ENTRIES);
    assert_eq!(
        snapshot.subscriptions.topics.last().unwrap(),
        "subscription/50"
    );
    assert!(!snapshot
        .subscriptions
        .topics
        .iter()
        .any(|topic| topic == "hidden/not-persisted"));
}

#[test]
fn caps_removes_and_clears_publish_message_history() {
    let temp = tempfile::tempdir().unwrap();
    let store = HistoryStore::new(temp.path());
    let removed = message("topic/remove", "remove");

    store
        .record_publish_message("connection-01", removed.clone())
        .unwrap();
    for index in 0..=MAX_HISTORY_ENTRIES {
        store
            .record_publish_message("connection-01", message("topic/cap", &index.to_string()))
            .unwrap();
    }

    let capped = store.load_publish_messages("connection-01").unwrap();
    assert_eq!(capped.messages.len(), MAX_HISTORY_ENTRIES);
    assert_eq!(
        capped.messages.first().unwrap().payload.as_deref(),
        Some("100")
    );
    assert_eq!(
        capped.messages.last().unwrap().payload.as_deref(),
        Some("1")
    );

    store
        .record_publish_message("connection-01", removed.clone())
        .unwrap();
    let after_remove = store
        .remove_published_message("connection-01", &removed)
        .unwrap();
    assert!(!after_remove
        .messages
        .iter()
        .any(|message| message == &removed));

    let cleared = store.clear_published_messages("connection-01").unwrap();
    assert!(cleared.messages.is_empty());
}

#[test]
fn migration_records_unknown_history_fields_and_unmappable_values() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("config.json"),
        r#"{"connections":[{"id":"connection-01","name":"Local","url":"localhost"}]}"#,
    )
    .unwrap();
    std::fs::write(
        temp.path().join("connection-01_publishHistory.json"),
        r#"{"topics":["topic/one"],"futureTopicField":true}"#,
    )
    .unwrap();
    std::fs::write(
        temp.path().join("connection-01_subscriptionHistory.json"),
        r#"{"topics":["topic/#"],"futureSubscriptionField":true}"#,
    )
    .unwrap();
    std::fs::write(
        temp.path().join("connection-01_publishMessageHistory.json"),
        r#"{"messages":[{"topic":"topic/one","retained":true,"qos":9,"futureMessageField":true}],"futureMessageListField":true}"#,
    )
    .unwrap();

    let profile = LegacyProfile::read_from(temp.path()).unwrap();
    let preview = MigrationPreview::from_legacy_profile(profile).unwrap();

    let connection = preview.histories.connections.get("connection-01").unwrap();
    assert_eq!(connection.publish_topics.topics, ["topic/one"]);
    assert_eq!(connection.subscriptions.topics, ["topic/#"]);
    assert!(connection.publish_messages.messages[0].retained);
    assert_eq!(connection.publish_messages.messages[0].qos, None);
    assert!(preview
        .report
        .unsupported_fields
        .iter()
        .any(|field| { field.path == "histories.connection-01.publishTopics.futureTopicField" }));
    assert!(preview.report.unsupported_fields.iter().any(|field| {
        field.path == "histories.connection-01.publishMessages.futureMessageListField"
    }));
    assert!(preview.report.unsupported_fields.iter().any(|field| {
        field.path == "histories.connection-01.publishMessages[0].futureMessageField"
    }));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.code == "legacy_history_qos_unknown"));
}
