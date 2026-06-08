use crate::current::{
    ScriptExecution as CurrentScriptExecution, ScriptExecutionError as CurrentScriptExecutionError,
    ScriptExecutionErrorType, ScriptExecutionStatus, ScriptFile as CurrentScriptFile,
    ScriptLogRecord, ScriptPersistenceSnapshot,
};
use crate::legacy::{
    LegacyScriptExecution, LegacyScriptExecutionError, ScriptFile as LegacyScriptFile,
};

use super::{record_extra_fields, MigrationReport, MigrationWarning};

pub(super) fn record_script_unknowns(scripts: &[LegacyScriptFile], report: &mut MigrationReport) {
    for script in scripts {
        let script_path = script.relative_path.to_string_lossy();
        for execution in &script.executions {
            let execution_path = format!(
                "scripts.executions.{}.{}",
                script_path,
                execution_id_from_path(execution)
            );
            record_extra_fields(&execution_path, &execution.extra, report);
            if let Some(error) = &execution.error {
                record_extra_fields(&format!("{execution_path}.error"), &error.extra, report);
            }
        }
    }
}

pub(super) fn migrate_scripts(
    scripts: &[LegacyScriptFile],
    report: &mut MigrationReport,
) -> ScriptPersistenceSnapshot {
    let mut snapshot = ScriptPersistenceSnapshot::default();
    for script in scripts {
        snapshot.files.push(CurrentScriptFile::new(
            script.relative_path.clone(),
            script.source.clone(),
        ));
        for execution in &script.executions {
            snapshot
                .executions
                .push(migrate_script_execution(script, execution, report));
        }
        for log in &script.logs {
            for (index, line) in log.content.lines().enumerate() {
                snapshot.logs.push(ScriptLogRecord::from_legacy_line(
                    &log.execution_id,
                    index as u64,
                    line,
                ));
            }
        }
    }
    snapshot
}

fn migrate_script_execution(
    script: &LegacyScriptFile,
    execution: &LegacyScriptExecution,
    report: &mut MigrationReport,
) -> CurrentScriptExecution {
    let execution_id = execution
        .execution_id
        .clone()
        .unwrap_or_else(|| execution_id_from_path(execution));
    let error = execution
        .error
        .as_ref()
        .map(|error| migrate_script_execution_error(error, &execution_id, report));
    let status = if execution.cancelled {
        ScriptExecutionStatus::Cancelled
    } else if error.is_some() {
        ScriptExecutionStatus::Failed
    } else if execution.execution_time.is_some() {
        ScriptExecutionStatus::Succeeded
    } else {
        ScriptExecutionStatus::Running
    };
    let script_name = script
        .relative_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let log_path = script
        .logs
        .iter()
        .find(|log| log.execution_id == execution_id)
        .map(|log| log.relative_path.clone());
    CurrentScriptExecution {
        execution_id,
        script_name,
        script_path: script.relative_path.clone(),
        connection_id: execution.connection_id.clone(),
        status,
        error,
        started_at: execution.start_time.clone(),
        ended_at: execution.end_time.clone(),
        duration_ms: execution.execution_time,
        cancelled: execution.cancelled,
        log_path,
    }
}

fn migrate_script_execution_error(
    error: &LegacyScriptExecutionError,
    execution_id: &str,
    report: &mut MigrationReport,
) -> CurrentScriptExecutionError {
    let error_type = match error.error_type.as_deref() {
        Some("GUEST" | "Guest" | "guest") => ScriptExecutionErrorType::Guest,
        Some("HOST" | "Host" | "host") | None => ScriptExecutionErrorType::Host,
        Some(other) => {
            report.warnings.push(MigrationWarning {
                code: "legacy_script_error_type_unknown",
                message: format!(
                    "Legacy script execution {execution_id} has unknown error type {other}; mapped to host error"
                ),
            });
            ScriptExecutionErrorType::Host
        }
    };
    CurrentScriptExecutionError {
        error_type,
        message: error.error_msg.clone().unwrap_or_default(),
    }
}

fn execution_id_from_path(execution: &LegacyScriptExecution) -> String {
    execution
        .metadata_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_owned())
}
