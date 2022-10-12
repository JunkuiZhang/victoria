use crate::core::Controller;

mod core;
mod settings;
mod utils;

fn main() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(
            settings::WINDOW_WIDTH,
            settings::WINDOW_HEIGHT,
        ))
        .with_title(settings::WINDOW_TITLE)
        .build(&event_loop)
        .unwrap();
    let controller = Controller::new(&window);
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }
            winit::event::Event::MainEventsCleared => {
                // Update here
                // Render here
                controller.draw();
            }
            _ => {}
        }
    });
}
