#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysTopicMetadata<'a> {
    pub label: &'a str,
    pub description: &'a str,
    pub window: Option<&'a str>,
}

pub fn metadata_for_topic(topic: &str) -> Option<SysTopicMetadata<'_>> {
    match topic {
        "$SYS/broker/clients/connected" => Some(SysTopicMetadata {
            label: "Connected clients",
            description: "Number of currently connected clients.",
            window: None,
        }),
        "$SYS/broker/clients/disconnected" => Some(SysTopicMetadata {
            label: "Disconnected clients",
            description: "Number of currently disconnected persistent clients.",
            window: None,
        }),
        "$SYS/broker/messages/stored" => Some(SysTopicMetadata {
            label: "Stored messages",
            description: "Number of messages currently stored by the broker.",
            window: None,
        }),
        "$SYS/broker/subscriptions/count" => Some(SysTopicMetadata {
            label: "Subscriptions",
            description: "Number of active subscriptions known by the broker.",
            window: None,
        }),
        _ => aggregate_metadata(topic),
    }
}

fn aggregate_metadata(topic: &str) -> Option<SysTopicMetadata<'_>> {
    let (prefix, window) = topic.rsplit_once('/')?;
    let window = match window {
        "1min" | "5min" | "15min" => window,
        _ => return None,
    };

    let (label, description) = match prefix {
        "$SYS/broker/load/messages/received" => (
            "Aggregated messages received",
            "Broker message receive rate over the reporting window.",
        ),
        "$SYS/broker/load/messages/sent" => (
            "Aggregated messages sent",
            "Broker message send rate over the reporting window.",
        ),
        "$SYS/broker/load/publish/received" => (
            "Aggregated publishes received",
            "Broker publish receive rate over the reporting window.",
        ),
        "$SYS/broker/load/publish/sent" => (
            "Aggregated publishes sent",
            "Broker publish send rate over the reporting window.",
        ),
        "$SYS/broker/load/bytes/received" => (
            "Aggregated received bytes",
            "Broker byte receive rate over the reporting window.",
        ),
        "$SYS/broker/load/bytes/sent" => (
            "Aggregated sent bytes",
            "Broker byte send rate over the reporting window.",
        ),
        _ => return None,
    };

    Some(SysTopicMetadata {
        label,
        description,
        window: Some(window),
    })
}
