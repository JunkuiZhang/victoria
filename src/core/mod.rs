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
        let graphics = Graphics::new(window, &game_settings);
        let input = UserInput::new();
        let p = Path::new("data").join("Inconsolata-Regular.ttf");
        let font_manager = FontManager::new(p);

        Controller {
            graphics,
            settings: game_settings.clone(),
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

    pub fn exit(&self) {
        self.settings.save();
    }
}
