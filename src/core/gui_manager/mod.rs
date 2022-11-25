pub struct GuiManager {
    window_size: [f32; 2],
}

impl GuiManager {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        GuiManager {
            window_size: [window_width as f32, window_height as f32],
        }
    }

    pub fn add_text(&self, _content: String) {}
}
