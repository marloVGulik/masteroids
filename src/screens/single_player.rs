//! The single-player screen: local game with pressure ramping.

use std::time::Duration;

use rand::RngExt;

use crate::game::game::{Game, GameInput, GameState, GameEvent};
use crate::screen::ScreenCommand;
use crate::screen::Screen;
use crate::core::scheduler::Scheduler;

/// Scheduled tasks for single-player pressure ramping.
enum Tasks {
    /// Periodically spawns an asteroid for pressure ramping.
    SummonAsteroid,
}

/// The single-player game screen.
///
/// Manages a local `Game` instance with a scheduler that periodically spawns
/// asteroids to increase pressure on the player over time.
pub struct SinglePlayer {
    game: Game,
    scheduler: Scheduler<Tasks>,
    pressure_enabled: bool,
    username: String,
    randomizer: rand::prelude::ThreadRng,
}

impl SinglePlayer {
    /// Creates a new single-player screen.
    pub fn new(username: String) -> Self {
        Self {
            game: Game::new(),
            scheduler: Scheduler::new(),
            pressure_enabled: true,
            username,
            randomizer: rand::rng(),
        }
    }

    /// Spawns a single random asteroid using the game's coordinate space.
    fn spawn_random_asteroid(&mut self) {
        let x: f32 = self.randomizer.random_range(0.0..=100.0);
        let y: f32 = self.randomizer.random_range(0.0..=100.0);
        let direction: f32 = self.randomizer.random_range(0.0..=360.0);
        let speed: f32 = self.randomizer.random_range(0.0..10.0);
        let size: u8 = self.randomizer.random_range(1..=3);

        self.game.interact(GameInput::SummonAsteroid { x, y, direction, speed, size });
    }
}

impl Screen for SinglePlayer {
    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, _order: egui::Order) -> Option<ScreenCommand> {
        let mut cmd = None;

        ui.vertical_centered(|ui| {
            ui.heading(format!("Single Player - {}", self.username));
            if ui.button("Back").clicked() {
                cmd = Some(ScreenCommand::Start);
            }
            if ui.button(if self.pressure_enabled { "Disable Pressure" } else { "Enable Pressure" }).clicked() {
                self.pressure_enabled = !self.pressure_enabled;
            }
        });

        ui.horizontal_centered(|ui| {
            ui.label(format!("Health: {}", self.game.get_health()));
            ui.label(format!("Collected rocks: {}", self.game.get_collected_rocks()));
        });

        if self.game.get_state() == GameState::Active {
            let available_rect = ui.available_rect_before_wrap();
            let width = available_rect.width();
            let height = available_rect.height();
            let size = width.min(height);
            let play_area = egui::Rect::from_center_size(
                available_rect.center(), 
                egui::vec2(size, size)
            );
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.painter().rect_stroke(
                    play_area, 
                    0.0, 
                    (2.0, egui::Color32::WHITE), 
                    egui::StrokeKind::Inside
                );
                let mut game_ui = ui.new_child(egui::UiBuilder::new().max_rect(play_area));
                self.game.draw(&mut game_ui, size, play_area);
            });
        }

        ctx.request_repaint();

        cmd
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {
        self.game.activate();
        self.game.set_state(GameState::Active);
        self.scheduler.schedule(
            "summon_asteroid", 
            Duration::from_secs(3), 
            true, 
            Tasks::SummonAsteroid
        );
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &eframe::Frame) -> Option<ScreenCommand> {
        let mut cmd = None;

        // Handle pressure ramping scheduler
        if self.pressure_enabled && self.game.get_state() == GameState::Active {
            let mut should_spawn = false;
            self.scheduler.update(|tasks| {
                if matches!(*tasks, Tasks::SummonAsteroid) {
                    should_spawn = true;
                }
            });
            if should_spawn {
                self.spawn_random_asteroid();
            }
        }

        // Process game events
        ctx.input(|i| {
            if i.key_down(egui::Key::W) {
                self.game.interact(GameInput::Forward { dt: i.stable_dt });
            }
            if i.key_down(egui::Key::A) {
                self.game.interact(GameInput::Left { dt: i.stable_dt });
            } 
            if i.key_down(egui::Key::D) {
                self.game.interact(GameInput::Right { dt: i.stable_dt });
            }

            if i.key_down(egui::Key::Space) {
                self.game.interact(GameInput::Shoot { current_time: i.time });
            }

            self.game.update(i.stable_dt, i.time, |event| {
                match event {
                    GameEvent::Damage { health } => {
                        if health == 0 {
                            cmd = Some(ScreenCommand::Start);
                        }
                    },
                    GameEvent::AsteroidDestroyed { size } => {
                        println!("Asteroid destroyed with size: {}", size);
                    },
                    GameEvent::PlayerTarget { id } => {
                        println!("Player targeted with ID: {}", id);
                    },
                }
            });
        });
        
        cmd
    }
}
