use std::time::{Duration, Instant};

use correo_storage::current::{
    AppConfig, HistoryPersistenceSnapshot, ScriptExecution, ScriptExecutionStatus, ScriptFile,
    ScriptLogLevel, ScriptLogRecord, ScriptPersistenceSnapshot, ScriptStore,
};

use crate::{
    startup_state_from_current, ScriptExecutionStatus as UiScriptExecutionStatus, ScriptingCommand,
    ScriptingEvent, ScriptingWorker, ThemeMode,
};

#[test]
fn worker_runs_reported_promise_sample_and_persists_logs() {
    let temp = tempfile::tempdir().unwrap();
    let worker = ScriptingWorker::start(temp.path());
    let source = "const client = clientFactory.getPromiseClient();\nlogger.info('starting new_script.js');\nqueue.process();\n";

    worker
        .dispatch(ScriptingCommand::Create {
            path: "new_script.js".to_owned(),
            source: source.to_owned(),
        })
        .unwrap();
    assert!(matches!(
        worker.recv_event_timeout(Duration::from_secs(2)),
        Some(ScriptingEvent::Stored { .. })
    ));

    worker
        .dispatch(ScriptingCommand::Run {
            execution_id: "exec-1".to_owned(),
            script_name: "new_script.js".to_owned(),
            script_path: "new_script.js".to_owned(),
            source: source.to_owned(),
            connection_id: Some("connection-1".to_owned()),
        })
        .unwrap();

    let mut saw_log = false;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let event = worker.recv_event_timeout(Duration::from_millis(50));
        match event {
            Some(ScriptingEvent::LogAppended { message, .. }) => {
                saw_log |= message.contains("starting new_script.js");
            }
            Some(ScriptingEvent::ExecutionUpdated { status, error, .. }) => {
                assert_eq!(status, UiScriptExecutionStatus::Succeeded);
                assert_eq!(error, None);
                break;
            }
            Some(_) => {}
            None if Instant::now() > deadline => panic!("script did not finish"),
            None => {}
        }
    }
    assert!(saw_log);

    let snapshot = ScriptStore::new(temp.path()).load_snapshot(200).unwrap();
    assert_eq!(snapshot.files[0].name, "new_script.js");
    assert_eq!(
        snapshot.executions[0].status,
        ScriptExecutionStatus::Succeeded
    );
    assert!(snapshot.logs.iter().any(|log| {
        log.execution_id == "exec-1" && log.message.contains("starting new_script.js")
    }));
}

#[test]
fn startup_hydrates_scripts_from_persistence_snapshot() {
    let scripts = ScriptPersistenceSnapshot {
        files: vec![ScriptFile::new(
            "stored.js".into(),
            "logger.info('stored');".to_owned(),
        )],
        executions: vec![ScriptExecution {
            execution_id: "exec-stored".to_owned(),
            script_name: "stored.js".to_owned(),
            script_path: "stored.js".into(),
            connection_id: None,
            status: ScriptExecutionStatus::Succeeded,
            error: None,
            started_at: Some("stored-time".to_owned()),
            ended_at: None,
            duration_ms: Some(42),
            cancelled: false,
            log_path: None,
        }],
        logs: vec![ScriptLogRecord {
            execution_id: "exec-stored".to_owned(),
            sequence: 0,
            timestamp: Some("stored-time".to_owned()),
            level: ScriptLogLevel::Info,
            message: "stored log".to_owned(),
        }],
    };

    let state = startup_state_from_current(
        AppConfig::default(),
        HistoryPersistenceSnapshot::default(),
        scripts,
        Vec::new(),
        ThemeMode::Dark,
    );

    assert_eq!(state.snapshot.scripts.selected_script, "stored.js");
    assert_eq!(
        state.snapshot.scripts.scripts[0].source,
        "logger.info('stored');"
    );
    assert_eq!(
        state.snapshot.scripts.executions[0].status,
        UiScriptExecutionStatus::Succeeded
    );
    assert_eq!(
        state.snapshot.scripts.log_lines[0].execution_id,
        "exec-stored"
    );
}
