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
    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, _order: egui::Order) -> Option<ScreenCommand> {
        let mut cmd = None;

        ui.vertical_centered(|ui| {
            ui.heading("Game");
            if ui.button("Back").clicked() {
                cmd = Some(ScreenCommand::Start);
            }

            // let painter = ui.painter();
            // let pos = egui::pos2(100.0, 100.0);
            // painter.circle_filled(pos, 5.0, egui::Color32::RED); // Draw an asteroid

            let available_rect = ui.available_rect_before_wrap();
            let width = available_rect.width();
            let height = available_rect.height();
            let size = width.min(height);
            let play_area = egui::Rect::from_center_size(
                available_rect.center(), 
                egui::vec2(size, size)
            );
            ui.painter().rect_stroke( // View box
                play_area, 
                0.0, 
                (2.0, egui::Color32::WHITE), 
                egui::StrokeKind::Inside
            );
            let mut game_ui = ui.new_child(egui::UiBuilder::new().max_rect(play_area));
            
            for asteroid in self.asteroids.iter() {
                asteroid.draw(&mut game_ui, size);
            }
            for bullet in self.bullets.iter() {
                bullet.draw(&mut game_ui, size);
            }
            self.ship.draw(&mut game_ui, size);
        });

        ctx.request_repaint();

        cmd
    }

    fn on_activate(&mut self, _ctx: &egui::Context) {
        self.asteroids.push(
            Asteroid::new(egui::pos2(50.0, 50.0), 40.0, 135.0, 5)
        );
    } 

    fn update(&mut self, ctx: &egui::Context, _frame: &eframe::Frame) {
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

            // Update ship
            self.ship.update(i.stable_dt);

            // Update bullets
            for bullet in &mut self.bullets {
                bullet.update(i.stable_dt);
            }
            self.bullets.retain(|bullet| bullet.is_alive(i.time)); // Remove bullets that have expired

            // New asteroids and update asteroids
            let mut new_asteroids: Vec<Asteroid> = vec!();
            for asteroid in &mut self.asteroids {
                asteroid.update(i.stable_dt);

                // Only retain bullets that have not hit
                self.bullets.retain(|bullet| {

                    // Check collision
                    if asteroid.check_collision(bullet.get_position()) {

                        // Add new asteroid
                        new_asteroids.push(Asteroid::hit_and_copy(asteroid));

                        // Remove bullet
                        return false; 
                    }

                    return true; // Retain bullet
                });
            }

            // Adding asteroids to self
            self.asteroids.append(&mut new_asteroids);
        });
    }
}