use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub topic: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(default, alias = "isRetained")]
    pub retained: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_qos",
        skip_serializing_if = "Option::is_none"
    )]
    pub qos: Option<Qos>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish_status: Option<PublishStatus>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qos {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl Qos {
    pub fn from_legacy(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::AtMostOnce),
            1 => Some(Self::AtLeastOnce),
            2 => Some(Self::ExactlyOnce),
            _ => None,
        }
    }
}

impl Serialize for Qos {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(match self {
            Self::AtMostOnce => 0,
            Self::AtLeastOnce => 1,
            Self::ExactlyOnce => 2,
        })
    }
}

impl<'de> Deserialize<'de> for Qos {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Self::from_legacy(value)
            .ok_or_else(|| de::Error::custom(format!("QoS must be 0, 1, or 2; got {value}")))
    }
}

fn deserialize_optional_qos<'de, D>(deserializer: D) -> std::result::Result<Option<Qos>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<u8>::deserialize(deserializer).map(|value| value.and_then(Qos::from_legacy))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublishStatus {
    Published,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishTopicHistory {
    pub topics: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionHistory {
    pub topics: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishMessageHistory {
    pub messages: Vec<Message>,
}
