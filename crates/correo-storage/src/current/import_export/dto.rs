use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WireConnectionExport {
    #[serde(default)]
    pub(crate) encryption_type: Option<String>,
    #[serde(default)]
    pub(crate) encrypted_data: Option<String>,
    #[serde(default, rename = "connectionConfigDTOS")]
    pub(crate) connection_config_dtos: Option<Vec<WireConnection>>,
    #[serde(flatten)]
    pub(crate) extra: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WireConnectionExportOut {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) encryption_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) encrypted_data: Option<String>,
    #[serde(
        rename = "connectionConfigDTOS",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) connection_config_dtos: Option<Vec<WireConnection>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WireConnection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) port: Option<u16>,
    #[serde(default, alias = "client_id", skip_serializing_if = "Option::is_none")]
    pub(crate) client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) password: Option<String>,
    #[serde(default)]
    pub(crate) clean_session: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) mqtt_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ssl: Option<String>,
    #[serde(
        default,
        alias = "ssl_keystore",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) ssl_keystore: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ssl_keystore_password: Option<String>,
    #[serde(default)]
    pub(crate) ssl_host_verification: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) proxy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ssh_host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ssh_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) local_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auth_keyfile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) lwt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) lwt_topic: Option<String>,
    #[serde(
        default,
        rename = "lwtQoS",
        alias = "lwtQos",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) lwt_qos: Option<u8>,
    #[serde(default)]
    pub(crate) lwt_retained: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) lwt_payload: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) connection_ui_settings: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) publish_list_view_config: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) subscribe_list_view_config: Option<Value>,
    #[serde(flatten, skip_serializing)]
    pub(crate) extra: BTreeMap<String, Value>,
}
