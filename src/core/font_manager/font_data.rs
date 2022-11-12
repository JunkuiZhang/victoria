use ttf_parser::Rect;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct FontData {
    curve_texel_index: u32,
    curve_info_index: u32,
    width: f32,
    height: f32,
}

impl FontData {
    pub fn new(glyph_start: usize, data_start: usize, glyph_rect: &Rect) -> Self {
        FontData {
            curve_texel_index: glyph_start as u32,
            curve_info_index: data_start as u32,
            width: glyph_rect.width() as f32,
            height: glyph_rect.height() as f32,
        }
    }

    pub fn empty() -> Self {
        FontData {
            curve_texel_index: 0,
            curve_info_index: 0,
            width: -1.0,
            height: -1.0,
        }
    }
}
