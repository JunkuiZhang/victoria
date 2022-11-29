use std::time::Instant;

use super::{graphics::Graphics, gui_manager::GuiManager};

pub struct GameTimeManager {
    frame_count: u64,
    last_frame: Instant,
    last_update: Instant,
    fps: usize,
}

impl GameTimeManager {
    pub fn new() -> Self {
        GameTimeManager {
            frame_count: 0,
            last_frame: Instant::now(),
            last_update: Instant::now(),
            fps: 0,
        }
    }

    pub fn update(
        &mut self,
        gui_manager: &mut GuiManager,
        face: &owned_ttf_parser::Face,
        graphics: &Graphics,
    ) {
        let current_time = Instant::now();
        let elpsed = current_time.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = current_time;
        self.frame_count = (1.0 / elpsed) as u64;
        if self.last_update.elapsed().as_secs() > 1 {
            gui_manager.update_at(self.fps, self.frame_count, face, graphics);
            self.last_update = current_time;
        }
    }
}
