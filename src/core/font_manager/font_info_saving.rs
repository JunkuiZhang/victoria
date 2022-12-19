use serde::{Deserialize, Serialize};

use super::font_data::FontData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FontDrawingData {
    pub font_data: Vec<FontData>,
    pub font_curves: Vec<[f32; 4]>,
    pub hor_band_list: Vec<u32>,
    pub ver_band_list: Vec<u32>,
}
