use ttf_parser::{FromData, Rect};

use self::font_outline::{FontOutlineData, OutlineDrawCommand};

mod font_outline;

pub struct FontManager {
    this_char: FontOutlineData,
    char_rect: Rect,
    units_per_em: f32,
}

impl FontManager {
    pub fn new<P: AsRef<std::path::Path>>(font_file: P) -> Self {
        let font_file = std::fs::read(font_file).unwrap();
        let font_face = ttf_parser::Face::parse(&font_file, 0).unwrap();
        let mut this_char = FontOutlineData::new();
        let char_rect = font_face
            .outline_glyph(ttf_parser::GlyphId(304), &mut this_char)
            .unwrap();
        this_char.finish();
        let units_per_em = font_face.units_per_em() as f32;

        FontManager {
            this_char,
            char_rect,
            units_per_em,
        }
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
            let axis;
            if x_direction {
                axis = 0;
            } else {
                axis = 1;
            }
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

#[inline]
fn cross_product(v1_x: f32, v1_y: f32, v2_x: f32, v2_y: f32) -> f32 {
    v1_x * v2_y - v1_y * v2_x
}
