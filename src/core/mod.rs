use self::graphics::Graphics;

pub mod graphics;

pub struct Controller {
    graphics: Graphics,
}

impl Controller {
    pub fn new(window: &winit::window::Window) -> Self {
        let graphics = Graphics::new(window);

        Controller { graphics }
    }
}
