pub mod current;
pub mod error;
pub mod legacy;
#[path = "migration.rs"]
pub mod migration;

pub use error::{Result, StorageError};
