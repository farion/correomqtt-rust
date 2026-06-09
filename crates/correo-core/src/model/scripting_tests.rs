use crate::{
    AppCommand, AppEvent, AppModel, ScriptExecutionErrorKind, ScriptExecutionRow,
    ScriptExecutionStatus, ScriptLogLevel,
};

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
fn discard_script_changes_restores_saved_source() {
    let mut model = AppModel::default();
    let original = model
        .snapshot()
        .scripts
        .selected_script()
        .unwrap()
        .saved_source
        .clone();

    model.apply_command(AppCommand::UpdateScriptSource(
        "logger.info('dirty');".to_owned(),
    ));
    assert!(model.snapshot().scripts.selected_script_is_dirty());

    model.apply_command(AppCommand::DiscardScriptChanges);

    let script = model.snapshot().scripts.selected_script().unwrap();
    assert_eq!(script.source, original);
    assert!(!script.is_dirty());
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
fn clear_finished_execution_logs_keeps_running_execution() {
    let mut model = AppModel::default();

    model.apply_command(AppCommand::RunScript);
    let running_id = model
        .snapshot()
        .scripts
        .active_execution_id
        .clone()
        .unwrap();
    model.snapshot.scripts.executions.push(ScriptExecutionRow {
        execution_id: "finished".to_owned(),
        script_name: model.snapshot().scripts.selected_script.clone(),
        status: ScriptExecutionStatus::Succeeded,
        duration: "42ms".to_owned(),
        timestamp: "now".to_owned(),
        error: None,
    });
    model.apply_event(AppEvent::ScriptExecutionLogAppended {
        execution_id: "finished".to_owned(),
        level: ScriptLogLevel::Info,
        message: "done".to_owned(),
        timestamp: "now".to_owned(),
    });

    model.apply_command(AppCommand::ClearFinishedScriptExecutions);

    assert!(model
        .snapshot()
        .scripts
        .executions
        .iter()
        .any(|execution| execution.execution_id == running_id));
    assert!(!model
        .snapshot()
        .scripts
        .executions
        .iter()
        .any(|execution| execution.execution_id == "finished"));
    assert!(!model
        .snapshot()
        .scripts
        .log_lines
        .iter()
        .any(|line| line.execution_id == "finished"));
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

    let line = model.snapshot().scripts.log_lines.last().unwrap();
    assert_eq!(line.execution_id, "exec-1");
    assert!(!line.message.contains("hunter2"));
    assert!(!line.message.contains("key material"));
    assert!(line.message.contains("[REDACTED"));
}
