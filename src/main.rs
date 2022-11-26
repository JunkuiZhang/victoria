use settings::GameSettings;

use crate::core::Controller;

mod core;
mod settings;
mod utils;

fn main() {
    env_logger::init();
    let game_settings = Box::new(GameSettings::new());
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(
            game_settings.get_window_width(),
            game_settings.get_window_height(),
        ))
        .with_title(game_settings.get_window_title())
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();
    let mut controller = Controller::new(&window, game_settings);
    controller.preprocess();
    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    controller.exit();
                    control_flow.set_exit();
                }
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            state: winit::event::ElementState::Pressed,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => match keycode {
                    winit::event::VirtualKeyCode::Escape => {
                        controller.exit();
                        control_flow.set_exit();
                    }
                    _ => {}
                },
                // TODO: impl these
                // winit::event::WindowEvent::ReceivedCharacter(_) => todo!(),
                // winit::event::WindowEvent::ModifiersChanged(_) => todo!(),
                // winit::event::WindowEvent::CursorMoved { position, .. } => todo!(),
                // winit::event::WindowEvent::MouseWheel { delta, phase, .. } => todo!(),
                // winit::event::WindowEvent::MouseInput { state, button, .. } => todo!(),
                _ => {}
            },
            winit::event::Event::MainEventsCleared => {
                // Update here
                controller.update();
                // Render here
                controller.draw();
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod test {

    #[test]
    fn general_test() {
        let resource_dir = std::path::Path::new("data");
        let content = std::fs::read(resource_dir.join("Inconsolata-Regular.ttf")).unwrap();
        panic!("{:?}", content);
    }
}
