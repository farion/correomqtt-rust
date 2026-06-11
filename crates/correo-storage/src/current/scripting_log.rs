use serde::{Deserialize, Serialize};

use super::redact_script_log_text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptLogRecord {
    pub execution_id: String,
    pub sequence: u64,
    pub timestamp: Option<String>,
    pub level: ScriptLogLevel,
    pub message: String,
}

impl ScriptLogRecord {
    pub fn redacted(&self) -> Self {
        let mut record = self.clone();
        record.message = redact_script_log_text(&record.message).replace('\n', "\\n");
        record
    }

    pub fn from_persisted_line(execution_id: &str, sequence: u64, line: &str) -> Self {
        let trimmed = line.trim_start();
        if let Some((timestamp, rest)) = trimmed.split_once(' ') {
            if timestamp.contains('T') && timestamp.contains('-') {
                let (level, message) = ScriptLogLevel::parse_prefix(rest);
                if level.is_explicit(rest) {
                    return Self::new(
                        execution_id,
                        sequence,
                        Some(timestamp.to_owned()),
                        level,
                        message,
                    );
                }
            }
        }
        let (level, message) = ScriptLogLevel::parse_prefix(trimmed);
        Self::new(execution_id, sequence, None, level, message)
    }

    pub fn from_legacy_line(execution_id: &str, sequence: u64, line: &str) -> Self {
        let (level, message) = ScriptLogLevel::parse_prefix(line.trim_start());
        Self::new(execution_id, sequence, None, level, message)
    }

    pub fn to_persisted_line(&self) -> String {
        let record = self.redacted();
        match record.timestamp {
            Some(timestamp) => {
                format!("{} {} {}", timestamp, record.level.as_str(), record.message)
            }
            None => format!("{} {}", record.level.as_str(), record.message),
        }
    }

    fn new(
        execution_id: &str,
        sequence: u64,
        timestamp: Option<String>,
        level: ScriptLogLevel,
        message: &str,
    ) -> Self {
        Self {
            execution_id: execution_id.to_owned(),
            sequence,
            timestamp,
            level,
            message: redact_script_log_text(message.trim()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl ScriptLogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    fn parse_prefix(line: &str) -> (Self, &str) {
        for (prefix, level) in [
            ("TRACE", Self::Trace),
            ("DEBUG", Self::Debug),
            ("INFO", Self::Info),
            ("WARN", Self::Warn),
            ("ERROR", Self::Error),
        ] {
            if let Some(rest) = line.trim_start().strip_prefix(prefix) {
                return (level, rest);
            }
        }
        (Self::Info, line)
    }

    fn is_explicit(self, line: &str) -> bool {
        line.trim_start().starts_with(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedScriptLog {
    pub execution_id: String,
    pub max_records: usize,
    pub records: Vec<ScriptLogRecord>,
    pub truncated_count: usize,
}

impl BoundedScriptLog {
    pub fn new(execution_id: impl Into<String>, max_records: usize) -> Self {
        Self {
            execution_id: execution_id.into(),
            max_records,
            records: Vec::new(),
            truncated_count: 0,
        }
    }

    pub fn push(&mut self, record: ScriptLogRecord) {
        if self.max_records == 0 {
            self.truncated_count += 1;
            return;
        }
        self.records.push(record);
        let overflow = self.records.len().saturating_sub(self.max_records);
        if overflow > 0 {
            self.records.drain(0..overflow);
            self.truncated_count += overflow;
        }
    }
}
