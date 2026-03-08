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

    pub fn update(&mut self, dt: f32) {
        let radians = self.direction.to_radians();
        self.position.x += SPEED * radians.cos() * dt;
        self.position.y += SPEED * radians.sin() * dt;
    }
}