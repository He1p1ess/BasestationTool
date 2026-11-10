use crate::i18n::Language;
use crate::{device::KnownDevice, protocol::PowerCommand};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub devices: Vec<KnownDevice>,
    #[serde(alias = "german", deserialize_with = "load_language")]
    pub language: Language,
    pub theme: u8,
    pub steam_autostart: bool,
    pub close_with_steam: bool,
    pub steam_on: bool,
    pub steam_exit: Option<PowerCommand>,
    pub openvr_library: String,
    pub auto_log: bool,
    pub debug: bool,
    pub auto_scan: bool,
    pub show_logs: bool,
}
fn load_language<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Language, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Saved {
        Current(Language),
        Legacy(bool),
    }
    Ok(match Saved::deserialize(deserializer)? {
        Saved::Current(lang) => lang,
        Saved::Legacy(true) => Language::De,
        Saved::Legacy(false) => Language::En,
    })
}
impl Default for Config {
    fn default() -> Self {
        Self {
            devices: vec![],
            language: Language::De,
            theme: 0,
            steam_autostart: false,
            close_with_steam: false,
            steam_on: false,
            steam_exit: None,
            openvr_library: String::new(),
            auto_log: true,
            debug: false,
            auto_scan: true,
            show_logs: true,
        }
    }
}
pub fn directory() -> PathBuf {
    if let Some(p) = std::env::var_os("BASESTATION_DATA_DIR") {
        return p.into();
    }
    directories::ProjectDirs::from("org", "BasestationTool", "BasestationTool")
        .expect("user directory")
        .config_dir()
        .to_owned()
}
impl Config {
    pub fn load() -> Result<Self> {
        let path = directory().join("settings.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        serde_json::from_slice(&std::fs::read(path)?)
            .context("Invalid settings.json; original retained")
    }
    pub fn save(&self) -> Result<()> {
        let dir = directory();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("settings.json");
        let tmp = dir.join("settings.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        if path.exists() {
            std::fs::copy(&path, dir.join("settings.previous.json"))?;
        }
        std::fs::rename(tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_language_migrates_and_round_trips() {
        for (flag, expected) in [(true, Language::De), (false, Language::En)] {
            assert_eq!(
                serde_json::from_value::<Config>(serde_json::json!({"german":flag}))
                    .unwrap()
                    .language,
                expected
            );
        }
        for language in Language::ALL {
            let cfg = Config {
                language,
                ..Default::default()
            };
            assert_eq!(
                serde_json::from_value::<Config>(serde_json::to_value(cfg).unwrap())
                    .unwrap()
                    .language,
                language
            );
        }
    }
    #[test]
    fn automatic_power_is_opt_in() {
        let default = Config::default();
        assert!(!default.steam_on);
        assert_eq!(default.steam_exit, None);
        assert!(!default.steam_autostart);
        let migrated: Config = serde_json::from_str("{}").unwrap();
        assert!(!migrated.steam_on);
        assert_eq!(migrated.steam_exit, None);
    }
    #[test]
    fn local_names_and_ids_round_trip() {
        let original = Config {
            devices: vec![KnownDevice {
                id: "windows:address".into(),
                name: "LHB-TEST".into(),
                alias: "Tür".into(),
                address: "ADDRESS".into(),
            }],
            language: Language::En,
            theme: 2,
            ..Default::default()
        };
        let restored: Config =
            serde_json::from_slice(&serde_json::to_vec(&original).unwrap()).unwrap();
        assert_eq!(restored.devices[0].id, "windows:address");
        assert_eq!(restored.devices[0].alias, "Tür");
        assert_eq!(restored.devices[0].name, "LHB-TEST");
        assert_eq!(restored.language, Language::En);
        assert_eq!(restored.theme, 2);
        assert!(serde_json::from_str::<Config>("invalid").is_err());
    }
}
