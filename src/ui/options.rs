use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    pub fov: f32,
    pub sensitivity: f32,
    pub view_distance: u32,
    pub vsync: bool,
    pub gui_scale: GuiScale,
}

#[derive(Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum GuiScale {
    #[default]
    Auto,
    Fixed(u32),
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fov: 70.0,
            sensitivity: 100.0,
            view_distance: 8,
            vsync: true,
            gui_scale: GuiScale::Auto,
        }
    }
}

impl GameSettings {
    pub fn fov_radians(&self) -> f32 {
        self.fov.to_radians()
    }

    pub fn raw_sensitivity(&self) -> f32 {
        self.sensitivity * 0.00003 + 0.001
    }
}

pub struct SettingsFile {
    pub settings: GameSettings,
    path: PathBuf,
}

impl SettingsFile {
    pub fn load(game_dir: &Path) -> Self {
        let path = game_dir.join("ferrite_options.json");
        let settings = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { settings, path }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.settings) {
            if let Err(e) = std::fs::write(&self.path, json) {
                log::warn!("Failed to save settings: {e}");
            }
        }
    }
}
