use std::{cell::RefCell, rc::Rc};

use wgpu::util::DeviceExt;

use crate::core::{
    font_manager::string_data::CharData,
    graphics::{Drawable, Graphics},
};

pub struct Text {
    string_vec: Vec<CharData>,
    string_vec_buffer: wgpu::Buffer,
}

impl Text {
    pub fn new(string_vec: Vec<CharData>, graphics: &Graphics) -> Self {
        let string_vec_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("String Vec Buffer"),
                    contents: bytemuck::cast_slice(&string_vec),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                });
        Text {
            string_vec,
            string_vec_buffer,
        }
    }

    fn get_string_vec(&self) -> (u64, usize, &Vec<CharData>) {
        (
            std::mem::size_of::<CharData>() as u64,
            self.string_vec.len(),
            &self.string_vec,
        )
    }
}

impl Drawable for Text {
    fn draw<'a>(&'a self, render_pass: Rc<RefCell<wgpu::RenderPass<'a>>>, graphics: &'a Graphics) {
        render_pass
            .borrow_mut()
            .set_pipeline(&graphics.font_graphics.render_pipeline);
        render_pass
            .borrow_mut()
            .set_vertex_buffer(0, graphics.font_graphics.vertex_buffer.slice(..));
        render_pass
            .borrow_mut()
            .set_vertex_buffer(1, self.string_vec_buffer.slice(..));
        render_pass.borrow_mut().set_index_buffer(
            graphics.font_graphics.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass
            .borrow_mut()
            .set_bind_group(0, &graphics.font_graphics.uniform_bindgroup, &[]);
        render_pass.borrow_mut().set_bind_group(
            1,
            &graphics.font_graphics.font_data_bindgroup,
            &[],
        );
        render_pass
            .borrow_mut()
            .draw_indexed(0..6, 0, 0..self.string_vec.len() as u32);
    }
}
