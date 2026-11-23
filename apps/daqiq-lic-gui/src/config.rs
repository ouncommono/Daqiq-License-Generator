use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::startup_log::get_app_data_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    pub fn config_path() -> PathBuf {
        let dir = get_app_data_dir();
        dir.join("config.json")
    }

    pub fn load() -> Self {
        let p = Self::config_path();
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                match serde_json::from_str::<Self>(&content) {
                    Ok(cfg) => return cfg,
                    Err(_) => {
                        let bak = p.with_extension("json.corrupt.bak");
                        let _ = std::fs::rename(&p, &bak);
                    }
                }
            }
        }
        let def = Self::default();
        def.save();
        def
    }

    pub fn save(&self) {
        let p = Self::config_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, content);
        }
    }
}
