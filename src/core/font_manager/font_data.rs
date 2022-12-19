use owned_ttf_parser::Rect;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
#[repr(C)]
pub struct FontData {
    pub curve_texel_index: u32,
    pub hband_index: u32,
    pub vband_index: u32,
    band_count: u32,
    width_in_em: f32,
    height_in_em: f32,
}

impl FontData {
    pub fn new(
        glyph_start: usize,
        hband_start: usize,
        vband_start: usize,
        band_count: u32,
        glyph_rect: &Rect,
        units_per_em: f32,
    ) -> Self {
        let width = glyph_rect.width() as f32 / units_per_em;
        let height = glyph_rect.height() as f32 / units_per_em;
        FontData {
            curve_texel_index: glyph_start as u32,
            hband_index: hband_start as u32,
            vband_index: vband_start as u32,
            band_count,
            width_in_em: width,
            height_in_em: height,
        }
    }

    pub fn empty() -> Self {
        FontData {
            curve_texel_index: 0,
            hband_index: 0,
            vband_index: 0,
            band_count: 0,
            width_in_em: -10.0,
            height_in_em: -10.0,
        }
    }
}
