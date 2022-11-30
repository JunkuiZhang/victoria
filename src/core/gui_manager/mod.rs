use std::{cell::RefCell, rc::Rc};

use crate::core::font_manager::string_data::CharData;

use self::text::Text;

use super::{
    font_manager::FontManager,
    graphics::{Drawable, Graphics},
};

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
        font_manager: Rc<FontManager>,
        graphics: &Graphics,
    ) {
        let text = Text::from_string(content, font_manager, graphics);
        self.content_list.push(Box::new(text));
    }

    pub fn update_at(&mut self, index: usize, content: Vec<u8>, graphics: &mut Graphics) {
        // self.content_list[index]
        self.content_list[index].update_self(content, graphics);
    }

    pub fn draw(&self, graphics: &mut Graphics) {
        for thing in self.content_list.iter() {
            // thing.draw(render_pass.clone(), graphics);
            thing.draw_queue(graphics);
        }
    }
}
