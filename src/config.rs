use directories::ProjectDirs;
use serde::Deserialize;
use std::{fs, path::PathBuf};

pub fn config_dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(path).join("tock"));
    }
    let home = std::env::var_os("HOME")?;
    let canonical = PathBuf::from(home).join(".config/tock");
    let legacy = ProjectDirs::from("", "", "tock").map(|p| p.config_dir().to_path_buf());
    if !canonical.exists()
        && let Some(ref path) = legacy
        && path.exists()
    {
        return legacy;
    }
    Some(canonical)
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub font: Option<String>,
    pub bg: Option<String>,
    pub fg: Option<String>,
    pub subtitle_fg: Option<String>,
    pub hidden_help: Option<bool>,
    pub panel_ratio: Option<u8>,
    pub panel_bg: Option<String>,
    pub panel_fg: Option<String>,
    pub panel_border: Option<String>,
    pub panel_border_sides: Option<String>,
    pub panel_border_style: Option<String>,
    pub alarm_sound: Option<String>,
    pub countdown_sound: Option<String>,
    pub color: Option<String>,
    pub silent: Option<bool>,
}

pub fn load_config() -> Config {
    if let Some(dir) = config_dir() {
        let config_file = dir.join("config.toml");
        if config_file.exists()
            && let Ok(contents) = fs::read_to_string(config_file)
            && let Ok(config) = toml::from_str::<Config>(&contents)
        {
            return config;
        }
    }
    Config::default()
}
