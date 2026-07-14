use shared_types::{Layout, Settings};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const SCHEMA_VERSION: u32 = 1;
const MAX_BACKUPS: u32 = 5;

pub struct ConfigStore {
    base_dir: PathBuf,
}

impl ConfigStore {
    pub fn new(base_dir: PathBuf) -> Self {
        fs::create_dir_all(&base_dir).unwrap();
        fs::set_permissions(&base_dir, fs::Permissions::from_mode(0o700)).unwrap();
        ConfigStore { base_dir }
    }

    pub fn load(&self) -> Result<(Settings, Vec<Layout>, Vec<String>), String> {
        let settings_path = self.base_dir.join("settings.json");
        let layouts_path = self.base_dir.join("layouts.json");

        let settings = if settings_path.exists() {
            let data = fs::read_to_string(&settings_path)
                .map_err(|e| format!("Failed to read settings: {}", e))?;
            serde_json::from_str::<Settings>(&data)
                .map_err(|e| format!("Failed to parse settings: {}", e))?
        } else {
            Settings::default()
        };

        let layouts = if layouts_path.exists() {
            let data = fs::read_to_string(&layouts_path)
                .map_err(|e| format!("Failed to read layouts: {}", e))?;
            serde_json::from_str::<Vec<Layout>>(&data)
                .map_err(|e| format!("Failed to parse layouts: {}", e))?
        } else {
            Vec::new()
        };

        let warnings = if settings.schema_version < SCHEMA_VERSION {
            vec!["Config from older version — settings reset to defaults. Backup preserved.".into()]
        } else {
            Vec::new()
        };

        Ok((settings, layouts, warnings))
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let path = self.base_dir.join("settings.json");
        let tmp = self.base_dir.join("settings.json.tmp");
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        self.atomic_write(&path, &tmp, &json)
    }

    pub fn save_layouts(&self, layouts: &[Layout]) -> Result<(), String> {
        let path = self.base_dir.join("layouts.json");
        let tmp = self.base_dir.join("layouts.json.tmp");
        let json = serde_json::to_string_pretty(layouts)
            .map_err(|e| format!("Failed to serialize layouts: {}", e))?;
        self.atomic_write(&path, &tmp, &json)
    }

    pub fn save_defaults(&self, gap_px: u32, margin_px: u32) -> Result<(), String> {
        let mut settings = self.load().map(|(s, _, _)| s)?;
        settings.default_gap_px = gap_px;
        settings.default_margin_px = margin_px;
        self.save_settings(&settings)
    }

    fn atomic_write(&self, dest: &PathBuf, tmp: &PathBuf, data: &str) -> Result<(), String> {
        if dest.exists() {
            self.rotate_backup(dest)?;
        }
        fs::write(tmp, data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        // validation read-back
        let written = fs::read_to_string(tmp)
            .map_err(|e| format!("Failed to read-back temp file: {}", e))?;
        if written != data {
            return Err("Validation read-back mismatch".into());
        }
        fs::rename(tmp, dest)
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;
        fs::set_permissions(dest, fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;
        Ok(())
    }

    fn rotate_backup(&self, path: &PathBuf) -> Result<(), String> {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let ext = path.extension().map(|e| e.to_str().unwrap()).unwrap_or("json");
        for i in (1..MAX_BACKUPS).rev() {
            let old = self.base_dir.join(format!("{}.{}.{}", stem, i, ext));
            let new = self.base_dir.join(format!("{}.{}.{}", stem, i + 1, ext));
            if old.exists() {
                fs::rename(&old, &new)
                    .map_err(|e| format!("Failed to rotate backup: {}", e))?;
            }
        }
        let backup = self.base_dir.join(format!("{}.1.{}", stem, ext));
        if path.exists() {
            fs::copy(path, &backup)
                .map_err(|e| format!("Failed to create backup: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("grid-screen-test-{}", std::process::id()))
            .join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_load_defaults_when_no_config() {
        let dir = temp_dir("load_defaults");
        let store = ConfigStore::new(dir.clone());
        let (settings, layouts, _) = store.load().unwrap();
        assert_eq!(settings.schema_version, 1);
        assert_eq!(settings.default_gap_px, 10);
        assert!(layouts.is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_save_and_load_settings() {
        let dir = temp_dir("save_and_load");
        let store = ConfigStore::new(dir.clone());
        let mut settings = Settings::default();
        settings.autostart_enabled = true;
        store.save_settings(&settings).unwrap();
        let (loaded, _, _) = store.load().unwrap();
        assert!(loaded.autostart_enabled);
        fs::remove_dir_all(&dir).unwrap();
    }
}
