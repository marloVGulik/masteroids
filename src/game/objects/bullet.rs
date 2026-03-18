const SPEED: f32 = 125.0;
const LIFETIME: f64 = 1.2; // seconds

pub struct Bullet {
    position: egui::Pos2,
    direction: f32,
    lifetime: f64,
}

impl Bullet {
    pub fn new(position: egui::Pos2, direction: f32, current_time: f64) -> Self {
        Self {
            position,
            direction,
            lifetime: current_time + LIFETIME,
        }
    }

    pub fn is_alive(&self, current_time: f64) -> bool {
        current_time < self.lifetime
    }

    pub fn get_position(&self) -> egui::Pos2 {
        self.position
    }

    pub fn draw(&self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect) {
        let size_mp: f32 = size / 100.0;
        let draw_position = egui::pos2(
            play_area.min.x + self.position.x * size_mp, 
            play_area.min.y + self.position.y * size_mp
        );

        ui.painter().circle_filled(draw_position, 0.5 * size_mp, egui::Color32::RED);
    }

    pub fn update(&mut self, dt: f32) {
        // let radians = self.direction.to_radians();
        self.position.x += SPEED * self.direction.cos() * dt;
        self.position.y += SPEED * self.direction.sin() * dt;

        self.position.x = self.position.x.rem_euclid(100.0);
        self.position.y = self.position.y.rem_euclid(100.0);
    }
}