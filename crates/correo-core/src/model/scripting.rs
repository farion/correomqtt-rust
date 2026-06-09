use crate::{
    redact_sensitive, Diagnostic, ScriptDetailTab, ScriptExecutionError, ScriptExecutionErrorKind,
    ScriptExecutionRow, ScriptExecutionStatus, ScriptFeedback, ScriptFileStatus, ScriptLogLevel,
    ScriptLogLine, ScriptRow,
};

use super::AppModel;

impl AppModel {
    pub(super) fn search_scripts(&mut self, filter: String) {
        self.snapshot.scripts.script_filter = filter;
    }

    pub(super) fn select_script(&mut self, name: String) {
        if self.script_index(&name).is_some() {
            self.snapshot.scripts.selected_script = name;
            self.snapshot.scripts.feedback = None;
        }
    }

    pub(super) fn update_new_script_name(&mut self, name: String) {
        self.snapshot.scripts.new_script_name = name;
    }

    pub(super) fn create_script(&mut self) {
        let draft_name = self.snapshot.scripts.new_script_name.trim();
        let name = if draft_name.is_empty() {
            self.next_default_script_name()
        } else {
            match normalize_script_path(draft_name) {
                Ok(name) => name,
                Err(message) => {
                    self.snapshot.scripts.feedback = Some(ScriptFeedback::error(message));
                    return;
                }
            }
        };
        if self.script_index(&name).is_some() {
            self.snapshot.scripts.feedback = Some(ScriptFeedback::error(format!(
                "Script already exists: {name}"
            )));
            return;
        }

        let source = default_script_source(&name);
        self.snapshot.scripts.scripts.push(ScriptRow {
            name: name.clone(),
            relative_path: name.clone(),
            status: ScriptFileStatus::Ready,
            execution_count: 0,
            saved_source: source.clone(),
            source,
        });
        self.snapshot.scripts.selected_script = name.clone();
        self.snapshot.scripts.new_script_name.clear();
        self.snapshot.scripts.feedback = Some(ScriptFeedback::info(format!("Created {name}.")));
    }

    fn next_default_script_name(&self) -> String {
        for index in 1.. {
            let candidate = if index == 1 {
                "new_script.js".to_owned()
            } else {
                format!("new_script_{index}.js")
            };
            if self.script_index(&candidate).is_none() {
                return candidate;
            }
        }
        unreachable!("unbounded default script name search should always find a candidate")
    }

    pub(super) fn update_script_source(&mut self, source: String) {
        let Some(index) = self.selected_script_index() else {
            return;
        };
        let script = &mut self.snapshot.scripts.scripts[index];
        script.source = source;
        if script.status != ScriptFileStatus::Running {
            script.status = if script.is_dirty() {
                ScriptFileStatus::Dirty
            } else {
                ScriptFileStatus::Ready
            };
        }
    }

    pub(super) fn save_script(&mut self) {
        let Some(index) = self.selected_script_index() else {
            self.snapshot.scripts.feedback =
                Some(ScriptFeedback::warning("Select a script before saving."));
            return;
        };
        let script = &mut self.snapshot.scripts.scripts[index];
        script.saved_source = script.source.clone();
        if script.status != ScriptFileStatus::Running {
            script.status = ScriptFileStatus::Ready;
        }
        self.snapshot.scripts.feedback =
            Some(ScriptFeedback::info(format!("Saved {}.", script.name)));
        self.push_diagnostic(Diagnostic::info("Script save command queued."));
    }

    pub(super) fn request_rename_script(&mut self) {
        if self.snapshot.scripts.selected_script().is_none() {
            self.snapshot.scripts.feedback =
                Some(ScriptFeedback::warning("Select a script before renaming."));
            return;
        }
        self.snapshot.scripts.rename_script_name = self.snapshot.scripts.selected_script.clone();
        self.snapshot.scripts.rename_dialog_open = true;
    }

    pub(super) fn update_rename_script_name(&mut self, name: String) {
        self.snapshot.scripts.rename_script_name = name;
    }

    pub(super) fn cancel_rename_script(&mut self) {
        self.snapshot.scripts.rename_dialog_open = false;
        self.snapshot.scripts.rename_script_name.clear();
    }

    pub(super) fn confirm_rename_script(&mut self) {
        let Some(index) = self.selected_script_index() else {
            self.snapshot.scripts.rename_dialog_open = false;
            return;
        };
        let name = match normalize_script_path(&self.snapshot.scripts.rename_script_name) {
            Ok(name) => name,
            Err(message) => {
                self.snapshot.scripts.feedback = Some(ScriptFeedback::error(message));
                return;
            }
        };
        if self
            .script_index(&name)
            .is_some_and(|existing| existing != index)
        {
            self.snapshot.scripts.feedback = Some(ScriptFeedback::error(format!(
                "Script already exists: {name}"
            )));
            return;
        }

        let old_name = self.snapshot.scripts.scripts[index].name.clone();
        self.snapshot.scripts.scripts[index].name = name.clone();
        self.snapshot.scripts.scripts[index].relative_path = name.clone();
        self.snapshot.scripts.selected_script = name.clone();
        self.snapshot.scripts.rename_dialog_open = false;
        self.snapshot.scripts.feedback = Some(ScriptFeedback::info(format!(
            "Renamed {old_name} to {name}."
        )));
    }

    pub(super) fn request_delete_script(&mut self) {
        if self.snapshot.scripts.selected_script().is_none() {
            self.snapshot.scripts.feedback =
                Some(ScriptFeedback::warning("Select a script before deleting."));
            return;
        }
        self.snapshot.scripts.delete_confirmation_open = true;
    }

    pub(super) fn cancel_delete_script(&mut self) {
        self.snapshot.scripts.delete_confirmation_open = false;
    }

    pub(super) fn confirm_delete_script(&mut self) {
        if self.snapshot.scripts.running {
            self.snapshot.scripts.feedback = Some(ScriptFeedback::warning(
                "Cancel the running script before deleting it.",
            ));
            return;
        }
        let Some(index) = self.selected_script_index() else {
            self.snapshot.scripts.delete_confirmation_open = false;
            return;
        };
        let deleted = self.snapshot.scripts.scripts.remove(index).name;
        self.snapshot.scripts.selected_script = self
            .snapshot
            .scripts
            .scripts
            .first()
            .map(|script| script.name.clone())
            .unwrap_or_default();
        self.snapshot.scripts.delete_confirmation_open = false;
        self.snapshot.scripts.feedback = Some(ScriptFeedback::warning(format!(
            "Deleted {deleted}; script sidecars will be removed by storage."
        )));
    }

    pub(super) fn select_script_detail_tab(&mut self, tab: ScriptDetailTab) {
        self.snapshot.scripts.active_tab = tab;
    }

    pub(super) fn run_script(&mut self) {
        if self.snapshot.scripts.running {
            self.snapshot.scripts.feedback = Some(ScriptFeedback::warning(
                "A script execution is already running.",
            ));
            return;
        }
        let Some(index) = self.selected_script_index() else {
            self.snapshot.scripts.feedback =
                Some(ScriptFeedback::warning("Select a script before running."));
            return;
        };

        let script_name = self.snapshot.scripts.scripts[index].name.clone();
        let execution_id = format!("script-exec-{}", self.snapshot.scripts.executions.len() + 1);
        self.snapshot.scripts.running = true;
        self.snapshot.scripts.active_execution_id = Some(execution_id.clone());
        self.snapshot.scripts.active_tab = ScriptDetailTab::Executions;
        self.snapshot.scripts.scripts[index].status = ScriptFileStatus::Running;
        self.snapshot.scripts.scripts[index].execution_count += 1;
        self.snapshot.scripts.executions.insert(
            0,
            ScriptExecutionRow {
                execution_id: execution_id.clone(),
                script_name: script_name.clone(),
                status: ScriptExecutionStatus::Running,
                duration: "running".to_owned(),
                timestamp: "now".to_owned(),
                error: None,
            },
        );
        self.append_script_log(
            execution_id,
            ScriptLogLevel::Info,
            format!(
                "{script_name} execution queued for {}",
                self.snapshot.scripts.selected_connection
            ),
            "now".to_owned(),
        );
        self.snapshot.scripts.feedback = Some(ScriptFeedback::info(format!(
            "Run command queued for {script_name}."
        )));
    }

    pub(super) fn cancel_script(&mut self) {
        let Some(execution_id) = self.snapshot.scripts.active_execution_id.clone() else {
            self.snapshot.scripts.feedback =
                Some(ScriptFeedback::warning("No running script to cancel."));
            return;
        };
        self.append_script_log(
            execution_id.clone(),
            ScriptLogLevel::Warning,
            "Cancellation requested by user.".to_owned(),
            "now".to_owned(),
        );
        self.update_script_execution(
            execution_id,
            ScriptExecutionStatus::Cancelled,
            "cancelled".to_owned(),
            Some(ScriptExecutionError {
                kind: ScriptExecutionErrorKind::Cancellation,
                message: "Script execution was cancelled.".to_owned(),
            }),
        );
    }

    pub(super) fn append_script_log(
        &mut self,
        execution_id: String,
        level: ScriptLogLevel,
        message: String,
        timestamp: String,
    ) {
        self.snapshot.scripts.log_lines.push(ScriptLogLine {
            timestamp,
            level,
            message: redact_script_output(&format!("[{execution_id}] {message}")),
        });
        if self.snapshot.scripts.log_lines.len() > 200 {
            let overflow = self.snapshot.scripts.log_lines.len() - 200;
            self.snapshot.scripts.log_lines.drain(0..overflow);
        }
    }

    pub(super) fn update_script_execution(
        &mut self,
        execution_id: String,
        status: ScriptExecutionStatus,
        duration: String,
        error: Option<ScriptExecutionError>,
    ) {
        let redacted_error = error.map(redact_script_error);
        if let Some(execution) = self
            .snapshot
            .scripts
            .executions
            .iter_mut()
            .find(|execution| execution.execution_id == execution_id)
        {
            execution.status = status;
            execution.duration = duration;
            execution.error = redacted_error.clone();
        }

        if status.is_terminal() && self.snapshot.scripts.active_execution_id == Some(execution_id) {
            self.snapshot.scripts.running = false;
            self.snapshot.scripts.active_execution_id = None;
            if let Some(index) = self.selected_script_index() {
                self.snapshot.scripts.scripts[index].status = match status {
                    ScriptExecutionStatus::Failed => ScriptFileStatus::Error,
                    _ if self.snapshot.scripts.scripts[index].is_dirty() => ScriptFileStatus::Dirty,
                    _ => ScriptFileStatus::Ready,
                };
            }
        }

        self.snapshot.scripts.last_error = redacted_error.clone();
        if let Some(error) = redacted_error {
            self.snapshot.scripts.feedback = Some(ScriptFeedback::warning(format!(
                "{}: {}",
                error.kind.label(),
                error.message
            )));
        } else if status.is_terminal() {
            self.snapshot.scripts.feedback = Some(ScriptFeedback::info(format!(
                "Execution {}.",
                status.label()
            )));
        }
    }

    fn selected_script_index(&self) -> Option<usize> {
        self.script_index(&self.snapshot.scripts.selected_script)
    }

    fn script_index(&self, name: &str) -> Option<usize> {
        self.snapshot
            .scripts
            .scripts
            .iter()
            .position(|script| script.name == name)
    }
}

fn normalize_script_path(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Script name is required.".to_owned());
    }
    if trimmed.contains('\\') {
        return Err("Use forward slashes for script folders.".to_owned());
    }
    let path = if trimmed.ends_with(".js") {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.js")
    };
    let mut components = path.split('/');
    let Some(first) = components.next() else {
        return Err("Script name is required.".to_owned());
    };
    if first == "logs" || first == "executions" {
        return Err("Script names cannot use sidecar storage folders.".to_owned());
    }
    if path
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err("Script name must be a safe relative .js path.".to_owned());
    }
    Ok(path)
}

fn default_script_source(name: &str) -> String {
    format!(
        "const client = clientFactory.getPromiseClient();\nlogger.info('starting {name}');\nqueue.process();\n"
    )
}

fn redact_script_error(mut error: ScriptExecutionError) -> ScriptExecutionError {
    error.message = redact_script_output(&error.message);
    error
}

fn redact_script_output(message: &str) -> String {
    let redacted = redact_sensitive(message);
    let lower = redacted.to_ascii_lowercase();
    if lower.contains("-----begin") && lower.contains("private key") {
        "[REDACTED KEY MATERIAL]".to_owned()
    } else if lower.contains("decrypted password map")
        || lower.contains("export password")
        || lower.contains("key material")
    {
        "[REDACTED SCRIPT OUTPUT: sensitive material]".to_owned()
    } else {
        redacted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppCommand, AppEvent};

    #[test]
    fn create_script_without_name_allocates_unique_default() {
        let mut model = AppModel::empty();

        model.apply_command(AppCommand::CreateScript);
        model.apply_command(AppCommand::CreateScript);

        let scripts = &model.snapshot().scripts;
        assert_eq!(scripts.scripts[0].name, "new_script.js");
        assert_eq!(scripts.scripts[1].name, "new_script_2.js");
        assert_eq!(scripts.selected_script, "new_script_2.js");
        assert!(scripts
            .feedback
            .as_ref()
            .unwrap()
            .message
            .contains("Created"));
    }

    #[test]
    fn script_edit_save_tracks_dirty_state() {
        let mut model = AppModel::default();
        let name = model.snapshot().scripts.selected_script.clone();

        model.apply_command(AppCommand::UpdateScriptSource(
            "logger.info('changed');".to_owned(),
        ));

        let script = model.snapshot().scripts.selected_script().unwrap();
        assert_eq!(script.name, name);
        assert!(script.is_dirty());
        assert!(model.snapshot().scripts.can_save());

        model.apply_command(AppCommand::SaveScript);

        assert!(!model
            .snapshot()
            .scripts
            .selected_script()
            .unwrap()
            .is_dirty());
        assert!(!model.snapshot().scripts.can_save());
    }

    #[test]
    fn run_and_cancel_script_updates_execution_state() {
        let mut model = AppModel::default();

        model.apply_command(AppCommand::RunScript);

        assert!(model.snapshot().scripts.running);
        assert_eq!(
            model.snapshot().scripts.executions[0].status,
            ScriptExecutionStatus::Running
        );

        model.apply_command(AppCommand::CancelScript);

        assert!(!model.snapshot().scripts.running);
        assert_eq!(
            model.snapshot().scripts.executions[0].status,
            ScriptExecutionStatus::Cancelled
        );
        assert_eq!(
            model.snapshot().scripts.last_error.as_ref().unwrap().kind,
            ScriptExecutionErrorKind::Cancellation
        );
    }

    #[test]
    fn script_log_events_are_redacted_before_snapshot_exposure() {
        let mut model = AppModel::default();

        model.apply_event(AppEvent::ScriptExecutionLogAppended {
            execution_id: "exec-1".to_owned(),
            level: ScriptLogLevel::Error,
            message: "password=hunter2 key material follows".to_owned(),
            timestamp: "now".to_owned(),
        });

        let message = &model.snapshot().scripts.log_lines.last().unwrap().message;
        assert!(!message.contains("hunter2"));
        assert!(!message.contains("key material"));
        assert!(message.contains("[REDACTED"));
    }
}
