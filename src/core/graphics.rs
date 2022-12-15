use std::rc::Rc;

use crate::settings::GameSettings;

use super::{
    font_manager::{font_graphics::FontGraphics, FontManager},
    resources::ResourceManager,
};

pub struct GpuContext {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
}

pub struct Graphics {
    pub context: GpuContext,
    queue: wgpu::Queue,
    staging_belt: wgpu::util::StagingBelt,
    pub update_queue: Vec<UpdateInfo>,
    pub draw_queue: Vec<DrawCall>,
}

pub trait Drawable {
    // fn draw(&self, render_pass: Rc<RefCell<wgpu::RenderPass>>, graphics: &Graphics);
    fn update_queue(&mut self, content: Vec<u8>, update_queue: &mut Vec<UpdateInfo>);
    fn get_update_info(&self) -> UpdateInfo;
    fn get_draw_info(&self, resource_manager: &ResourceManager) -> DrawCall;
}

pub struct UpdateInfo {
    pub target_buffer: Rc<wgpu::Buffer>,
    pub size: wgpu::BufferSize,
    pub content: Rc<Vec<u8>>,
}

pub enum DrawCall {
    DrawIndexed(DrawIndexedInfo),
    Draw(u32),
}

pub struct DrawIndexedInfo {
    pub pipeline: Rc<wgpu::RenderPipeline>,
    pub vertex_buffer: Vec<Rc<wgpu::Buffer>>,
    pub index_buffer: Rc<wgpu::Buffer>,
    pub bindgroup: Vec<Rc<wgpu::BindGroup>>,
    pub indices: u32,
    pub instance: u32,
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
    pub fn new(window: &winit::window::Window, settings: &GameSettings) -> Self {
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
        let context = GpuContext {
            device,
            surface,
            adapter,
        };

        Graphics {
            context,
            queue,
            staging_belt,
            update_queue: Vec::new(),
            draw_queue: Vec::new(),
        }
    }

    pub fn draw(&mut self) {
        // get view
        let texture = self
            .context
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
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        // update stuff
        let updated = !self.update_queue.is_empty();
        {
            // self.staging_belt.write_buffer(&mut command_encoder, target, offset, size, device).;
            for update in self.update_queue.iter() {
                self.staging_belt
                    .write_buffer(
                        &mut command_encoder,
                        update.target_buffer.as_ref(),
                        0,
                        update.size,
                        &self.context.device,
                    )
                    .copy_from_slice(update.content.as_ref());
            }
            self.staging_belt.finish();
        }
        self.update_queue.clear();
        // render stuff
        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_desc);
            for draw_call in self.draw_queue.iter() {
                match draw_call {
                    DrawCall::DrawIndexed(info) => {
                        render_pass.set_pipeline(info.pipeline.as_ref());
                        for (slot, buffer) in info.vertex_buffer.iter().enumerate() {
                            render_pass.set_vertex_buffer(slot as u32, buffer.slice(..));
                        }
                        for (index, group) in info.bindgroup.iter().enumerate() {
                            render_pass.set_bind_group(index as u32, group, &[]);
                        }
                        render_pass.set_index_buffer(
                            info.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        render_pass.draw_indexed(0..info.indices, 0, 0..info.instance);
                    }
                    DrawCall::Draw(_) => todo!(),
                }
            }
        }
        self.draw_queue.clear();

        self.queue.submit(Some(command_encoder.finish()));

        if updated {
            self.staging_belt.recall();
        }

        texture.present();
    }
}
