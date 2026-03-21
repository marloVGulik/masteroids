use std::time::Duration;

use crate::game::objects::{asteroid::Asteroid, bullet::Bullet, ship::Ship};
use crate::core::scheduler::{Scheduler};


pub enum GameInput {
    // Direct player input
    Left { dt: f32 },
    Right { dt: f32 },
    Forward { dt: f32 },
    Shoot { current_time: f64 },

    // Networking input
    SummonAsteroid { x: f32, y: f32, direction: f32, speed: f32, size: u8 },
}
#[derive(PartialEq)]
pub enum GameState {
    Waiting,
    Active,
    GameOver,
}
pub struct Game {
    state: GameState,
    scheduler: Scheduler<InternalEvents>,
    collected_rocks: u32,
    asteroids: Vec<Asteroid>,
    ship: Ship,
    bullets: Vec<Bullet>,

    health: u8,
    immune: bool
}
enum InternalEvents {
    Immunity { on: bool }
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::Waiting,
            scheduler: Scheduler::new(),
            collected_rocks: 0,
            asteroids: vec!(),
            ship: Ship::new(),
            bullets: vec!(),
            health: 3,
            immune: false
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
            },

            GameInput::SummonAsteroid { x, y, direction, speed , size} => {
                self.asteroids.push(Asteroid::new(egui::pos2(x, y), speed, direction, size));
            }
        }
    }
    pub fn activate(&mut self) {
        // self.asteroids.push(
        //     Asteroid::new(egui::pos2(20.0, 70.0), 10.0, 135.0, 5),
        // );
        // self.asteroids.push(
        //     Asteroid::new(egui::pos2(50.0, 50.0), 40.0, 45.0, 5),
        // );
        // self.state = GameState::Active;
    }

    pub fn update(&mut self, dt: f32, current_time: f64, mut handler: impl FnMut(GameEvent)) {
        if self.state != GameState::Active { return; }

        self.scheduler.update(|event| {
            match event {
                InternalEvents::Immunity { on}=> {
                    self.immune = *on;
                }
            }
        });

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
                self.ship.move_from_asteroid(asteroid);
                if !self.immune {
                    self.health -= 1;
                    self.immune = true;
                    self.scheduler.schedule(
                        "remove_immunity", 
                        Duration::from_secs(1), 
                        false, 
                        InternalEvents::Immunity { on: false }
                    );
                    handler(GameEvent::Damage { health: self.health });
                }
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

        // Remove asteroids with size 0 (Fixes the invisible asteroids?)
        self.asteroids.retain(|a| {
            a.get_size() > 0
        });
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
    pub fn get_health(&self) -> u8 {
        self.health
    }

    pub fn set_state(&mut self, state: GameState) {
        self.state = state;
    }
}

pub enum GameEvent {
    Damage { health: u8 },
    AsteroidDestroyed { size: u8 },
    PlayerTarget { id: u32 },
}
