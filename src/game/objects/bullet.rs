//! Bullet entity.

const SPEED: f32 = 125.0;
const LIFETIME: f64 = 1.2; // seconds

/// A bullet fired by the ship.
///
/// Travels in a fixed direction at constant speed and expires after `LIFETIME` seconds.
pub struct Bullet {
    position: egui::Pos2,
    direction: f32,
    lifetime: f64,
}

impl Bullet {
    /// Creates a new bullet at `position` traveling in `direction` (radians).
    pub fn new(position: egui::Pos2, direction: f32, current_time: f64) -> Self {
        Self {
            position,
            direction,
            lifetime: current_time + LIFETIME,
        }
    }

    /// Returns true if the bullet has not yet expired.
    pub fn is_alive(&self, current_time: f64) -> bool {
        current_time < self.lifetime
    }

    /// Returns the bullet's current position.
    pub fn get_position(&self) -> egui::Pos2 {
        self.position
    }

    /// Draws the bullet as a small red circle.
    pub fn draw(&self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect) {
        let size_mp: f32 = size / 100.0;
        let draw_position = egui::pos2(
            play_area.min.x + self.position.x * size_mp, 
            play_area.min.y + self.position.y * size_mp
        );
        ui.painter().circle_filled(draw_position, 0.5 * size_mp, egui::Color32::RED);
    }

    /// Advances position by SPEED * direction * dt and wraps around the play area.
    pub fn update(&mut self, dt: f32) {
        self.position.x += SPEED * self.direction.cos() * dt;
        self.position.y += SPEED * self.direction.sin() * dt;
        self.position.x = self.position.x.rem_euclid(100.0);
        self.position.y = self.position.y.rem_euclid(100.0);
    }
}
