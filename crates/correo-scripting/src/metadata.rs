use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptExecutionId(Uuid);

impl ScriptExecutionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ScriptExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptExecutionMetadata {
    pub id: ScriptExecutionId,
    pub script_name: String,
    pub started_at: OffsetDateTime,
    pub status: ScriptExecutionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptExecutionStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}
