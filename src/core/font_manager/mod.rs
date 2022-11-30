use std::rc::Rc;

use owned_ttf_parser::{AsFaceRef, OwnedFace};
use wgpu::util::DeviceExt;

use crate::utils::max_3number;

use self::{
    font_data::FontData, font_graphics::FontGraphics, font_outline::FontOutlineData,
    string_data::CharData,
};

mod font_data;
pub mod font_graphics;
mod font_manager_builder;
mod font_outline;
pub mod string_data;

pub struct FontManager {
    font_face: OwnedFace,
    window_size: [f32; 2],
}

impl FontManager {
    pub fn new<P: AsRef<std::path::Path>>(
        font_path: P,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        let font_file = std::fs::read(font_path).expect("Unable to read font!");
        let font_face = owned_ttf_parser::OwnedFace::from_vec(font_file, 0).unwrap();
        let window_size = [window_width as f32, window_height as f32];
        FontManager {
            font_face,
            window_size,
        }
    }

    pub fn prepare(
        &self,
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
    ) -> FontGraphics {
        let font_face = self.font_face.as_face_ref();
        let units_per_em = font_face.units_per_em() as f32;
        let mut font_data = Vec::new();
        let mut font_curves = Vec::new();
        let mut font_curve_ordering_list = Vec::new();
        let mut curves_index = 0;
        let mut ordering_index = 0;

        for glyph_id in 0..font_face.number_of_glyphs() {
            let mut this_char = FontOutlineData::new();

            let Some(bounding_box) = font_face
                .outline_glyph(owned_ttf_parser::GlyphId(glyph_id), &mut this_char)
                else {
                    font_data.push(FontData::empty());
                    println!("Skiped glyph: {}", glyph_id);
                    continue;
                };

            // processing
            let mut last_x = 0.0;
            let mut last_y = 0.0;
            let origin_x = bounding_box.x_min as f32 / units_per_em; // -0.1 for padding
            let origin_y = bounding_box.y_min as f32 / units_per_em;
            let this_char_curve_start = curves_index;
            font_data.push(FontData::new(
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
                        font_curves.push([-1.0, -1.0, last_x, last_y]);
                        curves_index += 1;
                    }
                    font_outline::OutlineDrawCommand::LineTo(a, b) => {
                        let x2 = a / units_per_em - origin_x;
                        let y2 = b / units_per_em - origin_y;
                        let x1 = (x2 + last_x) / 2.0;
                        let y1 = (y2 + last_y) / 2.0;
                        font_curves.push([x1, y1, x2, y2]);
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
                        font_curves.push([x1, y1, x2, y2]);
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

            font_curve_ordering_list.push(curve_info_data.len() as u32);
            for (offset, _) in curve_info_data.iter() {
                // let row_num = (*index / 4096) << 16;
                // let col_num = *index % 4096;
                // self.font_curve_ordering_list
                //     .push((row_num | col_num) as u32);
                font_curve_ordering_list.push(*offset as u32);
            }
            ordering_index += curve_info_data.len() + 1;
        }

        // shader config
        let draw_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
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
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font Bounding Box Vertex"),
            contents: bytemuck::cast_slice(&verteices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

        let window_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Window Size Info"),
            contents: bytemuck::cast_slice(&self.window_size),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let uniform_bindgroup_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bindgroup"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<[f32; 2]>() as _
                        ),
                    },
                    count: None,
                }],
            });
        let uniform_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bindgroup"),
            layout: &uniform_bindgroup_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: window_info_buffer.as_entire_binding(),
            }],
        });

        let font_data_mem_size = std::mem::size_of::<FontData>();
        let font_data_size = font_data.len();
        let font_texture_size = font_curves.len();
        let font_ordering_size = font_curve_ordering_list.len();
        let font_bindgroup_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                                (font_ordering_size * std::mem::size_of::<u16>()) as _,
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let font_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font Rect Buffer"),
            contents: bytemuck::cast_slice(&font_data),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let font_curves_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glyph Curve Buffer"),
            contents: bytemuck::cast_slice(&font_curves),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let font_ordering_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glyph Curve Ordering Buffer"),
            contents: bytemuck::cast_slice(&font_curve_ordering_list),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let font_data_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                    resource: font_ordering_buffer.as_entire_binding(),
                },
            ],
        });

        let rp_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Renderpipeline Layout"),
            bind_group_layouts: &[&uniform_bindgroup_layout, &font_bindgroup_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: surface.get_supported_formats(adapter)[0],
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

    pub fn get_face(&self) -> &owned_ttf_parser::Face {
        self.font_face.as_face_ref()
    }

    pub fn get_window_size(&self) -> [f32; 2] {
        self.window_size
    }
}
