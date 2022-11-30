use std::{cell::RefCell, rc::Rc};

use crate::settings::GameSettings;

use super::{
    font_manager::{font_graphics::FontGraphics, FontManager},
    gui_manager::GuiManager,
};

pub struct Graphics {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    queue: wgpu::Queue,
    pub font_graphics: FontGraphics,
    staging_belt: wgpu::util::StagingBelt,
}

pub trait Drawable {
    // fn draw(&self, render_pass: Rc<RefCell<wgpu::RenderPass>>, graphics: &Graphics);
    fn draw<'a>(&'a self, render_pass: Rc<RefCell<wgpu::RenderPass<'a>>>, graphics: &'a Graphics);
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
        let staging_belt = wgpu::util::StagingBelt::new(0x1000);

        // Font config
        let font_graphics = font_manager.prepare(&device, &surface, &adapter);

        Graphics {
            device,
            surface,
            queue,
            font_graphics,
            staging_belt,
        }
    }

    pub fn draw(&self, gui: &GuiManager) {
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

        {
            let render_pass = Rc::new(RefCell::new(
                command_encoder.begin_render_pass(&render_pass_desc),
            ));
            gui.draw(render_pass, self);
        }

        self.queue.submit(Some(command_encoder.finish()));
        texture.present();
    }
}
