#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, Debug)]
pub struct WindowSetting(pub u32, pub u32);

pub fn window_title() -> String {
    "Game".to_string()
}

impl Default for WindowSetting {
    fn default() -> Self {
        WindowSetting(1920, 1080)
    }
}
