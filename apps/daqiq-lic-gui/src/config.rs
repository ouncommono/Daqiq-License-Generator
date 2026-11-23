use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme_mode: String, // "dark" or "light"
    pub default_output_dir: Option<String>,
    pub last_vault_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: "dark".to_string(),
            default_output_dir: None,
            last_vault_path: None,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        let mut dir = dirs::data_local_dir()?;
        dir.push("DaqiqLicenseGenerator");
        let _ = std::fs::create_dir_all(&dir);
        Some(dir.join("config.json"))
    }

    pub fn load() -> Self {
        if let Some(p) = Self::config_path() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                if let Ok(cfg) = serde_json::from_str::<Self>(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(p) = Self::config_path() {
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(p, content);
            }
        }
    }
}
