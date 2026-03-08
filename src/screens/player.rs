use crate::screen::ScreenCommand;
use crate::screen::Screen;

use crate::screens::game::asteroid::Asteroid;
use crate::screens::game::ship::Ship;
use crate::screens::game::bullet::Bullet;

pub struct Game {
    ship: Ship,
    asteroids: Vec<Asteroid>,
    bullets: Vec<Bullet>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            asteroids: vec!(),
            ship: Ship::new(),
            bullets: vec!(),
        }
    }
}

impl Screen for Game {
    fn ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, _order: egui::Order) -> Option<ScreenCommand> {
        let mut cmd = None;

        ui.vertical_centered(|ui| {
            ui.heading("Game");
            if ui.button("Back").clicked() {
                cmd = Some(ScreenCommand::Start);
            }

            let painter = ui.painter();
            let pos = egui::pos2(100.0, 100.0);
            painter.circle_filled(pos, 5.0, egui::Color32::RED); // Draw an asteroid
        });

        cmd
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {

    } 

    fn update(&mut self, ctx: &egui::Context, _frame: &eframe::Frame) {
        // Player movement logic
        const ZERO: f32 = 0.0;
        const MAX_SPEED: f32 = 5.0;

        ctx.input(|i| {
            // Movement
            if i.key_down(egui::Key::W) {
                self.ship.foward(i.stable_dt);
            }
            if i.key_down(egui::Key::A) {
                self.ship.turn_left(i.stable_dt);
            } 
            if i.key_down(egui::Key::D) {
                self.ship.turn_right(i.stable_dt);
            }

            // Shooting
            if i.key_down(egui::Key::Space) {
                if let Some(bullet) = self.ship.shoot(i.time) {
                    self.bullets.push(bullet);
                }
            }

            self.ship.update(i.stable_dt);

            for bullet in &mut self.bullets {
                bullet.update(i.stable_dt);
            }
            for asteroid in &mut self.asteroids {
                asteroid.update(i.stable_dt);

                for bullet in self.bullets.iter() {
                    if asteroid.check_collision(bullet.get_position()) {
                        // Remove bullet
                        
                    }
                }
            }
        });


    }
}