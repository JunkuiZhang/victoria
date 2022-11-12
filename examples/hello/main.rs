use std::path::Path;

use ttf_parser::GlyphId;

fn main() {
    const LATTER_A: GlyphId = GlyphId(4);
    const LATTER_GBAR: GlyphId = GlyphId(309);

    let font_file = Path::new("data").join("chi1.ttf");
    let font_data = std::fs::read(font_file).expect("Unable to open file.");
    let font_face = ttf_parser::Face::parse(&font_data, 0).expect("Unable to parse font.");
    let mut glyph_builder = ExmapleBuilder(String::new());
    let _ = font_face
        .outline_glyph(LATTER_A, &mut glyph_builder)
        .unwrap();
    println!("A: {}", glyph_builder.0);
    let g = font_face
        .outline_glyph(LATTER_GBAR, &mut glyph_builder)
        .unwrap();
    println!("A: {}", u32::from('A'));
    println!("======================================================");
    println!("{:?}", g);
    println!("width: {}, height: {}", g.width(), g.height());
    println!("Q: {:?}", font_face.glyph_index('我').unwrap()); // 6215
}

struct ExmapleBuilder(String);

impl ttf_parser::OutlineBuilder for ExmapleBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "M {} {} ", x, y).unwrap()
    }

    fn line_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "L {} {} ", x, y).unwrap()
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap()
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap()
    }

    fn close(&mut self) {
        self.0.push_str("Z ")
    }
}
