use self::font_outline::FontOutlineData;

mod font_outline;

pub struct FontManager {
    latter_a: FontOutlineData,
    latter_g: FontOutlineData,
    units_per_em: [f32; 2],
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
        latter_a.finish();
        font_face
            .outline_glyph(ttf_parser::GlyphId(309), &mut latter_g)
            .unwrap();
        latter_g.finish();
        // let units_per_em = [600.0, font_face.units_per_em() as f32];
        let units_per_em = [700.0, 700.0];

        FontManager {
            latter_a,
            latter_g,
            units_per_em,
        }
    }

    pub fn generate_curve_list(&self, x_direction: bool) -> Vec<[[f32; 2]; 4]> {
        let mut curves = Vec::new();
        let mut result = Vec::new();
        let mut last_point = [0.0, 0.0];
        for command in self.latter_a.point_command_iter() {
            match *command {
                font_outline::OutlineDrawCommand::MoveTo(a, b) => {
                    let x = a / self.units_per_em[0];
                    let y = b / self.units_per_em[1];
                    last_point = [x, y];
                    println!("[{a}, {b}] ==> [{x}, {y}]");
                }
                font_outline::OutlineDrawCommand::LineTo(a, b) => {
                    let x = a / self.units_per_em[0];
                    let y = b / self.units_per_em[1];
                    let p2 = [x, y];
                    let p1 = [(x + last_point[0]) / 2.0, (y + last_point[1]) / 2.0];
                    curves.push([last_point, p1, p2]);
                    last_point = p2;
                    println!("[{a}, {b}] ==> [{x}, {y}]");
                }
                font_outline::OutlineDrawCommand::QuadTo(a1, b1, a, b) => {
                    let x1 = a1 / self.units_per_em[0];
                    let y1 = b1 / self.units_per_em[1];
                    let x = a / self.units_per_em[0];
                    let y = b / self.units_per_em[1];
                    curves.push([last_point, [x1, y1], [x, y]]);
                    println!("[{a}, {b}] ==> [{x}, {y}]");
                }
                font_outline::OutlineDrawCommand::CurveTo(_, _, _, _, _, _) => unreachable!(),
                font_outline::OutlineDrawCommand::Close => {}
            }
        }
        println!("{:?}", curves);
        println!("===================================");

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

    pub fn get_vertices(&self) -> Vec<[f32; 3]> {
        let mut verteices = Vec::new();
        let origin_x = -1.0;
        let origin_y = 1.0;
        let mut new_start = false;
        for command in self.latter_a.point_command_iter() {
            match *command {
                font_outline::OutlineDrawCommand::MoveTo(a, b) => {
                    new_start = true;
                    verteices.push([origin_x, origin_y, 0.0]);
                    let x = a / self.units_per_em[0] * 2.0 - 1.0;
                    let y = b / self.units_per_em[1] * 2.0 - 1.0;
                    verteices.push([x, y, 0.0]);
                    continue;
                }
                font_outline::OutlineDrawCommand::LineTo(a, b) => {
                    let x = a / self.units_per_em[0] * 2.0 - 1.0;
                    let y = b / self.units_per_em[1] * 2.0 - 1.0;
                    verteices.push([x, y, 0.0]);
                }
                font_outline::OutlineDrawCommand::QuadTo(_, _, _, _) => todo!(),
                font_outline::OutlineDrawCommand::CurveTo(_, _, _, _, _, _) => todo!(),
                font_outline::OutlineDrawCommand::Close => todo!(),
            }
            if new_start {
                new_start = false;
            }
        }

        verteices
    }

    pub fn get_vertices_indices(&self) -> (Vec<[f32; 3]>, Vec<[u16; 3]>) {
        let mut verteices = Vec::new();
        let mut indices = Vec::new();
        let origin_x = -1.0;
        let origin_y = 1.0;
        verteices.push([origin_x, origin_y, 0.0]);
        let mut start_point_index = 1;
        for command in self.latter_a.point_command_iter() {
            match *command {
                font_outline::OutlineDrawCommand::MoveTo(a, b) => {
                    if verteices.len() > 1 {
                        // indicates new start point
                        start_point_index = verteices.len() as u16;
                    }
                    let x = a / self.units_per_em[0] * 2.0 - 1.0;
                    let y = b / self.units_per_em[1] * 2.0 - 1.0;
                    verteices.push([x, y, 0.0]);
                }
                font_outline::OutlineDrawCommand::LineTo(a, b) => {
                    let x = a / self.units_per_em[0] * 2.0 - 1.0;
                    let y = b / self.units_per_em[1] * 2.0 - 1.0;
                    let next_point_index = verteices.len() as u16;
                    verteices.push([x, y, 0.0]);
                    indices.push([0, next_point_index - 1, next_point_index]);
                }
                font_outline::OutlineDrawCommand::QuadTo(_, _, a, b) => {
                    let x = a / self.units_per_em[0] * 2.0 - 1.0;
                    let y = b / self.units_per_em[1] * 2.0 - 1.0;
                    let next_point_index = verteices.len() as u16;
                    verteices.push([x, y, 0.0]);
                    indices.push([0, next_point_index - 1, next_point_index]);
                }
                font_outline::OutlineDrawCommand::CurveTo(_, _, _, _, _, _) => unreachable!(),
                font_outline::OutlineDrawCommand::Close => {
                    verteices.pop();
                    let last = indices.len() - 1;
                    indices[last][2] = start_point_index;
                }
            }
        }
        println!("{:?}", indices);
        // for [a, b, c] in indices.iter_mut() {
        //     let x1 = verteices[*b as usize][0] - verteices[*a as usize][0];
        //     let y1 = verteices[*b as usize][1] - verteices[*a as usize][1];
        //     let x2 = verteices[*c as usize][0] - verteices[*b as usize][0];
        //     let y2 = verteices[*c as usize][1] - verteices[*b as usize][1];
        //     println!("Cross: {:.01}", cross_product(x1, y1, x2, y2));
        //     if cross_product(x1, y1, x2, y2) < 0.0 {
        //         let temp = *a;
        //         *a = *b;
        //         *b = temp;
        //     }
        // }
        // println!("{:?}", indices);
        // for [a, b, c] in indices.iter_mut() {
        //     let x1 = verteices[*b as usize][0] - verteices[*a as usize][0];
        //     let y1 = verteices[*b as usize][1] - verteices[*a as usize][1];
        //     let x2 = verteices[*c as usize][0] - verteices[*b as usize][0];
        //     let y2 = verteices[*c as usize][1] - verteices[*b as usize][1];
        //     assert!(cross_product(x1, y1, x2, y2) > 0.0);
        //     if !(cross_product(x1, y1, x2, y2) > 0.0) {
        //         println!("({}, {}, {})", *a, *b, *c);
        //     }
        // }

        verteices.shrink_to_fit();
        indices.shrink_to_fit();

        (verteices, indices)
    }
}

#[inline]
fn cross_product(v1_x: f32, v1_y: f32, v2_x: f32, v2_y: f32) -> f32 {
    v1_x * v2_y - v1_y * v2_x
}
