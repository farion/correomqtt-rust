use correo_mqtt::PublishRequest;
use serde_json::Value;

use crate::{
    AppCommand, AppEvent, DetailBytesOutput, FormattedMessageDetail, MessageDiagnosticRow,
    MessageInspectorTab, MessageTransform, MqttCommand, MqttCommandBuildError, MqttEvent,
    PluginDiagnosticSeverity, PluginHookCall, PluginHookDiagnosticEvent, PluginHookError,
    PluginHookInput, PluginHookKind, PluginHookOutput, PluginHookStatus, PluginStatus,
    PluginValidation, PluginWorkflowEvent,
};

use super::plugin_helpers::*;
use super::AppRuntime;

impl AppRuntime {
    pub(super) fn mqtt_commands_for_app_command_with_plugins(
        &self,
        command: &AppCommand,
    ) -> Result<Vec<MqttCommand>, MqttCommandBuildError> {
        let commands = self.model.mqtt_commands_for_app_command(command)?;

        let mut transformed = Vec::new();
        for command in commands {
            match command {
                MqttCommand::Publish {
                    connection_id,
                    request,
                } => {
                    if let Some(request) = self.apply_publish_hooks(connection_id, request) {
                        transformed.push(MqttCommand::Publish {
                            connection_id,
                            request,
                        });
                    }
                }
                command => transformed.push(command),
            }
        }
        Ok(transformed)
    }

    pub(super) fn apply_incoming_hooks(
        &self,
        event: MqttEvent,
    ) -> Option<(MqttEvent, Vec<MessageDiagnosticRow>)> {
        let MqttEvent::IncomingMessage(message) = event else {
            return Some((event, Vec::new()));
        };
        let topic = message.topic.as_str().to_owned();
        let mut plugin_message = plugin_message_from_incoming(&message);
        let mut diagnostics = Vec::new();

        for hook in self.active_topic_hooks(PluginHookKind::IncomingTransform, &topic) {
            let Some(config) = self.parse_hook_config(&hook, false) else {
                continue;
            };
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::Message(plugin_message.clone()),
            };
            match self.plugin_hooks.execute(call) {
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Unchanged)) => {}
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Replace(message))) => {
                    plugin_message = message;
                }
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Drop { reason })) => {
                    self.emit_hook_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Warning,
                        "Incoming message dropped by plugin transform.",
                        reason.unwrap_or_else(|| {
                            "Plugin requested the message be dropped.".to_owned()
                        }),
                        false,
                    );
                    return None;
                }
                Ok(output) => {
                    let detail = format!("Unexpected plugin output: {output:?}");
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        &detail,
                    ));
                    self.emit_hook_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        "Incoming transform returned an incompatible result.",
                        detail,
                        true,
                    );
                }
                Err(error) => {
                    diagnostics.push(message_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        &error.to_string(),
                    ));
                    self.emit_hook_error(&hook, "Incoming transform failed.", error, true);
                }
            }
        }

        match incoming_from_plugin_message(message, plugin_message) {
            Ok(message) => Some((MqttEvent::IncomingMessage(message), diagnostics)),
            Err(error) => {
                self.emit_hook_diagnostic(
                    &ActiveHook {
                        plugin_id: "plugin-workflow".to_owned(),
                        hook: PluginHookKind::IncomingTransform,
                        target: topic,
                        config_json: "{}".to_owned(),
                    },
                    PluginDiagnosticSeverity::Error,
                    "Incoming transform returned an invalid topic.",
                    error,
                    false,
                );
                None
            }
        }
    }

    pub(super) fn append_incoming_diagnostics(&self, diagnostics: Vec<MessageDiagnosticRow>) {
        if diagnostics.is_empty() {
            return;
        }
        let Some(message_id) = self.model.snapshot().workbench.selected_message_id else {
            return;
        };
        self.emit_plugin_event(PluginWorkflowEvent::MessageDiagnosticsAppended {
            message_id,
            diagnostics,
        });
    }

    pub(super) fn should_refresh_detail_for_command(&self, command: &AppCommand) -> bool {
        matches!(
            command,
            AppCommand::SelectMessage(_)
                | AppCommand::SelectInspectorTab(MessageInspectorTab::Formatted)
                | AppCommand::SelectDetailTransform(_)
                | AppCommand::SelectDetailFormatter(_)
                | AppCommand::RefreshMessageDetail
        )
    }

    pub(super) fn refresh_message_detail(&self) {
        let snapshot = self.model.snapshot();
        let Some(message) = snapshot.workbench.selected_message() else {
            return;
        };
        let message_id = message.id;
        let mut bytes = message.payload.clone();
        let mut content_type = None;
        let mut diagnostics = message.diagnostics.clone();

        if let Some(plugin_id) = &snapshot.workbench.detail.selected_transform_plugin_id {
            if let Some(hook) =
                self.selected_detail_hook(plugin_id, PluginHookKind::DetailTransform)
            {
                match self.run_detail_transform(&hook, bytes.clone(), content_type.clone()) {
                    Ok(output) => {
                        bytes = output.bytes;
                        content_type = output.content_type;
                    }
                    Err(error) => {
                        diagnostics.push(message_diagnostic(
                            &hook,
                            PluginDiagnosticSeverity::Error,
                            &error.to_string(),
                        ));
                        self.emit_hook_error(&hook, "Detail byte transform failed.", error, false);
                    }
                }
            }
        }

        let detail =
            if let Some(plugin_id) = &snapshot.workbench.detail.selected_formatter_plugin_id {
                if let Some(hook) =
                    self.selected_detail_hook(plugin_id, PluginHookKind::DetailFormatter)
                {
                    match self.run_detail_formatter(&hook, bytes.clone(), content_type.clone()) {
                        Ok(detail) => detail,
                        Err(error) => {
                            diagnostics.push(message_diagnostic(
                                &hook,
                                PluginDiagnosticSeverity::Error,
                                &error.to_string(),
                            ));
                            self.emit_hook_error(&hook, "Detail formatter failed.", error, false);
                            plain_detail(bytes, content_type, diagnostics.clone())
                        }
                    }
                } else {
                    plain_detail(bytes, content_type, diagnostics.clone())
                }
            } else {
                plain_detail(bytes, content_type, diagnostics.clone())
            };
        self.emit_plugin_event(PluginWorkflowEvent::MessageDetailUpdated { message_id, detail });
    }

    fn apply_publish_hooks(
        &self,
        _connection_id: correo_mqtt::ConnectionId,
        request: PublishRequest,
    ) -> Option<PublishRequest> {
        let mut message = plugin_message_from_publish(&request);
        let topic = message.topic.clone();

        for hook in self.active_topic_hooks(PluginHookKind::Validator, &topic) {
            let config = self.parse_hook_config(&hook, true)?;
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::Message(message.clone()),
            };
            match self.plugin_hooks.execute(call) {
                Ok(PluginHookOutput::Validation(PluginValidation::Valid)) => {}
                Ok(PluginHookOutput::Validation(PluginValidation::Warning { message })) => {
                    self.emit_plugin_event(PluginWorkflowEvent::PublishWarning { message });
                }
                Ok(PluginHookOutput::Validation(PluginValidation::Block { message })) => {
                    self.emit_hook_diagnostic(
                        &hook,
                        PluginDiagnosticSeverity::Error,
                        "Validator blocked publish.",
                        message.clone(),
                        false,
                    );
                    self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
                    return None;
                }
                Ok(output) => return self.block_publish_for_output(&hook, output),
                Err(error) => return self.block_publish_for_error(&hook, error),
            }
        }

        for hook in self.active_topic_hooks(PluginHookKind::OutgoingTransform, &topic) {
            let config = self.parse_hook_config(&hook, true)?;
            let call = PluginHookCall {
                plugin_id: hook.plugin_id.clone(),
                hook: hook.hook,
                target: hook.target.clone(),
                config,
                input: PluginHookInput::Message(message.clone()),
            };
            match self.plugin_hooks.execute(call) {
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Unchanged)) => {}
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Replace(replacement))) => {
                    message = replacement;
                }
                Ok(PluginHookOutput::MessageTransform(MessageTransform::Drop { reason })) => {
                    let message = reason
                        .unwrap_or_else(|| "Outgoing transform rejected the publish.".to_owned());
                    self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
                    return None;
                }
                Ok(output) => return self.block_publish_for_output(&hook, output),
                Err(error) => return self.block_publish_for_error(&hook, error),
            }
        }

        PublishRequest::new(
            message.topic.as_str(),
            message.payload,
            mqtt_qos(message.qos),
            message.retained,
        )
        .map_err(|error| error.to_report().message)
        .map_err(|message| {
            self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
        })
        .ok()
    }

    fn run_detail_transform(
        &self,
        hook: &ActiveHook,
        bytes: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<DetailBytesOutput, PluginHookError> {
        let Some(config) = self.parse_hook_config(hook, false) else {
            return Err(PluginHookError::failed(
                "detail transform config is invalid",
            ));
        };
        let call = PluginHookCall {
            plugin_id: hook.plugin_id.clone(),
            hook: hook.hook,
            target: hook.target.clone(),
            config,
            input: PluginHookInput::DetailBytes {
                bytes,
                content_type,
            },
        };
        match self.plugin_hooks.execute(call)? {
            PluginHookOutput::DetailBytes(output) => Ok(output),
            output => Err(PluginHookError::failed(format!(
                "detail transform returned incompatible output: {output:?}"
            ))),
        }
    }

    fn run_detail_formatter(
        &self,
        hook: &ActiveHook,
        bytes: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<FormattedMessageDetail, PluginHookError> {
        let Some(config) = self.parse_hook_config(hook, false) else {
            return Err(PluginHookError::failed(
                "detail formatter config is invalid",
            ));
        };
        let call = PluginHookCall {
            plugin_id: hook.plugin_id.clone(),
            hook: hook.hook,
            target: hook.target.clone(),
            config,
            input: PluginHookInput::DetailBytes {
                bytes,
                content_type,
            },
        };
        match self.plugin_hooks.execute(call)? {
            PluginHookOutput::DetailFormat(detail) => Ok(detail),
            output => Err(PluginHookError::failed(format!(
                "detail formatter returned incompatible output: {output:?}"
            ))),
        }
    }

    fn selected_detail_hook(&self, plugin_id: &str, kind: PluginHookKind) -> Option<ActiveHook> {
        self.active_hooks(kind)
            .into_iter()
            .find(|hook| hook.plugin_id == plugin_id)
    }

    fn active_topic_hooks(&self, kind: PluginHookKind, topic: &str) -> Vec<ActiveHook> {
        self.active_hooks(kind)
            .into_iter()
            .filter(|hook| topic_matches_filter(topic, &hook.target))
            .collect()
    }

    fn active_hooks(&self, kind: PluginHookKind) -> Vec<ActiveHook> {
        self.model
            .snapshot()
            .plugins
            .plugins
            .iter()
            .filter(|plugin| {
                plugin.enabled
                    && !matches!(
                        plugin.status,
                        PluginStatus::Disabled
                            | PluginStatus::NeedsConfig
                            | PluginStatus::CapabilityDenied
                            | PluginStatus::LoadError
                            | PluginStatus::UnsupportedLegacy
                    )
            })
            .flat_map(|plugin| {
                plugin
                    .hooks
                    .iter()
                    .filter(move |hook| {
                        hook.hook == kind
                            && hook.enabled
                            && matches!(hook.status, PluginHookStatus::Ready)
                    })
                    .map(|hook| ActiveHook {
                        plugin_id: plugin.id.clone(),
                        hook: hook.hook,
                        target: hook.target.clone(),
                        config_json: hook.config_json.clone(),
                    })
            })
            .collect()
    }

    fn parse_hook_config(&self, hook: &ActiveHook, publish_blocking: bool) -> Option<Value> {
        match serde_json::from_str::<Value>(&hook.config_json) {
            Ok(value) => Some(value),
            Err(error) => {
                let detail = format!("Hook config JSON is invalid: {error}");
                self.emit_hook_diagnostic(
                    hook,
                    PluginDiagnosticSeverity::Error,
                    "Plugin hook config is invalid.",
                    detail.clone(),
                    true,
                );
                if publish_blocking {
                    self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message: detail });
                }
                None
            }
        }
    }

    fn block_publish_for_output(
        &self,
        hook: &ActiveHook,
        output: PluginHookOutput,
    ) -> Option<PublishRequest> {
        let message = format!("Plugin hook returned incompatible output: {output:?}");
        self.emit_hook_diagnostic(
            hook,
            PluginDiagnosticSeverity::Error,
            "Publish hook failed.",
            message.clone(),
            true,
        );
        self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
        None
    }

    fn block_publish_for_error(
        &self,
        hook: &ActiveHook,
        error: PluginHookError,
    ) -> Option<PublishRequest> {
        let message = error.to_string();
        self.emit_hook_error(hook, "Publish hook failed.", error, true);
        self.emit_plugin_event(PluginWorkflowEvent::PublishBlocked { message });
        None
    }

    fn emit_hook_error(
        &self,
        hook: &ActiveHook,
        message: &'static str,
        error: PluginHookError,
        mark_hook_failed: bool,
    ) {
        self.emit_hook_diagnostic(
            hook,
            PluginDiagnosticSeverity::Error,
            message,
            error.to_string(),
            mark_hook_failed,
        );
    }

    fn emit_hook_diagnostic(
        &self,
        hook: &ActiveHook,
        severity: PluginDiagnosticSeverity,
        message: impl Into<String>,
        detail: impl Into<String>,
        mark_hook_failed: bool,
    ) {
        self.emit_plugin_event(PluginWorkflowEvent::HookDiagnostic(
            PluginHookDiagnosticEvent {
                plugin_id: hook.plugin_id.clone(),
                hook: Some(hook.hook),
                severity,
                message: message.into(),
                detail: detail.into(),
                mark_hook_failed,
            },
        ));
    }

    fn emit_plugin_event(&self, event: PluginWorkflowEvent) {
        let _ = self.event_sender.emit(AppEvent::PluginWorkflow(event));
    }
}
