use crate::error::{read_json, read_to_string, Result, StorageError};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptFile {
    pub relative_path: PathBuf,
    pub source: String,
    pub executions: Vec<LegacyScriptExecution>,
    pub logs: Vec<LegacyScriptLog>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyScriptExecution {
    #[serde(skip)]
    pub metadata_path: PathBuf,
    pub execution_id: Option<String>,
    pub connection_id: Option<String>,
    pub error: Option<LegacyScriptExecutionError>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub execution_time: Option<u64>,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyScriptExecutionError {
    pub error_type: Option<String>,
    pub error_msg: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LegacyScriptLog {
    pub execution_id: String,
    pub relative_path: PathBuf,
    pub content: String,
}

pub fn read_scripts(root: &Path) -> Result<Vec<ScriptFile>> {
    let script_root = root.join("scripts");
    if !script_root.exists() {
        return Ok(Vec::new());
    }
    let mut scripts = Vec::new();
    collect_scripts(&script_root, &script_root, &mut scripts)?;
    scripts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(scripts)
}

fn collect_scripts(root: &Path, dir: &Path, out: &mut Vec<ScriptFile>) -> Result<()> {
    for entry in std::fs::read_dir(dir).map_err(|source| StorageError::Read {
        path: dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| StorageError::Read {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            if is_sidecar_dir(root, &path) {
                continue;
            }
            collect_scripts(root, &path, out)?;
        } else if path.extension() == Some(OsStr::new("js")) {
            let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            out.push(ScriptFile {
                executions: read_executions(root, &relative_path)?,
                logs: read_logs(root, &relative_path)?,
                relative_path,
                source: read_to_string(path)?,
            });
        }
    }
    Ok(())
}

fn read_executions(root: &Path, script_path: &Path) -> Result<Vec<LegacyScriptExecution>> {
    let dir = sidecar_dir(root, "executions", script_path);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut executions = Vec::new();
    for path in sidecar_files(&dir, "json")? {
        let mut execution: LegacyScriptExecution = read_json(path.clone())?;
        execution.metadata_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        executions.push(execution);
    }
    executions.sort_by(|left, right| left.metadata_path.cmp(&right.metadata_path));
    Ok(executions)
}

fn read_logs(root: &Path, script_path: &Path) -> Result<Vec<LegacyScriptLog>> {
    let dir = sidecar_dir(root, "logs", script_path);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut logs = Vec::new();
    for path in sidecar_files(&dir, "log")? {
        let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let execution_id = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("unknown")
            .to_owned();
        logs.push(LegacyScriptLog {
            execution_id,
            relative_path,
            content: read_to_string(path)?,
        });
    }
    logs.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(logs)
}

fn sidecar_files(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|source| StorageError::Read {
        path: dir.to_path_buf(),
        source,
    })? {
        let path = entry
            .map_err(|source| StorageError::Read {
                path: dir.to_path_buf(),
                source,
            })?
            .path();
        if path.extension() == Some(OsStr::new(extension)) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn sidecar_dir(root: &Path, sidecar: &str, script_path: &Path) -> PathBuf {
    root.join(sidecar).join(script_path)
}

fn is_sidecar_dir(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    matches!(
        relative.components().next(),
        Some(std::path::Component::Normal(name)) if name == "logs" || name == "executions"
    )
}
