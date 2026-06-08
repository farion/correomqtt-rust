mod crypto;
mod dto;
mod message;
mod wire;

use std::path::{Path, PathBuf};

use crate::error::{read_to_string, Result, StorageError};

use super::Message;
use super::{ConnectionConfig, ImportedSecret, PasswordEncryption};

pub(crate) const AES_GCM: &str = "AES/GCM/NoPadding";

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionExport {
    Plain(ConnectionImport),
    Encrypted(EncryptedConnectionExport),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConnectionImport {
    pub connections: Vec<ConnectionConfig>,
    pub secrets: Vec<ImportedSecret>,
    pub warnings: Vec<ConnectionImportWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptedConnectionExport {
    pub encryption: PasswordEncryption,
    pub encrypted_data: String,
    pub warnings: Vec<ConnectionImportWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionImportWarning {
    pub code: &'static str,
    pub message: String,
}

pub fn read_connection_export(path: impl AsRef<Path>) -> Result<ConnectionExport> {
    parse_connection_export_json(&read_to_string(path.as_ref().to_path_buf())?)
}

pub fn read_message_export(path: impl AsRef<Path>) -> Result<Message> {
    message::read_message_export(path.as_ref())
}

pub fn import_connection_export_json(
    json: &str,
    password: Option<&str>,
) -> Result<ConnectionImport> {
    match parse_connection_export_json(json)? {
        ConnectionExport::Plain(import) => Ok(import),
        ConnectionExport::Encrypted(export) => {
            let password = password.ok_or(StorageError::InvalidConnectionExportPayload(
                "encrypted connection export requires a password",
            ))?;
            decrypt_connection_export(&export, password)
        }
    }
}

pub fn parse_connection_export_json(json: &str) -> Result<ConnectionExport> {
    wire::parse_connection_export_json(json)
}

pub fn parse_message_export_json(json: &str) -> Result<Message> {
    message::parse_message_export_json(json)
}

pub fn decrypt_connection_export(
    export: &EncryptedConnectionExport,
    password: &str,
) -> Result<ConnectionImport> {
    if export.encryption != PasswordEncryption::AesGcmNoPadding {
        return Err(StorageError::UnsupportedConnectionExportEncryption(
            wire::encryption_name(export.encryption).to_owned(),
        ));
    }

    let plaintext = crypto::decrypt_gcm(&export.encrypted_data, password)?;
    wire::connection_import_from_encrypted_json(&plaintext, export.warnings.clone())
}

pub fn connection_export_plain_json(import: &ConnectionImport) -> Result<String> {
    wire::plain_export_json(import)
}

pub fn message_export_json(message: &Message) -> Result<String> {
    message::message_export_json(message)
}

pub fn connection_export_encrypted_json(
    import: &ConnectionImport,
    password: &str,
) -> Result<String> {
    if password.is_empty() {
        return Err(StorageError::InvalidConnectionExportPayload(
            "encrypted connection export password is empty",
        ));
    }

    let payload = wire::encrypted_payload_json(import)?;
    wire::encrypted_export_json(crypto::encrypt_gcm(&payload, password)?)
}

pub fn write_plain_connection_export(
    path: impl Into<PathBuf>,
    import: &ConnectionImport,
) -> Result<()> {
    write_export(path.into(), connection_export_plain_json(import)?)
}

pub fn write_message_export(path: impl Into<PathBuf>, message: &Message) -> Result<()> {
    message::write_message_export(path.into(), message)
}

pub fn write_encrypted_connection_export(
    path: impl Into<PathBuf>,
    import: &ConnectionImport,
    password: &str,
) -> Result<()> {
    write_export(
        path.into(),
        connection_export_encrypted_json(import, password)?,
    )
}

fn write_export(path: PathBuf, text: String) -> Result<()> {
    std::fs::write(&path, text).map_err(|source| StorageError::Write { path, source })
}
