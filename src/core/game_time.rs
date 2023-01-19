use std::time::Instant;

use super::{
    graphics::{GpuContext, UpdateInfo},
    gui_manager::GuiManager,
};

pub struct GameTimeManager {
    delta_time: f32,
    fps: usize,
    frame_count: u32,
    last_frame: Instant,
    last_update: Instant,
}

impl GameTimeManager {
    pub fn new() -> Self {
        GameTimeManager {
            delta_time: 0.01,
            fps: 0,
            frame_count: 0,
            last_frame: Instant::now(),
            last_update: Instant::now(),
        }
    }

    pub fn update(
        &mut self,
        gui_manager: &mut GuiManager,
        update_queue: &mut Vec<UpdateInfo>,
        context: &GpuContext,
    ) {
        let current_time = Instant::now();
        self.delta_time = current_time.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = current_time;
        self.frame_count = (1.0 / self.delta_time) as u32;
        if self.last_update.elapsed().as_secs_f32() > 0.5 {
            let content = format!("FPS: {}", self.frame_count).as_bytes().to_vec();
            gui_manager.update_at(self.fps, content, update_queue, context);
            self.last_update = current_time;
        }
    }
}
