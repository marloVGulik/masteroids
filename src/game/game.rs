//! The game loop, state machine, and entity management.
//!
//! `Game` manages asteroids, bullets, and the ship. It processes input via `interact()`,
//! updates physics each frame via `update()`, and draws all entities via `draw()`.

use std::time::Duration;

use crate::game::objects::{asteroid::Asteroid, bullet::Bullet, ship::Ship};
use crate::core::scheduler::Scheduler;

/// Input that can be fed into the game.
///
/// Comes from two sources: direct player input (keys) and networking (remote players).
pub enum GameInput {
    /// Rotate left by `dt` seconds.
    Left { dt: f32 },
    /// Rotate right by `dt` seconds.
    Right { dt: f32 },
    /// Thrust forward by `dt` seconds.
    Forward { dt: f32 },
    /// Fire a bullet at the current time.
    Shoot { current_time: f64 },
    /// Spawn an asteroid at a specific position (from the host).
    SummonAsteroid { x: f32, y: f32, direction: f32, speed: f32, size: u8 },
}

/// Current state of the game loop.
#[derive(PartialEq, Clone, Copy)]
pub enum GameState {
    /// Waiting for players to ready up.
    Waiting,
    /// Game is active and entities are updating.
    Active,
    /// Game has ended (e.g. player health reached zero).
    GameOver,
}

/// Top-level game state.
///
/// Contains the ship, asteroids, bullets, health, and a scheduler for timed events
/// like the invulnerability window after taking damage.
pub struct Game {
    state: GameState,
    scheduler: Scheduler<InternalEvents>,
    collected_rocks: u32,
    asteroids: Vec<Asteroid>,
    ship: Ship,
    bullets: Vec<Bullet>,
    health: u8,
    immune: bool,
}

/// Internal scheduler tasks used by `Game`.
enum InternalEvents {
    /// Toggles ship immunity after taking damage.
    Immunity { on: bool },
}

impl Game {
    /// Creates a new game with a single ship, no asteroids, and full health.
    pub fn new() -> Self {
        Self {
            state: GameState::Waiting,
            scheduler: Scheduler::new(),
            collected_rocks: 0,
            asteroids: vec!(),
            ship: Ship::new(),
            bullets: vec!(),
            health: 3,
            immune: false,
        }
    }

    /// Processes a single game input (movement, shooting, or remote asteroid spawn).
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
                self.spawn_asteroid(x, y, direction, speed, size);
            }
        }
    }

    /// Activates the game (called when the player is ready).
    ///
    /// Resets the game to a clean state: ship position, health, bullets cleared.
    /// Does NOT change the game state — that is the responsibility of the caller
    /// (host sends StartGame for multiplayer; single-player screen sets state directly).
    pub fn activate(&mut self) {
        self.ship = Ship::new();
        self.bullets.clear();
        self.health = 3;
        self.immune = false;
    }

    /// Spawns a single asteroid at the given position, direction, speed, and size.
    pub fn spawn_asteroid(&mut self, x: f32, y: f32, direction: f32, speed: f32, size: u8) {
        self.asteroids.push(Asteroid::new(egui::pos2(x, y), speed, direction, size));
    }

    /// Advances the game simulation by `dt` seconds.
    ///
    /// Updates the ship, bullets, and asteroids. Checks all collisions (bullet-asteroid,
    /// asteroid-ship, asteroid-asteroid) and emits `GameEvent` callbacks for the screen
    /// to handle (e.g. sending damage info over the network).
    pub fn update(&mut self, dt: f32, current_time: f64, mut handler: impl FnMut(GameEvent)) {
        if self.state == GameState::Waiting {
            return;
        }

        self.scheduler.update(|event| {
            match event {
                InternalEvents::Immunity { on}=> {
                    self.immune = *on;
                }
            }
        });
        self.scheduler.remove_fired();

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

    /// Draws all game entities (asteroids, bullets, ship) into the given egui UI.
    pub fn draw(&mut self, ui: &mut egui::Ui, size: f32, play_area: egui::Rect) {
        for asteroid in self.asteroids.iter() {
            asteroid.draw(ui, size, play_area);
        }
        for bullet in self.bullets.iter() {
            bullet.draw(ui, size, play_area);
        }
        self.ship.draw(ui, size, play_area);
    }

    /// Returns the number of rocks the player has collected.
    pub fn get_collected_rocks(&self) -> u32 {
        self.collected_rocks
    }

    /// Returns the player's current health.
    pub fn get_health(&self) -> u8 {
        self.health
    }

    /// Sets the game state.
    pub fn set_state(&mut self, state: GameState) {
        self.state = state;
    }

    /// Returns the current game state.
    pub fn get_state(&self) -> GameState {
        self.state
    }
}

/// Events emitted by `Game::update()` to notify the screen layer.
pub enum GameEvent {
    /// The player took damage; `health` is the remaining health after the hit.
    Damage { health: u8 },
    /// An asteroid was destroyed by a bullet; `size` is the pre-split size.
    AsteroidDestroyed { size: u8 },
    /// Another player is targeting this player.
    PlayerTarget { id: u32 },
}
