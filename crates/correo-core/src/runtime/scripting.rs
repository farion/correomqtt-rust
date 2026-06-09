use crate::{
    AppCommand, AppEvent, AppSnapshot, Diagnostic, ScriptRow, ScriptingCommand, ScriptingEvent,
};

use super::AppRuntime;

impl AppRuntime {
    pub(super) fn apply_scripting_event(&mut self, event: ScriptingEvent) {
        match event {
            ScriptingEvent::Stored { action, path } => {
                self.model
                    .apply_event(AppEvent::DiagnosticRaised(Diagnostic::info(format!(
                        "{} stored for {path}.",
                        action.label()
                    ))));
            }
            ScriptingEvent::Failed {
                action,
                path,
                error,
            } => {
                self.model
                    .apply_event(AppEvent::DiagnosticRaised(Diagnostic::error(format!(
                        "{} failed for {path}: {error}",
                        action.label()
                    ))));
            }
            ScriptingEvent::LogAppended {
                execution_id,
                level,
                message,
                timestamp,
            } => {
                self.model
                    .apply_event(AppEvent::ScriptExecutionLogAppended {
                        execution_id,
                        level,
                        message,
                        timestamp,
                    });
            }
            ScriptingEvent::ExecutionUpdated {
                execution_id,
                status,
                duration,
                error,
            } => {
                self.model.apply_event(AppEvent::ScriptExecutionUpdated {
                    execution_id,
                    status,
                    duration,
                    error,
                });
            }
        }
    }

    pub(super) fn dispatch_scripting_command(&self, command: &AppCommand, before: &AppSnapshot) {
        let Some(worker) = &self.scripting_worker else {
            return;
        };
        let Some(command) = scripting_command(command, before, self.model.snapshot()) else {
            return;
        };
        if let Err(error) = worker.dispatch(command) {
            let _ = self
                .event_sender
                .emit(AppEvent::DiagnosticRaised(Diagnostic::error(
                    error.to_string(),
                )));
        }
    }
}

fn scripting_command(
    command: &AppCommand,
    before: &AppSnapshot,
    after: &AppSnapshot,
) -> Option<ScriptingCommand> {
    match command {
        AppCommand::CreateScript => selected_script(after)
            .filter(|script| !script_exists(before, &script.relative_path))
            .map(|script| ScriptingCommand::Create {
                path: script.relative_path.clone(),
                source: script.source.clone(),
            }),
        AppCommand::SaveScript => selected_script(after).map(|script| ScriptingCommand::Save {
            path: script.relative_path.clone(),
            source: script.source.clone(),
        }),
        AppCommand::ConfirmRenameScript => {
            let old_path = selected_script(before)?.relative_path.clone();
            let new_path = selected_script(after)?.relative_path.clone();
            (old_path != new_path).then_some(ScriptingCommand::Rename { old_path, new_path })
        }
        AppCommand::ConfirmDeleteScript => {
            let script = selected_script(before)?;
            if script_exists(after, &script.relative_path) {
                return None;
            }
            let path = script.relative_path.clone();
            Some(ScriptingCommand::Delete { path })
        }
        AppCommand::RunScript => {
            if !after.scripts.running
                || before.scripts.active_execution_id == after.scripts.active_execution_id
            {
                return None;
            }
            let script = selected_script(after)?;
            let execution_id = after.scripts.active_execution_id.clone()?;
            Some(ScriptingCommand::Run {
                execution_id,
                script_name: script.name.clone(),
                script_path: script.relative_path.clone(),
                source: script.source.clone(),
                connection_id: after.scripts.selected_connection_id.clone(),
            })
        }
        AppCommand::CancelScript => {
            let execution_id = before.scripts.active_execution_id.clone()?;
            Some(ScriptingCommand::Cancel { execution_id })
        }
        _ => None,
    }
}

fn selected_script(snapshot: &AppSnapshot) -> Option<&ScriptRow> {
    snapshot.scripts.selected_script()
}

fn script_exists(snapshot: &AppSnapshot, relative_path: &str) -> bool {
    snapshot
        .scripts
        .scripts
        .iter()
        .any(|script| script.relative_path == relative_path)
}

#[cfg(test)]
mod tests {
    use crate::{sample_snapshot, AppCommand, ScriptingCommand, ThemeMode};

    use super::scripting_command;

    #[test]
    fn run_script_dispatch_uses_active_execution_and_selected_connection() {
        let before = sample_snapshot(ThemeMode::Light);
        let mut after = before.clone();
        after.scripts.active_execution_id = Some("exec-1".to_owned());

        let command = scripting_command(&AppCommand::RunScript, &before, &after)
            .expect("run script should dispatch");

        assert!(matches!(
            command,
            ScriptingCommand::Run {
                execution_id,
                connection_id: Some(_),
                ..
            } if execution_id == "exec-1"
        ));
    }
}
