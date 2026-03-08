const MIN_SIZE: f32 = 10.0;

pub struct Asteroid {
    position: egui::Pos2,
    speed: f32,
    direction: f32,
    size: i8,
}

impl Asteroid {
    pub fn new(position: egui::Pos2, speed: f32, direction: f32, size: i8) -> Self {
        Self {
            position,
            speed,
            direction,
            size,
        }
    }
    pub fn hit_and_copy(old_asteroid: &mut Asteroid) -> Self {
        old_asteroid.size -= 1; // Reduce size of the original asteroid

        Self {
            position: old_asteroid.position,
            speed: old_asteroid.speed,
            direction: old_asteroid.direction + 180.0, // Change direction for the new asteroid
            size: old_asteroid.size,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let radians = self.direction.to_radians();
        self.position.x += self.speed * radians.cos() * dt;
        self.position.y += self.speed * radians.sin() * dt;
    }

    pub fn check_collision(&self, point: egui::Pos2) -> bool {
        let radius = MIN_SIZE * self.size as f32;
        let dx = self.position.x - point.x;
        let dy = self.position.y - point.y;

        let is_hit = (dx * dx + dy * dy).sqrt() < radius;
        if is_hit == true {
            println!("Asteroid hit at position: {:?}", self.position);
        }

        is_hit
    }
}