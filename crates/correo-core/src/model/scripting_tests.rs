use crate::{
    AppCommand, AppEvent, AppModel, ScriptExecutionErrorKind, ScriptExecutionRow,
    ScriptExecutionStatus, ScriptLogLevel,
};

#[test]
fn request_create_script_prefills_unique_default() {
    let mut model = AppModel::empty();

    model.apply_command(AppCommand::RequestCreateScript);
    model.apply_command(AppCommand::CreateScript);
    model.apply_command(AppCommand::RequestCreateScript);
    model.apply_command(AppCommand::CreateScript);

    let scripts = &model.snapshot().scripts;
    assert_eq!(scripts.scripts[0].name, "new_script.js");
    assert_eq!(scripts.scripts[1].name, "new_script_2.js");
    assert_eq!(scripts.selected_script, "new_script_2.js");
    assert!(!scripts.create_dialog_open);
    assert!(!scripts.can_save());
    assert!(scripts.scripts.iter().all(|script| script.persisted));
    assert!(scripts.scripts.iter().all(|script| !script.is_dirty()));
    assert!(scripts
        .feedback
        .as_ref()
        .unwrap()
        .message
        .contains("Created"));
    assert!(model
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Created new_script_2.js")));
}

#[test]
fn create_script_validation_errors_stay_in_dialog() {
    let mut model = AppModel::empty();

    model.apply_command(AppCommand::RequestCreateScript);
    model.apply_command(AppCommand::UpdateNewScriptName("".to_owned()));
    model.apply_command(AppCommand::CreateScript);

    assert_eq!(
        model.snapshot().scripts.create_error.as_deref(),
        Some("Script name is required.")
    );
    assert!(model.snapshot().scripts.create_dialog_open);
    assert!(model.snapshot().scripts.scripts.is_empty());
    assert_eq!(
        model.snapshot().scripts.feedback.as_ref().unwrap().message,
        "Script name is required."
    );
    assert_eq!(
        model.snapshot().diagnostics[0].message,
        "Script name is required."
    );
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
    assert!(model.snapshot().diagnostics[0]
        .message
        .starts_with("Saved "));
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
fn rename_script_validation_errors_stay_in_dialog() {
    let mut model = AppModel::empty();
    model.apply_command(AppCommand::RequestCreateScript);
    model.apply_command(AppCommand::CreateScript);
    model.apply_command(AppCommand::RequestCreateScript);
    model.apply_command(AppCommand::CreateScript);

    model.apply_command(AppCommand::RequestRenameScript);
    model.apply_command(AppCommand::UpdateRenameScriptName(
        "new_script.js".to_owned(),
    ));
    model.apply_command(AppCommand::ConfirmRenameScript);

    assert_eq!(
        model.snapshot().scripts.rename_error.as_deref(),
        Some("Script already exists: new_script.js")
    );
    assert!(model.snapshot().scripts.rename_dialog_open);
    assert_eq!(
        model.snapshot().scripts.feedback.as_ref().unwrap().message,
        "Script already exists: new_script.js"
    );
    assert_eq!(
        model.snapshot().diagnostics[0].message,
        "Script already exists: new_script.js"
    );
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
fn cancel_script_uses_running_execution_row_when_active_id_is_missing() {
    let mut model = AppModel::default();
    model.snapshot.scripts.running = true;
    model.snapshot.scripts.active_execution_id = None;
    model.snapshot.scripts.executions.insert(
        0,
        ScriptExecutionRow {
            execution_id: "restored-running".to_owned(),
            script_name: model.snapshot().scripts.selected_script.clone(),
            status: ScriptExecutionStatus::Running,
            duration: "running".to_owned(),
            timestamp: "stored".to_owned(),
            error: None,
        },
    );

    model.apply_command(AppCommand::CancelScript);

    assert!(!model.snapshot().scripts.running);
    assert_eq!(
        model.snapshot().scripts.executions[0].status,
        ScriptExecutionStatus::Cancelled
    );
}

#[test]
fn run_script_uses_real_timestamps_in_execution_log() {
    let mut model = AppModel::empty();

    model.apply_command(AppCommand::RequestCreateScript);
    model.apply_command(AppCommand::CreateScript);
    model.apply_command(AppCommand::RunScript);

    let execution = &model.snapshot().scripts.executions[0];
    let log = model.snapshot().scripts.log_lines.last().unwrap();
    assert_real_timestamp(&execution.timestamp);
    assert_real_timestamp(&log.timestamp);
}

fn assert_real_timestamp(timestamp: &str) {
    assert_ne!(timestamp, "now");
    assert!(!timestamp
        .chars()
        .all(|character| character.is_ascii_digit()));
    assert!(timestamp.contains('T'));
    assert!(timestamp.contains('-'));
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
