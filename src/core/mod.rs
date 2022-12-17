use std::{path::Path, rc::Rc};

use crate::settings::GameSettings;

use self::{
    font_manager::FontManager, game_time::GameTimeManager, graphics::Graphics,
    gui_manager::GuiManager, resources::ResourceManager, user_input::UserInput,
};

mod font_manager;
mod game_time;
mod graphics;
mod gui_manager;
mod resources;
mod user_input;

pub struct Controller {
    graphics: Graphics,
    settings: Rc<GameSettings>,
    input: UserInput,
    time_manager: GameTimeManager,
    font_manager: Rc<FontManager>,
    gui_manager: GuiManager,
    resource_manager: ResourceManager,
}

impl Controller {
    pub fn new(window: &winit::window::Window, game_settings: Rc<GameSettings>) -> Self {
        let font_path = Path::new("data").join("chi1.ttf");
        let font_manager = Rc::new(FontManager::new(
            font_path,
            game_settings.get_window_width(),
            game_settings.get_window_height(),
        ));
        let gui_manager = GuiManager::new(
            game_settings.get_window_width(),
            game_settings.get_window_height(),
        );
        let input = UserInput::new();
        let graphics = Graphics::new(window, &game_settings);
        let resource_manager = ResourceManager::new(&font_manager, &graphics.context);

        Controller {
            graphics,
            settings: game_settings,
            input,
            time_manager: GameTimeManager::new(),
            font_manager,
            gui_manager,
            resource_manager,
        }
    }

    pub fn update(&mut self) {
        self.time_manager
            .update(&mut self.gui_manager, &mut self.graphics.update_queue);
    }

    pub fn draw(&mut self) {
        self.gui_manager
            .draw_queue(&self.resource_manager, &mut self.graphics.draw_queue);
        self.graphics.draw();
    }

    pub fn preprocess(&mut self) {
        self.gui_manager.add_text(
            "你好！123。？".to_string(),
            200.0,
            self.font_manager.clone(),
            &self.graphics.context,
        )
        // self.gui_manager
        //     .set_render_pipeline(&mut self.graphics, &self.font_manager);
    }

    pub fn exit(&self) {
        self.settings.save();
    }
}
