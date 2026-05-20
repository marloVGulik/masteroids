//! Asteroid entity.

use crate::core::physics;

const MIN_SIZE: f32 = 2.0;
const SIZE_MULTIPLIER: f32 = 0.75;

/// An asteroid that floats around the play area.
///
/// Each asteroid has a `size` (1–4) that determines its physical radius. When hit by a
/// bullet, `hit_and_copy` splits it into two smaller asteroids with randomized directions.
pub struct Asteroid {
    position: egui::Pos2,
    velocity: egui::Vec2,
    size: u8,
}

impl Asteroid {
    /// Creates a new asteroid at `position` moving at `speed` in `direction` (degrees).
    pub fn new(position: egui::Pos2, speed: f32, direction: f32, size: u8) -> Self {
        Self {
            position,
            velocity: egui::Vec2::new(speed * direction.to_radians().cos(), speed * direction.to_radians().sin()),
            size,
        }
    }

    /// Splits this asteroid into two smaller asteroids moving apart at 45-degree angles.
    ///
    /// Modifies `old_asteroid` in place (halving its size) and returns the new asteroid.
    pub fn hit_and_copy(old_asteroid: &mut Asteroid) -> Self {
        let new_size = old_asteroid.size - 1;
        let speed = old_asteroid.velocity.length();
        let angle = old_asteroid.velocity.angle();

        let spread = std::f32::consts::FRAC_PI_4; // 45 degrees in radians
        
        let vel_1 = egui::Vec2::angled(angle + spread) * speed;
        let vel_2 = egui::Vec2::angled(angle - spread) * speed;

        let offset = vel_1.normalized() * (new_size as f32 * 5.0);

        old_asteroid.size = new_size;
        old_asteroid.velocity = vel_1;
        old_asteroid.position += offset;

        Self {
            position: old_asteroid.position - offset,
            velocity: vel_2,
            size: new_size,
        }
    }

    /// Draws the asteroid as a filled circle.
    pub fn draw(&self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect) {
        let size_mp: f32 = size / 100.0;
        let draw_position = egui::pos2(play_area.min.x + self.position.x * size_mp, play_area.min.y + self.position.y * size_mp);

        ui.painter().circle_filled(
            draw_position, 
            self.get_physical_radius() * size_mp, 
            egui::Color32::GRAY
        );
    }

    /// Advances position by velocity * dt and wraps around the play area.
    pub fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
        self.position.x = self.position.x.rem_euclid(100.0);
        self.position.y = self.position.y.rem_euclid(100.0);
    }

    /// Returns true if `point` falls within this asteroid's collision circle.
    pub fn check_bullet_collision(&self, point: egui::Pos2) -> bool {
        physics::point_in_circle(point, self.position, self.get_physical_radius())
    }

    /// Returns true if this asteroid collides with `other`.
    pub fn check_asteroid_collision(&self, other: &Asteroid) -> bool {
        physics::circle_collision(self.position, self.get_physical_radius(), other.position, other.get_physical_radius())
    }

    /// Resolves a collision with another asteroid (position correction + velocity reflection).
    pub fn move_from_asteroid(&mut self, other: &Asteroid) {
        let delta = self.position - other.position;
        let distance = delta.length();
        let min_dist = self.get_physical_radius() + other.get_physical_radius();

        if distance < min_dist && distance > 0.0 {
            let push_direction = delta / distance;
            let overlap = min_dist - distance;
            self.position += push_direction * (overlap * 0.5);

            let mass_ratio = other.size as f32 / (self.size as f32 + other.size as f32);
            let speed = self.velocity.length();
            let dot = self.velocity.dot(push_direction);
            
            if dot < 0.0 {
                self.velocity = (self.velocity - 2.0 * dot * push_direction) * 0.85;
            }
            
            self.velocity += push_direction * (speed * mass_ratio * 0.2);
        }
    }

    /// Returns the asteroid's size category (1–4).
    pub fn get_size(&self) -> u8 {
        self.size
    }

    /// Returns the physical radius in the 0–100 coordinate space.
    pub fn get_physical_radius(&self) -> f32 {
        MIN_SIZE * (self.size as f32 * SIZE_MULTIPLIER)
    }

    /// Returns the asteroid's position.
    pub fn get_position(&self) -> egui::Pos2 {
        self.position
    }
}
