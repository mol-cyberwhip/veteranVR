use crate::models::settings::Settings;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SettingsService {
    settings: Arc<RwLock<Settings>>,
    config_path: PathBuf,
}

impl SettingsService {
    pub fn new() -> Self {
        let config_path = dirs::home_dir()
            .map(|p| p.join(".veteran").join("settings.json"))
            .unwrap_or_else(|| PathBuf::from("settings.json"));

        let settings = if config_path.exists() {
            std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Settings::default()
        };

        // Ensure download directory exists
        if !settings.download_dir.is_empty() {
            let download_path = PathBuf::from(&settings.download_dir);
            if !download_path.exists() {
                let _ = std::fs::create_dir_all(&download_path);
            }
        }

        Self {
            settings: Arc::new(RwLock::new(settings)),
            config_path,
        }
    }

    /// Create a SettingsService from pre-loaded settings (for testing)
    #[doc(hidden)]
    pub fn from_settings(settings: Settings, config_path: PathBuf) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings)),
            config_path,
        }
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsService {
    pub async fn get_settings(&self) -> Settings {
        self.settings.read().await.clone()
    }

    pub fn get_settings_sync(&self) -> Settings {
        self.settings.blocking_read().clone()
    }

    pub async fn save(&self) -> Result<()> {
        let settings = self.settings.read().await;
        let json = serde_json::to_string_pretty(&*settings)?;

        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.config_path, json).await?;
        Ok(())
    }

    pub async fn update_settings(&self, new_settings: Settings) -> Result<()> {
        // Ensure download directory exists if it changed
        if !new_settings.download_dir.is_empty() {
            let download_path = PathBuf::from(&new_settings.download_dir);
            if !download_path.exists() {
                let _ = tokio::fs::create_dir_all(&download_path).await;
            }
        }

        let mut settings = self.settings.write().await;
        *settings = new_settings;
        drop(settings); // Release lock
        self.save().await
    }

    // Patch settings supports partial updates from a JSON value
    pub async fn patch_settings(&self, patch: serde_json::Value) -> Result<Settings> {
        let mut current_settings = self.settings.write().await;
        let mut current_json = serde_json::to_value(&*current_settings)?;

        if let serde_json::Value::Object(map) = patch {
            if let serde_json::Value::Object(ref mut current_map) = current_json {
                for (k, v) in map {
                    current_map.insert(k, v);
                }
            }
        }

        let new_settings: Settings = serde_json::from_value(current_json)?;
        *current_settings = new_settings.clone();
        drop(current_settings); // Release lock

        self.save().await?;
        Ok(new_settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_settings_persistence() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("settings.json");

        // Initialize service with custom path
        let service = SettingsService {
            settings: Arc::new(RwLock::new(Settings::default())),
            config_path: config_path.clone(),
        };

        // Modify settings
        let mut settings = service.get_settings().await;
        settings.window_width = 1234;
        service.update_settings(settings.clone()).await.unwrap();

        // Verify file was written
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        let saved_settings: Settings = serde_json::from_str(&content).unwrap();
        assert_eq!(saved_settings.window_width, 1234);
    }
}
