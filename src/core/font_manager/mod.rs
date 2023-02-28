use std::{path::Path, rc::Rc};

use owned_ttf_parser::{AsFaceRef, OwnedFace};
use wgpu::util::DeviceExt;

use crate::utils::{max_3number, min_3number};

use self::{
    font_data::FontData, font_graphics::FontGraphics, font_info_saving::FontDrawingData,
    font_outline::FontOutlineData, string_data::CharData,
};

use super::graphics::GpuContext;

mod font_data;
pub mod font_graphics;
mod font_info_saving;
mod font_outline;
pub mod string_data;

pub struct FontManager {
    font_face: OwnedFace,
    window_size: [f32; 2],
    font_name: String,
}

impl FontManager {
    pub fn new<P: AsRef<std::path::Path> + Clone>(
        font_path: P,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        let font_file = std::fs::read(font_path.as_ref()).expect("Unable to read font!");
        let font_face = owned_ttf_parser::OwnedFace::from_vec(font_file, 0).unwrap();
        let window_size = [window_width as f32, window_height as f32];
        FontManager {
            font_face,
            window_size,
            font_name: font_path
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .to_lowercase(),
        }
    }

    pub fn prepare(&self, gpu_context: &GpuContext) -> FontGraphics {
        let font_face = self.font_face.as_face_ref();

        let mut font_save_path = Path::new("data").join("font");
        if !font_save_path.exists() {
            std::fs::create_dir_all(font_save_path.as_path()).unwrap();
        }
        let mut file_name = self.font_name.clone();
        file_name.push_str(".bin");
        font_save_path = font_save_path.join(file_name);

        let font_drawing_data: FontDrawingData = if font_save_path.exists() {
            bincode::deserialize(&std::fs::read(font_save_path.as_path()).unwrap()).unwrap_or_else(
                |_| {
                    let data = get_font_drawing_data(font_face);
                    std::fs::write(
                        font_save_path,
                        bincode::serialize(&data).expect("Unable to serialize font data!"),
                    )
                    .unwrap();
                    data
                },
            )
        } else {
            let data = get_font_drawing_data(font_face);
            std::fs::write(
                font_save_path,
                bincode::serialize(&data).expect("Unable to deserialize font data!"),
            )
            .unwrap();
            data
        };

        // shader config
        let draw_shader = gpu_context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Draw Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("draw_shader.wgsl").into()),
            });

        // pipeline config
        let verteices = [
            [0.0f32, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let indices = [0u16, 1, 2, 2, 1, 3];
        let vertex_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Font Bounding Box Vertex"),
                    contents: bytemuck::cast_slice(&verteices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let index_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Font Bounding Box Index"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 3]>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        };

        let string_vec_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CharData>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![1 => Uint32, 2 => Float32x2, 3 => Float32],
        };

        let window_info_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Window Size Info"),
                    contents: bytemuck::cast_slice(&self.window_size),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
        let uniform_bindgroup_layout =
            gpu_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Uniform Bindgroup"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<[f32; 2]>() as _,
                            ),
                        },
                        count: None,
                    }],
                });
        let uniform_bindgroup = gpu_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform Bindgroup"),
                layout: &uniform_bindgroup_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: window_info_buffer.as_entire_binding(),
                }],
            });

        let font_data_mem_size = std::mem::size_of::<FontData>();
        let font_data_size = font_drawing_data.font_data.len();
        let font_texture_size = font_drawing_data.font_curves.len();
        let font_bindgroup_layout =
            gpu_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Font Data"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (font_data_mem_size * font_data_size) as _,
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (font_texture_size * std::mem::size_of::<[f32; 4]>()) as _,
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (font_drawing_data.hor_band_list.len()
                                        * std::mem::size_of::<u32>())
                                        as _,
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (font_drawing_data.ver_band_list.len()
                                        * std::mem::size_of::<u32>())
                                        as _,
                                ),
                            },
                            count: None,
                        },
                    ],
                });
        let font_info_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Font Rect Buffer"),
                    contents: bytemuck::cast_slice(&font_drawing_data.font_data),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let font_curves_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Glyph Curve Buffer"),
                    contents: bytemuck::cast_slice(&font_drawing_data.font_curves),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let horizontal_band_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Glyph Curve Ordering Buffer"),
                    contents: bytemuck::cast_slice(&font_drawing_data.hor_band_list),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let vertical_band_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Glyph Curve Ordering Buffer"),
                    contents: bytemuck::cast_slice(&font_drawing_data.ver_band_list),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let font_data_bindgroup =
            gpu_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Glyph Curve Bindgroup"),
                    layout: &font_bindgroup_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: font_info_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: font_curves_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: horizontal_band_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: vertical_band_buffer.as_entire_binding(),
                        },
                    ],
                });

        let rp_layout =
            gpu_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Renderpipeline Layout"),
                    bind_group_layouts: &[&uniform_bindgroup_layout, &font_bindgroup_layout],
                    push_constant_ranges: &[],
                });
        let render_pipeline =
            gpu_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&rp_layout),
                    vertex: wgpu::VertexState {
                        module: &draw_shader,
                        entry_point: "vs_main",
                        buffers: &[vertex_buffer_layout, string_vec_buffer_layout],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &draw_shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gpu_context
                                .surface
                                .get_capabilities(&gpu_context.adapter)
                                .formats[0],
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });

        FontGraphics {
            vertex_buffer: Rc::new(vertex_buffer),
            index_buffer: Rc::new(index_buffer),
            render_pipeline: Rc::new(render_pipeline),
            uniform_bindgroup: Rc::new(uniform_bindgroup),
            font_data_bindgroup: Rc::new(font_data_bindgroup),
        }
    }

    #[inline]
    pub fn get_face(&self) -> &owned_ttf_parser::Face {
        self.font_face.as_face_ref()
    }

    #[inline]
    pub fn get_window_size(&self) -> [f32; 2] {
        self.window_size
    }
}

fn band_process(
    horzontal: bool,
    band_size: f32,
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    offset: usize,
    band_list: &mut [Vec<(f32, u32)>],
) {
    let x_axis;
    let y_axis;
    if horzontal {
        x_axis = 0;
        y_axis = 1;
    } else {
        x_axis = 1;
        y_axis = 0;
    }
    let maxy = max_3number(p0[y_axis], p1[y_axis], p2[y_axis]);
    let miny = min_3number(p0[y_axis], p1[y_axis], p2[y_axis]);
    for (index, target_band) in band_list.iter_mut().enumerate() {
        let starty = band_size * (index as f32);
        let endy = starty + band_size;
        if maxy < starty || miny > endy {
            continue;
        }
        target_band.push((
            max_3number(p0[x_axis], p1[x_axis], p2[x_axis]),
            offset as u32,
        ));
    }
}

fn get_font_drawing_data(font_face: &owned_ttf_parser::Face) -> FontDrawingData {
    let units_per_em = font_face.units_per_em() as f32;
    let mut font_data = Vec::new();
    let mut font_curves = Vec::new();
    let mut hor_band_list = Vec::new(); // from bottom to top
    let mut ver_band_list = Vec::new(); // from left to right
    let mut curves_index = 0;

    for glyph_id in 0..font_face.number_of_glyphs() {
        let mut this_char = FontOutlineData::new();

        let Some(bounding_box) = font_face
                .outline_glyph(owned_ttf_parser::GlyphId(glyph_id), &mut this_char)
                else {
                    font_data.push(FontData::empty());
                    println!("Skiped glyph: {}", glyph_id);
                    continue;
                };

        let this_char_curve_count = this_char.number_of_curves();
        let mut band_count = this_char_curve_count / 8; // 8 curves per band
        if band_count < 2 {
            band_count = 2;
        }
        if band_count > 16 {
            band_count = 16;
        }
        let hor_band_size = bounding_box.height() as f32 / units_per_em / band_count as f32;
        let ver_band_size = bounding_box.width() as f32 / units_per_em / band_count as f32;

        // processing
        let mut point0_x = 0.0;
        let mut point0_y = 0.0;
        let origin_x = bounding_box.x_min as f32 / units_per_em;
        let origin_y = bounding_box.y_min as f32 / units_per_em;
        let this_char_curve_start = curves_index;
        font_data.push(FontData::new(
            this_char_curve_start,
            hor_band_list.len(),
            ver_band_list.len(),
            band_count as u32,
            &bounding_box,
            units_per_em,
        ));

        let epsilon = 0.000001;
        let mut hor_band_temp = Vec::new();
        let mut ver_band_temp = Vec::new();
        for _ in 0..band_count {
            hor_band_temp.push(Vec::new());
            ver_band_temp.push(Vec::new());
        }
        for command in this_char.point_command_iter() {
            match *command {
                font_outline::OutlineDrawCommand::MoveTo(a, b) => {
                    point0_x = a / units_per_em - origin_x;
                    point0_y = b / units_per_em - origin_y;
                    font_curves.push([-1.0, -1.0, point0_x, point0_y]);
                    curves_index += 1;
                }
                font_outline::OutlineDrawCommand::LineTo(a, b) => {
                    let point2_x = a / units_per_em - origin_x;
                    let point2_y = b / units_per_em - origin_y;
                    let point1_x = (point2_x + point0_x) / 2.0;
                    let point1_y = (point2_y + point0_y) / 2.0;
                    font_curves.push([point1_x, point1_y, point2_x, point2_y]);

                    let this_char_glyph_offset = curves_index - this_char_curve_start;
                    // horizontal band detect
                    if (point2_y - point0_y).abs() > epsilon {
                        // reject horizontal line, cause it makes no contribute to winding number
                        band_process(
                            true,
                            hor_band_size,
                            [point0_x, point0_y],
                            [point1_x, point1_y],
                            [point2_x, point2_y],
                            this_char_glyph_offset,
                            &mut hor_band_temp,
                        );
                    }
                    // vertical band detect
                    if (point2_x - point0_x).abs() > epsilon {
                        // reject vertical line, cause it makes no contribute to winding number
                        band_process(
                            false,
                            ver_band_size,
                            [point0_x, point0_y],
                            [point1_x, point1_y],
                            [point2_x, point2_y],
                            this_char_glyph_offset,
                            &mut ver_band_temp,
                        );
                    }
                    point0_x = point2_x;
                    point0_y = point2_y;
                    curves_index += 1;
                }
                font_outline::OutlineDrawCommand::QuadTo(a1, b1, a, b) => {
                    let point1_x = a1 / units_per_em - origin_x;
                    let point1_y = b1 / units_per_em - origin_y;
                    let point2_x = a / units_per_em - origin_x;
                    let point2_y = b / units_per_em - origin_y;
                    font_curves.push([point1_x, point1_y, point2_x, point2_y]);
                    let this_char_glyph_offset = curves_index - this_char_curve_start;
                    // horizontal band detect
                    band_process(
                        true,
                        hor_band_size,
                        [point0_x, point0_y],
                        [point1_x, point1_y],
                        [point2_x, point2_y],
                        this_char_glyph_offset,
                        &mut hor_band_temp,
                    );
                    // vertical band detect
                    band_process(
                        false,
                        ver_band_size,
                        [point0_x, point0_y],
                        [point1_x, point1_y],
                        [point2_x, point2_y],
                        this_char_glyph_offset,
                        &mut ver_band_temp,
                    );
                    point0_x = point2_x;
                    point0_y = point2_y;
                    curves_index += 1;
                }
                font_outline::OutlineDrawCommand::CurveTo(_, _, _, _, _, _) => unreachable!(),
                font_outline::OutlineDrawCommand::Close => {}
            }
        }

        let mut hcount = 2 * band_count;
        let mut vcount = 2 * band_count;
        for index in 0..band_count {
            hor_band_list.push(hcount as u32);
            hor_band_list.push(hor_band_temp[index].len() as u32);
            hor_band_temp[index].sort_by(|(max0, _), (max1, _)| max1.partial_cmp(max0).unwrap());
            hcount += hor_band_temp[index].len();

            ver_band_list.push(vcount as u32);
            ver_band_list.push(ver_band_temp[index].len() as u32);
            ver_band_temp[index].sort_by(|(max0, _), (max1, _)| max1.partial_cmp(max0).unwrap());
            vcount += ver_band_temp[index].len();
        }

        for index in 0..band_count {
            for (_, offset) in hor_band_temp[index].iter() {
                hor_band_list.push(*offset);
            }
            for (_, offset) in ver_band_temp[index].iter() {
                ver_band_list.push(*offset);
            }
        }
    }

    FontDrawingData {
        font_data,
        font_curves,
        hor_band_list,
        ver_band_list,
    }
}
