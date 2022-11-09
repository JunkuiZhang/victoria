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
    x_curve_bindgroup: wgpu::BindGroup,
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
            [-0.7f32, 0.7, 0.0],
            [-0.7, -0.7, 0.0],
            [0.7, 0.7, 0.0],
            [0.7, -0.7, 0.0],
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

        let x_curve_list = font_manager.generate_curve_list(true);
        let x_curve_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("X Curve List"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        (x_curve_list.len() * std::mem::size_of::<[f32; 2]>() * 4) as _,
                    ),
                },
                count: None,
            }],
        });
        let x_curve_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glyph Curve Buffer"),
            contents: bytemuck::cast_slice(&x_curve_list),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        });
        println!("Curve list: {}", x_curve_list.len());
        println!("{:?}", x_curve_list);
        let x_curve_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph Curve Bindgroup"),
            layout: &x_curve_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: x_curve_buffer.as_entire_binding(),
            }],
        });

        let rp_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Renderpipeline Layout"),
            bind_group_layouts: &[&x_curve_bg_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&rp_layout),
            vertex: wgpu::VertexState {
                module: &draw_shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
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
            x_curve_bindgroup,
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
            render_pass.set_index_buffer(self.indices_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.x_curve_bindgroup, &[]);
            render_pass.draw_indexed(0..self.num, 0, 0..1);
        }

        self.queue.submit(Some(command_encoder.finish()));
        texture.present();
    }
}
