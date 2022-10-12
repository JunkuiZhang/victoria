pub struct Graphics {
    surface: wgpu::Surface,
    queue: wgpu::Queue,
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

        Graphics { surface, queue }
    }
}
