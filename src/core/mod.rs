use self::{graphics::Graphics, user_input::UserInput, game_time::GameTimeManager};

mod game_time;
mod graphics;
mod user_input;

pub struct Controller {
    graphics: Graphics,
    input: UserInput,
    time_manager: GameTimeManager,
}

impl Controller {
    pub fn new(window: &winit::window::Window) -> Self {
        let graphics = Graphics::new(window);
        let input = UserInput::new();

        Controller {
            graphics,
            input,
            time_manager: GameTimeManager::new(),
        }
    }

    pub fn update(&mut self) {
        self.time_manager.update();
    }

    pub fn draw(&self) {
        self.graphics.render();
    }
}
