use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn auto() -> String {
    "auto".into()
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    #[serde(default = "auto")]
    pub language: String,
    pub menu_battery: bool,
    pub last_device: String,
    pub eq_bass: f32,
    pub eq_mid: f32,
    pub eq_treble: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { language: "auto".into(), menu_battery: true, last_device: String::new(), eq_bass: 0.0, eq_mid: 0.0, eq_treble: 0.0 }
    }
}

fn path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join("Library/Application Support/NothingX/settings.json"))
}

impl Settings {
    pub fn load() -> Self {
        path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(p) = path() {
            let _ = std::fs::create_dir_all(p.parent().unwrap());
            if let Ok(s) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(p, s);
            }
        }
    }
}
