use std::{cell::RefCell, rc::Rc};

use crate::core::font_manager::string_data::CharData;

use self::text::Text;

use super::graphics::{Drawable, Graphics};

mod text;

pub struct GuiManager {
    window_size: [f32; 2],
    content_list: Vec<Box<dyn Drawable>>,
}

impl GuiManager {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        GuiManager {
            window_size: [window_width as f32, window_height as f32],
            content_list: Vec::new(),
        }
    }

    pub fn add_text(
        &mut self,
        content: String,
        face: &owned_ttf_parser::Face,
        graphics: &Graphics,
    ) {
        let text = Text::from_string(content, face, graphics, self.window_size);
        self.content_list.push(Box::new(text));
    }

    pub fn update_at(
        &mut self,
        index: usize,
        content: u64,
        face: &owned_ttf_parser::Face,
        graphics: &Graphics,
    ) {
        self.content_list[index] = Box::new(Text::from_string(
            format!("FPS: {}", content).to_string(),
            face,
            graphics,
            self.window_size,
        ));
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: Rc<RefCell<wgpu::RenderPass<'a>>>,
        graphics: &'a Graphics,
    ) {
        for thing in self.content_list.iter() {
            thing.draw(render_pass.clone(), graphics);
        }
    }
}
