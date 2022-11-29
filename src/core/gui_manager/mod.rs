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
        let mut last_width = 0.0;
        let mut x_drift = -0.8;
        let mut string_vec = Vec::new();
        for this_char in content.chars() {
            let glyph_index = face.glyph_index(this_char).unwrap();
            let info = face.glyph_bounding_box(glyph_index).unwrap();
            x_drift += last_width / self.window_size[0] * 2.0 * 1.05;
            let y_drift =
                info.y_min as f32 / face.units_per_em() as f32 * 200.0 / self.window_size[1] * 2.0;
            println!("Draw {} with id {}", this_char, glyph_index.0);
            string_vec.push(CharData::new(
                glyph_index.0 as u32,
                200.0,
                [x_drift, -0.3 + y_drift],
            ));
            last_width = info.width() as f32 / face.units_per_em() as f32 * 200.0;
        }

        let text = Text::new(string_vec, graphics);
        self.content_list.push(Box::new(text));
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
