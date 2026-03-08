const SPEED: f32 = 10.0;

pub struct Bullet {
    position: egui::Pos2,
    direction: f32,
}

impl Bullet {
    pub fn new(position: egui::Pos2, direction: f32) -> Self {
        Self {
            position,
            direction,
        }
    }

    pub fn get_position(&self) -> egui::Pos2 {
        self.position
    }

    pub fn draw(&self, ui: &mut egui::Ui, size: f32) {
        let size_mp: f32 = size / 100.0;
        let draw_position = egui::pos2(self.position.x * size_mp, self.position.y * size_mp);

        ui.painter().circle_filled(draw_position, 3.0 * size_mp, egui::Color32::RED);
    }

    pub fn update(&mut self, dt: f32) {
        // let radians = self.direction.to_radians();
        self.position.x += SPEED * self.direction.cos() * dt;
        self.position.y += SPEED * self.direction.sin() * dt;
    }
}