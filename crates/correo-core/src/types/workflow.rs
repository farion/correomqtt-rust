use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowFeedback {
    pub severity: WorkflowFeedbackSeverity,
    pub message: String,
}

impl WorkflowFeedback {
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(WorkflowFeedbackSeverity::Info, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(WorkflowFeedbackSeverity::Warning, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(WorkflowFeedbackSeverity::Error, message)
    }

    fn new(severity: WorkflowFeedbackSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowFeedbackSeverity {
    Info,
    Warning,
    Error,
}
