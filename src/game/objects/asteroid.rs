use crate::core::physics;

const MIN_SIZE: f32 = 2.0;

pub struct Asteroid {
    position: egui::Pos2,
    velocity: egui::Vec2,
    size: u8,
}

impl Asteroid {
    pub fn new(position: egui::Pos2, speed: f32, direction: f32, size: u8) -> Self {
        Self {
            position,
            velocity: egui::Vec2::new(speed * direction.to_radians().cos(), speed * direction.to_radians().sin()),
            size,
        }
    }
    pub fn hit_and_copy(old_asteroid: &mut Asteroid) -> Self {
        let new_size = old_asteroid.size - 1;
        let speed = old_asteroid.velocity.length();
        let angle = old_asteroid.velocity.angle(); // egui::Vec2 has an .angle() method (radians)

        // Calculate two new angles: +45 degrees and -45 degrees
        let spread = std::f32::consts::FRAC_PI_4; // 45 degrees in radians
        
        let vel_1 = egui::Vec2::angled(angle + spread) * speed;
        let vel_2 = egui::Vec2::angled(angle - spread) * speed;

        // Small offset so they don't immediately collide with each other
        let offset = vel_1.normalized() * (new_size as f32 * 5.0);

        old_asteroid.size = new_size;
        old_asteroid.velocity = vel_1;
        old_asteroid.position += offset;

        Self {
            position: old_asteroid.position - offset,
            velocity: vel_2, // New asteroid moves in the opposite direction
            size: new_size,
        }
    }

    pub fn draw(&self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect) {
        let size_mp: f32 = size / 100.0;
        let draw_position = egui::pos2(play_area.min.x + self.position.x * size_mp, play_area.min.y + self.position.y * size_mp);

        ui.painter().circle_filled(
            draw_position, 
            self.get_physical_radius() * size_mp, 
            egui::Color32::GRAY
        );
    }

    pub fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;

        self.position.x = self.position.x.rem_euclid(100.0); // Wrap around play area
        self.position.y = self.position.y.rem_euclid(100.0);
    }

    pub fn check_bullet_collision(&self, point: egui::Pos2) -> bool {
        physics::point_in_circle(point, self.position, self.get_physical_radius())
    }
    pub fn check_asteroid_collision(&self, other: &Asteroid) -> bool {
        physics::circle_collision(self.position, self.get_physical_radius(), other.position, other.get_physical_radius())
    }
    
    pub fn move_from_asteroid(&mut self, other: &Asteroid) {
        let delta = self.position - other.position;
        let distance = delta.length();
        let min_dist = self.get_physical_radius() + other.get_physical_radius();

        // 1. Only act if they are actually overlapping
        if distance < min_dist && distance > 0.0 {
            let push_direction = delta / distance; // Normalized vector
            
            // 2. Position Correction
            // Move the asteroid out of the collision immediately.
            // We move it by half the overlap (the other asteroid moves the other half).
            let overlap = min_dist - distance;
            self.position += push_direction * (overlap * 0.5);

            // 3. Velocity Reflection
            // We use a "conservation of momentum" approach.
            // The force factor you calculated is good for mass scaling!
            let mass_ratio = other.size as f32 / (self.size as f32 + other.size as f32);
            
            // Reflect velocity across the collision normal
            // This stops the infinite acceleration loop
            let speed = self.velocity.length();
            let dot = self.velocity.dot(push_direction);
            
            if dot < 0.0 {
                // Only bounce if they are moving TOWARDS each other
                // This prevents them from getting "sucked" together
                self.velocity = (self.velocity - 2.0 * dot * push_direction) * 0.85; // 0.8 for "bounciness" loss
            }
            
            // Add a small "impulse" based on the other's size to make it feel impactful
            self.velocity += push_direction * (speed * mass_ratio * 0.2);
        }
    }

    pub fn get_size(&self) -> u8 {
        self.size
    }
    pub fn get_physical_radius(&self) -> f32 {
        MIN_SIZE * (self.size as f32 / 1.5)
    }
    pub fn get_position(&self) -> egui::Pos2 {
        self.position
    }
}