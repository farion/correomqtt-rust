use crate::HookKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDiagnostic {
    #[serde(default)]
    pub plugin_id: Option<String>,
    #[serde(default)]
    pub hook: Option<HookKind>,
    pub severity: PluginDiagnosticSeverity,
    pub message: String,
}

impl PluginDiagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            plugin_id: None,
            hook: None,
            severity: PluginDiagnosticSeverity::Error,
            message: message.into(),
        }
    }

    pub fn for_plugin(mut self, plugin_id: impl Into<String>) -> Self {
        self.plugin_id = Some(plugin_id.into());
        self
    }

    pub fn for_hook(mut self, hook: HookKind) -> Self {
        self.hook = Some(hook);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

pub trait IntoPluginDiagnostic {
    fn diagnostic(&self) -> PluginDiagnostic;
}
