use crate::{game::objects::{asteroid::Asteroid, bullet::Bullet, ship::Ship}};


pub enum GameInput {
    Left { dt: f32 },
    Right { dt: f32 },
    Forward { dt: f32 },
    Shoot { current_time: f64 },
}
pub struct Game {
    collected_rocks: u32,
    asteroids: Vec<Asteroid>,
    ship: Ship,
    bullets: Vec<Bullet>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            collected_rocks: 0,
            asteroids: vec!(),
            ship: Ship::new(),
            bullets: vec!(),
        }
    }

    pub fn interact(&mut self, input: GameInput) {
        match input {
            GameInput::Left { dt } => self.ship.turn_left(dt),
            GameInput::Right { dt } => self.ship.turn_right(dt),
            GameInput::Forward { dt } => self.ship.forward(dt),
            GameInput::Shoot { current_time } => {
                if let Some(bullet) = self.ship.shoot(current_time) {
                    self.bullets.push(bullet);
                }
            }
        }
    }
    pub fn activate(&mut self) {
        self.asteroids.push(
            Asteroid::new(egui::pos2(50.0, 50.0), 40.0, 135.0, 5),
        );
        self.asteroids.push(
            Asteroid::new(egui::pos2(50.0, 50.0), 40.0, 45.0, 5),
        );
    }

    pub fn update(&mut self, dt: f32, current_time: f64, mut handler: impl FnMut(GameEvent)) {
        // Update ship
        self.ship.update(dt);

        // Update bullets
        for bullet in &mut self.bullets {
            bullet.update(dt);
        }
        self.bullets.retain(|bullet| bullet.is_alive(current_time)); // Remove bullets that have expired

        // New asteroids and update asteroids
        let mut new_asteroids: Vec<Asteroid> = vec!();
        for asteroid in &mut self.asteroids {
            asteroid.update(dt);

            // Only retain bullets that have not hit
            self.bullets.retain(|bullet| {

                // Check collision
                if asteroid.check_bullet_collision(bullet.get_position()) {
                    // Send asteroid destroyed event
                    handler(GameEvent::AsteroidDestroyed { size: asteroid.get_size() });

                    if asteroid.get_size() <= 1 {
                        println!("Collected a rock!");
                        self.collected_rocks += 1;
                    }
                    // Add new asteroid
                    new_asteroids.push(Asteroid::hit_and_copy(asteroid));
                    
                    // Remove bullet
                    return false; 
                }

                return true; // Retain bullet
            });

            // Check collision with ship
            if self.ship.collision_asteroid(asteroid) {
                handler(GameEvent::Died);
            }
        }

        // Adding asteroids to self
        self.asteroids.append(&mut new_asteroids);

        // Asteroid collisions
        let len = self.asteroids.len();
        if len < 2 { return; } // Cannot collide 1 or 0 asteroids

        for i in 0..len - 1 { // Loop to second-to-last
            let (head, tail) = self.asteroids.split_at_mut(i + 1);
            let asteroid_a = &mut head[i]; 

            for asteroid_b in tail {
                if asteroid_a.check_asteroid_collision(asteroid_b) {
                    // Apply physics to both
                    asteroid_a.move_from_asteroid(asteroid_b);
                    asteroid_b.move_from_asteroid(asteroid_a);
                }
            }
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect) {
        for asteroid in self.asteroids.iter() {
            asteroid.draw(ui, size, play_area);
        }
        for bullet in self.bullets.iter() {
            bullet.draw(ui, size, play_area);
        }
        self.ship.draw(ui, size, play_area);
    }

    pub fn get_collected_rocks(&self) -> u32 {
        self.collected_rocks
    }
}

pub enum GameEvent {
    Died,
    AsteroidDestroyed { size: u8 },
    // PlayerTargetChanged,
    
}
