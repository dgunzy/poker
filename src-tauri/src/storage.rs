//! Persistent preferences storage abstraction.
//!
//! The storage backend is decoupled from the persistence format. Implementations
//! may use JSON (Store), SQLite, or a remote API; the trait hides these details.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// User preferences and last-used state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub game_id: String,
    pub draw_count: u32,
    pub training_mode: TrainingMode,
    pub training_draw_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingMode {
    Percentile,
    Generator,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            game_id: "deuce_to_seven".to_string(),
            draw_count: 1000,
            training_mode: TrainingMode::Percentile,
            training_draw_count: 1000,
        }
    }
}

/// Abstraction over where preferences are persisted.
///
/// Implementations can use JSON files, SQLite, or a remote backend.
/// The trait is format-agnostic; serialization is an implementation detail.
pub trait PreferencesStorage: Send + Sync {
    fn load(&self) -> Result<Preferences, StorageError>;
    fn save(&self, prefs: &Preferences) -> Result<(), StorageError>;
}

#[derive(Debug)]
pub enum StorageError {
    Io(String),
    Serialization(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(s) => write!(f, "storage io: {}", s),
            StorageError::Serialization(s) => write!(f, "serialization: {}", s),
        }
    }
}

impl std::error::Error for StorageError {}

/// Erased storage backend for dependency injection.
/// Swap StoreBackend for SqliteBackend or RemoteBackend without changing call sites.
pub struct StorageState(pub Arc<dyn PreferencesStorage>);
