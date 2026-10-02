use crate::config::config_dir;
use crate::models::AppData;
use std::fs;

fn get_data_path() -> Option<std::path::PathBuf> {
    config_dir().map(|dir| dir.join("data.json"))
}

pub fn load_data() -> AppData {
    if let Some(data_path) = get_data_path()
        && data_path.exists()
        && let Ok(contents) = fs::read_to_string(data_path)
        && let Ok(data) = serde_json::from_str::<AppData>(&contents)
    {
        return data;
    }
    AppData::default()
}

pub fn save_data(data: &AppData) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(data_path) = get_data_path() {
        if let Some(parent) = data_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(data)?;
        fs::write(data_path, json)?;
    }
    Ok(())
}
