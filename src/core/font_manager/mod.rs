use owned_ttf_parser::{AsFaceRef, OwnedFace};

use crate::utils::max_3number;

use self::{font_data::FontData, font_outline::FontOutlineData, string_data::CharData};

mod font_data;
mod font_manager_builder;
mod font_outline;
mod string_data;

pub struct FontManager {
    font_face: OwnedFace,
    // font info in fontface, index using u16 glyph id
    font_data: Vec<FontData>,
    // rgba f32
    font_curves: Vec<[f32; 4]>,
    // rgba u32
    font_curve_ordering_list: Vec<u32>,
    string_vec: Vec<CharData>,
}

impl FontManager {
    pub fn new<P: AsRef<std::path::Path>>(font_path: P) -> Self {
        let font_file = std::fs::read(font_path).expect("Unable to read font!");
        let font_face = owned_ttf_parser::OwnedFace::from_vec(font_file, 0).unwrap();
        FontManager {
            font_face,
            font_data: Vec::new(),
            font_curves: Vec::new(),
            font_curve_ordering_list: Vec::new(),
            string_vec: Vec::new(),
        }
    }

    pub fn preprocess_font(&mut self) {
        let font_face = self.font_face.as_face_ref();
        let units_per_em = font_face.units_per_em() as f32;
        self.font_data.clear();
        self.font_curves.clear();
        self.font_curve_ordering_list.clear();
        let mut curves_index = 0;
        let mut ordering_index = 0;

        for glyph_id in 0..font_face.number_of_glyphs() {
            let mut this_char = FontOutlineData::new();

            let Some(bounding_box) = font_face
                .outline_glyph(owned_ttf_parser::GlyphId(glyph_id), &mut this_char)
                else {
                    self.font_data.push(FontData::empty());
                    println!("Skiped glyph: {}", glyph_id);
                    continue;
                };

            // processing
            let mut last_x = 0.0;
            let mut last_y = 0.0;
            let origin_x = bounding_box.x_min as f32 / units_per_em; // -0.1 for padding
            let origin_y = bounding_box.y_min as f32 / units_per_em;
            let this_char_curve_start = curves_index;
            self.font_data.push(FontData::new(
                this_char_curve_start,
                ordering_index,
                &bounding_box,
                units_per_em,
            ));
            let mut curve_info_data = Vec::new();
            for command in this_char.point_command_iter() {
                match *command {
                    font_outline::OutlineDrawCommand::MoveTo(a, b) => {
                        last_x = a / units_per_em - origin_x;
                        last_y = b / units_per_em - origin_y;
                        self.font_curves.push([-1.0, -1.0, last_x, last_y]);
                        curves_index += 1;
                    }
                    font_outline::OutlineDrawCommand::LineTo(a, b) => {
                        let x2 = a / units_per_em - origin_x;
                        let y2 = b / units_per_em - origin_y;
                        let x1 = (x2 + last_x) / 2.0;
                        let y1 = (y2 + last_y) / 2.0;
                        self.font_curves.push([x1, y1, x2, y2]);
                        // let minx = min_3number(last_x, x1, x2);
                        let maxx = max_3number(last_x, x1, x2);
                        let this_char_glyph_offset = curves_index - this_char_curve_start;
                        curve_info_data.push((this_char_glyph_offset, maxx));
                        last_x = x2;
                        last_y = y2;
                        curves_index += 1;
                    }
                    font_outline::OutlineDrawCommand::QuadTo(a1, b1, a, b) => {
                        let x1 = a1 / units_per_em - origin_x;
                        let y1 = b1 / units_per_em - origin_y;
                        let x2 = a / units_per_em - origin_x;
                        let y2 = b / units_per_em - origin_y;
                        self.font_curves.push([x1, y1, x2, y2]);
                        let maxx = max_3number(last_x, x1, x2);
                        let this_char_glyph_offset = curves_index - this_char_curve_start;
                        curve_info_data.push((this_char_glyph_offset, maxx));
                        last_x = x2;
                        last_y = y2;
                        curves_index += 1;
                    }
                    font_outline::OutlineDrawCommand::CurveTo(_, _, _, _, _, _) => unreachable!(),
                    font_outline::OutlineDrawCommand::Close => {}
                }
            }
            curve_info_data
                .sort_by(|(_, max_num0), (_, max_num2)| max_num2.partial_cmp(max_num0).unwrap());

            self.font_curve_ordering_list
                .push(curve_info_data.len() as u32);
            for (offset, _) in curve_info_data.iter() {
                // let row_num = (*index / 4096) << 16;
                // let col_num = *index % 4096;
                // self.font_curve_ordering_list
                //     .push((row_num | col_num) as u32);
                self.font_curve_ordering_list.push(*offset as u32);
            }
            ordering_index += curve_info_data.len() + 1;
        }
    }

    pub fn set_text(&mut self) {
        let face = self.font_face.as_face_ref();
        let mut last_width = 0.0;
        let mut x_drift = -0.8;
        for this_char in "Hi.!".chars().enumerate() {
            let glyph_index = face.glyph_index(this_char.1).unwrap();
            println!("Draw {} with id: {}", this_char.1, glyph_index.0);
            let info = face.glyph_bounding_box(glyph_index).unwrap();
            x_drift += last_width / 768.0 * 2.0 * 1.05;
            self.string_vec
                .push(CharData::new(glyph_index.0 as u32, 200.0, [x_drift, -0.3]));
            last_width = info.width() as f32 / 1000.0 * 200.0;
            println!("Last width: {}", last_width / 768.0 * 2.0);
        }
    }

    pub fn get_font_data(&self) -> (usize, usize, &Vec<FontData>) {
        (
            std::mem::size_of::<FontData>(),
            self.font_data.len(),
            &self.font_data,
        )
    }

    pub fn get_font_curves(&self) -> (usize, &Vec<[f32; 4]>) {
        (self.font_curves.len(), &self.font_curves)
    }

    pub fn get_font_curve_ordering_list(&self) -> (usize, &Vec<u32>) {
        (
            self.font_curve_ordering_list.len(),
            &self.font_curve_ordering_list,
        )
    }

    pub fn get_string_vec(&self) -> (u64, usize, &Vec<CharData>) {
        (
            std::mem::size_of::<CharData>() as u64,
            self.string_vec.len(),
            &self.string_vec,
        )
    }
}
