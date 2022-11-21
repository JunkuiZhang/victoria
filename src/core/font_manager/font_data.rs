use owned_ttf_parser::Rect;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct FontData {
    pub curve_texel_index: u32,
    pub curve_info_index: u32,
    width_in_em: f32,
    height_in_em: f32,
}

impl FontData {
    pub fn new(
        glyph_start: usize,
        data_start: usize,
        glyph_rect: &Rect,
        units_per_em: f32,
    ) -> Self {
        let width = glyph_rect.width() as f32 / units_per_em;
        let height = glyph_rect.height() as f32 / units_per_em;
        FontData {
            curve_texel_index: glyph_start as u32,
            curve_info_index: data_start as u32,
            width_in_em: width,
            height_in_em: height,
        }
    }

    pub fn empty() -> Self {
        FontData {
            curve_texel_index: 0,
            curve_info_index: 0,
            width_in_em: -10.0,
            height_in_em: -10.0,
        }
    }
}
