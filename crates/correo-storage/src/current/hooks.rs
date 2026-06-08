use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FreshPluginState {
    pub manifests: Vec<PluginManifest>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<PluginCapability>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    IncomingTransform,
    OutgoingTransform,
    Validator,
    DetailTransform,
    DetailFormatter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegacyHookExtension {
    pub hook_kind: LegacyHookKind,
    pub plugin_id: Option<String>,
    pub extension_id: Option<String>,
    pub config: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyHookKind {
    OutgoingMessage,
    IncomingMessage,
    DetailViewTask,
    MessageValidator,
}
