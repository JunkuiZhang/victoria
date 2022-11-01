use std::time::Instant;

pub struct GameTimeManager {
    frame_count: u64,
    last_update: Instant,
}

impl GameTimeManager {
    pub fn new() -> Self {
        GameTimeManager {
            frame_count: 0,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        self.frame_count += 1;
        let current_time = Instant::now();
        if self.last_update.elapsed().as_secs() > 1 {
            println!("FPS: {}", self.frame_count);
            self.frame_count = 0;
            self.last_update = current_time;
        }
    }
}
