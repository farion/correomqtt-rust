use correo_storage::current::{
    ScriptExecution as StoredScriptExecution,
    ScriptExecutionErrorType as StoredScriptExecutionErrorType,
    ScriptExecutionStatus as StoredScriptExecutionStatus, ScriptLogLevel as StoredScriptLogLevel,
    ScriptLogRecord, ScriptPersistenceSnapshot,
};

use crate::{
    AppSnapshot, ScriptExecutionError, ScriptExecutionErrorKind, ScriptExecutionRow,
    ScriptExecutionStatus, ScriptFileStatus, ScriptLogLevel, ScriptLogLine, ScriptRow,
    ScriptSurfaceSnapshot,
};

pub(super) fn script_surface(scripts: &ScriptPersistenceSnapshot) -> ScriptSurfaceSnapshot {
    let rows = scripts
        .files
        .iter()
        .map(|file| script_row(file, scripts))
        .collect::<Vec<_>>();
    let selected_script = rows
        .first()
        .map(|script| script.name.clone())
        .unwrap_or_default();
    let mut executions = scripts
        .executions
        .iter()
        .map(script_execution_row)
        .collect::<Vec<_>>();
    executions.reverse();
    let selected_execution_id = executions
        .first()
        .map(|execution| execution.execution_id.clone());
    let active_execution_id = executions
        .iter()
        .find(|execution| !execution.status.is_terminal())
        .map(|execution| execution.execution_id.clone());
    let running = active_execution_id.is_some();
    ScriptSurfaceSnapshot {
        selected_script,
        scripts: rows,
        running,
        active_execution_id,
        selected_execution_id,
        executions,
        log_lines: scripts.logs.iter().map(script_log_line).collect(),
        ..ScriptSurfaceSnapshot::default()
    }
}

pub(super) fn apply_default_script_connection(snapshot: &mut AppSnapshot) {
    let selected = snapshot
        .selected_connection()
        .or_else(|| snapshot.connections.first())
        .map(|connection| (connection.name.clone(), connection.id.to_string()));
    if let Some((name, id)) = selected {
        snapshot.scripts.selected_connection = name;
        snapshot.scripts.selected_connection_id = Some(id);
    } else {
        snapshot.scripts.selected_connection = "No connection selected".to_owned();
        snapshot.scripts.selected_connection_id = None;
    }
}

fn script_row(
    file: &correo_storage::current::ScriptFile,
    scripts: &ScriptPersistenceSnapshot,
) -> ScriptRow {
    let execution_count = scripts
        .executions
        .iter()
        .filter(|execution| execution.script_path == file.relative_path)
        .count();
    ScriptRow {
        name: file.name.clone(),
        relative_path: file.relative_path.to_string_lossy().into_owned(),
        status: ScriptFileStatus::Ready,
        execution_count,
        source: file.source.clone(),
        saved_source: file.source.clone(),
        persisted: true,
    }
}

fn script_execution_row(execution: &StoredScriptExecution) -> ScriptExecutionRow {
    ScriptExecutionRow {
        execution_id: execution.execution_id.clone(),
        script_name: execution.script_name.clone(),
        status: script_execution_status(execution.status),
        duration: execution
            .duration_ms
            .map(format_duration)
            .unwrap_or_else(|| script_execution_status(execution.status).label().to_owned()),
        timestamp: execution
            .started_at
            .clone()
            .unwrap_or_else(|| "stored".to_owned()),
        error: execution.error.as_ref().map(|error| ScriptExecutionError {
            kind: match error.error_type {
                StoredScriptExecutionErrorType::Guest => ScriptExecutionErrorKind::JavaScriptGuest,
                StoredScriptExecutionErrorType::Host => ScriptExecutionErrorKind::HostApi,
            },
            message: error.message.clone(),
        }),
    }
}

fn script_execution_status(status: StoredScriptExecutionStatus) -> ScriptExecutionStatus {
    match status {
        StoredScriptExecutionStatus::Queued => ScriptExecutionStatus::Queued,
        StoredScriptExecutionStatus::Running => ScriptExecutionStatus::Running,
        StoredScriptExecutionStatus::Succeeded => ScriptExecutionStatus::Succeeded,
        StoredScriptExecutionStatus::Failed => ScriptExecutionStatus::Failed,
        StoredScriptExecutionStatus::Cancelled => ScriptExecutionStatus::Cancelled,
    }
}

fn script_log_line(record: &ScriptLogRecord) -> ScriptLogLine {
    ScriptLogLine {
        execution_id: record.execution_id.clone(),
        timestamp: record.timestamp.clone().unwrap_or_default(),
        level: match record.level {
            StoredScriptLogLevel::Trace | StoredScriptLogLevel::Debug => ScriptLogLevel::Debug,
            StoredScriptLogLevel::Info => ScriptLogLevel::Info,
            StoredScriptLogLevel::Warn => ScriptLogLevel::Warning,
            StoredScriptLogLevel::Error => ScriptLogLevel::Error,
        },
        message: record.message.clone(),
    }
}

fn format_duration(duration_ms: u64) -> String {
    if duration_ms < 1000 {
        return format!("{duration_ms} ms");
    }
    let seconds = duration_ms / 1000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
