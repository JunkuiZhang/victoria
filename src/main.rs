fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
        .with_title("Test")
        .build(&event_loop)
        .unwrap();
    event_loop.run(move |event, _, control| match event {
        winit::event::Event::WindowEvent {
            window_id,
            event: winit::event::WindowEvent::CloseRequested,
        } if window.id() == window_id => {
            control.set_exit();
        }
        winit::event::Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
