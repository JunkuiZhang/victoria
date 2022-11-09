pub struct FontOutlineData {
    pub points: Vec<OutlineDrawCommand>,
}

#[derive(Debug)]
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

    pub fn finish(&mut self) {
        self.points.shrink_to_fit();
    }

    pub fn point_command_iter(&self) -> std::slice::Iter<OutlineDrawCommand> {
        self.points.iter()
    }

    pub fn print(&self) {
        for point in self.points.iter() {
            println!("{:?}", point);
        }
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
        unreachable!("Should not reach!");
    }

    fn close(&mut self) {
        self.points.push(OutlineDrawCommand::Close);
    }
}
