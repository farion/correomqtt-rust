use std::path::{Path, PathBuf};

use crate::current::Message;
use crate::error::{read_to_string, Result, StorageError};

pub(super) fn read_message_export(path: &Path) -> Result<Message> {
    parse_message_export_json(&read_to_string(path.to_path_buf())?)
}

pub(super) fn parse_message_export_json(json: &str) -> Result<Message> {
    serde_json::from_str(json).map_err(|source| StorageError::MessageExportJson { source })
}

pub(super) fn message_export_json(message: &Message) -> Result<String> {
    serde_json::to_string_pretty(message)
        .map_err(|source| StorageError::MessageExportJsonSerialize { source })
}

pub(super) fn write_message_export(path: PathBuf, message: &Message) -> Result<()> {
    std::fs::write(&path, message_export_json(message)?)
        .map_err(|source| StorageError::Write { path, source })
}
