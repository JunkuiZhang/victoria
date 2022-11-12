#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CharData {
    glyph_id: u32,
    coordinate: [f32; 2], // base line position
    pixels_per_em: f32,
}

impl CharData {
    pub fn new(glyph_id: u32, pixels_per_em: f32, pos: [f32; 2]) -> Self {
        CharData {
            coordinate: pos,
            glyph_id,
            pixels_per_em,
        }
    }
}
