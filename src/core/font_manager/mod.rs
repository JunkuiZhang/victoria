use self::font_outline::FontOutlineData;

mod font_outline;

pub struct FontManager {
    latter_a: FontOutlineData,
    latter_g: FontOutlineData,
}

impl FontManager {
    pub fn new<P: AsRef<std::path::Path>>(font_file: P) -> Self {
        let font_file = std::fs::read(font_file).unwrap();
        let font_face = ttf_parser::Face::parse(&font_file, 0).unwrap();
        let mut latter_a = FontOutlineData::new();
        let mut latter_g = FontOutlineData::new();
        font_face
            .outline_glyph(ttf_parser::GlyphId(4), &mut latter_a)
            .unwrap();
        font_face
            .outline_glyph(ttf_parser::GlyphId(309), &mut latter_g)
            .unwrap();

        FontManager { latter_a, latter_g }
    }
}
