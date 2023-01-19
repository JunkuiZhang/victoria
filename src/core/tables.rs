pub struct Tables {
    pub delta_time: f32,
    pub fps: u32,
}

impl Tables {
    pub fn new() -> Self {
        Tables {
            delta_time: 0.01,
            fps: 100,
        }
    }
}
