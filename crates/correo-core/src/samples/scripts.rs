use correo_mqtt::ConnectionId;

use crate::{
    ScriptExecutionRow, ScriptExecutionStatus, ScriptFileStatus, ScriptLogLevel, ScriptLogLine,
    ScriptRow, ScriptSurfaceSnapshot,
};

pub(super) fn sample_scripts(selected_connection: Option<ConnectionId>) -> ScriptSurfaceSnapshot {
    ScriptSurfaceSnapshot {
        selected_connection: "Local Broker".to_owned(),
        selected_connection_id: selected_connection.map(|id| id.to_string()),
        selected_script: "payload_replay.js".to_owned(),
        scripts: vec![
            script("payload_replay.js", ScriptFileStatus::Running, 12),
            script("validation_runner.js", ScriptFileStatus::Ready, 4),
            script("retain_cleanup.js", ScriptFileStatus::Error, 2),
        ],
        running: true,
        active_execution_id: Some("exec-1001".to_owned()),
        executions: vec![
            execution(
                "exec-1001",
                ScriptExecutionStatus::Running,
                "00:00:18",
                "10:25:02",
            ),
            execution(
                "exec-0999",
                ScriptExecutionStatus::Succeeded,
                "00:01:42",
                "09:44:31",
            ),
            execution(
                "exec-0998",
                ScriptExecutionStatus::Cancelled,
                "00:00:09",
                "09:12:07",
            ),
        ],
        log_lines: vec![
            log(
                "10:25:02",
                ScriptLogLevel::Info,
                "logger.info replay started",
            ),
            log(
                "10:25:03",
                ScriptLogLevel::Info,
                "queue.process telemetry/device-42/state",
            ),
            log("10:25:04", ScriptLogLevel::Debug, "sleep(250)"),
        ],
        ..ScriptSurfaceSnapshot::default()
    }
}

fn script(name: &str, status: ScriptFileStatus, execution_count: usize) -> ScriptRow {
    ScriptRow {
        name: name.to_owned(),
        relative_path: format!("scripts/{name}"),
        status,
        execution_count,
        source: "logger.info('running');\nqueue.process();".to_owned(),
        saved_source: "logger.info('running');\nqueue.process();".to_owned(),
        persisted: true,
    }
}

fn execution(
    id: &str,
    status: ScriptExecutionStatus,
    duration: &str,
    timestamp: &str,
) -> ScriptExecutionRow {
    ScriptExecutionRow {
        execution_id: id.to_owned(),
        script_name: "payload_replay.js".to_owned(),
        status,
        duration: duration.to_owned(),
        timestamp: timestamp.to_owned(),
        error: None,
    }
}

fn log(timestamp: &str, level: ScriptLogLevel, message: &str) -> ScriptLogLine {
    ScriptLogLine {
        execution_id: "exec-1001".to_owned(),
        timestamp: timestamp.to_owned(),
        level,
        message: message.to_owned(),
    }
}
