use std::path::Path;

use crate::settings::GameSettings;

use self::{
    font_manager::FontManager, game_time::GameTimeManager, graphics::Graphics,
    user_input::UserInput,
};

mod font_manager;
mod game_time;
mod graphics;
mod user_input;

pub struct Controller {
    graphics: Graphics,
    settings: Box<GameSettings>,
    input: UserInput,
    time_manager: GameTimeManager,
    font_manager: FontManager,
}

impl Controller {
    pub fn new(window: &winit::window::Window, game_settings: Box<GameSettings>) -> Self {
        // let font_path = Path::new("data").join("Inconsolata-Regular.ttf");
        let font_path = Path::new("data").join("eng1.ttf");
        let mut font_manager =
            FontManager::new(font_path.clone(), game_settings.get_font_texture_width());
        font_manager.read_font(font_path, game_settings.get_font_texture_width());
        font_manager.set_text();
        let graphics = Graphics::new(window, &game_settings, &font_manager);
        let input = UserInput::new();

        Controller {
            graphics,
            settings: game_settings,
            input,
            time_manager: GameTimeManager::new(),
            font_manager,
        }
    }

    pub fn update(&mut self) {
        self.time_manager.update();
    }

    pub fn draw(&self) {
        self.graphics.render();
    }

    pub fn preprocess(&mut self) {
        let font_path = Path::new("data").join("chi1.ttf");
        // self.font_manager
        //     .read_font(font_path, self.settings.get_font_texture_width());
        self.graphics.set_font();
    }

    pub fn exit(&self) {
        self.settings.save();
    }
}
