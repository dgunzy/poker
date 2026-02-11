//! Tauri Store plugin backend for preferences.
//!
//! Uses the Store plugin internally (JSON). The PreferencesStorage trait
//! keeps the rest of the app decoupled from this implementation.

use serde_json::Value;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::storage::{Preferences, PreferencesStorage, StorageError};

pub struct StoreBackend {
    app: AppHandle,
    path: String,
}

impl StoreBackend {
    pub fn new(app: AppHandle, path: impl Into<String>) -> Self {
        Self {
            app,
            path: path.into(),
        }
    }
}

impl PreferencesStorage for StoreBackend {
    fn load(&self) -> Result<Preferences, StorageError> {
        let store = self
            .app
            .store(&self.path)
            .map_err(|e| StorageError::Io(e.to_string()))?;

        let value = store
            .get("preferences")
            .and_then(|v| serde_json::to_value(v).ok())
            .unwrap_or(Value::Null);

        if value.is_null() {
            return Ok(Preferences::default());
        }

        serde_json::from_value(value).map_err(|e| StorageError::Serialization(e.to_string()))
    }

    fn save(&self, prefs: &Preferences) -> Result<(), StorageError> {
        let store = self
            .app
            .store(&self.path)
            .map_err(|e| StorageError::Io(e.to_string()))?;

        let value = serde_json::to_value(prefs).map_err(|e| StorageError::Serialization(e.to_string()))?;
        store.set("preferences", value);
        store.save().map_err(|e| StorageError::Io(e.to_string()))?;

        Ok(())
    }
}
