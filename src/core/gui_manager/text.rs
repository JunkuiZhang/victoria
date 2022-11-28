use crate::core::{font_manager::string_data::CharData, graphics::Drawable};

pub struct Text {
    pub string_vec: Vec<CharData>,
}

impl Text {
    pub fn new() -> Self {
        Text {
            string_vec: Vec::new(),
        }
    }
}

impl Drawable for Text {
    fn draw(&self) {
        todo!()
    }
}
