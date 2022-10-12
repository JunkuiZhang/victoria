pub struct Graphics {
    device: wgpu::Device,
    surface: wgpu::Surface,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
}

#[cfg(windows)]
fn get_backend() -> wgpu::Backends {
    wgpu::Backends::DX12
}

#[cfg(not(windows))]
fn get_backend() -> wgpu::Backends {
    wgpu::Backends::PRIMARY
}

impl Graphics {
    pub fn new(window: &winit::window::Window) -> Self {
        // surface queue config
        let instance = wgpu::Instance::new(get_backend());
        let surface = unsafe { instance.create_surface(window) };
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
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
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: crate::settings::WINDOW_WIDTH,
            height: crate::settings::WINDOW_HEIGHT,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface.get_supported_alpha_modes(&adapter)[0],
        };
        surface.configure(&device, &surface_config);

        // shader config
        let draw_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Draw Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("draw_shader.wgsl").into()),
        });

        // pipeline config
        let rp_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&rp_layout),
            vertex: wgpu::VertexState {
                module: &draw_shader,
                entry_point: "main_vs",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &draw_shader,
                entry_point: "main_fs",
                targets: &[Some(surface_config.format.into())],
            }),
            multiview: None,
        });

        Graphics {
            device,
            surface,
            queue,
            render_pipeline,
        }
    }

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
                load: wgpu::LoadOp::Load,
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
        }

        self.queue.submit(Some(command_encoder.finish()));
        texture.present();
    }
}
