use std::path::Path;

use owned_ttf_parser::GlyphId;

fn main() {
    const LATTER_A: GlyphId = GlyphId(5);

    let font_file = Path::new("data").join("eng1.ttf");
    let font_data = std::fs::read(font_file.clone()).expect("Unable to open file.");
    let font_face = owned_ttf_parser::Face::parse(&font_data, 0).expect("Unable to parse font.");
    let rect = font_face.glyph_bounding_box(LATTER_A).unwrap();
    let mut glyph_builder = ExmapleBuilder {
        data: String::new(),
        curve_count: 0,
    };
    let _ = font_face
        .outline_glyph(LATTER_A, &mut glyph_builder)
        .unwrap();
    println!("A: {}", glyph_builder.data);
    println!("A total curve count: {}", glyph_builder.curve_count);
    println!("Filename: {:?}", font_file.file_name());
    let x: Vec<&str> = font_file.file_name().unwrap().to_str().unwrap().rsplit(".").collect();
    println!("Filename: {:?}", x);
}

struct ExmapleBuilder {
    data: String,
    curve_count: u32,
}

impl owned_ttf_parser::OutlineBuilder for ExmapleBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.data, "M {} {} ", x, y).unwrap()
    }

    fn line_to(&mut self, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.data, "L {} {} ", x, y).unwrap();
        self.curve_count += 1;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        use std::fmt::Write;
        write!(self.data, "Q {} {} {} {} ", x1, y1, x, y).unwrap();
        self.curve_count += 1;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        unreachable!()
    }

    fn close(&mut self) {
        self.data.push_str("Z ")
    }
}
