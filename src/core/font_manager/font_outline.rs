pub struct FontOutlineData {
    points: Vec<OutlineDrawCommand>,
}

pub enum OutlineDrawCommand {
    // x, y
    MoveTo(f32, f32),
    // x, y
    LineTo(f32, f32),
    // x1, y1, x, y
    QuadTo(f32, f32, f32, f32),
    // x1, y1, x2, y2, x, y
    CurveTo(f32, f32, f32, f32, f32, f32),
    // Close curve
    Close,
}

impl FontOutlineData {
    pub fn new() -> Self {
        FontOutlineData { points: Vec::new() }
    }
}

impl ttf_parser::OutlineBuilder for FontOutlineData {
    fn move_to(&mut self, x: f32, y: f32) {
        self.points.push(OutlineDrawCommand::MoveTo(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.points.push(OutlineDrawCommand::LineTo(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.points.push(OutlineDrawCommand::QuadTo(x1, y1, x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.points
            .push(OutlineDrawCommand::CurveTo(x1, y1, x2, y2, x, y));
    }

    fn close(&mut self) {
        self.points.push(OutlineDrawCommand::Close);
    }
}
