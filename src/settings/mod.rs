mod window_setting;

use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::window_setting::{window_title, WindowSetting};

const PLAYER_SETTING_FILE: &str = "player_setting.toml";
const ENGINE_SETTING_FILE: &str = "engine_setting.toml";

pub trait FromGameSettingsFile {
    fn from_file<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized;
}

impl<A> FromGameSettingsFile for A
where
    A: DeserializeOwned + Default,
{
    fn from_file<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized,
    {
        toml::from_str(std::fs::read_to_string(path).unwrap().as_str()).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct GameSettings {
    // path_list: SettingPathList,
    has_changed: bool,
    engine_settings: GameEngineSettings,
    player_settings: GamePlayerSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameEngineSettings {
    #[serde(default = "window_title")]
    window_title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
struct GamePlayerSettings {
    #[serde(default)]
    window_setting: WindowSetting,
}

impl GameSettings {
    pub fn new() -> Self {
        let es_path = Path::new("data").join(ENGINE_SETTING_FILE);
        let ps_path = Path::new("data").join(PLAYER_SETTING_FILE);
        if es_path.exists() && ps_path.exists() {
            GameSettings {
                has_changed: false,
                engine_settings: GameEngineSettings::from_file(es_path),
                player_settings: GamePlayerSettings::from_file(ps_path),
            }
        } else {
            GameSettings {
                has_changed: true,
                engine_settings: GameEngineSettings::default(),
                player_settings: GamePlayerSettings::default(),
            }
        }
    }

    pub fn save(&self) {
        if !self.has_changed {
            return;
        }
        let save_path = Path::new("data");
        if !save_path.exists() {
            std::fs::create_dir_all(save_path).unwrap();
        }
        let es_path = save_path.join(ENGINE_SETTING_FILE);
        let ps_path = save_path.join(PLAYER_SETTING_FILE);
        std::fs::write(
            es_path,
            toml::to_string(&self.engine_settings)
                .expect("Unable to serialize game engine settings!"),
        )
        .unwrap();
        std::fs::write(
            ps_path,
            toml::to_string(&self.player_settings)
                .expect("Unable to serialize game player settings!"),
        )
        .unwrap();
    }

    #[inline]
    pub fn get_window_width(&self) -> u32 {
        self.player_settings.window_setting.0
    }

    #[inline]
    pub fn get_window_height(&self) -> u32 {
        self.player_settings.window_setting.1
    }

    #[inline]
    pub fn get_window_title(&self) -> String {
        self.engine_settings.window_title.clone()
    }
}

// https://serde.rs/attr-default.html
impl Default for GameEngineSettings {
    fn default() -> Self {
        Self {
            window_title: window_title(),
        }
    }
}
