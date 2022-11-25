use std::path::Path;

use crate::settings::GameSettings;

use self::{
    font_manager::FontManager, game_time::GameTimeManager, graphics::Graphics,
    gui_manager::GuiManager, user_input::UserInput,
};

mod font_manager;
mod game_time;
mod graphics;
mod gui_manager;
mod user_input;

pub struct Controller {
    graphics: Graphics,
    settings: Box<GameSettings>,
    input: UserInput,
    time_manager: GameTimeManager,
    font_manager: FontManager,
    gui_manager: GuiManager,
}

impl Controller {
    pub fn new(window: &winit::window::Window, game_settings: Box<GameSettings>) -> Self {
        let font_path = Path::new("data").join("chi1.ttf");
        let mut font_manager = FontManager::new(
            font_path,
            game_settings.get_window_width(),
            game_settings.get_window_height(),
        );
        font_manager.preprocess_font();
        font_manager.set_text();
        let gui_manager = GuiManager::new(
            game_settings.get_window_width(),
            game_settings.get_window_height(),
        );
        let graphics = Graphics::new(window, &game_settings, &font_manager);
        let input = UserInput::new();

        Controller {
            graphics,
            settings: game_settings,
            input,
            time_manager: GameTimeManager::new(),
            font_manager,
            gui_manager,
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
