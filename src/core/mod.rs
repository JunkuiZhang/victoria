use self::{graphics::Graphics, user_input::UserInput};

mod graphics;
mod user_input;

pub struct Controller {
    graphics: Graphics,
    input: UserInput,
}

impl Controller {
    pub fn new(window: &winit::window::Window) -> Self {
        let graphics = Graphics::new(window);
        let input = UserInput::new();

        Controller { graphics, input }
    }

    pub fn draw(&self) {
        self.graphics.render();
    }
}
