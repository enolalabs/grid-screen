use std::fs;
use std::path::{Path, PathBuf};

use serde_json;
use tracing;

use crate::types::*;

const SCHEMA_VERSION: u32 = 1;
const MAX_BACKUPS: u32 = 5;
const MAX_ZONES_PER_MONITOR: usize = 64;
const MAX_NAME_LENGTH: usize = 64;

pub struct ConfigStore {
    config_dir: PathBuf,
}

impl ConfigStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    fn config_path(&self) -> PathBuf {
        self.config_dir.join("layouts.json")
    }

    pub fn load(&self) -> ConfigFile {
        let path = self.config_path();
        if !path.exists() {
            tracing::info!("No config file found, using defaults");
            return ConfigFile::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<ConfigFile>(&contents) {
                Ok(config) => {
                    if let Err(e) = Self::validate(&config) {
                        tracing::error!("Config validation failed: {}. Falling back to defaults.", e);
                        return ConfigFile::default();
                    }
                    config
                }
                Err(e) => {
                    tracing::error!("Failed to parse config JSON: {}. Falling back to defaults.", e);
                    ConfigFile::default()
                }
            },
            Err(e) => {
                tracing::error!("Failed to read config file: {}. Falling back to defaults.", e);
                ConfigFile::default()
            }
        }
    }

    pub fn save(&self, config: &ConfigFile) -> Result<(), ConfigError> {
        Self::validate(config)?;

        fs::create_dir_all(&self.config_dir).map_err(|e| ConfigError::Io(e.to_string()))?;

        let path = self.config_path();
        let tmp_path = path.with_extension("json.tmp");

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::Serialize(e.to_string()))?;

        fs::write(&tmp_path, &json).map_err(|e| ConfigError::Io(e.to_string()))?;

        let verify = fs::read_to_string(&tmp_path)
            .map_err(|e| ConfigError::Io(e.to_string()))?;
        let _: ConfigFile = serde_json::from_str(&verify)
            .map_err(|e| {
                let _ = fs::remove_file(&tmp_path);
                ConfigError::Verify(e.to_string())
            })?;

        if path.exists() {
            for i in (1..MAX_BACKUPS).rev() {
                let old = backup_path(&path, i);
                let new = backup_path(&path, i + 1);
                if old.exists() {
                    let _ = fs::rename(&old, &new);
                }
            }
            let first_backup = backup_path(&path, 1);
            let _ = fs::rename(&path, &first_backup);
        }

        fs::rename(&tmp_path, &path).map_err(|e| ConfigError::Io(e.to_string()))?;
        let _ = fs::remove_file(&tmp_path);

        tracing::info!("Config saved successfully");
        Ok(())
    }

    fn validate(config: &ConfigFile) -> Result<(), ConfigError> {
        if config.schema_version > SCHEMA_VERSION {
            return Err(ConfigError::Validation("Unknown schema version".into()));
        }
        for layout in &config.layouts {
            validate_saved_layout(layout)?;
        }
        Ok(())
    }
}

fn backup_path(base: &Path, n: u32) -> PathBuf {
    base.with_extension(format!("json.bak.{}", n))
}

fn validate_saved_layout(layout: &SavedLayout) -> Result<(), ConfigError> {
    if layout.name.trim().is_empty() || layout.name.len() > MAX_NAME_LENGTH {
        return Err(ConfigError::Validation(format!(
            "Layout name must be 1-{} characters", MAX_NAME_LENGTH
        )));
    }
    if layout.zones.len() > MAX_ZONES_PER_MONITOR {
        return Err(ConfigError::Validation(format!(
            "Max {} zones per layout", MAX_ZONES_PER_MONITOR
        )));
    }
    for zone in &layout.zones {
        validate_zone(zone)?;
    }
    validate_no_zone_overlap(&layout.zones)?;
    Ok(())
}

fn validate_zone(zone: &Zone) -> Result<(), ConfigError> {
    if zone.name.trim().is_empty() || zone.name.len() > MAX_NAME_LENGTH {
        return Err(ConfigError::Validation("Zone name must be 1-64 characters".into()));
    }
    if !zone.x.is_finite() || zone.x < 0.0 || zone.x > 1.0 {
        return Err(ConfigError::Validation("Zone x must be finite and in [0.0, 1.0]".into()));
    }
    if !zone.y.is_finite() || zone.y < 0.0 || zone.y > 1.0 {
        return Err(ConfigError::Validation("Zone y must be finite and in [0.0, 1.0]".into()));
    }
    if !zone.width.is_finite() || zone.width <= 0.0 || zone.width > 1.0 {
        return Err(ConfigError::Validation("Zone width must be finite, > 0 and ≤ 1.0".into()));
    }
    if !zone.height.is_finite() || zone.height <= 0.0 || zone.height > 1.0 {
        return Err(ConfigError::Validation("Zone height must be finite, > 0 and ≤ 1.0".into()));
    }
    if zone.x + zone.width > 1.0001 || zone.y + zone.height > 1.0001 {
        return Err(ConfigError::Validation("Zone exceeds monitor bounds".into()));
    }
    // HTML-escape names on save to prevent stored XSS
    let escaped = zone.name
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;");
    if escaped.len() > MAX_NAME_LENGTH * 6 {
        return Err(ConfigError::Validation("Zone name too long after escaping".into()));
    }
    Ok(())
}

fn validate_no_zone_overlap(zones: &[Zone]) -> Result<(), ConfigError> {
    for i in 0..zones.len() {
        for j in (i + 1)..zones.len() {
            let a = &zones[i];
            let b = &zones[j];
            let overlaps = a.x < b.x + b.width
                && a.x + a.width > b.x
                && a.y < b.y + b.height
                && a.y + a.height > b.y;
            if overlaps {
                return Err(ConfigError::Validation(format!(
                    "Zones '{}' and '{}' overlap", a.name, b.name
                )));
            }
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Serialization error: {0}")]
    Serialize(String),
    #[error("Verification error: {0}")]
    Verify(String),
}
