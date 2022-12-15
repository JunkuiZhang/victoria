use std::time::Instant;

use super::{
    graphics::{Graphics, UpdateInfo},
    gui_manager::GuiManager,
};

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

    pub fn update(&mut self, gui_manager: &mut GuiManager, update_queue: &mut Vec<UpdateInfo>) {
        let current_time = Instant::now();
        let elpsed = current_time.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = current_time;
        self.frame_count = (1.0 / elpsed) as u64;
        if self.last_update.elapsed().as_secs_f64() > 0.5 {
            let content = format!("FPS: {}", self.frame_count).as_bytes().to_vec();
            gui_manager.update_at(self.fps, content, update_queue);
            self.last_update = current_time;
        }
    }
}
