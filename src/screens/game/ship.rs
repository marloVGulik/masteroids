use crate::screens::game::bullet::Bullet;

// const MAX_SPEED: f32 = 50.0;
const ACCELERATION: f32 = 50.0; // Acceleration factor
const FRICTION: f32 = 0.5; // Friction factor

const MAX_ROTATION_SPEED: f32 = 10.0;
const ROTATION_ACCELERATION: f32 = 5.0; // Rotation acceleration factor
// const ROTATION_FRICTION: f32 = 0.1; // Rotation friction factor

const SHOT_COOLDOWN: f64 = 0.5; // seconds

pub struct Ship {
    velocity: egui::Vec2,
    rotation_speed: f32,

    rotation: f32,
    position: egui::Pos2,

    last_shot_time: f64,
    thrusting: bool,
}

impl Ship {
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

    pub fn turn_left(&mut self, dt: f32) {
        self.rotation_speed = (self.rotation_speed - ROTATION_ACCELERATION * dt).max(-MAX_ROTATION_SPEED);
    }
    pub fn turn_right(&mut self, dt: f32) {
        self.rotation_speed = (self.rotation_speed + ROTATION_ACCELERATION * dt).min(MAX_ROTATION_SPEED);
    }
    pub fn foward(&mut self, dt: f32) {
        // self.speed = (self.speed + ACCELERATION * dt).min(MAX_SPEED);
        self.velocity.x += ACCELERATION * self.rotation.cos() * dt;
        self.velocity.y += ACCELERATION * self.rotation.sin() * dt;

        self.thrusting = true;
    }
    pub fn shoot(&mut self, current_time: f64) -> Option<Bullet> {
        if current_time - self.last_shot_time >= SHOT_COOLDOWN {
            self.last_shot_time = current_time;

            return Some(Bullet::new(self.position, self.rotation, current_time));
        }

        None
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, size: f32) {
        let size_mp: f32 = size / 100.0;
        let draw_position = egui::pos2(self.position.x * size_mp, self.position.y * size_mp);

        // ui.painter().circle_filled(draw_position, 4.0 * size_mp, egui::Color32::BLUE);
        // Ship dimensions (scaled)
        let ship_radius = 3.0 * size_mp;
        let angle = self.rotation; // Assuming this is in radians

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

    pub fn update(&mut self, dt:f32) {
        // Update position based on speed and rotation
        // let radians = self.rotation.to_radians();
        // self.position.x += self.speed * self.rotation.cos() * dt;
        // self.position.y += self.speed * self.rotation.sin() * dt;

        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;

        self.position.x = self.position.x.rem_euclid(100.0);
        self.position.y = self.position.y.rem_euclid(100.0);

        // Apply rotation
        self.rotation += self.rotation_speed * dt;

        // Slow down over time (friction)
        self.velocity *= 1.0 - (FRICTION * dt); // Friction factor
        self.rotation_speed *= 1.0 - (FRICTION * dt); // Friction factor
    }
}