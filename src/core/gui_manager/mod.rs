use std::rc::Rc;

use self::text::Text;

use super::{
    font_manager::FontManager,
    graphics::{DrawCall, Drawable, GpuContext, UpdateInfo},
    resources::ResourceManager,
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
        font_size: f32,
        font_manager: Rc<FontManager>,
        gpu_context: &GpuContext,
    ) {
        let text = Text::from_string(content, font_size, font_manager, gpu_context);
        self.content_list.push(Box::new(text));
    }

    pub fn update_at(
        &mut self,
        index: usize,
        content: Vec<u8>,
        update_queue: &mut Vec<UpdateInfo>,
        context: &GpuContext,
    ) {
        // self.content_list[index]
        self.content_list[index].update_queue(content, update_queue, context);
    }

    pub fn draw_queue(&self, resource_manager: &ResourceManager, draw_queue: &mut Vec<DrawCall>) {
        self.content_list.iter().for_each(|thing| {
            draw_queue.push(thing.get_draw_info(resource_manager));
        });
    }
}
