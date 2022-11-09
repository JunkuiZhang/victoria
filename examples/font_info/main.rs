use std::path::Path;

use ttf_parser::GlyphId;

fn main() {
    const LATTER_GBAR: GlyphId = GlyphId(309);

    let font_file = Path::new("data").join("Inconsolata-Regular.ttf");
    let font_data = std::fs::read(font_file).expect("Unable to open file.");
    let face = ttf_parser::Face::parse(&font_data, 0).expect("Unable to parse font.");
    let family_name = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::FULL_NAME && name.is_unicode())
        .and_then(|name| name.to_string());

    let post_script_name = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && name.is_unicode())
        .and_then(|name| name.to_string());

    println!("Family name: {:?}", family_name);
    println!("PostScript name: {:?}", post_script_name);
    println!("Units per EM: {:?}", face.units_per_em());
    println!("Ascender: {}", face.ascender());
    println!("Descender: {}", face.descender());
    println!("Line gap: {}", face.line_gap());
    println!("Global bbox: {:?}", face.global_bounding_box());
    println!("Number of glyphs: {}", face.number_of_glyphs());
    println!("Underline: {:?}", face.underline_metrics());
    println!("X height: {:?}", face.x_height());
    println!("Weight: {:?}", face.weight());
    println!("Width: {:?}", face.width());
    println!("Regular: {}", face.is_regular());
    println!("Italic: {}", face.is_italic());
    println!("Bold: {}", face.is_bold());
    println!("Oblique: {}", face.is_oblique());
    println!("Strikeout: {:?}", face.strikeout_metrics());
    println!("Subscript: {:?}", face.subscript_metrics());
    println!("Superscript: {:?}", face.superscript_metrics());
    println!("Permissions: {:?}", face.permissions());
    println!("Variable: {:?}", face.is_variable());
}
