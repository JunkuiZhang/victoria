use std::{rc::Rc, vec};

use wgpu::util::DeviceExt;

use crate::core::{
    font_manager::{string_data::CharData, FontManager},
    graphics::{DrawCall, DrawIndexedInfo, Drawable, Graphics, UpdateInfo},
};

pub struct Text {
    font_manager: Rc<FontManager>,
    string_vec_buffer: Rc<wgpu::Buffer>,
    raw_content: Rc<Vec<u8>>,
}

impl Text {
    pub fn from_string(s: String, font_manager: Rc<FontManager>, graphics: &Graphics) -> Self {
        let string_vec =
            Self::get_string_vec(s, font_manager.get_face(), font_manager.get_window_size());
        let string_vec_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("String Vec Buffer"),
                    contents: bytemuck::cast_slice(&string_vec),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                });
        let raw_content = Rc::new(bytemuck::cast_slice(&string_vec).to_vec());

        Text {
            font_manager,
            string_vec_buffer: Rc::new(string_vec_buffer),
            raw_content,
        }
    }

    pub fn update_string(&mut self, s: String) {
        let string_vec = Self::get_string_vec(
            s,
            self.font_manager.get_face(),
            self.font_manager.get_window_size(),
        );
        self.raw_content = Rc::new(bytemuck::cast_slice(&string_vec).to_vec());
    }

    fn get_string_vec(
        s: String,
        face: &owned_ttf_parser::Face,
        window_size: [f32; 2],
    ) -> Vec<CharData> {
        let mut last_width = 0.0;
        let mut x_drift = -0.8;
        let mut string_vec = Vec::new();
        for this_char in s.chars() {
            let glyph_index = face.glyph_index(this_char).unwrap();
            let info = face.glyph_bounding_box(glyph_index).unwrap_or_else(|| {
                face.glyph_bounding_box(owned_ttf_parser::GlyphId(299))
                    .unwrap()
            });
            x_drift += last_width / window_size[0] * 2.0 * 1.05;
            let y_drift =
                info.y_min as f32 / face.units_per_em() as f32 * 200.0 / window_size[1] * 2.0;
            string_vec.push(CharData::new(
                glyph_index.0 as u32,
                200.0,
                [x_drift, -0.3 + y_drift],
            ));
            last_width = info.width() as f32 / face.units_per_em() as f32 * 200.0;
        }

        string_vec
    }
}

impl Drawable for Text {
    fn update_self(&mut self, content: Vec<u8>, graphics: &mut Graphics) {
        let s = String::from_utf8(content).unwrap();
        self.update_string(s);
        self.update_queue(graphics);
    }

    fn update_queue(&self, graphics: &mut Graphics) {
        let res = UpdateInfo {
            target_buffer: self.string_vec_buffer.clone(),
            size: wgpu::BufferSize::new(self.raw_content.len() as _).unwrap(),
            content: self.raw_content.clone(),
        };
        graphics.update_queue.push(res);
    }

    fn draw_queue(&self, graphics: &mut Graphics) {
        let res = DrawCall::DrawIndexed(DrawIndexedInfo {
            pipeline: graphics.font_graphics.render_pipeline.clone(),
            vertex_buffer: vec![
                graphics.font_graphics.vertex_buffer.clone(),
                self.string_vec_buffer.clone(),
            ],
            index_buffer: graphics.font_graphics.index_buffer.clone(),
            bindgroup: vec![
                graphics.font_graphics.uniform_bindgroup.clone(),
                graphics.font_graphics.font_data_bindgroup.clone(),
            ],
            indices: 6,
            instance: (self.raw_content.len() / std::mem::size_of::<CharData>()) as u32,
        });
        graphics.draw_queue.push(res);
    }
}
