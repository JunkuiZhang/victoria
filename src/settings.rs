use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

const PLAYER_SETTING_FILE: &str = "data/player_setting.toml";
const ENGINE_SETTING_FILE: &str = "data/engine_setting.toml";

pub trait FromGameSettingsFile {
    fn from_file<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized;
}

impl<A> FromGameSettingsFile for A
where
    A: DeserializeOwned,
{
    fn from_file<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized,
    {
        toml::from_str(std::fs::read_to_string(path).unwrap().as_str()).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct GameSettings {
    engine_settings: GameEngineSettings,
    player_settings: GamePlayerSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameEngineSettings {
    window_title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct GamePlayerSettings {
    window_width: u32,
    window_height: u32,
}

impl GameSettings {
    pub fn new() -> Self {
        if Path::new(ENGINE_SETTING_FILE).exists() && Path::new(PLAYER_SETTING_FILE).exists() {
            GameSettings {
                engine_settings: GameEngineSettings::from_file(ENGINE_SETTING_FILE),
                player_settings: GamePlayerSettings::from_file(PLAYER_SETTING_FILE),
            }
        } else {
            GameSettings {
                engine_settings: GameEngineSettings::new(),
                player_settings: GamePlayerSettings::new(),
            }
        }
    }

    #[inline]
    pub fn get_window_width(&self) -> u32 {
        self.player_settings.window_width
    }

    #[inline]
    pub fn get_window_height(&self) -> u32 {
        self.player_settings.window_height
    }

    #[inline]
    pub fn get_window_title(&self) -> &str {
        self.engine_settings.window_title.as_str()
    }
}

impl GamePlayerSettings {
    pub fn new() -> Self {
        GamePlayerSettings {
            window_width: 800,
            window_height: 600,
        }
    }
}

impl GameEngineSettings {
    pub fn new() -> Self {
        GameEngineSettings {
            window_title: "My Game".into(),
        }
    }
}
