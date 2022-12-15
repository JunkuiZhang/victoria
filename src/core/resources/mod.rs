use super::{
    font_manager::{font_graphics::FontGraphics, FontManager},
    graphics::GpuContext,
};

pub struct ResourceManager {
    pub font: FontGraphics,
}

impl ResourceManager {
    pub fn new(font_manager: &FontManager, gpu_context: &GpuContext) -> Self {
        let font = font_manager.prepare(gpu_context);

        ResourceManager { font }
    }
}
