pub struct GameSettings {
    engine_settings: GameEngineSettings,
    player_settings: GamePlayerSettings,
}

struct GameEngineSettings {
    window_title: &'static str,
}

struct GamePlayerSettings {
    window_width: u32,
    window_height: u32,
}

impl GameSettings {
    pub fn new() -> Self {
        GameSettings {
            engine_settings: GameEngineSettings {
                window_title: "My Game",
            },
            player_settings: GamePlayerSettings {
                window_width: 800,
                window_height: 600,
            },
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
    pub fn get_window_title(&self) -> &'static str {
        self.engine_settings.window_title
    }
}
