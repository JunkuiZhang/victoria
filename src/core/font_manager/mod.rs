use ttf_parser::Rect;

use crate::utils::{max_3number, min_3number};

use self::{font_data::FontData, font_outline::FontOutlineData, string_data::CharData};

mod font_data;
mod font_manager_builder;
mod font_outline;
mod string_data;

pub struct FontManager {
    this_char: FontOutlineData,
    char_rect: Rect,
    units_per_em: f32,
    // font info in fontface, index using u16 glyph id
    font_data: Vec<FontData>,
    // rgba f32
    font_curves: Vec<[f32; 4]>,
    // rgba u32
    font_curve_ordering_list: Vec<u32>,
    string_vec: Vec<CharData>,
}

impl FontManager {
    pub fn new<P: AsRef<std::path::Path>>(font_file: P, font_curves_width: u32) -> Self {
        let font_file = std::fs::read(font_file).unwrap();
        let font_face = ttf_parser::Face::parse(&font_file, 0).unwrap();
        let mut this_char = FontOutlineData::new();
        let char_rect = font_face
            .outline_glyph(font_face.glyph_index('A').unwrap(), &mut this_char)
            .unwrap();
        this_char.finish();
        let units_per_em = font_face.units_per_em() as f32;

        FontManager {
            this_char,
            char_rect,
            units_per_em,
            font_data: Vec::new(),
            font_curves: Vec::new(),
            font_curve_ordering_list: Vec::new(),
            string_vec: Vec::new(),
        }
    }

    pub fn read_font<P: AsRef<std::path::Path>>(&mut self, font_path: P, font_texture_width: u32) {
        let font_file = std::fs::read(font_path).expect("Unable to read font!");
        let font_face = ttf_parser::Face::parse(&font_file, 0).unwrap();
        let units_per_em = font_face.units_per_em() as f32;
        self.font_data.clear();
        self.font_curves.clear();
        self.font_curve_ordering_list.clear();

        // for glyph_id in 0..font_face.number_of_glyphs() {
        for glyph_id in 0..6 {
            let mut this_char = FontOutlineData::new();

            let Some(char_rect) = font_face
                .outline_glyph(ttf_parser::GlyphId(glyph_id), &mut this_char)
                else {
                    self.font_data.push(FontData::empty());
                    println!("Skiped glyph: {}", glyph_id);
                    continue;
                };
            this_char.finish();

            // processing
            let mut last_x = 0.0;
            let mut last_y = 0.0;
            let origin_x = char_rect.x_min as f32 / units_per_em;
            let origin_y = char_rect.y_min as f32 / units_per_em;
            self.font_data.push(FontData::new(
                self.font_curves.len(),
                self.font_curve_ordering_list.len(),
                &char_rect,
            ));
            let mut curve_info_data = Vec::new();
            for command in this_char.point_command_iter() {
                match *command {
                    font_outline::OutlineDrawCommand::MoveTo(a, b) => {
                        last_x = a / units_per_em - origin_x;
                        last_y = b / units_per_em - origin_y;
                        self.font_curves.push([-1.0, -1.0, last_x, last_y]);
                    }
                    font_outline::OutlineDrawCommand::LineTo(a, b) => {
                        let x2 = a / units_per_em - origin_x;
                        let y2 = b / units_per_em - origin_y;
                        let x1 = (x2 + last_x) / 2.0;
                        let y1 = (y2 + last_y) / 2.0;
                        self.font_curves.push([x1, y1, x2, y2]);
                        // let minx = min_3number(last_x, x1, x2);
                        let maxx = max_3number(last_x, x1, x2);
                        curve_info_data.push((self.font_curves.len() - 1, maxx));
                        last_x = x2;
                        last_y = y2;
                    }
                    font_outline::OutlineDrawCommand::QuadTo(a1, b1, a, b) => {
                        let x1 = a1 / units_per_em - origin_x;
                        let y1 = b1 / units_per_em - origin_y;
                        let x2 = a / units_per_em - origin_x;
                        let y2 = b / units_per_em - origin_y;
                        self.font_curves.push([x1, y1, x2, y2]);
                        let maxx = max_3number(last_x, x1, x2);
                        curve_info_data.push((self.font_curves.len() - 1, maxx));
                        last_x = x2;
                        last_y = y2;
                    }
                    font_outline::OutlineDrawCommand::CurveTo(_, _, _, _, _, _) => unreachable!(),
                    font_outline::OutlineDrawCommand::Close => {}
                }
            }
            curve_info_data
                .sort_by(|(_, max_num0), (_, max_num2)| max_num2.partial_cmp(max_num0).unwrap());
            self.font_curve_ordering_list
                .push(curve_info_data.len() as u32);
            for (index, _) in curve_info_data.iter() {
                self.font_curve_ordering_list.push(*index as u32);
            }
        }

        println!("{:?}", self.font_data[4]);
    }

    pub fn set_text(&mut self) {
        self.string_vec.push(CharData::new(4, 600.0, [0.0, 0.0]));
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

    fn get_rect(&self) -> [f32; 4] {
        [
            self.char_rect.width() as f32,
            self.char_rect.height() as f32,
            self.char_rect.x_min as f32,
            self.char_rect.y_min as f32,
        ]
    }

    pub fn get_font_info(&self) -> (f32, [f32; 4]) {
        (self.units_per_em, self.get_rect())
    }

    pub fn generate_curve_list(&self, x_direction: bool) -> Vec<[[f32; 2]; 4]> {
        let mut curves = Vec::new();
        let mut result = Vec::new();
        let mut last_point = [0.0, 0.0];
        for command in self.this_char.point_command_iter() {
            match *command {
                font_outline::OutlineDrawCommand::MoveTo(a, b) => {
                    let x = a / self.units_per_em;
                    let y = b / self.units_per_em;
                    last_point = [x, y];
                }
                font_outline::OutlineDrawCommand::LineTo(a, b) => {
                    let x = a / self.units_per_em;
                    let y = b / self.units_per_em;
                    let p2 = [x, y];
                    let p1 = [(x + last_point[0]) / 2.0, (y + last_point[1]) / 2.0];
                    curves.push([last_point, p1, p2]);
                    last_point = p2;
                }
                font_outline::OutlineDrawCommand::QuadTo(a1, b1, a, b) => {
                    let x1 = a1 / self.units_per_em;
                    let y1 = b1 / self.units_per_em;
                    let p1 = [x1, y1];
                    let x = a / self.units_per_em;
                    let y = b / self.units_per_em;
                    let p2 = [x, y];
                    curves.push([last_point, p1, p2]);
                    last_point = p2;
                }
                font_outline::OutlineDrawCommand::CurveTo(_, _, _, _, _, _) => unreachable!(),
                font_outline::OutlineDrawCommand::Close => {}
            }
        }

        for [p0, p1, p2] in curves {
            let axis = if x_direction { 0 } else { 1 };
            let mut max = p0[axis];
            if max < p1[axis] {
                max = p1[axis];
            }
            if max < p2[axis] {
                max = p2[axis];
            }
            result.push([
                [max, 0.0],
                [p0[axis], p0[(axis + 1) % 2]],
                [p1[axis], p1[(axis + 1) % 2]],
                [p2[axis], p2[(axis + 1) % 2]],
            ]);
        }
        result.sort_by(|[m0, _, _, _], [m1, _, _, _]| m1[0].partial_cmp(&m0[0]).unwrap());

        result
    }
}
