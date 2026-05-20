//! The player ship entity.
//!
//! `Ship` tracks position (normalized 0–100), velocity, rotation, and thrust state.
//! It handles rotation, acceleration, drawing (triangle body + thrust flame), and
//! collision with asteroids.

use egui::Stroke;

use crate::{core::physics, game::objects::{asteroid::Asteroid, bullet::Bullet}};

const SHIP_RADIUS: f32 = 3.0;
// const MAX_SPEED: f32 = 50.0;
const ACCELERATION: f32 = 60.0; // Acceleration factor
const FRICTION: f32 = 0.7; // Friction factor

const MAX_ROTATION_SPEED: f32 = 20.0;
const ROTATION_ACCELERATION: f32 = 10.0; // Rotation acceleration factor
// const ROTATION_FRICTION: f32 = 0.1; // Rotation friction factor

const SHOT_COOLDOWN: f64 = 0.5; // seconds

/// The player-controlled ship.
///
/// Uses normalized 0–100 coordinates. Wraps around the play area edges via `rem_euclid`.
pub struct Ship {
    velocity: egui::Vec2,
    rotation_speed: f32,
    rotation: f32,
    position: egui::Pos2,
    last_shot_time: f64,
    thrusting: bool,
}

impl Ship {
    /// Creates a new ship centered at (50, 50) with zero velocity.
    pub fn new() -> Self {
        Self {
            velocity: egui::Vec2::ZERO,
            rotation_speed: 0.0,
            rotation: 0.0,
            position: egui::Pos2 { x: 50.0, y: 50.0 },
            last_shot_time: 0.0,
            thrusting: false,
        }
    }

    /// Rotates the ship left by the given delta time.
    pub fn turn_left(&mut self, dt: f32) {
        self.rotation_speed = (self.rotation_speed - ROTATION_ACCELERATION * dt).max(-MAX_ROTATION_SPEED);
    }

    /// Rotates the ship right by the given delta time.
    pub fn turn_right(&mut self, dt: f32) {
        self.rotation_speed = (self.rotation_speed + ROTATION_ACCELERATION * dt).min(MAX_ROTATION_SPEED);
    }

    /// Applies thrust in the direction the ship is currently facing.
    pub fn forward(&mut self, dt: f32) {
        self.velocity.x += ACCELERATION * self.rotation.cos() * dt;
        self.velocity.y += ACCELERATION * self.rotation.sin() * dt;
        self.thrusting = true;
    }

    /// Fires a bullet if the shot cooldown has expired.
    pub fn shoot(&mut self, current_time: f64) -> Option<Bullet> {
        if current_time - self.last_shot_time >= SHOT_COOLDOWN {
            self.last_shot_time = current_time;
            return Some(Bullet::new(self.position, self.rotation, current_time));
        }
        None
    }

    /// Draws the ship as a filled triangle with optional thrust flame.
    pub fn draw(&mut self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect) {
        let size_mp: f32 = size / 100.0;
        let draw_position = egui::pos2(
            play_area.min.x + self.position.x * size_mp, 
            play_area.min.y + self.position.y * size_mp
        );
        let ship_radius = SHIP_RADIUS * size_mp;
        let angle = self.rotation;

        // Debug circle for ship
        ui.painter().add(egui::Shape::circle_stroke(draw_position, ship_radius, Stroke::new(3.0, egui::Color32::DARK_RED)));

        // Define the 3 points of the triangle relative to (0,0)
        let points = [
            egui::pos2(ship_radius, 0.0),          // Nose
            egui::pos2(-ship_radius, -ship_radius * 0.8), // Back Left
            egui::pos2(-ship_radius, ship_radius * 0.8),  // Back Right
        ];

        // Rotate and Translate points to the ship's actual position
        let rotated_points: Vec<egui::Pos2> = points
            .iter()
            .map(|p| {
                let rx = p.x * angle.cos() - p.y * angle.sin();
                let ry = p.x * angle.sin() + p.y * angle.cos();
                egui::pos2(draw_position.x + rx, draw_position.y + ry)
            })
            .collect();
        
        // Draw the triangle
        ui.painter().add(egui::Shape::Path(egui::epaint::PathShape {
            points: rotated_points,
            closed: true,
            fill: egui::Color32::BLUE,
            stroke: egui::Stroke::new(1.0, egui::Color32::WHITE).into(),
        }));

        if self.thrusting {
            let flame_points = [
                egui::pos2(-ship_radius * 1.5, 0.0), // Base of the flame
                egui::pos2(-ship_radius, -ship_radius * 0.5), // Left tip
                egui::pos2(-ship_radius, ship_radius * 0.5), // Right tip
            ];
            let rotated_flame_points: Vec<egui::Pos2> = flame_points
                .iter()
                .map(|p| {
                    let rx = p.x * angle.cos() - p.y * angle.sin();
                    let ry = p.x * angle.sin() + p.y * angle.cos();
                    egui::pos2(draw_position.x + rx, draw_position.y + ry)
                })
                .collect();
            ui.painter().add(egui::Shape::Path(egui::epaint::PathShape {
                points: rotated_flame_points,
                closed: true,
                fill: if self.thrusting { egui::Color32::YELLOW } else { egui::Color32::TRANSPARENT },
                stroke: egui::Stroke::new(1.0, egui::Color32::RED).into(),
            }));

            self.thrusting = false; // Reset thrusting state after drawing
        }
    }

    /// Updates position, rotation, and applies friction.
    pub fn update(&mut self, dt:f32) {
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;

        self.position.x = self.position.x.rem_euclid(100.0);
        self.position.y = self.position.y.rem_euclid(100.0);

        // Apply rotation
        self.rotation += self.rotation_speed * dt;

        // Slow down over time (friction)
        self.velocity *= 1.0 - (FRICTION * dt);
        self.rotation_speed *= 1.0 - (FRICTION * dt);
    }

    /// Returns true if this ship collides with the given asteroid.
    pub fn collision_asteroid(&self, asteroid: &Asteroid) -> bool {
        physics::circle_collision(self.position, SHIP_RADIUS, asteroid.get_position(), asteroid.get_physical_radius() as f32)
    }

    /// Resolves a collision between this ship and an asteroid (position correction + velocity reflection).
    pub fn move_from_asteroid(&mut self, asteroid: &Asteroid) {
        let delta = self.position - asteroid.get_position();
        let distance = delta.length();
        let min_dist = SHIP_RADIUS + asteroid.get_physical_radius();

        if distance < min_dist && distance > 0.0 {
            let push_direction = delta / distance;
            let overlap = min_dist - distance;
            self.position += push_direction * (overlap * 0.5);

            let mass_ratio = 2.0;
            let speed = self.velocity.length();
            let dot = self.velocity.dot(push_direction);
            
            if dot < 0.0 {
                self.velocity = (self.velocity - 2.0 * dot * push_direction) * 0.85;
            }
            
            self.velocity += push_direction * (speed * mass_ratio * 0.2);
        }
    }
}
