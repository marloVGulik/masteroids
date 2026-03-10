pub trait GameObject {
    fn update(&mut self, dt: f64);
    fn draw(&self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect);
}

pub trait Collidable {
    fn check_collision(&self, point: egui::Pos2) -> bool;
}
