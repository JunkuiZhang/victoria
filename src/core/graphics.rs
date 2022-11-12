use wgpu::util::DeviceExt;

use crate::settings::GameSettings;

use super::font_manager::FontManager;

pub struct Graphics {
    device: wgpu::Device,
    surface: wgpu::Surface,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    indices_buffer: wgpu::Buffer,
    num: u32,
    font_data_bindgroup: wgpu::BindGroup,
    string_vec_buffer: wgpu::Buffer,
}

#[cfg(windows)]
fn get_backend() -> wgpu::Backends {
    // wgpu::Backends::DX12
    wgpu::Backends::VULKAN
}

#[cfg(not(windows))]
fn get_backend() -> wgpu::Backends {
    wgpu::Backends::PRIMARY
}

impl Graphics {
    pub fn new(
        window: &winit::window::Window,
        settings: &GameSettings,
        font_manager: &FontManager,
    ) -> Self {
        // surface queue config
        let instance = wgpu::Instance::new(get_backend());
        let surface = unsafe { instance.create_surface(window) };
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }))
            .unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Primary Device"),
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();
        println!("{:?}", adapter.get_info());

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: settings.get_window_width(),
            height: settings.get_window_height(),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &surface_config);

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
        let font_vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font Bounding Box Vertex"),
            contents: bytemuck::cast_slice(&verteices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let font_indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font Bounding Box Index"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_font_indices = indices.len();
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 3]>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        };
        let (string_vec_stride, string_vec_size, string_vec) = font_manager.get_string_vec();
        let string_vec_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("String Vec Buffer"),
            contents: bytemuck::cast_slice(string_vec),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let string_vec_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: string_vec_stride,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![1 => Uint32, 2 => Float32x2, 3 => Float32],
        };

        let (font_data_mem_size, font_data_size, font_data) = font_manager.get_font_data();
        println!("{:?}", font_data);
        println!("==================================");
        let (font_texture_size, font_curves) = font_manager.get_font_curves();
        println!("{:?}", font_curves);
        println!("==================================");
        let (font_ordering_size, font_ordering_list) = font_manager.get_font_curve_ordering_list();
        println!("{:?}", font_ordering_list);
        println!("==================================");
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
                                (font_ordering_size * std::mem::size_of::<u32>()) as _,
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let font_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font Rect Buffer"),
            contents: bytemuck::cast_slice(font_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        });
        let font_curves_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glyph Curve Buffer"),
            contents: bytemuck::cast_slice(font_curves),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        });
        let font_ordering_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glyph Curve Ordering Buffer"),
            contents: bytemuck::cast_slice(font_ordering_list),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
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
            bind_group_layouts: &[&font_bindgroup_layout],
            // bind_group_layouts: &[],
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
                    format: surface.get_supported_formats(&adapter)[0],
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Graphics {
            device,
            surface,
            queue,
            render_pipeline,
            vertex_buffer: font_vertices,
            indices_buffer: font_indices,
            num: num_font_indices as u32,
            font_data_bindgroup,
            string_vec_buffer,
        }
    }

    pub fn set_font(&mut self) {}

    pub fn render(&self) {
        // get view
        let texture = self
            .surface
            .get_current_texture()
            .expect("Error getting surface texture!");
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // preset
        let color_attach = [Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                // load: wgpu::LoadOp::Load,
                store: true,
            },
        })];
        let render_pass_desc = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &color_attach,
            depth_stencil_attachment: None,
        };

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        // render pass
        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.string_vec_buffer.slice(..));
            render_pass.set_index_buffer(self.indices_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.font_data_bindgroup, &[]);
            render_pass.draw_indexed(0..self.num, 0, 0..1);
        }

        self.queue.submit(Some(command_encoder.finish()));
        texture.present();
    }
}
