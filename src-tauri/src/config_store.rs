use std::fs;
use std::path::{Path, PathBuf};

use serde_json;
use tracing;

use crate::types::*;

const SCHEMA_VERSION: u32 = 1;
const MAX_BACKUPS: u32 = 5;
const MAX_ZONES_PER_MONITOR: usize = 64;
const MAX_NAME_LENGTH: usize = 64;
const MAX_GAP: u32 = 100;
const MAX_MARGIN: u32 = 100;
const SUPPORTED_LANGUAGES: &[&str] = &["en", "vi"];

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
                        tracing::error!(
                            "Config validation failed: {}. Falling back to defaults.",
                            e
                        );
                        return ConfigFile::default();
                    }
                    config
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to parse config JSON: {}. Falling back to defaults.",
                        e
                    );
                    ConfigFile::default()
                }
            },
            Err(e) => {
                tracing::error!(
                    "Failed to read config file: {}. Falling back to defaults.",
                    e
                );
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

        let verify = fs::read_to_string(&tmp_path).map_err(|e| ConfigError::Io(e.to_string()))?;
        let _: ConfigFile = serde_json::from_str(&verify).map_err(|e| {
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

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).ok();
        }

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
        validate_settings(&config.settings, &config.layouts)?;
        Ok(())
    }
}

fn backup_path(base: &Path, n: u32) -> PathBuf {
    base.with_extension(format!("json.bak.{}", n))
}

fn validate_saved_layout(layout: &SavedLayout) -> Result<(), ConfigError> {
    if layout.name.trim().is_empty() || layout.name.len() > MAX_NAME_LENGTH {
        return Err(ConfigError::Validation(format!(
            "Layout name must be 1-{} characters",
            MAX_NAME_LENGTH
        )));
    }
    if layout.zones.len() > MAX_ZONES_PER_MONITOR {
        return Err(ConfigError::Validation(format!(
            "Max {} zones per layout",
            MAX_ZONES_PER_MONITOR
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
        return Err(ConfigError::Validation(
            "Zone name must be 1-64 characters".into(),
        ));
    }
    if !zone.x.is_finite() || zone.x < 0.0 || zone.x > 1.0 {
        return Err(ConfigError::Validation(
            "Zone x must be finite and in [0.0, 1.0]".into(),
        ));
    }
    if !zone.y.is_finite() || zone.y < 0.0 || zone.y > 1.0 {
        return Err(ConfigError::Validation(
            "Zone y must be finite and in [0.0, 1.0]".into(),
        ));
    }
    if !zone.width.is_finite() || zone.width <= 0.0 || zone.width > 1.0 {
        return Err(ConfigError::Validation(
            "Zone width must be finite, > 0 and ≤ 1.0".into(),
        ));
    }
    if !zone.height.is_finite() || zone.height <= 0.0 || zone.height > 1.0 {
        return Err(ConfigError::Validation(
            "Zone height must be finite, > 0 and ≤ 1.0".into(),
        ));
    }
    if zone.x + zone.width > 1.0001 || zone.y + zone.height > 1.0001 {
        return Err(ConfigError::Validation(
            "Zone exceeds monitor bounds".into(),
        ));
    }
    if zone.gap > MAX_GAP {
        return Err(ConfigError::Validation(format!(
            "Zone gap must be ≤ {}",
            MAX_GAP
        )));
    }
    if zone.margin > MAX_MARGIN {
        return Err(ConfigError::Validation(format!(
            "Zone margin must be ≤ {}",
            MAX_MARGIN
        )));
    }
    // HTML-escape names on save to prevent stored XSS
    let escaped = zone
        .name
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;");
    if escaped.len() > MAX_NAME_LENGTH * 6 {
        return Err(ConfigError::Validation(
            "Zone name too long after escaping".into(),
        ));
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
                    "Zones '{}' and '{}' overlap",
                    a.name, b.name
                )));
            }
        }
    }
    Ok(())
}

fn validate_accent_color(color: &str) -> Result<(), ConfigError> {
    if color.len() != 7 || !color.starts_with('#') {
        return Err(ConfigError::Validation(
            "Accent color must be in #[0-9A-Fa-f]{6} format".into(),
        ));
    }
    for ch in color[1..].chars() {
        if !ch.is_ascii_hexdigit() {
            return Err(ConfigError::Validation(
                "Accent color must be in #[0-9A-Fa-f]{6} format".into(),
            ));
        }
    }
    Ok(())
}

fn validate_settings(settings: &AppSettings, layouts: &[SavedLayout]) -> Result<(), ConfigError> {
    validate_accent_color(&settings.accent_color)?;

    if !SUPPORTED_LANGUAGES.contains(&settings.language.as_str()) {
        return Err(ConfigError::Validation(format!(
            "Unsupported language '{}'. Supported: {:?}",
            settings.language, SUPPORTED_LANGUAGES
        )));
    }

    if let Some(ref layout_id) = settings.default_layout_id {
        if !layouts.iter().any(|l| l.id == *layout_id) {
            return Err(ConfigError::Validation(format!(
                "Default layout ID '{}' references a non-existent layout",
                layout_id
            )));
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
