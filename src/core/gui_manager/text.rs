use std::{rc::Rc, vec};

use wgpu::util::DeviceExt;

use crate::core::{
    font_manager::{string_data::CharData, FontManager},
    graphics::{DrawCall, DrawIndexedInfo, Drawable, GpuContext, Graphics, UpdateInfo},
    resources::ResourceManager,
};

pub struct Text {
    font_size: f32,
    font_manager: Rc<FontManager>,
    string_vec_buffer: Rc<wgpu::Buffer>,
    raw_content: Rc<Vec<u8>>,
}

impl Text {
    pub fn from_string(
        s: String,
        font_size: f32,
        font_manager: Rc<FontManager>,
        gpu_context: &GpuContext,
    ) -> Self {
        let string_vec = Self::get_string_vec(
            s,
            font_size,
            font_manager.get_face(),
            font_manager.get_window_size(),
        );
        let string_vec_buffer =
            gpu_context
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
            font_size,
            font_manager,
            string_vec_buffer: Rc::new(string_vec_buffer),
            raw_content,
        }
    }

    pub fn update_string(&mut self, s: String) {
        let string_vec = Self::get_string_vec(
            s,
            self.font_size,
            self.font_manager.get_face(),
            self.font_manager.get_window_size(),
        );
        self.raw_content = Rc::new(bytemuck::cast_slice(&string_vec).to_vec());
    }

    fn get_string_vec(
        s: String,
        font_size: f32,
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
                info.y_min as f32 / face.units_per_em() as f32 * font_size / window_size[1] * 2.0;
            string_vec.push(CharData::new(
                glyph_index.0 as u32,
                font_size,
                [x_drift, -0.3 + y_drift],
            ));
            last_width = info.width() as f32 / face.units_per_em() as f32 * font_size;
        }

        string_vec
    }
}

impl Drawable for Text {
    fn update_queue(&mut self, content: Vec<u8>, update_queue: &mut Vec<UpdateInfo>) {
        let s = String::from_utf8(content).unwrap();
        self.update_string(s);
        update_queue.push(self.get_update_info());
    }

    fn get_update_info(&self) -> UpdateInfo {
        UpdateInfo {
            target_buffer: self.string_vec_buffer.clone(),
            size: wgpu::BufferSize::new(self.raw_content.len() as _).unwrap(),
            content: self.raw_content.clone(),
        }
    }

    fn get_draw_info(&self, resource_manager: &ResourceManager) -> DrawCall {
        DrawCall::DrawIndexed(DrawIndexedInfo {
            pipeline: resource_manager.font.render_pipeline.clone(),
            vertex_buffer: vec![
                resource_manager.font.vertex_buffer.clone(),
                self.string_vec_buffer.clone(),
            ],
            index_buffer: resource_manager.font.index_buffer.clone(),
            bindgroup: vec![
                resource_manager.font.uniform_bindgroup.clone(),
                resource_manager.font.font_data_bindgroup.clone(),
            ],
            indices: 6,
            instance: (self.raw_content.len() / std::mem::size_of::<CharData>()) as u32,
        })
    }
}
